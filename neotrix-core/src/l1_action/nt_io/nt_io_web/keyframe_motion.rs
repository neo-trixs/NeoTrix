//! 关键帧锚定动画生成 (keyframe-anchored motion) — NT-IO 感知行动能力。
//!
//! 吸收 oil-motion 机制: 帧状态定义 (Keyframe) + 插值 (MotionAnimator) +
//! 帧→交互映射 (click_target)。任一时刻的动画面可由相邻关键帧线性插值得出,
//! 插值进度先经 Easing 归一, 供界面渲染与点击命中检测共用。

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Keyframe {
    pub t: f64,
    pub x: f64,
    pub y: f64,
    pub opacity: f64,
    pub rotation: f64,
    pub scale: f64,
}

impl Keyframe {
    /// 原点帧: t=0, 无位移, 完全不透明, 无旋转, 原始尺寸。
    pub fn origin() -> Self {
        Self {
            t: 0.0,
            x: 0.0,
            y: 0.0,
            opacity: 1.0,
            rotation: 0.0,
            scale: 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MotionClip {
    pub id: String,
    pub frames: Vec<Keyframe>,
    pub duration_ms: u64,
    pub loop_count: u32,
}

impl Default for MotionClip {
    fn default() -> Self {
        Self {
            id: String::new(),
            frames: Vec::new(),
            duration_ms: 1000,
            loop_count: 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MotionAnimator {
    pub easing: Easing,
    pub samples_per_ms: u32,
}

impl Default for MotionAnimator {
    fn default() -> Self {
        Self {
            easing: Easing::EaseInOut,
            samples_per_ms: 1,
        }
    }
}

impl MotionAnimator {
    pub fn new(easing: Easing, samples_per_ms: u32) -> Self {
        Self {
            easing,
            samples_per_ms: samples_per_ms.max(1),
        }
    }

    /// 归一化时间 t01 ∈ [0,1] → 缓动后进度。R-P6: 用 max/min 而非 clamp。
    pub fn ease(&self, t01: f64) -> f64 {
        let t = t01.max(0.0).min(1.0);
        match self.easing {
            Easing::Linear => t,
            Easing::EaseIn => t * t,
            Easing::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            Easing::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - 2.0 * (1.0 - t) * (1.0 - t)
                }
            }
        }
    }

    /// 时刻 t_ms 对应关键帧: 定位所在区间 [a,b], 以 ease(局部进度) 插值。
    /// 帧越界 (早于首帧或晚于末帧) 返回 None。
    pub fn sample(&self, clip: &MotionClip, t_ms: u64) -> Option<Keyframe> {
        let frames = &clip.frames;
        if frames.is_empty() {
            return None;
        }
        let first = frames.first()?;
        let last = frames.last()?;
        let t = t_ms as f64;
        if t < first.t || t > last.t {
            return None;
        }
        for pair in frames.windows(2) {
            let a = &pair[0];
            let b = &pair[1];
            if t >= a.t && t <= b.t {
                let span = b.t - a.t;
                let local = if span <= 0.0 { 0.0 } else { (t - a.t) / span };
                return Some(self.interpolate(a, b, self.ease(local)));
            }
        }
        // 单帧 clip: t 恰好落在该帧时间点。
        if t == first.t {
            return Some(*first);
        }
        None
    }

    /// 两帧线性插值, t01 为归一化进度 (应已 ease)。
    pub fn interpolate(&self, a: &Keyframe, b: &Keyframe, t01: f64) -> Keyframe {
        let t = t01.max(0.0).min(1.0);
        let lerp = |va: f64, vb: f64| va + (vb - va) * t;
        Keyframe {
            t: lerp(a.t, b.t),
            x: lerp(a.x, b.x),
            y: lerp(a.y, b.y),
            opacity: lerp(a.opacity, b.opacity),
            rotation: lerp(a.rotation, b.rotation),
            scale: lerp(a.scale, b.scale),
        }
    }

    /// 帧→交互映射: 返回该时刻帧的 (x, y), 用于点击命中检测。
    pub fn click_target(&self, clip: &MotionClip, at_ms: u64) -> (f64, f64) {
        match self.sample(clip, at_ms) {
            Some(f) => (f.x, f.y),
            None => (0.0, 0.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clip() -> MotionClip {
        MotionClip {
            id: "pop".into(),
            frames: vec![
                Keyframe::origin(),
                Keyframe { t: 500.0, x: 100.0, y: 50.0, opacity: 1.0, rotation: 90.0, scale: 1.5 },
                Keyframe { t: 1000.0, x: 200.0, y: 0.0, opacity: 0.0, rotation: 180.0, scale: 0.5 },
            ],
            duration_ms: 1000,
            loop_count: 1,
        }
    }

    #[test]
    fn ease_is_monotone_non_decreasing() {
        let animator = MotionAnimator::new(Easing::EaseInOut, 1);
        let mut prev = 0.0f64;
        for i in 0..=100 {
            let v = animator.ease(i as f64 / 100.0);
            assert!(v >= prev, "ease 在 {} 处下降", i);
            prev = v;
        }
        for easing in [Easing::EaseIn, Easing::EaseOut, Easing::Linear] {
            let a = MotionAnimator::new(easing, 1);
            let mut prev = 0.0f64;
            for i in 0..=100 {
                let v = a.ease(i as f64 / 100.0);
                assert!(v >= prev, "ease {:?} 在 {} 处下降", easing, i);
                prev = v;
            }
        }
    }

    #[test]
    fn ease_linear_is_identity() {
        let animator = MotionAnimator::new(Easing::Linear, 1);
        for i in 0..=100 {
            let t = i as f64 / 100.0;
            assert!((animator.ease(t) - t).abs() < 1e-9);
        }
        assert_eq!(animator.ease(-1.0), 0.0);
        assert_eq!(animator.ease(2.0), 1.0);
    }

    #[test]
    fn ease_bounds_and_midpoint() {
        let a = MotionAnimator::new(Easing::EaseIn, 1);
        assert_eq!(a.ease(0.0), 0.0);
        assert_eq!(a.ease(1.0), 1.0);
        assert!((a.ease(0.5) - 0.25).abs() < 1e-9);
        let e = MotionAnimator::new(Easing::EaseInOut, 1);
        assert!((e.ease(0.5) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn sample_interpolates_within_segment() {
        let animator = MotionAnimator::new(Easing::Linear, 1);
        let c = clip();
        let mid = animator.sample(&c, 250).unwrap();
        // 首帧→第二帧中点: x 从 0 到 100
        assert!((mid.x - 50.0).abs() < 1e-9);
        assert!((mid.y - 25.0).abs() < 1e-9);
        // 首帧 opacity=1, scale 从 1→1.5 中点
        assert!((mid.opacity - 1.0).abs() < 1e-9);
        assert!((mid.scale - 1.25).abs() < 1e-9);
        // 末帧原样返回
        let end = animator.sample(&c, 1000).unwrap();
        assert!((end.x - 200.0).abs() < 1e-9);
        assert!((end.opacity - 0.0).abs() < 1e-9);
    }

    #[test]
    fn sample_out_of_range_returns_none() {
        let animator = MotionAnimator::new(Easing::Linear, 1);
        let c = clip();
        assert!(animator.sample(&c, 1001).is_none());
        assert!(animator.sample(&c, 0).is_some());
        let empty = MotionClip::default();
        assert!(animator.sample(&empty, 0).is_none());
    }

    #[test]
    fn click_target_returns_frame_coordinates() {
        let animator = MotionAnimator::new(Easing::Linear, 1);
        let c = clip();
        let (x, y) = animator.click_target(&c, 0);
        assert!((x - 0.0).abs() < 1e-9 && (y - 0.0).abs() < 1e-9);
        let (x, y) = animator.click_target(&c, 1000);
        assert!((x - 200.0).abs() < 1e-9 && (y - 0.0).abs() < 1e-9);
        // 越界回退原点
        let (x, y) = animator.click_target(&c, 2000);
        assert!((x - 0.0).abs() < 1e-9 && (y - 0.0).abs() < 1e-9);
    }

    #[test]
    fn keyframe_origin_defaults() {
        let o = Keyframe::origin();
        assert_eq!(o, Keyframe { t: 0.0, x: 0.0, y: 0.0, opacity: 1.0, rotation: 0.0, scale: 1.0 });
        let d = MotionClip::default();
        assert_eq!(d.duration_ms, 1000);
        assert_eq!(d.loop_count, 1);
        let a = MotionAnimator::default();
        assert_eq!(a.easing, Easing::EaseInOut);
        assert_eq!(a.samples_per_ms, 1);
    }
}