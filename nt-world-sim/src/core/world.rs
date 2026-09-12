use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
use super::entity::{UniversalEntity, EntityId, Archetype, ArchetypeId};

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
mod tests {
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
