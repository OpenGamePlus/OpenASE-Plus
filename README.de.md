[English](README.md) | [Français](README.fr.md) | [Español](README.es.md) | [Português](README.pt.md) | [Italiano](README.it.md) | **Deutsch** | [Русский](README.ru.md)

# OpenASE-Plus

<p align="center"><img src="docs/banner.svg" alt="OpenASE-Plus" width="100%"></p>

[![Release](https://img.shields.io/github/v/release/OpenGamePlus/OpenASE-Plus)](https://github.com/OpenGamePlus/OpenASE-Plus/releases/latest)
[![License: GPL-3.0](https://img.shields.io/badge/license-GPL--3.0-blue)](LICENSE)
![Platforms: Windows | Linux | macOS](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey)
![Games: 80](https://img.shields.io/badge/games-80-19f5e1)
[![Fork of tsautier/OpenASE](https://img.shields.io/badge/fork%20of-tsautier%2FOpenASE-555)](https://github.com/tsautier/OpenASE)

**The All-Seeing Eye wacht noch immer.** Finde einen Server für die klassischen Spiele, die du noch liebst, und tritt mit einem Doppelklick bei.

OpenASE-Plus ist ein kostenloser Server-Browser für Windows, Linux und macOS, im Geist des alten *All-Seeing Eye*. Wähle ein Spiel, sieh, wer gerade spielt, und steig ein: keine Konsolenbefehle, keine IPs zum Kopieren.

<p align="center"><img src="docs/screenshots/classic.png" alt="OpenASE-Plus, Classic theme: Quake 3 servers sorted by players, with the players and rules of the selected server"></p>
<table>
  <tr>
    <td><img src="docs/screenshots/dark.png" alt="OpenASE-Plus, Dark theme"></td>
    <td><img src="docs/screenshots/goldsrc.png" alt="OpenASE-Plus, GoldSrc theme"></td>
  </tr>
</table>
<p align="center"><sub>Quake-3-Server in den Designs Classic, Dark und GoldSrc.</sub></p>

**[⬇ Herunterladen](https://github.com/OpenGamePlus/OpenASE-Plus/releases/latest)** · kostenlos und quelloffen (GPL-3.0)

## Was es kann

- 🎮 **80 klassische Spiele an einem Ort**: Quake, Call of Duty, Unreal Tournament, Counter-Strike, Medal of Honor, Battlefield, Tribes und viele mehr.
- ⚡ **Doppelklick zum Spielen**: es findet deine installierten Spiele und startet sie gleich verbunden, mit der richtigen Mod und dem richtigen Passwort.
- 📶 **Ping, Flux und Paketverlust** gemessen wie im originalen ASE, dazu die Version jedes Servers.
- 🔎 **Schnell den richtigen Server finden**: filtern nach Spielmodus, Mod, Karte, Version, Ping oder Land; leere, volle oder passwortgeschützte Server ausblenden; eigene Filter anlegen.
- ⭐ **Favoriten, Freunde und zuletzt besuchte Server**, damit der Server deines Clans immer nur einen Klick entfernt ist.
- 🔔 **Lass dich benachrichtigen**, wenn auf einem vollen Server ein Platz frei wird, wenn sich ein Match füllt oder wenn ein Freund online kommt.
- 👥 **Jetzt im Spiel**: eine Liste aller belegten Server in all deinen Spielen.
- 🏠 **LAN-Partys**: findet Server in deinem lokalen Netzwerk.
- 🎨 **Sechs Designs**, von Classic bis Synthwave und GoldSrc, in sieben Sprachen: Englisch, Französisch, Spanisch, Portugiesisch, Italienisch, Deutsch und Russisch.
- 🛠 **Für Server-Admins**: eine RCON-Konsole und ein Kommandozeilenwerkzeug.

Alle Funktionen im Detail: die [Bedienungsanleitung](docs/user-guide.md) (auf Englisch).

## Unterstützte Spiele

Gruppiert danach, wie sie mit ihren Servern sprechen (ihr Abfrageprotokoll).

| Protokoll | Spiele |
|---|---|
| Quake 3 | Quake 3: Arena, Urban Terror, OpenArena, Xonotic, Warsow, Warfork, Tremulous, Smokin' Guns, Wolfenstein: Enemy Territory, Return to Castle Wolfenstein, Call of Duty, Call of Duty: United Offensive, Call of Duty 2, Call of Duty 4: Modern Warfare, Star Wars Jedi Knight II: Jedi Outcast, Star Wars Jedi Knight: Jedi Academy, Star Trek: Voyager Elite Force, Soldier of Fortune II |
| Quake 2 und QuakeWorld | Quake 2, Kingpin, QuakeWorld |
| Doom 3 | DOOM 3, Quake 4 |
| Zandronum | Zandronum (Doom) |
| GameSpy | Unreal Tournament, Unreal, Deus Ex, Rune, Wheel of Time, Mobile Forces, Serious Sam: The First und Second Encounter, No One Lives Forever 1 und 2, Heretic II, Gore, Drakan, Turok 2, Global Operations, Vietcong, Star Trek: Bridge Commander, Medal of Honor: Allied Assault (+ Spearhead, Breakthrough), Battlefield 1942, SWAT 4, Soldier of Fortune |
| GameSpy v2 und v3 | Battlefield 2, Battlefield Vietnam, Halo: Combat Evolved, Halo: Custom Edition, Tribes: Vengeance, F.E.A.R., Star Wars: Battlefront |
| Unreal Engine 2 | Unreal Tournament 2004, Unreal Tournament 2003 |
| Tribes | Starsiege: Tribes, Tribes 2 |
| Valve (Steam)\* | Counter-Strike 1.6, Condition Zero, Team Fortress Classic, Day of Defeat, Half-Life, Opposing Force, Deathmatch Classic, Ricochet, Counter-Strike: Source, Day of Defeat: Source, Half-Life 2: Deathmatch, Team Fortress 2, Left 4 Dead, Left 4 Dead 2, Garry's Mod |
| Eigene | Multi Theft Auto: San Andreas, Savage: The Battle for Newerth, Rainbow Six 3: Raven Shield, Ghost Recon und The Sum of All Fears (+ demos) |

\* Braucht einen kostenlosen [Steam-Web-API-Schlüssel](docs/user-guide.md#steam-games) für die Serverliste.

Welche Master-Server jedes Spiel nutzt: [Spiele und Master](docs/games-and-masters.md).
Dein Spiel fehlt? [Eröffne ein Issue](https://github.com/OpenGamePlus/OpenASE-Plus/issues).

> [!IMPORTANT]
> **Es sind keine Spiele enthalten.** OpenASE-Plus listet Server auf und startet die Spiele, die du bereits installiert hast.

## Erste Schritte

1. **Lade** das Paket für dein System von den [Releases](https://github.com/OpenGamePlus/OpenASE-Plus/releases/latest) herunter: eine `.zip` für Windows, ein AppImage oder `.tar.gz` für Linux, eine `.zip` für macOS ([Details](docs/user-guide.md#install)).
2. **Starte es.** Windows und macOS warnen eventuell, weil die App nicht signiert ist; unter Windows klicke auf **Weitere Informationen → Trotzdem ausführen**, unter macOS klicke mit rechts auf die App und wähle **Öffnen**.
3. **Lass es deine Spiele finden**, mit der automatischen Suche beim ersten Start.
4. **Wähle ein Spiel, drücke `F5`, doppelklicke auf einen Server.** Das war's.

OpenASE-Plus sagt dir Bescheid, wenn eine neue Version erscheint.

> [!NOTE]
> **Erste Version.** Der Beitreten-Befehl vieler neuer Spiele wurde noch nicht mit installiertem Spiel getestet. Melde bitte, was nicht klappt, in den [Issues](https://github.com/OpenGamePlus/OpenASE-Plus/issues).

## Mehr erfahren

Die verlinkte Dokumentation ist auf Englisch.

- 📖 [Bedienungsanleitung](docs/user-guide.md): alle Funktionen, Tastenkürzel, Steam-Spiele und wo deine Einstellungen liegen.
- 🛰 [Spiele und Master](docs/games-and-masters.md): die Master-Server jedes Spiels.
- 💻 [Kommandozeile (`openase-cli`)](docs/cli.md): dieselben Abfragen ohne Fenster, mit JSON-Ausgabe.
- 🛠 [Selbst bauen und mitmachen](docs/README.md): technische Dokumentation, [Bauen](docs/development/building.md), [ein Spiel hinzufügen](docs/development/adding-a-game.md), [CONTRIBUTING.md](CONTRIBUTING.md).
- 📜 [Was sich geändert hat](CHANGELOG.md) und [was kommt](ROADMAP.md).

## Über das Projekt

OpenASE-Plus ist ein Fork von [OpenASE](https://github.com/tsautier/OpenASE) von Thomas Sautier und gehört zur [OpenGamePlus](https://github.com/OpenGamePlus)-Familie. Zusätzlich zu OpenASE bringt es die Funktionen von JASE mit, einem früheren Browser desselben Maintainers: mehr Spiele, Benachrichtigungen, Links, RCON, Designs, Flaggen und Kartenbilder. Wie der Rest der Familie wird es mit KI-Programmierassistenten entwickelt. Es ist neuer Code: Es verwendet, verändert und verbreitet nichts aus dem originalen Yahoo-Programm und steht in keiner Verbindung zu Yahoo oder den ursprünglichen Autoren von The All-Seeing Eye.

Danke an die Communitys, die diese Spiele am Leben halten: 333networks, OpenSpy, dpmaster, jkhub, 1fxmod, TribesNext, OpenRVS, Savage DRX und viele mehr.

## Lizenz

Copyright (C) 2026 Thomas Sautier (OpenASE)  
Copyright (C) 2026 OpenGamePlus contributors (OpenASE-Plus)

OpenASE-Plus ist freie Software unter der GNU General Public License, Version 3 oder später; siehe [LICENSE](LICENSE). Spielenamen sind Marken ihrer Inhaber. IP-Geolokalisierung von [DB-IP](https://db-ip.com) (CC BY 4.0), zur Laufzeit heruntergeladen. Flaggen von [flag-icons](https://github.com/lipis/flag-icons) (MIT, `rust/assets/flags-LICENSE.txt`). Mit diesem Projekt werden keine Dateien des originalen ASE verbreitet.
