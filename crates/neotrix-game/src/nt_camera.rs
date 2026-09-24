//! 相机死区 — Phaser/Celeste canon：目标在死区盒内相机静止，出盒才跟（lerp 保留）.
//!
//! 公理：贴脸跟随每步必抖（queasy 帧）；死区取屏宽 15–25%（1280 下半宽 140≈22%）；
//! 前视（lookahead 8–15%）是下一步，本模块只做死区。纯 f32，无头可测。

/// 死区盒（半尺寸）+ 平滑系数（与旧跟随同系数 5.0，手感连续）
#[derive(Debug, Clone, Copy)]
pub struct Deadzone {
    pub half_w: f32,
    pub half_h: f32,
    pub smooth: f32,
}

impl Deadzone {
    pub fn platformer() -> Self {
        Self { half_w: 140.0, half_h: 100.0, smooth: 5.0 }
    }
}

/// 跟随一步：只把相机推到"目标刚好压盒边"的位置，再按 smooth*dt 插值。
/// 盒内 → 原地不动（零抖动）；dt=0 → 不动。
pub fn follow(cam: (f32, f32), target: (f32, f32), dt: f32, dz: &Deadzone) -> (f32, f32) {
    follow_lookahead(cam, target, (0.0, 0.0), dt, dz, 0.0)
}

/// 前视跟随（Celeste 量级 8–15% 屏宽；1280 下取 120px）：目标点 = 位置 + 速度方向×dist，
/// 同样进死区盒——停下时若已在盒内则保持（死区本意，不晕），天然无跳变。
/// `vel` 为目标速度（静止即普通死区跟随）。
pub fn follow_lookahead(
    cam: (f32, f32),
    target: (f32, f32),
    vel: (f32, f32),
    dt: f32,
    dz: &Deadzone,
    dist: f32,
) -> (f32, f32) {
    // 单轴：超出 +half 推右，超出 -half 推左，盒内推 0
    let push = |c: f32, t: f32, half: f32| {
        c + (t - c - half).max(0.0) + (t - c + half).min(0.0)
    };
    let s = (dz.smooth * dt).clamp(0.0, 1.0);
    let sp = (vel.0 * vel.0 + vel.1 * vel.1).sqrt();
    let (lx, ly) = if sp > 1.0 && dist > 0.0 {
        (vel.0 / sp * dist, vel.1 / sp * dist)
    } else {
        (0.0, 0.0)
    };
    let wx = push(cam.0, target.0 + lx, dz.half_w);
    let wy = push(cam.1, target.1 + ly, dz.half_h);
    (cam.0 + (wx - cam.0) * s, cam.1 + (wy - cam.1) * s)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dz() -> Deadzone {
        Deadzone::platformer()
    }

    #[test]
    fn inside_box_holds_still() {
        // 盒内小范围移动：相机纹丝不动
        assert_eq!(follow((0.0, 0.0), (50.0, 30.0), 0.016, &dz()), (0.0, 0.0));
        assert_eq!(follow((100.0, 100.0), (100.0, 100.0), 0.016, &dz()), (100.0, 100.0));
    }

    #[test]
    fn outside_pushes_single_axis() {
        // 出右盒：只动 x，且向目标方向
        let (cx, cy) = follow((0.0, 0.0), (200.0, 0.0), 1.0, &dz());
        assert!(cx > 0.0 && cy == 0.0);
        // 出上盒：只动 y
        let (cx2, cy2) = follow((0.0, 0.0), (0.0, -150.0), 1.0, &dz());
        assert!(cx2 == 0.0 && cy2 < 0.0);
    }

    #[test]
    fn converges_and_dt_zero_holds() {
        // 长跟随收敛到盒边（非目标中心——死区本意）
        let (mut cx, mut cy) = (0.0, 0.0);
        for _ in 0..300 {
            (cx, cy) = follow((cx, cy), (500.0, 0.0), 0.016, &dz());
        }
        assert!((cx - (500.0 - 140.0)).abs() < 1.0); // 停在左盒边
        assert_eq!(follow((cx, cy), (500.0, 0.0), 0.0, &dz()), (cx, cy)); // dt=0 不动
    }

    #[test]
    fn lookahead_static_equals_plain() {
        // 静止（vel≈0）即普通死区跟随
        assert_eq!(
            follow_lookahead((0.0, 0.0), (50.0, 0.0), (0.0, 0.0), 0.016, &dz(), 120.0),
            follow((0.0, 0.0), (50.0, 0.0), 0.016, &dz())
        );
    }

    #[test]
    fn lookahead_leads_and_returns() {
        // 右移：收敛点比无前视更靠右（多看前方）
        let (mut cx, _) = (0.0, 0.0);
        for _ in 0..300 {
            (cx, _) = follow_lookahead((cx, 0.0), (500.0, 0.0), (200.0, 0.0), 0.016, &dz(), 120.0);
        }
        assert!((cx - (500.0 + 120.0 - 140.0)).abs() < 2.0);
        // 停下：保持（480 已在无前视死区盒 [360,640] 内，死区本意即静止），无跳变
        let (mut rx, _) = (cx, 0.0);
        let mut prev = rx;
        for _ in 0..300 {
            (rx, _) = follow_lookahead((rx, 0.0), (500.0, 0.0), (0.0, 0.0), 0.016, &dz(), 120.0);
            assert!((rx - prev).abs() < 30.0); // 单帧位移有界，无跳变
            prev = rx;
        }
        assert!((rx - 480.0).abs() < 2.0);
    }
}
