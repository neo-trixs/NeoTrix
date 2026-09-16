use std::collections::HashMap;

use crate::l5_cognition::nt_mind::nt_game::world::combat::inventory::{
    Inventory, Item, ItemRarity, ItemType,
};

// ═══════════════════════════════════════════════════════════════════
// Equipment System
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EquipSlot {
    Head,
    Chest,
    Legs,
    Feet,
    MainHand,
    OffHand,
    Ring,
    Amulet,
}

impl EquipSlot {
    pub fn all() -> &'static [EquipSlot] {
        &[
            EquipSlot::Head,
            EquipSlot::Chest,
            EquipSlot::Legs,
            EquipSlot::Feet,
            EquipSlot::MainHand,
            EquipSlot::OffHand,
            EquipSlot::Ring,
            EquipSlot::Amulet,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            EquipSlot::Head => "Head",
            EquipSlot::Chest => "Chest",
            EquipSlot::Legs => "Legs",
            EquipSlot::Feet => "Feet",
            EquipSlot::MainHand => "Main Hand",
            EquipSlot::OffHand => "Off Hand",
            EquipSlot::Ring => "Ring",
            EquipSlot::Amulet => "Amulet",
        }
    }
}

impl std::fmt::Display for EquipSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StatType {
    Attack,
    Defense,
    Health,
    Mana,
    Speed,
    CritChance,
    CritDamage,
}

impl StatType {
    pub fn display_name(&self) -> &'static str {
        match self {
            StatType::Attack => "Attack",
            StatType::Defense => "Defense",
            StatType::Health => "Health",
            StatType::Mana => "Mana",
            StatType::Speed => "Speed",
            StatType::CritChance => "Crit Chance",
            StatType::CritDamage => "Crit Damage",
        }
    }
}

impl std::fmt::Display for StatType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[derive(Debug, Clone)]
pub struct EquipableItem {
    pub item_id: u32,
    pub name: String,
    pub description: String,
    pub rarity: ItemRarity,
    pub slot: EquipSlot,
    pub required_level: u32,
    pub stat_bonuses: HashMap<StatType, i32>,
    pub durability_max: u32,
    pub durability_current: u32,
    pub break_chance_per_use: f64,
}

impl EquipableItem {
    pub fn new(item_id: u32, name: &str, slot: EquipSlot) -> Self {
        Self {
            item_id,
            name: name.to_string(),
            description: String::new(),
            rarity: ItemRarity::Common,
            slot,
            required_level: 1,
            stat_bonuses: HashMap::new(),
            durability_max: 100,
            durability_current: 100,
            break_chance_per_use: 0.0,
        }
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    pub fn with_rarity(mut self, rarity: ItemRarity) -> Self {
        self.rarity = rarity;
        self
    }

    pub fn with_stat_bonus(mut self, stat: StatType, amount: i32) -> Self {
        self.stat_bonuses.insert(stat, amount);
        self
    }

    pub fn with_durability(mut self, max: u32) -> Self {
        self.durability_max = max;
        self.durability_current = max;
        self
    }

    pub fn with_break_chance(mut self, chance: f64) -> Self {
        self.break_chance_per_use = chance;
        self
    }

    pub fn with_required_level(mut self, level: u32) -> Self {
        self.required_level = level;
        self
    }

    pub fn is_broken(&self) -> bool {
        self.durability_current == 0
    }

    pub fn durability_pct(&self) -> f64 {
        if self.durability_max == 0 {
            return 0.0;
        }
        self.durability_current as f64 / self.durability_max as f64
    }

    pub fn use_item(&mut self) -> bool {
        if self.is_broken() {
            return false;
        }
        self.durability_current = self.durability_current.saturating_sub(1);
        true
    }

    pub fn repair(&mut self, amount: u32) {
        self.durability_current = (self.durability_current + amount).min(self.durability_max);
    }

    pub fn full_repair(&mut self) {
        self.durability_current = self.durability_max;
    }

    pub fn stat_bonus(&self, stat: &StatType) -> i32 {
        if self.is_broken() {
            0
        } else {
            self.stat_bonuses.get(stat).copied().unwrap_or(0)
        }
    }

    pub fn total_stat_value(&self) -> i32 {
        self.stat_bonuses.values().sum()
    }

    pub fn power_score(&self) -> f64 {
        let stat_power = self.total_stat_value() as f64;
        let rarity_mult = match self.rarity {
            ItemRarity::Common => 1.0,
            ItemRarity::Uncommon => 1.25,
            ItemRarity::Rare => 1.5,
            ItemRarity::Epic => 2.0,
            ItemRarity::Legendary => 3.0,
        };
        stat_power * rarity_mult * self.durability_pct()
    }
}

// ═══════════════════════════════════════════════════════════════════
// Equipment Loadout
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct EquipmentLoadout {
    pub slots: HashMap<EquipSlot, EquipableItem>,
}

impl EquipmentLoadout {
    pub fn new() -> Self {
        Self {
            slots: HashMap::new(),
        }
    }

