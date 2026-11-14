//! Hand-drawn icons for the toolbar, tree and server list.
//!
//! Everything is painted with egui shapes so the app ships no image assets
//! and owes nothing to the original program's artwork.

use std::f32::consts::TAU;

use eframe::egui::{
    self, Color32, CornerRadius, FontId, Pos2, Rect, Response, Sense, Shape, Stroke, StrokeKind,
    Vec2, pos2, vec2,
};

use eframe::emath::GuiRounding;
use openase::catalog::Game;

const FLAG_ATLAS: &[u8] = include_bytes!("../../assets/flags.png");
const FLAG_CODES: &str = include_str!("../../assets/flags.txt");
/// A flag in the atlas (twice the drawn size, for high-DPI screens).
const FLAG_CELL: Vec2 = vec2(40.0, 30.0);
/// Drawn size, in points.
const FLAG_SIZE: Vec2 = vec2(20.0, 15.0);
const FLAGS_PER_ROW: usize = 16;

/// A sunken occupancy meter, 28x11 points at the left of `rect`.
pub fn meter(ui: &egui::Ui, rect: Rect, fraction: f32, fill: Color32) {
    let visuals = ui.visuals();
    let painter = ui.painter();
    let corner = (rect.left_center() + vec2(1.0, -5.5)).round_to_pixels(painter.pixels_per_point());
    let frame = Rect::from_min_size(corner, vec2(28.0, 11.0));
    painter.rect_filled(frame, CornerRadius::ZERO, visuals.extreme_bg_color);
    let mut bar = frame.shrink(2.0);
    bar.set_width(bar.width() * fraction);
    if fraction > 0.0 {
        painter.rect_filled(bar, CornerRadius::ZERO, fill);
    }
    let dark = visuals.widgets.noninteractive.bg_stroke.color;
    sunken(painter, frame, dark, visuals.widgets.inactive.bg_fill);
}

/// Sunken edges: dark top and left, light bottom and right.
pub fn sunken(painter: &egui::Painter, rect: Rect, dark: Color32, light: Color32) {
    edges(painter, rect, dark, light);
}

/// One-pixel edges on whole pixels: `top_left` on the top and left sides,
/// `bottom_right` on the others (sunken: dark then light; raised: the
/// reverse).
pub fn edges(painter: &egui::Painter, rect: Rect, top_left: Color32, bottom_right: Color32) {
    let ppp = painter.pixels_per_point();
    let r = rect.round_to_pixels(ppp);
    let px = 1.0 / ppp;
    let fill = |area: Rect, color: Color32| painter.rect_filled(area, CornerRadius::ZERO, color);
    fill(
        Rect::from_min_size(r.left_top(), vec2(r.width(), px)),
        top_left,
    );
    fill(
        Rect::from_min_size(r.left_top(), vec2(px, r.height())),
        top_left,
    );
    fill(
        Rect::from_min_size(pos2(r.left(), r.bottom() - px), vec2(r.width(), px)),
        bottom_right,
    );
    fill(
        Rect::from_min_size(pos2(r.right() - px, r.top()), vec2(px, r.height())),
        bottom_right,
    );
}

/// Country flags: one atlas texture (`rust/assets/flags.png`, from
/// flag-icons, MIT), loaded once.
pub struct Flags {
    texture: egui::TextureHandle,
    codes: Vec<&'static str>,
}

impl Flags {
    pub fn load(ctx: &egui::Context) -> Option<Flags> {
        let atlas = image::load_from_memory(FLAG_ATLAS).ok()?.into_rgba8();
        let size = [atlas.width() as usize, atlas.height() as usize];
        let image = egui::ColorImage::from_rgba_unmultiplied(size, atlas.as_raw());
        // Mipmaps keep the 13 stripes of a 40x30 flag drawn at 20x15 crisp.
        let options =
            egui::TextureOptions::LINEAR.with_mipmap_mode(Some(egui::TextureFilter::Linear));
        let texture = ctx.load_texture("flags", image, options);
        Some(Flags {
            texture,
            codes: FLAG_CODES.lines().collect(),
        })
    }

