//! # nt_tui_app — v2 事件驱动会话应用
//!
//! v1（`NtTuiHuman`）是阻塞式"打印+读行"；v2 让 TUI 拥有整个会话
//! （Claude/opencode 形态）：drive 跑工作线程，UI 主循环实时渲染，
//! channel 对话 + sink 进度 + Esc 取消。
//!
//! ```text
//! main 线程（UI）                    工作线程
//!   TuiApp::run ──spawn──▶ drive_with_sink(goal, core, ask, channel_human, sink)
//!       │  UiEvent ◀── mpsc ── sink：SubtaskStart/Chunk/Done
//!       │  Demands ◀── mpsc ── channel_human.prompt（阻塞等回复）
//!       │  replies ── mpsc ──▶ prompt 收齐 Finish 行后一次发回
//!       ▼
//!   crossterm 事件（键/鼠标滚轮）+ 渲染（实录/工作相/选择器/帮助/状态）
//! ```
//!
//! ## v2 能力映射
//! - 工作相：spinner + 流式答案尾行 + 已用时间；Esc 置取消旗（流式 ask 即 kill，
//!   非流式跑到提供方超时为止，如实声明）。
//! - 模型选择器：F2/Alt+P/`/model`，模糊过滤，Enter 定点到池（`set_pinned`），
//!   状态栏实时显示定点。
//! - `/` 命令：help/pool/model/quit/clear（清屏显存，不碰记忆）。
//! - 编辑器：光标/多行/Ctrl+J/历史行感知（状态机在 nt_dialogue_tui）。
//! - 鼠标：滚轮滚动实录（需终端开启鼠标上报）。
//!
//! # Safety
//! - 工作线程只经 mpsc 与 UI 通信；共享可变只有 `AtomicBool` 取消旗与
//!   `Mutex` 通道端点，无 unsafe (R-P1)；生产代码无 `unwrap/expect/panic`。

use crate::l1_action::nt_dialogue_tui::{
    apply_key, demand_row_text, demand_style, NtPicker, NtTuiState, TuiKeyOutcome,
};
use crate::l1_action::nt_free_pool::NtFreePoolAsk;
use crate::l1_action::nt_stdin_human::NtStdinHuman;
use crate::l5_cognition::nt_crystal_core::{
    CrystalCore, NtDemand, NtHumanChannel, NtHumanReply, NtInnerLoop, NtInnerLoopOutcome,
    NtLlmAsk, NtProgressSink, NtTaskLoopConfig,
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc, Mutex,
};
use std::time::{Duration, Instant};

use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseEventKind,
    },
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
// 通道
// ============================================================================

/// 工作线程 → UI 线程事件。
pub enum UiEvent {
    SubtaskStart { id: String, title: String },
    Chunk { id: String, delta: String },
    SubtaskDone { id: String, ok: bool },
    Demands { window: String, demands: Vec<NtDemand> },
    /// drive 终态（core 经工作线程 join 交还，不走事件）。
    Finished { outcome: NtInnerLoopOutcome },
}

/// 进度接收器（工作线程侧）：转交 UI + 暴露取消旗。
pub struct NtUiSink {
    tx: mpsc::Sender<UiEvent>,
    cancel: Arc<AtomicBool>,
}

impl NtUiSink {
    pub fn new(tx: mpsc::Sender<UiEvent>, cancel: Arc<AtomicBool>) -> Self {
        Self { tx, cancel }
    }
}

impl NtProgressSink for NtUiSink {
    fn on_subtask_start(&self, subtask_id: &str, title: &str) {
        let _ = self.tx.send(UiEvent::SubtaskStart {
            id: subtask_id.to_string(),
            title: title.to_string(),
        });
    }

    fn on_answer_chunk(&self, subtask_id: &str, delta: &str) {
        let _ = self.tx.send(UiEvent::Chunk {
            id: subtask_id.to_string(),
            delta: delta.to_string(),
        });
    }

    fn on_subtask_done(&self, subtask_id: &str, success: bool) {
        let _ = self.tx.send(UiEvent::SubtaskDone {
            id: subtask_id.to_string(),
            ok: success,
        });
    }

    fn cancelled(&self) -> bool {
        self.cancel.load(Ordering::SeqCst)
    }
}

/// 通道式人类通道：prompt 把需求单发给 UI 线程，阻塞等回复。
/// UI 消亡（对端挂）→ 返回空（Stalled 语义，不断轮）。
pub struct NtChannelHuman {
    tx: mpsc::Sender<UiEvent>,
    rx: Mutex<mpsc::Receiver<Vec<NtHumanReply>>>,
}

