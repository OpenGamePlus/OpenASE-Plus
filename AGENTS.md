# AGENTS.md — OpenASE-Plus

Game server browser for classic online games, in the spirit of The All-Seeing
Eye (Rust + egui). Independent fork of OpenASE. Users: `README.md`.
Technical docs: `docs/README.md`.

## Read before changing code
1. `docs/ai/README_AI.md` (reading order of the AI layer).
2. `docs/architecture/boundaries.md` (layers, threads, untrusted data).
3. `docs/roadmap/debt.md` (known risks and gaps).

## Hard rules
- From `rust/`: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` clean before every commit.
- No egui outside `rust/src/main.rs` and `rust/src/gui/`.
- No network, DNS, disk search or process wait on the UI thread; scan results carry their `generation`.
- A game on a supported protocol is a `rust/src/games.json` entry; never rename a game `id`.
- Server and master text is untrusted: plain text only, validated before it reaches a command line, never through a shell.
- Every visible string through `tr` / `fill` / `noop` with a French entry in `rust/src/i18n.rs`.
- Tests offline (captured packets). Never commit ASE files, game assets or `docs/superpowers/` (local notes).
- Be gentle with community masters: no bulk queries of 333networks-listed servers.

## Docs sync
A change that makes a doc wrong fixes it in the same commit; each finished
piece of work adds a stage to `docs/ai/PROJECT_MEMORY.md`. "Change X → update Y"
table: `docs/ai/DEVELOPMENT_GUIDE.md`. The checker (`rust/tests/docs.rs`) runs
with `cargo test`.

## Commands
`docs/development/building.md`. Add a game: `docs/development/adding-a-game.md`.
Release: `docs/development/releasing.md`.
