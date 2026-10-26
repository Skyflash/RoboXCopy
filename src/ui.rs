//! Hand-painted widgets for the X-Copy Pro look. Everything sits on a fixed virtual canvas
//! (W x H) that scales with the window, so the layout never reflows.

use crate::theme::Theme;
use eframe::egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Ui, vec2};

pub const W: f32 = 640.0;
pub const H: f32 = 330.0;

pub const CELL: f32 = 14.0;
pub const GRID_COLS: usize = 10;
pub const GRID_ROWS: usize = 8;
pub const GRID_CELLS: usize = GRID_COLS * GRID_ROWS;

#[derive(Clone, Copy)]
pub struct Canvas {
    pub origin: Pos2,
    pub s: f32,
    pub t: &'static Theme,
}

impl Canvas {
    pub fn fit(avail: Rect, t: &'static Theme) -> Self {
        let s = (avail.width() / W).min(avail.height() / H).max(0.4);
        let origin = avail.center() - vec2(W, H) * s / 2.0;
        Self { origin, s, t }
    }

    pub fn rect(&self, x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::from_min_size(self.origin + vec2(x, y) * self.s, vec2(w, h) * self.s)
    }

    pub fn font(&self, size: f32) -> FontId {
        FontId::monospace(size * self.s)
    }

