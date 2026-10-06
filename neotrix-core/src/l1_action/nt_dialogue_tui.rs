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

use crate::l1_action::nt_action_facade::{NtDemand, NtHumanChannel, NtHumanReply};
use crate::l1_action::nt_stdin_human::NtStdinHuman;
use crate::l1_action::nt_tui_theme as theme;
use std::sync::Mutex;

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
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
    /// 光标（字符下标，CJK 安全；v2 编辑器）。
    pub cursor: usize,
    /// 实录滚动偏移（鼠标滚轮/PageUpDown；0 = 底部跟随）。
    pub scroll: usize,
    pub history: Vec<String>,
    pub hist_idx: Option<usize>,
    pub stash: Option<String>,
    pub show_sidebar: bool,
    pub show_details: bool,
    pub show_help: bool,
    /// 模型选择器（F2/Alt+P/`/model` 开；None = 关闭）。
    pub picker: Option<NtPicker>,
    /// 选择器候选（TUI App 注入池模型；空 = 选择器打不开）。
    pub pool_models: Vec<String>,
    /// 本轮已收集的回复行（`parse_lines` 的输入）。
    pub collected: Vec<String>,
}

/// 模型选择器状态（opencode F2 式模糊单）。
pub struct NtPicker {
    pub filter: String,
    pub cursor: usize,
    pub selected: usize,
}

impl NtPicker {
    pub fn new() -> Self {
        Self {
            filter: String::new(),
            cursor: 0,
            selected: 0,
        }
    }

    /// 过滤后候选（空 query 全量；大小写不敏感子串；保序）。
    pub fn items(&self, models: &[String]) -> Vec<String> {
        picker_filter(models, &self.filter)
    }
}

impl Default for NtPicker {
    fn default() -> Self {
        Self::new()
    }
}

/// 纯函数：模糊过滤（子串即中）。
pub fn picker_filter(models: &[String], query: &str) -> Vec<String> {
    if query.trim().is_empty() {
        return models.to_vec();
    }
    let q = query.to_lowercase();
    models
        .iter()
        .filter(|m| m.to_lowercase().contains(&q))
        .cloned()
        .collect()
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
            cursor: 0,
            scroll: 0,
            history,
            hist_idx: None,
            stash: None,
            show_sidebar: true,
            show_details: false,
            show_help: false,
            picker: None,
            pool_models: Vec::new(),
            collected: Vec::new(),
        }
    }

    /// 注入池模型（选择器/`/pool` 数据源；App 层调用）。
    pub fn with_pool_models(mut self, models: Vec<String>) -> Self {
        self.pool_models = models;
        self
    }

    /// 输入字符数（光标运算统一走字符下标）。
    pub fn input_len_chars(&self) -> usize {
        self.input.chars().count()
    }

    /// 光标所在（行，行内列），字符级。
    pub fn cursor_row_col(&self) -> (usize, usize) {
        let mut row = 0;
        let mut col = 0;
        for (i, c) in self.input.chars().enumerate() {
            if i >= self.cursor {
                break;
            }
            if c == '\n' {
                row += 1;
                col = 0;
            } else {
                col += 1;
            }
        }
        (row, col)
    }

    /// 底栏 status line（Claude 式）：轮次·内需·置信·模型·已收集·键位。
    pub fn status_line(&self) -> String {
        format!(
            "第{}轮 · 内需{} · 已收集{} · {} · {} · Enter提交 空回结束 Esc结束 Ctrl+O详情 F2侧栏 ?帮助",
            self.round_no,
            self.demands.len(),
            self.collected.len(),
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
    /// 需求单行走 `demand_row_text`（渲染侧再套 `demand_style` 配色）。
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
            out.push(demand_row_text(d, self.show_details));
        }
        out.push(format!("池：{}", self.pool_line));
        out
    }
}

/// 需求单行文本（纯函数，侧栏与测试共用）。
pub fn demand_row_text(d: &NtDemand, detailed: bool) -> String {
    if detailed {
        format!("[{}] {}：{}", d.id, d.kind.label(), d.text)
    } else {
        format!("[{}] {}", d.id, d.kind.label())
    }
}

