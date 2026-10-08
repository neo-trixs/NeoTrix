/// NT-GAME 引擎核心视觉组件
///
/// 吸收 Godot 4 的 Node2D/Sprite2D/Camera2D 概念
/// 用 ECS 组合模式替代继承体系

use std::ops::{Add, Mul, Sub};

// ═══════════════════════════════════════════════════════════════════
// 基础数学类型
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const ONE: Self = Self { x: 1.0, y: 1.0 };

    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalized(self) -> Self {
        let len = self.length();
        if len > 0.0001 {
            Self {
                x: self.x / len,
                y: self.y / len,
            }
        } else {
            Self::ZERO
        }
    }

    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    pub fn distance_to(self, other: Self) -> f32 {
        (self - other).length()
    }

    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }
}

impl Add for Vec2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Vec2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn contains_point(self, point: Vec2) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.w
            && point.y >= self.y
            && point.y <= self.y + self.h
    }

    pub fn intersects(self, other: Rect) -> bool {
        self.x < other.x + other.w
            && self.x + self.w > other.x
            && self.y < other.y + other.h
            && self.y + self.h > other.y
    }

    pub fn center(self) -> Vec2 {
        Vec2::new(self.x + self.w / 2.0, self.y + self.h / 2.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    pub const BLACK: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const RED: Self = Self {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const GREEN: Self = Self {
        r: 0,
        g: 255,
        b: 0,
        a: 255,
    };
    pub const BLUE: Self = Self {
        r: 0,
        g: 0,
        b: 255,
        a: 255,
    };
    pub const TRANSPARENT: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };

    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }
}

// ═══════════════════════════════════════════════════════════════════
// 变换组件 — 吸收 Godot Node2D
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy)]
pub struct Transform2D {
    pub position: Vec2,
    pub rotation: f32,   // 弧度
    pub scale: Vec2,
    pub z_index: i32,
}

impl Default for Transform2D {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
            z_index: 0,
        }
    }
}

impl Transform2D {
    pub fn new(position: Vec2) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }

    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn with_scale(mut self, scale: Vec2) -> Self {
        self.scale = scale;
        self
    }

    pub fn with_z_index(mut self, z_index: i32) -> Self {
        self.z_index = z_index;
        self
    }

    /// 吸收 Godot 的全局变换：应用缩放和旋转后得到世界坐标
    pub fn apply_to_point(self, local: Vec2) -> Vec2 {
        let cos_r = self.rotation.cos();
        let sin_r = self.rotation.sin();
        let scaled = Vec2::new(local.x * self.scale.x, local.y * self.scale.y);
        Vec2::new(
            self.position.x + scaled.x * cos_r - scaled.y * sin_r,
            self.position.y + scaled.x * sin_r + scaled.y * cos_r,
        )
    }
}

// ═══════════════════════════════════════════════════════════════════
// 运动组件 — 吸收 Godot CharacterBody2D
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy)]
pub struct Velocity {
    pub vx: f32,
    pub vy: f32,
}

impl Default for Velocity {
    fn default() -> Self {
        Self { vx: 0.0, vy: 0.0 }
    }
}

impl Velocity {
    pub fn new(vx: f32, vy: f32) -> Self {
        Self { vx, vy }
    }

    pub fn speed(self) -> f32 {
        Vec2::new(self.vx, self.vy).length()
    }

    pub fn direction(self) -> Vec2 {
        Vec2::new(self.vx, self.vy).normalized()
    }
}

/// 移动系统参数
#[derive(Debug, Clone)]
pub struct Movement {
    pub speed: f32,
    pub acceleration: f32,
    pub deceleration: f32,
    pub max_speed: f32,
}

