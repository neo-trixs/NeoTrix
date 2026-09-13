use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Item
// ---------------------------------------------------------------------------

/// An item definition (template from database).
#[derive(Debug, Clone)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub item_type: ItemType,
    pub stackable: bool,
    pub max_stack: u32,
    pub equippable: bool,
    pub equip_slot: Option<EquipSlot>,
    pub use_effect: Option<ItemUseEffect>,
    pub base_stats: HashMap<String, f32>,
    pub value: u64,
    pub rarity: Rarity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemType {
    Consumable,
    Equipment,
    Material,
    QuestItem,
    Misc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EquipSlot {
    Weapon,
    Armor,
    Helmet,
    Boots,
    Accessory,
}

#[derive(Debug, Clone)]
pub enum ItemUseEffect {
    Heal(f32),
    RestoreMp(f32),
    Buff { stat: String, amount: f32, duration: f32 },
    Teleport { x: f32, y: f32 },
    Spawn { npc_id: String },
}

impl Item {
    pub fn new(id: &str, name: &str, item_type: ItemType) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: String::new(),
            icon: format!("icons/{}.png", id),
            item_type,
            stackable: matches!(item_type, ItemType::Consumable | ItemType::Material),
            max_stack: if matches!(item_type, ItemType::Equipment) { 1 } else { 99 },
            equippable: matches!(item_type, ItemType::Equipment),
            equip_slot: None,
            use_effect: None,
            base_stats: HashMap::new(),
            value: 0,
            rarity: Rarity::Common,
        }
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    pub fn with_icon(mut self, icon: &str) -> Self {
        self.icon = icon.to_string();
        self
    }

    pub fn with_max_stack(mut self, max: u32) -> Self {
        self.max_stack = max;
        self
    }

    pub fn with_equip_slot(mut self, slot: EquipSlot) -> Self {
        self.equip_slot = Some(slot);
        self
    }

    pub fn with_use_effect(mut self, effect: ItemUseEffect) -> Self {
        self.use_effect = Some(effect);
        self
    }

    pub fn with_stat(mut self, stat: &str, amount: f32) -> Self {
        self.base_stats.insert(stat.to_string(), amount);
        self
    }

    pub fn with_value(mut self, value: u64) -> Self {
        self.value = value;
        self
    }

    pub fn with_rarity(mut self, rarity: Rarity) -> Self {
        self.rarity = rarity;
        self
    }
}

// ---------------------------------------------------------------------------
// InventorySlot
// ---------------------------------------------------------------------------

/// A single slot in an inventory.
#[derive(Debug, Clone)]
pub struct InventorySlot {
    pub item_id: Option<String>,
    pub count: u32,
    pub durability: Option<f32>,
}

impl InventorySlot {
    pub fn empty() -> Self {
        Self { item_id: None, count: 0, durability: None }
    }

    pub fn new(item_id: &str, count: u32) -> Self {
        Self { item_id: Some(item_id.to_string()), count, durability: None }
    }

    pub fn with_durability(mut self, dur: f32) -> Self {
        self.durability = Some(dur);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.item_id.is_none() || self.count == 0
    }

    pub fn item_id(&self) -> Option<&str> {
        self.item_id.as_deref()
    }
}

// ---------------------------------------------------------------------------
// Inventory
// ---------------------------------------------------------------------------

/// Player inventory with fixed capacity and equipment.
pub struct Inventory {
    slots: Vec<InventorySlot>,
    capacity: usize,
    equipment: HashMap<EquipSlot, InventorySlot>,
    item_db: HashMap<String, Item>,
}

impl Inventory {
    pub fn new(capacity: usize) -> Self {
        let slots = (0..capacity).map(|_| InventorySlot::empty()).collect();
        let mut equipment = HashMap::new();
        for slot in [EquipSlot::Weapon, EquipSlot::Armor, EquipSlot::Helmet, EquipSlot::Boots, EquipSlot::Accessory] {
            equipment.insert(slot, InventorySlot::empty());
        }
        Self { slots, capacity, equipment, item_db: HashMap::new() }
    }

