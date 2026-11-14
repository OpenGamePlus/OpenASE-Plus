# System summary

Last updated: 2026-11-14 (fork point: OpenASE 0.9.6).

## Tree

```
rust/                Cargo package `openase` (lib + bin)
├── src/lib.rs       library modules (no GUI)
├── src/main.rs      binary entry
├── src/gui/         egui interface
├── src/games.json   game catalog
├── examples/        scan.rs, query.rs (CLI, no GUI)
├── tests/           offline integration tests + docs checker (docs.rs)
├── assets/          icon.png, icon.ico
└── build.rs         Windows icon and product name resource
docs/                website (index.html, CNAME) + technical docs (*.md)
packaging/           Inno Setup, Scoop, winget generator
.github/             CI, release workflow, issue / PR templates
```

## Executables

| Binary | Source | Use |
|---|---|---|
| `openase` | `rust/src/main.rs` | The GUI |
| `scan` example | `rust/examples/scan.rs` | Masters + queries of one game, printed |
| `query` example | `rust/examples/query.rs` | One server, everything printed |

## State per subsystem

| Subsystem | State |
|---|---|
| Catalog | 80 games, 16 query protocols, 15 master kinds, 108 master entries |
| Queries | All protocols tested offline; Ping / Flux / PL from samples |
| 333networks | JSON lists, pages of 100, queries on demand ([ADR 0005](../architecture/adr/0005-333networks-light-lists.md)) |
| Steam | Web API list; needs the user's key; not checked live |
| GUI | ASE layout, favorites / friends groups, special lists, custom and color filters, wizard, toolbar editor |
| Join / detection | Templates for every game; registry + disk Auto-Search |
| GeoIP | DB-IP country database, monthly download |
| i18n | English, French |
| Update check | Points at upstream releases (debt R2) |
| Docs | This system; checker in `rust/tests/docs.rs` |

## Commands

`cargo run`, `cargo test`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`
(from `rust/`). Full table: [building.md](../development/building.md).

## Contracts

| Contract | Rule |
|---|---|
| `games.json` `id` | Stored in user settings: never renamed |
| `settings.json` | Serde with `#[serde(default)]`: new fields need defaults; old files must load |
| Library API | Used by the GUI and the examples only; no stability promise outside the repo |
