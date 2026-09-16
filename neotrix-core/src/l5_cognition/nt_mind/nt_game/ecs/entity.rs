use std::collections::HashMap;

pub type EntityId = u64;

#[derive(Debug, Clone)]
pub struct Entity {
    pub id: EntityId,
    pub alive: bool,
    pub tags: Vec<String>,
}

pub struct EntityManager {
    entities: HashMap<EntityId, Entity>,
    next_id: EntityId,
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn create(&mut self) -> EntityId {
        let id = self.next_id;
        self.next_id += 1;
        self.entities.insert(
            id,
            Entity {
                id,
                alive: true,
                tags: Vec::new(),
            },
        );
        id
    }

    pub fn destroy(&mut self, id: EntityId) {
        if let Some(e) = self.entities.get_mut(&id) {
            e.alive = false;
        }
    }

    pub fn is_alive(&self, id: EntityId) -> bool {
        self.entities.get(&id).map_or(false, |e| e.alive)
    }

    pub fn tag(&mut self, id: EntityId, tag: &str) {
        if let Some(e) = self.entities.get_mut(&id) {
            e.tags.push(tag.to_string());
        }
    }

    pub fn has_tag(&self, id: EntityId, tag: &str) -> bool {
        self.entities
            .get(&id)
            .map_or(false, |e| e.tags.contains(&tag.to_string()))
    }

    pub fn count(&self) -> usize {
        self.entities.values().filter(|e| e.alive).count()
    }

    pub fn alive_ids(&self) -> Vec<EntityId> {
        self.entities
            .values()
            .filter(|e| e.alive)
            .map(|e| e.id)
            .collect()
    }

    pub fn with_tag(&self, tag: &str) -> Vec<EntityId> {
        self.entities
            .values()
            .filter(|e| e.alive && e.tags.contains(&tag.to_string()))
            .map(|e| e.id)
            .collect()
    }
}

impl Default for EntityManager {
    fn default() -> Self {
        Self::new()
    }
}