impl NtChannelHuman {
    pub fn new(
        tx: mpsc::Sender<UiEvent>,
        rx: mpsc::Receiver<Vec<NtHumanReply>>,
    ) -> Self {
        Self {
            tx,
            rx: Mutex::new(rx),
        }
    }
}

impl NtHumanChannel for NtChannelHuman {
    fn prompt(&self, window: &str, demands: &[NtDemand]) -> Vec<NtHumanReply> {
        if self
            .tx
            .send(UiEvent::Demands {
                window: window.to_string(),
                demands: demands.to_vec(),
            })
            .is_err()
        {
            return Vec::new();
        }
        match self.rx.lock() {
            Ok(rx) => rx.recv().unwrap_or_default(),
            Err(_) => Vec::new(),
        }
    }
}

// ============================================================================
// 斜杠命令（纯函数）
// ============================================================================

/// 斜杠命令。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NtSlash {
    Help,
    Pool,
    Model,
    Quit,
    Clear,
    Unknown,
}

/// 纯函数：`/` 行分发（未知命令回 Unknown，App 层提示）。
pub fn dispatch_slash(line: &str) -> NtSlash {
    let t = line.trim();
    if !t.starts_with('/') {
        return NtSlash::Unknown;
    }
    match t.split_whitespace().next().unwrap_or("") {
        "/help" => NtSlash::Help,
        "/pool" => NtSlash::Pool,
        "/model" => NtSlash::Model,
        "/quit" | "/exit" => NtSlash::Quit,
        "/clear" => NtSlash::Clear,
        _ => NtSlash::Unknown,
    }
}

/// spinner 帧（Claude 式忙指示；纯函数可测）。
pub fn spinner_frame(elapsed_ms: u64) -> char {
    const FRAMES: [char; 8] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧'];
    FRAMES[(elapsed_ms as usize / 120) % FRAMES.len()]
}

// ============================================================================
// 应用
// ============================================================================

/// 工作相视图（当前子任务流式状态）。
struct WorkingView {
    subtask_id: String,
    title: String,
    stream: String,
    started: Instant,
    done_note: Option<String>,
    /// 完成时间（用于延迟清除）。
    done_at: Option<Instant>,
}

/// v2 会话应用（UI 线程拥有）。
pub struct NtTuiApp {
    state: NtTuiState,
    pool: Arc<NtFreePoolAsk>,
    pool_lines: Vec<String>,
    /// 并行执行中的子任务（可能多个同时活跃）。
    active_tasks: Vec<WorkingView>,
    quit: bool,
}

impl NtTuiApp {
    pub fn new(
        goal: String,
        pool: Arc<NtFreePoolAsk>,
        pool_lines: Vec<String>,
        history: Vec<String>,
    ) -> Self {
        let models = pool.model_ids().to_vec();
        let mut state = NtTuiState::new("", &[], String::new(), 1, history);
        state.goal = goal;
        state.pool_models = models;
        Self {
            state,
            pool,
            pool_lines,
            active_tasks: Vec::new(),
            quit: false,
        }
    }

    /// 当前定点模型（状态栏用；无则轮转中）。
    pub fn pinned_label(&self) -> String {
        match self.pool.pinned() {
            Some(m) => format!("定点 {m}"),
            None => "轮转中".to_string(),
        }
    }

    fn on_event(&mut self, ev: UiEvent) {
        match ev {
            UiEvent::SubtaskStart { id, title } => {
                self.active_tasks.push(WorkingView {
                    subtask_id: id.clone(),
                    title: title.clone(),
                    stream: String::new(),
                    started: Instant::now(),
                    done_note: None,
                    done_at: None,
                });
                self.state.transcript.push(format!("▶ {id} {title}"));
            }
            UiEvent::Chunk { id, delta } => {
                if let Some(w) = self.active_tasks.iter_mut().find(|w| w.subtask_id == id) {
                    w.stream.push_str(&delta);
                }
            }
            UiEvent::SubtaskDone { id, ok } => {
                if let Some(w) = self.active_tasks.iter_mut().find(|w| w.subtask_id == id) {
                    w.done_note = Some(if ok { "✓".to_string() } else { "✗".to_string() });
                    w.done_at = Some(Instant::now());
                    let tail: String = w
                        .stream
                        .chars()
                        .rev()
                        .take(60)
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                        .collect();
                    self.state.transcript.push(format!(
                        "■ {id} {} {}",
                        if ok { "完成" } else { "失败" },
                        tail.replace('\n', " ")
                    ));
                }
                // 不立即清除——保留完成态供 UI 展示，在 tick 中延迟清除
            }
            UiEvent::Demands { window, demands } => {
                self.active_tasks.clear();
                for line in window.lines() {
                    self.state.transcript.push(line.to_string());
                }
                self.state.demands = demands;
                self.state.scroll = 0;
            }
            UiEvent::Finished { .. } => {
                // 终态由主循环直接处理，core 经工作线程 join 交还。
            }
        }
    }

