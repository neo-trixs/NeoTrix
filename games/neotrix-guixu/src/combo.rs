//! 连击识别 — swordgame.ai combo/cg-dots：限时窗口连打计数.
//!
//! 公理：连击是"限时内命中计数"，超时清零并结算 max。
//! 招式识别已由 `arts`（方向序列+门禁）接管，本模块只计数。
//! 纯逻辑，调用方每命中推 `hit`，每帧推 `timeout`.

/// 连击窗口（s）：超时 Honor 连击断
pub const COMBO_WINDOW: f32 = 1.2;

#[derive(Debug, Clone)]
pub struct ComboTracker {
    count: u32,
    max: u32,
    last_t: f32,
    window: f32,
}

impl ComboTracker {
    pub fn new() -> Self {
        Self { count: 0, max: 0, last_t: -999.0, window: COMBO_WINDOW }
    }

    /// 命中：窗内连击+1，窗外重起；返回当前连击数
    pub fn hit(&mut self, now: f32) -> u32 {
        if now - self.last_t > self.window {
            self.count = 1;
        } else {
            self.count += 1;
        }
        self.last_t = now;
        self.max = self.max.max(self.count);
        self.count
    }

    /// 每帧：超时则清零；返回是否刚断（曾连击≥2）
    pub fn timeout(&mut self, now: f32) -> bool {
        if self.count >= 2 && now - self.last_t > self.window {
            self.count = 0;
            return true;
        }
        if self.count < 2 && now - self.last_t > self.window {
            self.count = 0;
        }
        false
    }

    pub fn count(&self) -> u32 {
        self.count
    }

    pub fn max(&self) -> u32 {
        self.max
    }

    pub fn reset(&mut self) {
        self.count = 0;
    }
}

impl Default for ComboTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_chains_and_breaks() {
        let mut c = ComboTracker::new();
        assert_eq!(c.hit(0.0), 1);
        assert_eq!(c.hit(0.5), 2);
        assert_eq!(c.hit(1.0), 3);
        assert_eq!(c.max(), 3);
        assert!(c.timeout(2.5)); // 断连（≥2）
        assert_eq!(c.count(), 0);
        assert_eq!(c.max(), 3); // max 保留
    }

    #[test]
    fn single_hit_no_break_event() {
        let mut c = ComboTracker::new();
        c.hit(0.0);
        assert!(!c.timeout(5.0)); // 孤 hit 不算"断连"
    }

}
