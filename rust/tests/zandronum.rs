//! Zandronum (Doom): Huffman, master list and server query, against packets
//! captured from real servers (2026-10-27).

use std::net::UdpSocket;
use std::time::Duration;

use openase::catalog::game;
use openase::launch::join_args;
use openase::model::Server;
use openase::settings::GameSettings;
use openase::zandronum::{
    self as z, Reply, huffman_decode, parse_master_packet, parse_response, strip_colors,
};

const PLAYERS: &[u8] = include_bytes!("fixtures/zandronum_players.bin");
const TEAM: &[u8] = include_bytes!("fixtures/zandronum_team.bin");
const MASTER: &[u8] = include_bytes!("fixtures/zandronum_master.bin");

fn raw(body: &[u8]) -> Vec<u8> {
    [&[0xff][..], body].concat()
}

fn server(reply: Reply) -> Server {
    match reply {
        Reply::Server(server) => *server,
        Reply::Ignoring => panic!("ignoring"),
    }
}

fn zandronum() -> &'static openase::catalog::Game {
    game("zandronum").unwrap()
}

#[test]
fn huffman_reads_raw_and_compressed_packets() {
    assert_eq!(huffman_decode(b"\xffhello"), b"hello");
    assert_eq!(huffman_decode(b""), b"");
    let body = huffman_decode(PLAYERS);
    assert_eq!(
        i32::from_le_bytes(body[..4].try_into().unwrap()),
        z::SERVER_ACCEPTED
    );
}

#[test]
fn master_packet_lists_servers_and_end_marker() {
    let (found, last) = parse_master_packet(MASTER).unwrap();
    assert!(last);
    assert_eq!(found.len(), 78);
    assert!(found.iter().all(|a| !a.ends_with(":0")));
}

#[test]
fn master_errors_and_truncation_are_errors() {
    let banned = parse_master_packet(&raw(&3i32.to_le_bytes())).unwrap_err();
    assert!(banned.0.contains("banned"), "{banned:?}");
    let busy = parse_master_packet(&raw(&4i32.to_le_bytes())).unwrap_err();
    assert!(busy.0.contains("wait"), "{busy:?}");
    let mut truncated = 6i32.to_le_bytes().to_vec();
    truncated.extend_from_slice(&[0, 8, 2, 1]);
    assert!(parse_master_packet(&raw(&truncated)).is_err());
}

#[test]
fn team_game_with_bots() {
    let s = server(parse_response(zandronum(), "192.0.2.1", 10666, TEAM).unwrap());
    assert_eq!(s.name, "[TSPG-PK] Ultimate Railgun");
    assert_eq!(s.map, "BUR08");
    assert_eq!(s.game_type, "teamplay");
    assert_eq!(s.rules["version"], "3.3");
    assert_eq!((s.num_players, s.max_players, s.bots), (4, Some(32), 4));
    let names: Vec<&str> = s.players.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, ["Pluto", "KnightMan", "Neptune", "Sunstar"]);
    assert_eq!(s.mod_name, "megagame.wad");
}

#[test]
fn cooperative_game_players() {
    let s = server(parse_response(zandronum(), "192.0.2.1", 10666, PLAYERS).unwrap());
    assert_eq!(s.game_type, "cooperative");
    assert_eq!(s.map, "DARK_27");
    let players: Vec<(&str, i32, Option<u32>)> = s
        .players
        .iter()
        .map(|p| (p.name.as_str(), p.score, p.ping))
        .collect();
    assert_eq!(players, [("Player", 0, Some(76)), ("leg", 757, Some(45))]);
    assert_eq!(s.bots, 0);
    assert!(!s.password);
    assert_eq!(s.mod_name, "doom2.wad");
}

