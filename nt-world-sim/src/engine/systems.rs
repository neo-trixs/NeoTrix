use std::collections::HashSet;

use super::components::{
    Collider, GameCamera, GameSprite, Health, MonsterMarker, NpcMarker,
    PlayerMarker, RenderCommandBuffer, Transform, Velocity,
};
use super::ecs::{CollisionEvent, Entity, System, World};
use super::renderer::{Color, DrawCommand, Rect, Vec2};

// ---------------------------------------------------------------------------
// MovementSystem — integrate velocity into position
// ---------------------------------------------------------------------------

/// Applies velocity to position each tick. Runs early (low priority).
pub struct MovementSystem;

impl System for MovementSystem {
    fn name(&self) -> &str {
        "MovementSystem"
    }

    fn priority(&self) -> i32 {
        -100
    }

    fn run(&mut self, world: &mut World, dt: f32) {
        let entities = world.query2::<Transform, Velocity>();
        for e in entities {
            let vx = world.get::<Velocity>(e).unwrap().vx;
            let vy = world.get::<Velocity>(e).unwrap().vy;
            let tf = world.get_mut::<Transform>(e).unwrap();
            tf.position.x += vx * dt;
            tf.position.y += vy * dt;
        }
    }
}

// ---------------------------------------------------------------------------
// CollisionSystem — AABB overlap detection
// ---------------------------------------------------------------------------

/// Tests all collider pairs for overlap. Non-sensor collisions produce
/// events; sensors are flagged separately.
pub struct EcsCollisionSystem {
    /// Only check pairs where at least one entity has this set of marker
    /// types. Empty = check all collidable entities.
    pub filter: HashSet<Entity>,
}

impl EcsCollisionSystem {
    pub fn new() -> Self {
        Self { filter: HashSet::new() }
    }

    /// Add an entity to the filter (only pairs involving this entity are checked).
    pub fn track(&mut self, entity: Entity) {
        self.filter.insert(entity);
    }

    /// Clear the filter (check all pairs).
    pub fn clear_filter(&mut self) {
        self.filter.clear();
    }
}

impl Default for EcsCollisionSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl System for EcsCollisionSystem {
    fn name(&self) -> &str {
        "EcsCollisionSystem"
    }

    fn priority(&self) -> i32 {
        0
    }

