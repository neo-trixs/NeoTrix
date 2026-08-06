//! TuiApp — NeoTrix CLI 对话终端（ratatui + crossterm）。
//!
//! 架构：TuiApp 是纯 UI 状态机（可单测），不直接持有 LLM。
//! 事件循环由 `run()` 驱动：渲染层（layout/output/themes）已独立成模块，
//! 本文件只负责「键事件 → 状态迁移 + 动作回调」。
//!
//! 与 AgentLoop 的接线：`run()` 接收 `&mut AgentLoop`，
//! 每轮用户输入调用 `turn_stream()`（流式），chunk 经 `on_token` 更新
//! `streaming_text`，完成时 `push_message` 持久化到会话。

use std::collections::{HashSet, VecDeque};

use crate::cli::tui::app::types::{Session, ChatMessage, GoalDisplay};
use crate::cli::sandbox::SandboxMode;
use super::super::history::CommandHistory;
use super::super::vim_mode::VimModeManager;

/// 从 `run()` 返回给调用方的事件（供 entry 层收尾）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TuiExit {
    /// 用户主动退出（/exit、Ctrl+C、Ctrl+D）。
    Quit,
    /// 终端读取错误或 EOF。
    IoError(String),
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
    /// 会话开始时间戳（用于状态栏时长显示）。
    pub session_started_at: std::time::Instant,
    /// 流式开始时记录，用于计算 tokens/sec。
    stream_started_at: Option<std::time::Instant>,
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
            command_history: CommandHistory::load_or_new(),
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
            session_started_at: std::time::Instant::now(),
            stream_started_at: None,
        }
    }

    pub fn push_message(&mut self, role: &str, content: String) {
        if let Some(session) = self.sessions.get_mut(self.active_session) {
            session.messages.push_back(ChatMessage::new(role, content));
        }
    }

    pub fn active_session(&self) -> &Session {
        &self.sessions[self.active_session]
    }

    pub fn trim(&self) -> &str {
        self.input.trim()
    }

    /// 把流式文本标记为已持久化消息，并重置流式状态。
    pub fn commit_stream(&mut self, role: &str) {
        let text = std::mem::take(&mut self.streaming_text);
        if !text.is_empty() {
            self.push_message(role, text);
        }
        self.streaming = false;
        self.stream_started_at = None;
        self.tokens_per_sec = 0.0;
    }

    /// 记录一个流式 chunk：更新 streaming_text + token 计数 + tokens/sec。
    pub fn feed_stream(&mut self, chunk: &str) {
        let now = std::time::Instant::now();
        if self.stream_started_at.is_none() {
            self.stream_started_at = Some(now);
        }
        self.streaming_text.push_str(chunk);
        self.token_count += 1;
        if let Some(started) = self.stream_started_at {
            let elapsed = now.duration_since(started).as_secs_f64();
            if elapsed > 0.1 {
                self.tokens_per_sec = self.token_count as f64 / elapsed;
            }
        }
    }

    /// 新会话：当前会话保留，追加一个空会话并切换。
    pub fn new_session(&mut self) {
        let id = format!("s-{}", self.sessions.len() + 1);
        self.sessions.push(Session {
            id,
            name: format!("Session {}", self.sessions.len() + 1),
            messages: VecDeque::new(),
        });
        self.active_session = self.sessions.len() - 1;
        self.scroll_offset = 0;
        self.thinking_expanded.clear();
    }

    /// 清空当前会话消息。
    pub fn clear_session(&mut self) {
        if let Some(session) = self.sessions.get_mut(self.active_session) {
            session.messages.clear();
        }
        self.scroll_offset = 0;
        self.streaming = false;
        self.streaming_text.clear();
    }

    /// 核心键处理。返回动作由 `run()` 执行（提交输入/触发 LLM 轮询等）。
    pub fn handle_key(&mut self, key: crossterm::event::KeyCode, modifiers: crossterm::event::KeyModifiers) -> KeyAction {
        use crossterm::event::{KeyCode::*, KeyModifiers};

        // Ctrl+R 历史搜索（任何模式优先）
        if modifiers == KeyModifiers::CONTROL && key == Char('r') {
            self.command_history.start_search();
            return KeyAction::None;
        }
        if self.command_history.search_active {
            return self.handle_search_key(key, modifiers);
        }

        // vim 模式：当启用且非 insert 态时，输入先经 VimModeManager。
        if self.vim_mode.is_enabled() && self.vim_mode.mode != super::super::vim_mode::VimMode::Insert {
            let action = self.vim_mode.handle_key(key, modifiers);
            match action {
                super::super::vim_mode::VimAction::Quit => return KeyAction::Quit,
                super::super::vim_mode::VimAction::InsertChar(c) => self.input.push(c),
                super::super::vim_mode::VimAction::PassThrough
                | super::super::vim_mode::VimAction::None
                | super::super::vim_mode::VimAction::EnterNormalMode
                | super::super::vim_mode::VimAction::EnterInsertMode
                | super::super::vim_mode::VimAction::EnterVisualMode => {}
                _ => {}
            }
            return KeyAction::None;
        }

        match (modifiers, key) {
            (KeyModifiers::CONTROL, Char('c')) | (KeyModifiers::CONTROL, Char('d')) => {
                // 流式中 → 取消生成；空闲 → 退出。
                if self.streaming {
                    self.agent_busy = false;
                    self.streaming = false;
                    self.status_text = "生成已取消".into();
                    KeyAction::CancelGeneration
                } else if self.input.is_empty() {
                    KeyAction::Quit
                } else {
                    self.input.clear();
                    KeyAction::None
                }
            }
            (KeyModifiers::CONTROL, Char('l')) => KeyAction::ClearScreen,
            (KeyModifiers::CONTROL, Char('m')) | (KeyModifiers::CONTROL, Char('j')) => {
                // Ctrl+M / Ctrl+J：多行模式下硬换行，否则提交。
                if self.multi_line {
                    self.input.push('\n');
                    KeyAction::None
                } else {
                    KeyAction::Submit
                }
            }
            (KeyModifiers::NONE, Enter) | (KeyModifiers::NONE, Char('\n')) => {
                if self.multi_line {
                    self.input.push('\n');
                    KeyAction::None
                } else {
                    KeyAction::Submit
                }
            }
            (KeyModifiers::ALT, Char('e')) => { self.multi_line = !self.multi_line; KeyAction::None }
            (_, Char(' ')) => { self.input.push(' '); KeyAction::None }
            (_, Char(c)) => { self.input.push(c); KeyAction::None }
            (_, Backspace) => { self.input.pop(); KeyAction::None }
            (_, Delete) => {
                self.input.pop();
                KeyAction::None
            }
            (KeyModifiers::NONE, Up) => {
                if let Some(prev) = self.command_history.navigate_up() {
                    self.input = prev;
                }
                KeyAction::None
            }
            (KeyModifiers::NONE, Down) => {
                if let Some(next) = self.command_history.navigate_down() {
                    self.input = next;
                }
                KeyAction::None
            }
            (KeyModifiers::NONE, PageUp) => { self.scroll_offset = self.scroll_offset.saturating_add(5); KeyAction::None }
            (KeyModifiers::NONE, PageDown) => { self.scroll_offset = self.scroll_offset.saturating_sub(5); KeyAction::None }
            (_, Tab) => {
                if self.input.starts_with('/') && !self.input.contains(' ') {
                    self.complete_slash();
                }
                KeyAction::None
            }
            (KeyModifiers::NONE, Esc) => {
                if self.multi_line {
                    self.multi_line = false;
                }
                KeyAction::None
            }
            (KeyModifiers::ALT, Char('e')) => { self.multi_line = !self.multi_line; KeyAction::None }
            _ => KeyAction::None,
        }
    }

    fn handle_search_key(&mut self, key: crossterm::event::KeyCode, modifiers: crossterm::event::KeyModifiers) -> KeyAction {
        use crossterm::event::{KeyCode::*, KeyModifiers};
        match key {
            Char('r') if modifiers == KeyModifiers::CONTROL => {
                self.command_history.cycle_search();
                KeyAction::None
            }
            Esc => { self.command_history.cancel_search(); KeyAction::None }
            Enter => {
                if let Some(sel) = self.command_history.select_search() {
                    self.input = sel;
                }
                KeyAction::None
            }
            Backspace => {
                self.command_history.search_query.pop();
                self.command_history.update_search_results();
                KeyAction::None
            }
            Char(c) => {
                self.command_history.search_query.push(c);
                self.command_history.update_search_results();
                KeyAction::None
            }
            _ => KeyAction::None,
        }
    }

    /// 斜杠命令补全：把当前前缀匹配到的第一个命令补全。
    fn complete_slash(&mut self) {
        const SLASH_COMMANDS: &[&str] = &[
            "/clear", "/compact", "/exit", "/help", "/hist", "/model",
            "/new", "/quit", "/q", "/sessions", "/tools", "/undo", "/usage",
        ];
        let prefix = self.input.as_str();
        for cmd in SLASH_COMMANDS {
            if cmd.starts_with(prefix) && cmd.len() > prefix.len() {
                self.input = cmd.to_string();
                return;
            }
        }
    }
}

