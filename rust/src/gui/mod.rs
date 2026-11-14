//! Desktop GUI, laid out like The All-Seeing Eye: filter tree on the left,
//! server list on the right, friends / players / rules panes at the bottom.

mod dialogs;
mod extra;
mod icons;
mod list;
mod panes;
mod scan;
mod tree;

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::path::Path;

use eframe::emath::GuiRounding;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use eframe::egui::{
    self, Align, Color32, CornerRadius, FontId, Layout, RichText, Sense, Stroke, TextFormat,
    TextStyle, text::LayoutJob, vec2,
};
use egui_extras::{Column, TableBuilder};

use openase::alerts::{self, PartyTracker, SlotWatch};
use openase::catalog::{self, Game, Protocol};
use openase::colors::{parse_hex, quake_color_segments, strip_quake_colors};
use openase::countries::country_name;
use openase::detect;
use openase::geoip::{self, GeoIp};
use openase::i18n::{self, fill, group_name, noop, tr};
use openase::model::{Listed, Server};
use openase::networks333::{self, PAGE_SIZE};
use openase::quake3::parse_address;
use openase::query::{fetch_all_masters, fetch_more, query_server};
use openase::settings::{
    ColorSettings, FAVORITE_GROUPS, FRIEND_GROUPS, FriendSound, Rgb, ServerRef, Settings,
};

use dialogs::Dialogs;
use icons::{Tool, TreeIcon};

/// Row height of the lists: ASE's compact rows in Classic, JASE's taller
/// ones in the other themes.
const ROW_HEIGHT: f32 = 18.0;
const ROW_HEIGHT_TALL: f32 = 24.0;
const PING_LIMITS: [u32; 4] = [50, 100, 150, 250];
/// Most servers "Ping visible servers" queries at once.
const PING_VISIBLE_MAX: usize = 60;

/// Identifies a server across games.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ServerKey {
    pub game: String,
    pub address: String,
}

impl ServerKey {
    fn of(server: &Server) -> Self {
        Self {
            game: server.game.clone(),
            address: server.address(),
        }
    }

    fn to_ref(&self) -> ServerRef {
        ServerRef {
            game: self.game.clone(),
            address: self.address.clone(),
        }
    }

    fn from_ref(server: &ServerRef) -> Self {
        Self {
            game: server.game.clone(),
            address: server.address.clone(),
        }
    }
}

/// What the left tree has selected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Nav {
    Favorites,
    FavoriteGroup(String),
    Recent,
    Friends,
    FriendGroup(String),
    Special(Special),
    Custom(usize),
    Game(String),
    GameType(String, String),
}

/// ASE's "Special" folder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Special {
    Added,
    Imported,
    Lan,
    /// Busy servers of every game the user can join.
    Playing,
}

impl Special {
    const ALL: [Special; 4] = [
        Special::Playing,
        Special::Added,
        Special::Imported,
        Special::Lan,
    ];

    fn label(self) -> &'static str {
        match self {
            Special::Added => tr("recently added"),
            Special::Imported => tr("recently imported"),
            Special::Lan => tr("LAN servers"),
            Special::Playing => tr("Playing now"),
        }
    }
}

