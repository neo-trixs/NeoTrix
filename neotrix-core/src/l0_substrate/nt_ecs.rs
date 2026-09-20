//! ECS Foundation — absorbed from nt-world-sim
//!
//! Archetype-based Entity Component System with SoA storage,
//! parallel scheduler with topological sorting, change detection.
//!
//! Source: archive/nt-world-sim/src/core/

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

/// SoA storage — Vec-based for cache-friendly iteration (typical: 1-5 TypeIds per archetype)
#[derive(Debug)]
pub struct SoAStorage {
    pub arrays: Vec<(TypeId, Vec<u8>)>,
    pub stride: usize,
}

impl SoAStorage {
    pub fn new(stride: usize) -> Self {
        Self {
            arrays: Vec::new(),
            stride,
        }
    }

    /// Lookup by TypeId — linear scan is faster than HashMap for small N (≤6 components)
    #[inline]
    pub fn get_array(&self, type_id: TypeId) -> Option<&Vec<u8>> {
        self.arrays.iter()
            .find(|(tid, _)| *tid == type_id)
            .map(|(_, data)| data)
    }

    #[inline]
    pub fn get_array_mut(&mut self, type_id: TypeId) -> Option<&mut Vec<u8>> {
        self.arrays.iter_mut()
            .find(|(tid, _)| *tid == type_id)
            .map(|(_, data)| data)
    }

    #[inline]
    pub fn insert_array(&mut self, type_id: TypeId, data: Vec<u8>) {
        if let Some(existing) = self.arrays.iter_mut().find(|(tid, _)| *tid == type_id) {
            existing.1 = data;
        } else {
            self.arrays.push((type_id, data));
        }
    }

    #[inline]
    pub fn has_array(&self, type_id: TypeId) -> bool {
        self.arrays.iter().any(|(tid, _)| *tid == type_id)
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

// Entity types defined above

/// Component trait
pub trait Component: Send + Sync + Clone + 'static {
    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

/// Resource trait (global state)
pub trait Resource: Send + Sync + 'static {
    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

/// Event trait
pub trait Event: Send + Sync + 'static {
    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

/// Universal world
pub struct UniversalWorld {
    entities: Vec<Option<UniversalEntity>>,
    archetypes: HashMap<ArchetypeId, Archetype>,
    components: HashMap<(EntityId, TypeId), Box<dyn Any + Send + Sync>>,
    resources: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    events: Vec<Box<dyn Any + Send + Sync>>,
    next_entity_id: u64,
    next_archetype_id: u64,
}

impl UniversalWorld {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            archetypes: HashMap::new(),
            components: HashMap::new(),
            resources: HashMap::new(),
            events: Vec::new(),
            next_entity_id: 0,
            next_archetype_id: 0,
        }
    }

    /// Spawn a new entity
    pub fn spawn(&mut self) -> UniversalEntity {
        let id = EntityId(self.next_entity_id);
        self.next_entity_id += 1;
        
        let entity = UniversalEntity::new(id, 0);
        self.entities.push(Some(entity));
        
        entity
    }

    /// Despawn an entity
    pub fn despawn(&mut self, entity: UniversalEntity) -> bool {
        if let Some(slot) = self.entities.get_mut(entity.id.0 as usize) {
            if slot.is_some() {
                // Remove from archetype
                if let Some(archetype) = self.archetypes.get_mut(&entity.archetype) {
                    archetype.remove_entity(entity.id);
                }
                
                *slot = None;
                
                // Remove components
                self.remove_all_components(entity.id);
                
                return true;
            }
        }
        false
    }

    /// Insert a component into an entity
    pub fn insert_component<T: Component>(&mut self, entity: UniversalEntity, component: T) {
        let type_id = component.type_id();
        
        // Store component
        self.components.insert(
            (entity.id, type_id),
            Box::new(component),
        );
        
        // Update archetype
        self.update_archetype(entity, type_id);
    }

