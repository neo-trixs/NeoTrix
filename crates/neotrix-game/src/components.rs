//! ECS 组件词汇 — 引擎层（Slice purge-round17 后仅留通用组件）.
//!
//! 游戏域组件（Npc/Enemy/AI/Status/Power）已随旧游戏删除，
//! 备份见 `_archive/purge-round17/components.rs`。

/// 位置组件
#[derive(Debug, Clone)]
pub struct Position { pub x: f32, pub y: f32 }

/// 速度组件
#[derive(Debug, Clone)]
pub struct Velocity { pub vx: f32, pub vy: f32 }

/// 物理属性
#[derive(Debug, Clone)]
pub struct Physics {
    pub max_speed: f32,
    pub acceleration: f32,
    pub deceleration: f32,
}

/// 精灵渲染
#[derive(Debug, Clone)]
pub struct Sprite {
    pub w: f32, pub h: f32,
    pub color: macroquad::color::Color,
}

/// 生命值
#[derive(Debug, Clone)]
pub struct Health { pub hp: f32, pub max_hp: f32 }

/// 实体名称
#[derive(Debug, Clone)]
pub struct Name(pub String);

/// Marker: 玩家
#[derive(Debug, Clone)]
pub struct PlayerMarker;

/// Marker: 平台
#[derive(Debug, Clone)]
pub struct PlatformMarker;

// ══ L0 物理组件 ══

/// 重力组件 — 借鉴 Bevy Avian2d
#[derive(Debug, Clone)]
pub struct Gravity {
    pub scale: f32,  // 重力缩放因子 (1.0 = 标准重力)
}

impl Default for Gravity {
    fn default() -> Self { Self { scale: 1.0 } }
}

/// 静态物体标记 — 不受物理影响
#[derive(Debug, Clone)]
pub struct Static;

/// AABB 碰撞体
#[derive(Debug, Clone)]
pub struct Collider {
    pub w: f32, pub h: f32,
}

/// 物理常量
pub const GRAVITY_ACCEL: f32 = 980.0;  // 像素/秒²
pub const FRICTION: f32 = 0.88;        // 摩擦系数