impl Nav {
    fn game(&self) -> Option<&str> {
        match self {
            Nav::Game(game) | Nav::GameType(game, _) => Some(game),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Col {
    Icon,
    Location,
    Name,
    Ping,
    Flux,
    Loss,
    Players,
    GameType,
    Map,
    Address,
}

const COLUMNS: [(Col, &str, f32); 10] = [
    (Col::Icon, "", 44.0),
    (Col::Location, noop("Location"), 40.0),
    (Col::Name, noop("Name"), 300.0),
    (Col::Ping, noop("Ping"), 46.0),
    (Col::Flux, noop("Flux"), 40.0),
    (Col::Loss, noop("PL"), 38.0),
    (Col::Players, noop("Players"), 84.0),
    (Col::GameType, noop("Game"), 120.0),
    (Col::Map, noop("Map"), 120.0),
    (Col::Address, noop("Address"), 150.0),
];

/// Messages from worker threads.
pub enum Msg {
    MasterList {
        generation: u64,
        game: String,
        /// Listed servers; those with a preview are not queried by the scan.
        servers: Vec<Listed>,
        errors: Vec<String>,
        /// Servers the paged masters (333networks) list in all.
        paged_total: Option<usize>,
    },
    /// Next page of a game's 333networks list ("Load more").
    MorePage {
        generation: u64,
        game: String,
        page: usize,
        result: Result<networks333::Page, String>,
    },
    Server {
        generation: u64,
        server: Server,
    },
    ScanDone {
        generation: u64,
    },
    Single(Server),
    SearchProgress(String),
    SearchFound(String),
    /// "Search all games...": an executable and the game it may belong to.
    SearchHit {
        game: String,
        path: String,
        sure: bool,
    },
    SearchDone,
    GeoIp(Result<Arc<GeoIp>, String>),
    Lan(Result<Vec<openase::lan::LanServer>, String>),
    Update {
        result: Result<Option<String>, String>,
        manual: bool,
    },
    /// Saved servers to move from a game port to their query port.
    PortFixes(Vec<(ServerRef, ServerRef)>),
    /// Lists saved by an earlier session (`openase::lists`).
    SavedLists(Vec<(String, Vec<Server>)>),
    /// Icon of a game's executable (`None`: not a PE or no icon).
    ExeIcon {
        game: String,
        path: String,
        image: Option<([usize; 2], Vec<u8>)>,
    },
    /// A levelshot looked up for (game, map); `None` when there is none.
    MapImage {
        game: String,
        map: String,
        image: Option<([usize; 2], Vec<u8>)>,
    },
    /// Answer of an RCON command.
    Rcon {
        key: ServerKey,
        result: Result<String, openase::rcon::RconError>,
    },
    /// An `openase://` link, from the command line or a second instance.
    Link(String),
    /// Answer of a server with a slot or party alert.
    Watched(Server),
    /// The query of watched servers is over.
    WatchDone,
    /// Addresses from Add server (`group` None) or Import list, moved to
    /// their query port off the UI thread.
    Resolved {
        game: String,
        addresses: Vec<String>,
        group: Option<String>,
    },
}

struct Scan {
    generation: u64,
    stop: Arc<AtomicBool>,
    done: Arc<AtomicUsize>,
    total: usize,
    label: String,
    /// Games whose whole list this scan fetched: saved when it completes.
    games: Vec<String>,
}

/// Pages loaded of a game's 333networks list.
struct Paging {
    /// Scan that loaded the first page; later pages of older scans are dropped.
    generation: u64,
    /// Pages loaded so far.
    pages: usize,
    /// Rows added from the pages.
    listed: usize,
    /// Servers listed in all.
    total: usize,
}

impl Paging {
    fn has_more(&self) -> bool {
        self.pages * PAGE_SIZE < self.total
    }
}

/// Row actions collected while drawing, applied afterwards.
enum Action {
    Select(ServerKey),
    /// Start or stop watching a full server for a free slot.
    WatchSlot(ServerKey),
    /// Party alert threshold for a server; 0 removes it.
    PartyAlert(ServerKey, usize),
    Rcon(ServerKey),
    Join(ServerKey),
    ToggleFavorite(ServerKey),
    Refresh(ServerKey),
    LoadMore(String),
    Copy(String),
    FavoriteGroup(ServerKey, String),
}

pub struct OpenAseApp {
    pub(crate) settings: Settings,
    settings_dirty: bool,
    pub(crate) tx: Sender<Msg>,
    rx: Receiver<Msg>,
    pub(crate) store: BTreeMap<String, Vec<Server>>,
    nav: Nav,
    pub(crate) selected: Option<ServerKey>,
    sort: (Col, bool),
    scan: Option<Scan>,
    generation: u64,
    pub(crate) status: String,
    friend_seen: HashMap<String, ServerKey>,
    /// Map images by (game, map): `None` = looked up, none found. A key
    /// without an entry has not been looked up yet.
    map_images: HashMap<(String, String), Option<egui::TextureHandle>>,
    flags: Option<icons::Flags>,
    /// Ids of `Settings::ready_games` for the tree filter; `None` = stale.
    ready_cache: Option<Vec<String>>,
    /// Executable icons by game id: (program path it came from, icon).
    exe_icons: HashMap<String, (String, Option<egui::TextureHandle>)>,
    /// "Alert me when a slot is free" on one full server.
    slot_watch: Option<SlotWatch>,
    party: PartyTracker,
    /// A query of watched servers (slot or party alerts) is running.
    watch_in_flight: bool,
    last_party_poll: Option<Instant>,
    single_in_flight: bool,
    last_auto_refresh: Instant,
    show_toolbar: bool,
    /// Colors currently applied (Options > Colors previews them live).
    palette: ColorSettings,
    started: bool,
    geoip: Option<Arc<GeoIp>>,
    pub(crate) geoip_status: String,
    lan_servers: Vec<ServerKey>,
    scroll_to_selected: bool,
    /// Paging state of games listed by 333networks.
    paging: HashMap<String, Paging>,
    /// Game whose next page is being downloaded (one request at a time).
    more_in_flight: Option<String>,
    /// Rows drawn in the server list on the last frame.
    on_screen: Vec<ServerKey>,
    /// Scroll offset of the server list on the last frame.
    list_offset: f32,
    /// The "Load more" row was drawn on the last frame.
    more_row_drawn: bool,
    pub(crate) extra: extra::Extra,
    pub(crate) dialogs: Dialogs,
}

impl OpenAseApp {
    /// `link`: an `openase://` link from the command line.
    pub fn new(cc: &eframe::CreationContext<'_>, link: Option<String>) -> Self {
        setup_style(&cc.egui_ctx);
        let (tx, rx) = channel();
        // Become the instance that later `openase://` links are handed to.
        let (link_tx, link_ctx) = (tx.clone(), cc.egui_ctx.clone());
        let _ = openase::instance::listen(&openase::instance::port_file(), move |url| {
            let _ = link_tx.send(Msg::Link(url));
            link_ctx.request_repaint();
        });
        if let Some(url) = link {
            let _ = tx.send(Msg::Link(url));
        }
        let (lists_tx, lists_ctx) = (tx.clone(), cc.egui_ctx.clone());
        std::thread::spawn(move || {
            let lists = openase::lists::load_all(&openase::lists::dir());
            let _ = lists_tx.send(Msg::SavedLists(lists));
            lists_ctx.request_repaint();
        });
        let mut settings = Settings::load();
        i18n::set_language(settings.language);
        apply_palette(&cc.egui_ctx, &settings.colors);

        // Quick registry detection for games without a program location.
        for game in catalog::games() {
            let entry = settings.games.entry(game.id.clone()).or_default();
            if entry.program_location.is_empty()
                && let Some(path) = detect::detect_from_registry(game)
            {
                entry.program_location = path.display().to_string();
            }
        }
        if settings.open_nav.is_empty() {
            settings
                .open_nav
                .extend(["Modifiers".to_string(), "Modifiers/Players".to_string()]);
        }
        let first_game = catalog::game(&settings.last_game)
            .map(|game| game.id.clone())
            .unwrap_or_else(|| catalog::games()[0].id.clone());

        let palette = settings.colors.clone();
        Self {
            settings,
            settings_dirty: true,
            tx,
            rx,
            store: BTreeMap::new(),
            nav: Nav::Game(first_game),
            selected: None,
            sort: (Col::Ping, true),
            scan: None,
            generation: 0,
            status: tr("Ready").to_string(),
            friend_seen: HashMap::new(),
            slot_watch: None,
            map_images: HashMap::new(),
            flags: icons::Flags::load(&cc.egui_ctx),
            ready_cache: None,
            exe_icons: HashMap::new(),
            party: PartyTracker::default(),
            watch_in_flight: false,
            last_party_poll: None,
            single_in_flight: false,
            last_auto_refresh: Instant::now(),
            show_toolbar: true,
            palette,
            started: false,
            geoip: None,
            geoip_status: tr("Loading...").to_string(),
            lan_servers: Vec::new(),
            scroll_to_selected: false,
            paging: HashMap::new(),
            more_in_flight: None,
            on_screen: Vec::new(),
            list_offset: 0.0,
            more_row_drawn: false,
            extra: extra::Extra::default(),
            dialogs: Dialogs::default(),
        }
    }

    // ----- data ------------------------------------------------------------

    pub(crate) fn find(&self, key: &ServerKey) -> Option<&Server> {
        self.store
            .get(&key.game)?
            .iter()
            .find(|server| server.address() == key.address)
    }

    fn upsert(&mut self, mut server: Server) {
        let address = server.address();
        if server.country.is_none() {
            // Keep the country the master list gave when GeoIP is off.
            server.country = self
                .store
                .get(&server.game)
                .and_then(|list| list.iter().find(|existing| existing.address() == address))
                .and_then(|existing| existing.country.clone());
        }
        annotate(self.geoip.as_deref(), &mut server);
        self.check_friends(&server);
        self.check_alerts(&server);
        let list = self.store.entry(server.game.clone()).or_default();
        match list
            .iter_mut()
            .find(|existing| existing.address() == address)
        {
            // No answer from a server the list described: keep its details
            // and show the loss instead of an empty "no response" row.
            Some(existing) if !server.online && existing.listed => {
                existing.packet_loss = Some(100);
                existing.error = server.error;
            }
            Some(existing) => *existing = server,
            None => list.push(server),
        }
    }

    /// Query a server listed with details (333networks) and not queried yet.
    fn query_if_listed(&mut self, ctx: &egui::Context, key: &ServerKey) {
        if self.find(key).is_some_and(|server| server.listed) {
            self.query_single(ctx, key);
        }
    }

    /// Make sure a remembered server has at least an "unscanned" row.
    fn ensure_placeholder(&mut self, key: &ServerKey) {
        if self.find(key).is_some() {
            return;
        }
        let Some(game) = catalog::game(&key.game) else {
            return;
        };
        if let Ok((host, port)) = parse_address(&key.address, game.default_port) {
            let mut server = Server::new(&key.game, &host, port);
            annotate(self.geoip.as_deref(), &mut server);
            self.store.entry(key.game.clone()).or_default().push(server);
        }
    }

    fn check_friends(&mut self, server: &Server) {
        let key = ServerKey::of(server);
        for friend in &self.settings.friends {
            let needle = friend.to_lowercase();
            let game = self.settings.friend_game(friend);
            let present = server.online
                && (game.is_empty() || game == server.game)
                && server.players.iter().any(|player| {
                    strip_quake_colors(&player.name)
                        .to_lowercase()
                        .contains(&needle)
                });
            let previous = self.friend_seen.get(&needle).cloned();
            if present {
                if previous.as_ref() != Some(&key) {
                    self.friend_seen.insert(needle.clone(), key.clone());
                    if self.settings.events.notify_friend_joins {
                        self.dialogs.notify(fill(
                            tr("{} is playing on {}"),
                            &[friend, &strip_quake_colors(&server.name)],
                        ));
                    }
                    play_friend_sound(
                        &self.settings.friend_sound(friend),
                        self.settings.events.sound_friend_joins,
                    );
                }
            } else if previous.as_ref() == Some(&key) {
                self.friend_seen.remove(&needle);
            }
        }
    }

    fn check_alerts(&mut self, server: &Server) {
        let key = ServerKey::of(server);
        let id = key.to_ref().key();
        let name = strip_quake_colors(&server.name);
        if self
            .slot_watch
            .as_ref()
            .is_some_and(|watch| watch.server == key.to_ref())
            && alerts::has_free_slot(server)
        {
            self.slot_watch = None;
            self.dialogs.notify(fill(
                tr("A slot is free on {} ({}/{})"),
                &[&name, &server.num_players, &server.max_players.unwrap_or(0)],
            ));
            self.selected = Some(key);
            self.scroll_to_selected = true;
        }
        if let Some(&threshold) = self.settings.party_alerts.get(&id)
            && self.party.update(&id, server, threshold)
        {
            self.dialogs.notify(fill(
                tr("A match is on at {}: {} players"),
                &[&name, &alerts::humans(server)],
            ));
        }
    }

    /// Query the servers with a slot watch (every 10 s) or a party alert
    /// (every minute), one batch at a time.
    fn watch_tick(&mut self, ctx: &egui::Context) {
        let now = Instant::now();
        if let Some(watch) = &self.slot_watch
            && watch.expired(now)
        {
            let address = watch.server.address.clone();
            self.slot_watch = None;
            self.status = fill(
                tr("Stopped watching {}: no free slot in 30 minutes"),
                &[&address],
            );
        }
        if self.watch_in_flight {
            return;
        }
        let mut targets: Vec<ServerRef> = Vec::new();
        if let Some(watch) = &mut self.slot_watch
            && watch.due(now)
        {
            watch.queried(now);
            targets.push(watch.server.clone());
        }
        if !self.settings.party_alerts.is_empty()
            && self
                .last_party_poll
                .is_none_or(|last| now >= last + alerts::PARTY_INTERVAL)
        {
            self.last_party_poll = Some(now);
            for server in self
                .settings
                .party_alerts
                .keys()
                .filter_map(|k| ServerRef::from_key(k))
            {
                if !targets.contains(&server) {
                    targets.push(server);
                }
            }
        }
        if targets.is_empty() {
            return;
        }
        self.watch_in_flight = true;
        let tx = self.tx.clone();
        let ctx = ctx.clone();
        let options = self.settings.network.query_options();
        std::thread::spawn(move || {
            for server in targets {
                let Some(game) = catalog::game(&server.game) else {
                    continue;
                };
                if let Ok((host, port)) = parse_address(&server.address, game.default_port) {
                    let _ = tx.send(Msg::Watched(query_server(game, &host, port, &options)));
                }
            }
            let _ = tx.send(Msg::WatchDone);
            ctx.request_repaint();
        });
    }

    /// Keep the lists of a completed scan for the next session (off the UI
    /// thread: a Steam list is megabytes).
    fn save_lists(&self, games: Vec<String>) {
        let lists: Vec<(String, Vec<Server>)> = games
            .into_iter()
            .filter_map(|game| Some((game.clone(), self.store.get(&game)?.clone())))
            .collect();
        if lists.is_empty() {
            return;
        }
        std::thread::spawn(move || {
            for (game, servers) in lists {
                let _ = openase::lists::save(&openase::lists::dir(), &game, &servers);
            }
        });
    }

    /// Themes other than Classic follow JASE: taller rows, centered
    /// headers, no stripes.
    fn jase_look(&self) -> bool {
        self.palette.theme != openase::settings::Theme::Classic
    }

    fn row_height(&self) -> f32 {
        if self.jase_look() {
            ROW_HEIGHT_TALL
        } else {
            ROW_HEIGHT
        }
    }

    /// The filter bar under the tree (JASE's left column): the same
    /// settings as the tree's Modifiers.
    fn filter_bar(&mut self, ui: &mut egui::Ui) {
        // Edited as locals, so the country list can be built inside its combo
        // (only while it is open) with `&self`.
        let f = &self.settings.filters;
        let before = (
            f.not_empty,
            f.not_full,
            f.no_password,
            f.hide_offline,
            f.max_ping,
            f.countries.clone(),
        );
        let (
            mut not_empty,
            mut not_full,
            mut no_password,
            mut hide_offline,
            mut max_ping,
            mut countries,
        ) = before.clone();
        ui.checkbox(&mut not_empty, tr("Hide empty"));
        ui.checkbox(&mut not_full, tr("Hide full"));
        ui.checkbox(&mut no_password, tr("Hide passworded"));
        ui.checkbox(&mut hide_offline, tr("Hide unresponsive"));
        egui::Grid::new("filter-bar")
            .num_columns(2)
            .spacing([6.0, 4.0])
            .show(ui, |ui| {
                ui.label(tr("Max ping"));
                let shown = max_ping.map_or_else(|| tr("No limit").to_string(), |p| p.to_string());
                egui::ComboBox::from_id_salt("max-ping")
                    .selected_text(shown)
                    .width(120.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut max_ping, None, tr("No limit"));
                        for limit in [50, 100, 150, 250] {
                            ui.selectable_value(&mut max_ping, Some(limit), limit.to_string());
                        }
                    });
                ui.end_row();
                ui.label(tr("Country"));
                let shown = match countries.iter().next() {
                    Some(code) if countries.len() == 1 => country_label(code),
                    Some(_) => fill(tr("{} countries"), &[&countries.len()]),
                    None => tr("All countries").to_string(),
                };
                egui::ComboBox::from_id_salt("country")
                    .selected_text(shown)
                    .width(120.0)
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_label(countries.is_empty(), tr("All countries"))
                            .clicked()
                        {
                            countries.clear();
                        }
                        for (code, label, count) in self.country_options() {
                            let only = countries.len() == 1 && countries.contains(&code);
                            if ui
                                .selectable_label(only, format!("{label} ({count})"))
                                .clicked()
                            {
                                countries = [code].into();
                            }
                        }
                    });
                ui.end_row();
            });
        let after = (
            not_empty,
            not_full,
            no_password,
            hide_offline,
            max_ping,
            countries,
        );
        if before != after {
            let f = &mut self.settings.filters;
            (
                f.not_empty,
                f.not_full,
                f.no_password,
                f.hide_offline,
                f.max_ping,
                f.countries,
            ) = after;
            self.settings_dirty = true;
        }
    }

    fn exe_icon(&self, game: &str) -> Option<&egui::TextureHandle> {
        self.exe_icons.get(game)?.1.as_ref()
    }

    /// Read the icons of configured game executables once per path, off
    /// the UI thread; games without one keep their badge.
    fn request_exe_icons(&mut self, ctx: &egui::Context) {
        let mut wanted = Vec::new();
        for game in catalog::games() {
            // By reference: this runs every frame for every game.
            let program = self
                .settings
                .games
                .get(&game.id)
                .map_or("", |g| g.program_location.trim());
            let known = self.exe_icons.get(&game.id).map(|(path, _)| path.as_str());
            if program.is_empty() {
                if known.is_some() {
                    self.exe_icons.remove(&game.id);
                }
            } else if known != Some(program) {
                let program = program.to_string();
                self.exe_icons
                    .insert(game.id.clone(), (program.clone(), None));
                wanted.push((game.id.clone(), program));
            }
        }
        if wanted.is_empty() {
            return;
        }
        let (tx, ctx) = (self.tx.clone(), ctx.clone());
        std::thread::spawn(move || {
            for (game, path) in wanted {
                let image = openase::exe_icon::icon_from_exe(Path::new(&path))
                    .and_then(|ico| openase::exe_icon::decode_icon(&ico));
                let _ = tx.send(Msg::ExeIcon { game, path, image });
            }
            ctx.request_repaint();
        });
    }

    /// Look up the levelshot of the selected server's map once, off the UI
    /// thread, in the game's own `.pk3` files (the server's mod first).
    fn request_map_image(&mut self, ctx: &egui::Context) {
        let Some(server) = self.selected.as_ref().and_then(|key| self.find(key)) else {
            return;
        };
        let Some(game) = catalog::game(&server.game).filter(|g| !g.levelshots.is_empty()) else {
            return;
        };
        let key = (game.id.clone(), server.map.clone());
        if server.map.is_empty() || self.map_images.contains_key(&key) {
            return;
        }
        let program = self.settings.game(&game.id).program_location;
        let Some(root) = Path::new(program.trim())
            .parent()
            .filter(|_| !program.trim().is_empty())
        else {
            return;
        };
        let mut dirs = Vec::new();
        let mod_dir = game.launch_mod(&server.mod_name);
        if !mod_dir.is_empty() {
            dirs.push(root.join(mod_dir));
        }
        dirs.push(root.join(&game.levelshots));
        // Marked as looked up now, so one lookup runs per map.
        self.map_images.insert(key.clone(), None);
        let (tx, ctx) = (self.tx.clone(), ctx.clone());
        std::thread::spawn(move || {
            let image = openase::maps::find_levelshot(&dirs, &key.1)
                .and_then(|bytes| openase::maps::decode(&bytes));
            if image.is_some() {
                let (game, map) = key;
                let _ = tx.send(Msg::MapImage { game, map, image });
                ctx.request_repaint();
            }
        });
    }

    /// Run one RCON command off the UI thread; the answer comes as `Msg::Rcon`.
    pub(crate) fn send_rcon(
        &self,
        ctx: &egui::Context,
        key: ServerKey,
        password: String,
        command: String,
    ) {
        let Some(game) = catalog::game(&key.game) else {
            return;
        };
        // RCON goes to the game port (the join address).
        let address = self
            .find(&key)
            .map_or(key.address.clone(), Server::join_address);
        let (tx, ctx) = (self.tx.clone(), ctx.clone());
        std::thread::spawn(move || {
            let result = match parse_address(&address, game.default_port) {
                Ok((host, port)) => openase::rcon::send(game, &host, port, &password, &command),
                Err(error) => Err(openase::rcon::RconError::Network(error.0)),
            };
            let _ = tx.send(Msg::Rcon { key, result });
            ctx.request_repaint();
        });
    }

    /// Show the server of an `openase://` link and ask before joining.
    fn open_link(&mut self, ctx: &egui::Context, url: &str) {
        let Some((game, address)) = openase::links::parse(url) else {
            return;
        };
        let key = ServerKey { game, address };
        self.ensure_placeholder(&key);
        self.nav = Nav::Game(key.game.clone());
        self.selected = Some(key.clone());
        self.scroll_to_selected = true;
        self.query_single(ctx, &key);
        self.dialogs.link = Some(key);
        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
    }

    pub(crate) fn friend_location(&self, friend: &str) -> Option<&ServerKey> {
        self.friend_seen.get(&friend.to_lowercase())
    }

    fn server_has_friend(&self, server: &Server) -> bool {
        let key = ServerKey::of(server);
        self.friend_seen.values().any(|seen| *seen == key)
    }

    // ----- actions ---------------------------------------------------------

    pub(crate) fn join(&mut self, key: &ServerKey) {
        let Some(server) = self.find(key) else { return };
        if server.password {
            self.dialogs.password = Some((key.clone(), String::new()));
        } else {
            self.join_with_password(key, "");
        }
    }

    pub(crate) fn join_with_password(&mut self, key: &ServerKey, password: &str) {
        let (Some(server), Some(game)) = (self.find(key).cloned(), catalog::game(&key.game)) else {
            return;
        };
        let settings = self.settings.game(&key.game);
        match openase::launch::join(game, &settings, &server, password) {
            Ok(launched) => {
                self.settings.add_recent(key.to_ref());
                self.settings_dirty = true;
                self.status = match launched {
                    openase::launch::Launched::Program(program) => {
                        fill(tr("Launched {}"), &[&program])
                    }
                    openase::launch::Launched::SteamUrl(url) => fill(tr("Opened {}"), &[&url]),
                };
            }
            Err(error) => self.dialogs.error = Some(error.to_string()),
        }
    }

    fn apply(&mut self, ctx: &egui::Context, action: Action) {
        match action {
            Action::Select(key) => {
                if self.selected.as_ref() != Some(&key) {
                    self.query_if_listed(ctx, &key);
                }
                self.selected = Some(key);
            }
            Action::Join(key) => {
                // A double click selected (and so queried) the row first.
                if self.selected.as_ref() != Some(&key) {
                    self.query_if_listed(ctx, &key);
                }
                self.selected = Some(key.clone());
                self.join(&key);
            }
            Action::LoadMore(game) => self.load_more(ctx, &game),
            Action::ToggleFavorite(key) => {
                self.settings.toggle_favorite(key.to_ref());
                self.settings_dirty = true;
            }
            Action::Refresh(key) => self.query_single(ctx, &key),
            Action::FavoriteGroup(key, group) => {
                self.settings.add_favorite_to(key.to_ref(), &group);
                self.settings_dirty = true;
                self.status = fill(tr("Added to {}"), &[&group_name(&group)]);
            }
            Action::Rcon(key) => {
                self.dialogs.rcon = Some(dialogs::RconConsole {
                    key,
                    command: String::new(),
                    output: String::new(),
                    in_flight: false,
                });
            }
            Action::WatchSlot(key) => {
                if self
                    .slot_watch
                    .as_ref()
                    .is_some_and(|watch| watch.server == key.to_ref())
                {
                    self.slot_watch = None;
                } else {
                    self.slot_watch = Some(SlotWatch::new(key.to_ref(), Instant::now()));
                    self.status = fill(tr("Watching {} for a free slot..."), &[&key.address]);
                }
            }
            Action::PartyAlert(key, threshold) => {
                let id = key.to_ref().key();
                self.party.reset(&id);
                if threshold == 0 {
                    self.settings.party_alerts.remove(&id);
                    self.status = fill(tr("No more match alerts for {}"), &[&key.address]);
                } else {
                    self.settings.party_alerts.insert(id, threshold);
                    self.last_party_poll = None;
                    self.status = fill(
                        tr("I will alert you when {} has {} players"),
                        &[&key.address, &threshold],
                    );
                }
                self.settings_dirty = true;
            }
            Action::Copy(text) => {
                ctx.copy_text(text.clone());
                self.status = fill(tr("Copied {}"), &[&text]);
            }
        }
    }

    pub(crate) fn mark_dirty(&mut self) {
        self.settings_dirty = true;
    }

    pub(crate) fn add_server(
        &mut self,
        ctx: &egui::Context,
        game_id: &str,
        address: &str,
    ) -> Result<(), String> {
        let game = catalog::game(game_id).ok_or_else(|| tr("Unknown game").to_string())?;
        let (host, port) =
            parse_address(address.trim(), game.default_port).map_err(|error| error.to_string())?;
        self.resolve_in_background(ctx, &game.id, vec![format!("{host}:{port}")], None);
        Ok(())
    }

    /// Players share the game port of Unreal engine servers: move typed or
    /// imported addresses to their query port in a worker (it sends a query),
    /// so OpenASE never queries the game port. Answers with `Msg::Resolved`.
    pub(crate) fn resolve_in_background(
        &self,
        ctx: &egui::Context,
        game: &str,
        addresses: Vec<String>,
        group: Option<String>,
    ) {
        let timeout = self.settings.network.query_options().timeout;
        let (tx, ctx, game) = (self.tx.clone(), ctx.clone(), game.to_string());
        std::thread::spawn(move || {
            let addresses = extra::resolve_query_ports(&game, addresses, timeout);
            let _ = tx.send(Msg::Resolved {
                game,
                addresses,
                group,
            });
            ctx.request_repaint();
        });
    }

    fn finish_add(&mut self, ctx: &egui::Context, game: &str, address: &str) {
        let key = ServerKey {
            game: game.to_string(),
            address: address.to_string(),
        };
        if !self.settings.is_favorite(&key.to_ref()) {
            self.settings.toggle_favorite(key.to_ref());
            self.settings_dirty = true;
        }
        self.ensure_placeholder(&key);
        self.nav = Nav::Favorites;
        self.selected = Some(key.clone());
        self.query_single(ctx, &key);
    }

    // ----- UI: menus and toolbar -------------------------------------------

    fn menu_bar(&mut self, ui: &mut egui::Ui) {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button(tr("File"), |ui| self.file_menu(ui));
            ui.menu_button(tr("View"), |ui| {
                ui.checkbox(&mut self.show_toolbar, tr("Toolbar"));
            });
            ui.menu_button(tr("Filter"), |ui| self.filter_menu(ui));
            ui.menu_button(tr("Server"), |ui| {
                let selected = self.selected.clone();
                if ui.button(tr("Refresh list\tF5")).clicked() {
                    self.refresh(ui.ctx());
                    ui.close();
                }
                if ui
                    .add_enabled(self.scan.is_some(), egui::Button::new(tr("Stop")))
                    .clicked()
                {
                    self.stop_scan();
                    ui.close();
                }
                if ui
                    .add_enabled(
                        selected.is_some(),
                        egui::Button::new(tr("Refresh selected server")),
                    )
                    .clicked()
                {
                    if let Some(key) = &selected {
                        self.query_single(ui.ctx(), key);
                    }
                    ui.close();
                }
                if ui
                    .add_enabled(
                        self.scan.is_none() && !self.on_screen.is_empty(),
                        egui::Button::new(tr("Ping visible servers")),
                    )
                    .on_hover_text(fill(
                        tr("Query the servers shown in the list (at most {})"),
                        &[&PING_VISIBLE_MAX],
                    ))
                    .clicked()
                {
                    self.ping_visible(ui.ctx());
                    ui.close();
                }
                ui.separator();
                if ui
                    .add_enabled(selected.is_some(), egui::Button::new(tr("Join\tEnter")))
                    .clicked()
                {
                    if let Some(key) = &selected {
                        self.join(key);
                    }
                    ui.close();
                }
                if ui
                    .add_enabled(
                        selected.is_some(),
                        egui::Button::new(tr("Add to / remove from favorites")),
                    )
                    .clicked()
                {
                    if let Some(key) = selected.clone() {
                        self.apply(ui.ctx(), Action::ToggleFavorite(key));
                    }
                    ui.close();
                }
                if ui
                    .add_enabled(selected.is_some(), egui::Button::new(tr("Copy address")))
                    .clicked()
                {
                    if let Some(server) = selected.as_ref().and_then(|key| self.find(key)) {
                        let text = server.join_address();
                        self.apply(ui.ctx(), Action::Copy(text));
                    }
                    ui.close();
                }
                ui.separator();
                if ui.button(tr("Add server...")).clicked() {
                    self.dialogs.open_add(self.nav.game());
                    ui.close();
                }
                if ui.button(tr("Find LAN servers")).clicked() {
                    self.nav = Nav::Special(Special::Lan);
                    self.scan_lan(ui.ctx());
                    ui.close();
                }
            });
            ui.menu_button(tr("Player"), |ui| {
                if ui.button(tr("Find player...")).clicked() {
                    self.dialogs.find = Some(String::new());
                    ui.close();
                }
            });
            ui.menu_button(tr("Friends"), |ui| self.friends_menu(ui));
            ui.menu_button(tr("Tools"), |ui| {
                if ui.button(tr("Options...")).clicked() {
                    self.dialogs.open_options(&self.settings);
                    ui.close();
                }
            });
            ui.menu_button(tr("Help"), |ui| self.help_menu(ui));
        });
    }

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        let selected = self.selected.clone();
        let has = selected.is_some();
        let ids: Vec<String> = self
            .settings
            .toolbar
            .visible_ids()
            .into_iter()
            .map(str::to_string)
            .collect();
        // Registered first so the buttons, added later, win the clicks.
        let background = ui.interact(
            ui.max_rect(),
            ui.scope_id().with("toolbar-menu"),
            Sense::click(),
        );
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 1.0;
            let mut last_group = None;
            for id in &ids {
                let group = toolbar_group(id);
                if last_group.is_some_and(|last| last != group) {
                    ui.add_space(6.0);
                }
                last_group = Some(group);
                match id.as_str() {
                    "refresh" => {
                        if icons::tool_button(
                            ui,
                            Tool::Refresh,
                            tr(tr("Refresh server list (F5)")),
                            self.scan.is_none(),
                        )
                        .clicked()
                        {
                            self.refresh(ui.ctx());
                        }
                    }
                    "stop" => {
                        if icons::tool_button(ui, Tool::Stop, tr("Stop"), self.scan.is_some())
                            .clicked()
                        {
                            self.stop_scan();
                        }
                    }
                    "ping" => {
                        if icons::tool_button(
                            ui,
                            Tool::Ping,
                            tr("Refresh selected server"),
                            has && !self.single_in_flight,
                        )
                        .clicked()
                            && let Some(key) = &selected
                        {
                            self.query_single(ui.ctx(), key);
                        }
                    }
                    "join" => {
                        if icons::tool_button(ui, Tool::Join, tr(tr("Join server (Enter)")), has)
                            .clicked()
                            && let Some(key) = &selected
                        {
                            self.join(key);
                        }
                    }
                    "favorite" => {
                        if icons::tool_button(
                            ui,
                            Tool::Favorite,
                            tr("Add to / remove from favorites"),
                            has,
                        )
                        .clicked()
                            && let Some(key) = selected.clone()
                        {
                            self.apply(ui.ctx(), Action::ToggleFavorite(key));
                        }
                    }
                    "add" => {
                        if icons::tool_button(ui, Tool::Add, tr("Add server"), true).clicked() {
                            self.dialogs.open_add(self.nav.game());
                        }
                    }
                    "find" => {
                        if icons::tool_button(ui, Tool::Find, tr("Find player"), true).clicked() {
                            self.dialogs.find = Some(String::new());
                        }
                    }
                    "friends" => {
                        if icons::tool_button(ui, Tool::Friends, tr("Friends"), true).clicked() {
                            self.dialogs.friends = Some(String::new());
                        }
                    }
                    "options"
                        if icons::tool_button(ui, Tool::Options, tr("Options"), true).clicked() =>
                    {
                        self.dialogs.open_options(&self.settings);
                    }
                    _ => {}
                }
            }
        });
        background.context_menu(|ui| {
            let mut changed = false;
            for item in &mut self.settings.toolbar.items {
                changed |= ui
                    .checkbox(&mut item.visible, tr(dialogs::toolbar_label(&item.id)))
                    .changed();
            }
            ui.separator();
            if ui.button(tr(TOOLBAR_CUSTOMIZE)).clicked() {
                self.dialogs
                    .open_options_at(&self.settings, dialogs::TOOLBAR_TAB);
                ui.close();
            }
            if ui.button(tr(TOOLBAR_RESET)).clicked() {
                self.settings.toolbar = Default::default();
                changed = true;
                ui.close();
            }
            if changed {
                self.settings_dirty = true;
            }
        });
    }
}

