use crate::engine::renderer::Rect;

pub struct InventoryUi {
    pub open: bool,
    pub selected_slot: usize,
    pub rect: Rect,
}

impl InventoryUi {
    pub fn new() -> Self {
        Self {
            open: false,
            selected_slot: 0,
            rect: Rect::new(100.0, 50.0, 600.0, 400.0),
        }
    }

    pub fn toggle(&mut self) {
        self.open = !self.open;
    }

    pub fn select_slot(&mut self, index: usize) {
        self.selected_slot = index;
    }

    pub fn slot_rect(&self, index: usize) -> Rect {
        let col = (index % 12) as f32;
        let row = (index / 12) as f32;
        Rect::new(
            self.rect.x + 20.0 + col * 48.0,
            self.rect.y + 60.0 + row * 48.0,
            44.0, 44.0,
        )
    }
}

impl Default for InventoryUi {
    fn default() -> Self { Self::new() }
}
