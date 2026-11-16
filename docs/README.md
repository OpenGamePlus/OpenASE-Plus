# Technical documentation

`docs/` also hosts the website (`docs/index.html`). Users start at the
[README](../README.md); agents at [AGENTS.md](../AGENTS.md).

| Area | Documents |
|---|---|
| AI memory layer | [README_AI](ai/README_AI.md) · [PROJECT_CONTEXT](ai/PROJECT_CONTEXT.md) · [SYSTEM_SUMMARY](ai/SYSTEM_SUMMARY.md) · [ARCHITECTURE_NOTES](ai/ARCHITECTURE_NOTES.md) · [DEVELOPMENT_GUIDE](ai/DEVELOPMENT_GUIDE.md) · [PROJECT_MEMORY](ai/PROJECT_MEMORY.md) |
| Architecture | [overview](architecture/overview.md) · [boundaries](architecture/boundaries.md) · [protocols](architecture/protocols.md) · [gui](architecture/gui.md) · [ADRs](architecture/adr/README.md) |
| Development | [building and testing](development/building.md) · [adding a game](development/adding-a-game.md) · [releasing](development/releasing.md) |
| Roadmap | product plan [ROADMAP.md](../ROADMAP.md) · [debt and risks](roadmap/debt.md) · history [CHANGELOG.md](../CHANGELOG.md) |
| Users | [user guide](user-guide.md) · [games and masters](games-and-masters.md) |
| Reference | [glossary](glossary.md) · [openase-cli](cli.md) |

Writing conventions: English, tables and short bullets over prose, one topic
per file, relative links (with their `#anchor`) and backticked repository paths must exist (checked by `rust/tests/docs.rs`).
