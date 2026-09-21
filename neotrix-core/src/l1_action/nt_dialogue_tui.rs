//! # nt_dialogue_tui — 对话终端的 TUI 形态
//!
//! 借鉴映射（Claude Code + opencode，2026）：
//!
//! | 来源 | 模式 | 本终端映射 |
//! |---|---|---|
//! | Claude | 底栏 status line（模型/上下文%/花费） | 底栏：轮次·内需数·置信·模型·键位 |
//! | Claude | Ctrl+O 详情显隐 | Ctrl+O 切换需求详情窗 |
//! | Claude | Esc 结束/中断；空回行为 | Esc/空回/Ctrl+C/D = 结束本轮输入（对齐 Stalled 语义） |
//! | Claude | Ctrl+S 暂存，Up/Down 历史 | 同样：输入暂存 + 跨轮历史 |
//! | Claude | `?` 空输入帮助，`/` 命令 | `?` 覆盖层 + `/help` 行命令 |
//! | Claude | Enter 提交 / Ctrl+J 换行 | 同样（v1 单行输入，Ctrl+J 预留） |
//! | opencode | 右侧 sidebar（>120 列自动开，折叠区） | 右栏：融合结论/内需单/池状态，窄屏自动收 |
//! | opencode | PermissionPrompt 行内确认 | ok/no 行协议即确认框的文本形态 |
//! | opencode | tool 行 pending/complete/failed | 需求单行 kind 标签 + 计数 |
//!
//! ## v1 边界（诚实声明）
//! - `NtHumanChannel::prompt` 的阻塞语义不变：TUI 只替换"打印+读行"，
//!   循环/融合/池子一行不用动；终端建不起（管道/CI）自动回退 stdin 行式。
//! - 真事件驱动（流式答案、执行中 Esc 中断、模型切换面板）要拆消息通道，
//!   列入 v2，不在本文件撒谎。
//!
//! # Safety
//! - ratatui + crossterm 标准全屏流程，`Drop` 守卫必还终端，无 unsafe (R-P1)。
//! - 生产代码无 `unwrap/expect/panic`。

use crate::l1_action::nt_stdin_human::NtStdinHuman;
use crate::neotrix::nt_crystal_core::{NtDemand, NtHumanChannel, NtHumanReply};
use std::sync::Mutex;

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Terminal,
};

// ============================================================================
// 状态（纯数据 + 纯状态机，可测）
// ============================================================================

/// TUI 会话状态。
pub struct NtTuiState {
    pub goal: String,
    pub fused_line: String,
    pub transcript: Vec<String>,
    pub demands: Vec<NtDemand>,
    pub pool_line: String,
    pub round_no: usize,
    pub input: String,
    pub history: Vec<String>,
    pub hist_idx: Option<usize>,
    pub stash: Option<String>,
    pub show_sidebar: bool,
    pub show_details: bool,
    pub show_help: bool,
    /// 本轮已收集的回复行（`parse_lines` 的输入）。
    pub collected: Vec<String>,
}

impl NtTuiState {
    pub fn new(
        window: &str,
        demands: &[NtDemand],
        pool_line: String,
        round_no: usize,
        history: Vec<String>,
    ) -> Self {
        let lines: Vec<String> = window.lines().map(|l| l.to_string()).collect();
        Self {
            goal: extract_line(window, "目标："),
            fused_line: extract_line(window, "融合结论"),
            transcript: lines,
            demands: demands.to_vec(),
            pool_line,
            round_no,
            input: String::new(),
            history,
            hist_idx: None,
            stash: None,
            show_sidebar: true,
            show_details: false,
            show_help: false,
            collected: Vec::new(),
        }
    }

    /// 底栏 status line（Claude 式）：轮次·内需·置信·模型·键位。
    pub fn status_line(&self) -> String {
        format!(
            "第{}轮 · 内需{} · {} · {} · Enter提交 空回结束 Esc结束 Ctrl+O详情 F2侧栏 ?帮助",
            self.round_no,
            self.demands.len(),
            if self.fused_line.is_empty() {
                "暂无结论".to_string()
            } else {
                self.fused_line.clone()
            },
            if self.pool_line.is_empty() {
                "模型：未指定".to_string()
            } else {
                self.pool_line.clone()
            }
        )
    }

    /// 侧栏行（opencode 式折叠区 meanings）：结论 / 内需单 / 池。
    pub fn sidebar_lines(&self) -> Vec<String> {
        let mut out = vec![format!(
            "融合：{}",
            if self.fused_line.is_empty() {
                "（暂无）".to_string()
            } else {
                self.fused_line.clone()
            }
        )];
        out.push(format!("内需（{}）：", self.demands.len()));
        for d in &self.demands {
            if self.show_details {
                out.push(format!("[{}] {}：{}", d.id, d.kind.label(), d.text));
            } else {
                out.push(format!("[{}] {}", d.id, d.kind.label()));
            }
        }
        out.push(format!("池：{}", self.pool_line));
        out
    }
}

