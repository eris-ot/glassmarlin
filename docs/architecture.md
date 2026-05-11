# Architecture

GlassMarlin is a **Tauri 2.x Rust shell** that bundles a relocatable CPython interpreter plus the `marlinspike` Python package as an embedded blob, extracts them on first launch, and points a native webview at the local Flask process they expose.

The goal is *one file, no dependencies, no system Python, no Wireshark install* — same workbench UX you get from `eris-ot/marlinspike` running on a team server, but local-only and read-only.

## Process model

```
                            ┌──────────────────────────────────────┐
                            │ GlassMarlin (single Rust binary)     │
                            │                                      │
   user launches  ─────────▶│  glassmarlin (Tauri shell)           │
                            │   ├─ embedded bundle.tar.zst (blob)  │
                            │   ├─ webview (system WebView/WebKit) │
                            │   └─ signal handler (SIGTERM/SIGINT) │
                            │            │                         │
                            │            │ spawn (Unix domain or   │
                            │            ▼ direct fork+exec)       │
                            │  python3 -m marlinspike.app          │
                            │   ├─ Flask (random 127.0.0.1 port)   │
                            │   ├─ SQLite (marlinspike.db)         │
                            │   └─ in-process pcap parser          │
                            │                                      │
                            └──────────────────────────────────────┘
                                          ▲
                                          │ HTTP GET /login (health probe)
                                          │ then window.location.replace()
                                          ▼
                              [user sees workbench in webview]
```

Two processes total: the Rust shell (Tauri) and the bundled Python (Flask app). The webview is the same process as Tauri — it's the system's native web rendering library, not Chromium. macOS: WebKit. Linux: WebKitGTK. Windows: WebView2 (Edge engine).

The Rust shell does **not** import Python. It treats the bundled CPython as an opaque subprocess. This means:

- The Flask app at `marlinspike/app.py` is unchanged from the team-server deployment — same code, same templates, same DB schema. GlassMarlin is a **packaging story**, not a fork.
- Anything the user can do via the workbench UI also works from `python -m marlinspike <args>` against the bundled interpreter directly (for power users / scripting).

## Boot sequence

1. **Rust shell starts.** `env_logger` initialises. Tauri builder configures plugins (`shell`, `process`).
2. **Signal handler thread spawns** (Unix only). Watches for SIGTERM/SIGINT, flips an atomic flag, drives a clean shutdown of the Python subprocess. Necessary because Tauri's `CloseRequested` event only fires on user-driven window close — not on `pkill`, `kill`, container stop, deploy restart.
3. **Tauri `.setup()` callback** kicks off the async boot task.
4. **`bundle::ensure_extracted(data_dir)`** — checks `<data_dir>/runtime/.bundle-version`. If it matches `CARGO_PKG_VERSION`, skip. Otherwise, decompress the embedded `bundle.tar.zst` blob (`include_bytes!`) into `<data_dir>/runtime/`, write the stamp.
5. **`PythonRuntime::spawn`** — picks a random free port, generates per-installation `SECRET_KEY` (mode 0600), spawns:
   ```
   <runtime>/python/bin/python3 -m marlinspike.app
       PORT=<random>
       HOST=127.0.0.1
       DATABASE_URL=sqlite:///<data_dir>/marlinspike.db
       MARLINSPIKE_PROJECT_ROOT=<runtime>/marlinspike
       MARLINSPIKE_DATA_DIR=<data_dir>/marlinspike-data
       MARLINSPIKE_RESET_TOKEN_DELIVERY=disabled    # single-user
       LIVE_CAPTURE_ENABLED=false                   # no privileged sidecar
       MARLINSPIKE_DEV_INSECURE_COOKIES=true        # local HTTP, no TLS
       MARLINSPIKE_RUN_STORE=memory                 # single-worker
       PYTHONPATH=<runtime>/python/lib/python3.12/site-packages
   ```
6. **Health probe loop.** Tauri polls `GET http://127.0.0.1:<port>/login` every 250ms until status `< 500`, max 30 seconds. The `/login` route exercises the full `create_app()` path, so a successful probe means the Flask app is fully booted.
7. **Webview pivot.** On health success, `window.location.replace("http://127.0.0.1:<port>")`. The splash HTML disappears, the marlinspike workbench replaces it.
8. **Steady state.** User interacts with the workbench via the webview. All HTTP traffic is loopback. No outbound network traffic at runtime.
9. **Shutdown.** On `CloseRequested` (window close) or signal (SIGTERM/SIGINT), the Rust shell sends SIGTERM to the Python subprocess, waits up to 2 seconds for graceful exit, then SIGKILL.

## Bundle format

