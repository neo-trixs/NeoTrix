pub struct Theme {
    pub primary: ratatui::style::Color,
    pub secondary: ratatui::style::Color,
    pub accent: ratatui::style::Color,
    pub bg: ratatui::style::Color,
    pub code_bg: ratatui::style::Color,
    pub highlight: ratatui::style::Color,
    /// 用户消息背景色
    pub user_msg_bg: ratatui::style::Color,
    /// 助手消息背景色
    pub assistant_msg_bg: ratatui::style::Color,
}

impl Theme {
    pub fn status_style(&self, streaming: bool, busy: bool) -> ratatui::style::Style {
        let bg = if streaming { self.accent } else if busy { self.secondary } else { self.highlight };
        ratatui::style::Style::default().bg(bg).fg(ratatui::style::Color::White).add_modifier(ratatui::style::Modifier::BOLD)
    }
}

pub fn theme_list() -> Vec<String> {
    vec!["dark".to_string(), "light".to_string(), "gruvbox".to_string(), "default".to_string()]
}

pub fn theme_by_name(name: &str) -> Theme {
    match name {
        "light" => Theme {
            primary: ratatui::style::Color::Black,
            secondary: ratatui::style::Color::DarkGray,
            accent: ratatui::style::Color::Blue,
            bg: ratatui::style::Color::White,
            code_bg: ratatui::style::Color::Gray,
            highlight: ratatui::style::Color::Cyan,
            user_msg_bg: ratatui::style::Color::Rgb(0xE3, 0xF2, 0xFD), // 浅蓝
            assistant_msg_bg: ratatui::style::Color::Rgb(0xF0, 0xF0, 0xF0), // 浅灰
        },
        "gruvbox" => Theme {
            primary: ratatui::style::Color::Rgb(0xEB, 0xDB, 0xB2),
            secondary: ratatui::style::Color::Rgb(0x92, 0x83, 0x74),
            accent: ratatui::style::Color::Rgb(0xFE, 0x80, 0x19),
            bg: ratatui::style::Color::Rgb(0x28, 0x28, 0x28),
            code_bg: ratatui::style::Color::Rgb(0x3C, 0x38, 0x36),
            highlight: ratatui::style::Color::Rgb(0x8E, 0xC0, 0x7C),
            user_msg_bg: ratatui::style::Color::Rgb(0x3C, 0x38, 0x36),
            assistant_msg_bg: ratatui::style::Color::Rgb(0x50, 0x49, 0x45),
        },
        _ => Theme {
            primary: ratatui::style::Color::White,
            secondary: ratatui::style::Color::Gray,
            accent: ratatui::style::Color::Blue,
            bg: ratatui::style::Color::Black,
            code_bg: ratatui::style::Color::DarkGray,
            highlight: ratatui::style::Color::Cyan,
            user_msg_bg: ratatui::style::Color::Rgb(0x1A, 0x3A, 0x5C), // 深蓝背景
            assistant_msg_bg: ratatui::style::Color::Rgb(0x2D, 0x2D, 0x2D), // 深灰背景
        },
    }
}