impl eframe::App for OpenAseApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        i18n::set_language(
            self.dialogs
                .preview_language()
                .unwrap_or(self.settings.language),
        );
        self.drain_messages(&ctx);
        if !self.started {
            self.started = true;
            self.refresh(&ctx);
            if self.settings.geoip_disabled {
                self.geoip_status = tr("Disabled").to_string();
            } else {
                let tx = self.tx.clone();
                let ctx = ctx.clone();
                std::thread::spawn(move || {
                    let result = geoip::load_or_download(Duration::from_secs(60))
                        .map(Arc::new)
                        .map_err(|error| error.to_string());
                    let _ = tx.send(Msg::GeoIp(result));
                    ctx.request_repaint();
                });
            }
            self.startup_tasks(&ctx);
        }

        if self.settings.events.auto_refresh_selected
            && !self.single_in_flight
            && self.last_auto_refresh.elapsed()
                >= Duration::from_secs(u64::from(self.settings.events.auto_refresh_seconds.max(5)))
        {
            self.last_auto_refresh = Instant::now();
            if let Some(key) = self.selected.clone() {
                self.query_single(&ctx, &key);
            }
        }
        if self.settings.events.auto_refresh_selected
            || self.scan.is_some()
            || self.dialogs.has_notices()
        {
            ctx.request_repaint_after(Duration::from_millis(500));
        }
        self.watch_tick(&ctx);
        self.request_map_image(&ctx);
        self.request_exe_icons(&ctx);
        if self.slot_watch.is_some() || !self.settings.party_alerts.is_empty() {
            ctx.request_repaint_after(Duration::from_secs(1));
        }

        let no_dialog = !self.dialogs.any_modal();
        if no_dialog && !self.extra.any_modal() {
            self.shortcuts(&ctx);
        }
        if no_dialog && ctx.input(|input| input.key_pressed(egui::Key::F5)) && self.scan.is_none() {
            self.refresh(&ctx);
        }
        if no_dialog
            && !ctx.egui_wants_keyboard_input()
            && ctx.input(|input| input.key_pressed(egui::Key::Enter))
            && let Some(key) = self.selected.clone()
        {
            self.join(&key);
        }

        let frame = egui::Frame::NONE
            .fill(ui.visuals().panel_fill)
            .inner_margin(egui::Margin::symmetric(4, 2));
        egui::Panel::top("menu")
            .frame(frame)
            .show(ui, |ui| self.menu_bar(ui));
        if self.show_toolbar {
            egui::Panel::top("toolbar")
                .frame(frame)
                .show(ui, |ui| self.toolbar(ui));
        }
        egui::Panel::bottom("status")
            .frame(frame)
            .show(ui, |ui| self.status_bar(ui));

        // A gap of panel color around each box, so the theme's border shows.
        let list_frame = egui::Frame::NONE
            .outer_margin(egui::Margin::same(2))
            .fill(rgb(self.palette.list_bg))
            // With a bevel (GoldSrc) the sunken edges replace the stroke.
            .stroke(if self.palette.theme.bevel().is_some() {
                Stroke::NONE
            } else {
                ui.visuals().widgets.noninteractive.bg_stroke
            });
        let mut boxes: Vec<egui::Rect> = Vec::new();
        // GoldSrc's sunken boxes and gaps separate the panels; egui's line
        // would draw a thick dark bar between them.
        let separators = self.palette.theme.bevel().is_none();
        egui::Panel::bottom("details")
            .show_separator_line(separators)
            .resizable(true)
            .default_size(200.0)
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                let selected = self
                    .selected
                    .clone()
                    .and_then(|key| self.find(&key).cloned());
                let shown = egui::Panel::left("friends-panel")
                    .show_separator_line(separators)
                    .resizable(true)
                    .default_size(220.0)
                    .frame(list_frame)
                    .show(ui, |ui| {
                        self.friends_pane(ui);
                    });
                boxes.push(shown.response.rect);
                let shown = egui::Panel::left("players-panel")
                    .show_separator_line(separators)
                    .resizable(true)
                    .default_size(380.0)
                    .frame(list_frame)
                    .show(ui, |ui| {
                        self.players_pane(ui, selected.as_ref());
                    });
                boxes.push(shown.response.rect);
                let shown = egui::CentralPanel::default()
                    .frame(list_frame)
                    .show(ui, |ui| {
                        let image = selected.as_ref().and_then(|s| {
                            self.map_images
                                .get(&(s.game.clone(), s.map.clone()))?
                                .as_ref()
                        });
                        if let Some(texture) = image {
                            // Levelshots are 4:3 at most 256 wide; shown small.
                            ui.add(egui::Image::new(texture).max_size(vec2(160.0, 120.0)));
                        }
                        self.rules_pane(ui, selected.as_ref());
                    });
                boxes.push(shown.response.rect);
            });

        let shown = egui::Panel::left("tree")
            .show_separator_line(separators)
            .resizable(true)
            .default_size(230.0)
            .frame(list_frame.inner_margin(egui::Margin::same(2)))
            .show(ui, |ui| {
                if ui
                    .checkbox(
                        &mut self.settings.filters.installed_only,
                        tr("Installed games only"),
                    )
                    .changed()
                {
                    self.settings_dirty = true;
                }
                egui::Panel::bottom("tree-filters")
                    .frame(egui::Frame::NONE.inner_margin(egui::Margin::symmetric(2, 6)))
                    .show(ui, |ui| self.filter_bar(ui));
                egui::ScrollArea::both()
                    .auto_shrink(false)
                    .show(ui, |ui| self.tree(ui));
            });
        boxes.push(shown.response.rect);

        let shown = egui::CentralPanel::default()
            .frame(list_frame)
            .show(ui, |ui| self.server_list(ui));
        boxes.push(shown.response.rect);
        // GoldSrc: sunken boxes, as in Half-Life's VGUI.
        if let Some((light, dark)) = self.palette.theme.bevel() {
            // Boxes use a softer light edge than the raised headers.
            let soft = rgb(light).lerp_to_gamma(ui.visuals().panel_fill, 0.6);
            for rect in boxes {
                icons::sunken(ui.painter(), rect.shrink(2.0), rgb(dark), soft);
            }
        }

        self.show_dialogs(&ctx);
        self.show_extra(&ctx);

        if self.settings_dirty {
            self.settings_dirty = false;
            self.ready_cache = None;
            if let Err(error) = self.settings.save() {
                self.status = fill(tr("Could not save settings: {}"), &[&error]);
            }
        }
    }
}

