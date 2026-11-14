# Adding a game

A game on a supported protocol is one entry in `rust/src/games.json` (array in
display order). Protocols and master kinds: [protocols.md](../architecture/protocols.md).
A new protocol needs code ([B5](../architecture/boundaries.md)); open an issue first.

## Entry

Copy an entry of the same protocol. Quake 3, shortened:

```json
{
  "id": "q3", "name": "Quake 3: Arena", "short": "Q3", "color": "#c8321e",
  "protocol": "quake3", "default_port": 27960,
  "masters": [
    { "kind": "quake3", "address": "master.quake3arena.com:27950", "query": "getservers 68 empty full" }
  ],
  "game_type_key": "g_gametype",
  "game_types": [ { "value": "0", "name": "FFA" }, { "value": "4", "name": "CTF" } ],
  "mod_key": "gamename", "base_mod": "baseq3",
  "filters": [
    { "name": "Mods", "key": "gamename", "dynamic": true },
    { "name": "Pure", "key": "sv_pure", "options": [ { "label": "Yes", "values": ["1"] }, { "label": "No", "values": ["0"] } ] }
  ],
  "registry": [
    { "key": "HKEY_LOCAL_MACHINE\\SOFTWARE\\Id\\Quake III Arena", "value": "INSTALLPATH", "exe": "quake3.exe" }
  ],
  "exe_names": ["quake3.exe", "ioquake3.x86_64", "ioquake3"],
  "join": "[+set fs_game {mod} ][+exec \"{config}\" ][+name \"{name}\" ][+password \"{password}\" ]+connect {address}"
}
```

| Field | Req. | Meaning |
|---|---|---|
| `id` | yes | Unique, lowercase; stored in settings and used by `scan -- <id>`: never rename |
| `name`, `short`, `color` | yes | Tree name, list badge, badge color `#rrggbb` (a test parses it) |
| `protocol`, `default_port` | yes | Query protocol; port when an address has none |
| `masters` | yes | List of masters (may be `[]`); queried in parallel, merged |
| `steam_appid` | no | Steam app id (`cs16`: `10`); joins through `steam://connect` when no program is set |
| `game_type_key`, `game_types` | no | Rule with the game type; raw value → name (unknown shown raw) |
| `mod_key`, `base_mod` | no | Rule with the mod (`gamename` in Quake 3, `folder` in CS 1.6); the base mod is not passed as `{mod}` |
| `filters` | no | Tree checkboxes: `"dynamic": true` builds options from scanned values; otherwise `options`, each a `label` and the raw `values` it matches |
| `registry`, `exe_names` | no | Install detection: full registry `key`, the `value` holding the folder, `exe` relative to that folder; names for disk search (add Linux and macOS names) |
| `query_port_offset`, `game_port_query_offset` | no | See Ports in [protocols.md](../architecture/protocols.md) |
| `ping_request` | no | GameSpy v1 ping request, `\info\` when empty (SWAT 4: `\status\`) |
| `join` | yes | Command-line template, must contain `{address}` |

## Join template

Placeholders: `{address}` (`host:port`), `{ip}` and `{port}` (host and join port as separate arguments), `{name}`,
`{password}`, `{config}`, `{mod}`. A `[...]` group is dropped when a
placeholder inside is empty. Non-flag styles work too (`ut99`:
`{address}[?Name={name}][?Password={password}]`). Expansion and argument
splitting: `rust/src/launch.rs`.

## Interface texts

Visible GUI strings: `tr("English")`; with values one whole-sentence entry with `{}` placeholders, `fill(&tr("... {} ..."), &[&v])`;
in constants `noop("...")`, translated where shown. Each needs a French entry
in `rust/src/i18n.rs`; `rust/tests/i18n.rs` fails otherwise. Language names
are not translated.

## Check

1. Live: `cargo run --example scan -- <id> 20` (server count, sane names, maps, players, ping), then `cargo run --release` and find the game in the tree; join if installed.
2. Tests in `rust/tests/protocols.rs`: add the id to `catalog_has_every_game_with_masters_and_join_template` (exact list), game types to `game_type_names_are_mapped`, a captured packet test for any parser change.
3. Update the games table in `README.md` and `ROADMAP.md`, and the counts in [SYSTEM_SUMMARY](../ai/SYSTEM_SUMMARY.md).