    pub fn equip(
        &mut self,
        item: EquipableItem,
        level: u32,
    ) -> Result<Option<EquipableItem>, EquipError> {
        if item.required_level > level {
            return Err(EquipError::LevelTooLow {
                required: item.required_level,
                current: level,
            });
        }
        let prev = self.slots.insert(item.slot, item);
        Ok(prev)
    }

    pub fn unequip(&mut self, slot: EquipSlot) -> Option<EquipableItem> {
        self.slots.remove(&slot)
    }

    pub fn get(&self, slot: EquipSlot) -> Option<&EquipableItem> {
        self.slots.get(&slot)
    }

    pub fn get_mut(&mut self, slot: EquipSlot) -> Option<&mut EquipableItem> {
        self.slots.get_mut(&slot)
    }

    pub fn total_stat_bonus(&self, stat: &StatType) -> i32 {
        self.slots.values().map(|item| item.stat_bonus(stat)).sum()
    }

    pub fn total_power_score(&self) -> f64 {
        self.slots.values().map(|item| item.power_score()).sum()
    }

    pub fn equipped_count(&self) -> usize {
        self.slots.len()
    }

    pub fn all_equipped(&self) -> Vec<&EquipableItem> {
        self.slots.values().collect()
    }

    pub fn broken_items(&self) -> Vec<&EquipableItem> {
        self.slots.values().filter(|i| i.is_broken()).collect()
    }

    pub fn repair_all(&mut self) {
        for item in self.slots.values_mut() {
            item.full_repair();
        }
    }

    pub fn repair_slot(&mut self, slot: EquipSlot, amount: u32) -> bool {
        if let Some(item) = self.slots.get_mut(&slot) {
            item.repair(amount);
            true
        } else {
            false
        }
    }

