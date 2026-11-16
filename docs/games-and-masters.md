# Games and master servers

Which master server lists each game, grouped by query protocol. The source of
truth is [`rust/src/games.json`](../rust/src/games.json): masters, game types,
filter groups, registry detection and the command line used to join
([adding a game](development/adding-a-game.md)).

"OpenSpy" means the OpenSpy master servers (<https://github.com/openspy>),
"333networks" the 333networks masters.

| Game | Protocol | Server list from |
|---|---|---|
| Quake 3: Arena | Quake 3 | master.quake3arena.com, master0.excessiveplus.net, dpmaster.deathmask.net, master.ioquake3.org, master.maverickservers.com |
| Urban Terror | Quake 3 | master.urbanterror.info, master2.urbanterror.info |
| OpenArena | Quake 3 | dpmaster.deathmask.net, master.ioquake3.org |
| Xonotic, Warsow | Quake 3 (DarkPlaces, qfusion) | dpmaster.deathmask.net, dpmaster.tchr.no |
| Warfork | Quake 3 (qfusion) | dpmaster.deathmask.net |
| Tremulous | Quake 3 | master.tremulous.net |
| Smokin' Guns | Quake 3 | master.smokin-guns.org |
| Wolfenstein: Enemy Territory | Quake 3 | etmaster.idsoftware.com, etmaster.etlegacy.com |
| Call of Duty, CoD: United Offensive, CoD 4 | Quake 3 | Activision masters |
| Call of Duty 2 | Quake 3 | Activision master, master.cod2x.me |
| Star Wars Jedi Knight II: Jedi Outcast | Quake 3 | master.jkhub.org, master.jk2mv.org, master.ouned.de |
| Star Wars Jedi Knight: Jedi Academy | Quake 3 | master.jkhub.org, master.jk2mv.org |
| Return to Castle Wolfenstein | Quake 3 | wolfmaster.idsoftware.com |
| Star Trek: Voyager Elite Force | Quake 3 | master.stvef.org, efmaster.tjps.eu |
| Soldier of Fortune II: Double Helix | Quake 3 | master.1fxmod.org |
| DOOM 3 | Doom 3 | idnet.ua-corp.com |
| Quake 4 | Doom 3 | q4master.idsoftware.com |
| QuakeWorld | QuakeWorld | master.quakeservers.net, qwmaster.fodquake.net, master.quakeworld.nu |
| Quake 2 | Quake 2 | master.quakeservers.net, master.itbacon.com |
| Kingpin | Quake 2 | master.kingpin.info, master4.kingpin.info |
| Unreal Tournament (99), Unreal, Rune | GameSpy | 333networks, OpenSpy |
| Deus Ex, Wheel of Time, Mobile Forces, Drakan, Turok 2 | GameSpy | 333networks |
| Serious Sam: The First / Second Encounter, Heretic II, Global Operations, Medal of Honor: Allied Assault (+ Spearhead, Breakthrough) | GameSpy | 333networks, OpenSpy |
| Soldier of Fortune | GameSpy | 333networks, errorist.eu, OpenSpy |
| No One Lives Forever 1 and 2, Gore, Vietcong, Star Trek: Bridge Commander, Battlefield 1942 | GameSpy | OpenSpy |
| SWAT 4 | GameSpy | master.swat4stats.com, OpenSpy |
| Battlefield Vietnam, F.E.A.R., Star Wars: Battlefront, Tribes: Vengeance | GameSpy v2 | OpenSpy |
| Halo: Combat Evolved, Halo: Custom Edition | GameSpy v2 | s1.master.hosthpc.com |
| Battlefield 2 | GameSpy v3 | OpenSpy, battlefield2.ms.playbf2.ru |
| Unreal Tournament 2004, Unreal Tournament 2003 | Unreal Engine 2 | 333networks, OpenSpy |
| Multi Theft Auto: San Andreas | ASE (MTA) | master.multitheftauto.com |
| Starsiege: Tribes | Tribes | t1m1.tribes1.co, t1m1.pu.net |
| Tribes 2 | Tribes 2 | TribesNext (master.tribesnext.com) |
| Savage: The Battle for Newerth | Savage | Savage DRX server list |
| Rainbow Six 3: Raven Shield | Raven Shield | OpenRVS server list |
| Ghost Recon (+ demo), The Sum of All Fears (+ demo) | Ghost Recon | none: Add server and LAN only |
| Zandronum (Doom) | Zandronum | master.zandronum.com |
| Counter-Strike 1.6, Condition Zero, Team Fortress Classic, Day of Defeat, Half-Life, Opposing Force, Deathmatch Classic, Ricochet | Valve A2S | Steam Web API (needs your own key) |
| Counter-Strike: Source, Day of Defeat: Source, Half-Life 2: Deathmatch, Team Fortress 2, Left 4 Dead, Left 4 Dead 2, Garry's Mod | Valve A2S | Steam Web API (needs your own key) |

## Notes

- Ghost Recon and The Sum of All Fears have no master server left: their
  servers work through Add server and LAN broadcast.
- Rogue Spear, Urban Ops and Black Thorn are not in yet (no live server,
  undocumented protocol).
- Games from the original All-Seeing Eye that are not in the catalog yet are
  listed in [ROADMAP.md](../ROADMAP.md).
- How each protocol and master kind is spoken: [protocols](architecture/protocols.md).
