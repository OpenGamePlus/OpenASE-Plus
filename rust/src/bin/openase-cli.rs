//! openase-cli: OpenASE's master lists and server queries without the GUI.
//! Usage and the JSON contract: docs/cli.md.

use std::process::ExitCode;

use openase::catalog::{Game, Master, game, games};
use openase::colors::strip_quake_colors;
use openase::model::Server;
use openase::quake3::parse_address;
use openase::query::{QueryOptions, fetch_all_masters, query_server, resolve_query_port};
use serde_json::{Value, json};

const USAGE: &str = "usage:
  openase-cli games [--json]
  openase-cli scan <game> [--json] [--limit N] [--steam-key KEY]
  openase-cli query <game> <host:port> [--json]";
const SCHEMA: u32 = 1;
const PARALLEL: usize = 32;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(code) => code,
        Err(message) => {
            eprintln!("{message}\n{USAGE}");
            ExitCode::from(2)
        }
    }
}

/// Options after the command; positional arguments first.
struct Args {
    positional: Vec<String>,
    json: bool,
    limit: usize,
    steam_key: String,
}

fn parse(args: &[String]) -> Result<Args, String> {
    let mut parsed = Args {
        positional: Vec::new(),
        json: false,
        limit: 500,
        steam_key: std::env::var("STEAM_API_KEY").unwrap_or_default(),
    };
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--json" => parsed.json = true,
            "--limit" => {
                let value = args.next().ok_or("--limit needs a number")?;
                parsed.limit = value.parse().map_err(|_| format!("bad --limit: {value}"))?;
            }
            "--steam-key" => {
                parsed.steam_key = args.next().ok_or("--steam-key needs a key")?.clone()
            }
            option if option.starts_with("--") => return Err(format!("unknown option {option}")),
            value => parsed.positional.push(value.to_string()),
        }
    }
    Ok(parsed)
}

fn find_game(id: Option<&String>) -> Result<&'static Game, String> {
    let id = id.ok_or("missing game id (see `openase-cli games`)")?;
    game(id).ok_or_else(|| format!("unknown game {id} (see `openase-cli games`)"))
}

fn run(args: &[String]) -> Result<ExitCode, String> {
    let (command, rest) = args.split_first().ok_or("missing command")?;
    let args = parse(rest)?;
    match command.as_str() {
        "games" => {
            print_games(args.json);
            Ok(ExitCode::SUCCESS)
        }
        "scan" => {
            let game = find_game(args.positional.first())?;
            Ok(scan(game, &args))
        }
        "query" => {
            let game = find_game(args.positional.first())?;
            let address = args.positional.get(1).ok_or("missing host:port")?;
            let (host, port) = parse_address(address, game.default_port).map_err(|e| e.0)?;
            Ok(query(game, &host, port, args.json))
        }
        other => Err(format!("unknown command {other}")),
    }
}

fn header() -> Value {
    json!({ "schema": SCHEMA, "openase": env!("CARGO_PKG_VERSION") })
}

fn with(mut doc: Value, extra: Value) -> Value {
    if let (Some(doc), Value::Object(extra)) = (doc.as_object_mut(), extra) {
        doc.extend(extra);
    }
    doc
}

fn print_json(doc: &Value) {
    println!("{}", serde_json::to_string_pretty(doc).unwrap());
}

fn print_games(as_json: bool) {
    if !as_json {
        for game in games() {
            println!("{:<12} {}", game.id, game.name);
        }
        return;
    }
    let list: Vec<Value> = games()
        .iter()
        .map(|game| {
            let steam_only = !game.masters.is_empty()
                && game
                    .masters
                    .iter()
                    .all(|m| matches!(m, Master::SteamWeb { .. }));
            json!({
                "id": game.id,
                "name": game.name,
                "short": game.short,
                "protocol": game.protocol,
                "default_port": game.default_port,
                "needs_steam_key": steam_only,
                "masters": game.masters.iter().map(Master::label).collect::<Vec<_>>(),
            })
        })
        .collect();
    print_json(&with(header(), json!({ "games": list })));
}

fn game_json(game: &Game) -> Value {
    json!({ "id": game.id, "name": game.name })
}

fn scan(game: &'static Game, args: &Args) -> ExitCode {
    let options = QueryOptions::default();
    let mut list = fetch_all_masters(game, options.timeout, &args.steam_key, args.limit);
    list.servers.truncate(args.limit.max(1));
    let listed = list.servers.len();
    // 333networks rows come with their details and are not queried
    // (their admins asked for that); the others are queried.
    let (previews, addresses): (Vec<_>, Vec<_>) =
        list.servers.into_iter().partition(|e| e.preview.is_some());
    let mut servers: Vec<Server> = previews.into_iter().filter_map(|e| e.preview).collect();
    for chunk in addresses.chunks(PARALLEL) {
        std::thread::scope(|scope| {
            let handles: Vec<_> = chunk
                .iter()
                .map(|entry| {
                    let options = &options;
                    scope.spawn(move || {
                        let (host, port) = parse_address(&entry.address, game.default_port).ok()?;
                        Some(query_server(game, &host, port, options))
                    })
                })
                .collect();
            servers.extend(handles.into_iter().filter_map(|h| h.join().ok().flatten()));
        });
    }
    let answered = listed > 0 || list.errors.is_empty();
    if args.json {
        print_json(&with(
            header(),
            json!({
                "game": game_json(game),
                "listed": listed,
                "errors": list.errors,
                "servers": servers,
            }),
        ));
    } else {
        for error in &list.errors {
            eprintln!("master error: {error}");
        }
        for s in servers.iter().filter(|s| s.has_info()) {
            println!(
                "{:<22} {:>4} {:>3}/{:<3} {:<14} {}",
                s.address(),
                s.ping.map_or("-".into(), |p| p.to_string()),
                s.num_players,
                s.max_players.map_or("?".into(), |m| m.to_string()),
                s.map,
                strip_quake_colors(&s.name),
            );
        }
        eprintln!(
            "{}: {} listed, {} answered",
            game.name,
            listed,
            servers.iter().filter(|s| s.online).count()
        );
    }
    if answered {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn query(game: &'static Game, host: &str, port: u16, as_json: bool) -> ExitCode {
    let options = QueryOptions::default();
    // Like Add server: a game port of an Unreal engine server becomes its query port.
    let port = resolve_query_port(game, host, port, options.timeout);
    let server = query_server(game, host, port, &options);
    if as_json {
        print_json(&with(
            header(),
            json!({ "game": game_json(game), "server": server }),
        ));
    } else if server.online {
        println!("{}", strip_quake_colors(&server.name));
        println!(
            "map {}  players {}/{}  ping {} ms",
            server.map,
            server.num_players,
            server.max_players.map_or("?".into(), |m| m.to_string()),
            server.ping.unwrap_or(0)
        );
    } else {
        eprintln!("no answer: {}", server.error.as_deref().unwrap_or_default());
    }
    if server.online {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}
