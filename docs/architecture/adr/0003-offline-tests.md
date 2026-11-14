# 0003. Tests are offline

- Status: Accepted
- Date: 2026-11-14 (recorded; decided upstream before the fork)

## Context
Masters and servers come and go; CI runs on Windows, Linux and macOS runners
where outbound UDP is unreliable.

## Decision
Tests use captured packets or hand-written samples and never reach the
network. Live checks are done by hand with `cargo run --example scan` and
`cargo run --example query`.

## Consequences
- `cargo test` is deterministic and fast on every OS.
- A parser is only as good as its captures: a new protocol needs a real capture in its test.
- Live breakage (a dead master) is not caught by CI.
