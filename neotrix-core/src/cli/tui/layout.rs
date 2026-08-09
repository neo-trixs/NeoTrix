use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Paragraph, Wrap},
    style::{Style, Color, Modifier},
    text::{Line, Span, Text},
    Frame,
};
use super::app::TuiApp;
use super::output::{render_markdown, render_thinking_block};
use super::themes::Theme;
use crate::cli::cost_tracker::COST_TRACKER;

/// 计算布局：可选左侧会话面板 + 聊天 + 输入 + 状态栏。
/// `show_sessions` 为 true 时切出左侧 20% 会话列表；否则全宽聊天区。
/// `input_lines` 为输入区当前行数（多行输入时自适应增高，上限 8 行）。
/// 返回 `(Option<会话面板>, 聊天区, 输入区, 状态栏)`。
pub fn compute_layout(area: Rect, show_sessions: bool, input_lines: usize) -> (Option<Rect>, Rect, Rect, Rect) {
    let main = if show_sessions {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(0), Constraint::Percentage(100)])
            .split(area)
    };
    let (left, right) = (main[0], main[1]);
    // 输入区高度：至少 3 行，随内容增长，上限 8 行
    let input_h = (input_lines.max(1) + 2).clamp(3, 8) as u16;
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(input_h),
            Constraint::Length(1),
        ])
        .split(right);
    let left_opt = if show_sessions { Some(left) } else { None };
    (left_opt, vertical[0], vertical[1], vertical[2])
}

/// 渲染左侧会话列表面板 — 无边框，逐行列出 session 名称与消息数。
pub fn render_session_list(frame: &mut Frame, area: Rect, app: &TuiApp, theme: &Theme) {
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(vec![
        Span::styled("会话", Style::default().fg(theme.highlight).add_modifier(Modifier::BOLD)),
        Span::styled("  Ctrl+S 隐藏", Style::default().fg(theme.secondary).add_modifier(Modifier::ITALIC)),
    ]));
    lines.push(Line::from(""));
    for (i, session) in app.sessions.iter().enumerate() {
        let active = i == app.active_session;
        let marker = if active { "▶ " } else { "  " };
        let name = if session.name.len() > 18 {
            format!("{}…", &session.name[..17])
        } else {
            session.name.clone()
        };
        let line = Line::from(vec![
            Span::styled(marker, Style::default().fg(theme.accent)),
            Span::styled(
                format!("{} ({})", name, session.messages.len()),
                if active {
                    Style::default().fg(theme.highlight).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.primary)
                },
            ),
        ]);
        lines.push(if active { apply_bg(line, theme.user_msg_bg) } else { line });
    }
    let paragraph = Paragraph::new(Text::from(lines));
    frame.render_widget(paragraph, area);
}

/// 渲染聊天区 — 无边框消息流（opencode 风格）。
/// - user: `❯ 内容`（accent 前缀）
/// - assistant: `assistant` 标签 + 工具调用折叠行 + thinking 折叠 + markdown 正文
/// - system/error: 特殊样式
/// - 目标状态内联（有 goal 时顶部一行）
/// 对 Line 的所有 Span 应用背景色（用于消息气泡效果）。
fn apply_bg(line: Line, bg: Color) -> Line {
    Line::from(line.spans.into_iter().map(|span| {
        let mut style = span.style;
        style = style.bg(bg);
        Span::styled(span.content, style)
    }).collect::<Vec<_>>())
}

