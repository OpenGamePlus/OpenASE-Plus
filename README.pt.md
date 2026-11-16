[English](README.md) | [Français](README.fr.md) | [Español](README.es.md) | **Português** | [Italiano](README.it.md) | [Deutsch](README.de.md) | [Русский](README.ru.md)

# OpenASE-Plus

<p align="center"><img src="docs/banner.svg" alt="OpenASE-Plus" width="100%"></p>

[![Release](https://img.shields.io/github/v/release/OpenGamePlus/OpenASE-Plus)](https://github.com/OpenGamePlus/OpenASE-Plus/releases/latest)
[![License: GPL-3.0](https://img.shields.io/badge/license-GPL--3.0-blue)](LICENSE)
![Platforms: Windows | Linux | macOS](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey)
![Games: 80](https://img.shields.io/badge/games-80-19f5e1)
[![Fork of tsautier/OpenASE](https://img.shields.io/badge/fork%20of-tsautier%2FOpenASE-555)](https://github.com/tsautier/OpenASE)

**The All-Seeing Eye continua de olho.** Encontre um servidor para os jogos clássicos que você ainda ama e entre com um clique duplo.

OpenASE-Plus é um navegador de servidores gratuito para Windows, Linux e macOS, no espírito do antigo *All-Seeing Eye*. Escolha um jogo, veja quem está jogando agora e entre: sem comandos de console, sem copiar IPs.

<p align="center"><img src="docs/screenshots/classic.png" alt="OpenASE-Plus, Classic theme: Quake 3 servers sorted by players, with the players and rules of the selected server"></p>
<table>
  <tr>
    <td><img src="docs/screenshots/dark.png" alt="OpenASE-Plus, Dark theme"></td>
    <td><img src="docs/screenshots/goldsrc.png" alt="OpenASE-Plus, GoldSrc theme"></td>
  </tr>
</table>
<p align="center"><sub>Servidores de Quake 3 nos temas Classic, Dark e GoldSrc.</sub></p>

**[⬇ Baixar](https://github.com/OpenGamePlus/OpenASE-Plus/releases/latest)** · gratuito e de código aberto (GPL-3.0)

## O que ele faz

- 🎮 **80 jogos clássicos em um só lugar**: Quake, Call of Duty, Unreal Tournament, Counter-Strike, Medal of Honor, Battlefield, Tribes e muitos outros.
- ⚡ **Clique duplo para jogar**: ele encontra seus jogos instalados e os abre já conectados, com o mod e a senha certos.
- 📶 **Ping, Flux e perda de pacotes** medidos como no ASE original, além da versão de cada servidor.
- 🔎 **Encontre rápido o servidor certo**: filtre por tipo de jogo, mod, mapa, versão, ping ou país; esconda servidores vazios, cheios ou com senha; crie seus próprios filtros.
- ⭐ **Favoritos, amigos e servidores recentes**, para o servidor do seu clã estar sempre a um clique.
- 🔔 **Receba avisos** quando abrir uma vaga num servidor cheio, quando uma partida estiver enchendo ou quando um amigo ficar online.
- 👥 **Jogando agora**: uma lista com todos os servidores com gente em todos os seus jogos.
- 🏠 **LAN parties**: encontra servidores na sua rede local.
- 🎨 **Seis temas**, do Classic ao Synthwave e GoldSrc, em sete idiomas: inglês, francês, espanhol, português, italiano, alemão e russo.
- 🛠 **Para administradores de servidores**: um console RCON e uma ferramenta de linha de comando.

Todos os recursos em detalhe: o [guia do usuário](docs/user-guide.md) (em inglês).

## Jogos suportados

Agrupados pela forma como falam com seus servidores (seu protocolo de consulta).

| Protocolo | Jogos |
|---|---|
| Quake 3 | Quake 3: Arena, Urban Terror, OpenArena, Xonotic, Warsow, Warfork, Tremulous, Smokin' Guns, Wolfenstein: Enemy Territory, Return to Castle Wolfenstein, Call of Duty, Call of Duty: United Offensive, Call of Duty 2, Call of Duty 4: Modern Warfare, Star Wars Jedi Knight II: Jedi Outcast, Star Wars Jedi Knight: Jedi Academy, Star Trek: Voyager Elite Force, Soldier of Fortune II |
| Quake 2 e QuakeWorld | Quake 2, Kingpin, QuakeWorld |
| Doom 3 | DOOM 3, Quake 4 |
| Zandronum | Zandronum (Doom) |
| GameSpy | Unreal Tournament, Unreal, Deus Ex, Rune, Wheel of Time, Mobile Forces, Serious Sam: The First e Second Encounter, No One Lives Forever 1 e 2, Heretic II, Gore, Drakan, Turok 2, Global Operations, Vietcong, Star Trek: Bridge Commander, Medal of Honor: Allied Assault (+ Spearhead, Breakthrough), Battlefield 1942, SWAT 4, Soldier of Fortune |
| GameSpy v2 e v3 | Battlefield 2, Battlefield Vietnam, Halo: Combat Evolved, Halo: Custom Edition, Tribes: Vengeance, F.E.A.R., Star Wars: Battlefront |
| Unreal Engine 2 | Unreal Tournament 2004, Unreal Tournament 2003 |
| Tribes | Starsiege: Tribes, Tribes 2 |
| Valve (Steam)\* | Counter-Strike 1.6, Condition Zero, Team Fortress Classic, Day of Defeat, Half-Life, Opposing Force, Deathmatch Classic, Ricochet, Counter-Strike: Source, Day of Defeat: Source, Half-Life 2: Deathmatch, Team Fortress 2, Left 4 Dead, Left 4 Dead 2, Garry's Mod |
| Próprios | Multi Theft Auto: San Andreas, Savage: The Battle for Newerth, Rainbow Six 3: Raven Shield, Ghost Recon e The Sum of All Fears (+ demos) |

\* Precisa de uma [chave da Steam Web API](docs/user-guide.md#steam-games) gratuita para a lista de servidores.

Quais servidores master cada jogo usa: [jogos e masters](docs/games-and-masters.md).
Falta o seu jogo? [Abra uma issue](https://github.com/OpenGamePlus/OpenASE-Plus/issues).

> [!IMPORTANT]
> **Nenhum jogo incluído.** OpenASE-Plus lista servidores e abre os jogos que você já tem instalados.

## Primeiros passos

1. **Baixe** o pacote do seu sistema em [Releases](https://github.com/OpenGamePlus/OpenASE-Plus/releases/latest): um `.zip` para Windows, um AppImage ou `.tar.gz` para Linux, um `.zip` para macOS ([detalhes](docs/user-guide.md#install)).
2. **Abra.** Windows e macOS podem avisar porque o app não é assinado; no Windows clique em **Mais informações → Executar assim mesmo**, no macOS clique com o botão direito no app e escolha **Abrir**.
3. **Deixe-o encontrar seus jogos** com a busca automática na primeira vez.
4. **Escolha um jogo, aperte `F5` e dê um clique duplo num servidor.** Só isso.

OpenASE-Plus avisa quando sai uma versão nova.

> [!NOTE]
> **Primeira versão.** O comando para entrar de muitos dos jogos novos não foi testado com o jogo instalado. Conte o que der errado nas [issues](https://github.com/OpenGamePlus/OpenASE-Plus/issues).

## Saiba mais

A documentação dos links está em inglês.

- 📖 [Guia do usuário](docs/user-guide.md): todos os recursos, atalhos, jogos da Steam e onde ficam suas configurações.
- 🛰 [Jogos e masters](docs/games-and-masters.md): os servidores master de cada jogo.
- 💻 [Linha de comando (`openase-cli`)](docs/cli.md): as mesmas consultas sem janela, com saída JSON.
- 🛠 [Compilar e contribuir](docs/README.md): documentação técnica, [compilação](docs/development/building.md), [adicionar um jogo](docs/development/adding-a-game.md), [CONTRIBUTING.md](CONTRIBUTING.md).
- 📜 [O que mudou](CHANGELOG.md) e [o que vem por aí](ROADMAP.md).

## Sobre

OpenASE-Plus é um fork do [OpenASE](https://github.com/tsautier/OpenASE) de Thomas Sautier e faz parte da família [OpenGamePlus](https://github.com/OpenGamePlus). Além do OpenASE, traz os recursos do JASE, um navegador anterior do mesmo mantenedor: mais jogos, alertas, links, RCON, temas, bandeiras e imagens de mapas. Como o resto da família, é desenvolvido com assistentes de programação com IA. É código novo: não usa, modifica nem distribui nada do programa original da Yahoo, e não tem relação com a Yahoo nem com os autores originais do The All-Seeing Eye.

Obrigado às comunidades que mantêm esses jogos vivos: 333networks, OpenSpy, dpmaster, jkhub, 1fxmod, TribesNext, OpenRVS, Savage DRX e muitas outras.

## Licença

Copyright (C) 2026 Thomas Sautier (OpenASE)  
Copyright (C) 2026 OpenGamePlus contributors (OpenASE-Plus)

OpenASE-Plus é software livre sob a Licença Pública Geral GNU, versão 3 ou posterior; veja [LICENSE](LICENSE). Os nomes dos jogos são marcas de seus donos. Geolocalização de IP por [DB-IP](https://db-ip.com) (CC BY 4.0), baixada durante o uso. Bandeiras de [flag-icons](https://github.com/lipis/flag-icons) (MIT, `rust/assets/flags-LICENSE.txt`). Nenhum arquivo do ASE original é distribuído com este projeto.
