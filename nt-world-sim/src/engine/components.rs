use super::ecs::{Component, Entity};
use super::renderer::{Color, DrawCommand, Rect, Vec2};

// ---------------------------------------------------------------------------
// Transform — position, rotation, scale
// ---------------------------------------------------------------------------

/// Spatial transform: position, rotation (radians), scale.
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
}

impl Transform {
    pub fn new(position: Vec2, rotation: f32, scale: Vec2) -> Self {
        Self { position, rotation, scale }
    }

    pub fn from_position(x: f32, y: f32) -> Self {
        Self { position: Vec2::new(x, y), rotation: 0.0, scale: Vec2::one() }
    }

    pub fn zero() -> Self {
        Self { position: Vec2::zero(), rotation: 0.0, scale: Vec2::one() }
    }

    /// Translate by (dx, dy).
    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.position.x += dx;
        self.position.y += dy;
    }

    /// Rotate by additional radians.
    pub fn rotate(&mut self, radians: f32) {
        self.rotation += radians;
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::zero()
    }
}

impl Component for Transform {}

// ---------------------------------------------------------------------------
// Velocity
// ---------------------------------------------------------------------------

/// Linear velocity in 2D.
#[derive(Debug, Clone, Copy)]
pub struct Velocity {
    pub vx: f32,
    pub vy: f32,
}

impl Velocity {
    pub fn new(vx: f32, vy: f32) -> Self {
        Self { vx, vy }
    }

    pub fn zero() -> Self {
        Self { vx: 0.0, vy: 0.0 }
    }

    pub fn speed(&self) -> f32 {
        (self.vx * self.vx + self.vy * self.vy).sqrt()
    }

    pub fn normalize(&self) -> Self {
        let s = self.speed();
        if s < 1e-8 {
            Self::zero()
        } else {
            Self { vx: self.vx / s, vy: self.vy / s }
        }
    }
}

impl Default for Velocity {
    fn default() -> Self {
        Self::zero()
    }
}

impl Component for Velocity {}

// ---------------------------------------------------------------------------
// Sprite
// ---------------------------------------------------------------------------

/// Renderable sprite: texture key, source rect, tint color, flip flags.
#[derive(Debug, Clone)]
pub struct GameSprite {
    pub texture: String,
    pub source: Rect,
    pub color: Color,
    pub flip_x: bool,
    pub flip_y: bool,
    pub z_index: i32,
}

impl GameSprite {
    pub fn new(texture: &str) -> Self {
        Self {
            texture: texture.to_string(),
            source: Rect::new(0.0, 0.0, 16.0, 16.0),
            color: Color::white(),
            flip_x: false,
            flip_y: false,
            z_index: 0,
        }
    }

    pub fn with_z_index(mut self, z: i32) -> Self {
        self.z_index = z;
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

impl Component for GameSprite {}

// ---------------------------------------------------------------------------
// Health
// ---------------------------------------------------------------------------

/// Health component with current/max and death threshold.
#[derive(Debug, Clone)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }

    pub fn is_alive(&self) -> bool {
        self.current > 0.0
    }

    pub fn is_full(&self) -> bool {
        (self.current - self.max).abs() < f32::EPSILON
    }

    pub fn ratio(&self) -> f32 {
        if self.max <= 0.0 { 0.0 } else { (self.current / self.max).clamp(0.0, 1.0) }
    }

    pub fn take_damage(&mut self, amount: f32) -> f32 {
        let actual = amount.min(self.current);
        self.current -= actual;
        actual
    }

    pub fn heal(&mut self, amount: f32) -> f32 {
        let actual = amount.min(self.max - self.current);
        self.current += actual;
        actual
    }
}

impl Component for Health {}

// ---------------------------------------------------------------------------
// Collider — AABB
// ---------------------------------------------------------------------------

/// Axis-aligned bounding box collider.
#[derive(Debug, Clone, Copy)]
pub struct Collider {
    /// Offset from entity's Transform position.
    pub offset: Vec2,
    /// Half-extents (width/2, height/2).
    pub half_extents: Vec2,
    /// Whether this collider is a sensor (triggers but no physical response).
    pub is_sensor: bool,
}

impl Collider {
    pub fn aabb(width: f32, height: f32) -> Self {
        Self {
            offset: Vec2::zero(),
            half_extents: Vec2::new(width / 2.0, height / 2.0),
            is_sensor: false,
        }
    }

