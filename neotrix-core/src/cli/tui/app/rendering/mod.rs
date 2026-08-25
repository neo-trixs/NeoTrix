//! Rendering System - 主题、Markdown、语法高亮、Diff

#[allow(unused_imports)]
use ratatui::{
    style::{Style, Color, Modifier},
    text::{Line, Span},
};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use syntect::{highlighting::ThemeSet, parsing::SyntaxSet};

pub mod serde_color;
pub use serde_color::SerdeColor;

/// 主题系统
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub colors: ThemeColors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    // 基础颜色
    #[serde(serialize_with = "serde_color::serialize", deserialize_with = "serde_color::deserialize")]
    pub primary: Color,
    #[serde(serialize_with = "serde_color::serialize", deserialize_with = "serde_color::deserialize")]
    pub secondary: Color,
    #[serde(serialize_with = "serde_color::serialize", deserialize_with = "serde_color::deserialize")]
    pub accent: Color,
    #[serde(serialize_with = "serde_color::serialize", deserialize_with = "serde_color::deserialize")]
    pub highlight: Color,
    #[serde(serialize_with = "serde_color::serialize", deserialize_with = "serde_color::deserialize")]
    pub background: Color,
    #[serde(serialize_with = "serde_color::serialize", deserialize_with = "serde_color::deserialize")]
    pub surface: Color,
    #[serde(serialize_with = "serde_color::serialize", deserialize_with = "serde_color::deserialize")]
    pub border: Color,
    
    // 文本颜色
    #[serde(serialize_with = "serde_color::serialize", deserialize_with = "serde_color::deserialize")]
    pub primary_text: Color,
    #[serde(serialize_with = "serde_color::serialize", deserialize_with = "serde_color::deserialize")]
    pub secondary_text: Color,
    #[serde(serialize_with = "serde_color::serialize", deserialize_with = "serde_color::deserialize")]
    pub muted_text: Color,
    #[serde(serialize_with = "serde_color::serialize", deserialize_with = "serde_color::deserialize")]
    pub accent_text: Color,
    
    // 消息气泡背景
    #[serde(serialize_with = "serde_color::serialize", deserialize_with = "serde_color::deserialize")]
    pub user_msg_bg: Color,
    #[serde(serialize_with = "serde_color::serialize", deserialize_with = "serde_color::deserialize")]
    pub assistant_msg_bg: Color,
    #[serde(serialize_with = "serde_color::serialize", deserialize_with = "serde_color::deserialize")]
    pub system_msg_bg: Color,
    #[serde(serialize_with = "serde_color::serialize", deserialize_with = "serde_color::deserialize")]
    pub error_bg: Color,
    #[serde(serialize_with = "serde_color::serialize", deserialize_with = "serde_color::deserialize")]
    pub code_bg: Color,
    
    // 状态颜色
    #[serde(serialize_with = "serde_color::serialize", deserialize_with = "serde_color::deserialize")]
    pub success: Color,
    #[serde(serialize_with = "serde_color::serialize", deserialize_with = "serde_color::deserialize")]
    pub warning: Color,
    #[serde(serialize_with = "serde_color::serialize", deserialize_with = "serde_color::deserialize")]
    pub error: Color,
    #[serde(serialize_with = "serde_color::serialize", deserialize_with = "serde_color::deserialize")]
    pub info: Color,
    
    // 语法高亮
    #[serde(serialize_with = "serde_color::serialize", deserialize_with = "serde_color::deserialize")]
    pub syntax_keyword: Color,
    #[serde(serialize_with = "serde_color::serialize", deserialize_with = "serde_color::deserialize")]
    pub syntax_string: Color,
    #[serde(serialize_with = "serde_color::serialize", deserialize_with = "serde_color::deserialize")]
    pub syntax_number: Color,
    #[serde(serialize_with = "serde_color::serialize", deserialize_with = "serde_color::deserialize")]
    pub syntax_comment: Color,
    #[serde(serialize_with = "serde_color::serialize", deserialize_with = "serde_color::deserialize")]
    pub syntax_type: Color,
    #[serde(serialize_with = "serde_color::serialize", deserialize_with = "serde_color::deserialize")]
    pub syntax_function: Color,
    #[serde(serialize_with = "serde_color::serialize", deserialize_with = "serde_color::deserialize")]
    pub syntax_variable: Color,
    #[serde(serialize_with = "serde_color::serialize", deserialize_with = "serde_color::deserialize")]
    pub syntax_constant: Color,
    
    // Git 状态
    #[serde(serialize_with = "serde_color::serialize", deserialize_with = "serde_color::deserialize")]
    pub git_added: Color,
    #[serde(serialize_with = "serde_color::serialize", deserialize_with = "serde_color::deserialize")]
    pub git_modified: Color,
    #[serde(serialize_with = "serde_color::serialize", deserialize_with = "serde_color::deserialize")]
    pub git_deleted: Color,
    #[serde(serialize_with = "serde_color::serialize", deserialize_with = "serde_color::deserialize")]
    pub git_untracked: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: "dark".to_string(),
            colors: ThemeColors::dark(),
        }
    }
}

