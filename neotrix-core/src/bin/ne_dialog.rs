//! NeoTrix Dialog — standalone ratatui-based dual-panel TUI dialog.
//!
//! Uses NeoTrix's internal ReasoningKernel via StandaloneEngine —
//! no external LLM required.
//!
//! Usage: ne-dialog [--stage N]

use std::fs;
use std::io;
use std::path::PathBuf;

use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame, Terminal,
};
use serde::{Deserialize, Serialize};

use neotrix::neotrix::nt_core_kernel::EVOLUTION;
use neotrix::neotrix::nt_io_standalone::StandaloneEngine;
use neotrix::neotrix::nt_world_browse::circuits_types::ReasoningMethod;

#[derive(Parser, Debug)]
#[command(
    name = "ne-dialog",
    about = "NeoTrix Dialog — ratatui TUI reasoning kernel"
)]
struct Args {
    #[arg(long, default_value_t = 9, help = "Initial evolution stage (0-18)")]
    stage: usize,
}

fn circuit_label(m: ReasoningMethod) -> &'static str {
    match m {
        ReasoningMethod::Deductive => "deductive",
        ReasoningMethod::Inductive => "inductive",
        ReasoningMethod::Abductive => "abductive",
        ReasoningMethod::Analogical => "analogical",
        ReasoningMethod::Compositional => "compositional",
        ReasoningMethod::Recursive => "recursive",
        ReasoningMethod::Adversarial => "adversarial",
        ReasoningMethod::FirstPrinciples => "first principles",
        ReasoningMethod::AutoFetch => "auto-fetch",
        ReasoningMethod::KnowledgeRetrieval => "knowledge retrieval",
        ReasoningMethod::GradientLearning => "gradient learning",
        ReasoningMethod::ArchitectureSearch => "arch search",
        ReasoningMethod::GpuCompute => "GPU compute",
        ReasoningMethod::DistributedConsensus => "distributed consensus",
        ReasoningMethod::ExperienceDistill => "exp. distill",
        ReasoningMethod::EmergentAnalysis => "emergent analysis",
        ReasoningMethod::SystemIntegration => "system integration",
        ReasoningMethod::EnsembleVoting => "ensemble voting",
        ReasoningMethod::SelfImprovement => "self-improvement",
        ReasoningMethod::SparseRouting => "sparse routing",
    }
}

#[derive(Serialize, Deserialize)]
struct Session {
    messages: Vec<[String; 2]>,
}

struct App {
    engine: StandaloneEngine,
    input: String,
    scroll: usize,
    msg: String,
    sessions_dir: PathBuf,
    running: bool,
    pending_g: bool,
}

impl App {
    fn new(stage: usize) -> Self {
        let s = stage.min(18);
        let mut engine = StandaloneEngine::new(s);
        engine.max_history = 1000;
        let sessions_dir = dirs::home_dir()
            .unwrap_or_else(|| std::env::temp_dir())
            .join(".neotrix")
            .join("sessions");
        let _ = fs::create_dir_all(&sessions_dir);
        let standalone_note = "Running in standalone mode — connect to daemon with --connect for live consciousness data".to_string();
        App {
            engine,
            input: String::new(),
            scroll: 0,
            msg: standalone_note,
            sessions_dir,
            running: true,
            pending_g: false,
        }
    }

    fn send(&mut self, text: &str) {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return;
        }
        match trimmed {
            "/exit" | "/q" => self.running = false,
            "/clear" => {
                self.engine.conversation.clear();
                self.scroll = 0;
            }
            "/stats" => self.msg = self.engine.stats(),
            cmd if cmd.starts_with("/stage") => {
                let n = cmd.split_whitespace().nth(1).and_then(|s| s.parse().ok()).unwrap_or(9).min(18);
                let kernel = neotrix::neotrix::nt_core_kernel::ReasoningKernel::new(n);
                let stage = kernel.stage;
                self.engine.kernel = kernel;
                self.msg = format!("Switched to {}", EVOLUTION[stage].label);
            }
            "/save" => {
                let ts = chrono::Local::now().format("%Y%m%d_%H%M%S");
                let path = self.sessions_dir.join(format!("session_{}.json", ts));
                let session = Session { messages: self.engine.conversation.iter().map(|(a, b)| [a.clone(), b.clone()]).collect() };
                match serde_json::to_string_pretty(&session).map_err(|e| e.to_string()).and_then(|j| fs::write(&path, j).map_err(|e| e.to_string())) {
                    Ok(_) => self.msg = format!("Saved: {}", path.display()),
                    Err(e) => self.msg = format!("Save failed: {}", e),
                }
            }
            "/load" => {
                let mut entries: Vec<_> = fs::read_dir(&self.sessions_dir)
                    .map(|d| d.filter_map(|e| e.ok()).map(|e| e.path()).collect())
                    .unwrap_or_default();
                entries.sort();
                entries.reverse();
                if let Some(path) = entries.first() {
                    match fs::read_to_string(path).and_then(|j| serde_json::from_str::<Session>(&j).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))) {
                        Ok(session) => {
                            self.engine.conversation = session.messages.into_iter().map(|[q, r]| (q, r)).collect();
                            self.scroll = self.engine.conversation.len().saturating_sub(1);
                            self.msg = format!("Loaded: {}", path.display());
                        }
                        Err(e) => self.msg = format!("Load failed: {}", e),
                    }
                } else {
                    self.msg = "No saved sessions".into();
                }
            }
            "/help" => self.msg = "/stats /stage N /clear /save /load /help /exit | j/k scroll, g top, G bottom, q quit".into(),
            _ => {
                self.engine.reason(trimmed);
                self.scroll = self.engine.conversation.len().saturating_sub(1);
            }
        }
        self.input.clear();
    }

    fn kernel_stats(&self) -> neotrix::neotrix::nt_core_kernel::KernelStats {
        self.engine.kernel.stats()
    }

    fn kernel_state(&self) -> &[f64] {
        &self.engine.kernel.state
    }
}

