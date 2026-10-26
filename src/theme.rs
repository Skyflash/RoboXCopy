//! Colour themes. The main screen is hand-painted from a `Theme`; egui visuals only matter for
//! the text edits and pop-up windows.

use eframe::egui::{self, Color32, FontFamily, FontId, Stroke, TextStyle};

const fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

pub struct Theme {
    pub name: &'static str,
    pub bg: Color32,
    pub panel: Color32,
    /// Main frames (banner, parameter row, status bar).
    pub frame: Color32,
    pub frame_dim: Color32,
    pub label: Color32,
    pub heading: Color32,
    pub value: Color32,
    /// Draw a boxed black background behind field values (the classic screen has none).
    pub value_box: bool,
    pub btn_fill: Color32,
    pub btn_text: Color32,
    pub btn_text_alt: Color32,
    pub danger_fill: Color32,
    pub danger_text: Color32,
    pub sep: Color32,
    pub arrow: Color32,
    pub grid_line: Option<Color32>,
    pub cell_off: Color32,
    pub cell_on: Color32,
    pub cell_on_alt: Color32,
    /// Finished cells show a "0" glyph instead of a filled block.
    pub cell_glyph: bool,
    pub bulb_on: Color32,
    pub bulb_off: Color32,
    pub drive_fill: Color32,
    pub drive_border: Color32,
    pub status_text: Color32,
    pub title: Color32,
    pub title_hi: Color32,
    pub title_shadow: Color32,
    pub sub: Color32,
}

pub static MODERN: Theme = Theme {
    name: "MODERN AMIGA",
    bg: rgb(0x0A, 0x1E, 0x78),
    panel: rgb(0x05, 0x10, 0x48),
    frame: rgb(0x30, 0xD0, 0xF0),
    frame_dim: rgb(0x10, 0x80, 0xA8),
    label: Color32::WHITE,
    heading: rgb(0x30, 0xD0, 0xF0),
    value: rgb(0xFF, 0xB0, 0x20),
    value_box: true,
    btn_fill: rgb(0x20, 0x50, 0xC8),
    btn_text: rgb(0x30, 0xD0, 0xF0),
    btn_text_alt: rgb(0x30, 0xD0, 0xF0),
    danger_fill: rgb(0xC8, 0x20, 0x48),
    danger_text: Color32::WHITE,
    sep: rgb(0x10, 0x80, 0xA8),
    arrow: rgb(0x30, 0xD0, 0xF0),
    grid_line: None,
    cell_off: rgb(0x12, 0x30, 0x98),
    cell_on: rgb(0xFF, 0xB0, 0x20),
    cell_on_alt: Color32::WHITE,
    cell_glyph: false,
    bulb_on: rgb(0xFF, 0xE8, 0x40),
    bulb_off: rgb(0x38, 0x48, 0x90),
    drive_fill: rgb(0x20, 0x50, 0xC8),
    drive_border: rgb(0x10, 0x80, 0xA8),
    status_text: Color32::WHITE,
    title: rgb(0x30, 0xD0, 0xF0),
    title_hi: rgb(0x30, 0xD0, 0xF0),
    title_shadow: Color32::BLACK,
    sub: rgb(0xFF, 0xB0, 0x20),
};

