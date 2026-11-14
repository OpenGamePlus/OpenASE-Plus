//! User settings, stored as JSON in the user's config folder
//! (`%APPDATA%\OpenASE-Plus\settings.json` on Windows; upstream OpenASE's
//! file is imported once, see [`import_upstream`]).

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::query::QueryOptions;
use crate::rules::CustomFilter;

/// Per-game options (ASE's Options > Games).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GameSettings {
    pub visible: bool,
    pub player_name: String,
    pub config_file: String,
    pub program_location: String,
    pub allow_invitations: bool,
    pub draw_player_names_with_game_font: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            visible: true,
            player_name: String::new(),
            config_file: String::new(),
            program_location: String::new(),
            allow_invitations: true,
            draw_player_names_with_game_font: true,
        }
    }
}

/// A server remembered in Favorites or Recently played.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ServerRef {
    pub game: String,
    /// Query address, `host:port`.
    pub address: String,
}

/// ASE's Options > Network.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct NetworkSettings {
    /// Timeout when querying a server, in milliseconds.
    pub timeout_ms: u64,
    /// Number of times to ping a server.
    pub pings: u32,
    /// Number of times to retry a scan.
    pub retries: u32,
    /// Servers queried at the same time.
    pub parallel_queries: usize,
    /// Maximum servers taken from the masters per refresh.
    pub scan_limit: usize,
}

impl Default for NetworkSettings {
    fn default() -> Self {
        Self {
            timeout_ms: 1000,
            pings: 3,
            retries: 1,
            parallel_queries: 32,
            scan_limit: 1500,
        }
    }
}

impl NetworkSettings {
    pub fn query_options(&self) -> QueryOptions {
        QueryOptions {
            timeout: Duration::from_millis(self.timeout_ms.clamp(100, 10_000)),
            pings: self.pings.clamp(1, 10),
            retries: self.retries.min(5),
        }
    }
}

/// ASE's Options > Events.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct EventSettings {
    /// Show a notice when a friend is found on a server.
    pub notify_friend_joins: bool,
    /// Play the system notification sound for friend events.
    pub sound_friend_joins: bool,
    /// Re-query the selected server periodically.
    pub auto_refresh_selected: bool,
    pub auto_refresh_seconds: u32,
}

impl Default for EventSettings {
    fn default() -> Self {
        Self {
            notify_friend_joins: true,
            sound_friend_joins: false,
            auto_refresh_selected: false,
            auto_refresh_seconds: 15,
        }
    }
}

/// Filter state of the left tree ("Modifiers" and per-game checkbox groups).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct FilterSettings {
    pub not_empty: bool,
    pub not_full: bool,
    pub no_password: bool,
    pub hide_offline: bool,
    /// `None` means no ping limit.
    pub max_ping: Option<u32>,
    /// Checked options, keyed by `game_id/group name`.
    pub checked: BTreeMap<String, BTreeSet<String>>,
    /// Country codes to show (`??` for unknown); empty shows all.
    pub countries: BTreeSet<String>,
    /// The tree lists only games the user can join (`Settings::ready_games`).
    pub installed_only: bool,
}

/// Toolbar button ids, in the default order.
pub const TOOLBAR_DEFAULT: [&str; 9] = [
    "refresh", "stop", "ping", "join", "favorite", "add", "find", "friends", "options",
];

/// One toolbar button: its id and whether it is shown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolbarItem {
    pub id: String,
    pub visible: bool,
}

/// Which toolbar buttons are shown, and in which order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ToolbarSettings {
    pub items: Vec<ToolbarItem>,
}

impl Default for ToolbarSettings {
    fn default() -> Self {
        Self {
            items: TOOLBAR_DEFAULT
                .iter()
                .map(|id| ToolbarItem {
                    id: (*id).to_string(),
                    visible: true,
                })
                .collect(),
        }
    }
}

