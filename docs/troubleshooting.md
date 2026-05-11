# Troubleshooting

If something's not working, this page covers the issues we've seen most often.

## Launch issues

### Splash window appears but workbench never loads

The bundled Python subprocess hasn't responded to a `GET /login` health probe within 30 seconds. Common causes:

1. **Port conflict.** GlassMarlin binds a random localhost port — but if the picked port is firewalled or in use by a fast-rebind race, the probe fails. Restart the app; it picks a new port on each launch.
2. **Disk full.** The bundle extraction needs ~150MB of free space in your data dir. Free space and restart.
3. **Anti-virus / EDR is sandboxing the extracted Python.** Check your AV's quarantine. Allow-list the data dir (paths in [Installing → Where GlassMarlin stores data](install.md#where-glassmarlin-stores-data)).

To see what went wrong, check the launcher log:

| Platform | Log path |
|---|---|
| macOS | `~/Library/Logs/com.erisforge.glassmarlin/launcher.log` |
| Linux | `~/.local/share/com.erisforge.glassmarlin/logs/launcher.log` |
| Windows | `%LOCALAPPDATA%\com.erisforge.glassmarlin\logs\launcher.log` |

The log includes the chosen port, the bundle extraction status, and the Python subprocess stdout/stderr.

### "bundled python not found at …"

The Rust shell can't find the extracted Python interpreter. This means:

1. **Bundle extraction failed silently.** Delete the runtime dir to force re-extraction:

   ```sh
   # macOS
   rm -rf "$HOME/Library/Application Support/com.erisforge.glassmarlin/runtime"
   # Linux
   rm -rf "$HOME/.local/share/com.erisforge.glassmarlin/runtime"
   # Windows (PowerShell)
   Remove-Item -Recurse "$env:LOCALAPPDATA\com.erisforge.glassmarlin\runtime"
   ```

   Then relaunch.

