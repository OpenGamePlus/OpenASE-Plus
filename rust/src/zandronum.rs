//! Zandronum (multiplayer Doom) launcher protocol: master list and server query.
//!
//! Every packet is Huffman-compressed with a fixed tree, or sent raw behind a
//! `0xFF` byte (servers and the master accept raw requests). Integers are
//! little-endian, strings NUL-terminated. Layouts checked against Zandronum's
//! `sv_master.cpp` and live servers (ported from JASE, 2026-10-27).
//!
//! A server ignores a second query from the same address for about 10 s
//! (`sv_queryignoretime`), so one query gives one ping sample and an
//! "ignoring" answer is retried once after a pause.
//!
//! The Huffman tree comes from Zandronum's `huffman.cpp`:
//! Copyright 2009 Timothy Landers. Permission is hereby granted, free of
//! charge, to any person obtaining a copy of this software and associated
//! documentation files (the "Software"), to deal in the Software without
//! restriction, including without limitation the rights to use, copy, modify,
//! merge, publish, distribute, sublicense, and/or sell copies of the Software,
//! and to permit persons to whom the Software is furnished to do so, subject
//! to the following conditions: The above copyright notice and this permission
//! notice shall be included in all copies or substantial portions of the
//! Software. THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
//! EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
//! MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO
//! EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES
//! OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
//! ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
//! DEALINGS IN THE SOFTWARE.

use std::collections::HashSet;
use std::io;
use std::net::{ToSocketAddrs, UdpSocket};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use crate::catalog::Game;
use crate::model::{Player, Server};
use crate::quake3::{MASTER_IDLE_TIMEOUT, ParseError, latin1, parse_address};

pub const MASTER_CHALLENGE: i32 = 5_660_028;
const MASTER_PROTOCOL: u16 = 2;
pub const SERVER_CHALLENGE: i32 = 199;
pub const SERVER_ACCEPTED: i32 = 5_660_023;
/// The server answered another query from this address a moment ago.
pub const SERVER_IGNORING: i32 = 5_660_024;
/// Pause before asking again after `SERVER_IGNORING`.
pub const IGNORE_WAIT: Duration = Duration::from_millis(10_500);

const MSC_ENDSERVERLIST: u8 = 2;
const MSC_IPISBANNED: i32 = 3;
const MSC_REQUESTIGNORED: i32 = 4;
const MSC_WRONGVERSION: i32 = 5;
const MSC_BEGINSERVERLISTPART: i32 = 6;
const MSC_ENDSERVERLISTPART: u8 = 7;
const MSC_SERVERBLOCK: u8 = 8;

// Fields asked of a server (SQF_*), in the order they come back.
const SQF_NAME: u32 = 0x1;
const SQF_MAPNAME: u32 = 0x8;
const SQF_MAXCLIENTS: u32 = 0x10;
const SQF_MAXPLAYERS: u32 = 0x20;
const SQF_PWADS: u32 = 0x40;
const SQF_GAMETYPE: u32 = 0x80;
const SQF_GAMENAME: u32 = 0x100;
const SQF_IWAD: u32 = 0x200;
const SQF_FORCEPASSWORD: u32 = 0x400;
const SQF_FORCEJOINPASSWORD: u32 = 0x800;
const SQF_NUMPLAYERS: u32 = 0x80000;
const SQF_PLAYERDATA: u32 = 0x100000;
const QUERY_FLAGS: u32 = SQF_NAME
    | SQF_MAPNAME
    | SQF_MAXCLIENTS
    | SQF_MAXPLAYERS
    | SQF_PWADS
    | SQF_GAMETYPE
    | SQF_GAMENAME
    | SQF_IWAD
    | SQF_FORCEPASSWORD
    | SQF_FORCEJOINPASSWORD
    | SQF_NUMPLAYERS
    | SQF_PLAYERDATA;

/// Game modes (`GAMEMODE_e`), by number.
const GAME_MODES: [&str; 16] = [
    "cooperative",
    "survival",
    "invasion",
    "deathmatch",
    "teamplay",
    "duel",
    "terminator",
    "lastmanstanding",
    "teamlms",
    "possession",
    "teampossession",
    "teamgame",
    "ctf",
    "oneflagctf",
    "skulltag",
    "domination",
];
/// Modes whose player rows carry a team byte.
const TEAM_MODES: [u8; 8] = [4, 8, 10, 11, 12, 13, 14, 15];

