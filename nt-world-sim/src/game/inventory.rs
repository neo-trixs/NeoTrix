use super::item::{ItemDef, ItemQuality};

#[derive(Debug, Clone)]
pub struct InventorySlot {
    pub item_id: Option<u32>,
    pub quantity: u32,
    pub quality: ItemQuality,
}

impl InventorySlot {
    pub fn empty() -> Self {
        Self { item_id: None, quantity: 0, quality: ItemQuality::Normal }
    }
    
    pub fn is_empty(&self) -> bool {
        self.item_id.is_none() || self.quantity == 0
    }
    
    pub fn new(item_id: u32, quantity: u32, quality: ItemQuality) -> Self {
        Self { item_id: Some(item_id), quantity, quality }
    }
}

#[derive(Debug, Clone)]
pub struct Inventory {
    pub slots: Vec<InventorySlot>,
    pub hotbar: Vec<InventorySlot>,
    pub selected_hotbar: usize,
    pub gold: i32,
}

impl Inventory {
    pub fn new(size: usize, hotbar_size: usize) -> Self {
        Self {
            slots: vec![InventorySlot::empty(); size],
            hotbar: vec![InventorySlot::empty(); hotbar_size],
            selected_hotbar: 0,
            gold: 500,
        }
    }
    
    pub fn add_item(&mut self, item_id: u32, quantity: u32, quality: ItemQuality) -> u32 {
        let mut remaining = quantity;
        
        // Try to stack in hotbar first
        for slot in &mut self.hotbar {
            if remaining == 0 { break; }
            if slot.item_id == Some(item_id) && slot.quality == quality {
                let can_add = slot.quantity.min(remaining);
                slot.quantity += can_add;
                remaining -= can_add;
            }
        }
        
        // Then inventory
        for slot in &mut self.slots {
            if remaining == 0 { break; }
            if slot.item_id == Some(item_id) && slot.quality == quality {
                let can_add = slot.quantity.min(remaining);
                slot.quantity += can_add;
                remaining -= can_add;
            }
        }
        
        // Fill empty slots
        for slot in &mut self.hotbar {
            if remaining == 0 { break; }
            if slot.is_empty() {
                let to_add = remaining.min(999);
                *slot = InventorySlot::new(item_id, to_add, quality);
                remaining -= to_add;
            }
        }
        for slot in &mut self.slots {
            if remaining == 0 { break; }
            if slot.is_empty() {
                let to_add = remaining.min(999);
                *slot = InventorySlot::new(item_id, to_add, quality);
                remaining -= to_add;
            }
        }
        
        quantity - remaining // return amount actually added
    }
    
    pub fn remove_item(&mut self, item_id: u32, quantity: u32) -> bool {
        let mut to_remove = quantity;
        
        for slot in self.hotbar.iter_mut().chain(self.slots.iter_mut()) {
            if to_remove == 0 { break; }
            if slot.item_id == Some(item_id) {
                let remove = to_remove.min(slot.quantity);
                slot.quantity -= remove;
                to_remove -= remove;
                if slot.quantity == 0 {
                    *slot = InventorySlot::empty();
                }
            }
        }
        
        to_remove == 0
    }
    
    pub fn count_item(&self, item_id: u32) -> u32 {
        self.hotbar.iter().chain(self.slots.iter())
            .filter(|s| s.item_id == Some(item_id))
            .map(|s| s.quantity)
            .sum()
    }
    
    pub fn selected_item(&self) -> Option<&InventorySlot> {
        self.hotbar.get(self.selected_hotbar)
    }
    
    pub fn select_hotbar(&mut self, index: usize) {
        if index < self.hotbar.len() {
            self.selected_hotbar = index;
        }
    }
    
    pub fn has_item(&self, item_id: u32) -> bool {
        self.count_item(item_id) > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_inventory_add_remove() {
        let mut inv = Inventory::new(20, 12);
        inv.add_item(1, 10, ItemQuality::Normal);
        assert_eq!(inv.count_item(1), 10);
        
        inv.remove_item(1, 5);
        assert_eq!(inv.count_item(1), 5);
        
        assert!(inv.remove_item(1, 5));
        assert_eq!(inv.count_item(1), 0);
    }
    
    #[test]
    fn test_hotbar_selection() {
        let mut inv = Inventory::new(20, 12);
        inv.add_item(1, 5, ItemQuality::Normal);
        inv.select_hotbar(0);
        assert!(inv.selected_item().is_some());
    }
}
