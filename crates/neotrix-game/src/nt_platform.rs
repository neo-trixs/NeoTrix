//! 单向平台物理 — 自 swords 主循环提炼（与 `ecs_systems` 实心碰撞互补）.
//!
//! 公理：顶面单向（可跳穿，只落不停）；调用方自持位置，返回落地与否。
//! 与实心 AABB（推挤+反弹）分工：平台跳跃用本模块，箱体堆叠用 ecs_systems。

/// AABB 重叠判定（中心系）
pub fn overlap(
    ax: f32, ay: f32, aw: f32, ah: f32,
    bx: f32, by: f32, bw: f32, bh: f32,
) -> bool {
    (ax - bx).abs() < (aw + bw) / 2.0 && (ay - by).abs() < (ah + bh) / 2.0
}

/// 平台移动积分：重力+世界钳制+顶面着陆。
/// 着陆条件（防穿隧）：下落中 + 上帧脚在顶上 + 本帧脚过顶 + 横向重叠。
/// `drop` 为真时忽略顶面（跳穿下降，Celeste descent 语义）。
/// 返回是否落地（调用方据此 land()/walk_off()）。
#[allow(clippy::too_many_arguments)]
pub fn move_on_platforms(
    x: &mut f32,
    y: &mut f32,
    vx: f32,
    vy: &mut f32,
    w: f32,
    h: f32,
    plats: &[(f32, f32, f32, f32)],
    dt: f32,
    gravity: f32,
    max_fall: f32,
    world_w: f32,
    drop: bool,
) -> bool {
    *x += vx * dt;
    *x = (*x).clamp(w / 2.0, world_w - w / 2.0);
    let prev_feet = *y + h / 2.0;
    *vy = (*vy + gravity * dt).min(max_fall);
    *y += *vy * dt;
    if *vy < 0.0 || drop {
        return false; // 上升不碰顶（跳穿）；下降忽略（跳穿下降）
    }
    let feet = *y + h / 2.0;
    for &(px, py, pw, _) in plats {
        if prev_feet <= py + 1.0
            && feet >= py
            && *x + w / 2.0 > px
            && *x - w / 2.0 < px + pw
        {
            *y = py - h / 2.0;
            *vy = 0.0;
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLATS: [(f32, f32, f32, f32); 1] = [(100.0, 300.0, 200.0, 24.0)];

    #[test]
    fn falls_and_lands_on_top() {
        let (mut x, mut y, mut vy) = (200.0, 100.0, 0.0);
        let mut landed = false;
        for _ in 0..120 {
            landed = move_on_platforms(&mut x, &mut y, 0.0, &mut vy, 28.0, 44.0,
                &PLATS, 1.0 / 60.0, 1500.0, 900.0, 1280.0, false);
            if landed {
                break;
            }
        }
        assert!(landed);
        assert!((y - (300.0 - 22.0)).abs() < 1.0); // 脚落顶面
        assert_eq!(vy, 0.0);
    }

    #[test]
    fn jumps_through_from_below() {
        // 顶下方上升穿过：不粘顶不落地
        let (mut x, mut y, mut vy) = (200.0, 380.0, -600.0);
        let landed = move_on_platforms(&mut x, &mut y, 0.0, &mut vy, 28.0, 44.0,
            &PLATS, 1.0 / 60.0, 1500.0, 900.0, 1280.0, false);
        assert!(!landed);
        assert!(vy < 0.0); // 上升保持
    }

    #[test]
    fn drop_falls_through_top() {
        // 站立顶面 + drop：直接穿过，不着陆
        let (mut x, mut y, mut vy) = (200.0, 300.0 - 22.0, 0.0);
        let mut landed = true;
        for _ in 0..30 {
            landed = move_on_platforms(&mut x, &mut y, 0.0, &mut vy, 28.0, 44.0,
                &PLATS, 1.0 / 60.0, 1500.0, 900.0, 1280.0, true);
            if !landed && y > 300.0 {
                break;
            }
        }
        assert!(!landed);
        assert!(y > 300.0); // 已穿到顶面之下
    }

    #[test]
    fn walks_off_edge_and_world_clamps() {
        let (mut x, mut y, mut vy) = (5.0, 100.0, 0.0);
        let landed = move_on_platforms(&mut x, &mut y, -500.0, &mut vy, 28.0, 44.0,
            &PLATS, 1.0 / 60.0, 1500.0, 900.0, 1280.0, false);
        assert!(!landed); // 平台外：悬空
        assert!(x >= 14.0); // 世界钳制（w/2）
        // 下落限速
        let (_, mut y2, mut vy2) = (0.0, 0.0, 5000.0);
        move_on_platforms(&mut x, &mut y2, 0.0, &mut vy2, 10.0, 10.0,
            &PLATS, 1.0, 1500.0, 900.0, 1280.0, false);
        assert!(vy2 <= 900.0);
    }

    #[test]
    fn overlap_edges() {
        assert!(overlap(0.0, 0.0, 10.0, 10.0, 5.0, 0.0, 10.0, 10.0));
        assert!(!overlap(0.0, 0.0, 10.0, 10.0, 10.0, 0.0, 10.0, 10.0)); // 相切不算
        assert!(!overlap(0.0, 0.0, 10.0, 10.0, 50.0, 50.0, 10.0, 10.0));
    }
}
