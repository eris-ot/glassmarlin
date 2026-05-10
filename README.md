# GlassMarlin

> **One file. PCAP in. Full OT/ICS triage workbench out. No setup.**

Drop GlassMarlin on a laptop, double-click, drag in a capture. The map renders, findings populate, ATT&CK techniques surface, IOCs match. The same workbench OT defenders use on the server — running locally, offline, on the engagement host.

![GlassMarlin workbench](docs/screenshots/hero.png)

## What it does

- **Maps OT networks from PCAP.** Topology with Purdue-level inference, vendor fingerprinting, asset roles. Modbus / S7 / DNP3 / IEC 60870-5-104 / EtherNet/IP / OPC UA / BACnet / PROFINET / 30+ more.
- **Surfaces risk findings.** Cross-zone communications, cleartext engineering, beaconing, suspicious external comms, port scans, missing auth, OPC SecurityMode=None — with IEC 62443 SR mapping and remediation guidance.
- **MITRE ATT&CK alignment.** Every finding mapped to techniques (ICS + Enterprise). Drop-in to ATT&CK Navigator. Tactic matrix in the workbench.
- **IOC threat hunting.** Paste a CISA advisory, ingest STIX, scan a capture's nodes + DNS + flows + payloads against IPs / domains / hashes / MACs / OUIs.
- **Per-asset baselines.** Walks every capture you've loaded and shows you what changed for this host — new peers, new protocols, new findings since last time.
- **Time-window carve-out.** Drag a span on the timeline, extract just those packets as a sub-PCAP for Wireshark.
- **Exports SIEM-ready.** Every scan produces report.json + OCSF NDJSON + STIX 2.1 bundle + Sigma rules + ATT&CK Navigator layer alongside the human-readable workbench view.

## What it's for

The defender's local tool. The thing you put on the engagement laptop. The thing you run on an air-gapped host, on a flight, in a vendor's SCIF, in a bunker. No Python. No Docker. No internet. No team server required.

When you're back on the network, the reports it produces upload cleanly into a [team MarlinSpike server](https://github.com/eris-ot/marlinspike) for collaboration, project history, audit trails, and cross-engagement intel sharing.

## When to use this vs the team server

| Use **GlassMarlin** when | Use the [team server](https://github.com/eris-ot/marlinspike) when |
|---|---|
| You have a PCAP and a laptop | You have a team and an engagement that spans weeks |
| You're on an engagement and the host has no infrastructure | You want shared projects, shared IOC lists, audit logs |
| You're on a plane / in a SCIF / in a bunker | You want cross-capture baselines and longitudinal asset profiles |
| You want zero-setup triage | You want multi-user collaboration |
| You produce ad-hoc reports | You produce engagement-grade artifacts |

Same workbench UX in both. Same report contract. Reports flow from GlassMarlin → team server when you reconnect.

## Status

**v0.1.0 — pre-release.** macOS (Apple Silicon + Intel) first. Windows + Linux in v0.2.0. Auto-updater in v0.3.0.

## Installing

When v0.1.0 ships:

```sh
# macOS — download, open, drag to Applications.
# (signed; Gatekeeper accepts on first launch)
curl -L https://github.com/eris-ot/glassmarlin/releases/latest/download/GlassMarlin.dmg -o GlassMarlin.dmg
open GlassMarlin.dmg

# Linux (AppImage) — download, chmod +x, double-click.
curl -L https://github.com/eris-ot/glassmarlin/releases/latest/download/GlassMarlin.AppImage -o GlassMarlin.AppImage
chmod +x GlassMarlin.AppImage
./GlassMarlin.AppImage

# Windows — download .msi installer.
# https://github.com/eris-ot/glassmarlin/releases/latest/download/GlassMarlin.msi
```

All releases are signed by ERISFORGE Ltd. (`8C4879D492DE808D52D2C3F02CBC9B8E1FBAF06C`) and OpenTimestamped.

## First-run

1. Open GlassMarlin. A window opens with a splash screen.
2. The first launch extracts the bundled runtime (~3 seconds; subsequent launches are ~1 second).
3. The workbench loads. You're logged in as `admin` with a freshly-generated password printed once in the on-disk credential file — change it from the profile page.
4. Drag a PCAP onto the upload page (or use the file picker).
5. Triage. See [Workbench Guide](https://github.com/eris-ot/marlinspike/blob/main/docs/workbench-guide.md) for the analyst loop.

## Where things live

On macOS:

```
~/Library/Application Support/GlassMarlin/
├── runtime/              ← extracted Python + marlinspike (managed; safe to delete to force re-extraction)
├── marlinspike.db        ← your local SQLite DB — projects, scans, IOC lists, tags
├── marlinspike-data/     ← uploaded PCAPs, generated reports, OCSF / STIX / Sigma / Navigator exports
└── secret_key            ← per-installation session-signing secret (mode 0600)
```

On Linux: `~/.local/share/GlassMarlin/`. On Windows: `%LOCALAPPDATA%\GlassMarlin\`.

## Live capture

GlassMarlin is read-only — it analyses captures you've already collected (Wireshark, tcpdump, span ports, taps). It doesn't open raw sockets and doesn't need elevated privileges.

For continuous on-host live capture, run the [team server](https://github.com/eris-ot/marlinspike) with the `marlinspike-capd` sidecar. That's a deliberately different security posture for a deliberately different deployment shape.

## License & verification

AGPL-3.0-or-later. Source at https://github.com/eris-ot/glassmarlin. Verifying a downloaded release:

```sh
gpg --recv-keys 2CBC9B8E1FBAF06C
gh release download v0.1.0
gpg --verify SHA256SUMS.asc SHA256SUMS
sha256sum -c SHA256SUMS
ots verify SHA256SUMS.ots
```

## Roadmap

- **v0.2.0** — Windows + Linux builds. Drag-and-drop PCAP onto the dock icon.
- **v0.3.0** — Auto-updater (Tauri-native, signed deltas).
- **v0.5.0** — Native file associations: double-click a `.pcap` and the analyzer opens.
- **v1.0.0** — Native Rust internals replace the bundled Python. Binary shrinks from ~120MB to ~15MB, cold start drops from ~3s to ~300ms. UX unchanged.

## See also

- [`eris-ot/marlinspike`](https://github.com/eris-ot/marlinspike) — the team server (Python web app, multi-user, project workbench)
- [`eris-ot/marlinspike-dpi`](https://github.com/eris-ot/marlinspike-dpi) — the Rust DPI engine (bundled here)
- [Workbench Guide](https://github.com/eris-ot/marlinspike/blob/main/docs/workbench-guide.md)
- [Triage Methodology](https://github.com/eris-ot/marlinspike/blob/main/docs/triage-methodology.md)
