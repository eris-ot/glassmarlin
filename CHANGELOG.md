# Changelog

All notable changes to GlassMarlin are documented in this file. Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

Track work in flight against the [v0.2.0 milestone](https://github.com/eris-ot/glassmarlin/milestones).

## [0.1.2] — 2026-05-11

Platform-matrix expansion. Five binaries now ship per release instead of three.

### Added

- **macOS Intel (x86_64)** build — `GlassMarlin_0.1.2_x86_64.dmg`. Runs natively on Intel Macs without Rosetta. Built on the `macos-13` CI runner.
- **Linux ARM64 (aarch64)** build — `GlassMarlin_0.1.2_aarch64.AppImage` and `_aarch64.deb`. For Raspberry Pi 4/5, Apple Silicon under Asahi/UTM, AWS Graviton, and other ARM servers. Built on the `ubuntu-22.04-arm` CI runner.

### Changed

- CI matrix is now 5 platforms (was 3). All five build in parallel and feed the same sign-and-release job.
- Artifact names include explicit arch (`_x86_64`, `_aarch64`, `_amd64`, `_x64`) so cross-platform downloads can't pick the wrong file.

### Bundled software

Same as v0.1.1 — CPython 3.12.7, marlinspike 3.5.x, marlinspike-dpi (the Rust DPI engine), Flask, SQLAlchemy, Alembic. Each per-arch binary picks up the matching `python-build-standalone` interpreter at bundle time.

## [0.1.1] — 2026-05-11

The "actually clean Windows binary" release. Closes the gap between what v0.1.0's release notes claimed ("no libpcap, no Npcap, no Wireshark install required") and what v0.1.0 actually delivered (silently degraded to empty reports when Wireshark wasn't installed).

### Added

- **Bundled `marlinspike-dpi`** — pure-Rust DPI engine ships inside the binary on all three platforms. 50+ OT/ICS + IT protocol dissectors, 21 anomaly signatures, no C dependencies. The runtime sets `MARLINSPIKE_DPI_BIN` automatically so `marlinspike`'s engine.py uses the Rust path instead of falling back to a missing `tshark`. ([`eris-ot/marlinspike-dpi`](https://github.com/eris-ot/marlinspike-dpi))
- **Pure-Python PCAP / PCAPNG validator** in `marlinspike` (`marlinspike/_pcap_header.py`) — replaces the previous `capinfos` / `tshark` shell-out for Stage 1 validation. Stdlib only. Verified parity with `capinfos` on both classic PCAP and PCAPNG inputs.
- `marlinspike-dpi` itself is now [dual-licensed](https://github.com/eris-ot/marlinspike-dpi/blob/main/LICENSE-COMMERCIAL.md): AGPL-3.0-or-later for OSS use, commercial licence available for proprietary embeds.

### Changed

- README and v0.1.0 release-notes claim "no libpcap, no Npcap, no Wireshark install required" is now actually true on the Windows binary. v0.1.0 was technically delivering a degraded path with empty reports on no-Wireshark hosts; this release fixes that.

### Known limitations carried forward from v0.1.0

- macOS Apple Silicon only. Intel build still pending.
- Time-window extract (the "carve sub-PCAP" workbench feature) still shells to `tshark` + `editcap` on the back end. The workbench will surface a 500 error if Wireshark CLI tools aren't on `$PATH`. A pure-Python replacement lands in v0.2.0 alongside the Sigstore signing rollout.

## [0.1.0] — 2026-05-10

First public release.

### Added

- **Single-file desktop app** wrapping the [`eris-ot/marlinspike`](https://github.com/eris-ot/marlinspike) workbench. Drop on a laptop, open a PCAP, get the full OT/ICS analyst workbench — topology with Purdue layering, risk findings, MITRE ATT&CK alignment, IOC threat hunting, per-asset baselines, time-window carve-out.
- **Three platforms**: macOS Apple Silicon (`.dmg`), Linux x86_64 (`.AppImage` + `.deb`), Windows x86_64 (`.msi` + `.exe`).
- **Zero external dependencies.** Every binary bundles CPython 3.12.7 (via [`python-build-standalone`](https://github.com/astral-sh/python-build-standalone)), the `marlinspike` package, Flask, SQLAlchemy, Alembic, all plugins (MITRE/APT/ARP), all rule packs, and the OUI database. No system Python, no `pip install`, no Wireshark, no `tshark`, no libpcap, no Npcap.
- **Per-installation security**: random `SECRET_KEY` and admin password generated on first launch (mode `0600`), CSRF protection, localhost-only Flask bind.
- **Graceful subprocess lifecycle**: SIGTERM/SIGINT handler ensures the bundled Python is cleanly shut down on `pkill`, container stop, or system shutdown — no orphaned processes.
- **Build provenance** via:
  - GPG-signed `SHA256SUMS.asc` (ERISFORGE Ltd. key `8C4879D492DE808D52D2C3F02CBC9B8E1FBAF06C`).
  - OpenTimestamps proof `SHA256SUMS.ots` (Bitcoin-anchored).
  - GPG-signed git tag (`git tag -v v0.1.0`).
- **CI matrix** building all three platforms on every PR / main push.
- **Release CI workflow** ready for v0.2.0 — Sigstore + GitHub attestations + auto-publish on tag push.

### Known limitations

- **macOS**: Apple Silicon only. Intel build lands in v0.1.1.
- **Windows**: x86_64 only. ARM64 build not yet shipped.
- **Code signing**: No paid Apple Developer ID / Authenticode Windows cert. First launch shows Gatekeeper / SmartScreen warnings — see [Installing](docs/install.md) for the bypass. Sigstore + GitHub attestations land in v0.2.0 via CI.
- **DPI engine**: GlassMarlin v0.1.x uses the in-process Python parser. The Rust DPI engine ([`marlinspike-dpi`](https://github.com/eris-ot/marlinspike-dpi)) lands bundled in v0.2.0 — ~10–30× faster on large PCAPs.
- **Live capture**: Not bundled. By design — desktop apps with `CAP_NET_RAW` are a security regression. For continuous on-host live capture, run the [team marlinspike server](https://github.com/eris-ot/marlinspike) with the `marlinspike-capd` sidecar.
- **Auto-update**: Not yet. Update by downloading the latest release. Lands in v0.3.0 (Tauri signed-delta updater).

### Bundled software

| Component | Version |
|---|---|
| CPython | 3.12.7 (python-build-standalone 20241016 build) |
| marlinspike | 3.5.0 |
| Flask | 3.1.8 |
| SQLAlchemy | latest at build time |
| Alembic | latest at build time |
| Tauri | 2.x |

The full bundled dependency set is reproduced from `marlinspike`'s `pyproject.toml` plus the explicit list in `scripts/build-bundle.sh`.

[Unreleased]: https://github.com/eris-ot/glassmarlin/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/eris-ot/glassmarlin/releases/tag/v0.1.0
