// Tween 缓动系统 + 打击感
// Ease 函数 / 通用 Tween 插值器 / HitFlash 受击闪白系统

use super::ecs::SimpleEcs;

/// 缓动类型
/// （生产当前仅用 QuadOut；全表保留供单测 + 未来 UI 动画选型）
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ease {
    Linear,
    QuadIn, QuadOut, QuadInOut,
    CubicOut,
    SineInOut,
    BackOut,
    BounceOut,
}

/// 缓动求值 — t ∈ [0,1] → [0,1]
pub fn ease(e: Ease, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    match e {
        Ease::Linear => t,
        Ease::QuadIn => t * t,
        Ease::QuadOut => 1.0 - (1.0 - t) * (1.0 - t),
        Ease::QuadInOut => {
            if t < 0.5 { 2.0 * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(2) / 2.0 }
        }
        Ease::CubicOut => 1.0 - (1.0 - t).powi(3),
        Ease::SineInOut => -(std::f32::consts::PI * t).cos() / 2.0 + 0.5,
        Ease::BackOut => {
            let c1 = 1.70158;
            let c3 = c1 + 1.0;
            1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
        }
        Ease::BounceOut => {
            let (n1, d1) = (7.5625, 2.75);
            if t < 1.0 / d1 { n1 * t * t }
            else if t < 2.0 / d1 { n1 * (t - 1.5 / d1) * (t - 1.5 / d1) + 0.75 }
            else if t < 2.5 / d1 { n1 * (t - 2.25 / d1) * (t - 2.25 / d1) + 0.9375 }
            else { n1 * (t - 2.625 / d1) * (t - 2.625 / d1) + 0.984375 }
        }
    }
}

/// 通用 Tween 插值器 — UI/特效数值动画 (HP平滑、标题淡入、通知滑动)
/// （生产当前走 ease() 直调；插值器保留供单测 + 未来标题/通知动画用）
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Tween {
    pub from: f32,
    pub to: f32,
    pub duration: f32,
    pub elapsed: f32,
    pub ease: Ease,
}

#[allow(dead_code)]
impl Tween {
    pub fn new(from: f32, to: f32, duration: f32, ease: Ease) -> Self {
        Self { from, to, duration: duration.max(0.0001), elapsed: 0.0, ease }
    }

    /// 推进 dt，返回当前值；完成后返回 None
    pub fn update(&mut self, dt: f32) -> Option<f32> {
        self.elapsed += dt;
        if self.elapsed >= self.duration {
            return None;
        }
        Some(self.value())
    }

    pub fn value(&self) -> f32 {
        let t = (self.elapsed / self.duration).clamp(0.0, 1.0);
        self.from + (self.to - self.from) * ease(self.ease, t)
    }

    pub fn is_done(&self) -> bool { self.elapsed >= self.duration }
    pub fn reset(&mut self, from: f32, to: f32) {
        self.from = from; self.to = to; self.elapsed = 0.0;
    }
}

/// 受击闪白组件 — 渲染层读 t/duration 做白色叠加
#[derive(Debug, Clone)]
pub struct HitFlash {
    pub t: f32,
    pub duration: f32,
}

impl HitFlash {
    pub fn new(duration: f32) -> Self { Self { t: duration, duration } }
    /// 闪白强度 0~1
    pub fn intensity(&self) -> f32 { (self.t / self.duration).clamp(0.0, 1.0) }
}

/// HitFlash 衰减系统 — 每帧调用
pub fn tick_hit_flash(ecs: &mut SimpleEcs, dt: f32) {
    let ids: Vec<u64> = ecs.query1_ids::<HitFlash>();
    for id in ids {
        if let Some(h) = ecs.get_mut::<HitFlash>(id) {
            h.t -= dt;
        }
    }
}

/// 触发受击闪白 (不存在则创建)
pub fn flash(ecs: &mut SimpleEcs, id: u64, duration: f32) {
    if !ecs.is_alive(id) { return; }
    if let Some(h) = ecs.get_mut::<HitFlash>(id) {
        h.t = duration;
        h.duration = duration;
    } else {
        ecs.insert(id, HitFlash::new(duration));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ease_boundaries() {
        for e in [Ease::Linear, Ease::QuadIn, Ease::QuadOut, Ease::QuadInOut,
                  Ease::CubicOut, Ease::SineInOut, Ease::BackOut, Ease::BounceOut] {
            assert!((ease(e, 0.0) - 0.0).abs() < 0.001, "{:?} t=0", e);
            assert!((ease(e, 1.0) - 1.0).abs() < 0.001, "{:?} t=1", e);
        }
    }

    #[test]
    fn tween_completes() {
        let mut tw = Tween::new(0.0, 100.0, 1.0, Ease::Linear);
        assert!(tw.update(0.5).is_some());
        assert!(tw.update(0.6).is_none()); // 超时完成
    }

    #[test]
    fn tween_mid_value() {
        let mut tw = Tween::new(0.0, 100.0, 1.0, Ease::Linear);
        let v = tw.update(0.5).unwrap();
        assert!((v - 50.0).abs() < 0.01);
    }
}
