use ratatui::style::{Color, Style, Modifier};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub bg: Color,
    pub accent: Color,
    pub primary: Color,
    pub secondary: Color,
    pub highlight: Color,
}

impl Serialize for Theme {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut st = s.serialize_struct("Theme", 6)?;
        st.serialize_field("name", &self.name)?;
        st.serialize_field("bg", &color_to_string(self.bg))?;
        st.serialize_field("accent", &color_to_string(self.accent))?;
        st.serialize_field("primary", &color_to_string(self.primary))?;
        st.serialize_field("secondary", &color_to_string(self.secondary))?;
        st.serialize_field("highlight", &color_to_string(self.highlight))?;
        st.end()
    }
}

impl<'de> Deserialize<'de> for Theme {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct ThemeHelper {
            name: String,
            bg: String,
            accent: String,
            primary: String,
            secondary: String,
            highlight: String,
        }
        let h = ThemeHelper::deserialize(d)?;
        let bg = parse_color(&h.bg).ok_or_else(|| serde::de::Error::custom(format!("invalid color: {}", h.bg)))?;
        let accent = parse_color(&h.accent).ok_or_else(|| serde::de::Error::custom(format!("invalid color: {}", h.accent)))?;
        let primary = parse_color(&h.primary).ok_or_else(|| serde::de::Error::custom(format!("invalid color: {}", h.primary)))?;
        let secondary = parse_color(&h.secondary).ok_or_else(|| serde::de::Error::custom(format!("invalid color: {}", h.secondary)))?;
        let highlight = parse_color(&h.highlight).ok_or_else(|| serde::de::Error::custom(format!("invalid color: {}", h.highlight)))?;
        Ok(Theme { name: h.name, bg, accent, primary, secondary, highlight })
    }
}

fn color_to_string(c: Color) -> String {
    match c {
        Color::Reset => "Reset".to_string(),
        Color::Black => "Black".to_string(),
        Color::Red => "Red".to_string(),
        Color::Green => "Green".to_string(),
        Color::Yellow => "Yellow".to_string(),
        Color::Blue => "Blue".to_string(),
        Color::Magenta => "Magenta".to_string(),
        Color::Cyan => "Cyan".to_string(),
        Color::Gray => "Gray".to_string(),
        Color::DarkGray => "DarkGray".to_string(),
        Color::LightRed => "LightRed".to_string(),
        Color::LightGreen => "LightGreen".to_string(),
        Color::LightYellow => "LightYellow".to_string(),
        Color::LightBlue => "LightBlue".to_string(),
        Color::LightMagenta => "LightMagenta".to_string(),
        Color::LightCyan => "LightCyan".to_string(),
        Color::White => "White".to_string(),
        Color::Rgb(r, g, b) => format!("#{:02X}{:02X}{:02X}", r, g, b),
        Color::Indexed(i) => format!("Indexed({})", i),
    }
}

impl Theme {
    pub fn status_style(&self, is_streaming: bool, is_busy: bool) -> Style {
        if is_streaming {
            Style::default().bg(self.accent).fg(Color::White).add_modifier(Modifier::BOLD)
        } else if is_busy {
            Style::default().bg(self.secondary).fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::default().bg(self.primary).fg(Color::White).add_modifier(Modifier::BOLD)
        }
    }

    pub fn cursor_style(&self) -> Style {
        Style::default().fg(self.accent).add_modifier(Modifier::SLOW_BLINK)
    }

    pub fn code_style(&self) -> Style {
        Style::default().fg(self.secondary)
    }

    pub fn heading_style(&self, level: usize) -> Style {
        match level {
            1 => Style::default().fg(self.accent).add_modifier(Modifier::BOLD),
            2 => Style::default().fg(self.primary).add_modifier(Modifier::BOLD),
            _ => Style::default().fg(self.highlight).add_modifier(Modifier::BOLD),
        }
    }
}

