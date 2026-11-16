[English](README.md) | [Français](README.fr.md) | **Español** | [Português](README.pt.md) | [Italiano](README.it.md) | [Deutsch](README.de.md) | [Русский](README.ru.md)

# OpenASE-Plus

<p align="center"><img src="docs/banner.svg" alt="OpenASE-Plus" width="100%"></p>

[![Release](https://img.shields.io/github/v/release/OpenGamePlus/OpenASE-Plus)](https://github.com/OpenGamePlus/OpenASE-Plus/releases/latest)
[![License: GPL-3.0](https://img.shields.io/badge/license-GPL--3.0-blue)](LICENSE)
![Platforms: Windows | Linux | macOS](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey)
![Games: 80](https://img.shields.io/badge/games-80-19f5e1)
[![Fork of tsautier/OpenASE](https://img.shields.io/badge/fork%20of-tsautier%2FOpenASE-555)](https://github.com/tsautier/OpenASE)

**The All-Seeing Eye sigue mirando.** Encuentra un servidor para los juegos clásicos que todavía te gustan y entra con un doble clic.

OpenASE-Plus es un navegador de servidores gratuito para Windows, Linux y macOS, al estilo del antiguo *All-Seeing Eye*. Elige un juego, mira quién está jugando ahora mismo y entra: sin comandos de consola ni copiar IPs.

<p align="center"><img src="docs/screenshots/classic.png" alt="OpenASE-Plus, Classic theme: Quake 3 servers sorted by players, with the players and rules of the selected server"></p>
<table>
  <tr>
    <td><img src="docs/screenshots/dark.png" alt="OpenASE-Plus, Dark theme"></td>
    <td><img src="docs/screenshots/goldsrc.png" alt="OpenASE-Plus, GoldSrc theme"></td>
  </tr>
</table>
<p align="center"><sub>Servidores de Quake 3 con los temas Classic, Dark y GoldSrc.</sub></p>

**[⬇ Descargar](https://github.com/OpenGamePlus/OpenASE-Plus/releases/latest)** · gratis y de código abierto (GPL-3.0)

## Qué hace

- 🎮 **80 juegos clásicos en un solo lugar**: Quake, Call of Duty, Unreal Tournament, Counter-Strike, Medal of Honor, Battlefield, Tribes y muchos más.
- ⚡ **Doble clic para jugar**: encuentra tus juegos instalados y los abre ya conectados, con el mod y la contraseña correctos.
- 📶 **Ping, Flux y pérdida de paquetes** medidos como en el ASE original, además de la versión de cada servidor.
- 🔎 **Encuentra el servidor adecuado rápido**: filtra por tipo de juego, mod, mapa, versión, ping o país; oculta servidores vacíos, llenos o con contraseña; crea tus propios filtros.
- ⭐ **Favoritos, amigos y servidores recientes**, para que el servidor de tu clan esté siempre a un clic.
- 🔔 **Recibe avisos** cuando un servidor lleno tiene un hueco libre, cuando una partida se está llenando o cuando un amigo se conecta.
- 👥 **Jugando ahora**: una lista con todos los servidores con gente de todos tus juegos.
- 🏠 **LAN parties**: encuentra servidores en tu red local.
- 🎨 **Seis temas**, de Classic a Synthwave y GoldSrc, en siete idiomas: inglés, francés, español, portugués, italiano, alemán y ruso.
- 🛠 **Para administradores de servidores**: una consola RCON y una herramienta de línea de comandos.

Todas las funciones en detalle: la [guía de uso](docs/user-guide.md) (en inglés).

## Juegos compatibles

Agrupados según la forma en que hablan con sus servidores (su protocolo de consulta).

| Protocolo | Juegos |
|---|---|
| Quake 3 | Quake 3: Arena, Urban Terror, OpenArena, Xonotic, Warsow, Warfork, Tremulous, Smokin' Guns, Wolfenstein: Enemy Territory, Return to Castle Wolfenstein, Call of Duty, Call of Duty: United Offensive, Call of Duty 2, Call of Duty 4: Modern Warfare, Star Wars Jedi Knight II: Jedi Outcast, Star Wars Jedi Knight: Jedi Academy, Star Trek: Voyager Elite Force, Soldier of Fortune II |
| Quake 2 y QuakeWorld | Quake 2, Kingpin, QuakeWorld |
| Doom 3 | DOOM 3, Quake 4 |
| Zandronum | Zandronum (Doom) |
| GameSpy | Unreal Tournament, Unreal, Deus Ex, Rune, Wheel of Time, Mobile Forces, Serious Sam: The First y Second Encounter, No One Lives Forever 1 y 2, Heretic II, Gore, Drakan, Turok 2, Global Operations, Vietcong, Star Trek: Bridge Commander, Medal of Honor: Allied Assault (+ Spearhead, Breakthrough), Battlefield 1942, SWAT 4, Soldier of Fortune |
| GameSpy v2 y v3 | Battlefield 2, Battlefield Vietnam, Halo: Combat Evolved, Halo: Custom Edition, Tribes: Vengeance, F.E.A.R., Star Wars: Battlefront |
| Unreal Engine 2 | Unreal Tournament 2004, Unreal Tournament 2003 |
| Tribes | Starsiege: Tribes, Tribes 2 |
| Valve (Steam)\* | Counter-Strike 1.6, Condition Zero, Team Fortress Classic, Day of Defeat, Half-Life, Opposing Force, Deathmatch Classic, Ricochet, Counter-Strike: Source, Day of Defeat: Source, Half-Life 2: Deathmatch, Team Fortress 2, Left 4 Dead, Left 4 Dead 2, Garry's Mod |
| Propios | Multi Theft Auto: San Andreas, Savage: The Battle for Newerth, Rainbow Six 3: Raven Shield, Ghost Recon y The Sum of All Fears (+ demos) |

\* Necesita una [clave de la Steam Web API](docs/user-guide.md#steam-games) gratuita para la lista de servidores.

Qué servidores master usa cada juego: [juegos y masters](docs/games-and-masters.md).
¿Falta tu juego? [Abre un issue](https://github.com/OpenGamePlus/OpenASE-Plus/issues).

> [!IMPORTANT]
> **No incluye juegos.** OpenASE-Plus lista servidores y abre los juegos que ya tienes instalados.

## Primeros pasos

1. **Descarga** el paquete para tu sistema desde [Releases](https://github.com/OpenGamePlus/OpenASE-Plus/releases/latest): un `.zip` para Windows, un AppImage o `.tar.gz` para Linux, un `.zip` para macOS ([detalles](docs/user-guide.md#install)).
2. **Ábrelo.** Windows y macOS pueden avisarte porque la app no está firmada; en Windows haz clic en **Más información → Ejecutar de todas formas**, en macOS haz clic derecho en la app y elige **Abrir**.
3. **Deja que encuentre tus juegos** con la búsqueda automática en el primer inicio.
4. **Elige un juego, pulsa `F5` y haz doble clic en un servidor.** Eso es todo.

OpenASE-Plus te avisa cuando sale una versión nueva.

> [!NOTE]
> **Primera versión.** El comando para unirse de muchos de los juegos nuevos no se ha probado con el juego instalado. Cuéntanos lo que falle en los [issues](https://github.com/OpenGamePlus/OpenASE-Plus/issues).

## Más información

La documentación enlazada está en inglés.

- 📖 [Guía de uso](docs/user-guide.md): todas las funciones, atajos, juegos de Steam y dónde se guardan tus ajustes.
- 🛰 [Juegos y masters](docs/games-and-masters.md): los servidores master de cada juego.
- 💻 [Línea de comandos (`openase-cli`)](docs/cli.md): las mismas consultas sin ventana, con salida JSON.
- 🛠 [Compilar y contribuir](docs/README.md): documentación técnica, [compilación](docs/development/building.md), [agregar un juego](docs/development/adding-a-game.md), [CONTRIBUTING.md](CONTRIBUTING.md).
- 📜 [Qué cambió](CHANGELOG.md) y [qué viene](ROADMAP.md).

## Acerca de

OpenASE-Plus es un fork de [OpenASE](https://github.com/tsautier/OpenASE) de Thomas Sautier y forma parte de la familia [OpenGamePlus](https://github.com/OpenGamePlus). Sobre OpenASE suma las funciones de JASE, un navegador anterior del mismo mantenedor: más juegos, alertas, enlaces, RCON, temas, banderas e imágenes de mapas. Como el resto de la familia, se desarrolla con asistentes de programación con IA. Es código nuevo: no usa, modifica ni distribuye nada del programa original de Yahoo, y no tiene relación con Yahoo ni con los autores originales de The All-Seeing Eye.

Gracias a las comunidades que mantienen vivos estos juegos: 333networks, OpenSpy, dpmaster, jkhub, 1fxmod, TribesNext, OpenRVS, Savage DRX y muchas más.

## Licencia

Copyright (C) 2026 Thomas Sautier (OpenASE)  
Copyright (C) 2026 OpenGamePlus contributors (OpenASE-Plus)

OpenASE-Plus es software libre bajo la Licencia Pública General de GNU, versión 3 o posterior; ver [LICENSE](LICENSE). Los nombres de los juegos son marcas de sus dueños. Geolocalización IP de [DB-IP](https://db-ip.com) (CC BY 4.0), descargada al usarse. Banderas de [flag-icons](https://github.com/lipis/flag-icons) (MIT, `rust/assets/flags-LICENSE.txt`). Este proyecto no distribuye ningún archivo del ASE original.