    pub fn sensor(width: f32, height: f32) -> Self {
        Self {
            offset: Vec2::zero(),
            half_extents: Vec2::new(width / 2.0, height / 2.0),
            is_sensor: true,
        }
    }

    /// Compute world-space AABB from a Transform.
    pub fn world_aabb(&self, transform: &Transform) -> Rect {
        let cx = transform.position.x + self.offset.x;
        let cy = transform.position.y + self.offset.y;
        let hw = self.half_extents.x * transform.scale.x.abs();
        let hh = self.half_extents.y * transform.scale.y.abs();
        Rect::new(cx - hw, cy - hh, hw * 2.0, hh * 2.0)
    }

    /// Test overlap between two colliders at given transforms.
    pub fn overlaps(&self, a_tf: &Transform, other: &Collider, b_tf: &Transform) -> Option<Overlap> {
        let a = self.world_aabb(a_tf);
        let b = other.world_aabb(b_tf);

        if !a.intersects(&b) {
            return None;
        }

        // Minimum translation vector
        let a_cx = a.center().x;
        let a_cy = a.center().y;
        let b_cx = b.center().x;
        let b_cy = b.center().y;

        let dx = b_cx - a_cx;
        let dy = b_cy - a_cy;
        let overlap_x = (a.width / 2.0 + b.width / 2.0) - dx.abs();
        let overlap_y = (a.height / 2.0 + b.height / 2.0) - dy.abs();

        Some(Overlap { overlap_x, overlap_y, from_a_to_b: Vec2::new(dx.signum(), dy.signum()) })
    }
}

impl Component for Collider {}

/// Result of a collision overlap test.
#[derive(Debug, Clone, Copy)]
pub struct Overlap {
    pub overlap_x: f32,
    pub overlap_y: f32,
    pub from_a_to_b: Vec2,
}

// ---------------------------------------------------------------------------
// Marker components (zero-size tags for archetype queries)
// ---------------------------------------------------------------------------

/// Marks an entity as the player character.
#[derive(Debug, Clone, Copy, Default)]
pub struct PlayerMarker;
impl Component for PlayerMarker {}

/// Marks an entity as an NPC.
#[derive(Debug, Clone, Copy, Default)]
pub struct NpcMarker;
impl Component for NpcMarker {}

/// Marks an entity as a monster/enemy.
#[derive(Debug, Clone, Copy, Default)]
pub struct MonsterMarker;
impl Component for MonsterMarker {}

// ---------------------------------------------------------------------------
// Rendering resources
// ---------------------------------------------------------------------------

/// Camera resource stored in the World for systems to access.
#[derive(Debug, Clone)]
pub struct GameCamera {
    pub position: Vec2,
    pub target: Option<Entity>,
    pub zoom: f32,
    pub viewport_width: f32,
    pub viewport_height: f32,
    pub follow_speed: f32,
    pub dead_zone: Vec2,
}

impl GameCamera {
    pub fn new(viewport_width: f32, viewport_height: f32) -> Self {
        Self {
            position: Vec2::zero(),
            target: None,
            zoom: 1.0,
            viewport_width,
            viewport_height,
            follow_speed: 5.0,
            dead_zone: Vec2::new(32.0, 32.0),
        }
    }

    pub fn with_target(mut self, entity: Entity) -> Self {
        self.target = Some(entity);
        self
    }

    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        Vec2::new(
            (world_pos.x - self.position.x) * self.zoom + self.viewport_width / 2.0,
            (world_pos.y - self.position.y) * self.zoom + self.viewport_height / 2.0,
        )
    }

    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        Vec2::new(
            (screen_pos.x - self.viewport_width / 2.0) / self.zoom + self.position.x,
            (screen_pos.y - self.viewport_height / 2.0) / self.zoom + self.position.y,
        )
    }

    pub fn visible_rect(&self) -> Rect {
        let hw = self.viewport_width / (2.0 * self.zoom);
        let hh = self.viewport_height / (2.0 * self.zoom);
        Rect::new(self.position.x - hw, self.position.y - hh, hw * 2.0, hh * 2.0)
    }
}

