//! Alerts on servers the user chose: a slot frees up on a full server, or a
//! match fills up to a number of players. Decisions only; the GUI queries
//! the servers and shows the notices.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::model::Server;

/// Default and highest "fills up" thresholds offered.
pub const DEFAULT_THRESHOLD: usize = 4;
pub const MAX_THRESHOLD: usize = 128;
/// How often servers with a party alert are queried.
pub const PARTY_INTERVAL: Duration = Duration::from_secs(60);

/// People on the server: bots do not make a match.
pub fn humans(server: &Server) -> usize {
    server.num_players.saturating_sub(server.bots)
}

/// The server answered and has room for one more player.
pub fn has_free_slot(server: &Server) -> bool {
    server.online
        && server
            .max_players
            .is_some_and(|max| server.num_players < max)
}

/// Decides when a party alert fires: once when the humans cross the
/// threshold upwards, re-armed when they drop to half of it (so 4, 3, 4 does
/// not fire twice). The first answer only sets the starting point, so opening
/// the app does not fire for servers that are already busy.
#[derive(Debug, Default)]
pub struct PartyTracker {
    armed: HashMap<String, bool>,
}

impl PartyTracker {
    pub fn reset(&mut self, key: &str) {
        self.armed.remove(key);
    }

    /// Whether to alert for this answer of the server `key`.
    pub fn update(&mut self, key: &str, server: &Server, threshold: usize) -> bool {
        if !server.online {
            return false;
        }
        let humans = humans(server);
        match self.armed.get(key).copied() {
            None => {
                self.armed.insert(key.to_string(), humans < threshold);
                false
            }
            Some(true) if humans >= threshold => {
                self.armed.insert(key.to_string(), false);
                true
            }
            Some(false) if humans <= threshold / 2 => {
                self.armed.insert(key.to_string(), true);
                false
            }
            Some(_) => false,
        }
    }
}

/// Watching one full server for a free slot.
#[derive(Debug, Clone)]
pub struct SlotWatch {
    /// [`crate::settings::ServerRef::key`] of the watched server.
    pub key: String,
    started: Instant,
    last_query: Option<Instant>,
}

impl SlotWatch {
    pub const INTERVAL: Duration = Duration::from_secs(10);
    pub const MAX_WATCH: Duration = Duration::from_secs(30 * 60);

    pub fn new(key: String, now: Instant) -> Self {
        SlotWatch {
            key,
            started: now,
            last_query: None,
        }
    }

    pub fn due(&self, now: Instant) -> bool {
        self.last_query
            .is_none_or(|last| now >= last + Self::INTERVAL)
    }

    pub fn queried(&mut self, now: Instant) {
        self.last_query = Some(now);
    }

    pub fn expired(&self, now: Instant) -> bool {
        now >= self.started + Self::MAX_WATCH
    }
}
