/// NT-GAME 物理碰撞检测系统
///
/// 包含空间哈希加速结构和碰撞检测系统
/// 支持 AABB 和圆形碰撞检测，碰撞层/掩码过滤
use std::collections::{HashMap, HashSet};

use super::components::{Collider, ColliderShape, Rect, Transform2D, Vec2};
use crate::l5_cognition::nt_mind::nt_game::events::CollisionEvent;

// ═══════════════════════════════════════════════════════════════════
// 空间哈希网格 — 加速碰撞查询
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct SpatialHash {
    cell_size: f32,
    cells: HashMap<(i32, i32), Vec<u64>>,
    entity_cells: HashMap<u64, Vec<(i32, i32)>>,
    entity_aabbs: HashMap<u64, Rect>,
}

impl SpatialHash {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: HashMap::new(),
            entity_cells: HashMap::new(),
            entity_aabbs: HashMap::new(),
        }
    }

    fn world_to_cell(&self, x: f32, y: f32) -> (i32, i32) {
        (
            (x / self.cell_size).floor() as i32,
            (y / self.cell_size).floor() as i32,
        )
    }

    fn get_covering_cells(&self, aabb: Rect) -> Vec<(i32, i32)> {
        let min_cell = self.world_to_cell(aabb.x, aabb.y);
        let max_cell = self.world_to_cell(aabb.x + aabb.w, aabb.y + aabb.h);
        let mut cells = Vec::new();
        for cy in min_cell.1..=max_cell.1 {
            for cx in min_cell.0..=max_cell.0 {
                cells.push((cx, cy));
            }
        }
        cells
    }

    pub fn insert(&mut self, entity_id: u64, aabb: Rect) {
        self.remove(entity_id);
        self.entity_aabbs.insert(entity_id, aabb);
        let covering_cells = self.get_covering_cells(aabb);
        self.entity_cells.insert(entity_id, covering_cells.clone());
        for cell in covering_cells {
            self.cells.entry(cell).or_default().push(entity_id);
        }
    }

    pub fn remove(&mut self, entity_id: u64) {
        if let Some(cells) = self.entity_cells.remove(&entity_id) {
            for cell in cells {
                if let Some(entities) = self.cells.get_mut(&cell) {
                    entities.retain(|&id| id != entity_id);
                    if entities.is_empty() {
                        self.cells.remove(&cell);
                    }
                }
            }
        }
        self.entity_aabbs.remove(&entity_id);
    }

    pub fn query(&self, rect: Rect) -> Vec<u64> {
        let query_cells = self.get_covering_cells(rect);
        let mut candidates = HashSet::new();
        for cell in query_cells {
            if let Some(entities) = self.cells.get(&cell) {
                for &entity_id in entities {
                    candidates.insert(entity_id);
                }
            }
        }
        let mut result = Vec::new();
        for entity_id in candidates {
            if let Some(aabb) = self.entity_aabbs.get(&entity_id) {
                if rect.intersects(*aabb) {
                    result.push(entity_id);
                }
            }
        }
        result
    }

    pub fn clear(&mut self) {
        self.cells.clear();
        self.entity_cells.clear();
        self.entity_aabbs.clear();
    }

    pub fn entity_count(&self) -> usize {
        self.entity_aabbs.len()
    }

    pub fn cell_size(&self) -> f32 {
        self.cell_size
    }
}

// ═══════════════════════════════════════════════════════════════════
// 碰撞检测系统
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct EntityData {
    pub id: u64,
    pub transform: Transform2D,
    pub collider: Collider,
}

pub struct CollisionSystem {
    spatial_hash: SpatialHash,
}

impl CollisionSystem {
    pub fn new(cell_size: f32) -> Self {
        Self {
            spatial_hash: SpatialHash::new(cell_size),
        }
    }

