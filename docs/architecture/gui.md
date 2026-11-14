# GUI (`rust/src/gui/`)

egui immediate mode through eframe; binary only ([B1](boundaries.md)).

## State

`OpenAseApp` (`src/gui/mod.rs`) holds everything: `settings` (+ `settings_dirty`),
`store` (servers per game id), `nav` (tree selection), `selected`, `sort`,
`scan` (current scan), `generation`, `paging` (333networks pages), `geoip`,
`extra` (`extra::Extra`) and `dialogs` (`dialogs::Dialogs`).

`ServerKey` = game id + address: the identity of a row across scans, favorites
and friends. `Nav` = what the tree shows: favorites and their groups, recent, friends and
their groups, `Special` lists (recently added / imported, LAN), a custom
filter, a game or one of its game types. `Special::Playing` ("Playing now")
shows every loaded server with at least one human; F5 or a first visit scans
`Settings::ready_games` (program set, or Steam with a key).

## Frame (`eframe::App::ui`)

1. Apply the language (Options previews it live).
2. `drain_messages`: `try_recv` every `Msg`; scan messages of another `generation` are dropped.
3. First frame: refresh, GeoIP load/download thread, `startup_tasks`.
4. Auto-refresh of the selected server; `request_repaint_after(500 ms)` while scanning or with notices.
5. Draw menus, toolbar, tree, server list, Friends / Players / Rules panes, dialogs.

## Threads

| Work | Started by | Result |
|---|---|---|
| Game scan (masters + queries) | `scan_game`, `scan_ready_games` → `scan_games` (masters of several games in parallel, then one query pass) | `MasterList`*, `Server`*, `ScanDone` |
| Refresh favorites / visible / list | `start_scan` callers | `Server`*, `ScanDone` |
| One server | `query_single` | `Single` |
| Next 333networks page | Load more | `MorePage` |
| LAN, GeoIP, update check, Auto-Search | menus, first frame (Auto-Search after its "where to look" step, `search_places`; choice in `Settings::search_roots`); "Search all games..." walks once for every game (`detect::search_all`, `detect::owners`) | `Lan`, `GeoIp`, `Update`, `Search*`, `SearchHit` |
| Saved Unreal servers moved to their query port | startup (`extra.rs`) | `PortFixes` |
| Add server / Import list query-port check | `resolve_in_background` | `Resolved` |
| Slot watch (every 10 s, 30 min max) and party alerts (every 60 s) | `watch_tick`, each frame | `Watched`*, `WatchDone` |
| Friend sound | friend found | none (waits for the player process) |
| Saved lists: load at start, save after a completed game scan | `OpenAseApp::new`, `save_lists` | `SavedLists` |
| Executable icons of configured games (once per path) | `request_exe_icons`, each frame | `ExeIcon` |
| Map image of the selected server (once per game and map) | `request_map_image`, each frame | `MapImage` |
| RCON command | `send_rcon` (console window, right-click > RCON console) | `Rcon` |
| `openase://` links from a second instance | `instance::listen` in `OpenAseApp::new` | `Link` (also sent once for a link on the command line) |

`run_queries` runs `parallel_queries` (1..128) scoped threads over a shared
queue and checks the stop flag between servers.

## List cells

Location draws only the server's country flag (country or location as a
hover tooltip; text when there is no flag). The icon column shows the lock,
then the game icon, then the favorite star. Location draws the server's country flag from `rust/assets/flags.png` (a
40x30-per-flag atlas drawn at 20x15 with mipmaps, built by
`scripts/make_flag_atlas.sh` from flag-icons, MIT, which rewrites straight
stroked lines as rectangles for ImageMagick; codes in `rust/assets/flags.txt`)
with `icons::Flags`. Ping is colored by `settings::ping_level` and
`Theme::ping_colors`; Players has a sunken meter (`icons::meter`) in
`Theme::occupancy`. Scroll bars are solid; boxes have a 2 px gap, and GoldSrc
draws them sunken (`icons::sunken`, one-pixel `icons::edges` on whole pixels,
no frame stroke, no egui separator lines between panels). Classic keeps ASE's compact rows, left headers and striped
panes; the other themes follow JASE (`jase_look`: 24 px rows, centered
headers, no stripes). A filter bar under the tree (`filter_bar`: hide empty,
full, passworded, unresponsive; max ping; country) edits the same settings
as the Modifiers branch. The tree shows server counts and can
list installed games only (`FilterSettings::installed_only`). Bottom pane
columns are capped below the pane width, so a resize handle never leaves the
screen.

## Game icons

`icons::game_icon` draws the icon of the game's configured executable
(`exe_icon.rs`, 32 px image preferred) or, without one, JASE's badge: the
catalog `badge` letters in white on the game's color, in a rounded square.
Used in the tree and the list's icon column.

## Alerts

Decisions live in `rust/src/alerts.rs` (`has_free_slot`, `PartyTracker`,
`SlotWatch`); `check_alerts` runs on every answer in `upsert`, next to
`check_friends`. One slot watch at a time, in memory; party thresholds are
saved in `Settings::party_alerts`. Both are set from the server row's
context menu.

## Files

| File | Holds |
|---|---|
| `src/gui/mod.rs` | `OpenAseApp`, `Msg`, columns (`Col`), actions, menus, toolbar, style and palette |
| `src/gui/scan.rs` | Scans, worker threads (`run_queries`), `drain_messages` |
| `src/gui/list.rs` | `candidates`, `filtered_rows`, `sorted`, the server table, sort keys |
| `src/gui/tree.rs` | The tree and `tree_row` |
| `src/gui/panes.rs` | Friends, players and rules panes, status bar |
| `src/gui/dialogs.rs` | Options (General, Network, Events, Games, Toolbar, Colors), Find player, Add server, notices |
| `src/gui/extra.rs` | Setup wizard, Add friend, Import list, filter editor, About / license |
| `src/gui/icons.rs` | Painted toolbar and tree icons, badges, lock, sort arrows |

Look: `setup_style` sets ASE-like fonts and spacing; `apply_palette` applies
`ColorSettings` (Options > Colors). Its `Theme` (Classic, Dark, Terminal,
Amber terminal, Synthwave, GoldSrc; `settings.rs`) picks the light or dark
base, rebuilt each time by `ase_visuals` (ASE geometry; Windows-like greys on
the light base only), then the panel, border, selection-text and widget
colors, and for GoldSrc raised header bevels (`Theme::bevel`, read by
`header_cell` from egui temp data); the
four list colors on top stay editable. Bars, headers, hover, frames and
selected text come from the active visuals; `online_green`, `friend_fill`
and `readable` (Quake colors) adapt to light or dark. Do not add fixed light
colors: read them from `ui.visuals()`.
