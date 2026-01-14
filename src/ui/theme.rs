use ratatui::style::Color;
use serde::Deserialize;
use std::fs;
use std::sync::LazyLock;

use crate::storage;

/// Parse color from string: "R,G,B" or named colors like "cyan", "yellow", etc.
fn parse_color(s: &str) -> Option<Color> {
    let s = s.trim().to_lowercase();

    // Named colors
    match s.as_str() {
        "black" => return Some(Color::Black),
        "red" => return Some(Color::Red),
        "green" => return Some(Color::Green),
        "yellow" => return Some(Color::Yellow),
        "blue" => return Some(Color::Blue),
        "magenta" => return Some(Color::Magenta),
        "cyan" => return Some(Color::Cyan),
        "white" => return Some(Color::White),
        "gray" | "grey" => return Some(Color::Gray),
        "darkgray" | "darkgrey" => return Some(Color::DarkGray),
        _ => {}
    }

    // RGB format: "R,G,B"
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() == 3 {
        let r = parts[0].trim().parse::<u8>().ok()?;
        let g = parts[1].trim().parse::<u8>().ok()?;
        let b = parts[2].trim().parse::<u8>().ok()?;
        return Some(Color::Rgb(r, g, b));
    }

    None
}

#[derive(Deserialize, Default)]
struct ConfigFile {
    #[serde(default)]
    colors: ColorsConfig,
}

#[derive(Deserialize, Default)]
struct ColorsConfig {
    bg_dark: Option<String>,
    fg_primary: Option<String>,
    fg_muted: Option<String>,
    accent: Option<String>,
    accent_yellow: Option<String>,
    accent_green: Option<String>,
    cursor_bg: Option<String>,
    cursor_fg: Option<String>,
    modal_bg: Option<String>,
    modal_fg: Option<String>,
    modal_fg_muted: Option<String>,
}

pub struct Theme {
    pub bg_dark: Color,
    pub fg_primary: Color,
    pub fg_muted: Color,
    pub accent: Color,
    pub accent_yellow: Color,
    pub accent_green: Color,
    pub cursor_bg: Color,
    pub cursor_fg: Color,
    pub modal_bg: Color,
    pub modal_fg: Color,
    pub modal_fg_muted: Color,
}

impl Theme {
    fn load() -> Self {
        let config = load_config();

        Self {
            bg_dark: config.colors.bg_dark
                .and_then(|s| parse_color(&s))
                .unwrap_or(Color::Rgb(25, 25, 30)),
            fg_primary: config.colors.fg_primary
                .and_then(|s| parse_color(&s))
                .unwrap_or(Color::Rgb(160, 160, 160)),
            fg_muted: config.colors.fg_muted
                .and_then(|s| parse_color(&s))
                .unwrap_or(Color::Rgb(100, 100, 100)),
            accent: config.colors.accent
                .and_then(|s| parse_color(&s))
                .unwrap_or(Color::Cyan),
            accent_yellow: config.colors.accent_yellow
                .and_then(|s| parse_color(&s))
                .unwrap_or(Color::Yellow),
            accent_green: config.colors.accent_green
                .and_then(|s| parse_color(&s))
                .unwrap_or(Color::Green),
            cursor_bg: config.colors.cursor_bg
                .and_then(|s| parse_color(&s))
                .unwrap_or(Color::Cyan),
            cursor_fg: config.colors.cursor_fg
                .and_then(|s| parse_color(&s))
                .unwrap_or(Color::Black),
            modal_bg: config.colors.modal_bg
                .and_then(|s| parse_color(&s))
                .unwrap_or(Color::Rgb(240, 240, 240)),
            modal_fg: config.colors.modal_fg
                .and_then(|s| parse_color(&s))
                .unwrap_or(Color::Rgb(40, 40, 40)),
            modal_fg_muted: config.colors.modal_fg_muted
                .and_then(|s| parse_color(&s))
                .unwrap_or(Color::Rgb(120, 120, 120)),
        }
    }
}

fn load_config() -> ConfigFile {
    storage::get_config_path()
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|content| toml::from_str(&content).ok())
        .unwrap_or_default()
}

pub static THEME: LazyLock<Theme> = LazyLock::new(Theme::load);

// Convenience accessors
pub fn bg_dark() -> Color { THEME.bg_dark }
pub fn fg_primary() -> Color { THEME.fg_primary }
pub fn fg_muted() -> Color { THEME.fg_muted }
pub fn accent() -> Color { THEME.accent }
pub fn accent_yellow() -> Color { THEME.accent_yellow }
pub fn accent_green() -> Color { THEME.accent_green }
pub fn cursor_bg() -> Color { THEME.cursor_bg }
pub fn cursor_fg() -> Color { THEME.cursor_fg }
pub fn modal_bg() -> Color { THEME.modal_bg }
pub fn modal_fg() -> Color { THEME.modal_fg }
pub fn modal_fg_muted() -> Color { THEME.modal_fg_muted }