fn ui(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(area);

    let horiz = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(vert[0]);

    render_conversation(frame, horiz[0], app);
    render_kernel_state(frame, horiz[1], app);
    render_input(frame, vert[1], app);
    render_status(frame, vert[2], app);
}

fn render_conversation(frame: &mut Frame, area: Rect, app: &App) {
    let mut lines: Vec<Line> = Vec::new();
    for (i, (q, r)) in app.engine.conversation.iter().enumerate() {
        lines.push(Line::from(vec![
            Span::styled(format!("{:>3}", i + 1), Style::new().fg(Color::DarkGray)),
            Span::styled(
                " Q:",
                Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(q.as_str(), Style::new().fg(Color::Cyan)),
        ]));
        for rline in r.split('\n') {
            lines.push(Line::from(vec![
                Span::raw("     "),
                Span::styled(rline, Style::new().fg(Color::White)),
            ]));
        }
        lines.push(Line::from(""));
    }
    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No messages yet. Type a question below.",
            Style::new().fg(Color::DarkGray),
        )));
    }

    let content_h = lines.len();
    let view_h = (area.height as usize).saturating_sub(2);
    let max_scroll = content_h.saturating_sub(view_h);
    let scroll = app.scroll.min(max_scroll);

    frame.render_widget(
        Paragraph::new(Text::from(lines))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Conversation ")
                    .border_type(BorderType::Rounded)
                    .border_style(Style::new().fg(Color::Cyan)),
            )
            .scroll((scroll as u16, 0)),
        area,
    );
}

