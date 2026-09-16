#[derive(Debug, Clone)]
pub struct LootEntry {
    pub item_id: u32,
    pub item_name: String,
    pub drop_rate: f64,
    pub min_count: u32,
    pub max_count: u32,
    pub guaranteed: bool,
}

impl LootEntry {
    pub fn new(item_id: u32, item_name: &str, drop_rate: f64) -> Self {
        Self {
            item_id,
            item_name: item_name.to_string(),
            drop_rate,
            min_count: 1,
            max_count: 1,
            guaranteed: false,
        }
    }

    pub fn with_count(mut self, min: u32, max: u32) -> Self {
        self.min_count = min;
        self.max_count = max;
        self
    }

    pub fn guaranteed(mut self) -> Self {
        self.guaranteed = true;
        self
    }
}

#[derive(Debug, Clone)]
pub struct RollResult {
    pub item_id: u32,
    pub item_name: String,
    pub count: u32,
}

pub struct LootTable {
    pub entries: Vec<LootEntry>,
    pub max_drops: u32,
    pub bonus_drop_rate: f64,
}

impl LootTable {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            max_drops: 5,
            bonus_drop_rate: 0.0,
        }
    }

    pub fn with_entry(mut self, entry: LootEntry) -> Self {
        self.entries.push(entry);
        self
    }

    pub fn with_max_drops(mut self, max: u32) -> Self {
        self.max_drops = max;
        self
    }

    pub fn roll(&self) -> Vec<RollResult> {
        let mut drops = Vec::new();
        for entry in &self.entries {
            if entry.guaranteed {
                let count = entry.min_count;
                drops.push(RollResult {
                    item_id: entry.item_id,
                    item_name: entry.item_name.clone(),
                    count,
                });
            } else {
                let effective_rate = (entry.drop_rate + self.bonus_drop_rate).min(1.0);
                if rand_f64() < effective_rate {
                    let count = entry.min_count
                        + (rand_f64() * (entry.max_count - entry.min_count) as f64) as u32;
                    drops.push(RollResult {
                        item_id: entry.item_id,
                        item_name: entry.item_name.clone(),
                        count,
                    });
                }
            }
        }
        drops.truncate(self.max_drops as usize);
        drops
    }

    pub fn expected_value(&self) -> f64 {
        self.entries
            .iter()
            .map(|e| {
                let rate = e.drop_rate + self.bonus_drop_rate;
                let avg_count = (e.min_count + e.max_count) as f64 / 2.0;
                rate * avg_count
            })
            .sum()
    }
}

impl Default for LootTable {
    fn default() -> Self {
        Self::new()
    }
}

fn rand_f64() -> f64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut h = DefaultHasher::new();
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .hash(&mut h);
    h.finish() as f64 / u64::MAX as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loot_roll_guaranteed() {
        let table = LootTable::new().with_entry(LootEntry::new(1, "Sword", 0.5).guaranteed());
        let drops = table.roll();
        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0].item_name, "Sword");
    }

    #[test]
    fn test_loot_empty() {
        let table = LootTable::new();
        assert!(table.roll().is_empty());
    }
}
