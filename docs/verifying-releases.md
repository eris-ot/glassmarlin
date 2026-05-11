# Verifying releases

GlassMarlin is signed with industry-standard crypto — GPG for now, Sigstore + GitHub artifact attestations starting in v0.2.0.

We do **not** use paid OS-vendor code-signing certs (Apple Developer ID, Windows Authenticode). The trade-off:

|  | OS-vendor certs | What we do instead |
|---|---|---|
| Proves the publisher is who they say they are | ✅ via CA chain | ✅ via GPG web of trust + (v0.2.0+) Sigstore OIDC |
| Proves the binary was built from a specific commit | ❌ | ✅ via (v0.2.0+) GitHub artifact attestations |
| Tamper-evident at a specific point in time | ❌ | ✅ via OpenTimestamps |
| Waives Gatekeeper / SmartScreen warnings on first launch | ✅ | ❌ |

For an open-source defender tool, we'd rather give you stronger cryptographic provenance and eat the OS-vendor warning friction than pay rent to CA gatekeepers. If that ever changes (e.g., a sponsor covers the certs), we'll add them without removing the GPG/Sigstore path.

## Quick verify (any platform)

You need `gpg` and `ots` (the [OpenTimestamps client](https://github.com/opentimestamps/opentimestamps-client)).

```sh
# 1. Get the ERISFORGE Ltd. signing key.
gpg --recv-keys 2CBC9B8E1FBAF06C
# Fingerprint should be: 8C4879D492DE808D52D2C3F02CBC9B8E1FBAF06C

# 2. Pull all the verification artifacts for the release you downloaded.
gh release download v0.1.0 --repo eris-ot/glassmarlin

# 3. Verify the SHA256SUMS file was signed by ERISFORGE Ltd.
gpg --verify SHA256SUMS.asc SHA256SUMS
# Expect: "Good signature from ERISFORGE Ltd. (a Rwanda Corp)"

# 4. Verify your downloaded binary's hash matches the signed manifest.
sha256sum -c SHA256SUMS
# Expect: "<file>: OK" for every file you have on disk.

# 5. Verify the SHA256SUMS manifest existed at release time.
ots verify SHA256SUMS.ots
```

If all four checks pass, the binary is authentic.

## Verifying the git tag

The tag itself is GPG-signed by the same key:

```sh
git clone https://github.com/eris-ot/glassmarlin
cd glassmarlin
git tag -v v0.1.0
# Expect: "Good signature from ERISFORGE Ltd."
```

## Key trust

The ERISFORGE Ltd. signing key fingerprint is:

```
8C4879D492DE808D52D2C3F02CBC9B8E1FBAF06C
```

Fetch it from any keyserver:

```sh
gpg --keyserver keys.openpgp.org --recv-keys 2CBC9B8E1FBAF06C
gpg --keyserver keyserver.ubuntu.com --recv-keys 2CBC9B8E1FBAF06C
gpg --keyserver pgp.mit.edu --recv-keys 2CBC9B8E1FBAF06C
```

The key is also published in this repo's `.well-known/` (coming in v0.2.0) and pinned to the [`eris-ot` GitHub org profile](https://github.com/eris-ot).

If you've never trusted our key before, verify the fingerprint via a separate channel (signed PGP message, in-person verification at a conference, etc.) before relying on it. We're a small org — happy to verify out-of-band on request.

## What's signed and how

| Artifact | How it's signed |
|---|---|
| `SHA256SUMS` (the hash manifest) | GPG-signed in `SHA256SUMS.asc` (detached, ASCII-armored) |
| Each binary (`*.dmg`, `*.msi`, etc.) | Hash is in `SHA256SUMS`, which is GPG-signed — so the binary is transitively signed |
| The git tag | GPG-signed by the same key (`git tag -v`) |
| Release time | OpenTimestamps proof (`SHA256SUMS.ots`) — Bitcoin-anchored, irreversible |

The OpenTimestamps proof can take ~6 hours to fully anchor to a Bitcoin block. The `.ots` file is usable immediately but `ots verify` may report *"Pending confirmation in Bitcoin blockchain"* during that window — that's normal. Run `ots upgrade SHA256SUMS.ots` after a few hours to fold in the Bitcoin attestation, then re-verify.

## Coming in v0.2.0

The CI release workflow (`.github/workflows/release.yml`) is wired to add two more signature layers on the next tag push:

### Sigstore (keyless OIDC)

Each release artifact will ship with `.sig` + `.crt` files. Sigstore proves the binary was built by *this specific GitHub Actions workflow on this specific commit* — stronger provenance than CA-chain Authenticode.

```sh
cosign verify-blob \
  --certificate GlassMarlin_X.Y.Z_aarch64.dmg.crt \
  --signature GlassMarlin_X.Y.Z_aarch64.dmg.sig \
  --certificate-identity-regexp 'https://github.com/eris-ot/glassmarlin/.*' \
  --certificate-oidc-issuer 'https://token.actions.githubusercontent.com' \
  GlassMarlin_X.Y.Z_aarch64.dmg
```

### GitHub artifact attestations

```sh
gh attestation verify GlassMarlin_X.Y.Z_aarch64.dmg --repo eris-ot/glassmarlin
```

This gives you a single command verification that the artifact came from a specific GitHub Actions run on a specific commit, signed at build time by GitHub's OIDC identity provider.

## Reporting a bad signature

If `gpg --verify` returns *"BAD signature"* on an official release — **stop**. Do not run the binary. Open an issue or email `security@erisforge.com` immediately. We'd rather false-alarm than ship a compromised artifact.

Things that are *not* bad signature reports:

- `gpg: Can't check signature: No public key` — you haven't fetched the key yet. Run `gpg --recv-keys 2CBC9B8E1FBAF06C`.
- `WARNING: This key is not certified with a trusted signature!` — the key is unknown to your local web of trust. Verify the fingerprint via a separate channel (above) then run `gpg --lsign-key 2CBC9B8E1FBAF06C` to locally sign it.
- `ots verify: Pending confirmation in Bitcoin blockchain` — normal for new releases. Wait ~6h and run `ots upgrade`.

## See also

- [Installing](install.md) — platform-specific install + first-launch friction
- [Troubleshooting](troubleshooting.md) — runtime issues after install
- [`opentimestamps-client`](https://github.com/opentimestamps/opentimestamps-client) — the `ots` CLI
- [Sigstore](https://www.sigstore.dev/) — keyless signing primer
