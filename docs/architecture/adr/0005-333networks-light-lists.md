# 0005. 333networks lists from the JSON, queries on demand

- Status: Accepted
- Date: 2026-11-11

## Context
333networks' admins asked for a lighter client: OpenASE queried every listed server
by UDP on each refresh. Their JSON API already
returns name, map, players, game type and country.

## Decision
Games with a `networks333` master show rows from the JSON (`Listed.preview`)
and are queried only on demand: selection, join, refresh, favorites, or
Server > Ping visible servers (60 at most). Lists load in pages of 100,
busiest first, one request at a time, each page cached 7.5 minutes. The
`\sync\` master request is not used.

## Consequences
- Ping shows "—" until a 333networks server is queried.
- `query.rs` and the GUI handle paging (`fetch_more`, `Paging`, `Msg::MorePage`).
- Any new bulk query feature must respect this for 333networks games.
