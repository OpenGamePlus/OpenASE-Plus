//! Server lists kept between sessions.

use openase::lists::{load_all, save};
use openase::model::{Player, Server};

fn server(name: &str, players: usize) -> Server {
    let mut s = Server::new("q3", "192.0.2.4", 27960);
    s.online = true;
    s.name = name.into();
    s.map = "q3dm17".into();
    s.num_players = players;
    s.max_players = Some(16);
    s.ping = Some(42);
    s.players = vec![Player {
        name: "Visor".into(),
        score: 3,
        ping: Some(50),
        time_secs: None,
    }];
    s.rules.insert("g_gametype".into(), "4".into());
    s
}

#[test]
fn a_saved_list_loads_as_listed_rows_without_ping() {
    let dir = tempfile::tempdir().unwrap();
    save(dir.path(), "q3", &[server("^1Red", 1)]).unwrap();
    let lists = load_all(dir.path());
    assert_eq!(lists.len(), 1);
    let (game, servers) = &lists[0];
    assert_eq!(game, "q3");
    let s = &servers[0];
    assert_eq!(
        (s.name.as_str(), s.map.as_str(), s.num_players),
        ("^1Red", "q3dm17", 1)
    );
    assert_eq!(s.rules["g_gametype"], "4");
    assert_eq!(s.players[0].name, "Visor");
    // Old details: shown, queried again on demand, never taken as live.
    assert!(s.listed && s.saved && !s.online);
    assert_eq!(s.ping, None);
}

#[test]
fn offline_rows_are_not_saved_and_an_empty_list_removes_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let mut offline = server("gone", 0);
    offline.online = false;
    save(dir.path(), "q3", &[server("up", 2), offline.clone()]).unwrap();
    assert_eq!(load_all(dir.path())[0].1.len(), 1);
    save(dir.path(), "q3", &[offline]).unwrap();
    assert!(load_all(dir.path()).is_empty());
}

#[test]
fn damaged_or_unknown_files_are_skipped() {
    let dir = tempfile::tempdir().unwrap();
    save(dir.path(), "q3", &[server("ok", 1)]).unwrap();
    std::fs::write(dir.path().join("cod2.json"), b"{ not json").unwrap();
    std::fs::write(
        dir.path().join("nosuchgame.json"),
        b"{\"schema\":1,\"servers\":[]}",
    )
    .unwrap();
    std::fs::write(dir.path().join("..json"), b"[]").unwrap();
    let lists = load_all(dir.path());
    assert_eq!(
        lists.iter().map(|(g, _)| g.as_str()).collect::<Vec<_>>(),
        ["q3"]
    );
    // A game id that is not in the catalog is never written either.
    assert!(save(dir.path(), "../evil", &[server("x", 1)]).is_err());
}
