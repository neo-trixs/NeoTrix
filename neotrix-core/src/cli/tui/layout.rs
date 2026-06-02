use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    style::{Style, Color, Modifier},
    text::{Line, Span, Text},
    Frame,
};
use super::app::TuiApp;
use super::output::{render_markdown, role_style, render_thinking_block};
use super::themes::Theme;
use crate::cli::cost_tracker::COST_TRACKER;

/// 计算五面板布局（含目标状态面板）
pub fn compute_layout(area: Rect) -> (Rect, Rect, Rect, Rect, Rect) {
    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(80),
        ])
        .split(area);

    let left = main[0];
    let right = main[1];

    let right_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(6),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(right);

    (left, right_split[0], right_split[1], right_split[2], right_split[3])
}

/// 渲染左面板 — 会话列表
pub fn render_session_list(frame: &mut Frame, area: Rect, app: &TuiApp, theme: &Theme) {
    let items: Vec<ListItem> = app.sessions.iter().enumerate().map(|(i, s)| {
        let prefix = if i == app.active_session { "▶ " } else { "  " };
        let n = s.messages.len();
        ListItem::new(format!("{}{} ({}条)", prefix, s.name, n))
    }).collect();

    let list = List::new(items)
        .block(Block::default()
            .title(format!("会话 {}", app.sessions.len()))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.secondary)))
        .highlight_style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
        .style(Style::default().fg(theme.primary));

    frame.render_widget(list, area);
}

/// 渲染右上面板 — 聊天输出（Markdown 渲染 + Thinking 折叠 + 工具调用可视化）
pub fn render_chat_panel(frame: &mut Frame, area: Rect, app: &TuiApp, theme: &Theme) {
    let session = app.active_session();
    let mut lines = Vec::new();

    for (msg_idx, msg) in session.messages.iter().enumerate() {
        let style = role_style(&msg.role);
        lines.push(Line::from(Span::styled(format!("[{}]", msg.role), style)));

        // Tool calls visualization
        for tc in &msg.tool_calls {
            let icon = if tc.success { "🛠️" } else { "⚠️" };
            let dur = if tc.duration_ms > 0 { format!(" ({}ms)", tc.duration_ms) } else { String::new() };
            lines.push(Line::from(vec![
                Span::styled(icon, Style::default().fg(theme.accent)),
                Span::styled(format!(" {}", tc.name), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                Span::styled(dur, Style::default().fg(theme.secondary)),
            ]));
            if !tc.args.is_empty() {
                lines.push(Line::from(Span::styled(
                    format!("  └ args: {}", tc.args),
                    Style::default().fg(theme.secondary),
                )));
            }
        }

        // Image attachment indicator
        if let Some(ref img_name) = msg.image_name {
            lines.push(Line::from(Span::styled(
                format!(" [📷 {}]", img_name),
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
            )));
        }

        // Thinking blocks (collapsible)
        let key = format!("{}:{}", app.active_session, msg_idx);
        let expanded = app.thinking_expanded.contains(&key);
        if !msg.thinking_blocks.is_empty() {
            let indicator = if expanded { "▼" } else { "▶" };
            lines.push(Line::from(Span::styled(
                format!(" {} Thinking (press t to toggle)", indicator),
                Style::default().fg(theme.highlight).add_modifier(Modifier::ITALIC),
            )));
            if expanded {
                lines.extend(render_thinking_block(&msg.thinking_blocks));
            }
        }

        // Main content
        lines.extend(render_markdown(&msg.content));
        lines.push(Line::from(""));
    }

    // 流式输出内容（尚未持久化到消息列表）
    if app.streaming {
        let style = role_style(&app.streaming_role);
        lines.push(Line::from(Span::styled(format!("[{}]", app.streaming_role), style)));
        lines.extend(render_markdown(&app.streaming_text));
        lines.push(Line::from(Span::styled(
            " ▌",
            Style::default().fg(theme.highlight).add_modifier(Modifier::SLOW_BLINK),
        )));
    }

    let paragraph = Paragraph::new(Text::from(lines))
        .block(Block::default()
            .title(if app.agent_busy { "对话 [思考中...]" } else { "对话" })
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.secondary)))
        .wrap(Wrap { trim: false })
        .scroll((app.scroll_offset as u16, 0));

    frame.render_widget(paragraph, area);
}