/// Serialized tree: per branch, a description byte (bit i set = child i is a
/// leaf), then each child in order: the leaf value or its subtree.
const TREE: [u8; 511] = [
    0, 0, 0, 1, 128, 0, 0, 0, 3, 38, 34, 2, 1, 80, 3, 110, 144, 67, 0, 2, 1, 74, 3, 243, 142, 37,
    2, 3, 124, 58, 182, 0, 0, 1, 36, 0, 3, 221, 131, 3, 245, 163, 1, 35, 3, 113, 85, 0, 1, 41, 1,
    77, 3, 199, 130, 0, 1, 206, 3, 185, 153, 3, 70, 118, 0, 3, 3, 5, 0, 0, 1, 24, 0, 2, 3, 198,
    190, 63, 2, 3, 139, 186, 75, 0, 1, 44, 2, 3, 240, 218, 56, 3, 40, 39, 0, 0, 2, 2, 3, 244, 247,
    81, 65, 0, 3, 9, 125, 3, 68, 60, 0, 0, 1, 25, 3, 191, 138, 3, 86, 17, 0, 1, 23, 3, 220, 178, 2,
    3, 165, 194, 14, 1, 0, 2, 2, 0, 0, 2, 1, 208, 3, 150, 157, 181, 1, 222, 2, 3, 216, 230, 211, 0,
    2, 2, 3, 252, 141, 10, 42, 0, 2, 3, 134, 135, 104, 1, 103, 3, 187, 225, 95, 32, 0, 0, 0, 0, 0,
    0, 1, 57, 1, 61, 3, 183, 237, 0, 0, 3, 233, 234, 3, 246, 203, 2, 3, 250, 147, 79, 1, 129, 0, 1,
    7, 3, 143, 136, 1, 20, 3, 179, 148, 0, 0, 0, 3, 28, 106, 3, 101, 87, 1, 66, 0, 3, 180, 219, 3,
    227, 241, 0, 1, 26, 1, 251, 3, 229, 214, 3, 54, 69, 0, 0, 0, 0, 0, 3, 231, 212, 3, 156, 176, 3,
    93, 83, 0, 3, 96, 253, 3, 30, 13, 0, 0, 2, 3, 175, 254, 94, 3, 159, 27, 2, 1, 8, 3, 204, 226,
    78, 0, 0, 0, 3, 107, 88, 1, 31, 3, 137, 169, 2, 2, 3, 215, 145, 6, 4, 1, 127, 0, 1, 99, 3, 209,
    217, 0, 3, 213, 238, 3, 177, 170, 1, 132, 0, 0, 0, 2, 3, 22, 12, 114, 2, 2, 3, 158, 197, 97,
    45, 0, 1, 46, 1, 112, 3, 174, 249, 0, 3, 224, 102, 2, 3, 171, 151, 193, 0, 0, 0, 3, 15, 16, 3,
    2, 168, 1, 49, 3, 91, 146, 0, 1, 48, 3, 173, 29, 0, 3, 19, 126, 3, 92, 242, 0, 0, 0, 0, 0, 0,
    3, 205, 192, 2, 3, 235, 149, 255, 2, 3, 223, 184, 248, 0, 0, 3, 108, 236, 3, 111, 90, 2, 3,
    117, 115, 71, 0, 0, 3, 11, 50, 0, 3, 188, 119, 1, 122, 3, 167, 162, 1, 160, 1, 133, 3, 123, 21,
    0, 0, 2, 1, 59, 2, 3, 155, 154, 98, 43, 0, 3, 76, 51, 2, 3, 201, 116, 72, 2, 0, 2, 3, 109, 100,
    121, 2, 3, 195, 232, 18, 1, 0, 2, 0, 1, 164, 2, 3, 120, 189, 73, 0, 1, 196, 3, 239, 210, 3, 64,
    62, 89, 0, 0, 1, 33, 2, 3, 228, 161, 55, 2, 3, 84, 152, 47, 0, 0, 2, 3, 207, 172, 140, 3, 82,
    166, 0, 3, 53, 105, 1, 52, 3, 202, 200,
];

#[derive(Clone, Copy)]
enum Node {
    Leaf(u8),
    Branch(usize, usize),
}