impl ToolbarSettings {
    /// Drop unknown and duplicate ids and append the buttons missing from an
    /// older or hand-edited settings file.
    pub fn normalize(&mut self) {
        let mut seen = BTreeSet::new();
        self.items.retain(|item| {
            TOOLBAR_DEFAULT.contains(&item.id.as_str()) && seen.insert(item.id.clone())
        });
        for id in TOOLBAR_DEFAULT {
            if !seen.contains(id) {
                self.items.push(ToolbarItem {
                    id: id.to_string(),
                    visible: true,
                });
            }
        }
    }

    /// Ids of the visible buttons, in order.
    pub fn visible_ids(&self) -> Vec<&str> {
        self.items
            .iter()
            .filter(|item| item.visible)
            .map(|item| item.id.as_str())
            .collect()
    }

    /// Move an item one place up (`-1`) or down (`1`).
    pub fn move_item(&mut self, index: usize, delta: isize) {
        let Some(target) = index.checked_add_signed(delta) else {
            return;
        };
        if index < self.items.len() && target < self.items.len() {
            self.items.swap(index, target);
        }
    }
}

/// What to play when a friend is found.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FriendSound {
    /// Follow the global "play a sound" option, with the system sound.
    #[default]
    Default,
    /// Never play a sound for this friend.
    Silent,
    /// Play this .wav file (even when the global option is off).
    File(String),
}

/// A color as `[r, g, b]`.
pub type Rgb = [u8; 3];

/// Options > Colors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ColorSettings {
    pub list_bg: Rgb,
    pub alt_row: Rgb,
    pub selection: Rgb,
    pub text: Rgb,
    /// Draw Quake color codes in server and player names.
    pub quake_colors: bool,
    /// Base look; choosing one loads its colors above, which stay editable.
    pub theme: Theme,
}

/// 0 good (up to 80 ms), 1 medium (up to 150 ms), 2 bad: the index into
/// [`Theme::ping_colors`] (JASE's thresholds).
pub fn ping_level(ping: u32) -> usize {
    match ping {
        0..=80 => 0,
        81..=150 => 1,
        _ => 2,
    }
}

/// Look presets (from JASE's themes). Classic is the original ASE look.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    #[default]
    Classic,
    Dark,
    Terminal,
    Amber,
    Synthwave,
    /// The olive and gold of Half-Life 1 / Counter-Strike 1.6 menus.
    #[serde(rename = "goldsrc")]
    GoldSrc,
}

impl Theme {
    pub const ALL: [Theme; 6] = [
        Theme::Classic,
        Theme::Dark,
        Theme::Terminal,
        Theme::Amber,
        Theme::Synthwave,
        Theme::GoldSrc,
    ];

