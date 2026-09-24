//! 通用效用决策 — judgment 模式提炼（去游戏类型，M2 敌 AI 直接可用）.
//!
//! 公理：候选独立打分 → 门禁清零 → 迟滞防抖 → 取最高；全负/空/非有限 → fail-open
//! 回退（永不 panic，永有输出）。调用方定义自己的 Mode（`Copy + PartialEq + Debug`），
//! 本模块只做选择数学，不认识任何游戏。

/// 带分候选：模式 + 分数（<0 表示被门禁）+ 调试理由
#[derive(Debug, Clone)]
pub struct Scored<M> {
    pub mode: M,
    pub score: f32,
    pub reason: &'static str,
}

impl<M> Scored<M> {
    pub fn new(mode: M, score: f32, reason: &'static str) -> Self {
        Self { mode, score, reason }
    }

    /// 门禁：分数置负并改理由
    pub fn gate(&mut self, reason: &'static str) {
        self.score = -1.0;
        self.reason = reason;
    }
}

/// 迟滞：当前行为加分，防帧间抖动
pub fn hysteresis<M: PartialEq>(current: M, bonus: f32) -> impl Fn(M) -> f32 {
    move |m: M| if m == current { bonus } else { 0.0 }
}

/// 决策：过滤非有限/负分，取最高；无可选 → 回退（fail-open）
pub fn decide<M: Copy + PartialEq + std::fmt::Debug>(
    scores: &[Scored<M>],
    fallback: M,
) -> (M, f32, &'static str) {
    let best = scores
        .iter()
        .filter(|s| s.score.is_finite() && s.score >= 0.0)
        .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal));
    match best {
        Some(b) => (b.mode, b.score, b.reason),
        None => (fallback, 0.0, "fail-open"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Act {
        Patrol,
        Chase,
        Attack,
    }

    fn scores(chase: f32, attack: f32) -> Vec<Scored<Act>> {
        vec![
            Scored::new(Act::Patrol, 0.3, "默认"),
            Scored::new(Act::Chase, chase, "追击"),
            Scored::new(Act::Attack, attack, "攻击"),
        ]
    }

    #[test]
    fn picks_highest() {
        let (m, s, _) = decide(&scores(0.8, 0.5), Act::Patrol);
        assert_eq!(m, Act::Chase);
        assert!((s - 0.8).abs() < 1e-6);
    }

    #[test]
    fn gated_scores_lose() {
        let mut sc = scores(0.9, 0.5);
        sc[1].gate("墙后");
        let (m, _, r) = decide(&sc, Act::Patrol);
        assert_eq!(m, Act::Attack);
        assert_eq!(r, "攻击");
    }

    #[test]
    fn all_negative_falls_back() {
        let mut sc = scores(0.9, 0.5);
        for s in &mut sc {
            s.gate("门禁");
        }
        let (m, s, r) = decide(&sc, Act::Patrol);
        assert_eq!(m, Act::Patrol);
        assert_eq!(s, 0.0);
        assert_eq!(r, "fail-open");
    }

    #[test]
    fn nan_and_inf_fall_back() {
        let sc = vec![
            Scored::new(Act::Chase, f32::NAN, "?"),
            Scored::new(Act::Attack, f32::INFINITY, "?"),
        ];
        // NaN 被过滤；+Inf 有限性检查：is_finite 为 false → 同样过滤
        let (m, _, _) = decide(&sc, Act::Patrol);
        assert_eq!(m, Act::Patrol);
    }

    #[test]
    fn hysteresis_adds_bonus() {
        let h = hysteresis(Act::Chase, 0.1);
        assert_eq!(h(Act::Chase), 0.1);
        assert_eq!(h(Act::Patrol), 0.0);
    }
}
