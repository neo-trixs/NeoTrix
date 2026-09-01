//! YagniLadder — 7 级 YAGNI 阶梯 (ponytail 纪律, R-P81)
//!
//! 七级逐步升级的 "你真的需要这个功能吗?" 拷问, 全部为确定性阈值函数:
//!   usage 低 + consumers 低 → Cut (建议删除)
//!   medium                → Reconsider
//!   usage 高 + consumers 高 → Keep
//!
//! 纯确定性代码: 无网络 / 无 tokio / 无文件 IO。

/// 单级阶梯
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YagniLevel {
    pub level: u8,
    pub name: &'static str,
    pub question: &'static str,
}

/// 判定结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YagniVerdict {
    /// 建议删除 (低 usage + 低 consumers)
    Cut,
    /// 建议重新考虑 (中等证据)
    Reconsider,
    /// 保留 (高 usage / 高 consumers)
    Keep,
}

/// 7 级 YAGNI 阶梯
#[derive(Debug, Clone, Default)]
pub struct YagniLadder;

impl YagniLadder {
    pub fn new() -> Self {
        Self
    }

    /// 返回全部 7 级阶梯 (供内省/展示)
    pub fn ladder(&self) -> [YagniLevel; 7] {
        [
            YagniLevel {
                level: 1,
                name: "Minimal Viable",
                question: "Does the system fail without it? Is it required by an existing contract?",
            },
            YagniLevel {
                level: 2,
                name: "Single Consumer",
                question: "Is there exactly one consumer today, or is it speculative demand?",
            },
            YagniLevel {
                level: 3,
                name: "Call Site",
                question: "Can you name the exact call site that needs it right now?",
            },
            YagniLevel {
                level: 4,
                name: "Testing",
                question: "Will it ship and be testable before the guess about it changes?",
            },
            YagniLevel {
                level: 5,
                name: "Extrapolation",
                question: "Are you building it for a user that does not exist yet?",
            },
            YagniLevel {
                level: 6,
                name: "Reward",
                question: "Does the payoff justify the complexity and maintenance it adds today?",
            },
            YagniLevel {
                level: 7,
                name: "Rewrite",
                question: "Would you be better off deleting and rewriting it than adding this feature?",
            },
        ]
    }

    /// 打分候选功能, 返回确定性判定。
    ///
    /// score = usage*2 + consumers*3, 若 candidate 含投机性措辞 (eventually/might/
    /// someday/future/probably/potentially) 则 -1。
    ///   score < 6  → Cut
    ///   6..12      → Reconsider
    ///   >= 12      → Keep
    pub fn evaluate(&self, candidate: &str, usage: u32, consumers: u32) -> YagniVerdict {
        let mut score = usage as u64 * 2 + consumers as u64 * 3;
        if is_speculative(candidate) {
            score = score.saturating_sub(1);
        }
        if score < 6 {
            YagniVerdict::Cut
        } else if score < 12 {
            YagniVerdict::Reconsider
        } else {
            YagniVerdict::Keep
        }
    }

    /// 检查候选是否通过某级 (level 1-7) 的拷问 — 用于阶梯内省。
    /// 简化模型: 达到 Keep 视为通过全部 7 级, Reconsider 视为通过前 4 级。
    pub fn reached_level(&self, candidate: &str, usage: u32, consumers: u32, level: u8) -> bool {
        match self.evaluate(candidate, usage, consumers) {
            YagniVerdict::Cut => false,
            YagniVerdict::Reconsider => level <= 4,
            YagniVerdict::Keep => level <= 7,
        }
    }
}

