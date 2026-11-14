# Releasing

| Step | Where |
|---|---|
| Bump `version` in `rust/Cargo.toml` (and `rust/Cargo.lock`), add the `CHANGELOG.md` section | commit on `main` |
| Push tag `vX.Y.Z` | `.github/workflows/release.yml` builds and publishes |
| Rebuild an existing tag | run the Release workflow by hand (`tag`, `publish`) |
| winget / Scoop manifests (by hand, never automatic) | [packaging/README.md](../../packaging/README.md) |

Packages per release:

| OS | Files |
|---|---|
| Windows | `OpenASE-<tag>-windows-x64.zip`, `OpenASE-<tag>-windows-x64-setup.exe` (Inno Setup, `packaging/windows/openase.iss`) |
| Linux (built on Ubuntu 22.04 for an old glibc) | `OpenASE-<tag>-linux-x64.tar.gz` with a `.desktop` file, `OpenASE-<tag>-linux-x86_64.AppImage` |
| macOS | `OpenASE-<tag>-macos-universal.zip` with `OpenASE.app` (Apple Silicon + Intel) |
| All | `SHA256SUMS.txt` with the checksum of every package |

Not set up: code signing (SignPath notes in `release.yml`). The workflow,
packaging and the update check still point at upstream: see
[debt](../roadmap/debt.md) before the first OpenASE-Plus release.