impl ThemeColors {
    pub fn dark() -> Self {
        Self {
            primary: Color::Cyan,
            secondary: Color::Gray,
            accent: Color::Magenta,
            highlight: Color::Yellow,
            background: Color::Rgb(28, 28, 30),
            surface: Color::Rgb(38, 38, 42),
            border: Color::Rgb(58, 58, 60),
            
            primary_text: Color::White,
            secondary_text: Color::Gray,
            muted_text: Color::DarkGray,
            accent_text: Color::Magenta,
            
            user_msg_bg: Color::Rgb(30, 50, 80),
            assistant_msg_bg: Color::Rgb(40, 40, 50),
            system_msg_bg: Color::Rgb(50, 40, 30),
            error_bg: Color::Rgb(80, 30, 30),
            code_bg: Color::Rgb(20, 20, 25),
            
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            info: Color::Blue,
            
            syntax_keyword: Color::Magenta,
            syntax_string: Color::Green,
            syntax_number: Color::Yellow,
            syntax_comment: Color::DarkGray,
            syntax_type: Color::Cyan,
            syntax_function: Color::Blue,
            syntax_variable: Color::White,
            syntax_constant: Color::Yellow,
            
            git_added: Color::Green,
            git_modified: Color::Yellow,
            git_deleted: Color::Red,
            git_untracked: Color::Gray,
        }
    }

    pub fn light() -> Self {
        Self {
            primary: Color::Blue,
            secondary: Color::Gray,
            accent: Color::Magenta,
            highlight: Color::Yellow,
            background: Color::White,
            surface: Color::Rgb(240, 240, 245),
            border: Color::Rgb(200, 200, 205),
            
            primary_text: Color::Black,
            secondary_text: Color::DarkGray,
            muted_text: Color::Gray,
            accent_text: Color::Magenta,
            
            user_msg_bg: Color::Rgb(220, 240, 255),
            assistant_msg_bg: Color::Rgb(245, 245, 250),
            system_msg_bg: Color::Rgb(255, 245, 230),
            error_bg: Color::Rgb(255, 230, 230),
            code_bg: Color::Rgb(245, 245, 250),
            
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            info: Color::Blue,
            
            syntax_keyword: Color::Blue,
            syntax_string: Color::Green,
            syntax_number: Color::Yellow,
            syntax_comment: Color::Gray,
            syntax_type: Color::Cyan,
            syntax_function: Color::Blue,
            syntax_variable: Color::Black,
            syntax_constant: Color::Yellow,
            
            git_added: Color::Green,
            git_modified: Color::Yellow,
            git_deleted: Color::Red,
            git_untracked: Color::Gray,
        }
    }

