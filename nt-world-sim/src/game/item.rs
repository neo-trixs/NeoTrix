use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemType {
    Seed,
    Crop,
    Tool,
    Material,
    Fish,
    Forage,
    Gift,
    Artifact,
    Consumable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemQuality {
    Normal,
    Silver,
    Gold,
    Iridium,
}

impl ItemQuality {
    pub fn multiplier(&self) -> f32 {
        match self {
            ItemQuality::Normal => 1.0,
            ItemQuality::Silver => 1.1,
            ItemQuality::Gold => 1.25,
            ItemQuality::Iridium => 1.5,
        }
    }
    
    pub fn star_char(&self) -> &str {
        match self {
            ItemQuality::Normal => "",
            ItemQuality::Silver => "★",
            ItemQuality::Gold => "★★",
            ItemQuality::Iridium => "★★★",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ItemDef {
    pub id: u32,
    pub name: String,
    pub item_type: ItemType,
    pub base_price: i32,
    pub description: String,
    pub stackable: bool,
    pub max_stack: u32,
    pub seasons: Vec<u32>,
    pub growth_days: Option<u32>,
    pub regrowth_days: Option<u32>,
}

impl ItemDef {
    pub fn seed(id: u32, name: &str, price: i32, growth_days: u32, seasons: Vec<u32>) -> Self {
        Self {
            id, name: name.to_string(), item_type: ItemType::Seed, base_price: price,
            description: format!("Plant in {} days", growth_days), stackable: true, max_stack: 999,
            seasons, growth_days: Some(growth_days), regrowth_days: None,
        }
    }
    
    pub fn crop(id: u32, name: &str, price: i32) -> Self {
        Self {
            id, name: name.to_string(), item_type: ItemType::Crop, base_price: price,
            description: "A harvested crop".to_string(), stackable: true, max_stack: 999,
            seasons: vec![], growth_days: None, regrowth_days: None,
        }
    }
    
    pub fn tool(id: u32, name: &str) -> Self {
        Self {
            id, name: name.to_string(), item_type: ItemType::Tool, base_price: 0,
            description: "A tool".to_string(), stackable: false, max_stack: 1,
            seasons: vec![], growth_days: None, regrowth_days: None,
        }
    }
}

pub struct ItemRegistry {
    items: HashMap<u32, ItemDef>,
}

impl ItemRegistry {
    pub fn new() -> Self {
        let mut items = HashMap::new();
        
        // Seeds
        items.insert(1, ItemDef::seed(1, "Thought Seed", 10, 4, vec![0, 1, 2]));
        items.insert(2, ItemDef::seed(2, "Insight Bulb", 15, 6, vec![1, 2]));
        items.insert(3, ItemDef::seed(3, "Memory Bloom", 20, 8, vec![2, 3]));
        items.insert(4, ItemDef::seed(4, "Wisdom Root", 30, 12, vec![0, 1, 2, 3]));
        
        // Crops
        items.insert(101, ItemDef::crop(101, "Thought Fruit", 25));
        items.insert(102, ItemDef::crop(102, "Insight Gem", 40));
        items.insert(103, ItemDef::crop(103, "Memory Petal", 60));
        items.insert(104, ItemDef::crop(104, "Wisdom Essence", 100));
        
        // Tools
        items.insert(201, ItemDef::tool(201, "Hoe"));
        items.insert(202, ItemDef::tool(202, "Watering Can"));
        items.insert(203, ItemDef::tool(203, "Pickaxe"));
        items.insert(204, ItemDef::tool(204, "Axe"));
        
        Self { items }
    }
    
    pub fn get(&self, id: u32) -> Option<&ItemDef> {
        self.items.get(&id)
    }
    
    pub fn register(&mut self, item: ItemDef) {
        self.items.insert(item.id, item);
    }
}

impl Default for ItemRegistry {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_item_registry() {
        let reg = ItemRegistry::new();
        assert!(reg.get(1).is_some());
        assert_eq!(reg.get(1).unwrap().name, "Thought Seed");
    }
    
    #[test]
    fn test_quality_multiplier() {
        assert!((ItemQuality::Normal.multiplier() - 1.0).abs() < 0.01);
        assert!((ItemQuality::Gold.multiplier() - 1.25).abs() < 0.01);
    }
}
