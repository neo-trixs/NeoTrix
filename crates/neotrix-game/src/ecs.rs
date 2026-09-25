// L0 简单 ECS — 缓存友好查询
// 借鉴: Bevy Archetype + Blubber Engine (SoA 布局)

use std::any::{Any, TypeId};
use std::collections::HashMap;

/// 组件存储 — 类型擦除的 Vec<Option<T>>
/// 使用 Option<T> 避免 unsafe zeroed 初始化
struct ComponentStore {
    data: Box<dyn Any>,
    /// 按实体下标置空槽位（despawn 清理用，T 在构造时绑定）
    clear: fn(&mut Box<dyn Any>, u64),
}

impl ComponentStore {
    fn new<T: 'static>() -> Self {
        Self {
            data: Box::new(Vec::<Option<T>>::new()),
            clear: |data, id| {
                if let Some(vec) = data.downcast_mut::<Vec<Option<T>>>() {
                    if let Some(slot) = vec.get_mut(id as usize) {
                        *slot = None;
                    }
                }
            },
        }
    }

    fn as_vec<T: 'static>(&self) -> &Vec<Option<T>> {
        self.data.downcast_ref::<Vec<Option<T>>>().expect("组件类型不匹配")
    }

    fn as_vec_mut<T: 'static>(&mut self) -> &mut Vec<Option<T>> {
        self.data.downcast_mut::<Vec<Option<T>>>().expect("组件类型不匹配")
    }
}

/// 实体句柄（代际索引：ID 复用时代际+1，旧引用自然失效——
///
/// 此前裸 u64 引用有 ABA 风险：怪 A 死 → ID 被飞弹复用 → 怪 B 的仇恨
/// 目标一夜变成一颗飞弹。`same/valid` 做（id, gen）双校验）.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity {
    pub id: u64,
    pub gen: u32,
}

/// 简单 ECS — 按 Entity ID 索引，按 TypeId 分组存储
/// 借鉴 Blubber Engine: SoA (Array of Structures) 布局
/// 相同组件连续存储 → 缓存友好
/// 使用 Option<T> 确保 zero-cost 初始化 (无 unsafe)
/// despawn 清组件表（防 ID 复用时僵尸组件复活——2026-09 实测：飞弹复用死怪 ID 被当尸体反复结算）
pub struct SimpleEcs {
    /// 每种组件类型一个存储
    stores: HashMap<TypeId, ComponentStore>,
    /// Entity 存活标记
    alive: Vec<bool>,
    /// 空闲 ID 池 (借鉴 Specs 的 slot map)
    free_ids: Vec<u64>,
    next_id: u64,
    /// 实体 → 所持组件类型（despawn 时清理用）
    members: HashMap<u64, Vec<TypeId>>,
    /// ID → 代际（复用时 +1）
    gens: Vec<u32>,
}

impl SimpleEcs {
    pub fn new() -> Self {
        Self { stores: HashMap::new(), alive: Vec::new(), free_ids: Vec::new(), next_id: 0, members: HashMap::new(), gens: Vec::new() }
    }

    /// 创建实体，返回 ID（裸 ID 兼容口；新代码优先 `spawn_entity`）
    pub fn spawn(&mut self) -> u64 {
        self.spawn_entity().id
    }

    /// 创建实体，返回代际句柄（ID 复用时自动 +1）
    pub fn spawn_entity(&mut self) -> Entity {
        if let Some(id) = self.free_ids.pop() {
            self.alive[id as usize] = true;
            let g = self.gens[id as usize].wrapping_add(1);
            self.gens[id as usize] = g;
            Entity { id, gen: g }
        } else {
            let id = self.next_id;
            self.alive.push(true);
            self.gens.push(0);
            self.next_id += 1;
            Entity { id, gen: 0 }
        }
    }

    /// 当前代际（无记录 → 0）
    pub fn generation(&self, id: u64) -> u32 {
        self.gens.get(id as usize).copied().unwrap_or(0)
    }

    /// 当前句柄（查表/比较用）
    pub fn entity_of(&self, id: u64) -> Entity {
        Entity { id, gen: self.generation(id) }
    }

    /// 代际有效：存活且代际一致（旧引用自动失效）
    pub fn valid(&self, e: Entity) -> bool {
        self.is_alive(e.id) && self.generation(e.id) == e.gen
    }

    /// 句柄与 ID 是否指向同一代实体
    pub fn same(&self, e: Entity, id: u64) -> bool {
        e.id == id && self.valid(e)
    }