// ----- workers ---------------------------------------------------------------

// ----- helpers -----------------------------------------------------------------

fn setup_style(ctx: &egui::Context) {
    ctx.set_theme(egui::ThemePreference::Light);
    ctx.all_styles_mut(|style| {
        use egui::FontFamily::{Monospace, Proportional};
        style.text_styles = [
            (TextStyle::Small, FontId::new(10.0, Proportional)),
            (TextStyle::Body, FontId::new(12.5, Proportional)),
            (TextStyle::Button, FontId::new(12.5, Proportional)),
            (TextStyle::Heading, FontId::new(16.0, Proportional)),
            (TextStyle::Monospace, FontId::new(12.0, Monospace)),
        ]
        .into();
        style.spacing.item_spacing = vec2(6.0, 3.0);
        style.spacing.button_padding = vec2(8.0, 2.0);
        style.spacing.interact_size.y = 18.0;
        // Always-visible scroll bars, as in ASE (egui's float over the content).
        style.spacing.scroll = egui::style::ScrollStyle::solid();
    });
}

/// ASE's flat look on egui's light or dark visuals: square widgets, a small
/// shadow, and on the light base Windows-like greys.
fn ase_visuals(dark: bool) -> egui::Visuals {
    let mut visuals = if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    visuals.selection.stroke = Stroke::new(1.0, Color32::WHITE);
    visuals.window_corner_radius = CornerRadius::same(2);
    visuals.menu_corner_radius = CornerRadius::ZERO;
    visuals.window_shadow = egui::Shadow {
        offset: [2, 2],
        blur: 6,
        spread: 0,
        color: Color32::from_black_alpha(60),
    };
    for widgets in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        widgets.corner_radius = CornerRadius::ZERO;
    }
    if !dark {
        visuals.panel_fill = Color32::from_gray(0xf0);
        visuals.window_fill = Color32::from_gray(0xf0);
        visuals.widgets.inactive.weak_bg_fill = Color32::from_gray(0xe1);
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_gray(0xad));
        visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(0xe5, 0xf1, 0xfb);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(0x00, 0x78, 0xd7));
    }
    visuals
}

