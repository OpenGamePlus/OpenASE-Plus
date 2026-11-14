# Architecture Decision Records

MADR-lite. An accepted ADR is never edited: a later ADR supersedes it and the
old one only gets its Status changed to `Superseded by NNNN`.

| # | Decision | Status |
|---|---|---|
| [0001](0001-rust-egui-single-binary.md) | Rust + egui, one native binary | Accepted |
| [0002](0002-catalog-as-data.md) | The game catalog is data (`games.json`) | Accepted |
| [0003](0003-offline-tests.md) | Tests are offline | Accepted |
| [0004](0004-no-proprietary-ase-files.md) | Nothing from the proprietary ASE | Accepted |
| [0005](0005-333networks-light-lists.md) | 333networks lists from the JSON, queries on demand | Accepted |
| [0006](0006-independent-fork.md) | OpenASE-Plus is an independent fork | Accepted |

## Template

File `NNNN-short-title.md`, next free number:

```markdown
# NNNN. Title

- Status: Proposed | Accepted | Superseded by NNNN
- Date: YYYY-MM-DD

## Context
What forces the decision (facts, constraints), 2-5 lines.

## Decision
What we do, stated as a rule.

## Consequences
What gets easier, what gets harder, what is now forbidden.
```