    /// 清理已完成超过3秒的子任务（每次 tick 调用）。
    fn tick_cleanup(&mut self) {
        let _now = Instant::now();
        self.active_tasks.retain(|w| {
            w.done_at
                .map(|t| t.elapsed() < Duration::from_secs(3))
                .unwrap_or(true)
        });
    }

    /// 当前活跃（执行中）子任务数。
    pub fn active_count(&self) -> usize {
        self.active_tasks
            .iter()
            .filter(|w| w.done_note.is_none())
            .count()
    }

    fn clear_input(&mut self) {
        self.state.input.clear();
        self.state.cursor = 0;
        self.state.hist_idx = None;
    }

    /// 结束本轮输入：解析已收集行发回工作线程。
    fn finish_input(&mut self, rep_tx: &mpsc::Sender<Vec<NtHumanReply>>) -> bool {
        let replies = NtStdinHuman::parse_lines(&self.state.collected);
        self.state.collected.clear();
        self.state.cursor = 0;
        self.state.hist_idx = None;
        let _ = rep_tx.send(replies);
        false
    }

    fn apply_pick(&mut self, model: String) {
        self.pool.set_pinned(Some(model.clone()));
        self.state
            .transcript
            .push(format!("已定点模型：{model}"));
    }
}

/// v2 入口：UI 线程跑应用，工作线程跑 drive；返回（终态，晶体）。
/// 终端建失败返回 Err（含原样晶体），调用方回退行式（NtTuiHuman）。
/// 退出语义：终态到达或 `/quit` 后 join 工作线程（Esc 取消保证流式问答快速
/// kill；非流式实现至多阻塞到提供方超时，如实声明）。
pub fn run_tui_session(
    goal: String,
    core: CrystalCore,
    ask: Arc<dyn NtLlmAsk>,
    pool: Arc<NtFreePoolAsk>,
    pool_lines: Vec<String>,
    history: Vec<String>,
    config: NtTaskLoopConfig,
    max_rounds: usize,
) -> Result<(NtInnerLoopOutcome, CrystalCore), (String, CrystalCore)> {
    let (ev_tx, ev_rx) = mpsc::channel::<UiEvent>();
    let (rep_tx, rep_rx) = mpsc::channel::<Vec<NtHumanReply>>();
    let cancel = Arc::new(AtomicBool::new(false));

    // 先建终端：失败则晶体原样奉还，调用方回退行式（工作线程尚未启动）。
    let mut app = NtTuiApp::new(goal.clone(), pool, pool_lines, history);
    let term = build_terminal();
    let mut terminal = match term {
        Ok(t) => t,
        Err(e) => return Err((e, core)),
    };
    let _guard = TuiTerminalGuard::new();

    // 工作线程：drive 全程（含 prompt 阻塞等 UI 回复）。
    let worker_ev = ev_tx.clone();
    let worker_cancel = cancel.clone();
    let goal_worker = goal.clone();
    let worker = std::thread::spawn(move || {
        let mut core = core;
        let human = NtChannelHuman::new(worker_ev.clone(), rep_rx);
        let sink = NtUiSink::new(worker_ev.clone(), worker_cancel);
        let engine = NtInnerLoop::new(config, max_rounds);
        let outcome =
            engine.drive_with_sink(&goal_worker, &mut core, &*ask, &human, Some(&sink));
        let _ = worker_ev.send(UiEvent::Finished { outcome: outcome.clone() });
        (outcome, core)
    });

    app_main_loop(&mut app, &mut terminal, ev_rx, rep_tx, &cancel);
    // 工作线程收尾（join 有界，见上）。
    worker.join().map_err(|_| {
        (
            "worker 线程崩溃".to_string(),
            CrystalCore::new("ntcode-recovered"),
        )
    })
}

struct TuiTerminalGuard;

impl TuiTerminalGuard {
    fn new() -> Self {
        Self
    }
}

impl Drop for TuiTerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
    }
}