`bundle.tar.zst` is a zstandard-compressed tarball produced by `scripts/build-bundle.sh`. Layout:

```
python/                   Relocatable CPython 3.12.7 from python-build-standalone
├── bin/python3           (or python/python.exe on Windows)
├── lib/python3.12/
│   └── site-packages/
│       ├── marlinspike/  Installed via `uv pip install <marlinspike-source>`
│       ├── flask/
│       ├── sqlalchemy/
│       ├── alembic/
│       └── ...           every other transitive dep
marlinspike/              Marlinspike "data side" — what MARLINSPIKE_PROJECT_ROOT points at
├── rules/                YAML rule packs (MITRE, APT, ARP)
├── presets/              Asset/device taxonomy presets
├── plugins/              Python plugins (MITRE, APT, ARP)
├── migrations/           Alembic migration scripts
├── data/oui.json         OUI vendor lookup database
└── dpi/
    └── marlinspike-dpi   Compiled Rust DPI engine binary (since v0.1.1)
```

The DPI engine is **the** reason the Windows binary doesn't need libpcap or Wireshark installed. The Rust shell sets `MARLINSPIKE_DPI_BIN` to point at this binary when spawning the Python subprocess, and `marlinspike`'s engine.py dispatches Stage 2 dissection to it. If the binary isn't present in the bundle (e.g., `build-bundle.sh` was called with `MARLINSPIKE_DPI_SRC=skip`), the runtime logs the absence and the engine falls back to its built-in parser path — degraded protocol coverage, not a crash.

Compression: zstd level 19 with `--long=27`. Compresses ~120MB uncompressed to ~50MB (Linux), ~19MB (macOS, smaller because no glibc), ~30MB (Windows).

The blob is embedded into the Rust binary at compile time via `include_bytes!`:

```rust
#[cfg(feature = "embed-bundle")]
const BUNDLE_BLOB: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/bundle/",
    env!("GLASSMARLIN_TARGET_OS_STAMP"),
    "/bundle.tar.zst"
));
```

`GLASSMARLIN_TARGET_OS_STAMP` is set at build time by `scripts/build-bundle.sh` to one of `macos`, `linux`, `windows`. CI sets it explicitly per matrix entry.

The `embed-bundle` feature is on by default. With it off (`--no-default-features`), the binary expects an already-extracted bundle at `<data_dir>/runtime/` — useful for dev iteration where you don't want to rebake a 20–50MB blob into the binary every cargo build.

## Bundle lifecycle

- **Build time**: `scripts/build-bundle.sh` downloads python-build-standalone, runs `uv pip install marlinspike + deps` into the bundled interpreter's site-packages, copies marlinspike's data dirs, packs into `bundle.tar.zst`.
- **First launch**: blob is decompressed and unpacked into `<data_dir>/runtime/`. Stamp file `<data_dir>/runtime/.bundle-version` is written with the current `CARGO_PKG_VERSION`.
- **Subsequent launches**: stamp file matches — skip extraction. Boot in ~1s instead of ~3s.
- **Upgrade**: new binary has a new `CARGO_PKG_VERSION` baked in. Stamp mismatches. Old `runtime/` is deleted, new blob extracted. User's `marlinspike.db`, `marlinspike-data/`, `secret_key`, and `admin_password` are untouched.

## What lives where

Path resolution is deliberately split:

| Logical location | Physical path | Notes |
|---|---|---|
| Bundled interpreter | `<data_dir>/runtime/python/bin/python3` | Replaced on every bundle version bump |
| Bundled marlinspike package | `<data_dir>/runtime/python/lib/python3.12/site-packages/marlinspike/` | Replaced on every bundle version bump |
| Marlinspike data side (rules, plugins, migrations) | `<data_dir>/runtime/marlinspike/` | `MARLINSPIKE_PROJECT_ROOT` |
| User's projects, scans, IOC lists | `<data_dir>/marlinspike.db` | Persisted across upgrades |
| User's PCAPs, reports, exports | `<data_dir>/marlinspike-data/` | Persisted across upgrades |
| Session secret | `<data_dir>/secret_key` | Generated once, mode 0600 |
| Admin credential | `<data_dir>/admin_password` | Generated on first launch, mode 0600 |
| Launcher logs | Platform log dir (see [Troubleshooting](troubleshooting.md)) | Recreated each launch |

The split exists because everything under `runtime/` is **owned by GlassMarlin** and is safe to delete + recreate. Everything under `<data_dir>/` directly is **owned by the user** and must be preserved across upgrades.

## Security model

