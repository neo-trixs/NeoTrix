use std::any::{Any, TypeId};
use std::collections::HashMap;

/// Entity: 身份标识 (generational arena)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity {
    pub id: u32,
    pub generation: u32,
}

impl Entity {
    pub fn new(id: u32, generation: u32) -> Self {
        Self { id, generation }
    }
}

/// Component: 数据容器 (trait object)
pub trait Component: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Any + Send + Sync> Component for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Component storage for a specific type
pub struct ComponentStorage {
    components: HashMap<u32, Box<dyn Component>>,
}

impl ComponentStorage {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    pub fn insert<T: Component + 'static>(&mut self, entity: u32, component: T) {
        self.components.insert(entity, Box::new(component));
    }

    pub fn get<T: Component + 'static>(&self, entity: u32) -> Option<&T> {
        self.components
            .get(&entity)
            .and_then(|c| c.as_any().downcast_ref::<T>())
    }

    pub fn get_mut<T: Component + 'static>(&mut self, entity: u32) -> Option<&mut T> {
        self.components
            .get_mut(&entity)
            .and_then(|c| c.as_any_mut().downcast_mut::<T>())
    }

    pub fn remove(&mut self, entity: u32) -> Option<Box<dyn Component>> {
        self.components.remove(&entity)
    }

    pub fn contains(&self, entity: u32) -> bool {
        self.components.contains_key(&entity)
    }

    pub fn entities(&self) -> Vec<u32> {
        self.components.keys().copied().collect()
    }
}

/// World: 容器 + 调度器
pub struct World {
    entities: Vec<Option<u32>>,
    generations: Vec<u32>,
    components: HashMap<TypeId, ComponentStorage>,
    resources: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            generations: Vec::new(),
            components: HashMap::new(),
            resources: HashMap::new(),
        }
    }

    /// 创建新实体
    pub fn spawn(&mut self) -> Entity {
        let id = self.entities.len() as u32;
        let generation = 0;
        self.entities.push(Some(id));
        self.generations.push(generation);
        Entity::new(id, generation)
    }

    /// 销毁实体
    pub fn despawn(&mut self, entity: Entity) {
        if (entity.id as usize) < self.entities.len() {
            self.entities[entity.id as usize] = None;
            self.generations[entity.id as usize] += 1;
            // 移除所有组件
            for storage in self.components.values_mut() {
                storage.remove(entity.id);
            }
        }
    }

    /// 添加组件
    pub fn insert_component<T: Component + 'static>(&mut self, entity: Entity, component: T) {
        let type_id = TypeId::of::<T>();
        let storage = self
            .components
            .entry(type_id)
            .or_insert_with(ComponentStorage::new);
        storage.insert(entity.id, component);
    }

    /// 获取组件
    pub fn get_component<T: Component + 'static>(&self, entity: Entity) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.components
            .get(&type_id)
            .and_then(|storage| storage.get(entity.id))
    }

    /// 获取可变组件
    pub fn get_component_mut<T: Component + 'static>(&mut self, entity: Entity) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.components
            .get_mut(&type_id)
            .and_then(|storage| storage.get_mut(entity.id))
    }

    /// 移除组件
    pub fn remove_component<T: Component + 'static>(&mut self, entity: Entity) {
        let type_id = TypeId::of::<T>();
        if let Some(storage) = self.components.get_mut(&type_id) {
            storage.remove(entity.id);
        }
    }

    /// 检查实体是否有组件
    pub fn has_component<T: Component + 'static>(&self, entity: Entity) -> bool {
        let type_id = TypeId::of::<T>();
        self.components
            .get(&type_id)
            .map_or(false, |storage| storage.contains(entity.id))
    }

    /// 获取所有具有特定组件的实体
    pub fn query<T: Component + 'static>(&self) -> Vec<Entity> {
        let type_id = TypeId::of::<T>();
        if let Some(storage) = self.components.get(&type_id) {
            storage
                .entities()
                .into_iter()
                .map(|id| Entity::new(id, self.generations[id as usize]))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// 获取所有实体
    pub fn entities(&self) -> Vec<Entity> {
        self.entities
            .iter()
            .enumerate()
            .filter_map(|(id, &gen)| {
                if let Some(generation) = gen {
                    Some(Entity::new(id as u32, generation))
                } else {
                    None
                }
            })
            .collect()
    }

    /// 设置资源
    pub fn insert_resource<T: Any + Send + Sync>(&mut self, resource: T) {
        let type_id = TypeId::of::<T>();
        self.resources.insert(type_id, Box::new(resource));
    }

    /// 获取资源
    pub fn get_resource<T: Any + Send + Sync>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.resources
            .get(&type_id)
            .and_then(|r| r.downcast_ref::<T>())
    }

    /// 获取可变资源
    pub fn get_resource_mut<T: Any + Send + Sync>(&mut self) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.resources
            .get_mut(&type_id)
            .and_then(|r| r.downcast_mut::<T>())
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