    /// Draw the flag of an ISO country code at the left of `rect`, 20x15
    /// points on whole pixels; false when there is none.
    pub fn draw(&self, painter: &egui::Painter, rect: Rect, code: &str) -> bool {
        let code = code.to_ascii_lowercase();
        let Some(index) = self.codes.iter().position(|c| *c == code) else {
            return false;
        };
        let atlas = self.texture.size_vec2();
        let cell = pos2(
            (index % FLAGS_PER_ROW) as f32 * FLAG_CELL.x,
            (index / FLAGS_PER_ROW) as f32 * FLAG_CELL.y,
        );
        let uv = Rect::from_min_size(
            pos2(cell.x / atlas.x, cell.y / atlas.y),
            vec2(FLAG_CELL.x / atlas.x, FLAG_CELL.y / atlas.y),
        );
        let corner = (rect.left_center() + vec2(1.0, -FLAG_SIZE.y / 2.0))
            .round_to_pixels(painter.pixels_per_point());
        let target = Rect::from_min_size(corner, FLAG_SIZE);
        painter.image(self.texture.id(), target, uv, Color32::WHITE);
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Refresh,
    Stop,
    Ping,
    Add,
    Join,
    Favorite,
    Find,
    Friends,
    Options,
}

const DARK: Color32 = Color32::from_rgb(0x30, 0x30, 0x30);

/// A flat toolbar button with a painted icon, in the style of a 2000s toolbar.
pub fn tool_button(ui: &mut egui::Ui, tool: Tool, tooltip: &str, enabled: bool) -> Response {
    let (rect, response) = ui.allocate_exact_size(
        vec2(26.0, 24.0),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    let painter = ui.painter();
    if enabled && (response.hovered() || response.is_pointer_button_down_on()) {
        let down = response.is_pointer_button_down_on();
        painter.rect_filled(
            rect,
            CornerRadius::ZERO,
            if down {
                Color32::from_gray(0xd8)
            } else {
                Color32::from_gray(0xe8)
            },
        );
        painter.rect_stroke(
            rect,
            CornerRadius::ZERO,
            Stroke::new(1.0, Color32::from_gray(0x99)),
            StrokeKind::Inside,
        );
    }
    paint_tool(painter, rect.shrink(4.0), tool, enabled);
    response.on_hover_text(tooltip)
}

fn paint_tool(painter: &egui::Painter, r: Rect, tool: Tool, enabled: bool) {
    let fade = |color: Color32| {
        if enabled {
            color
        } else {
            Color32::from_gray(0xb0)
        }
    };
    let c = r.center();
    match tool {
        Tool::Refresh => {
            let color = fade(Color32::from_rgb(0x1e, 0x6e, 0xc8));
            let radius = r.width() * 0.38;
            let points: Vec<Pos2> = (0..=20)
                .map(|i| {
                    let angle = TAU * (0.10 + 0.75 * i as f32 / 20.0);
                    c + radius * Vec2::angled(angle)
                })
                .collect();
            let end = *points.last().unwrap();
            painter.add(Shape::line(points, Stroke::new(2.2, color)));
            let dir = Vec2::angled(TAU * 0.85 + TAU / 4.0);
            let side = dir.rot90();
            painter.add(Shape::convex_polygon(
                vec![end + dir * 4.0, end + side * 4.0, end - side * 4.0],
                color,
                Stroke::NONE,
            ));
        }
        Tool::Stop => {
            painter.rect_filled(
                r.shrink(1.5),
                CornerRadius::same(2),
                fade(Color32::from_rgb(0xc8, 0x28, 0x1e)),
            );
        }
        Tool::Ping => {
            let color = fade(Color32::from_rgb(0xe6, 0xa0, 0x00));
            let (w, h) = (r.width(), r.height());
            let p = |x: f32, y: f32| pos2(r.left() + x * w, r.top() + y * h);
            painter.add(Shape::convex_polygon(
                vec![p(0.62, 0.0), p(0.15, 0.58), p(0.48, 0.58)],
                color,
                Stroke::NONE,
            ));
            painter.add(Shape::convex_polygon(
                vec![p(0.52, 0.42), p(0.85, 0.42), p(0.38, 1.0)],
                color,
                Stroke::NONE,
            ));
        }
        Tool::Add => {
            let color = fade(Color32::from_rgb(0x28, 0x96, 0x32));
            let stroke = Stroke::new(3.0, color);
            painter.line_segment(
                [pos2(c.x, r.top() + 1.0), pos2(c.x, r.bottom() - 1.0)],
                stroke,
            );
            painter.line_segment(
                [pos2(r.left() + 1.0, c.y), pos2(r.right() - 1.0, c.y)],
                stroke,
            );
        }
        Tool::Join => {
            let color = fade(Color32::from_rgb(0x28, 0x96, 0x32));
            painter.add(Shape::convex_polygon(
                vec![
                    pos2(r.left() + 2.0, r.top()),
                    pos2(r.right(), c.y),
                    pos2(r.left() + 2.0, r.bottom()),
                ],
                color,
                Stroke::new(1.0, fade(Color32::from_rgb(0x14, 0x5a, 0x1e))),
            ));
        }
        Tool::Favorite => star(
            painter,
            c,
            r.width() * 0.55,
            fade(Color32::from_rgb(0xf0, 0xc0, 0x1e)),
            fade(Color32::from_rgb(0x96, 0x6e, 0x00)),
        ),
        Tool::Find => {
            let color = fade(DARK);
            let lens = c + vec2(-2.0, -2.0);
            painter.circle_filled(lens, 5.0, fade(Color32::from_rgb(0xc8, 0xe6, 0xff)));
            painter.circle_stroke(lens, 5.0, Stroke::new(1.8, color));
            painter.line_segment(
                [lens + vec2(3.6, 3.6), r.right_bottom()],
                Stroke::new(2.6, color),
            );
        }
        Tool::Friends => {
            friends(
                painter,
                r,
                fade(Color32::from_rgb(0x1e, 0x6e, 0xc8)),
                fade(Color32::from_rgb(0x28, 0x96, 0x32)),
            );
        }
        Tool::Options => {
            let color = fade(Color32::from_gray(0x60));
            for i in 0..8 {
                let dir = Vec2::angled(TAU * i as f32 / 8.0);
                painter.line_segment([c + dir * 3.0, c + dir * 7.5], Stroke::new(3.0, color));
            }
            painter.circle_filled(c, 5.2, color);
            painter.circle_filled(c, 2.0, Color32::from_gray(0xf0));
        }
    }
}

/// Five-pointed star.
pub fn star(painter: &egui::Painter, center: Pos2, radius: f32, fill: Color32, stroke: Color32) {
    let points: Vec<Pos2> = (0..10)
        .map(|i| {
            let r = if i % 2 == 0 { radius } else { radius * 0.45 };
            center + r * Vec2::angled(-TAU / 4.0 + TAU * i as f32 / 10.0)
        })
        .collect();
    // A star is concave: paint it as five triangles around the center plus an outline.
    for i in 0..5 {
        let a = points[i * 2];
        let b = points[(i * 2 + 1) % 10];
        let z = points[(i * 2 + 9) % 10];
        painter.add(Shape::convex_polygon(
            vec![z, a, b, center],
            fill,
            Stroke::NONE,
        ));
    }
    let mut outline = points.clone();
    outline.push(points[0]);
    painter.add(Shape::line(outline, Stroke::new(1.0, stroke)));
}

fn friends(painter: &egui::Painter, r: Rect, back: Color32, front: Color32) {
    for (offset, color) in [(vec2(3.0, -1.0), back), (vec2(-3.0, 1.0), front)] {
        let c = r.center() + offset;
        painter.circle_filled(c + vec2(0.0, -3.5), 3.0, color);
        let body: Vec<Pos2> = (0..=10)
            .map(|i| {
                c + vec2(0.0, 6.0) + 5.0 * Vec2::angled(TAU / 2.0 + TAU / 2.0 * i as f32 / 10.0)
            })
            .collect();
        painter.add(Shape::convex_polygon(body, color, Stroke::NONE));
    }
}

/// Small colored game badge with the game's short name.
pub fn badge(painter: &egui::Painter, rect: Rect, game: &Game) {
    let [r, g, b] = game.rgb();
    painter.rect_filled(rect, CornerRadius::same(2), Color32::from_rgb(r, g, b));
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        &game.short,
        FontId::proportional(if game.short.len() > 3 { 8.0 } else { 9.0 }),
        Color32::WHITE,
    );
}

/// Padlock for password-protected servers.
pub fn lock(painter: &egui::Painter, center: Pos2) {
    let color = Color32::from_rgb(0x8c, 0x6e, 0x14);
    painter.circle_stroke(center + vec2(0.0, -2.5), 2.6, Stroke::new(1.3, color));
    painter.rect_filled(
        Rect::from_center_size(center + vec2(0.0, 1.8), vec2(7.0, 5.0)),
        CornerRadius::same(1),
        color,
    );
}

#[derive(Debug, Clone, Copy)]
pub enum TreeIcon<'a> {
    None,
    Folder,
    Check(bool),
    Game(&'a Game),
    Star,
    Clock,
    Friends,
    Funnel,
}

pub fn tree_icon(painter: &egui::Painter, rect: Rect, icon: TreeIcon<'_>) {
    let c = rect.center();
    match icon {
        TreeIcon::None => {}
        TreeIcon::Folder => {
            let body = Rect::from_center_size(c + vec2(0.0, 1.0), vec2(13.0, 9.0));
            let tab = Rect::from_min_size(body.left_top() + vec2(0.0, -2.0), vec2(6.0, 3.0));
            let fill = Color32::from_rgb(0xf5, 0xd2, 0x5a);
            let edge = Stroke::new(1.0, Color32::from_rgb(0xb4, 0x8c, 0x1e));
            painter.rect_filled(tab, CornerRadius::ZERO, fill);
            painter.rect_filled(body, CornerRadius::ZERO, fill);
            painter.rect_stroke(body, CornerRadius::ZERO, edge, StrokeKind::Inside);
        }
        TreeIcon::Check(checked) => {
            let b = Rect::from_center_size(c, vec2(11.0, 11.0));
            painter.rect_filled(b, CornerRadius::ZERO, Color32::WHITE);
            painter.rect_stroke(
                b,
                CornerRadius::ZERO,
                Stroke::new(1.0, Color32::from_gray(0x50)),
                StrokeKind::Inside,
            );
            if checked {
                painter.add(Shape::line(
                    vec![
                        b.left_center() + vec2(2.5, 0.0),
                        b.center_bottom() + vec2(-1.0, -3.0),
                        b.right_top() + vec2(-2.5, 2.5),
                    ],
                    Stroke::new(1.8, Color32::from_rgb(0x14, 0x14, 0x14)),
                ));
            }
        }
        TreeIcon::Game(game) => badge(painter, Rect::from_center_size(c, vec2(16.0, 12.0)), game),
        TreeIcon::Star => star(
            painter,
            c,
            6.5,
            Color32::from_rgb(0xf0, 0xc0, 0x1e),
            Color32::from_rgb(0x96, 0x6e, 0x00),
        ),
        TreeIcon::Clock => {
            painter.circle_filled(c, 6.0, Color32::WHITE);
            painter.circle_stroke(c, 6.0, Stroke::new(1.2, Color32::from_gray(0x50)));
            painter.line_segment([c, c + vec2(0.0, -4.0)], Stroke::new(1.2, DARK));
            painter.line_segment([c, c + vec2(3.0, 0.0)], Stroke::new(1.2, DARK));
        }
        TreeIcon::Friends => friends(
            painter,
            Rect::from_center_size(c, vec2(14.0, 14.0)),
            Color32::from_rgb(0x1e, 0x6e, 0xc8),
            Color32::from_rgb(0x28, 0x96, 0x32),
        ),
        TreeIcon::Funnel => {
            let color = Color32::from_gray(0x60);
            painter.add(Shape::convex_polygon(
                vec![
                    c + vec2(-6.0, -5.0),
                    c + vec2(6.0, -5.0),
                    c + vec2(1.5, 0.5),
                    c + vec2(-1.5, 0.5),
                ],
                color,
                Stroke::NONE,
            ));
            painter.rect_filled(
                Rect::from_min_max(c + vec2(-1.5, 0.0), c + vec2(1.5, 6.0)),
                CornerRadius::ZERO,
                color,
            );
        }
    }
}

/// The classic `[+]` / `[-]` tree expander box.
pub fn expander(painter: &egui::Painter, center: Pos2, open: bool) {
    let b = Rect::from_center_size(center, vec2(9.0, 9.0));
    painter.rect_filled(b, CornerRadius::ZERO, Color32::WHITE);
    painter.rect_stroke(
        b,
        CornerRadius::ZERO,
        Stroke::new(1.0, Color32::from_gray(0x90)),
        StrokeKind::Inside,
    );
    let stroke = Stroke::new(1.0, Color32::from_gray(0x20));
    painter.line_segment([center + vec2(-2.5, 0.0), center + vec2(2.5, 0.0)], stroke);
    if !open {
        painter.line_segment([center + vec2(0.0, -2.5), center + vec2(0.0, 2.5)], stroke);
    }
}

/// Small sort arrow for column headers.
pub fn sort_arrow(painter: &egui::Painter, center: Pos2, ascending: bool) {
    let color = Color32::from_gray(0x50);
    let points = if ascending {
        vec![
            center + vec2(-3.5, 2.0),
            center + vec2(3.5, 2.0),
            center + vec2(0.0, -2.0),
        ]
    } else {
        vec![
            center + vec2(-3.5, -2.0),
            center + vec2(3.5, -2.0),
            center + vec2(0.0, 2.0),
        ]
    };
    painter.add(Shape::convex_polygon(points, color, Stroke::NONE));
}
