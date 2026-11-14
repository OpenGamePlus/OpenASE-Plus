# Development guide

## Workflow
1. Read [README_AI](README_AI.md) order; check [debt](../roadmap/debt.md) for the area.
2. Branch from `main`; one topic per branch; small commits.
3. Before each commit, from `rust/`: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`.
4. Finished work: add a stage to [PROJECT_MEMORY](PROJECT_MEMORY.md), update the docs it made wrong, `CHANGELOG.md` for user-visible changes.

## Commits
English, imperative subject ≤ 72 chars ("Add Xonotic to the catalog"), body with
the why when not obvious. Docs changes go in the same commit as the code they describe.

## Doc-sync: when you change X, update Y

| Change | Update |
|---|---|
| Game added or removed | `README.md` games table, `ROADMAP.md`, counts in [SYSTEM_SUMMARY](SYSTEM_SUMMARY.md), `rust/tests/protocols.rs` id list |
| New protocol or master kind | [protocols.md](../architecture/protocols.md), [adding-a-game](../development/adding-a-game.md) if fields changed, SYSTEM_SUMMARY |
| New `games.json` field | [adding-a-game](../development/adding-a-game.md) |
| New module or a module changes layer | [overview.md](../architecture/overview.md), [boundaries.md](../architecture/boundaries.md) |
| New thread / `Msg` variant | [gui.md](../architecture/gui.md) |
| Settings file, config folder, outside resource | overview "Persistence", SYSTEM_SUMMARY contracts |
| Commands, toolchain, CI | [building.md](../development/building.md) |
| Release workflow or packaging | [releasing.md](../development/releasing.md), `packaging/README.md` |
| Debt fixed or found | [debt.md](../roadmap/debt.md) |
| Decision that a rule must change | new ADR, then boundaries and [ARCHITECTURE_NOTES](ARCHITECTURE_NOTES.md) |

## Docs checker
`rust/tests/docs.rs` runs with `cargo test` (alone: `cargo test --test docs`).
It checks `README.md`, `CONTRIBUTING.md`, `AGENTS.md`, `packaging/README.md`
and `docs/**/*.md` (not `docs/superpowers/`, local notes, git-ignored):

- relative links that do not exist, and `#anchors` that match no heading;
- backticked repository paths that do not exist (first segment a top-level
  folder, or `src/`, `tests/`, `examples/` resolved under `rust/`; `*` globs; `<...>` placeholders skipped);
- personal data in `AGENTS.md` and `docs/`: drive paths, `/home`, `/mnt`, `/Users`, emails (except `noreply@`, `@example.`), IPv4
  outside 192.0.2.x / 198.51.100.x / 203.0.113.x, 32+ hex digit strings;
- PROJECT_MEMORY stages: heading `## YYYY-MM-DD — topic`, at most 3 bullets;
- catalog counts in SYSTEM_SUMMARY and overview equal `games.json`.

Tests of the checker itself use placeholders only; their one private-range IP exists to test detection.

## Recipes
- Add a game: [adding-a-game.md](../development/adding-a-game.md).
- Port a JASE feature: read its module and tests in the JASE repository, write the Rust version against this repo's boundaries, record the stage.
- Write an ADR: template in [adr/README.md](../architecture/adr/README.md).

## Memory stage template

```markdown
## YYYY-MM-DD — topic
- What changed and why it mattered.
- What was rejected (and why).
- What is left open.
```
