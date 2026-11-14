//! Server lists kept between sessions: the last scan of each game, so the
//! list shows at once on the next start. One file per game, `<id>.json`, in
//! the `lists` folder of the config folder.
//!
//! Loaded rows keep name, map and players but are marked `listed` and
//! `saved`, with no ping: they are queried again on demand, like a
//! 333networks row, and never count as live. Loading is tolerant: a damaged
//! or unknown file is skipped.

use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::catalog;
use crate::model::Server;

const SCHEMA: u32 = 1;
/// A Steam list of 10 000 servers is about 10 MB.
const MAX_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Serialize, Deserialize)]
struct File {
    schema: u32,
    servers: Vec<Server>,
}

pub fn dir() -> PathBuf {
    crate::settings::config_dir().join("lists")
}

/// Save the answering servers of `game`; with none, remove its file.
pub fn save(dir: &Path, game: &str, servers: &[Server]) -> io::Result<()> {
    let Some(game) = catalog::game(game) else {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "unknown game"));
    };
    let path = dir.join(format!("{}.json", game.id));
    let servers: Vec<Server> = servers.iter().filter(|s| s.online).cloned().collect();
    if servers.is_empty() {
        return match std::fs::remove_file(&path) {
            Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error),
            _ => Ok(()),
        };
    }
    std::fs::create_dir_all(dir)?;
    let text = serde_json::to_vec(&File {
        schema: SCHEMA,
        servers,
    })?;
    // Write then rename, so a crash never leaves half a list.
    let partial = path.with_extension("json.partial");
    std::fs::write(&partial, text)?;
    std::fs::rename(&partial, &path)
}

/// Every saved list, as (game id, rows).
pub fn load_all(dir: &Path) -> Vec<(String, Vec<Server>)> {
    catalog::games()
        .iter()
        .filter_map(|game| {
            let path = dir.join(format!("{}.json", game.id));
            if std::fs::metadata(&path).ok()?.len() > MAX_BYTES {
                return None;
            }
            let file: File = serde_json::from_slice(&std::fs::read(&path).ok()?).ok()?;
            if file.schema != SCHEMA {
                return None;
            }
            let rows = file
                .servers
                .into_iter()
                .filter(|s| s.game == game.id)
                .map(|mut s| {
                    s.online = false;
                    s.listed = true;
                    s.saved = true;
                    s.error = None;
                    (s.ping, s.flux, s.packet_loss) = (None, None, None);
                    s
                })
                .collect::<Vec<_>>();
            (!rows.is_empty()).then(|| (game.id.clone(), rows))
        })
        .collect()
}
