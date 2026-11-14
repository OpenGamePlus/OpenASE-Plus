# 0006. OpenASE-Plus is an independent fork

- Status: Accepted
- Date: 2026-11-14

## Context
OpenASE-Plus forks OpenASE 0.9.6 as part of the maintainer's "-Plus" family of
forks, and will take ideas from the maintainer's earlier Python browser (JASE).

## Decision
The fork does not track upstream: files are restructured freely (for example
`CONTRIBUTING.md`), and upstream changes are ported by hand when wanted.
Documentation, code and commits stay in English.

## Consequences
- No merge-conflict constraints on docs or code layout.
- Upstream identity still in the code must be replaced on purpose (update URL, site, packaging): see [debt](../../roadmap/debt.md).
- Fixes worth sharing are offered upstream as separate, hand-made PRs.