/// 从窗口文本按前缀取行（render() 格式的逆操作）。
pub fn extract_line(window: &str, prefix: &str) -> String {
    window
        .lines()
        .find(|l| l.trim_start().starts_with(prefix))
        .map(|l| l.trim().to_string())
        .unwrap_or_default()
}

/// 按键处理结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuiKeyOutcome {
    Continue,
    /// 结束本轮输入（Esc/空回/Ctrl+C/D/`.` 行）。
    Finish,
}

/// 纯状态机：按键 → 变异 state，返回是否结束本轮。
/// 键位（Claude/opencode 对齐）：Enter 提交行；空回结束；Esc 清空→再按结束；
/// Up/Down 历史；Ctrl+S 暂存；Ctrl+O 详情；F2 侧栏；`?` 空输入帮助。
pub fn apply_key(state: &mut NtTuiState, code: KeyCode, mods: KeyModifiers) -> TuiKeyOutcome {
    let ctrl = mods.contains(KeyModifiers::CONTROL);
    match (code, ctrl) {
        (KeyCode::Esc, _) => {
            if state.input.trim().is_empty() {
                return TuiKeyOutcome::Finish;
            }
            // 有草稿：进历史后清空（Claude 双 Esc 语义的一半）
            state.history.push(state.input.clone());
            state.input.clear();
            state.hist_idx = None;
            TuiKeyOutcome::Continue
        }
        (KeyCode::Char('c'), true) | (KeyCode::Char('d'), true) => TuiKeyOutcome::Finish,
        (KeyCode::Char('o'), true) => {
            state.show_details = !state.show_details;
            TuiKeyOutcome::Continue
        }
        (KeyCode::Char('s'), true) => {
            if state.input.trim().is_empty() {
                if let Some(stashed) = state.stash.take() {
                    state.input = stashed;
                }
            } else {
                state.stash = Some(state.input.clone());
                state.input.clear();
            }
            state.hist_idx = None;
            TuiKeyOutcome::Continue
        }
        (KeyCode::F(2), _) => {
            state.show_sidebar = !state.show_sidebar;
            TuiKeyOutcome::Continue
        }
        (KeyCode::Up, _) => {
            if !state.history.is_empty() {
                let next = match state.hist_idx {
                    None => state.history.len() - 1,
                    Some(0) => 0,
                    Some(i) => i - 1,
                };
                state.hist_idx = Some(next);
                state.input = state.history[next].clone();
            }
            TuiKeyOutcome::Continue
        }
        (KeyCode::Down, _) => {
            match state.hist_idx {
                None => {}
                Some(i) if i + 1 >= state.history.len() => {
                    state.hist_idx = None;
                    state.input.clear();
                }
                Some(i) => {
                    state.hist_idx = Some(i + 1);
                    state.input = state.history[i + 1].clone();
                }
            }
            TuiKeyOutcome::Continue
        }
        (KeyCode::Enter, _) => {
            let line = state.input.clone();
            if line.trim().is_empty() {
                return TuiKeyOutcome::Finish;
            }
            if line.trim() == "?" || line.trim() == "/help" {
                state.show_help = !state.show_help;
                state.input.clear();
                state.hist_idx = None;
                return TuiKeyOutcome::Continue;
            }
            if line.trim() == "." {
                return TuiKeyOutcome::Finish;
            }
            state.history.push(line.clone());
            state.collected.push(line);
            state.input.clear();
            state.hist_idx = None;
            TuiKeyOutcome::Continue
        }
        (KeyCode::Backspace, _) => {
            state.input.pop();
            TuiKeyOutcome::Continue
        }
        (KeyCode::Char('?'), _) if state.input.is_empty() => {
            state.show_help = !state.show_help;
            TuiKeyOutcome::Continue
        }
        (KeyCode::Char(c), false) => {
            state.input.push(c);
            state.hist_idx = None;
            TuiKeyOutcome::Continue
        }
        _ => TuiKeyOutcome::Continue,
    }
}

// ============================================================================
// 终端人类通道（TUI 形态）
// ============================================================================

/// TUI 人类通道：与 stdin 行式同协议（`NtStdinHuman::parse_lines`），只换皮肤。
pub struct NtTuiHuman {
    pool_line: String,
    history: Mutex<Vec<String>>,
    calls: Mutex<usize>,
}

impl NtTuiHuman {
    pub fn new(pool_line: impl Into<String>) -> Self {
        Self {
            pool_line: pool_line.into(),
            history: Mutex::new(Vec::new()),
            calls: Mutex::new(0),
        }
    }
}

impl Default for NtTuiHuman {
    fn default() -> Self {
        Self::new(String::new())
    }
}