/// The decoding tree as an arena; the root is the last node.
fn tree() -> &'static [Node] {
    static NODES: OnceLock<Vec<Node>> = OnceLock::new();
    NODES.get_or_init(|| {
        fn build(pos: &mut usize, nodes: &mut Vec<Node>) -> usize {
            let desc = TREE[*pos];
            *pos += 1;
            let mut children = [0; 2];
            for (i, child) in children.iter_mut().enumerate() {
                if desc & (1 << i) != 0 {
                    nodes.push(Node::Leaf(TREE[*pos]));
                    *pos += 1;
                    *child = nodes.len() - 1;
                } else {
                    *child = build(pos, nodes);
                }
            }
            nodes.push(Node::Branch(children[0], children[1]));
            nodes.len() - 1
        }
        let (mut pos, mut nodes) = (0, Vec::new());
        build(&mut pos, &mut nodes);
        assert_eq!(pos, TREE.len());
        nodes
    })
}

/// Decode one packet. A leading `0xFF` means raw; otherwise the first byte
/// is the count of padding bits at the end, and bits are read from the least
/// significant of each byte.
pub fn huffman_decode(data: &[u8]) -> Vec<u8> {
    match data.first() {
        None => Vec::new(),
        Some(0xff) => data[1..].to_vec(),
        Some(&padding) => {
            let nodes = tree();
            let root = nodes.len() - 1;
            let bits = ((data.len() - 1) * 8).saturating_sub(usize::from(padding));
            let mut out = Vec::new();
            let mut node = root;
            for index in 0..bits {
                let bit = (data[1 + index / 8] >> (index % 8)) & 1;
                let Node::Branch(zero, one) = nodes[node] else {
                    unreachable!("the walk restarts at the root after a leaf")
                };
                node = if bit == 0 { zero } else { one };
                if let Node::Leaf(value) = nodes[node] {
                    out.push(value);
                    node = root;
                }
            }
            out
        }
    }
}

fn raw(body: &[u8]) -> Vec<u8> {
    [&[0xff][..], body].concat()
}

/// Request for the master's list.
pub fn master_request() -> Vec<u8> {
    let mut body = MASTER_CHALLENGE.to_le_bytes().to_vec();
    body.extend_from_slice(&MASTER_PROTOCOL.to_le_bytes());
    raw(&body)
}

/// Request for a server's details and players.
pub fn query_request(time_ms: i32) -> Vec<u8> {
    let mut body = SERVER_CHALLENGE.to_le_bytes().to_vec();
    body.extend_from_slice(&QUERY_FLAGS.to_le_bytes());
    body.extend_from_slice(&(time_ms & i32::MAX).to_le_bytes());
    raw(&body)
}

fn err<T>(message: impl Into<String>) -> Result<T, ParseError> {
    Err(ParseError(message.into()))
}

/// Addresses in one master packet, and whether it was the last one.
pub fn parse_master_packet(data: &[u8]) -> Result<(Vec<String>, bool), ParseError> {
    let body = huffman_decode(data);
    let mut r = Reader::new(&body);
    match r.i32()? {
        MSC_BEGINSERVERLISTPART => {}
        MSC_IPISBANNED => return err("the master has banned this IP address"),
        MSC_REQUESTIGNORED => return err("the master asks to wait a few seconds between queries"),
        MSC_WRONGVERSION => return err("the master rejected the protocol version"),
        code => return err(format!("unexpected master answer {code}")),
    }
    r.byte()?; // packet number
    let mut found = Vec::new();
    loop {
        match r.byte()? {
            MSC_ENDSERVERLISTPART => return Ok((found, false)),
            MSC_ENDSERVERLIST => return Ok((found, true)),
            MSC_SERVERBLOCK => loop {
                let count = r.byte()?;
                if count == 0 {
                    break;
                }
                let ip = r.take(4)?;
                let host = format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3]);
                for _ in 0..count {
                    let port = r.u16()?;
                    if port != 0 {
                        found.push(format!("{host}:{port}"));
                    }
                }
            },
            block => return err(format!("unknown master block {block}")),
        }
    }
}

struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Reader { data, pos: 0 }
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], ParseError> {
        let Some(bytes) = self.data.get(self.pos..self.pos + n) else {
            return err("truncated Zandronum packet");
        };
        self.pos += n;
        Ok(bytes)
    }

    fn byte(&mut self) -> Result<u8, ParseError> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, ParseError> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }

    fn i16(&mut self) -> Result<i16, ParseError> {
        Ok(i16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }

    fn i32(&mut self) -> Result<i32, ParseError> {
        Ok(i32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn u32(&mut self) -> Result<u32, ParseError> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn string(&mut self) -> Result<String, ParseError> {
        let Some(len) = self.data[self.pos..].iter().position(|&b| b == 0) else {
            return err("unterminated string in Zandronum packet");
        };
        let text = latin1(&self.data[self.pos..self.pos + len]);
        self.pos += len + 1;
        Ok(text)
    }
}

/// Remove Doom color codes (`\x1c` + a letter, or `\x1c[name]`) and other
/// control characters.
pub fn strip_colors(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1c' {
            if chars.peek() == Some(&'[') {
                for c in chars.by_ref() {
                    if c == ']' {
                        break;
                    }
                }
            } else {
                chars.next();
            }
        } else if !c.is_control() {
            out.push(c);
        }
    }
    out
}

/// A server's answer to a query.
#[derive(Debug)]
pub enum Reply {
    Server(Box<Server>),
    /// Asked again too soon (see [`IGNORE_WAIT`]).
    Ignoring,
}

fn players(r: &mut Reader, count: u8, teams: bool) -> Result<(Vec<Player>, usize), ParseError> {
    let mut list = Vec::new();
    let mut bots = 0;
    for _ in 0..count {
        let name = strip_colors(&r.string()?);
        let score = r.i16()?;
        let ping = r.i16()?;
        let _spectator = r.byte()?;
        bots += usize::from(r.byte()? != 0);
        if teams {
            r.byte()?;
        }
        let minutes = r.byte()?;
        list.push(Player {
            name,
            score: i32::from(score),
            ping: u32::try_from(ping).ok(),
            time_secs: Some(u32::from(minutes) * 60),
        });
    }
    Ok((list, bots))
}

/// Parse a query answer. The IWAD (`DOOM2.WAD`...) is the server's mod.
pub fn parse_response(
    game: &Game,
    host: &str,
    port: u16,
    data: &[u8],
) -> Result<Reply, ParseError> {
    let body = huffman_decode(data);
    let mut r = Reader::new(&body);
    match r.i32()? {
        SERVER_ACCEPTED => {}
        SERVER_IGNORING => return Ok(Reply::Ignoring),
        code => return err(format!("the server refused the query ({code})")),
    }
    r.i32()?; // our time, echoed
    let version = r.string()?;
    let flags = r.u32()?;
    if flags & !QUERY_FLAGS != 0 {
        return err(format!("unrequested fields in the answer: {flags:#x}"));
    }
    let mut s = Server::new(&game.id, host, port);
    s.online = true;
    // Filterable version: "3.2.1-r230709 on Linux..." -> "3.2.1".
    let short = version.split(' ').next().unwrap_or_default();
    s.rules.insert(
        "version".into(),
        short.split('-').next().unwrap_or_default().into(),
    );
    s.rules.insert("fullversion".into(), version.clone());
    if flags & SQF_NAME != 0 {
        s.name = strip_colors(&r.string()?);
    }
    if flags & SQF_MAPNAME != 0 {
        s.map = r.string()?;
    }
    let max_clients = if flags & SQF_MAXCLIENTS != 0 {
        Some(r.byte()?)
    } else {
        None
    };
    let max_players = if flags & SQF_MAXPLAYERS != 0 {
        Some(r.byte()?)
    } else {
        max_clients
    };
    s.max_players = max_players.map(usize::from);
    if flags & SQF_PWADS != 0 {
        let count = r.byte()?;
        let pwads = (0..count)
            .map(|_| r.string())
            .collect::<Result<Vec<_>, _>>()?;
        s.rules.insert("pwads".into(), pwads.join(" "));
    }
    let mut mode = None;
    if flags & SQF_GAMETYPE != 0 {
        mode = Some(r.byte()?);
        s.rules.insert("instagib".into(), r.byte()?.to_string());
        s.rules.insert("buckshot".into(), r.byte()?.to_string());
    }
    if flags & SQF_GAMENAME != 0 {
        s.rules.insert("game".into(), r.string()?);
    }
    if flags & SQF_IWAD != 0 {
        s.rules.insert("iwad".into(), r.string()?);
    }
    if flags & SQF_FORCEPASSWORD != 0 {
        s.password = r.byte()? != 0;
    }
    if flags & SQF_FORCEJOINPASSWORD != 0 {
        s.rules
            .insert("forcejoinpassword".into(), r.byte()?.to_string());
    }
    let count = if flags & SQF_NUMPLAYERS != 0 {
        r.byte()?
    } else {
        0
    };
    if flags & SQF_PLAYERDATA != 0 && count > 0 {
        // Team rows have one more byte; for a mode we do not know, the
        // layout that ends exactly at the end of the packet wins.
        let teams = mode.is_some_and(|m| TEAM_MODES.contains(&m));
        let start = r.pos;
        let mut parsed = players(&mut r, count, teams)
            .ok()
            .filter(|_| r.pos == body.len());
        if parsed.is_none() {
            r.pos = start;
            parsed = players(&mut r, count, !teams)
                .ok()
                .filter(|_| r.pos == body.len());
        }
        let Some((list, bots)) = parsed else {
            return err("inconsistent Zandronum player data");
        };
        s.players = list;
        s.bots = bots;
    }
    s.num_players = usize::from(count);
    s.game_type = mode
        .and_then(|m| GAME_MODES.get(usize::from(m)))
        .map_or_else(String::new, |m| m.to_string());
    s.mod_name = if game.mod_key.is_empty() {
        String::new()
    } else {
        s.rules.get(&game.mod_key).cloned().unwrap_or_default()
    };
    Ok(Reply::Server(Box::new(s)))
}

