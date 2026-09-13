//! NT-MIND — BPCO 吸收 (arXiv:2608.23566v2).
//!
//! BPCO: 最佳实践评论优化 (Best Practice Critic Optimization) — 用 critic 反馈
//! 优化 LLM 训练管线。本模块为 C0 级结构吸收: 仅定义 trait 与 stub 方法骨架,
//! 编译通过即可, 后续 C1-C4 迭代填充训练反馈逻辑。

use crate::core::nt_core_self_test::SelfTest;

/// 最佳实践评论反馈: critic 对某次生成的 critiques 与评分。
#[derive(Debug, Clone, Default)]
pub struct _CriticFeedback {
    pub critiques: Vec<String>,
    pub score: f64,
}

/// 最佳实践评论优化 trait (BPCO).
pub trait _BestPracticeCritic {
    /// 对生成文本给出评论反馈 (stub: 当前返回中性占位反馈)。
    fn critique(&self, _generated: &str) -> _CriticFeedback;
    /// 反馈是否通过质量标准 (stub: 默认通过)。
    fn passes(&self, fb: &_CriticFeedback) -> bool;
}

/// BPCO critic 实现 (C0 结构 stub).
pub struct _BpcoCritic {
    min_score: f64,
}

impl _BpcoCritic {
    pub fn new() -> Self {
        Self { min_score: 0.5 }
    }
}

impl Default for _BpcoCritic {
    fn default() -> Self {
        Self::new()
    }
}

impl _BestPracticeCritic for _BpcoCritic {
    fn critique(&self, _generated: &str) -> _CriticFeedback {
        // not wired: C0 structural stub — no real critic model connected.
        // Returns explicit rejection signal instead of silent neutral placeholder.
        _CriticFeedback {
            critiques: vec!["not wired: BPCO critic has no real model behind it — \
             cannot evaluate best-practice compliance"
                .into()],
            score: 0.0,
        }
    }

    fn passes(&self, fb: &_CriticFeedback) -> bool {
        fb.score >= self.min_score
    }
}

impl SelfTest for _BpcoCritic {
    fn name(&self) -> &'static str {
        "_BpcoCritic"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        if self.min_score < 0.0 || self.min_score > 1.0 {
            return Err(vec![format!("min_score out of range: {}", self.min_score)]);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_critique_returns_rejection_signal() {
        let c = _BpcoCritic::new();
        let fb = c.critique("hello world");
        // C0 stub returns explicit rejection: score=0.0, non-empty critique explaining why.
        assert_eq!(fb.score, 0.0, "C0 stub must return 0.0 (not wired)");
        assert!(!fb.critiques.is_empty(), "must include rejection explanation");
        assert!(
            fb.critiques[0].contains("not wired"),
            "critique should explain the stub is unwired"
        );
    }

    #[test]
    fn test_passes_threshold() {
        let c = _BpcoCritic::new();
        // Verify threshold logic against the critic's own min_score (0.5).
        // Use the actual output from critique() to confirm it fails the threshold.
        let fb = c.critique("any input");
        assert!(!c.passes(&fb), "C0 stub output (score=0.0) must NOT pass threshold=0.5");
        // Manually构造 a passing feedback to verify the threshold check itself works.
        let passing = _CriticFeedback {
            critiques: vec!["looks good".into()],
            score: 0.8,
        };
        assert!(c.passes(&passing), "score 0.8 >= 0.5 should pass");
    }

    #[test]
    fn test_selftest_pass() {
        // TODO(R-P79): self_test() is a C0 stub that always returns Ok. Replace with
        // integration test that validates real critique behavior on actual skill content.
        let c = _BpcoCritic::new();
        assert!(c.self_test().is_ok());
    }
}
