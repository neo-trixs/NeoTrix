use super::types::{ask_side_llm, ChatMessage, GoalDisplay, Session, SideMessage};
use crate::cli::commands::{default_registry, CommandOutput, CommandRegistry};
use crate::cli::sandbox::{global_sandbox, CliSandboxMode};
use crate::cli::tui::diff_viewer::DiffViewer;
use crate::cli::tui::history::CommandHistory;
use crate::cli::tui::session_store::SessionStore;
use crate::cli::tui::themes::{self, Theme};
use crate::cli::tui::vim_mode::{VimAction, VimModeManager};
use crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain;
use base64::Engine;
use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers, MouseButton};
use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::timeout;

pub struct TuiApp {
    pub sessions: Vec<Session>,
    pub active_session: usize,
    pub input: String,
    pub command_history: CommandHistory,
    pub side_conversation: Vec<SideMessage>,
    pub scroll_offset: usize,
    pub status_text: String,
    pub running: bool,
    pub multi_line: bool,
    pub agent_busy: bool,
    pub token_count: u64,
    pub thinking_expanded: HashSet<String>,
    pub streaming: bool,
    pub streaming_role: String,
    pub streaming_text: String,
    pub tokens_per_sec: f64,
    pub goal_display: GoalDisplay,
    pub diff_viewer: Option<DiffViewer>,
    pub workspace_name: String,
    pub workspace_count: usize,
    pub vim_mode: VimModeManager,
    pub image_attachment: Option<(String, String)>,
    pub current_theme: Theme,
    pub sandbox_mode: CliSandboxMode,
    pub session_store: SessionStore,
    pub ephemeral: bool,
    pub completions: Vec<String>,
    pub selected_completion: usize,
    cmds: CommandRegistry,
}

