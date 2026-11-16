# Protocols and master kinds

`protocol` (in `games.json`) says how a server is queried; master `kind` says
how the list is fetched. They are independent: SWAT 4 is queried with
`gamespy` and listed by a `gamespy_sb` master. Source of truth: `Protocol` and
`Master` in `rust/src/catalog.rs`.

## Query protocols

| `protocol` | Module | Games (examples) |
|---|---|---|
| `quake3` | `quake3.rs` | Quake 3, Urban Terror, OpenArena, ET, RtCW, CoD 1/2/4, JK2/JKA, Elite Force, SoF2 |
| `quake2`, `quakeworld` | `quake2.rs` | Quake 2, Kingpin, QuakeWorld |
| `a2s` | `a2s.rs` | Half-Life 1 and Source games (Steam) |
| `gamespy` | `gamespy.rs` | UT99, Unreal, Deus Ex, MOHAA family, Serious Sam, BF1942, SoF |
| `gamespy2`, `gamespy3` | `gamespy2.rs` | BF Vietnam, Halo, F.E.A.R., SW Battlefront (v2); BF2 (v3) |
| `unreal2` | `unreal2.rs` | UT2003, UT2004 |
| `ase` | `ase.rs` | Multi Theft Auto (ASE's own query) |
| `tribes`, `tribes2` | `tribes.rs` | Starsiege: Tribes, Tribes 2 |
| `doom3` | `doom3.rs` | DOOM 3, Quake 4 |
| `savage` | `savage.rs` | Savage |
| `ravenshield` | `ravenshield.rs` | Raven Shield (query port = game port + 1000) |
| `ghostrecon` | `ghostrecon.rs` | Ghost Recon, The Sum of All Fears (no master) |
| `zandronum` | `zandronum.rs` | Zandronum (Doom); Huffman packets, one ping sample (a server ignores a second query for ~10 s) |

## Master kinds

| `kind` | Module | Fields |
|---|---|---|
| `quake3` | `quake3.rs` | `address` (`host:port`), `query` (`getservers <protocol> empty full`) |
| `quake2` | `quake2.rs` | `address` (`master.quakeservers.net:27900`), `query` (literally `query`) |
| `quakeworld` | `quake2.rs` | `address` (`master.quakeworld.nu:27000`) |
| `gamespy` | `gamespy.rs` | `address`, `gamename`, `key` (the game's public GameSpy key) |
| `gamespy_sb` | `gamespy_sb.rs` | `address` (TCP 28910 server browsing, encrypted: `battlefield2.master.openspy.net:28910`), `gamename`, `key` |
| `networks333` | `networks333.rs` | `gamename`, optional `api`; JSON API with server details, paged |
| `steam_web` | `steam.rs` | `filter` (`\appid\10`); needs a Steam Web API key |
| `unreal2` | `unreal2.rs` | `address` (`utmaster.openspy.net:28902`), `client` + `version` (`UT2K4CLIENT` 3369 for UT2004, `CLIENT` 2225 for UT2003) |
| `mta` | `ase.rs` | `url` (HTTP, binary list: `https://master.multitheftauto.com/ase/mta/`) |
| `tribes` | `tribes.rs` | `address` (UDP: `t1m1.tribes1.co:28000`) |
| `http_list` | `tribes.rs` | `url` (one `ip:port` per line: `http://master.tribesnext.com/list`) |
| `doom3` | `doom3.rs` | `address`, `version` (`1.0` DOOM 3, `2.0` Quake 4) |
| `savage_drx` | `savage.rs` | `url` (JSON: `https://savagedrx.com/api/server/online`) |
| `zandronum` | `zandronum.rs` | `address` (UDP: `master.zandronum.com:15300`); asks ~10 s between queries |
| `csv` | `ravenshield.rs` | `url` (CSV with `ip`, `port`: `http://api.openrvs.org/servers`) |

`"masters": []` is valid: servers then come from Add server, lists and LAN.

## Ports

| Field | Meaning |
|---|---|
| `default_port` | Port when an address has none |
| `query_port_offset` | Join port + offset = query port, default 0; addresses hold the join port (MTA 123, Ghost Recon / SoAF 2, Raven Shield 1000) |
| `game_port_query_offset` | Unreal engine: query = game port + 1; addresses hold the query port, never query the game port (`query::resolve_query_port`) |

## Ping, Flux, PL

`QueryOptions` (`pings`, `retries`, `timeout`) from Options > Network.
`query::query_server` takes `pings` round-trip samples (the status query is
the first); `query::apply_ping_stats` sets Ping = mean, Flux = (max − min) / 2,
PL = lost / attempts in %. A ping request that is rate-limited or not supported
is retried once after a pause instead of counting as loss.

## References

- Quake 3 connectionless queries: Jacek Fedorynski, "Quake 3 Network Protocol"
  (2020), https://www.jfedor.org/quake3/
- Valve server queries (A2S): Valve Developer Wiki, "Server queries".
- GameSpy master protocol: 333networks, https://333networks.com/
- GameSpy v2 / v3 and server browsing (Battlefield, Halo, F.E.A.R., SWAT 4):
  OpenSpy, https://github.com/openspy
- Unreal Engine 2 master (UT2003, UT2004): 333networks and OpenSpy, as above.
- Multi Theft Auto master list: mtasa-blue source,
  https://github.com/multitheftauto/mtasa-blue
- Quake 2 and QuakeWorld protocols: connectionless queries as in the id
  Software sources, with the community masters at QuakeServers.net.
- Starsiege: Tribes and Tribes 2 queries, and the Tribes 2 master list:
  TribesNext, http://www.tribesnext.com/ and the qstat protocol descriptions
  (https://github.com/multiplay/qstat).
- DOOM 3 and Quake 4 `getInfo` / `getServers` (id Tech 4): the dhewm3 modwiki
  and the qstat descriptions.
- Savage: The Battle for Newerth: query as described by qstat, server list from
  Savage DRX, https://savagedrx.com/
- Raven Shield: beacon protocol as described by qstat, server list from the
  OpenRVS project, https://openrvs.org/
- Ghost Recon and The Sum of All Fears: the Red Storm status query, as
  described by qstat.
- Yahoo acquired The All-Seeing Eye in September 2004:
  https://www.gamedeveloper.com/game-platforms/yahoo-games-adds-all-seeing-eye-gamespot-syndication