    pub fn gruvbox() -> Self {
        Self {
            primary: Color::Rgb(211, 134, 155),
            secondary: Color::Rgb(146, 131, 116),
            accent: Color::Rgb(251, 73, 52),
            highlight: Color::Rgb(250, 189, 47),
            background: Color::Rgb(40, 40, 40),
            surface: Color::Rgb(50, 48, 47),
            border: Color::Rgb(80, 73, 69),
            
            primary_text: Color::Rgb(235, 219, 178),
            secondary_text: Color::Rgb(168, 153, 132),
            muted_text: Color::Rgb(124, 111, 94),
            accent_text: Color::Rgb(251, 73, 52),
            
            user_msg_bg: Color::Rgb(60, 56, 54),
            assistant_msg_bg: Color::Rgb(50, 48, 47),
            system_msg_bg: Color::Rgb(60, 50, 45),
            error_bg: Color::Rgb(80, 40, 40),
            code_bg: Color::Rgb(30, 30, 30),
            
            success: Color::Rgb(184, 187, 38),
            warning: Color::Rgb(250, 189, 47),
            error: Color::Rgb(251, 73, 52),
            info: Color::Rgb(131, 165, 152),
            
            syntax_keyword: Color::Rgb(251, 73, 52),
            syntax_string: Color::Rgb(184, 187, 38),
            syntax_number: Color::Rgb(211, 134, 155),
            syntax_comment: Color::Rgb(124, 111, 94),
            syntax_type: Color::Rgb(131, 165, 152),
            syntax_function: Color::Rgb(108, 113, 196),
            syntax_variable: Color::Rgb(235, 219, 178),
            syntax_constant: Color::Rgb(250, 189, 47),
            
            git_added: Color::Rgb(184, 187, 38),
            git_modified: Color::Rgb(250, 189, 47),
            git_deleted: Color::Rgb(251, 73, 52),
            git_untracked: Color::Rgb(146, 131, 116),
        }
    }
}

/// 主题预设
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThemePreset {
    Dark,
    Light,
    Gruvbox,
    Nord,
    Dracula,
    Solarized,
}

impl Default for ThemePreset {
    fn default() -> Self {
        ThemePreset::Dark
    }
}

/// 主题管理器
pub struct ThemeManager {
    pub current: Theme,
    pub presets: HashMap<ThemePreset, Theme>,
}

impl ThemeManager {
    pub fn new() -> Self {
        let mut presets = HashMap::new();
        presets.insert(ThemePreset::Dark, Theme { name: "dark".into(), colors: ThemeColors::dark() });
        presets.insert(ThemePreset::Light, Theme { name: "light".into(), colors: ThemeColors::light() });
        presets.insert(ThemePreset::Gruvbox, Theme { name: "gruvbox".into(), colors: ThemeColors::gruvbox() });
        
        Self {
            current: Theme { name: "dark".into(), colors: ThemeColors::dark() },
            presets,
        }
    }

    pub fn set_preset(&mut self, preset: ThemePreset) {
        if let Some(theme) = self.presets.get(&preset) {
            self.current = theme.clone();
        }
    }

    pub fn set_custom(&mut self, theme: Theme) {
        self.current = theme;
    }

    pub fn current(&self) -> &Theme {
        &self.current
    }

