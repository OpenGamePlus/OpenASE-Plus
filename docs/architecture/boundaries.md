# Boundaries

The architectural constitution. A change that needs to break a rule writes an
ADR first ([adr/](adr/README.md)).

## Rules

| # | Rule |
|---|---|
| B1 | The library (`rust/src/*.rs`) has no egui or eframe: only `src/main.rs` and `src/gui/` use them. The examples prove it. |
| B2 | No network, DNS, disk search or process wait on the UI thread. Work runs in `std::thread` workers that report through the `Msg` channel and call `request_repaint`. Add server and Import list resolve Unreal query ports in a worker (`Msg::Resolved`). |
| B3 | Late results are dropped: every scan message carries the `generation` it was started with, and a stop flag ends its workers. |
| B4 | A game that fits a supported protocol is data: one entry in `rust/src/games.json`, no Rust code. |
| B5 | New protocol = a module, a `Protocol` and/or `Master` variant in `catalog.rs`, and a branch in `query.rs`. Nothing else dispatches on protocol, except `lan.rs` for broadcast and the GUI checking `Protocol::Quake3` for color codes. |
| B6 | Protocol modules return `model::Server` / `Listed`; they do not know the GUI, settings or the catalog's display data. |
| B7 | Text from servers and masters is untrusted: shown as plain text, decoded with `text.rs`, never passed to a shell. Same for `openase://` links: only `links::parse` reads them, and they never join without the user's yes. RCON answers are shown as plain text; a command is one line (`rcon::clean_command`). RCON passwords live in memory only. |
| B8 | Values never shape a command line: `launch::join_args` splits the template with markers, then fills values into single arguments; `Game::launch_mod` keeps only folder-like mod names. No shell: URLs open with `launch::url_command` (`rundll32 url.dll,FileProtocolHandler` on Windows). |
| B9 | Every user-visible GUI string goes through `tr` / `fill` / `noop` and has a French entry in `i18n.rs` (`rust/tests/i18n.rs` enforces it). |
| B10 | Tests are offline: captured packets or hand-written samples, no live server ([ADR 0003](adr/0003-offline-tests.md)). |
| B11 | Nothing from the proprietary All-Seeing Eye and no copyrighted game assets in the repository ([ADR 0004](adr/0004-no-proprietary-ase-files.md)). |

## Dependency matrix

Row may use column (`x`), `—` same layer allowed only as listed under exceptions.

| from \ to | leaves (incl. `location`) | protocols | catalog / query | services | gui |
|---|---|---|---|---|---|
| leaves | x | | | | |
| protocols | x | `quake3` helpers only | | | |
| catalog / query | x | x | x | | |
| services | x | x | x | x | |
| gui | x | x | x | x | x |

## Accepted exceptions

| Where | Why |
|---|---|
| Protocol modules and `ase_filters` use `quake3` helpers (`parse_address`, `latin1`, `ParseError`, `MASTER_IDLE_TIMEOUT`) | Shared address and packet helpers; moving them is churn without gain |
| `networks333.rs` uses `unreal2::strip_colors` | Same Unreal color codes in the JSON names |
| `quake2.rs` (`Game`, `Protocol`), `doom3.rs` (`Game`) use `catalog` | They read game data for their own family |
| `tribes.rs` uses `settings::parse_address_list` | Same `ip:port` list parser; tracked in [debt](../roadmap/debt.md) |
| `catalog.rs` names `query` only in a doc comment | No code dependency |

## Forbidden

- egui types in the library; GUI state in protocol modules.
- `std::process::Command` with a shell (`sh -c`, `cmd /C`).
- Blocking `recv` on the UI thread.
- Tests that need the Internet.
