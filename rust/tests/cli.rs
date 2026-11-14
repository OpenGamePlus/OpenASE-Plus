//! openase-cli: the JSON contract and exit codes (no Internet: a fake
//! Zandronum server on localhost answers with a real capture).

use std::net::UdpSocket;
use std::process::Command;
use std::time::Duration;

fn cli(args: &[&str]) -> (i32, serde_json::Value, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_openase-cli"))
        .args(args)
        .output()
        .unwrap();
    let json = serde_json::from_slice(&out.stdout).unwrap_or(serde_json::Value::Null);
    (
        out.status.code().unwrap(),
        json,
        String::from_utf8_lossy(&out.stderr).into(),
    )
}

#[test]
fn games_lists_the_catalog() {
    let (code, json, _) = cli(&["games", "--json"]);
    assert_eq!(code, 0);
    assert_eq!(json["schema"], 1);
    let games = json["games"].as_array().unwrap();
    let q3 = games.iter().find(|g| g["id"] == "q3").unwrap();
    assert_eq!(q3["protocol"], "quake3");
    assert_eq!(q3["needs_steam_key"], false);
    let tf2 = games.iter().find(|g| g["id"] == "tf2").unwrap();
    assert_eq!(tf2["needs_steam_key"], true);
}

#[test]
fn usage_errors_exit_with_2() {
    assert_eq!(cli(&[]).0, 2);
    assert_eq!(cli(&["scan", "nosuchgame"]).0, 2);
    assert_eq!(cli(&["query", "q3"]).0, 2);
    assert_eq!(cli(&["frobnicate"]).0, 2);
}

#[test]
fn query_prints_the_server_as_json() {
    let fake = UdpSocket::bind("127.0.0.1:0").unwrap();
    fake.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let port = fake.local_addr().unwrap().port();
    let thread = std::thread::spawn(move || {
        let mut buffer = [0u8; 2048];
        if let Ok((_, from)) = fake.recv_from(&mut buffer) {
            fake.send_to(include_bytes!("fixtures/zandronum_players.bin"), from)
                .unwrap();
        }
    });
    let address = format!("127.0.0.1:{port}");
    let (code, json, stderr) = cli(&["query", "zandronum", &address, "--json"]);
    thread.join().unwrap();
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(json["schema"], 1);
    assert_eq!(json["game"]["id"], "zandronum");
    let server = &json["server"];
    assert_eq!(server["online"], true);
    assert_eq!(server["map"], "DARK_27");
    assert_eq!(server["players"][1]["name"], "leg");
}

#[test]
fn a_silent_server_exits_with_1() {
    // Nothing listens on this port (bound, then dropped).
    let port = UdpSocket::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let (code, json, _) = cli(&["query", "zandronum", &format!("127.0.0.1:{port}"), "--json"]);
    assert_eq!(code, 1);
    assert_eq!(json["server"]["online"], false);
}
