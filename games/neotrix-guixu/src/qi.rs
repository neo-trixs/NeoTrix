//! 氣值 — swordgame.ai 氣：命中回气，轻功/特招耗气.
//!
//! 公理：氣是"战斗 *"（打得越凶资源越多）；花费原子（不足则拒，状态不动）。

#[derive(Debug, Clone)]
pub struct Qi {
    cur: f32,
    max: f32,
}

impl Qi {
    pub fn new(max: f32) -> Self {
        Self { cur: 0.0, max: max.max(1.0) }
    }

    /// 命中回气
    pub fn gain(&mut self, amount: f32) {
        self.cur = (self.cur + amount.max(0.0)).min(self.max);
    }

    /// 预检：够付吗（不改变状态，供出招决策）
    pub fn afford(&self, cost: f32) -> bool {
        cost <= 0.0 || self.cur >= cost
    }

    /// 花费：足额扣减 true，不足 false（状态不动，原子）
    pub fn spend(&mut self, cost: f32) -> bool {
        if cost <= 0.0 {
            return true;
        }
        if self.cur >= cost {
            self.cur -= cost;
            true
        } else {
            false
        }
    }

    pub fn cur(&self) -> f32 {
        self.cur
    }

    pub fn pct(&self) -> f32 {
        (self.cur / self.max).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gain_clamps_and_spend_atomic() {
        let mut q = Qi::new(100.0);
        q.gain(30.0);
        assert_eq!(q.cur(), 30.0);
        assert!(q.spend(20.0));
        assert_eq!(q.cur(), 10.0);
        assert!(!q.spend(50.0)); // 不足拒收
        assert_eq!(q.cur(), 10.0); // 状态不动
        q.gain(1000.0);
        assert_eq!(q.cur(), 100.0); // 钳制
        assert!((q.pct() - 1.0).abs() < 1e-6);
        assert!(q.afford(100.0));
        assert!(!q.afford(100.01));
        assert!(Qi::new(10.0).afford(0.0));
    }
}
