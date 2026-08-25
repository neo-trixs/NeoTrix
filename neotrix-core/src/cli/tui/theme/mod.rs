//! Theme module - Theme system exports (local stubs)

use ratatui::style::Color;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Custom serialization for ratatui::style::Color
fn serialize_color<S>(color: &Color, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // Serialize as hex string based on Color variant
    let hex = match color {
        Color::Rgb(r, g, b) => format!("{:02X}{:02X}{:02X}", r, g, b),
        Color::Indexed(idx) => format!("{:02X}", idx),
        Color::Reset => "RESET".to_string(),
        Color::Black => "000000".to_string(),
        Color::Red => "FF0000".to_string(),
        Color::Green => "00FF00".to_string(),
        Color::Yellow => "FFFF00".to_string(),
        Color::Blue => "0000FF".to_string(),
        Color::Magenta => "FF00FF".to_string(),
        Color::Cyan => "00FFFF".to_string(),
        Color::White => "FFFFFF".to_string(),
        Color::Gray => "808080".to_string(),
        Color::DarkGray => "A9A9A9".to_string(),
        Color::LightRed => "FF5555".to_string(),
        Color::LightGreen => "55FF55".to_string(),
        Color::LightYellow => "FFFF55".to_string(),
        Color::LightBlue => "5555FF".to_string(),
        Color::LightMagenta => "FF55FF".to_string(),
        Color::LightCyan => "55FFFF".to_string(),

    };
    serializer.serialize_str(&hex)
}

fn deserialize_color<'de, D>(deserializer: D) -> Result<Color, D::Error>
where
    D: Deserializer<'de>,
{
    let hex = String::deserialize(deserializer)?;
    // Parse hex string - assume RGB for simplicity
    if hex == "RESET" {
        return Ok(Color::Reset);
    }
    if hex.len() == 2 {
        let idx = u8::from_str_radix(&hex, 16).map_err(serde::de::Error::custom)?;
        return Ok(Color::Indexed(idx));
    }
    let r = u8::from_str_radix(&hex[0..2], 16).map_err(serde::de::Error::custom)?;
    let g = u8::from_str_radix(&hex[2..4], 16).map_err(serde::de::Error::custom)?;
    let b = u8::from_str_radix(&hex[4..6], 16).map_err(serde::de::Error::custom)?;
    Ok(Color::Rgb(r, g, b))
}

/// 主题颜色
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    #[serde(serialize_with = "serialize_color", deserialize_with = "deserialize_color")]
    pub primary: Color,
    #[serde(serialize_with = "serialize_color", deserialize_with = "deserialize_color")]
    pub secondary: Color,
    #[serde(serialize_with = "serialize_color", deserialize_with = "deserialize_color")]
    pub accent: Color,
    #[serde(serialize_with = "serialize_color", deserialize_with = "deserialize_color")]
    pub highlight: Color,
    #[serde(serialize_with = "serialize_color", deserialize_with = "deserialize_color")]
    pub background: Color,
    #[serde(serialize_with = "serialize_color", deserialize_with = "deserialize_color")]
    pub surface: Color,
    #[serde(serialize_with = "serialize_color", deserialize_with = "deserialize_color")]
    pub border: Color,
    #[serde(serialize_with = "serialize_color", deserialize_with = "deserialize_color")]
    pub primary_text: Color,
    #[serde(serialize_with = "serialize_color", deserialize_with = "deserialize_color")]
    pub secondary_text: Color,
    #[serde(serialize_with = "serialize_color", deserialize_with = "deserialize_color")]
    pub muted_text: Color,
    #[serde(serialize_with = "serialize_color", deserialize_with = "deserialize_color")]
    pub accent_text: Color,
    #[serde(serialize_with = "serialize_color", deserialize_with = "deserialize_color")]
    pub user_msg_bg: Color,
    #[serde(serialize_with = "serialize_color", deserialize_with = "deserialize_color")]
    pub assistant_msg_bg: Color,
    #[serde(serialize_with = "serialize_color", deserialize_with = "deserialize_color")]
    pub system_msg_bg: Color,
    #[serde(serialize_with = "serialize_color", deserialize_with = "deserialize_color")]
    pub error_bg: Color,
    #[serde(serialize_with = "serialize_color", deserialize_with = "deserialize_color")]
    pub code_bg: Color,
    #[serde(serialize_with = "serialize_color", deserialize_with = "deserialize_color")]
    pub success: Color,
    #[serde(serialize_with = "serialize_color", deserialize_with = "deserialize_color")]
    pub warning: Color,
    #[serde(serialize_with = "serialize_color", deserialize_with = "deserialize_color")]
    pub error: Color,
    #[serde(serialize_with = "serialize_color", deserialize_with = "deserialize_color")]
    pub info: Color,
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            primary: Color::Cyan,
            secondary: Color::Gray,
            accent: Color::Magenta,
            highlight: Color::Yellow,
            background: Color::Rgb(28, 28, 30),
            surface: Color::Rgb(38, 38, 42),
            border: Color::Rgb(58, 58, 60),
            primary_text: Color::White,
            secondary_text: Color::Gray,
            muted_text: Color::DarkGray,
            accent_text: Color::Magenta,
            user_msg_bg: Color::Rgb(30, 50, 80),
            assistant_msg_bg: Color::Rgb(40, 40, 50),
            system_msg_bg: Color::Rgb(50, 40, 30),
            error_bg: Color::Rgb(80, 30, 30),
            code_bg: Color::Rgb(20, 20, 25),
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            info: Color::Blue,
        }
    }
}

/// 主题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub colors: ThemeColors,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: "dark".to_string(),
            colors: ThemeColors::default(),
        }
    }
}

/// 主题预设
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemePreset {
    Dark,
    Light,
    Gruvbox,
    Nord,
    Dracula,
    Solarized,
}

impl Default for ThemePreset {
    fn default() -> Self {
        ThemePreset::Dark
    }
}

/// 主题管理器
pub struct ThemeManager {
    pub current: Theme,
}

impl ThemeManager {
    pub fn new() -> Self {
        Self {
            current: Theme::default(),
        }
    }

    pub fn set_preset(&mut self, _preset: ThemePreset) {
        // TODO
    }

    pub fn set_custom(&mut self, theme: Theme) {
        self.current = theme;
    }

    pub fn current(&self) -> &Theme {
        &self.current
    }

    pub fn colors(&self) -> &ThemeColors {
        &self.current.colors
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}
