# Project memory

Append-only, newest first. Never rewrite a stage; correct it in a new one.
Format and rules: [DEVELOPMENT_GUIDE](DEVELOPMENT_GUIDE.md).

## 2026-11-14 — Playing now
- New `Special::Playing` tree entry: loaded servers with humans > 0; F5 / first visit scans `Settings::ready_games` with `scan_games` (masters in parallel, one query pass). `scan_game` now goes through it; `scan.total` sums per-game counts.
- Rejected a separate dialog as in JASE: a tree entry reuses the list, sorting, filters and panes.

## 2026-11-14 — slot and party alerts
- Ported JASE's two alerts: decisions in `alerts.rs` (tested), GUI polls watched servers with `watch_tick` and checks every answer in `upsert`. Party alerts fire on crossing up and re-arm at half the threshold; the first answer only sets the baseline.
- Rejected a confirm-to-join dialog (JASE asks): a notice plus selecting the row fits OpenASE's notices; join is one double click.
- Open: no tray, so alerts are only seen while the window is visible (roadmap item 7).

## 2026-11-14 — Zandronum
- New protocol module `zandronum.rs` (Huffman tree from Zandronum, MIT notice kept in the module), ported from JASE with its captures in `rust/tests/fixtures/`; live: 786 servers listed.
- A server ignores a second query for ~10 s: `query_server` takes one sample for this protocol, and an "ignoring" answer waits 10.5 s and asks once more.
- `Game::launch_mod` now rejects names starting with `-` or `.` (the IWAD comes from the server).

## 2026-11-14 — nine games from JASE
- Added Xonotic, Warsow, Warfork, Tremulous, Smokin' Guns (live-checked) and TF2, L4D, L4D2, GMod (Steam, not live-checked: no key); the Quake 3 master parser now accepts Tremulous' trailing `\`.
- dpmaster lists many dead entries first: judge a game with `scan -- <id> 40`, not 6. Dropped dpmaster.tchr.no (no answer). Xonotic joins without `fs_game` (DarkPlaces).
- Postponed D-Day: Normandy (one server, needs a `gamedir` filter on the Quake 2 list).

## 2026-11-14 — security and UI-thread fixes
- Join arguments can no longer be added by a name, password or host (template split with markers, values filled in after); Steam URLs open with `rundll32 url.dll,FileProtocolHandler` instead of `cmd /C start`.
- Add server and Import list resolve Unreal query ports in a worker (`Msg::Resolved`), reusing the `PortFixes` pattern; the dialog closes at once, the server appears when resolved.
- Open: `rundll32` path only exercised by Windows CI, not by hand; a Steam password with spaces is untested.

## 2026-11-14 — documentation system
- Added `AGENTS.md`, `docs/` architecture / development / debt / glossary / AI layer, six ADRs recording existing decisions, and the docs checker as a Rust test so it runs in CI on every OS.
- `CONTRIBUTING.md` shrunk to a welcome page; its layout, protocol and adding-a-game content moved to `docs/` (fork is independent, ADR 0006). Rejected: a Python checker like JASE's (adds a toolchain to CI).
- Fresh-context review found real code issues, now debt R1 (shell in Steam join), R3 (UDP on the UI thread) and R4 (unquoted join values); open with the fork identity list.

## 2026-11-14 — fork
- OpenASE-Plus forked from OpenASE 0.9.6; Rust toolchain set up with rustup, build and tests green.
- Plan: bring ideas from JASE (more games, slot / filling notifications, "playing now", RCON, themes).
