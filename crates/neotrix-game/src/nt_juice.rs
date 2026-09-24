//! 打击感 — game-feel（trauma 震屏 / 顿帧 / 缓动 / 分级 bundle）.
//!
//! 公理（gamma 5–8 微反馈/100ms；juice 瞬态必回静；按重要性分级；不动关键模拟；不阻塞输入）：
//! - `Juice::tick(real_dt) -> sim_dt`：顿帧期返回 0.0（仿真冻结），计时器走**真实时间**
//!   （F 坑：scaled timer 永不恢复）；创伤按 1.2/s 衰减；相位推进供震屏噪声采样
//! - `shake_offset()`：trauma² × 上限 × 正弦采样（F 坑：逐帧随机=静电噪声；
//!   只动世界相机，HUD 零位移——调用方在 render 世界层包相机）
//! - `trigger(params)`：`neotrix_abilities::nt_tuning::TierTable` 分档消费
//!
//! 本模块零 macroquad 依赖（纯 f32），单测无头可跑；相机装配由 render.rs 完成。
//! 缓动曲线不另起炉灶：pop（过冲）= `tween::ease(BackOut)`，settle = `tween::ease(CubicOut)`
//!（game-feel F4：线性=机械；曲线单一事实源在 tween.rs）。

use neotrix_abilities::nt_tuning::TierParams;

/// 创伤震屏上限（skill 默认 12,8）与衰减（skill 默认 1.2/s）
pub const SHAKE_MAX_X: f32 = 12.0;
pub const SHAKE_MAX_Y: f32 = 8.0;
pub const TRAUMA_DECAY: f32 = 1.2;
/// 顿帧安全上限（F 坑：长冻结阻塞输入；超限钳制）
pub const FREEZE_MAX_SECS: f32 = 0.25;

#[derive(Debug, Clone)]
pub struct Juice {
    trauma: f32,
    freeze_left: f32,
    phase: f32,
}

impl Juice {
    pub fn new() -> Self {
        Self { trauma: 0.0, freeze_left: 0.0, phase: 0.0 }
    }

    /// 当前创伤（0..1，debug overlay 显示用）
    pub fn trauma(&self) -> f32 {
        self.trauma
    }

    /// 是否处于顿帧
    pub fn frozen(&self) -> bool {
        self.freeze_left > 0.0
    }

    /// 创伤累加（hits ADD，不 reset），钳制 0..1
    pub fn add_trauma(&mut self, amount: f32) {
        self.trauma = (self.trauma + amount).clamp(0.0, 1.0);
    }

    /// 顿帧（秒），超限钳制防滥用
    pub fn freeze(&mut self, secs: f32) {
        if secs > 0.0 {
            self.freeze_left = (self.freeze_left + secs).min(FREEZE_MAX_SECS);
        }
    }

    /// 每帧调用：入真实 dt，出仿真 dt。冻结期仿真 dt=0（创伤/相位不同步进）。
    pub fn tick(&mut self, real_dt: f32) -> f32 {
        let dt = real_dt.max(0.0);
        if self.freeze_left > 0.0 {
            self.freeze_left = (self.freeze_left - dt).max(0.0);
            return 0.0;
        }
        self.trauma = (self.trauma - TRAUMA_DECAY * dt).max(0.0);
        self.phase += dt * 30.0;
        dt
    }

    /// 震屏偏移（像素）：trauma²（小打几乎不动，大打重拳）× 正弦噪声
    pub fn shake_offset(&self) -> (f32, f32) {
        let s = self.trauma * self.trauma;
        if s <= 0.0 {
            return (0.0, 0.0);
        }
        (
            SHAKE_MAX_X * s * (self.phase * 1.7).sin(),
            SHAKE_MAX_Y * s * (self.phase * 2.3).sin(),
        )
    }

    /// 分级 bundle 入口：trauma + 顿帧（声音/粒子/数字由调用方既有 API 发——
    /// audio.play + particles.spawn_* 已存在，只叠新通道）
    pub fn trigger(&mut self, params: &TierParams) {
        self.add_trauma(params.trauma);
        if params.hitstop_ms > 0 {
            self.freeze(params.hitstop_ms as f32 / 1000.0);
        }
    }
}

impl Default for Juice {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trauma_additive_and_clamped() {
        let mut j = Juice::new();
        j.add_trauma(0.4);
        j.add_trauma(0.4);
        assert!((j.trauma() - 0.8).abs() < 1e-6);
        j.add_trauma(0.5);
        assert!((j.trauma() - 1.0).abs() < 1e-6); // 钳制，不超 1
    }

    #[test]
    fn freeze_holds_sim_and_resumes_on_real_time() {
        let mut j = Juice::new();
        j.add_trauma(0.5);
        j.freeze(0.1);
        assert!(j.frozen());
        // 冻结期：仿真 dt=0，创伤不衰减
        assert_eq!(j.tick(0.016), 0.0);
        assert!((j.trauma() - 0.5).abs() < 1e-6);
        // 真实时间走完 0.1s 后恢复
        assert_eq!(j.tick(0.1), 0.0); // 这一帧仍在冻结算尾
        assert!(!j.frozen());
        let dt = j.tick(0.016);
        assert!(dt > 0.0 && j.trauma() < 0.5); // 恢复推进 + 衰减
    }

    #[test]
    fn freeze_clamped_against_abuse() {
        let mut j = Juice::new();
        j.freeze(10.0);
        assert!(j.freeze_left <= FREEZE_MAX_SECS);
    }

    #[test]
    fn shake_rest_zero_and_bounded() {
        let j = Juice::new();
        assert_eq!(j.shake_offset(), (0.0, 0.0)); // 静息零位移（回静原则）
        let mut k = Juice::new();
        k.add_trauma(1.0);
        // 相位推进若干帧，偏移恒在包络内
        for _ in 0..120 {
            k.tick(1.0 / 60.0);
            let (ox, oy) = k.shake_offset();
            assert!(ox.abs() <= SHAKE_MAX_X && oy.abs() <= SHAKE_MAX_Y);
        }
    }

    #[test]
    fn trigger_applies_tier() {
        use neotrix_abilities::nt_tuning::TierTable;
        let t = TierTable::default_juice();
        let mut j = Juice::new();
        let (_, small) = t.tier_for(5.0, false, false);
        j.trigger(&small);
        assert!(!j.frozen()); // small 无顿帧
        assert!(j.trauma() > 0.0);
        let (_, large) = t.tier_for(5.0, false, true);
        j.trigger(&large);
        assert!(j.frozen()); // large 有顿帧
    }
}