impl TuiApp {
    pub fn new(ephemeral: bool) -> Self {
        let store = SessionStore::new();
        let sessions = if !ephemeral {
            let saved = store.list_full_sessions();
            if !saved.is_empty() {
                saved
            } else {
                vec![Session {
                    id: "s-1".to_string(),
                    name: "Default Session".to_string(),
                    messages: VecDeque::new(),
                }]
            }
        } else {
            vec![Session {
                id: "s-1".to_string(),
                name: "Default Session".to_string(),
                messages: VecDeque::new(),
            }]
        };
        Self {
            sessions,
            active_session: 0,
            input: String::new(),
            command_history: CommandHistory::new(),
            side_conversation: Vec::new(),
            scroll_offset: 0,
            status_text: "Ready | Provider: not configured".to_string(),
            running: true,
            multi_line: false,
            agent_busy: false,
            token_count: 0,
            thinking_expanded: HashSet::new(),
            streaming: false,
            streaming_role: String::new(),
            streaming_text: String::new(),
            tokens_per_sec: 0.0,
            goal_display: GoalDisplay::idle(),
            diff_viewer: None,
            workspace_name: {
                let mgr = crate::core::WORKSPACE_MANAGER
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());
                mgr.active()
                    .map(|w| w.name.clone())
                    .unwrap_or_else(|| "default".to_string())
            },
            workspace_count: {
                let mgr = crate::core::WORKSPACE_MANAGER
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());
                mgr.list().len()
            },
            vim_mode: VimModeManager::new(),
            image_attachment: None,
            current_theme: {
                let pref = themes::load_theme_pref();
                let name = pref.as_deref().unwrap_or("pitaya");
                themes::theme_by_name(name)
            },
            sandbox_mode: global_sandbox()
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .mode(),
            session_store: store,
            ephemeral,
            cmds: default_registry(),
            completions: vec![],
            selected_completion: 0,
        }
    }

    pub fn active_session_mut(&mut self) -> &mut Session {
        &mut self.sessions[self.active_session]
    }

    pub fn active_session(&self) -> &Session {
        &self.sessions[self.active_session]
    }

    pub fn push_message(&mut self, role: &str, content: String) {
        if role == "user" {
            if let Some((name, data)) = self.image_attachment.take() {
                self.active_session_mut()
                    .messages
                    .push_back(ChatMessage::with_image(
                        role,
                        content,
                        Some(data),
                        Some(name),
                    ));
                return;
            }
        }
        self.active_session_mut()
            .messages
            .push_back(ChatMessage::new(role, content));
    }

    pub fn toggle_thinking(&mut self) {
        let session_idx = self.active_session;
        for (msg_idx, msg) in self.sessions[session_idx].messages.iter().enumerate().rev() {
            if msg.role == "assistant" && !msg.thinking_blocks.is_empty() {
                let key = format!("{}:{}", session_idx, msg_idx);
                if self.thinking_expanded.contains(&key) {
                    self.thinking_expanded.remove(&key);
                } else {
                    self.thinking_expanded.insert(key);
                }
                return;
            }
        }
    }

    pub fn auto_save(&self) {
        if self.ephemeral {
            return;
        }
        let session = self.active_session();
        if let Err(e) = self.session_store.save_full_session(session) {
            log::error!("Auto-save failed: {}", e);
        }
    }

    /// Save session and distill conversation into KB evolution records.
    /// Call this when KB access is available (e.g., after SEAL iteration).
    pub fn save_and_distill(&self, kb: &crate::neotrix::nt_memory_kb::KnowledgeBase) {
        if self.ephemeral {
            return;
        }
        let session = self.active_session();
        if let Err(e) = self.session_store.save_full_session(session) {
            log::error!("Auto-save failed: {}", e);
            return;
        }
        let messages: Vec<(String, String)> = session
            .messages
            .iter()
            .map(|m| (m.role.clone(), m.content.clone()))
            .collect();
        if !messages.is_empty() {
            match kb.distill_session(&session.id, &session.name, &messages) {
                Ok(id) => log::info!("[distill] session {} → KB record {}", session.id, id),
                Err(e) => log::warn!("[distill] session {} failed: {}", session.id, e),
            }
        }
    }

    pub fn handle_fork(&mut self, text: &str) {
        let name = text.trim_start_matches("/fork").trim();
        let session = self.active_session().clone();
        let new_id = format!(
            "s-{}",
            uuid::Uuid::new_v4()
                .to_string()
                .split('-')
                .next()
                .unwrap_or("0")
        );
        let new_name = if name.is_empty() {
            format!("{} (fork)", session.name)
        } else {
            name.to_string()
        };
        let name_display = new_name.clone();
        let forked = Session {
            id: new_id,
            name: new_name,
            messages: session.messages.clone(),
        };
        self.sessions.push(forked);
        self.active_session = self.sessions.len() - 1;
        self.scroll_offset = 0;
        self.auto_save();
        self.status_text = format!("Forked: {}", name_display);
    }

    pub fn sync_goal_display(&mut self, brain: &SelfIteratingBrain) {
        if let Some(ref g) = brain.goal_loop.active_goal {
            self.goal_display = GoalDisplay {
                has_goal: true,
                id: g.id.clone(),
                description: g.description.chars().take(34).collect(),
                state_label: g.state.label().to_string(),
                state_icon: g.state.icon().to_string(),
                iterations: g.iterations_completed,
                max_iterations: g.config.max_iterations,
                score_before: g.score_before,
                score_current: g.score_current,
                stalled_count: g.stalled_count,
                queue_count: brain.goal_loop.goal_queue.len(),
                completed_count: brain.goal_loop.completed_goals.len(),
            };
        } else {
            self.goal_display = GoalDisplay {
                has_goal: false,
                queue_count: brain.goal_loop.goal_queue.len(),
                completed_count: brain.goal_loop.completed_goals.len(),
                ..GoalDisplay::idle()
            };
        }
    }

    pub fn stream_token(&mut self, role: &str, chunk: &str) {
        if !self.streaming {
            self.streaming = true;
            self.streaming_role = role.to_string();
            self.streaming_text.clear();
            self.agent_busy = true;
            self.status_text = format!("Generating... ({:.0} tok/s)", self.tokens_per_sec);
        }
        self.streaming_text.push_str(chunk);
        self.token_count += 1;
    }

    pub fn stream_finish(&mut self) {
        if self.streaming {
            let text = std::mem::take(&mut self.streaming_text);
            let role = std::mem::take(&mut self.streaming_role);
            if !text.is_empty() {
                self.push_message(&role, text);
            }
            self.streaming = false;
            self.agent_busy = false;
            self.status_text = "Ready".to_string();
        }
    }

    pub fn update_tokens_per_sec(&mut self, tokens: f64) {
        self.tokens_per_sec = tokens;
    }

    pub fn handle_key(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Option<String> {
        match key {
            KeyCode::Char('n') if modifiers == KeyModifiers::CONTROL => {
                let id = format!("s-{}", self.sessions.len() + 1);
                let name = format!("会话 {}", self.sessions.len() + 1);
                self.sessions.push(Session {
                    id,
                    name,
                    messages: VecDeque::new(),
                });
                self.active_session = self.sessions.len() - 1;
                self.auto_save();
            }
            KeyCode::Char('w') if modifiers == KeyModifiers::CONTROL && self.sessions.len() > 1 => {
                let removed_id = self.sessions[self.active_session].id.clone();
                self.sessions.remove(self.active_session);
                if self.active_session >= self.sessions.len() {
                    self.active_session = self.sessions.len() - 1;
                }
                let _ = self.session_store.delete_full_session(&removed_id);
            }
            KeyCode::Char('l') if modifiers == KeyModifiers::CONTROL => {
                let session = self.active_session_mut();
                session.messages.clear();
            }
            KeyCode::Char('s') if modifiers == KeyModifiers::CONTROL => {
                self.status_text = "Saving...".to_string();
            }
            KeyCode::Char('t') if modifiers == KeyModifiers::NONE => {
                self.toggle_thinking();
                self.status_text = "Thinking toggled".to_string();
            }
            KeyCode::Char('d') if modifiers == KeyModifiers::CONTROL => {
                self.running = false;
            }
            KeyCode::Char('p') if modifiers == KeyModifiers::CONTROL => {
                self.status_text = "Switch Provider... | not yet implemented".to_string();
            }
            KeyCode::Char('r') if modifiers == KeyModifiers::CONTROL => {
                if self.command_history.search_active {
                    self.command_history.cycle_search();
                    if let Some(&idx) = self
                        .command_history
                        .search_results
                        .get(self.command_history.search_selection)
                    {
                        self.input = self.command_history.entries[idx].clone();
                    }
                } else {
                    self.command_history.start_search();
                }
            }
            KeyCode::Char(c) if modifiers == KeyModifiers::NONE => {
                if self.command_history.search_active {
                    self.command_history.search_query.push(c);
                    self.command_history.update_search_results();
                    if let Some(&idx) = self.command_history.search_results.get(0) {
                        self.input = self.command_history.entries[idx].clone();
                    }
                } else {
                    self.input.push(c);
                    self.completions.clear();
                }
            }
            KeyCode::Enter if modifiers == KeyModifiers::ALT => {
                self.input.push('\n');
                self.multi_line = true;
            }
            KeyCode::Enter => {
                if self.command_history.search_active {
                    if let Some(cmd) = self.command_history.select_search() {
                        self.input = cmd;
                    }
                    return None;
                }
                self.completions.clear();
                if self.multi_line {
                    self.input.push('\n');
                } else {
                    let trimmed = self.input.trim().to_string();
                    if !trimmed.is_empty() {
                        self.command_history.push(trimmed.clone());
                        self.input.clear();
                        self.multi_line = false;
                        return Some(trimmed);
                    }
                }
            }
            KeyCode::Backspace => {
                if self.command_history.search_active {
                    self.command_history.search_query.pop();
                    self.command_history.update_search_results();
                    if let Some(&idx) = self.command_history.search_results.get(0) {
                        self.input = self.command_history.entries[idx].clone();
                    } else {
                        self.input.clear();
                    }
                } else {
                    self.input.pop();
                    self.completions.clear();
                }
            }
            KeyCode::Up => {
                if let Some(cmd) = self.command_history.navigate_up() {
                    self.input = cmd;
                    self.completions.clear();
                }
            }
            KeyCode::Down => {
                if let Some(cmd) = self.command_history.navigate_down() {
                    self.input = cmd;
                    self.completions.clear();
                }
            }
            KeyCode::Esc => {
                if self.command_history.search_active {
                    self.command_history.cancel_search();
                } else {
                    self.running = false;
                }
            }
            KeyCode::Tab => {
                if !self.completions.is_empty() {
                    self.selected_completion =
                        (self.selected_completion + 1) % self.completions.len();
                    self.input = self.completions[self.selected_completion].clone();
                } else {
                    self.build_completions();
                    if !self.completions.is_empty() {
                        self.selected_completion = 0;
                        self.input = self.completions[0].clone();
                    }
                }
            }
            KeyCode::PageUp => {
                self.scroll_offset = self.scroll_offset.saturating_sub(10);
            }
            KeyCode::PageDown => {
                self.scroll_offset += 10;
            }
            KeyCode::Left => {
                if self.active_session > 0 {
                    self.active_session -= 1;
                }
                self.scroll_offset = 0;
            }
            KeyCode::Right => {
                if self.active_session + 1 < self.sessions.len() {
                    self.active_session += 1;
                }
                self.scroll_offset = 0;
            }
            _ => {}
        }
        None
    }

    fn build_completions(&mut self) {
        self.completions.clear();
        let trimmed = self.input.trim();
        if let Some(cmd_end) = trimmed.find(' ') {
            let cmd = &trimmed[..cmd_end];
            let after = trimmed[cmd_end + 1..].trim();
            match cmd {
                "/model" | "/provider" | "/llm" => {
                    let subs = ["list", "set", "current"];
                    for s in &subs {
                        if s.starts_with(after) {
                            self.completions.push(format!("{} {}", cmd, s));
                        }
                    }
                }
                "/approval" | "/approve" => {
                    let subs = [
                        "mode",
                        "status",
                        "list",
                        "approve",
                        "deny",
                        "approve-all",
                        "deny-all",
                    ];
                    for s in &subs {
                        if s.starts_with(after) {
                            self.completions.push(format!("{} {}", cmd, s));
                        }
                    }
                }
                "/session" => {
                    let subs = [
                        "list", "save", "load", "delete", "fork", "export", "import", "share",
                    ];
                    for s in &subs {
                        if s.starts_with(after) {
                            self.completions.push(format!("{} {}", cmd, s));
                        }
                    }
                }
                "/route" => {
                    let subs = ["status", "enable", "disable", "reset", "set", "classify"];
                    for s in &subs {
                        if s.starts_with(after) {
                            self.completions.push(format!("{} {}", cmd, s));
                        }
                    }
                }
                "/workspace" => {
                    let subs = ["create", "list", "switch", "delete", "rename", "status"];
                    for s in &subs {
                        if s.starts_with(after) {
                            self.completions.push(format!("{} {}", cmd, s));
                        }
                    }
                }
                "/mem" => {
                    let subs = [
                        "view",
                        "list",
                        "search",
                        "edit",
                        "tag",
                        "delete",
                        "pin",
                        "checkpoint",
                        "rollback",
                        "dream",
                        "stats",
                    ];
                    for s in &subs {
                        if s.starts_with(after) {
                            self.completions.push(format!("{} {}", cmd, s));
                        }
                    }
                }
                "/git" => {
                    let subs = ["status", "diff", "log", "worktree"];
                    for s in &subs {
                        if s.starts_with(after) {
                            self.completions.push(format!("{} {}", cmd, s));
                        }
                    }
                }
                "/cost" => {
                    let subs = ["detail", "budget", "reset"];
                    for s in &subs {
                        if s.starts_with(after) {
                            self.completions.push(format!("{} {}", cmd, s));
                        }
                    }
                }
                _ => {}
            }
        } else {
            let names = self.cmds.list();
            for name in &names {
                if name.starts_with(trimmed) && !trimmed.is_empty() {
                    self.completions.push(name.to_string());
                }
            }
        }
    }

    pub fn process_submitted_text(
        &mut self,
        text: &str,
        brain: Option<&Arc<RwLock<SelfIteratingBrain>>>,
    ) -> Result<CommandOutput, String> {
        if text.starts_with('/') {
            Ok(self.cmds.execute(text, brain))
        } else {
            Err(text.to_string())
        }
    }

    pub fn handle_theme_command(&mut self, text: &str, output: &CommandOutput) {
        if !text.starts_with("/theme") {
            return;
        }
        if let Some(ref json) = output.json {
            if let Some(theme_name) = json.get("theme").and_then(|v| v.as_str()) {
                let theme = themes::theme_by_name(theme_name);
                self.current_theme = theme;
                self.status_text = format!("Theme: {}", theme_name);
            }
            if json.get("action").and_then(|v| v.as_str()) == Some("save") {
                if let Some(json_theme) = json.get("theme").and_then(|v| v.as_str()) {
                    let _ = themes::save_theme_pref(json_theme);
                } else {
                    let _ = themes::save_theme_pref(&self.current_theme.name);
                }
                self.status_text = "Theme saved to config".to_string();
            }
        }
    }

    fn handle_vim_toggle(&mut self, text: &str) {
        let subcmd = text.trim_start_matches("/vim").trim();
        match subcmd {
            "on" => {
                self.vim_mode.set_enabled(true);
                self.status_text = "Vim: ON".to_string();
            }
            "off" => {
                self.vim_mode.set_enabled(false);
                self.status_text = "Vim: OFF".to_string();
            }
            "toggle" | "" => {
                self.vim_mode.toggle();
                self.status_text = if self.vim_mode.is_enabled() {
                    "Vim: ON".to_string()
                } else {
                    "Vim: OFF".to_string()
                };
            }
            _ => {
                self.push_message(
                    "system",
                    format!(
                        "Unknown /vim subcommand: {subcmd}. Available: toggle, on, off, status"
                    ),
                );
            }
        }
    }

    async fn handle_vim_command(
        &mut self,
        cmd: &str,
        agent: &Arc<RwLock<SelfIteratingBrain>>,
        terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    ) -> bool {
        let text = if cmd.starts_with(':') {
            let rest = cmd[1..].trim();
            if rest.is_empty() {
                return true;
            }
            if rest.starts_with('/') {
                rest.to_string()
            } else {
                format!("/{rest}")
            }
        } else {
            cmd.to_string()
        };
        if text.is_empty() {
            return true;
        }
        self.command_history.push(text.clone());
        match self.process_submitted_text(&text, Some(agent)) {
            Ok(output) => {
                self.handle_theme_command(&text, &output);
                self.push_message(
                    "system",
                    if output.success {
                        output.message
                    } else {
                        format!("Error: {}", output.message)
                    },
                );
            }
            Err(msg) => {
                self.agent_busy = true;
                let pending_image = self.image_attachment.take();
                let image_data = pending_image.as_ref().map(|(_, data)| data.clone());
                let image_name = pending_image.as_ref().map(|(name, _)| name.clone());
                let raw_image = image_data.as_ref().and_then(|s| {
                    s.split("base64,")
                        .nth(1)
                        .and_then(|b64| base64::engine::general_purpose::STANDARD.decode(b64).ok())
                });
                let mut a = agent.write().await;
                self.active_session_mut()
                    .messages
                    .push_back(ChatMessage::with_image(
                        "user",
                        msg.clone(),
                        image_data.clone(),
                        image_name,
                    ));
                if let Some(ref mut engine) = a.reasoning_engine {
                    {
                        let tracker = crate::cli::cost_tracker::COST_TRACKER
                            .lock()
                            .unwrap_or_else(|e| e.into_inner());
                        let session_cost = tracker.current_session_cost();
                        if let Some(warning) = tracker.check_budget_and_warn(session_cost) {
                            if warning.contains("🛑") {
                                self.push_message("system", warning.clone());
                                self.status_text = "Ready".to_string();
                                self.agent_busy = false;
                                return true;
                            }
                            self.push_message("system", warning);
                        }
                    }
                    match engine.reason_stream(&msg, raw_image).await {
                        Ok((_full_response, mut rx)) => {
                            self.agent_busy = true;
                            self.status_text = "Generating...".to_string();
                            let stream_start = std::time::Instant::now();
                            let mut token_count: u64 = 0;
                            loop {
                                match timeout(Duration::from_secs(1), rx.recv()).await {
                                    Ok(Some(token)) => {
                                        self.stream_token("assistant", &token);
                                        token_count += 1;
                                        let elapsed =
                                            stream_start.elapsed().as_secs_f64().max(0.001);
                                        self.update_tokens_per_sec(token_count as f64 / elapsed);
                                        let _ = terminal.draw(|frame| {
                                            let area = frame.area();
                                            let (
                                                left,
                                                chat_area,
                                                goal_area,
                                                input_area,
                                                status_area,
                                            ) = crate::cli::tui::layout::compute_layout(area);
                                            crate::cli::tui::layout::render_session_list(
                                                frame,
                                                left,
                                                self,
                                                &self.current_theme,
                                            );
                                            if self.diff_viewer.is_some() {
                                                crate::cli::tui::layout::render_diff_viewer(
                                                    frame,
                                                    chat_area,
                                                    self,
                                                    &self.current_theme,
                                                );
                                            } else {
                                                crate::cli::tui::layout::render_chat_panel(
                                                    frame,
                                                    chat_area,
                                                    self,
                                                    &self.current_theme,
                                                );
                                            }
                                            crate::cli::tui::layout::render_goal_panel(
                                                frame,
                                                goal_area,
                                                self,
                                                "",
                                                &self.current_theme,
                                            );
                                            crate::cli::tui::layout::render_input_panel(
                                                frame,
                                                input_area,
                                                self,
                                                &self.current_theme,
                                            );
                                            crate::cli::tui::layout::render_status_bar(
                                                frame,
                                                status_area,
                                                self,
                                                &self.current_theme,
                                            );
                                        });
                                    }
                                    Ok(None) => break,
                                    Err(_) => continue,
                                }
                            }
                            self.stream_finish();
                            if let Some(ref kb) = a._nt_memory_kb {
                                self.save_and_distill(kb);
                            } else {
                                self.auto_save();
                            }
                        }
                        Err(e) => {
                            self.push_message("system", format!("Reasoning error: {e}"));
                        }
                    }
                    let _ = a.brain.save();
                } else {
                    let task_type = crate::neotrix::nt_expert_routing::TaskType::General;
                    let result = a.iterate(task_type);
                    self.push_message(
                        "assistant",
                        format!(
                            "Evolved: {:.3} → {:.3}",
                            result.score_before, result.score_after
                        ),
                    );
                    if let Some(ref kb) = a._nt_memory_kb {
                        self.save_and_distill(kb);
                    } else {
                        self.auto_save();
                    }
                }
                self.agent_busy = false;
            }
        }
        self.status_text = "Ready".to_string();
        true
    }

    async fn handle_side_question(&mut self, text: &str, _agent: &Arc<RwLock<SelfIteratingBrain>>) {
        let rest = text.trim().trim_start_matches("/side").trim();
        if rest == "clear" {
            self.side_conversation.clear();
            self.push_message("system", "[Side] Conversation cleared".to_string());
            return;
        }
        if rest.is_empty() {
            self.push_message("system",
                "[Side] Usage:\n  /side <question>    Quick question (non-interfering)\n  /side clear         Clear side conversation".to_string());
            return;
        }
        self.status_text = "Side Q 回答中...".to_string();
        let start = Instant::now();
        match ask_side_llm(rest).await {
            Ok(answer) => {
                let duration = start.elapsed();
                self.side_conversation.push(SideMessage {
                    question: rest.to_string(),
                    answer: answer.clone(),
                    duration,
                    timestamp: start,
                });
                let msg = format!(
                    "[Side] Q: {}\n[Side] A: {}\n[Side] Side Q answered in {:.1}s",
                    rest,
                    answer,
                    duration.as_secs_f64()
                );
                self.push_message("system", msg);
            }
            Err(e) => {
                self.push_message("system", format!("[Side] Error: {}", e));
            }
        }
    }

    pub async fn run(&mut self, agent: Arc<RwLock<SelfIteratingBrain>>) {
        let loaded = CommandHistory::load_or_new();
        self.command_history.entries = loaded.entries;

        use crossterm::event::{DisableMouseCapture, EnableMouseCapture, MouseEventKind};
        use crossterm::execute;
        use crossterm::terminal::{
            disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
        };
        use ratatui::{backend::CrosstermBackend, Terminal};
        use std::io::IsTerminal;

        let mut stdout = std::io::stdout();
        if stdout.is_terminal() {
            let _ = enable_raw_mode().unwrap_or_else(|e| {
                log::warn!("[tui] failed to enable raw mode: {e}");
            });
            let _ =
                execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap_or_else(|e| {
                    log::warn!("[tui] failed to enter alternate screen: {e}");
                });
        } else {
            log::warn!("[tui] not a terminal, skipping raw mode");
        }

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = match Terminal::new(backend) {
            Ok(t) => t,
            Err(e) => {
                log::error!("[tui] failed to create terminal: {e}");
                return;
            }
        };

        while self.running {
            {
                let a = agent.read().await;
                self.sync_goal_display(&a);
            }
            {
                let mgr = crate::core::WORKSPACE_MANAGER
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());
                self.workspace_name = mgr
                    .active()
                    .map(|w| w.name.clone())
                    .unwrap_or_else(|| "default".to_string());
                self.workspace_count = mgr.list().len();
            }
            terminal
                .draw(|frame| {
                    let area = frame.area();
                    let (left, chat_area, goal_area, input_area, status_area) =
                        crate::cli::tui::layout::compute_layout(area);
                    crate::cli::tui::layout::render_session_list(
                        frame,
                        left,
                        self,
                        &self.current_theme,
                    );
                    if self.diff_viewer.is_some() {
                        crate::cli::tui::layout::render_diff_viewer(
                            frame,
                            chat_area,
                            self,
                            &self.current_theme,
                        );
                    } else {
                        crate::cli::tui::layout::render_chat_panel(
                            frame,
                            chat_area,
                            self,
                            &self.current_theme,
                        );
                    }
                    crate::cli::tui::layout::render_goal_panel(
                        frame,
                        goal_area,
                        self,
                        "",
                        &self.current_theme,
                    );
                    crate::cli::tui::layout::render_input_panel(
                        frame,
                        input_area,
                        self,
                        &self.current_theme,
                    );
                    crate::cli::tui::layout::render_status_bar(
                        frame,
                        status_area,
                        self,
                        &self.current_theme,
                    );
                })
                .expect("Failed to draw");

            if !crossterm::event::poll(std::time::Duration::from_millis(50)).unwrap_or(false) {
                continue;
            }

            match crossterm::event::read() {
                Ok(crossterm::event::Event::Key(key_event)) => {
                    if key_event.kind == KeyEventKind::Press {
                        if key_event.code == KeyCode::Char('s')
                            && key_event.modifiers == KeyModifiers::CONTROL
                        {
                            let a = agent.read().await;
                            match a.brain.save() {
                                Ok(()) => self.status_text = "💾 Saved".to_string(),
                                Err(e) => self.status_text = format!("Save failed: {}", e),
                            }
                            continue;
                        }
                        if key_event.code == KeyCode::Char('p')
                            && key_event.modifiers == KeyModifiers::CONTROL
                        {
                            self.status_text =
                                "Switch Provider: type provider name and press Enter".to_string();
                            self.input = "/provider ".to_string();
                            continue;
                        }

                        if self.diff_viewer.is_some() {
                            match key_event.code {
                                KeyCode::Char('q') | KeyCode::Esc => {
                                    self.diff_viewer = None;
                                }
                                KeyCode::Char('j') | KeyCode::Down => {
                                    if let Some(dv) = self.diff_viewer.as_mut() {
                                        dv.scroll_down(1);
                                    }
                                }
                                KeyCode::Char('k') | KeyCode::Up => {
                                    if let Some(dv) = self.diff_viewer.as_mut() {
                                        dv.scroll_up(1);
                                    }
                                }
                                KeyCode::PageDown => {
                                    if let Some(dv) = self.diff_viewer.as_mut() {
                                        dv.scroll_down(20);
                                    }
                                }
                                KeyCode::PageUp => {
                                    if let Some(dv) = self.diff_viewer.as_mut() {
                                        dv.scroll_up(20);
                                    }
                                }
                                KeyCode::Home => {
                                    if let Some(dv) = self.diff_viewer.as_mut() {
                                        dv.scroll_offset = 0;
                                    }
                                }
                                _ => {}
                            }
                            continue;
                        }

                        // Vim mode handling
                        if self.vim_mode.is_enabled() {
                            let action = self
                                .vim_mode
                                .handle_key(key_event.code, key_event.modifiers);
                            let vim_submitted = match &action {
                                VimAction::PassThrough => false,
                                VimAction::Search(cmd) => {
                                    self.handle_vim_command(cmd, &agent, &mut terminal).await
                                }
                                VimAction::Quit => {
                                    self.running = false;
                                    true
                                }
                                VimAction::SwitchSession(n) => {
                                    if *n == usize::MAX {
                                        if self.active_session > 0 {
                                            self.active_session -= 1;
                                        }
                                    } else if *n < self.sessions.len() {
                                        self.active_session = *n;
                                    }
                                    self.scroll_offset = 0;
                                    true
                                }
                                VimAction::MoveDown => {
                                    self.scroll_offset += 1;
                                    true
                                }
                                VimAction::MoveUp => {
                                    self.scroll_offset = self.scroll_offset.saturating_sub(1);
                                    true
                                }
                                VimAction::MovePageDown => {
                                    self.scroll_offset += 10;
                                    true
                                }
                                VimAction::MovePageUp => {
                                    self.scroll_offset = self.scroll_offset.saturating_sub(10);
                                    true
                                }
                                VimAction::MoveLineStart => {
                                    self.scroll_offset = 0;
                                    true
                                }
                                VimAction::MoveLineEnd => {
                                    self.scroll_offset = usize::MAX;
                                    true
                                }
                                VimAction::EnterInsertMode
                                | VimAction::EnterNormalMode
                                | VimAction::EnterVisualMode
                                | VimAction::None
                                | VimAction::DeleteChar
                                | VimAction::InsertChar(_)
                                | VimAction::Yank
                                | VimAction::Paste
                                | VimAction::Undo
                                | VimAction::MoveLeft
                                | VimAction::MoveRight
                                | VimAction::MoveWordForward
                                | VimAction::MoveWordBack => true,
                            };
                            if vim_submitted {
                                continue;
                            }
                        }

                        if let Some(text) = self.handle_key(key_event.code, key_event.modifiers) {
                            let trimmed = text.trim();
                            if trimmed.starts_with("/fork") {
                                let fork_name = trimmed.trim_start_matches("/fork").trim();
                                if !fork_name.is_empty() && fork_name.contains(' ') {
                                    self.push_message("system", "用法: /fork [name]".to_string());
                                } else {
                                    self.handle_fork(trimmed);
                                }
                                self.status_text = "Ready".to_string();
                                continue;
                            }
                            if trimmed == "/side" || trimmed.starts_with("/side ") {
                                self.handle_side_question(trimmed, &agent).await;
                                self.status_text = "Ready".to_string();
                                continue;
                            }
                            if trimmed == "/vim" || trimmed.starts_with("/vim ") {
                                self.handle_vim_toggle(trimmed);
                                continue;
                            }
                            if trimmed.starts_with("/image ") {
                                let path = trimmed.trim_start_matches("/image ").trim();
                                match load_image_attachment(path) {
                                    Ok((name, data)) => {
                                        self.image_attachment = Some((name.clone(), data));
                                        self.push_message(
                                            "system",
                                            format!("已加载图片: {}", name),
                                        );
                                        self.status_text = format!("图片已加载: {}", name);
                                    }
                                    Err(e) => {
                                        self.push_message("system", format!("图片加载失败: {}", e));
                                        self.status_text = "图片加载失败".to_string();
                                    }
                                }
                                continue;
                            }
                            if !trimmed.is_empty()
                                && is_image_path(trimmed)
                                && std::path::Path::new(trimmed).exists()
                            {
                                match load_image_attachment(trimmed) {
                                    Ok((name, data)) => {
                                        self.image_attachment = Some((name.clone(), data));
                                        self.status_text = format!("图片已加载: {}", name);
                                    }
                                    Err(e) => {
                                        self.push_message("system", format!("图片加载失败: {}", e));
                                        continue;
                                    }
                                }
                            }

                            self.status_text = "思考中...".to_string();

                            match self.process_submitted_text(&text, Some(&agent)) {
                                Ok(output) => {
                                    self.handle_theme_command(&text, &output);
                                    let msg = output.message.clone();
                                    if text.starts_with("/diff") && output.success {
                                        let diff_content = if let Some(pos) = msg.find("diff --git")
                                        {
                                            msg[pos..].to_string()
                                        } else {
                                            msg.clone()
                                        };
                                        let viewer = DiffViewer::new(diff_content);
                                        if !viewer.is_empty() {
                                            self.diff_viewer = Some(viewer);
                                        }
                                    }
                                    self.push_message(
                                        "system",
                                        if output.success {
                                            msg
                                        } else {
                                            format!("错误: {}", msg)
                                        },
                                    );
                                }
                                Err(msg) => {
                                    self.agent_busy = true;
                                    let pending_image = self.image_attachment.take();
                                    let image_data =
                                        pending_image.as_ref().map(|(_, data)| data.clone());
                                    let image_name =
                                        pending_image.as_ref().map(|(name, _)| name.clone());
                                    let raw_image = image_data.as_ref().and_then(|s| {
                                        s.split("base64,").nth(1).and_then(|b64| {
                                            base64::engine::general_purpose::STANDARD
                                                .decode(b64)
                                                .ok()
                                        })
                                    });
                                    let mut a = agent.write().await;
                                    self.active_session_mut().messages.push_back(
                                        ChatMessage::with_image(
                                            "user",
                                            msg.clone(),
                                            image_data.clone(),
                                            image_name,
                                        ),
                                    );
                                    if let Some(ref mut engine) = a.reasoning_engine {
                                        // Budget check before LLM call
                                        {
                                            let tracker = crate::cli::cost_tracker::COST_TRACKER
                                                .lock()
                                                .unwrap_or_else(|e| e.into_inner());
                                            let session_cost = tracker.current_session_cost();
                                            if let Some(warning) =
                                                tracker.check_budget_and_warn(session_cost)
                                            {
                                                if warning.contains("🛑") {
                                                    self.push_message("system", warning.clone());
                                                    self.status_text = "Ready".to_string();
                                                    self.agent_busy = false;
                                                    continue;
                                                }
                                                self.push_message("system", warning);
                                            }
                                        }
                                        match engine.reason_stream(&msg, raw_image).await {
                                            Ok((_full_response, mut rx)) => {
                                                self.agent_busy = true;
                                                self.status_text = "Generating...".to_string();
                                                let stream_start = std::time::Instant::now();
                                                let mut token_count: u64 = 0;
                                                loop {
                                                    match timeout(Duration::from_secs(1), rx.recv())
                                                        .await
                                                    {
                                                        Ok(Some(token)) => {
                                                            self.stream_token("assistant", &token);
                                                            token_count += 1;
                                                            let elapsed = stream_start
                                                                .elapsed()
                                                                .as_secs_f64()
                                                                .max(0.001);
                                                            self.update_tokens_per_sec(
                                                                token_count as f64 / elapsed,
                                                            );
                                                            let _ = terminal.draw(|frame| {
                                                                let area = frame.area();
                                                                let (left, chat_area, goal_area, input_area, status_area) = crate::cli::tui::layout::compute_layout(area);
                                                                crate::cli::tui::layout::render_session_list(frame, left, self, &self.current_theme);
                                                                if self.diff_viewer.is_some() {
                                                                    crate::cli::tui::layout::render_diff_viewer(frame, chat_area, self, &self.current_theme);
                                                                } else {
                                                                    crate::cli::tui::layout::render_chat_panel(frame, chat_area, self, &self.current_theme);
                                                                }
                                                                crate::cli::tui::layout::render_goal_panel(frame, goal_area, self, "", &self.current_theme);
                                                                crate::cli::tui::layout::render_input_panel(frame, input_area, self, &self.current_theme);
                                                                crate::cli::tui::layout::render_status_bar(frame, status_area, self, &self.current_theme);
                                                            });
                                                        }
                                                        Ok(None) => break,
                                                        Err(_) => continue,
                                                    }
                                                }
                                                self.stream_finish();
                                                if let Some(ref kb) = a._nt_memory_kb {
                                                    self.save_and_distill(kb);
                                                } else {
                                                    self.auto_save();
                                                }
                                            }
                                            Err(e) => {
                                                self.push_message(
                                                    "system",
                                                    format!("Reasoning error: {}", e),
                                                );
                                            }
                                        }
                                        let _ = a.brain.save();
                                    } else {
                                        let task_type =
                                            crate::neotrix::nt_expert_routing::TaskType::General;
                                        let result = a.iterate(task_type);
                                        self.push_message(
                                            "assistant",
                                            format!(
                                                "Evolved: {:.3} → {:.3}",
                                                result.score_before, result.score_after
                                            ),
                                        );
                                        if let Some(ref kb) = a._nt_memory_kb {
                                            self.save_and_distill(kb);
                                        } else {
                                            self.auto_save();
                                        }
                                    }
                                    self.agent_busy = false;
                                }
                            }

                            self.status_text = "Ready".to_string();
                        }
                    }
                }
                Ok(crossterm::event::Event::Mouse(mouse_event)) => {
                    if mouse_event.kind == MouseEventKind::ScrollDown {
                        self.scroll_offset += 5;
                    } else if mouse_event.kind == MouseEventKind::ScrollUp {
                        self.scroll_offset = self.scroll_offset.saturating_sub(5);
                    } else if mouse_event.kind == MouseEventKind::Down(MouseButton::Left) {
                        if mouse_event.column < 5 {
                            self.toggle_thinking();
                        }
                    }
                }
                Ok(_) => {}
                Err(_) => continue,
            }
        }

        if std::io::stdout().is_terminal() {
            let _ = disable_raw_mode().unwrap_or_else(|e| {
                log::warn!("[tui] failed to disable raw mode: {e}");
            });
            let _ = execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture)
                .unwrap_or_else(|e| {
                    log::warn!("[tui] failed to leave alternate screen: {e}");
                });
        }
        let a = agent.write().await;
        let _ = a.brain.save();
        log::info!("\nExiting...");
    }
}

fn is_image_path(s: &str) -> bool {
    let lower = s.to_lowercase();
    lower.ends_with(".png")
        || lower.ends_with(".jpg")
        || lower.ends_with(".jpeg")
        || lower.ends_with(".gif")
        || lower.ends_with(".webp")
}

fn load_image_attachment(path: &str) -> Result<(String, String), String> {
    let p = std::path::Path::new(path);
    let name = p
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("image")
        .to_string();
    let data = std::fs::read(p).map_err(|e| format!("读取图片失败: {}", e))?;
    let ext = p
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png")
        .to_lowercase();
    let mime = match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        _ => "image/png",
    };
    let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
    let data_url = format!("data:{};base64,{}", mime, b64);
    Ok((name, data_url))
}

impl Default for TuiApp {
    fn default() -> Self {
        Self::new(false)
    }
}