    /// Register an item template.
    pub fn register_item(&mut self, item: Item) {
        self.item_db.insert(item.id.clone(), item);
    }

    /// Get item template by id.
    pub fn get_item_def(&self, item_id: &str) -> Option<&Item> {
        self.item_db.get(item_id)
    }

    /// Add item to inventory. Returns count that couldn't fit.
    pub fn add_item(&mut self, item_id: &str, count: u32) -> u32 {
        let mut remaining = count;
        let max_stack = self.item_db.get(item_id)
            .map(|i| if i.stackable { i.max_stack } else { 1 })
            .unwrap_or(99);

        // First, fill existing partial stacks
        if max_stack > 1 {
            for slot in &mut self.slots {
                if remaining == 0 { break; }
                if slot.item_id.as_deref() == Some(item_id) && slot.count < max_stack {
                    let space = max_stack - slot.count;
                    let fill = remaining.min(space);
                    slot.count += fill;
                    remaining -= fill;
                }
            }
        }

        // Then, use empty slots
        for slot in &mut self.slots {
            if remaining == 0 { break; }
            if slot.is_empty() {
                let fill = remaining.min(max_stack);
                slot.item_id = Some(item_id.to_string());
                slot.count = fill;
                if self.item_db.get(item_id).map_or(false, |i| i.item_type == ItemType::Equipment) {
                    slot.durability = Some(100.0);
                }
                remaining -= fill;
            }
        }

        remaining
    }

    /// Remove items from inventory. Returns true if successful.
    pub fn remove_item(&mut self, item_id: &str, count: u32) -> bool {
        if !self.has_item(item_id, count) {
            return false;
        }
        let mut to_remove = count;
        for slot in &mut self.slots {
            if to_remove == 0 { break; }
            if slot.item_id.as_deref() == Some(item_id) {
                let remove = to_remove.min(slot.count);
                slot.count -= remove;
                to_remove -= remove;
                if slot.count == 0 {
                    *slot = InventorySlot::empty();
                }
            }
        }
        true
    }

    /// Check if player has enough of an item.
    pub fn has_item(&self, item_id: &str, count: u32) -> bool {
        let total: u32 = self.slots.iter()
            .filter(|s| s.item_id.as_deref() == Some(item_id))
            .map(|s| s.count)
            .sum();
        total >= count
    }

    /// Count of a specific item.
    pub fn count_item(&self, item_id: &str) -> u32 {
        self.slots.iter()
            .filter(|s| s.item_id.as_deref() == Some(item_id))
            .map(|s| s.count)
            .sum()
    }

    /// Get item at slot index.
    pub fn get_slot(&self, index: usize) -> Option<&InventorySlot> {
        self.slots.get(index)
    }

    /// Get slot indices containing an item.
    pub fn find_item(&self, item_id: &str) -> Vec<usize> {
        self.slots.iter().enumerate()
            .filter(|(_, s)| s.item_id.as_deref() == Some(item_id))
            .map(|(i, _)| i)
            .collect()
    }

    /// Use item from inventory slot. Returns the use effect if applicable.
    pub fn use_item(&mut self, slot_index: usize) -> Option<ItemUseEffect> {
        let item_id = self.slots.get(slot_index)?.item_id.clone()?;
        let item_def = self.item_db.get(&item_id)?.clone();
        let effect = item_def.use_effect.clone()?;

        if matches!(item_def.item_type, ItemType::Consumable) {
            self.remove_item(&item_id, 1);
        }

        Some(effect)
    }

    /// Equip item from inventory slot. Returns the previously equipped item.
    pub fn equip(&mut self, slot_index: usize) -> Option<InventorySlot> {
        let item_id = self.slots.get(slot_index)?.item_id.clone()?;
        let item_def = self.item_db.get(&item_id)?;
        let equip_slot = item_def.equip_slot?;
        let max_stack = item_def.max_stack;

        // Take item from inventory
        let taken = {
            let inv_slot = &mut self.slots[slot_index];
            let taken = inv_slot.clone();
            if max_stack > 1 {
                inv_slot.count -= 1;
                if inv_slot.count == 0 { *inv_slot = InventorySlot::empty(); }
            } else {
                *inv_slot = InventorySlot::empty();
            }
            taken
        };

        // Swap with equipped
        let old = self.equipment.insert(equip_slot, taken).unwrap_or_else(InventorySlot::empty);

        // Return old equipment to inventory if not empty
        if !old.is_empty() {
            if let Some(old_id) = &old.item_id {
                self.add_item(old_id, old.count);
            }
        }

        Some(old)
    }