/// Toolbar texts (French translation pass: keep together).
const TOOLBAR_CUSTOMIZE: &str = noop("Customize toolbar...");
const TOOLBAR_RESET: &str = noop("Reset toolbar");

/// Buttons of the same group are drawn next to each other.
fn toolbar_group(id: &str) -> u8 {
    match id {
        "refresh" | "stop" | "ping" => 0,
        "join" | "favorite" | "add" => 1,
        _ => 2,
    }
}

impl OpenAseApp {
    /// Show these colors (a no-op when they are already applied).
    pub(crate) fn show_palette(&mut self, ctx: &egui::Context, colors: &ColorSettings) {
        if self.palette != *colors {
            apply_palette(ctx, colors);
            self.palette = colors.clone();
        }
    }
}

/// Apply the user's list colors to the egui visuals.
fn apply_palette(ctx: &egui::Context, colors: &ColorSettings) {
    let theme = colors.theme;
    let base = if theme.dark() {
        egui::Theme::Dark
    } else {
        egui::Theme::Light
    };
    ctx.set_theme(base);
    let bevel = theme.bevel().map(|(light, dark)| (rgb(light), rgb(dark)));
    ctx.data_mut(|data| data.insert_temp(bevel_id(), bevel));
    // Rebuilt from the base each time, so nothing of a previous theme stays.
    ctx.style_mut_of(base, |style| {
        style.visuals = ase_visuals(theme.dark());
        let visuals = &mut style.visuals;
        visuals.extreme_bg_color = rgb(colors.list_bg);
        visuals.faint_bg_color = rgb(colors.alt_row);
        visuals.selection.bg_fill = rgb(colors.selection);
        visuals.widgets.noninteractive.fg_stroke.color = rgb(colors.text);
        if let Some(panel) = theme.panel() {
            visuals.panel_fill = rgb(panel);
            visuals.window_fill = rgb(panel);
        }
        if let Some(text) = theme.selection_text() {
            visuals.selection.stroke.color = rgb(text);
        }
        if let Some(border) = theme.border() {
            visuals.widgets.noninteractive.bg_stroke.color = rgb(border);
            visuals.window_stroke.color = rgb(border);
        }
        if let Some((idle, hovered)) = theme.widgets() {
            let widgets = &mut visuals.widgets;
            widgets.inactive.bg_fill = rgb(idle);
            widgets.inactive.weak_bg_fill = rgb(idle);
            widgets.hovered.bg_fill = rgb(hovered);
            widgets.hovered.weak_bg_fill = rgb(hovered);
            widgets.active.bg_fill = rgb(colors.selection);
            widgets.active.weak_bg_fill = rgb(colors.selection);
        }
    });
}

