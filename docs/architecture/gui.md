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
| LAN, GeoIP, update check, Auto-Search | menus, first frame | `Lan`, `GeoIp`, `Update`, `Search*` |
| Saved Unreal servers moved to their query port | startup (`extra.rs`) | `PortFixes` |
| Add server / Import list query-port check | `resolve_in_background` | `Resolved` |
| Slot watch (every 10 s, 30 min max) and party alerts (every 60 s) | `watch_tick`, each frame | `Watched`*, `WatchDone` |
| Friend sound | friend found | none (waits for the player process) |
| Saved lists: load at start, save after a completed game scan | `OpenAseApp::new`, `save_lists` | `SavedLists` |
| `openase://` links from a second instance | `instance::listen` in `OpenAseApp::new` | `Link` (also sent once for a link on the command line) |

`run_queries` runs `parallel_queries` (1..128) scoped threads over a shared
queue and checks the stop flag between servers.

## Alerts

Decisions live in `rust/src/alerts.rs` (`has_free_slot`, `PartyTracker`,
`SlotWatch`); `check_alerts` runs on every answer in `upsert`, next to
`check_friends`. One slot watch at a time, in memory; party thresholds are
saved in `Settings::party_alerts`. Both are set from the server row's
context menu.

## Files

| File | Holds |
|---|---|
| `src/gui/mod.rs` | `OpenAseApp`, `Msg`, columns (`Col`), main window, workers, style and palette |
| `src/gui/dialogs.rs` | Options (General, Network, Events, Games, Toolbar, Colors), Find player, Add server, notices |
| `src/gui/extra.rs` | Setup wizard, Add friend, Import list, filter editor, About / license |
| `src/gui/icons.rs` | Painted toolbar and tree icons, badges, lock, sort arrows |

Look: `setup_style` forces egui's light theme and ASE-like fonts;
`ColorSettings` (Options > Colors) is applied by `apply_palette`. No themes.
