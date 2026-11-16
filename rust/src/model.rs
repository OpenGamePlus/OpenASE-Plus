//! Protocol-independent server and player model.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// One player on a server.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Player {
    pub name: String,
    pub score: i32,
    pub ping: Option<u32>,
    /// Time connected, in seconds (A2S only).
    pub time_secs: Option<u32>,
}

/// One game server, normalized across protocols. Its JSON form is the
/// `openase-cli` contract and the saved-list format (docs/cli.md).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Server {
    /// Catalog id of the game (`q3`, `cod2`, `ut99`...).
    pub game: String,
    pub host: String,
    /// Port used for queries.
    pub port: u16,
    /// Port players connect to (differs from `port` for Unreal).
    pub join_port: u16,
    /// The server answered a query; `name`, `players`, `ping`... are live.
    pub online: bool,
    /// Listed by a master that sends server details with its list
    /// (333networks JSON API), and not queried yet: name, map, game type,
    /// player counts and country come from the list, ping is unknown.
    /// Such a server is queried only on demand.
    pub listed: bool,
    /// Loaded from the saved list of an earlier session: the details are old,
    /// so the row is never taken as live (alerts, Playing now).
    #[serde(skip)]
    pub saved: bool,
    pub error: Option<String>,

    /// Raw server name, possibly with Quake color codes.
    pub name: String,
    pub map: String,
    /// Raw game type value as reported (`4`, `sd`, `CTFGame`...).
    pub game_type: String,
    pub mod_name: String,
    pub num_players: usize,
    pub max_players: Option<usize>,
    pub bots: usize,
    pub password: bool,
    /// Location reported by the server itself, or `Unknown`.
    pub location: String,
    /// ISO country code from GeoIP, when known.
    pub country: Option<String>,

    /// Average round-trip time in milliseconds.
    pub ping: Option<u32>,
    /// Ping variation (half the spread between fastest and slowest sample).
    pub flux: Option<u32>,
    /// Packet loss in percent over the ping samples.
    pub packet_loss: Option<u8>,

    pub players: Vec<Player>,
    /// Every key/value the server reported.
    pub rules: BTreeMap<String, String>,
}

impl Server {
    pub fn new(game: &str, host: &str, port: u16) -> Self {
        Self {
            game: game.to_string(),
            host: host.to_string(),
            port,
            join_port: port,
            ..Self::default()
        }
    }

    pub fn offline(game: &str, host: &str, port: u16, error: impl Into<String>) -> Self {
        Self {
            error: Some(error.into()),
            packet_loss: Some(100),
            ..Self::new(game, host, port)
        }
    }

    /// Name, map and players are known: queried live, or listed with details.
    pub fn has_info(&self) -> bool {
        self.online || self.listed
    }

    /// Query address, `host:port`.
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Address players connect to, `host:join_port`.
    pub fn join_address(&self) -> String {
        format!("{}:{}", self.host, self.join_port)
    }

    /// Value of a rule, with `mapname` falling back to the normalized map.
    pub fn rule(&self, key: &str) -> Option<&str> {
        match self.rules.get(key) {
            Some(value) => Some(value.as_str()),
            None if key == "mapname" => Some(self.map.as_str()),
            None => None,
        }
    }

    /// Version the server reports, short: the first rule present among the
    /// short keys, else the long engine string cut down ("ioq3 1.36+u2022 linux" → "ioq3 1.36"),
    /// then named by the game's `version_aliases`.
    pub fn version(&self) -> String {
        let short = ["shortversion", "gamever", "gameversion"]
            .iter()
            .find_map(|key| self.rules.get(*key).filter(|v| !v.trim().is_empty()))
            .map(|v| v.trim().to_string());
        let version = short.unwrap_or_else(|| {
            let long = self
                .rules
                .get("version")
                .or_else(|| self.rules.get("si_version"));
            long.map(|v| short_engine_version(v)).unwrap_or_default()
        });
        match crate::catalog::game(&self.game) {
            Some(game) => game.version_name(&version),
            None => version,
        }
    }

    pub fn is_full(&self) -> bool {
        self.max_players
            .is_some_and(|max| max > 0 && self.num_players >= max)
    }
}

/// One entry of a master list: a query address, with the server details when
/// the master sends them (333networks JSON API).
#[derive(Debug, Clone)]
pub struct Listed {
    /// Query address, `ip:port`.
    pub address: String,
    /// Details from the list (`listed` set, `online` not set); `None` when
    /// the master sends addresses only and the server must be queried.
    pub preview: Option<Server>,
}

impl Listed {
    /// An address-only entry.
    pub fn address(address: String) -> Self {
        Self {
            address,
            preview: None,
        }
    }
}

/// Parse a rule value as an unsigned integer.
pub fn parse_count(value: Option<&String>) -> Option<usize> {
    value.and_then(|raw| raw.trim().parse().ok())
}

/// The words of an engine version string up to the first that starts with a
/// digit (or "v" + digit), cut at its first `+ _ / -`: "Wolf 1.41b-MP win-x86" → "Wolf 1.41b".
pub fn short_engine_version(version: &str) -> String {
    let words: Vec<&str> = version.split_whitespace().collect();
    for (i, word) in words.iter().enumerate() {
        let digits = word.strip_prefix('v').unwrap_or(word);
        if digits.starts_with(|c: char| c.is_ascii_digit()) {
            let cut = word.split(['+', '_', '/', '-']).next().unwrap_or(word);
            let mut short = words[..i].to_vec();
            short.push(cut);
            return short.join(" ");
        }
    }
    words.join(" ")
}
