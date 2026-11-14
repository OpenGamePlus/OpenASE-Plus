# 0004. Nothing from the proprietary ASE

- Status: Accepted
- Date: 2026-11-14 (recorded; decided upstream before the fork)

## Context
OpenASE recreates the experience of The All-Seeing Eye, a closed Yahoo
program, and its users may still have its files.

## Decision
The repository holds new code only: no `filters.txt`, `countries.txt`,
executables or `.dat` files from ASE (also blocked in `.gitignore`), and no
copyrighted game assets (logos, art, sounds). Facts are fine: registry keys,
executable names, ports, public protocol descriptions, master addresses.

## Consequences
- `ase_filters.rs` can read the ASE format a user has, but no such file ships.
- Icons are painted in code (`src/gui/icons.rs`); game badges are text and color.