    /// 销毁实体（清组件 + 回收 ID）
    pub fn despawn(&mut self, id: u64) {
        if (id as usize) < self.alive.len() {
            self.alive[id as usize] = false;
            self.free_ids.push(id);
            // 清组件：逐存储置空（ID 复用不再继承僵尸）
            if let Some(types) = self.members.remove(&id) {
                for t in types {
                    if let Some(store) = self.stores.get_mut(&t) {
                        (store.clear)(&mut store.data, id);
                    }
                }
            }
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
        self.members.clear();
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
        // 登记成员关系（despawn 清理用）
        let entry = self.members.entry(id).or_default();
        if !entry.contains(&type_id) {
            entry.push(type_id);
        }
    }

    /// 删除组件（实体保留；无组件/死亡实体 → false，幂等）.
    /// 此前调用方只能用哨兵值模拟删除（如读条 `t<0`），现直删.
    pub fn remove<T: 'static>(&mut self, id: u64) -> bool {
        if !self.is_alive(id) {
            return false;
        }
        let type_id = TypeId::of::<T>();
        let store = match self.stores.get_mut(&type_id) {
            Some(s) => s,
            None => return false,
        };
        let vec = store.as_vec_mut::<T>();
        let had = match vec.get_mut(id as usize) {
            Some(slot) => slot.take().is_some(),
            None => false,
        };
        if had {
            if let Some(list) = self.members.get_mut(&id) {
                list.retain(|t| *t != type_id);
            }
        }
        had
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

    /// 惰性查询：拥有 A 的 `(id, &A)` 迭代器（零分配，热循环用；语义同 `query1`）.
    pub fn iter1<A: 'static>(&self) -> impl Iterator<Item = (u64, &A)> + '_ {
        let vec = self.stores.get(&TypeId::of::<A>()).map(|s| s.as_vec::<A>());
        let next_id = self.next_id;
        let alive = &self.alive;
        (0..next_id).filter_map(move |id| {
            if (id as usize) < alive.len() && alive[id as usize] {
                vec.and_then(|v| v.get(id as usize)?.as_ref()).map(|a| (id, a))
            } else {
                None
            }
        })
    }

    /// 惰性查询：拥有 A 的实体 ID 迭代器（零分配；语义同 `query1_ids`）.
    pub fn iter1_ids<A: 'static>(&self) -> impl Iterator<Item = u64> + '_ {
        self.iter1::<A>().map(|(id, _)| id)
    }

    /// 惰性查询：同时拥有 A 和 B 的迭代器（零分配；语义同 `query2`）.
    pub fn iter2<A: 'static, B: 'static>(&self) -> impl Iterator<Item = (u64, &A, &B)> + '_ {
        let a_vec = self.stores.get(&TypeId::of::<A>()).map(|s| s.as_vec::<A>());
        let b_vec = self.stores.get(&TypeId::of::<B>()).map(|s| s.as_vec::<B>());
        let next_id = self.next_id;
        let alive = &self.alive;
        (0..next_id).filter_map(move |id| {
            if (id as usize) < alive.len() && alive[id as usize] {
                match (
                    a_vec.and_then(|v| v.get(id as usize)?.as_ref()),
                    b_vec.and_then(|v| v.get(id as usize)?.as_ref()),
                ) {
                    (Some(a), Some(b)) => Some((id, a, b)),
                    _ => None,
                }
            } else {
                None
            }
        })
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

    #[test]
    fn remove_component_idempotent() {
        let mut ecs = SimpleEcs::new();
        let id = ecs.spawn();
        ecs.insert(id, 7u32);
        assert!(ecs.remove::<u32>(id));
        assert!(ecs.get::<u32>(id).is_none());
        assert!(!ecs.remove::<u32>(id)); // 二次删除 false
        assert!(!ecs.remove::<u32>(9999)); // 越界 false
        assert!(ecs.is_alive(id)); // 实体保留
    }

    #[test]
    fn despawn_clears_components_no_zombie() {
        // 回归：ID 复用不再继承僵尸组件（飞弹复用死怪 ID 被当尸体反复结算）
        let mut ecs = SimpleEcs::new();
        let id = ecs.spawn();
        ecs.insert(id, 7u32);
        ecs.insert(id, "dead");
        ecs.despawn(id);
        let id2 = ecs.spawn();
        assert_eq!(id, id2); // 复用同一 ID
        assert!(ecs.get::<u32>(id2).is_none());
        assert!(ecs.get::<&str>(id2).is_none());
        // 新实体挂新组件正常工作
        ecs.insert(id2, 42u32);
        assert_eq!(*ecs.get::<u32>(id2).unwrap(), 42);
    }

    #[test]
    fn iter_matches_query() {
        let mut ecs = SimpleEcs::new();
        let a = ecs.spawn();
        ecs.insert(a, 1u32);
        ecs.insert(a, "x");
        let b = ecs.spawn();
        ecs.insert(b, 2u32);
        ecs.despawn(b);
        let c = ecs.spawn();
        ecs.insert(c, 3u32);
        // iter1 与 query1 一致（含死亡过滤）
        let from_iter: Vec<(u64, &u32)> = ecs.iter1::<u32>().collect();
        let from_query = ecs.query1::<u32>();
        assert_eq!(from_iter.len(), from_query.len());
        assert!(from_iter.iter().all(|(id, _)| *id == a || *id == c));
        // iter2 交集
        let pairs: Vec<(u64, &u32, &&str)> = ecs.iter2::<u32, &str>().collect();
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].0, a);
        // ids
        let ids: Vec<u64> = ecs.iter1_ids::<u32>().collect();
        assert_eq!(ids.len(), 2);
    }

    #[test]
    fn generations_invalidate_stale_handles() {
        let mut ecs = SimpleEcs::new();
        let a = ecs.spawn_entity();
        assert_eq!((a.id, a.gen), (0, 0));
        ecs.despawn(a.id);
        let b = ecs.spawn_entity();
        assert_eq!(b.id, a.id); // 同一 ID
        assert_ne!(b.gen, a.gen); // 代际不同
        assert!(!ecs.valid(a)); // 旧句柄失效
        assert!(ecs.valid(b));
        assert!(ecs.same(b, b.id));
        assert!(!ecs.same(a, b.id)); // 旧句柄不再匹配
        assert_eq!(ecs.entity_of(b.id), b);
    }
}
