#![allow(dead_code, unused_imports)]

use std::collections::{HashSet, VecDeque};

use crate::cli::tui::app::types::{Session, ChatMessage, GoalDisplay};
use crate::cli::sandbox::SandboxMode;
use super::super::vim_mode::VimModeManager;

#[derive(Debug, Clone, Default)]
pub struct CommandHistory {
    pub entries: Vec<String>,
    pub position: Option<usize>,
    pub search_active: bool,
    pub search_query: String,
    pub search_results: Vec<usize>,
    pub search_selection: usize,
}

impl CommandHistory {
    pub fn new() -> Self {
        Self { entries: Vec::new(), position: None, search_active: false, search_query: String::new(), search_results: Vec::new(), search_selection: 0 }
    }

    pub fn push(&mut self, entry: String) {
        self.entries.push(entry);
        self.position = None;
    }
}

#[derive(Clone)]
pub struct TuiApp {
    pub running: bool,
    pub sessions: Vec<Session>,
    pub active_session: usize,
    pub input: String,
    pub command_history: CommandHistory,
    pub scroll_offset: usize,
    pub multi_line: bool,
    pub agent_busy: bool,
    pub streaming: bool,
    pub token_count: usize,
    pub status_text: String,
    pub diff_viewer: Option<crate::cli::tui::diff_viewer::DiffViewer>,
    pub thinking_expanded: HashSet<String>,
    pub streaming_role: String,
    pub streaming_text: String,
    pub goal_display: GoalDisplay,
    pub sandbox_mode: SandboxMode,
    pub vim_mode: VimModeManager,
    pub workspace_name: String,
    pub workspace_count: usize,
    pub tokens_per_sec: f64,
}

impl TuiApp {
    pub fn new(ephemeral: bool) -> Self {
        Self {
            running: true,
            sessions: vec![Session {
                id: "s-1".into(),
                name: "Default Session".into(),
                messages: VecDeque::new(),
            }],
            active_session: 0,
            input: String::new(),
            command_history: CommandHistory::new(),
            scroll_offset: 0,
            multi_line: false,
            agent_busy: false,
            streaming: false,
            token_count: 0,
            status_text: if ephemeral { "Ready".into() } else { "Ready | Provider: not configured".into() },
            diff_viewer: None,
            thinking_expanded: HashSet::new(),
            streaming_role: String::new(),
            streaming_text: String::new(),
            goal_display: GoalDisplay::idle(),
            sandbox_mode: SandboxMode::Disabled,
            vim_mode: VimModeManager::new(),
            workspace_name: String::new(),
            workspace_count: 0,
            tokens_per_sec: 0.0,
        }
    }

    pub async fn run(&mut self, _agent: ()) {}

    pub fn push_message(&mut self, role: &str, content: String) {
        if let Some(session) = self.sessions.get_mut(self.active_session) {
            session.messages.push_back(ChatMessage::new(role, content));
        }
    }

    pub fn active_session(&self) -> &Session {
        &self.sessions[self.active_session]
    }

    #[allow(unused_variables)]
    pub fn handle_key(&mut self, key: u8, modifiers: u8) -> bool {
        false
    }

    pub fn trim(&self) -> &str {
        self.input.trim()
    }
}