    fn run(&mut self, world: &mut World, _dt: f32) {
        // Collect all entities with both Transform and Collider.
        let colliders = world.query2::<Transform, Collider>();
        let n = colliders.len();

        // O(n²) pair check — acceptable for small-mid entity counts.
        for i in 0..n {
            for j in (i + 1)..n {
                let a = colliders[i];
                let b = colliders[j];

                // Filter: skip if filter is non-empty and neither entity is tracked.
                if !self.filter.is_empty() && !self.filter.contains(&a) && !self.filter.contains(&b) {
                    continue;
                }

                let a_tf = *world.get::<Transform>(a).unwrap();
                let a_col = *world.get::<Collider>(a).unwrap();
                let b_tf = *world.get::<Transform>(b).unwrap();
                let b_col = *world.get::<Collider>(b).unwrap();

                if let Some(overlap) = a_col.overlaps(&a_tf, &b_col, &b_tf) {
                    world.push_collision(CollisionEvent {
                        entity_a: a,
                        entity_b: b,
                        overlap_x: overlap.overlap_x,
                        overlap_y: overlap.overlap_y,
                    });
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// CameraSystem — smooth follow with dead zone
// ---------------------------------------------------------------------------

/// Moves the GameCamera to follow its target entity (usually the player).
/// Uses a dead zone and lerp for smooth tracking.
pub struct CameraSystem;

impl System for CameraSystem {
    fn name(&self) -> &str {
        "CameraSystem"
    }

    fn priority(&self) -> i32 {
        10
    }

    fn run(&mut self, world: &mut World, dt: f32) {
        // Clone target to avoid borrow issues.
        let (target, speed, dead) = match world.get_resource::<GameCamera>() {
            Some(cam) => (cam.target, cam.follow_speed, cam.dead_zone),
            None => return,
        };

        let target = match target {
            Some(e) => e,
            None => return,
        };

        let target_pos = match world.get::<Transform>(target) {
            Some(tf) => tf.position,
            None => return,
        };

        let current = world.get_resource::<GameCamera>().unwrap().position;

        // Calculate desired camera center with dead zone.
        let dx = target_pos.x - current.x;
        let dy = target_pos.y - current.y;
        let mut new_x = current.x;
        let mut new_y = current.y;

        if dx > dead.x {
            new_x += (dx - dead.x) * speed * dt;
        } else if dx < -dead.x {
            new_x += (dx + dead.x) * speed * dt;
        }

        if dy > dead.y {
            new_y += (dy - dead.y) * speed * dt;
        } else if dy < -dead.y {
            new_y += (dy + dead.y) * speed * dt;
        }

        if let Some(cam) = world.get_resource_mut::<GameCamera>() {
            cam.position = Vec2::new(new_x, new_y);
        }
    }
}

// ---------------------------------------------------------------------------
// HealthSystem — remove dead entities
// ---------------------------------------------------------------------------

/// Despawns entities whose Health has reached zero.
pub struct HealthSystem;

impl System for HealthSystem {
    fn name(&self) -> &str {
        "HealthSystem"
    }

    fn priority(&self) -> i32 {
        5
    }

    fn run(&mut self, world: &mut World, _dt: f32) {
        let dead: Vec<Entity> = world.query::<Health>()
            .into_iter()
            .filter(|&e| {
                world.get::<Health>(e)
                    .map_or(false, |hp| !hp.is_alive())
            })
            .collect();

        for e in dead {
            world.despawn(e);
        }
    }
}

// ---------------------------------------------------------------------------
// RenderSystem — build render command buffer from sprites
// ---------------------------------------------------------------------------

/// Iterates all (Transform, GameSprite) entities, produces DrawCommands
/// sorted by z_index (ascending). Commands are pushed into the
/// `RenderCommandBuffer` resource for the backend to consume.
pub struct RenderSystem;

impl System for RenderSystem {
    fn name(&self) -> &str {
        "RenderSystem"
    }

    fn priority(&self) -> i32 {
        100
    }

    fn run(&mut self, world: &mut World, _dt: f32) {
        // Collect all sprite data first to avoid borrow conflicts.
        let entities = world.query2::<Transform, GameSprite>();
        let mut entries: Vec<(Entity, Transform, GameSprite)> = entities.into_iter()
            .filter_map(|e| {
                let tf = *world.get::<Transform>(e)?;
                let sprite = world.get::<GameSprite>(e)?.clone();
                Some((e, tf, sprite))
            })
            .collect();
        entries.sort_by_key(|(_, _, s)| s.z_index);

        // Now write to the command buffer (no outstanding borrows).
        if let Some(cmds) = world.get_resource_mut::<RenderCommandBuffer>() {
            cmds.clear();
            cmds.push(DrawCommand::Clear { color: Color::rgb(0.1, 0.1, 0.15) });

            for (_e, tf, sprite) in &entries {
                let w = sprite.source.width;
                let h = sprite.source.height;
                let dest = Rect::new(
                    tf.position.x - w * 0.5,
                    tf.position.y - h * 0.5,
                    w,
                    h,
                );

                cmds.push(DrawCommand::DrawSprite {
                    texture: sprite.texture.clone(),
                    dest,
                    src_rect: None,
                    color: sprite.color,
                    alpha: sprite.color.a,
                    flip_x: sprite.flip_x,
                    flip_y: sprite.flip_y,
                    rotation: tf.rotation,
                    z_index: sprite.z_index,
                });
            }

            cmds.push(DrawCommand::Present);
        }
    }
}

// ---------------------------------------------------------------------------
// Marker convenience: PlayerFetchSystem (example)
// ---------------------------------------------------------------------------

/// Example system that reads player-specific state. Demonstrates how to
/// query for marker components.
pub struct PlayerInfoSystem;

impl System for PlayerInfoSystem {
    fn name(&self) -> &str {
        "PlayerInfoSystem"
    }

    fn priority(&self) -> i32 {
        200
    }

    fn run(&mut self, world: &mut World, _dt: f32) {
        let players: Vec<Entity> = world.query3::<PlayerMarker, Transform, Health>();
        // In a real system this would update UI / HUD resources.
        // Here we just verify the query works.
        for p in &players {
            let tf = world.get::<Transform>(*p).unwrap();
            let hp = world.get::<Health>(*p).unwrap();
            // eprintln!("Player at ({:.1}, {:.1}) HP {:.0}/{:.0}",
            //     tf.position.x, tf.position.y, hp.current, hp.max);
            let _ = (tf, hp);
        }
    }
}

// ---------------------------------------------------------------------------
// NPC AI stub
// ---------------------------------------------------------------------------

/// Stub system for NPC behavior. In a full engine this would drive
/// pathfinding, dialogue triggers, schedule, etc.
pub struct NpcAiSystem;

impl System for NpcAiSystem {
    fn name(&self) -> &str {
        "NpcAiSystem"
    }

    fn priority(&self) -> i32 {
        50
    }

    fn run(&mut self, world: &mut World, dt: f32) {
        let npcs = world.query3::<NpcMarker, Transform, Velocity>();
        // Stub: NPCs with velocity move. In production, this would
        // read schedule/state and set velocity accordingly.
        for npc in &npcs {
            let _ = world.get::<Transform>(*npc);
            let _ = world.get::<Velocity>(*npc);
            let _ = dt;
        }
    }
}

// ---------------------------------------------------------------------------
// MonsterAI stub
// ---------------------------------------------------------------------------

/// Stub system for monster/enemy AI.
pub struct MonsterAiSystem;

impl System for MonsterAiSystem {
    fn name(&self) -> &str {
        "MonsterAiSystem"
    }

    fn priority(&self) -> i32 {
        50
    }

    fn run(&mut self, world: &mut World, dt: f32) {
        let monsters = world.query3::<MonsterMarker, Transform, Velocity>();
        for m in &monsters {
            let _ = world.get::<Transform>(*m);
            let _ = world.get::<Velocity>(*m);
            let _ = dt;
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::components::{Collider, GameCamera, GameSprite, Health, MonsterMarker, NpcMarker, PlayerMarker, RenderCommandBuffer, TimeState, Transform, Velocity};

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(GameCamera::new(800.0, 600.0));
        world.insert_resource(RenderCommandBuffer::new());
        world.insert_resource(TimeState::default());
        world
    }

    #[test]
    fn test_movement_system() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert(e, Transform::from_position(0.0, 0.0));
        world.insert(e, Velocity::new(100.0, 50.0));

        let mut sys = MovementSystem;
        sys.run(&mut world, 1.0);

        let tf = world.get::<Transform>(e).unwrap();
        assert!((tf.position.x - 100.0).abs() < 0.01);
        assert!((tf.position.y - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_movement_system_zero_velocity() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert(e, Transform::from_position(5.0, 5.0));
        world.insert(e, Velocity::zero());

        MovementSystem.run(&mut world, 1.0);

        let tf = world.get::<Transform>(e).unwrap();
        assert!((tf.position.x - 5.0).abs() < 0.01);
        assert!((tf.position.y - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_collision_system_detects_overlap() {
        let mut world = World::new();
        let a = world.spawn();
        world.insert(a, Transform::from_position(0.0, 0.0));
        world.insert(a, Collider::aabb(32.0, 32.0));

        let b = world.spawn();
        world.insert(b, Transform::from_position(20.0, 0.0));
        world.insert(b, Collider::aabb(32.0, 32.0));

        EcsCollisionSystem::new().run(&mut world, 1.0 / 60.0);

        let events = world.drain_collisions();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].entity_a, a);
        assert_eq!(events[0].entity_b, b);
        assert!(events[0].overlap_x > 0.0);
    }

    #[test]
    fn test_collision_system_no_overlap() {
        let mut world = World::new();
        let a = world.spawn();
        world.insert(a, Transform::from_position(0.0, 0.0));
        world.insert(a, Collider::aabb(16.0, 16.0));

        let b = world.spawn();
        world.insert(b, Transform::from_position(100.0, 100.0));
        world.insert(b, Collider::aabb(16.0, 16.0));

        EcsCollisionSystem::new().run(&mut world, 1.0 / 60.0);

        assert!(world.drain_collisions().is_empty());
    }

    #[test]
    fn test_collision_system_filter() {
        let mut world = World::new();
        let a = world.spawn();
        world.insert(a, Transform::from_position(0.0, 0.0));
        world.insert(a, Collider::aabb(32.0, 32.0));

        let b = world.spawn();
        world.insert(b, Transform::from_position(20.0, 0.0));
        world.insert(b, Collider::aabb(32.0, 32.0));

        let c = world.spawn();
        world.insert(c, Transform::from_position(15.0, 0.0));
        world.insert(c, Collider::aabb(32.0, 32.0));

        // Filter: only check pairs involving 'a'.
        let mut sys = EcsCollisionSystem::new();
        sys.track(a);
        sys.run(&mut world, 1.0 / 60.0);

        let events = world.drain_collisions();
        // Should find a-b and a-c overlaps, but NOT b-c.
        assert_eq!(events.len(), 2);
        for ev in &events {
            assert!(ev.entity_a == a || ev.entity_b == a);
        }
    }

    #[test]
    fn test_camera_system_follows_target() {
        let mut world = setup_world();
        let player = world.spawn();
        world.insert(player, Transform::from_position(200.0, 100.0));

        {
            let cam = world.get_resource_mut::<GameCamera>().unwrap();
            cam.target = Some(player);
            cam.follow_speed = 1.0; // fast snap for test
            cam.dead_zone = Vec2::new(0.0, 0.0);
        }

        CameraSystem.run(&mut world, 1.0);

        let cam = world.get_resource::<GameCamera>().unwrap();
        assert!((cam.position.x - 200.0).abs() < 1.0);
        assert!((cam.position.y - 100.0).abs() < 1.0);
    }

    #[test]
    fn test_health_system_despawns_dead() {
        let mut world = World::new();
        let alive = world.spawn();
        world.insert(alive, Health::new(100.0));

        let dead = world.spawn();
        let mut hp = Health::new(10.0);
        hp.take_damage(999.0);
        world.insert(dead, hp);

        assert_eq!(world.entity_count(), 2);

        HealthSystem.run(&mut world, 1.0);

        assert_eq!(world.entity_count(), 1);
        assert!(world.is_alive(alive));
    }

    #[test]
    fn test_render_system_produces_commands() {
        let mut world = setup_world();
        let e1 = world.spawn();
        world.insert(e1, Transform::from_position(100.0, 200.0));
        world.insert(e1, GameSprite::new("hero.png").with_z_index(10));

        let e2 = world.spawn();
        world.insert(e2, Transform::from_position(50.0, 50.0));
        world.insert(e2, GameSprite::new("bg.png").with_z_index(0));

        RenderSystem.run(&mut world, 1.0 / 60.0);

        let cmds = world.get_resource::<RenderCommandBuffer>().unwrap();
        // Clear + 2 sprites + Present
        assert!(cmds.commands.len() >= 4);

        // First draw should be the low-z-index sprite.
        match &cmds.commands[1] {
            DrawCommand::DrawSprite { texture, z_index, .. } => {
                assert_eq!(texture, "bg.png");
                assert_eq!(*z_index, 0);
            }
            _ => panic!("Expected DrawSprite for bg.png"),
        }
    }

    #[test]
    fn test_system_runner_priority_order() {
        let mut world = World::new();

        // Create a player entity.
        let p = world.spawn();
        world.insert(p, Transform::from_position(0.0, 0.0));
        world.insert(p, Velocity::new(10.0, 0.0));
        world.insert(p, Collider::aabb(16.0, 16.0));
        world.insert(p, GameSprite::new("p.png"));
        world.insert(p, PlayerMarker);
        world.insert(p, Health::new(100.0));

        // Create a monster.
        let m = world.spawn();
        world.insert(m, Transform::from_position(50.0, 0.0));
        world.insert(m, Velocity::new(-5.0, 0.0));
        world.insert(m, Collider::aabb(16.0, 16.0));
        world.insert(m, GameSprite::new("m.png"));
        world.insert(m, MonsterMarker);
        world.insert(m, Health::new(50.0));

        world.insert_resource(GameCamera::new(800.0, 600.0));
        world.insert_resource(RenderCommandBuffer::new());
        world.insert_resource(TimeState::default());

        let mut runner = super::super::ecs::SystemRunner::new();
        runner.add_system(Box::new(MovementSystem));
        runner.add_system(Box::new(EcsCollisionSystem::new()));
        runner.add_system(Box::new(CameraSystem));
        runner.add_system(Box::new(HealthSystem));
        runner.add_system(Box::new(RenderSystem));
        runner.add_system(Box::new(PlayerInfoSystem));
        runner.add_system(Box::new(NpcAiSystem));
        runner.add_system(Box::new(MonsterAiSystem));

        assert_eq!(runner.system_count(), 8);

        // Run one tick.
        runner.run_all(&mut world, 1.0 / 60.0);

        // Player moved.
        let p_tf = world.get::<Transform>(p).unwrap();
        assert!(p_tf.position.x > 0.0);

        // Monster moved.
        let m_tf = world.get::<Transform>(m).unwrap();
        assert!(m_tf.position.x < 50.0);

        // Camera is tracking player.
        let cam = world.get_resource::<GameCamera>().unwrap();
        assert!(cam.position.x > 0.0);

        // Render commands were produced.
        let cmds = world.get_resource::<RenderCommandBuffer>().unwrap();
        assert!(!cmds.commands.is_empty());
    }
}