    pub fn stroke(&self, w: f32, color: Color32) -> Stroke {
        Stroke::new((w * self.s).max(1.0), color)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Kind {
    /// Primary gadgets (START, REPEAT, STOP).
    Normal,
    /// Secondary gadgets; only differs from `Normal` in themes that colour them differently.
    Alt,
    Danger,
}

pub fn frame(ui: &Ui, c: &Canvas, r: Rect, color: Color32) {
    ui.painter().rect_stroke(r, 0.0, c.stroke(1.5, color), StrokeKind::Inside);
}

pub fn text(ui: &Ui, c: &Canvas, x: f32, y: f32, align: Align2, size: f32, color: Color32, s: &str) {
    ui.painter().text(c.origin + vec2(x, y) * c.s, align, s, c.font(size), color);
}

/// Horizontal rule across `x..x+w` at virtual height `y`.
pub fn sep(ui: &Ui, c: &Canvas, x: f32, y: f32, w: f32) {
    ui.painter().rect_filled(c.rect(x, y, w, 1.5), 0.0, c.t.sep);
}

/// Raised gadget; returns true when clicked.
pub fn button(ui: &mut Ui, c: &Canvas, r: Rect, label: &str, kind: Kind, enabled: bool) -> bool {
    // Position is part of the id: two BROWSE gadgets share a label.
    let id = ui.scope_id().with(("btn", label, (r.min.x - c.origin.x) as i32, (r.min.y - c.origin.y) as i32));
    let resp = ui.interact(r, id, if enabled { Sense::click() } else { Sense::hover() });
    let t = c.t;
    let (mut fill, mut fg) = match kind {
        Kind::Normal => (t.btn_fill, t.btn_text),
        Kind::Alt => (t.btn_fill, t.btn_text_alt),
        Kind::Danger => (t.danger_fill, t.danger_text),
    };
    if enabled && resp.is_pointer_button_down_on() {
        std::mem::swap(&mut fill, &mut fg);
    } else if enabled && resp.hovered() {
        fill = fill.lerp_to_gamma(Color32::WHITE, 0.2);
    }
    if !enabled {
        fill = fill.gamma_multiply(0.6);
        fg = fg.gamma_multiply(0.45);
    }
    let p = ui.painter();
    p.rect_filled(r, 0.0, fill);
    p.rect_stroke(r, 0.0, c.stroke(1.5, t.frame_dim), StrokeKind::Inside);
    p.text(r.center(), Align2::CENTER_CENTER, label, c.font(10.0), fg);
    enabled && resp.clicked()
}

/// A field value (click to cycle / toggle). Themes may draw it boxed or as bare text.
pub fn value_box(ui: &mut Ui, c: &Canvas, r: Rect, key: &str, value: &str, enabled: bool) -> bool {
    let id = ui.scope_id().with(("val", key));
    let resp = ui.interact(r, id, if enabled { Sense::click() } else { Sense::hover() });
    let hot = enabled && resp.hovered();
    let mut color = if hot { Color32::WHITE } else { c.t.value };
    if !enabled {
        color = color.gamma_multiply(0.55);
    }
    if c.t.value_box {
        ui.painter().rect_filled(r, 0.0, Color32::BLACK);
        let border = if hot { Color32::WHITE } else { c.t.frame_dim };
        ui.painter().rect_stroke(r, 0.0, c.stroke(1.0, border), StrokeKind::Inside);
    }
    ui.painter().text(r.center(), Align2::CENTER_CENTER, value, c.font(11.0), color);
    enabled && resp.clicked()
}

pub fn arrow(ui: &mut Ui, c: &Canvas, r: Rect, key: &str, up: bool, enabled: bool) -> bool {
    let id = ui.scope_id().with(("arrow", key, up));
    let resp = ui.interact(r, id, if enabled { Sense::click() } else { Sense::hover() });
    let color = if !enabled {
        c.t.arrow.gamma_multiply(0.4)
    } else if resp.hovered() {
        Color32::WHITE
    } else {
        c.t.arrow
    };
    let (l, rt, cx) = (r.left() + r.width() * 0.25, r.right() - r.width() * 0.25, r.center().x);
    let pts = if up {
        vec![Pos2::new(cx, r.top()), Pos2::new(l, r.bottom()), Pos2::new(rt, r.bottom())]
    } else {
        vec![Pos2::new(cx, r.bottom()), Pos2::new(l, r.top()), Pos2::new(rt, r.top())]
    };
    ui.painter().add(Shape::convex_polygon(pts, color, Stroke::NONE));
    enabled && resp.clicked()
}

/// Small filled dot marking a field whose value toggles on click.
pub fn dot(ui: &Ui, c: &Canvas, x: f32, y: f32) {
    let center = c.origin + vec2(x, y) * c.s;
    ui.painter().circle_filled(center, 3.0 * c.s, c.t.sub);
    ui.painter().circle_stroke(center, 3.0 * c.s, c.stroke(1.0, Color32::BLACK));
}

fn bulb(ui: &Ui, c: &Canvas, cx: f32, cy: f32, on: bool) {
    let p = ui.painter();
    let center = c.origin + vec2(cx, cy) * c.s;
    let glass = if on { c.t.bulb_on } else { c.t.bulb_off };
    p.circle_filled(center, 3.8 * c.s, glass);
    p.circle_stroke(center, 3.8 * c.s, c.stroke(1.0, Color32::BLACK));
    if on {
        p.circle_filled(center - vec2(1.2, 1.2) * c.s, 1.2 * c.s, Color32::WHITE);
    }
    p.rect_filled(c.rect(cx - 2.0, cy + 3.5, 4.0, 3.0), 0.0, Color32::from_gray(0x90));
}

/// Floppy-style drive icon with a light bulb above it. `lit` = this drive is selected.
pub fn drive_icon(ui: &mut Ui, c: &Canvas, x: f32, y: f32, letter: char, lit: bool, enabled: bool) -> bool {
    let r = c.rect(x, y + 9.0, 32.0, 28.0);
    let id = ui.scope_id().with(("drive", x as i32, y as i32));
    let resp = ui.interact(r, id, if enabled { Sense::click() } else { Sense::hover() });
    bulb(ui, c, x + 16.0, y + 3.0, lit);
    let mut fill = c.t.drive_fill;
    if enabled && resp.hovered() {
        fill = fill.lerp_to_gamma(Color32::WHITE, 0.2);
    }
    if !enabled {
        fill = fill.gamma_multiply(0.5);
    }
    let p = ui.painter();
    p.rect_filled(r, 0.0, fill);
    p.rect_stroke(r, 0.0, c.stroke(1.5, if lit { Color32::WHITE } else { c.t.drive_border }), StrokeKind::Inside);
    // Grey label with the dark slot, like the floppy icons on the original screen.
    p.rect_filled(c.rect(x + 6.0, y + 26.0, 20.0, 9.0), 0.0, Color32::from_gray(0xC8));
    p.rect_filled(c.rect(x + 9.0, y + 27.0, 4.0, 7.0), 0.0, c.t.drive_border);
    p.text(
        c.origin + vec2(x + 16.0, y + 17.0) * c.s,
        Align2::CENTER_CENTER,
        format!("{letter}:"),
        c.font(9.0),
        Color32::WHITE,
    );
    enabled && resp.clicked()
}

/// 10x8 cylinder grid. `lit` cells are on; `lap` flips the colours on each wrap-around.
pub fn grid(ui: &Ui, c: &Canvas, x: f32, y: f32, lit: usize, lap: usize) {
    let t = c.t;
    let (on, after) = if lap % 2 == 0 { (t.cell_on, t.cell_on_alt) } else { (t.cell_on_alt, t.cell_on) };
    let size = CELL + 1.0;
    for col in 0..GRID_COLS {
        text(ui, c, x + col as f32 * size + CELL / 2.0, y - 6.0, Align2::CENTER_CENTER, 8.0, t.label, &col.to_string());
    }
    for row in 0..GRID_ROWS {
        text(ui, c, x - 6.0, y + row as f32 * size + CELL / 2.0, Align2::CENTER_CENTER, 8.0, t.label, &row.to_string());
    }
    ui.painter().rect_filled(
        c.rect(x - 1.0, y - 1.0, GRID_COLS as f32 * size + 1.0, GRID_ROWS as f32 * size + 1.0),
        0.0,
        Color32::BLACK,
    );
    for i in 0..GRID_CELLS {
        let (col, row) = (i % GRID_COLS, i / GRID_COLS);
        let r = c.rect(x + col as f32 * size, y + row as f32 * size, CELL, CELL);
        let done = i < lit;
        let color = if done {
            on
        } else if lap > 0 {
            after
        } else {
            t.cell_off
        };
        if t.cell_glyph {
            if done || lap > 0 {
                ui.painter().text(r.center(), Align2::CENTER_CENTER, "0", c.font(11.0), color);
            }
        } else {
            ui.painter().rect_filled(r, 0.0, color);
        }
        if let Some(line) = t.grid_line {
            ui.painter().rect_stroke(r.expand(0.5 * c.s), 0.0, c.stroke(1.0, line), StrokeKind::Inside);
        }
    }
}
