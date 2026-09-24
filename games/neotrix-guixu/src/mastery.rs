//! 修炼 — 用进废退（太吾修炼度精神）：一切能力从实际中来.
//!
//! 公理：挥兵器涨兵器、挨打涨体魄、命中涨臂力；升级只加数值/开招式，
//! 不改操作；阈值表即数据。纯逻辑，调用方每行止损（produce→consume）。

/// 兵器熟练：每出招 +1 use；升级需求 6 + 6×level uses
#[derive(Debug, Clone, Copy)]
pub struct WeaponMastery {
    pub uses: u32,
    pub level: u32,
}

impl WeaponMastery {
    pub fn new() -> Self {
        Self { uses: 0, level: 1 }
    }

    pub fn need(level: u32) -> u32 {
        6 + level * 6
    }

    /// 出招一次；返回升级后的新等级（未升级 None）
    pub fn add_use(&mut self) -> Option<u32> {
        self.uses += 1;
        if self.uses >= Self::need(self.level) {
            self.uses = 0;
            self.level += 1;
            Some(self.level)
        } else {
            None
        }
    }
}

impl Default for WeaponMastery {
    fn default() -> Self {
        Self::new()
    }
}

/// 属性 track：每攒满 step 点经验升 1 级（体魄/臂力共用）
#[derive(Debug, Clone, Copy)]
pub struct AttrTrack {
    pub exp: u32,
    pub level: u32,
    pub step: u32,
}

impl AttrTrack {
    pub fn new(step: u32) -> Self {
        Self { exp: 0, level: 0, step: step.max(1) }
    }

    /// 记经验；返回本次升的级数（0 即无事发生）
    pub fn add(&mut self, n: u32) -> u32 {
        self.exp += n;
        let mut ups = 0;
        while self.exp >= self.step {
            self.exp -= self.step;
            self.level += 1;
            ups += 1;
        }
        ups
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weapon_levels_on_schedule() {
        let mut m = WeaponMastery::new();
        assert_eq!(WeaponMastery::need(1), 12);
        for _ in 0..11 {
            assert_eq!(m.add_use(), None);
        }
        assert_eq!(m.add_use(), Some(2)); // 第 12 次升级
        assert_eq!(m.level, 2);
        assert_eq!(WeaponMastery::need(2), 18);
    }

    #[test]
    fn attr_steps_and_carryover() {
        let mut a = AttrTrack::new(40);
        assert_eq!(a.add(25), 0);
        assert_eq!(a.add(20), 1); // 45 = 40+5 结转
        assert_eq!((a.level, a.exp), (1, 5));
        assert_eq!(a.add(80), 2); // 85 → 2 级余 5
        assert_eq!((a.level, a.exp), (3, 5));
    }
}
