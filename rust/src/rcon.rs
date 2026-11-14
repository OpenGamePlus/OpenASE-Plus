//! RCON: admin commands to a server, in the three dialects (ported from JASE).
//!
//! - Quake (Quake 3, Call of Duty, Quake 2 families), UDP on the game port:
//!   `FF FF FF FF rcon <password> <command>`, answered by `print\n...` packets.
//! - GoldSrc (Half-Life 1 games), UDP: `challenge rcon` first, then
//!   `rcon <challenge> "<password>" <command>`; answers start with `l`.
//! - Source (Source games), TCP on the game port: `[size][id][type][body\0\0]`
//!   packets, AUTH (3) then EXECCOMMAND (2); an empty packet sent after the
//!   command is echoed after the last part of a long answer.
//!
//! The password is sent as typed; it is never stored. The answer is plain
//! text from the server: shown as such.

use std::io::{self, Read, Write};
use std::net::{TcpStream, ToSocketAddrs, UdpSocket};
use std::time::Duration;

use crate::catalog::{Game, Protocol};
use crate::colors::strip_quake_colors;
use crate::quake3::latin1;

const OOB: &[u8] = b"\xff\xff\xff\xff";
/// After the first UDP packet, how long to wait for more parts.
const GRACE: Duration = Duration::from_millis(400);
const MAX_COMMAND: usize = 1000;
const MAX_SOURCE_PACKET: usize = 4096 + 14;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Quake,
    GoldSrc,
    Source,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RconError {
    Timeout,
    BadPassword,
    NoChallenge,
    Unsupported,
    /// Malformed answer.
    Invalid,
    /// Could not connect or the connection broke.
    Network(String),
}

impl From<io::Error> for RconError {
    fn from(error: io::Error) -> Self {
        match error.kind() {
            io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut => RconError::Timeout,
            _ => RconError::Network(error.to_string()),
        }
    }
}

/// The dialect of a game, if it has RCON.
pub fn kind_of(game: &Game) -> Option<Kind> {
    match game.protocol {
        Protocol::Quake3 | Protocol::Quake2 | Protocol::QuakeWorld => Some(Kind::Quake),
        // Half-Life 1 games have app ids below 200 (10 CS, 20 TFC, 30 DoD,
        // 70 HL...); Source games start at 220.
        Protocol::A2s => match game.steam_appid {
            Some(id) if id < 200 => Some(Kind::GoldSrc),
            _ => Some(Kind::Source),
        },
        _ => None,
    }
}

/// One line (a line break would smuggle a second command), at most 1000 chars.
pub fn clean_command(command: &str) -> String {
    let line = command.lines().collect::<Vec<_>>().join(" ");
    line.trim().chars().take(MAX_COMMAND).collect()
}

/// Latin-1 bytes, `?` for anything outside it.
fn latin1_bytes(text: &str) -> Vec<u8> {
    text.chars()
        .map(|c| u8::try_from(u32::from(c)).unwrap_or(b'?'))
        .collect()
}

fn udp(host: &str, port: u16, timeout: Duration) -> io::Result<UdpSocket> {
    let target = (host, port)
        .to_socket_addrs()?
        .find(|a| a.is_ipv4())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("no IPv4 address for {host}"),
            )
        })?;
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect(target)?;
    socket.set_read_timeout(Some(timeout))?;
    Ok(socket)
}

/// The first packet (within `timeout`), then any that follow within GRACE.
fn packets(socket: &UdpSocket, timeout: Duration) -> Result<Vec<Vec<u8>>, RconError> {
    let mut buffer = [0u8; 65536];
    socket.set_read_timeout(Some(timeout))?;
    let size = socket.recv(&mut buffer)?;
    let mut out = vec![buffer[..size].to_vec()];
    socket.set_read_timeout(Some(GRACE))?;
    while let Ok(size) = socket.recv(&mut buffer) {
        out.push(buffer[..size].to_vec());
    }
    Ok(out)
}

fn starts_with_bad_password(text: &str) -> bool {
    // Only at the start: a command's output may mention "invalid password".
    let start = text.trim_start().to_lowercase();
    ["bad rconpassword", "bad rcon_password", "invalid password"]
        .iter()
        .any(|prefix| start.starts_with(prefix))
}

pub fn quake(
    host: &str,
    port: u16,
    password: &str,
    command: &str,
    timeout: Duration,
) -> Result<String, RconError> {
    let socket = udp(host, port, timeout)?;
    socket.send(&[OOB, &latin1_bytes(&format!("rcon {password} {command}"))].concat())?;
    let text: String = packets(&socket, timeout)?
        .iter()
        .filter_map(|p| p.strip_prefix(OOB))
        .map(|p| {
            let text = latin1(p);
            text.strip_prefix("print\n")
                .map(str::to_string)
                .unwrap_or(text)
        })
        .collect();
    let text = strip_quake_colors(&text);
    if starts_with_bad_password(&text) {
        return Err(RconError::BadPassword);
    }
    Ok(text)
}

