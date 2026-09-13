use std::any::{Any, TypeId};
use std::collections::HashMap;

// Re-export Vec2 from renderer for convenience
pub use super::renderer::Vec2;

// ---------------------------------------------------------------------------
// Entity — generation-stamped ID for safe despawn/respawn
// ---------------------------------------------------------------------------

/// Entity is a (generation, index) pair. The generation counter prevents
/// stale references after despawn — a classic generational arena pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity {
    pub id: u64,
    pub generation: u32,
}

impl Entity {
    /// Create an entity handle with a given index and generation.
    pub fn new(id: u64, generation: u32) -> Self {
        Self { id, generation }
    }

    /// Legacy compatibility: raw u64 id.
    pub fn id(self) -> u64 {
        self.id
    }
}

// ---------------------------------------------------------------------------
// Component trait
// ---------------------------------------------------------------------------

/// Marker trait for components. All components must be Send + Sync + 'static.
pub trait Component: Any + Send + Sync + 'static {}

/// Type-erased component storage for a single component type.
struct ComponentVec {
    data: HashMap<u64, Box<dyn Any + Send + Sync>>,
}

impl ComponentVec {
    fn new() -> Self {
        Self { data: HashMap::new() }
    }

    fn insert<T: Component>(&mut self, entity: u64, component: T) {
        self.data.insert(entity, Box::new(component));
    }

    fn get<T: Component>(&self, entity: u64) -> Option<&T> {
        self.data.get(&entity)?.downcast_ref::<T>()
    }

    fn get_mut<T: Component>(&mut self, entity: u64) -> Option<&mut T> {
        self.data.get_mut(&entity)?.downcast_mut::<T>()
    }

    fn remove(&mut self, entity: u64) -> bool {
        self.data.remove(&entity).is_some()
    }

    fn has(&self, entity: u64) -> bool {
        self.data.contains_key(&entity)
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    fn entities(&self) -> Vec<u64> {
        self.data.keys().copied().collect()
    }
}

// ---------------------------------------------------------------------------
// Resource storage (global singletons)
// ---------------------------------------------------------------------------

/// Type-erased resource storage for global singletons (e.g. Camera, TimeState).
struct ResourceVec {
    data: Box<dyn Any + Send + Sync>,
}

impl ResourceVec {
    fn new<T: Send + Sync + 'static>(val: T) -> Self {
        Self { data: Box::new(val) }
    }

    fn get<T: 'static>(&self) -> Option<&T> {
        self.data.downcast_ref::<T>()
    }

    fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.data.downcast_mut::<T>()
    }
}

// ---------------------------------------------------------------------------
// Collision event (for inter-system communication)
// ---------------------------------------------------------------------------

/// A collision pair reported by the collision system.
#[derive(Debug, Clone)]
pub struct CollisionEvent {
    pub entity_a: Entity,
    pub entity_b: Entity,
    pub overlap_x: f32,
    pub overlap_y: f32,
}

// ---------------------------------------------------------------------------
// World
// ---------------------------------------------------------------------------

/// The ECS World: manages entities, components, and global resources.
pub struct World {
    next_entity: u64,
    generations: Vec<u32>,
    alive: Vec<u64>,
    free_list: Vec<u64>,
    components: HashMap<TypeId, ComponentVec>,
    resources: HashMap<TypeId, ResourceVec>,
    tagged: HashMap<String, u64>,
    collision_events: Vec<CollisionEvent>,
}

impl World {
    pub fn new() -> Self {
        Self {
            next_entity: 0,
            generations: Vec::new(),
            alive: Vec::new(),
            free_list: Vec::new(),
            components: HashMap::new(),
            resources: HashMap::new(),
            tagged: HashMap::new(),
            collision_events: Vec::new(),
        }
    }

    // -- entity ops --

    /// Spawn a new entity, return its handle (with generation 0).
    pub fn spawn(&mut self) -> Entity {
        if let Some(id) = self.free_list.pop() {
            let gen = self.generations[id as usize];
            self.alive.push(id);
            Entity::new(id, gen)
        } else {
            let id = self.next_entity;
            self.next_entity += 1;
            self.generations.push(0);
            self.alive.push(id);
            Entity::new(id, 0)
        }
    }