/// "Online" green, readable on the theme's background.
pub(crate) fn online_green(visuals: &egui::Visuals) -> Color32 {
    if visuals.dark_mode {
        Color32::from_rgb(0x5c, 0xd0, 0x6a)
    } else {
        Color32::from_rgb(0x14, 0x78, 0x1e)
    }
}

/// Background of rows with a friend on them.
fn friend_fill(visuals: &egui::Visuals) -> Color32 {
    if visuals.dark_mode {
        Color32::from_rgb(0x24, 0x4a, 0x2c)
    } else {
        Color32::from_rgb(0xe0, 0xf5, 0xe0)
    }
}

/// A server-chosen color made readable on the list: near-white is darkened
/// on a light theme, dark colors (`^0`, `^4`) are lightened on a dark one.
fn readable(color: Color32, visuals: &egui::Visuals) -> Color32 {
    let [r, g, b, _] = color.to_array();
    if visuals.dark_mode {
        let luma = (299 * u32::from(r) + 587 * u32::from(g) + 114 * u32::from(b)) / 1000;
        if luma < 90 {
            return color.lerp_to_gamma(Color32::WHITE, 0.5);
        }
    } else if r > 0xd0 && g > 0xd0 && b > 0xd0 {
        return Color32::from_gray(0x50);
    }
    color
}