    /// English name, translated where shown.
    pub fn name(self) -> &'static str {
        match self {
            Theme::Classic => crate::i18n::noop("Classic"),
            Theme::Dark => crate::i18n::noop("Dark"),
            Theme::Terminal => crate::i18n::noop("Terminal"),
            Theme::Amber => crate::i18n::noop("Amber terminal"),
            Theme::Synthwave => crate::i18n::noop("Synthwave"),
            Theme::GoldSrc => crate::i18n::noop("GoldSrc (Counter-Strike 1.6)"),
        }
    }

    /// Built on egui's dark visuals.
    pub fn dark(self) -> bool {
        self != Theme::Classic
    }

    /// Window and panel background; `None` keeps egui's.
    pub fn panel(self) -> Option<Rgb> {
        match self {
            Theme::Classic => None,
            Theme::Dark => Some([0x16, 0x18, 0x1d]),
            Theme::Terminal => Some([0x05, 0x0a, 0x05]),
            Theme::Amber => Some([0x12, 0x0c, 0x02]),
            Theme::Synthwave => Some([0x1a, 0x10, 0x33]),
            Theme::GoldSrc => Some([0x4c, 0x58, 0x44]),
        }
    }

    /// Frames, separators and header lines; `None` keeps egui's.
    pub fn border(self) -> Option<Rgb> {
        match self {
            Theme::Classic => None,
            Theme::Dark => Some([0x2a, 0x2e, 0x37]),
            Theme::Terminal => Some([0x1d, 0x6b, 0x2c]),
            Theme::Amber => Some([0x5a, 0x3a, 0x00]),
            Theme::Synthwave => Some([0xff, 0x3f, 0xa4]),
            Theme::GoldSrc => Some([0x29, 0x2c, 0x21]),
        }
    }

    /// Ping colors (good, medium, bad), see [`ping_level`].
    pub fn ping_colors(self) -> [Rgb; 3] {
        match self {
            Theme::Classic => [[0x15, 0x80, 0x3d], [0xa1, 0x62, 0x07], [0xb9, 0x1c, 0x1c]],
            Theme::Dark => [[0x3e, 0xcf, 0x6e], [0xe5, 0xb9, 0x3a], [0xef, 0x5a, 0x5a]],
            Theme::Terminal => [[0x33, 0xff, 0x66], [0xe6, 0xff, 0x4d], [0xff, 0x6a, 0x4a]],
            Theme::Amber => [[0xb8, 0xe0, 0x60], [0xff, 0xd0, 0x60], [0xff, 0x6a, 0x3a]],
            Theme::Synthwave => [[0x3f, 0xf7, 0xff], [0xff, 0xe1, 0x4a], [0xff, 0x5c, 0x8a]],
            Theme::GoldSrc => [[0xa6, 0xd9, 0x6a], [0xe8, 0xc8, 0x60], [0xef, 0x6a, 0x5a]],
        }
    }

    /// Fill of the occupancy bar behind the Players column.
    pub fn occupancy(self) -> Rgb {
        match self {
            Theme::Classic => [0x60, 0xa5, 0xfa],
            Theme::Dark => [0x3b, 0x82, 0xf6],
            Theme::Terminal => [0x33, 0xff, 0x66],
            Theme::Amber => [0xff, 0xb0, 0x00],
            Theme::Synthwave => [0xff, 0x3f, 0xa4],
            Theme::GoldSrc => [0x95, 0x88, 0x31],
        }
    }

    /// Raised 3D edges of headers, (light, dark), as in Half-Life's VGUI.
    pub fn bevel(self) -> Option<(Rgb, Rgb)> {
        match self {
            Theme::GoldSrc => Some(([0x8c, 0x92, 0x84], [0x29, 0x2c, 0x21])),
            _ => None,
        }
    }

    /// Fill of buttons and other controls, (idle, hovered); `None` keeps egui's.
    pub fn widgets(self) -> Option<(Rgb, Rgb)> {
        match self {
            Theme::GoldSrc => Some(([0x5a, 0x66, 0x50], [0x6b, 0x77, 0x5f])),
            _ => None,
        }
    }

    /// Text on the selection; `None` keeps egui's.
    pub fn selection_text(self) -> Option<Rgb> {
        match self {
            Theme::Classic | Theme::Dark | Theme::Synthwave => None,
            Theme::GoldSrc => Some([0xff, 0xff, 0xff]),
            Theme::Terminal => Some([0x05, 0x0a, 0x05]),
            Theme::Amber => Some([0x12, 0x0c, 0x02]),
        }
    }

    /// The theme's starting colors.
    pub fn colors(self) -> ColorSettings {
        let (list_bg, alt_row, selection, text) = match self {
            Theme::Classic => return ColorSettings::default(),
            Theme::Dark => (
                [0x1d, 0x20, 0x27],
                [0x22, 0x26, 0x2e],
                [0x24, 0x32, 0x4a],
                [0xd7, 0xdb, 0xe2],
            ),
            Theme::Terminal => (
                [0x08, 0x10, 0x08],
                [0x0c, 0x18, 0x0c],
                [0x33, 0xff, 0x66],
                [0x33, 0xff, 0x66],
            ),
            Theme::Amber => (
                [0x16, 0x0f, 0x03],
                [0x1e, 0x15, 0x05],
                [0xff, 0xb0, 0x00],
                [0xff, 0xb0, 0x00],
            ),
            Theme::GoldSrc => (
                [0x3e, 0x46, 0x37],
                [0x45, 0x4e, 0x3d],
                [0x80, 0x75, 0x2a],
                [0xd8, 0xde, 0xd3],
            ),
            Theme::Synthwave => (
                [0x21, 0x14, 0x42],
                [0x29, 0x19, 0x50],
                [0x4a, 0x1f, 0x5e],
                [0xf3, 0xe6, 0xff],
            ),
        };
        ColorSettings {
            list_bg,
            alt_row,
            selection,
            text,
            quake_colors: true,
            theme: self,
        }
    }
}