/// Time resource: per-tick dt and total elapsed time.
#[derive(Debug, Clone, Copy)]
pub struct TimeState {
    pub dt: f32,
    pub elapsed: f64,
    pub frame: u64,
}

impl Default for TimeState {
    fn default() -> Self {
        Self { dt: 1.0 / 60.0, elapsed: 0.0, frame: 0 }
    }
}

/// Render command buffer — systems append `DrawCommand`s, the renderer consumes them.
#[derive(Debug, Clone)]
pub struct RenderCommandBuffer {
    pub commands: Vec<DrawCommand>,
}

impl RenderCommandBuffer {
    pub fn new() -> Self {
        Self { commands: Vec::new() }
    }

    pub fn push(&mut self, cmd: DrawCommand) {
        self.commands.push(cmd);
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }

    pub fn drain(&mut self) -> Vec<DrawCommand> {
        std::mem::take(&mut self.commands)
    }
}

impl Default for RenderCommandBuffer {
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

    #[test]
    fn test_health_damage_and_heal() {
        let mut hp = Health::new(100.0);
        assert!(hp.is_alive());
        assert!(hp.is_full());
        assert_eq!(hp.ratio(), 1.0);

        hp.take_damage(30.0);
        assert_eq!(hp.current, 70.0);
        assert!((hp.ratio() - 0.7).abs() < 0.01);

        hp.heal(10.0);
        assert_eq!(hp.current, 80.0);

        // cannot heal above max
        hp.heal(999.0);
        assert_eq!(hp.current, 100.0);
    }

    #[test]
    fn test_health_lethal() {
        let mut hp = Health::new(10.0);
        let actual = hp.take_damage(999.0);
        assert_eq!(actual, 10.0);
        assert!(!hp.is_alive());
        assert_eq!(hp.current, 0.0);
    }

    #[test]
    fn test_collider_aabb() {
        let col = Collider::aabb(32.0, 16.0);
        let tf = Transform::from_position(100.0, 50.0);
        let aabb = col.world_aabb(&tf);
        assert_eq!(aabb.x, 84.0); // 100 - 16
        assert_eq!(aabb.y, 42.0); // 50 - 8
        assert_eq!(aabb.width, 32.0);
        assert_eq!(aabb.height, 16.0);
    }

    #[test]
    fn test_collider_overlap() {
        let a = Collider::aabb(32.0, 32.0);
        let b = Collider::aabb(32.0, 32.0);
        let tf_a = Transform::from_position(0.0, 0.0);
        let tf_b = Transform::from_position(20.0, 0.0);

        let overlap = a.overlaps(&tf_a, &b, &tf_b);
        assert!(overlap.is_some());
        let o = overlap.unwrap();
        assert!(o.overlap_x > 0.0);
    }

    #[test]
    fn test_collider_no_overlap() {
        let a = Collider::aabb(16.0, 16.0);
        let b = Collider::aabb(16.0, 16.0);
        let tf_a = Transform::from_position(0.0, 0.0);
        let tf_b = Transform::from_position(100.0, 100.0);

        assert!(a.overlaps(&tf_a, &b, &tf_b).is_none());
    }

    #[test]
    fn test_collider_sensor() {
        let col = Collider::sensor(16.0, 16.0);
        assert!(col.is_sensor);
    }

    #[test]
    fn test_game_camera() {
        let cam = GameCamera::new(800.0, 600.0);
        let screen = cam.world_to_screen(Vec2::new(0.0, 0.0));
        assert_eq!(screen, Vec2::new(400.0, 300.0));
    }

    #[test]
    fn test_velocity_speed_and_normalize() {
        let v = Velocity::new(3.0, 4.0);
        assert!((v.speed() - 5.0).abs() < 0.01);
        let n = v.normalize();
        assert!((n.speed() - 1.0).abs() < 0.01);
    }
}
