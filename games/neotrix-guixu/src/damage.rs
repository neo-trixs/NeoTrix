//! 伤害管线 — PoE/CS 指标体系提炼（wuxia 动作适配）.
//!
//! 公理（PoE 计算顺序）：
//!   基础 → 攻击方乘区（武器/臂力/招式）→ 情境乘区（背刺/挑空）→
//!   暴击 roll（默认 1.5x）→ 护甲（穿透→PoE 比率式）→ 方差（±10%）→ 保底 1。
//! CS 分区思想：头 4x/胸 1x/腹 1.25x/腿 0.75x；2D 化为背刺 1.5x（身后）
//! 与挑空 1.25x（浮空）；护甲公式 armor/(armor+hit)（大击穿透，小击减免）。
//! 全纯函数，随机数由调用方以 rng01 传入（确定性可复现）。

/// 暴击默认倍率（PoE 1.5x）
pub const CRIT_MULT: f32 = 1.5;
/// 背刺倍率（身后出手）
pub const BACKSTAB_MULT: f32 = 1.5;
/// 挑空倍率（目标浮空）
pub const JUGGLE_MULT: f32 = 1.25;
/// 伤害方差（±10%，PoE 非闪电档）
pub const VARIANCE: f32 = 0.10;

/// 单次命中结果
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HitResult {
    pub damage: f32,
    pub crit: bool,
}

/// 暴击判定：rng01 < chance
pub fn roll_crit(rng01: f32, chance: f32) -> bool {
    rng01 < chance.clamp(0.0, 1.0)
}

/// 护甲减免（PoE 比率式）：reduction = armor / (armor + premit)。
/// 大击穿透（减免趋近 0），小击减免（趋近 armor 主导）。
pub fn armor_reduce(premit: f32, armor: f32) -> f32 {
    if premit <= 0.0 || armor <= 0.0 {
        return premit.max(0.0);
    }
    premit * (1.0 - armor / (armor + premit))
}

/// 护甲穿透：先生效减甲（下限 0），再走比率式
pub fn armor_pierce(premit: f32, armor: f32, pen: f32) -> f32 {
    armor_reduce(premit, (armor - pen).max(0.0))
}

/// 方差：rng01 ∈ [0,1) 映射 ±VARIANCE
pub fn apply_variance(dmg: f32, rng01: f32) -> f32 {
    dmg * (1.0 + (rng01 * 2.0 - 1.0) * VARIANCE)
}

/// 完整管线（调用方已算好攻击方乘区与情境，传入 premit）：
/// 暴击 → 护甲（含穿透）→ 方差 → 保底 1
pub fn resolve(
    premit: f32,
    crit: bool,
    armor: f32,
    pen: f32,
    rng01: f32,
) -> HitResult {
    let mut d = premit.max(0.0);
    if crit {
        d *= CRIT_MULT;
    }
    d = armor_pierce(d, armor, pen);
    d = apply_variance(d, rng01);
    HitResult { damage: d.max(1.0), crit }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn order_crit_then_armor() {
        // 20 基础暴击（×1.5=30）再过 10 甲：30×(1-10/40)=22.5，无方差点 rng=0.5
        let r = resolve(20.0, true, 10.0, 0.0, 0.5);
        assert!((r.damage - 22.5).abs() < 1e-4);
        assert!(r.crit);
    }

    #[test]
    fn armor_scales_against_hit_size() {
        // 小击被减免得多，大击穿透（PoE 本质）
        let small = armor_reduce(10.0, 10.0); // 10×0.5=5
        let big = armor_reduce(100.0, 10.0); // 100×(1-10/110)≈90.9
        assert!((small - 5.0).abs() < 1e-4);
        assert!(big > 90.0);
    }

    #[test]
    fn pen_negates_armor() {
        assert!((armor_pierce(20.0, 6.0, 6.0) - 20.0).abs() < 1e-4);
        assert!(armor_pierce(20.0, 10.0, 0.0) < 20.0);
    }

    #[test]
    fn variance_bounds_and_floor() {
        assert!((apply_variance(100.0, 0.5) - 100.0).abs() < 1e-4);
        assert!((apply_variance(100.0, 0.0) - 90.0).abs() < 1e-4);
        assert!((apply_variance(100.0, 1.0 - 1e-6) - 110.0).abs() < 1.0);
        let r = resolve(0.0, false, 0.0, 0.0, 0.0);
        assert_eq!(r.damage, 1.0); // 保底
    }

    #[test]
    fn zone_multipliers_documented() {
        // 分区乘区为调用方约定值（CS：头4/胸1/腹1.25/腿0.75；我方：背刺/挑空）
        assert_eq!((BACKSTAB_MULT, JUGGLE_MULT, CRIT_MULT), (1.5, 1.25, 1.5));
    }
}
