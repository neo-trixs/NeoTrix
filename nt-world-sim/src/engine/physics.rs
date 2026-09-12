use super::renderer::{Vec2, Rect};

pub type PhysicsEntity = crate::core::entity::EntityId;

/// 刚体类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BodyType {
    /// 静态 (不动)
    Static,
    /// 动态 (受物理影响)
    Dynamic,
    /// 运动学 (脚本控制)
    Kinematic,
}

/// 刚体
#[derive(Debug, Clone)]
pub struct RigidBody {
    pub position: Vec2,
    pub velocity: Vec2,
    pub acceleration: Vec2,
    pub mass: f32,
    pub body_type: BodyType,
    pub gravity_scale: f32,
    pub friction: f32,
    pub restitution: f32,
}

impl RigidBody {
    pub fn new(body_type: BodyType) -> Self {
        Self {
            position: Vec2::zero(),
            velocity: Vec2::zero(),
            acceleration: Vec2::zero(),
            mass: 1.0,
            body_type,
            gravity_scale: 1.0,
            friction: 0.5,
            restitution: 0.3,
        }
    }

    pub fn dynamic() -> Self {
        Self::new(BodyType::Dynamic)
    }

    pub fn static_body() -> Self {
        Self::new(BodyType::Static)
    }

    pub fn kinematic() -> Self {
        Self::new(BodyType::Kinematic)
    }
}

/// 碰撞体
#[derive(Debug, Clone)]
pub enum Collider {
    /// 轴对齐包围盒
    AABB {
        half_extents: Vec2,
    },
    /// 圆形
    Circle {
        radius: f32,
    },
    /// 多边形
    Polygon {
        vertices: Vec<Vec2>,
    },
}

impl Collider {
    pub fn aabb(half_extents: Vec2) -> Self {
        Self::AABB { half_extents }
    }

    pub fn circle(radius: f32) -> Self {
        Self::Circle { radius }
    }

    pub fn polygon(vertices: Vec<Vec2>) -> Self {
        Self::Polygon { vertices }
    }

    /// 获取包围盒
    pub fn bounding_box(&self, position: Vec2) -> Rect {
        match self {
            Self::AABB { half_extents } => Rect::new(
                position.x - half_extents.x,
                position.y - half_extents.y,
                half_extents.x * 2.0,
                half_extents.y * 2.0,
            ),
            Self::Circle { radius } => Rect::new(
                position.x - radius,
                position.y - radius,
                radius * 2.0,
                radius * 2.0,
            ),
            Self::Polygon { vertices } => {
                let mut min_x = f32::MAX;
                let mut min_y = f32::MAX;
                let mut max_x = f32::MIN;
                let mut max_y = f32::MIN;
                for v in vertices {
                    min_x = min_x.min(v.x);
                    min_y = min_y.min(v.y);
                    max_x = max_x.max(v.x);
                    max_y = max_y.max(v.y);
                }
                Rect::new(min_x, min_y, max_x - min_x, max_y - min_y)
            }
        }
    }
}

/// 碰撞信息
#[derive(Debug, Clone)]
pub struct CollisionInfo {
    pub entity_a: PhysicsEntity,
    pub entity_b: PhysicsEntity,
    pub normal: Vec2,
    pub penetration: f32,
    pub contact_point: Vec2,
}

/// 物理世界 trait
pub trait PhysicsWorld {
    /// 添加刚体
    fn add_body(&mut self, entity: PhysicsEntity, body: RigidBody);

    /// 移除刚体
    fn remove_body(&mut self, entity: PhysicsEntity);

    /// 添加碰撞体
    fn add_collider(&mut self, entity: PhysicsEntity, collider: Collider);

    /// 移除碰撞体
    fn remove_collider(&mut self, entity: PhysicsEntity);

    /// 获取刚体
    fn get_body(&self, entity: PhysicsEntity) -> Option<&RigidBody>;

    /// 获取可变刚体
    fn get_body_mut(&mut self, entity: PhysicsEntity) -> Option<&mut RigidBody>;

    /// 应用力
    fn apply_force(&mut self, entity: PhysicsEntity, force: Vec2);

    /// 应用冲量
    fn apply_impulse(&mut self, entity: PhysicsEntity, impulse: Vec2);

    /// 步进物理模拟
    fn step(&mut self, dt: f32);

    /// 查询点上的实体
    fn query_point(&self, point: Vec2) -> Vec<PhysicsEntity>;

    /// 查询矩形内的实体
    fn query_rect(&self, rect: Rect) -> Vec<PhysicsEntity>;

    /// 获取碰撞列表
    fn collisions(&self) -> &[CollisionInfo];

    /// 检测碰撞
    fn detect_collisions(&mut self);

    /// 解决碰撞
    fn resolve_collisions(&mut self);
}

/// 简单物理世界实现
pub struct SimplePhysicsWorld {
    bodies: std::collections::HashMap<PhysicsEntity, RigidBody>,
    colliders: std::collections::HashMap<PhysicsEntity, Collider>,
    collisions: Vec<CollisionInfo>,
    gravity: Vec2,
}

impl SimplePhysicsWorld {
    pub fn new() -> Self {
        Self {
            bodies: std::collections::HashMap::new(),
            colliders: std::collections::HashMap::new(),
            collisions: Vec::new(),
            gravity: Vec2::new(0.0, 9.81),
        }
    }

    pub fn set_gravity(&mut self, gravity: Vec2) {
        self.gravity = gravity;
    }
}

impl PhysicsWorld for SimplePhysicsWorld {
    fn add_body(&mut self, entity: PhysicsEntity, body: RigidBody) {
        self.bodies.insert(entity, body);
    }

