use super::ecs::SimpleEcs;
use super::components::*;

/// 重力系统 — 非 Static 实体受重力影响
pub fn gravity_system(ecs: &mut SimpleEcs, dt: f32) {
    // 收集所有拥有 Position + Velocity + Gravity 的实体 ID
    let ids: Vec<u64> = ecs.query3::<Position, Velocity, Gravity>()
        .into_iter().map(|(id, _, _, _)| id).collect();

    for id in ids {
        // 跳过 Static 实体
        if ecs.get::<Static>(id).is_some() { continue; }

        // 先读取重力值 (不可变借用), 再修改速度 (可变借用)
        let gravity_scale = ecs.get::<Gravity>(id).map(|g| g.scale).unwrap_or(1.0);
        if let Some(vel) = ecs.get_mut::<Velocity>(id) {
            vel.vy += GRAVITY_ACCEL * gravity_scale * dt;
        }
    }
}

/// 运动系统 — 速度 × 时间 → 位置
pub fn movement_system(ecs: &mut SimpleEcs, dt: f32) {
    let ids: Vec<u64> = ecs.query2::<Position, Velocity>()
        .into_iter().map(|(id, _, _)| id).collect();

    for id in ids {
        if ecs.get::<Static>(id).is_some() { continue; }

        let (vx, vy) = {
            match ecs.get::<Velocity>(id) {
                Some(v) => (v.vx, v.vy),
                None => continue,
            }
        };
        if let Some(pos) = ecs.get_mut::<Position>(id) {
            pos.x += vx * dt;
            pos.y += vy * dt;
        }
    }
}

/// 摩擦系统 — 非重力实体速度衰减
pub fn friction_system(ecs: &mut SimpleEcs, _dt: f32) {
    let ids: Vec<u64> = ecs.query1::<Velocity>()
        .into_iter().map(|(id, _)| id).collect();

    for id in ids {
        if ecs.get::<Static>(id).is_some() { continue; }
        // 有 Gravity 的实体不衰减 (空中无摩擦)
        if ecs.get::<Gravity>(id).is_some() { continue; }

        if let Some(vel) = ecs.get_mut::<Velocity>(id) {
            vel.vx *= FRICTION;
            vel.vy *= FRICTION;
            // 低速停止
            if vel.vx.abs() < 1.0 { vel.vx = 0.0; }
            if vel.vy.abs() < 1.0 { vel.vy = 0.0; }
        }
    }
}

/// 速度钳制 — 限制最大速度 (Physics.max_speed)
pub fn speed_clamp_system(ecs: &mut SimpleEcs) {
    let ids: Vec<u64> = ecs.query2::<Velocity, Physics>()
        .into_iter().map(|(id, _, _)| id).collect();

    for id in ids {
        let max_speed = match ecs.get::<Physics>(id) {
            Some(p) => p.max_speed,
            None => continue,
        };
        if let Some(vel) = ecs.get_mut::<Velocity>(id) {
            let spd = (vel.vx * vel.vx + vel.vy * vel.vy).sqrt();
            if spd > max_speed && spd > 0.0 {
                vel.vx *= max_speed / spd;
                vel.vy *= max_speed / spd;
            }
        }
    }
}

/// AABB 碰撞检测 — 借鉴 Rust Roguelike Tutorial
pub struct CollisionPair {
    pub a: u64,
    pub b: u64,
    pub overlap_x: f32,
    pub overlap_y: f32,
}

/// 检测所有碰撞对
pub fn collision_detect(ecs: &SimpleEcs) -> Vec<CollisionPair> {
    let entities: Vec<(u64, Position, Collider)> = ecs.query2::<Position, Collider>()
        .into_iter()
        .filter(|(id, _, _)| ecs.get::<Static>(*id).is_none()) // 只检测动态物体
        .map(|(id, p, c)| (id, p.clone(), c.clone()))
        .collect();

    let mut pairs = Vec::new();
    for i in 0..entities.len() {
        for j in (i + 1)..entities.len() {
            let (id_a, pos_a, col_a) = &entities[i];
            let (id_b, pos_b, col_b) = &entities[j];

            // AABB 重叠检测
            let a_left = pos_a.x - col_a.w / 2.0;
            let a_right = pos_a.x + col_a.w / 2.0;
            let a_top = pos_a.y - col_a.h / 2.0;
            let a_bottom = pos_a.y + col_a.h / 2.0;

            let b_left = pos_b.x - col_b.w / 2.0;
            let b_right = pos_b.x + col_b.w / 2.0;
            let b_top = pos_b.y - col_b.h / 2.0;
            let b_bottom = pos_b.y + col_b.h / 2.0;

            if a_left < b_right && a_right > b_left && a_top < b_bottom && a_bottom > b_top {
                // 计算最小穿透深度
                let overlap_x = (a_right - b_left).min(b_right - a_left);
                let overlap_y = (a_bottom - b_top).min(b_bottom - a_top);
                pairs.push(CollisionPair { a: *id_a, b: *id_b, overlap_x, overlap_y });
            }
        }
    }
    pairs
}

