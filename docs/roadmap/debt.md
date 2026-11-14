# Debt, risks and known gaps

Product plan: `ROADMAP.md`. This file: what is known to be missing, risky or
deferred in the code. Remove an item in the commit that fixes it.

## Risks

| # | Risk | Where | Note |
|---|---|---|---|
| R2 | The update check reports upstream releases | `rust/src/update.rs` (`RELEASES_API`, `RELEASES_PAGE`) | Point at the fork before its first release |

## Fork identity still upstream's

Fix together, before the first OpenASE-Plus release ([ADR 0006](../architecture/adr/0006-independent-fork.md)):

| What | Where |
|---|---|
| Site, manual, message board links | `rust/src/gui/extra.rs` (`SITE`, `MANUAL`, `MESSAGE_BOARD`), `rust/src/gui/dialogs.rs` |
| 333networks User-Agent `OpenASE/<v> (+https://openase.org)` | `rust/src/networks333.rs` |
| `authors`, `repository` | `rust/Cargo.toml` |
| Installer `AppId` / `AppName` (same as upstream: installs over it), URLs | `packaging/windows/openase.iss` |
| winget / Scoop ids and URLs | `packaging/make-manifests.mjs`, `packaging/scoop/openase.json` |
| Config folder `OpenASE` shared with an upstream install | `settings::config_dir` |
| `docs/CNAME` = `openase.org` (GitHub Pages of the fork would claim it), site `docs/index.html` | `docs/` |
| Fork repo is `OpenGamePlus/OpenASE-Plus` (`origin`); update URLs above to it | local git config |
| README, website texts, issue templates | `README.md`, `docs/index.html`, `.github/ISSUE_TEMPLATE/add_game.yml` |

## Accepted debt

| Item | Why accepted |
|---|---|
| `src/gui/mod.rs` is ~2900 lines | One app struct; split only along a real seam (list, tree, panes) when touched |
| `tribes.rs` uses `settings::parse_address_list` | Small shared parser; move to `model` or `text` if another protocol needs it |
| Protocol modules depend on `quake3` helpers | See [boundaries](../architecture/boundaries.md) exceptions |
| No unit tests in `src/`; GUI untested | Logic lives in the library and is tested in `rust/tests/` |

## Known gaps

- Steam list never checked with a real API key (unit-tested only).
- Join commands of many newer games not tested with the game installed (upstream issue #1).
- No code signing; IPv6 addresses not supported (`parse_address`).
- `openase://` links do nothing on macOS (delivered as Apple Events, which eframe does not expose; the scheme is not registered there) and need the installer on Windows (not the portable zip). Untested by hand on Windows.
- Only English and French.
- Docs checker does not check bare file names without `/` (`quake3.rs`, `ROADMAP.md`), skip fenced code for paths, check memory date order, or cover `ROADMAP.md` / `CHANGELOG.md`.