fn build_terminal(
) -> Result<Terminal<CrosstermBackend<std::io::Stdout>>, String> {
    enable_raw_mode().map_err(|e| format!("raw mode failed: {e}"))?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .map_err(|e| format!("alt screen failed: {e}"))?;
    Terminal::new(CrosstermBackend::new(stdout))
        .map_err(|e| format!("terminal build failed: {e}"))
}

#[allow(clippy::too_many_lines)]
fn app_main_loop(
    app: &mut NtTuiApp,
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ev_rx: mpsc::Receiver<UiEvent>,
    rep_tx: mpsc::Sender<Vec<NtHumanReply>>,
    cancel: &Arc<AtomicBool>,
) {
    loop {
        // 1. 收工作线程事件（非阻塞 draining）
        while let Ok(ev) = ev_rx.try_recv() {
            match ev {
                UiEvent::Finished { .. } => {
                    // 终态到达：core 经 join 交还，这里只负责收尾退出
                    draw_app(terminal, app);
                    return;
                }
                _ => app.on_event(ev),
            }
        }
        // 1b. 清理超时完成态
        app.tick_cleanup();
        if app.quit {
            cancel.store(true, Ordering::SeqCst);
            return;
        }
        // 2. 读键/鼠标（100ms 节拍，spinner 动画需要重绘）
        if event::poll(Duration::from_millis(100)).unwrap_or(false) {
            match event::read() {
                Ok(Event::Key(k)) => {
                    if app_key(app, &rep_tx, cancel, k.code, k.modifiers) {
                        cancel.store(true, Ordering::SeqCst);
                        return;
                    }
                }
                Ok(Event::Mouse(m)) => match m.kind {
                    MouseEventKind::ScrollUp => {
                        app.state.scroll = app.state.scroll.saturating_add(3);
                    }
                    MouseEventKind::ScrollDown => {
                        app.state.scroll = app.state.scroll.saturating_sub(3);
                    }
                    _ => {}
                },
                Ok(_) => {}
                Err(_) => return,
            }
        }
        draw_app(terminal, app);
    }
}

/// 单按键分发（返回 true = 要求退出进程循环）。
/// 顺序：工作相中断 > 选择器/斜杠 > 通用状态机。
fn app_key(
    app: &mut NtTuiApp,
    rep_tx: &mpsc::Sender<Vec<NtHumanReply>>,
    cancel: &Arc<AtomicBool>,
    code: KeyCode,
    mods: KeyModifiers,
) -> bool {
    // Ctrl+Q 退出（Claude 无此键，取 tmux/通用习惯；文档声明）
    if code == KeyCode::Char('q') && mods.contains(KeyModifiers::CONTROL) {
        app.quit = true;
        return true;
    }
    // 工作相中断：无内需（等 drive 回包）时的 Esc/空回 = 取消当前问答，
    // 不是结束整轮（结束整轮走 Stalled 语义，留给 Input 相）。
    let working_phase =
        app.state.demands.is_empty() && app.state.picker.is_none();
    if working_phase {
        match code {
            KeyCode::Esc => {
                cancel.store(true, Ordering::SeqCst);
                app.state.transcript.push("…已请求中断当前问答…".to_string());
                return false;
            }
            KeyCode::Enter if app.state.input.trim().is_empty() => {
                cancel.store(true, Ordering::SeqCst);
                app.state.transcript.push("…已请求中断当前问答…".to_string());
                return false;
            }
            _ => {}
        }
    }
    // 选择器开时：全权交状态机（Pick 回模型名）
    if app.state.picker.is_some() {
        match apply_key(&mut app.state, code, mods) {
            TuiKeyOutcome::Pick(m) => app.apply_pick(m),
            _ => {}
        }
        return false;
    }
    // 斜杠命令（Enter 且行首 `/`）：App 层截获，不进通用收集
    if code == KeyCode::Enter && app.state.input.trim_start().starts_with('/') {
        return app_slash(app);
    }
    match apply_key(&mut app.state, code, mods) {
        TuiKeyOutcome::Pick(m) => app.apply_pick(m),
        TuiKeyOutcome::Finish => {
            app.finish_input(rep_tx);
        }
        TuiKeyOutcome::Continue => {}
    }
    false
}