    /// Get a component reference
    pub fn get_component<T: Component>(&self, entity: UniversalEntity) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.components
            .get(&(entity.id, type_id))
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }

    /// Get a mutable component reference
    pub fn get_component_mut<T: Component>(&mut self, entity: UniversalEntity) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.components
            .get_mut(&(entity.id, type_id))
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }

    /// Remove a component from an entity
    pub fn remove_component<T: Component>(&mut self, entity: UniversalEntity) -> Option<T> {
        let type_id = TypeId::of::<T>();
        
        if let Some(component) = self.components.remove(&(entity.id, type_id)) {
            // Update archetype
            self.update_archetype_remove(entity, type_id);
            
            component.downcast::<T>().ok().map(|b| *b)
        } else {
            None
        }
    }

    /// Check if entity has a component
    pub fn has_component<T: Component>(&self, entity: UniversalEntity) -> bool {
        let type_id = TypeId::of::<T>();
        self.components.contains_key(&(entity.id, type_id))
    }

    /// Query entities with specific components
    pub fn query<T: ComponentTuple>(&self) -> Vec<(UniversalEntity, T::Item)>
    where
        T: 'static,
    {
        let mut results = Vec::new();
        
        for entity_option in &self.entities {
            if let Some(entity) = entity_option {
                if let Some(item) = T::get_components(self, *entity) {
                    results.push((*entity, item));
                }
            }
        }
        
        results
    }

    /// Get all entities
    pub fn entities(&self) -> Vec<UniversalEntity> {
        self.entities
            .iter()
            .filter_map(|e| *e)
            .collect()
    }

    /// Get entity count
    pub fn entity_count(&self) -> usize {
        self.entities.iter().filter(|e| e.is_some()).count()
    }

    /// Insert a resource
    pub fn insert_resource<T: Resource>(&mut self, resource: T) {
        self.resources.insert(TypeId::of::<T>(), Box::new(resource));
    }

    /// Get a resource reference
    pub fn get_resource<T: Resource>(&self) -> Option<&T> {
        self.resources
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }

    /// Get a mutable resource reference
    pub fn get_resource_mut<T: Resource>(&mut self) -> Option<&mut T> {
        self.resources
            .get_mut(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }

    /// Send an event
    pub fn send_event<T: Event>(&mut self, event: T) {
        self.events.push(Box::new(event));
    }

    /// Receive events
    pub fn receive_events<T: Event>(&mut self) -> Vec<T> {
        let mut events = Vec::new();
        let mut remaining = Vec::new();
        for event in self.events.drain(..) {
            if event.is::<T>() {
                events.push(*event.downcast::<T>().unwrap());
            } else {
                remaining.push(event);
            }
        }
        self.events = remaining;
        events
    }

    /// Helper: update archetype when component is added
    fn update_archetype(&mut self, entity: UniversalEntity, type_id: TypeId) {
        // Get current archetype
        let current_archetype = entity.archetype;
        
        // Get or create archetype with new component
        let mut component_types = if let Some(archetype) = self.archetypes.get(&current_archetype) {
            archetype.component_types.clone()
        } else {
            HashSet::new()
        };
        
        component_types.insert(type_id);
        
        // Find or create matching archetype
        let archetype_id = self.find_or_create_archetype(component_types);
        
        // Update entity archetype
        if let Some(slot) = self.entities.get_mut(entity.id.0 as usize) {
            if let Some(e) = slot {
                e.archetype = archetype_id;
            }
        }
        
        // Add entity to new archetype
        if let Some(archetype) = self.archetypes.get_mut(&archetype_id) {
            archetype.add_entity(entity);
        }
    }

    /// Helper: update archetype when component is removed
    fn update_archetype_remove(&mut self, entity: UniversalEntity, type_id: TypeId) {
        let current_archetype = entity.archetype;
        
        if let Some(archetype) = self.archetypes.get(&current_archetype) {
            let mut component_types = archetype.component_types.clone();
            component_types.remove(&type_id);
            
            let archetype_id = self.find_or_create_archetype(component_types);
            
            // Update entity archetype
            if let Some(slot) = self.entities.get_mut(entity.id.0 as usize) {
                if let Some(e) = slot {
                    e.archetype = archetype_id;
                }
            }
            
            // Add entity to new archetype
            if let Some(new_archetype) = self.archetypes.get_mut(&archetype_id) {
                new_archetype.add_entity(entity);
            }
        }
    }

    /// Helper: find or create archetype
    fn find_or_create_archetype(&mut self, component_types: HashSet<TypeId>) -> ArchetypeId {
        // Find existing archetype
        for (id, archetype) in &self.archetypes {
            if archetype.component_types == component_types {
                return *id;
            }
        }
        
        // Create new archetype
        let id = ArchetypeId(self.next_archetype_id);
        self.next_archetype_id += 1;
        
        let archetype = Archetype::new(id, component_types);
        self.archetypes.insert(id, archetype);
        
        id
    }

    /// Helper: remove all components for an entity
    fn remove_all_components(&mut self, entity_id: EntityId) {
        self.components.retain(|(id, _), _| *id != entity_id);
    }
}