impl Default for ColorSettings {
    fn default() -> Self {
        Self {
            list_bg: [0xff, 0xff, 0xff],
            alt_row: [0xf6, 0xf6, 0xf6],
            selection: [0x31, 0x6a, 0xc5],
            text: [0x50, 0x50, 0x50],
            quake_colors: true,
            theme: Theme::Classic,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub games: BTreeMap<String, GameSettings>,
    pub favorites: Vec<ServerRef>,
    pub recent: Vec<ServerRef>,
    pub friends: Vec<String>,
    pub steam_api_key: String,
    pub network: NetworkSettings,
    pub events: EventSettings,
    pub filters: FilterSettings,
    pub open_nav: BTreeSet<String>,
    /// Game selected at startup.
    pub last_game: String,
    /// Download the DB-IP country database for the Location column.
    pub geoip_disabled: bool,
    /// Favorite group of each favorite, keyed by [`ServerRef::key`].
    pub favorite_groups: BTreeMap<String, String>,
    /// Friend group of each friend, keyed by lowercase name.
    pub friend_groups: BTreeMap<String, String>,
    /// Game a friend is tracked in (empty = any), keyed by lowercase name.
    pub friend_games: BTreeMap<String, String>,
    /// Servers added by hand ("recently added").
    pub recently_added: Vec<ServerRef>,
    /// Servers from imported lists ("recently imported").
    pub recently_imported: Vec<ServerRef>,
    /// User filters, also used as color filters.
    pub custom_filters: Vec<CustomFilter>,
    /// Do not look for a new release at startup.
    pub skip_update_check: bool,
    /// The first-run setup wizard has been completed or skipped.
    pub wizard_done: bool,
    /// Visible toolbar buttons and their order.
    pub toolbar: ToolbarSettings,
    /// Sound per friend, keyed by lowercase name.
    pub friend_sounds: BTreeMap<String, FriendSound>,
    pub colors: ColorSettings,
    /// Interface language (System follows the operating system).
    pub language: crate::i18n::Language,
    /// Favorites of Unreal engine games saved with their game port have been
    /// moved to the query port once (see `query::resolve_query_port`).
    pub unreal_ports_checked: bool,
    /// "Alert when players reach": humans threshold, keyed by [`ServerRef::key`].
    pub party_alerts: BTreeMap<String, usize>,
    /// Places Auto-Search looked in last time (drives, folders); empty = all.
    pub search_roots: Vec<String>,
}

pub const RECENT_LIMIT: usize = 20;
pub const LIST_LIMIT: usize = 200;
pub const FAVORITE_GROUPS: [&str; 8] = [
    "Favorites1",
    "Favorites2",
    "Favorites3",
    "Favorites4",
    "Favorites5",
    "Favorites6",
    "Favorites7",
    "Favorites8",
];
pub const FRIEND_GROUPS: [&str; 4] = ["Friends1", "Friends2", "Friends3", "Friends4"];

impl ServerRef {
    /// Stable map key, `game|address`.
    pub fn key(&self) -> String {
        format!("{}|{}", self.game, self.address)
    }
}

/// Extract every `a.b.c.d:port` (or `host:port` line) from a server list file.
pub fn parse_address_list(text: &str) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for token in text.split(|c: char| {
        c.is_whitespace() || matches!(c, ',' | ';' | '"' | '\'' | '\\' | '/' | '=')
    }) {
        let token = token.trim();
        let Some((host, port)) = token.rsplit_once(':') else {
            continue;
        };
        // Any text file is scanned: a host name also needs a dot and a
        // letter, so tokens like `time:12` are not taken for servers.
        let valid_host = host.parse::<std::net::Ipv4Addr>().is_ok()
            || (crate::links::valid_host(host)
                && host.contains('.')
                && host.chars().any(|c| c.is_ascii_alphabetic()));
        if valid_host
            && port.parse::<u16>().is_ok_and(|port| port > 0)
            && seen.insert(token.to_string())
        {
            out.push(token.to_string());
        }
    }
    out
}

impl Settings {
    pub fn path() -> PathBuf {
        config_dir().join("settings.json")
    }