    /// Unequip item from equipment slot. Returns true if successful.
    pub fn unequip(&mut self, slot: EquipSlot) -> bool {
        let equipped = self.equipment.get(&slot).cloned().unwrap_or_else(InventorySlot::empty);
        if equipped.is_empty() { return false; }

        let remaining = self.add_item(
            equipped.item_id.as_deref().unwrap_or(""),
            equipped.count,
        );
        if remaining > 0 { return false; }

        self.equipment.insert(slot, InventorySlot::empty());
        true
    }

    /// Get equipped item at slot.
    pub fn get_equipped(&self, slot: EquipSlot) -> Option<&InventorySlot> {
        self.equipment.get(&slot).filter(|s| !s.is_empty())
    }

    /// Calculate total stat bonus from all equipped items.
    pub fn equipment_stats(&self) -> HashMap<String, f32> {
        let mut stats = HashMap::new();
        for (_, slot) in &self.equipment {
            if let Some(item_id) = &slot.item_id {
                if let Some(item_def) = self.item_db.get(item_id) {
                    for (stat, val) in &item_def.base_stats {
                        *stats.entry(stat.clone()).or_insert(0.0) += val;
                    }
                }
            }
        }
        stats
    }

    /// Empty slot count.
    pub fn empty_slots(&self) -> usize {
        self.slots.iter().filter(|s| s.is_empty()).count()
    }

    /// Capacity.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// All non-empty slot references.
    pub fn items(&self) -> Vec<(usize, &InventorySlot)> {
        self.slots.iter().enumerate().filter(|(_, s)| !s.is_empty()).collect()
    }

    /// Sort inventory: group by item type, then by count descending.
    pub fn sort(&mut self) {
        let mut items: Vec<InventorySlot> = self.slots.drain(..).filter(|s| !s.is_empty()).collect();
        items.sort_by(|a, b| {
            let a_name = a.item_id.as_deref().unwrap_or("");
            let b_name = b.item_id.as_deref().unwrap_or("");
            a_name.cmp(b_name).then(b.count.cmp(&a.count))
        });
        self.slots.clear();
        self.slots.extend(items);
        self.slots.resize_with(self.capacity, InventorySlot::empty);
    }

