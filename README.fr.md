[English](README.md) | **Français** | [Español](README.es.md) | [Português](README.pt.md) | [Italiano](README.it.md) | [Deutsch](README.de.md) | [Русский](README.ru.md)

# OpenASE-Plus

<p align="center"><img src="docs/banner.svg" alt="OpenASE-Plus" width="100%"></p>

[![Release](https://img.shields.io/github/v/release/OpenGamePlus/OpenASE-Plus)](https://github.com/OpenGamePlus/OpenASE-Plus/releases/latest)
[![License: GPL-3.0](https://img.shields.io/badge/license-GPL--3.0-blue)](LICENSE)
![Platforms: Windows | Linux | macOS](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey)
![Games: 80](https://img.shields.io/badge/games-80-19f5e1)
[![Fork of tsautier/OpenASE](https://img.shields.io/badge/fork%20of-tsautier%2FOpenASE-555)](https://github.com/tsautier/OpenASE)

**The All-Seeing Eye veille toujours.** Trouvez un serveur pour les jeux classiques que vous aimez encore, et rejoignez-le d'un double-clic.

OpenASE-Plus est un navigateur de serveurs gratuit pour Windows, Linux et macOS, dans l'esprit de l'ancien *All-Seeing Eye*. Choisissez un jeu, voyez qui joue en ce moment, et lancez-vous : pas de commandes console, pas d'IP à copier.

<p align="center"><img src="docs/screenshots/classic.png" alt="OpenASE-Plus, Classic theme: Quake 3 servers sorted by players, with the players and rules of the selected server"></p>
<table>
  <tr>
    <td><img src="docs/screenshots/dark.png" alt="OpenASE-Plus, Dark theme"></td>
    <td><img src="docs/screenshots/goldsrc.png" alt="OpenASE-Plus, GoldSrc theme"></td>
  </tr>
</table>
<p align="center"><sub>Des serveurs Quake 3 avec les thèmes Classic, Dark et GoldSrc.</sub></p>

**[⬇ Télécharger](https://github.com/OpenGamePlus/OpenASE-Plus/releases/latest)** · gratuit et open source (GPL-3.0)

## Ce qu'il fait

- 🎮 **80 jeux classiques au même endroit** : Quake, Call of Duty, Unreal Tournament, Counter-Strike, Medal of Honor, Battlefield, Tribes et bien d'autres.
- ⚡ **Double-clic pour jouer** : il trouve vos jeux installés et les lance déjà connectés, avec le bon mod et le bon mot de passe.
- 📶 **Ping, Flux et perte de paquets** mesurés comme dans l'ASE d'origine, plus la version de chaque serveur.
- 🔎 **Trouvez vite le bon serveur** : filtrez par type de jeu, mod, carte, version, ping ou pays ; masquez les serveurs vides, pleins ou protégés ; créez vos propres filtres.
- ⭐ **Favoris, amis et serveurs récents**, pour que le serveur de votre clan soit toujours à un clic.
- 🔔 **Soyez prévenu** quand une place se libère sur un serveur plein, quand une partie se remplit ou quand un ami se connecte.
- 👥 **En jeu maintenant** : une liste de tous les serveurs occupés de tous vos jeux.
- 🏠 **LAN parties** : il trouve les serveurs de votre réseau local.
- 🎨 **Six thèmes**, de Classic à Synthwave et GoldSrc, en sept langues : anglais, français, espagnol, portugais, italien, allemand et russe.
- 🛠 **Pour les administrateurs de serveurs** : une console RCON et un outil en ligne de commande.

Toutes les fonctions en détail : le [guide d'utilisation](docs/user-guide.md) (en anglais).

## Jeux pris en charge

Regroupés selon la façon dont ils parlent à leurs serveurs (leur protocole de requête).

| Protocole | Jeux |
|---|---|
| Quake 3 | Quake 3: Arena, Urban Terror, OpenArena, Xonotic, Warsow, Warfork, Tremulous, Smokin' Guns, Wolfenstein: Enemy Territory, Return to Castle Wolfenstein, Call of Duty, Call of Duty: United Offensive, Call of Duty 2, Call of Duty 4: Modern Warfare, Star Wars Jedi Knight II: Jedi Outcast, Star Wars Jedi Knight: Jedi Academy, Star Trek: Voyager Elite Force, Soldier of Fortune II |
| Quake 2 et QuakeWorld | Quake 2, Kingpin, QuakeWorld |
| Doom 3 | DOOM 3, Quake 4 |
| Zandronum | Zandronum (Doom) |
| GameSpy | Unreal Tournament, Unreal, Deus Ex, Rune, Wheel of Time, Mobile Forces, Serious Sam: The First et Second Encounter, No One Lives Forever 1 et 2, Heretic II, Gore, Drakan, Turok 2, Global Operations, Vietcong, Star Trek: Bridge Commander, Medal of Honor: Allied Assault (+ Spearhead, Breakthrough), Battlefield 1942, SWAT 4, Soldier of Fortune |
| GameSpy v2 et v3 | Battlefield 2, Battlefield Vietnam, Halo: Combat Evolved, Halo: Custom Edition, Tribes: Vengeance, F.E.A.R., Star Wars: Battlefront |
| Unreal Engine 2 | Unreal Tournament 2004, Unreal Tournament 2003 |
| Tribes | Starsiege: Tribes, Tribes 2 |
| Valve (Steam)\* | Counter-Strike 1.6, Condition Zero, Team Fortress Classic, Day of Defeat, Half-Life, Opposing Force, Deathmatch Classic, Ricochet, Counter-Strike: Source, Day of Defeat: Source, Half-Life 2: Deathmatch, Team Fortress 2, Left 4 Dead, Left 4 Dead 2, Garry's Mod |
| Les leurs | Multi Theft Auto: San Andreas, Savage: The Battle for Newerth, Rainbow Six 3: Raven Shield, Ghost Recon et The Sum of All Fears (+ demos) |

\* Nécessite une [clé Steam Web API](docs/user-guide.md#steam-games) gratuite pour la liste des serveurs.

Les serveurs maîtres de chaque jeu : [jeux et masters](docs/games-and-masters.md).
Votre jeu manque ? [Ouvrez une issue](https://github.com/OpenGamePlus/OpenASE-Plus/issues).

> [!IMPORTANT]
> **Aucun jeu n'est inclus.** OpenASE-Plus liste les serveurs et lance les jeux que vous avez déjà installés.

## Pour commencer

1. **Téléchargez** le paquet de votre système depuis les [Releases](https://github.com/OpenGamePlus/OpenASE-Plus/releases/latest) : un `.zip` pour Windows, un AppImage ou un `.tar.gz` pour Linux, un `.zip` pour macOS ([détails](docs/user-guide.md#install)).
2. **Lancez-le.** Windows et macOS peuvent vous avertir car l'application n'est pas signée ; sous Windows cliquez sur **Informations complémentaires → Exécuter quand même**, sous macOS faites un clic droit sur l'application et choisissez **Ouvrir**.
3. **Laissez-le trouver vos jeux** avec la recherche automatique au premier démarrage.
4. **Choisissez un jeu, appuyez sur `F5`, double-cliquez sur un serveur.** C'est tout.

OpenASE-Plus vous prévient quand une nouvelle version sort.

> [!NOTE]
> **Première version.** La commande pour rejoindre de nombreux nouveaux jeux n'a pas été testée avec le jeu installé. Signalez ce qui ne marche pas dans les [issues](https://github.com/OpenGamePlus/OpenASE-Plus/issues).

## En savoir plus

La documentation liée est en anglais.

- 📖 [Guide d'utilisation](docs/user-guide.md) : toutes les fonctions, les raccourcis, les jeux Steam et l'emplacement de vos réglages.
- 🛰 [Jeux et masters](docs/games-and-masters.md) : les serveurs maîtres de chaque jeu.
- 💻 [Ligne de commande (`openase-cli`)](docs/cli.md) : les mêmes requêtes sans fenêtre, avec sortie JSON.
- 🛠 [Compiler et contribuer](docs/README.md) : documentation technique, [compilation](docs/development/building.md), [ajouter un jeu](docs/development/adding-a-game.md), [CONTRIBUTING.md](CONTRIBUTING.md).
- 📜 [Ce qui a changé](CHANGELOG.md) et [la suite](ROADMAP.md).

## À propos

OpenASE-Plus est un fork d'[OpenASE](https://github.com/tsautier/OpenASE) de Thomas Sautier et fait partie de la famille [OpenGamePlus](https://github.com/OpenGamePlus). En plus d'OpenASE, il reprend les fonctions de JASE, un navigateur antérieur du même mainteneur : plus de jeux, alertes, liens, RCON, thèmes, drapeaux et images de cartes. Comme le reste de la famille, il est développé avec des assistants de programmation IA. C'est du code nouveau : il n'utilise, ne modifie ni ne distribue rien du programme Yahoo d'origine, et n'est affilié ni à Yahoo ni aux auteurs d'origine de The All-Seeing Eye.

Merci aux communautés qui font vivre ces jeux : 333networks, OpenSpy, dpmaster, jkhub, 1fxmod, TribesNext, OpenRVS, Savage DRX et bien d'autres.

## Licence

Copyright (C) 2026 Thomas Sautier (OpenASE)  
Copyright (C) 2026 OpenGamePlus contributors (OpenASE-Plus)

OpenASE-Plus est un logiciel libre sous licence publique générale GNU, version 3 ou ultérieure ; voir [LICENSE](LICENSE). Les noms des jeux sont des marques de leurs propriétaires. Géolocalisation IP par [DB-IP](https://db-ip.com) (CC BY 4.0), téléchargée à l'exécution. Drapeaux de [flag-icons](https://github.com/lipis/flag-icons) (MIT, `rust/assets/flags-LICENSE.txt`). Aucun fichier de l'ASE d'origine n'est distribué avec ce projet.
