# Project memory

Append-only, newest first. Never rewrite a stage; correct it in a new one.
Format and rules: [DEVELOPMENT_GUIDE](DEVELOPMENT_GUIDE.md).

## 2026-11-14 — Auto-Search asks where to look
- User request: Auto-Search started on C: and crawled every drive. It now opens on a checklist of `detect::search_roots()` plus added folders (rfd), remembered in `Settings::search_roots`; registry lookup and disk walk start with "Search".

## 2026-11-14 — five more languages
- i18n moved from a French `match` to one TSV table per language (`rust/src/locales/`, embedded, parsed once); fr migrated by script, es/pt/it/de/ru translated by five parallel agents with shared rules; Spanish uses "cupo" (as in JASE). Tests now require every source text in every table, same `{}` and shortcuts.
- System language detection covers the six languages (Windows LANGID, LC_*/LANG). egui's Ubuntu font has Cyrillic.
- Open: no native-speaker review; some strings may be too long for their widgets.

## 2026-11-14 — new app icon
- The user chose a neon cyan Eye of Providence (eye, inscribed triangle, circle) on JASE's dark background, from a stock reference redrawn from scratch (generic symbol, no copying). Drawn with ImageMagick primitives in `scripts/make_icon.sh` (its SVG renderer is unreliable); 16 and 24 px use a bolder variant without the circle.

## 2026-11-14 — game icons
- `exe_icon.rs`: bounds-checked PE resource reader (RT_GROUP_ICON -> best RT_ICON -> one-image .ico, decoded by `image` with the `ico` feature); tested on a synthetic PE and checked on real CoD2, OpenCoD2-Plus, notepad and explorer executables.
- Not-installed games: JASE badges, new unique two-letter `badge` field for all 80 games (JASE's letters where ids match). The old rectangular `short` badge is gone.

## 2026-11-14 — fork identity for 1.0.0
- User's choices: name OpenASE-Plus, version 1.0.0, own config folder importing OpenASE's settings.json once, links to the GitHub repo. New installer `AppId` (BE92ED7C-...; also the winget ProductCode), packages `OpenASE-Plus-*`, `OpenASE-Plus.exe`, `openase-plus`, `io.github.opengameplus.openase-plus`.
- Kept Thomas Sautier's copyright next to the fork's (GPL), the Cargo binary names (`openase`, `openase-cli`) and the `openase://` scheme. Removed the CNAME file of docs/ and upstream's 0.9.1 winget manifests.

## 2026-11-14 — JASE look for non-Classic themes, filter bar
- Rule agreed with the user: Classic keeps ASE's look; other themes follow JASE (24 px rows, centered headers, no stripes). Filter bar under the tree reuses `FilterSettings`.
- Thick GoldSrc frames came from the frame stroke doubling the bevel and from half-pixel lines; bevels are now one-pixel rects on whole pixels (`icons::edges`), no stroke when a bevel exists, softer light edge on boxes.

## 2026-11-14 — visual polish after a second JASE comparison
- Flags: 2x atlas with mipmaps, pixel-aligned; ImageMagick's MSVG drops straight stroked lines (US stripes), so the script rewrites them as rects. Installing resvg was not allowed; the preprocessor avoids it.
- Sunken occupancy meter and sunken GoldSrc boxes, tree server counts, "Installed games only" (`FilterSettings::installed_only`, uses `ready_games`), bottom pane columns capped at pane width - 40.

## 2026-11-14 — flags, ping colors, occupancy bars
- From the user's comparison with JASE: country flags (flag-icons SVG rasterized once by ImageMagick into a 74 KB atlas, `scripts/make_flag_atlas.sh`), ping colors and occupancy bars per theme (JASE tokens), solid scroll bars, 2 px gaps around boxes.
- Rejected shipping the 2.4 MB of SVGs plus an SVG renderer (resvg); flags-LICENSE.txt ships in the zip, tar.gz, macOS zip and installer (MIT notice).

## 2026-11-14 — theme colors fixed after a Windows review
- The user's Windows build showed dark themes with light menu, toolbar, headers and status bar (unreadable): about 15 fixed light colors predated themes. They now come from `ui.visuals()`; Quake colors go through `readable` (dark `^0`/`^4` lightened on dark themes).
- Built and tested natively on Windows (MSVC, 195 tests) in the user's E: checkout.

## 2026-11-14 — map images, phase 7 closed
- `maps.rs`: `levelshots/<map>` from the `.pk3` files named by the new `levelshots` catalog field (values from JASE), mod folder first, last pk3 in load order wins; decoded with `image` (TGA tried when no magic bytes match). ADR 0007: local files only, no GameTracker.
- New dependencies `zip` (no default features, deflate via flate2) and `image` (jpeg, tga, png). The system tray was postponed with the user's agreement.
- Open: CoD IWI and Unreal readers; not checked with a real install.

## 2026-11-14 — GoldSrc theme
- Added `Theme::GoldSrc` (JASE's olive/gold tokens, olive widgets via `Theme::widgets`).
- Fixed a theme leak: `setup_style` used to paint ASE's light greys on both egui bases, so dark themes had light buttons; `ase_visuals(dark)` now rebuilds the active base on every `apply_palette`.

## 2026-11-14 — themes
- `Theme` in `ColorSettings` (five presets from JASE): base light/dark visuals, panel and selection-text colors, plus the four editable colors; `apply_palette` touches only the active base style (`style_mut_of`).
- Rejected JASE's eleven token-based themes: egui has no stylesheet, five presets cover light, dark and the terminal looks.
- Open: not checked by eye (no screenshot under WSLg); fixed light highlights in dark themes are debt.

## 2026-11-14 — RCON
- `rcon.rs` ported from JASE (Quake UDP, GoldSrc UDP with challenge, Source TCP with an empty marker packet after the command), tested against fake servers on localhost; non-modal console window, answers through `Msg::Rcon`.
- Dialect from the protocol; A2S games with a Steam app id below 200 are GoldSrc. Passwords only in memory (`Dialogs::rcon_passwords`), never in settings.
- Not checked against a live server with a real password.

## 2026-11-14 — saved lists and openase-cli
- `lists.rs` saves each completed game scan (`lists/<id>.json`, atomic, off the UI thread) and loads them at start as `listed` + `saved` rows (no ping); `saved` keeps them out of alerts and Playing now. `model::Server` is now serde: one form for the lists and the CLI JSON.
- `openase-cli` (`src/bin/`, `default-run = "openase"` keeps `cargo run` on the GUI): games / scan / query, schema 1, exit codes 0/1/2; tested against a fake Zandronum server. 333networks rows are not queried.
- Not shipped in the macOS app or the Windows installer (zip and tar.gz only).

## 2026-11-14 — openase:// links
- `links.rs` (strict parse: catalog id, IPv4/host name, port), `instance.rs` (localhost TCP, port in `instance.port`, one line `OPENASE1 <url>` answered `OK`), confirm dialog before joining; scheme registered by Inno (HKA) and the Linux `.desktop` files. Checked end to end with two real processes.
- Rejected a token on the instance port: any local process can already start OpenASE with a link, and links still need the user's yes.
- Open: macOS (Apple Events not exposed by eframe), portable zip on Windows, no manual Windows test.

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
