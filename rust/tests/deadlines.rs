//! Queries give up on servers that never send a valid answer: a localhost
//! fake keeps sending packets that do not match, and the query must still
//! return within a bounded time.

use std::net::UdpSocket;
use std::sync::mpsc;
use std::time::Duration;

use openase::quake3::MAX_PACKETS;

const TIMEOUT: Duration = Duration::from_millis(300);
const LIMIT: Duration = Duration::from_secs(3);

/// A fake that waits for the first request, then sends `packets` with
/// `pause` between them (repeating the last one until 5 s are over).
fn fake(packets: Vec<Vec<u8>>, pause: Duration) -> u16 {
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let port = socket.local_addr().unwrap().port();
    std::thread::spawn(move || {
        socket
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut buffer = [0u8; 2048];
        let Ok((_, from)) = socket.recv_from(&mut buffer) else {
            return;
        };
        let started = std::time::Instant::now();
        let mut i = 0;
        while started.elapsed() < Duration::from_secs(5) {
            std::thread::sleep(pause);
            let packet = &packets[i.min(packets.len() - 1)];
            if socket.send_to(packet, from).is_err() {
                return;
            }
            i += 1;
        }
    });
    port
}

/// Run `query` in a thread; `None` when it did not return within LIMIT.
fn bounded<T: Send + 'static>(query: impl FnOnce() -> T + Send + 'static) -> Option<T> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(query());
    });
    rx.recv_timeout(LIMIT).ok()
}

#[test]
fn ase_status_gives_up_on_junk() {
    let port = fake(vec![b"xx".to_vec()], Duration::from_millis(100));
    let result = bounded(move || openase::ase::status("mta", "127.0.0.1", port, TIMEOUT));
    assert!(matches!(result, Some(Err(_))), "the query did not give up");
}

#[test]
fn gamespy_without_final_stops() {
    let port = fake(vec![b"\\hostname\\x".to_vec()], Duration::from_millis(100));
    let result = bounded(move || openase::gamespy::query("127.0.0.1", port, "\\status\\", TIMEOUT));
    let (pairs, _) = result
        .expect("the query did not stop")
        .expect("pairs read so far");
    assert!(
        !pairs.is_empty() && pairs.len() <= MAX_PACKETS,
        "{} pairs",
        pairs.len()
    );
}

#[test]
fn honest_slow_answer_is_read() {
    let reply = b"EYE1\x04mta\x0622003\x05Test\x05race\x05none\x041.6\x020\x022\x0332\x01".to_vec();
    let port = fake(vec![reply], Duration::from_millis(200));
    // A wide margin over the 200 ms answer: macOS runners oversleep.
    let timeout = Duration::from_secs(1);
    let result = bounded(move || openase::ase::status("mta", "127.0.0.1", port, timeout));
    let (server, _) = result.expect("returned").expect("answer read");
    assert_eq!(server.name, "Test");
}

#[test]
fn tribes2_status_gives_up_on_junk() {
    let port = fake(vec![b"xx".to_vec()], Duration::from_millis(100));
    let result = bounded(move || openase::tribes::t2_status("tribes2", "127.0.0.1", port, TIMEOUT));
    assert!(matches!(result, Some(Err(_))), "the query did not give up");
}
