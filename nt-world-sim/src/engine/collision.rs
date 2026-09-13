use super::map::{CollisionType, TileMap};
use super::renderer::Vec2;

// ---------------------------------------------------------------------------
// AABB
// ---------------------------------------------------------------------------

/// Axis-aligned bounding box.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AABB {
    pub min: Vec2,
    pub max: Vec2,
}

impl AABB {
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    /// Create from center + half-extents.
    pub fn from_center_half(center: Vec2, half: Vec2) -> Self {
        Self {
            min: Vec2::new(center.x - half.x, center.y - half.y),
            max: Vec2::new(center.x + half.x, center.y + half.y),
        }
    }

    /// Create from position (top-left) + size.
    pub fn from_pos_size(pos: Vec2, size: Vec2) -> Self {
        Self {
            min: pos,
            max: Vec2::new(pos.x + size.x, pos.y + size.y),
        }
    }

    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    pub fn center(&self) -> Vec2 {
        Vec2::new(
            (self.min.x + self.max.x) * 0.5,
            (self.min.y + self.max.y) * 0.5,
        )
    }

    pub fn half_extents(&self) -> Vec2 {
        Vec2::new(self.width() * 0.5, self.height() * 0.5)
    }

    /// Check if this AABB overlaps another.
    pub fn overlaps(&self, other: &AABB) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
    }

    /// Check if a point is inside this AABB.
    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }

    /// Compute overlap displacement to push `self` out of `other`.
    /// Returns None if no overlap.
    pub fn overlap_displacement(&self, other: &AABB) -> Option<Vec2> {
        if !self.overlaps(other) {
            return None;
        }
        let dx1 = other.max.x - self.min.x;
        let dx2 = self.max.x - other.min.x;
        let dy1 = other.max.y - self.min.y;
        let dy2 = self.max.y - other.min.y;

        let dx = if dx1 < dx2 { dx1 } else { -dx2 };
        let dy = if dy1 < dy2 { dy1 } else { -dy2 };

        if dx.abs() < dy.abs() {
            Some(Vec2::new(dx, 0.0))
        } else {
            Some(Vec2::new(0.0, dy))
        }
    }
}

// ---------------------------------------------------------------------------
// TileCollisionResult
// ---------------------------------------------------------------------------

/// Result of a tile collision check.
#[derive(Debug, Clone)]
pub struct TileCollisionResult {
    /// The tile's collision type.
    pub collision: CollisionType,
    /// The tile position (tile coords).
    pub tile_pos: (usize, usize),
    /// Overlap displacement vector to push entity out.
    pub displacement: Vec2,
    /// Damage from the tile.
    pub damage: i32,
    /// Speed multiplier from the tile.
    pub speed: f32,
}

// ---------------------------------------------------------------------------
// CollisionSystem
// ---------------------------------------------------------------------------

/// Tile-based collision detection and resolution system.
#[allow(dead_code)]
pub struct TileCollisionSystem {
    /// Temporary buffer for collision results.
    results: Vec<TileCollisionResult>,
}

impl TileCollisionSystem {
    pub fn new() -> Self {
        Self { results: Vec::with_capacity(8) }
    }

    /// Get the AABB for a tile at (tx, ty) in world coordinates.
    pub fn tile_aabb(tilemap: &TileMap, tx: usize, ty: usize) -> AABB {
        let ts = tilemap.tile_size;
        AABB::from_pos_size(
            Vec2::new(tx as f32 * ts, ty as f32 * ts),
            Vec2::new(ts, ts),
        )
    }

    /// Get all tiles that overlap a world-space AABB.
    /// Returns (tile_x, tile_y) pairs.
    pub fn overlapping_tiles(tilemap: &TileMap, aabb: &AABB) -> Vec<(usize, usize)> {
        let ts = tilemap.tile_size;
        let min_tx = (aabb.min.x / ts).floor().max(0.0) as usize;
        let min_ty = (aabb.min.y / ts).floor().max(0.0) as usize;
        let max_tx = ((aabb.max.x / ts).ceil() as usize).min(tilemap.width.saturating_sub(1));
        let max_ty = ((aabb.max.y / ts).ceil() as usize).min(tilemap.height.saturating_sub(1));

        let mut tiles = Vec::new();
        for ty in min_ty..=max_ty {
            for tx in min_tx..=max_tx {
                tiles.push((tx, ty));
            }
        }
        tiles
    }