    pub fn load() -> Self {
        import_upstream(&upstream_config_dir(), &config_dir());
        Self::load_from(&Self::path())
    }

    /// Settings from `path`. A file that exists but does not parse is
    /// renamed to `<name>.bad` first, so the next save cannot overwrite the
    /// user's favorites and friends with defaults.
    pub fn load_from(path: &std::path::Path) -> Self {
        // Bytes, not a string: a file that is not UTF-8 (saved as ANSI by an
        // editor) must reach the `.bad` path too, not be silently replaced.
        let Ok(bytes) = std::fs::read(path) else {
            return Self::default();
        };
        match serde_json::from_slice::<Self>(&bytes) {
            Ok(settings) => settings.normalized(),
            Err(_) => {
                let mut bad = path.as_os_str().to_owned();
                bad.push(".bad");
                let _ = std::fs::rename(path, bad);
                Self::default()
            }
        }
    }

    fn normalized(mut self) -> Self {
        self.toolbar.normalize();
        self
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        self.export_to(&path)
    }

    /// Games the user can join: their program file exists (the same rule as
    /// Options > Games), or a Steam game and a Steam Web API key to list it.
    pub fn ready_games(&self) -> Vec<&'static crate::catalog::Game> {
        crate::catalog::games()
            .iter()
            .filter(|game| {
                let program = self
                    .games
                    .get(&game.id)
                    .is_some_and(|g| std::path::Path::new(g.program_location.trim()).is_file());
                program || (game.steam_appid.is_some() && !self.steam_api_key.trim().is_empty())
            })
            .collect()
    }

    pub fn game(&self, id: &str) -> GameSettings {
        self.games.get(id).cloned().unwrap_or_default()
    }

    /// Like [`Settings::game`] without the clone, for code that runs every
    /// frame (tree, list rows, panes).
    pub fn game_ref(&self, id: &str) -> &GameSettings {
        static DEFAULT: std::sync::LazyLock<GameSettings> =
            std::sync::LazyLock::new(GameSettings::default);
        self.games.get(id).unwrap_or(&DEFAULT)
    }

    pub fn is_favorite(&self, server: &ServerRef) -> bool {
        self.favorites.contains(server)
    }

    pub fn toggle_favorite(&mut self, server: ServerRef) {
        if let Some(index) = self.favorites.iter().position(|entry| *entry == server) {
            self.favorites.remove(index);
        } else {
            self.favorites.push(server);
        }
    }

    pub fn add_recent(&mut self, server: ServerRef) {
        self.recent.retain(|entry| *entry != server);
        self.recent.insert(0, server);
        self.recent.truncate(RECENT_LIMIT);
    }

    pub fn favorite_group(&self, server: &ServerRef) -> String {
        self.favorite_groups
            .get(&server.key())
            .cloned()
            .unwrap_or_else(|| FAVORITE_GROUPS[0].to_string())
    }

