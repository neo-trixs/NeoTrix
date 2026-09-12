use crate::engine::renderer::Color;

pub struct StardewTheme {
    // Colors
    pub wood_dark: Color,      // #3d2510
    pub wood_medium: Color,    // #5d3a1a
    pub wood_light: Color,     // #8b6914
    pub wood_highlight: Color, // #a08030
    pub gold_text: Color,      // #f4d03f
    pub white_text: Color,     // #ffffff
    pub shadow_text: Color,    // #2a1a0a
    pub panel_bg: Color,       // rgba(30, 20, 10, 0.9)
    pub slot_bg: Color,        // rgba(40, 30, 15, 0.8)
    pub slot_hover: Color,     // rgba(80, 60, 30, 0.9)
    pub slot_selected: Color,  // rgba(120, 90, 40, 0.9)
    pub health_red: Color,     // #e74c3c
    pub energy_green: Color,   // #2ecc71
    pub xp_blue: Color,        // #3498db

    // Spacing
    pub border_width: f32,
    pub padding: f32,
    pub slot_size: f32,
    pub slot_gap: f32,
    pub corner_radius: f32,
}

impl StardewTheme {
    pub fn default() -> Self {
        Self {
            wood_dark: Color { r: 0.24, g: 0.15, b: 0.06, a: 1.0 },
            wood_medium: Color { r: 0.36, g: 0.23, b: 0.10, a: 1.0 },
            wood_light: Color { r: 0.55, g: 0.41, b: 0.08, a: 1.0 },
            wood_highlight: Color { r: 0.63, g: 0.50, b: 0.19, a: 1.0 },
            gold_text: Color { r: 0.96, g: 0.82, b: 0.25, a: 1.0 },
            white_text: Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
            shadow_text: Color { r: 0.16, g: 0.10, b: 0.04, a: 1.0 },
            panel_bg: Color { r: 0.12, g: 0.08, b: 0.04, a: 0.95 },
            slot_bg: Color { r: 0.16, g: 0.12, b: 0.06, a: 0.85 },
            slot_hover: Color { r: 0.31, g: 0.24, b: 0.12, a: 0.9 },
            slot_selected: Color { r: 0.47, g: 0.35, b: 0.16, a: 0.95 },
            health_red: Color { r: 0.91, g: 0.30, b: 0.24, a: 1.0 },
            energy_green: Color { r: 0.18, g: 0.80, b: 0.44, a: 1.0 },
            xp_blue: Color { r: 0.20, g: 0.60, b: 0.86, a: 1.0 },
            border_width: 4.0,
            padding: 12.0,
            slot_size: 48.0,
            slot_gap: 4.0,
            corner_radius: 6.0,
        }
    }

    pub fn clarity_spring() -> Self {
        let mut theme = Self::default();
        theme.panel_bg = Color { r: 0.10, g: 0.15, b: 0.10, a: 0.95 };
        theme.slot_bg = Color { r: 0.12, g: 0.18, b: 0.10, a: 0.85 };
        theme.wood_light = Color { r: 0.45, g: 0.55, b: 0.25, a: 1.0 };
        theme.wood_highlight = Color { r: 0.55, g: 0.65, b: 0.30, a: 1.0 };
        theme.gold_text = Color { r: 0.85, g: 0.92, b: 0.40, a: 1.0 };
        theme
    }

    pub fn flow_summer() -> Self {
        let mut theme = Self::default();
        theme.panel_bg = Color { r: 0.15, g: 0.10, b: 0.05, a: 0.95 };
        theme.slot_bg = Color { r: 0.20, g: 0.12, b: 0.06, a: 0.85 };
        theme.wood_light = Color { r: 0.65, g: 0.45, b: 0.15, a: 1.0 };
        theme.wood_highlight = Color { r: 0.75, g: 0.55, b: 0.20, a: 1.0 };
        theme.gold_text = Color { r: 1.0, g: 0.85, b: 0.30, a: 1.0 };
        theme
    }

    pub fn reflection_fall() -> Self {
        let mut theme = Self::default();
        theme.panel_bg = Color { r: 0.15, g: 0.08, b: 0.03, a: 0.95 };
        theme.slot_bg = Color { r: 0.20, g: 0.10, b: 0.04, a: 0.85 };
        theme.wood_light = Color { r: 0.70, g: 0.40, b: 0.10, a: 1.0 };
        theme.wood_highlight = Color { r: 0.80, g: 0.50, b: 0.15, a: 1.0 };
        theme.gold_text = Color { r: 0.95, g: 0.75, b: 0.25, a: 1.0 };
        theme
    }

    pub fn stillness_winter() -> Self {
        let mut theme = Self::default();
        theme.panel_bg = Color { r: 0.05, g: 0.08, b: 0.15, a: 0.95 };
        theme.slot_bg = Color { r: 0.08, g: 0.10, b: 0.18, a: 0.85 };
        theme.wood_light = Color { r: 0.30, g: 0.40, b: 0.60, a: 1.0 };
        theme.wood_highlight = Color { r: 0.40, g: 0.50, b: 0.70, a: 1.0 };
        theme.gold_text = Color { r: 0.75, g: 0.85, b: 1.0, a: 1.0 };
        theme
    }
}