fn render_kernel_state(frame: &mut Frame, area: Rect, app: &App) {
    let stats = app.kernel_stats();
    let state = app.kernel_state();

    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(1)])
        .split(area);

    let mut top = Vec::new();
    top.push(Line::from(vec![
        Span::styled("Stage: ", Style::new().fg(Color::Yellow)),
        Span::styled(
            EVOLUTION[stats.stage].label,
            Style::new().fg(Color::White).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" — "),
        Span::styled(
            EVOLUTION[stats.stage].description,
            Style::new().fg(Color::White),
        ),
    ]));

    let e = stats.energy.clamp(0.0, 1.0);
    let bar_len = (e * 20.0) as usize;
    let bar: String = "▓".repeat(bar_len) + &"░".repeat(20usize.saturating_sub(bar_len));
    let ecolor = if e < 0.3 {
        Color::Red
    } else if e < 0.7 {
        Color::Yellow
    } else {
        Color::Green
    };
    top.push(Line::from(vec![
        Span::styled("Energy: ", Style::new().fg(Color::Yellow)),
        Span::styled(format!("{:.2} ", e), Style::new().fg(ecolor)),
        Span::styled(bar, Style::new().fg(ecolor)),
    ]));

    let circuits: Vec<&str> = stats.active.iter().map(|m| circuit_label(*m)).collect();
    top.push(Line::from(vec![
        Span::styled("Active: ", Style::new().fg(Color::Yellow)),
        Span::styled(
            if circuits.is_empty() {
                "—".into()
            } else {
                circuits.join(", ")
            },
            Style::new().fg(Color::White),
        ),
    ]));
    top.push(Line::from(""));
    top.push(Line::from(vec![
        Span::styled("State: ", Style::new().fg(Color::Yellow)),
        Span::styled(
            format!("{} dims", state.len()),
            Style::new().fg(Color::White),
        ),
    ]));
    let active_dims = state.iter().filter(|x| x.abs() > 0.3).count();
    top.push(Line::from(vec![
        Span::styled("Active: ", Style::new().fg(Color::Yellow)),
        Span::styled(
            format!("{}/{} (>0.3)", active_dims, state.len()),
            Style::new().fg(if active_dims > state.len() / 4 {
                Color::Green
            } else {
                Color::DarkGray
            }),
        ),
    ]));
    top.push(Line::from(vec![
        Span::styled("Msg hist: ", Style::new().fg(Color::Yellow)),
        Span::styled(
            app.engine.conversation.len().to_string(),
            Style::new().fg(Color::White),
        ),
    ]));

    frame.render_widget(
        Paragraph::new(Text::from(top)).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Kernel State ")
                .border_type(BorderType::Rounded),
        ),
        vert[0],
    );

    let mut dims = Vec::new();
    dims.push(Line::from(Span::styled(
        "Dim activation (groups of 16):",
        Style::new().fg(Color::Yellow),
    )));
    for (ci, chunk) in state.chunks(16).enumerate() {
        let avg = chunk.iter().map(|x| x.abs()).sum::<f64>() / chunk.len().max(1) as f64;
        let pct = (avg * 100.0) as u8;
        let n = (avg * 10.0) as usize;
        let color = if avg > 0.5 {
            Color::Green
        } else if avg > 0.2 {
            Color::Yellow
        } else {
            Color::DarkGray
        };
        dims.push(Line::from(vec![
            Span::styled(format!("{:>3}", ci * 16), Style::new().fg(Color::DarkGray)),
            Span::raw(" "),
            Span::styled(format!("{:>3}%", pct), Style::new().fg(color)),
            Span::raw(" "),
            Span::styled("█".repeat(n), Style::new().fg(color)),
        ]));
    }

    frame.render_widget(
        Paragraph::new(Text::from(dims)).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Dimensions ")
                .border_type(BorderType::Rounded),
        ),
        vert[1],
    );
}

fn render_input(frame: &mut Frame, area: Rect, app: &App) {
    let text = if app.input.is_empty() {
        Text::from(Line::from(Span::styled(
            " Type a question...",
            Style::new().fg(Color::DarkGray),
        )))
    } else {
        Text::from(Line::from(Span::styled(
            format!(" {}", app.input),
            Style::new().fg(Color::Green),
        )))
    };
    frame.render_widget(
        Paragraph::new(text).block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::new().fg(Color::DarkGray)),
        ),
        area,
    );
}

fn render_status(frame: &mut Frame, area: Rect, app: &App) {
    if !app.msg.is_empty() {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                format!(" {}", app.msg),
                Style::new().fg(Color::Yellow),
            )))
            .style(Style::new().bg(Color::Black)),
            area,
        );
        return;
    }
    let stats = app.kernel_stats();
    let text = format!(
        " standalone | stage={}/{} | energy={:.2} | circuits={} | msgs={} | /help",
        stats.stage,
        EVOLUTION.len() - 1,
        stats.energy,
        stats.active.len(),
        app.engine.conversation.len(),
    );
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            text,
            Style::new().fg(Color::DarkGray),
        )))
        .style(Style::new().bg(Color::Black)),
        area,
    );
}

fn handle_event(app: &mut App, ev: Event) {
    if let Event::Key(key) = ev {
        if key.kind != KeyEventKind::Press {
            return;
        }
        if app.pending_g {
            app.pending_g = false;
            if key.code == KeyCode::Char('g') {
                app.scroll = 0;
                return;
            }
            app.input.push('g');
        }
        match key.code {
            KeyCode::Char('q') if app.input.is_empty() => app.running = false,
            KeyCode::Char('j') if app.input.is_empty() => {
                let view_h = 20usize;
                app.scroll = app.scroll.saturating_add(view_h.saturating_sub(2));
            }
            KeyCode::Char('k') if app.input.is_empty() => {
                let view_h = 20usize;
                app.scroll = app.scroll.saturating_sub(view_h.saturating_sub(2));
            }
            KeyCode::Char('g') if app.input.is_empty() => app.pending_g = true,
            KeyCode::Char('G') if app.input.is_empty() => app.scroll = usize::MAX,
            KeyCode::Enter => {
                let text = std::mem::take(&mut app.input);
                app.send(&text);
            }
            KeyCode::Char(c) => app.input.push(c),
            KeyCode::Backspace => {
                app.input.pop();
            }
            KeyCode::Esc => app.input.clear(),
            _ => {}
        }
    }
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(args.stage);
    while app.running {
        terminal.draw(|f| ui(f, &app))?;
        if event::poll(std::time::Duration::from_millis(100))? {
            handle_event(&mut app, event::read()?);
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