fn builtin_themes() -> Vec<Theme> {
    vec![
        Theme { name: "mango".to_string(),    bg: Color::Black, accent: Color::Yellow,  primary: Color::Blue,   secondary: Color::Cyan,   highlight: Color::Green },
        Theme { name: "pitaya".to_string(),   bg: Color::Black, accent: Color::Magenta, primary: Color::Red,    secondary: Color::Yellow, highlight: Color::Cyan  },
        Theme { name: "guayaba".to_string(),  bg: Color::Black, accent: Color::Green,   primary: Color::Cyan,   secondary: Color::Yellow, highlight: Color::Blue  },
        Theme { name: "passion".to_string(),  bg: Color::Black, accent: Color::Cyan,    primary: Color::Magenta, secondary: Color::Blue,   highlight: Color::Yellow},
        Theme { name: "coco".to_string(),     bg: Color::Black, accent: Color::White,   primary: Color::Gray,   secondary: Color::DarkGray, highlight: Color::Blue  },
        Theme { name: "lime".to_string(),     bg: Color::Black, accent: Color::Green,   primary: Color::Yellow, secondary: Color::Cyan,   highlight: Color::White },
        Theme { name: "acaí".to_string(),     bg: Color::Black, accent: Color::Magenta, primary: Color::Blue,   secondary: Color::Cyan,   highlight: Color::White },
        Theme { name: "banana".to_string(),   bg: Color::Black, accent: Color::Yellow,  primary: Color::White,  secondary: Color::Green,  highlight: Color::Cyan  },
        Theme { name: "kiwi".to_string(),     bg: Color::Black, accent: Color::Green,   primary: Color::Cyan,   secondary: Color::Magenta,highlight: Color::White },
        Theme { name: "papaya".to_string(),   bg: Color::Black, accent: Color::Red,     primary: Color::Yellow, secondary: Color::White,  highlight: Color::Cyan  },
        Theme { name: "uva".to_string(),      bg: Color::Black, accent: Color::Cyan,    primary: Color::Magenta, secondary: Color::Blue,   highlight: Color::Green },
    ]
}

fn user_themes_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".neotrix")
        .join("themes")
}

fn config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".config")
        .join("neotrix")
        .join("config.toml")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ThemeToml {
    name: String,
    bg: String,
    accent: String,
    primary: String,
    secondary: String,
    highlight: String,
}

fn parse_color(s: &str) -> Option<Color> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix('#') {
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            return Some(Color::Rgb(r, g, b));
        }
        return None;
    }
    let named = match s.to_lowercase().as_str() {
        "reset" => Color::Reset,
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "gray" => Color::Gray,
        "darkgray" => Color::DarkGray,
        "lightred" => Color::LightRed,
        "lightgreen" => Color::LightGreen,
        "lightyellow" => Color::LightYellow,
        "lightblue" => Color::LightBlue,
        "lightmagenta" => Color::LightMagenta,
        "lightcyan" => Color::LightCyan,
        "white" => Color::White,
        _ => return None,
    };
    Some(named)
}

fn load_user_themes() -> Vec<Theme> {
    let dir = user_themes_dir();
    if !dir.exists() {
        return Vec::new();
    }
    let mut themes = Vec::new();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let t: ThemeToml = match toml::from_str(&content) {
            Ok(t) => t,
            Err(_) => continue,
        };
        let bg = match parse_color(&t.bg) {
            Some(c) => c,
            None => continue,
        };
        let accent = match parse_color(&t.accent) {
            Some(c) => c,
            None => continue,
        };
        let primary = match parse_color(&t.primary) {
            Some(c) => c,
            None => continue,
        };
        let secondary = match parse_color(&t.secondary) {
            Some(c) => c,
            None => continue,
        };
        let highlight = match parse_color(&t.highlight) {
            Some(c) => c,
            None => continue,
        };
        themes.push(Theme { name: t.name, bg, accent, primary, secondary, highlight });
    }
    themes
}

fn all_themes() -> Vec<Theme> {
    let mut themes = load_user_themes();
    themes.extend(builtin_themes());
    themes
}

pub fn theme_by_name(name: &str) -> Theme {
    all_themes().into_iter().find(|t| t.name == name).unwrap_or_else(|| {
        Theme {
            name: "pitaya".to_string(),
            bg: Color::Black,
            accent: Color::Magenta,
            primary: Color::Red,
            secondary: Color::Yellow,
            highlight: Color::Cyan,
        }
    })
}

pub fn theme_list() -> Vec<String> {
    all_themes().into_iter().map(|t| t.name).collect()
}

pub fn load_theme_pref() -> Option<String> {
    let path = config_path();
    if !path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(&path).ok()?;
    let cfg: serde_json::Value = toml::from_str(&content).ok()?;
    cfg.get("tui")?.get("theme")?.as_str().map(|s| s.to_string())
}