pub struct UiRenderer;

impl UiRenderer {
    pub fn draw_panel(theme: &StardewTheme, x: f32, y: f32, w: f32, h: f32) -> Vec<UiDrawCommand> {
        let mut cmds = Vec::new();

        // Outer border (dark wood)
        cmds.push(UiDrawCommand::Rect {
            x: x - theme.border_width,
            y: y - theme.border_width,
            width: w + theme.border_width * 2.0,
            height: h + theme.border_width * 2.0,
            color: theme.wood_dark,
        });

        // Main panel (medium wood)
        cmds.push(UiDrawCommand::Rect { x, y, width: w, height: h, color: theme.panel_bg });

        // Inner border (light wood highlight)
        cmds.push(UiDrawCommand::Rect {
            x: x + 2.0, y: y + 2.0,
            width: w - 4.0, height: h - 4.0,
            color: theme.wood_medium,
        });

        cmds
    }

    pub fn draw_text(theme: &StardewTheme, text: &str, x: f32, y: f32, size: f32) -> Vec<UiDrawCommand> {
        vec![
            // Shadow
            UiDrawCommand::Text { text: text.to_string(), x: x + 1.0, y: y + 1.0, size, color: theme.shadow_text },
            // Main text
            UiDrawCommand::Text { text: text.to_string(), x, y, size, color: theme.gold_text },
        ]
    }

    pub fn draw_slot(theme: &StardewTheme, x: f32, y: f32, selected: bool, hovered: bool) -> Vec<UiDrawCommand> {
        let bg = if selected { theme.slot_selected }
            else if hovered { theme.slot_hover }
            else { theme.slot_bg };

        vec![
            UiDrawCommand::Rect { x, y, width: theme.slot_size, height: theme.slot_size, color: bg },
            UiDrawCommand::RectBorder { x, y, width: theme.slot_size, height: theme.slot_size, color: theme.wood_light, thickness: 2.0 },
        ]
    }

    pub fn draw_bar(theme: &StardewTheme, x: f32, y: f32, w: f32, h: f32, pct: f32, color: Color) -> Vec<UiDrawCommand> {
        vec![
            UiDrawCommand::Rect { x, y, width: w, height: h, color: theme.slot_bg },
            UiDrawCommand::Rect { x, y, width: w * pct, height: h, color },
            UiDrawCommand::RectBorder { x, y, width: w, height: h, color: theme.wood_light, thickness: 1.0 },
        ]
    }
}

#[derive(Debug, Clone)]
pub enum UiDrawCommand {
    Rect { x: f32, y: f32, width: f32, height: f32, color: Color },
    RectBorder { x: f32, y: f32, width: f32, height: f32, color: Color, thickness: f32 },
    Text { text: String, x: f32, y: f32, size: f32, color: Color },
    Sprite { texture: String, x: f32, y: f32, width: f32, height: f32 },
    Circle { x: f32, y: f32, radius: f32, color: Color },
    Line { x1: f32, y1: f32, x2: f32, y2: f32, color: Color, thickness: f32 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stardew_theme_defaults() {
        let t = StardewTheme::default();
        assert_eq!(t.border_width, 4.0);
        assert_eq!(t.padding, 12.0);
        assert_eq!(t.slot_size, 48.0);
        assert_eq!(t.slot_gap, 4.0);
        assert_eq!(t.corner_radius, 6.0);
    }

    #[test]
    fn test_stardew_theme_colors() {
        let t = StardewTheme::default();
        assert!((t.wood_dark.r - 0.24).abs() < 0.01);
        assert!((t.gold_text.r - 0.96).abs() < 0.01);
        assert!((t.health_red.r - 0.91).abs() < 0.01);
        assert!((t.energy_green.g - 0.80).abs() < 0.01);
    }

    #[test]
    fn test_draw_panel_produces_three_commands() {
        let t = StardewTheme::default();
        let cmds = UiRenderer::draw_panel(&t, 10.0, 10.0, 200.0, 100.0);
        assert_eq!(cmds.len(), 3);
    }

    #[test]
    fn test_draw_text_produces_shadow_and_main() {
        let t = StardewTheme::default();
        let cmds = UiRenderer::draw_text(&t, "Hello", 0.0, 0.0, 14.0);
        assert_eq!(cmds.len(), 2);
    }

    #[test]
    fn test_draw_slot_default_bg() {
        let t = StardewTheme::default();
        let cmds = UiRenderer::draw_slot(&t, 0.0, 0.0, false, false);
        assert_eq!(cmds.len(), 2);
    }

    #[test]
    fn test_draw_slot_selected_bg() {
        let t = StardewTheme::default();
        let cmds = UiRenderer::draw_slot(&t, 0.0, 0.0, true, false);
        if let UiDrawCommand::Rect { color, .. } = &cmds[0] {
            assert_eq!(color.r, t.slot_selected.r);
        } else {
            panic!("Expected Rect");
        }
    }

    #[test]
    fn test_draw_bar_three_commands() {
        let t = StardewTheme::default();
        let cmds = UiRenderer::draw_bar(&t, 0.0, 0.0, 100.0, 10.0, 0.75, t.energy_green);
        assert_eq!(cmds.len(), 3);
    }
}
