pub struct Theme {
    pub primary: ratatui::style::Color,
    pub secondary: ratatui::style::Color,
    pub accent: ratatui::style::Color,
    pub bg: ratatui::style::Color,
    pub code_bg: ratatui::style::Color,
    pub highlight: ratatui::style::Color,
}

impl Theme {
    pub fn status_style(&self, streaming: bool, busy: bool) -> ratatui::style::Style {
        let bg = if streaming { self.accent } else if busy { self.secondary } else { self.highlight };
        ratatui::style::Style::default().bg(bg).fg(ratatui::style::Color::White).add_modifier(ratatui::style::Modifier::BOLD)
    }
}

pub fn theme_list() -> Vec<String> {
    vec!["default".to_string()]
}

pub fn theme_by_name(_name: &str) -> Theme {
    Theme {
        primary: ratatui::style::Color::White,
        secondary: ratatui::style::Color::Gray,
        accent: ratatui::style::Color::Blue,
        bg: ratatui::style::Color::Black,
        code_bg: ratatui::style::Color::DarkGray,
        highlight: ratatui::style::Color::Cyan,
    }
}