/// 需求单配色 —— 已平移至 [`crate::l1_action::nt_tui_theme::demand`]。
///
/// ⛔ 2026-10-05：本函数原为唯一定义，现改为**纯转出**（不是第二份定义）。
/// 理由：配色真源必须唯一，否则 v1/v2 会各自漂移。
/// 保留旧名 `demand_style` 是为了不动 2 个渲染调用点与 4 条测试断言；
/// 若直接删名，测试里钉死调色板的断言会一并失效（那是覆盖率的倒退）。
pub use crate::l1_action::nt_tui_theme::demand as demand_style;

/// 从窗口文本按前缀取行（render() 格式的逆操作）。
pub fn extract_line(window: &str, prefix: &str) -> String {
    window
        .lines()
        .find(|l| l.trim_start().starts_with(prefix))
        .map(|l| l.trim().to_string())
        .unwrap_or_default()
}

/// 按键处理结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TuiKeyOutcome {
    Continue,
    /// 结束本轮输入（Esc/空回/Ctrl+C/D/`.` 行）。
    Finish,
    /// 选择器选中模型（v2：App 层接池定点）。
    Pick(String),
}

/// 纯状态机：按键 → 变异 state，返回是否结束本轮。
/// 键位（Claude/opencode 对齐）：Enter 提交行；空回结束；Esc 清空→再按结束；
/// Up/Down 历史；Ctrl+S 暂存；Ctrl+O 详情；F2 侧栏；`?` 空输入帮助。
pub fn apply_key(state: &mut NtTuiState, code: KeyCode, mods: KeyModifiers) -> TuiKeyOutcome {
    let ctrl = mods.contains(KeyModifiers::CONTROL);
    let alt = mods.contains(KeyModifiers::ALT);

    // 选择器开时：按键只喂选择器，不漏进输入框
    if state.picker.is_some() {
        return apply_key_picker(state, code, ctrl);
    }

    match (code, ctrl, alt) {
        (KeyCode::Esc, _, _) => {
            if state.input.trim().is_empty() {
                return TuiKeyOutcome::Finish;
            }
            // 有草稿：进历史后清空（Claude 双 Esc 语义的一半）
            state.history.push(state.input.clone());
            state.input.clear();
            state.cursor = 0;
            state.hist_idx = None;
            TuiKeyOutcome::Continue
        }
        (KeyCode::Char('c'), true, _) | (KeyCode::Char('d'), true, _) => TuiKeyOutcome::Finish,
        (KeyCode::Char('o'), true, _) => {
            state.show_details = !state.show_details;
            TuiKeyOutcome::Continue
        }
        (KeyCode::Char('s'), true, _) => {
            if state.input.trim().is_empty() {
                if let Some(stashed) = state.stash.take() {
                    state.input = stashed;
                    state.cursor = state.input_len_chars();
                }
            } else {
                state.stash = Some(state.input.clone());
                state.input.clear();
                state.cursor = 0;
            }
            state.hist_idx = None;
            TuiKeyOutcome::Continue
        }
        (KeyCode::Char('p'), _, true) => {
            // Alt+P 模型选择器（Claude 式；池空则打不开）
            if state.pool_models.is_empty() {
                return TuiKeyOutcome::Continue;
            }
            state.picker = Some(NtPicker::new());
            TuiKeyOutcome::Continue
        }
        (KeyCode::F(2), _, _) => {
            state.show_sidebar = !state.show_sidebar;
            TuiKeyOutcome::Continue
        }
        (KeyCode::Up, _, _) => {
            if move_cursor_row(state, true) {
                return TuiKeyOutcome::Continue;
            }
            if !state.history.is_empty() {
                let next = match state.hist_idx {
                    None => state.history.len() - 1,
                    Some(0) => 0,
                    Some(i) => i - 1,
                };
                state.hist_idx = Some(next);
                state.input = state.history[next].clone();
                state.cursor = state.input_len_chars();
            }
            TuiKeyOutcome::Continue
        }
        (KeyCode::Down, _, _) => {
            if move_cursor_row(state, false) {
                return TuiKeyOutcome::Continue;
            }
            match state.hist_idx {
                None => {}
                Some(i) if i + 1 >= state.history.len() => {
                    state.hist_idx = None;
                    state.input.clear();
                    state.cursor = 0;
                }
                Some(i) => {
                    state.hist_idx = Some(i + 1);
                    state.input = state.history[i + 1].clone();
                    state.cursor = state.input_len_chars();
                }
            }
            TuiKeyOutcome::Continue
        }
        (KeyCode::Left, _, _) => {
            state.cursor = state.cursor.saturating_sub(1);
            TuiKeyOutcome::Continue
        }
        (KeyCode::Right, _, _) => {
            state.cursor = (state.cursor + 1).min(state.input_len_chars());
            TuiKeyOutcome::Continue
        }
        (KeyCode::Home, _, _) => {
            state.cursor = line_start(state);
            TuiKeyOutcome::Continue
        }
        (KeyCode::End, _, _) => {
            state.cursor = line_end(state);
            TuiKeyOutcome::Continue
        }
        (KeyCode::Char('j'), true, _) => {
            // Ctrl+J 换行（Claude 式；Enter 永远提交）
            insert_at_cursor(state, '\n');
            TuiKeyOutcome::Continue
        }
        (KeyCode::Enter, _, _) => {
            let line = state.input.clone();
            if line.trim().is_empty() {
                return TuiKeyOutcome::Finish;
            }
            if line.trim() == "?" || line.trim() == "/help" {
                state.show_help = !state.show_help;
                state.input.clear();
                state.cursor = 0;
                state.hist_idx = None;
                return TuiKeyOutcome::Continue;
            }
            if line.trim() == "." {
                return TuiKeyOutcome::Finish;
            }
            state.history.push(line.clone());
            state.collected.push(line);
            state.input.clear();
            state.cursor = 0;
            state.hist_idx = None;
            TuiKeyOutcome::Continue
        }
        (KeyCode::Backspace, _, _) => {
            if state.cursor > 0 && state.cursor <= state.input_len_chars() {
                let mut chars: Vec<char> = state.input.chars().collect();
                chars.remove(state.cursor - 1);
                state.input = chars.into_iter().collect();
                state.cursor -= 1;
            }
            TuiKeyOutcome::Continue
        }
        (KeyCode::Delete, _, _) => {
            let len = state.input_len_chars();
            if state.cursor < len {
                let mut chars: Vec<char> = state.input.chars().collect();
                chars.remove(state.cursor);
                state.input = chars.into_iter().collect();
            }
            TuiKeyOutcome::Continue
        }
        (KeyCode::Char('?'), _, _) if state.input.is_empty() => {
            state.show_help = !state.show_help;
            TuiKeyOutcome::Continue
        }
        (KeyCode::Char(c), false, false) => {
            insert_at_cursor(state, c);
            state.hist_idx = None;
            TuiKeyOutcome::Continue
        }
        _ => TuiKeyOutcome::Continue,
    }
}

