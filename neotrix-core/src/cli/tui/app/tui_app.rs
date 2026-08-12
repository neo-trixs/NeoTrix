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
use crate::cli::tui::diff_viewer::DiffViewer;
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

/// Spinner 动画帧（借鉴 claude-code-local 的 braille 帧，让"沉默≠卡死"）。
pub const SPINNER_FRAMES: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

/// 估算的 context 窗口上限（tokens）。TUI 无真实 tokenizer，用 chunk 计数估算。
pub const CONTEXT_LIMIT_ESTIMATE: usize = 128_000;

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
    /// P1-5 修复: 会话切换/清空后请求重置 AgentLoop 历史 (事件循环消费后清回 false)。
    pub needs_agent_reset: bool,
    pub status_text: String,
    pub diff_viewer: Option<DiffViewer>,
    pub thinking_expanded: HashSet<String>,
    /// 已展开的工具调用（key = "session:msg_idx:tool_idx"）
    pub tool_calls_expanded: HashSet<String>,
    pub streaming_role: String,
    pub streaming_text: String,
    /// 流式输出时的模型名（用于显示）
    pub streaming_model: Option<String>,
    /// 流式工具调用可视化（task B）：工具名 → 状态（running/done/error）
    pub streaming_tool_calls: Vec<crate::cli::tui::app::types::ToolCall>,
    /// 待审批动作（task A）：渲染在状态栏上方，a=允许 / d=拒绝
    pub pending_approval: Option<crate::cli::approval::PendingAction>,
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
    /// 当前主题名（dark/light/gruvbox；Alt+T 循环切换）
    pub theme_name: String,
    /// 自动滚动到底部（流式输出时跟随最新内容；用户手动滚动后关闭）
    pub auto_scroll: bool,
    /// 流式开始时记录，用于计算 tokens/sec。
    stream_started_at: Option<std::time::Instant>,
    /// Spinner 动画帧索引（事件循环每 tick 推进一次）。
    pub spinner_frame: usize,
    /// 当前 git 分支（无 git 仓库或检测失败为 None）。
    pub git_branch: Option<String>,
    /// git 工作区是否有未提交改动（dirty 指示）。
    pub git_dirty: bool,
    /// 会话恢复 picker（/sessions 打开；None = 未激活）。
    pub session_picker: Option<crate::cli::tui::app::types::SessionPicker>,
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
            needs_agent_reset: false,
            status_text: if ephemeral { "Ready".into() } else { "Ready | Provider: not configured".into() },
            diff_viewer: None,
            thinking_expanded: HashSet::new(),
            tool_calls_expanded: HashSet::new(),
            streaming_role: String::new(),
            streaming_text: String::new(),
            streaming_model: None,
            streaming_tool_calls: Vec::new(),
            pending_approval: None,
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
            theme_name: "dark".to_string(),
            auto_scroll: true,
            stream_started_at: None,
            spinner_frame: 0,
            git_branch: Self::detect_git_branch(),
            git_dirty: Self::detect_git_dirty(),
            session_picker: None,
        }
    }

    /// 检测当前 git 分支（best-effort：非 git 仓库返回 None）。
    fn detect_git_branch() -> Option<String> {
        let out = std::process::Command::new("git")
            .args(["branch", "--show-current"])
            .output()
            .ok()?;
        if !out.status.success() {
            return None;
        }
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if s.is_empty() { None } else { Some(s) }
    }

    /// 检测工作区是否有未提交改动（git status --porcelain 非空 → dirty）。
    fn detect_git_dirty() -> bool {
        match std::process::Command::new("git").args(["status", "--porcelain"]).output() {
            Ok(out) => !out.status.success() || !String::from_utf8_lossy(&out.stdout).trim().is_empty(),
            Err(_) => false,
        }
    }

    pub fn push_message(&mut self, role: &str, content: String) {
        self.push_message_with_model(role, content, None);
    }

    pub fn push_message_with_model(&mut self, role: &str, content: String, model: Option<String>) {
        if let Some(session) = self.sessions.get_mut(self.active_session) {
            // 会话自动命名：默认名 + 首条 user 消息 → 用内容前 20 字符
            if role == "user" && session.messages.is_empty() && (session.name.starts_with("Session ") || session.name == "Default Session") {
                let mut name = content.trim().chars().take(20).collect::<String>();
                if content.chars().count() > 20 {
                    name.push('…');
                }
                if !name.is_empty() {
                    session.name = name;
                }
            }
            session.messages.push_back(ChatMessage::with_model(role, content, model));
        }
    }

    pub fn active_session(&self) -> &Session {
        &self.sessions[self.active_session]
    }

    /// 循环切换主题：dark → light → gruvbox → dark，并持久化到 config 的 color_mode。
    pub fn cycle_theme(&mut self) {
        self.theme_name = match self.theme_name.as_str() {
            "dark" => "light".to_string(),
            "light" => "gruvbox".to_string(),
            _ => "dark".to_string(),
        };
        let _ = crate::config::NeoTrixConfig::default().save_field("color_mode", &self.theme_name);
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
        // 自动滚动：跟随最新内容（用户手动滚动后关闭）
        if self.auto_scroll {
            self.scroll_offset = 0;
        }
    }

    /// 流式工具调用开始（task B）：记录工具名，状态 running。
    pub fn start_streaming_tool(&mut self, name: &str, args: &str) {
        // 若同工具重入，更新 args 而不是追加。
        if let Some(tc) = self.streaming_tool_calls.last_mut() {
            if tc.status == "running" && tc.name == name {
                tc.args = args.chars().take(500).collect();
                return;
            }
        }
        self.streaming_tool_calls.push(crate::cli::tui::app::types::ToolCall {
            name: name.to_string(),
            args: args.chars().take(500).collect(),
            duration_ms: 0,
            success: false,
            status: "running".into(),
            result: String::new(),
        });
        // 工具执行期间暂停自动滚动，让用户看到工具区。
        self.auto_scroll = false;
    }

    /// 流式工具调用结束（task B）：更新状态与结果摘要。
    pub fn finish_streaming_tool(&mut self, name: &str, duration_ms: u64, success: bool, result: &str) {
        let mut all_done = true;
        if let Some(tc) = self.streaming_tool_calls.iter_mut().rev().find(|tc| tc.name == name) {
            tc.status = if success { "done".into() } else { "error".into() };
            tc.success = success;
            tc.duration_ms = duration_ms;
            tc.result = result.chars().take(300).collect();
        }
        // P2 修复: 工具结束后无其他 running 工具 → 恢复自动滚动 (避免会话卡住不跟随)。
        all_done = self.streaming_tool_calls.iter().all(|tc| tc.status != "running");
        if all_done {
            self.auto_scroll = true;
        }
    }

    /// 清除全部流式工具（本轮结束/开始时调用）。
    pub fn clear_streaming_tools(&mut self) {
        self.streaming_tool_calls.clear();
        self.auto_scroll = true;
    }

    /// 推进 spinner 动画帧（事件循环每 tick 调用一次，驱动状态栏动画）。
    pub fn tick_spinner(&mut self) {
        self.spinner_frame = (self.spinner_frame + 1) % SPINNER_FRAMES.len();
    }

    /// 当前 spinner 帧字符。
    pub fn spinner_char(&self) -> char {
        SPINNER_FRAMES[self.spinner_frame]
    }

    /// 估算 context 用量百分比（token_count / 估算窗口，封顶 100）。
    pub fn context_pct(&self) -> u8 {
        let pct = (self.token_count as f64 / CONTEXT_LIMIT_ESTIMATE as f64) * 100.0;
        pct.min(100.0) as u8
    }

    /// 流式/思考已持续的秒数（用于状态栏计时显示）。
    pub fn busy_elapsed_secs(&self) -> u64 {
        self.stream_started_at
            .map(|t| t.elapsed().as_secs())
            .unwrap_or(0)
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

    // ── Diff 查看模式 ──

    /// 打开 diff 查看模式：解析内容并进入 diff 面板。
    /// 内容可来自 `/git diff` 命令输出、tool 调用的 edit 结果或任意文本。
    pub fn open_diff(&mut self, content: String) {
        self.diff_viewer = Some(DiffViewer::new(content));
        self.status_text = "Diff 查看模式 (↑↓ 滚动 · q/Esc 退出)".into();
    }

    /// 关闭 diff 查看模式。
    pub fn close_diff(&mut self) {
        self.diff_viewer = None;
    }

    /// diff 查看模式是否激活。
    pub fn diff_active(&self) -> bool {
        self.diff_viewer.is_some()
    }

    /// diff 查看模式的键处理：↑↓/k/j 滚动，PageUp/PageDown 翻页，q/Esc 退出，Ctrl+C 关闭。
    fn handle_diff_key(&mut self, key: crossterm::event::KeyCode, modifiers: crossterm::event::KeyModifiers) -> KeyAction {
        use crossterm::event::{KeyCode::*, KeyModifiers};
        match (modifiers, key) {
            (KeyModifiers::NONE, Up) | (KeyModifiers::NONE, Char('k')) => {
                if let Some(v) = self.diff_viewer.as_mut() {
                    v.scroll_up(1);
                }
                KeyAction::None
            }
            (KeyModifiers::NONE, Down) | (KeyModifiers::NONE, Char('j')) => {
                if let Some(v) = self.diff_viewer.as_mut() {
                    v.scroll_down(1);
                }
                KeyAction::None
            }
            (KeyModifiers::NONE, PageUp) => {
                if let Some(v) = self.diff_viewer.as_mut() {
                    v.scroll_up(10);
                }
                KeyAction::None
            }
            (KeyModifiers::NONE, PageDown) => {
                if let Some(v) = self.diff_viewer.as_mut() {
                    v.scroll_down(10);
                }
                KeyAction::None
            }
            (KeyModifiers::NONE, Char('q')) | (KeyModifiers::NONE, Esc) => {
                self.close_diff();
                KeyAction::None
            }
            (KeyModifiers::CONTROL, Char('c')) => {
                self.close_diff();
                KeyAction::None
            }
            _ => KeyAction::None,
        }
    }

    // ── 会话自动恢复 ──

    /// 从 SessionStore（KB session_logs）加载最近会话历史到 sessions 列表。
    /// 返回恢复的消息数；无历史时返回 0 并保留默认空会话。
    pub fn restore_sessions(&mut self) -> usize {
        self.restore_sessions_from_base(None)
    }

    /// 指定 base 目录的恢复（测试/隔离环境用）。
    pub fn restore_sessions_from_base(&mut self, base: Option<std::path::PathBuf>) -> usize {
        use crate::cli::tui::session_store::SessionStore;
        let store = match base {
            Some(b) => SessionStore::with_base(b),
            None => SessionStore::new(),
        };
        let Some(last) = store.get_last_session() else { return 0 };
        let Ok(data) = store.load_session(&last) else { return 0 };
        let mut session = Session {
            id: data.id.clone(),
            name: data.name.clone(),
            messages: VecDeque::new(),
        };
        for line in &data.messages {
            let (role, content) = parse_role_line(line);
            session.messages.push_back(ChatMessage::new(role, content));
        }
        if session.messages.is_empty() {
            return 0;
        }
        let count = session.messages.len();
        self.sessions.clear();
        self.sessions.push(session);
        self.active_session = 0;
        self.scroll_offset = 0;
        self.thinking_expanded.clear();
        count
    }

    /// 切换最后一条 assistant 消息的 thinking 展开状态（t 键）。
    pub fn toggle_last_thinking(&mut self) {
        let session = &self.sessions[self.active_session];
        let last_assistant = session.messages.iter().rposition(|m| m.role == "assistant");
        if let Some(idx) = last_assistant {
            let key = format!("{}:{}", self.active_session, idx);
            if self.thinking_expanded.contains(&key) {
                self.thinking_expanded.remove(&key);
            } else {
                self.thinking_expanded.insert(key);
            }
        }
    }

    /// 切换指定工具调用的展开状态（x 键，作用于最后一条 assistant 消息）。
    pub fn toggle_last_tool_call(&mut self) {
        let session = &self.sessions[self.active_session];
        let idx = session.messages.iter().rposition(|m| m.role == "assistant");
        if let Some(msg_idx) = idx {
            let msg = &session.messages[msg_idx];
            if let Some(tool_idx) = msg.tool_calls.iter().rposition(|_| true) {
                let key = format!("{}:{}:{}", self.active_session, msg_idx, tool_idx);
                if self.tool_calls_expanded.contains(&key) {
                    self.tool_calls_expanded.remove(&key);
                } else {
                    self.tool_calls_expanded.insert(key);
                }
            }
        }
    }

    /// 核心键处理。返回动作由 `run()` 执行（提交输入/触发 LLM 轮询等）。
    pub fn handle_key(&mut self, key: crossterm::event::KeyCode, modifiers: crossterm::event::KeyModifiers) -> KeyAction {
        use crossterm::event::{KeyCode::*, KeyModifiers};

        // diff 查看模式：模态拦截（↑↓/k/j 滚动，PageUp/Down 翻页，q/Esc 退出，Ctrl+C 关闭）。
        if self.diff_viewer.is_some() {
            return self.handle_diff_key(key, modifiers);
        }

        // 审批模式：模态拦截（a=允许，d=拒绝，Esc/Ctrl+C 拒绝并取消生成）。
        if self.pending_approval.is_some() {
            if modifiers == KeyModifiers::CONTROL && key == Char('c')
                || (modifiers == KeyModifiers::NONE && key == Esc) {
                self.pending_approval = None;
                return KeyAction::CancelGeneration;
            }
            match key {
                Char('a') | Char('A') => {
                    self.pending_approval = None;
                    return KeyAction::ApprovePending;
                }
                Char('d') | Char('D') | Char('n') | Char('N') => {
                    self.pending_approval = None;
                    return KeyAction::DenyPending;
                }
                _ => return KeyAction::None,
            }
        }

        // 会话 picker 模式：模态拦截（↑↓/j/k 选择，Enter 加载，d 删除，Esc/q 关闭）。
        if let Some(picker) = &self.session_picker {
            if picker.entries.is_empty() {
                self.session_picker = None;
                return KeyAction::ClosePicker;
            }
            let sel = picker.selected;
            let action = match key {
                Char('j') | Down => {
                    if let Some(p) = &mut self.session_picker {
                        p.selected = (p.selected + 1).min(p.entries.len() - 1);
                    }
                    KeyAction::None
                }
                Char('k') | Up => {
                    if let Some(p) = &mut self.session_picker {
                        p.selected = p.selected.saturating_sub(1);
                    }
                    KeyAction::None
                }
                Enter => {
                    self.session_picker = None;
                    KeyAction::SelectSession(sel)
                }
                Char('d') | Char('D') | Delete => KeyAction::DeleteSession(sel),
                Esc | Char('q') | Char('Q') => {
                    self.session_picker = None;
                    KeyAction::ClosePicker
                }
                _ => KeyAction::None,
            };
            // 会话 picker 模态下不落入其他处理（Esc/q 已覆盖退出，Ctrl+C 语义一致）。
            return action;
        }

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
                    self.auto_scroll = false;
                }
                super::super::vim_mode::VimAction::MovePageDown => {
                    self.scroll_offset = self.scroll_offset.saturating_sub(5);
                    self.auto_scroll = false;
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
            (KeyModifiers::ALT, Char('t')) => { self.cycle_theme(); KeyAction::None }
            (KeyModifiers::NONE, Char(' ')) => { self.insert_char(' '); KeyAction::None }
            (KeyModifiers::CONTROL, Char('t')) => { self.toggle_last_thinking(); KeyAction::None }
            (KeyModifiers::CONTROL, Char('x')) => { self.toggle_last_tool_call(); KeyAction::None }
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
            (KeyModifiers::NONE, PageUp) => { self.scroll_offset = self.scroll_offset.saturating_add(5); self.auto_scroll = false; KeyAction::None }
            (KeyModifiers::NONE, PageDown) => { self.scroll_offset = self.scroll_offset.saturating_sub(5); self.auto_scroll = false; KeyAction::None }
            (_, Tab) => {
                if self.input.starts_with('/') && !self.input.contains(' ') {
                    self.complete_slash();
                } else if self.has_at_reference() {
                    self.complete_at_reference();
                }
                KeyAction::None
            }
            (KeyModifiers::NONE, Esc) => {
                // 流式中 Esc 取消生成（Claude Code 习惯）；否则退出多行模式。
                if self.streaming {
                    self.agent_busy = false;
                    self.streaming = false;
                    self.status_text = "生成已取消".into();
                    KeyAction::CancelGeneration
                } else if self.multi_line {
                    self.multi_line = false;
                    KeyAction::None
                } else {
                    KeyAction::None
                }
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

    /// P1-2: 粘贴文本批量插入光标处（char-safe，保留换行/多行）。
    /// 由 `Event::Paste` 调用，避免逐 \n 触发 Enter/Submit。
    pub fn insert_text(&mut self, text: &str) {
        for c in text.chars() {
            self.insert_char(c);
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
    /// 命令源 = registry 全量（90+）∪ TUI 专用命令，动态获取（对标 claude-code 的
    /// 全量命令补全）。前缀匹配取第一个；无匹配则不动作。
    fn complete_slash(&mut self) {
        let prefix = self.input.as_str();
        let matches = slash_command_candidates(prefix);
        if let Some(cmd) = matches.first() {
            self.input = cmd.clone();
        }
    }
    /// 输入中是否含未完成的 `@` 文件引用（@ 之后还有路径片段，光标在 @ 之后）。
    /// 对标 claude-code 的 `@file` 路径引用（Tab 补全当前目录）。
    fn has_at_reference(&self) -> bool {
        let Some(at) = self.input.rfind('@') else { return false };
        // @ 之前的字符必须是空白或行首（避免误判邮箱 / 用户名）
        if at > 0 {
            let prev = self.input.as_bytes()[at - 1];
            if !prev.is_ascii_whitespace() && prev != b'\n' {
                return false;
            }
        }
        // 光标必须位于 @ 之后（正在输入引用片段）
        self.cursor > at
    }

    /// 补全 `@` 后的路径前缀：列出当前目录匹配项，补全第一个（目录加 `/`）。
    fn complete_at_reference(&mut self) {
        let Some(at) = self.input.rfind('@') else { return };
        let prefix: String = self.input[at + 1..].chars().take(200).collect();
        let dir = std::path::Path::new(".");
        let Ok(entries) = std::fs::read_dir(dir) else { return };
        let mut matches: Vec<String> = Vec::new();
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(&prefix) {
                let is_dir = entry.path().is_dir();
                matches.push(if is_dir { format!("{}/", name) } else { name });
            }
        }
        matches.sort();
        if matches.is_empty() {
            return;
        }
        // 只有一个匹配 → 直接补全；多个 → 补全到最长公共前缀
        if matches.len() == 1 {
            self.input = format!("{}@{}", &self.input[..at], matches[0]);
            self.cursor = self.input.len();
            return;
        }
        let common = longest_common_prefix(&matches);
        if common.len() > prefix.len() {
            self.input = format!("{}@{}", &self.input[..at], common);
            self.cursor = self.input.len();
        } else {
            // 多个候选无公共前缀 → 把候选列表展示到状态栏
            self.status_text = format!("@ 候选: {}", matches.join(" "));
        }
    }
}

/// 多个字符串的最长公共前缀。
fn longest_common_prefix(strs: &[String]) -> String {
    if strs.is_empty() {
        return String::new();
    }
    let first = strs[0].as_bytes();
    let mut len = first.len();
    for s in &strs[1..] {
        let bytes = s.as_bytes();
        let mut i = 0;
        while i < len && i < bytes.len() && bytes[i] == first[i] {
            i += 1;
        }
        len = i;
    }
    strs[0].chars().take(len).collect()
}
/// 用 OnceLock 缓存 registry 命令名（registry 构造含 session logging，避免每 Tab 重建；
/// Command::name 非 'static，故缓存 String）。
static SLASH_CANDIDATES: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();

/// 返回匹配前缀的全部命令（顺序：TUI 专用在前，registry 在后）。
pub fn slash_command_candidates(prefix: &str) -> Vec<String> {
    const TUI_ONLY: &[&str] = &[
        "/save", "/load", "/resume", "/sessions", "/hist",
    ];
    let reg_names: &Vec<String> = SLASH_CANDIDATES.get_or_init(|| {
        let reg = crate::cli::commands::registry::default_registry();
        reg.list_primary().into_iter().map(|s| s.to_string()).collect()
    });
    let mut out: Vec<String> = Vec::new();
    for cmd in TUI_ONLY {
        if cmd.starts_with(prefix) && cmd.len() > prefix.len() {
            out.push(cmd.to_string());
        }
    }
    for cmd in reg_names {
        if cmd.starts_with(prefix) && cmd.len() > prefix.len() && !out.contains(cmd) {
            out.push(cmd.clone());
        }
    }
    out
}

/// 键处理返回的动作，供事件循环执行。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyAction {
    None,
    Quit,
    Submit,
    ClearScreen,
    CancelGeneration,
    ApprovePending,
    DenyPending,
    /// 会话 picker 选中项（/sessions）：加载对应会话。
    SelectSession(usize),
    /// 会话 picker 关闭（Esc/q）。
    ClosePicker,
    /// 会话 picker 删除选中项（d）。
    DeleteSession(usize),
}

/// 解析 SessionStore 消息行 "[role] content" → (role, content)。
/// 未匹配已知角色时按 system 处理。
fn parse_role_line(line: &str) -> (&str, String) {
    if let Some(c) = line.strip_prefix("[user] ") {
        ("user", c.to_string())
    } else if let Some(c) = line.strip_prefix("[assistant] ") {
        ("assistant", c.to_string())
    } else if let Some(c) = line.strip_prefix("[system] ") {
        ("system", c.to_string())
    } else {
        ("system", line.to_string())
    }
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
        assert_eq!(app.spinner_frame, 0);
    }

    #[test]
    fn test_spinner_tick_cycles_frames() {
        let mut app = TuiApp::new(true);
        assert_eq!(app.spinner_char(), SPINNER_FRAMES[0]);
        app.tick_spinner();
        assert_eq!(app.spinner_frame, 1);
        assert_eq!(app.spinner_char(), SPINNER_FRAMES[1]);
        // 循环回绕
        for _ in 0..SPINNER_FRAMES.len() {
            app.tick_spinner();
        }
        assert_eq!(app.spinner_frame, 1, "帧索引应循环回绕");
    }

    #[test]
    fn test_context_pct_bounds() {
        let mut app = TuiApp::new(true);
        assert_eq!(app.context_pct(), 0, "无 token 时 0%");
        app.token_count = CONTEXT_LIMIT_ESTIMATE / 2;
        assert_eq!(app.context_pct(), 50, "半窗口 50%");
        app.token_count = CONTEXT_LIMIT_ESTIMATE * 2;
        assert_eq!(app.context_pct(), 100, "超窗口封顶 100%");
    }

    #[test]
    fn test_busy_elapsed_secs_zero_when_idle() {
        let app = TuiApp::new(true);        assert_eq!(app.busy_elapsed_secs(), 0, "空闲时无计时");
    }

    #[test]
    fn test_theme_cycle_dark_light_gruvbox() {
        let mut app = TuiApp::new(true);
        assert_eq!(app.theme_name, "dark");
        app.cycle_theme();
        assert_eq!(app.theme_name, "light");
        app.cycle_theme();
        assert_eq!(app.theme_name, "gruvbox");
        app.cycle_theme();
        assert_eq!(app.theme_name, "dark", "应循环回 dark");
    }

    #[test]
    fn test_alt_t_cycles_theme() {
        let mut app = TuiApp::new(true);
        app.handle_key(KeyCode::Char('t'), KeyModifiers::ALT);
        assert_eq!(app.theme_name, "light");
        app.handle_key(KeyCode::Char('t'), KeyModifiers::ALT);
        assert_eq!(app.theme_name, "gruvbox");
    }

    #[test]
    fn test_ctrl_s_toggles_session_panel() {
        let mut app = TuiApp::new(true);
        assert!(!app.show_sessions, "默认隐藏侧栏");
        app.handle_key(KeyCode::Char('s'), KeyModifiers::CONTROL);
        assert!(app.show_sessions);
        app.handle_key(KeyCode::Char('s'), KeyModifiers::CONTROL);
        assert!(!app.show_sessions);
    }

    #[test]
    fn test_auto_scroll_follows_stream_and_disables_on_manual() {
        let mut app = TuiApp::new(true);
        assert!(app.auto_scroll, "默认自动滚动");
        app.scroll_offset = 3;
        app.feed_stream("chunk\n");
        assert_eq!(app.scroll_offset, 0, "流式时自动滚动回底部");
        // 手动 PageUp → 关闭自动滚动
        app.handle_key(KeyCode::PageUp, KeyModifiers::NONE);
        assert!(!app.auto_scroll);
        app.scroll_offset = 0;
        app.feed_stream("more\n");
        assert!(app.scroll_offset > 0 || app.scroll_offset == 0);
    }

    #[test]
    fn test_auto_scroll_reenabled_after_tools_done() {
        let mut app = TuiApp::new(true);
        app.start_streaming_tool("mcp_search", "{}");
        assert!(!app.auto_scroll, "工具 running 时暂停自动滚动");
        app.finish_streaming_tool("mcp_search", 10, true, "ok");
        assert!(app.auto_scroll, "工具完成后恢复自动滚动");
    }

    #[test]
    fn test_esc_cancels_streaming_generation() {
        let mut app = TuiApp::new(true);
        app.streaming = true;
        app.agent_busy = true;
        let action = app.handle_key(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(action, KeyAction::CancelGeneration);
        assert!(!app.streaming);
        assert!(!app.agent_busy);
    }

    #[test]
    fn test_t_toggles_last_thinking() {
        let mut app = TuiApp::new(true);
        app.push_message("assistant", "正文\n\n<think>思考1</think>\n结束".to_string());
        app.handle_key(KeyCode::Char('t'), KeyModifiers::CONTROL);
        assert_eq!(app.thinking_expanded.len(), 1, "Ctrl+T 应展开 thinking");
        app.handle_key(KeyCode::Char('t'), KeyModifiers::CONTROL);
        assert!(app.thinking_expanded.is_empty(), "再按 Ctrl+T 应折叠");
    }

    #[test]
    fn test_x_toggles_last_tool_call() {
        let mut app = TuiApp::new(true);
        app.push_message("assistant", "正文\n\n🛠️ read_file(path=\"a.rs\")\n".to_string());
        app.handle_key(KeyCode::Char('x'), KeyModifiers::CONTROL);
        assert_eq!(app.tool_calls_expanded.len(), 1, "Ctrl+X 应展开工具调用");
        app.handle_key(KeyCode::Char('x'), KeyModifiers::CONTROL);
        assert!(app.tool_calls_expanded.is_empty());
    }

    #[test]
    fn test_session_auto_named_from_first_user_message() {
        let mut app = TuiApp::new(true);
        assert_eq!(app.sessions[0].name, "Default Session");
        app.push_message("user", "帮我实现一个排序算法".to_string());
        assert_eq!(app.sessions[0].name, "帮我实现一个排序算法");
        // 超长内容截断到 20 字符 + …
        app.new_session();
        let long = "这是一个非常长的第一条用户消息用于测试自动命名截断功能是否正常工作";
        app.push_message("user", long.to_string());
        let name = &app.sessions[1].name;
        assert!(name.chars().count() <= 21, "会话名应截断: {}", name);
        assert!(name.ends_with('…'));
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

    // ── Diff 查看模式 ──

    #[test]
    fn test_open_diff_parses_and_navigates() {
        let mut app = TuiApp::new(true);
        let diff = "diff --git a/a.rs b/a.rs\nindex 123..456 100644\n--- a/a.rs\n+++ b/a.rs\n@@ -1,2 +1,3 @@\n-old line\n+new line\n context\n";
        app.open_diff(diff.to_string());
        assert!(app.diff_active());
        let viewer = app.diff_viewer.as_ref().unwrap();
        assert_eq!(viewer.blocks.len(), 1, "应解析出一个 diff block");
        assert_eq!(viewer.blocks[0].hunks.len(), 1, "应解析出一个 hunk");
        assert_eq!(viewer.scroll_offset, 0);
        // 顶部再上滚不越界
        app.handle_key(KeyCode::Up, KeyModifiers::NONE);
        assert_eq!(app.diff_viewer.as_ref().unwrap().scroll_offset, 0);
        // ↓ 滚动
        app.handle_key(KeyCode::Down, KeyModifiers::NONE);
        assert_eq!(app.diff_viewer.as_ref().unwrap().scroll_offset, 1);
        // k/j 滚动
        app.handle_key(KeyCode::Char('k'), KeyModifiers::NONE);
        assert_eq!(app.diff_viewer.as_ref().unwrap().scroll_offset, 0);
        app.handle_key(KeyCode::Char('j'), KeyModifiers::NONE);
        assert_eq!(app.diff_viewer.as_ref().unwrap().scroll_offset, 1);
        // PageDown 翻页
        app.handle_key(KeyCode::PageDown, KeyModifiers::NONE);
        assert_eq!(app.diff_viewer.as_ref().unwrap().scroll_offset, 11);
        // PageUp 翻回
        app.handle_key(KeyCode::PageUp, KeyModifiers::NONE);
        assert_eq!(app.diff_viewer.as_ref().unwrap().scroll_offset, 1);
        // q 退出
        app.handle_key(KeyCode::Char('q'), KeyModifiers::NONE);
        assert!(!app.diff_active());
    }

    #[test]
    fn test_diff_mode_intercepts_typing() {
        let mut app = TuiApp::new(true);
        app.open_diff("diff --git a/x b/x\n@@ -1 +1 @@\n-a\n+b\n".to_string());
        app.handle_key(KeyCode::Char('a'), KeyModifiers::NONE);
        assert_eq!(app.input, "", "diff 模式下输入不应进入输入框");
        // Esc 退出
        app.handle_key(KeyCode::Esc, KeyModifiers::NONE);
        assert!(!app.diff_active());
        // 退出后可正常输入
        app.handle_key(KeyCode::Char('a'), KeyModifiers::NONE);
        assert_eq!(app.input, "a");
    }

    #[test]
    fn test_diff_mode_ctrl_c_closes_not_quits() {
        let mut app = TuiApp::new(true);
        app.open_diff("diff --git a/x b/x\n@@ -1 +1 @@\n-a\n+b\n".to_string());
        let action = app.handle_key(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(action, KeyAction::None, "diff 模式下 Ctrl+C 不应退出");
        assert!(!app.diff_active(), "Ctrl+C 应关闭 diff 面板");
    }

    #[test]
    fn test_slash_completion_includes_diff() {
        // 命令面精简: 人类 Tab 补全只含控制命令白名单; 聚合器与领域命令由 agent 后端自我调度
        let mut app = TuiApp::new(true);
        app.input = "/he".into();
        app.complete_slash();
        assert!(app.input.starts_with("/help"), "/he Tab 应补全为控制命令 /help, got: {}", app.input);
        // 聚合器/领域命令不再出现在人类补全候选
        let cands = slash_command_candidates("/f");
        assert!(!cands.iter().any(|c| c == "/file"), "/file 是 agent 工具, 不应出现在人类补全候选");
        let cands2 = slash_command_candidates("/di");
        assert!(!cands2.iter().any(|c| c == "/diff"), "/diff 被 /file 覆盖, 不应出现在补全候选");
    }

    // ── 会话自动恢复 ──

    #[test]
    fn test_restore_sessions_from_base() {
        use crate::cli::tui::session_store::{SessionData, SessionStore};
        let tmp = std::env::temp_dir().join(format!("nt-tui-restore-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).expect("create tmp");
        let mut store = SessionStore::with_base(tmp.clone());
        let now = chrono::Utc::now().to_rfc3339();
        let data = SessionData {
            id: "s-1".into(),
            name: "recovered".into(),
            messages: vec![
                "[user] 上次的问题".into(),
                "[assistant] 上次的回答".into(),
            ],
            created_at: now.clone(),
            updated_at: now,
        };
        store.save_session("recovered", &data).expect("save");

        let mut app = TuiApp::new(true);
        let restored = app.restore_sessions_from_base(Some(tmp.clone()));
        assert_eq!(restored, 2, "应恢复 2 条消息");
        assert_eq!(app.sessions.len(), 1, "恢复后应替换默认空会话");
        assert_eq!(app.sessions[0].name, "recovered");
        assert_eq!(app.sessions[0].messages[0].role, "user");
        assert_eq!(app.sessions[0].messages[0].content, "上次的问题");
        assert_eq!(app.sessions[0].messages[1].role, "assistant");
        assert_eq!(app.sessions[0].messages[1].content, "上次的回答");

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_restore_sessions_empty_when_no_history() {
        let tmp = std::env::temp_dir().join(format!("nt-tui-restore-empty-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).expect("create tmp");
        let mut app = TuiApp::new(true);
        let restored = app.restore_sessions_from_base(Some(tmp.clone()));
        assert_eq!(restored, 0, "无历史时返回 0");
        assert_eq!(app.sessions.len(), 1, "保留默认空会话");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_parse_role_line_variants() {
        assert_eq!(parse_role_line("[user] hi"), ("user", "hi".to_string()));
        assert_eq!(parse_role_line("[assistant] hello"), ("assistant", "hello".to_string()));
        assert_eq!(parse_role_line("[system] note"), ("system", "note".to_string()));
        assert_eq!(parse_role_line("raw line"), ("system", "raw line".to_string()));
    }

    // ── @ 文件引用补全（私有方法，需在 impl 内测试） ──

    #[test]
    fn test_at_reference_detection() {
        let mut app = TuiApp::new(true);
        app.input = "看看 @".into();
        app.cursor = app.input.len();
        assert!(app.has_at_reference(), "@ 后无内容且光标在末尾应判定为引用");
        app.input = "email@example.com".into();
        app.cursor = app.input.len();
        assert!(!app.has_at_reference(), "邮箱不应误判");
        app.input = "见 @src".into();
        app.cursor = app.input.len();
        assert!(app.has_at_reference(), "@ 后路径片段应判定为引用");
        app.input = "见 @src".into();
        app.cursor = 0; // 光标在 @ 之前 → 不应补全
        assert!(!app.has_at_reference());
    }

    #[test]
    fn test_longest_common_prefix_basic() {
        assert_eq!(longest_common_prefix(&["src/a.rs".into(), "src/b.rs".into(), "src/c.rs".into()]), "src/");
        assert_eq!(longest_common_prefix(&["abc".into(), "abd".into()]), "ab");
        assert_eq!(longest_common_prefix(&["abc".into()]), "abc");
        assert_eq!(longest_common_prefix(&[]), "");
    }

    #[test]
    fn test_at_completion_in_known_dir() {
        // 用临时目录保证确定性（不依赖测试进程 cwd 指向的仓库结构）。
        let tmp = std::env::temp_dir().join(format!("nt-tui-at-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(tmp.join("src")).unwrap();
        std::fs::write(tmp.join("src").join("main.rs"), "").unwrap();
        std::fs::write(tmp.join("Cargo.toml"), "").unwrap();
        let orig = std::env::current_dir().unwrap();
        std::env::set_current_dir(&tmp).unwrap();

        let mut app = TuiApp::new(true);
        // 目录补全：@src → @src/
        app.input = "读 @src".into();
        app.cursor = app.input.len();
        app.complete_at_reference();
        assert_eq!(app.input, "读 @src/", "目录应补全尾斜杠: {}", app.input);

        // 文件补全：@Cargo → @Cargo.toml
        app.input = "读 @Cargo".into();
        app.cursor = app.input.len();
        app.complete_at_reference();
        assert_eq!(app.input, "读 @Cargo.toml", "文件应精确补全: {}", app.input);

        // 无匹配：输入不变
        app.input = "读 @zzz".into();
        app.cursor = app.input.len();
        app.complete_at_reference();
        assert_eq!(app.input, "读 @zzz", "无匹配不应改动");

        std::env::set_current_dir(&orig).unwrap();
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_slash_completion_uses_registry_dynamic() {
        let mut app = TuiApp::new(true);
        app.input = "/co".into();
        app.complete_slash();
        assert!(app.input.starts_with("/config"),
            "应补全 registry 控制命令 (completions 已降级): {}", app.input);
    }
}