2. **You built from source without the `embed-bundle` feature.** Either rebuild with `cargo build --release` (default features include `embed-bundle`) or manually place an extracted bundle at `<data-dir>/runtime/` — see [Architecture](architecture.md#bundle-format).

### macOS: "GlassMarlin.app is damaged and can't be opened"

This is Gatekeeper's stricter variant when the quarantine attribute looks corrupted (often after using `curl` or `wget` to download to a non-standard location). Strip it recursively:

```sh
xattr -dr com.apple.quarantine /Applications/GlassMarlin.app
```

(Note the `-r` — without it, only the top-level attribute is cleared and the message persists.)

### Windows: "This app can't run on your PC"

You downloaded the wrong architecture. GlassMarlin v0.1.0 is x86_64 only — ARM64 Windows builds are not yet shipped. Check `wmic os get osarchitecture`. If it's `64-bit` you're good. If it's `ARM64` we don't have a build for you yet; track [open issues for ARM64 Windows](https://github.com/eris-ot/glassmarlin/issues?q=is%3Aissue+arm64).

## Authentication issues

### Lost admin password

The freshly-generated admin password is written on first launch to a credentials file (mode `0600`):

| Platform | Path |
|---|---|
| macOS | `~/Library/Application Support/com.erisforge.glassmarlin/admin_password` |
| Linux | `~/.local/share/com.erisforge.glassmarlin/admin_password` |
| Windows | `%LOCALAPPDATA%\com.erisforge.glassmarlin\admin_password` |

If you've deleted that file and forgotten the password:

1. Delete the SQLite database to force a fresh admin account (this loses all projects, scans, IOC lists, baselines):
   ```sh
   rm "$HOME/Library/Application Support/com.erisforge.glassmarlin/marlinspike.db"
   ```
2. Relaunch. A new admin password will be generated and written to `admin_password`.

If you want to keep your data, you can reset the password without nuking the database — see the [marlinspike auth docs](https://github.com/eris-ot/marlinspike/blob/main/docs/admin-and-audit.md#password-reset).

### "CSRF token mismatch" on login

This happens if you've migrated session cookies between browsers, or if you're hitting the app from a different host than the one that issued the cookie. Clear cookies for `127.0.0.1:<port>` and reload. GlassMarlin is local-only — cross-origin auth is intentionally not supported.

## Performance issues

### First launch takes more than 10 seconds

Likely AV scanning the extracted Python tree. On macOS, also check `Activity Monitor → Energy → mds_stores` for Spotlight indexing. Either:

- Wait it out — second launch is ~1 second.
- Add the data dir to your AV's exclusion list (paths in [Installing → Where GlassMarlin stores data](install.md#where-glassmarlin-stores-data)).
- macOS: add the data dir to Spotlight's Privacy exclusions (System Settings → Siri & Spotlight → Spotlight Privacy).

### Workbench feels slow on large PCAPs

GlassMarlin v0.1.x uses the in-process Python parser. For captures > 1GB, expect minute-scale scan times. v0.2.0 bundles the Rust DPI engine ([`marlinspike-dpi`](https://github.com/eris-ot/marlinspike-dpi)) which is ~10–30× faster — same workbench UX.

In the meantime, narrow the scan window in the time-scrubbing UI before triggering a chain run (see [time-scrubbing-and-extract](https://github.com/eris-ot/marlinspike/blob/main/docs/time-scrubbing-and-extract.md)).

### Webview is laggy or fonts look fuzzy

The embedded webview uses the system's WebKit/WebView2:

- **macOS**: Safari's renderer. Update macOS to the latest minor version.
- **Linux**: WebKitGTK. Update `libwebkit2gtk-4.1-0`.
- **Windows**: WebView2 (Edge engine). Update Edge via Windows Update.

You can also bypass the embedded webview entirely — note the port from the splash log and open `http://127.0.0.1:<port>` in Firefox or Chrome.

## Process / lifecycle issues

### Python subprocess orphaned after `pkill glassmarlin`

The Rust shell installs a SIGTERM/SIGINT handler that gracefully shuts down the bundled Python. If you `kill -9` the parent, the child is reparented to init and continues running. Find and kill it:

```sh
pgrep -af "marlinspike.app"
# kill <pid>
```

This shouldn't happen in normal use — close the window or `pkill glassmarlin` (regular SIGTERM, not `-9`).

### Multiple GlassMarlin instances

Currently supported but not recommended — each instance picks a separate port and uses the same SQLite database. SQLite's write-ahead log handles concurrent reads fine, but concurrent writes (two scans in flight) will serialise and may report database-busy errors. A proper singleton lock is on the v0.2.0 roadmap.

## Data issues

### "no such table: …" SQLAlchemy errors

The database schema is out of date. GlassMarlin embeds Alembic migrations and runs them on boot — if you see this, the migration failed or was interrupted. Force a re-migration:

```sh
# Back up the DB first
cp "$HOME/Library/Application Support/com.erisforge.glassmarlin/marlinspike.db" \
   "$HOME/Library/Application Support/com.erisforge.glassmarlin/marlinspike.db.bak"

# Delete the alembic version row to force re-migration on next launch
sqlite3 "$HOME/Library/Application Support/com.erisforge.glassmarlin/marlinspike.db" \
  "DELETE FROM alembic_version;"
```

Then relaunch — migrations will rerun. If the issue persists, file an issue with the launcher log attached.

### Workbench shows old data after upgrading

The bundle version stamp at `runtime/.bundle-version` triggers re-extraction when GlassMarlin starts and the stamp doesn't match `CARGO_PKG_VERSION`. If you see truly stale behaviour after upgrade:

```sh
rm -rf "$HOME/Library/Application Support/com.erisforge.glassmarlin/runtime"
```

This is the same as a "factory reset of the bundled runtime" — it does **not** touch your `marlinspike.db`, `marlinspike-data/`, or session cookies.

## Getting more help

If none of the above fixes your issue:

1. Reproduce with logging cranked up: launch from a terminal with `RUST_LOG=debug ./GlassMarlin.app/Contents/MacOS/glassmarlin` (or platform equivalent).
2. Collect the launcher log + the bundled Python's stderr (printed to the launcher's stdout).
3. [Open an issue](https://github.com/eris-ot/glassmarlin/issues/new) with the log, the OS + arch, and what you tried.

Security issues: please email `security@erisforge.com` instead — see [SECURITY.md](../SECURITY.md).
