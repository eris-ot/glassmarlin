# Installing GlassMarlin

Download from [the latest release](https://github.com/eris-ot/glassmarlin/releases/latest).

## macOS

**Apple Silicon (M-series).** Intel builds land in v0.1.2.

```sh
curl -L https://github.com/eris-ot/glassmarlin/releases/latest/download/GlassMarlin_0.1.2_aarch64.dmg \
  -o GlassMarlin.dmg
open GlassMarlin.dmg
# In the Finder window: drag GlassMarlin.app to Applications.
```

**First launch** — macOS Gatekeeper will refuse the app with *"Apple cannot verify…"*. This is expected: GlassMarlin is not signed with an Apple Developer ID cert (see [Verifying releases](verifying-releases.md) for why). Two ways past it:

```sh
# Easy: clear the quarantine attribute Gatekeeper sets on downloaded files.
xattr -d com.apple.quarantine /Applications/GlassMarlin.app
# Then double-click as normal.
```

Or right-click `GlassMarlin.app` → **Open** → confirm in the dialog. Once approved, every subsequent launch is normal.

## Linux

Two artifacts. Choose by distro:

### AppImage (any glibc 2.28+ host)

```sh
curl -L https://github.com/eris-ot/glassmarlin/releases/latest/download/GlassMarlin_0.1.2_amd64.AppImage \
  -o GlassMarlin.AppImage
chmod +x GlassMarlin.AppImage
./GlassMarlin.AppImage
```

Self-contained — won't write outside `~/.local/share/com.erisforge.glassmarlin/`.

### Debian / Ubuntu (.deb)

```sh
curl -L https://github.com/eris-ot/glassmarlin/releases/latest/download/GlassMarlin_0.1.2_amd64.deb \
  -o glassmarlin.deb
sudo apt install ./glassmarlin.deb
glassmarlin   # or launch from the application menu
```

## Windows

Two artifacts — `.msi` (recommended, installs to `Program Files`) or `.exe` (NSIS, more configurable).

```powershell
# PowerShell, run as admin
Invoke-WebRequest `
  -Uri "https://github.com/eris-ot/glassmarlin/releases/latest/download/GlassMarlin_0.1.2_x64_en-US.msi" `
  -OutFile "GlassMarlin.msi"
Start-Process msiexec.exe -ArgumentList "/i GlassMarlin.msi" -Wait
```

**First launch** — Windows SmartScreen will block with *"Windows protected your PC"*. GlassMarlin is not Authenticode-signed (see [Verifying releases](verifying-releases.md) for why). To proceed:

1. Click **More info** in the SmartScreen dialog.
2. Click **Run anyway**.

After the first launch, SmartScreen remembers your decision and won't ask again on this machine.

## Verifying signatures

Every release ships with a GPG-signed `SHA256SUMS.asc` and an OpenTimestamps proof `SHA256SUMS.ots`. See [Verifying releases](verifying-releases.md) for the full recipe.

## First-run experience

1. Launch GlassMarlin. A splash window opens for ~3 seconds (first launch only — subsequent launches: ~1 second) while the bundled runtime extracts.
2. The workbench loads at `http://127.0.0.1:<port>` (random port, local-only). Bookmark in your browser of choice if you'd rather use Firefox over the embedded webview.
3. You're auto-logged in as `admin` with a freshly-generated password written to a credentials file (mode `0600`) — change it from the profile page on first sign-in.
4. Drop a PCAP onto the upload page (or use the file picker). Triage from there.

For walking through the workbench itself, see the shared [Workbench Guide](https://github.com/eris-ot/marlinspike/blob/main/docs/workbench-guide.md) and [Triage Methodology](https://github.com/eris-ot/marlinspike/blob/main/docs/triage-methodology.md).

## Where GlassMarlin stores data

| Platform | Data dir |
|---|---|
| macOS | `~/Library/Application Support/com.erisforge.glassmarlin/` |
| Linux | `~/.local/share/com.erisforge.glassmarlin/` |
| Windows | `%LOCALAPPDATA%\com.erisforge.glassmarlin\` |

Inside:

```
runtime/              Extracted bundled Python + marlinspike. Safe to delete to force re-extraction.
marlinspike.db        SQLite — projects, scans, IOC lists, tags, baselines.
marlinspike-data/     Uploaded PCAPs, generated reports, OCSF / STIX / Sigma / Navigator exports.
secret_key            Per-installation session-signing secret (mode 0600).
admin_password        Generated first-run admin credential (mode 0600).
```

GlassMarlin **never** writes outside this directory. There is no registry write, no `/etc` modification, no system-wide service install.

## Uninstalling

### macOS

```sh
rm -rf "$HOME/Library/Application Support/com.erisforge.glassmarlin"
rm -rf /Applications/GlassMarlin.app
```

### Linux (AppImage)

```sh
rm GlassMarlin.AppImage
rm -rf "$HOME/.local/share/com.erisforge.glassmarlin"
```

### Linux (.deb)

```sh
sudo apt remove glassmarlin
rm -rf "$HOME/.local/share/com.erisforge.glassmarlin"
```

### Windows

Settings → Apps → GlassMarlin → Uninstall. Then:

```powershell
Remove-Item -Recurse "$env:LOCALAPPDATA\com.erisforge.glassmarlin"
```

## Common install issues

| Symptom | Fix |
|---|---|
| macOS: "App is damaged" instead of "cannot verify" | `xattr -dr com.apple.quarantine /Applications/GlassMarlin.app` (note the `-r`). |
| Linux AppImage: `error while loading shared libraries: libfuse.so.2` | Install `libfuse2`: `sudo apt install libfuse2` (Debian/Ubuntu) or your distro's equivalent. |
| Linux: webview window is blank | Install `libwebkit2gtk-4.1`: `sudo apt install libwebkit2gtk-4.1-0`. |
| Windows: SmartScreen won't show "Run anyway" | Some EDR products strip the button. Right-click the `.msi` → **Properties** → **Unblock**, then re-run. |
| App boots but spinner never resolves | See [Troubleshooting](troubleshooting.md). |

See also: [Troubleshooting](troubleshooting.md).