/// 斜杠分发（返回 true = 退出）。未知命令记实录一行，不打断。
fn app_slash(app: &mut NtTuiApp) -> bool {
    let line = app.state.input.trim().to_string();
    app.state.input.clear();
    app.state.cursor = 0;
    app.state.hist_idx = None;
    match dispatch_slash(&line) {
        NtSlash::Help => {
            app.state.show_help = true;
            false
        }
        NtSlash::Pool => {
            for l in app.pool_lines.clone() {
                app.state.transcript.push(l);
            }
            false
        }
        NtSlash::Model => {
            if app.state.pool_models.is_empty() {
                app.state.transcript.push("池中无模型可切换。".to_string());
            } else {
                app.state.picker = Some(NtPicker::new());
            }
            false
        }
        NtSlash::Quit => {
            app.quit = true;
            true
        }
        NtSlash::Clear => {
            app.state.transcript.clear();
            app.state.scroll = 0;
            false
        }
        NtSlash::Unknown => {
            app.state
                .transcript
                .push(format!("未知命令：{line}（/help 查看）"));
            false
        }
    }
}

fn draw_app(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, app: &NtTuiApp) {
    let _ = terminal.draw(|f| {
        let area = f.area();
        let active_count = app.active_tasks.len();
        let mut constraints = vec![Constraint::Min(3)];
        if active_count > 0 {
            // 每个活跃子任务一行 + 边框上下各一行
            let work_height = (active_count as u16 + 2).min(8);
            constraints.push(Constraint::Length(work_height));
        }
        constraints.push(Constraint::Length(3));
        constraints.push(Constraint::Length(1));
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        let wide = area.width >= 100;
        let mut row_idx = 0;
        if app.state.show_sidebar && wide {
            let cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(68), Constraint::Percentage(32)])
                .split(rows[0]);
            render_transcript_app(f, cols[0], app);
            render_sidebar_app(f, cols[1], app);
        } else {
            render_transcript_app(f, rows[0], app);
        }
        row_idx += 1;
        if active_count > 0 {
            render_working(f, rows[row_idx], app);
            row_idx += 1;
        }
        render_input_app(f, rows[row_idx], app);
        row_idx += 1;
        render_status_app(f, rows[row_idx], app);

        if app.state.show_help {
            render_help_app(f, area);
        }
        if app.state.picker.is_some() {
            render_picker_app(f, area, app);
        }
    });
}

fn render_transcript_app(
    f: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    app: &NtTuiApp,
) {
    let items: Vec<ListItem> = app
        .state
        .transcript
        .iter()
        .skip(app.state.scroll)
        .map(|l| ListItem::new(Line::from(l.clone())))
        .collect();
    let title = if app.state.scroll > 0 {
        format!(" 实录 · {}（上滚{}） ", app.state.goal, app.state.scroll)
    } else {
        format!(" 实录 · {} ", app.state.goal)
    };
    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(list, area);
}

fn render_sidebar_app(
    f: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    app: &NtTuiApp,
) {
    let mut items: Vec<ListItem> = Vec::new();
    items.push(ListItem::new(Line::from(format!(
        "融合：{}",
        if app.state.fused_line.is_empty() {
            "（暂无）".to_string()
        } else {
            app.state.fused_line.clone()
        }
    ))));
    items.push(ListItem::new(Line::from(format!(
        "内需（{}）：",
        app.state.demands.len()
    ))));
    for d in &app.state.demands {
        items.push(ListItem::new(Line::from(vec![Span::styled(
            demand_row_text(d, app.state.show_details),
            demand_style(d.kind),
        )])));
    }
    items.push(ListItem::new(Line::from(format!(
        "池：{} · {}",
        app.pinned_label(),
        app.state.pool_line
    ))));
    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" 内需/结论/池 "),
    );
    f.render_widget(list, area);
}

fn render_working(f: &mut ratatui::Frame, area: ratatui::layout::Rect, app: &NtTuiApp) {
    if app.active_tasks.is_empty() {
        let p = Paragraph::new(Line::from("")).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 工作中 ")
                .style(Style::default().fg(Color::Yellow)),
        );
        f.render_widget(p, area);
        return;
    }

    // 并行度指示 + 各子任务状态
    let parallelism = app.active_tasks.len();
    let header = if parallelism > 1 {
        format!(" 并行×{parallelism} ")
    } else {
        " 工作中 ".to_string()
    };

    let mut lines: Vec<Line> = Vec::new();
    for w in &app.active_tasks {
        let elapsed = w.started.elapsed().as_millis() as u64;
        let spin = spinner_frame(elapsed);
        let tail: String = w
            .stream
            .chars()
            .rev()
            .take(80)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        let tail = tail.replace('\n', " ");
        let status_icon = match &w.done_note {
            Some(s) => s.as_str(),
            None => "",
        };
        lines.push(Line::from(vec![
            Span::styled(
                format!(" {spin} "),
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                format!("{} ", w.subtask_id),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw(format!("{} ", w.title)),
            Span::raw(format!("{} ", status_icon)),
            Span::styled(tail, Style::default().fg(Color::DarkGray)),
        ]));
    }

    let p = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(header)
            .style(Style::default().fg(Color::Yellow)),
    );
    f.render_widget(p, area);
}

