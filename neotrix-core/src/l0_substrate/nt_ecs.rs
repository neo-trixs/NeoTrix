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
    /// ⛔ **世代**：槽位每次被回收时递增。**零值的世代是合法的**，但
    /// 持有者必须去 [`UniversalWorld::contains`] 核对 —— 裸比 `id` 会在
    /// 槽位复用后**静默别名**到新实体（ABA）。
    ///
    /// 移植自 `ra-ecs`（Apache-2.0）的 `EcsEntity { slot, generation }`。
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

/// 槽位的存活与世代元数据。
///
/// **为什么必须是独立并行数组**，而不是从 `entities: Vec<Option<…>>` 里读：
/// `despawn` 之后槽位变空（`None`），若世代只活在句柄里，**释放即丢失**
/// ⇒ 复用该槽位时没有可递增的基准，也就无法区分「旧句柄」与「新实体」。
/// `ra-ecs` 的 `EntityMeta { generation, alive }` 正是为此存在。
/// 世代号存在 `metas` 里 ⇒ 槽位空着也**保留**上次那个值。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EntityMeta {
    pub(crate) generation: u32,
    pub(crate) alive: bool,
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

    /// **幂等**：同一实体重复登记会静默变成**多条**，于是任何按 archetype
    /// 迭代的代码都会把同一个实体返回多次（实测：插两个组件后，
    /// 一个实体在全体 archetype 里共两条登记）。
    ///
    /// O(n) 扫一遍换掉一整类「静默重复」的错；`entities` 本就是
    /// 装饰性结构（当前无外部消费者），这里要正确性不要吞吐。
    pub fn add_entity(&mut self, entity: UniversalEntity) {
        if !self.entities.iter().any(|e| e.id == entity.id) {
            self.entities.push(entity);
        }
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
    /// 槽位 → 活实体的 archetype（`None` = 该槽位空着）。
    /// **与 [`Self::metas`] 等长**，下标即 `EntityId.0`。
    entities: Vec<Option<UniversalEntity>>,
    /// 槽位世代与存活位。**空槽也保留世代**（见 [`EntityMeta`]）。
    metas: Vec<EntityMeta>,
    /// 已释放槽位的 **LIFO** 空闲表 —— 分配走 `pop()`，释放走 `push()`。
    free_slots: Vec<u32>,
    archetypes: HashMap<ArchetypeId, Archetype>,
    components: HashMap<(EntityId, TypeId), Box<dyn Any + Send + Sync>>,
    resources: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    events: Vec<Box<dyn Any + Send + Sync>>,
    next_archetype_id: u64,
}

