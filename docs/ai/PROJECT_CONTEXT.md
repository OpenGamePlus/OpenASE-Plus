# Project context

## Identity
OpenASE-Plus (1.0.0+): an independent fork of OpenASE 0.9.6 by Thomas Sautier, the open-source game server
browser in the spirit of The All-Seeing Eye. Part of the maintainer's "-Plus"
family of forks. It takes ideas from JASE, the maintainer's earlier Python
browser.

## Goal
One native browser for the classic online games that still have players, with
ASE's look and feel: filter tree, server list with Ping / Flux / PL, Friends /
Players / Rules panes, and joining with a double click.

## Non-goals
- Shipping or reusing anything from the proprietary ASE, or game assets.
- Running game servers, anti-cheat, accounts or a server of our own.
- Modern-game coverage: the focus is classic games with community masters.

## Principles
- A game is data; code only for a new protocol.
- Gentle on community masters (333networks, OpenSpy, dpmaster): no needless bulk queries.
- UI never blocks; untrusted remote text stays plain text.
- Offline tests; small, focused changes.

## Stack and constraints
Rust ≥ 1.88 (edition 2024), egui / eframe 0.37, `ureq`, `serde_json`,
`regex`, `flate2`, `rfd`; `winreg` on Windows. Targets Windows, Linux, macOS.
License GPL-3.0-or-later.
