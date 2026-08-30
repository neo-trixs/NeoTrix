use super::*;
use std::collections::HashMap;

/// 运行时配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuntimeConfig {
    pub theme_name: String,
    pub theme_preset: ThemePreset,
    pub keymap: KeyMap,
    pub max_history: usize,
    pub max_message_length: usize,
    pub auto_scroll: bool,
    pub show_line_numbers: bool,
    pub word_wrap: bool,
    pub syntax_highlighting: bool,
    pub show_timestamps: bool,
    pub show_model_name: bool,
    pub stream_chunk_delay_ms: u64,
    pub max_message_history: usize,
    pub session_persistence: bool,
    pub auto_save_interval_secs: u64,
    pub vim_mode: bool,
    pub mouse_support: bool,
    pub bell_on_completion: bool,
    pub compact_mode: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ThemePreset {
    #[default]
    Dark,
    Light,
    Gruvbox,
    Nord,
    Dracula,
    Solarized,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyMap {
    pub bindings: HashMap<String, String>,
}