/// Component tuple trait for queries
pub trait ComponentTuple {
    type Item;
    fn get_components(world: &UniversalWorld, entity: UniversalEntity) -> Option<Self::Item>;
}

/// Implement for single component
impl<T: Component + Clone + 'static> ComponentTuple for (T,) {
    type Item = T;
    
    fn get_components(world: &UniversalWorld, entity: UniversalEntity) -> Option<Self::Item> {
        world.get_component::<T>(entity).cloned()
    }
}

/// Implement for tuple of two components
impl<T1: Component + Clone + 'static, T2: Component + Clone + 'static> ComponentTuple for (T1, T2) {
    type Item = (T1, T2);
    
    fn get_components(world: &UniversalWorld, entity: UniversalEntity) -> Option<Self::Item> {
        let t1 = world.get_component::<T1>(entity)?.clone();
        let t2 = world.get_component::<T2>(entity)?.clone();
        Some((t1, t2))
    }
}

impl Default for UniversalWorld {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests_2 {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }
    impl Component for Position {}

    #[derive(Clone, Debug, PartialEq)]
    struct Velocity {
        x: f32,
        y: f32,
    }
    impl Component for Velocity {}

    struct Time {
        delta: f32,
    }
    impl Resource for Time {}

    #[test]
    fn test_entity_spawn() {
        let mut world = UniversalWorld::new();
        let entity = world.spawn();
        assert_eq!(world.entity_count(), 1);
        
        world.despawn(entity);
        assert_eq!(world.entity_count(), 0);
    }

    #[test]
    fn test_component_insert_get() {
        let mut world = UniversalWorld::new();
        let entity = world.spawn();
        
        world.insert_component(entity, Position { x: 1.0, y: 2.0 });
        
        let pos = world.get_component::<Position>(entity).unwrap();
        assert_eq!(pos.x, 1.0);
        assert_eq!(pos.y, 2.0);
    }