fn render_input_app(
    f: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    app: &NtTuiApp,
) {
    let state = &app.state;
    let mut spans = vec![Span::raw("> ")];
    if state.input.is_empty() {
        spans.push(Span::styled(
            "<id>: 文字批准 · <id>! 文字驳回 · ok/no <id> · /命令 · 空回结束",
            Style::default().fg(Color::DarkGray),
        ));
    } else {
        let chars: Vec<char> = state.input.chars().collect();
        let pos = state.cursor.min(chars.len());
        let before: String = chars[..pos].iter().collect();
        let at = chars
            .get(pos)
            .map(|c| c.to_string())
            .unwrap_or_else(|| " ".to_string());
        let after: String = chars[pos + usize::from(chars.get(pos).is_some())..]
            .iter()
            .collect();
        spans.push(Span::raw(before));
        spans.push(Span::styled(
            at,
            Style::default().bg(Color::DarkGray).fg(Color::White),
        ));
        spans.push(Span::raw(after));
    }
    let p = Paragraph::new(Line::from(spans)).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" 回复（已收集{}） ", state.collected.len())),
    );
    f.render_widget(p, area);
    let (row, col) = state.cursor_row_col();
    f.set_cursor_position((area.x + 2 + col as u16, area.y + 1 + row as u16));
}

fn render_status_app(
    f: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    app: &NtTuiApp,
) {
    let active = app.active_count();
    let parallel_info = if active > 1 {
        format!(" · 并行×{active}")
    } else {
        String::new()
    };
    let p = Paragraph::new(format!(
        "{} · {}{}",
        app.state.status_line(),
        app.pinned_label(),
        parallel_info,
    ))
    .style(Style::default().fg(Color::Cyan));
    f.render_widget(p, area);
}

fn render_help_app(f: &mut ratatui::Frame, area: ratatui::layout::Rect) {
    let text = vec![
        Line::from("键位（v2 事件驱动）："),
        Line::from("  Enter 提交 · 空回/Esc 结束本轮 · 工作中Esc=中断问答 · Ctrl+C/D 结束"),
        Line::from("  ←/→/Home/End 光标 · Ctrl+J 换行 · Up/Down 历史/行移"),
        Line::from("  Ctrl+S 暂存 · Ctrl+O 详情 · F2 侧栏 · Alt+P 选模型 · Ctrl+Q 退出"),
        Line::from("  /help /pool /model /quit /clear · 鼠标滚轮滚动"),
    ];
    let p = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title(" 帮助 "))
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
}