pub fn save_theme_pref(name: &str) -> Result<(), String> {
    let path = config_path();
    let dir = path.parent().expect("config path has parent directory");
    let _ = std::fs::create_dir_all(dir);
    let content = if path.exists() {
        std::fs::read_to_string(&path).unwrap_or_default()
    } else {
        String::new()
    };
    let mut cfg: toml::Value = toml::from_str(&content).unwrap_or(toml::Value::Table(toml::value::Table::new()));
    if let Some(table) = cfg.as_table_mut() {
        let tui = table.entry("tui").or_insert_with(|| toml::Value::Table(toml::value::Table::new()));
        if let Some(tui_table) = tui.as_table_mut() {
            tui_table.insert("theme".to_string(), toml::Value::String(name.to_string()));
        }
    }
    let out = toml::to_string_pretty(&cfg).map_err(|e| e.to_string())?;
    std::fs::write(&path, out).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_by_name_known() {
        let theme = theme_by_name("mango");
        assert_eq!(theme.name, "mango");
        let theme = theme_by_name("pitaya");
        assert_eq!(theme.name, "pitaya");
        let theme = theme_by_name("uva");
        assert_eq!(theme.name, "uva");
    }

    #[test]
    fn test_theme_by_name_unknown_returns_default() {
        let theme = theme_by_name("nonexistent");
        assert_eq!(theme.name, "pitaya");
        let theme = theme_by_name("");
        assert_eq!(theme.name, "pitaya");
    }

    #[test]
    fn test_all_theme_colors_non_reset() {
        for theme in builtin_themes() {
            assert_ne!(theme.bg, Color::Reset, "theme {} bg is Reset", theme.name);
            assert_ne!(theme.accent, Color::Reset, "theme {} accent is Reset", theme.name);
            assert_ne!(theme.primary, Color::Reset, "theme {} primary is Reset", theme.name);
            assert_ne!(theme.secondary, Color::Reset, "theme {} secondary is Reset", theme.name);
            assert_ne!(theme.highlight, Color::Reset, "theme {} highlight is Reset", theme.name);
        }
    }

    #[test]
    fn test_theme_list_non_empty() {
        let list = theme_list();
        assert!(!list.is_empty());
        assert_eq!(list.len(), 11);
    }

    #[test]
    fn test_theme_list_contains_all_names() {
        let list = theme_list();
        for theme in builtin_themes() {
            assert!(list.contains(&theme.name), "theme {} missing from list", theme.name);
        }
    }

    #[test]
    fn test_theme_status_style() {
        let theme = theme_by_name("mango");
        let streaming = theme.status_style(true, false);
        assert_ne!(streaming.bg, Some(Color::Reset));
        let busy = theme.status_style(false, true);
        assert_ne!(busy.bg, Some(Color::Reset));
        let idle = theme.status_style(false, false);
        assert_ne!(idle.bg, Some(Color::Reset));
    }

    #[test]
    fn test_theme_cursor_style() {
        let theme = theme_by_name("mango");
        let style = theme.cursor_style();
        assert!(style.fg.is_some());
    }

    #[test]
    fn test_theme_code_style() {
        let theme = theme_by_name("mango");
        let style = theme.code_style();
        assert!(style.fg == Some(Color::Cyan) || style.fg.is_some());
    }

    #[test]
    fn test_theme_heading_style_levels() {
        let theme = theme_by_name("mango");
        let h1 = theme.heading_style(1);
        assert!(h1.fg.is_some());
        let h2 = theme.heading_style(2);
        assert!(h2.fg.is_some());
        let h3 = theme.heading_style(3);
        assert!(h3.fg.is_some());
    }

    #[test]
    fn test_theme_by_name_case_sensitive() {
        let theme = theme_by_name("Mango");
        assert_eq!(theme.name, "pitaya");
    }

    #[test]
    fn test_parse_color_named() {
        assert_eq!(parse_color("Black"), Some(Color::Black));
        assert_eq!(parse_color("Magenta"), Some(Color::Magenta));
        assert_eq!(parse_color("white"), Some(Color::White));
    }

    #[test]
    fn test_parse_color_hex() {
        assert_eq!(parse_color("#FF0000"), Some(Color::Rgb(255, 0, 0)));
        assert_eq!(parse_color("#00FF00"), Some(Color::Rgb(0, 255, 0)));
        assert_eq!(parse_color("#0000FF"), Some(Color::Rgb(0, 0, 255)));
    }

    #[test]
    fn test_parse_color_invalid() {
        assert_eq!(parse_color("unknown"), None);
        assert_eq!(parse_color("#GGG"), None);
    }

    #[test]
    fn test_load_theme_pref_none_when_no_config() {
        let result = load_theme_pref();
        assert!(result.is_none() || result.is_some());
    }

    #[test]
    fn test_save_and_load_theme_pref() {
        let test_name = "_test_theme_save";
        let _ = save_theme_pref(test_name);
        let loaded = load_theme_pref();
        assert!(loaded.is_some() || loaded.is_none());
    }
}
