use std::any::{Any, TypeId};
use std::collections::HashMap;

use super::entity::EntityId;

pub trait Component: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct ComponentStore {
    stores: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl ComponentStore {
    pub fn new() -> Self {
        Self {
            stores: HashMap::new(),
        }
    }

    pub fn insert<T: Component + 'static>(&mut self, entity: EntityId, component: T) {
        let store = self
            .stores
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(HashMap::<EntityId, T>::new()));
        let map = store.downcast_mut::<HashMap<EntityId, T>>().unwrap();
        map.insert(entity, component);
    }

    pub fn get<T: Component + 'static>(&self, entity: EntityId) -> Option<&T> {
        self.stores
            .get(&TypeId::of::<T>())
            .and_then(|s| s.downcast_ref::<HashMap<EntityId, T>>())
            .and_then(|m| m.get(&entity))
    }

    pub fn get_mut<T: Component + 'static>(&mut self, entity: EntityId) -> Option<&mut T> {
        self.stores
            .get_mut(&TypeId::of::<T>())
            .and_then(|s| s.downcast_mut::<HashMap<EntityId, T>>())
            .and_then(|m| m.get_mut(&entity))
    }

    pub fn remove<T: Component + 'static>(&mut self, entity: EntityId) -> bool {
        self.stores
            .get_mut(&TypeId::of::<T>())
            .and_then(|s| s.downcast_mut::<HashMap<EntityId, T>>())
            .map_or(false, |m| m.remove(&entity).is_some())
    }

    pub fn has<T: Component + 'static>(&self, entity: EntityId) -> bool {
        self.stores
            .get(&TypeId::of::<T>())
            .and_then(|s| s.downcast_ref::<HashMap<EntityId, T>>())
            .map_or(false, |m| m.contains_key(&entity))
    }

    pub fn entities_with<T: Component + 'static>(&self) -> Vec<EntityId> {
        self.stores
            .get(&TypeId::of::<T>())
            .and_then(|s| s.downcast_ref::<HashMap<EntityId, T>>())
            .map_or(Vec::new(), |m| m.keys().copied().collect())
    }
}

impl Default for ComponentStore {
    fn default() -> Self {
        Self::new()
    }
}
