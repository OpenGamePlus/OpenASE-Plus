# Building and testing

## Setup

| Need | How |
|---|---|
| Rust ≥ 1.88 (edition 2024) | [rustup](https://rustup.rs/); `rustup` installs `rustfmt` and `clippy` with the default profile |
| Linux GUI libraries (Debian / Ubuntu) | `sudo apt install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libgtk-3-dev` |
| Windows, macOS | nothing else |

## Commands

All from `rust/`:

| Task | Command |
|---|---|
| Run (debug / release) | `cargo run` / `cargo run --release` |
| Tests (includes the docs checker) | `cargo test` |
| Only the docs checker | `cargo test --test docs` |
| Format | `cargo fmt` (CI: `cargo fmt --check`) |
| Lint | `cargo clippy --all-targets -- -D warnings` |
| List game ids | `cargo run --example scan -- list` |
| Scan one game live | `cargo run --example scan -- <id> 20` |
| Query one server live | `cargo run --example query -- <id> host:port` |
| Command-line tool | `cargo run --bin openase-cli -- games` ([cli.md](../cli.md)) |

Steam games need `STEAM_API_KEY` in the environment for the examples.

## Quality bar before a commit

`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and
`cargo test` all clean. CI (`.github/workflows/ci.yml`, on push to `main` and
pull requests) runs fmt, clippy and tests on Windows, and tests on Linux and
macOS.

## Tests

`rust/tests/*.rs`: `protocols` (catalog-wide checks and several protocols), `parsers`,
per-family files (`quake2`, `doom3`, `tribes`, `unreal2_ase`...), `settings`, `text`, `i18n` and `docs`. No unit tests
inside `src/`. Offline only ([ADR 0003](../architecture/adr/0003-offline-tests.md)).
Binary captures too large to inline live in `rust/tests/fixtures/` (loaded with `include_bytes!`).
