use std::collections::{HashMap, HashSet};

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
    /// Inverted index: keyword → set of item keys
    keyword_index: HashMap<String, HashSet<String>>,
}

impl MemoryPalace {
    pub fn new() -> Self {
        Self {
            rooms: HashMap::new(),
            room_order: Vec::new(),
            keyword_index: HashMap::new(),
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
            // Index keywords from key + value + associations
            let keywords = extract_keywords(&item.key, &item.value, &item.associations);
            let key = item.key.clone();
            for kw in keywords {
                self.keyword_index
                    .entry(kw)
                    .or_insert_with(HashSet::new)
                    .insert(key.clone());
            }
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

    /// Semantic recall — find items matching query keywords, ranked by strength
    pub fn recall_semantic(&self, query: &str, limit: usize) -> Vec<(&MemoryItem, f64)> {
        let query_keywords = tokenize(query);
        if query_keywords.is_empty() {
            return Vec::new();
        }

        // Score each item: sum of matching keyword strengths × item strength
        let mut scores: HashMap<String, f64> = HashMap::new();
        for kw in &query_keywords {
            if let Some(matching_keys) = self.keyword_index.get(kw.as_str()) {
                for key in matching_keys {
                    let entry = scores.entry(key.clone()).or_insert(0.0);
                    *entry += 1.0; // one point per matching keyword
                }
            }
        }

        // Convert to (item, score) and sort by score × strength
        let mut results: Vec<(&MemoryItem, f64)> = scores
            .iter()
            .filter_map(|(key, keyword_score)| {
                self.recall(key)
                    .map(|item| (item, *keyword_score * item.strength))
            })
            .collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.into_iter().take(limit).collect()
    }

    /// Semantic search across all rooms — returns items with relevance scores
    pub fn semantic_search(&self, query: &str, limit: usize) -> Vec<SemanticMatch> {
        let query_keywords = tokenize(query);
        if query_keywords.is_empty() {
            return Vec::new();
        }

        let query_set: HashSet<&str> = query_keywords.iter().map(|s| s.as_str()).collect();

        let mut results: Vec<SemanticMatch> = Vec::new();
        for room in self.rooms.values() {
            for item in &room.items {
                let item_keywords = extract_keywords(&item.key, &item.value, &item.associations);
                let item_set: HashSet<&str> = item_keywords.iter().map(|s| s.as_str()).collect();

                // Jaccard-like similarity: intersection / union
                let intersection = query_set.intersection(&item_set).count();
                let union = query_set.union(&item_set).count();
                let similarity = if union > 0 {
                    intersection as f64 / union as f64
                } else {
                    0.0
                };

                if similarity > 0.0 {
                    results.push(SemanticMatch {
                        item: item.clone(),
                        room: room.name.clone(),
                        similarity,
                        strength: item.strength,
                        score: similarity * item.strength,
                    });
                }
            }
        }

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.into_iter().take(limit).collect()
    }

    /// Get keyword index stats
    pub fn keyword_stats(&self) -> KeywordStats {
        let total_keywords = self.keyword_index.len();
        let avg_entries_per_keyword = if total_keywords > 0 {
            self.keyword_index.values().map(|s| s.len()).sum::<usize>() as f64
                / total_keywords as f64
        } else {
            0.0
        };
        KeywordStats {
            total_keywords,
            avg_entries_per_keyword,
        }
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

/// Semantic search result
#[derive(Debug, Clone)]
pub struct SemanticMatch {
    pub item: MemoryItem,
    pub room: String,
    pub similarity: f64,
    pub strength: f64,
    pub score: f64,
}

/// Keyword index stats
#[derive(Debug, Clone)]
pub struct KeywordStats {
    pub total_keywords: usize,
    pub avg_entries_per_keyword: f64,
}

impl Default for MemoryPalace {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Keyword Helpers ────────────────────────────────────────────────────────

/// Tokenize a string into lowercase keywords (2+ chars, no stopwords)
fn tokenize(s: &str) -> Vec<String> {
    s.to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
        .filter(|w| w.len() >= 2 && !STOPWORDS.contains(w))
        .map(|w| w.to_string())
        .collect()
}

/// Extract keywords from item fields
fn extract_keywords(key: &str, value: &str, associations: &[String]) -> Vec<String> {
    let mut all = tokenize(key);
    all.extend(tokenize(value));
    for assoc in associations {
        all.extend(tokenize(assoc));
    }
    all.sort();
    all.dedup();
    all
}

/// Common stopwords to filter out
const STOPWORDS: &[&str] = &[
    "the", "is", "at", "of", "on", "in", "to", "for", "a", "an", "and", "or", "but",
    "with", "by", "from", "as", "this", "that", "it", "be", "are", "was", "were",
    "的", "了", "在", "是", "和", "与", "或", "但", "对", "从", "到", "为",
];

// ─── Tests ──────────────────────────────────────────────────────────────────

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

    #[test]
    fn test_semantic_recall() {
        let mut p = MemoryPalace::new();
        p.create_room("code", "Code knowledge");
        p.place_item(
            "code",
            MemoryItem {
                key: "rust-ownership".into(),
                value: "Rust ownership prevents data races at compile time".into(),
                associations: vec!["memory".into(), "safety".into()],
                strength: 0.8,
            },
        );
        p.place_item(
            "code",
            MemoryItem {
                key: "python-gil".into(),
                value: "Python GIL limits true parallelism in threads".into(),
                associations: vec!["concurrency".into(), "parallel".into()],
                strength: 0.6,
            },
        );

        let results = p.recall_semantic("rust memory safety", 5);
        assert!(!results.is_empty());
        assert_eq!(results[0].0.key, "rust-ownership");
    }

    #[test]
    fn test_semantic_search() {
        let mut p = MemoryPalace::new();
        p.create_room("r", "d");
        p.place_item(
            "r",
            MemoryItem {
                key: "k1".into(),
                value: "machine learning model training".into(),
                associations: vec!["ai".into(), "neural".into()],
                strength: 0.9,
            },
        );
        p.place_item(
            "r",
            MemoryItem {
                key: "k2".into(),
                value: "web application frontend".into(),
                associations: vec!["html".into(), "css".into()],
                strength: 0.5,
            },
        );

        let results = p.semantic_search("ai neural network", 5);
        assert!(!results.is_empty());
        assert_eq!(results[0].item.key, "k1");
    }

    #[test]
    fn test_keyword_index_stats() {
        let mut p = MemoryPalace::new();
        p.create_room("r", "d");
        p.place_item(
            "r",
            MemoryItem {
                key: "test".into(),
                value: "hello world".into(),
                associations: vec!["foo".into()],
                strength: 0.5,
            },
        );
        let stats = p.keyword_stats();
        assert!(stats.total_keywords > 0);
    }

    #[test]
    fn test_tokenize() {
        let tokens = tokenize("Hello, World! This is a test.");
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
        assert!(!tokens.contains(&"is".to_string())); // stopword
    }

    #[test]
    fn test_tokenize_chinese() {
        let tokens = tokenize("Rust的所有权系统");
        assert!(tokens.iter().any(|t| t.contains("rust")));
    }
}