impl UniversalWorld {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            metas: Vec::new(),
            free_slots: Vec::new(),
            archetypes: HashMap::new(),
            components: HashMap::new(),
            resources: HashMap::new(),
            events: Vec::new(),
            next_archetype_id: 0,
        }
    }

    /// 句柄是否仍指向一个**活着**的实体。
    ///
    /// **每个公开的按实体操作都必须先过这里** —— 这是「槽位复用后旧句柄
    /// 静默串写到新实体」（ABA）唯一的检测点。移植自 `ra-ecs`
    /// （Apache-2.0）的 `EcsWorld::contains`。
    pub fn contains(&self, entity: UniversalEntity) -> bool {
        match self.metas.get(entity.id.0 as usize) {
            Some(meta) => meta.alive && meta.generation == entity.generation,
            None => false,
        }
    }

    /// Spawn a new entity
    ///
    /// 优先复用 [`Self::free_slots`] 的 LIFO 空闲表。**不是**单调递增的新槽位：
    /// 单调递增意味着长跑的世界里 `entities`/`metas` 无限增长（实测：每 64 次
    /// spawn/despawn 循环就 +64 槽，永不回收）。
    ///
    /// 新槽位的世代取 **1**（不是 0）—— 与 `ra-ecs` 一致，让「构造出来的
    /// 世代」与「递增出来的世代」在直觉上区分得开。
    pub fn spawn(&mut self) -> UniversalEntity {
        let slot = match self.free_slots.pop() {
            Some(slot) => {
                self.metas[slot as usize].alive = true;
                slot
            }
            None => {
                let slot = self.metas.len() as u32;
                self.metas.push(EntityMeta { generation: 1, alive: true });
                self.entities.push(None);
                slot
            }
        };
        let generation = self.metas[slot as usize].generation;
        let entity = UniversalEntity::new(EntityId(u64::from(slot)), generation);
        self.entities[slot as usize] = Some(entity);
        entity
    }

    /// Despawn an entity
    ///
    /// ⛔ 陈旧句柄（已被复用槽位取代，或已被 despawn 过）**返回 `false` 且不做
    /// 任何事** —— 绝不拿它去清别人的组件或改别人的 archetype。
    ///
    /// 槽位**不被销毁**，只是入 [`Self::free_slots`] 空闲表并递增世代。
    pub fn despawn(&mut self, entity: UniversalEntity) -> bool {
        if !self.contains(entity) {
            return false;
        }
        let idx = entity.id.0 as usize;

        // 从**世界里存的那份** archetype 移出。⛔ 不是句柄拷贝 —— 那份可能早已
        // 过期（没有任何途径能刷新调用方手里的 `.archetype`）。
        if let Some(stored) = self.entities[idx] {
            if let Some(arch) = self.archetypes.get_mut(&stored.archetype) {
                arch.remove_entity(entity.id);
            }
        }

        self.entities[idx] = None;
        self.remove_all_components(entity.id);

        // 世代递增 + 入空闲表。世代号活在 `metas` 里，所以槽位空着也留着它。
        self.metas[idx].alive = false;
        self.metas[idx].generation = self.metas[idx].generation.wrapping_add(1);
        self.free_slots.push(idx as u32);
        true
    }

    /// Insert a component into an entity
    ///
    /// 返回是否真的写入。**`false` = 句柄已失效，本次什么都没做。**
    ///
    /// ⛔ 不能是 `()`：「拒收」与「接受」必须能被调用方区分，否则一次被
    /// 拒的写入与一次成功写入在调用点长得一模一样 —— 而组件行已经落进了
    /// `components`，`query()` 却遍历不到它（它只扫存活槽位）
    /// ⇒ 一个**索引里有、迭代里没有、且 `despawn` 早已跑过所以永不清**的
    /// 幽灵行。返回 `bool` 是让这件事**可检测**的最小改动。
    pub fn insert_component<T: Component>(&mut self, entity: UniversalEntity, component: T) -> bool {
        if !self.contains(entity) {
            return false;
        }
        let type_id = component.type_id();
        
        // Store component
        self.components.insert(
            (entity.id, type_id),
            Box::new(component),
        );
        
        // Update archetype
        self.update_archetype(entity, type_id);
        true
    }

    /// Get a component reference
    pub fn get_component<T: Component>(&self, entity: UniversalEntity) -> Option<&T> {
        if !self.contains(entity) {
            return None;
        }
        let type_id = TypeId::of::<T>();
        self.components
            .get(&(entity.id, type_id))
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }

    /// Get a mutable component reference
    pub fn get_component_mut<T: Component>(&mut self, entity: UniversalEntity) -> Option<&mut T> {
        if !self.contains(entity) {
            return None;
        }
        let type_id = TypeId::of::<T>();
        self.components
            .get_mut(&(entity.id, type_id))
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }

    /// Remove a component from an entity
    pub fn remove_component<T: Component>(&mut self, entity: UniversalEntity) -> Option<T> {
        if !self.contains(entity) {
            return None;
        }
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
        if !self.contains(entity) {
            return false;
        }
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
        // ⭐ 直接按 `downcast::<T>()` 的 `Result` 分派：
        //   原写法先 `is::<T>()` 再 `downcast::<T>().unwrap()` —— **unwrap 是纯冗余**
        //   （刚 `is` 过又 `unwrap`）。两者语义完全等价，但后者无 panic 路径。
        for event in self.events.drain(..) {
            match event.downcast::<T>() {
                Ok(t) => events.push(*t),
                // 换不回原类型 ⇒ 保留（与原 `else` 分支同义）
                Err(ev) => remaining.push(ev),
            }
        }
        self.events = remaining;
        events
    }

    /// Helper: update archetype when component is added
    fn update_archetype(&mut self, entity: UniversalEntity, type_id: TypeId) {
        let idx = entity.id.0 as usize;

        // ⛔ 读**世界里存的那份** archetype，不是 `entity.archetype`（句柄拷贝）。
        //   调用方手里的句柄在首次 insert 之后 archetype 字段就**永久过期**
        //   （没有任何 API 能刷新它）⇒ 拿它算，等于每次都从同一份陈旧基底重算，
        //   实测让 archetype 的 `component_types` **插两个组件后仍然只有 1 个**。
        let current_archetype = self.entities[idx]
            .map(|e| e.archetype)
            .unwrap_or(ArchetypeId(0));
        
        // Get or create archetype with new component
        let mut component_types = self.archetypes
            .get(&current_archetype)
            .map(|a| a.component_types.clone())
            .unwrap_or_default();
        
        component_types.insert(type_id);
        
        // Find or create matching archetype
        let archetype_id = self.find_or_create_archetype(component_types);
        
        // **从旧 archetype 移出**。否则实体留在它待过的**每一个** archetype 里，
        //   而 `add_entity` 无去重 ⇒ 每插一个新组件就多留一份登记
        //   （实测：一个实体两条登记）。任何按 archetype 迭代的代码
        //   ——那正是 archetype 存在的全部理由——都会把它返回多次。
        if archetype_id != current_archetype {
            if let Some(old) = self.archetypes.get_mut(&current_archetype) {
                old.remove_entity(entity.id);
            }
        }
        
        // Update entity archetype
        if let Some(e) = self.entities[idx].as_mut() {
            e.archetype = archetype_id;
        }
        
        // Add entity to new archetype
        if let Some(archetype) = self.archetypes.get_mut(&archetype_id) {
            archetype.add_entity(entity);
        }
    }

    /// Helper: update archetype when component is removed
    fn update_archetype_remove(&mut self, entity: UniversalEntity, type_id: TypeId) {
        let idx = entity.id.0 as usize;
        // 同上：读存的那份，不是句柄拷贝。
        let current_archetype = self.entities[idx]
            .map(|e| e.archetype)
            .unwrap_or(ArchetypeId(0));
        
        if let Some(archetype) = self.archetypes.get(&current_archetype) {
            let mut component_types = archetype.component_types.clone();
            component_types.remove(&type_id);
            
            let archetype_id = self.find_or_create_archetype(component_types);
            
            // 从旧 archetype 移出（同 `update_archetype`）。
            if archetype_id != current_archetype {
                if let Some(old) = self.archetypes.get_mut(&current_archetype) {
                    old.remove_entity(entity.id);
                }
            }
            
            // Update entity archetype
            if let Some(e) = self.entities[idx].as_mut() {
                e.archetype = archetype_id;
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

// ===========================================================================
// 句柄契约测试 —— 移植自 `ra-ecs`（Apache-2.0）的 `EntityMeta` + LIFO 空闲表
//
// ⛔ 这组测试**先于**实现写成，作用是把两个被证伪的缺陷钉成红的：
//   ① `generation` 字段只出现在声明/构造/赋值三处（`:23/:28/:31`），
//      **从未被任何代码读过** ⇒ 它承诺「防复用误命中」却零强制；
//   ② `update_archetype` 读的是**调用方手里那份句柄拷贝**的 `.archetype`，
//      而不是世界里存的那份 ⇒ 拿同一个句柄连插两个组件，第二个算出的
//      仍是「从 ArchetypeId(0) 起算的 1 个组件」，archetype 集合**永不累积**。
// ===========================================================================
#[cfg(test)]
mod tests_5 {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct Hp(i32);
    impl Component for Hp {}

    #[derive(Clone, Debug, PartialEq)]
    struct Xp(i32);
    impl Component for Xp {}

    /// 活着的实体的当前 archetype 存了几个组件类型。
    fn live_archetype_size(world: &UniversalWorld) -> usize {
        let e = world.entities().into_iter().next().expect("应有一个活实体");
        world
            .archetypes
            .get(&e.archetype)
            .expect("活实体必属某个 archetype")
            .component_types
            .len()
    }

    /// ⛔ **幽灵组件**：`despawn` 之后再拿**旧句柄**插组件，不得凭空造出
    /// 一个「`has_component` 说有、`query` 看不见、且永远不会被回收」的行。
    ///
    /// 旧实现的病：`insert_component` 直接 `self.components.insert((entity.id, type_id), ..)`，
    /// **从不检查该实体是否还活着** ⇒ 而 `query()` 只遍历存活槽位
    /// ⇒ 落进一个「索引里有、迭代里没有、且 `despawn` 早已跑过所以永不清」的黑洞。
    #[test]
    fn inserting_through_a_dead_handle_must_not_create_a_ghost_component() {
        let mut world = UniversalWorld::new();
        let e = world.spawn();
        world.insert_component(e, Hp(10));
        assert!(world.despawn(e));

        // 拿死句柄再插一次 —— 必须被拒。
        let accepted = world.insert_component(e, Hp(99));
        assert!(
            !accepted,
            "死句柄的插入必须被拒并让调用方知道；静默接受 = 造出迭代看不见的幽灵行"
        );
        assert!(
            world.get_component::<Hp>(e).is_none(),
            "死句柄不得读出任何组件"
        );
        assert!(!world.has_component::<Hp>(e), "死句柄不得被说有组件");
        // 关键：不能有「索引里有、迭代里没有」的行。
        assert_eq!(world.entity_count(), 0, "世界里不该有活实体");
        assert!(
            world.components.is_empty(),
            "组件表必须被清空 —— 幽灵行就是这里漏的"
        );
    }

    /// **槽位复用 + 世代递增**：这是 `generation` 字段唯一能兑现承诺的方式。
    ///
    /// 旧实现的病：`spawn()` 只 `self.entities.push(...)`，`next_entity_id` 单调递增
    /// ⇒ **槽位永不回收** ⇒ 长跑的世界里 `entities` Vec 无限增长，
    /// 且没有任何机制能让 `generation` 变成非 0。
    #[test]
    fn despawned_slot_is_reused_and_carries_a_new_generation() {
        let mut world = UniversalWorld::new();
        let first = world.spawn();
        assert!(world.despawn(first));

        let second = world.spawn();
        assert_eq!(
            first.id, second.id,
            "释放的槽位必须被复用，否则 entities Vec 单调增长、长跑必泄漏"
        );
        assert_ne!(
            first.generation, second.generation,
            "复用槽位必须换世代，否则旧句柄会静默别名到新实体（ABA）"
        );
        assert_eq!(world.entity_count(), 1);
    }

    /// **ABA 契约**：旧句柄在槽位被复用后**绝不能**读写新实体。
    ///
    /// 这是上两条合起来的**真正目的**：单独看「复用槽位」只是省内存，
    /// 单独看「换世代」只是个数字；合起来才让「拿旧句柄操作」变成可检测的失败
    /// 而不是一次静默的串写。这正是 `ra-ecs` 每个公开操作都过 `contains()` 的原因。
    #[test]
    fn a_stale_handle_never_aliases_the_reused_slot() {
        let mut world = UniversalWorld::new();
        let stale = world.spawn();
        assert!(world.despawn(stale));

        let fresh = world.spawn();
        world.insert_component(fresh, Hp(7));
        assert_eq!(world.get_component::<Hp>(fresh), Some(&Hp(7)));

        // 旧句柄：读不到新实体的组件，写也进不去，despawn 也不能把新实体杀掉。
        assert!(
            world.get_component::<Hp>(stale).is_none(),
            "旧句柄读到了复用槽位上的新实体 ⇒ ABA 串写"
        );
        assert!(!world.has_component::<Hp>(stale));
        assert!(
            !world.insert_component(stale, Hp(123)),
            "旧句柄不得写入复用槽位"
        );
        assert!(
            !world.despawn(stale),
            "旧句柄不得 despawn 掉复用槽位上的新实体"
        );
        // 新实体毫发无伤。
        assert_eq!(world.get_component::<Hp>(fresh), Some(&Hp(7)));
        assert_eq!(world.entity_count(), 1);
    }

    /// ⛔ **archetype 的组件集合必须累积**。
    ///
    /// 旧实现的病：`update_archetype(entity, ..)` 读 `entity.archetype` ——
    /// 那是**调用方手里那份句柄拷贝**。调用方拿同一个句柄连插两个组件时，
    /// 第二次读到的仍是 spawn 时的 `ArchetypeId(0)` ⇒ 于是每次都从空集重算
    /// ⇒ archetype 里的 `component_types` **永远只有 1 个**，
    /// 而实体在真实意义上已经有两个组件了。
    ///
    /// 旧测试测不出来：`query()` 遍历的是 `entities` 而不是 archetype，
    /// 所以 `test_query` 照样绿 —— **测试绕过了坏掉的那条路径**。
    #[test]
    fn archetype_component_set_accumulates_across_inserts() {
        let mut world = UniversalWorld::new();
        let e = world.spawn();
        // 同一个句柄连插两个 —— 不刷新句柄拷贝。
        world.insert_component(e, Hp(1));
        assert_eq!(live_archetype_size(&world), 1, "插一个 ⇒ 集合里 1 个");
        world.insert_component(e, Xp(2));
        assert_eq!(
            live_archetype_size(&world),
            2,
            "插第二个后 archetype 必须记 2 个组件；\
             读句柄拷贝重算的实现会让这里停在 1（组件集合永不累积）"
        );
        world.insert_component(e, Hp(3));
        assert_eq!(
            live_archetype_size(&world),
            2,
            "覆盖写不得让集合涨到 3（同一 TypeId 只算一次）"
        );
    }

    /// ⛔ **archetype 成员不得累积**：实体换 archetype 时必须**从旧的移出**。
    ///
    /// 旧实现的病：`update_archetype` 只 `new_archetype.add_entity(entity)`，
    /// **从不** `old.remove_entity` ⇒ 实体留在它待过的**每一个** archetype 里，
    /// 且 `add_entity` 是 `Vec::push`（无去重）⇒ 每插一个新组件就多留一份。
    /// 一旦有任何按 archetype 迭代的代码（这是 archetype 存在的全部理由），
    /// 同一个实体会被返回 N 次。
    #[test]
    fn entity_is_not_duplicated_into_every_archetype_it_ever_had() {
        let mut world = UniversalWorld::new();
        let e = world.spawn();

        world.insert_component(e, Hp(1));
        world.insert_component(e, Xp(2));

        // 每个 archetype 里，同一个实体至多出现一次。
        for (id, arch) in &world.archetypes {
            let occurrences = arch.entities.iter().filter(|x| x.id == e.id).count();
            assert!(
                occurrences <= 1,
                "实体在 archetype {id:?} 里出现了 {occurrences} 次 \
                 （换 archetype 时没从旧的移出 + add_entity 无去重）"
            );
        }
        // 且总数必须等于活实体数 —— 没有「重复登记」也没有「漏登记」。
        let total: usize = world.archetypes.values().map(|a| a.entities.len()).sum();
        assert_eq!(
            total, 1,
            "一个实体只应登记在一个 archetype 里，实测共 {total} 条登记"
        );
    }

    /// `despawn` 幂等 + 拒陈旧句柄（契约锁定，防回归）。
    #[test]
    fn despawn_is_idempotent_and_rejects_stale_handles() {
        let mut world = UniversalWorld::new();
        let e = world.spawn();
        assert!(world.despawn(e), "第一次 despawn 必须成功");
        assert!(!world.despawn(e), "第二次 despawn 同一个句柄必须返回 false");
        assert_eq!(world.entity_count(), 0);
    }

    /// 大量 spawn/despawn 循环后，槽位表必须**不增长**（旧实现每轮泄漏 1 槽）。
    #[test]
    fn churn_does_not_grow_the_slot_table() {
        let mut world = UniversalWorld::new();
        let mut handles = Vec::new();
        for i in 0..64 {
            let e = world.spawn();
            world.insert_component(e, Hp(i));
            handles.push(e);
        }
        for e in &handles {
            assert!(world.despawn(*e));
        }
        assert_eq!(world.entity_count(), 0);
        assert!(world.components.is_empty(), "全部 despawn 后组件表必须空");
        // 再开一轮：槽位表不该比第一轮结束时更大。
        let after_first = world.entities.len();
        for i in 0..64 {
            let e = world.spawn();
            world.insert_component(e, Xp(i));
            assert!(world.despawn(e));
        }
        assert_eq!(
            world.entities.len(),
            after_first,
            "第二轮 churn 后槽位表不得增长（旧实现 push-only ⇒ 每轮 +64）"
        );
        assert_eq!(world.entity_count(), 0);
    }
}
