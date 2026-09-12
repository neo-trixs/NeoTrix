use super::item::ItemQuality;
use crate::core::Resource;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolType {
    Hoe,
    WateringCan,
    Pickaxe,
    Axe,
    Scythe,
    FishingRod,
}

#[derive(Debug, Clone)]
pub struct Tool {
    pub tool_type: ToolType,
    pub level: u8,
    pub energy_cost: u32,
}

impl Tool {
    pub fn new(tool_type: ToolType) -> Self {
        let energy_cost = match tool_type {
            ToolType::Hoe | ToolType::WateringCan => 5,
            ToolType::Pickaxe | ToolType::Axe => 5,
            ToolType::Scythe => 3,
            ToolType::FishingRod => 0,
        };
        Self { tool_type, level: 0, energy_cost }
    }

    pub fn upgrade_cost(&self) -> u32 {
        (self.level as u32 + 1) * 5000
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemCategory {
    Seed,
    Crop,
    Tool,
    Material,
    Food,
    Forage,
    Mineral,
    Fish,
    Crafted,
}

#[derive(Debug, Clone)]
pub struct Item {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub stack: u32,
    pub max_stack: u32,
    pub base_value: u32,
    pub category: ItemCategory,
    pub quality: ItemQuality,
}

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

    pub fn can_add(&self, item_id: u32, max_stack: u32) -> bool {
        match self.item_id {
            None => true,
            Some(id) => id == item_id && self.quantity < max_stack,
        }
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

impl Default for Inventory {
    fn default() -> Self { Self::new(20, 12) }
}

impl Resource for Inventory {}

pub struct ItemDatabase {
    pub items: HashMap<u32, Item>,
}

impl ItemDatabase {
    pub fn new() -> Self {
        Self { items: HashMap::new() }
    }

    pub fn register(&mut self, item: Item) {
        self.items.insert(item.id, item);
    }

    pub fn get(&self, id: u32) -> Option<&Item> {
        self.items.get(&id)
    }

    pub fn get_mut(&mut self, id: u32) -> Option<&mut Item> {
        self.items.get_mut(&id)
    }

    pub fn item_count(&self) -> usize {
        self.items.len()
    }
}

impl Default for ItemDatabase {
    fn default() -> Self { Self::new() }
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
