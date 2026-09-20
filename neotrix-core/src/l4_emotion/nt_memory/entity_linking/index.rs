#![forbid(unsafe_code)]

//! In-memory entity index with name and type search.
//!
//! Provides fast lookup of linked entities by ID, name substring, or type.
//! Designed for ADD-only (R-P117): entities are inserted, never mutated.

use std::collections::HashMap;

use super::entity::{Entity, EntityType};

/// In-memory index for linked entities.
#[derive(Debug)]
pub struct EntityIndex {
    /// Primary store: id -> Entity.
    by_id: HashMap<String, Entity>,
    /// Reverse index: lowercase name substring -> set of entity IDs.
    name_index: HashMap<String, Vec<String>>,
    /// Type index: EntityType -> set of entity IDs.
    type_index: HashMap<EntityType, Vec<String>>,
}

impl EntityIndex {
    pub fn new() -> Self {
        Self {
            by_id: HashMap::new(),
            name_index: HashMap::new(),
            type_index: HashMap::new(),
        }
    }

    /// Add an entity to the index. Returns `None` if the ID already exists.
    pub fn add(&mut self, entity: Entity) -> Option<Entity> {
        if entity.id.is_empty() {
            return None;
        }
        if self.by_id.contains_key(&entity.id) {
            return Some(entity); // already indexed, return original
        }

        // Index name substrings (split on whitespace, lowercase)
        let id = entity.id.clone();
        let name_lower = entity.name.to_lowercase();
        let etype = entity.entity_type;

        // Index full name
        self.name_index
            .entry(name_lower.clone())
            .or_default()
            .push(id.clone());

        // Index individual words (skip short words)
        for word in name_lower.split_whitespace() {
            if word.len() >= 2 {
                self.name_index
                    .entry(word.to_string())
                    .or_default()
                    .push(id.clone());
            }
        }

        // Type index
        self.type_index
            .entry(etype)
            .or_default()
            .push(id.clone());

        self.by_id.insert(id, entity);
        None
    }

    /// Get an entity by its stable ID.
    pub fn get_entity(&self, id: &str) -> Option<&Entity> {
        self.by_id.get(id)
    }

    /// Search entities by name substring (case-insensitive).
    pub fn search_by_name(&self, query: &str) -> Vec<&Entity> {
        let query_lower = query.to_lowercase();

        // Exact name match first
        if let Some(ids) = self.name_index.get(&query_lower) {
            return ids.iter()
                .filter_map(|id| self.by_id.get(id))
                .collect();
        }

        // Substring match across all indexed names
        let mut matched_ids: Vec<&str> = Vec::new();
        for (name, ids) in &self.name_index {
            if name.contains(&query_lower) {
                for id in ids {
                    if !matched_ids.contains(&id.as_str()) {
                        matched_ids.push(id.as_str());
                    }
                }
            }
        }

        matched_ids.into_iter()
            .filter_map(|id| self.by_id.get(id))
            .collect()
    }

    /// Search entities by type.
    pub fn search_by_type(&self, etype: EntityType) -> Vec<&Entity> {
        self.type_index
            .get(&etype)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.by_id.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Return the total number of indexed entities.
    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    /// Check if the index is empty.
    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    /// Iterate over all entities in the index.
    pub fn iter(&self) -> impl Iterator<Item = &Entity> {
        self.by_id.values()
    }

    /// Remove an entity by ID. Returns `true` if it existed.
    pub fn remove(&mut self, id: &str) -> bool {
        if let Some(entity) = self.by_id.remove(id) {
            // Clean up name index
            let name_lower = entity.name.to_lowercase();
            self.name_index.retain(|_, ids| {
                ids.retain(|eid| eid != id);
                !ids.is_empty()
            });
            for word in name_lower.split_whitespace() {
                if word.len() >= 2 {
                    self.name_index.retain(|_, ids| {
                        ids.retain(|eid| eid != id);
                        !ids.is_empty()
                    });
                }
            }
            // Clean up type index
            if let Some(ids) = self.type_index.get_mut(&entity.entity_type) {
                ids.retain(|eid| eid != id);
            }
            true
        } else {
            false
        }
    }
}

impl Default for EntityIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ts() -> i64 {
        1_700_000_000
    }