    /// Add to favorites (if needed) and put in `group`.
    pub fn add_favorite_to(&mut self, server: ServerRef, group: &str) {
        self.favorite_groups.insert(server.key(), group.to_string());
        if !self.favorites.contains(&server) {
            self.favorites.push(server);
        }
    }

    pub fn favorites_in(&self, group: &str) -> Vec<ServerRef> {
        self.favorites
            .iter()
            .filter(|server| self.favorite_group(server) == group)
            .cloned()
            .collect()
    }

    pub fn friend_group(&self, friend: &str) -> String {
        self.friend_groups
            .get(&friend.to_lowercase())
            .cloned()
            .unwrap_or_else(|| FRIEND_GROUPS[0].to_string())
    }

    pub fn friend_game(&self, friend: &str) -> String {
        self.friend_games
            .get(&friend.to_lowercase())
            .cloned()
            .unwrap_or_default()
    }

    pub fn friend_sound(&self, friend: &str) -> FriendSound {
        self.friend_sounds
            .get(&friend.to_lowercase())
            .cloned()
            .unwrap_or_default()
    }

    /// Point every list that holds `old` (favorites and their group, recent,
    /// recently added and imported) at `new` instead.
    pub fn replace_server(&mut self, old: &ServerRef, new: &ServerRef) {
        if let Some(group) = self.favorite_groups.remove(&old.key()) {
            self.favorite_groups.insert(new.key(), group);
        }
        for list in [
            &mut self.favorites,
            &mut self.recent,
            &mut self.recently_added,
            &mut self.recently_imported,
        ] {
            let already = list.contains(new);
            list.retain(|entry| !(already && entry == old));
            for entry in list.iter_mut().filter(|entry| *entry == old) {
                *entry = new.clone();
            }
        }
    }

    /// Prepend to a bounded "recently ..." list.
    pub fn push_front(list: &mut Vec<ServerRef>, server: ServerRef) {
        list.retain(|entry| *entry != server);
        list.insert(0, server);
        list.truncate(LIST_LIMIT);
    }

    /// Save to any path (File > Export user profile).
    pub fn export_to(&self, path: &std::path::Path) -> std::io::Result<()> {
        let text = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        // Write then rename, so a crash never leaves half a file.
        let mut partial = path.as_os_str().to_owned();
        partial.push(".partial");
        // Flushed to disk before the rename, or a power cut could leave the
        // renamed file empty.
        let mut file = std::fs::File::create(&partial)?;
        std::io::Write::write_all(&mut file, text.as_bytes())?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&partial, path)
    }

    /// Read a profile written by [`Settings::export_to`].
    pub fn import_from(path: &std::path::Path) -> std::io::Result<Self> {
        let text = std::fs::read_to_string(path)?;
        serde_json::from_str::<Self>(&text)
            .map(Self::normalized)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))
    }
}

/// Folder holding OpenASE-Plus's settings.
pub fn config_dir() -> PathBuf {
    config_base().join("OpenASE-Plus")
}

/// Folder of upstream OpenASE's settings, imported once by [`import_upstream`].
pub fn upstream_config_dir() -> PathBuf {
    config_base().join("OpenASE")
}

/// Copy upstream OpenASE's `settings.json` (favorites, friends, options)
/// into a new OpenASE-Plus folder, once: never over an existing profile.
pub fn import_upstream(old: &std::path::Path, new: &std::path::Path) -> bool {
    let (from, to) = (old.join("settings.json"), new.join("settings.json"));
    if to.exists() || !from.is_file() {
        return false;
    }
    std::fs::create_dir_all(new).is_ok() && std::fs::copy(&from, &to).is_ok()
}

fn config_base() -> PathBuf {
    let base = if cfg!(windows) {
        std::env::var_os("APPDATA").map(PathBuf::from)
    } else if cfg!(target_os = "macos") {
        std::env::var_os("HOME").map(|home| PathBuf::from(home).join("Library/Application Support"))
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
    };
    base.unwrap_or_else(|| PathBuf::from("."))
}
