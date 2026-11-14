//! The server list: filtering, sorting and the table.

use super::*;

impl OpenAseApp {
    // ----- view ------------------------------------------------------------

    /// Servers of the current tree selection, before filters.
    pub(super) fn candidates(&self) -> Vec<&Server> {
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
    pub(super) fn color_filter(&self, server: &Server) -> Option<Color32> {
        self.settings
            .custom_filters
            .iter()
            .filter(|filter| filter.color.is_some() && !filter.rules.is_empty())
            .find(|filter| filter.matches(server))
            .and_then(|filter| parse_hex(filter.color.as_deref()?))
            .map(|[r, g, b]| Color32::from_rgb(r, g, b))
    }

    /// Countries present in the current selection: (code, label, count), by label.
    pub(super) fn country_options(&self) -> Vec<(String, String, usize)> {
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

    /// The rows of the list, filtered and sorted.
    pub(super) fn visible_rows(&self) -> Vec<&Server> {
        self.sorted(self.filtered_rows())
    }

    /// The rows the filters keep, in no particular order (enough to count).
    pub(super) fn filtered_rows(&self) -> Vec<&Server> {
        let filters = &self.settings.filters;
        let candidates = self.candidates();
        let game = self.nav.game().and_then(catalog::game);
        // The checked filter groups of the game, resolved once, not per server.
        let groups = game
            .map(|game| active_groups(game, &filters.checked))
            .unwrap_or_default();

        candidates
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
                passes_groups(&groups, server)
            })
            .collect()
    }

    /// `rows` in the list's sort order.
    pub(super) fn sorted<'a>(&self, mut rows: Vec<&'a Server>) -> Vec<&'a Server> {
        let (col, ascending) = self.sort;
        let direction = |ordering: std::cmp::Ordering| {
            if ascending {
                ordering
            } else {
                ordering.reverse()
            }
        };
        // Servers that answered always come first, whatever the sort direction.
        if is_text_column(col) {
            // Text keys (names without color codes, lower case) are built once
            // per row: in the comparator they cost regexes per comparison.
            let mut keyed: Vec<(bool, String, &Server)> = rows
                .into_iter()
                .map(|server| (server.has_info(), sort_text(col, server), server))
                .collect();
            keyed.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| direction(a.1.cmp(&b.1))));
            return keyed.into_iter().map(|(_, _, server)| server).collect();
        }
        rows.sort_by(|a, b| {
            b.has_info()
                .cmp(&a.has_info())
                .then_with(|| direction(compare(col, a, b)))
        });
        rows
    }

    // ----- UI: server list ---------------------------------------------------

    pub(super) fn server_list(&mut self, ui: &mut egui::Ui) {
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
                        // The flag column is too narrow for its name: shown on hover.
                        let text = if col == Col::Location { "" } else { tr(label) };
                        let label_widget = egui::Label::new(RichText::new(text))
                            .selectable(false)
                            .sense(Sense::click());
                        let response = if jase_look {
                            ui.centered_and_justified(|ui| ui.add(label_widget)).inner
                        } else {
                            ui.add(label_widget)
                        };
                        let response = if col == Col::Location {
                            response.on_hover_text(tr(label))
                        } else {
                            response
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
                        .is_some_and(|watch| watch.server == key.to_ref());
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
                                        colored_label(
                                            ui,
                                            server.name.trim_start_matches(|c: char| {
                                                c.is_whitespace() || c.is_control()
                                            }),
                                            color,
                                        );
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
}

fn display_name(server: &Server) -> String {
    if server.has_info() {
        strip_quake_colors(&server.name).trim().to_string()
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
        _ => sort_text(col, a).cmp(&sort_text(col, b)),
    }
    .then_with(|| b.has_info().cmp(&a.has_info()))
}

/// Columns sorted by their text, see [`sort_text`].
fn is_text_column(col: Col) -> bool {
    !matches!(
        col,
        Col::Icon | Col::Ping | Col::Flux | Col::Loss | Col::Players | Col::Address
    )
}

/// The text a column sorts by: as shown, in lower case.
fn sort_text(col: Col, server: &Server) -> String {
    cell_text(col, server, catalog::game(&server.game)).to_lowercase()
}

/// The filter groups of `game` with at least one option checked.
fn active_groups<'a>(
    game: &'a Game,
    checked: &'a BTreeMap<String, std::collections::BTreeSet<String>>,
) -> Vec<(
    &'a openase::catalog::FilterGroup,
    &'a std::collections::BTreeSet<String>,
)> {
    game.filters
        .iter()
        .filter_map(|group| {
            let set = checked.get(&format!("{}/{}", game.id, group.name))?;
            (!set.is_empty()).then_some((group, set))
        })
        .collect()
}

/// Server passes every active filter group ([`active_groups`]).
fn passes_groups(
    groups: &[(
        &openase::catalog::FilterGroup,
        &std::collections::BTreeSet<String>,
    )],
    server: &Server,
) -> bool {
    groups.iter().all(|(group, set)| {
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