    #[test]
    fn test_query() {
        let mut world = UniversalWorld::new();
        
        let e1 = world.spawn();
        world.insert_component(e1, Position { x: 0.0, y: 0.0 });
        world.insert_component(e1, Velocity { x: 1.0, y: 1.0 });
        
        let e2 = world.spawn();
        world.insert_component(e2, Position { x: 5.0, y: 5.0 });
        
        let results = world.query::<(Position, Velocity)>();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_resource() {
        let mut world = UniversalWorld::new();
        world.insert_resource(Time { delta: 0.016 });
        
        let time = world.get_resource::<Time>().unwrap();
        assert_eq!(time.delta, 0.016);
    }

    #[test]
    fn test_spawn_despawn() {
        let mut world = UniversalWorld::new();
        let e = world.spawn();
        assert_eq!(world.entity_count(), 1);
        world.despawn(e);
        assert_eq!(world.entity_count(), 0);
    }

    #[test]
    fn test_insert_get_component() {
        let mut world = UniversalWorld::new();
        let e = world.spawn();
        world.insert_component(e, Position { x: 1.0, y: 2.0 });
        let pos = world.get_component::<Position>(e).unwrap();
        assert_eq!(pos.x, 1.0);
    }

    #[test]
    fn test_query_two_components() {
        let mut world = UniversalWorld::new();
        let e1 = world.spawn();
        world.insert_component(e1, Position { x: 0.0, y: 0.0 });
        world.insert_component(e1, Velocity { x: 1.0, y: 1.0 });
        let e2 = world.spawn();
        world.insert_component(e2, Position { x: 5.0, y: 5.0 });

        let results = world.query::<(Position, Velocity)>();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_resource_insert_get() {
        let mut world = UniversalWorld::new();
        world.insert_resource(Time { delta: 0.016 });
        let t = world.get_resource::<Time>().unwrap();
        assert_eq!(t.delta, 0.016);
    }
}

/// System trait for the universal scheduler
pub trait UniversalSystem: Send + Sync {
    fn name(&self) -> &str;
    fn priority(&self) -> i32 { 0 }
    fn update(&mut self, world: &mut UniversalWorld, dt: f32);
    fn read_components(&self) -> Vec<TypeId> { vec![] }
    fn write_components(&self) -> Vec<TypeId> { vec![] }
    fn enabled(&self) -> bool { true }
}

/// System dependency declaration
pub struct SystemDependency {
    pub reads: Vec<TypeId>,
    pub writes: Vec<TypeId>,
    pub before: Vec<String>,
    pub after: Vec<String>,
}

impl SystemDependency {
    pub fn new() -> Self {
        Self { reads: vec![], writes: vec![], before: vec![], after: vec![] }
    }
    pub fn reads(mut self, types: Vec<TypeId>) -> Self { self.reads = types; self }
    pub fn writes(mut self, types: Vec<TypeId>) -> Self { self.writes = types; self }
    pub fn after(mut self, names: Vec<String>) -> Self { self.after = names; self }
}

/// Parallel scheduler
pub struct ParallelScheduler {
    systems: Vec<Box<dyn UniversalSystem>>,
    dependencies: Vec<SystemDependency>,
    execution_waves: Vec<Vec<usize>>,
    sorted: bool,
}

impl ParallelScheduler {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
            dependencies: Vec::new(),
            execution_waves: Vec::new(),
            sorted: false,
        }
    }

    pub fn add_system(&mut self, system: Box<dyn UniversalSystem>, deps: SystemDependency) {
        self.systems.push(system);
        self.dependencies.push(deps);
        self.sorted = false;
    }

    /// Build execution schedule using topological sort + wave scheduling
    pub fn build_schedule(&mut self) {
        let n = self.systems.len();
        if n == 0 { return; }

        // Build dependency graph
        let mut in_degree = vec![0usize; n];
        let mut adjacency: Vec<Vec<usize>> = vec![vec![]; n];
        let name_to_index: HashMap<String, usize> = self.systems.iter()
            .enumerate()
            .map(|(i, s)| (s.name().to_string(), i))
            .collect();

        for (i, deps) in self.dependencies.iter().enumerate() {
            for after_name in &deps.after {
                if let Some(&j) = name_to_index.get(after_name) {
                    adjacency[j].push(i);
                    in_degree[i] += 1;
                }
            }
        }

        // Wave scheduling: group non-conflicting systems
        let mut waves: Vec<Vec<usize>> = Vec::new();
        let mut assigned = vec![false; n];
        let mut remaining = n;

        while remaining > 0 {
            let mut wave = Vec::new();
            for i in 0..n {
                if assigned[i] { continue; }
                if in_degree[i] == 0 {
                    wave.push(i);
                }
            }

            if wave.is_empty() {
                // All remaining have dependencies; just pick highest priority
                for i in 0..n {
                    if !assigned[i] {
                        wave.push(i);
                        break;
                    }
                }
            }

            for &idx in &wave {
                assigned[idx] = true;
                remaining -= 1;
                for &next in &adjacency[idx] {
                    in_degree[next] -= 1;
                }
            }

            waves.push(wave);
        }

        self.execution_waves = waves;
        self.sorted = true;
    }

    /// Run all systems in waves (parallel where possible)
    pub fn run(&mut self, world: &mut UniversalWorld, dt: f32) {
        if !self.sorted {
            self.build_schedule();
        }

        let waves = self.execution_waves.clone();
        for wave in &waves {
            // In a real implementation, systems in the same wave
            // would run in parallel using rayon or similar.
            // For now, run sequentially within each wave.
            for &system_idx in wave {
                if self.systems[system_idx].enabled() {
                    self.systems[system_idx].update(world, dt);
                }
            }
        }
    }