/// 光标处插入字符（字符下标，CJK 安全）。
fn insert_at_cursor(state: &mut NtTuiState, c: char) {
    let mut chars: Vec<char> = state.input.chars().collect();
    let pos = state.cursor.min(chars.len());
    chars.insert(pos, c);
    state.input = chars.into_iter().collect();
    state.cursor = pos + 1;
}

/// 当前行首/行尾（字符下标）。
fn line_start(state: &NtTuiState) -> usize {
    let chars: Vec<char> = state.input.chars().collect();
    let mut start = 0;
    for (i, c) in chars.iter().enumerate() {
        if i >= state.cursor {
            break;
        }
        if *c == '\n' {
            start = i + 1;
        }
    }
    start
}

fn line_end(state: &NtTuiState) -> usize {
    let chars: Vec<char> = state.input.chars().collect();
    let mut end = chars.len();
    for (i, c) in chars.iter().enumerate() {
        if i >= state.cursor && *c == '\n' {
            end = i;
            break;
        }
    }
    end
}

/// 上下行移动：多行且光标不在首/末行才吃掉按键（同列 clamp），否则返回 false 走历史。
fn move_cursor_row(state: &mut NtTuiState, up: bool) -> bool {
    let rows: Vec<&str> = state.input.split('\n').collect();
    if rows.len() < 2 {
        return false;
    }
    let (row, col) = state.cursor_row_col();
    if up && row == 0 {
        return false;
    }
    if !up && row + 1 >= rows.len() {
        return false;
    }
    let target = if up { row - 1 } else { row + 1 };
    let target_len = rows[target].chars().count();
    let new_col = col.min(target_len);
    let mut pos = 0;
    for r in 0..target {
        pos += rows[r].chars().count() + 1;
    }
    state.cursor = pos + new_col;
    state.hist_idx = None;
    true
}

