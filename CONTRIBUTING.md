# Contributing to GlassMarlin

GlassMarlin is the Tauri shell + Python-bundle packaging for [`eris-ot/marlinspike`](https://github.com/eris-ot/marlinspike). Most of the analyst-side logic lives in `marlinspike` itself — this repo is the desktop wrapper, the bundle pipeline, and the release plumbing.

Before opening a non-trivial PR, please discuss the approach in an issue.

## Where to send what

Roughly: if your change touches…

| | …open a PR against |
|---|---|
| Workbench routes, templates, analysis logic, plugins, rules | [`eris-ot/marlinspike`](https://github.com/eris-ot/marlinspike) |
| PCAP parsing, OT protocol dissection, packet-level analysis | [`eris-ot/marlinspike-dpi`](https://github.com/eris-ot/marlinspike-dpi) |
| Tauri shell, bundle pipeline, release workflow, install/packaging UX | This repo |

If you're not sure, open an issue here and we'll triage.

## Development setup

You need:

- **Rust** (stable, via [rustup](https://rustup.rs/))
- **uv** ([astral.sh/uv](https://docs.astral.sh/uv/))
- **Python 3.12** (only needed to build the bundle — the released binary contains its own)
- A clone of `eris-ot/marlinspike` somewhere on disk (the bundle builder needs the source tree)

Platform-specific:

| Platform | Extras |
|---|---|
| macOS | Xcode Command Line Tools (`xcode-select --install`) |
| Linux | `libwebkit2gtk-4.1-dev libgtk-3-dev librsvg2-dev libayatana-appindicator3-dev patchelf zstd` |
| Windows | [WebView2 runtime](https://developer.microsoft.com/microsoft-edge/webview2/) (usually preinstalled on Windows 11), `zstd` |

### Building the bundle

```sh
# Defaults to ../marlinspike — override with MARLINSPIKE_SRC
./scripts/build-bundle.sh
# Produces bundle/<your-os>/bundle.tar.zst
```

This downloads python-build-standalone, runs `uv pip install <marlinspike-source>` against it, copies marlinspike's `rules/`, `presets/`, `plugins/`, `migrations/`, and `data/oui.json`, then packs the result into a zstd-compressed tarball. Takes 1–3 minutes on a fast connection; ~20–60MB of artifact depending on platform.

### Running locally

```sh
cargo tauri dev
```

This rebuilds the bundle if necessary, embeds it into the binary, launches the app, and reloads on file changes.

To iterate without re-baking the bundle into the binary every time, build with `--no-default-features` and place an unpacked bundle at the platform data dir manually:

```sh
# One-time: unpack bundle to where the app expects it
mkdir -p "$HOME/Library/Application Support/com.erisforge.glassmarlin/runtime"
tar --use-compress-program="zstd -d" \
    -xf bundle/macos/bundle.tar.zst \
    -C "$HOME/Library/Application Support/com.erisforge.glassmarlin/runtime"

# Then: builds skip the embed step — faster iteration
cargo run --no-default-features
```

### Release build

```sh
cargo tauri build
# Produces target/release/bundle/{dmg,macos,msi,nsis,appimage,deb}/...
```

CI does this on every PR (without signing) and on tag push (with signing). See `.github/workflows/`.

## Architecture

See [docs/architecture.md](docs/architecture.md). The short version:

- The Rust shell **does not import Python**. It spawns `python -m marlinspike.app` as a subprocess.
- Bundle is `bundle.tar.zst`, decompressed on first launch.
- The webview navigates to `http://127.0.0.1:<port>` once Flask's `/login` returns < 500.

When in doubt about boundaries between this repo and `marlinspike`, default to **change marlinspike, not GlassMarlin**. GlassMarlin should stay a thin packaging layer.

## Coding conventions

### Rust

- `rustfmt` defaults (no override config).
- `cargo clippy --all-targets` should pass cleanly — open an issue if you want to disable a lint.
- Prefer `anyhow::Result` at the binary edges, `thiserror`-derived errors for library boundaries.
- No `unsafe` outside `lib.rs`'s signal handler thread (which is necessary for `libc::signal`). If you need unsafe elsewhere, justify it in the PR.

### Shell scripts

- `set -euo pipefail` at the top of every script.
- Use `shellcheck` to validate — `brew install shellcheck` or your distro equivalent.

### Commit messages

- Imperative mood, no period: `add Sigstore signing step`, not `Added Sigstore signing step.`
- For non-trivial changes, include a body explaining *why* the change is needed — `git log` should read like a coherent project history, not a diff narration.
- No `Co-Authored-By: Claude` or similar AI trailers (we strip them).

### PR scope

- One logical change per PR. Refactors that aren't load-bearing for the feature can go in separate PRs.
- Don't bundle unrelated formatting changes (whitespace, import sort) with substantive ones — review fatigue is real.
- If the PR touches the bundle pipeline, the release workflow, or the bundle format: tag it `infra` and ping a maintainer in the description.

## Testing

- `cargo test` runs the unit tests (small surface — mostly bundle/secrets logic).
- For end-to-end testing, build the release binary and run it against a known PCAP. We don't have automated UI testing for the webview path; manual smoke test is the standard.
- If you touch the bundle pipeline, verify a fresh extraction works:
  ```sh
  rm -rf "$HOME/Library/Application Support/com.erisforge.glassmarlin"
  open target/release/bundle/macos/GlassMarlin.app
  ```

## Release process

Releases are tag-triggered. See `.github/workflows/release.yml`.

1. Update `version` in `Cargo.toml` and `tauri.conf.json`.
2. Update [`CHANGELOG.md`](CHANGELOG.md) — move `[Unreleased]` content under the new version, add a fresh `[Unreleased]` heading.
3. Commit, get review.
4. Once merged to `main`, tag and push:
   ```sh
   git tag -s -u 2CBC9B8E1FBAF06C v0.X.Y -m "GlassMarlin v0.X.Y"
   git push origin v0.X.Y
   ```
5. CI builds all three platforms, signs everything, drafts the GitHub Release. Review the draft, edit notes if needed, click **Publish**.

For v0.1.0 we cut the release manually (no GPG passphrase in CI yet). v0.2.0 onwards is fully CI-driven once `GPG_PASSPHRASE` is in the repo secrets.

## Security disclosures

Please **do not** open public issues for security vulnerabilities. See [SECURITY.md](SECURITY.md) for the disclosure process.

## License

GlassMarlin is AGPL-3.0-or-later. By contributing you agree to license your changes under the same terms. No CLA — the AGPL itself is the contribution agreement.

## Code of conduct

We don't have a formal CoC document because we're a small org and reasonable adult behaviour covers it. The standard ones apply: be respectful, assume good faith, don't be a jerk. Maintainers reserve the right to lock or close threads that aren't productive.
