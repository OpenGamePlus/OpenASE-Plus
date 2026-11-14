//! Slot and party alerts: when they fire.

use openase::alerts::{PartyTracker, SlotWatch, has_free_slot, humans};
use openase::model::Server;
use std::time::{Duration, Instant};

fn server(players: usize, bots: usize, max: Option<usize>) -> Server {
    let mut s = Server::new("q3", "192.0.2.1", 27960);
    s.online = true;
    s.num_players = players;
    s.bots = bots;
    s.max_players = max;
    s
}

#[test]
fn free_slot_needs_an_online_server_with_a_known_limit() {
    assert!(has_free_slot(&server(7, 0, Some(8))));
    assert!(!has_free_slot(&server(8, 0, Some(8))));
    assert!(!has_free_slot(&server(3, 0, None)));
    let mut offline = server(1, 0, Some(8));
    offline.online = false;
    assert!(!has_free_slot(&offline));
}

#[test]
fn bots_are_not_players_for_a_party() {
    assert_eq!(humans(&server(6, 4, Some(16))), 2);
    assert_eq!(humans(&server(2, 5, Some(16))), 0);
}

#[test]
fn party_fires_once_when_crossing_up_and_rearms_at_half() {
    let mut t = PartyTracker::default();
    let key = "q3|192.0.2.1:27960";
    // The first answer only sets the starting point.
    assert!(!t.update(key, &server(5, 0, Some(16)), 4));
    assert!(!t.update(key, &server(6, 0, Some(16)), 4));
    t.reset(key);
    assert!(!t.update(key, &server(1, 0, Some(16)), 4));
    assert!(t.update(key, &server(4, 0, Some(16)), 4));
    // 4 -> 3 -> 4 does not fire twice; dropping to half re-arms.
    assert!(!t.update(key, &server(3, 0, Some(16)), 4));
    assert!(!t.update(key, &server(4, 0, Some(16)), 4));
    assert!(!t.update(key, &server(2, 0, Some(16)), 4));
    assert!(t.update(key, &server(5, 0, Some(16)), 4));
    // Bots do not count, offline answers are ignored.
    t.reset(key);
    assert!(!t.update(key, &server(0, 0, Some(16)), 4));
    assert!(!t.update(key, &server(8, 8, Some(16)), 4));
    let mut offline = server(9, 0, Some(16));
    offline.online = false;
    assert!(!t.update(key, &offline, 4));
}

#[test]
fn slot_watch_is_due_every_interval_and_expires() {
    let start = Instant::now();
    let mut watch = SlotWatch::new("q3|192.0.2.1:27960".into(), start);
    assert!(watch.due(start));
    watch.queried(start);
    assert!(!watch.due(start + Duration::from_secs(5)));
    assert!(watch.due(start + SlotWatch::INTERVAL));
    assert!(!watch.expired(start + Duration::from_secs(60)));
    assert!(watch.expired(start + SlotWatch::MAX_WATCH));
}