    /// Check which tiles an entity AABB collides with.
    /// Returns collision results for all solid/damaging tiles.
    pub fn check_entity_collisions(
        &self,
        tilemap: &TileMap,
        entity_aabb: &AABB,
    ) -> Vec<TileCollisionResult> {
        let mut results = Vec::new();
        let tiles = Self::overlapping_tiles(tilemap, entity_aabb);

        for (tx, ty) in tiles {
            for layer in &tilemap.layers {
                if !layer.collision_enabled || !layer.visible {
                    continue;
                }
                let tile = match layer.get_tile(tx, ty) {
                    Some(t) => t,
                    None => continue,
                };

                match tile.collision {
                    CollisionType::None => continue,
                    _ => {}
                }

                let tile_aabb = Self::tile_aabb(tilemap, tx, ty);
                if let Some(displacement) = entity_aabb.overlap_displacement(&tile_aabb) {
                    results.push(TileCollisionResult {
                        collision: tile.collision,
                        tile_pos: (tx, ty),
                        displacement,
                        damage: tile.damage,
                        speed: tile.speed,
                    });
                }
            }
        }

        results
    }

    /// Resolve entity movement against tilemap collisions.
    /// Takes desired position, returns corrected position.
    pub fn resolve_movement(
        &self,
        tilemap: &TileMap,
        current_pos: Vec2,
        desired_pos: Vec2,
        entity_size: Vec2,
    ) -> (Vec2, Vec<TileCollisionResult>) {
        let half = Vec2::new(entity_size.x * 0.5, entity_size.y * 0.5);
        let mut resolved = desired_pos;
        let mut all_results = Vec::new();

        // Try X axis first
        let aabb_x = AABB::from_center_half(
            Vec2::new(resolved.x, current_pos.y),
            half,
        );
        let collisions_x = self.check_entity_collisions(tilemap, &aabb_x);
        for col in &collisions_x {
            if col.collision == CollisionType::Solid {
                resolved.x += col.displacement.x;
                all_results.push(col.clone());
            }
        }

        // Then Y axis
        let aabb_y = AABB::from_center_half(
            Vec2::new(resolved.x, resolved.y),
            half,
        );
        let collisions_y = self.check_entity_collisions(tilemap, &aabb_y);
        for col in &collisions_y {
            match col.collision {
                CollisionType::Solid => {
                    // Compute Y-only displacement to avoid axis mixing
                    let tile_aabb = Self::tile_aabb(tilemap, col.tile_pos.0, col.tile_pos.1);
                    if aabb_y.overlaps(&tile_aabb) {
                        let dy1 = tile_aabb.max.y - aabb_y.min.y;
                        let dy2 = aabb_y.max.y - tile_aabb.min.y;
                        let dy = if dy1 < dy2 { -dy1 } else { dy2 };
                        resolved.y += dy;
                    }
                    all_results.push(col.clone());
                }
                CollisionType::OneWay => {
                    // Only resolve if entity is falling onto the platform
                    if col.displacement.y < 0.0 && current_pos.y <=
                        tilemap.tile_size * col.tile_pos.1 as f32 - half.y {
                        resolved.y = tilemap.tile_size * col.tile_pos.1 as f32 - half.y;
                        all_results.push(col.clone());
                    }
                }
                _ => {}
            }
        }

        (resolved, all_results)
    }

