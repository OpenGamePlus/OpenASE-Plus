//! openase:// links and passing them to the running instance.

use std::time::Duration;

use openase::instance;
use openase::links::{build, parse};

#[test]
fn links_round_trip() {
    let url = build("q3", "192.0.2.5:27960");
    assert_eq!(url, "openase://connect/q3/192.0.2.5:27960");
    assert_eq!(parse(&url), Some(("q3".into(), "192.0.2.5:27960".into())));
    // Browsers may add a slash or change the case of the scheme.
    assert_eq!(
        parse("OpenASE://connect/cod2/play.example.org:28960/"),
        Some(("cod2".into(), "play.example.org:28960".into()))
    );
}

#[test]
fn anything_else_is_rejected() {
    for bad in [
        "",
        "openase://",
        "openase://connect/q3/192.0.2.5",
        "openase://connect/q3/192.0.2.5:0",
        "openase://connect/q3/192.0.2.5:70000",
        "openase://connect/nosuchgame/192.0.2.5:27960",
        "openase://join/q3/192.0.2.5:27960",
        "openase://connect/q3/192.0.2.5:27960?x=1",
        "openase://connect/q3/192.0.2.5:27960/extra",
        "openase://connect/q3/a%20b:27960",
        "openase://connect/q3/host&calc:27960",
        "openase://connect/q3/-host:27960",
        "openase://connect/q3/[::1]:27960",
        "http://connect/q3/192.0.2.5:27960",
        "\"openase://connect/q3/192.0.2.5:27960\" +exec x",
    ] {
        assert_eq!(parse(bad), None, "{bad}");
    }
    let long = format!("openase://connect/q3/{}.org:27960", "a".repeat(300));
    assert_eq!(parse(&long), None);
}

#[test]
fn a_second_instance_hands_its_link_to_the_first() {
    let dir = tempfile::tempdir().unwrap();
    let port_file = dir.path().join("instance.port");
    // Nobody listens yet: the link cannot be handed over.
    assert!(!instance::forward(
        &port_file,
        "openase://connect/q3/192.0.2.5:27960"
    ));

    let (tx, rx) = std::sync::mpsc::channel();
    instance::listen(&port_file, move |url| tx.send(url).unwrap()).unwrap();
    assert!(instance::forward(
        &port_file,
        "openase://connect/q3/192.0.2.5:27960"
    ));
    assert_eq!(
        rx.recv_timeout(Duration::from_secs(2)).unwrap(),
        "openase://connect/q3/192.0.2.5:27960"
    );
    // Invalid links are not handed over, and a stale port file is harmless.
    assert!(!instance::forward(&port_file, "openase://connect/q3/x&y:1"));
    std::fs::write(&port_file, "1").unwrap();
    assert!(!instance::forward(
        &port_file,
        "openase://connect/q3/192.0.2.5:27960"
    ));
}
