# Debt, risks and known gaps

Product plan: `ROADMAP.md`. This file: what is known to be missing, risky or
deferred in the code. Remove an item in the commit that fixes it.

## Risks

| # | Risk | Where | Note |
|---|---|---|---|

## Fork identity

Done for 1.0.0 (OpenASE-Plus name, own installer `AppId`, config folder with a
one-time import of OpenASE's, URLs and User-Agents at OpenGamePlus/OpenASE-Plus).
Left: `docs/index.html` is still upstream's website (no Pages for the fork);
the winget package and a Scoop bucket do not exist yet.

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
- Translations other than French were made in one pass and not reviewed by native speakers; long strings may not fit some buttons and columns.
- Docs checker does not check bare file names without `/` (`quake3.rs`, `ROADMAP.md`), skip fenced code for paths, check memory date order, or cover `ROADMAP.md` / `CHANGELOG.md`.
