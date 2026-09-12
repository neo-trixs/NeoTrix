use super::theme::{StardewTheme, UiRenderer, UiDrawCommand};
use super::hud::HUD;
use super::inventory_ui::InventoryUI;
use super::dialogue::DialogueBox;
use crate::game::inventory::Inventory;
use crate::game::time::GameTime;
use crate::game::weather::Weather;
use crate::engine::renderer::Color;

pub struct GameUI {
    pub theme: StardewTheme,
    pub hud: HUD,
    pub inventory: InventoryUI,
    pub dialogue: DialogueBox,
    pub crafting_visible: bool,
    pub menu_visible: bool,
    pub notifications: Vec<Notification>,
    pub tooltip: Option<Tooltip>,
}

pub struct Notification {
    pub text: String,
    pub timer: f32,
    pub color: Color,
}

pub struct Tooltip {
    pub text: String,
    pub x: f32,
    pub y: f32,
}

impl GameUI {
    pub fn new() -> Self {
        Self {
            theme: StardewTheme::default(),
            hud: HUD::new(),
            inventory: InventoryUI::new(),
            dialogue: DialogueBox::new(),
            crafting_visible: false,
            menu_visible: false,
            notifications: Vec::new(),
            tooltip: None,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.notifications.retain_mut(|n| {
            n.timer -= dt;
            n.timer > 0.0
        });
    }

    pub fn add_notification(&mut self, text: &str) {
        self.notifications.push(Notification {
            text: text.to_string(),
            timer: 3.0,
            color: self.theme.gold_text,
        });
    }

    pub fn show_tooltip(&mut self, text: &str, x: f32, y: f32) {
        self.tooltip = Some(Tooltip { text: text.to_string(), x, y });
    }

    pub fn hide_tooltip(&mut self) {
        self.tooltip = None;
    }

    pub fn render(
        &self,
        inventory: &Inventory,
        time: &GameTime,
        weather: &Weather,
        screen_w: f32,
        screen_h: f32,
    ) -> Vec<UiDrawCommand> {
        let mut cmds = Vec::new();

        // HUD (always visible in Playing state)
        cmds.extend(self.hud.render(time, weather));

        // Toolbar (bottom center)
        cmds.extend(self.render_toolbar());

        // Inventory (if open)
        if self.inventory.visible {
            cmds.extend(self.inventory.render(inventory));
        }

        // Crafting menu (if open)
        if self.crafting_visible {
            cmds.extend(self.render_crafting_menu());
        }

        // Dialogue box
        if let Some(d) = self.dialogue.render() {
            cmds.extend(self.render_dialogue_box(d.0, &d.1, &d.2, &d.3));
        }

        // Notifications
        cmds.extend(self.render_notifications(screen_w));

        // Tooltip
        if let Some(ref tip) = self.tooltip {
            cmds.extend(self.render_tooltip(tip));
        }

        // Menu (if open)
        if self.menu_visible {
            cmds.extend(self.render_menu(screen_w, screen_h));
        }

        cmds
    }

    fn render_toolbar(&self) -> Vec<UiDrawCommand> {
        let mut cmds = Vec::new();
        let slot_count = 12;
        let total_w = slot_count as f32 * (self.theme.slot_size + self.theme.slot_gap) - self.theme.slot_gap;
        let start_x = (800.0 - total_w) / 2.0;
        let y = 560.0;

        // Toolbar background
        cmds.extend(UiRenderer::draw_panel(&self.theme, start_x - 8.0, y - 8.0, total_w + 16.0, self.theme.slot_size + 16.0));

        for i in 0..slot_count {
            let x = start_x + i as f32 * (self.theme.slot_size + self.theme.slot_gap);
            let selected = i == 0; // TODO: get from game state
            cmds.extend(UiRenderer::draw_slot(&self.theme, x, y, selected, false));
        }

        cmds
    }

    fn render_crafting_menu(&self) -> Vec<UiDrawCommand> {
        let mut cmds = Vec::new();
        cmds.extend(UiRenderer::draw_panel(&self.theme, 200.0, 100.0, 400.0, 350.0));
        cmds.extend(UiRenderer::draw_text(&self.theme, "Crafting", 350.0, 110.0, 18.0));
        cmds
    }

    fn render_dialogue_box(
        &self,
        rect: crate::engine::renderer::Rect,
        speaker: &str,
        text: &str,
        responses: &[(String, usize)],
    ) -> Vec<UiDrawCommand> {
        let mut cmds = Vec::new();

        // Dialogue panel
        cmds.extend(UiRenderer::draw_panel(&self.theme, rect.x, rect.y, rect.width, rect.height));

        // Speaker name
        cmds.extend(UiRenderer::draw_text(&self.theme, speaker, rect.x + 16.0, rect.y + 8.0, 14.0));

        // Dialogue text
        cmds.extend(UiRenderer::draw_text(&self.theme, text, rect.x + 16.0, rect.y + 30.0, 12.0));

        // Response buttons
        for (i, (response_text, _)) in responses.iter().enumerate() {
            let btn_y = rect.y + rect.height - 40.0 - i as f32 * 28.0;
            cmds.push(UiDrawCommand::Rect {
                x: rect.x + 16.0, y: btn_y,
                width: rect.width - 32.0, height: 24.0,
                color: self.theme.slot_bg,
            });
            cmds.extend(UiRenderer::draw_text(&self.theme, response_text, rect.x + 24.0, btn_y + 4.0, 11.0));
        }

        cmds
    }

    fn render_notifications(&self, screen_w: f32) -> Vec<UiDrawCommand> {
        let mut cmds = Vec::new();
        for (i, notif) in self.notifications.iter().enumerate() {
            let y = 100.0 + i as f32 * 28.0;
            let alpha = (notif.timer / 3.0).min(1.0);
            cmds.push(UiDrawCommand::Text {
                text: notif.text.clone(),
                x: screen_w / 2.0 - 100.0,
                y,
                size: 12.0,
                color: Color { r: notif.color.r, g: notif.color.g, b: notif.color.b, a: alpha },
            });
        }
        cmds
    }

    fn render_tooltip(&self, tip: &Tooltip) -> Vec<UiDrawCommand> {
        let mut cmds = Vec::new();
        cmds.extend(UiRenderer::draw_panel(&self.theme, tip.x + 10.0, tip.y + 10.0, 150.0, 30.0));
        cmds.extend(UiRenderer::draw_text(&self.theme, &tip.text, tip.x + 18.0, tip.y + 18.0, 10.0));
        cmds
    }

    fn render_menu(&self, screen_w: f32, screen_h: f32) -> Vec<UiDrawCommand> {
        let mut cmds = Vec::new();
        let w = 300.0;
        let h = 250.0;
        let x = (screen_w - w) / 2.0;
        let y = (screen_h - h) / 2.0;

        cmds.extend(UiRenderer::draw_panel(&self.theme, x, y, w, h));
        cmds.extend(UiRenderer::draw_text(&self.theme, "Menu", x + w / 2.0 - 20.0, y + 10.0, 16.0));

        let items = ["Resume", "Save", "Load", "Settings", "Quit"];
        for (i, item) in items.iter().enumerate() {
            let btn_y = y + 40.0 + i as f32 * 36.0;
            cmds.push(UiDrawCommand::Rect {
                x: x + 20.0, y: btn_y,
                width: w - 40.0, height: 30.0,
                color: self.theme.slot_bg,
            });
            cmds.extend(UiRenderer::draw_text(&self.theme, item, x + 30.0, btn_y + 6.0, 13.0));
        }

        cmds
    }
}

impl Default for GameUI {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::inventory::Inventory;
    use crate::game::time::GameTime;
    use crate::game::weather::Weather;

