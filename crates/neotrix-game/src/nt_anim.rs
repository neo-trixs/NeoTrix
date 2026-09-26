//! 精灵帧动画 — 吸收 kaplay `SpriteAnims/playAnim` + bevy `TextureAtlasLayout` 思想.
//!
//! kaplay 侧：`sprite(id, {anim}) + playAnim(name)` 按名切帧；
//! bevy 侧：`TextureAtlasLayout` 描述格子，`AnimationPlayer` 按 clip 推进.
//! 本模块只做**数据侧**（无头、可测、无渲染依赖）：`AnimClip` 描述帧序列与
//! 帧率，`Animator` 按 `tick(dt)` 推进并给出当前帧号；渲染侧按帧号切图
//! （图集/多文件均可，游戏侧 `SpriteTex` 配帧图时接线）.
//!
//! 公理：`dt <= 0` 不推进；空帧序列恒指 0；非循环播完停在末帧并 `is_done`.

/// 动画片段（帧号由渲染侧解释：图集格 index 或帧文件序号）.
#[derive(Debug, Clone)]
pub struct AnimClip {
    /// 帧序列（可重复/倒放，逐项解释）.
    pub frames: Vec<u32>,
    /// 每秒帧数（`<= 0` 按静帧处理）.
    pub fps: f32,
    /// 循环.
    pub looping: bool,
}

impl AnimClip {
    /// 单帧静画（缺省安全态）.
    pub fn still(frame: u32) -> Self {
        Self { frames: vec![frame], fps: 0.0, looping: true }
    }

    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }
}

/// 播放器（`tick` 推进，`frame` 取帧）.
#[derive(Debug, Clone)]
pub struct Animator {
    clip: AnimClip,
    t: f32,
    done: bool,
}

impl Animator {
    pub fn new(clip: AnimClip) -> Self {
        Self { clip, t: 0.0, done: false }
    }

    /// 换片段（不同名才重置，避免同 clip 反复 `play` 抖帧）.
    pub fn play(&mut self, clip: AnimClip) {
        if self.clip.frames != clip.frames
            || (self.clip.fps - clip.fps).abs() > f32::EPSILON
            || self.clip.looping != clip.looping
        {
            self.clip = clip;
            self.t = 0.0;
            self.done = false;
        }
    }

    pub fn reset(&mut self) {
        self.t = 0.0;
        self.done = false;
    }

    /// 推进 `dt` 秒（`dt <= 0` / 空序列 / 已播完非循环 → 不动）.
    pub fn tick(&mut self, dt: f32) {
        if dt <= 0.0 || self.clip.is_empty() || self.done {
            return;
        }
        if self.clip.fps <= 0.0 {
            return;
        }
        self.t += dt;
        let span = self.clip.len() as f32 / self.clip.fps;
        if self.t >= span {
            if self.clip.looping {
                self.t %= span;
            } else {
                self.t = span;
                self.done = true;
            }
        }
    }

    /// 当前帧号（空序列 → 0）.
    pub fn frame(&self) -> u32 {
        if self.clip.is_empty() || self.clip.fps <= 0.0 {
            return self.clip.frames.first().copied().unwrap_or(0);
        }
        let span = self.clip.len() as f32 / self.clip.fps;
        let t = if self.done { span } else { self.t.min(span) };
        let mut idx = (t * self.clip.fps) as usize;
        if idx >= self.clip.len() {
            idx = self.clip.len() - 1;
        }
        self.clip.frames[idx]
    }

    pub fn is_done(&self) -> bool {
        self.done
    }

    pub fn elapsed(&self) -> f32 {
        self.t
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loops_over_frames() {
        let mut a = Animator::new(AnimClip { frames: vec![0, 1, 2, 3], fps: 4.0, looping: true });
        assert_eq!(a.frame(), 0);
        a.tick(0.25);
        assert_eq!(a.frame(), 1);
        a.tick(0.75); // t=1.0 → 正好一轮
        assert_eq!(a.frame(), 0);
    }

    #[test]
    fn once_stops_at_last_frame() {
        let mut a = Animator::new(AnimClip { frames: vec![5, 6], fps: 2.0, looping: false });
        a.tick(10.0);
        assert!(a.is_done());
        assert_eq!(a.frame(), 6);
        a.tick(10.0); // 播完不再动
        assert_eq!(a.frame(), 6);
    }

    #[test]
    fn empty_and_still_are_safe() {
        let mut a = Animator::new(AnimClip { frames: vec![], fps: 4.0, looping: true });
        a.tick(1.0);
        assert_eq!(a.frame(), 0);
        let mut s = Animator::new(AnimClip::still(7));
        s.tick(99.0);
        assert_eq!(s.frame(), 7);
        assert!(!s.is_done());
    }

    #[test]
    fn non_positive_dt_never_advances() {
        let mut a = Animator::new(AnimClip { frames: vec![0, 1], fps: 2.0, looping: true });
        a.tick(0.0);
        a.tick(-1.0);
        assert_eq!(a.frame(), 0);
        assert_eq!(a.elapsed(), 0.0);
    }

    #[test]
    fn replay_same_clip_keeps_phase() {
        let mut a = Animator::new(AnimClip { frames: vec![0, 1, 2, 3], fps: 4.0, looping: true });
        a.tick(0.5);
        assert_eq!(a.frame(), 2);
        a.play(AnimClip { frames: vec![0, 1, 2, 3], fps: 4.0, looping: true });
        assert_eq!(a.frame(), 2); // 同片段不重置，防抖帧
        a.play(AnimClip { frames: vec![9], fps: 4.0, looping: true });
        assert_eq!(a.frame(), 9);
    }
}
