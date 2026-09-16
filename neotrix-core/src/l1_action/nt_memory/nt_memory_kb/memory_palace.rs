use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PalaceRoom {
    pub name: String,
    pub items: Vec<MemoryItem>,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct MemoryItem {
    pub key: String,
    pub value: String,
    pub associations: Vec<String>,
    pub strength: f64,
}

pub struct MemoryPalace {
    rooms: HashMap<String, PalaceRoom>,
    room_order: Vec<String>,
}

impl MemoryPalace {
    pub fn new() -> Self {
        Self {
            rooms: HashMap::new(),
            room_order: Vec::new(),
        }
    }

    pub fn create_room(&mut self, name: &str, description: &str) {
        if !self.rooms.contains_key(name) {
            self.rooms.insert(
                name.to_string(),
                PalaceRoom {
                    name: name.to_string(),
                    items: Vec::new(),
                    description: description.to_string(),
                },
            );
            self.room_order.push(name.to_string());
        }
    }

    pub fn place_item(&mut self, room_name: &str, item: MemoryItem) -> bool {
        if let Some(room) = self.rooms.get_mut(room_name) {
            room.items.push(item);
            true
        } else {
            false
        }
    }

    pub fn recall(&self, key: &str) -> Option<&MemoryItem> {
        for room in self.rooms.values() {
            if let Some(item) = room.items.iter().find(|i| i.key == key) {
                return Some(item);
            }
        }
        None
    }

    pub fn strengthen(&mut self, key: &str, amount: f64) -> bool {
        for room in self.rooms.values_mut() {
            if let Some(item) = room.items.iter_mut().find(|i| i.key == key) {
                item.strength = (item.strength + amount).min(1.0);
                return true;
            }
        }
        false
    }

    pub fn rooms(&self) -> &[String] {
        &self.room_order
    }

    pub fn room_count(&self) -> usize {
        self.rooms.len()
    }

    pub fn total_items(&self) -> usize {
        self.rooms.values().map(|r| r.items.len()).sum()
    }

    pub fn weakest_items(&self, n: usize) -> Vec<(&str, f64)> {
        let mut all: Vec<(&str, f64)> = self
            .rooms
            .values()
            .flat_map(|r| r.items.iter().map(|i| (i.key.as_str(), i.strength)))
            .collect();
        all.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        all.into_iter().take(n).collect()
    }
}

impl Default for MemoryPalace {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_place() {
        let mut p = MemoryPalace::new();
        p.create_room("entrance", "The entrance");
        assert!(p.place_item(
            "entrance",
            MemoryItem {
                key: "k1".into(),
                value: "v1".into(),
                associations: vec![],
                strength: 0.5,
            }
        ));
        assert!(p.recall("k1").is_some());
    }

    #[test]
    fn test_strengthen() {
        let mut p = MemoryPalace::new();
        p.create_room("r1", "d");
        p.place_item(
            "r1",
            MemoryItem {
                key: "k".into(),
                value: "v".into(),
                associations: vec![],
                strength: 0.3,
            },
        );
        p.strengthen("k", 0.5);
        assert!((p.recall("k").unwrap().strength - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_weakest() {
        let mut p = MemoryPalace::new();
        p.create_room("r", "d");
        p.place_item(
            "r",
            MemoryItem {
                key: "a".into(),
                value: "v".into(),
                associations: vec![],
                strength: 0.1,
            },
        );
        p.place_item(
            "r",
            MemoryItem {
                key: "b".into(),
                value: "v".into(),
                associations: vec![],
                strength: 0.9,
            },
        );
        let w = p.weakest_items(1);
        assert_eq!(w[0].0, "a");
    }
}