/// 选择器按键（打开时独占）：过滤/移动/确认/关闭。
fn apply_key_picker(state: &mut NtTuiState, code: KeyCode, ctrl: bool) -> TuiKeyOutcome {
    let picker = match state.picker.as_mut() {
        Some(p) => p,
        None => return TuiKeyOutcome::Continue,
    };
    let items = picker.items(&state.pool_models);
    match (code, ctrl) {
        (KeyCode::Esc, _) => {
            state.picker = None;
            TuiKeyOutcome::Continue
        }
        (KeyCode::Enter, _) => {
            let pick = items
                .get(picker.selected)
                .cloned()
                .unwrap_or_default();
            state.picker = None;
            if pick.is_empty() {
                TuiKeyOutcome::Continue
            } else {
                TuiKeyOutcome::Pick(pick)
            }
        }
        (KeyCode::Up, _) => {
            if picker.selected > 0 {
                picker.selected -= 1;
            }
            TuiKeyOutcome::Continue
        }
        (KeyCode::Down, _) => {
            if !items.is_empty() {
                picker.selected = (picker.selected + 1).min(items.len() - 1);
            }
            TuiKeyOutcome::Continue
        }
        (KeyCode::Backspace, _) => {
            picker.filter.pop();
            picker.cursor = picker.cursor.saturating_sub(1);
            picker.selected = 0;
            TuiKeyOutcome::Continue
        }
        (KeyCode::Char(c), false) => {
            picker.filter.push(c);
            picker.cursor += 1;
            picker.selected = 0;
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
        if state.picker.is_some() {
            render_picker(f, area, state);
        }
    });
}

fn render_transcript(
    f: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    state: &NtTuiState,
) {
    // 滚动：跳过前 scroll 行（鼠标滚轮/PageUpDown；0 = 底部跟随）
    let items: Vec<ListItem> = state
        .transcript
        .iter()
        .skip(state.scroll)
        .map(|l| ListItem::new(Line::from(l.clone())))
        .collect();
    let title = if state.scroll > 0 {
        format!(" 实录 · {}（上滚{}） ", state.goal, state.scroll)
    } else {
        format!(" 实录 · {}", state.goal)
    };
    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(list, area);
}

fn render_sidebar(f: &mut ratatui::Frame, area: ratatui::layout::Rect, state: &NtTuiState) {
    let mut items: Vec<ListItem> = Vec::new();
    items.push(ListItem::new(Line::from(format!(
        "融合：{}",
        if state.fused_line.is_empty() {
            "（暂无）".to_string()
        } else {
            state.fused_line.clone()
        }
    ))));
    items.push(ListItem::new(Line::from(format!(
        "内需（{}）：",
        state.demands.len()
    ))));
    for d in &state.demands {
        items.push(ListItem::new(Line::from(vec![Span::styled(
            demand_row_text(d, state.show_details),
            demand_style(d.kind),
        )])));
    }
    items.push(ListItem::new(Line::from(format!("池：{}", state.pool_line))));
    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" 内需/结论/池 "),
    );
    f.render_widget(list, area);
}