    pub fn system_count(&self) -> usize {
        self.systems.len()
    }

    pub fn wave_count(&self) -> usize {
        self.execution_waves.len()
    }

    pub fn clear(&mut self) {
        self.systems.clear();
        self.dependencies.clear();
        self.execution_waves.clear();
        self.sorted = false;
    }
}

impl Default for ParallelScheduler {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests_3 {
    use super::*;

    struct TestSystem { name: String }
    impl UniversalSystem for TestSystem {
        fn name(&self) -> &str { &self.name }
        fn update(&mut self, _world: &mut UniversalWorld, _dt: f32) {}
    }

    #[test]
    fn test_scheduler_wave_count() {
        let mut sched = ParallelScheduler::new();
        sched.add_system(Box::new(TestSystem { name: "a".into() }), SystemDependency::new());
        sched.add_system(Box::new(TestSystem { name: "b".into() }), SystemDependency::new().after(vec!["a".into()]));
        sched.build_schedule();
        assert!(sched.wave_count() >= 2);
    }
}

impl<T: Component> Component for Changed<T> {}

#[derive(Clone)]
pub struct Changed<T: Component> {
    pub value: T,
    pub tick: u64,
    pub changed: bool,
}

impl<T: Component> Changed<T> {
    pub fn new(value: T, tick: u64) -> Self {
        Self {
            value,
            tick,
            changed: false,
        }
    }

    pub fn mark_changed(&mut self) {
        self.changed = true;
    }

    pub fn was_changed(&self) -> bool {
        self.changed
    }

    pub fn into_inner(self) -> T {
        self.value
    }

    pub fn inner(&self) -> &T {
        &self.value
    }

    pub fn inner_mut(&mut self) -> &mut T {
        self.changed = true;
        &mut self.value
    }
}

impl<T: Component> std::ops::Deref for Changed<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T: Component> std::ops::DerefMut for Changed<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.changed = true;
        &mut self.value
    }
}

pub struct ChangeTick {
    pub current: u64,
    pub last_read: u64,
}

impl ChangeTick {
    pub fn new() -> Self {
        Self {
            current: 0,
            last_read: 0,
        }
    }

    pub fn increment(&mut self) {
        self.current += 1;
    }

    pub fn is_changed(&self, tick: u64) -> bool {
        tick > self.last_read
    }

    pub fn mark_read(&mut self) {
        self.last_read = self.current;
    }

    pub fn ticks_since(&self, tick: u64) -> u64 {
        self.current.saturating_sub(tick)
    }
}

impl Default for ChangeTick {
    fn default() -> Self {
        Self::new()
    }
}

struct ChangeRecord {
    tick: u64,
    changed: bool,
}

pub struct ChangeDetector {
    records: HashMap<(EntityId, TypeId), ChangeRecord>,
    change_ticks: HashMap<TypeId, ChangeTick>,
}