pub fn goldsrc(
    host: &str,
    port: u16,
    password: &str,
    command: &str,
    timeout: Duration,
) -> Result<String, RconError> {
    let socket = udp(host, port, timeout)?;
    socket.send(&[OOB, b"challenge rcon\n"].concat())?;
    let reply = packets(&socket, timeout)?.remove(0);
    let reply = latin1(reply.strip_prefix(OOB).unwrap_or_default());
    let challenge = reply
        .strip_prefix("challenge rcon ")
        .map(|rest| {
            rest.trim()
                .chars()
                .take_while(char::is_ascii_digit)
                .collect::<String>()
        })
        .filter(|digits| !digits.is_empty())
        .ok_or(RconError::NoChallenge)?;
    let line = format!("rcon {challenge} \"{password}\" {command}\n");
    socket.send(&[OOB, &latin1_bytes(&line)].concat())?;
    let text: String = packets(&socket, timeout)?
        .iter()
        .filter_map(|p| p.strip_prefix(OOB)?.strip_prefix(b"l"))
        .map(latin1)
        .collect();
    if starts_with_bad_password(&text) {
        return Err(RconError::BadPassword);
    }
    Ok(text)
}

const SERVERDATA_AUTH: i32 = 3;
const SERVERDATA_AUTH_RESPONSE: i32 = 2;
const SERVERDATA_EXECCOMMAND: i32 = 2;
const SERVERDATA_RESPONSE_VALUE: i32 = 0;

fn source_packet(id: i32, kind: i32, body: &str) -> Vec<u8> {
    let mut payload = id.to_le_bytes().to_vec();
    payload.extend_from_slice(&kind.to_le_bytes());
    payload.extend_from_slice(body.as_bytes());
    payload.extend_from_slice(b"\0\0");
    let mut packet = (payload.len() as i32).to_le_bytes().to_vec();
    packet.extend(payload);
    packet
}

fn read_source(stream: &mut TcpStream) -> Result<(i32, i32, String), RconError> {
    let mut size = [0u8; 4];
    stream.read_exact(&mut size)?;
    let size = i32::from_le_bytes(size);
    let size = usize::try_from(size)
        .ok()
        .filter(|s| (10..=MAX_SOURCE_PACKET).contains(s));
    let mut data = vec![0u8; size.ok_or(RconError::Invalid)?];
    stream.read_exact(&mut data)?;
    let id = i32::from_le_bytes(data[..4].try_into().unwrap());
    let kind = i32::from_le_bytes(data[4..8].try_into().unwrap());
    let body = data[8..].split(|&b| b == 0).next().unwrap_or_default();
    Ok((id, kind, String::from_utf8_lossy(body).into_owned()))
}

pub fn source(
    host: &str,
    port: u16,
    password: &str,
    command: &str,
    timeout: Duration,
) -> Result<String, RconError> {
    let target = (host, port)
        .to_socket_addrs()?
        .find(|a| a.is_ipv4())
        .ok_or_else(|| RconError::Network(format!("no IPv4 address for {host}")))?;
    let mut stream = TcpStream::connect_timeout(&target, timeout)?;
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;
    stream.write_all(&source_packet(1, SERVERDATA_AUTH, password))?;
    // An empty RESPONSE_VALUE may come before the AUTH_RESPONSE.
    let id = loop {
        let (id, kind, _) = read_source(&mut stream)?;
        if kind == SERVERDATA_AUTH_RESPONSE {
            break id;
        }
    };
    if id == -1 {
        return Err(RconError::BadPassword);
    }
    let mut request = source_packet(2, SERVERDATA_EXECCOMMAND, command);
    request.extend(source_packet(3, SERVERDATA_RESPONSE_VALUE, ""));
    stream.write_all(&request)?;
    let mut answer = String::new();
    loop {
        match read_source(&mut stream)? {
            (3, _, _) => return Ok(answer),
            (2, _, body) => answer.push_str(&body),
            _ => {}
        }
    }
}

/// Send `command` to the server at `host:port` (its game port).
pub fn send(
    game: &Game,
    host: &str,
    port: u16,
    password: &str,
    command: &str,
) -> Result<String, RconError> {
    let command = clean_command(command);
    let kind = kind_of(game).ok_or(RconError::Unsupported)?;
    if command.is_empty() {
        return Ok(String::new());
    }
    match kind {
        Kind::Quake => quake(host, port, password, &command, Duration::from_secs(3)),
        Kind::GoldSrc => goldsrc(host, port, password, &command, Duration::from_secs(3)),
        Kind::Source => source(host, port, password, &command, Duration::from_secs(5)),
    }
}