/// Modelled on X-Copy III / Professional: black screen, silver frames, green values.
pub static CLASSIC: Theme = Theme {
    name: "CLASSIC X-COPY",
    bg: Color32::BLACK,
    panel: Color32::BLACK,
    frame: rgb(0xA8, 0xA8, 0xA8),
    frame_dim: rgb(0x60, 0x60, 0x60),
    label: rgb(0xC8, 0xC8, 0xC8),
    heading: rgb(0xC8, 0xC8, 0xC8),
    value: rgb(0x38, 0xC0, 0x38),
    value_box: false,
    btn_fill: rgb(0x28, 0x28, 0x28),
    btn_text: rgb(0xF0, 0xE0, 0x20),
    btn_text_alt: rgb(0x38, 0xD0, 0x38),
    danger_fill: rgb(0x28, 0x28, 0x28),
    danger_text: rgb(0xE0, 0x30, 0x20),
    sep: rgb(0x20, 0x30, 0xD8),
    arrow: rgb(0x30, 0x90, 0xFF),
    grid_line: Some(rgb(0xA8, 0xA8, 0xA8)),
    cell_off: Color32::BLACK,
    cell_on: rgb(0x38, 0xD0, 0x38),
    cell_on_alt: rgb(0xF0, 0xE0, 0x20),
    cell_glyph: true,
    bulb_on: rgb(0xFF, 0xC0, 0x20),
    bulb_off: rgb(0x98, 0x98, 0x98),
    drive_fill: rgb(0x20, 0x60, 0xE0),
    drive_border: rgb(0x10, 0x30, 0x90),
    status_text: rgb(0xF0, 0x80, 0x10),
    title: rgb(0xB8, 0xB8, 0xB8),
    title_hi: Color32::WHITE,
    title_shadow: rgb(0x48, 0x48, 0x48),
    sub: rgb(0xF0, 0x80, 0x10),
};

pub static THEMES: [&Theme; 2] = [&MODERN, &CLASSIC];

pub fn apply(ctx: &egui::Context, t: &Theme) {
    let mut v = egui::Visuals::dark();
    v.panel_fill = t.bg;
    v.window_fill = t.panel;
    v.extreme_bg_color = Color32::BLACK;
    v.override_text_color = Some(t.label);
    v.selection.bg_fill = t.value;
    v.selection.stroke = Stroke::new(1.0, Color32::BLACK);
    v.window_stroke = Stroke::new(2.0, t.frame);
    v.window_corner_radius = egui::CornerRadius::ZERO;

    for w in [
        &mut v.widgets.noninteractive,
        &mut v.widgets.inactive,
        &mut v.widgets.hovered,
        &mut v.widgets.active,
        &mut v.widgets.open,
    ] {
        w.corner_radius = egui::CornerRadius::ZERO;
        w.bg_stroke = Stroke::new(1.5, t.frame_dim);
        w.expansion = 0.0;
    }
    v.widgets.inactive.weak_bg_fill = t.btn_fill;
    v.widgets.inactive.bg_fill = t.btn_fill;
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, t.btn_text);
    v.widgets.hovered.weak_bg_fill = t.btn_fill.lerp_to_gamma(Color32::WHITE, 0.25);
    v.widgets.hovered.bg_fill = t.btn_fill.lerp_to_gamma(Color32::WHITE, 0.25);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    v.widgets.active.weak_bg_fill = t.btn_text;
    v.widgets.active.bg_fill = t.btn_text;
    v.widgets.active.fg_stroke = Stroke::new(1.0, Color32::BLACK);
    ctx.set_visuals(v);

    ctx.all_styles_mut(|s| {
        let mono = |size| FontId::new(size, FontFamily::Monospace);
        s.text_styles = [
            (TextStyle::Heading, mono(16.0)),
            (TextStyle::Body, mono(13.0)),
            (TextStyle::Button, mono(13.0)),
            (TextStyle::Monospace, mono(12.0)),
            (TextStyle::Small, mono(11.0)),
        ]
        .into();
        s.spacing.item_spacing = egui::vec2(8.0, 5.0);
        s.spacing.button_padding = egui::vec2(8.0, 3.0);
    });
}

/// The chosen theme survives restarts in `%APPDATA%\RoboXCopy\theme.txt`.
fn theme_file() -> Option<std::path::PathBuf> {
    std::env::var_os("APPDATA").map(|d| std::path::Path::new(&d).join("RoboXCopy").join("theme.txt"))
}

pub fn load_index() -> usize {
    theme_file()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| THEMES.iter().position(|t| t.name == s.trim()))
        .unwrap_or(0)
}

pub fn save_index(i: usize) {
    if let Some(p) = theme_file() {
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(p, THEMES[i].name);
    }
}