pub(crate) fn rgb([r, g, b]: Rgb) -> Color32 {
    Color32::from_rgb(r, g, b)
}

/// Where `apply_palette` keeps the theme's bevel colors for `header_cell`.
fn bevel_id() -> egui::Id {
    egui::Id::unique("theme-bevel")
}

fn header_cell(ui: &egui::Ui, rect: egui::Rect) {
    let rect = rect.expand2(vec2(4.0, 2.0));
    let visuals = ui.visuals();
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::ZERO, visuals.panel_fill);
    let bevel: Option<(Color32, Color32)> = ui.ctx().data(|d| d.get_temp(bevel_id())).flatten();
    match bevel {
        // Raised, as in Half-Life's VGUI.
        Some((light, dark)) => icons::edges(painter, rect, light, dark),
        None => {
            let line = visuals.widgets.noninteractive.bg_stroke.color;
            icons::edges(painter, rect, Color32::TRANSPARENT, line);
        }
    }
}

fn header_label(ui: &mut egui::Ui, text: &str, color: Color32, centered: bool) {
    if centered {
        ui.centered_and_justified(|ui| cell(ui, text, color));
    } else {
        cell(ui, text, color);
    }
}

fn cell(ui: &mut egui::Ui, text: &str, color: Color32) {
    ui.add(
        egui::Label::new(RichText::new(text).color(color))
            .selectable(false)
            .truncate(),
    );
}