impl Default for Movement {
    fn default() -> Self {
        Self {
            speed: 200.0,
            acceleration: 800.0,
            deceleration: 600.0,
            max_speed: 300.0,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// 精灵组件 — 吸收 Godot Sprite2D
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct SpriteComponent {
    pub texture_path: String,  // 资产路径
    pub region: Rect,          // 精灵表中的区域
    pub anchor: Vec2,          // 锚点 (0-1)
    pub flip_x: bool,
    pub flip_y: bool,
    pub color: Color,
    /// ⛔ **未接线规格**（nt-unwired-spec）：本字段零读点 ——
    ///    功能已**声明**但读者未实现。⛔ **不要删**（删掉即销毁规格）。
    pub visible: bool,
    pub opacity: f32,
}

impl Default for SpriteComponent {
    fn default() -> Self {
        Self {
            texture_path: String::new(),
            region: Rect::new(0.0, 0.0, 32.0, 32.0),
            anchor: Vec2::new(0.5, 0.5),
            flip_x: false,
            flip_y: false,
            color: Color::WHITE,
            visible: true,
            opacity: 1.0,
        }
    }
}

impl SpriteComponent {
    pub fn new(texture_path: &str) -> Self {
        Self {
            texture_path: texture_path.to_string(),
            ..Default::default()
        }
    }

    pub fn with_region(mut self, x: f32, y: f32, w: f32, h: f32) -> Self {
        self.region = Rect::new(x, y, w, h);
        self
    }

    pub fn with_anchor(mut self, anchor: Vec2) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn with_flip_x(mut self, flip: bool) -> Self {
        self.flip_x = flip;
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

// ═══════════════════════════════════════════════════════════════════
// 碰撞组件 — 吸收 Godot CollisionShape2D
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub enum ColliderShape {
    Rectangle { size: Vec2 },
    Circle { radius: f32 },
}

#[derive(Debug, Clone)]
pub struct Collider {
    pub shape: ColliderShape,
    pub offset: Vec2,
    pub is_sensor: bool,  // 触发器（不阻挡移动）
    pub layer: u32,       // 碰撞层
    pub mask: u32,        // 碰撞掩码
}

impl Default for Collider {
    fn default() -> Self {
        Self {
            shape: ColliderShape::Rectangle {
                size: Vec2::new(32.0, 32.0),
            },
            offset: Vec2::ZERO,
            is_sensor: false,
            layer: 1,
            mask: 0xFFFF,
        }
    }
}

impl Collider {
    pub fn rectangle(w: f32, h: f32) -> Self {
        Self {
            shape: ColliderShape::Rectangle {
                size: Vec2::new(w, h),
            },
            ..Default::default()
        }
    }

    pub fn circle(radius: f32) -> Self {
        Self {
            shape: ColliderShape::Circle { radius },
            ..Default::default()
        }
    }

    pub fn sensor(mut self) -> Self {
        self.is_sensor = true;
        self
    }

    /// 计算世界空间 AABB
    pub fn world_aabb(&self, position: Vec2) -> Rect {
        let pos = position + self.offset;
        match &self.shape {
            ColliderShape::Rectangle { size } => Rect::new(
                pos.x - size.x / 2.0,
                pos.y - size.y / 2.0,
                size.x,
                size.y,
            ),
            ColliderShape::Circle { radius } => Rect::new(
                pos.x - radius,
                pos.y - radius,
                radius * 2.0,
                radius * 2.0,
            ),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// 生命值组件
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Default for Health {
    fn default() -> Self {
        Self {
            current: 100.0,
            max: 100.0,
        }
    }
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self {
            current: max,
            max,
        }
    }

    pub fn is_alive(self) -> bool {
        self.current > 0.0
    }

    pub fn percentage(self) -> f32 {
        if self.max > 0.0 {
            self.current / self.max
        } else {
            0.0
        }
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

// ═══════════════════════════════════════════════════════════════════
// 动画组件 — 吸收 Godot AnimationPlayer
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct AnimationPlayer {
    pub animations: std::collections::HashMap<String, AnimationClip>,
    pub current: Option<String>,
    pub time: f32,
    pub speed: f32,
    pub playing: bool,
}

#[derive(Debug, Clone)]
pub struct AnimationClip {
    pub keyframes: Vec<Keyframe>,
    pub looping: bool,
    pub duration: f32,
}

#[derive(Debug, Clone)]
pub struct Keyframe {
    pub time: f32,
    pub sprite_region: Rect,
    pub offset: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
}

impl Default for AnimationPlayer {
    fn default() -> Self {
        Self {
            animations: std::collections::HashMap::new(),
            current: None,
            time: 0.0,
            speed: 1.0,
            playing: false,
        }
    }
}

impl AnimationPlayer {
    pub fn play(&mut self, name: &str) {
        if self.animations.contains_key(name) {
            self.current = Some(name.to_string());
            self.time = 0.0;
            self.playing = true;
        }
    }

    pub fn stop(&mut self) {
        self.playing = false;
    }

    pub fn tick(&mut self, dt: f32) -> Option<&Keyframe> {
        if !self.playing {
            return None;
        }
        let name = self.current.as_ref()?;
        let clip = self.animations.get(name)?;
        self.time += dt * self.speed;

        if self.time >= clip.duration {
            if clip.looping {
                self.time %= clip.duration;
            } else {
                self.playing = false;
                return clip.keyframes.last();
            }
        }

        // 找到当前关键帧
        clip.keyframes
            .iter()
            .rfind(|kf| kf.time <= self.time)
            .or(clip.keyframes.first())
    }

    pub fn add_animation(&mut self, name: &str, clip: AnimationClip) {
        self.animations.insert(name.to_string(), clip);
    }
}

// ═══════════════════════════════════════════════════════════════════
// 相机组件 — 吸收 Godot Camera2D
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct Camera2D {
    pub zoom: f32,
    pub smoothing: f32,
    pub limits: CameraLimits,
    pub shake: ScreenShake,
}

#[derive(Debug, Clone)]
pub struct CameraLimits {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
}

impl Default for CameraLimits {
    fn default() -> Self {
        Self {
            min_x: f32::MIN,
            max_x: f32::MAX,
            min_y: f32::MIN,
            max_y: f32::MAX,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ScreenShake {
    pub intensity: f32,
    pub duration: f32,
    pub elapsed: f32,
}

impl Default for ScreenShake {
    fn default() -> Self {
        Self {
            intensity: 0.0,
            duration: 0.0,
            elapsed: 0.0,
        }
    }
}

impl Camera2D {
    pub fn new() -> Self {
        Self {
            zoom: 1.0,
            smoothing: 5.0,
            limits: CameraLimits::default(),
            shake: ScreenShake::default(),
        }
    }

    pub fn with_zoom(mut self, zoom: f32) -> Self {
        self.zoom = zoom;
        self
    }

    pub fn with_smoothing(mut self, smoothing: f32) -> Self {
        self.smoothing = smoothing;
        self
    }

    pub fn with_limits(mut self, min_x: f32, max_x: f32, min_y: f32, max_y: f32) -> Self {
        self.limits = CameraLimits {
            min_x,
            max_x,
            min_y,
            max_y,
        };
        self
    }

    pub fn shake(&mut self, intensity: f32, duration: f32) {
        self.shake = ScreenShake {
            intensity,
            duration,
            elapsed: 0.0,
        };
    }

    /// 吸收 Godot Camera2D 的平滑跟随
    pub fn follow(&mut self, target: Vec2, current: Vec2, dt: f32) -> Vec2 {
        let t = 1.0 - (-self.smoothing * dt).exp();
        let mut new_pos = current.lerp(target, t);

        // 应用限制
        new_pos.x = new_pos.x.clamp(self.limits.min_x, self.limits.max_x);
        new_pos.y = new_pos.y.clamp(self.limits.min_y, self.limits.max_y);

        // 应用屏幕震动
        if self.shake.elapsed < self.shake.duration {
            let decay = 1.0 - (self.shake.elapsed / self.shake.duration);
            new_pos.x += (self.shake.elapsed * 100.0).sin() * self.shake.intensity * decay;
            new_pos.y += (self.shake.elapsed * 130.0).cos() * self.shake.intensity * decay;
            self.shake.elapsed += dt;
        }

        new_pos
    }
}

impl Default for Camera2D {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════
// 标签组件 — 吸收 Godot 的 Node.name
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct Name(pub String);

impl Name {
    pub fn new(name: &str) -> Self {
        Self(name.to_string())
    }
}

// ═══════════════════════════════════════════════════════════════════
// 定时器组件 — 吸收 Godot Timer + Unity Coroutine
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct Timer {
    pub duration: f32,
    pub elapsed: f32,
    pub repeating: bool,
    pub active: bool,
    pub callback_id: Option<u64>,
}

impl Timer {
    pub fn once(duration: f32) -> Self {
        Self {
            duration,
            elapsed: 0.0,
            repeating: false,
            active: true,
            callback_id: None,
        }
    }

    pub fn repeating(duration: f32) -> Self {
        Self {
            duration,
            elapsed: 0.0,
            repeating: true,
            active: true,
            callback_id: None,
        }
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        if !self.active {
            return false;
        }
        self.elapsed += dt;
        if self.elapsed >= self.duration {
            if self.repeating {
                self.elapsed -= self.duration;
                true
            } else {
                self.active = false;
                true
            }
        } else {
            false
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// 光照组件 — 吸收 Godot 2D Lighting
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy)]
pub struct Light2D {
    pub color: Color,
    pub intensity: f32,
    pub radius: f32,
    pub shadow: bool,
}

impl Default for Light2D {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            intensity: 1.0,
            radius: 100.0,
            shadow: false,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// 混合组件 — 吸收 Unity BlendTree
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct BlendState {
    pub current_animation: String,
    pub blend_parameter: f32,
    pub blends: Vec<BlendNode>,
}

#[derive(Debug, Clone)]
pub struct BlendNode {
    pub animation: String,
    pub threshold: f32,
}

impl Default for BlendState {
    fn default() -> Self {
        Self {
            current_animation: "idle".to_string(),
            blend_parameter: 0.0,
            blends: Vec::new(),
        }
    }
}

impl BlendState {
    pub fn update(&mut self, parameter: f32) -> &str {
        self.blend_parameter = parameter;
        if self.blends.is_empty() {
            return &self.current_animation;
        }

        // 找到最接近当前参数的混合节点
        let best = self
            .blends
            .iter()
            .min_by_key(|n| ((n.threshold - parameter).abs() * 1000.0) as u32)
            .unwrap();
        self.current_animation = best.animation.clone();
        &self.current_animation
    }
}

// ═══════════════════════════════════════════════════════════════════
// 测试
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec2_math() {
        let a = Vec2::new(3.0, 4.0);
        assert!((a.length() - 5.0).abs() < 0.001);
        assert!((a.normalized().length() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_rect_intersect() {
        let a = Rect::new(0.0, 0.0, 10.0, 10.0);
        let b = Rect::new(5.0, 5.0, 10.0, 10.0);
        let c = Rect::new(20.0, 20.0, 10.0, 10.0);
        assert!(a.intersects(b));
        assert!(!a.intersects(c));
    }

    #[test]
    fn test_health() {
        let mut h = Health::new(100.0);
        assert!(h.is_alive());
        let dmg = h.take_damage(30.0);
        assert!((dmg - 30.0).abs() < 0.001);
        assert!((h.current - 70.0).abs() < 0.001);
        h.heal(10.0);
        assert!((h.current - 80.0).abs() < 0.001);
    }

    #[test]
    fn test_timer_tick() {
        let mut t = Timer::once(1.0);
        assert!(!t.tick(0.5));
        assert!(t.tick(0.6));
        assert!(!t.active);
    }

    #[test]
    fn test_timer_repeating() {
        let mut t = Timer::repeating(1.0);
        assert!(!t.tick(0.5));
        assert!(t.tick(0.6));
        assert!(t.active);
        assert!((t.elapsed - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_animation_player() {
        let mut ap = AnimationPlayer::default();
        let clip = AnimationClip {
            keyframes: vec![
                Keyframe {
                    time: 0.0,
                    sprite_region: Rect::new(0.0, 0.0, 32.0, 32.0),
                    offset: Vec2::ZERO,
                    rotation: 0.0,
                    scale: Vec2::ONE,
                },
                Keyframe {
                    time: 0.5,
                    sprite_region: Rect::new(32.0, 0.0, 32.0, 32.0),
                    offset: Vec2::ZERO,
                    rotation: 0.0,
                    scale: Vec2::ONE,
                },
            ],
            looping: true,
            duration: 1.0,
        };
        ap.add_animation("walk", clip);
        ap.play("walk");
        assert!(ap.playing);
        let kf = ap.tick(0.25);
        assert!(kf.is_some());
    }

    #[test]
    fn test_camera_follow() {
        let mut cam = Camera2D::new().with_smoothing(5.0);
        let target = Vec2::new(100.0, 100.0);
        let current = Vec2::new(0.0, 0.0);
        let new_pos = cam.follow(target, current, 0.1);
        assert!(new_pos.x > 0.0);
        assert!(new_pos.x < 100.0);
    }

    #[test]
    fn test_blend_state() {
        let mut bs = BlendState {
            blends: vec![
                BlendNode {
                    animation: "walk".to_string(),
                    threshold: 0.5,
                },
                BlendNode {
                    animation: "run".to_string(),
                    threshold: 1.0,
                },
            ],
            ..Default::default()
        };
        assert_eq!(bs.update(0.3), "walk");
        assert_eq!(bs.update(0.8), "run");
    }
}
