use super::*;
#[allow(unused_imports)]
use std::collections::HashSet;

/// UI 状态
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UIState {
    pub show_sessions: bool,
    pub show_tools: bool,
    pub show_thinking: bool,
    pub show_tool_calls: bool,
    pub active_panel: Panel,
    pub focus: Focus,
    pub scroll_offset: usize,
    pub theme_name: String,
    pub show_line_numbers: bool,
    pub word_wrap: bool,
    pub auto_scroll: bool,
    pub show_git_status: bool,
    pub show_token_count: bool,
    pub status_message: Option<String>,
    pub error_message: Option<String>,
    pub spinner_frame: usize,
    pub show_sessions_panel: bool,
    pub show_thinking_blocks: bool,
    pub vim_mode: bool,
    vim_mode_state: VimModeState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Panel {
    #[default]
    Chat,
    Sessions,
    Tools,
    Settings,
    Diff,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Focus {
    #[default]
    Chat,
    Input,
    Sessions,
    Tools,
    CommandPalette,
    Search,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VimModeState {
    pub enabled: bool,
    pub mode: VimMode,
    pub pending_operator: Option<char>,
    pub pending_count: Option<usize>,
    register: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum VimMode {
    #[default]
    Normal,
    Insert,
    Visual,
    VisualLine,
    Command,
}