    fn remove_body(&mut self, entity: PhysicsEntity) {
        self.bodies.remove(&entity);
    }

    fn add_collider(&mut self, entity: PhysicsEntity, collider: Collider) {
        self.colliders.insert(entity, collider);
    }

    fn remove_collider(&mut self, entity: PhysicsEntity) {
        self.colliders.remove(&entity);
    }

    fn get_body(&self, entity: PhysicsEntity) -> Option<&RigidBody> {
        self.bodies.get(&entity)
    }

    fn get_body_mut(&mut self, entity: PhysicsEntity) -> Option<&mut RigidBody> {
        self.bodies.get_mut(&entity)
    }

    fn apply_force(&mut self, entity: PhysicsEntity, force: Vec2) {
        if let Some(body) = self.bodies.get_mut(&entity) {
            body.acceleration = body.acceleration + force * (1.0 / body.mass);
        }
    }

    fn apply_impulse(&mut self, entity: PhysicsEntity, impulse: Vec2) {
        if let Some(body) = self.bodies.get_mut(&entity) {
            body.velocity = body.velocity + impulse * (1.0 / body.mass);
        }
    }

    fn step(&mut self, dt: f32) {
        // 更新物理状态
        for body in self.bodies.values_mut() {
            if body.body_type == BodyType::Dynamic {
                // 应用重力
                body.acceleration = body.acceleration + self.gravity * body.gravity_scale;
                // 更新速度
                body.velocity = body.velocity + body.acceleration * dt;
                // 应用摩擦力
                body.velocity = body.velocity * (1.0 - body.friction * dt);
                // 更新位置
                body.position = body.position + body.velocity * dt;
                // 重置加速度
                body.acceleration = Vec2::zero();
            }
        }

        // 检测碰撞
        self.detect_collisions();

        // 解决碰撞
        self.resolve_collisions();
    }

    fn query_point(&self, point: Vec2) -> Vec<PhysicsEntity> {
        let mut result = Vec::new();
        for (entity, body) in &self.bodies {
            if let Some(collider) = self.colliders.get(entity) {
                let bbox = collider.bounding_box(body.position);
                if bbox.contains(&point) {
                    result.push(*entity);
                }
            }
        }
        result
    }

    fn query_rect(&self, rect: Rect) -> Vec<PhysicsEntity> {
        let mut result = Vec::new();
        for (entity, body) in &self.bodies {
            if let Some(collider) = self.colliders.get(entity) {
                let bbox = collider.bounding_box(body.position);
                if bbox.intersects(&rect) {
                    result.push(*entity);
                }
            }
        }
        result
    }

    fn collisions(&self) -> &[CollisionInfo] {
        &self.collisions
    }

    fn detect_collisions(&mut self) {
        self.collisions.clear();
        let entities: Vec<PhysicsEntity> = self.bodies.keys().copied().collect();
        for i in 0..entities.len() {
            for j in (i + 1)..entities.len() {
                let a = entities[i];
                let b = entities[j];
                if let (Some(body_a), Some(body_b)) = (self.bodies.get(&a), self.bodies.get(&b)) {
                    if let (Some(collider_a), Some(collider_b)) =
                        (self.colliders.get(&a), self.colliders.get(&b))
                    {
                        // 简单 AABB 碰撞检测
                        let bbox_a = collider_a.bounding_box(body_a.position);
                        let bbox_b = collider_b.bounding_box(body_b.position);
                        if bbox_a.intersects(&bbox_b) {
                            let normal = (body_b.position - body_a.position).normalize();
                            let penetration = (bbox_a.width.min(bbox_b.width)
                                + bbox_a.height.min(bbox_b.height))
                                / 2.0;
                            let contact_point =
                                (body_a.position + body_b.position) * 0.5;
                            self.collisions.push(CollisionInfo {
                                entity_a: a,
                                entity_b: b,
                                normal,
                                penetration,
                                contact_point,
                            });
                        }
                    }
                }
            }
        }
    }

    fn resolve_collisions(&mut self) {
        // Collect collision pairs to resolve (avoid borrow issues)
        let pairs: Vec<(PhysicsEntity, PhysicsEntity, Vec2, f32)> = self.collisions.iter().map(|c| {
            (c.entity_a, c.entity_b, c.normal, c.penetration)
        }).collect();

        for (entity_a, entity_b, normal, penetration) in pairs {
            let push = penetration * 0.5;

            if normal.x.abs() > normal.y.abs() {
                // Horizontal separation
                if normal.x > 0.0 {
                    if let Some(body) = self.bodies.get_mut(&entity_a) { body.position.x -= push; }
                    if let Some(body) = self.bodies.get_mut(&entity_b) { body.position.x += push; }
                } else {
                    if let Some(body) = self.bodies.get_mut(&entity_a) { body.position.x += push; }
                    if let Some(body) = self.bodies.get_mut(&entity_b) { body.position.x -= push; }
                }
            } else {
                // Vertical separation
                if normal.y > 0.0 {
                    if let Some(body) = self.bodies.get_mut(&entity_a) { body.position.y -= push; }
                    if let Some(body) = self.bodies.get_mut(&entity_b) { body.position.y += push; }
                } else {
                    if let Some(body) = self.bodies.get_mut(&entity_a) { body.position.y += push; }
                    if let Some(body) = self.bodies.get_mut(&entity_b) { body.position.y -= push; }
                }
            }
        }
    }
}

impl Default for SimplePhysicsWorld {
    fn default() -> Self {
        Self::new()
    }
}
