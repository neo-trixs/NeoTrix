// L0 简单 ECS — 缓存友好查询
// 借鉴: Bevy Archetype + Blubber Engine (SoA 布局)

use std::any::{Any, TypeId};
use std::collections::HashMap;

/// 组件存储 — 类型擦除的 Vec<Option<T>>
/// 使用 Option<T> 避免 unsafe zeroed 初始化
struct ComponentStore {
    data: Box<dyn Any>,
}

impl ComponentStore {
    fn new<T: 'static>() -> Self {
        Self { data: Box::new(Vec::<Option<T>>::new()) }
    }

    fn as_vec<T: 'static>(&self) -> &Vec<Option<T>> {
        self.data.downcast_ref::<Vec<Option<T>>>().expect("组件类型不匹配")
    }

    fn as_vec_mut<T: 'static>(&mut self) -> &mut Vec<Option<T>> {
        self.data.downcast_mut::<Vec<Option<T>>>().expect("组件类型不匹配")
    }
}

/// 简单 ECS — 按 Entity ID 索引，按 TypeId 分组存储
/// 借鉴 Blubber Engine: SoA (Array of Structures) 布局
/// 相同组件连续存储 → 缓存友好
/// 使用 Option<T> 确保 zero-cost 初始化 (无 unsafe)
pub struct SimpleEcs {
    /// 每种组件类型一个存储
    stores: HashMap<TypeId, ComponentStore>,
    /// Entity 存活标记
    alive: Vec<bool>,
    /// 空闲 ID 池 (借鉴 Specs 的 slot map)
    free_ids: Vec<u64>,
    next_id: u64,
}

impl SimpleEcs {
    pub fn new() -> Self {
        Self { stores: HashMap::new(), alive: Vec::new(), free_ids: Vec::new(), next_id: 0 }
    }

    /// 创建实体，返回 ID
    pub fn spawn(&mut self) -> u64 {
        if let Some(id) = self.free_ids.pop() {
            self.alive[id as usize] = true;
            id
        } else {
            let id = self.next_id;
            self.alive.push(true);
            self.next_id += 1;
            id
        }
    }

    /// 销毁实体
    pub fn despawn(&mut self, id: u64) {
        if (id as usize) < self.alive.len() {
            self.alive[id as usize] = false;
            self.free_ids.push(id);
        }
    }

    /// 实体是否存活
    pub fn is_alive(&self, id: u64) -> bool {
        (id as usize) < self.alive.len() && self.alive[id as usize]
    }

    /// 清空所有实体 (用于重新同步)
    pub fn clear(&mut self) {
        self.alive.clear();
        self.free_ids.clear();
        self.next_id = 0;
        self.stores.clear();
    }

