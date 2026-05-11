# Security policy

## Reporting a vulnerability

**Do not open a public GitHub issue for security vulnerabilities.** Send them to:

```
security@erisforge.com
```

PGP-encrypted reports preferred — use the ERISFORGE Ltd. key:

```
8C4879D492DE808D52D2C3F02CBC9B8E1FBAF06C
```

Available from:

```sh
gpg --keyserver keys.openpgp.org --recv-keys 2CBC9B8E1FBAF06C
```

We'll acknowledge within **72 hours** and aim to ship a fix within **14 days** for high/critical severity issues, longer for lower severity.

## What we'd like to know

- The version you tested (`GlassMarlin → About` or `glassmarlin --version`).
- The platform and OS version.
- Steps to reproduce — ideally a self-contained PCAP or workflow we can replay.
- The impact (auth bypass? RCE? local privilege escalation? info leak? DoS?).
- Whether you've disclosed to anyone else.

We'll credit you in the release notes unless you ask us not to. We don't currently run a bug-bounty program, but we'll publicly thank serious finders and prioritise their reports.

## Supported versions

The latest **minor** release line is supported with security fixes. Older minor lines are not.

| Version | Supported |
|---|---|
| 0.1.x | ✅ |
| < 0.1 | n/a (no public releases) |

When v0.2.0 ships, 0.1.x moves to unsupported.

## Scope

In scope:

- The Rust shell (`src/`).
- The bundle pipeline (`scripts/build-bundle.sh`).
- The bundled Python runtime as configured by us — specifically, vulnerabilities introduced by GlassMarlin's bundling choices (env vars, file modes, signal handling, path resolution).
- The release process (artifact signing, CI workflow).

Out of scope here (please report to the upstream project instead):

- Vulnerabilities in the marlinspike Python app itself — file at [`eris-ot/marlinspike`](https://github.com/eris-ot/marlinspike) (same `security@erisforge.com` address — we triage across both).
- Vulnerabilities in upstream CPython, Flask, SQLAlchemy, etc. — report to those projects first; if GlassMarlin needs to ship a fix faster than upstream, we'll do that and credit your upstream report.
- Vulnerabilities in `python-build-standalone` — report to [astral-sh/python-build-standalone](https://github.com/astral-sh/python-build-standalone).
- Vulnerabilities in Tauri or the system webview — report to [Tauri](https://github.com/tauri-apps/tauri) or your OS vendor.
- Social engineering, phishing, or threats requiring physical access to an unlocked machine where the user has already authenticated.

## What we consider valid

Roughly, anything that lets an attacker:

- Escape the GlassMarlin app boundary to affect other applications or the OS (the bundled Python is *supposed* to be sandboxed by the user-data dir).
- Read or modify data outside the GlassMarlin data dir.
- Cause the bundled Python to make network connections we don't intend (the only intended traffic is loopback HTTP from the webview).
- Bypass authentication or authorisation in the bundled marlinspike workbench.
- Execute code in a privilege boundary we didn't intend (e.g., RCE via a crafted PCAP).
- Cause GlassMarlin to leak the `SECRET_KEY`, the admin password file, or session cookies to anyone other than the local user.
- Trigger predictable secret material (weak `SECRET_KEY`, predictable admin password generation).

## What we don't consider valid

- Reports requiring an attacker who already has filesystem access to the user-data dir (mode `0600` files notwithstanding — if they're already in your homedir as your UID, the game is over). The threat model assumes the local user is trusted.
- Reports requiring an attacker on the local loopback interface but not on the user's host. GlassMarlin binds to `127.0.0.1` only; the threat model assumes no other process running as the same user is hostile.
- "Gatekeeper / SmartScreen shows a warning" — yes, intentionally, see [docs/verifying-releases.md](docs/verifying-releases.md). Not a vulnerability.
- Self-XSS in the workbench requiring you to paste attacker-crafted JS into your own browser console.
- Outdated transitive dependencies that have no exploitable path through GlassMarlin's use of them. If there's a real exploit path, we want to know.

## Disclosure timeline

We follow a **90-day coordinated disclosure** policy:

- **Day 0** — you report.
- **Day 0–3** — we acknowledge.
- **Day 0–14** — we triage and respond with our timeline.
- **Day 14–90** — we develop and ship a fix.
- **Day 90** — public disclosure, ideally simultaneous with a patched release.

We're happy to extend the window if you ask and have a reason. We won't reduce it below 7 days unless the vulnerability is being actively exploited in the wild.

## Past advisories

We'll publish security advisories via GitHub's [security advisories feature](https://github.com/eris-ot/glassmarlin/security/advisories). At the time of v0.1.0 there are none.