#[test]
fn ignoring_and_bad_answers() {
    let mut ignoring = z::SERVER_IGNORING.to_le_bytes().to_vec();
    ignoring.extend_from_slice(&0i32.to_le_bytes());
    assert!(matches!(
        parse_response(zandronum(), "192.0.2.1", 10666, &raw(&ignoring)),
        Ok(Reply::Ignoring)
    ));
    let body = huffman_decode(TEAM);
    assert!(
        parse_response(
            zandronum(),
            "192.0.2.1",
            10666,
            &raw(&body[..body.len() - 5])
        )
        .is_err()
    );
}

#[test]
fn player_rows_fall_back_to_the_other_team_layout() {
    let body = huffman_decode(TEAM);
    let original = server(parse_response(zandronum(), "192.0.2.1", 10666, TEAM).unwrap());
    // Same packet with the mode changed to deathmatch (no team byte): the
    // first pass is misaligned and the second one reads the rows.
    let map_end = body.windows(6).position(|w| w == b"BUR08\0").unwrap() + 6;
    let mut pos = map_end + 2; // max clients, max players
    let count = body[pos];
    pos += 1;
    for _ in 0..count {
        pos += body[pos..].iter().position(|&b| b == 0).unwrap() + 1;
    }
    assert_eq!(body[pos], 4); // teamplay
    let mut patched = body.clone();
    patched[pos] = 3;
    let again = server(parse_response(zandronum(), "192.0.2.1", 10666, &raw(&patched)).unwrap());
    let names = |s: &Server| s.players.iter().map(|p| p.name.clone()).collect::<Vec<_>>();
    assert_eq!(names(&again), names(&original));
}

#[test]
fn doom_color_codes_are_stripped() {
    assert_eq!(strip_colors("\x1cGRed\x1c[Gold]Gold\x1c-x"), "RedGoldx");
}

#[test]
fn join_passes_the_iwad_and_both_passwords() {
    let mut s = Server::new("zandronum", "192.0.2.1", 10666);
    s.mod_name = "DOOM2.WAD".into();
    assert_eq!(
        join_args(zandronum(), &GameSettings::default(), &s, "pw"),
        [
            "-iwad",
            "DOOM2.WAD",
            "+cl_password",
            "pw",
            "+cl_joinpassword",
            "pw",
            "-connect",
            "192.0.2.1:10666"
        ]
    );
    for evil in [
        "+quit",
        "-file x.wad",
        "-quit",
        "..\\doom2.wad",
        "C:\\doom2.wad",
        ".hidden",
        "",
    ] {
        s.mod_name = evil.into();
        assert_eq!(
            join_args(zandronum(), &GameSettings::default(), &s, ""),
            ["-connect", "192.0.2.1:10666"],
            "{evil}"
        );
    }
}

/// A fake server that answers "ignoring" first, then the real reply: the
/// query waits and asks again once.
#[test]
fn ignored_query_waits_and_retries() {
    let fake = UdpSocket::bind("127.0.0.1:0").unwrap();
    fake.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
    let port = fake.local_addr().unwrap().port();
    let mut ignoring = z::SERVER_IGNORING.to_le_bytes().to_vec();
    ignoring.extend_from_slice(&0i32.to_le_bytes());
    let replies = [raw(&ignoring), PLAYERS.to_vec()];
    let thread = std::thread::spawn(move || {
        let mut asked = Vec::new();
        for reply in replies {
            let mut buffer = [0u8; 2048];
            let Ok((size, from)) = fake.recv_from(&mut buffer) else {
                break;
            };
            asked.push(huffman_decode(&buffer[..size]));
            fake.send_to(&reply, from).unwrap();
        }
        asked
    });
    let (s, _) = z::status_with_wait(
        zandronum(),
        "127.0.0.1",
        port,
        Duration::from_secs(1),
        Duration::from_millis(100),
    )
    .unwrap();
    let asked = thread.join().unwrap();
    assert_eq!(asked.len(), 2);
    assert_eq!(
        i32::from_le_bytes(asked[0][..4].try_into().unwrap()),
        z::SERVER_CHALLENGE
    );
    assert_eq!(s.map, "DARK_27");
}
