// Serde helpers for ratatui::Color

use ratatui::style::Color;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Serialize a Color as a hex string or named color.
pub fn serialize<S>(color: &Color, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let hex = match color {
        Color::Rgb(r, g, b) => format!("#{:02X}{:02X}{:02X}", r, g, b),
        Color::Indexed(idx) => format!("idx:{}", idx),
        Color::Reset => "reset".to_string(),
        Color::Black => "black".to_string(),
        Color::Red => "red".to_string(),
        Color::Green => "green".to_string(),
        Color::Yellow => "yellow".to_string(),
        Color::Blue => "blue".to_string(),
        Color::Magenta => "magenta".to_string(),
        Color::Cyan => "cyan".to_string(),
        Color::Gray => "gray".to_string(),
        Color::DarkGray => "darkgray".to_string(),
        Color::LightRed => "lightred".to_string(),
        Color::LightGreen => "lightgreen".to_string(),
        Color::LightYellow => "lightyellow".to_string(),
        Color::LightBlue => "lightblue".to_string(),
        Color::LightMagenta => "lightmagenta".to_string(),
        Color::LightCyan => "lightcyan".to_string(),
        _ => "white".to_string(),
    };
    hex.serialize(serializer)
}

/// Deserialize a Color from a hex string or named color.
pub fn deserialize<'de, D>(deserializer: D) -> Result<Color, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if let Some(hex) = s.strip_prefix('#') {
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).map_err(serde::de::Error::custom)?;
            let g = u8::from_str_radix(&hex[2..4], 16).map_err(serde::de::Error::custom)?;
            let b = u8::from_str_radix(&hex[4..6], 16).map_err(serde::de::Error::custom)?;
            return Ok(Color::Rgb(r, g, b));
        }
        return Err(serde::de::Error::custom("Invalid hex color length"));
    }
    if let Some(idx) = s.strip_prefix("idx:") {
        let i: u8 = idx.parse().map_err(serde::de::Error::custom)?;
        return Ok(Color::Indexed(i));
    }
    Ok(match s.as_str() {
        "reset" => Color::Reset,
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "gray" => Color::Gray,
        "darkgray" | "dark_gray" => Color::DarkGray,
        "lightred" | "light_red" => Color::LightRed,
        "lightgreen" | "light_green" => Color::LightGreen,
        "lightyellow" | "light_yellow" => Color::LightYellow,
        "lightblue" | "light_blue" => Color::LightBlue,
        "lightmagenta" | "light_magenta" => Color::LightMagenta,
        "lightcyan" | "light_cyan" => Color::LightCyan,
        _ => Color::White,
    })
}

/// Wrapper type for contexts needing a concrete serde-compatible Color newtype.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SerdeColor(pub Color);

impl From<SerdeColor> for Color {
    fn from(sc: SerdeColor) -> Self {
        sc.0
    }
}

impl From<Color> for SerdeColor {
    fn from(c: Color) -> Self {
        SerdeColor(c)
    }
}

impl Serialize for SerdeColor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize(&self.0, serializer)
    }
}

impl<'de> Deserialize<'de> for SerdeColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize(deserializer).map(SerdeColor)
    }
}