impl ChangeDetector {
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
            change_ticks: HashMap::new(),
        }
    }

    pub fn register_component<T: Component>(&mut self) {
        let type_id = TypeId::of::<T>();
        self.change_ticks.entry(type_id).or_insert_with(ChangeTick::new);
    }

    pub fn mark_changed<T: Component>(&mut self, entity_id: EntityId, tick: u64) {
        let type_id = TypeId::of::<T>();
        self.records.insert(
            (entity_id, type_id),
            ChangeRecord { tick, changed: true },
        );
        if let Some(ct) = self.change_ticks.get_mut(&type_id) {
            if tick > ct.current {
                ct.current = tick;
            }
        }
    }

    pub fn was_changed<T: Component>(&self, entity_id: EntityId, since_tick: u64) -> bool {
        let type_id = TypeId::of::<T>();
        self.records
            .get(&(entity_id, type_id))
            .map_or(false, |r| r.changed && r.tick > since_tick)
    }

    pub fn was_any_changed<T: Component>(&self, since_tick: u64) -> Vec<EntityId> {
        let type_id = TypeId::of::<T>();
        self.records
            .iter()
            .filter(|((_, tid), r)| *tid == type_id && r.changed && r.tick > since_tick)
            .map(|((eid, _), _)| *eid)
            .collect()
    }

    pub fn clear_changed<T: Component>(&mut self, entity_id: EntityId) {
        let type_id = TypeId::of::<T>();
        if let Some(record) = self.records.get_mut(&(entity_id, type_id)) {
            record.changed = false;
        }
    }

    pub fn get_change_tick<T: Component>(&self) -> Option<&ChangeTick> {
        self.change_ticks.get(&TypeId::of::<T>())
    }

    pub fn get_change_tick_mut<T: Component>(&mut self) -> Option<&mut ChangeTick> {
        self.change_ticks.get_mut(&TypeId::of::<T>())
    }

    pub fn clear_all(&mut self) {
        self.records.clear();
        for ct in self.change_ticks.values_mut() {
            ct.mark_read();
        }
    }
}

impl Default for ChangeDetector {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ChangeTracker {
    tick: u64,
    detector: ChangeDetector,
    world_ticks: HashMap<EntityId, HashMap<TypeId, u64>>,
}

impl ChangeTracker {
    pub fn new() -> Self {
        Self {
            tick: 0,
            detector: ChangeDetector::new(),
            world_ticks: HashMap::new(),
        }
    }

    pub fn advance_tick(&mut self) -> u64 {
        self.tick += 1;
        self.tick
    }

    pub fn current_tick(&self) -> u64 {
        self.tick
    }

    pub fn detect_component_change<T: Component>(
        &mut self,
        entity_id: EntityId,
        value: &T,
    ) -> Changed<T> {
        let entity_ticks = self.world_ticks.entry(entity_id).or_insert_with(HashMap::new);
        let type_id = TypeId::of::<T>();
        let last_tick = entity_ticks.get(&type_id).copied().unwrap_or(0);
        let is_changed = self.tick > last_tick;

        if is_changed {
            entity_ticks.insert(type_id, self.tick);
            self.detector.mark_changed::<T>(entity_id, self.tick);
        }

        Changed::new(value.clone(), self.tick)
    }

    pub fn was_changed<T: Component>(&self, entity_id: EntityId) -> bool {
        self.detector.was_changed::<T>(entity_id, self.tick.saturating_sub(1))
    }

    pub fn was_any_changed<T: Component>(&self) -> Vec<EntityId> {
        self.detector.was_any_changed::<T>(self.tick.saturating_sub(1))
    }

    pub fn get_detector(&self) -> &ChangeDetector {
        &self.detector
    }

    pub fn get_detector_mut(&mut self) -> &mut ChangeDetector {
        &mut self.detector
    }

    pub fn entities_changed_since<T: Component>(&self, since_tick: u64) -> Vec<EntityId> {
        self.detector.was_any_changed::<T>(since_tick)
    }

    pub fn clear(&mut self) {
        self.detector.clear_all();
    }

    pub fn insert_changed<T: Component>(&mut self, world: &mut UniversalWorld, entity: UniversalEntity, value: T) {
        let changed = Changed::new(value.clone(), self.tick);
        self.detector.mark_changed::<T>(entity.id, self.tick);
        world.insert_component(entity, changed);
    }
}

impl Default for ChangeTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests_4 {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }

    impl Component for Position {}

    #[derive(Clone, Debug, PartialEq)]
    struct Health(i32);

    impl Component for Health {}

    #[test]
    fn test_changed_new() {
        let pos = Position { x: 1.0, y: 2.0 };
        let changed = Changed::new(pos.clone(), 5);
        assert_eq!(changed.value, pos);
        assert_eq!(changed.tick, 5);
        assert!(!changed.changed);
    }

    #[test]
    fn test_changed_mark_changed() {
        let pos = Position { x: 0.0, y: 0.0 };
        let mut changed = Changed::new(pos, 1);
        assert!(!changed.was_changed());
        changed.mark_changed();
        assert!(changed.was_changed());
    }