/// 键处理返回的动作，供事件循环执行。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyAction {
    None,
    Quit,
    Submit,
    ClearScreen,
    CancelGeneration,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers};

    #[test]
    fn test_app_initialization() {
        let app = TuiApp::new(true);
        assert!(app.running);
        assert_eq!(app.sessions.len(), 1);
        assert_eq!(app.active_session, 0);
        assert!(app.input.is_empty());
        assert!(!app.multi_line);
        assert!(!app.agent_busy);
        assert!(!app.streaming);
        assert_eq!(app.token_count, 0);
    }

    #[test]
    fn test_enter_returns_submit() {
        let mut app = TuiApp::new(true);
        let action = app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(action, KeyAction::Submit);
    }

    #[test]
    fn test_typing_accumulates() {
        let mut app = TuiApp::new(true);
        for c in ['h', 'e', 'y'] {
            app.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
        }
        assert_eq!(app.input, "hey");
        app.handle_key(KeyCode::Backspace, KeyModifiers::NONE);
        assert_eq!(app.input, "he");
    }

    #[test]
    fn test_history_navigation() {
        let mut app = TuiApp::new(true);
        app.command_history.entries.push("first".into());
        app.command_history.entries.push("second".into());
        app.command_history.position = None;
        app.handle_key(KeyCode::Up, KeyModifiers::NONE);
        assert_eq!(app.input, "second");
        app.handle_key(KeyCode::Up, KeyModifiers::NONE);
        assert_eq!(app.input, "first");
        app.handle_key(KeyCode::Down, KeyModifiers::NONE);
        assert_eq!(app.input, "second");
    }

    #[test]
    fn test_multiline_enter_appends_newline() {
        let mut app = TuiApp::new(true);
        app.multi_line = true;
        let action = app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(action, KeyAction::None);
        assert_eq!(app.input, "\n");
        app.handle_key(KeyCode::Char('e'), KeyModifiers::ALT);
        assert!(!app.multi_line);
    }

    #[test]
    fn test_ctrl_c_quits_when_idle_empty() {
        let mut app = TuiApp::new(true);
        let action = app.handle_key(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(action, KeyAction::Quit);
    }

    #[test]
    fn test_ctrl_c_cancels_when_streaming() {
        let mut app = TuiApp::new(true);
        app.streaming = true;
        let action = app.handle_key(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(action, KeyAction::CancelGeneration);
    }

    #[test]
    fn test_ctrl_r_starts_search() {
        let mut app = TuiApp::new(true);
        let action = app.handle_key(KeyCode::Char('r'), KeyModifiers::CONTROL);
        assert_eq!(action, KeyAction::None);
        assert!(app.command_history.search_active);
    }

    #[test]
    fn test_slash_tab_completion() {
        let mut app = TuiApp::new(true);
        app.input = "/ex".into();
        app.handle_key(KeyCode::Tab, KeyModifiers::NONE);
        assert!(app.input.starts_with("/exit"));
        app.input = "/h".into();
        app.handle_key(KeyCode::Tab, KeyModifiers::NONE);
        assert!(app.input.starts_with("/help") || app.input.starts_with("/hist"));
    }

    #[test]
    fn test_feed_and_commit_stream() {
        let mut app = TuiApp::new(true);
        app.streaming_role = "assistant".into();
        app.feed_stream("Hello ");
        app.feed_stream("world");
        assert_eq!(app.streaming_text, "Hello world");
        app.commit_stream("assistant");
        assert_eq!(app.sessions[0].messages.len(), 1);
        assert_eq!(app.sessions[0].messages[0].content, "Hello world");
        assert!(!app.streaming);
        assert!(app.streaming_text.is_empty());
    }

    #[test]
    fn test_new_session_appends() {
        let mut app = TuiApp::new(true);
        app.new_session();
        assert_eq!(app.sessions.len(), 2);
        assert_eq!(app.active_session, 1);
        app.push_message("user", "hello".into());
        assert_eq!(app.sessions[1].messages.len(), 1);
        assert_eq!(app.sessions[0].messages.len(), 0);
    }

    #[test]
    fn test_clear_session() {
        let mut app = TuiApp::new(true);
        app.push_message("user", "x".into());
        app.push_message("assistant", "y".into());
        app.clear_session();
        assert!(app.sessions[0].messages.is_empty());
    }

    #[test]
    fn test_vim_mode_toggle() {
        let mut app = TuiApp::new(true);
        assert!(!app.vim_mode.is_enabled());
        app.vim_mode.toggle();
        assert!(app.vim_mode.is_enabled());
    }

    #[test]
    fn test_pageup_scroll() {
        let mut app = TuiApp::new(true);
        app.handle_key(KeyCode::PageUp, KeyModifiers::NONE);
        assert_eq!(app.scroll_offset, 5);
        app.handle_key(KeyCode::PageDown, KeyModifiers::NONE);
        assert_eq!(app.scroll_offset, 0);
    }
}