    /// Push an entity out of any solid tile it overlaps.
    /// Used for spawn placement and teleportation.
    pub fn push_out_of_solids(
        &self,
        tilemap: &TileMap,
        pos: Vec2,
        entity_size: Vec2,
    ) -> Vec2 {
        let half = Vec2::new(entity_size.x * 0.5, entity_size.y * 0.5);
        let mut result = pos;

        // Iterative resolution (max 4 iterations to prevent infinite loops)
        for _ in 0..4 {
            let aabb = AABB::from_center_half(result, half);
            let collisions = self.check_entity_collisions(tilemap, &aabb);
            if collisions.is_empty() {
                break;
            }

            // Find the smallest displacement
            let mut best_disp = Vec2::new(0.0, 0.0);
            let mut best_dist = f32::MAX;
            for col in &collisions {
                if col.collision == CollisionType::Solid {
                    let dist = col.displacement.x.abs() + col.displacement.y.abs();
                    if dist < best_dist {
                        best_dist = dist;
                        best_disp = col.displacement;
                    }
                }
            }

            if best_dist == f32::MAX {
                break;
            }
            result.x += best_disp.x;
            result.y += best_disp.y;
        }

        result
    }

    /// Check if a line segment from `start` to `end` hits any solid tiles.
    /// Returns the first collision hit (if any) with distance.
    pub fn raycast_tilemap(
        &self,
        tilemap: &TileMap,
        start: Vec2,
        end: Vec2,
    ) -> Option<(Vec2, f32, (usize, usize))> {
        let dir = Vec2::new(end.x - start.x, end.y - start.y);
        let len = (dir.x * dir.x + dir.y * dir.y).sqrt();
        if len < 0.001 {
            return None;
        }

        let ts = tilemap.tile_size;
        let step_x = if dir.x > 0.0 { 1.0 } else { -1.0 };
        let step_y = if dir.y > 0.0 { 1.0 } else { -1.0 };

        let mut t = 0.0f32;
        let t_step_x = if dir.x.abs() > 0.001 { ts / dir.x.abs() } else { f32::MAX };
        let t_step_y = if dir.y.abs() > 0.001 { ts / dir.y.abs() } else { f32::MAX };

        let mut tx = (start.x / ts).floor() as i32;
        let mut ty = (start.y / ts).floor() as i32;

        let mut t_max_x = if dir.x > 0.0 {
            ((tx as f32 + 1.0) * ts - start.x) / dir.x
        } else if dir.x < 0.0 {
            (start.x - tx as f32 * ts) / (-dir.x)
        } else {
            f32::MAX
        };
        let mut t_max_y = if dir.y > 0.0 {
            ((ty as f32 + 1.0) * ts - start.y) / dir.y
        } else if dir.y < 0.0 {
            (start.y - ty as f32 * ts) / (-dir.y)
        } else {
            f32::MAX
        };

        loop {
            // Check current tile
            if tx >= 0 && ty >= 0 && (tx as usize) < tilemap.width && (ty as usize) < tilemap.height {
                for layer in &tilemap.layers {
                    if !layer.collision_enabled { continue; }
                    if let Some(tile) = layer.get_tile(tx as usize, ty as usize) {
                        if tile.collision == CollisionType::Solid {
                            let hit_point = Vec2::new(start.x + dir.x * t, start.y + dir.y * t);
                            return Some((hit_point, t, (tx as usize, ty as usize)));
                        }
                    }
                }
            }

            // Step to next tile boundary
            if t_max_x < t_max_y {
                t = t_max_x;
                tx += step_x as i32;
                t_max_x += t_step_x;
            } else {
                t = t_max_y;
                ty += step_y as i32;
                t_max_y += t_step_y;
            }

            if t > len || tx < 0 || ty < 0 || (tx as usize) >= tilemap.width || (ty as usize) >= tilemap.height {
                break;
            }
        }

        None
    }
}

impl Default for TileCollisionSystem {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::map::{MapLayer, Tile};

    fn make_test_tilemap() -> TileMap {
        let mut map = TileMap::new(8, 8, 16.0);
        let mut layer = MapLayer::new("ground", 8, 8);

        // Create a wall at row 4
        for x in 0..8 {
            layer.set_tile(x, 4, Tile::new(1).with_collision(CollisionType::Solid));
        }
        // Create a water tile
        layer.set_tile(3, 3, Tile::new(2).with_collision(CollisionType::Water).with_speed(0.5));

        // Create a one-way platform
        layer.set_tile(5, 6, Tile::new(3).with_collision(CollisionType::OneWay));

        map.add_layer(layer);
        map
    }