    /// 添加组件 (entity 必须先 spawn)
    pub fn insert<T: 'static>(&mut self, id: u64, component: T) {
        assert!(self.is_alive(id), "不能为不存在的实体插入组件");
        let type_id = TypeId::of::<T>();
        let store = self.stores.entry(type_id).or_insert_with(|| ComponentStore::new::<T>());
        let vec = store.as_vec_mut::<T>();
        while vec.len() <= id as usize {
            vec.push(None);
        }
        vec[id as usize] = Some(component);
    }

    /// 获取组件 (不可变)
    pub fn get<T: 'static>(&self, id: u64) -> Option<&T> {
        if !self.is_alive(id) { return None; }
        let type_id = TypeId::of::<T>();
        let store = self.stores.get(&type_id)?;
        let vec = store.as_vec::<T>();
        vec.get(id as usize)?.as_ref()
    }

    /// 获取组件 (可变)
    pub fn get_mut<T: 'static>(&mut self, id: u64) -> Option<&mut T> {
        if !self.is_alive(id) { return None; }
        let type_id = TypeId::of::<T>();
        let store = self.stores.get_mut(&type_id)?;
        let vec = store.as_vec_mut::<T>();
        vec.get_mut(id as usize)?.as_mut()
    }

    /// 查询: 获取所有同时拥有 A 和 B 的实体
    pub fn query2<A: 'static, B: 'static>(&self) -> Vec<(u64, &A, &B)> {
        let a_store = self.stores.get(&TypeId::of::<A>());
        let b_store = self.stores.get(&TypeId::of::<B>());
        let (a_store, b_store) = match (a_store, b_store) {
            (Some(a), Some(b)) => (a, b),
            _ => return Vec::new(),
        };
        let a_vec = a_store.as_vec::<A>();
        let b_vec = b_store.as_vec::<B>();

        let mut results = Vec::new();
        for id in 0..self.next_id {
            if self.is_alive(id) {
                if let (Some(a), Some(b)) = (a_vec.get(id as usize).and_then(|o| o.as_ref()),
                                               b_vec.get(id as usize).and_then(|o| o.as_ref())) {
                    results.push((id, a, b));
                }
            }
        }
        results
    }

    /// 查询: 获取所有同时拥有 A, B, C 的实体
    pub fn query3<A: 'static, B: 'static, C: 'static>(&self) -> Vec<(u64, &A, &B, &C)> {
        let a_store = self.stores.get(&TypeId::of::<A>());
        let b_store = self.stores.get(&TypeId::of::<B>());
        let c_store = self.stores.get(&TypeId::of::<C>());
        let (a_store, b_store, c_store) = match (a_store, b_store, c_store) {
            (Some(a), Some(b), Some(c)) => (a, b, c),
            _ => return Vec::new(),
        };
        let a_vec = a_store.as_vec::<A>();
        let b_vec = b_store.as_vec::<B>();
        let c_vec = c_store.as_vec::<C>();

        let mut results = Vec::new();
        for id in 0..self.next_id {
            if self.is_alive(id) {
                if let (Some(a), Some(b), Some(c)) = (
                    a_vec.get(id as usize).and_then(|o| o.as_ref()),
                    b_vec.get(id as usize).and_then(|o| o.as_ref()),
                    c_vec.get(id as usize).and_then(|o| o.as_ref()),
                ) {
                    results.push((id, a, b, c));
                }
            }
        }
        results
    }

    /// 查询: 获取所有拥有 A 的实体
    pub fn query1<A: 'static>(&self) -> Vec<(u64, &A)> {
        let a_store = match self.stores.get(&TypeId::of::<A>()) {
            Some(s) => s,
            None => return Vec::new(),
        };
        let a_vec = a_store.as_vec::<A>();

        let mut results = Vec::new();
        for id in 0..self.next_id {
            if self.is_alive(id) {
                if let Some(a) = a_vec.get(id as usize).and_then(|o| o.as_ref()) {
                    results.push((id, a));
                }
            }
        }
        results
    }

    /// 查询: 获取所有拥有 A 的实体 ID (轻量)
    pub fn query1_ids<A: 'static>(&self) -> Vec<u64> {
        let a_store = match self.stores.get(&TypeId::of::<A>()) {
            Some(s) => s,
            None => return Vec::new(),
        };
        let a_vec = a_store.as_vec::<A>();

        let mut results = Vec::new();
        for id in 0..self.next_id {
            if self.is_alive(id) {
                if a_vec.get(id as usize).and_then(|o| o.as_ref()).is_some() {
                    results.push(id);
                }
            }
        }
        results
    }

    /// 存活实体数量
    pub fn count(&self) -> usize {
        self.alive.iter().filter(|&&a| a).count()
    }
}

impl Default for SimpleEcs {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_insert_get() {
        let mut ecs = SimpleEcs::new();
        let id = ecs.spawn();
        ecs.insert(id, 42u32);
        assert_eq!(*ecs.get::<u32>(id).unwrap(), 42);
    }

    #[test]
    fn despawn_hides_components() {
        let mut ecs = SimpleEcs::new();
        let id = ecs.spawn();
        ecs.insert(id, 7u32);
        ecs.despawn(id);
        assert!(!ecs.is_alive(id));
        assert!(ecs.get::<u32>(id).is_none());
    }

    #[test]
    fn query2_filters_correctly() {
        let mut ecs = SimpleEcs::new();
        let a = ecs.spawn();
        ecs.insert(a, 1u32);
        ecs.insert(a, "x");
        let b = ecs.spawn();
        ecs.insert(b, 2u32);
        let r = ecs.query2::<u32, &str>();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].0, a);
    }

    #[test]
    fn free_ids_reused() {
        let mut ecs = SimpleEcs::new();
        let a = ecs.spawn();
        ecs.despawn(a);
        let b = ecs.spawn();
        assert_eq!(a, b);
        assert_eq!(ecs.count(), 1);
    }
}
