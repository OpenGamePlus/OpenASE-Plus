//! RCON against fake servers on localhost: Quake (UDP), GoldSrc (UDP with a
//! challenge) and Source (TCP).

use std::io::{Read, Write};
use std::net::{TcpListener, UdpSocket};
use std::time::Duration;

use openase::catalog::game;
use openase::rcon::{self, Kind, RconError, clean_command, kind_of};

const OOB: &[u8] = b"\xff\xff\xff\xff";
const T: Duration = Duration::from_secs(2);

fn udp_server(replies: Vec<Vec<Vec<u8>>>) -> (u16, std::thread::JoinHandle<Vec<Vec<u8>>>) {
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    socket.set_read_timeout(Some(T)).unwrap();
    let port = socket.local_addr().unwrap().port();
    let thread = std::thread::spawn(move || {
        let mut asked = Vec::new();
        for answer in replies {
            let mut buffer = [0u8; 2048];
            let Ok((size, from)) = socket.recv_from(&mut buffer) else {
                break;
            };
            asked.push(buffer[..size].to_vec());
            for packet in answer {
                socket.send_to(&packet, from).unwrap();
            }
        }
        asked
    });
    (port, thread)
}

fn oob(text: &str) -> Vec<u8> {
    [OOB, text.as_bytes()].concat()
}

#[test]
fn each_game_family_has_its_kind() {
    assert_eq!(kind_of(game("q3").unwrap()), Some(Kind::Quake));
    assert_eq!(kind_of(game("cod2").unwrap()), Some(Kind::Quake));
    assert_eq!(kind_of(game("q2").unwrap()), Some(Kind::Quake));
    assert_eq!(kind_of(game("cs16").unwrap()), Some(Kind::GoldSrc));
    assert_eq!(kind_of(game("tf2").unwrap()), Some(Kind::Source));
    assert_eq!(kind_of(game("css").unwrap()), Some(Kind::Source));
    assert_eq!(kind_of(game("ut99").unwrap()), None);
}

#[test]
fn a_command_is_one_line() {
    assert_eq!(clean_command("status\nquit"), "status quit");
    assert_eq!(clean_command("  map q3dm17 \r\n"), "map q3dm17");
    assert_eq!(clean_command(&"x".repeat(5000)).len(), 1000);
}

#[test]
fn quake_joins_print_packets_and_strips_colors() {
    let (port, server) = udp_server(vec![vec![
        oob("print\n^1map: ^7q3dm17\n"),
        oob("print\nnum score\n"),
    ]]);
    let out = rcon::quake("127.0.0.1", port, "secret", "status", T).unwrap();
    assert_eq!(out, "map: q3dm17\nnum score\n");
    assert_eq!(server.join().unwrap()[0], oob("rcon secret status"));
}

#[test]
fn quake_bad_password_and_timeout() {
    let (port, _server) = udp_server(vec![vec![oob("print\nBad rconpassword.\n")]]);
    assert_eq!(
        rcon::quake("127.0.0.1", port, "nope", "status", T),
        Err(RconError::BadPassword)
    );
    // Output that only mentions a password is kept.
    let (port, _server) = udp_server(vec![vec![oob(
        "print\nset g_password: invalid password ignored\n",
    )]]);
    assert!(rcon::quake("127.0.0.1", port, "ok", "status", T).is_ok());
    let (port, _server) = udp_server(vec![vec![]]);
    assert_eq!(
        rcon::quake("127.0.0.1", port, "x", "status", Duration::from_millis(300)),
        Err(RconError::Timeout)
    );
}

#[test]
fn goldsrc_asks_a_challenge_then_sends_the_command() {
    let (port, server) = udp_server(vec![
        vec![oob("challenge rcon 123456\n")],
        vec![oob("lhostname: Test\n")],
    ]);
    let out = rcon::goldsrc("127.0.0.1", port, "pw", "status", T).unwrap();
    assert_eq!(out, "hostname: Test\n");
    let asked = server.join().unwrap();
    assert_eq!(asked[0], oob("challenge rcon\n"));
    assert_eq!(asked[1], oob("rcon 123456 \"pw\" status\n"));
}

fn source_packet(id: i32, kind: i32, body: &str) -> Vec<u8> {
    let mut payload = id.to_le_bytes().to_vec();
    payload.extend_from_slice(&kind.to_le_bytes());
    payload.extend_from_slice(body.as_bytes());
    payload.extend_from_slice(b"\0\0");
    [(payload.len() as i32).to_le_bytes().to_vec(), payload].concat()
}

fn read_source(stream: &mut impl Read) -> (i32, i32, String) {
    let mut size = [0u8; 4];
    stream.read_exact(&mut size).unwrap();
    let mut data = vec![0u8; i32::from_le_bytes(size) as usize];
    stream.read_exact(&mut data).unwrap();
    let id = i32::from_le_bytes(data[..4].try_into().unwrap());
    let kind = i32::from_le_bytes(data[4..8].try_into().unwrap());
    let body = String::from_utf8_lossy(&data[8..data.len() - 2]).into_owned();
    (id, kind, body)
}

#[test]
fn source_authenticates_and_joins_a_long_answer() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = std::thread::spawn(move || {
        let (mut s, _) = listener.accept().unwrap();
        let auth = read_source(&mut s);
        // An empty RESPONSE_VALUE may come before the AUTH_RESPONSE.
        s.write_all(&source_packet(auth.0, 0, "")).unwrap();
        s.write_all(&source_packet(auth.0, 2, "")).unwrap();
        let command = read_source(&mut s);
        let marker = read_source(&mut s);
        s.write_all(&source_packet(command.0, 0, "part one, "))
            .unwrap();
        s.write_all(&source_packet(command.0, 0, "part two"))
            .unwrap();
        s.write_all(&source_packet(marker.0, 0, "")).unwrap();
        (auth, command)
    });
    let out = rcon::source("127.0.0.1", port, "pw", "status", T).unwrap();
    assert_eq!(out, "part one, part two");
    let (auth, command) = server.join().unwrap();
    assert_eq!((auth.1, auth.2.as_str()), (3, "pw"));
    assert_eq!((command.1, command.2.as_str()), (2, "status"));
}

#[test]
fn source_bad_password() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        let (mut s, _) = listener.accept().unwrap();
        read_source(&mut s);
        s.write_all(&source_packet(-1, 2, "")).unwrap();
        let _ = s.read(&mut [0u8; 16]);
    });
    assert_eq!(
        rcon::source("127.0.0.1", port, "bad", "status", T),
        Err(RconError::BadPassword)
    );
}
