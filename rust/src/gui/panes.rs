//! The bottom panes (friends, players, rules) and the status bar.

use super::*;

impl OpenAseApp {
    // ----- UI: bottom panes --------------------------------------------------

    pub(super) fn friends_pane(&mut self, ui: &mut egui::Ui) {
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

    pub(super) fn players_pane(&self, ui: &mut egui::Ui, server: Option<&Server>) {
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

    pub(super) fn rules_pane(&self, ui: &mut egui::Ui, server: Option<&Server>) {
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

    pub(super) fn status_bar(&self, ui: &mut egui::Ui) {
        // Counting needs no sort: the list sorts once per frame.
        let rows = self.filtered_rows();
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