- **No raw socket access.** GlassMarlin is read-only — it analyses captures you've already collected (Wireshark, tcpdump, span ports, taps). For continuous live capture, run the [team marlinspike server](https://github.com/eris-ot/marlinspike) with the `marlinspike-capd` privileged sidecar — separate deployment shape, separate security model.
- **No elevated privileges.** Installs without admin rights (AppImage, .dmg drag-to-Applications) where the platform allows. The `.msi` and `.deb` paths use the platform's standard installer privilege model (admin / sudo) only for placing files in `Program Files` / `/usr/bin`.
- **127.0.0.1 only.** Flask binds to localhost. No external network listen.
- **No outbound traffic at runtime.** Once installed, GlassMarlin doesn't phone home. The bundled Python doesn't make outbound HTTPS calls except for plugins that explicitly request it (e.g., MITRE ATT&CK update via the `--update-mitre` CLI flag — not used by the GUI).
- **Per-installation `SECRET_KEY`** at `<data_dir>/secret_key` (mode 0600). Generated via `secrets.token_urlsafe(48)` on first launch.
- **Per-installation admin password** generated via `secrets.token_urlsafe(24)` on first launch, written to `admin_password` (mode 0600). User is prompted to change on first sign-in.
- **CSRF protection** enabled (constant-time `secrets.compare_digest`). The team-server's `MARLINSPIKE_DEV_INSECURE_COOKIES=true` is set because GlassMarlin runs over HTTP on `127.0.0.1` — TLS would be theatre. The trade is intentional: localhost-only deployment doesn't need cookie hardening that defends against a network attacker who doesn't exist here.
- **No telemetry.** Zero. Forever.

## Why Tauri + bundled Python instead of native Rust?

We get to ship the existing marlinspike codebase today without rewriting the workbench in Rust + a JS frontend. The trade is binary size (~120–180MB) and cold-start time (~3s on first launch).

The v1.0 roadmap replaces the bundled Python with native Rust internals. Binary shrinks ~6×, cold start drops from ~3s to ~300ms. UX unchanged because the workbench HTML is already self-contained (Jinja templates + small amount of JS, no SPA framework). That migration happens after the v0.5.x file-association / auto-update / DPI-bundle work has stabilised — not before.

## Why webview instead of a true browser?

GlassMarlin uses the system's native webview because:

- **Smaller binary.** Bundling Chromium would add ~150MB on top of the Python bundle. We're already at 21–126MB depending on platform.
- **System-managed updates.** WebView2 / WebKitGTK / WebKit get security updates via the OS, not via us.
- **Same renderer as system browsers.** macOS WebKit ≡ Safari's engine, Windows WebView2 ≡ Edge. So if the workbench renders fine in those browsers (it does — it's tested in marlinspike CI), it renders fine in GlassMarlin.

If your system's webview is broken or out of date, GlassMarlin exposes the underlying Flask app at `http://127.0.0.1:<port>` — bookmark it in Firefox or Chrome and bypass the embedded webview entirely. See [Troubleshooting → Webview is laggy or fonts look fuzzy](troubleshooting.md#webview-is-laggy-or-fonts-look-fuzzy).

## Code map

```
glassmarlin/
├── src/
│   ├── main.rs          Thin entry-point — calls glassmarlin_lib::run()
│   ├── lib.rs           Tauri builder + signal handler + boot orchestration
│   ├── runtime.rs       PythonRuntime: spawn, health-probe, graceful shutdown
│   ├── bundle.rs        ensure_extracted() — decompress embedded blob to data dir
│   └── secrets.rs       ensure_secret() — per-installation SECRET_KEY
├── scripts/
│   └── build-bundle.sh  CI + dev: produce bundle/<target_os>/bundle.tar.zst
├── bundle/
│   └── <target_os>/     Per-platform compiled bundle (gitignored)
├── assets/splash/       The HTML shown for ~3s while Python boots
├── icons/               App icons (8 sizes per platform)
├── tauri.conf.json      Bundle config: identifier, targets, CSP, codesign hooks
├── Cargo.toml           opt-level=z, lto=true, strip=true for the release profile
└── .github/workflows/
    ├── build.yml        PR + main: matrix build (macOS / Linux / Windows)
    └── release.yml      tag push: matrix build + sign + GitHub Release
```

## See also

- [Verifying releases](verifying-releases.md) — signing model, GPG / Sigstore / OTS
- [Troubleshooting](troubleshooting.md) — runtime issues
- [`eris-ot/marlinspike`](https://github.com/eris-ot/marlinspike) — the Python web app + analysis engine
- [`python-build-standalone`](https://github.com/astral-sh/python-build-standalone) — the CPython we bundle
- [Tauri 2.x](https://tauri.app/) — the desktop framework
