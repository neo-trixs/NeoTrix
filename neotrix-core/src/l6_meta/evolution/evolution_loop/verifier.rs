//! Evolution Verifier — 进化验证器
//!
//! 对比进化前后的系统状态, 计算净改进分数,
//! 检测回归 (regressions) 并确认改进 (improvements)。

use serde::{Deserialize, Serialize};

use super::loop_runner::SystemState;

/// 验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub improvements: Vec<String>,
    pub regressions: Vec<String>,
    pub net_score: f64,
}

/// 进化验证器
pub struct EvolutionVerifier {
    /// 最小改进阈值 — net_score 低于此值视为无改进
    min_improvement_threshold: f64,
}

impl EvolutionVerifier {
    pub fn new(min_improvement_threshold: f64) -> Self {
        Self {
            min_improvement_threshold,
        }
    }

    /// 验证进化结果
    pub fn verify(&self, before: &SystemState, after: &SystemState) -> VerificationResult {
        let mut improvements = Vec::new();
        let mut regressions = Vec::new();

        // 检查 capability_count 变化
        match after.capability_count.cmp(&before.capability_count) {
            std::cmp::Ordering::Greater => {
                improvements.push(format!(
                    "Capability count: {} → {} (+{})",
                    before.capability_count,
                    after.capability_count,
                    after.capability_count - before.capability_count
                ));
            }
            std::cmp::Ordering::Less => {
                regressions.push(format!(
                    "Capability count: {} → {} ({})",
                    before.capability_count,
                    after.capability_count,
                    after.capability_count as isize - before.capability_count as isize
                ));
            }
            std::cmp::Ordering::Equal => {}
        }

        // 检查 overall_score 变化
        let score_delta = after.overall_score - before.overall_score;
        if score_delta > self.min_improvement_threshold {
            improvements.push(format!(
                "Overall score: {:.4} → {:.4} (+{:.4})",
                before.overall_score, after.overall_score, score_delta
            ));
        } else if score_delta < -self.min_improvement_threshold {
            regressions.push(format!(
                "Overall score: {:.4} → {:.4} ({:.4})",
                before.overall_score, after.overall_score, score_delta
            ));
        }

        // 检查活跃模块变化
        let before_set: std::collections::HashSet<&str> =
            before.active_modules.iter().map(|s| s.as_str()).collect();
        let after_set: std::collections::HashSet<&str> =
            after.active_modules.iter().map(|s| s.as_str()).collect();

        let added: Vec<&str> = after_set.difference(&before_set).copied().collect();
        let removed: Vec<&str> = before_set.difference(&after_set).copied().collect();

        for module in &added {
            improvements.push(format!("Added module: {}", module));
        }
        for module in &removed {
            regressions.push(format!("Removed module: {}", module));
        }

        // 计算净得分: improvements 贡献正分, regressions 贡献负分
        let improvement_score = improvements.len() as f64 * 0.05;
        let regression_penalty = regressions.len() as f64 * 0.1;
        let net_score = score_delta + improvement_score - regression_penalty;

        VerificationResult {
            improvements,
            regressions,
            net_score,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_state(score: f64, cap_count: usize, modules: Vec<&str>) -> SystemState {
        SystemState {
            capability_count: cap_count,
            overall_score: score,
            growth_phase: "C1-UnitTest".into(),
            active_modules: modules.into_iter().map(String::from).collect(),
            metadata: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn verify_detects_improvements() {
        let before = mock_state(0.5, 5, vec!["a", "b"]);
        let after = mock_state(0.6, 7, vec!["a", "b", "c"]);
        let verifier = EvolutionVerifier::new(0.01);
        let result = verifier.verify(&before, &after);
        assert!(!result.improvements.is_empty());
        assert!(result.regressions.is_empty());
        assert!(result.net_score > 0.0);
    }

    #[test]
    fn verify_detects_regressions() {
        let before = mock_state(0.8, 10, vec!["a", "b", "c"]);
        let after = mock_state(0.6, 5, vec!["a"]);
        let verifier = EvolutionVerifier::new(0.01);
        let result = verifier.verify(&before, &after);
        assert!(!result.regressions.is_empty());
        assert!(result.net_score < 0.0);
    }

    #[test]
    fn verify_no_change() {
        let state = mock_state(0.5, 5, vec!["a"]);
        let verifier = EvolutionVerifier::new(0.01);
        let result = verifier.verify(&state, &state);
        assert!(result.improvements.is_empty());
        assert!(result.regressions.is_empty());
        assert!(result.net_score == 0.0);
    }

    #[test]
    fn verification_result_serializes() {
        let result = VerificationResult {
            improvements: vec!["test".into()],
            regressions: vec![],
            net_score: 0.05,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("net_score"));
    }
}
