# 0001. Rust + egui, one native binary

- Status: Accepted
- Date: 2026-11-07

## Context
OpenASE started with a Python (Tkinter) prototype next to a Rust (egui)
version. Keeping two implementations doubled every protocol and feature.

## Decision
Only Rust remains (`rust/`). The interface is egui through eframe; the
library crate holds all logic, the binary adds the GUI. The Python prototype
was retired (it stays in git history).

## Consequences
- One self-contained executable per platform; Windows, Linux (AppImage) and macOS builds from CI.
- The library is usable without a GUI (`rust/examples/`), and must stay so ([B1](../boundaries.md)).
- egui is immediate mode: all slow work runs in threads ([B2](../boundaries.md)).