    pub fn compare_power(&self, other: &EquipableItem) -> i32 {
        let current_power = self
            .slots
            .get(&other.slot)
            .map(|i| i.power_score() as i32)
            .unwrap_or(0);
        let new_power = other.power_score() as i32;
        new_power - current_power
    }
}

impl Default for EquipmentLoadout {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════
// Equip Errors
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub enum EquipError {
    LevelTooLow { required: u32, current: u32 },
    SlotOccupied { slot: EquipSlot },
    NotEquipable,
    ItemBroken,
}

impl std::fmt::Display for EquipError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EquipError::LevelTooLow { required, current } => {
                write!(f, "Level too low: need {}, have {}", required, current)
            }
            EquipError::SlotOccupied { slot } => {
                write!(f, "Slot {} is occupied", slot)
            }
            EquipError::NotEquipable => write!(f, "Item is not equipable"),
            EquipError::ItemBroken => write!(f, "Item is broken"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Crafting Recipes
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct CraftingIngredient {
    pub item_id: u32,
    pub item_name: String,
    pub count: u32,
}

impl CraftingIngredient {
    pub fn new(item_id: u32, item_name: &str, count: u32) -> Self {
        Self {
            item_id,
            item_name: item_name.to_string(),
            count,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CraftingRecipe {
    pub id: String,
    pub name: String,
    pub ingredients: Vec<CraftingIngredient>,
    pub result_item_id: u32,
    pub result_name: String,
    pub result_count: u32,
    pub required_level: u32,
    pub crafting_time_ms: u64,
    pub skill_required: Option<String>,
}

impl CraftingRecipe {
    pub fn new(id: &str, name: &str, result_item_id: u32, result_name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            ingredients: Vec::new(),
            result_item_id,
            result_name: result_name.to_string(),
            result_count: 1,
            required_level: 1,
            crafting_time_ms: 1000,
            skill_required: None,
        }
    }

    pub fn with_ingredient(mut self, item_id: u32, item_name: &str, count: u32) -> Self {
        self.ingredients
            .push(CraftingIngredient::new(item_id, item_name, count));
        self
    }

    pub fn with_result_count(mut self, count: u32) -> Self {
        self.result_count = count;
        self
    }

    pub fn with_required_level(mut self, level: u32) -> Self {
        self.required_level = level;
        self
    }

    pub fn with_crafting_time(mut self, ms: u64) -> Self {
        self.crafting_time_ms = ms;
        self
    }

    pub fn with_skill_required(mut self, skill: &str) -> Self {
        self.skill_required = Some(skill.to_string());
        self
    }

    pub fn can_craft(&self, inventory: &Inventory, player_level: u32) -> bool {
        if player_level < self.required_level {
            return false;
        }
        for ingredient in &self.ingredients {
            if inventory.count_item(ingredient.item_id) < ingredient.count {
                return false;
            }
        }
        true
    }

    pub fn craft(&self, inventory: &mut Inventory) -> Result<(), CraftError> {
        if !self.can_craft(inventory, u32::MAX) {
            for ingredient in &self.ingredients {
                if inventory.count_item(ingredient.item_id) < ingredient.count {
                    return Err(CraftError::MissingIngredient {
                        item_name: ingredient.item_name.clone(),
                        required: ingredient.count,
                        available: inventory.count_item(ingredient.item_id),
                    });
                }
            }
            return Err(CraftError::InsufficientLevel {
                required: self.required_level,
            });
        }

        for ingredient in &self.ingredients {
            inventory.remove_item(ingredient.item_id, ingredient.count);
        }

        let result_item = Item {
            id: self.result_item_id,
            name: self.result_name.clone(),
            description: format!("Crafted: {}", self.name),
            rarity: ItemRarity::Uncommon,
            stackable: true,
            max_stack: 99,
            weight: 1.0,
            value: 0,
            item_type: ItemType::Misc,
        };

        let leftover = inventory.add_item(result_item, self.result_count);
        if leftover > 0 {
            return Err(CraftError::InventoryFull {
                items_lost: leftover,
            });
        }

        Ok(())
    }

    pub fn ingredient_summary(&self) -> String {
        self.ingredients
            .iter()
            .map(|i| format!("{} x{}", i.item_name, i.count))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

// ═══════════════════════════════════════════════════════════════════
// Craft Errors
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub enum CraftError {
    MissingIngredient {
        item_name: String,
        required: u32,
        available: u32,
    },
    InsufficientLevel {
        required: u32,
    },
    InventoryFull {
        items_lost: u32,
    },
    MissingSkill {
        skill: String,
    },
}

impl std::fmt::Display for CraftError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CraftError::MissingIngredient {
                item_name,
                required,
                available,
            } => write!(f, "Need {} x{}, have {}", item_name, required, available),
            CraftError::InsufficientLevel { required } => {
                write!(f, "Crafting level {} required", required)
            }
            CraftError::InventoryFull { items_lost } => {
                write!(f, "Inventory full, {} items lost", items_lost)
            }
            CraftError::MissingSkill { skill } => {
                write!(f, "Missing required skill: {}", skill)
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Recipe Book
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct RecipeBook {
    pub recipes: Vec<CraftingRecipe>,
    pub discovered: Vec<String>,
}

impl RecipeBook {
    pub fn new() -> Self {
        Self {
            recipes: Vec::new(),
            discovered: Vec::new(),
        }
    }

    pub fn add_recipe(&mut self, recipe: CraftingRecipe) {
        self.recipes.push(recipe);
    }

    pub fn discover(&mut self, recipe_id: &str) -> bool {
        if !self.discovered.contains(&recipe_id.to_string()) {
            self.discovered.push(recipe_id.to_string());
            true
        } else {
            false
        }
    }

    pub fn is_discovered(&self, recipe_id: &str) -> bool {
        self.discovered.contains(&recipe_id.to_string())
    }

    pub fn find_craftable(&self, inventory: &Inventory, level: u32) -> Vec<&CraftingRecipe> {
        self.recipes
            .iter()
            .filter(|r| self.is_discovered(&r.id) && r.can_craft(inventory, level))
            .collect()
    }

    pub fn find_by_ingredient(&self, item_id: u32) -> Vec<&CraftingRecipe> {
        self.recipes
            .iter()
            .filter(|r| r.ingredients.iter().any(|i| i.item_id == item_id))
            .collect()
    }
}

impl Default for RecipeBook {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::inventory::Inventory;
    use super::*;

    fn iron_sword_recipe() -> CraftingRecipe {
        CraftingRecipe::new("iron_sword", "Iron Sword", 101, "Iron Sword")
            .with_ingredient(200, "Iron Ingot", 3)
            .with_ingredient(201, "Wood Handle", 1)
            .with_result_count(1)
    }

    #[test]
    fn test_equip_unequip() {
        let mut loadout = EquipmentLoadout::new();
        let sword = EquipableItem::new(1, "Sword", EquipSlot::MainHand)
            .with_stat_bonus(StatType::Attack, 10);

        let prev = loadout.equip(sword, 1).unwrap();
        assert!(prev.is_none());
        assert_eq!(loadout.get(EquipSlot::MainHand).unwrap().name, "Sword");

        let removed = loadout.unequip(EquipSlot::MainHand);
        assert!(removed.is_some());
    }

    #[test]
    fn test_level_requirement() {
        let mut loadout = EquipmentLoadout::new();
        let sword = EquipableItem::new(1, "Sword", EquipSlot::MainHand).with_required_level(10);

        let result = loadout.equip(sword, 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_durability() {
        let mut item = EquipableItem::new(1, "Shield", EquipSlot::OffHand).with_durability(50);
        assert_eq!(item.durability_pct(), 1.0);
        assert!(item.use_item());
        assert_eq!(item.durability_current, 49);
        item.repair(10);
        assert_eq!(item.durability_current, 50);
        item.full_repair();
        assert_eq!(item.durability_current, 50);
    }

    #[test]
    fn test_power_score() {
        let item = EquipableItem::new(1, "Sword", EquipSlot::MainHand)
            .with_stat_bonus(StatType::Attack, 10)
            .with_rarity(ItemRarity::Rare)
            .with_durability(100);
        let power = item.power_score();
        assert!(power > 0.0);
    }

    #[test]
    fn test_recipe_can_craft() {
        let recipe = iron_sword_recipe();
        let mut inv = Inventory::new(20);
        inv.add_item(
            Item {
                id: 200,
                name: "Iron Ingot".into(),
                description: "".into(),
                rarity: ItemRarity::Common,
                stackable: true,
                max_stack: 99,
                weight: 1.0,
                value: 10,
                item_type: ItemType::Material,
            },
            3,
        );
        inv.add_item(
            Item {
                id: 201,
                name: "Wood Handle".into(),
                description: "".into(),
                rarity: ItemRarity::Common,
                stackable: true,
                max_stack: 99,
                weight: 0.5,
                value: 2,
                item_type: ItemType::Material,
            },
            1,
        );
        assert!(recipe.can_craft(&inv, 1));
    }

    #[test]
    fn test_recipe_craft() {
        let recipe = iron_sword_recipe();
        let mut inv = Inventory::new(20);
        inv.add_item(
            Item {
                id: 200,
                name: "Iron Ingot".into(),
                description: "".into(),
                rarity: ItemRarity::Common,
                stackable: true,
                max_stack: 99,
                weight: 1.0,
                value: 10,
                item_type: ItemType::Material,
            },
            3,
        );
        inv.add_item(
            Item {
                id: 201,
                name: "Wood Handle".into(),
                description: "".into(),
                rarity: ItemRarity::Common,
                stackable: true,
                max_stack: 99,
                weight: 0.5,
                value: 2,
                item_type: ItemType::Material,
            },
            1,
        );

        let result = recipe.craft(&mut inv);
        assert!(result.is_ok());
        assert_eq!(inv.count_item(200), 0);
        assert_eq!(inv.count_item(201), 0);
        assert_eq!(inv.count_item(101), 1);
    }

    #[test]
    fn test_recipe_missing_ingredients() {
        let recipe = iron_sword_recipe();
        let mut inv = Inventory::new(20);
        let result = recipe.craft(&mut inv);
        assert!(result.is_err());
    }

    #[test]
    fn test_recipe_book() {
        let mut book = RecipeBook::new();
        let recipe = iron_sword_recipe();
        book.add_recipe(recipe);
        book.discover("iron_sword");
        assert!(book.is_discovered("iron_sword"));
        assert!(!book.is_discovered("unknown"));
    }
}