/// Draw Quake-colored text, made readable on the theme ([`readable`]).
fn colored_label(ui: &mut egui::Ui, raw: &str, default_color: Color32) {
    let default_hex = format!(
        "#{:02x}{:02x}{:02x}",
        default_color.r(),
        default_color.g(),
        default_color.b()
    );
    let font = TextStyle::Body.resolve(ui.style());
    let mut job = LayoutJob::default();
    for segment in quake_color_segments(raw, &default_hex) {
        let [r, g, b] = parse_hex(&segment.color).unwrap_or([0x20, 0x20, 0x20]);
        let color = readable(Color32::from_rgb(r, g, b), ui.visuals());
        job.append(&segment.text, 0.0, TextFormat::simple(font.clone(), color));
    }
    ui.add(egui::Label::new(job).selectable(false).truncate());
}

fn format_duration(secs: u32) -> String {
    let (h, m, s) = (secs / 3600, secs / 60 % 60, secs % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

/// Play the sound of a friend: the system sound when the global option is on,
/// nothing when the friend is silenced, or the chosen .wav file.
fn play_friend_sound(sound: &FriendSound, global: bool) {
    match sound {
        FriendSound::Default if global => play_sound(None),
        FriendSound::File(path) if !path.trim().is_empty() => play_sound(Some(path.trim())),
        _ => {}
    }
}

/// Play a .wav file asynchronously, or the system "information" sound.
#[cfg(windows)]
pub(crate) fn play_sound(file: Option<&str>) {
    #[link(name = "user32")]
    unsafe extern "system" {
        fn MessageBeep(kind: u32) -> i32;
    }
    #[link(name = "winmm")]
    unsafe extern "system" {
        fn PlaySoundW(sound: *const u16, module: *mut std::ffi::c_void, flags: u32) -> i32;
    }
    const SND_ASYNC: u32 = 0x0001;
    const SND_NODEFAULT: u32 = 0x0002;
    const SND_FILENAME: u32 = 0x0002_0000;
    match file {
        Some(path) => {
            let wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
            // SAFETY: `wide` is NUL-terminated and outlives the call; for a file
            // name, PlaySoundW reads the string before returning.
            unsafe {
                PlaySoundW(
                    wide.as_ptr(),
                    std::ptr::null_mut(),
                    SND_FILENAME | SND_ASYNC | SND_NODEFAULT,
                );
            }
        }
        None => unsafe {
            // MB_ICONASTERISK: the system "information" sound.
            MessageBeep(0x40);
        },
    }
}

/// Play a .wav file, or the desktop's notification sound, with the player
/// programs the system already has (no audio library is linked).
#[cfg(not(windows))]
pub(crate) fn play_sound(file: Option<&str>) {
    let candidates: Vec<(&str, Vec<String>)> = match file {
        Some(path) if cfg!(target_os = "macos") => vec![("afplay", vec![path.to_string()])],
        Some(path) => ["pw-play", "paplay", "aplay"]
            .into_iter()
            .map(|player| (player, vec![path.to_string()]))
            .collect(),
        None if cfg!(target_os = "macos") => vec![(
            "afplay",
            vec!["/System/Library/Sounds/Glass.aiff".to_string()],
        )],
        None => {
            const FREEDESKTOP: &str =
                "/usr/share/sounds/freedesktop/stereo/message-new-instant.oga";
            vec![
                (
                    "canberra-gtk-play",
                    vec!["-i".to_string(), "message-new-instant".to_string()],
                ),
                ("pw-play", vec![FREEDESKTOP.to_string()]),
                ("paplay", vec![FREEDESKTOP.to_string()]),
            ]
        }
    };
    // Try each player in turn off the UI thread, and wait for the one that
    // started so it does not stay a zombie process.
    std::thread::spawn(move || {
        for (program, args) in candidates {
            let started = std::process::Command::new(program)
                .args(&args)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn();
            if let Ok(mut child) = started
                && child.wait().is_ok_and(|status| status.success())
            {
                return;
            }
        }
    });
}

fn rcon_error(error: &openase::rcon::RconError) -> String {
    use openase::rcon::RconError;
    match error {
        RconError::Timeout => tr("No answer from the server").to_string(),
        RconError::BadPassword => tr("Wrong RCON password").to_string(),
        RconError::NoChallenge => {
            tr("The server gave no RCON challenge (is RCON enabled?)").to_string()
        }
        RconError::Unsupported => tr("This game has no RCON").to_string(),
        RconError::Invalid => tr("Invalid answer from the server").to_string(),
        RconError::Network(detail) => fill(tr("Network error: {}"), &[detail]),
    }
}

/// Fill the GeoIP country of a server once.
fn annotate(geoip: Option<&GeoIp>, server: &mut Server) {
    if server.country.is_none()
        && let Some(geoip) = geoip
    {
        server.country = geoip.lookup_host(&server.host).map(str::to_string);
    }
}

/// Country code used by the Country filter, `??` when unknown.
fn country_code(server: &Server) -> &str {
    server.country.as_deref().unwrap_or("??")
}

fn country_label(code: &str) -> String {
    country_name(code).map_or_else(|| tr("Unknown").to_string(), str::to_string)
}
