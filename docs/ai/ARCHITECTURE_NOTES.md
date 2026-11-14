# Architecture notes

Normative summary. Full rules and exceptions: [boundaries.md](../architecture/boundaries.md).

## Must
- Keep egui in `src/main.rs` and `src/gui/` only.
- Run network, DNS, disk search and process waits in worker threads; report through `Msg`; tag scan messages with `generation`.
- Add games as `games.json` entries; add protocols as module + `Protocol`/`Master` variant + `query.rs` branch.
- Treat server and master text as untrusted; validate anything that reaches a command line.
- Put every visible string through `tr` / `fill` / `noop` with a French entry.
- Write offline tests with captured packets.

## Must not
- Block the UI thread or call `recv` there.
- Start a shell (`sh -c`, `cmd /C`) or build arguments by string concatenation.
- Commit ASE files or game assets.
- Rename a game `id` or drop a settings field without a default.
- Bulk-query 333networks-listed servers.

## Irreversible or costly decisions
Index and status: [ADRs](../architecture/adr/README.md). Do not download map images ([ADR 0007](../architecture/adr/0007-map-images-from-local-files.md)).
