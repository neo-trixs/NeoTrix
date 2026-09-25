//! 对象/标签层 — 吸收 `kaplay` GameObj 组合思想.
//!
//! KAPLAY 侧：`k.add([sprite(), pos(), area(), body(), "enemy"])` ——
//! 组件自由组合 + 字符串 tag 分组（`k.get("enemy")` / `k.onCollide`）。
//! 本模块是引擎侧最小闭环：`Tags` 组件 + 无头 helpers
//!（`tag/untag/has_tag/get_by_tag`），查询走 `SimpleEcs` 现有索引，
//! 碰撞回调走 `events::EventBus`（`CollideEvent` 由调用方按 `ecs_systems`
//! 的 `collision_detect` 结果发射，不在本模块做 Broadphase）。
//!
//! 公理：标签即数据（`HashSet<String>`），无全局注册表；对象销毁时标签随
//! 组件自然失效（`is_alive` 门控），不做跨帧缓存。

use std::collections::HashSet;

use crate::ecs::SimpleEcs;

/// 标签组件 — KAPLAY `"enemy"` 字符串组的引擎等价物.
#[derive(Debug, Clone, Default)]
pub struct Tags {
    tags: HashSet<String>,
}

impl Tags {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(tag: &str) -> Self {
        let mut s = Self::new();
        s.insert(tag);
        s
    }

    pub fn insert(&mut self, tag: &str) {
        self.tags.insert(tag.to_string());
    }

    pub fn remove(&mut self, tag: &str) {
        self.tags.remove(tag);
    }

    pub fn contains(&self, tag: &str) -> bool {
        self.tags.contains(tag)
    }

    pub fn is_empty(&self) -> bool {
        self.tags.is_empty()
    }

    pub fn len(&self) -> usize {
        self.tags.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.tags.iter()
    }
}

/// 打标签（实体无 `Tags` 组件则创建；死亡实体无操作）.
pub fn tag(ecs: &mut SimpleEcs, id: u64, tag: &str) {
    if !ecs.is_alive(id) {
        return;
    }
    if let Some(t) = ecs.get_mut::<Tags>(id) {
        t.insert(tag);
    } else {
        ecs.insert(id, Tags::with(tag));
    }
}

/// 去标签（无组件/死亡实体无操作）.
pub fn untag(ecs: &mut SimpleEcs, id: u64, tag: &str) {
    if !ecs.is_alive(id) {
        return;
    }
    if let Some(t) = ecs.get_mut::<Tags>(id) {
        t.remove(tag);
    }
}

/// 是否持有标签.
pub fn has_tag(ecs: &SimpleEcs, id: u64, tag: &str) -> bool {
    if !ecs.is_alive(id) {
        return false;
    }
    ecs.get::<Tags>(id).map_or(false, |t| t.contains(tag))
}

/// 取全部持有 `tag` 的存活实体（id 升序，KAPLAY `k.get(tag)` 语义）.
pub fn get_by_tag(ecs: &SimpleEcs, tag: &str) -> Vec<u64> {
    let mut out: Vec<u64> = ecs
        .query1::<Tags>()
        .into_iter()
        .filter_map(|(id, t)| if t.contains(tag) { Some(id) } else { None })
        .collect();
    out.sort_unstable();
    out
}

/// 取同时持有 `all` 中全部标签的存活实体（多 tag 交集查询）.
pub fn get_by_all_tags(ecs: &SimpleEcs, all: &[&str]) -> Vec<u64> {
    let mut out: Vec<u64> = ecs
        .query1::<Tags>()
        .into_iter()
        .filter_map(|(id, t)| {
            if all.iter().all(|tag| t.contains(tag)) {
                Some(id)
            } else {
                None
            }
        })
        .collect();
    out.sort_unstable();
    out
}

/// 碰撞事件载荷 — 调用方按 `ecs_systems::collision_detect` 结果发射到 `EventBus`.
#[derive(Debug, Clone)]
pub struct CollideEvent {
    pub a: u64,
    pub b: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tag_creates_component_and_queries() {
        let mut ecs = SimpleEcs::new();
        let e = ecs.spawn();
        tag(&mut ecs, e, "enemy");
        tag(&mut ecs, e, "flying");
        assert!(has_tag(&ecs, e, "enemy"));
        assert_eq!(get_by_tag(&ecs, "enemy"), vec![e]);
        assert_eq!(get_by_all_tags(&ecs, &["enemy", "flying"]), vec![e]);
        assert!(get_by_all_tags(&ecs, &["enemy", "boss"]).is_empty());
    }

    #[test]
    fn untag_removes_only_named() {
        let mut ecs = SimpleEcs::new();
        let e = ecs.spawn();
        tag(&mut ecs, e, "enemy");
        tag(&mut ecs, e, "flying");
        untag(&mut ecs, e, "enemy");
        assert!(!has_tag(&ecs, e, "enemy"));
        assert!(has_tag(&ecs, e, "flying"));
        assert!(get_by_tag(&ecs, "enemy").is_empty());
    }

    #[test]
    fn dead_entity_is_invisible() {
        let mut ecs = SimpleEcs::new();
        let e = ecs.spawn();
        tag(&mut ecs, e, "enemy");
        ecs.despawn(e);
        assert!(!has_tag(&ecs, e, "enemy"));
        assert!(get_by_tag(&ecs, "enemy").is_empty());
        // 死亡实体上打标签无操作，不复活
        tag(&mut ecs, e, "boss");
        assert!(!ecs.is_alive(e));
    }

    #[test]
    fn multi_entity_sorted() {
        let mut ecs = SimpleEcs::new();
        let b = ecs.spawn();
        let a = ecs.spawn();
        tag(&mut ecs, b, "coin");
        tag(&mut ecs, a, "coin");
        let got = get_by_tag(&ecs, "coin");
        assert_eq!(got, vec![b.min(a), b.max(a)]);
    }
}
