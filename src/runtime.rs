//! Spawns and supervises the bundled CPython interpreter running
//! `python -m marlinspike.app`. Handles health-check polling and
//! graceful shutdown on window close.

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use log::{debug, warn};

pub struct SpawnConfig {
    pub runtime_root: PathBuf,
    pub data_dir: PathBuf,
    pub port: u16,
    pub secret_key: String,
    pub database_url: String,
}

pub struct PythonRuntime {
    child: Option<Child>,
    pub port: u16,
}

impl PythonRuntime {
    pub fn spawn(cfg: SpawnConfig) -> Result<Self> {
        let python = python_executable(&cfg.runtime_root)?;
        let marlinspike_root = cfg.runtime_root.join("marlinspike");

        debug!("python: {}", python.display());
        debug!("marlinspike root: {}", marlinspike_root.display());

        let mut cmd = Command::new(&python);
        cmd.current_dir(&marlinspike_root)
            .arg("-m")
            .arg("marlinspike.app")
            .env("PORT", cfg.port.to_string())
            .env("HOST", "127.0.0.1")
            .env("DATABASE_URL", &cfg.database_url)
            .env("SECRET_KEY", &cfg.secret_key)
            .env("MARLINSPIKE_DATA_DIR", cfg.data_dir.join("marlinspike-data"))
            .env(
                "MARLINSPIKE_PROJECT_ROOT",
                &marlinspike_root,
            )
            // GlassMarlin is single-user; the v3.5.4 password reset is
            // disabled by default. The user changes their own password
            // via the existing /api/account/password route.
            .env("MARLINSPIKE_RESET_TOKEN_DELIVERY", "disabled")
            // Single-user, no live capture by default (capd is a separate
            // privileged daemon — not bundled into a desktop app).
            .env("LIVE_CAPTURE_ENABLED", "false")
            // Local-only HTTP — no TLS in the desktop shell.
            .env("MARLINSPIKE_DEV_INSECURE_COOKIES", "true")
            // Cookie-based session uses the SQLite db; no need for db
            // mode of the run store at this scale.
            .env("MARLINSPIKE_RUN_STORE", "memory")
            // Make Python find the bundled site-packages.
            .env(
                "PYTHONPATH",
                bundled_site_packages(&cfg.runtime_root)
                    .to_string_lossy()
                    .to_string(),
            )
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let child = cmd
            .spawn()
            .with_context(|| format!("failed to spawn python: {}", python.display()))?;

        Ok(Self {
            child: Some(child),
            port: cfg.port,
        })
    }

    pub async fn wait_for_health(&self, timeout: Duration) -> Result<()> {
        let url = format!("http://127.0.0.1:{}/login", self.port);
        let deadline = std::time::Instant::now() + timeout;
        let mut attempt = 0;
        loop {
            attempt += 1;
            match ureq::get(&url)
                .timeout(Duration::from_secs(2))
                .call()
            {
                Ok(resp) if resp.status() < 500 => return Ok(()),
                Ok(resp) => {
                    debug!("health probe {attempt}: status {}", resp.status());
                }
                Err(e) => {
                    debug!("health probe {attempt}: {e}");
                }
            }
            if std::time::Instant::now() >= deadline {
                return Err(anyhow!(
                    "marlinspike did not become healthy within {timeout:?} ({attempt} probes)"
                ));
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
    }

    pub fn shutdown(mut self) {
        if let Some(mut child) = self.child.take() {
            // Try a graceful SIGTERM first (Unix). Windows: just kill.
            #[cfg(unix)]
            {
                use std::os::unix::process::ExitStatusExt;
                let _ = signal_term(&child);
                // Wait briefly for graceful exit.
                for _ in 0..20 {
                    if let Ok(Some(status)) = child.try_wait() {
                        debug!("python exited gracefully: {:?}", status.code().or_else(|| status.signal()));
                        return;
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
            if let Err(e) = child.kill() {
                warn!("failed to kill python: {e}");
            }
            let _ = child.wait();
        }
    }
}

impl Drop for PythonRuntime {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

#[cfg(unix)]
fn signal_term(child: &Child) -> std::io::Result<()> {
    use std::os::unix::io::AsRawFd as _;
    // Send SIGTERM to the child's PID.
    let pid = child.id() as i32;
    let rc = unsafe { libc::kill(pid, libc::SIGTERM) };
    if rc != 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

fn python_executable(runtime_root: &PathBuf) -> Result<PathBuf> {
    #[cfg(target_os = "windows")]
    let path = runtime_root.join("python").join("python.exe");
    #[cfg(not(target_os = "windows"))]
    let path = runtime_root.join("python").join("bin").join("python3");
    if !path.exists() {
        return Err(anyhow!(
            "bundled python not found at {}",
            path.display()
        ));
    }
    Ok(path)
}

fn bundled_site_packages(runtime_root: &PathBuf) -> PathBuf {
    #[cfg(target_os = "windows")]
    return runtime_root.join("python").join("Lib").join("site-packages");
    #[cfg(not(target_os = "windows"))]
    return runtime_root
        .join("python")
        .join("lib")
        .join("python3.12")
        .join("site-packages");
}
