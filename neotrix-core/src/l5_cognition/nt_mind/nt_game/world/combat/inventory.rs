#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ItemRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

impl ItemRarity {
    pub fn color(&self) -> &str {
        match self {
            ItemRarity::Common => "#ffffff",
            ItemRarity::Uncommon => "#1eff00",
            ItemRarity::Rare => "#0070dd",
            ItemRarity::Epic => "#a335ee",
            ItemRarity::Legendary => "#ff8000",
        }
    }
    pub fn drop_weight(&self) -> f64 {
        match self {
            ItemRarity::Common => 50.0,
            ItemRarity::Uncommon => 30.0,
            ItemRarity::Rare => 15.0,
            ItemRarity::Epic => 4.0,
            ItemRarity::Legendary => 1.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Item {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub rarity: ItemRarity,
    pub stackable: bool,
    pub max_stack: u32,
    pub weight: f64,
    pub value: u32,
    pub item_type: ItemType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemType {
    Weapon,
    Armor,
    Consumable,
    Material,
    Quest,
    Misc,
}

#[derive(Debug, Clone)]
pub struct ItemStack {
    pub item: Item,
    pub count: u32,
}

impl ItemStack {
    pub fn new(item: Item, count: u32) -> Self {
        let max_stack = item.max_stack;
        Self {
            item,
            count: count.min(max_stack),
        }
    }
    pub fn add(&mut self, amount: u32) -> u32 {
        let can_add = (self.item.max_stack - self.count).min(amount);
        self.count += can_add;
        amount - can_add
    }
    pub fn remove(&mut self, amount: u32) -> u32 {
        let remove = amount.min(self.count);
        self.count -= remove;
        remove
    }
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

pub struct Inventory {
    pub slots: Vec<Option<ItemStack>>,
    pub max_slots: usize,
}

impl Inventory {
    pub fn new(max_slots: usize) -> Self {
        Self {
            slots: vec![None; max_slots],
            max_slots,
        }
    }
    pub fn add_item(&mut self, item: Item, count: u32) -> u32 {
        let mut remaining = count;
        if item.stackable {
            for slot in &mut self.slots {
                if let Some(ref mut stack) = slot {
                    if stack.item.id == item.id {
                        let added = stack.add(remaining);
                        remaining -= added;
                        if remaining == 0 {
                            return 0;
                        }
                    }
                }
            }
        }
        for slot in &mut self.slots {
            if slot.is_none() {
                let to_add = remaining.min(item.max_stack);
                *slot = Some(ItemStack::new(item.clone(), to_add));
                remaining -= to_add;
                if remaining == 0 {
                    return 0;
                }
            }
        }
        remaining
    }
    pub fn remove_item(&mut self, item_id: u32, count: u32) -> u32 {
        let mut remaining = count;
        for slot in &mut self.slots {
            if let Some(ref mut stack) = slot {
                if stack.item.id == item_id {
                    let removed = stack.remove(remaining);
                    remaining -= removed;
                    if stack.is_empty() {
                        *slot = None;
                    }
                    if remaining == 0 {
                        return 0;
                    }
                }
            }
        }
        remaining
    }
    pub fn count_item(&self, item_id: u32) -> u32 {
        self.slots
            .iter()
            .filter_map(|s| s.as_ref())
            .filter(|s| s.item.id == item_id)
            .map(|s| s.count)
            .sum()
    }
    pub fn is_full(&self) -> bool {
        self.slots.iter().all(|s| s.is_some())
    }
    pub fn used_slots(&self) -> usize {
        self.slots.iter().filter(|s| s.is_some()).count()
    }
    pub fn total_weight(&self) -> f64 {
        self.slots
            .iter()
            .filter_map(|s| s.as_ref())
            .map(|s| s.item.weight * s.count as f64)
            .sum()
    }
    pub fn sort_by_rarity(&mut self) {
        self.slots.sort_by(|a, b| {
            b.as_ref()
                .map(|s| s.item.rarity)
                .unwrap_or(ItemRarity::Common)
                .cmp(
                    &a.as_ref()
                        .map(|s| s.item.rarity)
                        .unwrap_or(ItemRarity::Common),
                )
        });
    }
}
