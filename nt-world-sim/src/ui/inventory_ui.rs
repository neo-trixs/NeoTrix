use crate::engine::renderer::{Color, Vec2, Rect};
use super::widget::UiStyle;
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
    
    pub fn render(&self, inventory: &Inventory) -> Vec<(Rect, Color, Option<String>)> {
        let mut elements = Vec::new();
        if !self.visible { return elements; }
        
        let total_rows = ((inventory.slots.len() as u32 + self.columns - 1) / self.columns) as f32;
        elements.push((
            Rect {
                x: self.position.x - 8.0, y: self.position.y - 8.0,
                width: self.columns as f32 * (self.slot_size + 4.0) + 16.0,
                height: total_rows * (self.slot_size + 4.0) + 16.0,
            },
            UiStyle::stardew_wood().background,
            None,
        ));
        
        for (i, slot) in inventory.slots.iter().enumerate() {
            let rect = self.slot_rect(i as u32);
            let bg = if i == self.selected_slot {
                Color { r: 0.8, g: 0.7, b: 0.3, a: 0.8 }
            } else {
                Color { r: 0.2, g: 0.15, b: 0.1, a: 0.7 }
            };
            let label = if let Some(item_id) = slot.item_id {
                Some(format!("ID:{}\nx{}", item_id, slot.quantity))
            } else {
                None
            };
            elements.push((rect, bg, label));
        }
        
        elements
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
        let elements = ui.render(&inv);
        assert!(!elements.is_empty());
    }
}
