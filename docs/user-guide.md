# User guide

Everything OpenASE-Plus does, in detail. Start with the
[README](../README.md); every change is in [CHANGELOG.md](../CHANGELOG.md).

## Install

Every [GitHub release](https://github.com/OpenGamePlus/OpenASE-Plus/releases/latest) ships these packages:

| System | Package | How to run |
| --- | --- | --- |
| Windows 10/11 (x64) | `OpenASE-Plus-<version>-windows-x64.zip` | Unzip, run `OpenASEplus.exe`. |
| Linux (x64) | `OpenASE-Plus-<version>-linux-x86_64.AppImage` or `OpenASE-Plus-<version>-linux-x64.tar.gz` | AppImage: `chmod +x` it and run it, on any distribution. Archive: extract, run `./openase-plus`; an `openase-plus.desktop` file and icon are included. |
| macOS 11+ (Apple Silicon and Intel) | `OpenASE-Plus-<version>-macos-universal.zip` | Unzip, move `OpenASE-Plus.app` to Applications. The app is not notarized: the first time, right-click it and choose **Open**, or run `xattr -dr com.apple.quarantine /Applications/OpenASE-Plus.app`. |

### Games on Linux and macOS

The games themselves still need to run on your system: native Linux and macOS
builds exist for ioquake3, Urban Terror, OpenArena, ET: Legacy, Jedi Outcast
(JK2MV), Jedi Academy (OpenJK), RtCW (iortcw), Unreal Tournament (OldUnreal
patch) and the Half-Life and Source games (Steam). More ports: Quake 2 (Yamagi
Quake II, Q2PRO), QuakeWorld (ezQuake, FTE), Serious Sam (SamTFE and SamTSE
ports) and Unreal Tournament 2004, which has a native Linux client. Call of
Duty, Multi Theft Auto, Battlefield 2, Halo and SWAT 4 run on Windows only, or
through Wine / Proton.

## Features

### For players

- **Ping, Flux and PL** measured like ASE: several queries per server, average
  round-trip time, half the spread between fastest and slowest, and packet loss.
- **Version** column: the version each server reports, short ("1.3",
  "ioq3 1.36", "1.4 (CoD2x)").
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
  wizard, customizable toolbar.

### For server admins

- **RCON console** (right-click a server) for Quake 3, Call of Duty, Quake 2,
  Half-Life and Source servers. The password is kept for the session only.
- **`openase-cli`**: `games`, `scan` and `query` without the GUI, with `--json`
  output ([cli](cli.md)), in the Windows zip and the Linux tar.gz.
- **Add server** by address, and server lists to share.

## Shortcuts

| Key | Action |
|---|---|
| F5 | Refresh |
| Enter | Join |
| Ctrl+E | Show selected |
| Ctrl+D | Clear modifiers |
| Ctrl+Q | Collapse all |
| F1 | Manual |

## Steam games

This covers every Half-Life and Source game in the catalog: Counter-Strike 1.6,
Team Fortress Classic, Day of Defeat, Half-Life, Condition Zero, Opposing
Force, Deathmatch Classic, Ricochet, Counter-Strike: Source, Day of Defeat:
Source, Half-Life 2: Deathmatch, Team Fortress 2, Left 4 Dead, Left 4 Dead 2
and Garry's Mod. Valve's old UDP master no longer answers, so the list comes
from the Steam Web API. Get a free key at https://steamcommunity.com/dev/apikey
and paste it in Options > General. Servers added by hand work without a key.

## Where settings are stored

| System | Folder |
|---|---|
| Windows | `%APPDATA%\OpenASE-Plus\settings.json` |
| Linux | `~/.config/OpenASE-Plus` |
| macOS | `~/Library/Application Support/OpenASE-Plus` |

On first start, OpenASE's settings are copied from the `OpenASE` folder next to
it, once.