    /// Spawn an entity with a tag for lookup.
    pub fn spawn_tagged(&mut self, tag: &str) -> Entity {
        let e = self.spawn();
        self.tagged.insert(tag.to_string(), e.id);
        e
    }

    /// Find an entity by tag.
    pub fn find_by_tag(&self, tag: &str) -> Option<Entity> {
        let &id = self.tagged.get(tag)?;
        let gen = self.generations[id as usize];
        Some(Entity::new(id, gen))
    }

    /// Despawn an entity, removing all its components. Increments generation
    /// so stale Entity handles become invalid.
    pub fn despawn(&mut self, entity: Entity) {
        let id = entity.id;
        self.alive.retain(|&e| e != id);
        self.generations[id as usize] += 1;
        self.free_list.push(id);
        for cv in self.components.values_mut() {
            cv.remove(id);
        }
        self.tagged.retain(|_, &mut eid| eid != id);
    }

    /// Check if an entity handle is still valid (generation matches).
    pub fn is_alive(&self, entity: Entity) -> bool {
        entity.id < self.next_entity
            && self.generations[entity.id as usize] == entity.generation
            && self.alive.contains(&entity.id)
    }

    /// Number of living entities.
    pub fn entity_count(&self) -> usize {
        self.alive.len()
    }

    /// All living entities with current generation.
    pub fn entities(&self) -> Vec<Entity> {
        self.alive.iter()
            .map(|&id| Entity::new(id, self.generations[id as usize]))
            .collect()
    }

    // -- component ops --

    /// Attach a component to an entity.
    pub fn insert<T: Component>(&mut self, entity: Entity, component: T) {
        let type_id = TypeId::of::<T>();
        self.components
            .entry(type_id)
            .or_insert_with(ComponentVec::new)
            .insert(entity.id, component);
    }

    /// Get an immutable reference to an entity's component.
    pub fn get<T: Component>(&self, entity: Entity) -> Option<&T> {
        self.components.get(&TypeId::of::<T>())?.get(entity.id)
    }

