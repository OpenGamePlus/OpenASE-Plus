# Architecture overview

One Cargo package in `rust/`: a library crate (`openase`, `src/lib.rs`) with
everything except the interface, and a binary (`src/main.rs`) that adds the
egui GUI (`src/gui/`). The command-line tools in `rust/examples/` use the
library only.

## Layers

```
main.rs ──► gui/  (binary only: egui, threads, channels)
               │
               ▼
   services:  settings · rules · launch · detect · lan · geoip · update · i18n
               │
               ▼
   dispatch:  query  ◄── catalog (games.json)
               │
               ▼
   protocols: quake3 quake2 a2s gamespy gamespy2 gamespy_sb unreal2 ase tribes
              doom3 savage ravenshield ghostrecon networks333 steam
               │
               ▼
   leaves:    model · text · colors · countries · ase_filters · location
```

Rules for these layers: [boundaries.md](boundaries.md).

## Modules

| Module | Layer | Role |
|---|---|---|
| `rust/src/main.rs` | binary | Window options, starts `gui::OpenAseApp` |
| `rust/src/gui/mod.rs` | binary | App state, main window (tree, list, panes), worker threads, `Msg` channel |
| `rust/src/gui/dialogs.rs` | binary | Options dialog (tabs), Find player, Add server |
| `rust/src/gui/extra.rs` | binary | First-run wizard, friends, import list, custom filter editor, About |
| `rust/src/gui/icons.rs` | binary | Toolbar and tree icons painted by hand, game badges |
| `rust/src/catalog.rs` | dispatch | Loads `games.json` (`include_str!`), `Game`, `Protocol`, `Master` |
| `rust/src/games.json` | data | The game catalog: 79 games, 15 protocols, 14 master kinds |
| `rust/src/query.rs` | dispatch | `fetch_all_masters`, `query_server`, Ping/Flux/PL from samples |
| protocol modules | protocols | One per query or master protocol, see [protocols.md](protocols.md) |
| `rust/src/model.rs` | leaf | `Server`, `Player`, `Listed`: protocol-independent results |
| `rust/src/text.rs` | leaf | Decoding server text (UTF-8, Windows-1251, Latin-1) |
| `rust/src/colors.rs` | leaf | Quake color codes, hex colors |
| `rust/src/countries.rs` | leaf | ISO code to country name |
| `rust/src/ase_filters.rs` | leaf | Parser of the original ASE `filters.txt` format (never shipped) |
| `rust/src/settings.rs` | service | `Settings` as JSON in the config folder; favorites, friends, groups |
| `rust/src/rules.rs` | service | Custom filters ("My filters") and color filters |
| `rust/src/launch.rs` | service | `join` template expansion, starting the game, `open_url` |
| `rust/src/detect.rs` | service | Install folder detection: registry, then disk search |
| `rust/src/lan.rs` | service | LAN discovery by UDP broadcast, every protocol that supports it |
| `rust/src/geoip.rs` | service | DB-IP country database: monthly download, lookup |
| `rust/src/location.rs` | leaf | Country / region of a server for the Location column |
| `rust/src/update.rs` | service | Update check against GitHub Releases |
| `rust/src/i18n.rs` | service | English and French texts: `tr`, `fill`, `noop` |

## Data flow of a scan

1. GUI starts a scan: new `generation`, stop flag, worker thread.
2. Worker: `query::fetch_all_masters` asks every master of the game in parallel and merges the list.
3. Worker sends `Msg::MasterList`; then `run_queries` queries the servers with
   `parallel_queries` scoped threads, each result sent as `Msg::Server`.
4. GUI drains the channel each frame (`try_recv`) and drops messages of an older `generation`.

333networks lists come with server details: those rows are shown as listed and
queried only on demand ([ADR 0005](adr/0005-333networks-light-lists.md)).

## Persistence and outside resources

| What | Where |
|---|---|
| Settings, favorites, friends | `settings.json` in the config folder (`settings::config_dir`: `%APPDATA%\OpenASE`, `~/Library/Application Support/OpenASE`, `$XDG_CONFIG_HOME/OpenASE`, else `~/.config/OpenASE`) |
| Window, panel and column sizes | eframe persistence |
| GeoIP database | config folder, downloaded monthly from DB-IP |
| Network | UDP/TCP to masters and servers, HTTP(S) (`ureq`) to Steam, 333networks, list APIs, DB-IP, GitHub |
| Processes | the game (join), the system URL opener, sound players |
