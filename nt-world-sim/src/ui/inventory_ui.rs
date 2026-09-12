use crate::engine::renderer::{Vec2, Rect};
use super::theme::{StardewTheme, UiRenderer, UiDrawCommand};
use crate::game::inventory::Inventory;

pub struct InventoryUI {
    pub visible: bool,
    pub selected_slot: usize,
    pub slot_size: f32,
    pub columns: u32,
    pub position: Vec2,
}

impl InventoryUI {
    pub fn new() -> Self {
        Self {
            visible: false, selected_slot: 0, slot_size: 48.0,
            columns: 12, position: Vec2 { x: 50.0, y: 50.0 },
        }
    }

    pub fn toggle(&mut self) { self.visible = !self.visible; }

    pub fn slot_rect(&self, index: u32) -> Rect {
        let col = index % self.columns;
        let row = index / self.columns;
        Rect {
            x: self.position.x + col as f32 * (self.slot_size + 4.0),
            y: self.position.y + row as f32 * (self.slot_size + 4.0),
            width: self.slot_size,
            height: self.slot_size,
        }
    }

    pub fn render(&self, inventory: &Inventory) -> Vec<UiDrawCommand> {
        let mut cmds = Vec::new();
        if !self.visible { return cmds; }

        let theme = StardewTheme::default();
        let total_rows = ((inventory.slots.len() as u32 + self.columns - 1) / self.columns) as f32;

        // Background panel
        cmds.extend(UiRenderer::draw_panel(
            &theme,
            self.position.x - 8.0,
            self.position.y - 8.0,
            self.columns as f32 * (self.slot_size + 4.0) + 16.0,
            total_rows * (self.slot_size + 4.0) + 16.0,
        ));

        // Inventory slots
        for (i, slot) in inventory.slots.iter().enumerate() {
            let rect = self.slot_rect(i as u32);
            let selected = i == self.selected_slot;
            cmds.extend(UiRenderer::draw_slot(&theme, rect.x, rect.y, selected, false));

            if let Some(item_id) = slot.item_id {
                cmds.push(UiDrawCommand::Text {
                    text: format!("ID:{}", item_id),
                    x: rect.x + 4.0, y: rect.y + 4.0, size: 9.0, color: theme.white_text,
                });
                cmds.push(UiDrawCommand::Text {
                    text: format!("x{}", slot.quantity),
                    x: rect.x + 4.0, y: rect.y + 16.0, size: 9.0, color: theme.gold_text,
                });
            }
        }

        cmds
    }
}

impl Default for InventoryUI {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::item::ItemQuality;

    #[test]
    fn test_slot_rect() {
        let ui = InventoryUI::new();
        let r0 = ui.slot_rect(0);
        assert_eq!(r0.x, 50.0);
        let r1 = ui.slot_rect(1);
        assert_eq!(r1.x, 50.0 + 52.0);
        let r12 = ui.slot_rect(12);
        assert_eq!(r12.y, 50.0 + 52.0);
    }

    #[test]
    fn test_render_empty_when_hidden() {
        let ui = InventoryUI::new();
        let inv = Inventory::new(24, 12);
        assert!(ui.render(&inv).is_empty());
    }

    #[test]
    fn test_render_visible() {
        let mut ui = InventoryUI::new();
        ui.visible = true;
        let inv = Inventory::new(24, 12);
        let cmds = ui.render(&inv);
        assert!(!cmds.is_empty());
    }

    #[test]
    fn test_render_with_items() {
        let mut ui = InventoryUI::new();
        ui.visible = true;
        let mut inv = Inventory::new(24, 12);
        inv.add_item(42, 5, ItemQuality::Normal);
        let cmds = ui.render(&inv);
        // Panel(3) + slot(2) + 2 text labels = 7 minimum for 1 item
        assert!(cmds.len() >= 7);
    }
}
