//! The window and taskbar icon. It is drawn at start-up from `assets/icon.txt`, the same file the exe
//! icon and the website images are generated from, so there is a single drawing to maintain.

use eframe::egui::IconData;
use std::collections::HashMap;

const SOURCE: &str = include_str!("../assets/icon.txt");
/// 32 px drawing x 4 = 128 px, enough for the largest taskbar and Alt+Tab sizes.
const SCALE: usize = 4;

pub fn window_icon() -> IconData {
    let mut palette: HashMap<char, [u8; 3]> = HashMap::new();
    let mut rows: Vec<Vec<char>> = Vec::new();
    let mut in_grid = false;
    for line in SOURCE.lines().map(str::trim_end) {
        if line.starts_with('#') {
            continue;
        }
        if line == "---" {
            in_grid = true;
        } else if in_grid {
            rows.push(line.chars().collect());
        } else if let Some((key, hex)) = line.split_once('=') {
            if let (Some(key), Ok(rgb)) = (key.chars().next(), u32::from_str_radix(hex.trim(), 16)) {
                palette.insert(key, [(rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8]);
            }
        }
    }
    let size = rows.len() * SCALE;
    let mut rgba = vec![0u8; size * size * 4];
    for y in 0..size {
        for x in 0..size {
            if let Some(color) = rows[y / SCALE].get(x / SCALE).and_then(|c| palette.get(c)) {
                let i = (y * size + x) * 4;
                rgba[i..i + 3].copy_from_slice(color);
                rgba[i + 3] = 255;
            }
        }
    }
    IconData { rgba, width: size as u32, height: size as u32 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_is_drawn_from_the_source_file() {
        let icon = window_icon();
        assert_eq!((icon.width, icon.height), (128, 128));
        assert_eq!(icon.rgba.len(), 128 * 128 * 4);
        let alpha = |x: usize, y: usize| icon.rgba[(y * 128 + x) * 4 + 3];
        assert_eq!(alpha(0, 0), 0, "the corner around the floppy is transparent");
        assert_eq!(alpha(64, 64), 255, "the body is opaque");
        let opaque = icon.rgba.chunks_exact(4).filter(|p| p[3] == 255).count();
        assert!(opaque > 128 * 128 / 2, "most of the picture is the floppy");
    }
}
