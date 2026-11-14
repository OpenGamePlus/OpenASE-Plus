# Glossary

| Term | Meaning |
|---|---|
| ASE | The All-Seeing Eye, the closed Yahoo browser OpenASE recreates |
| Master | Server that lists game servers (`masters` in `games.json`) |
| Master kind | How a master list is fetched (`kind`), independent of the query protocol |
| Protocol | How one server is queried (`protocol`) |
| Query port | Port that answers queries; may differ from the join port (`query_port_offset`, `game_port_query_offset`) |
| Ping / Flux / PL | Mean round trip, (max − min) / 2, lost samples in % ([protocols](architecture/protocols.md)) |
| Rule | A server key/value (cvar), shown in the Rules pane |
| Listed | A master entry before or without a query (`model::Listed`); 333networks entries carry a `preview` |
| Scan / generation | One run of master + queries; messages of an older generation are dropped |
| Join template | `join` command line with `{placeholders}` and optional `[...]` groups |
| Auto-Search | Finding installed games: registry, then disk search for `exe_names` |
| Special lists | ASE's "Special" folder: recently added, recently imported, LAN servers (`Special` in `src/gui/mod.rs`) |
| Groups | Favorites1..8 and Friends1..4 (`Nav::FavoriteGroup`, `Nav::FriendGroup`) |
| My filters / color filters | User rules over servers (`rules.rs`) |
| 333networks, OpenSpy, dpmaster | Community-run masters reviving dead official ones |
| A2S | Valve's server query protocol |
| JASE | The maintainer's earlier Python browser; source of ideas for OpenASE-Plus |
| -Plus | The maintainer's family of forks (OpenASE-Plus, OpenCoD2-Plus, ...) |