pub fn render_chat_panel(frame: &mut Frame, area: Rect, app: &TuiApp, theme: &Theme) {
    let session = app.active_session();
    let mut lines: Vec<Line> = Vec::new();

    // 目标状态内联（有 goal 时显示一行，无 goal 不占空间）
    if app.goal_display.has_goal {
        lines.push(render_goal_line(app, theme));
        lines.push(Line::from(""));
    }

    for (msg_idx, msg) in session.messages.iter().enumerate() {
        match msg.role.as_str() {
            "user" => {
                let header = Line::from(vec![
                    Span::styled("You: ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                    Span::raw(msg.content.clone()),
                ]);
                lines.push(apply_bg(header, theme.user_msg_bg));
            }
            "assistant" => {
                let model_suffix = msg.model.as_ref().map(|m| format!(" ({})", m)).unwrap_or_default();
                let header = Line::from(vec![
                    Span::styled(format!("assistant{}", model_suffix), Style::default().fg(theme.highlight).add_modifier(Modifier::BOLD)),
                ]);
                lines.push(apply_bg(header, theme.assistant_msg_bg));

                // 工具调用（折叠行；x 键展开最后一条的 args）
                for (tc_idx, tc) in msg.tool_calls.iter().enumerate() {
                    let icon = if tc.success { "✳" } else { "⚠" };
                    let dur = if tc.duration_ms > 0 {
                        format!(" ({}ms)", tc.duration_ms)
                    } else {
                        String::new()
                    };
                    let tc_key = format!("{}:{}:{}", app.active_session, msg_idx, tc_idx);
                    let expanded = app.tool_calls_expanded.contains(&tc_key);
                    let toggle_hint = if expanded { "▼" } else { "▶" };
                    let tool_line = Line::from(vec![
                        Span::styled(format!("  {} {} ", toggle_hint, icon), Style::default().fg(theme.accent)),
                        Span::styled(tc.name.clone(), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                        Span::styled(dur, Style::default().fg(theme.secondary)),
                    ]);
                    lines.push(apply_bg(tool_line, theme.assistant_msg_bg));
                    if expanded && !tc.args.is_empty() {
                        let args_line = Line::from(Span::styled(
                            format!("    {}", tc.args),
                            Style::default().fg(theme.secondary),
                        ));
                        lines.push(apply_bg(args_line, theme.assistant_msg_bg));
                    }
                }

                // 图片附件指示
                if let Some(ref img_name) = msg.image_name {
                    let img_line = Line::from(Span::styled(
                        format!("  [📷 {}]", img_name),
                        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                    ));
                    lines.push(apply_bg(img_line, theme.assistant_msg_bg));
                }

                // thinking 折叠
                let key = format!("{}:{}", app.active_session, msg_idx);
                let expanded = app.thinking_expanded.contains(&key);
                if !msg.thinking_blocks.is_empty() {
                    let indicator = if expanded { "▼" } else { "▶" };
                    let think_line = Line::from(Span::styled(
                        format!("  {} thinking (t)", indicator),
                        Style::default().fg(theme.secondary).add_modifier(Modifier::ITALIC),
                    ));
                    lines.push(apply_bg(think_line, theme.assistant_msg_bg));
                    if expanded {
                        for think_block_line in render_thinking_block(&msg.thinking_blocks) {
                            lines.push(apply_bg(think_block_line, theme.assistant_msg_bg));
                        }
                    }
                }

                // 正文 - 需要对 render_markdown 返回的每行应用背景
                for md_line in render_markdown(&msg.content) {
                    lines.push(apply_bg(md_line, theme.assistant_msg_bg));
                }
            }
            "system" => {
                lines.push(Line::from(Span::styled(
                    format!("[system] {}", msg.content),
                    Style::default().fg(theme.secondary).add_modifier(Modifier::ITALIC),
                )));
            }
            "error" => {
                lines.push(Line::from(Span::styled(
                    format!("[error] {}", msg.content),
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )));
            }
            _ => {
                for md_line in render_markdown(&msg.content) {
                    lines.push(md_line);
                }
            }
        }
        // 消息分隔线（opencode 风格，弱化视觉噪音）
        lines.push(Line::from(Span::styled(
            "─".repeat(40),
            Style::default().fg(theme.code_bg),
        )));
    }

    // 流式输出（尚未持久化）——使用增量渲染器预生成的 Lines，避免闪烁
    if app.streaming {
        let model_suffix = app.streaming_model.as_ref().map(|m| format!(" ({})", m)).unwrap_or_default();
        let stream_header = Line::from(vec![
            Span::styled(format!("assistant{}", model_suffix), Style::default().fg(theme.highlight).add_modifier(Modifier::BOLD)),
        ]);
        lines.push(apply_bg(stream_header, theme.assistant_msg_bg));
        // 使用预渲染的增量 Lines
        for rendered_line in &app.streaming_rendered_lines {
            lines.push(apply_bg(rendered_line.clone(), theme.assistant_msg_bg));
        }
        lines.push(apply_bg(Line::from(Span::styled(
            " ▌",
            Style::default().fg(theme.highlight).add_modifier(Modifier::SLOW_BLINK),
        )), theme.assistant_msg_bg));
    }

    let paragraph = Paragraph::new(Text::from(lines))
        .wrap(Wrap { trim: false })
        .scroll((app.scroll_offset as u16, 0));

    frame.render_widget(paragraph, area);
}

/// 渲染输入区 — 无边框，`❯ ` 提示符（opencode 风格）。
/// 多行输入逐行渲染；历史搜索激活时覆盖为搜索面板。
pub fn render_input_panel(frame: &mut Frame, area: Rect, app: &TuiApp, theme: &Theme) {
    if app.command_history.search_active {
        render_history_search(frame, area, app, theme);
        return;
    }

    let prompt = Span::styled("❯ ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD));
    let mut text_lines: Vec<Line> = Vec::new();

    if app.input.is_empty() {
        let placeholder = if app.multi_line {
            "多行模式 (Enter=换行, Ctrl+Enter=发送)"
        } else {
            "输入消息... (Enter 发送 | ↑↓ 历史 | Ctrl+R 搜索 | Alt+E 多行 | Ctrl+L 清屏)"
        };
        text_lines.push(Line::from(vec![
            prompt.clone(),
            Span::styled(placeholder, Style::default().fg(theme.secondary)),
        ]));
    } else {
        let mut first = true;
        for line in app.input.lines() {
            if first {
                // slash 命令高亮（opencode 风格）
                if line.starts_with('/') {
                    text_lines.push(Line::from(vec![
                        prompt.clone(),
                        Span::styled(line.to_string(), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                    ]));
                } else {
                    text_lines.push(Line::from(vec![prompt.clone(), Span::raw(line.to_string())]));
                }
                first = false;
            } else {
                text_lines.push(Line::from(Span::raw(line.to_string())));
            }
        }
    }

    if app.multi_line {
        text_lines.push(Line::from(Span::styled(
            "[多行模式]",
            Style::default().fg(theme.secondary).add_modifier(Modifier::ITALIC),
        )));
    }

    let paragraph = Paragraph::new(Text::from(text_lines))
        .style(Style::default().fg(theme.primary));
    frame.render_widget(paragraph, area);

    // 光标位置：基于 cursor 字节索引 + 显示宽度（CJK 宽字符按 2 列计）。
    let (cursor_col, cursor_row) = cursor_position(&app.input, app.cursor);
    let x = if cursor_row == 0 {
        area.x + 2 + cursor_col as u16 // "❯ " 前缀 2 列
    } else {
        area.x + cursor_col as u16
    };
    frame.set_cursor_position((x, area.y + cursor_row as u16));
}

/// 计算输入文本中光标 (字节索引) 对应的 (列, 行)，按显示宽度处理 CJK。
/// 光标落在字符中间字节时，该字符不计入（光标在其之前）。
fn cursor_position(input: &str, cursor: usize) -> (usize, usize) {
    let mut col = 0usize;
    let mut row = 0usize;
    let mut byte_idx = 0usize;
    for ch in input.chars() {
        if byte_idx + ch.len_utf8() > cursor {
            break;
        }
        if ch == '\n' {
            row += 1;
            col = 0;
        } else {
            col += display_width(ch);
        }
        byte_idx += ch.len_utf8();
    }
    (col, row)
}

/// 字符显示宽度：CJK/全角按 2 列，其余按 1 列。
fn display_width(ch: char) -> usize {
    if is_wide_char(ch) { 2 } else { 1 }
}

/// 判断字符是否为 CJK 宽字符（东亚全角）。
fn is_wide_char(ch: char) -> bool {
    let c = ch as u32;
    // CJK Unified Ideographs + Ext A/B + 全角标点 + 假名 + 谚文 + 兼容表意
    (0x1100..=0x115F).contains(&c)      // Hangul Jamo
        || (0x2E80..=0x303E).contains(&c) // CJK Radicals..CJK Symbols
        || (0x3041..=0x33FF).contains(&c) // Hiragana..CJK Compatibility
        || (0x3400..=0x4DBF).contains(&c) // CJK Ext A
        || (0x4E00..=0x9FFF).contains(&c) // CJK Unified
        || (0xA000..=0xA4CF).contains(&c) // Yi
        || (0xAC00..=0xD7A3).contains(&c) // Hangul Syllables
        || (0xF900..=0xFAFF).contains(&c) // CJK Compatibility Ideographs
        || (0xFE30..=0xFE4F).contains(&c) // CJK Compatibility Forms
        || (0xFF00..=0xFF60).contains(&c) // Fullwidth Forms
        || (0xFFE0..=0xFFE6).contains(&c) // Fullwidth Signs
}

/// 渲染 Ctrl+R 历史搜索面板（无边框，opencode 风格）。
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
                let limit = area.width.saturating_sub(7) as usize;
                let mut cut = limit.min(entry.len());
                while !entry.is_char_boundary(cut) {
                    cut -= 1;
                }
                format!("{}…", &entry[..cut])
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
        .style(Style::default().fg(theme.primary));

    frame.render_widget(paragraph, area);
}

/// 目标状态内联行（有 goal 时显示在聊天区顶部）。
fn render_goal_line(app: &TuiApp, theme: &Theme) -> Line<'static> {
    let g = &app.goal_display;
    let pct = if g.max_iterations > 0 {
        (g.iterations as f64 / g.max_iterations as f64) * 100.0
    } else {
        0.0
    };
    let mut spans = vec![
        Span::styled(
            format!(" {} {} ", g.state_icon, g.state_label),
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Goal: ", Style::default().fg(theme.secondary)),
        Span::raw(g.description.clone()),
        Span::styled(
            format!(" [Iter {}/{} {:.0}%]", g.iterations, g.max_iterations, pct),
            Style::default().fg(theme.secondary),
        ),
        Span::styled(
            format!(" [Score {:.2}→{:.2}]", g.score_before, g.score_current),
            Style::default().fg(theme.secondary),
        ),
    ];
    if g.queue_count > 0 || g.completed_count > 0 {
        spans.push(Span::styled(
            format!(" [Q:{} C:{}]", g.queue_count, g.completed_count),
            Style::default().fg(theme.secondary),
        ));
    }
    Line::from(spans)
}

/// 渲染底部状态栏 — 无边框信息行（左状态 + 右信息）。
pub fn render_status_bar(frame: &mut Frame, area: Rect, app: &TuiApp, theme: &Theme) {
    // 左段：状态
    let status = if app.streaming {
        format!("生成中 {:.0} tok/s", app.tokens_per_sec)
    } else if app.agent_busy {
        "思考中".to_string()
    } else {
        "就绪".to_string()
    };

    // vim 模式指示
    let vim = if app.vim_mode.is_enabled() {
        format!(" --{}-- ", app.vim_mode.mode.label())
    } else {
        String::new()
    };

    // 沙箱指示
    let sandbox = {
        let label = app.sandbox_mode.label();
        if label.is_empty() {
            String::new()
        } else {
            format!(" | {}", label)
        }
    };

    // 右段：会话 / 工作区 / token / 费用
    let session_info = format!("会话 {}/{}", app.active_session + 1, app.sessions.len());
    let ws_info = format!("WS:{} ({})", app.workspace_name, app.workspace_count);
    let cost_info = {
        if let Ok(tracker) = COST_TRACKER.lock() {
            let line = tracker.status_line();
            if line.len() > 40 {
                format!(" | {}", &line[..37])
            } else {
                format!(" | {}", line)
            }
        } else {
            String::new()
        }
    };
    let tokens = format!("Tokens:{}", app.token_count);

    let left = format!("{}{}{}", status, vim, sandbox);
    let right = format!("{} | {} | {}{}", session_info, ws_info, tokens, cost_info);

    // 左对齐 + 右对齐
    let left_len = left.chars().count();
    let right_len = right.chars().count();
    let pad = area.width as usize;
    let text = if left_len + right_len >= pad {
        format!("{} {}", left, right)
    } else {
        format!("{}{}{}", left, " ".repeat(pad - left_len - right_len), right)
    };

    let is_sandbox = app.sandbox_mode != crate::cli::sandbox::SandboxMode::Disabled;
    let bg = if app.streaming {
        theme.accent
    } else if app.agent_busy {
        theme.secondary
    } else if is_sandbox {
        Color::Red
    } else {
        theme.bg
    };
    let status = Paragraph::new(Line::from(Span::raw(text)))
        .style(Style::default().bg(bg).fg(Color::White));
    frame.render_widget(status, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;
    use ratatui::layout::Rect;

    fn test_theme() -> Theme {
        Theme {
            bg: Color::Black,
            accent: Color::Magenta,
            primary: Color::Red,
            secondary: Color::Yellow,
            highlight: Color::Cyan,
            code_bg: Color::DarkGray,
            user_msg_bg: Color::Rgb(0x1A, 0x3A, 0x5C),
            assistant_msg_bg: Color::Rgb(0x2D, 0x2D, 0x2D),
        }
    }

    #[test]
    fn test_compute_layout_returns_three_areas() {
        let area = Rect { x: 0, y: 0, width: 100, height: 50 };
        let (left, chat, input, status) = compute_layout(area, false, 0);
        assert!(left.is_none());
        assert_eq!(chat.x, 0);
        assert_eq!(chat.y, 0);
        assert_eq!(chat.width, 100);
        let total_height = chat.height + input.height + status.height;
        assert_eq!(total_height, 50);
        assert_eq!(status.height, 1);
        assert_eq!(input.height, 3);
    }

    #[test]
    fn test_compute_layout_with_sessions_shrinks_chat() {
        let area = Rect { x: 0, y: 0, width: 100, height: 50 };
        let (left, chat, input, status) = compute_layout(area, true, 0);
        let left = left.expect("会话面板应为 Some");
        assert_eq!(left.width, 20); // 20%
        assert_eq!(left.height, 50);
        assert_eq!(chat.x, 20);
        assert_eq!(chat.width, 80);
        assert_eq!(input.height, 3);
        assert_eq!(status.height, 1);
    }

    #[test]
    fn test_compute_layout_all_areas_nonzero() {
        let area = Rect { x: 0, y: 0, width: 80, height: 24 };
        let (_, chat, input, status) = compute_layout(area, false, 0);
        assert!(chat.width > 0);
        assert!(chat.height > 0);
        assert!(input.width > 0);
        assert!(input.height > 0);
        assert!(status.width > 0);
        assert!(status.height > 0);
    }

    #[test]
    fn test_compute_layout_input_grows_with_lines() {
        let area = Rect { x: 0, y: 0, width: 100, height: 50 };
        // 单行输入 → 3 行输入区
        let (_, _, input1, _) = compute_layout(area, false, 0);
        assert_eq!(input1.height, 3);
        // 5 行输入 → 输入区增高
        let (_, _, input5, _) = compute_layout(area, false, 5);
        assert!(input5.height > input1.height);
        // 大量行 → 封顶 8 行
        let (_, _, input_many, _) = compute_layout(area, false, 50);
        assert_eq!(input_many.height, 8);
    }

    #[test]
    fn test_render_status_bar_style() {
        let theme = test_theme();
        assert_eq!(theme.status_style(true, false).bg, Some(Color::Magenta));
        assert_eq!(theme.status_style(false, true).bg, Some(Color::Yellow));
        assert_eq!(theme.status_style(false, false).bg, Some(Color::Cyan));
    }

    // ── P0-1: 光标位置计算（CJK 宽字符按 2 列） ──

    #[test]
    fn test_cursor_position_ascii() {
        assert_eq!(cursor_position("abc", 0), (0, 0));
        assert_eq!(cursor_position("abc", 3), (3, 0));
        assert_eq!(cursor_position("abc", 1), (1, 0));
    }

    #[test]
    fn test_cursor_position_cjk_wide() {
        // "你好" 各占 2 列 → 光标在末尾 = 4 列
        assert_eq!(cursor_position("你好", 6), (4, 0));
        // 混合：a + 你 + b → 列 = 1 + 2 + 1 = 4
        assert_eq!(cursor_position("a你b", 5), (4, 0));
        // 光标在 CJK 字符中间（字节 1）→ 列 0（该字符尚未计入，光标在其之前）
        assert_eq!(cursor_position("你好", 1), (0, 0));
        // 光标在 CJK 字符之后（字节 3）→ 列 2
        assert_eq!(cursor_position("你好", 3), (2, 0));
    }

    #[test]
    fn test_cursor_position_multiline() {
        // "a\nbc"：光标在末尾 → 第 2 行第 2 列
        assert_eq!(cursor_position("a\nbc", 4), (2, 1));
        // 光标在第 1 行末尾（字节 1）→ 第 0 行第 1 列
        assert_eq!(cursor_position("a\nbc", 1), (1, 0));
        // 光标在换行符处（字节 2）→ 第 1 行第 0 列
        assert_eq!(cursor_position("a\nbc", 2), (0, 1));
    }

    #[test]
    fn test_cursor_position_cjk_multiline() {
        // "你\n好" → 光标末尾：第 1 行第 2 列
        assert_eq!(cursor_position("你\n好", 7), (2, 1));
    }

    #[test]
    fn test_is_wide_char_ranges() {
        assert!(is_wide_char('中'));
        assert!(is_wide_char('你'));
        assert!(is_wide_char('好'));
        assert!(is_wide_char('，')); // 全角逗号
        assert!(is_wide_char('あ')); // 平假名
        assert!(!is_wide_char('a'));
        assert!(!is_wide_char('1'));
        assert!(!is_wide_char(' '));
        assert!(!is_wide_char('\n'));
    }
}