    pub fn colors(&self) -> &ThemeColors {
        &self.current.colors
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Markdown 渲染器
pub mod markdown {
    use super::*;
    use pulldown_cmark::{Parser, Options, Event, Tag, TagEnd, HeadingLevel};
    use ratatui::text::{Line, Span};
    use ratatui::style::{Style, Color};

    /// 渲染 Markdown 为 ratatui Lines
    pub fn render_markdown(text: &str, theme: &ThemeColors) -> Vec<Line<'static>> {
        let parser = Parser::new_ext(text, Options::all());
        let mut lines = Vec::new();
        let mut current_line = Vec::new();
        let mut in_code_block = false;
        let mut code_language = String::new();

        for event in parser {
            match event {
                Event::Start(Tag::Heading(level, _, _)) => {
                    if !current_line.is_empty() {
                        lines.push(Line::from(current_line.clone()));
                        current_line.clear();
                    }
                    let prefix = match level {
                        HeadingLevel::H1 => "█ ",
                        HeadingLevel::H2 => "▓ ",
                        HeadingLevel::H3 => "▒ ",
                        HeadingLevel::H4 => "░ ",
                        HeadingLevel::H5 => "» ",
                        HeadingLevel::H6 => "» ",
                    };
                    current_line.push(Span::styled(
                        format!("{} ", "█".repeat(level as usize)),
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                    ));
                }
                Event::End(TagEnd::Heading(_level)) => {
                    if !current_line.is_empty() {
                        lines.push(Line::from(current_line.clone()));
                        current_line.clear();
                    }
                }
                Event::Start(Tag::CodeBlock(kind)) => {
                    in_code_block = true;
                    code_language = match kind {
                        pulldown_cmark::CodeBlockKind::Fenced(lang) => lang.to_string(),
                        pulldown_cmark::CodeBlockKind::Indented => String::new(),
                    };
                    if !current_line.is_empty() {
                        lines.push(Line::from(current_line.clone()));
                        current_line.clear();
                    }
                    current_line.push(Span::styled(
                        format!("```{}", code_language),
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::ITALIC)
                    ));
                    lines.push(Line::from(current_line.clone()));
                    current_line.clear();
                }
                Event::End(TagEnd::CodeBlock) => {
                    in_code_block = false;
                    code_language.clear();
                    current_line.push(Span::styled(
                        "```",
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::ITALIC)
                    ));
                    lines.push(Line::from(current_line.clone()));
                    current_line.clear();
                }
                Event::Start(Tag::List(_)) => {}
                Event::End(TagEnd::List(_)) => {
                    if !current_line.is_empty() {
                        lines.push(Line::from(current_line.clone()));
                        current_line.clear();
                    }
                }
                Event::Start(Tag::Item) => {
                    current_line.push(Span::styled("• ", Style::default().fg(Color::Yellow)));
                }
                Event::End(TagEnd::Item) => {
                    if !current_line.is_empty() {
                        lines.push(Line::from(current_line.clone()));
                        current_line.clear();
                    }
                }
                Event::Text(text) => {
                    if in_code_block {
                        current_line.push(Span::styled(text.to_string(), 
                            Style::default().fg(Color::Green).bg(Color::Rgb(20, 20, 25))
                        ));
                    } else {
                        current_line.push(Span::styled(text.to_string(), Style::default()));
                    }
                }
                Event::Code(text) => {
                    current_line.push(Span::styled(
                        format!("`{}`", text),
                        Style::default().fg(Color::Green).bg(Color::Rgb(20, 20, 25))
                    ));
                }
                Event::SoftBreak => {
                    if !current_line.is_empty() {
                        lines.push(Line::from(current_line.clone()));
                        current_line.clear();
                    }
                }
                Event::HardBreak => {
                    if !current_line.is_empty() {
                        lines.push(Line::from(current_line.clone()));
                        current_line.clear();
                    }
                }
                Event::Rule => {
                    if !current_line.is_empty() {
                        lines.push(Line::from(current_line.clone()));
                        current_line.clear();
                    }
                    lines.push(Line::from(Span::styled(
                        "─".repeat(40),
                        Style::default().fg(Color::DarkGray)
                    )));
                }
                _ => {}
            }
        }
        
        if !current_line.is_empty() {
            lines.push(Line::from(current_line.clone()));
        }
        
        lines
    }

    /// 增量渲染器 - 用于流式输出
    pub struct StreamingMarkdownRenderer {
        buffer: String,
        lines: Vec<Line<'static>>,
        theme: ThemeColors,
    }

    impl StreamingMarkdownRenderer {
        pub fn new(theme: ThemeColors) -> Self {
            Self {
                buffer: String::new(),
                lines: Vec::new(),
                theme,
            }
        }

        pub fn feed(&mut self, chunk: &str) -> Vec<Line<'static>> {
            self.buffer.push_str(chunk);
            let new_lines = render_markdown(&self.buffer, &self.theme);
            let _new_count = new_lines.len().saturating_sub(self.lines.len());
            let new_lines_vec = new_lines.into_iter().skip(self.lines.len()).collect::<Vec<_>>();
            self.lines = self.render_full();
            new_lines_vec
        }

        fn render_full(&mut self) -> Vec<Line<'static>> {
            self.lines = render_markdown(&self.buffer, &self.theme);
            self.lines.clone()
        }

        pub fn reset(&mut self) {
            self.buffer.clear();
            self.lines.clear();
        }
    }
}