    fn make_entity(id: &str, name: &str, etype: EntityType) -> Entity {
        let mut e = Entity::new(name, etype, 0, ts());
        e.id = id.to_string();
        e
    }

    #[test]
    fn add_and_get() {
        let mut idx = EntityIndex::new();
        let e = make_entity("e/1", "Alice Smith", EntityType::Person);
        assert!(idx.add(e).is_none());
        assert_eq!(idx.len(), 1);
        assert!(idx.get_entity("e/1").is_some());
    }

    #[test]
    fn add_duplicate_returns_original() {
        let mut idx = EntityIndex::new();
        let e1 = make_entity("e/1", "Alice", EntityType::Person);
        let e2 = make_entity("e/1", "Alice", EntityType::Person);
        idx.add(e1);
        let returned = idx.add(e2);
        assert!(returned.is_some());
        assert_eq!(idx.len(), 1);
    }

    #[test]
    fn search_by_name_exact() {
        let mut idx = EntityIndex::new();
        idx.add(make_entity("e/1", "Bob Johnson", EntityType::Person));
        let results = idx.search_by_name("Bob Johnson");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "e/1");
    }

    #[test]
    fn search_by_name_substring() {
        let mut idx = EntityIndex::new();
        idx.add(make_entity("e/1", "Bob Johnson", EntityType::Person));
        idx.add(make_entity("e/2", "Alice Smith", EntityType::Person));
        let results = idx.search_by_name("bob");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "e/1");
    }

    #[test]
    fn search_by_name_no_match() {
        let mut idx = EntityIndex::new();
        idx.add(make_entity("e/1", "Alice", EntityType::Person));
        let results = idx.search_by_name("Charlie");
        assert!(results.is_empty());
    }

    #[test]
    fn search_by_type() {
        let mut idx = EntityIndex::new();
        idx.add(make_entity("e/1", "Alice", EntityType::Person));
        idx.add(make_entity("e/2", "Google", EntityType::Org));
        idx.add(make_entity("e/3", "Bob", EntityType::Person));

        let persons = idx.search_by_type(EntityType::Person);
        assert_eq!(persons.len(), 2);
        let orgs = idx.search_by_type(EntityType::Org);
        assert_eq!(orgs.len(), 1);
    }

    #[test]
    fn remove_entity() {
        let mut idx = EntityIndex::new();
        idx.add(make_entity("e/1", "Alice", EntityType::Person));
        assert!(idx.remove("e/1"));
        assert_eq!(idx.len(), 0);
        assert!(idx.get_entity("e/1").is_none());
    }

    #[test]
    fn iter_all() {
        let mut idx = EntityIndex::new();
        idx.add(make_entity("e/1", "Alice", EntityType::Person));
        idx.add(make_entity("e/2", "Google", EntityType::Org));
        assert_eq!(idx.iter().count(), 2);
    }

    #[test]
    fn empty_index() {
        let idx = EntityIndex::new();
        assert!(idx.is_empty());
        assert_eq!(idx.len(), 0);
        assert!(idx.get_entity("nope").is_none());
    }

    #[test]
    fn add_empty_id_returns_none() {
        let mut idx = EntityIndex::new();
        let e = Entity::new("X", EntityType::Concept, 0, ts());
        assert!(idx.add(e).is_none());
        assert!(idx.is_empty());
    }

    #[test]
    fn search_by_name_case_insensitive() {
        let mut idx = EntityIndex::new();
        idx.add(make_entity("e/1", "Acme Corp", EntityType::Org));
        assert_eq!(idx.search_by_name("acme corp").len(), 1);
        assert_eq!(idx.search_by_name("ACME").len(), 1);
    }

    #[test]
    fn remove_nonexistent_returns_false() {
        let mut idx = EntityIndex::new();
        assert!(!idx.remove("nope"));
    }

    #[test]
    fn search_by_type_empty_when_no_entities() {
        let idx = EntityIndex::new();
        assert!(idx.search_by_type(EntityType::Person).is_empty());
    }
}
