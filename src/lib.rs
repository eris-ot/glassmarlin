//! GlassMarlin — Tauri shell that bundles CPython + the MarlinSpike package,
//! extracts them on first launch, spawns Flask on a random localhost port,
//! and points a native webview window at it.
//!
//! Public API is the `run()` function consumed by both `bin/glassmarlin.rs`
//! (production) and tests / dev harnesses.

pub mod bundle;
pub mod runtime;
pub mod secrets;

use std::sync::Arc;

use anyhow::Result;
use log::info;
use tauri::Manager;

pub use runtime::PythonRuntime;

/// Top-level entry — initialises Tauri, sets up the Python subprocess,
/// and shows the main window. Blocks until the user closes the window.
pub fn run() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .try_init()
        .ok();

    info!("GlassMarlin v{} starting", env!("CARGO_PKG_VERSION"));

    let runtime = Arc::new(std::sync::Mutex::new(None::<PythonRuntime>));
    let runtime_setup = Arc::clone(&runtime);
    let runtime_teardown = Arc::clone(&runtime);
    let runtime_signal = Arc::clone(&runtime);

    // Forward SIGTERM/SIGINT to the bundled Python subprocess so it
    // doesn't orphan when the parent is killed (deploy restart,
    // `pkill glassmarlin`, container stop). The Tauri CloseRequested
    // path handles normal user-driven close.
    #[cfg(unix)]
    std::thread::spawn(move || {
        use std::sync::atomic::{AtomicBool, Ordering};
        static FIRED: AtomicBool = AtomicBool::new(false);
        unsafe extern "C" fn handler(_: libc::c_int) {
            // Just flip the flag — actual cleanup must happen outside
            // the signal handler (Rust async/Mutex aren't async-signal-safe).
            FIRED.store(true, Ordering::SeqCst);
        }
        unsafe {
            libc::signal(libc::SIGTERM, handler as libc::sighandler_t);
            libc::signal(libc::SIGINT, handler as libc::sighandler_t);
        }
        loop {
            std::thread::sleep(std::time::Duration::from_millis(200));
            if FIRED.load(Ordering::SeqCst) {
                log::info!("signal received — shutting down MarlinSpike subprocess");
                if let Some(rt) = runtime_signal.lock().unwrap().take() {
                    rt.shutdown();
                }
                std::process::exit(0);
            }
        }
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let runtime_setup = Arc::clone(&runtime_setup);
            tauri::async_runtime::spawn(async move {
                match boot(app_handle.clone()).await {
                    Ok(rt) => {
                        let url = format!("http://127.0.0.1:{}", rt.port);
                        info!("MarlinSpike ready at {url}");
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.eval(&format!(
                                "window.location.replace('{url}')"
                            ));
                        }
                        *runtime_setup.lock().unwrap() = Some(rt);
                    }
                    Err(e) => {
                        log::error!("boot failed: {e:?}");
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.eval(&format!(
                                "document.body.innerHTML = '<pre style=\"padding:40px;color:#ef4444;font-family:monospace\">GlassMarlin failed to start:\\n\\n{}</pre>'",
                                e.to_string().replace('\'', "&#39;").replace('\n', "\\n")
                            ));
                        }
                    }
                }
            });
            Ok(())
        })
        .on_window_event(move |_window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                if let Some(rt) = runtime_teardown.lock().unwrap().take() {
                    info!("shutting down MarlinSpike subprocess");
                    rt.shutdown();
                }
            }
        })
        .run(tauri::generate_context!())
        .map_err(|e| anyhow::anyhow!("tauri error: {e}"))
}

async fn boot(app: tauri::AppHandle) -> Result<PythonRuntime> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| anyhow::anyhow!("no app data dir: {e}"))?;
    std::fs::create_dir_all(&data_dir)?;
    info!("data dir: {}", data_dir.display());

    // Extract bundle on first launch (or when version mismatches).
    let runtime_root = bundle::ensure_extracted(&data_dir).await?;
    info!("runtime root: {}", runtime_root.display());

    // Pick a random free port.
    let port = portpicker::pick_unused_port().ok_or_else(|| anyhow::anyhow!("no free port"))?;
    info!("flask port: {port}");

    // Spawn marlinspike Flask process.
    let secret_key = secrets::ensure_secret(&data_dir)?;
    let database_url = format!(
        "sqlite:///{}/marlinspike.db",
        data_dir.to_string_lossy().replace('\'', "''")
    );
    let runtime = PythonRuntime::spawn(runtime::SpawnConfig {
        runtime_root,
        data_dir: data_dir.clone(),
        port,
        secret_key,
        database_url,
    })?;

    // Wait until Flask responds to /login (the cheapest GET that
    // exercises the full create_app() path).
    runtime
        .wait_for_health(std::time::Duration::from_secs(30))
        .await?;

    Ok(runtime)
}