impl crate::core::nt_core_self_test::SelfTest for YagniLadder {
    fn name(&self) -> &str {
        "nt_act_code_yagni_ladder"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        let ladder = YagniLadder::new();
        let levels = ladder.ladder();
        if levels.len() != 7 {
            failures.push("ladder must have 7 levels".into());
        }
        if levels.iter().any(|l| l.level == 0) || levels[0].level != 1 || levels[6].level != 7 {
            failures.push("levels must be 1..=7".into());
        }
        if ladder.evaluate("quick hack", 0, 0) != YagniVerdict::Cut {
            failures.push("zero usage should Cut".into());
        }
        if ladder.evaluate("core endpoint", 10, 5) != YagniVerdict::Keep {
            failures.push("high usage should Keep".into());
        }
        let mid = ladder.evaluate("cache helper", 2, 2);
        if mid != YagniVerdict::Reconsider {
            failures.push("medium usage should Reconsider".into());
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

/// 投机性措辞检测 — 确定性小写子串扫描
fn is_speculative(candidate: &str) -> bool {
    let lower = candidate.to_lowercase();
    ["eventually", "might", "someday", "future", "probably", "potentially"]
        .iter()
        .any(|w| lower.contains(w))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ladder() -> YagniLadder {
        YagniLadder::new()
    }

    #[test]
    fn ladder_has_seven_levels() {
        let levels = ladder().ladder();
        assert_eq!(levels.len(), 7);
        let expected = [
            "Minimal Viable",
            "Single Consumer",
            "Call Site",
            "Testing",
            "Extrapolation",
            "Reward",
            "Rewrite",
        ];
        for (i, l) in levels.iter().enumerate() {
            assert_eq!(l.level, (i + 1) as u8);
            assert_eq!(l.name, expected[i]);
            assert!(!l.question.is_empty(), "level {} must have a question", l.level);
        }
    }

    #[test]
    fn low_usage_low_consumers_cut() {
        assert_eq!(ladder().evaluate("toy feature", 0, 0), YagniVerdict::Cut);
        assert_eq!(ladder().evaluate("toy feature", 1, 1), YagniVerdict::Cut);
        assert_eq!(ladder().evaluate("toy feature", 2, 0), YagniVerdict::Cut);
    }

    #[test]
    fn medium_usage_reconsider() {
        assert_eq!(ladder().evaluate("mid feature", 2, 1), YagniVerdict::Reconsider);
        assert_eq!(ladder().evaluate("mid feature", 1, 2), YagniVerdict::Reconsider);
        assert_eq!(ladder().evaluate("mid feature", 2, 2), YagniVerdict::Reconsider);
        assert_eq!(ladder().evaluate("mid feature", 4, 1), YagniVerdict::Reconsider);
        assert_eq!(ladder().evaluate("mid feature", 5, 0), YagniVerdict::Reconsider);
    }

    #[test]
    fn high_usage_keep() {
        assert_eq!(ladder().evaluate("core endpoint", 10, 5), YagniVerdict::Keep);
        assert_eq!(ladder().evaluate("core endpoint", 100, 1), YagniVerdict::Keep);
        assert_eq!(ladder().evaluate("core endpoint", 6, 0), YagniVerdict::Keep);
    }

    #[test]
    fn verdict_boundary_exact_thresholds() {
        // score 5 → Cut, score 6 → Reconsider
        assert_eq!(ladder().evaluate("x", 1, 1), YagniVerdict::Cut); // 2+3=5
        assert_eq!(ladder().evaluate("x", 2, 0), YagniVerdict::Cut); // 4+0=4
        // score 6 → Reconsider (6 is not < 6)
        assert_eq!(ladder().evaluate("x", 3, 0), YagniVerdict::Reconsider); // 6+0=6
        assert_eq!(ladder().evaluate("x", 2, 1), YagniVerdict::Reconsider); // 4+3=7
        // score 11 → Reconsider, score 12 → Keep
        assert_eq!(ladder().evaluate("x", 4, 1), YagniVerdict::Reconsider); // 8+3=11
        assert_eq!(ladder().evaluate("x", 3, 2), YagniVerdict::Keep); // 6+6=12
        assert_eq!(ladder().evaluate("x", 2, 3), YagniVerdict::Keep); // 4+9=13
    }

    #[test]
    fn speculative_language_penalty() {
        // 3*2 + 0*3 = 6 → Reconsider, 但投机措辞 → 5 → Cut
        assert_eq!(ladder().evaluate("we might need it someday", 3, 0), YagniVerdict::Cut);
        // 没有投机措辞 → Reconsider
        assert_eq!(ladder().evaluate("we need it now", 3, 0), YagniVerdict::Reconsider);
        // 高 usage 不受投机措辞影响降级太多: 100*2=200 → Keep
        assert_eq!(ladder().evaluate("future-proofing helper", 100, 0), YagniVerdict::Keep);
    }

    #[test]
    fn reached_level_progression() {
        let l = ladder();
        assert!(!l.reached_level("toy", 0, 0, 1), "Cut feature reaches nothing");
        assert!(l.reached_level("mid", 2, 2, 4), "Reconsider reaches up to level 4");
        assert!(!l.reached_level("mid", 2, 2, 5), "Reconsider does not reach level 5");
        assert!(l.reached_level("keep", 10, 5, 7), "Keep reaches all 7 levels");
    }

    #[test]
    fn self_test_name_is_stable() {
        let l = YagniLadder::new();
        use crate::core::nt_core_self_test::SelfTest;
        assert_eq!(l.name(), "nt_act_code_yagni_ladder");
    }
}