    #[test]
    fn test_changed_deref() {
        let pos = Position { x: 3.0, y: 4.0 };
        let mut changed = Changed::new(pos, 0);
        assert_eq!(changed.x, 3.0);
        assert_eq!(changed.y, 4.0);
        assert!(!changed.changed);
        changed.x = 10.0;
        assert!(changed.changed);
    }

    #[test]
    fn test_changed_into_inner() {
        let pos = Position { x: 1.0, y: 2.0 };
        let changed = Changed::new(pos.clone(), 0);
        let inner = changed.into_inner();
        assert_eq!(inner, pos);
    }

    #[test]
    fn test_change_tick() {
        let mut ct = ChangeTick::new();
        assert_eq!(ct.current, 0);
        assert_eq!(ct.last_read, 0);

        ct.increment();
        assert_eq!(ct.current, 1);
        assert!(ct.is_changed(1));
        assert!(!ct.is_changed(0));

        ct.mark_read();
        assert_eq!(ct.last_read, 1);
        assert!(!ct.is_changed(1));
    }

    #[test]
    fn test_change_tick_ticks_since() {
        let ct = ChangeTick { current: 10, last_read: 3 };
        assert_eq!(ct.ticks_since(5), 5);
        assert_eq!(ct.ticks_since(10), 0);
        assert_eq!(ct.ticks_since(15), 0);
    }

    #[test]
    fn test_change_detector_mark_and_query() {
        let mut det = ChangeDetector::new();
        let eid = EntityId(1);

        det.mark_changed::<Position>(eid, 5);
        assert!(det.was_changed::<Position>(eid, 3));
        assert!(!det.was_changed::<Position>(eid, 5));

        det.clear_changed::<Position>(eid);
        assert!(!det.was_changed::<Position>(eid, 0));
    }

    #[test]
    fn test_change_detector_any_changed() {
        let mut det = ChangeDetector::new();
        det.mark_changed::<Position>(EntityId(0), 2);
        det.mark_changed::<Position>(EntityId(1), 4);
        det.mark_changed::<Health>(EntityId(2), 3);

        let changed = det.was_any_changed::<Position>(2);
        assert_eq!(changed.len(), 1);
        assert!(changed.contains(&EntityId(1)));
    }

    #[test]
    fn test_change_tracker_detect() {
        let mut tracker = ChangeTracker::new();
        let pos = Position { x: 0.0, y: 0.0 };

        tracker.advance_tick();
        let changed = tracker.detect_component_change(EntityId(0), &pos);
        assert!(!changed.was_changed());
        assert_eq!(changed.tick, 1);
        assert!(tracker.was_changed::<Position>(EntityId(0)));

        let changed2 = tracker.detect_component_change(EntityId(0), &pos);
        assert!(!changed2.was_changed());
    }

    #[test]
    fn test_change_tracker_entities_changed_since() {
        let mut tracker = ChangeTracker::new();
        let pos = Position { x: 0.0, y: 0.0 };

        tracker.advance_tick();
        tracker.detect_component_change(EntityId(0), &pos);

        tracker.advance_tick();
        tracker.detect_component_change(EntityId(1), &pos);

        let changed = tracker.entities_changed_since::<Position>(1);
        assert_eq!(changed.len(), 1);
        assert!(changed.contains(&EntityId(1)));
    }

    #[test]
    fn test_change_tracker_insert_changed() {
        let mut tracker = ChangeTracker::new();
        let mut world = UniversalWorld::new();
        let entity = world.spawn();

        tracker.advance_tick();
        tracker.insert_changed(&mut world, entity, Position { x: 1.0, y: 2.0 });

        let changed = world.get_component::<Changed<Position>>(entity).unwrap();
        assert_eq!(changed.value.x, 1.0);
        assert!(tracker.was_changed::<Position>(entity.id));
    }

    #[test]
    fn test_change_tick_increment() {
        let mut tick = ChangeTick { current: 0, last_read: 0 };
        tick.increment();
        assert_eq!(tick.current, 1);
        assert!(tick.is_changed(1));
        assert!(!tick.is_changed(0));
    }
}

