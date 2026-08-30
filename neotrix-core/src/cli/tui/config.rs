//! TUI Config - 统一配置管理

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use dirs;

/// 完整配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub theme: ThemeConfig,
    pub keymap: KeyMapConfig,
    pub editor: EditorConfig,
    pub terminal: TerminalConfig,
    pub network: NetworkConfig,
    pub privacy: PrivacyConfig,
    pub experimental: ExperimentalConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: ThemeConfig {
                preset: "dark".to_string(),
                custom_colors: None,
                transparent_background: false,
                animations: true,
            },
            keymap: KeyMapConfig {
                bindings: HashMap::new(),
                leader_key: "ctrl".to_string(),
                vim_mode: false,
            },
            editor: EditorConfig {
                tab_size: 2,
                use_spaces: true,
                auto_indent: true,
                show_line_numbers: true,
                word_wrap: true,
                cursor_style: "block".to_string(),
                cursor_blink: true,
            },
            terminal: TerminalConfig {
                shell: "auto".to_string(),
                font_size: 14.0,
                font_family: "JetBrains Mono".to_string(),
                ligatures: true,
                scrollback_lines: 10000,
            },
            network: NetworkConfig {
                proxy: None,
                timeout_secs: 30,
                max_retries: 3,
                verify_ssl: true,
            },
            privacy: PrivacyConfig {
                telemetry: false,
                crash_reporting: true,
                auto_update: true,
                data_directory: None,
            },
            experimental: ExperimentalConfig {
                enable_new_features: false,
                debug_mode: false,
                profiling: false,
            },
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let path = Self::default_config_path()?;
        if path.exists() {
            let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            toml::from_str(&content).map_err(|e| e.to_string())
        } else {
            Ok(Self::default())
        }
    }

    fn default_config_path() -> Result<PathBuf, String> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| "无法获取配置目录".to_string())?
            .join("neotrix");
        std::fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;
        Ok(config_dir.join("config.toml"))
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::default_config_path()?;
        let content = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, content).map_err(|e| e.to_string())
    }

    pub fn get_theme_preset(&self) -> String {
        self.theme.preset.clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThemeConfig {
    pub preset: String,
    pub custom_colors: Option<HashMap<String, String>>,
    pub transparent_background: bool,
    pub animations: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyMapConfig {
    pub bindings: HashMap<String, String>,
    pub leader_key: String,
    pub vim_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EditorConfig {
    pub tab_size: usize,
    pub use_spaces: bool,
    pub auto_indent: bool,
    pub show_line_numbers: bool,
    pub word_wrap: bool,
    pub cursor_style: String,
    pub cursor_blink: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TerminalConfig {
    pub shell: String,
    pub font_size: f32,
    pub font_family: String,
    pub ligatures: bool,
    pub scrollback_lines: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkConfig {
    pub proxy: Option<String>,
    pub timeout_secs: u64,
    pub max_retries: u32,
    pub verify_ssl: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PrivacyConfig {
    pub telemetry: bool,
    pub crash_reporting: bool,
    pub auto_update: bool,
    pub data_directory: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExperimentalConfig {
    pub enable_new_features: bool,
    pub debug_mode: bool,
    pub profiling: bool,
}

/// 兼容性存根 - 供 TUI 模块使用
#[derive(Debug, Clone, Default)]
pub struct TuiConfig {
    pub theme: String,
    pub keymap: String,
    pub max_history: usize,
    pub auto_save: bool,
    pub mouse_support: bool,
    pub bracketed_paste: bool,
    pub data_dir: Option<PathBuf>,
}

pub struct ConfigManager {
    config: TuiConfig,
}

impl ConfigManager {
    pub fn new() -> Self {
        Self { config: TuiConfig::default() }
    }

    pub fn load() -> Self {
        Self::new()
    }

    pub fn get(&self) -> &TuiConfig {
        &self.config
    }

    pub fn set_theme(&mut self, theme: String) {
        self.config.theme = theme;
    }
}

pub fn get_config() -> TuiConfig {
    TuiConfig::default()
}

pub fn init_config() -> ConfigManager {
    ConfigManager::new()
}

pub fn get_theme_preset() -> String {
    "dark".to_string()
}

