//! Desktop GUI, laid out like The All-Seeing Eye: filter tree on the left,
//! server list on the right, friends / players / rules panes at the bottom.

mod dialogs;
mod extra;
mod icons;

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
            .is_some_and(|watch| watch.key == id)
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
            let address = watch.key.split_once('|').map_or("", |(_, a)| a).to_string();
            self.slot_watch = None;
            self.status = fill(
                tr("Stopped watching {}: no free slot in 30 minutes"),
                &[&address],
            );
        }
        if self.watch_in_flight {
            return;
        }
        let mut targets: Vec<String> = Vec::new();
        if let Some(watch) = &mut self.slot_watch
            && watch.due(now)
        {
            watch.queried(now);
            targets.push(watch.key.clone());
        }
        if !self.settings.party_alerts.is_empty()
            && self
                .last_party_poll
                .is_none_or(|last| now >= last + alerts::PARTY_INTERVAL)
        {
            self.last_party_poll = Some(now);
            for id in self.settings.party_alerts.keys() {
                if !targets.contains(id) {
                    targets.push(id.clone());
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
            for id in targets {
                let Some((game, address)) = id.split_once('|') else {
                    continue;
                };
                let Some(game) = catalog::game(game) else {
                    continue;
                };
                if let Ok((host, port)) = parse_address(address, game.default_port) {
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
        let countries = self.country_options();
        let filters = &mut self.settings.filters;
        let before = (
            filters.not_empty,
            filters.not_full,
            filters.no_password,
            filters.hide_offline,
            filters.max_ping,
            filters.countries.clone(),
        );
        ui.checkbox(&mut filters.not_empty, tr("Hide empty"));
        ui.checkbox(&mut filters.not_full, tr("Hide full"));
        ui.checkbox(&mut filters.no_password, tr("Hide passworded"));
        ui.checkbox(&mut filters.hide_offline, tr("Hide unresponsive"));
        egui::Grid::new("filter-bar")
            .num_columns(2)
            .spacing([6.0, 4.0])
            .show(ui, |ui| {
                ui.label(tr("Max ping"));
                let shown = filters
                    .max_ping
                    .map_or_else(|| tr("No limit").to_string(), |p| p.to_string());
                egui::ComboBox::from_id_salt("max-ping")
                    .selected_text(shown)
                    .width(120.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut filters.max_ping, None, tr("No limit"));
                        for limit in [50, 100, 150, 250] {
                            ui.selectable_value(
                                &mut filters.max_ping,
                                Some(limit),
                                limit.to_string(),
                            );
                        }
                    });
                ui.end_row();
                ui.label(tr("Country"));
                let shown = match filters.countries.iter().next() {
                    Some(code) if filters.countries.len() == 1 => country_label(code),
                    Some(_) => fill(tr("{} countries"), &[&filters.countries.len()]),
                    None => tr("All countries").to_string(),
                };
                egui::ComboBox::from_id_salt("country")
                    .selected_text(shown)
                    .width(120.0)
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_label(filters.countries.is_empty(), tr("All countries"))
                            .clicked()
                        {
                            filters.countries.clear();
                        }
                        for (code, label, count) in &countries {
                            let only =
                                filters.countries.len() == 1 && filters.countries.contains(code);
                            if ui
                                .selectable_label(only, format!("{label} ({count})"))
                                .clicked()
                            {
                                filters.countries = [code.clone()].into();
                            }
                        }
                    });
                ui.end_row();
            });
        let after = (
            filters.not_empty,
            filters.not_full,
            filters.no_password,
            filters.hide_offline,
            filters.max_ping,
            filters.countries.clone(),
        );
        if before != after {
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

    // ----- scanning --------------------------------------------------------

    /// Remembered server lists behind list-type tree entries.
    fn nav_list(&self, nav: &Nav) -> Option<Vec<ServerKey>> {
        let refs = match nav {
            Nav::Favorites => self.settings.favorites.clone(),
            Nav::FavoriteGroup(group) => self.settings.favorites_in(group),
            Nav::Recent => self.settings.recent.clone(),
            Nav::Special(Special::Added) => self.settings.recently_added.clone(),
            Nav::Special(Special::Imported) => self.settings.recently_imported.clone(),
            Nav::Special(Special::Lan) => return Some(self.lan_servers.clone()),
            _ => return None,
        };
        Some(refs.iter().map(ServerKey::from_ref).collect())
    }

    fn known_servers(&self) -> Vec<ServerKey> {
        self.store
            .values()
            .flatten()
            .filter(|server| server.online)
            .map(ServerKey::of)
            .collect()
    }

    fn refresh(&mut self, ctx: &egui::Context) {
        let nav = self.nav.clone();
        if let Some(keys) = self.nav_list(&nav) {
            if nav == Nav::Special(Special::Lan) {
                self.scan_lan(ctx);
            } else {
                self.scan_list(ctx, keys, noop("servers"));
            }
            return;
        }
        match nav {
            Nav::Game(game) | Nav::GameType(game, _) => self.scan_game(ctx, &game),
            Nav::Special(Special::Playing) => self.scan_ready_games(ctx),
            Nav::Custom(index) => {
                let game = self
                    .settings
                    .custom_filters
                    .get(index)
                    .map(|filter| filter.game.clone());
                match game.filter(|game| !game.is_empty()) {
                    Some(game) => self.scan_game(ctx, &game),
                    None => {
                        let keys = self.known_servers();
                        self.scan_list(ctx, keys, noop("known servers"));
                    }
                }
            }
            _ => {
                let keys = self.known_servers();
                self.scan_list(ctx, keys, noop("known servers"));
            }
        }
    }

    fn scan_lan(&mut self, ctx: &egui::Context) {
        self.status = tr("Looking for servers on the local network...").to_string();
        let tx = self.tx.clone();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let result = openase::lan::discover(Duration::from_millis(1500))
                .map_err(|error| error.to_string());
            let _ = tx.send(Msg::Lan(result));
            ctx.request_repaint();
        });
    }

    fn stop_scan(&mut self) {
        if let Some(scan) = &self.scan {
            scan.stop.store(true, Ordering::Relaxed);
        }
    }

    fn start_scan(&mut self, label: String) -> (u64, Arc<AtomicBool>, Arc<AtomicUsize>) {
        self.stop_scan();
        self.generation += 1;
        let stop = Arc::new(AtomicBool::new(false));
        let done = Arc::new(AtomicUsize::new(0));
        self.scan = Some(Scan {
            generation: self.generation,
            stop: stop.clone(),
            done: done.clone(),
            total: 0,
            label,
            games: Vec::new(),
        });
        (self.generation, stop, done)
    }

    fn scan_game(&mut self, ctx: &egui::Context, game_id: &str) {
        let Some(game) = catalog::game(game_id) else {
            return;
        };
        self.status = fill(tr("Querying {} master servers..."), &[&game.name]);
        self.scan_games(ctx, vec![game], fill(tr("Scanning {}"), &[&game.name]));
    }

    /// "Playing now": scan every game the user can join.
    fn scan_ready_games(&mut self, ctx: &egui::Context) {
        let games = self.settings.ready_games();
        if games.is_empty() {
            self.status = tr("No game is set up: set its program in Options > Games").to_string();
            return;
        }
        self.status = fill(
            tr("Querying the master servers of {} games..."),
            &[&games.len()],
        );
        self.scan_games(ctx, games, tr("Playing now").to_string());
    }

    /// Fetch the master lists of `games` in parallel, then query their servers.
    fn scan_games(&mut self, ctx: &egui::Context, games: Vec<&'static Game>, label: String) {
        let (generation, stop, done) = self.start_scan(label);
        if let Some(scan) = &mut self.scan {
            scan.games = games.iter().map(|game| game.id.clone()).collect();
        }
        let tx = self.tx.clone();
        let ctx = ctx.clone();
        let network = self.settings.network.clone();
        let steam_key = self.settings.steam_api_key.clone();
        std::thread::spawn(move || {
            let options = network.query_options();
            let targets = std::sync::Mutex::new(Vec::new());
            std::thread::scope(|scope| {
                for game in games {
                    let (tx, ctx, targets) = (&tx, &ctx, &targets);
                    let (network, steam_key, options) = (&network, &steam_key, &options);
                    scope.spawn(move || {
                        let mut list =
                            fetch_all_masters(game, options.timeout, steam_key, network.scan_limit);
                        list.servers.truncate(network.scan_limit.max(1));
                        // Servers listed with their details (333networks) are shown
                        // as listed and queried only on demand; the others now.
                        targets.lock().unwrap().extend(
                            list.servers
                                .iter()
                                .filter(|entry| entry.preview.is_none())
                                .map(|entry| (game.id.clone(), entry.address.clone())),
                        );
                        let _ = tx.send(Msg::MasterList {
                            generation,
                            game: game.id.clone(),
                            servers: list.servers,
                            errors: list.errors,
                            paged_total: list.paged_total,
                        });
                        ctx.request_repaint();
                    });
                }
            });
            if stop.load(Ordering::Relaxed) {
                let _ = tx.send(Msg::ScanDone { generation });
                return;
            }
            let targets = targets.into_inner().unwrap();
            run_queries(targets, &network, &tx, &ctx, generation, &stop, &done);
        });
    }

    /// Download the next page of a game's 333networks list, if there is one
    /// and no page request is already running.
    fn load_more(&mut self, ctx: &egui::Context, game_id: &str) {
        if self.more_in_flight.is_some() {
            return;
        }
        let Some(game) = catalog::game(game_id) else {
            return;
        };
        let Some(paging) = self.paging.get(game_id).filter(|paging| paging.has_more()) else {
            return;
        };
        let (generation, page) = (paging.generation, paging.pages + 1);
        self.more_in_flight = Some(game_id.to_string());
        self.status = tr("Loading more servers...").to_string();
        let tx = self.tx.clone();
        let ctx = ctx.clone();
        let timeout = self.settings.network.query_options().timeout;
        std::thread::spawn(move || {
            let result = fetch_more(game, page, timeout).map_err(|error| error.to_string());
            let _ = tx.send(Msg::MorePage {
                generation,
                game: game.id.clone(),
                page,
                result,
            });
            ctx.request_repaint();
        });
    }

    /// Query the rows drawn in the server list, at most [`PING_VISIBLE_MAX`].
    fn ping_visible(&mut self, ctx: &egui::Context) {
        let keys: Vec<ServerKey> = self
            .on_screen
            .iter()
            .take(PING_VISIBLE_MAX)
            .cloned()
            .collect();
        self.scan_list(ctx, keys, noop("visible servers"));
    }

    fn scan_list(&mut self, ctx: &egui::Context, keys: Vec<ServerKey>, what: &'static str) {
        if keys.is_empty() {
            self.status = fill(tr("No {} to refresh"), &[&tr(what)]);
            return;
        }
        for key in &keys {
            self.ensure_placeholder(key);
        }
        let (generation, stop, done) = self.start_scan(fill(tr("Refreshing {}"), &[&tr(what)]));
        if let Some(scan) = &mut self.scan {
            scan.total = keys.len();
        }
        let tx = self.tx.clone();
        let ctx = ctx.clone();
        let network = self.settings.network.clone();
        std::thread::spawn(move || {
            let targets = keys
                .into_iter()
                .map(|key| (key.game, key.address))
                .collect();
            run_queries(targets, &network, &tx, &ctx, generation, &stop, &done);
        });
    }

    pub(crate) fn query_single(&mut self, ctx: &egui::Context, key: &ServerKey) {
        let Some(game) = catalog::game(&key.game) else {
            return;
        };
        let Ok((host, port)) = parse_address(&key.address, game.default_port) else {
            self.dialogs.error = Some(fill(tr("Invalid address {}"), &[&key.address]));
            return;
        };
        self.single_in_flight = true;
        let tx = self.tx.clone();
        let ctx = ctx.clone();
        let options = self.settings.network.query_options();
        std::thread::spawn(move || {
            let _ = tx.send(Msg::Single(query_server(game, &host, port, &options)));
            ctx.request_repaint();
        });
    }

    fn drain_messages(&mut self, ctx: &egui::Context) {
        while let Ok(msg) = self.rx.try_recv() {
            let current = self.scan.as_ref().map(|scan| scan.generation);
            match msg {
                Msg::MasterList {
                    generation,
                    game,
                    servers,
                    errors,
                    paged_total,
                } if Some(generation) == current => {
                    let mut list = Vec::with_capacity(servers.len());
                    let old = self.store.remove(&game).unwrap_or_default();
                    let mut old: HashMap<String, Server> = old
                        .into_iter()
                        .map(|server| (server.address(), server))
                        .collect();
                    let definition = catalog::game(&game);
                    let mut listed = 0;
                    let mut queried = 0;
                    for entry in servers {
                        let existing = old.remove(&entry.address);
                        if let Some(mut server) = entry.preview {
                            // Fresh details from the list replace older ones.
                            listed += 1;
                            annotate(self.geoip.as_deref(), &mut server);
                            list.push(server);
                        } else if let Some(existing) = existing {
                            queried += 1;
                            list.push(existing);
                        } else if let Some(definition) = definition
                            && let Ok((host, port)) =
                                parse_address(&entry.address, definition.default_port)
                        {
                            queried += 1;
                            let mut server = Server::new(&game, &host, port);
                            annotate(self.geoip.as_deref(), &mut server);
                            list.push(server);
                        }
                    }
                    let empty = list.is_empty();
                    self.store.insert(game.clone(), list);
                    match paged_total {
                        Some(total) => {
                            self.paging.insert(
                                game.clone(),
                                Paging {
                                    generation,
                                    pages: 1,
                                    listed,
                                    total,
                                },
                            );
                        }
                        None => {
                            self.paging.remove(&game);
                        }
                    }
                    // Summed: a "Playing now" scan lists several games.
                    if let Some(scan) = &mut self.scan {
                        scan.total += queried;
                    }
                    // The details pane of a selected listed server needs a query.
                    if let Some(key) = self.selected.clone().filter(|key| key.game == game) {
                        self.query_if_listed(ctx, &key);
                    }
                    if empty && !errors.is_empty() {
                        self.dialogs.error = Some(fill(
                            tr("No server list could be downloaded:\n\n{}"),
                            &[&errors.join("\n")],
                        ));
                    } else if !errors.is_empty() {
                        self.status = fill(tr("Some masters failed: {}"), &[&errors.join("; ")]);
                    }
                }
                Msg::Server { generation, server } if Some(generation) == current => {
                    self.upsert(server)
                }
                Msg::MorePage {
                    generation,
                    game,
                    page,
                    result,
                } => {
                    if self.more_in_flight.as_deref() == Some(game.as_str()) {
                        self.more_in_flight = None;
                    }
                    match result {
                        Ok(more) => {
                            let Some(paging) = self
                                .paging
                                .get_mut(&game)
                                .filter(|paging| paging.generation == generation)
                            else {
                                continue;
                            };
                            paging.pages = page;
                            paging.total = more.total;
                            let list = self.store.entry(game.clone()).or_default();
                            let mut added = 0;
                            for entry in more.servers {
                                // Rows already present (live or from another
                                // page) are kept as they are.
                                if let Some(mut server) = entry.preview
                                    && !list.iter().any(|row| row.address() == entry.address)
                                {
                                    annotate(self.geoip.as_deref(), &mut server);
                                    list.push(server);
                                    added += 1;
                                }
                            }
                            paging.listed += added;
                            self.status = fill(tr("{} more servers loaded"), &[&added]);
                        }
                        Err(error) => {
                            self.status = fill(tr("Could not load more servers: {}"), &[&error]);
                        }
                    }
                }
                Msg::ScanDone { generation } if Some(generation) == current => {
                    let stopped = self
                        .scan
                        .as_ref()
                        .is_some_and(|scan| scan.stop.load(Ordering::Relaxed));
                    if let Some(scan) = self.scan.take()
                        && !stopped
                    {
                        self.save_lists(scan.games);
                    }
                    self.status = if stopped {
                        tr("Scan stopped").to_string()
                    } else {
                        tr("Scan complete").to_string()
                    };
                }
                Msg::Single(server) => {
                    self.single_in_flight = false;
                    self.status = if server.online {
                        fill(tr("Refreshed {}"), &[&strip_quake_colors(&server.name)])
                    } else {
                        fill(tr("{} did not answer"), &[&server.address()])
                    };
                    self.upsert(server);
                }
                Msg::SearchProgress(dir) => self.dialogs.search_progress(dir),
                Msg::SearchFound(path) => self.dialogs.search_found(path),
                Msg::SearchHit { game, path, sure } => self.dialogs.search_hit(game, path, sure),
                Msg::SearchDone => self.dialogs.search_done(),
                Msg::GeoIp(result) => match result {
                    Ok(geoip) => {
                        self.geoip_status = fill(tr("{} IPv4 ranges loaded"), &[&geoip.len()]);
                        for server in self.store.values_mut().flatten() {
                            annotate(Some(&geoip), server);
                        }
                        self.geoip = Some(geoip);
                    }
                    Err(error) => {
                        self.geoip_status = fill(tr("Not available: {}"), &[&error]);
                        self.status = fill(tr("GeoIP database not available: {}"), &[&error]);
                    }
                },
                Msg::Lan(result) => match result {
                    Ok(found) => {
                        self.lan_servers = found
                            .into_iter()
                            .map(|server| ServerKey {
                                game: server.game,
                                address: server.address,
                            })
                            .collect();
                        if self.lan_servers.is_empty() {
                            self.status = tr("No server found on the local network").to_string();
                        } else {
                            let keys = self.lan_servers.clone();
                            self.scan_list(ctx, keys, noop("LAN servers"));
                        }
                    }
                    Err(error) => {
                        self.dialogs.error = Some(fill(tr("LAN search failed: {}"), &[&error]))
                    }
                },
                Msg::Link(url) => self.open_link(ctx, &url),
                Msg::ExeIcon { game, path, image } => {
                    // A result for a path changed meanwhile is stale.
                    if self.exe_icons.get(&game).is_some_and(|(p, _)| *p == path) {
                        let texture = image.map(|(size, rgba)| {
                            let image = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
                            let options = egui::TextureOptions::LINEAR
                                .with_mipmap_mode(Some(egui::TextureFilter::Linear));
                            ctx.load_texture(format!("exe-{game}"), image, options)
                        });
                        self.exe_icons.insert(game, (path, texture));
                    }
                }
                Msg::MapImage { game, map, image } => {
                    let texture = image.map(|(size, rgba)| {
                        let image = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
                        ctx.load_texture(format!("map-{game}-{map}"), image, Default::default())
                    });
                    self.map_images.insert((game, map), texture);
                }
                Msg::Rcon { key, result } => {
                    if let Some(console) = self.dialogs.rcon.as_mut().filter(|c| c.key == key) {
                        console.in_flight = false;
                        match result {
                            Ok(text) => console.output.push_str(&text),
                            Err(error) => console.output.push_str(&rcon_error(&error)),
                        }
                        if !console.output.ends_with('\n') {
                            console.output.push('\n');
                        }
                    }
                }
                Msg::SavedLists(lists) => {
                    // A list scanned meanwhile is newer: keep it.
                    for (game, mut rows) in lists {
                        if !self.store.contains_key(&game) {
                            for server in &mut rows {
                                annotate(self.geoip.as_deref(), server);
                            }
                            self.store.insert(game, rows);
                        }
                    }
                }
                Msg::Watched(server) => self.upsert(server),
                Msg::WatchDone => self.watch_in_flight = false,
                Msg::Resolved {
                    game,
                    addresses,
                    group,
                } => match group {
                    None => addresses
                        .iter()
                        .for_each(|address| self.finish_add(ctx, &game, address)),
                    Some(group) => self.finish_import(ctx, &game, &group, addresses),
                },
                Msg::PortFixes(fixes) => {
                    for (old, new) in &fixes {
                        self.settings.replace_server(old, new);
                    }
                    self.settings.unreal_ports_checked = true;
                    self.settings_dirty = true;
                }
                Msg::Update { result, manual } => match result {
                    Ok(Some(tag)) => self.extra.update_available = Some(tag),
                    Ok(None) if manual => self.dialogs.notify(fill(
                        tr("OpenASE-Plus {} is up to date."),
                        &[&env!("CARGO_PKG_VERSION")],
                    )),
                    Err(error) if manual => {
                        self.dialogs.error =
                            Some(fill(tr("Could not check for updates: {}"), &[&error]))
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    // ----- view ------------------------------------------------------------

    /// Servers of the current tree selection, before filters.
    fn candidates(&self) -> Vec<&Server> {
        if let Some(keys) = self.nav_list(&self.nav) {
            return keys.iter().filter_map(|key| self.find(key)).collect();
        }
        match &self.nav {
            Nav::Game(game) | Nav::GameType(game, _) => self
                .store
                .get(game)
                .map(|list| list.iter().collect())
                .unwrap_or_default(),
            Nav::Friends => self
                .store
                .values()
                .flatten()
                .filter(|server| self.server_has_friend(server))
                .collect(),
            Nav::Special(Special::Playing) => self
                .store
                .values()
                .flatten()
                .filter(|server| server.has_info() && !server.saved && alerts::humans(server) > 0)
                .collect(),
            Nav::FriendGroup(group) => {
                let keys: Vec<&ServerKey> = self
                    .settings
                    .friends
                    .iter()
                    .filter(|friend| self.settings.friend_group(friend) == *group)
                    .filter_map(|friend| self.friend_location(friend))
                    .collect();
                self.store
                    .values()
                    .flatten()
                    .filter(|server| keys.contains(&&ServerKey::of(server)))
                    .collect()
            }
            Nav::Custom(index) => match self.settings.custom_filters.get(*index) {
                Some(filter) => self
                    .store
                    .iter()
                    .filter(|(game, _)| filter.game.is_empty() || **game == filter.game)
                    .flat_map(|(_, list)| list.iter())
                    .filter(|server| filter.matches(server))
                    .collect(),
                None => Vec::new(),
            },
            _ => Vec::new(),
        }
    }

    /// Highlight color of the first color filter matching a server.
    fn color_filter(&self, server: &Server) -> Option<Color32> {
        self.settings
            .custom_filters
            .iter()
            .filter(|filter| filter.color.is_some() && !filter.rules.is_empty())
            .find(|filter| filter.matches(server))
            .and_then(|filter| parse_hex(filter.color.as_deref()?))
            .map(|[r, g, b]| Color32::from_rgb(r, g, b))
    }

    /// Countries present in the current selection: (code, label, count), by label.
    fn country_options(&self) -> Vec<(String, String, usize)> {
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for server in self.candidates() {
            *counts.entry(country_code(server).to_string()).or_default() += 1;
        }
        let mut options: Vec<(String, String, usize)> = counts
            .into_iter()
            .map(|(code, count)| {
                let label = country_label(&code);
                (code, label, count)
            })
            .collect();
        options.sort_by(|a, b| a.1.cmp(&b.1));
        options
    }

    fn visible_rows(&self) -> Vec<&Server> {
        let filters = &self.settings.filters;
        let candidates = self.candidates();
        let game = self.nav.game().and_then(catalog::game);

        let mut rows: Vec<&Server> = candidates
            .into_iter()
            .filter(|server| {
                let unanswered = server.error.is_some();
                if filters.hide_offline && unanswered {
                    return false;
                }
                if !filters.countries.is_empty()
                    && !filters.countries.contains(country_code(server))
                {
                    return false;
                }
                // Unscanned or silent servers cannot satisfy content filters.
                if !server.has_info() {
                    return !filters.not_empty
                        && filters.max_ping.is_none()
                        && !matches!(self.nav, Nav::GameType(..));
                }
                if filters.not_empty && server.num_players == 0 {
                    return false;
                }
                if filters.not_full && server.is_full() {
                    return false;
                }
                if filters.no_password && server.password {
                    return false;
                }
                if let (Some(limit), Some(ping)) = (filters.max_ping, server.ping)
                    && ping > limit
                {
                    return false;
                }
                if let Nav::GameType(_, value) = &self.nav
                    && !server.game_type.eq_ignore_ascii_case(value)
                {
                    return false;
                }
                game.is_none_or(|game| passes_groups(game, server, &filters.checked))
            })
            .collect();

        let (col, ascending) = self.sort;
        // Servers that answered always come first, whatever the sort direction.
        rows.sort_by(|a, b| {
            b.has_info().cmp(&a.has_info()).then_with(|| {
                let ordering = compare(col, a, b);
                if ascending {
                    ordering
                } else {
                    ordering.reverse()
                }
            })
        });
        rows
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
                let id = key.to_ref().key();
                if self
                    .slot_watch
                    .as_ref()
                    .is_some_and(|watch| watch.key == id)
                {
                    self.slot_watch = None;
                } else {
                    self.slot_watch = Some(SlotWatch::new(id, Instant::now()));
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

    // ----- UI: tree ----------------------------------------------------------

    fn tree(&mut self, ui: &mut egui::Ui) {
        ui.spacing_mut().item_spacing.y = 0.0;
        let installed_only = self.settings.filters.installed_only;
        // Cached: `ready_games` checks files on disk, which can block on a
        // network drive; refreshed when the settings are saved.
        if installed_only && self.ready_cache.is_none() {
            let ids = self
                .settings
                .ready_games()
                .iter()
                .map(|g| g.id.clone())
                .collect();
            self.ready_cache = Some(ids);
        }
        let ready: Vec<String> = if installed_only {
            self.ready_cache.clone().unwrap_or_default()
        } else {
            Vec::new()
        };
        let mut new_nav: Option<Nav> = None;
        let mut toggles: Vec<String> = Vec::new();
        let mut filter_clicks: Vec<(String, String)> = Vec::new();
        let mut modifier_clicks: Vec<&'static str> = Vec::new();
        let mut country_clicks: Vec<String> = Vec::new();

        let mut row = |ui: &mut egui::Ui,
                       depth: usize,
                       path: Option<&str>,
                       icon: TreeIcon<'_>,
                       label: &str,
                       selected: bool|
         -> bool {
            let open = path.map(|path| self.settings.open_nav.contains(path));
            let (toggled, clicked) = tree_row(ui, depth, open, icon, label, selected);
            if toggled && let Some(path) = path {
                toggles.push(path.to_string());
            }
            clicked
        };

        // Favorites, with ASE's Favorites1..8 groups.
        if row(
            ui,
            0,
            Some("Favorites"),
            TreeIcon::Star,
            tr("Favorites"),
            self.nav == Nav::Favorites,
        ) {
            new_nav = Some(Nav::Favorites);
        }
        if self.settings.open_nav.contains("Favorites") {
            for group in FAVORITE_GROUPS {
                let count = self.settings.favorites_in(group).len();
                let label = if count > 0 {
                    format!("{} ({count})", group_name(group))
                } else {
                    group_name(group)
                };
                let nav = Nav::FavoriteGroup(group.to_string());
                if row(ui, 1, None, TreeIcon::Folder, &label, self.nav == nav) {
                    new_nav = Some(nav);
                }
            }
        }

        // Friends, with Friends1..4 groups.
        if row(
            ui,
            0,
            Some("Friends"),
            TreeIcon::Friends,
            tr("Friends"),
            self.nav == Nav::Friends,
        ) {
            new_nav = Some(Nav::Friends);
        }
        if self.settings.open_nav.contains("Friends") {
            for group in FRIEND_GROUPS {
                let online = self
                    .settings
                    .friends
                    .iter()
                    .filter(|friend| self.settings.friend_group(friend) == group)
                    .filter(|friend| self.friend_location(friend).is_some())
                    .count();
                let label = if online > 0 {
                    fill(tr("{} ({} on)"), &[&group_name(group), &online])
                } else {
                    group_name(group)
                };
                let nav = Nav::FriendGroup(group.to_string());
                if row(ui, 1, None, TreeIcon::Folder, &label, self.nav == nav) {
                    new_nav = Some(nav);
                }
            }
        }

        // Special lists.
        row(
            ui,
            0,
            Some("Special"),
            TreeIcon::Folder,
            tr("Special"),
            false,
        );
        if self.settings.open_nav.contains("Special") {
            for special in Special::ALL {
                let nav = Nav::Special(special);
                if row(
                    ui,
                    1,
                    None,
                    TreeIcon::Folder,
                    special.label(),
                    self.nav == nav,
                ) {
                    new_nav = Some(nav);
                }
            }
            if row(
                ui,
                1,
                None,
                TreeIcon::Clock,
                tr("recently joined"),
                self.nav == Nav::Recent,
            ) {
                new_nav = Some(Nav::Recent);
            }
        }

        // User filters.
        if !self.settings.custom_filters.is_empty() {
            row(
                ui,
                0,
                Some("My filters"),
                TreeIcon::Funnel,
                tr("My filters"),
                false,
            );
            if self.settings.open_nav.contains("My filters") {
                for (index, filter) in self.settings.custom_filters.iter().enumerate() {
                    let label = match &filter.color {
                        Some(_) => fill(tr("{} (color)"), &[&filter.name]),
                        None => filter.name.clone(),
                    };
                    let nav = Nav::Custom(index);
                    if row(ui, 1, None, TreeIcon::Folder, &label, self.nav == nav) {
                        new_nav = Some(nav);
                    }
                }
            }
        }

        for game in catalog::games() {
            if !self.settings.game_ref(&game.id).visible
                || (installed_only && !ready.contains(&game.id))
            {
                continue;
            }
            let path = game.id.clone();
            // Server count of a scanned list, as in JASE: "Call of Duty 2 · 621".
            let label = match self.store.get(&game.id).map(Vec::len) {
                Some(count) if count > 0 => format!("{} · {count}", game.name),
                _ => game.name.clone(),
            };
            if row(
                ui,
                0,
                Some(&path),
                TreeIcon::Game(game, self.exe_icon(&game.id)),
                &label,
                self.nav == Nav::Game(game.id.clone()),
            ) {
                new_nav = Some(Nav::Game(game.id.clone()));
            }
            if !self.settings.open_nav.contains(&path) {
                continue;
            }
            for game_type in &game.game_types {
                let nav = Nav::GameType(game.id.clone(), game_type.value.clone());
                if row(
                    ui,
                    1,
                    None,
                    TreeIcon::Folder,
                    &game_type.name,
                    self.nav == nav,
                ) {
                    new_nav = Some(nav);
                }
            }
            for group in &game.filters {
                let group_path = format!("{}/{}", game.id, group.name);
                let checked_key = group_path.clone();
                let any = self
                    .settings
                    .filters
                    .checked
                    .get(&checked_key)
                    .is_some_and(|set| !set.is_empty());
                let label = if any {
                    format!("{} *", group.name)
                } else {
                    group.name.clone()
                };
                row(ui, 1, Some(&group_path), TreeIcon::Folder, &label, false);
                if !self.settings.open_nav.contains(&group_path) {
                    continue;
                }
                let options: Vec<String> = if group.dynamic {
                    dynamic_options(
                        self.store.get(&game.id).map(Vec::as_slice).unwrap_or(&[]),
                        &group.key,
                    )
                } else {
                    group
                        .options
                        .iter()
                        .map(|option| option.label.clone())
                        .collect()
                };
                if options.is_empty() {
                    row(ui, 2, None, TreeIcon::None, tr("(refresh to fill)"), false);
                }
                for option in options {
                    let checked = self
                        .settings
                        .filters
                        .checked
                        .get(&checked_key)
                        .is_some_and(|set| set.contains(&option));
                    let shown = if option.is_empty() {
                        tr(tr("(none)"))
                    } else {
                        option.as_str()
                    };
                    if row(ui, 2, None, TreeIcon::Check(checked), shown, false) {
                        filter_clicks.push((checked_key.clone(), option));
                    }
                }
            }
        }

        let filters = self.settings.filters.clone();
        row(
            ui,
            0,
            Some("Modifiers"),
            TreeIcon::Funnel,
            tr("Modifiers"),
            false,
        );
        if self.settings.open_nav.contains("Modifiers") {
            row(
                ui,
                1,
                Some("Modifiers/Players"),
                TreeIcon::Folder,
                tr("Players"),
                false,
            );
            if self.settings.open_nav.contains("Modifiers/Players") {
                for (id, label, value) in [
                    ("not_empty", tr("Not empty"), filters.not_empty),
                    ("not_full", tr("Not full"), filters.not_full),
                    ("no_password", tr("No password"), filters.no_password),
                    (
                        "hide_offline",
                        tr("Hide unresponsive"),
                        filters.hide_offline,
                    ),
                ] {
                    if row(ui, 2, None, TreeIcon::Check(value), label, false) {
                        modifier_clicks.push(id);
                    }
                }
            }
            row(
                ui,
                1,
                Some("Modifiers/Ping"),
                TreeIcon::Folder,
                tr("Ping"),
                false,
            );
            if self.settings.open_nav.contains("Modifiers/Ping") {
                for (id, limit) in [
                    ("ping50", 50),
                    ("ping100", 100),
                    ("ping150", 150),
                    ("ping250", 250),
                ] {
                    if row(
                        ui,
                        2,
                        None,
                        TreeIcon::Check(filters.max_ping == Some(limit)),
                        &fill(tr("less than {}"), &[&limit]),
                        false,
                    ) {
                        modifier_clicks.push(id);
                    }
                }
            }
            let any_country = !filters.countries.is_empty();
            row(
                ui,
                1,
                Some("Modifiers/Country"),
                TreeIcon::Folder,
                if any_country {
                    tr("Country *")
                } else {
                    tr("Country")
                },
                false,
            );
            if self.settings.open_nav.contains("Modifiers/Country") {
                let options = self.country_options();
                if options.is_empty() {
                    row(ui, 2, None, TreeIcon::None, tr("(refresh to fill)"), false);
                }
                for (code, label, count) in options {
                    let checked = filters.countries.contains(&code);
                    if row(
                        ui,
                        2,
                        None,
                        TreeIcon::Check(checked),
                        &format!("{label} ({count})"),
                        false,
                    ) {
                        country_clicks.push(code);
                    }
                }
            }
        }

        for code in country_clicks {
            let countries = &mut self.settings.filters.countries;
            if !countries.remove(&code) {
                countries.insert(code);
            }
            self.settings_dirty = true;
        }
        for path in toggles {
            if !self.settings.open_nav.remove(&path) {
                self.settings.open_nav.insert(path);
            }
            self.settings_dirty = true;
        }
        for (group, option) in filter_clicks {
            let set = self.settings.filters.checked.entry(group).or_default();
            if !set.remove(&option) {
                set.insert(option);
            }
            self.settings_dirty = true;
        }
        for id in modifier_clicks {
            let filters = &mut self.settings.filters;
            match id {
                "not_empty" => filters.not_empty ^= true,
                "not_full" => filters.not_full ^= true,
                "no_password" => filters.no_password ^= true,
                "hide_offline" => filters.hide_offline ^= true,
                ping => {
                    let limit = PING_LIMITS[["ping50", "ping100", "ping150", "ping250"]
                        .iter()
                        .position(|p| *p == ping)
                        .unwrap_or(0)];
                    filters.max_ping = if filters.max_ping == Some(limit) {
                        None
                    } else {
                        Some(limit)
                    };
                }
            }
            self.settings_dirty = true;
        }
        if let Some(nav) = new_nav {
            if let Some(game) = nav.game() {
                self.settings.last_game = game.to_string();
                let game = game.to_string();
                let first_visit = !self.store.contains_key(&game);
                if self.selected.as_ref().is_some_and(|key| key.game != game) {
                    self.selected = None;
                }
                self.nav = nav;
                if first_visit && self.scan.is_none() {
                    self.scan_game(ui.ctx(), &game);
                }
            } else {
                if let Some(keys) = self.nav_list(&nav) {
                    for key in &keys {
                        self.ensure_placeholder(key);
                    }
                }
                let lan_first_visit =
                    nav == Nav::Special(Special::Lan) && self.lan_servers.is_empty();
                // First visit: scan the ready games that have no list yet.
                let playing_first_visit = nav == Nav::Special(Special::Playing)
                    && self.settings.ready_games().iter().any(|game| {
                        !self
                            .store
                            .get(&game.id)
                            .is_some_and(|list| list.iter().any(|s| !s.saved))
                    });
                self.nav = nav;
                if lan_first_visit {
                    self.scan_lan(ui.ctx());
                }
                if playing_first_visit {
                    self.scan_ready_games(ui.ctx());
                }
            }
            self.settings_dirty = true;
        }
    }

    // ----- UI: server list ---------------------------------------------------

    fn server_list(&mut self, ui: &mut egui::Ui) {
        let want_scroll = std::mem::take(&mut self.scroll_to_selected);
        let rows = self.visible_rows();
        let mut actions: Vec<Action> = Vec::new();
        let mut sort_click: Option<Col> = None;
        let (sort_col, ascending) = self.sort;
        let selected = self.selected.clone();
        let text_color = ui.visuals().text_color();
        let visuals = ui.visuals().clone();
        let selection = ui.visuals().selection.bg_fill;
        // A "Load more" row ends the list while 333networks lists more servers.
        let more = self
            .nav
            .game()
            .filter(|game| self.paging.get(*game).is_some_and(Paging::has_more))
            .map(|game| {
                let paging = &self.paging[game];
                let shown = self.store.get(game).map_or(0, Vec::len);
                let total = paging.total + shown.saturating_sub(paging.listed);
                let loading = self.more_in_flight.is_some();
                (game.to_string(), shown, total, loading)
            });
        let mut on_screen: Vec<ServerKey> = Vec::new();
        let mut more_drawn = false;

        if rows.is_empty() && self.scan.is_none() && more.is_none() {
            self.on_screen.clear();
            ui.add_space(30.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new(tr("<- select a game and click Refresh, or add a server"))
                        .color(ui.visuals().weak_text_color()),
                );
            });
            return;
        }
        let scroll_to = if want_scroll {
            selected
                .as_ref()
                .and_then(|key| rows.iter().position(|server| ServerKey::of(server) == *key))
        } else {
            None
        };

        let (row_height, jase_look) = (self.row_height(), self.jase_look());
        let mut table = TableBuilder::new(ui);
        if let Some(index) = scroll_to {
            table = table.scroll_to_row(index, Some(Align::Center));
        }
        let mut table = table
            .id_salt("server-list")
            .striped(false)
            .resizable(true)
            .sense(Sense::click())
            .cell_layout(Layout::left_to_right(Align::Center))
            .auto_shrink(false);
        for (index, (_, _, width)) in COLUMNS.iter().enumerate() {
            let column = if index == COLUMNS.len() - 1 {
                Column::remainder()
            } else {
                Column::initial(*width)
            };
            table = table.column(column.at_least(24.0).clip(true));
        }

        let output = table
            .header(20.0, |mut header| {
                for (col, label, _) in COLUMNS {
                    header.col(|ui| {
                        let rect = ui.max_rect();
                        header_cell(ui, rect);
                        let label_widget = egui::Label::new(RichText::new(tr(label)))
                            .selectable(false)
                            .sense(Sense::click());
                        let response = if jase_look {
                            ui.centered_and_justified(|ui| ui.add(label_widget)).inner
                        } else {
                            ui.add(label_widget)
                        };
                        if response.clicked()
                            || ui
                                .interact(rect, ui.scope_id().with(label), Sense::click())
                                .clicked()
                        {
                            sort_click = Some(col);
                        }
                        if col == sort_col {
                            icons::sort_arrow(
                                ui.painter(),
                                egui::pos2(rect.right() - 8.0, rect.center().y),
                                ascending,
                            );
                        }
                    });
                }
            })
            .body(|body| {
                let count = rows.len() + usize::from(more.is_some());
                body.rows(row_height, count, |mut row| {
                    let Some(&server) = rows.get(row.index()) else {
                        // The "Load more" row.
                        let Some((game, shown, total, loading)) = &more else {
                            return;
                        };
                        more_drawn = true;
                        for (col, _, _) in COLUMNS {
                            row.col(|ui| {
                                if col != Col::Name {
                                    return;
                                }
                                let text = if *loading {
                                    tr("Loading more servers...").to_string()
                                } else {
                                    fill(tr("Load more servers ({} of {})"), &[shown, total])
                                };
                                if ui
                                    .add_enabled(!*loading, egui::Button::new(text).small())
                                    .clicked()
                                {
                                    actions.push(Action::LoadMore(game.clone()));
                                }
                            });
                        }
                        return;
                    };
                    let key = ServerKey::of(server);
                    on_screen.push(key.clone());
                    let is_selected = selected.as_ref() == Some(&key);
                    row.set_selected(is_selected);
                    let game = catalog::game(&server.game);
                    let favorite = self.settings.is_favorite(&key.to_ref());
                    let alert_id = key.to_ref().key();
                    let watching = self
                        .slot_watch
                        .as_ref()
                        .is_some_and(|watch| watch.key == alert_id);
                    let threshold = self.settings.party_alerts.get(&alert_id).copied();
                    let full = server.is_full();
                    let friend = self.server_has_friend(server);
                    let highlight = if friend {
                        Some(friend_fill(&visuals))
                    } else {
                        self.color_filter(server)
                    };
                    let color = if is_selected {
                        visuals.selection.stroke.color
                    } else if !server.has_info() {
                        Color32::from_gray(0x90)
                    } else {
                        text_color
                    };

                    for (col, _, _) in COLUMNS {
                        row.col(|ui| {
                            if is_selected {
                                ui.painter().rect_filled(
                                    ui.max_rect().expand2(vec2(4.0, 1.0)),
                                    CornerRadius::ZERO,
                                    selection,
                                );
                            } else if let Some(fill) = highlight {
                                ui.painter().rect_filled(
                                    ui.max_rect().expand2(vec2(4.0, 1.0)),
                                    CornerRadius::ZERO,
                                    fill,
                                );
                            }
                            match col {
                                Col::Icon => {
                                    let rect = ui.max_rect();
                                    if let Some(game) = game {
                                        let icon = egui::Rect::from_min_size(
                                            rect.left_center() + vec2(13.0, -7.0),
                                            vec2(14.0, 14.0),
                                        )
                                        .round_to_pixels(ui.pixels_per_point());
                                        icons::game_icon(
                                            ui.painter(),
                                            icon,
                                            game,
                                            self.exe_icon(&game.id),
                                        );
                                    }
                                    if server.password {
                                        icons::lock(
                                            ui.painter(),
                                            rect.left_center() + vec2(5.0, 0.0),
                                        );
                                    }
                                    if favorite {
                                        icons::star(
                                            ui.painter(),
                                            rect.left_center() + vec2(35.0, 0.0),
                                            5.0,
                                            Color32::from_rgb(0xf0, 0xc0, 0x1e),
                                            Color32::from_rgb(0x96, 0x6e, 0x00),
                                        );
                                    }
                                }
                                Col::Name => {
                                    let colored = self.palette.quake_colors
                                        && game
                                            .is_some_and(|game| game.protocol == Protocol::Quake3)
                                        && server.has_info()
                                        && !is_selected
                                        && self
                                            .settings
                                            .game_ref(&server.game)
                                            .draw_player_names_with_game_font;
                                    if colored {
                                        colored_label(ui, &server.name, color);
                                    } else {
                                        cell(ui, &display_name(server), color);
                                    }
                                }
                                Col::Location => {
                                    // The flag alone; the country or location shows on hover.
                                    let text = cell_text(col, server, game);
                                    let rect = ui.max_rect();
                                    let flag = server.country.as_deref().zip(self.flags.as_ref());
                                    if let Some((code, flags)) = flag
                                        && flags.draw(ui.painter(), rect, code)
                                    {
                                        if !text.is_empty() {
                                            ui.interact(
                                                rect,
                                                ui.scope_id().with("flag"),
                                                Sense::hover(),
                                            )
                                            .on_hover_text(text);
                                        }
                                    } else {
                                        cell(ui, &text, color);
                                    }
                                }
                                Col::Players => {
                                    // A small sunken meter at the left, as in JASE.
                                    if let Some(max) = server.max_players.filter(|&m| m > 0)
                                        && server.has_info()
                                    {
                                        let fraction =
                                            (server.num_players as f32 / max as f32).min(1.0);
                                        let fill = rgb(self.palette.theme.occupancy());
                                        icons::meter(ui, ui.max_rect(), fraction, fill);
                                    }
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        cell(ui, &cell_text(col, server, game), color);
                                    });
                                }
                                Col::Ping => {
                                    let ping_color = match server.ping {
                                        Some(ping) if server.online && !is_selected => {
                                            let level = openase::settings::ping_level(ping);
                                            rgb(self.palette.theme.ping_colors()[level])
                                        }
                                        _ => color,
                                    };
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        cell(ui, &cell_text(col, server, game), ping_color);
                                    });
                                }
                                Col::Flux | Col::Loss => {
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        cell(ui, &cell_text(col, server, game), color);
                                    });
                                }
                                _ => cell(ui, &cell_text(col, server, game), color),
                            }
                        });
                    }

                    let response = row.response();
                    if response.double_clicked() {
                        actions.push(Action::Join(key.clone()));
                    } else if response.clicked() || response.secondary_clicked() {
                        actions.push(Action::Select(key.clone()));
                    }
                    let address = server.join_address();
                    response.context_menu(|ui| {
                        if ui.button(tr("Join")).clicked() {
                            actions.push(Action::Join(key.clone()));
                            ui.close();
                        }
                        if ui.button(tr("Refresh")).clicked() {
                            actions.push(Action::Refresh(key.clone()));
                            ui.close();
                        }
                        let label = if favorite {
                            tr("Remove from favorites")
                        } else {
                            tr("Add to favorites")
                        };
                        if ui.button(label).clicked() {
                            actions.push(Action::ToggleFavorite(key.clone()));
                            ui.close();
                        }
                        ui.menu_button(tr("Add to favorites group"), |ui| {
                            for group in FAVORITE_GROUPS {
                                if ui.button(group_name(group)).clicked() {
                                    actions.push(Action::FavoriteGroup(
                                        key.clone(),
                                        group.to_string(),
                                    ));
                                    ui.close();
                                }
                            }
                        });
                        ui.separator();
                        if watching {
                            if ui.button(tr("Stop watching for a free slot")).clicked() {
                                actions.push(Action::WatchSlot(key.clone()));
                                ui.close();
                            }
                        } else if ui
                            .add_enabled(
                                full,
                                egui::Button::new(tr("Alert me when a slot is free")),
                            )
                            .clicked()
                        {
                            actions.push(Action::WatchSlot(key.clone()));
                            ui.close();
                        }
                        ui.menu_button(tr("Alert when players reach"), |ui| {
                            for n in [2, alerts::DEFAULT_THRESHOLD, 6, 8, 12, 16, 24, 32] {
                                let label = fill(tr("At {} players"), &[&n]);
                                if ui.radio(threshold == Some(n), label).clicked() {
                                    actions.push(Action::PartyAlert(key.clone(), n));
                                    ui.close();
                                }
                            }
                            if ui.radio(threshold.is_none(), tr("No alert")).clicked() {
                                actions.push(Action::PartyAlert(key.clone(), 0));
                                ui.close();
                            }
                        });
                        ui.separator();
                        if ui.button(tr("Copy address")).clicked() {
                            actions.push(Action::Copy(address.clone()));
                            ui.close();
                        }
                        let has_rcon = game.and_then(openase::rcon::kind_of).is_some();
                        if ui
                            .add_enabled(has_rcon, egui::Button::new(tr("RCON console...")))
                            .clicked()
                        {
                            actions.push(Action::Rcon(key.clone()));
                            ui.close();
                        }
                        if ui.button(tr("Copy openase:// link")).clicked() {
                            let link = openase::links::build(&key.game, &key.address);
                            actions.push(Action::Copy(link));
                            ui.close();
                        }
                    });
                });
            });

        // Scrolling down to the end of the list loads the next page once: only
        // when the user's scroll brings the "Load more" row into view, never
        // again while it stays in view (filtered pages, short lists).
        let offset = output.state.clamped_offset().y;
        let scrolled_down = offset > self.list_offset + 0.5;
        self.list_offset = offset;
        if more_drawn
            && !self.more_row_drawn
            && scrolled_down
            && let Some((game, _, _, false)) = &more
        {
            actions.push(Action::LoadMore(game.clone()));
        }
        self.more_row_drawn = more_drawn;
        self.on_screen = on_screen;

        if let Some(col) = sort_click {
            self.sort = if self.sort.0 == col {
                (col, !self.sort.1)
            } else {
                (col, !matches!(col, Col::Players))
            };
        }
        let ctx = ui.ctx().clone();
        for action in actions {
            self.apply(&ctx, action);
        }
    }

    // ----- UI: bottom panes --------------------------------------------------

    fn friends_pane(&mut self, ui: &mut egui::Ui) {
        let friends = self.settings.friends.clone();
        let text_color = ui.visuals().text_color();
        let green = online_green(ui.visuals());
        let mut select: Option<ServerKey> = None;
        // Room for the next column handle: a column widened to the edge stays shrinkable.
        let cap = (ui.available_width() - 40.0).max(60.0);
        let (row_height, jase_look) = (self.row_height(), self.jase_look());
        TableBuilder::new(ui)
            .id_salt("friends-pane")
            .resizable(true)
            .sense(Sense::click())
            .cell_layout(Layout::left_to_right(Align::Center))
            .auto_shrink(false)
            .column(
                Column::initial(118.0)
                    .at_least(40.0)
                    .at_most(cap)
                    .clip(true),
            )
            .column(Column::remainder().clip(true))
            .header(20.0, |mut header| {
                let online = friends
                    .iter()
                    .filter(|friend| self.friend_location(friend).is_some())
                    .count();
                for label in [
                    if online > 0 {
                        tr("Friend - on")
                    } else {
                        tr("Friend - off")
                    },
                    tr("Server"),
                ] {
                    header.col(|ui| {
                        header_cell(ui, ui.max_rect());
                        header_label(ui, label, text_color, jase_look);
                    });
                }
            })
            .body(|body| {
                body.rows(row_height, friends.len(), |mut row| {
                    let friend = &friends[row.index()];
                    let location = self.friend_location(friend).cloned();
                    let server_name = location
                        .as_ref()
                        .and_then(|key| self.find(key))
                        .map(|server| strip_quake_colors(&server.name))
                        .unwrap_or_default();
                    let color = if location.is_some() {
                        green
                    } else {
                        Color32::from_gray(0x80)
                    };
                    row.col(|ui| cell(ui, friend, color));
                    row.col(|ui| cell(ui, &server_name, text_color));
                    if row.response().clicked() {
                        select = location;
                    }
                });
            });
        if let Some(key) = select {
            self.nav = Nav::Game(key.game.clone());
            self.selected = Some(key);
        }
    }

    fn players_pane(&self, ui: &mut egui::Ui, server: Option<&Server>) {
        let text_color = ui.visuals().text_color();
        let players = server
            .map(|server| server.players.as_slice())
            .unwrap_or(&[]);
        let colored = self.palette.quake_colors
            && server.is_some_and(|server| {
                catalog::game(&server.game).is_some_and(|game| game.protocol == Protocol::Quake3)
                    && self
                        .settings
                        .game_ref(&server.game)
                        .draw_player_names_with_game_font
            });
        let friends: Vec<String> = self
            .settings
            .friends
            .iter()
            .map(|friend| friend.to_lowercase())
            .collect();
        let mut order: Vec<usize> = (0..players.len()).collect();
        order.sort_by_key(|&index| std::cmp::Reverse(players[index].score));

        // Room for the next column handle: a column widened to the edge stays shrinkable.
        let cap = (ui.available_width() - 40.0).max(60.0);
        let (row_height, jase_look) = (self.row_height(), self.jase_look());
        TableBuilder::new(ui)
            .id_salt("players-pane")
            .resizable(true)
            .striped(!jase_look)
            .cell_layout(Layout::left_to_right(Align::Center))
            .auto_shrink(false)
            .column(
                Column::initial(170.0)
                    .at_least(60.0)
                    .at_most(cap)
                    .clip(true),
            )
            .column(Column::initial(50.0).at_least(30.0).at_most(cap))
            .column(Column::initial(44.0).at_least(30.0).at_most(cap))
            .column(Column::remainder().at_least(40.0))
            .header(20.0, |mut header| {
                for label in [tr("Player Name"), tr("Score"), tr("Ping"), tr("Time")] {
                    header.col(|ui| {
                        header_cell(ui, ui.max_rect());
                        header_label(ui, label, text_color, jase_look);
                    });
                }
            })
            .body(|body| {
                body.rows(row_height, order.len(), |mut row| {
                    let player = &players[order[row.index()]];
                    let plain = strip_quake_colors(&player.name);
                    let is_friend = friends
                        .iter()
                        .any(|friend| plain.to_lowercase().contains(friend));
                    row.col(|ui| {
                        if is_friend {
                            ui.painter().rect_filled(
                                ui.max_rect(),
                                CornerRadius::ZERO,
                                friend_fill(ui.visuals()),
                            );
                        }
                        if colored {
                            colored_label(ui, &player.name, text_color);
                        } else {
                            cell(ui, &plain, text_color);
                        }
                    });
                    for value in [
                        player.score.to_string(),
                        player.ping.map(|ping| ping.to_string()).unwrap_or_default(),
                        player.time_secs.map(format_duration).unwrap_or_default(),
                    ] {
                        row.col(|ui| {
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                cell(ui, &value, text_color)
                            });
                        });
                    }
                });
            });
    }

    fn rules_pane(&self, ui: &mut egui::Ui, server: Option<&Server>) {
        let text_color = ui.visuals().text_color();
        let rules: Vec<(&String, &String)> = server
            .map(|server| server.rules.iter().collect())
            .unwrap_or_default();
        // Room for the next column handle: a column widened to the edge stays shrinkable.
        let cap = (ui.available_width() - 40.0).max(60.0);
        let (row_height, jase_look) = (self.row_height(), self.jase_look());
        TableBuilder::new(ui)
            .id_salt("rules-pane")
            .resizable(true)
            .striped(!jase_look)
            .cell_layout(Layout::left_to_right(Align::Center))
            .auto_shrink(false)
            .column(
                Column::initial(150.0)
                    .at_least(60.0)
                    .at_most(cap)
                    .clip(true),
            )
            .column(Column::remainder().clip(true))
            .header(20.0, |mut header| {
                for label in [tr("Key"), tr("Value")] {
                    header.col(|ui| {
                        header_cell(ui, ui.max_rect());
                        header_label(ui, label, text_color, jase_look);
                    });
                }
            })
            .body(|body| {
                body.rows(row_height, rules.len(), |mut row| {
                    let (key, value) = rules[row.index()];
                    row.col(|ui| cell(ui, key, text_color));
                    row.col(|ui| cell(ui, &strip_quake_colors(value), text_color));
                });
            });
    }

    fn status_bar(&self, ui: &mut egui::Ui) {
        let rows = self.visible_rows();
        let total: usize = match self.nav.game() {
            Some(game) => self.store.get(game).map_or(0, Vec::len),
            None => rows.len(),
        };
        let players: usize = rows.iter().map(|server| server.num_players).sum();
        let slots: usize = rows.iter().filter_map(|server| server.max_players).sum();
        ui.horizontal(|ui| {
            if let Some(scan) = &self.scan {
                let done = scan.done.load(Ordering::Relaxed);
                let fraction = if scan.total > 0 {
                    done as f32 / scan.total as f32
                } else {
                    0.0
                };
                ui.add(
                    egui::ProgressBar::new(fraction)
                        .desired_width(140.0)
                        .desired_height(12.0),
                );
                ui.label(format!("{} {}/{}", scan.label, done, scan.total));
            } else {
                ui.label(&self.status);
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let paged = self
                    .nav
                    .game()
                    .and_then(|game| self.paging.get(game))
                    .filter(|paging| paging.has_more());
                ui.label(match paged {
                    // "100 of 808 servers": the rest of the 333networks list
                    // is loaded on demand.
                    Some(paging) => fill(
                        tr("{} of {} servers, {}/{} players total, {} visible"),
                        &[
                            &total,
                            &(paging.total + total.saturating_sub(paging.listed)),
                            &players,
                            &slots,
                            &rows.len(),
                        ],
                    ),
                    None => fill(
                        tr("{} servers, {}/{} players total, {} visible"),
                        &[&total, &players, &slots, &rows.len()],
                    ),
                });
            });
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

fn run_queries(
    targets: Vec<(String, String)>,
    network: &openase::settings::NetworkSettings,
    tx: &Sender<Msg>,
    ctx: &egui::Context,
    generation: u64,
    stop: &AtomicBool,
    done: &AtomicUsize,
) {
    let options = network.query_options();
    let queue = Mutex::new(targets.into_iter().collect::<VecDeque<_>>());
    std::thread::scope(|scope| {
        for _ in 0..network.parallel_queries.clamp(1, 128) {
            scope.spawn(|| {
                loop {
                    if stop.load(Ordering::Relaxed) {
                        break;
                    }
                    let Some((game_id, address)) = queue.lock().unwrap().pop_front() else {
                        break;
                    };
                    let Some(game) = catalog::game(&game_id) else {
                        continue;
                    };
                    if let Ok((host, port)) = parse_address(&address, game.default_port) {
                        let server = query_server(game, &host, port, &options);
                        let _ = tx.send(Msg::Server { generation, server });
                    }
                    done.fetch_add(1, Ordering::Relaxed);
                    ctx.request_repaint_after(Duration::from_millis(100));
                }
            });
        }
    });
    let _ = tx.send(Msg::ScanDone { generation });
    ctx.request_repaint();
}

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

/// One tree line: indentation, optional `[+]`/`[-]` box, icon and label.
/// Returns (expander clicked, label clicked).
fn tree_row(
    ui: &mut egui::Ui,
    depth: usize,
    open: Option<bool>,
    icon: TreeIcon<'_>,
    label: &str,
    selected: bool,
) -> (bool, bool) {
    let height = 18.0;
    let indent = 14.0 * depth as f32;
    let font = TextStyle::Body.resolve(ui.style());
    let galley = ui
        .painter()
        // PLACEHOLDER: the color is chosen when painting (selection, theme).
        .layout_no_wrap(label.to_string(), font, Color32::PLACEHOLDER);
    let width = indent + 14.0 + 20.0 + galley.size().x + 8.0;
    let (rect, _) = ui.allocate_exact_size(
        vec2(width.max(ui.available_width()), height),
        Sense::hover(),
    );

    let expander_center = egui::pos2(rect.left() + indent + 7.0, rect.center().y);
    let expander_rect = egui::Rect::from_center_size(expander_center, vec2(12.0, 12.0));
    let icon_rect = egui::Rect::from_min_size(
        egui::pos2(rect.left() + indent + 14.0, rect.top()),
        vec2(20.0, height),
    );
    let label_rect = egui::Rect::from_min_size(
        egui::pos2(icon_rect.right() + 2.0, rect.top() + 1.0),
        vec2(galley.size().x + 6.0, height - 2.0),
    );

    let mut toggled = false;
    if let Some(open) = open {
        icons::expander(ui.painter(), expander_center, open);
        toggled = ui
            .interact(
                expander_rect,
                ui.scope_id()
                    .with(("expander", label, depth, rect.top() as i32)),
                Sense::click(),
            )
            .clicked();
    }
    icons::tree_icon(ui.painter(), icon_rect, icon);

    let response = ui.interact(
        label_rect.union(icon_rect),
        ui.scope_id()
            .with(("label", label, depth, rect.top() as i32)),
        Sense::click(),
    );
    let text_color = if selected {
        ui.visuals().selection.stroke.color
    } else if ui.visuals().dark_mode {
        // White names stand out on dark themes, as in JASE.
        ui.visuals().strong_text_color()
    } else if ui.visuals().text_color() == rgb(ColorSettings::default().text) {
        Color32::from_gray(0x10)
    } else {
        ui.visuals().text_color()
    };
    if selected {
        ui.painter().rect_filled(
            label_rect,
            CornerRadius::ZERO,
            ui.visuals().selection.bg_fill,
        );
    } else if response.hovered() {
        ui.painter().rect_filled(
            label_rect,
            CornerRadius::ZERO,
            ui.visuals().widgets.hovered.weak_bg_fill,
        );
    }
    ui.painter().galley(
        label_rect.left_top() + vec2(3.0, (label_rect.height() - galley.size().y) / 2.0),
        galley,
        text_color,
    );

    if response.double_clicked() && open.is_some() {
        toggled = true;
    }
    (toggled, response.clicked())
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

fn display_name(server: &Server) -> String {
    if server.has_info() {
        strip_quake_colors(&server.name)
    } else if server.error.is_some() {
        tr("(no response)").to_string()
    } else {
        tr("unscanned").to_string()
    }
}

fn cell_text(col: Col, server: &Server, game: Option<&Game>) -> String {
    let number = |value: Option<u32>| value.map(|v| v.to_string()).unwrap_or_default();
    match col {
        Col::Icon => String::new(),
        Col::Location => {
            // GameSpy servers often send `\location\0`, a region number:
            // the country (from the list or GeoIP) says more.
            let region_number = server.location.chars().all(|c| c.is_ascii_digit());
            if server.location != openase::location::UNKNOWN_LOCATION
                && !server.location.is_empty()
                && !(region_number && server.country.is_some())
            {
                server.location.clone()
            } else {
                server
                    .country
                    .as_deref()
                    .and_then(country_name)
                    .unwrap_or_default()
                    .to_string()
            }
        }
        Col::Name => display_name(server),
        Col::Ping => {
            if server.online {
                number(server.ping)
            } else if server.listed {
                // Listed by 333networks and not pinged yet.
                "—".to_string()
            } else {
                "9999".to_string()
            }
        }
        Col::Flux => server
            .flux
            .map(|flux| format!("±{flux}"))
            .unwrap_or_default(),
        Col::Loss => server
            .packet_loss
            .map(|loss| format!("{loss}%"))
            .unwrap_or_default(),
        Col::Players => {
            if server.has_info() {
                format!(
                    "{}/{}",
                    server.num_players,
                    server
                        .max_players
                        .map_or("?".to_string(), |max| max.to_string())
                )
            } else {
                String::new()
            }
        }
        Col::GameType => {
            // Masters sometimes list servers of a related game (Urban Terror on the
            // Quake 3 master): name the game type with that game's table.
            let naming = catalog::games()
                .iter()
                .find(|other| {
                    !other.base_mod.is_empty()
                        && other.base_mod.eq_ignore_ascii_case(&server.mod_name)
                })
                .or(game);
            let game_type = naming
                .map(|game| game.game_type_name(&server.game_type))
                .unwrap_or_default();
            match (
                game_type.is_empty(),
                game.map(|game| game.launch_mod(&server.mod_name))
                    .unwrap_or_default(),
            ) {
                (true, modname) => modname,
                (false, modname) if modname.is_empty() => game_type,
                (false, modname) => format!("{game_type} ({modname})"),
            }
        }
        Col::Map => server.map.clone(),
        Col::Address => server.join_address(),
    }
}

fn compare(col: Col, a: &Server, b: &Server) -> std::cmp::Ordering {
    let ping = |server: &Server| {
        if server.online {
            server.ping.unwrap_or(9999)
        } else {
            9999
        }
    };
    match col {
        Col::Icon => a.game.cmp(&b.game),
        Col::Ping => ping(a).cmp(&ping(b)),
        Col::Flux => a.flux.unwrap_or(9999).cmp(&b.flux.unwrap_or(9999)),
        Col::Loss => a
            .packet_loss
            .unwrap_or(101)
            .cmp(&b.packet_loss.unwrap_or(101)),
        Col::Players => a
            .num_players
            .cmp(&b.num_players)
            .then_with(|| ping(b).cmp(&ping(a))),
        Col::Address => {
            let parse = |server: &Server| {
                server
                    .host
                    .parse::<std::net::Ipv4Addr>()
                    .ok()
                    .map(u32::from)
                    .unwrap_or(0)
            };
            parse(a).cmp(&parse(b)).then(a.port.cmp(&b.port))
        }
        _ => {
            let game_a = catalog::game(&a.game);
            let game_b = catalog::game(&b.game);
            cell_text(col, a, game_a)
                .to_lowercase()
                .cmp(&cell_text(col, b, game_b).to_lowercase())
        }
    }
    .then_with(|| b.has_info().cmp(&a.has_info()))
}

/// Server passes every checked filter group of its game.
fn passes_groups(
    game: &Game,
    server: &Server,
    checked: &BTreeMap<String, std::collections::BTreeSet<String>>,
) -> bool {
    game.filters.iter().all(|group| {
        let Some(set) = checked
            .get(&format!("{}/{}", game.id, group.name))
            .filter(|set| !set.is_empty())
        else {
            return true;
        };
        let value = server.rule(&group.key).unwrap_or("");
        if group.dynamic {
            set.iter().any(|option| option.eq_ignore_ascii_case(value))
        } else {
            group
                .options
                .iter()
                .filter(|option| set.contains(&option.label))
                .any(|option| {
                    option
                        .values
                        .iter()
                        .any(|v| v.eq_ignore_ascii_case(value.trim()))
                })
        }
    })
}

/// Most common values of a rule among scanned servers, for dynamic filter groups.
fn dynamic_options(servers: &[Server], key: &str) -> Vec<String> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for server in servers.iter().filter(|server| server.has_info()) {
        *counts
            .entry(server.rule(key).unwrap_or("").to_string())
            .or_default() += 1;
    }
    let mut values: Vec<(String, usize)> = counts.into_iter().collect();
    values.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    values.truncate(25);
    values.sort_by_key(|a| a.0.to_lowercase());
    values.into_iter().map(|(value, _)| value).collect()
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