/// 渲染右下面板 — 输入行（支持多行模式 + 历史搜索）
pub fn render_input_panel(frame: &mut Frame, area: Rect, app: &TuiApp, theme: &Theme) {
    if app.command_history.search_active {
        render_history_search(frame, area, app, theme);
        return;
    }

    let placeholder = if app.multi_line {
        "多行模式 (Enter=换行, Ctrl+Enter=发送)"
    } else {
        "输入消息... (Tab补全 ↑↓历史 Ctrl+R搜索 Alt+Enter=多行 Ctrl+L=清屏)"
    };

    let text = if app.input.is_empty() {
        Text::from(Line::from(Span::styled(placeholder, Style::default().fg(theme.secondary))))
    } else {
        let lines: Vec<Line> = app.input.lines().map(|l| Line::from(Span::raw(l.to_string()))).collect();
        Text::from(lines)
    };

    let hist_indicator = match app.command_history.position {
        Some(pos) => format!(" (history {}/{})", pos + 1, app.command_history.entries.len()),
        None => String::new(),
    };
    let title = if app.multi_line { format!("输入 [多行]{}", hist_indicator) } else { format!("输入{}", hist_indicator) };
    let paragraph = Paragraph::new(text)
        .block(Block::default().title(title).borders(Borders::ALL).border_style(Style::default().fg(theme.secondary)))
        .style(Style::default().fg(theme.primary));

    frame.render_widget(paragraph, area);

    // 光标位置
    let last_line_len = app.input.lines().last().map(|l| l.len()).unwrap_or(0);
    frame.set_cursor_position((
        area.x + 1 + last_line_len as u16,
        area.y + 1 + app.input.lines().count().saturating_sub(1) as u16,
    ));
}