/// 语法高亮
pub mod syntax {
    #[allow(unused_imports)]
    use syntect::{highlighting::ThemeSet, parsing::SyntaxSet, html::highlighted_html_for_string};
    use ratatui::style::{Style, Color, Modifier};
    use ratatui::text::{Line, Span};

    /// 语法高亮器
    pub struct SyntaxHighlighter {
        syntax_set: SyntaxSet,
        theme: syntect::highlighting::Theme,
    }

    impl SyntaxHighlighter {
        pub fn new(theme_name: &str) -> Self {
            let ps = SyntaxSet::load_defaults_newlines();
            let ts = ThemeSet::load_defaults();
            let theme = ts.themes.get(theme_name).unwrap_or_else(|| ts.themes.get("base16-ocean.dark").unwrap()).clone();
            
            Self {
                syntax_set: ps,
                theme,
            }
        }

        pub fn highlight(&self, code: &str, lang: &str) -> Vec<Line<'static>> {
            let syntax = self.syntax_set.find_syntax_by_token(lang)
                .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());
            
            let mut h = syntect::easy::HighlightLines::new(syntax, &self.theme);
            let ranges: Vec<(syntect::highlighting::Style, &str)> = 
                h.highlight_line(code, &self.syntax_set).unwrap();
            
            let mut lines = Vec::new();
            for range in ranges {
                let mut spans = Vec::new();
                for (style, text) in range {
                    let color = syntect_to_color(style.foreground);
                    spans.push(Span::styled(
                        text.to_string(),
                        Style::default().fg(color).add_modifier(if style.font_style.contains(syntect::highlighting::FontStyle::BOLD) { Modifier::BOLD } else { Modifier::empty() })
                    ));
                }
                lines.push(Line::from(spans));
            }
            lines
        }
    }

    fn syntect_to_color(color: syntect::highlighting::Color) -> Color {
        Color::Rgb(color.r, color.g, color.b)
    }
}

/// Diff 视图
pub mod diff {
    use super::*;
    use ratatui::style::{Style, Color};
    use ratatui::text::{Line, Span};

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum DiffLine {
        Added(String),
        Removed(String),
        Context(String),
    }

    /// Simple diff implementation
    pub fn render_diff(old: &str, new: &str, theme: &ThemeColors) -> Vec<Line<'static>> {
        let old_lines: Vec<&str> = old.lines().collect();
        let new_lines: Vec<&str> = new.lines().collect();
        let mut lines = Vec::new();

        let max_len = old_lines.len().max(new_lines.len());
        for i in 0..max_len {
            let old_line = old_lines.get(i);
            let new_line = new_lines.get(i);

            match (old_line, new_line) {
                (Some(old_l), Some(new_l)) if old_l == new_l => {
                    lines.push(Line::from(vec![
                        Span::styled("  ", Style::default()),
                        Span::styled(*old_l, Style::default().fg(Color::Gray)),
                    ]));
                }
                (Some(old_l), None) => {
                    lines.push(Line::from(vec![
                        Span::styled("- ", Style::default().fg(Color::Red)),
                        Span::styled(*old_l, Style::default().fg(Color::Red).bg(Color::Rgb(60, 20, 20))),
                    ]));
                }
                (None, Some(new_l)) => {
                    lines.push(Line::from(vec![
                        Span::styled("+ ", Style::default().fg(Color::Green)),
                        Span::styled(*new_l, Style::default().fg(Color::Green).bg(Color::Rgb(20, 60, 20))),
                    ]));
                }
                (Some(old_l), Some(new_l)) => {
                    lines.push(Line::from(vec![
                        Span::styled("- ", Style::default().fg(Color::Red)),
                        Span::styled(*old_l, Style::default().fg(Color::Red).bg(Color::Rgb(60, 20, 20))),
                    ]));
                    lines.push(Line::from(vec![
                        Span::styled("+ ", Style::default().fg(Color::Green)),
                        Span::styled(*new_l, Style::default().fg(Color::Green).bg(Color::Rgb(20, 60, 20))),
                    ]));
                }
                (None, None) => {}
            }
        }

        lines
    }
}
