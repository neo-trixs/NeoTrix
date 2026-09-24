//! 数值调参 — game-design-theory（§9 BALANCE）+ game-feel 分级预设的数据层.
//!
//! 公理（§9.5）：所有平衡数进数据，一次只调一个。本模块即那份数据 + 纯函数：
//! - `TierTable`：juice 三档（small/medium/large）阈值与强度，`nt_juice` 消费
//! - `PityCounter`：保底计数器，软化连黑（§9.4：streaks read as cheating）
//! - `power_gap`：体感难度 = 挑战曲线 − 玩家曲线（§9.2，只调 gap）
//! - `soft_cost`：下一步花费 > 其回报，软减速代替墙（§9.2）
//! - `on_cost_curve`：定价选项落到同一成本曲线上（§9.1，偏离即 dominant/junk）
//!
//! 确定性：全部纯函数，RNG 由调用方以种子传入（不断言 wall-clock）。

/// juice 强度档（game-feel F1：按事件重要性分级，全局一致）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JuiceTier { Small, Medium, Large }

/// 单档强度参数：创伤值（0..1，加法叠加后钳制）、顿帧毫秒、粒子数、白闪毫秒
#[derive(Debug, Clone, Copy)]
pub struct TierParams {
    pub trauma: f32,
    pub hitstop_ms: u32,
    pub particles: u32,
    pub flash_ms: u32,
}

/// 分级表：阈值 + 三档参数（默认即调参起点，改数不改码）
#[derive(Debug, Clone)]
pub struct TierTable {
    /// 达到此伤害（或暴击）即 medium
    pub medium_damage: f32,
    pub small: TierParams,
    pub medium: TierParams,
    pub large: TierParams,
}

impl TierTable {
    pub fn default_juice() -> Self {
        Self {
            medium_damage: 15.0,
            small: TierParams { trauma: 0.15, hitstop_ms: 0, particles: 3, flash_ms: 0 },
            medium: TierParams { trauma: 0.40, hitstop_ms: 50, particles: 6, flash_ms: 0 },
            large: TierParams { trauma: 0.80, hitstop_ms: 120, particles: 30, flash_ms: 60 },
        }
    }

    /// 命中 → 档位：击杀恒 large；暴击/重伤 medium；其余 small（F1 比例原则）
    pub fn tier_for(&self, damage: f32, critical: bool, killed: bool) -> (JuiceTier, TierParams) {
        if killed {
            (JuiceTier::Large, self.large)
        } else if critical || damage >= self.medium_damage {
            (JuiceTier::Medium, self.medium)
        } else {
            (JuiceTier::Small, self.small)
        }
    }
}

/// 保底计数器（§9.4）：连续未命中达 `guarantee_after` 次后，下一次必中。
/// `rng01` 为调用方种子流提供的 [0,1) 随机数（模块自身不取随机，保证复现）。
#[derive(Debug, Clone)]
pub struct PityCounter {
    pub misses: u32,
    pub guarantee_after: u32,
}

impl PityCounter {
    pub fn new(guarantee_after: u32) -> Self {
        Self { misses: 0, guarantee_after: guarantee_after.max(1) }
    }

    /// 一次判定：返回是否命中，并推进计数
    pub fn roll(&mut self, rng01: f64, base_rate: f64) -> bool {
        if self.misses >= self.guarantee_after {
            self.misses = 0;
            return true;
        }
        if rng01 < base_rate {
            self.misses = 0;
            true
        } else {
            self.misses += 1;
            false
        }
    }
}

/// 体感难度 = 挑战 − 玩家（§9.2：steer the gap；正=压迫，负=碾压）
pub fn power_gap(player_power: f32, challenge: f32) -> f32 {
    challenge - player_power
}

/// 软减速花费：第 step 步花费 = base × 1.15^step（下一步永远比回报贵一点，不筑墙）
pub fn soft_cost(base: f32, step: u32) -> f32 {
    base * 1.15_f32.powi(step as i32)
}

/// 成本曲线校验（§9.1）：强度应落在 slope×cost+intercept ± tolerance 内
pub fn on_cost_curve(cost: f32, power: f32, slope: f32, intercept: f32, tolerance: f32) -> bool {
    (power - (slope * cost + intercept)).abs() <= tolerance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_routing() {
        let t = TierTable::default_juice();
        assert_eq!(t.tier_for(5.0, false, false).0, JuiceTier::Small);
        assert_eq!(t.tier_for(5.0, true, false).0, JuiceTier::Medium);
        assert_eq!(t.tier_for(20.0, false, false).0, JuiceTier::Medium);
        assert_eq!(t.tier_for(1.0, false, true).0, JuiceTier::Large);
        // small 档无顿帧（F4：顿帧留给 impact）
        assert_eq!(t.tier_for(5.0, false, false).1.hitstop_ms, 0);
    }

    #[test]
    fn pity_guarantees() {
        let mut p = PityCounter::new(3);
        // base 0.0：连黑 3 次后第 4 次必中
        assert!(!p.roll(0.99, 0.0));
        assert!(!p.roll(0.99, 0.0));
        assert!(!p.roll(0.99, 0.0));
        assert!(p.roll(0.99, 0.0));
        // 命中清零
        let mut q = PityCounter::new(5);
        assert!(q.roll(0.01, 0.5));
        assert_eq!(q.misses, 0);
    }

    #[test]
    fn gap_and_cost_math() {
        assert!((power_gap(10.0, 14.0) - 4.0).abs() < 1e-6);
        assert!(soft_cost(100.0, 1) > 100.0);
        assert!(soft_cost(100.0, 2) > soft_cost(100.0, 1));
        assert!(on_cost_curve(3.0, 6.0, 2.0, 0.0, 0.5));
        assert!(!on_cost_curve(3.0, 9.0, 2.0, 0.0, 0.5)); // dominant：超曲线即超模
    }
}