    /// Get a mutable reference to an entity's component.
    pub fn get_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        self.components.get_mut(&TypeId::of::<T>())?.get_mut(entity.id)
    }

    /// Check if an entity has a component.
    pub fn has<T: Component>(&self, entity: Entity) -> bool {
        self.components.get(&TypeId::of::<T>())
            .map_or(false, |cv| cv.has(entity.id))
    }

    /// Remove a component from an entity.
    pub fn remove<T: Component>(&mut self, entity: Entity) -> bool {
        self.components.get_mut(&TypeId::of::<T>())
            .map_or(false, |cv| cv.remove(entity.id))
    }

    /// Get all entities with a specific component.
    pub fn query<T: Component>(&self) -> Vec<Entity> {
        self.components.get(&TypeId::of::<T>())
            .map_or(Vec::new(), |cv| {
                cv.entities().into_iter()
                    .map(|id| Entity::new(id, self.generations[id as usize]))
                    .collect()
            })
    }

    /// Number of entities with a given component.
    pub fn count<T: Component>(&self) -> usize {
        self.components.get(&TypeId::of::<T>())
            .map_or(0, |cv| cv.len())
    }

    /// Iterate all entities that have both component A and B.
    pub fn query2<A: Component, B: Component>(&self) -> Vec<Entity> {
        let a_entities = self.query::<A>();
        a_entities.into_iter()
            .filter(|&e| self.has::<B>(e))
            .collect()
    }

    /// Iterate all entities that have components A, B, and C.
    pub fn query3<A: Component, B: Component, C: Component>(&self) -> Vec<Entity> {
        let a_entities = self.query::<A>();
        a_entities.into_iter()
            .filter(|&e| self.has::<B>(e) && self.has::<C>(e))
            .collect()
    }

    /// Iterate entities matching an arbitrary list of TypeIds.
    /// The slice must contain at least one TypeId.
    pub fn query_many(&self, type_ids: &[TypeId]) -> Vec<Entity> {
        if type_ids.is_empty() {
            return self.entities();
        }
        // Start from the smallest component store for efficiency.
        let (first_id, rest) = type_ids.split_first().unwrap();
        let candidates = match self.components.get(first_id) {
            Some(cv) => cv.entities(),
            None => return Vec::new(),
        };
        candidates.into_iter()
            .filter(|&id| rest.iter().all(|tid| {
                self.components.get(tid).map_or(false, |cv| cv.has(id))
            }))
            .map(|id| Entity::new(id, self.generations[id as usize]))
            .collect()
    }

    // -- resource ops --

    /// Insert a global resource (singleton).
    pub fn insert_resource<T: Send + Sync + 'static>(&mut self, resource: T) {
        let type_id = TypeId::of::<T>();
        self.resources.insert(type_id, ResourceVec::new(resource));
    }

    /// Get an immutable reference to a global resource.
    pub fn get_resource<T: 'static>(&self) -> Option<&T> {
        self.resources.get(&TypeId::of::<T>())?.get::<T>()
    }

    /// Get a mutable reference to a global resource.
    pub fn get_resource_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.resources.get_mut(&TypeId::of::<T>())?.get_mut::<T>()
    }

    /// Check if a resource exists.
    pub fn has_resource<T: 'static>(&self) -> bool {
        self.resources.contains_key(&TypeId::of::<T>())
    }

    /// Remove a resource.
    pub fn remove_resource<T: 'static>(&mut self) -> bool {
        self.resources.remove(&TypeId::of::<T>()).is_some()
    }

    // -- collision events --

    /// Push a collision event (called by CollisionSystem).
    pub fn push_collision(&mut self, event: CollisionEvent) {
        self.collision_events.push(event);
    }

    /// Drain all pending collision events.
    pub fn drain_collisions(&mut self) -> Vec<CollisionEvent> {
        self.collision_events.drain(..).collect()
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// System trait
// ---------------------------------------------------------------------------

/// A system that operates on the world each tick.
pub trait System: Send + Sync {
    /// Human-readable name for debugging.
    fn name(&self) -> &str;

    /// Run the system for one tick.
    fn run(&mut self, world: &mut World, dt: f32);

    /// Priority ordering (lower = earlier).
    fn priority(&self) -> i32 {
        0
    }
}

// ---------------------------------------------------------------------------
// SystemRunner
// ---------------------------------------------------------------------------

/// Runs systems in priority order.
pub struct SystemRunner {
    systems: Vec<Box<dyn System>>,
}

impl SystemRunner {
    pub fn new() -> Self {
        Self { systems: Vec::new() }
    }

    /// Register a system.
    pub fn add_system(&mut self, system: Box<dyn System>) {
        self.systems.push(system);
        self.systems.sort_by_key(|s| s.priority());
    }

    /// Run all systems sequentially in priority order.
    pub fn run_all(&mut self, world: &mut World, dt: f32) {
        for system in &mut self.systems {
            system.run(world, dt);
        }
    }

    /// Number of registered systems.
    pub fn system_count(&self) -> usize {
        self.systems.len()
    }

    /// System names.
    pub fn system_names(&self) -> Vec<&str> {
        self.systems.iter().map(|s| s.name()).collect()
    }

    /// Clear all systems.
    pub fn clear(&mut self) {
        self.systems.clear();
    }
}

impl Default for SystemRunner {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Component impls
// ---------------------------------------------------------------------------

// Implement Component for common types
impl Component for f32 {}
impl Component for i32 {}
impl Component for u32 {}
impl Component for u64 {}
impl Component for String {}
impl Component for bool {}
impl Component for Vec2 {}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Test components
    #[derive(Debug, Clone)]
    struct Position { x: f32, y: f32 }
    impl Component for Position {}

    #[derive(Debug, Clone)]
    struct Velocity { dx: f32, dy: f32 }
    impl Component for Velocity {}

    #[derive(Debug, Clone)]
    struct Health(i32);
    impl Component for Health {}

    #[derive(Debug, Clone)]
    struct Tag(String);
    impl Component for Tag {}

    #[test]
    fn test_spawn_and_despawn() {
        let mut world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();
        assert_eq!(world.entity_count(), 2);
        assert!(world.is_alive(e1));
        world.despawn(e1);
        assert!(!world.is_alive(e1));
        assert_eq!(world.entity_count(), 1);
        // e2 is still alive
        assert!(world.is_alive(e2));
    }

    #[test]
    fn test_generation_invalidation() {
        let mut world = World::new();
        let e1 = world.spawn();
        let stale = e1; // copy before despawn
        world.despawn(e1);
        // stale handle is now invalid
        assert!(!world.is_alive(stale));
        // new entity at same index gets incremented generation
        let e2 = world.spawn();
        assert_eq!(e2.id, stale.id);
        assert_ne!(e2.generation, stale.generation);
        // old stale handle is still invalid
        assert!(!world.is_alive(stale));
    }

    #[test]
    fn test_insert_get_component() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert(e, Position { x: 10.0, y: 20.0 });
        let pos = world.get::<Position>(e).unwrap();
        assert_eq!(pos.x, 10.0);
        assert_eq!(pos.y, 20.0);
    }

    #[test]
    fn test_get_mut_component() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert(e, Health(100));
        world.get_mut::<Health>(e).unwrap().0 -= 25;
        assert_eq!(world.get::<Health>(e).unwrap().0, 75);
    }

    #[test]
    fn test_has_component() {
        let mut world = World::new();
        let e = world.spawn();
        assert!(!world.has::<Position>(e));
        world.insert(e, Position { x: 0.0, y: 0.0 });
        assert!(world.has::<Position>(e));
    }

    #[test]
    fn test_remove_component() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert(e, Position { x: 0.0, y: 0.0 });
        assert!(world.remove::<Position>(e));
        assert!(!world.has::<Position>(e));
    }

    #[test]
    fn test_query() {
        let mut world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();
        let e3 = world.spawn();
        world.insert(e1, Position { x: 0.0, y: 0.0 });
        world.insert(e2, Position { x: 1.0, y: 1.0 });
        world.insert(e3, Health(100));
        let with_pos = world.query::<Position>();
        assert_eq!(with_pos.len(), 2);
    }

    #[test]
    fn test_query2() {
        let mut world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();
        world.insert(e1, Position { x: 0.0, y: 0.0 });
        world.insert(e1, Velocity { dx: 1.0, dy: 0.0 });
        world.insert(e2, Position { x: 0.0, y: 0.0 });
        let both = world.query2::<Position, Velocity>();
        assert_eq!(both.len(), 1);
        assert_eq!(both[0], e1);
    }

    #[test]
    fn test_query3() {
        let mut world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();
        let e3 = world.spawn();
        world.insert(e1, Position { x: 0.0, y: 0.0 });
        world.insert(e1, Velocity { dx: 1.0, dy: 0.0 });
        world.insert(e1, Health(100));
        world.insert(e2, Position { x: 0.0, y: 0.0 });
        world.insert(e2, Velocity { dx: 1.0, dy: 0.0 });
        world.insert(e3, Position { x: 0.0, y: 0.0 });
        world.insert(e3, Health(100));
        let all_three = world.query3::<Position, Velocity, Health>();
        assert_eq!(all_three.len(), 1);
        assert_eq!(all_three[0], e1);
    }

    #[test]
    fn test_query_many() {
        let mut world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();
        world.insert(e1, Position { x: 0.0, y: 0.0 });
        world.insert(e1, Velocity { dx: 1.0, dy: 0.0 });
        world.insert(e1, Health(100));
        world.insert(e2, Position { x: 0.0, y: 0.0 });
        world.insert(e2, Velocity { dx: 1.0, dy: 0.0 });
        let tids = vec![TypeId::of::<Position>(), TypeId::of::<Velocity>(), TypeId::of::<Health>()];
        let result = world.query_many(&tids);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], e1);
    }

    #[test]
    fn test_tagged_entity() {
        let mut world = World::new();
        let player = world.spawn_tagged("player");
        assert_eq!(world.find_by_tag("player"), Some(player));
        assert!(world.find_by_tag("enemy").is_none());
    }

    #[test]
    fn test_despawn_removes_components() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert(e, Position { x: 0.0, y: 0.0 });
        world.insert(e, Health(100));
        world.despawn(e);
        assert!(!world.has::<Position>(e));
        assert!(!world.has::<Health>(e));
    }

    #[test]
    fn test_despawn_removes_tag() {
        let mut world = World::new();
        let e = world.spawn_tagged("player");
        assert!(world.find_by_tag("player").is_some());
        world.despawn(e);
        assert!(world.find_by_tag("player").is_none());
    }

    // Resources
    #[derive(Debug)]
    struct GameTime { elapsed: f64 }

    #[test]
    fn test_resource_insert_get() {
        let mut world = World::new();
        world.insert_resource(GameTime { elapsed: 0.0 });
        assert!(world.has_resource::<GameTime>());
        assert_eq!(world.get_resource::<GameTime>().unwrap().elapsed, 0.0);
        world.get_resource_mut::<GameTime>().unwrap().elapsed = 1.5;
        assert_eq!(world.get_resource::<GameTime>().unwrap().elapsed, 1.5);
    }

    // Collision events
    #[test]
    fn test_collision_events() {
        let mut world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();
        world.push_collision(CollisionEvent {
            entity_a: e1,
            entity_b: e2,
            overlap_x: 0.5,
            overlap_y: 0.0,
        });
        assert_eq!(world.drain_collisions().len(), 1);
        assert!(world.drain_collisions().is_empty());
    }

    // Test system
    struct MoveSystem;
    impl System for MoveSystem {
        fn name(&self) -> &str { "MoveSystem" }
        fn run(&mut self, world: &mut World, dt: f32) {
            let entities = world.query2::<Position, Velocity>();
            for e in entities {
                let dx = world.get::<Velocity>(e).unwrap().dx;
                let dy = world.get::<Velocity>(e).unwrap().dy;
                let pos = world.get_mut::<Position>(e).unwrap();
                pos.x += dx * dt;
                pos.y += dy * dt;
            }
        }
    }

    #[test]
    fn test_system_runner() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert(e, Position { x: 0.0, y: 0.0 });
        world.insert(e, Velocity { dx: 10.0, dy: 5.0 });

        let mut runner = SystemRunner::new();
        runner.add_system(Box::new(MoveSystem));
        assert_eq!(runner.system_count(), 1);
        assert_eq!(runner.system_names(), vec!["MoveSystem"]);

        runner.run_all(&mut world, 1.0);
        let pos = world.get::<Position>(e).unwrap();
        assert!((pos.x - 10.0).abs() < 0.01);
        assert!((pos.y - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_system_priority_order() {
        use std::cell::Cell;
        use std::rc::Rc;

        let order = Rc::new(Cell::new(Vec::<i32>::new()));
        let order_clone = order.clone();

        struct HighPri { order: Rc<Cell<Vec<i32>>> }
        impl System for HighPri {
            fn name(&self) -> &str { "High" }
            fn priority(&self) -> i32 { -10 }
            fn run(&mut self, _: &mut World, _: f32) {
                let mut v = self.order.take();
                v.push(1);
                self.order.set(v);
            }
        }

        struct LowPri { order: Rc<Cell<Vec<i32>>> }
        impl System for LowPri {
            fn name(&self) -> &str { "Low" }
            fn priority(&self) -> i32 { 10 }
            fn run(&mut self, _: &mut World, _: f32) {
                let mut v = self.order.take();
                v.push(2);
                self.order.set(v);
            }
        }

        let mut runner = SystemRunner::new();
        runner.add_system(Box::new(LowPri { order: order.clone() }));
        runner.add_system(Box::new(HighPri { order: order.clone() }));

        let mut world = World::new();
        runner.run_all(&mut world, 1.0);

        let v = order_clone.take();
        assert_eq!(v, vec![1, 2]); // high priority first
    }
}