    #[test]
    fn test_game_ui_new() {
        let ui = GameUI::new();
        assert!(!ui.crafting_visible);
        assert!(!ui.menu_visible);
        assert!(ui.notifications.is_empty());
        assert!(ui.tooltip.is_none());
    }

    #[test]
    fn test_add_notification() {
        let mut ui = GameUI::new();
        ui.add_notification("Hello!");
        assert_eq!(ui.notifications.len(), 1);
        assert_eq!(ui.notifications[0].text, "Hello!");
        assert_eq!(ui.notifications[0].timer, 3.0);
    }

    #[test]
    fn test_update_removes_expired_notifications() {
        let mut ui = GameUI::new();
        ui.add_notification("Test");
        ui.update(3.5);
        assert!(ui.notifications.is_empty());
    }

    #[test]
    fn test_show_hide_tooltip() {
        let mut ui = GameUI::new();
        ui.show_tooltip("Item info", 100.0, 200.0);
        assert!(ui.tooltip.is_some());
        assert_eq!(ui.tooltip.as_ref().unwrap().text, "Item info");
        ui.hide_tooltip();
        assert!(ui.tooltip.is_none());
    }

    #[test]
    fn test_render_returns_commands() {
        let ui = GameUI::new();
        let inv = Inventory::new(24, 12);
        let time = GameTime::new();
        let weather = Weather::new();
        let cmds = ui.render(&inv, &time, &weather, 800.0, 600.0);
        assert!(!cmds.is_empty());
    }

    #[test]
    fn test_render_with_notifications() {
        let mut ui = GameUI::new();
        ui.add_notification("Test");
        let inv = Inventory::new(24, 12);
        let time = GameTime::new();
        let weather = Weather::new();
        let cmds = ui.render(&inv, &time, &weather, 800.0, 600.0);
        // Should include HUD + toolbar + notification
        assert!(cmds.len() > 3);
    }

    #[test]
    fn test_render_with_menu() {
        let mut ui = GameUI::new();
        ui.menu_visible = true;
        let inv = Inventory::new(24, 12);
        let time = GameTime::new();
        let weather = Weather::new();
        let cmds = ui.render(&inv, &time, &weather, 800.0, 600.0);
        // Menu adds panel + text + 5 button rects + 5 button texts
        assert!(cmds.len() > 10);
    }
}
