use crate::core::Resource;
use super::inventory::{Item, ItemCategory};
use super::item::ItemQuality;
use super::economy::Economy;
use super::inventory::Inventory;
use super::time::Season;

#[derive(Debug, Clone)]
pub struct ShopItem {
    pub item: Item,
    pub price: u32,
    pub stock: u32,
    pub infinite: bool,
    pub season_limit: Option<Season>,
}

pub struct Shop {
    pub name: String,
    pub owner: String,
    pub items: Vec<ShopItem>,
    pub open_hour: u32,
    pub close_hour: u32,
}

impl Shop {
    pub fn pierre_shop() -> Self {
        Self {
            name: "Pierre's Thought Market".to_string(),
            owner: "Pierre".to_string(),
            items: vec![
                ShopItem {
                    item: Item {
                        id: 1001,
                        name: "Basic Thought Seed".to_string(),
                        description: "A simple seed of awareness".to_string(),
                        stack: 1,
                        max_stack: 99,
                        base_value: 20,
                        category: ItemCategory::Seed,
                        quality: ItemQuality::Normal,
                    },
                    price: 20,
                    stock: 999,
                    infinite: true,
                    season_limit: None,
                },
                ShopItem {
                    item: Item {
                        id: 1002,
                        name: "Curiosity Seed".to_string(),
                        description: "Seeds of wonder".to_string(),
                        stack: 1,
                        max_stack: 99,
                        base_value: 40,
                        category: ItemCategory::Seed,
                        quality: ItemQuality::Normal,
                    },
                    price: 40,
                    stock: 999,
                    infinite: true,
                    season_limit: None,
                },
                ShopItem {
                    item: Item {
                        id: 1003,
                        name: "Logic Seed".to_string(),
                        description: "Seeds of reason".to_string(),
                        stack: 1,
                        max_stack: 99,
                        base_value: 60,
                        category: ItemCategory::Seed,
                        quality: ItemQuality::Normal,
                    },
                    price: 60,
                    stock: 999,
                    infinite: true,
                    season_limit: Some(Season::Clarity),
                },
                ShopItem {
                    item: Item {
                        id: 1004,
                        name: "Empathy Seed".to_string(),
                        description: "Seeds of compassion".to_string(),
                        stack: 1,
                        max_stack: 99,
                        base_value: 35,
                        category: ItemCategory::Seed,
                        quality: ItemQuality::Normal,
                    },
                    price: 35,
                    stock: 999,
                    infinite: true,
                    season_limit: Some(Season::Flow),
                },
                ShopItem {
                    item: Item {
                        id: 4001,
                        name: "Hoe".to_string(),
                        description: "Tills soil for planting".to_string(),
                        stack: 1,
                        max_stack: 1,
                        base_value: 0,
                        category: ItemCategory::Tool,
                        quality: ItemQuality::Normal,
                    },
                    price: 50,
                    stock: 1,
                    infinite: false,
                    season_limit: None,
                },
                ShopItem {
                    item: Item {
                        id: 4002,
                        name: "Watering Can".to_string(),
                        description: "Waters crops".to_string(),
                        stack: 1,
                        max_stack: 1,
                        base_value: 0,
                        category: ItemCategory::Tool,
                        quality: ItemQuality::Normal,
                    },
                    price: 50,
                    stock: 1,
                    infinite: false,
                    season_limit: None,
                },
                ShopItem {
                    item: Item {
                        id: 4003,
                        name: "Pickaxe".to_string(),
                        description: "Breaks rocks and minerals".to_string(),
                        stack: 1,
                        max_stack: 1,
                        base_value: 0,
                        category: ItemCategory::Tool,
                        quality: ItemQuality::Normal,
                    },
                    price: 50,
                    stock: 1,
                    infinite: false,
                    season_limit: None,
                },
            ],
            open_hour: 9,
            close_hour: 20,
        }
    }

    pub fn traveling_cart() -> Self {
        Self {
            name: "Traveling Thought Cart".to_string(),
            owner: "Mysterious Traveler".to_string(),
            items: vec![
                ShopItem {
                    item: Item {
                        id: 1008,
                        name: "Wisdom Seed".to_string(),
                        description: "Rare wisdom seeds".to_string(),
                        stack: 1,
                        max_stack: 10,
                        base_value: 200,
                        category: ItemCategory::Seed,
                        quality: ItemQuality::Normal,
                    },
                    price: 500,
                    stock: 3,
                    infinite: false,
                    season_limit: None,
                },
                ShopItem {
                    item: Item {
                        id: 3007,
                        name: "Wisdom Ore".to_string(),
                        description: "Precious ore of understanding".to_string(),
                        stack: 1,
                        max_stack: 10,
                        base_value: 200,
                        category: ItemCategory::Mineral,
                        quality: ItemQuality::Normal,
                    },
                    price: 800,
                    stock: 1,
                    infinite: false,
                    season_limit: None,
                },
            ],
            open_hour: 11,
            close_hour: 18,
        }
    }

