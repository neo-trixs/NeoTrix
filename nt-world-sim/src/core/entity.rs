use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};

/// Archetype ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArchetypeId(pub u64);

/// Entity ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(pub u64);

/// Universal entity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UniversalEntity {
    pub id: EntityId,
    pub generation: u32,
    pub archetype: ArchetypeId,
}

impl UniversalEntity {
    pub fn new(id: EntityId, generation: u32) -> Self {
        Self {
            id,
            generation,
            archetype: ArchetypeId(0),
        }
    }
}

/// Component storage types
pub enum ComponentStorage {
    /// Dense storage for small component sets
    Dense(Vec<Box<dyn Any + Send + Sync>>),
    /// Sparse storage for large, sparse sets
    Sparse(HashMap<u64, Box<dyn Any + Send + Sync>>),
    /// SoA (Structure of Arrays) for performance-critical
    SoA(SoAStorage),
}

impl ComponentStorage {
    pub fn dense() -> Self {
        Self::Dense(Vec::new())
    }

    pub fn sparse() -> Self {
        Self::Sparse(HashMap::new())
    }

    pub fn soa(stride: usize) -> Self {
        Self::SoA(SoAStorage::new(stride))
    }
}

/// SoA storage
#[derive(Debug)]
pub struct SoAStorage {
    pub arrays: HashMap<TypeId, Vec<u8>>,
    pub stride: usize,
}

impl SoAStorage {
    pub fn new(stride: usize) -> Self {
        Self {
            arrays: HashMap::new(),
            stride,
        }
    }
}

/// Archetype: unique component combination
#[derive(Debug)]
pub struct Archetype {
    pub id: ArchetypeId,
    pub component_types: HashSet<TypeId>,
    pub entities: Vec<UniversalEntity>,
    pub chunks: Vec<Chunk>,
}

impl Archetype {
    pub fn new(id: ArchetypeId, component_types: HashSet<TypeId>) -> Self {
        Self {
            id,
            component_types,
            entities: Vec::new(),
            chunks: Vec::new(),
        }
    }

    pub fn add_entity(&mut self, entity: UniversalEntity) {
        self.entities.push(entity);
    }

    pub fn remove_entity(&mut self, entity: EntityId) {
        self.entities.retain(|e| e.id != entity);
    }

    pub fn has_component(&self, type_id: TypeId) -> bool {
        self.component_types.contains(&type_id)
    }

    pub fn matches_query(&self, required: &HashSet<TypeId>, excluded: &HashSet<TypeId>) -> bool {
        required.is_subset(&self.component_types) && excluded.is_disjoint(&self.component_types)
    }
}

/// Chunk: fixed-size memory block
#[derive(Debug)]
pub struct Chunk {
    pub capacity: usize,
    pub component_data: Vec<Vec<u8>>,
    pub entity_ids: Vec<EntityId>,
}

impl Chunk {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            component_data: Vec::new(),
            entity_ids: Vec::with_capacity(capacity),
        }
    }

    pub fn is_full(&self) -> bool {
        self.entity_ids.len() >= self.capacity
    }

    pub fn add_entity(&mut self, entity_id: EntityId) -> bool {
        if self.is_full() {
            return false;
        }
        self.entity_ids.push(entity_id);
        true
    }

    pub fn remove_entity(&mut self, entity_id: EntityId) -> bool {
        if let Some(pos) = self.entity_ids.iter().position(|&id| id == entity_id) {
            self.entity_ids.remove(pos);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archetype_query_matching() {
        let mut component_types = HashSet::new();
        component_types.insert(TypeId::of::<String>());
        component_types.insert(TypeId::of::<i32>());
        
        let archetype = Archetype::new(ArchetypeId(1), component_types);
        
        let mut required = HashSet::new();
        required.insert(TypeId::of::<String>());
        
        let excluded = HashSet::new();
        
        assert!(archetype.matches_query(&required, &excluded));
    }

    #[test]
    fn test_chunk_capacity() {
        let mut chunk = Chunk::new(2);
        assert!(!chunk.is_full());
        
        chunk.add_entity(EntityId(1));
        assert!(!chunk.is_full());
        
        chunk.add_entity(EntityId(2));
        assert!(chunk.is_full());
    }

    #[test]
    fn test_archetype_creation() {
        let mut types = HashSet::new();
        types.insert(TypeId::of::<i32>());
        let arch = Archetype::new(ArchetypeId(1), types);
        assert_eq!(arch.id, ArchetypeId(1));
    }

    #[test]
    fn test_chunk_capacity_three() {
        let mut chunk = Chunk::new(3);
        assert!(!chunk.is_full());
        chunk.add_entity(EntityId(1));
        chunk.add_entity(EntityId(2));
        chunk.add_entity(EntityId(3));
        assert!(chunk.is_full());
    }

    #[test]
    fn test_archetype_query_match() {
        let mut types = HashSet::new();
        types.insert(TypeId::of::<i32>());
        types.insert(TypeId::of::<String>());
        let arch = Archetype::new(ArchetypeId(1), types);

        let mut required = HashSet::new();
        required.insert(TypeId::of::<i32>());
        let excluded = HashSet::new();
        assert!(arch.matches_query(&required, &excluded));
    }
}