    pub fn check_collisions(&mut self, entities: &[EntityData]) -> Vec<CollisionEvent> {
        self.spatial_hash.clear();
        for entity in entities {
            let aabb = entity.collider.world_aabb(entity.transform.position);
            self.spatial_hash.insert(entity.id, aabb);
        }

        let mut events = Vec::new();
        let mut checked_pairs: HashSet<(u64, u64)> = HashSet::new();

        for entity in entities {
            let aabb = entity.collider.world_aabb(entity.transform.position);
            let candidates = self.spatial_hash.query(aabb);

            for &other_id in &candidates {
                if other_id == entity.id {
                    continue;
                }

                let pair = if entity.id < other_id {
                    (entity.id, other_id)
                } else {
                    (other_id, entity.id)
                };
                if !checked_pairs.insert(pair) {
                    continue;
                }

                if let Some(other) = entities.iter().find(|e| e.id == other_id) {
                    if !layers_compatible(
                        entity.collider.layer,
                        entity.collider.mask,
                        other.collider.layer,
                        other.collider.mask,
                    ) && !layers_compatible(
                        other.collider.layer,
                        other.collider.mask,
                        entity.collider.layer,
                        entity.collider.mask,
                    ) {
                        continue;
                    }

                    if let Some(collision_point) = detect_collision(
                        &entity.transform,
                        &entity.collider,
                        &other.transform,
                        &other.collider,
                    ) {
                        events.push(CollisionEvent {
                            entity_a: entity.id,
                            entity_b: other_id,
                            x: collision_point.x as f64,
                            y: collision_point.y as f64,
                        });
                    }
                }
            }
        }

        events
    }

    pub fn spatial_hash(&self) -> &SpatialHash {
        &self.spatial_hash
    }

    pub fn spatial_hash_mut(&mut self) -> &mut SpatialHash {
        &mut self.spatial_hash
    }
}

// ═══════════════════════════════════════════════════════════════════
// 碰撞检测内部函数
// ═══════════════════════════════════════════════════════════════════

fn layers_compatible(a_layer: u32, a_mask: u32, b_layer: u32, b_mask: u32) -> bool {
    (a_layer & b_mask) != 0 && (b_layer & a_mask) != 0
}

fn detect_collision(
    transform_a: &Transform2D,
    collider_a: &Collider,
    transform_b: &Transform2D,
    collider_b: &Collider,
) -> Option<Vec2> {
    let pos_a = transform_a.position + collider_a.offset;
    let pos_b = transform_b.position + collider_b.offset;

    match (&collider_a.shape, &collider_b.shape) {
        (ColliderShape::Rectangle { size: size_a }, ColliderShape::Rectangle { size: size_b }) => {
            let aabb_a = Rect::new(
                pos_a.x - size_a.x / 2.0,
                pos_a.y - size_a.y / 2.0,
                size_a.x,
                size_a.y,
            );
            let aabb_b = Rect::new(
                pos_b.x - size_b.x / 2.0,
                pos_b.y - size_b.y / 2.0,
                size_b.x,
                size_b.y,
            );
            if aabb_a.intersects(aabb_b) {
                let ca = aabb_a.center();
                let cb = aabb_b.center();
                Some(Vec2::new((ca.x + cb.x) / 2.0, (ca.y + cb.y) / 2.0))
            } else {
                None
            }
        }
        (ColliderShape::Circle { radius: r_a }, ColliderShape::Circle { radius: r_b }) => {
            let distance = pos_a.distance_to(pos_b);
            if distance < r_a + r_b {
                Some(Vec2::new(
                    (pos_a.x + pos_b.x) / 2.0,
                    (pos_a.y + pos_b.y) / 2.0,
                ))
            } else {
                None
            }
        }
        (ColliderShape::Rectangle { size }, ColliderShape::Circle { radius }) => {
            let rect = Rect::new(
                pos_a.x - size.x / 2.0,
                pos_a.y - size.y / 2.0,
                size.x,
                size.y,
            );
            rect_circle_collision(&rect, pos_b, *radius)
        }
        (ColliderShape::Circle { radius }, ColliderShape::Rectangle { size }) => {
            let rect = Rect::new(
                pos_b.x - size.x / 2.0,
                pos_b.y - size.y / 2.0,
                size.x,
                size.y,
            );
            rect_circle_collision(&rect, pos_a, *radius)
        }
    }
}

