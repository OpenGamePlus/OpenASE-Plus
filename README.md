# OpenASE-Plus

<p align="center"><img src="docs/banner.svg" alt="OpenASE-Plus" width="100%"></p>

[![Release](https://img.shields.io/github/v/release/OpenGamePlus/OpenASE-Plus)](https://github.com/OpenGamePlus/OpenASE-Plus/releases/latest)
[![License: GPL-3.0](https://img.shields.io/badge/license-GPL--3.0-blue)](LICENSE)
![Platforms: Windows | Linux | macOS](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey)
![Games: 80](https://img.shields.io/badge/games-80-19f5e1)
![Rust 1.88+](https://img.shields.io/badge/rust-1.88%2B-orange)
[![Fork of tsautier/OpenASE](https://img.shields.io/badge/fork%20of-tsautier%2FOpenASE-555)](https://github.com/tsautier/OpenASE)
[![CI](https://github.com/OpenGamePlus/OpenASE-Plus/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/OpenGamePlus/OpenASE-Plus/actions/workflows/ci.yml)
[![Last commit](https://img.shields.io/github/last-commit/OpenGamePlus/OpenASE-Plus/main)](https://github.com/OpenGamePlus/OpenASE-Plus/commits/main)

**The All-Seeing Eye is still watching.** OpenASE-Plus is a game server browser for
the classic online games that still have players, in the spirit of The All-Seeing
Eye: a filter tree on the left, the server list with Ping, Flux and PL on the right,
and friends, players and server rules at the bottom. For Windows, Linux and macOS.

It is a fork of [OpenASE](https://github.com/tsautier/OpenASE) by Thomas Sautier,
part of the [OpenGamePlus](https://github.com/OpenGamePlus) family. On top of
OpenASE's work it brings in the features of JASE, an earlier browser by the same
maintainer: more games, alerts, links, RCON, themes, flags and map images. Like the
rest of the family, it is developed with AI coding assistants. It is new code: it
does not use, modify or ship anything from the original Yahoo program.

<p align="center"><img src="docs/screenshots/classic.png" alt="OpenASE-Plus, Classic theme: Quake 3 servers sorted by players, with the players and rules of the selected server"></p>
<table>
  <tr>
    <td><img src="docs/screenshots/dark.png" alt="OpenASE-Plus, Dark theme"></td>
    <td><img src="docs/screenshots/goldsrc.png" alt="OpenASE-Plus, GoldSrc theme"></td>
  </tr>
</table>
<p align="center"><sub>Quake 3 servers in the Classic, Dark and GoldSrc themes.</sub></p>

## What works today

### For players

- **80 games** in 16 query protocols, listed by 49 master servers: Quake 3 and its
  family, Call of Duty 1 to 4, Unreal Tournament, Battlefield, Half-Life and Source
  games, Tribes, Zandronum and more ([the list](#supported-games)).
- **Ping, Flux and PL** measured like ASE: several queries per server, average
  round-trip time, half the spread between fastest and slowest, and packet loss.
- **Filter tree**: game types as folders, checkbox groups per game (Mods, Friendly
  Fire, Kill Cam, Pure, PunkBuster, Version, Maps), **Modifiers** (not empty, not
  full, no password, unresponsive, ping, country), **Installed games only** and a
  filter bar under the tree.
- **My filters**: rules such as `players greater than 4` or `map contains toujane`;
  tick *color filter* to highlight matching servers.
- **Join** with a double-click or Enter, with the right mod, player name, config and
  password. Steam games fall back to `steam://connect`.
- **Alerts**: be told when a slot frees up on a full server, or when a match is
  filling up.
- **Playing now**: one list of every busy server across the games you can join.
- **Favorites 1 to 8**, **Special** lists (recently added, imported, joined), **LAN
  servers** found by broadcast, and the last scan of each game kept between sessions.
- **Friends** in groups, optionally per game, with a Friends pane, notices and sounds,
  and a friend name search across every scanned server.
- **`openase://connect/<game>/<host>:<port>` links**: right-click a server to copy
  one; opening a link shows the server in the running window (Windows, Linux).
- **Country flags**, ping colors, an occupancy bar, game icons taken from your
  installed games, and the **map image** of the selected server read from your
  game's `.pk3` files (Quake 3 engine games). Nothing is downloaded.
- **Themes**: Classic, Dark, Terminal, Amber, Synthwave and GoldSrc (the olive and
  gold of Counter-Strike 1.6), with editable colors.
- **Auto-Search** finds installed games on the drives and folders you pick, or all of
  them in one pass.
- **Seven languages**: English, French, Spanish, Portuguese, Italian, German and
  Russian ("System" follows your operating system).
- Profile export and import, server list save and import, update check, setup
  wizard, customizable toolbar. Shortcuts: F5 refresh, Enter join, Ctrl+E show
  selected, Ctrl+D clear modifiers, Ctrl+Q collapse all, F1 manual.

### For server admins

- **RCON console** (right-click a server) for Quake 3, Call of Duty, Quake 2,
  Half-Life and Source servers. The password is kept for the session only.
- **`openase-cli`**: `games`, `scan` and `query` without the GUI, with `--json`
  output ([docs/cli.md](docs/cli.md)), in the Windows zip and the Linux tar.gz.
- **Add server** by address, and server lists to share.

Every change since OpenASE 0.9.6 is in [CHANGELOG.md](CHANGELOG.md).

> [!NOTE]
> **First release.** The Join command of many of the newer games has not been tested
> with the game installed ([OpenASE issue #1](https://github.com/tsautier/OpenASE/issues/1)).
> Please report what goes wrong in the [issues](https://github.com/OpenGamePlus/OpenASE-Plus/issues).

> [!IMPORTANT]
> **No games are included.** OpenASE-Plus lists servers and starts the games you
> already have installed. The Half-Life and Source games need your own free
> [Steam Web API key](#steam-games) for their server list.

## How to install

Every [GitHub release](https://github.com/OpenGamePlus/OpenASE-Plus/releases/latest) ships three packages:

| System | Package | How to run |
| --- | --- | --- |
| Windows 10/11 (x64) | `OpenASE-Plus-<version>-windows-x64.zip` | Unzip, run `OpenASEplus.exe`. |
| Linux (x64) | `OpenASE-Plus-<version>-linux-x86_64.AppImage` or `OpenASE-Plus-<version>-linux-x64.tar.gz` | AppImage: `chmod +x` it and run it, on any distribution. Archive: extract, run `./openase-plus`; an `openase-plus.desktop` file and icon are included. |
| macOS 11+ (Apple Silicon and Intel) | `OpenASE-Plus-<version>-macos-universal.zip` | Unzip, move `OpenASE-Plus.app` to Applications. The app is not notarized: the first time, right-click it and choose **Open**, or run `xattr -dr com.apple.quarantine /Applications/OpenASE-Plus.app`. |

The games themselves still need to run on your system: native Linux and macOS
builds exist for ioquake3, Urban Terror, OpenArena, ET: Legacy, Jedi Outcast
(JK2MV), Jedi Academy (OpenJK), RtCW (iortcw), Unreal Tournament (OldUnreal
patch) and the Half-Life and Source games (Steam). More ports: Quake 2 (Yamagi
Quake II, Q2PRO), QuakeWorld (ezQuake, FTE), Serious Sam (SamTFE and SamTSE
ports) and Unreal Tournament 2004, which has a native Linux client. Call of
Duty, Multi Theft Auto, Battlefield 2, Halo and SWAT 4 run on Windows only, or
through Wine / Proton.

## Supported games

| Game | Protocol | Server list from |
|---|---|---|
| Quake 3: Arena | Quake 3 | master.quake3arena.com, master0.excessiveplus.net, dpmaster.deathmask.net |
| Urban Terror | Quake 3 | master.urbanterror.info, master2.urbanterror.info |
| OpenArena | Quake 3 | dpmaster.deathmask.net |
| Xonotic, Warsow, Warfork | Quake 3 (DarkPlaces, qfusion) | dpmaster.deathmask.net |
| Tremulous | Quake 3 | master.tremulous.net |
| Smokin' Guns | Quake 3 | master.smokin-guns.org |
| Wolfenstein: Enemy Territory | Quake 3 | etmaster.idsoftware.com, etmaster.etlegacy.com |
| Call of Duty, CoD: United Offensive, CoD 2, CoD 4 | Quake 3 | Activision masters |
| Star Wars Jedi Knight II: Jedi Outcast | Quake 3 | master.jkhub.org, master.jk2mv.org |
| Star Wars Jedi Knight: Jedi Academy | Quake 3 | master.jkhub.org, master.jk2mv.org |
| Return to Castle Wolfenstein | Quake 3 | wolfmaster.idsoftware.com |
| Star Trek: Voyager Elite Force | Quake 3 | master.stvef.org, efmaster.tjps.eu |
| Soldier of Fortune II: Double Helix | Quake 3 | master.1fxmod.org |
| DOOM 3 | Doom 3 | idnet.ua-corp.com |
| Quake 4 | Doom 3 | q4master.idsoftware.com |
| QuakeWorld | QuakeWorld | master.quakeservers.net, qwmaster.fodquake.net, master.quakeworld.nu |
| Quake 2 | Quake 2 | master.quakeservers.net, master.itbacon.com |
| Kingpin | Quake 2 | master.kingpin.info, master4.kingpin.info |
| Soldier of Fortune | GameSpy | 333networks, errorist.eu, OpenSpy |
| Unreal Tournament 2004, Unreal Tournament 2003 | Unreal Engine 2 | 333networks, OpenSpy |
| Unreal Tournament (99), Unreal, Rune | GameSpy | 333networks, OpenSpy |
| Deus Ex, Wheel of Time, Mobile Forces, Drakan, Turok 2 | GameSpy | 333networks |
| Serious Sam: The First / Second Encounter, Heretic II, Global Operations, Medal of Honor: Allied Assault (+ Spearhead, Breakthrough) | GameSpy | 333networks, OpenSpy |
| No One Lives Forever 1 and 2, Gore, Vietcong, Star Trek: Bridge Commander, Battlefield 1942 | GameSpy | OpenSpy |
| Battlefield 2 | GameSpy v3 | OpenSpy, battlefield2.ms.playbf2.ru |
| Battlefield Vietnam, F.E.A.R., Star Wars: Battlefront, Tribes: Vengeance | GameSpy v2 | OpenSpy |
| Halo: Combat Evolved, Halo: Custom Edition | GameSpy v2 | s1.master.hosthpc.com |
| SWAT 4 | GameSpy | master.swat4stats.com, OpenSpy |
| Multi Theft Auto: San Andreas | ASE (MTA) | master.multitheftauto.com |
| Starsiege: Tribes | Tribes | t1m1.tribes1.co, t1m1.pu.net |
| Tribes 2 | Tribes 2 | TribesNext (master.tribesnext.com) |
| Savage: The Battle for Newerth | Savage | Savage DRX server list |
| Rainbow Six 3: Raven Shield | Raven Shield | OpenRVS server list |
| Zandronum (Doom) | Zandronum | master.zandronum.com |
| Ghost Recon (+ demo), The Sum of All Fears (+ demo) | Ghost Recon | none: Add server and LAN only |
| Counter-Strike 1.6, Condition Zero, Team Fortress Classic, Day of Defeat, Half-Life, Opposing Force, Deathmatch Classic, Ricochet | Valve A2S | Steam Web API (needs your own key) |
| Counter-Strike: Source, Day of Defeat: Source, Half-Life 2: Deathmatch, Team Fortress 2, Left 4 Dead, Left 4 Dead 2, Garry's Mod | Valve A2S | Steam Web API (needs your own key) |

80 games in total, spoken in 16 query protocols and listed by 49 master
servers. "OpenSpy" means the OpenSpy master servers
(<https://github.com/openspy>), "333networks" the 333networks masters.
Ghost Recon and The Sum of All Fears have no master server left: their
servers work through Add server and LAN broadcast. Rogue Spear, Urban Ops and
Black Thorn are not in yet (no live server, undocumented protocol). Games from
the original All-Seeing Eye that are not in the catalog yet are listed in
[ROADMAP.md](ROADMAP.md).

Games are described in [`rust/src/games.json`](rust/src/games.json): masters,
game types, filter groups, registry detection and the command line used to join.

## For contributors

Every kind of help counts: reporting a broken master or server, testing a game's
Join, translating, or code. Most new games are a single entry in the catalog, see
[adding a game](docs/development/adding-a-game.md). Work happens on branches merged
into `main` through pull requests. The technical documentation starts at
[docs/README.md](docs/README.md); [AGENTS.md](AGENTS.md) is the entry point for AI
coding agents, and [CONTRIBUTING.md](CONTRIBUTING.md) has the rules.

## Building

Requires Rust 1.88 or newer. On Linux, install the GUI libraries first
(Debian / Ubuntu: `sudo apt install libxcb-render0-dev libxcb-shape0-dev
libxcb-xfixes0-dev libxkbcommon-dev libgtk-3-dev`).

```powershell
cd rust
cargo run --release
```

Command-line tools, useful to test a master or a server:

```powershell
cargo run --example scan -- list              # list game ids
cargo run --example scan -- cod2 20           # scan 20 CoD2 servers
cargo run --example query -- ut99 1.2.3.4:7778
cargo test
```

Settings are stored in `%APPDATA%\OpenASE-Plus\settings.json` on Windows
(`~/.config/OpenASE-Plus` on Linux, `~/Library/Application Support/OpenASE-Plus`
on macOS). On first start, OpenASE's settings are copied from the `OpenASE` folder
next to it, once.

### Steam games

This covers every Half-Life and Source game in the catalog: Counter-Strike 1.6,
Team Fortress Classic, Day of Defeat, Half-Life, Condition Zero, Opposing
Force, Deathmatch Classic, Ricochet, Counter-Strike: Source, Day of Defeat:
Source and Half-Life 2: Deathmatch. Valve's old UDP master no longer answers,
so the list comes from the Steam Web API. Get a free key at https://steamcommunity.com/dev/apikey and paste it in
Options > General. Servers added by hand work without a key.

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

## License

Copyright (C) 2026 Thomas Sautier (OpenASE)
Copyright (C) 2026 OpenGamePlus contributors (OpenASE-Plus)

OpenASE-Plus is free software: you can redistribute it and/or modify it under the
terms of the GNU General Public License as published by the Free Software
Foundation, either version 3 of the License, or (at your option) any later
version. See [LICENSE](LICENSE) for the full text.

OpenASE-Plus is not affiliated with Yahoo or the original All-Seeing Eye authors.
Game names are trademarks of their owners.

IP geolocation by [DB-IP](https://db-ip.com) (CC BY 4.0), downloaded at runtime.
Country flags from [flag-icons](https://github.com/lipis/flag-icons) (MIT,
`rust/assets/flags-LICENSE.txt`). No original ASE files are distributed
with this project.