    #[test]
    fn test_aabb_overlap() {
        let a = AABB::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let b = AABB::new(Vec2::new(5.0, 5.0), Vec2::new(15.0, 15.0));
        assert!(a.overlaps(&b));

        let c = AABB::new(Vec2::new(20.0, 20.0), Vec2::new(30.0, 30.0));
        assert!(!a.overlaps(&c));
    }

    #[test]
    fn test_aabb_contains() {
        let a = AABB::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        assert!(a.contains(Vec2::new(5.0, 5.0)));
        assert!(!a.contains(Vec2::new(15.0, 5.0)));
    }

    #[test]
    fn test_aabb_displacement() {
        let a = AABB::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let b = AABB::new(Vec2::new(8.0, 0.0), Vec2::new(18.0, 10.0));
        let d = a.overlap_displacement(&b).unwrap();
        assert!(d.x.abs() > 0.0 || d.y.abs() > 0.0);
    }

    #[test]
    fn test_overlapping_tiles() {
        let map = make_test_tilemap();
        let aabb = AABB::new(Vec2::new(10.0, 10.0), Vec2::new(50.0, 50.0));
        let tiles = TileCollisionSystem::overlapping_tiles(&map, &aabb);
        assert!(!tiles.is_empty());
        // Should include tiles covering the range
        assert!(tiles.iter().any(|&(x, y)| x == 0 && y == 0));
    }

    #[test]
    fn test_entity_collision_detection() {
        let map = make_test_tilemap();
        let system = TileCollisionSystem::new();

        // Entity near the wall at y=4 (world y=64..80)
        let entity = AABB::from_center_half(
            Vec2::new(32.0, 72.0), // center at tile (2, 4)
            Vec2::new(8.0, 8.0),
        );
        let results = system.check_entity_collisions(&map, &entity);
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.collision == CollisionType::Solid));
    }

    #[test]
    fn test_resolve_movement() {
        let map = make_test_tilemap();
        let system = TileCollisionSystem::new();

        // Entity tries to move into solid wall
        let current = Vec2::new(32.0, 56.0); // just above wall
        let desired = Vec2::new(32.0, 72.0); // into wall
        let (resolved, _) = system.resolve_movement(&map, current, desired, Vec2::new(16.0, 16.0));
        // Should be pushed back above the wall
        assert!(resolved.y < 72.0);
    }

    #[test]
    fn test_push_out_of_solids() {
        let map = make_test_tilemap();
        let system = TileCollisionSystem::new();

        // Entity spawned inside a wall
        let pos = system.push_out_of_solids(&map, Vec2::new(32.0, 72.0), Vec2::new(16.0, 16.0));
        // Should be pushed out
        let aabb = AABB::from_center_half(pos, Vec2::new(8.0, 8.0));
        let collisions = system.check_entity_collisions(&map, &aabb);
        assert!(collisions.is_empty() || collisions.iter().all(|c| c.collision != CollisionType::Solid));
    }

    #[test]
    fn test_tile_aabb() {
        let map = make_test_tilemap();
        let aabb = TileCollisionSystem::tile_aabb(&map, 3, 2);
        assert!((aabb.min.x - 48.0).abs() < 0.01);
        assert!((aabb.min.y - 32.0).abs() < 0.01);
        assert!((aabb.width() - 16.0).abs() < 0.01);
    }

    #[test]
    fn test_raycast_tilemap() {
        let map = make_test_tilemap();
        let system = TileCollisionSystem::new();

        // Ray from top to bottom, should hit wall at y=4
        let start = Vec2::new(32.0, 0.0);
        let end = Vec2::new(32.0, 128.0);
        let hit = system.raycast_tilemap(&map, start, end);
        assert!(hit.is_some());
        let (_, _, (_tx, ty)) = hit.unwrap();
        assert_eq!(ty, 4);
    }

    #[test]
    fn test_raycast_miss() {
        let map = make_test_tilemap();
        let system = TileCollisionSystem::new();

        // Ray that doesn't hit any solid
        let start = Vec2::new(0.0, 0.0);
        let end = Vec2::new(16.0, 0.0); // horizontal, stays in row 0
        let hit = system.raycast_tilemap(&map, start, end);
        assert!(hit.is_none());
    }
}
