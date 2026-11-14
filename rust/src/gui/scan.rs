//! Scanning: master lists, LAN, server queries and the worker messages.

use super::*;

impl OpenAseApp {
    // ----- scanning --------------------------------------------------------

    /// Remembered server lists behind list-type tree entries.
    pub(super) fn nav_list(&self, nav: &Nav) -> Option<Vec<ServerKey>> {
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

    pub(super) fn known_servers(&self) -> Vec<ServerKey> {
        self.store
            .values()
            .flatten()
            .filter(|server| server.online)
            .map(ServerKey::of)
            .collect()
    }

    pub(super) fn refresh(&mut self, ctx: &egui::Context) {
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

    pub(super) fn scan_lan(&mut self, ctx: &egui::Context) {
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

    pub(super) fn stop_scan(&mut self) {
        if let Some(scan) = &self.scan {
            scan.stop.store(true, Ordering::Relaxed);
        }
    }

    pub(super) fn start_scan(&mut self, label: String) -> (u64, Arc<AtomicBool>, Arc<AtomicUsize>) {
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

    pub(super) fn scan_game(&mut self, ctx: &egui::Context, game_id: &str) {
        let Some(game) = catalog::game(game_id) else {
            return;
        };
        self.status = fill(tr("Querying {} master servers..."), &[&game.name]);
        self.scan_games(ctx, vec![game], fill(tr("Scanning {}"), &[&game.name]));
    }

    /// "Playing now": scan every game the user can join.
    pub(super) fn scan_ready_games(&mut self, ctx: &egui::Context) {
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
    pub(super) fn scan_games(
        &mut self,
        ctx: &egui::Context,
        games: Vec<&'static Game>,
        label: String,
    ) {
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
    pub(super) fn load_more(&mut self, ctx: &egui::Context, game_id: &str) {
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
    pub(super) fn ping_visible(&mut self, ctx: &egui::Context) {
        let keys: Vec<ServerKey> = self
            .on_screen
            .iter()
            .take(PING_VISIBLE_MAX)
            .cloned()
            .collect();
        self.scan_list(ctx, keys, noop("visible servers"));
    }

    pub(super) fn scan_list(
        &mut self,
        ctx: &egui::Context,
        keys: Vec<ServerKey>,
        what: &'static str,
    ) {
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

    pub(super) fn drain_messages(&mut self, ctx: &egui::Context) {
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
}

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
