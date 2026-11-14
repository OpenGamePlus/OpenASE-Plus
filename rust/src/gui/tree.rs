//! The navigation tree (games, favorites, filters, modifiers).

use super::*;

impl OpenAseApp {
    // ----- UI: tree ----------------------------------------------------------

    pub(super) fn tree(&mut self, ui: &mut egui::Ui) {
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
