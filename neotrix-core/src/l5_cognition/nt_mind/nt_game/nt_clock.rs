//! 定步长时钟 — game-engine（B1：fixed timestep 仿真，渲染插值/可变帧只管呈现）.
//!
//! 公理：固定 dt 仿真 → 确定性（§12.1 同输入同局）；可变 dt 只进渲染。
//! `FixedStepper::advance(real_dt) -> steps`：累积真实时间，按固定步长吐步数，
//! 上限钳制防螺旋死亡（lag spike 不滚雪球）。生产接线：SHOT 审计循环固定 1/60
//! 步进（同场景同帧 ⇒ 同画面）；主循环迁移为下一步（input 轮询在 update 内，
//! 多步/帧会重复采样，需先把 input 移出 update——已记录，不碰）.

/// 固定步长（60Hz）与单帧最大步数（螺旋 guard）
pub const FIXED_DT: f32 = 1.0 / 60.0;
pub const MAX_STEPS: u32 = 3;

#[derive(Debug, Clone)]
pub struct FixedStepper {
    acc: f32,
    step: f32,
    max_steps: u32,
}

impl FixedStepper {
    pub fn new() -> Self {
        Self { acc: 0.0, step: FIXED_DT, max_steps: MAX_STEPS }
    }

    /// 喂真实 dt，返回本帧应跑的仿真步数（0..=max_steps）
    pub fn advance(&mut self, real_dt: f32) -> u32 {
        self.acc += real_dt.max(0.0);
        // 螺旋 guard：积压超上限直接丢弃（慢帧不追帧，只保当下）
        if self.acc > self.step * self.max_steps as f32 {
            self.acc = self.step * self.max_steps as f32;
        }
        let mut n = 0;
        while self.acc >= self.step && n < self.max_steps {
            self.acc -= self.step;
            n += 1;
        }
        n
    }

    /// 步长（调用方 update 用此 dt，保证仿真侧恒定）
    pub fn step(&self) -> f32 {
        self.step
    }
}

impl Default for FixedStepper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sixty_fps_yields_one_step() {
        let mut c = FixedStepper::new();
        assert_eq!(c.advance(1.0 / 60.0), 1);
        assert_eq!(c.advance(1.0 / 60.0), 1);
    }

    #[test]
    fn remainder_carries_over() {
        let mut c = FixedStepper::new();
        assert_eq!(c.advance(1.0 / 120.0), 0); // 半步先存
        assert_eq!(c.advance(1.0 / 120.0), 1); // 凑整一步
    }

    #[test]
    fn spiral_guard_caps_and_drops_backlog() {
        let mut c = FixedStepper::new();
        assert_eq!(c.advance(1.0), MAX_STEPS); // 1s 卡顿：钳制 3 步，不追 60 步
        assert_eq!(c.advance(0.0), 0); // 无残留滚雪球
    }

    #[test]
    fn deterministic_same_input_same_steps() {
        let seq = [0.016, 0.017, 0.033, 0.008];
        let run = || {
            let mut c = FixedStepper::new();
            seq.iter().map(|d| c.advance(*d)).collect::<Vec<_>>()
        };
        assert_eq!(run(), run());
    }
}