fn render_input(f: &mut ratatui::Frame, area: ratatui::layout::Rect, state: &NtTuiState) {
    // 光标：输入字符中插入反白块（终端原生 cursor 另由 ratatui 定位）
    let mut spans = vec![Span::raw("> ")];
    if state.input.is_empty() {
        spans.push(Span::styled(
            "<id>: 文字批准 · <id>! 文字驳回 · ok/no <id> · 空回结束",
            theme::dim(),
        ));
    } else {
        let chars: Vec<char> = state.input.chars().collect();
        let pos = state.cursor.min(chars.len());
        let before: String = chars[..pos].iter().collect();
        let at = chars
            .get(pos)
            .map(|c| c.to_string())
            .unwrap_or_else(|| " ".to_string());
        let after: String = chars[pos + (chars.get(pos).map(|_| 1).unwrap_or(0))..]
            .iter()
            .collect();
        spans.push(Span::raw(before));
        spans.push(Span::styled(
            at,
            theme::cursor(),
        ));
        spans.push(Span::raw(after));
    }
    let p = Paragraph::new(Line::from(spans)).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" 回复（已收集{}） ", state.collected.len())),
    );
    f.render_widget(p, area);
    // 原生光标跟随（多行时按行列定位）
    let (row, col) = state.cursor_row_col();
    f.set_cursor_position((area.x + 2 + col as u16, area.y + 1 + row as u16));
}

fn render_status(f: &mut ratatui::Frame, area: ratatui::layout::Rect, state: &NtTuiState) {
    let p = Paragraph::new(state.status_line()).style(theme::status());
    f.render_widget(p, area);
}

fn render_picker(f: &mut ratatui::Frame, area: ratatui::layout::Rect, state: &NtTuiState) {
    let picker = match &state.picker {
        Some(p) => p,
        None => return,
    };
    let items_all = picker.items(&state.pool_models);
    let show: Vec<ListItem> = items_all
        .iter()
        .enumerate()
        .take(10)
        .map(|(i, m)| {
            let marker = if i == picker.selected.min(items_all.len().saturating_sub(1)) {
                "▶ "
            } else {
                "  "
            };
            ListItem::new(Line::from(vec![
                Span::raw(marker),
                Span::styled(
                    m.clone(),
                    if i == picker.selected {
                        theme::selected()
                    } else {
                        theme::normal()
                    },
                ),
            ]))
        })
        .collect();
    let w = area.width.saturating_sub(20).max(40);
    let h = (items_all.len().min(10) + 4)
        .min(area.height.saturating_sub(4).max(6) as usize)
        .max(6) as u16;
    let x = (area.width.saturating_sub(w)) / 2;
    let y = (area.height.saturating_sub(h)) / 2;
    let popup = ratatui::layout::Rect::new(x, y, w, h);
    f.render_widget(ratatui::widgets::Clear, popup);
    let list = List::new(show).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" 模型（{}）· 输入过滤 · Enter定点 · Esc关闭 ", picker.filter)),
    );
    f.render_widget(list, popup);
}