/// 终端守卫：Drop 时必还终端（raw mode + 主屏）。
struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(std::io::stdout(), LeaveAlternateScreen);
    }
}

impl NtHumanChannel for NtTuiHuman {
    fn prompt(&self, window: &str, demands: &[NtDemand]) -> Vec<NtHumanReply> {
        let round_no = match self.calls.lock() {
            Ok(mut c) => {
                *c += 1;
                *c
            }
            Err(_) => 1,
        };
        let history = match self.history.lock() {
            Ok(h) => h.clone(),
            Err(_) => Vec::new(),
        };
        let mut state = NtTuiState::new(window, demands, self.pool_line.clone(), round_no, history);

        // 建终端；任何一步失败 → 回退 stdin 行式（管道/CI 安全）。
        let build: std::io::Result<Terminal<CrosstermBackend<std::io::Stdout>>> = (|| {
            enable_raw_mode()?;
            let mut stdout = std::io::stdout();
            execute!(stdout, EnterAlternateScreen)?;
            Terminal::new(CrosstermBackend::new(stdout))
        })();
        let mut terminal = match build {
            Ok(t) => t,
            Err(_) => {
                let _ = disable_raw_mode();
                return NtStdinHuman::new().prompt(window, demands);
            }
        };
        let _guard = TerminalGuard;

        loop {
            draw(&mut terminal, &state);
            let ev = match event::read() {
                Ok(e) => e,
                Err(_) => break,
            };
            if let Event::Key(k) = ev {
                if apply_key(&mut state, k.code, k.modifiers) == TuiKeyOutcome::Finish {
                    break;
                }
            }
        }

        if let Ok(mut h) = self.history.lock() {
            *h = state.history.clone();
        }
        NtStdinHuman::parse_lines(&state.collected)
    }
}

fn draw(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, state: &NtTuiState) {
    let _ = terminal.draw(|f| {
        let area = f.area();
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),
                Constraint::Length(3),
                Constraint::Length(1),
            ])
            .split(area);

        let wide = area.width >= 100;
        if state.show_sidebar && wide {
            let cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(68), Constraint::Percentage(32)])
                .split(rows[0]);
            render_transcript(f, cols[0], state);
            render_sidebar(f, cols[1], state);
        } else {
            render_transcript(f, rows[0], state);
        }
        render_input(f, rows[1], state);
        render_status(f, rows[2], state);

        if state.show_help {
            render_help(f, area);
        }
    });
}

fn render_transcript(
    f: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    state: &NtTuiState,
) {
    let items: Vec<ListItem> = state
        .transcript
        .iter()
        .map(|l| ListItem::new(Line::from(l.clone())))
        .collect();
    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" 实录 · {}", state.goal)),
    );
    f.render_widget(list, area);
}

fn render_sidebar(f: &mut ratatui::Frame, area: ratatui::layout::Rect, state: &NtTuiState) {
    let items: Vec<ListItem> = state
        .sidebar_lines()
        .into_iter()
        .map(|l| ListItem::new(Line::from(l)))
        .collect();
    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" 内需/结论/池 "),
    );
    f.render_widget(list, area);
}

fn render_input(f: &mut ratatui::Frame, area: ratatui::layout::Rect, state: &NtTuiState) {
    let hint = if state.input.is_empty() {
        Span::styled(
            "<id>: 文字批准 · <id>! 文字驳回 · ok/no <id> · 空回结束",
            Style::default().fg(Color::DarkGray),
        )
    } else {
        Span::raw(state.input.clone())
    };
    let p = Paragraph::new(Line::from(vec![Span::raw("> "), hint]))
        .block(Block::default().borders(Borders::ALL).title(" 回复 "));
    f.render_widget(p, area);
}

fn render_status(f: &mut ratatui::Frame, area: ratatui::layout::Rect, state: &NtTuiState) {
    let p = Paragraph::new(state.status_line()).style(Style::default().fg(Color::Cyan));
    f.render_widget(p, area);
}

