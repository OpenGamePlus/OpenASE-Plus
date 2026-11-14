# Changelog

All notable changes to OpenASE-Plus, and before 1.0.0 to OpenASE. Downloads
are on the [OpenASE-Plus releases page](https://github.com/OpenGamePlus/OpenASE-Plus/releases);
versions up to 0.9.6 on [OpenASE's](https://github.com/tsautier/OpenASE/releases).

## 1.0.0 — unreleased (first OpenASE-Plus release)

### Identity
- **OpenASE-Plus**, a fork of OpenASE 0.9.6 by Thomas Sautier. It installs next to
  OpenASE (own installer id, `OpenASE-Plus.exe`, `openase-plus` on Linux) and keeps its
  settings in an `OpenASE-Plus` folder; OpenASE's settings are imported on first start.
- New icon: a neon cyan eye in a triangle in a circle, after JASE's colors
  (`scripts/make_icon.sh`; a bolder variant at 16 and 24 px).
- Update check, links, packages and the 333networks User-Agent point at
  github.com/OpenGamePlus/OpenASE-Plus.

### Auto-Search
- Auto-Search first asks where to look: tick drives and usual folders, or add a folder;
  the choice is remembered. Searching one drive instead of all of them is much faster.
- **Search all games...** (Options > Games) finds every installed game in one pass and
  lists them; sure matches are ticked, and "Apply" sets their program locations. A
  program shared by several games (hl2.exe) is ticked only when its folder names the game.

### Languages
- **Spanish, Portuguese (Brazil), Italian, German and Russian** join English and French.
  Options > General > Language; "System" follows the operating system. Translations
  live in `rust/src/locales/*.tsv`.

### Game icons
- Games show the icon of their installed executable; the others a two-letter badge in the
  game's color, as in JASE.

### New games
- Xonotic, Warsow, Warfork, Tremulous and Smokin' Guns (Quake 3 protocol).
- Team Fortress 2, Left 4 Dead, Left 4 Dead 2 and Garry's Mod (Steam list, needs your own key).
- Zandronum (multiplayer Doom), with its own protocol and master. The server's IWAD is
  passed to the game with `-iwad`.

### New
- **Alert me when a slot is free** (right-click a full server): OpenASE checks it every
  10 seconds for up to 30 minutes, then selects it and shows a notice.
- **Alert when players reach N** (right-click a server): a notice when that many people,
  not bots, are playing. Saved between sessions; the servers are checked every minute.

- **Playing now** (in the Special folder): every server with people on it, across all the
  games you can join (a program set in Options > Games, or Steam games with your key).
  F5 there scans those games together.

- **`openase://connect/<game>/<host>:<port>` links**: right-click a server to copy one.
  Opening a link shows the server and asks before joining; if OpenASE is already open,
  the link goes to that window. The Windows installer and the Linux `.desktop` file
  register the scheme (not macOS yet).

- **Lists kept between sessions**: the last complete scan of each game shows at start
  (ping "—" until a server is selected or the list is refreshed).
- **`openase-cli`**: `games`, `scan` and `query` without the GUI, with `--json` output
  (schema 1, see `docs/cli.md`). In the Windows zip and the Linux tar.gz.

- **RCON console** (right-click a server): send admin commands to Quake 3, Call of Duty,
  Quake 2, Half-Life and Source servers. The password is kept for the session only.

- **Themes** in Options > Colors: Classic, Dark, Terminal, Amber terminal, Synthwave and
  GoldSrc (the olive and gold of Counter-Strike 1.6).
  The colors stay editable on top of the theme.

- **Map images**: the levelshot of the selected server's map, read from your installed
  game's `.pk3` files (Quake 3 engine games). Nothing is downloaded.

- **Country flags** in Location, **ping colors** (good, medium, bad), an **occupancy bar**
  in Players, always-visible scroll bars and framed panels. Flags from flag-icons (MIT).

- **Installed games only** at the top of the tree, server counts next to each game, and a
  filter bar under it (hide empty, full, passworded or unresponsive servers; max ping;
  country).
- Themes other than Classic get taller rows, centered headers and no stripes; GoldSrc
  draws thin, pixel-exact sunken frames.

### Fixes
- Columns of the Friends, Players and Rules panes can no longer be widened past the
  pane, where their handle could not be reached again.
- The US flag (and other flags drawn with stroked lines) shows its stripes.
- A mod name sent by a server that starts with `-` or `.` is no longer passed to the game.
- The Tremulous master list is read: its packets end with a lone `\` instead of `\EOT`.
- **Security:** a player name, password or server address with spaces or quotes can no
  longer add arguments to the game's command line.
- **Security (Windows):** Steam joins no longer go through `cmd`, so `&` or `|` in a server
  address or password cannot run commands.
- Add server and Import server list no longer freeze the window while Unreal engine servers
  are checked for their query port.

## 0.9.6 — 2026-11-11

A lighter client for the 17 games listed by 333networks, after feedback from its admins.

### Changes
- **The list comes from 333networks' data, with no query per server.** Name, map, players,
  game type and country are shown straight from the JSON API. OpenASE no longer sends UDP
  queries to every listed server on each refresh; Ping shows "—" until a server is queried.
- **Servers are queried on demand**: when you select one, join it, refresh it, or refresh
  your favorites. The new **Server > Ping visible servers** queries only the rows on
  screen (60 at most).
- **Pages of 100 servers**, busiest first, instead of up to 1000 at once. The next page
  loads when you reach the end of the list or click "Load more servers". Each page is
  cached for 7.5 minutes and only one request runs at a time.
- A listed server that does not answer keeps its name and map and shows 100% packet loss.
- The Location column shows the country when a server only reports a numeric region.

### Removed
- The `\sync\` master request (0.9.3). It is meant for master-to-master synchronisation,
  not for clients; it was already unused since 0.9.4.

## 0.9.5 — 2026-11-08

### Changes
- **UT2004 list from 333networks' UT2004 JSON API**
  (https://ut2004master.333networks.com/json/ut2004) instead of its UE2 master on TCP
  28902, same rules as the other 333networks games (7.5-minute cache, OpenASE User-Agent).
  The OpenSpy UE2 master is still used as a second source.

## 0.9.4 — 2026-11-08

### Changes
- **333networks lists come from its JSON API**: at 333networks' request, the 16 games it
  lists now use https://master.333networks.com/json/ (served by its web server) instead of
  the master on TCP 28900, which stays free for game clients. One request per game, cached
  for 7.5 minutes (their refresh interval), with an `OpenASE` User-Agent. Help > About
  credits the community master servers, as the API terms ask.

## 0.9.3 — 2026-11-08

### Changes
- **One request for all 333networks games**: the 16 games listed by
  master.333networks.com (UT99, Unreal, Deus Ex, Rune, MOHAA, Soldier of Fortune...) now
  get their lists with a single `\sync\` request, cached for 5 minutes, instead of one
  `\list\` request per game. OpenASE identifies itself with `\msid\OpenASE\`. Suggested by
  333networks.

## 0.9.2 — 2026-11-08

### Fixes
- **Unreal engine servers added with their game port** (UT99, Unreal, Deus Ex, Rune,
  Wheel of Time, Mobile Forces, UT2003, UT2004, SWAT 4, Tribes: Vengeance) are now queried
  on their query port. Players usually share the game port (for example `ip:7777`), and
  querying it made the server open and drop a dead connection on every refresh. Add server
  and list imports check port + 1 once and keep it when that server reports the given port
  as its game port. Favorites saved with a game port are moved once, at the next start.
  Thanks to 333networks for spotting and helping debug this.

## 0.9.1 — 2026-11-08

### New
- **Windows installer**: `OpenASE-v0.9.1-windows-x64-setup.exe`, built with Inno Setup.
  Installs per user by default (no administrator prompt), adds a Start menu entry and an
  optional desktop shortcut, upgrades older installs in place, and keeps your settings when
  uninstalled. English and French. The portable zip is still published.
- **SHA256SUMS.txt** is attached to every release with the checksum of each package.
- Package manager manifests (winget, Scoop) can now be generated from `packaging/`.

### Changes
- The website moved to **https://openase.org**. The Help menu and About window link there.
- **Friend sounds on Linux and macOS**: .wav files and the default notification sound now
  play there too, with the system's own player (pw-play, paplay or aplay on Linux, afplay
  on macOS).

### Known issues
- The installer and the program are not code-signed yet, so Windows SmartScreen may warn
  before running them ("More info" > "Run anyway").

## 0.9.0 — 2026-11-08

70 games (up from 59) and 15 query protocols (up from 9).

### New games
- **Starsiege: Tribes** and **Tribes 2**: their own binary protocols. Lists come from
  the community masters (tribes1.co, pu.net) and TribesNext.
- **DOOM 3** and **Quake 4**: id Tech 4 protocol and the id masters.
- **Soldier of Fortune**: GameSpy query, lists from 333networks, errorist and OpenSpy.
- **Savage: The Battle for Newerth**: its own protocol, list from Savage DRX.
- **Rainbow Six 3: Raven Shield**: its own protocol, list from OpenRVS.
- **Ghost Recon** and **The Sum of All Fears** (and their demos): query only. No master
  is left, so add servers by hand or find them on the LAN.

### New features
- **English and French interface**: Options > General > Language. "System" follows
  the language of your operating system.
- **Customizable toolbar**: show, hide and reorder the buttons in Options > Toolbar,
  or right-click the toolbar.
- **A sound per friend**: in the Friends window, each friend can have the default
  sound, no sound, or a .wav file (.wav playback is Windows only).
- **Colors tab** in Options: list background, alternating rows, selection, text, and
  Quake color codes on or off.

### Known issues
- The Join command of many new games is not tested with the game installed yet.
  Help is welcome: [issue #1](https://github.com/tsautier/OpenASE/issues/1).
- Rogue Spear, Urban Ops and Black Thorn are not supported: no live server and no
  documented protocol.

## 0.8.1 — 2026-11-08

### Fixes
- Server and player names sent as UTF-8 or as Russian Windows-1251 now display
  correctly, instead of garbled text such as "CafÃ©" or "Ðóññêèé".
- The "Update available" window and other dialogs are no longer stretched with
  empty space.

## 0.8.0 — 2026-11-08

Back to the full ASE list: 59 games (up from 14) and 9 query protocols.

### New games
- **Steam**: Half-Life, Condition Zero, Opposing Force, Deathmatch Classic, Ricochet,
  Counter-Strike: Source, Day of Defeat: Source, Half-Life 2: Deathmatch.
- **Quake 3 engine**: Call of Duty: United Offensive, Star Trek: Elite Force,
  Soldier of Fortune II (community masters).
- **Quake 2 and QuakeWorld protocols**: QuakeWorld, Quake 2, Kingpin.
- **20 GameSpy v1 games** from 333networks and OpenSpy: Unreal, Deus Ex, Rune,
  Wheel of Time, Mobile Forces, Serious Sam 1 and 2, No One Lives Forever 1 and 2,
  Heretic II, Gore, Drakan, Turok 2, Global Operations, Vietcong, Bridge Commander,
  Medal of Honor: Allied Assault, Spearhead and Breakthrough, Battlefield 1942.
- **Unreal Engine 2 protocol and master**: Unreal Tournament 2003 and 2004.
- **The ASE protocol** and the Multi Theft Auto: San Andreas master.
- **GameSpy v2/v3 queries and the encrypted GameSpy master**: Battlefield 2,
  Battlefield Vietnam, Halo and Halo Custom Edition, SWAT 4, Tribes: Vengeance,
  F.E.A.R., Star Wars: Battlefront.

### Fixes
- Packet loss no longer counts servers that rate-limit queries (MTA) or ignore the
  short ping request (Elite Force, NOLF 2).

## 0.7.0 — 2026-11-07

### New games
- Star Wars Jedi Knight II: Jedi Outcast, Jedi Academy, Return to Castle Wolfenstein.
- Team Fortress Classic and Day of Defeat (Steam list).

### New features
- Linux AppImage and macOS builds next to Windows, smaller release binaries.
- Auto-Search finds native Linux and macOS game binaries.
- `CONTRIBUTING.md` with a guide to adding a game, issue and pull request templates.

### Removed
- The Python prototype. It remains in the git history.

## 0.6.0 — 2026-11-07

First public release: 9 games (Quake 3, Urban Terror, OpenArena, Wolfenstein: Enemy
Territory, Call of Duty 1, 2 and 4, Unreal Tournament, Counter-Strike 1.6) and the
ASE experience:

- Quake 3, GameSpy and Valve A2S query protocols, several masters per game.
- Ping, Flux and Packet Loss from several samples per server.
- The ASE filter tree with game types, mods, maps and per-game settings.
- Favorites1..8, Friends1..4, Special lists, LAN servers, custom and color filters.
- Join with double-click or Enter, Find player, friends tracking with a notice.
- Location column and country filter (DB-IP GeoIP, monthly download).
- Profile export/import, server list save/import, update check, first-run wizard.
- Windows build, website with manual.