/// 渲染 Ctrl+R 历史搜索面板
fn render_history_search(frame: &mut Frame, area: Rect, app: &TuiApp, theme: &Theme) {
    let total = app.command_history.search_results.len();
    let sel = app.command_history.search_selection;
    let query = &app.command_history.search_query;

    let mut lines = Vec::new();
    lines.push(Line::from(vec![
        Span::styled("⌕ ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled(query.clone(), Style::default().fg(theme.primary)),
        Span::styled(format!("  ({} matches | Ctrl+R cycle | Esc cancel)", total), Style::default().fg(theme.secondary)),
    ]));

    // Show up to 5 matching entries
    let max_show = (area.height.saturating_sub(2)) as usize;
    for i in 0..max_show.min(total).min(5) {
        let idx = app.command_history.search_results.get(i);
        if let Some(&entry_idx) = idx {
            let entry = &app.command_history.entries[entry_idx];
            let display = if entry.len() > area.width.saturating_sub(4) as usize {
                format!("{}…", &entry[..area.width.saturating_sub(7) as usize])
            } else {
                entry.clone()
            };
            let style = if i == sel {
                Style::default().fg(theme.bg).bg(theme.accent).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.primary)
            };
            lines.push(Line::from(Span::styled(format!(" {}", display), style)));
        }
    }

    let paragraph = Paragraph::new(Text::from(lines))
        .block(Block::default()
            .title(" History Search ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent)))
        .style(Style::default().fg(theme.primary));

    frame.render_widget(paragraph, area);
}

/// 渲染目标状态面板 — 显示当前 GoalLoop 状态 + Agent 摘要
pub fn render_goal_panel(frame: &mut Frame, area: Rect, app: &TuiApp, agent_team_summary: &str, theme: &Theme) {
    let (mut lines, title) = if app.goal_display.has_goal {
        let g = &app.goal_display;
        let pct = if g.max_iterations > 0 {
            (g.iterations as f64 / g.max_iterations as f64) * 100.0
        } else {
            0.0
        };
        let mut lines = vec![
            Line::from(vec![
                Span::styled(
                    format!(" {} {} ", g.state_icon, g.state_label),
                    Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled(" Goal: ", Style::default().fg(theme.secondary)),
                Span::raw(g.description.clone()),
            ]),
            Line::from(vec![
                Span::styled(" Iter: ", Style::default().fg(theme.secondary)),
                Span::raw(format!("{}/{} ({:.0}%)", g.iterations, g.max_iterations, pct)),
                Span::styled(" Score: ", Style::default().fg(theme.secondary)),
                Span::raw(format!("{:.2}→{:.2}", g.score_before, g.score_current)),
            ]),
            Line::from(vec![
                Span::styled(" Stalled: ", Style::default().fg(theme.secondary)),
                Span::raw(format!("{}x", g.stalled_count)),
            ]),
        ];
        if g.queue_count > 0 || g.completed_count > 0 {
            let suffix = format!("Q:{} C:{}", g.queue_count, g.completed_count);
            lines.push(Line::from(Span::styled(
                format!(" {} ", suffix),
                Style::default().fg(theme.secondary),
            )));
        }
        (lines, " Goal ")
    } else {
        let extra = if app.goal_display.queue_count > 0 || app.goal_display.completed_count > 0 {
            format!(" (Q:{}, C:{})", app.goal_display.queue_count, app.goal_display.completed_count)
        } else {
            String::new()
        };
        let lines = vec![
            Line::from(Span::styled(
                format!(" No active goal.{}. Use /goal <desc> to start.", extra),
                Style::default().fg(theme.secondary),
            )),
        ];
        (lines, " No Goal ")
    };

    if !agent_team_summary.is_empty() {
        lines.push(Line::from(vec![
            Span::styled(" Agents: ", Style::default().fg(theme.secondary)),
            Span::raw(agent_team_summary),
        ]));
    }

    let paragraph = Paragraph::new(Text::from(lines))
        .block(Block::default().title(title).borders(Borders::ALL).border_style(Style::default().fg(theme.secondary)))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

/// 渲染 diff 查看面板
pub fn render_diff_viewer(frame: &mut Frame, area: Rect, app: &TuiApp, theme: &Theme) {
    let dv = match &app.diff_viewer {
        Some(dv) => dv,
        None => {
            let paragraph = Paragraph::new("No diff loaded")
                .block(Block::default()
                    .title(" Diff Viewer ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.secondary)));
            frame.render_widget(paragraph, area);
            return;
        }
    };

    let inner_height = (area.height.saturating_sub(2)) as usize;
    let all_lines = dv.all_rendered_lines();
    let visible: Vec<Line> = all_lines.iter().skip(dv.scroll_offset).take(inner_height).cloned().collect();
    let file_count = dv.blocks.len();

    let paragraph = Paragraph::new(Text::from(visible))
        .block(Block::default()
            .title(format!(" Diff Viewer — {} file(s) (j/k scroll, q close) ", file_count))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent)))
        .style(Style::default().fg(theme.primary));

    frame.render_widget(paragraph, area);
}

/// 渲染底部状态栏（增强版，含费用追踪 + 沙箱指示器）
pub fn render_status_bar(frame: &mut Frame, area: Rect, app: &TuiApp, theme: &Theme) {
    let sandbox_indicator = {
        let label = app.sandbox_mode.label();
        if label.is_empty() {
            String::new()
        } else {
            format!(" {} |", label)
        }
    };

    let vim_prefix = if app.vim_mode.is_enabled() {
        format!("-- {} -- ", app.vim_mode.mode.label())
    } else {
        String::new()
    };
    let session_info = format!(" 会话{}/{}", app.active_session + 1, app.sessions.len());
    let ws_info = format!(" WS:{} ({})", app.workspace_name, app.workspace_count);

    let cost_info = {
        if let Ok(tracker) = COST_TRACKER.lock() {
            let line = tracker.status_line();
            if line.len() > 60 {
                format!(" | {}", &line[..57])
            } else {
                format!(" | {}", line)
            }
        } else {
            String::new()
        }
    };

    let status_text = if app.streaming {
        format!("{}{}{} 生成中... {:.0} tok/s | Tokens: {}{} |{} | {}",
            sandbox_indicator, vim_prefix, app.status_text, app.tokens_per_sec, app.token_count, cost_info, ws_info, session_info)
    } else if app.agent_busy {
        format!("{}{}{} 思考中 | Tokens: {}{} |{} | {}", sandbox_indicator, vim_prefix, app.status_text, app.token_count, cost_info, ws_info, session_info)
    } else {
        format!("{}{}{} 就绪 | Tokens: {}{} |{} | {}", sandbox_indicator, vim_prefix, app.status_text, app.token_count, cost_info, ws_info, session_info)
    };

    let is_sandbox = app.sandbox_mode != crate::cli::sandbox::SandboxMode::Disabled;
    let status = Paragraph::new(Line::from(Span::raw(status_text)))
        .style(Style::default()
            .bg(if app.streaming { theme.accent }
                else if app.agent_busy { theme.secondary }
                else if is_sandbox { Color::Red }
                else { theme.primary })
            .fg(Color::White)
            .add_modifier(Modifier::BOLD));
    frame.render_widget(status, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;
    use ratatui::layout::Rect;

    fn test_theme() -> Theme {
        Theme {
            name: "test".to_string(),
            bg: Color::Black,
            accent: Color::Magenta,
            primary: Color::Red,
            secondary: Color::Yellow,
            highlight: Color::Cyan,
        }
    }

    #[test]
    fn test_compute_layout_returns_five_areas() {
        let area = Rect { x: 0, y: 0, width: 100, height: 50 };
        let (left, chat, goal, input, status) = compute_layout(area);
        assert_eq!(left.x, 0);
        assert_eq!(left.y, 0);
        assert_eq!(left.width, 20);
        assert_eq!(left.height, 50);
        let right_total_height = chat.height + goal.height + input.height + status.height;
        assert_eq!(right_total_height, 50);
    }

    #[test]
    fn test_compute_layout_left_right_split() {
        let area = Rect { x: 0, y: 0, width: 80, height: 24 };
        let (left, _chat, _goal, _input, _status) = compute_layout(area);
        assert_eq!(left.width, 16);
        assert_eq!(left.height, 24);
    }

    #[test]
    fn test_compute_layout_all_areas_nonzero() {
        let area = Rect { x: 0, y: 0, width: 100, height: 50 };
        let (left, chat, _goal, input, status) = compute_layout(area);
        assert!(left.width > 0);
        assert!(left.height > 0);
        assert!(chat.width > 0);
        assert!(chat.height > 0);
        assert!(input.width > 0);
        assert!(status.width > 0);
    }

    #[test]
    fn test_render_status_bar_style() {
        let theme = test_theme();
        assert_eq!(theme.status_style(true, false).bg, Some(Color::Magenta));
        assert_eq!(theme.status_style(false, true).bg, Some(Color::Yellow));
        assert_eq!(theme.status_style(false, false).bg, Some(Color::Red));
    }
}
