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

/// 二维相机变换 — 中心 `(x, y)` + 缩放 + 旋转 + 震屏（trauma 模型）.
/// 纯 f32 元组数学，无头可测；`zoom` 为倍率，`rotation` 为弧度（逆时针为正）.
#[derive(Debug, Clone, Copy)]
pub struct Camera2D {
    pub x: f32,
    pub y: f32,
    pub zoom: f32,
    pub rotation: f32,
    trauma: f32,
    t: f32,
}

impl Camera2D {
    /// 新建相机：`zoom=1`（无缩放），`rotation=0`（无旋转），`trauma=0`（无震动）.
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y, zoom: 1.0, rotation: 0.0, trauma: 0.0, t: 0.0 }
    }

    /// 世界→屏幕（相对坐标系，原点即相机中心）：平移（以 `x,y` 为中心）→旋转→缩放.
    /// 旋向约定：数学逆时针为正（右手系），`rotation=+90°` 时 `(1,0)->(0,1)`.
    pub fn world_to_screen(&self, wx: f32, wy: f32) -> (f32, f32) {
        let dx = wx - self.x;
        let dy = wy - self.y;
        let (s, c) = self.rotation.sin_cos();
        let rx = dx * c - dy * s;
        let ry = dx * s + dy * c;
        (rx * self.zoom, ry * self.zoom)
    }

    /// 屏幕→世界：`world_to_screen` 的逆变换；`zoom<=0` 时按 `1.0` 处理防除零.
    pub fn screen_to_world(&self, sx: f32, sy: f32) -> (f32, f32) {
        let z = if self.zoom <= 0.0 { 1.0 } else { self.zoom };
        let dx = sx / z;
        let dy = sy / z;
        let (s, c) = self.rotation.sin_cos();
        // 逆旋转 R(-r)：[c, s; -s, c]
        (self.x + dx * c + dy * s, self.y + (-dx * s) + dy * c)
    }

    /// 累加震动强度后钳制到 `[0,1]`（trauma 模型：1=满震）.
    pub fn add_trauma(&mut self, amount: f32) {
        self.trauma = (self.trauma + amount).clamp(0.0, 1.0);
    }

    /// 时间步进：`dt<=0` 不动；`trauma` 按 `1.2/s` 线性衰减到 `0`；`t` 累积供震动采样.
    pub fn update(&mut self, dt: f32) {
        if dt <= 0.0 {
            return;
        }
        self.t += dt;
        self.trauma = (self.trauma - 1.2 * dt).max(0.0);
    }

    /// 震动偏移：`trauma^2 * 14px`，x 轴 `sin(t*47)`、y 轴 `cos(t*39)`；`trauma=0` 返回 `(0,0)`.
    pub fn shake_offset(&self) -> (f32, f32) {
        if self.trauma <= 0.0 {
            return (0.0, 0.0);
        }
        let mag = self.trauma * self.trauma * 14.0;
        (mag * (self.t * 47.0).sin(), mag * (self.t * 39.0).cos())
    }

    /// 设置缩放倍率，钳制到 `[0.25, 4.0]`（防翻转/防爆像素）.
    pub fn set_zoom(&mut self, z: f32) {
        self.zoom = z.clamp(0.25, 4.0);
    }
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

    #[test]
    fn cam_roundtrip_identity() {
        // zoom=1/rot=0 下 world<->screen 往返一致（中心平移而已）
        let cam = Camera2D::new(10.0, -20.0);
        let (sx, sy) = cam.world_to_screen(13.0, -16.0);
        assert!((sx - 3.0).abs() < 1e-5 && (sy - 4.0).abs() < 1e-5);
        let (wx, wy) = cam.screen_to_world(sx, sy);
        assert!((wx - 13.0).abs() < 1e-4 && (wy + 16.0).abs() < 1e-4);
    }

    #[test]
    fn cam_zoom2_scales() {
        // zoom=2：相对偏移翻倍
        let mut cam = Camera2D::new(0.0, 0.0);
        cam.set_zoom(2.0);
        let (sx, sy) = cam.world_to_screen(3.0, 4.0);
        assert!((sx - 6.0).abs() < 1e-5 && (sy - 8.0).abs() < 1e-5);
        let (wx, wy) = cam.screen_to_world(sx, sy);
        assert!((wx - 3.0).abs() < 1e-4 && (wy - 4.0).abs() < 1e-4);
    }

    #[test]
    fn cam_rot90_ccw_maps_x_to_y() {
        // 旋向约定：数学逆时针为正（右手系）；rotation=+90° 时 (1,0)->(0,1)
        use core::f32::consts::FRAC_PI_2;
        let mut cam = Camera2D::new(0.0, 0.0);
        cam.rotation = FRAC_PI_2;
        let (sx, sy) = cam.world_to_screen(1.0, 0.0);
        assert!(sx.abs() < 1e-5 && (sy - 1.0).abs() < 1e-5);
        // 逆变换能回来
        let (wx, wy) = cam.screen_to_world(sx, sy);
        assert!((wx - 1.0).abs() < 1e-4 && wy.abs() < 1e-4);
    }

    #[test]
    fn cam_trauma_decays_and_offset_zeroes() {
        // trauma 按 1.2/s 衰减到 0，且 offset 归零
        let mut cam = Camera2D::new(0.0, 0.0);
        cam.add_trauma(1.0);
        assert!(cam.shake_offset() != (0.0, 0.0) || cam.t == 0.0); // t=0 时 sin=0，允许零向量
        cam.add_trauma(10.0); // 钳制到 1
        cam.update(1.0); // 1.2/s * 1s → 归零
        assert_eq!(cam.shake_offset(), (0.0, 0.0));
        // 部分衰减：0.5 - 1.2*0.1 = 0.38
        let mut cam2 = Camera2D::new(0.0, 0.0);
        cam2.add_trauma(0.5);
        cam2.update(0.1);
        let (ox, oy) = cam2.shake_offset();
        let mag = 0.38 * 0.38 * 14.0;
        // 双轴频率不同（47/39），故按分量各自以 mag 为界，而非以合向量为界
        assert!(ox.abs() <= mag + 1e-4 && oy.abs() <= mag + 1e-4);
        assert!(ox.abs() + oy.abs() > 0.0);
        // dt<=0 不动
        let before = cam2.shake_offset();
        cam2.update(0.0);
        cam2.update(-1.0);
        assert_eq!(cam2.shake_offset(), before);
    }

    #[test]
    fn cam_bad_zoom_never_explodes() {
        // 非法 zoom：set_zoom 钳制到 [0.25,4.0]；直接写 0/负数时逆变换按 1.0 处理
        let mut cam = Camera2D::new(0.0, 0.0);
        cam.set_zoom(0.0);
        assert!((cam.zoom - 0.25).abs() < 1e-6);
        cam.set_zoom(-3.0);
        assert!((cam.zoom - 0.25).abs() < 1e-6);
        cam.set_zoom(100.0);
        assert!((cam.zoom - 4.0).abs() < 1e-6);
        // 防除零：zoom 被外部直接置 0/负数（字段 pub），逆变换仍可用、不出 inf/nan
        cam.zoom = 0.0;
        let (wx, wy) = cam.screen_to_world(5.0, -3.0);
        assert!(wx.is_finite() && wy.is_finite());
        assert!((wx - 5.0).abs() < 1e-5 && (wy + 3.0).abs() < 1e-5);
        cam.zoom = -2.0;
        let (wx2, wy2) = cam.screen_to_world(5.0, -3.0);
        assert!(wx2.is_finite() && wy2.is_finite());
    }
}