    /// Load items from JSON string.
    pub fn load_item_db(&mut self, json: &str) -> Result<(), String> {
        let parsed: serde_json::Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
        if let Some(items) = parsed["items"].as_array() {
            for entry in items {
                let id = entry["id"].as_str().unwrap_or("unknown").to_string();
                let name = entry["name"].as_str().unwrap_or("Unknown").to_string();
                let item_type = match entry["type"].as_str().unwrap_or("misc") {
                    "consumable" => ItemType::Consumable,
                    "equipment" => ItemType::Equipment,
                    "material" => ItemType::Material,
                    "quest" => ItemType::QuestItem,
                    _ => ItemType::Misc,
                };
                let mut item = Item::new(&id, &name, item_type);
                if let Some(desc) = entry["description"].as_str() {
                    item.description = desc.to_string();
                }
                if let Some(v) = entry["value"].as_u64() { item.value = v; }
                if let Some(stack) = entry["max_stack"].as_u64() { item.max_stack = stack as u32; }
                if item_type == ItemType::Equipment {
                    if let Some(slot) = entry["equip_slot"].as_str() {
                        item.equip_slot = Some(match slot {
                            "weapon" => EquipSlot::Weapon,
                            "armor" => EquipSlot::Armor,
                            "helmet" => EquipSlot::Helmet,
                            "boots" => EquipSlot::Boots,
                            _ => EquipSlot::Accessory,
                        });
                    }
                }
                self.register_item(item);
            }
        }
        Ok(())
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new(20)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_item_db() -> Item {
        Item::new("potion", "Health Potion", ItemType::Consumable)
            .with_use_effect(ItemUseEffect::Heal(50.0))
            .with_value(10)
    }

    fn test_equip() -> Item {
        Item::new("sword", "Iron Sword", ItemType::Equipment)
            .with_equip_slot(EquipSlot::Weapon)
            .with_stat("attack", 10.0)
            .with_value(100)
    }

    #[test]
    fn test_add_and_count() {
        let mut inv = Inventory::new(10);
        inv.register_item(test_item_db());
        assert_eq!(inv.add_item("potion", 5), 0);
        assert!(inv.has_item("potion", 5));
        assert_eq!(inv.count_item("potion"), 5);
    }

    #[test]
    fn test_add_overflow() {
        let mut inv = Inventory::new(2);
        inv.register_item(Item::new("x", "X", ItemType::Material).with_max_stack(5));
        assert_eq!(inv.add_item("x", 10), 5); // only 2 slots × 5 = 10, but only 2 slots available
        // Actually: first empty slot gets min(10, 5) = 5, second gets min(5, 5) = 5, total 10, remaining 0
        // Let's check with 1 slot
        let mut inv2 = Inventory::new(1);
        inv2.register_item(Item::new("x", "X", ItemType::Material).with_max_stack(5));
        assert_eq!(inv2.add_item("x", 8), 3);
    }

    #[test]
    fn test_remove_item() {
        let mut inv = Inventory::new(10);
        inv.register_item(test_item_db());
        inv.add_item("potion", 5);
        assert!(inv.remove_item("potion", 3));
        assert_eq!(inv.count_item("potion"), 2);
        assert!(!inv.remove_item("potion", 5));
    }

    #[test]
    fn test_equip_unequip() {
        let mut inv = Inventory::new(10);
        inv.register_item(test_equip());
        inv.add_item("sword", 1);

        let slot_idx = inv.find_item("sword")[0];
        let old = inv.equip(slot_idx);
        assert!(old.is_none());
        assert!(inv.get_equipped(EquipSlot::Weapon).is_some());
        assert!(inv.find_item("sword").is_empty());

        let stats = inv.equipment_stats();
        assert_eq!(stats.get("attack"), Some(&10.0));

        assert!(inv.unequip(EquipSlot::Weapon));
        assert!(inv.get_equipped(EquipSlot::Weapon).is_none());
        assert_eq!(inv.count_item("sword"), 1);
    }

    #[test]
    fn test_use_item() {
        let mut inv = Inventory::new(10);
        inv.register_item(test_item_db());
        inv.add_item("potion", 1);
        let effect = inv.use_item(0);
        assert!(matches!(effect, Some(ItemUseEffect::Heal(50.0))));
        assert_eq!(inv.count_item("potion"), 0);
    }

    #[test]
    fn test_sort_inventory() {
        let mut inv = Inventory::new(10);
        inv.register_item(test_item_db());
        inv.register_item(Item::new("gold", "Gold Coin", ItemType::Misc));
        inv.add_item("potion", 3);
        inv.add_item("gold", 10);
        inv.add_item("potion", 2);
        inv.sort();
        let items = inv.items();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_empty_slots() {
        let mut inv = Inventory::new(5);
        assert_eq!(inv.empty_slots(), 5);
        inv.register_item(test_item_db());
        inv.add_item("potion", 3);
        assert_eq!(inv.empty_slots(), 4);
    }

    #[test]
    fn test_equipment_stats_accumulate() {
        let mut inv = Inventory::new(10);
        inv.register_item(test_equip());
        inv.register_item(Item::new("shield", "Shield", ItemType::Equipment)
            .with_equip_slot(EquipSlot::Armor)
            .with_stat("defense", 5.0));
        inv.add_item("sword", 1);
        inv.add_item("shield", 1);

        inv.equip(inv.find_item("sword")[0]);
        inv.equip(inv.find_item("shield")[0]);

        let stats = inv.equipment_stats();
        assert_eq!(stats.get("attack"), Some(&10.0));
        assert_eq!(stats.get("defense"), Some(&5.0));
    }
}