/// 碰撞响应 — 位置修正 + 速度反弹
pub fn collision_resolve(ecs: &mut SimpleEcs, pairs: &[CollisionPair]) {
    for pair in pairs {
        // 获取实体类型
        let a_static = ecs.get::<Static>(pair.a).is_some();
        let b_static = ecs.get::<Static>(pair.b).is_some();

        match (a_static, b_static) {
            (true, true) => continue, // 两个静态物体不处理
            (true, false) => {
                // A 静态, B 动态 — 推 B 出去
                let (push_x, push_y) = compute_push_direction(ecs, pair.a, pair.b, pair);
                if let Some(pos) = ecs.get_mut::<Position>(pair.b) {
                    pos.x += push_x;
                    pos.y += push_y;
                }
                // 反弹 B 的速度
                if let Some(vel) = ecs.get_mut::<Velocity>(pair.b) {
                    if push_x.abs() > push_y.abs() {
                        vel.vx = -vel.vx * 0.5;
                    } else {
                        vel.vy = -vel.vy * 0.5;
                    }
                }
            }
            (false, true) => {
                // A 动态, B 静态 — 推 A 出去
                let (push_x, push_y) = compute_push_direction(ecs, pair.b, pair.a, pair);
                if let Some(pos) = ecs.get_mut::<Position>(pair.a) {
                    pos.x += push_x;
                    pos.y += push_y;
                }
                if let Some(vel) = ecs.get_mut::<Velocity>(pair.a) {
                    if push_x.abs() > push_y.abs() {
                        vel.vx = -vel.vx * 0.5;
                    } else {
                        vel.vy = -vel.vy * 0.5;
                    }
                }
            }
            (false, false) => {
                // 两个动态物体 — 各推一半
                let push_x = pair.overlap_x / 2.0;
                let push_y = pair.overlap_y / 2.0;
                let (dir_x, dir_y) = compute_push_direction_raw(ecs, pair.a, pair.b);
                if let Some(pos) = ecs.get_mut::<Position>(pair.a) {
                    pos.x -= dir_x * push_x;
                    pos.y -= dir_y * push_y;
                }
                if let Some(pos) = ecs.get_mut::<Position>(pair.b) {
                    pos.x += dir_x * push_x;
                    pos.y += dir_y * push_y;
                }
            }
        }
    }
}

/// 计算推方向 — 从 static → dynamic
fn compute_push_direction(ecs: &SimpleEcs, static_id: u64, dynamic_id: u64, pair: &CollisionPair) -> (f32, f32) {
    let static_pos = ecs.get::<Position>(static_id);
    let dynamic_pos = ecs.get::<Position>(dynamic_id);
    match (static_pos, dynamic_pos) {
        (Some(sp), Some(dp)) => {
            let dx = dp.x - sp.x;
            let dy = dp.y - sp.y;
            if dx.abs() > dy.abs() {
                (pair.overlap_x * dx.signum(), 0.0)
            } else {
                (0.0, pair.overlap_y * dy.signum())
            }
        }
        _ => (0.0, 0.0),
    }
}

fn compute_push_direction_raw(ecs: &SimpleEcs, id_a: u64, id_b: u64) -> (f32, f32) {
    let pos_a = ecs.get::<Position>(id_a);
    let pos_b = ecs.get::<Position>(id_b);
    match (pos_a, pos_b) {
        (Some(a), Some(b)) => {
            let dx = b.x - a.x;
            let dy = b.y - a.y;
            let len = (dx * dx + dy * dy).sqrt();
            if len > 0.0 { (dx / len, dy / len) } else { (0.0, 0.0) }
        }
        _ => (0.0, 0.0),
    }
}