fn rect_circle_collision(rect: &Rect, circle_center: Vec2, circle_radius: f32) -> Option<Vec2> {
    let closest_x = circle_center.x.clamp(rect.x, rect.x + rect.w);
    let closest_y = circle_center.y.clamp(rect.y, rect.y + rect.h);
    let closest = Vec2::new(closest_x, closest_y);
    let distance = closest.distance_to(circle_center);
    if distance < circle_radius {
        Some(Vec2::new(
            (rect.center().x + circle_center.x) / 2.0,
            (rect.center().y + circle_center.y) / 2.0,
        ))
    } else {
        None
    }
}

// ═══════════════════════════════════════════════════════════════════
// 测试
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── SpatialHash 测试 ──

    #[test]
    fn test_spatial_hash_insert_and_query() {
        let mut sh = SpatialHash::new(32.0);
        sh.insert(1, Rect::new(0.0, 0.0, 10.0, 10.0));
        sh.insert(2, Rect::new(50.0, 50.0, 10.0, 10.0));
        assert_eq!(sh.entity_count(), 2);

        let found = sh.query(Rect::new(0.0, 0.0, 15.0, 15.0));
        assert!(found.contains(&1));
        assert!(!found.contains(&2));
    }

    #[test]
    fn test_spatial_hash_remove() {
        let mut sh = SpatialHash::new(32.0);
        sh.insert(1, Rect::new(0.0, 0.0, 10.0, 10.0));
        sh.insert(2, Rect::new(50.0, 50.0, 10.0, 10.0));
        sh.remove(1);
        assert_eq!(sh.entity_count(), 1);
        let found = sh.query(Rect::new(0.0, 0.0, 100.0, 100.0));
        assert!(!found.contains(&1));
        assert!(found.contains(&2));
    }

    #[test]
    fn test_spatial_hash_clear() {
        let mut sh = SpatialHash::new(32.0);
        sh.insert(1, Rect::new(0.0, 0.0, 10.0, 10.0));
        sh.insert(2, Rect::new(50.0, 50.0, 10.0, 10.0));
        sh.clear();
        assert_eq!(sh.entity_count(), 0);
    }

    #[test]
    fn test_spatial_hash_large_entity() {
        let mut sh = SpatialHash::new(16.0);
        sh.insert(1, Rect::new(0.0, 0.0, 50.0, 50.0));
        assert_eq!(sh.entity_count(), 1);
        let found = sh.query(Rect::new(40.0, 40.0, 10.0, 10.0));
        assert!(found.contains(&1));
    }

    // ── AABB vs AABB 碰撞测试 ──

    #[test]
    fn test_rect_vs_rect_collision() {
        let mut sys = CollisionSystem::new(32.0);
        let entities = vec![
            EntityData {
                id: 1,
                transform: Transform2D::new(Vec2::new(0.0, 0.0)),
                collider: Collider::rectangle(20.0, 20.0),
            },
            EntityData {
                id: 2,
                transform: Transform2D::new(Vec2::new(10.0, 0.0)),
                collider: Collider::rectangle(20.0, 20.0),
            },
        ];
        let events = sys.check_collisions(&entities);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].entity_a, 1);
        assert_eq!(events[0].entity_b, 2);
    }

    #[test]
    fn test_no_rect_collision_when_separated() {
        let mut sys = CollisionSystem::new(32.0);
        let entities = vec![
            EntityData {
                id: 1,
                transform: Transform2D::new(Vec2::new(0.0, 0.0)),
                collider: Collider::rectangle(10.0, 10.0),
            },
            EntityData {
                id: 2,
                transform: Transform2D::new(Vec2::new(100.0, 100.0)),
                collider: Collider::rectangle(10.0, 10.0),
            },
        ];
        let events = sys.check_collisions(&entities);
        assert!(events.is_empty());
    }

    // ── Circle vs Circle 碰撞测试 ──

    #[test]
    fn test_circle_vs_circle_collision() {
        let mut sys = CollisionSystem::new(32.0);
        let entities = vec![
            EntityData {
                id: 1,
                transform: Transform2D::new(Vec2::new(0.0, 0.0)),
                collider: Collider::circle(10.0),
            },
            EntityData {
                id: 2,
                transform: Transform2D::new(Vec2::new(15.0, 0.0)),
                collider: Collider::circle(10.0),
            },
        ];
        let events = sys.check_collisions(&entities);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_no_circle_collision_when_separated() {
        let mut sys = CollisionSystem::new(32.0);
        let entities = vec![
            EntityData {
                id: 1,
                transform: Transform2D::new(Vec2::new(0.0, 0.0)),
                collider: Collider::circle(5.0),
            },
            EntityData {
                id: 2,
                transform: Transform2D::new(Vec2::new(100.0, 0.0)),
                collider: Collider::circle(5.0),
            },
        ];
        let events = sys.check_collisions(&entities);
        assert!(events.is_empty());
    }

    // ── Rect vs Circle 碰撞测试 ──

    #[test]
    fn test_rect_vs_circle_collision() {
        let mut sys = CollisionSystem::new(32.0);
        let entities = vec![
            EntityData {
                id: 1,
                transform: Transform2D::new(Vec2::new(0.0, 0.0)),
                collider: Collider::rectangle(20.0, 20.0),
            },
            EntityData {
                id: 2,
                transform: Transform2D::new(Vec2::new(15.0, 0.0)),
                collider: Collider::circle(10.0),
            },
        ];
        let events = sys.check_collisions(&entities);
        assert_eq!(events.len(), 1);
    }

    // ── 碰撞层/掩码过滤测试 ──

    #[test]
    fn test_layer_mask_filtering() {
        let mut sys = CollisionSystem::new(32.0);
        let mut c1 = Collider::rectangle(20.0, 20.0);
        c1.layer = 1;
        c1.mask = 2;
        let mut c2 = Collider::rectangle(20.0, 20.0);
        c2.layer = 4;
        c2.mask = 8;

        let entities = vec![
            EntityData {
                id: 1,
                transform: Transform2D::new(Vec2::new(0.0, 0.0)),
                collider: c1,
            },
            EntityData {
                id: 2,
                transform: Transform2D::new(Vec2::new(5.0, 0.0)),
                collider: c2,
            },
        ];
        let events = sys.check_collisions(&entities);
        assert!(events.is_empty(), "masks should prevent collision");
    }

    #[test]
    fn test_layer_mask_compatible() {
        let mut sys = CollisionSystem::new(32.0);
        let mut c1 = Collider::rectangle(20.0, 20.0);
        c1.layer = 1;
        c1.mask = 2;
        let mut c2 = Collider::rectangle(20.0, 20.0);
        c2.layer = 2;
        c2.mask = 1;

        let entities = vec![
            EntityData {
                id: 1,
                transform: Transform2D::new(Vec2::new(0.0, 0.0)),
                collider: c1,
            },
            EntityData {
                id: 2,
                transform: Transform2D::new(Vec2::new(5.0, 0.0)),
                collider: c2,
            },
        ];
        let events = sys.check_collisions(&entities);
        assert_eq!(events.len(), 1, "compatible layers should collide");
    }

    // ── Sensor 仍产生碰撞事件 ──

    #[test]
    fn test_sensor_collision_emits_event() {
        let mut sys = CollisionSystem::new(32.0);
        let entities = vec![
            EntityData {
                id: 1,
                transform: Transform2D::new(Vec2::new(0.0, 0.0)),
                collider: Collider::rectangle(20.0, 20.0).sensor(),
            },
            EntityData {
                id: 2,
                transform: Transform2D::new(Vec2::new(5.0, 0.0)),
                collider: Collider::rectangle(20.0, 20.0),
            },
        ];
        let events = sys.check_collisions(&entities);
        assert_eq!(events.len(), 1, "sensor should still emit collision event");
    }

    // ── 碰撞点坐标正确性 ──

    #[test]
    fn test_collision_event_coords() {
        let mut sys = CollisionSystem::new(32.0);
        let entities = vec![
            EntityData {
                id: 10,
                transform: Transform2D::new(Vec2::new(0.0, 0.0)),
                collider: Collider::circle(10.0),
            },
            EntityData {
                id: 20,
                transform: Transform2D::new(Vec2::new(10.0, 0.0)),
                collider: Collider::circle(10.0),
            },
        ];
        let events = sys.check_collisions(&entities);
        assert_eq!(events.len(), 1);
        let e = &events[0];
        assert_eq!(e.entity_a, 10);
        assert_eq!(e.entity_b, 20);
        assert!((e.x - 5.0).abs() < 0.01);
        assert!((e.y - 0.0).abs() < 0.01);
    }

    // ── 多实体混合检测 ──

    #[test]
    fn test_multiple_entities() {
        let mut sys = CollisionSystem::new(32.0);
        let entities = vec![
            EntityData {
                id: 1,
                transform: Transform2D::new(Vec2::new(0.0, 0.0)),
                collider: Collider::rectangle(20.0, 20.0),
            },
            EntityData {
                id: 2,
                transform: Transform2D::new(Vec2::new(5.0, 0.0)),
                collider: Collider::rectangle(20.0, 20.0),
            },
            EntityData {
                id: 3,
                transform: Transform2D::new(Vec2::new(500.0, 500.0)),
                collider: Collider::rectangle(20.0, 20.0),
            },
        ];
        let events = sys.check_collisions(&entities);
        assert_eq!(events.len(), 1);
    }

    // ── offset 碰撞器偏移测试 ──

    #[test]
    fn test_collider_offset() {
        let mut sys = CollisionSystem::new(32.0);
        let mut c1 = Collider::rectangle(10.0, 10.0);
        c1.offset = Vec2::new(10.0, 0.0);
        let entities = vec![
            EntityData {
                id: 1,
                transform: Transform2D::new(Vec2::new(0.0, 0.0)),
                collider: c1,
            },
            EntityData {
                id: 2,
                // 2026-09-27: 原 20.0 → 碰撞盒 15..25 与 5..15 仅边缘相接,
                // 按 test_adjacent_no_overlap 的既定语义属"不相交"; 移至 18.0
                // 形成 3 单位真实重叠, 才是本例要验的 offset 场景
                transform: Transform2D::new(Vec2::new(18.0, 0.0)),
                collider: Collider::rectangle(10.0, 10.0),
            },
        ];
        let events = sys.check_collisions(&entities);
        assert_eq!(events.len(), 1, "offset collider should collide");
    }

    // ── 边界接触测试 ──

    #[test]
    fn test_adjacent_no_overlap() {
        let mut sys = CollisionSystem::new(32.0);
        let entities = vec![
            EntityData {
                id: 1,
                transform: Transform2D::new(Vec2::new(0.0, 0.0)),
                collider: Collider::rectangle(10.0, 10.0),
            },
            EntityData {
                id: 2,
                transform: Transform2D::new(Vec2::new(10.0, 0.0)),
                collider: Collider::rectangle(10.0, 10.0),
            },
        ];
        let events = sys.check_collisions(&entities);
        assert!(
            events.is_empty(),
            "adjacent rects touching edge should not overlap"
        );
    }

    // ── 单实体无碰撞 ──

    #[test]
    fn test_single_entity_no_collision() {
        let mut sys = CollisionSystem::new(32.0);
        let entities = vec![EntityData {
            id: 1,
            transform: Transform2D::new(Vec2::new(0.0, 0.0)),
            collider: Collider::rectangle(20.0, 20.0),
        }];
        let events = sys.check_collisions(&entities);
        assert!(events.is_empty());
    }

    // ── 空实体列表 ──

    #[test]
    fn test_empty_entities() {
        let mut sys = CollisionSystem::new(32.0);
        let events = sys.check_collisions(&[]);
        assert!(events.is_empty());
    }

    // ── SpatialHash 区域查询覆盖多单元格 ──

    #[test]
    fn test_query_crosses_cell_boundary() {
        let mut sh = SpatialHash::new(16.0);
        sh.insert(1, Rect::new(10.0, 10.0, 30.0, 30.0));
        let found = sh.query(Rect::new(0.0, 0.0, 15.0, 15.0));
        assert!(found.contains(&1));
    }
}
