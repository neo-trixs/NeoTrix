#![forbid(unsafe_code)]

//! Entity linker: merges duplicate mentions into unified entity records.
//!
//! After extraction, the linker groups entities that refer to the same
//! real-world thing (fuzzy name matching + type agreement), then produces
//! deduplicated entities with stable IDs and consolidated mention lists.

use std::collections::HashMap;

use super::entity::{Entity, EntityType};

/// Configuration for the entity linker.
#[derive(Debug, Clone)]
pub struct LinkerConfig {
    /// Maximum edit distance (Levenshtein) for fuzzy name matching.
    pub max_edit_distance: usize,
    /// Minimum overlap ratio (0.0..1.0) for name matching.
    pub min_name_overlap: f64,
    /// Prefix for generated stable IDs.
    pub id_prefix: String,
}

impl Default for LinkerConfig {
    fn default() -> Self {
        Self {
            max_edit_distance: 2,
            min_name_overlap: 0.6,
            id_prefix: "ent".to_string(),
        }
    }
}

/// Entity linker that merges same-entity mentions.
pub struct EntityLinker {
    config: LinkerConfig,
    next_id: u64,
}

impl EntityLinker {
    pub fn new(config: LinkerConfig) -> Self {
        Self {
            config,
            next_id: 1,
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(LinkerConfig::default())
    }

    /// Link a list of extracted entities: group, merge, and assign stable IDs.
    pub fn link(&mut self, entities: Vec<Entity>) -> Vec<Entity> {
        if entities.is_empty() {
            return Vec::new();
        }

        // Group by entity type first, then merge within each group
        let mut by_type: HashMap<EntityType, Vec<Entity>> = HashMap::new();
        for e in entities {
            by_type.entry(e.entity_type).or_default().push(e);
        }

        let mut linked = Vec::new();
        for (etype, group) in by_type {
            let merged = self.merge_group(group, etype);
            linked.extend(merged);
        }

        // Assign stable IDs
        for e in &mut linked {
            if e.id.is_empty() {
                e.id = self.next_id();
            }
        }

        linked
    }

    /// Merge entities within a type group.
    fn merge_group(&self, mut entities: Vec<Entity>, _etype: EntityType) -> Vec<Entity> {
        let mut merged: Vec<Entity> = Vec::new();

        // Sort by mention count descending to use the most-mentioned as the base
        entities.sort_by(|a, b| b.mentions.len().cmp(&a.mentions.len()));

        for entity in entities {
            // Try to find an existing merged entity to append to
            let target = merged.iter_mut().find(|m| {
                self.names_match(&m.name, &entity.name)
            });

            match target {
                Some(target) => {
                    // Merge mentions
                    for mention in entity.mentions {
                        let already_present = target.mentions.iter().any(|existing| {
                            existing.surface == mention.surface && existing.offset == mention.offset
                        });
                        if !already_present {
                            target.mentions.push(mention);
                        }
                    }
                    // Update temporal window
                    if entity.first_seen < target.first_seen {
                        target.first_seen = entity.first_seen;
                    }
                    if entity.last_seen > target.last_seen {
                        target.last_seen = entity.last_seen;
                    }
                    // Prefer longer name as canonical
                    if entity.name.len() > target.name.len() {
                        target.name = entity.name;
                    }
                }
                None => {
                    merged.push(entity);
                }
            }
        }

        merged
    }

    /// Check if two names refer to the same entity.
    fn names_match(&self, a: &str, b: &str) -> bool {
        // Exact match
        if a.eq_ignore_ascii_case(b) {
            return true;
        }

        // Substring containment (e.g., "MIT" matches "Massachusetts Institute of Technology")
        let a_lower = a.to_lowercase();
        let b_lower = b.to_lowercase();
        if a_lower.contains(&b_lower) || b_lower.contains(&a_lower) {
            // Only if the shorter name is at least 3 chars to avoid false positives
            if a.len().min(b.len()) >= 3 {
                return true;
            }
        }

        // Levenshtein distance
        let dist = levenshtein(&a_lower, &b_lower);
        let max_len = a.len().max(b.len());
        if max_len == 0 {
            return true;
        }
        let similarity = 1.0 - (dist as f64 / max_len as f64);
        similarity >= self.config.min_name_overlap
    }

    /// Generate a stable, deterministic ID.
    fn next_id(&mut self) -> String {
        let id = format!("{}/{:06}", self.config.id_prefix, self.next_id);
        self.next_id += 1;
        id
    }
}

/// Compute Levenshtein edit distance between two strings.
pub fn levenshtein(a: &str, b: &str) -> usize {
    let a_len = a.len();
    let b_len = b.len();
    if a_len == 0 { return b_len; }
    if b_len == 0 { return a_len; }

    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();

    let mut prev = vec![0usize; b_len + 1];
    let mut curr = vec![0usize; b_len + 1];

    for j in 0..=b_len {
        prev[j] = j;
    }

    for i in 1..=a_len {
        curr[0] = i;
        for j in 1..=b_len {
            let cost = if a_bytes[i - 1] == b_bytes[j - 1] { 0 } else { 1 };
            curr[j] = (curr[j - 1] + 1)
                .min(prev[j] + 1)
                .min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[b_len]
}

/// Merge two entities by consolidating their mentions and temporal windows.
pub fn merge_entities(mut a: Entity, b: Entity) -> Entity {
    for mention in b.mentions {
        let already_present = a.mentions.iter().any(|existing| {
            existing.surface == mention.surface && existing.offset == mention.offset
        });
        if !already_present {
            a.mentions.push(mention);
        }
    }

    if b.first_seen < a.first_seen {
        a.first_seen = b.first_seen;
    }
    if b.last_seen > a.last_seen {
        a.last_seen = b.last_seen;
    }

    // Prefer longer name
    if b.name.len() > a.name.len() {
        a.name = b.name;
    }

    a
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ts() -> i64 {
        1_700_000_000
    }

    #[test]
    fn linker_merges_exact_duplicates() {
        let mut linker = EntityLinker::with_default_config();
        let e1 = Entity::new("Alice", EntityType::Person, 0, ts());
        let mut e2 = Entity::new("Alice", EntityType::Person, 50, ts() + 100);
        e2.add_mention("Alice Smith", 60, ts() + 100);

        let linked = linker.link(vec![e1, e2]);
        assert_eq!(linked.len(), 1);
        assert_eq!(linked[0].mentions.len(), 2);
        assert_eq!(linked[0].first_seen, ts());
        assert_eq!(linked[0].last_seen, ts() + 100);
    }

    #[test]
    fn linker_merges_substring_matches() {
        let mut linker = EntityLinker::with_default_config();
        let e1 = Entity::new("MIT", EntityType::Org, 0, ts());
        let e2 = Entity::new("Massachusetts Institute of Technology", EntityType::Org, 10, ts());

        let linked = linker.link(vec![e1, e2]);
        assert_eq!(linked.len(), 1);
    }

    #[test]
    fn linker_does_not_merge_different_types() {
        let mut linker = EntityLinker::with_default_config();
        let e1 = Entity::new("Apple", EntityType::Org, 0, ts());
        let e2 = Entity::new("Apple", EntityType::Concept, 10, ts());

        let linked = linker.link(vec![e1, e2]);
        assert_eq!(linked.len(), 2);
    }

    #[test]
    fn linker_assigns_stable_ids() {
        let mut linker = EntityLinker::with_default_config();
        let e1 = Entity::new("Alice", EntityType::Person, 0, ts());
        let e2 = Entity::new("Bob", EntityType::Person, 10, ts());

        let linked = linker.link(vec![e1, e2]);
        assert!(linked[0].id.starts_with("ent/"));
        assert!(linked[1].id.starts_with("ent/"));
        assert_ne!(linked[0].id, linked[1].id);
    }

    #[test]
    fn levenshtein_basic() {
        assert_eq!(levenshtein("", ""), 0);
        assert_eq!(levenshtein("abc", ""), 3);
        assert_eq!(levenshtein("", "abc"), 3);
        assert_eq!(levenshtein("abc", "abc"), 0);
        assert_eq!(levenshtein("abc", "abd"), 1);
        assert_eq!(levenshtein("kitten", "sitting"), 3);
    }

    #[test]
    fn merge_entities_combines_mentions() {
        let mut a = Entity::new("Google", EntityType::Org, 0, 100);
        let b = Entity::new("Google Inc", EntityType::Org, 50, 200);
        let merged = merge_entities(a.clone(), b.clone());
        assert_eq!(merged.mentions.len(), 2);
        assert_eq!(merged.first_seen, 100);
        assert_eq!(merged.last_seen, 200);
        assert_eq!(merged.name, "Google Inc");
    }

    #[test]
    fn linker_empty_input() {
        let mut linker = EntityLinker::with_default_config();
        let linked = linker.link(vec![]);
        assert!(linked.is_empty());
    }

    #[test]
    fn custom_id_prefix() {
        let config = LinkerConfig {
            id_prefix: "my".into(),
            ..Default::default()
        };
        let mut linker = EntityLinker::new(config);
        let e = Entity::new("Alice", EntityType::Person, 0, ts());
        let linked = linker.link(vec![e]);
        assert!(linked[0].id.starts_with("my/"));
    }

    #[test]
    fn merge_entities_no_new_duplicates() {
        let mut a = Entity::new("Foo", EntityType::Org, 0, 100);
        a.add_mention("Foo Inc", 50, 200);
        let mut b = Entity::new("Foo", EntityType::Org, 10, 150);
        b.add_mention("Foo Inc", 55, 250);
        let merged = merge_entities(a, b);
        let unique_surfaces: Vec<&str> = merged.mentions.iter().map(|m| m.surface.as_str()).collect();
        assert_eq!(merged.mentions.len(), 2);
        assert!(unique_surfaces.contains(&"Foo"));
        assert!(unique_surfaces.contains(&"Foo Inc"));
    }
}