fn render_help(f: &mut ratatui::Frame, area: ratatui::layout::Rect) {
    let text = vec![
        Line::from("键位（借鉴 Claude Code / opencode）："),
        Line::from("  Enter 提交行 · 空回/Esc 结束本轮 · Ctrl+C/D 结束"),
        Line::from("  Up/Down 历史 · Ctrl+S 暂存/取回 · Ctrl+O 详情 · F2 侧栏"),
        Line::from("  ? 空输入帮助 · /help 切换本层 · . 结束"),
        Line::from("行协议：<id>: 文字批准 · <id>! 文字驳回 · ok/no <id>"),
    ];
    let p = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title(" 帮助 "))
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_crystal_core::NtDemandKind;

    fn demand(id: &str) -> NtDemand {
        NtDemand {
            id: id.to_string(),
            kind: NtDemandKind::RetryFailed,
            text: "重试子任务 st-1".to_string(),
        }
    }

    fn state() -> NtTuiState {
        NtTuiState::new(
            "目标：G\n融合结论（置信 0.62）：答案文本\n内需：1 项",
            &[demand("retry-1")],
            "cli-free×2".to_string(),
            2,
            vec![],
        )
    }

    fn key(code: KeyCode) -> (KeyCode, KeyModifiers) {
        (code, KeyModifiers::NONE)
    }

    fn ctrl(code: KeyCode) -> (KeyCode, KeyModifiers) {
        (code, KeyModifiers::CONTROL)
    }

    #[test]
    fn test_extract_and_status() {
        let s = state();
        assert_eq!(s.goal, "目标：G");
        assert!(s.fused_line.contains("0.62"));
        let status = s.status_line();
        assert!(status.contains("第2轮"));
        assert!(status.contains("内需1"));
        assert!(status.contains("cli-free×2"));
    }

    #[test]
    fn test_type_submit_collects() {
        let mut s = state();
        for c in "ok retry-1".chars() {
            let (code, mods) = (KeyCode::Char(c), KeyModifiers::NONE);
            assert_eq!(apply_key(&mut s, code, mods), TuiKeyOutcome::Continue);
        }
        let (code, mods) = key(KeyCode::Enter);
        assert_eq!(apply_key(&mut s, code, mods), TuiKeyOutcome::Continue);
        assert_eq!(s.collected, vec!["ok retry-1".to_string()]);
        assert_eq!(s.history, vec!["ok retry-1".to_string()]);
    }

    #[test]
    fn test_empty_enter_and_esc_finish() {
        let mut s = state();
        let (code, mods) = key(KeyCode::Enter);
        assert_eq!(apply_key(&mut s, code, mods), TuiKeyOutcome::Finish);
        let mut s = state();
        let (code, mods) = key(KeyCode::Esc);
        assert_eq!(apply_key(&mut s, code, mods), TuiKeyOutcome::Finish);
    }

    #[test]
    fn test_esc_with_draft_clears_to_history() {
        let mut s = state();
        let (code, mods) = (KeyCode::Char('a'), KeyModifiers::NONE);
        apply_key(&mut s, code, mods);
        let (code, mods) = key(KeyCode::Esc);
        assert_eq!(apply_key(&mut s, code, mods), TuiKeyOutcome::Continue);
        assert!(s.input.is_empty());
        assert_eq!(s.history, vec!["a".to_string()]);
    }

    #[test]
    fn test_history_nav_and_stash() {
        let mut s = state();
        s.history = vec!["first".to_string(), "second".to_string()];
        let (code, mods) = key(KeyCode::Up);
        apply_key(&mut s, code, mods);
        assert_eq!(s.input, "second");
        apply_key(&mut s, code, mods);
        assert_eq!(s.input, "first");
        let (code, mods) = key(KeyCode::Down);
        apply_key(&mut s, code, mods);
        assert_eq!(s.input, "second");
        // stash
        let (code, mods) = ctrl(KeyCode::Char('s'));
        apply_key(&mut s, code, mods);
        assert!(s.input.is_empty());
        apply_key(&mut s, code, mods);
        assert_eq!(s.input, "second");
    }

    #[test]
    fn test_toggles_and_help() {
        let mut s = state();
        let (code, mods) = ctrl(KeyCode::Char('o'));
        apply_key(&mut s, code, mods);
        assert!(s.show_details);
        let (code, mods) = (KeyCode::F(2), KeyModifiers::NONE);
        apply_key(&mut s, code, mods);
        assert!(!s.show_sidebar);
        let (code, mods) = (KeyCode::Char('?'), KeyModifiers::NONE);
        apply_key(&mut s, code, mods);
        assert!(s.show_help);
        // sidebar compact vs detailed rows
        s.show_details = false;
        assert!(s.sidebar_lines().iter().any(|l| l.contains("内需（1）")));
        s.show_details = true;
        assert!(s.sidebar_lines().iter().any(|l| l.contains("重试子任务")));
    }

    #[test]
    fn test_dot_finishes_and_slash_help() {
        let mut s = state();
        for c in ".".chars() {
            apply_key(&mut s, KeyCode::Char(c), KeyModifiers::NONE);
        }
        assert_eq!(
            apply_key(&mut s, KeyCode::Enter, KeyModifiers::NONE),
            TuiKeyOutcome::Finish
        );
        assert!(s.collected.is_empty());
        let mut s = state();
        for c in "/help".chars() {
            apply_key(&mut s, KeyCode::Char(c), KeyModifiers::NONE);
        }
        assert_eq!(
            apply_key(&mut s, KeyCode::Enter, KeyModifiers::NONE),
            TuiKeyOutcome::Continue
        );
        assert!(s.show_help);
    }
}
