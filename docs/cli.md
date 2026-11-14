# openase-cli

OpenASE's master lists and server queries without the GUI
(`rust/src/bin/openase-cli.rs`). Shipped in the Windows zip and the Linux
tar.gz; from source: `cargo run --bin openase-cli -- <command>` in `rust/`.

| Command | Does |
|---|---|
| `games [--json]` | List the catalog |
| `scan <game> [--json] [--limit N] [--steam-key KEY]` | Ask the game's masters and query the servers (default limit 500; key default: `STEAM_API_KEY`) |
| `query <game> <host:port> [--json]` | Query one server (an Unreal game port becomes its query port) |

Exit codes: `0` done (`scan`: a master answered or none failed; `query`: the
server answered), `1` no answer, `2` usage error (unknown command, game or
option).

333networks rows are printed from the list (`"listed": true`) and not
queried, as its admins asked ([ADR 0005](architecture/adr/0005-333networks-light-lists.md)).
Without `--json`: a text table on stdout, errors and a summary on stderr.

## JSON contract, schema 1

Every document has `"schema": 1` and `"openase": "<version>"`. Fields may be
added within schema 1; removing, renaming or changing the meaning of one
raises the schema.

| Command | Fields |
|---|---|
| `games` | `games`: `id`, `name`, `short`, `protocol` (as in `games.json`), `default_port`, `needs_steam_key`, `masters` (labels) |
| `scan` | `game` (`id`, `name`), `listed` (entries from the masters), `errors` (one string per failed master), `servers` |
| `query` | `game`, `server` |

A server is the serde form of `model::Server`: `game`, `host`, `port`,
`join_port`, `online`, `listed`, `error`, `name` (raw, with color codes),
`map`, `game_type` (raw value), `mod_name`, `num_players`, `max_players`,
`bots`, `password`, `location`, `country`, `ping`, `flux`, `packet_loss`,
`players` (`name`, `score`, `ping`, `time_secs`) and `rules` (every
key/value). The same form is used for the saved lists (`rust/src/lists.rs`).
