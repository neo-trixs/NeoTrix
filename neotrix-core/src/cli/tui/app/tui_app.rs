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

use ratatui::text::Line;

use crate::cli::tui::app::types::{Session, ChatMessage, GoalDisplay};
use crate::cli::sandbox::SandboxMode;
use super::super::history::CommandHistory;
use super::super::vim_mode::VimModeManager;
use super::super::output::StreamingMarkdownRenderer;

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
    /// 输入光标位置（字节索引，始终落在 char boundary 上）。
    pub cursor: usize,
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
    /// 流式输出时的模型名（用于显示）
    pub streaming_model: Option<String>,
    /// 流式 markdown 渲染器（增量渲染，避免闪烁）
    pub streaming_renderer: StreamingMarkdownRenderer,
    /// 流式渲染已生成的 Lines（避免每帧重新渲染）
    pub streaming_rendered_lines: Vec<Line<'static>>,
    pub goal_display: GoalDisplay,
    pub sandbox_mode: SandboxMode,
    pub vim_mode: VimModeManager,
    pub workspace_name: String,
    pub workspace_count: usize,
    pub tokens_per_sec: f64,
    /// 会话开始时间戳（用于状态栏时长显示）。
    pub session_started_at: std::time::Instant,
    /// 是否显示左侧会话列表面板（Ctrl+S 切换，默认隐藏）
    pub show_sessions: bool,
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
            cursor: 0,
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
            streaming_model: None,
            streaming_renderer: StreamingMarkdownRenderer::new(),
            streaming_rendered_lines: Vec::new(),
            goal_display: GoalDisplay::idle(),
            sandbox_mode: SandboxMode::Disabled,
            vim_mode: VimModeManager::new(),
            workspace_name: String::new(),
            workspace_count: 0,
            tokens_per_sec: 0.0,
            session_started_at: std::time::Instant::now(),
            show_sessions: false,
            stream_started_at: None,
        }
    }

    pub fn push_message(&mut self, role: &str, content: String) {
        self.push_message_with_model(role, content, None);
    }

    pub fn push_message_with_model(&mut self, role: &str, content: String, model: Option<String>) {
        if let Some(session) = self.sessions.get_mut(self.active_session) {
            session.messages.push_back(ChatMessage::with_model(role, content, model));
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
        self.commit_stream_with_model(role, None);
    }

    pub fn commit_stream_with_model(&mut self, role: &str, model: Option<String>) {
        let text = std::mem::take(&mut self.streaming_text);
        // 完成流式渲染，刷新剩余缓冲
        let final_lines = self.streaming_renderer.finish();
        self.streaming_rendered_lines.extend(final_lines);
        if !text.is_empty() {
            self.push_message_with_model(role, text, model);
        }
        self.streaming = false;
        self.stream_started_at = None;
        self.tokens_per_sec = 0.0;
        // 清理渲染状态，准备下一次流式
        self.streaming_rendered_lines.clear();
        self.streaming_renderer = StreamingMarkdownRenderer::new();
    }

    /// 记录一个流式 chunk：更新 streaming_text + token 计数 + tokens/sec + 增量渲染。
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
        // 增量渲染：喂入 chunk 获取新增 Lines
        let new_lines = self.streaming_renderer.feed(chunk);
        self.streaming_rendered_lines.extend(new_lines);
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
        self.input.clear();
        self.cursor = 0;
    }

    /// 清空当前会话消息。
    pub fn clear_session(&mut self) {
        if let Some(session) = self.sessions.get_mut(self.active_session) {
            session.messages.clear();
        }
        self.scroll_offset = 0;
        self.streaming = false;
        self.streaming_text.clear();
        self.input.clear();
        self.cursor = 0;
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

        // vim 模式：启用时所有键经 VimModeManager（含 Insert 态，保证 Esc 能退出）。
        if self.vim_mode.is_enabled() {
            let action = self.vim_mode.handle_key(key, modifiers);
            // PassThrough → 落回普通输入处理（Enter 提交、Ctrl+C 等）。
            let passthrough = matches!(action, super::super::vim_mode::VimAction::PassThrough);
            if !passthrough {
            match action {
                super::super::vim_mode::VimAction::Quit => return KeyAction::Quit,
                super::super::vim_mode::VimAction::InsertChar(c) => {
                    self.insert_char(c);
                }
                super::super::vim_mode::VimAction::DeleteChar => {
                    self.delete_char();
                }
                super::super::vim_mode::VimAction::MoveLeft => self.move_cursor_left(),
                super::super::vim_mode::VimAction::MoveRight => self.move_cursor_right(),
                super::super::vim_mode::VimAction::MoveUp => self.move_cursor_line_up(),
                super::super::vim_mode::VimAction::MoveDown => self.move_cursor_line_down(),
                super::super::vim_mode::VimAction::MoveWordForward => self.move_word_forward(),
                super::super::vim_mode::VimAction::MoveWordBack => self.move_word_back(),
                super::super::vim_mode::VimAction::MoveLineStart => self.cursor = 0,
                super::super::vim_mode::VimAction::MoveLineEnd => self.cursor = self.input.len(),
                super::super::vim_mode::VimAction::MovePageUp => {
                    self.scroll_offset = self.scroll_offset.saturating_add(5);
                }
                super::super::vim_mode::VimAction::MovePageDown => {
                    self.scroll_offset = self.scroll_offset.saturating_sub(5);
                }
                super::super::vim_mode::VimAction::SwitchSession(n) => {
                    if n < self.sessions.len() {
                        self.active_session = n;
                        self.scroll_offset = 0;
                        self.thinking_expanded.clear();
                    }
                }
                super::super::vim_mode::VimAction::Yank
                | super::super::vim_mode::VimAction::Paste
                | super::super::vim_mode::VimAction::Undo
                | super::super::vim_mode::VimAction::Search(_)
                | super::super::vim_mode::VimAction::None
                | super::super::vim_mode::VimAction::EnterNormalMode
                | super::super::vim_mode::VimAction::EnterInsertMode
                | super::super::vim_mode::VimAction::EnterVisualMode
                | super::super::vim_mode::VimAction::PassThrough => {}
            }
            return KeyAction::None;
            }
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
                    self.cursor = 0;
                    KeyAction::None
                }
            }
            (KeyModifiers::CONTROL, Char('l')) => KeyAction::ClearScreen,
            (KeyModifiers::CONTROL, Char('s')) => {
                // 切换左侧会话面板显示/隐藏
                self.show_sessions = !self.show_sessions;
                self.scroll_offset = 0;
                KeyAction::None
            }
            (KeyModifiers::CONTROL, Char('m')) | (KeyModifiers::CONTROL, Char('j')) => {
                // Ctrl+M / Ctrl+J：多行模式下硬换行，否则提交。
                if self.multi_line {
                    self.insert_char('\n');
                    KeyAction::None
                } else {
                    KeyAction::Submit
                }
            }
            (KeyModifiers::NONE, Enter) | (KeyModifiers::NONE, Char('\n')) => {
                if self.multi_line {
                    self.insert_char('\n');
                    KeyAction::None
                } else {
                    KeyAction::Submit
                }
            }
            (KeyModifiers::ALT, Char('e')) => { self.multi_line = !self.multi_line; KeyAction::None }
            (KeyModifiers::NONE, Char(' ')) => { self.insert_char(' '); KeyAction::None }
            (KeyModifiers::NONE, Char(c)) => { self.insert_char(c); KeyAction::None }
            (KeyModifiers::NONE, Backspace) => { self.delete_char(); KeyAction::None }
            (KeyModifiers::NONE, Delete) => { self.delete_at_cursor(); KeyAction::None }
            (KeyModifiers::NONE, Left) => { self.move_cursor_left(); KeyAction::None }
            (KeyModifiers::NONE, Right) => { self.move_cursor_right(); KeyAction::None }
            (KeyModifiers::NONE, Home) => { self.cursor = 0; KeyAction::None }
            (KeyModifiers::NONE, End) => { self.cursor = self.input.len(); KeyAction::None }
            (KeyModifiers::NONE, Up) => {
                // 多行编辑时优先光标上移；否则命令历史导航。
                if self.multi_line && self.input.contains('\n') {
                    self.move_cursor_line_up();
                } else if let Some(prev) = self.command_history.navigate_up() {
                    self.input = prev;
                    self.cursor = self.input.len();
                }
                KeyAction::None
            }
            (KeyModifiers::NONE, Down) => {
                // 多行编辑时优先光标下移；否则命令历史导航。
                if self.multi_line && self.input.contains('\n') {
                    self.move_cursor_line_down();
                } else if let Some(next) = self.command_history.navigate_down() {
                    self.input = next;
                    self.cursor = self.input.len();
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
            _ => KeyAction::None,
        }
    }

    /// 在光标处插入字符（光标右移）。
    fn insert_char(&mut self, c: char) {
        if self.cursor <= self.input.len() {
            self.input.insert(self.cursor, c);
            self.cursor += c.len_utf8();
        }
    }

    /// Backspace：删除光标前一个字符。
    fn delete_char(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let prev = self.prev_char_boundary(self.cursor);
        self.input.drain(prev..self.cursor);
        self.cursor = prev;
    }

    /// Delete：删除光标处字符。
    fn delete_at_cursor(&mut self) {
        if self.cursor >= self.input.len() {
            return;
        }
        let next = self.next_char_boundary(self.cursor);
        self.input.drain(self.cursor..next);
    }

    /// 光标左移一个字符（保持 char boundary）。
    fn move_cursor_left(&mut self) {
        self.cursor = self.prev_char_boundary(self.cursor);
    }

    /// 光标右移一个字符（保持 char boundary）。
    fn move_cursor_right(&mut self) {
        self.cursor = self.next_char_boundary(self.cursor);
    }

    /// 光标前一个 char boundary。
    fn prev_char_boundary(&self, pos: usize) -> usize {
        if pos == 0 {
            return 0;
        }
        let mut p = pos - 1;
        while p > 0 && !self.input.is_char_boundary(p) {
            p -= 1;
        }
        p
    }

    /// 光标后一个 char boundary。
    fn next_char_boundary(&self, pos: usize) -> usize {
        let len = self.input.len();
        if pos >= len {
            return len;
        }
        let mut p = pos + 1;
        while p < len && !self.input.is_char_boundary(p) {
            p += 1;
        }
        p
    }

    /// 光标移到上一行（多行输入），保持列位置。
    fn move_cursor_line_up(&mut self) {
        let before = &self.input[..self.cursor];
        let line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
        if line_start == 0 {
            return;
        }
        let col = self.cursor - line_start;
        let prev_line_start = self.input[..line_start - 1].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let prev_line_len = (line_start - 1) - prev_line_start;
        self.cursor = prev_line_start + col.min(prev_line_len);
    }

    /// 光标移到下一行开头（多行输入），保持列。
    fn move_cursor_line_down(&mut self) {
        let rest = &self.input[self.cursor..];
        let line_end = rest.find('\n').map(|i| self.cursor + i).unwrap_or(self.input.len());
        if line_end >= self.input.len() {
            return;
        }
        let col = self.cursor - self.input[..self.cursor].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let next_line_start = line_end + 1;
        let next_line_end = self.input[next_line_start..].find('\n')
            .map(|i| next_line_start + i)
            .unwrap_or(self.input.len());
        let next_line_len = next_line_end - next_line_start;
        self.cursor = next_line_start + col.min(next_line_len);
    }

    /// 光标移到下一个单词开头。
    fn move_word_forward(&mut self) {
        let rest = &self.input[self.cursor..];
        let mut i = 0;
        let bytes = rest.as_bytes();
        // 跳过空白
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        // 跳过单词
        while i < bytes.len() && !bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        self.cursor = (self.cursor + i).min(self.input.len());
    }

    /// 光标移到上一个单词开头。
    fn move_word_back(&mut self) {
        let before = &self.input[..self.cursor];
        let bytes = before.as_bytes();
        let mut i = before.len();
        // 跳过空白
        while i > 0 && bytes[i - 1].is_ascii_whitespace() {
            i -= 1;
        }
        // 跳过单词
        while i > 0 && !bytes[i - 1].is_ascii_whitespace() {
            i -= 1;
        }
        self.cursor = i;
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
                    self.cursor = self.input.len();
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
    fn test_cursor_arrow_keys_move_within_bounds() {
        let mut app = TuiApp::new(true);
        for c in ['a', 'b', 'c'] {
            app.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
        }
        assert_eq!(app.cursor, 3);
        // 左移
        app.handle_key(KeyCode::Left, KeyModifiers::NONE);
        assert_eq!(app.cursor, 2);
        // 左移到 0 后不再越界
        app.handle_key(KeyCode::Left, KeyModifiers::NONE);
        app.handle_key(KeyCode::Left, KeyModifiers::NONE);
        app.handle_key(KeyCode::Left, KeyModifiers::NONE);
        assert_eq!(app.cursor, 0, "左移不越界");
        // 右移回末尾
        app.handle_key(KeyCode::Right, KeyModifiers::NONE);
        app.handle_key(KeyCode::Right, KeyModifiers::NONE);
        app.handle_key(KeyCode::Right, KeyModifiers::NONE);
        app.handle_key(KeyCode::Right, KeyModifiers::NONE);
        assert_eq!(app.cursor, 3, "右移不越界");
        // Home / End
        app.handle_key(KeyCode::End, KeyModifiers::NONE);
        assert_eq!(app.cursor, 3);
        app.handle_key(KeyCode::Home, KeyModifiers::NONE);
        assert_eq!(app.cursor, 0);
    }

    #[test]
    fn test_backspace_at_cursor_start_is_safe() {
        let mut app = TuiApp::new(true);
        for c in ['a', 'b'] {
            app.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
        }
        app.handle_key(KeyCode::Home, KeyModifiers::NONE);
        app.handle_key(KeyCode::Backspace, KeyModifiers::NONE);
        assert_eq!(app.input, "ab", "光标在行首时 Backspace 无操作");
        assert_eq!(app.cursor, 0);
    }

    #[test]
    fn test_insert_at_cursor_midline() {
        let mut app = TuiApp::new(true);
        for c in ['a', 'b', 'c'] {
            app.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
        }
        app.handle_key(KeyCode::Left, KeyModifiers::NONE); // cursor=2
        app.handle_key(KeyCode::Char('X'), KeyModifiers::NONE);
        assert_eq!(app.input, "abXc", "光标处插入");
        assert_eq!(app.cursor, 3);
        // Backspace 删除光标前字符
        app.handle_key(KeyCode::Backspace, KeyModifiers::NONE);
        assert_eq!(app.input, "abc", "Backspace 删除光标前字符");
        assert_eq!(app.cursor, 2);
    }

    #[test]
    fn test_multiline_cursor_vertical_move() {
        let mut app = TuiApp::new(true);
        app.multi_line = true;
        for c in ['a', '\n', 'b', 'c'] {
            app.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
        }
        // 光标在 (2,0)：第 2 行末尾
        assert_eq!(app.cursor, 4);
        // 上移一行，保持列 0 → 行 1 末尾（len=1）
        app.handle_key(KeyCode::Up, KeyModifiers::NONE);
        assert_eq!(app.cursor, 1, "上移一行");
        // 下移保持列 1 → 第 2 行偏移 1（b 处），即 3
        app.handle_key(KeyCode::Down, KeyModifiers::NONE);
        assert_eq!(app.cursor, 3, "下移保持列位置");
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

    // ── P0-1: 光标移动 + Backspace/Delete 语义 ──

    #[test]
    fn test_cursor_starts_at_end() {
        let mut app = TuiApp::new(true);
        for c in ['a', 'b', 'c'] {
            app.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
        }
        assert_eq!(app.input, "abc");
        assert_eq!(app.cursor, 3);
    }

    #[test]
    fn test_cursor_left_right() {
        let mut app = TuiApp::new(true);
        for c in ['a', 'b', 'c'] {
            app.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
        }
        app.handle_key(KeyCode::Left, KeyModifiers::NONE);
        assert_eq!(app.cursor, 2);
        app.handle_key(KeyCode::Right, KeyModifiers::NONE);
        assert_eq!(app.cursor, 3);
        // 边界: 左移不越界
        for _ in 0..5 {
            app.handle_key(KeyCode::Left, KeyModifiers::NONE);
        }
        assert_eq!(app.cursor, 0);
        // 边界: 右移不越界
        for _ in 0..5 {
            app.handle_key(KeyCode::Right, KeyModifiers::NONE);
        }
        assert_eq!(app.cursor, 3);
    }

    #[test]
    fn test_cursor_home_end() {
        let mut app = TuiApp::new(true);
        for c in ['a', 'b', 'c'] {
            app.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
        }
        app.handle_key(KeyCode::Home, KeyModifiers::NONE);
        assert_eq!(app.cursor, 0);
        app.handle_key(KeyCode::End, KeyModifiers::NONE);
        assert_eq!(app.cursor, 3);
    }

    #[test]
    fn test_insert_at_cursor_position() {
        let mut app = TuiApp::new(true);
        for c in ['a', 'b', 'c'] {
            app.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
        }
        app.handle_key(KeyCode::Left, KeyModifiers::NONE); // cursor=2
        app.handle_key(KeyCode::Char('X'), KeyModifiers::NONE);
        assert_eq!(app.input, "abXc");
        assert_eq!(app.cursor, 3);
    }

    #[test]
    fn test_backspace_deletes_before_cursor() {
        let mut app = TuiApp::new(true);
        for c in ['a', 'b', 'c'] {
            app.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
        }
        app.handle_key(KeyCode::Left, KeyModifiers::NONE); // cursor=2
        app.handle_key(KeyCode::Backspace, KeyModifiers::NONE);
        assert_eq!(app.input, "ac");
        assert_eq!(app.cursor, 1);
        // 边界: cursor=0 时 Backspace 无操作
        app.handle_key(KeyCode::Home, KeyModifiers::NONE);
        app.handle_key(KeyCode::Backspace, KeyModifiers::NONE);
        assert_eq!(app.input, "ac");
        assert_eq!(app.cursor, 0);
    }

    #[test]
    fn test_delete_deletes_at_cursor() {
        let mut app = TuiApp::new(true);
        for c in ['a', 'b', 'c'] {
            app.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
        }
        app.handle_key(KeyCode::Left, KeyModifiers::NONE); // cursor=2
        app.handle_key(KeyCode::Delete, KeyModifiers::NONE);
        assert_eq!(app.input, "ab");
        assert_eq!(app.cursor, 2);
        // 边界: cursor=len 时 Delete 无操作
        app.handle_key(KeyCode::Delete, KeyModifiers::NONE);
        assert_eq!(app.input, "ab");
    }

    #[test]
    fn test_cursor_survives_clear_and_new_session() {
        let mut app = TuiApp::new(true);
        for c in ['a', 'b'] {
            app.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
        }
        app.handle_key(KeyCode::Left, KeyModifiers::NONE);
        app.clear_session();
        assert_eq!(app.cursor, 0);
        app.handle_key(KeyCode::Char('z'), KeyModifiers::NONE);
        assert_eq!(app.input, "z");
        assert_eq!(app.cursor, 1);
    }

    // ── P0-3: Ctrl+字母组合键不得插入字符 ──

    #[test]
    fn test_ctrl_letter_does_not_insert_char() {
        let mut app = TuiApp::new(true);
        app.handle_key(KeyCode::Char('a'), KeyModifiers::CONTROL);
        assert_eq!(app.input, "", "Ctrl+A 不应插入 'a'");
        app.handle_key(KeyCode::Char('e'), KeyModifiers::CONTROL);
        assert_eq!(app.input, "", "Ctrl+E 不应插入 'e'");
        app.handle_key(KeyCode::Char('w'), KeyModifiers::CONTROL);
        assert_eq!(app.input, "", "Ctrl+W 不应插入 'w'");
    }

    #[test]
    fn test_ctrl_space_does_not_insert() {
        let mut app = TuiApp::new(true);
        app.handle_key(KeyCode::Char(' '), KeyModifiers::CONTROL);
        assert_eq!(app.input, "", "Ctrl+Space 不应插入空格");
    }

    // ── P0-2: vim 模式移动动作接线 ──

    #[test]
    fn test_vim_normal_hjkl_moves_cursor() {
        let mut app = TuiApp::new(true);
        app.vim_mode.toggle(); // 启用 vim
        // Normal 模式按 i 进入 Insert，输入 abc，Esc 回 Normal
        app.handle_key(KeyCode::Char('i'), KeyModifiers::NONE);
        for c in ['a', 'b', 'c'] {
            app.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
        }
        app.handle_key(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(app.input, "abc");
        assert_eq!(app.cursor, 3);
        // h = 左移
        app.handle_key(KeyCode::Char('h'), KeyModifiers::NONE);
        assert_eq!(app.cursor, 2, "vim h 应左移光标");
        // l = 右移
        app.handle_key(KeyCode::Char('l'), KeyModifiers::NONE);
        assert_eq!(app.cursor, 3, "vim l 应右移光标");
        // 0 = 行首
        app.handle_key(KeyCode::Char('0'), KeyModifiers::NONE);
        assert_eq!(app.cursor, 0, "vim 0 应到行首");
        // $ = 行尾
        app.handle_key(KeyCode::Char('$'), KeyModifiers::NONE);
        assert_eq!(app.cursor, 3, "vim $ 应到行尾");
    }

    #[test]
    fn test_vim_normal_i_enters_insert_and_types() {
        let mut app = TuiApp::new(true);
        app.vim_mode.toggle();
        // 输入 ab
        app.handle_key(KeyCode::Char('i'), KeyModifiers::NONE);
        for c in ['a', 'b'] {
            app.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
        }
        app.handle_key(KeyCode::Esc, KeyModifiers::NONE);
        app.handle_key(KeyCode::Char('h'), KeyModifiers::NONE); // cursor=1
        app.handle_key(KeyCode::Char('i'), KeyModifiers::NONE); // 进入 insert
        app.handle_key(KeyCode::Char('X'), KeyModifiers::NONE);
        assert_eq!(app.input, "aXb", "vim i 后输入应插入光标处");
        assert_eq!(app.cursor, 2);
    }

    #[test]
    fn test_vim_gg_and_G_line_edges() {
        let mut app = TuiApp::new(true);
        app.vim_mode.toggle();
        app.handle_key(KeyCode::Char('i'), KeyModifiers::NONE);
        for c in ['a', 'b', 'c'] {
            app.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
        }
        app.handle_key(KeyCode::Esc, KeyModifiers::NONE);
        app.handle_key(KeyCode::Char('g'), KeyModifiers::NONE);
        app.handle_key(KeyCode::Char('g'), KeyModifiers::NONE);
        assert_eq!(app.cursor, 0, "vim gg 应到行首");
        app.handle_key(KeyCode::Char('G'), KeyModifiers::NONE);
        assert_eq!(app.cursor, 3, "vim G 应到行尾");
    }
}
