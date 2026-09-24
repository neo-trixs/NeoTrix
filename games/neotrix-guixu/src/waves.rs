//! 波次导演 — swordgame.ai wave-info/chapter-banner：预算刷怪 + 章节断.
//!
//! 公理：预算 = base + wave×step；弱1/快2/精4 费；精锐 3 波解锁；
//! 每 3 波一章（休整横幅）。纯逻辑，调用方按表 spawn，打空推进。

/// 单怪规格
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FoeSpec {
    pub hp: f32,
    pub speed: f32,
    pub dmg: f32,
    pub elite: bool,
    pub score: u32,
    /// 基础护甲（出生再按波次叠甲）
    pub armor: f32,
}

pub const WEAK: FoeSpec = FoeSpec { hp: 30.0, speed: 90.0, dmg: 8.0, elite: false, score: 10, armor: 0.0 };
pub const FAST: FoeSpec = FoeSpec { hp: 20.0, speed: 150.0, dmg: 6.0, elite: false, score: 15, armor: 0.0 };
pub const ELITE: FoeSpec = FoeSpec { hp: 90.0, speed: 70.0, dmg: 16.0, elite: true, score: 40, armor: 6.0 };

/// 波次预算
pub fn budget(wave: u32) -> u32 {
    3 + wave * 2
}

/// 波次阵容：先塞满弱怪，余预算升级快怪，3 波起掺精锐（费 4）
pub fn composition(wave: u32) -> Vec<FoeSpec> {
    let wave = wave.max(1);
    let mut left = budget(wave) as i32;
    let mut out = Vec::new();
    // 精锐（3 波解锁，每 2 波多一只，上限 3）
    if wave >= 3 {
        let elites = ((wave - 1) / 2).min(3) as i32;
        for _ in 0..elites {
            if left >= 4 {
                out.push(ELITE);
                left -= 4;
            }
        }
    }
    // 快怪：余量偶数位升级
    let mut i = 0;
    while left >= 2 && i < out.len() + 4 {
        out.push(FAST);
        left -= 2;
        i += 1;
    }
    while left >= 1 {
        out.push(WEAK);
        left -= 1;
    }
    // 保底：至少一只
    if out.is_empty() {
        out.push(WEAK);
    }
    out
}

/// 章节：每 3 波一章（1-3 第一章…）
pub fn chapter_of(wave: u32) -> u32 {
    (wave.max(1) - 1) / 3 + 1
}

/// 波次怪强度成长（hp/speed 随波微涨，封顶 2 倍）
pub fn scale_of(wave: u32) -> f32 {
    (1.0 + wave as f32 * 0.06).min(2.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wave1_all_weak() {
        let c = composition(1);
        assert!(!c.is_empty());
        assert!(c.iter().all(|f| !f.elite));
        let cost: i32 = c.iter().map(|f| if f.elite { 4 } else if f.speed > 100.0 { 2 } else { 1 }).sum();
        assert!(cost <= budget(1) as i32);
    }

    #[test]
    fn elites_gated_and_budget_grows() {
        assert!(composition(2).iter().all(|f| !f.elite));
        assert!(composition(3).iter().any(|f| f.elite));
        assert!(composition(5).len() >= composition(1).len());
    }

    #[test]
    fn chapters_and_scale() {
        assert_eq!(chapter_of(1), 1);
        assert_eq!(chapter_of(3), 1);
        assert_eq!(chapter_of(4), 2);
        assert!((scale_of(1) - 1.06).abs() < 1e-6);
        assert_eq!(scale_of(100), 2.0);
    }
}
