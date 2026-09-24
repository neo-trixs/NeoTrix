//! 能力层数回合结算纯函数.
//!
//! burn: 伤=层数 → 层数减半；poison: 伤=层数 → -1；regen: 疗=层数 → -1；
//! vulnerable/weak: -1/轮；strength/thorns/fortify：战斗内不衰减。

/// 一轮时长（秒），运行时计时用。
pub const ROUND_SECS: f32 = 4.0;

/// 某 Counter 能力一轮结算 → (hp_delta, new_stacks)。
/// hp_delta 负=掉血，正=回血；new_stacks <= 0 表示移除。
pub fn tick_counter(power_id: &str, stacks: i32) -> (f32, i32) {
    match power_id {
        "burn" => (-(stacks as f32), stacks / 2),
        "poison" => (-(stacks as f32), stacks - 1),
        "regen" => (stacks as f32, stacks - 1),
        "vulnerable" | "weak" => (0.0, stacks - 1),
        _ => (0.0, stacks),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counter_ticks() {
        // burn：伤=层，减半
        assert_eq!(tick_counter("burn", 6), (-6.0, 3));
        assert_eq!(tick_counter("burn", 1), (-1.0, 0));
        // poison：伤=层，-1
        assert_eq!(tick_counter("poison", 4), (-4.0, 3));
        // regen：疗=层，-1
        assert_eq!(tick_counter("regen", 3), (3.0, 2));
        // vulnerable/weak：无伤，-1
        assert_eq!(tick_counter("vulnerable", 2), (0.0, 1));
        assert_eq!(tick_counter("weak", 1), (0.0, 0));
        // strength/thorns：不结算不衰减
        assert_eq!(tick_counter("strength", 5), (0.0, 5));
        assert_eq!(tick_counter("thorns", 3), (0.0, 3));
        // 未知：透传
        assert_eq!(tick_counter("vigor", 2), (0.0, 2));
    }
}