fn render_picker_app(
    f: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    app: &NtTuiApp,
) {
    let picker = match &app.state.picker {
        Some(p) => p,
        None => return,
    };
    let items_all = picker.items(&app.state.pool_models);
    let sel = picker.selected.min(items_all.len().saturating_sub(1));
    let show: Vec<ListItem> = items_all
        .iter()
        .enumerate()
        .take(10)
        .map(|(i, m)| {
            let marker = if i == sel { "▶ " } else { "  " };
            ListItem::new(Line::from(vec![
                Span::raw(marker),
                Span::styled(
                    m.clone(),
                    if i == sel {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatch_slash() {
        assert_eq!(dispatch_slash("/help"), NtSlash::Help);
        assert_eq!(dispatch_slash("/pool  "), NtSlash::Pool);
        assert_eq!(dispatch_slash("/model x"), NtSlash::Model);
        assert_eq!(dispatch_slash("/quit"), NtSlash::Quit);
        assert_eq!(dispatch_slash("/exit"), NtSlash::Quit);
        assert_eq!(dispatch_slash("/clear"), NtSlash::Clear);
        assert_eq!(dispatch_slash("/nope"), NtSlash::Unknown);
        assert_eq!(dispatch_slash("plain"), NtSlash::Unknown);
        assert_eq!(dispatch_slash(""), NtSlash::Unknown);
    }

    #[test]
    fn test_spinner_cycles() {
        let a = spinner_frame(0);
        let b = spinner_frame(120);
        assert_ne!(a, b);
        assert_eq!(spinner_frame(0), spinner_frame(8 * 120));
    }

    #[test]
    fn test_channel_human_roundtrip() {
        let (ev_tx, ev_rx) = mpsc::channel::<UiEvent>();
        let (rep_tx, rep_rx) = mpsc::channel::<Vec<NtHumanReply>>();
        let human = NtChannelHuman::new(ev_tx, rep_rx);
        let handle = std::thread::spawn(move || {
            human.prompt(
                "目标：G",
                &[NtDemand {
                    id: "d1".to_string(),
                    kind: crate::l5_cognition::nt_crystal_core::NtDemandKind::ReviewFusion,
                    text: "复核".to_string(),
                }],
            )
        });
        // UI 侧：收到需求单 → 回一条批准
        let ev = ev_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        match ev {
            UiEvent::Demands { demands, .. } => assert_eq!(demands.len(), 1),
            _ => panic!("expected Demands"),
        }
        rep_tx
            .send(vec![NtHumanReply {
                demand_id: Some("d1".to_string()),
                text: String::new(),
                approved: true,
            }])
            .unwrap();
        let replies = handle.join().unwrap();
        assert_eq!(replies.len(), 1);
        assert!(replies[0].approved);
    }

    #[test]
    fn test_channel_human_dead_ui_returns_empty() {
        let (ev_tx, _ev_rx) = mpsc::channel::<UiEvent>();
        drop(_ev_rx);
        let (_rep_tx, rep_rx) = mpsc::channel::<Vec<NtHumanReply>>();
        let human = NtChannelHuman::new(ev_tx, rep_rx);
        // UI 端已丢：prompt 不阻塞，直接空（Stalled 语义）
        assert!(human.prompt("w", &[]).is_empty());
    }

    #[test]
    fn test_sink_forwards_and_cancel() {
        let (tx, rx) = mpsc::channel::<UiEvent>();
        let cancel = Arc::new(AtomicBool::new(false));
        let sink = NtUiSink::new(tx, cancel.clone());
        sink.on_subtask_start("st-1", "标题");
        sink.on_answer_chunk("st-1", "abc");
        sink.on_subtask_done("st-1", true);
        assert!(!sink.cancelled());
        cancel.store(true, Ordering::SeqCst);
        assert!(sink.cancelled());
        let mut kinds = Vec::new();
        for _ in 0..3 {
            match rx.recv_timeout(Duration::from_secs(2)).unwrap() {
                UiEvent::SubtaskStart { .. } => kinds.push("start"),
                UiEvent::Chunk { .. } => kinds.push("chunk"),
                UiEvent::SubtaskDone { .. } => kinds.push("done"),
                _ => kinds.push("other"),
            }
        }
        assert_eq!(kinds, vec!["start", "chunk", "done"]);
    }

    #[test]
    fn test_app_pick_pins_pool() {
        let pool = Arc::new(NtFreePoolAsk::new(vec!["m1".to_string()]));
        let mut app = NtTuiApp::new("g".to_string(), pool.clone(), vec![], vec![]);
        assert!(pool.pinned().is_none());
        app.apply_pick("m1".to_string());
        assert_eq!(pool.pinned().as_deref(), Some("m1"));
        assert!(app.state.transcript.iter().any(|l| l.contains("m1")));
    }

    /// 脚本问答桩：成功时按块流式吐出，回调 false 即取消错。
    struct ScriptAsk {
        chunks: Vec<String>,
        fail: bool,
    }

    impl crate::l5_cognition::nt_crystal_core::NtLlmAsk for ScriptAsk {
        fn ask(
            &self,
            _prompt: &str,
        ) -> Result<crate::l5_cognition::nt_crystal_core::NtLlmReply, crate::l5_cognition::nt_crystal_core::NtTaskFusionError>
        {
            if self.fail {
                return Err(crate::l5_cognition::nt_crystal_core::NtTaskFusionError::Llm(
                    "boom".to_string(),
                ));
            }
            Ok(crate::l5_cognition::nt_crystal_core::NtLlmReply {
                text: self.chunks.concat(),
                confidence: 0.8,
                model: "script".to_string(),
            })
        }

        fn ask_stream(
            &self,
            prompt: &str,
            on_chunk: &dyn Fn(&str) -> bool,
        ) -> Result<crate::l5_cognition::nt_crystal_core::NtLlmReply, crate::l5_cognition::nt_crystal_core::NtTaskFusionError>
        {
            if self.fail {
                return Err(crate::l5_cognition::nt_crystal_core::NtTaskFusionError::Llm(
                    "boom".to_string(),
                ));
            }
            let mut full = String::new();
            for c in &self.chunks {
                full.push_str(c);
                if !on_chunk(c) {
                    return Err(crate::l5_cognition::nt_crystal_core::NtTaskFusionError::Llm(
                        "cancelled".to_string(),
                    ));
                }
            }
            let _ = prompt;
            Ok(crate::l5_cognition::nt_crystal_core::NtLlmReply {
                text: full,
                confidence: 0.8,
                model: "script".to_string(),
            })
        }
    }

    /// 记录型进度接收器（断言事件序列用）。
    struct RecSink {
        events: Mutex<Vec<String>>,
    }

    impl crate::l5_cognition::nt_crystal_core::NtProgressSink for RecSink {
        fn on_subtask_start(&self, id: &str, _title: &str) {
            self.events.lock().unwrap().push(format!("start:{id}"));
        }
        fn on_answer_chunk(&self, _id: &str, delta: &str) {
            self.events.lock().unwrap().push(format!("chunk:{delta}"));
        }
        fn on_subtask_done(&self, id: &str, ok: bool) {
            self.events.lock().unwrap().push(format!("done:{id}:{ok}"));
        }
    }

    #[test]
    fn test_threaded_drive_streams_and_converges() {
        use crate::l5_cognition::nt_crystal_core::{NtCrystalTaskLoop, NtTaskLoopConfig};
        // 成功路径：chunks 流式到达 sink，单答案共识收敛，无需人类介入
        let ask = ScriptAsk {
            chunks: vec!["甲乙丙丁戊己庚辛".to_string(), "壬癸子丑".to_string()],
            fail: false,
        };
        let sink = RecSink {
            events: Mutex::new(Vec::new()),
        };
        let engine = NtCrystalTaskLoop::new(NtTaskLoopConfig::default());
        let core = CrystalCore::new("t");
        let report = engine.run_with_sink("足够长的目标描述文本内容", &core, &ask, Some(&sink));
        let ev = sink.events.lock().unwrap().clone();
        assert!(ev.iter().any(|e| e.starts_with("start:")));
        assert!(ev.contains(&"chunk:甲乙丙丁戊己庚辛".to_string()));
        assert!(ev.contains(&"chunk:壬癸子丑".to_string()));
        assert!(ev.iter().any(|e| e.starts_with("done:") && e.ends_with(":true")));
        assert!(!report.fused.text.is_empty());
    }

    #[test]
    fn test_threaded_channel_human_full_loop() {
        use crate::l5_cognition::nt_crystal_core::{
            NtDemandKind, NtInnerLoop, NtLoopStatus, NtTaskLoopConfig,
        };
        // 失败路径：retry 需求单经通道发给"UI"，人带 id 批准后关闭需求单，
        // 下轮无内需 → Converged（完整往返，不断轮不挂起）
        let ask = ScriptAsk { chunks: vec![], fail: true };
        let (ev_tx, ev_rx) = mpsc::channel::<UiEvent>();
        let (rep_tx, rep_rx) = mpsc::channel::<Vec<NtHumanReply>>();
        let human = NtChannelHuman::new(ev_tx, rep_rx);
        let sink = RecSink { events: Mutex::new(Vec::new()) };
        let handle = std::thread::spawn(move || {
            let engine = NtInnerLoop::new(NtTaskLoopConfig::default(), 3);
            let mut core = CrystalCore::new("t2");
            let outcome =
                engine.drive_with_sink("一定会炸的问题", &mut core, &ask, &human, Some(&sink));
            (outcome, core)
        });
        // UI 侧：收需求单 → 取 retry id → 带 id 批准
        let mut retry_id = None;
        for _ in 0..60 {
            match ev_rx.recv_timeout(Duration::from_secs(5)) {
                Ok(UiEvent::Demands { demands, .. }) => {
                    retry_id = demands
                        .iter()
                        .find(|d| d.kind == NtDemandKind::RetryFailed)
                        .map(|d| d.id.clone());
                    break;
                }
                Ok(_) => continue,
                Err(_) => break,
            }
        }
        let retry_id = retry_id.unwrap();
        rep_tx
            .send(vec![NtHumanReply {
                demand_id: Some(retry_id),
                text: "人工已手动执行完毕结果符合预期".to_string(),
                approved: true,
            }])
            .unwrap();
        let (outcome, core) = handle.join().unwrap();
        assert_eq!(outcome.status, NtLoopStatus::Converged);
        assert!(!core.experience.episodes.is_empty());
    }
}
