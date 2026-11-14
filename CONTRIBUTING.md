# Contributing to OpenASE-Plus

Bug reports, new games and small fixes are welcome. Not sure where to start?
Open an issue and ask.

## Start here

| Need | Read |
|---|---|
| Build, run, test | [docs/development/building.md](docs/development/building.md) |
| Add a game | [docs/development/adding-a-game.md](docs/development/adding-a-game.md) |
| How the code is organized | [docs/architecture/overview.md](docs/architecture/overview.md) and [boundaries](docs/architecture/boundaries.md) |
| Everything else | [docs/README.md](docs/README.md) |

## Before you open a pull request

From `rust/`:

```sh
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
```

Tests must work offline: use captured packets or hand-written samples, never a
live server. Keep pull requests small and focused, and say what you tested.

## Legal rule

OpenASE-Plus is new code. Never commit anything from the original, proprietary
All-Seeing Eye: no `filters.txt`, `countries.txt`, executables or `.dat` files.
No copyrighted assets either (game logos, art, sounds, screenshots of game art).

Facts are fine: registry key names, executable names, default ports, public
protocol descriptions and master server addresses. If you are unsure, ask in an
issue first.

## License

GPL-3.0-or-later. By sending a contribution you agree that it is licensed under
the same terms. See [LICENSE](LICENSE).