fn connect(host: &str, port: u16, timeout: Duration) -> io::Result<UdpSocket> {
    let target = (host, port)
        .to_socket_addrs()?
        .find(|address| address.is_ipv4())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("no IPv4 address for {host}"),
            )
        })?;
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_read_timeout(Some(timeout))?;
    socket.connect(target)?;
    Ok(socket)
}

fn query_once(
    game: &Game,
    host: &str,
    port: u16,
    timeout: Duration,
) -> io::Result<(Reply, Duration)> {
    let socket = connect(host, port, timeout)?;
    let started = Instant::now();
    socket.send(&query_request(0))?;
    let mut buffer = [0u8; 8192];
    let size = socket.recv(&mut buffer)?;
    let rtt = started.elapsed();
    Ok((parse_response(game, host, port, &buffer[..size])?, rtt))
}

/// Query a server; when it answers "ignoring", wait `wait` and ask once more.
pub fn status_with_wait(
    game: &Game,
    host: &str,
    port: u16,
    timeout: Duration,
    wait: Duration,
) -> io::Result<(Server, Duration)> {
    for attempt in 0..2 {
        match query_once(game, host, port, timeout)? {
            (Reply::Server(server), rtt) => return Ok((*server, rtt)),
            (Reply::Ignoring, _) if attempt == 0 => std::thread::sleep(wait),
            (Reply::Ignoring, _) => {}
        }
    }
    Err(io::Error::new(
        io::ErrorKind::WouldBlock,
        "the server ignores queries for a few seconds",
    ))
}

/// Query a server (one sample: see the module notes).
pub fn status(
    game: &Game,
    host: &str,
    port: u16,
    timeout: Duration,
) -> io::Result<(Server, Duration)> {
    status_with_wait(game, host, port, timeout, IGNORE_WAIT)
}

/// Fetch the master list: numbered packets, the last one marked.
pub fn master_list(address: &str, timeout: Duration) -> io::Result<Vec<String>> {
    let (host, port) = parse_address(address, 15300)?;
    let socket = connect(&host, port, timeout)?;
    socket.send(&master_request())?;
    let deadline = Instant::now() + timeout;
    let (mut seen, mut addresses) = (HashSet::new(), Vec::new());
    let mut received = false;
    let mut buffer = [0u8; 65536];
    loop {
        let mut wait = deadline.saturating_duration_since(Instant::now());
        if received {
            wait = wait.min(MASTER_IDLE_TIMEOUT);
        }
        if wait.is_zero() {
            break;
        }
        socket.set_read_timeout(Some(wait))?;
        match socket.recv(&mut buffer) {
            Ok(size) => {
                received = true;
                let (entries, last) = parse_master_packet(&buffer[..size])?;
                addresses.extend(entries.into_iter().filter(|a| seen.insert(a.clone())));
                if last {
                    break;
                }
            }
            Err(error) if !received => return Err(error),
            Err(_) => break,
        }
    }
    Ok(addresses)
}