fn render_help(f: &mut ratatui::Frame, area: ratatui::layout::Rect) {
    let text = vec![
        Line::from("键位（借鉴 Claude Code / opencode）："),
        Line::from("  Enter 提交行 · 空回/Esc 结束本轮 · Ctrl+C/D 结束"),
        Line::from("  Up/Down 历史（多行时行内移动）· ←/→/Home/End 光标 · Ctrl+J 换行"),
        Line::from("  Ctrl+S 暂存/取回 · Ctrl+O 详情 · F2 侧栏 · Alt+P/F2选模型"),
        Line::from("  ? 空输入帮助 · /help 切换本层 · . 结束 · /pool /model /quit"),
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
    // 2026-10-03 跨域错位收敛（函数内 use，与顶部 import 同一根因）
    use crate::l1_action::nt_action_facade::NtDemandKind;

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

    #[test]
    fn test_demand_row_and_style() {
        let d = demand("retry-1");
        assert_eq!(
            demand_row_text(&d, false),
            "[retry-1] 失败重试".to_string()
        );
        assert!(demand_row_text(&d, true).contains("重试子任务"));
        // ⚠️ 2026-10-05 移走 4 条调色板断言到 `nt_tui_theme::tests`。
        //
        // 理由：`demand_style` 已改为从 `nt_tui_theme::demand` **纯转出**，
        // 配色真源唯一。若断言留在这里，会让人以为本文件是配色真源；
        // 留在真源模块里才能保证「改配色必改测试」。
        //
        // ⛔ 覆盖面**未减少**：新位置的 `demand_palette_is_stable` 逐字覆盖
        // 全部 **7 个**变体（旧处只断言 4 个），另新增
        // `same_color_different_semantics` 钉住「一色一名」不变量。
        //
        // 行为侧仍有断言在本测试里：`demand_row_text` 的两种形态。
        assert!(demand_style(NtDemandKind::RetryFailed).fg.is_some());
    }

    #[test]
    fn test_status_shows_collected() {
        let mut s = state();
        assert!(s.status_line().contains("已收集0"));
        s.collected.push("ok retry-1".to_string());
        assert!(s.status_line().contains("已收集1"));
    }

    fn type_text(s: &mut NtTuiState, text: &str) {
        for c in text.chars() {
            apply_key(s, KeyCode::Char(c), KeyModifiers::NONE);
        }
    }

    #[test]
    fn test_cursor_edit() {
        let mut s = state();
        type_text(&mut s, "abc");
        assert_eq!(s.cursor, 3);
        apply_key(&mut s, KeyCode::Left, KeyModifiers::NONE);
        apply_key(&mut s, KeyCode::Left, KeyModifiers::NONE);
        assert_eq!(s.cursor, 1);
        apply_key(&mut s, KeyCode::Char('X'), KeyModifiers::NONE);
        assert_eq!(s.input, "aXbc");
        apply_key(&mut s, KeyCode::Backspace, KeyModifiers::NONE);
        assert_eq!(s.input, "abc");
        // CJK 安全
        let mut s = state();
        type_text(&mut s, "支付好");
        apply_key(&mut s, KeyCode::Left, KeyModifiers::NONE);
        apply_key(&mut s, KeyCode::Backspace, KeyModifiers::NONE);
        assert_eq!(s.input, "支好");
    }

    #[test]
    fn test_multiline_and_row_move() {
        let mut s = state();
        type_text(&mut s, "ab");
        apply_key(&mut s, KeyCode::Char('j'), KeyModifiers::CONTROL);
        type_text(&mut s, "cd");
        assert_eq!(s.input, "ab\ncd");
        // 光标在末行：Up 行内上移，不进历史
        s.history = vec!["h".to_string()];
        apply_key(&mut s, KeyCode::Up, KeyModifiers::NONE);
        assert_eq!(s.input, "ab\ncd");
        let (row, _) = s.cursor_row_col();
        assert_eq!(row, 0);
        // 首行再 Up：进历史
        apply_key(&mut s, KeyCode::Up, KeyModifiers::NONE);
        assert_eq!(s.input, "h");
    }

    #[test]
    fn test_picker_flow() {
        let mut s = state();
        // 池空：打不开
        apply_key(&mut s, KeyCode::F(2), KeyModifiers::NONE);
        assert!(s.picker.is_none());
        s.pool_models = vec!["mimo-free".to_string(), "spark-free".to_string()];
        // Alt+P 打开
        apply_key(&mut s, KeyCode::Char('p'), KeyModifiers::ALT);
        assert!(s.picker.is_some());
        // 过滤
        type_text(&mut s, "spark");
        assert!(s.input.is_empty());
        // Enter 选中
        let r = apply_key(&mut s, KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(r, TuiKeyOutcome::Pick("spark-free".to_string()));
        assert!(s.picker.is_none());
    }

    #[test]
    fn test_picker_filter_fn() {
        let models = vec!["a-free".to_string(), "b-pro".to_string()];
        assert_eq!(picker_filter(&models, "").len(), 2);
        assert_eq!(picker_filter(&models, "FREE"), vec!["a-free".to_string()]);
        assert!(picker_filter(&models, "zzz").is_empty());
    }

    #[test]
    fn test_picker_esc_closes() {
        let mut s = state();
        s.pool_models = vec!["m".to_string()];
        apply_key(&mut s, KeyCode::Char('p'), KeyModifiers::ALT);
        assert!(s.picker.is_some());
        type_text(&mut s, "zzz");
        // 空结果 Enter：关闭不选中
        let r = apply_key(&mut s, KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(r, TuiKeyOutcome::Continue);
        assert!(s.picker.is_none());
    }
}
