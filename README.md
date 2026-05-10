# GlassMarlin

> **The successor to GrassMarlin.**
>
> One file. PCAP in. Full OT/ICS triage workbench out. No Wireshark required. No Python install. No Docker. No internet. No team server. Defender-on-a-laptop tooling for OT engagements where the host has nothing.

GrassMarlin was the NSA-released OT topology mapper that field defenders quietly carried on engagement laptops for years. It worked, until it didn't — abandoned, Java-bound, single-platform, no longer maintained. GlassMarlin picks up where it left off: same defender utility, modernised, cross-platform, with the full risk + MITRE ATT&CK + IOC + baseline + sub-PCAP-carve stack on top of topology mapping.

![GlassMarlin workbench](docs/screenshots/hero.png)

## What it does

- **Maps OT networks from PCAP.** Topology with Purdue-level inference, vendor fingerprinting, asset role detection. 30+ OT protocol dissectors: Modbus, S7, DNP3, IEC 60870-5-104, EtherNet/IP, OPC UA, BACnet, PROFINET, OMRON FINS, HART-IP, EtherCAT, Sparkplug B, IEC 61850 (MMS / GOOSE / SV), and more.
- **Surfaces risk findings.** Cross-Purdue communications, cleartext engineering, beaconing, suspicious external comms, port scans, missing authentication, OPC `SecurityMode=None`, Modbus writes from unexpected sources — each with IEC 62443 SR mapping and remediation guidance.
- **MITRE ATT&CK alignment.** Every finding mapped to techniques (ICS + Enterprise). Tactic-matrix workbench view. One-click export to ATT&CK Navigator layer JSON.
- **IOC threat hunting.** Paste a CISA advisory, ingest a STIX bundle, or hand-curate a list. Scan a capture's nodes, DNS queries, flows, and payloads against IPs / domains / SHA-256 / MD5 / MACs / OUIs.
- **Per-asset baselines.** Walks every capture you've loaded and shows what changed for a given host — new peers, new protocols, new findings since last time, drift in vendor/role/device-type.
- **Time-window carve-out.** Drag a span on the capture timeline, extract just those packets as a sub-PCAP for Wireshark. The drag is local — no upload, no server.
- **SIEM-ready exports.** Every scan produces `report.json` + OCSF NDJSON + STIX 2.1 bundle + Sigma rules + ATT&CK Navigator layer alongside the workbench view. Pipe straight into Splunk, Sentinel, AWS Security Lake — or upload into a team [MarlinSpike server](https://github.com/eris-ot/marlinspike) for cross-engagement collaboration.

## No external dependencies. Period.

Every dependency ships inside the binary:

- **Python runtime + every Python dependency** — bundled via `python-build-standalone`. No `pip install`, no venv, no system Python.
- **PCAP parsing + protocol dissection** — pure Rust ([`marlinspike-dpi`](https://github.com/eris-ot/marlinspike-dpi)). No libpcap, no Npcap, no Wireshark install required on Windows. No `tshark.exe` shelling.
- **Time-window extraction** — pure Rust. No `editcap` dependency.
- **Database** — embedded SQLite. No Postgres install, no DB server to manage.
- **MITRE ATT&CK runtime** — the `marlinspike-mitre` rule packs bundle in.
- **Plugins** (MITRE / APT lateral movement / ARP poisoning detection) — compiled in.

On Windows: just `GlassMarlin.msi`. On macOS: `GlassMarlin.dmg`, signed, Gatekeeper-clean. On Linux: `GlassMarlin.AppImage`, runs on any glibc 2.28+ host. None of them require additional installs.

## What it's for

The defender's local tool. The thing you put on the engagement laptop. The thing you run on an air-gapped host, on a flight to the site, in a vendor's SCIF, in a bunker. No infrastructure. No internet.

When you're back on the network, the reports it produces upload cleanly into a [team MarlinSpike server](https://github.com/eris-ot/marlinspike) for collaboration, project history, audit trails, and cross-engagement intel sharing.

## When to use GlassMarlin vs the team server

| Use **GlassMarlin** when | Use the [team server](https://github.com/eris-ot/marlinspike) when |
|---|---|
| You have a PCAP and a laptop | Your team has an engagement that spans weeks |
| You're on-site and the host has no infrastructure | You want shared projects, shared IOC lists, audit logs |
| You're on a plane, in a SCIF, in a bunker | You want cross-capture baselines and longitudinal asset profiles |
| You want zero-setup triage | You want multi-user collaboration |
| You produce ad-hoc reports | You produce engagement-grade artifacts with audit trails |
| You're replacing GrassMarlin on the same kind of host | You're consolidating a team's triage workflow |

Same workbench UX in both. Same report contract. Reports flow from GlassMarlin → team server when you reconnect.

## What this replaces

| Tool | What GlassMarlin does better |
|---|---|
| **GrassMarlin** (NSA, 2015, abandoned) | Modern UI. Cross-platform (GrassMarlin was Java + Windows-only-friendly). Active OT protocol parser support. Risk findings + ATT&CK + IOCs alongside topology. SIEM-ready exports. Maintained. |
| **NetworkMiner** (free edition) | OT-focused. Purdue layering. IEC 62443 alignment. ATT&CK mapping. AGPL not proprietary. |
| **Wireshark + manual analysis** | Same packet-level fidelity (it uses our Rust DPI engine, which dissects 30+ OT protocols), plus the analyst-grade layer on top: pre-built finding rules, asset inventory, baseline diffs, IOC matching. |
| **Dragos / Claroty / Nozomi appliances** | Different deployment shape. Those are continuous-monitoring sensors; this is a defender's triage laptop tool. Pairs alongside, doesn't replace. |

## Status

**v0.1.0 — pre-release.** macOS (Apple Silicon + Intel) first. Windows + Linux in v0.2.0. Auto-updater in v0.3.0.

## Installing

When v0.1.0 ships (target: end of week):

```sh
# macOS — download, open, drag to Applications.
curl -L https://github.com/eris-ot/glassmarlin/releases/latest/download/GlassMarlin.dmg -o GlassMarlin.dmg
open GlassMarlin.dmg

# Linux (AppImage) — download, chmod +x, double-click.
curl -L https://github.com/eris-ot/glassmarlin/releases/latest/download/GlassMarlin.AppImage -o GlassMarlin.AppImage
chmod +x GlassMarlin.AppImage && ./GlassMarlin.AppImage

# Windows — download the .msi installer, double-click.
# https://github.com/eris-ot/glassmarlin/releases/latest/download/GlassMarlin.msi
```

All releases are signed by ERISFORGE Ltd. (`8C4879D492DE808D52D2C3F02CBC9B8E1FBAF06C`) and OpenTimestamped. Verifying:

```sh
gpg --recv-keys 2CBC9B8E1FBAF06C
gh release download v0.1.0
gpg --verify SHA256SUMS.asc SHA256SUMS
sha256sum -c SHA256SUMS
ots verify SHA256SUMS.ots
```

## First run

1. Open GlassMarlin. A window opens with a splash screen.
2. First launch extracts the bundled runtime (~3 seconds). Subsequent launches: ~1 second.
3. The workbench loads. You're logged in as `admin` with a freshly-generated password written to a local credentials file (mode 0600) — change it from the profile page.
4. Drag a PCAP onto the upload page (or use the file picker).
5. Triage. See [Workbench Guide](https://github.com/eris-ot/marlinspike/blob/main/docs/workbench-guide.md) and [Triage Methodology](https://github.com/eris-ot/marlinspike/blob/main/docs/triage-methodology.md).

## Where things live

On macOS: `~/Library/Application Support/GlassMarlin/`

```
runtime/              extracted bundled Python + marlinspike (managed; safe to delete to force re-extraction)
marlinspike.db        local SQLite — projects, scans, IOC lists, tags
marlinspike-data/     uploaded PCAPs, generated reports, OCSF / STIX / Sigma / Navigator exports
secret_key            per-installation session-signing secret (mode 0600)
```

On Linux: `~/.local/share/GlassMarlin/`. On Windows: `%LOCALAPPDATA%\GlassMarlin\`.

## Live capture

GlassMarlin is read-only — it analyses captures you've already collected (Wireshark, tcpdump, span ports, taps). It doesn't open raw sockets and doesn't need elevated privileges. This is deliberate: a desktop app running with `CAP_NET_RAW` would be a security regression.

For continuous on-host live capture, run the [team server](https://github.com/eris-ot/marlinspike) with the `marlinspike-capd` sidecar. That's a separate deployment shape with a separate security model.

## License

AGPL-3.0-or-later (matching MarlinSpike). Source at https://github.com/eris-ot/glassmarlin.

The AGPL network-distribution clause means that if you serve GlassMarlin's web UI to remote users (you shouldn't — it binds to localhost by default), you're obligated to offer source. For the desktop use case the clause doesn't activate.

## Roadmap

- **v0.1.0** — macOS-only signed `.dmg`. Bundles the current MarlinSpike Python codebase + Rust DPI engine. No system deps.
- **v0.2.0** — Windows `.msi` + Linux `.AppImage`. Pure-Rust replacement for the residual `tshark`/`editcap` shells so the Windows binary is clean (no libpcap, no Npcap, no Wireshark install).
- **v0.3.0** — Tauri auto-updater (signed deltas).
- **v0.5.0** — Native file associations: double-click a `.pcap` and the analyzer opens.
- **v1.0.0** — Native Rust internals replace the bundled Python. Binary shrinks from ~120MB to ~15MB, cold start drops from ~3s to ~300ms. UX unchanged.

## See also

- [`eris-ot/marlinspike`](https://github.com/eris-ot/marlinspike) — the team server (Python web app, multi-user, project workbench)
- [`eris-ot/marlinspike-dpi`](https://github.com/eris-ot/marlinspike-dpi) — the Rust DPI engine (bundled here)
- [Workbench Guide](https://github.com/eris-ot/marlinspike/blob/main/docs/workbench-guide.md)
- [Triage Methodology](https://github.com/eris-ot/marlinspike/blob/main/docs/triage-methodology.md)