    pub fn is_open(&self, hour: u32) -> bool {
        hour >= self.open_hour && hour < self.close_hour
    }

    pub fn available_items(&self, current_season: Season, hour: u32) -> Vec<&ShopItem> {
        self.items
            .iter()
            .filter(|si| {
                si.stock > 0
                    && si.season_limit.map_or(true, |s| s == current_season)
            })
            .filter(|_| self.is_open(hour))
            .collect()
    }

    pub fn buy(
        &self,
        item_index: usize,
        economy: &mut Economy,
        inventory: &mut Inventory,
        current_season: Season,
        hour: u32,
    ) -> Option<Item> {
        if !self.is_open(hour) {
            return None;
        }
        let shop_item = self.items.get(item_index)?;
        if shop_item.stock == 0 {
            return None;
        }
        if let Some(s) = shop_item.season_limit {
            if s != current_season {
                return None;
            }
        }
        if !economy.can_afford(shop_item.price as i32) {
            return None;
        }
        let added = inventory.add_item(shop_item.item.id, 1, shop_item.item.quality);
        if added > 0 {
            economy.spend(shop_item.price as i32);
            Some(shop_item.item.clone())
        } else {
            None
        }
    }
}

impl Resource for Shop {}

pub struct ShippingBin {
    pub items: Vec<(Item, u32)>,
    pub capacity: u32,
}

impl ShippingBin {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            capacity: 36,
        }
    }

    pub fn add(&mut self, item: Item, quantity: u32) -> bool {
        if self.items.len() < self.capacity as usize {
            self.items.push((item, quantity));
            true
        } else {
            false
        }
    }

    pub fn collect_payment(&mut self, _day: u32) -> Vec<(String, i32)> {
        let mut payments = Vec::new();
        for (item, qty) in self.items.drain(..) {
            let payment = item.base_value as i32 * qty as i32;
            payments.push((item.name, payment));
        }
        payments
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn count(&self) -> usize {
        self.items.len()
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }
}

impl Default for ShippingBin {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_item(id: u32, name: &str, value: u32) -> Item {
        Item {
            id,
            name: name.to_string(),
            description: String::new(),
            stack: 1,
            max_stack: 99,
            base_value: value,
            category: ItemCategory::Crop,
            quality: ItemQuality::Normal,
        }
    }

    #[test]
    fn test_shop_hours() {
        let shop = Shop::pierre_shop();
        assert!(shop.is_open(12));
        assert!(!shop.is_open(8));
        assert!(!shop.is_open(20));
    }

    #[test]
    fn test_buy_basic() {
        let mut eco = Economy::new(200);
        let mut inv = Inventory::new(20, 12);
        let shop = Shop::pierre_shop();
        let result = shop.buy(0, &mut eco, &mut inv, Season::Clarity, 12);
        assert!(result.is_some());
        assert_eq!(eco.gold, 180);
        assert!(inv.has_item(1001));
    }

    #[test]
    fn test_buy_too_poor() {
        let mut eco = Economy::new(5);
        let mut inv = Inventory::new(20, 12);
        let shop = Shop::pierre_shop();
        let result = shop.buy(0, &mut eco, &mut inv, Season::Clarity, 12);
        assert!(result.is_none());
        assert_eq!(eco.gold, 5);
    }

    #[test]
    fn test_buy_wrong_season() {
        let mut eco = Economy::new(500);
        let mut inv = Inventory::new(20, 12);
        let shop = Shop::pierre_shop();
        // Logic Seed (index 2) is Season::Clarity only
        let result = shop.buy(2, &mut eco, &mut inv, Season::Flow, 12);
        assert!(result.is_none());
    }

    #[test]
    fn test_shipping_bin() {
        let mut bin = ShippingBin::new();
        let item = make_test_item(1, "Test", 50);
        bin.add(item.clone(), 5);
        assert_eq!(bin.count(), 1);
        assert!(!bin.is_empty());
        let payments = bin.collect_payment(1);
        assert_eq!(payments.len(), 1);
        assert_eq!(payments[0].1, 250);
        assert!(bin.is_empty());
    }

    #[test]
    fn test_shipping_bin_capacity() {
        let mut bin = ShippingBin::new();
        bin.capacity = 2;
        let item = make_test_item(1, "A", 10);
        assert!(bin.add(item.clone(), 1));
        assert!(bin.add(item.clone(), 1));
        assert!(!bin.add(item.clone(), 1));
    }

    #[test]
    fn test_available_items_season_filter() {
        let shop = Shop::pierre_shop();
        let items = shop.available_items(Season::Flow, 12);
        // Empathy Seed (Flow-only) and all non-season-locked items should be visible
        assert!(items.len() >= 5);
    }
}