use std::ops::{Add, Sub, Neg, Mul};

/// 2D vector — single fact source for all Vec2 usage across engine/game/ui.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self { Self { x, y } }
    pub fn zero() -> Self { Self { x: 0.0, y: 0.0 } }
    pub fn one() -> Self { Self { x: 1.0, y: 1.0 } }
    pub fn length(&self) -> f32 { (self.x * self.x + self.y * self.y).sqrt() }
    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len < 1e-8 { Self::zero() } else { Self::new(self.x / len, self.y / len) }
    }
    pub fn dot(&self, other: &Self) -> f32 { self.x * other.x + self.y * other.y }
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        Self { x: self.x + (other.x - self.x) * t, y: self.y + (other.y - self.y) * t }
    }
    pub fn distance_to(&self, other: &Self) -> f32 { (*self - *other).length() }
    pub fn min(&self, other: &Self) -> Self { Self::new(self.x.min(other.x), self.y.min(other.y)) }
    pub fn max(&self, other: &Self) -> Self { Self::new(self.x.max(other.x), self.y.max(other.y)) }
    pub fn clamp(&self, min: &Self, max: &Self) -> Self {
        Self::new(self.x.clamp(min.x, max.x), self.y.clamp(min.y, max.y))
    }
}

impl Add for Vec2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self { Self::new(self.x + rhs.x, self.y + rhs.y) }
}
impl Sub for Vec2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self { Self::new(self.x - rhs.x, self.y - rhs.y) }
}
impl Neg for Vec2 {
    type Output = Self;
    fn neg(self) -> Self { Self::new(-self.x, -self.y) }
}
impl Mul<f32> for Vec2 {
    type Output = Self;
    fn mul(self, s: f32) -> Self { Self::new(self.x * s, self.y * s) }
}

/// Axis-aligned rectangle — single fact source for all Rect usage.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self { Self { x, y, width, height } }
    pub fn contains(&self, p: &Vec2) -> bool {
        p.x >= self.x && p.x <= self.x + self.width && p.y >= self.y && p.y <= self.y + self.height
    }
    pub fn intersects(&self, o: &Rect) -> bool {
        self.x < o.x + o.width && self.x + self.width > o.x && self.y < o.y + o.height && self.y + self.height > o.y
    }
    pub fn left(&self) -> f32 { self.x }
    pub fn right(&self) -> f32 { self.x + self.width }
    pub fn top(&self) -> f32 { self.y }
    pub fn bottom(&self) -> f32 { self.y + self.height }
    pub fn center(&self) -> Vec2 { Vec2::new(self.x + self.width * 0.5, self.y + self.height * 0.5) }
    pub fn size(&self) -> Vec2 { Vec2::new(self.width, self.height) }
}

/// RGBA color — single fact source for all Color usage.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self { Self { r, g, b, a } }
    pub fn rgb(r: f32, g: f32, b: f32) -> Self { Self { r, g, b, a: 1.0 } }
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self { Self { r, g, b, a } }
    pub fn white() -> Self { Self::rgb(1.0, 1.0, 1.0) }
    pub fn black() -> Self { Self::rgb(0.0, 0.0, 0.0) }
    pub fn red() -> Self { Self::rgb(1.0, 0.0, 0.0) }
    pub fn green() -> Self { Self::rgb(0.0, 1.0, 0.0) }
    pub fn blue() -> Self { Self::rgb(0.0, 0.0, 1.0) }
    pub fn yellow() -> Self { Self::rgb(1.0, 1.0, 0.0) }
    pub fn clear() -> Self { Self::rgba(0.0, 0.0, 0.0, 0.0) }
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
            a: self.a + (other.a - self.a) * t,
        }
    }
    pub fn with_alpha(&self, a: f32) -> Self { Self { a, ..*self } }
}

/// 2D transform — single fact source for all Transform usage.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
}

impl Transform {
    pub fn new(position: Vec2, rotation: f32, scale: Vec2) -> Self { Self { position, rotation, scale } }
}

impl Default for Transform {
    fn default() -> Self {
        Self { position: Vec2::zero(), rotation: 0.0, scale: Vec2::one() }
    }
}
