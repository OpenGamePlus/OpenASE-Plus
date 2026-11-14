# 0002. The game catalog is data

- Status: Accepted
- Date: 2026-11-14 (recorded; decided upstream before the fork)

## Context
ASE described games in a `filters.txt` file. Most new games reuse a protocol
that already exists and only differ in names, ports, masters and join syntax.

## Decision
Every game is one entry in `rust/src/games.json`, embedded at build time
(`include_str!`) and loaded by `catalog.rs`. Code is written only for a new
protocol or master kind.

## Consequences
- Adding a game is usually a JSON edit plus a test update ([adding-a-game](../../development/adding-a-game.md)).
- Game `id`s are stored in user settings: never rename one.
- The catalog cannot be changed without a rebuild (no remote catalog).
