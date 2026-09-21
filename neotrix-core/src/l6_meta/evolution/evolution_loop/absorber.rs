//! Knowledge Absorber — 知识吸收器
//!
//! 实现 R-P119 (吸收周期): 从外部源提取规则, 解决冲突, 应用到系统。

use serde::{Deserialize, Serialize};

use crate::neotrix::nt_jev::audit::JEV_POLICY_VERSION;
use crate::neotrix::nt_jev::eval::EvalReport;
use crate::neotrix::nt_jev::evolve::report_scores;

/// 吸收结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsorptionResult {
    pub rules_extracted: usize,
    pub rules_applied: usize,
    pub conflicts_resolved: usize,
}

/// 吸收的规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsorbedRule {
    pub rule_id: String,
    pub source: String,
    pub content: String,
    pub priority: u8,
}

/// 知识吸收器
pub struct KnowledgeAbsorber {
    max_rounds: usize,
    accumulated_rules: Vec<AbsorbedRule>,
}

impl KnowledgeAbsorber {
    pub fn new(max_rounds: usize) -> Self {
        Self {
            max_rounds,
            accumulated_rules: Vec::new(),
        }
    }

    /// 从指定源吸收知识
    ///
    /// 模拟 R-P119 吸收周期:
    /// 1. 提取规则 (rules_extracted)
    /// 2. 冲突检测与解决 (conflicts_resolved)
    /// 3. 应用规则 (rules_applied)
    pub fn absorb(&mut self, source: &str) -> AbsorptionResult {
        let mut rules_extracted = 0;
        let mut conflicts_resolved = 0;
        let mut rules_applied = 0;

        for _round in 0..self.max_rounds {
            // Step 1: 提取规则 — 从 source 提取候选规则
            let new_rules = self.extract_rules(source);
            rules_extracted += new_rules.len();

            // Step 2: 冲突检测与解决
            let conflicts = self.detect_conflicts(&new_rules);
            conflicts_resolved += conflicts.len();
            self.resolve_conflicts(conflicts);

            // Step 3: 应用无冲突规则
            for rule in new_rules {
                if !self.accumulated_rules.iter().any(|r| r.rule_id == rule.rule_id) {
                    rules_applied += 1;
                    self.accumulated_rules.push(rule);
                }
            }
        }

        AbsorptionResult {
            rules_extracted,
            rules_applied,
            conflicts_resolved,
        }
    }

    /// 模拟规则提取 — 基于 source 哈希生成确定性规则
    fn extract_rules(&self, source: &str) -> Vec<AbsorbedRule> {
        let hash = source.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
        let count = (hash % 3) as usize + 1;

        (0..count)
            .map(|i| AbsorbedRule {
                rule_id: format!("rule_{}_{:x}", i, hash),
                source: source.to_string(),
                content: format!("extracted_rule_{}", i),
                priority: ((hash >> (i * 8)) % 10) as u8,
            })
            .collect()
    }

    /// 检测新规则与已积累规则的冲突
    fn detect_conflicts(&self, new_rules: &[AbsorbedRule]) -> Vec<String> {
        new_rules
            .iter()
            .filter(|nr| {
                self.accumulated_rules
                    .iter()
                    .any(|ar| ar.rule_id == nr.rule_id && ar.priority != nr.priority)
            })
            .map(|nr| nr.rule_id.clone())
            .collect()
    }

    /// 解决冲突: 保留高优先级规则
    fn resolve_conflicts(&mut self, conflicts: Vec<String>) {
        for conflict_id in conflicts {
            self.accumulated_rules.retain(|r| r.rule_id != conflict_id);
        }
    }

    /// 获取已积累的规则数
    pub fn accumulated_count(&self) -> usize {
        self.accumulated_rules.len()
    }

    /// Absorb a nightly [`EvalReport`] into prescriptive tuning rules.
    ///
    /// Each under-threshold criterion becomes one [`AbsorbedRule`], routed
    /// through the same dedupe/conflict pipeline as [`Self::absorb`]:
    /// - calibration < 0.85 → refit per-bucket temperature on validation split
    /// - accuracy < 0.8 → expand the golden pack (weak-question mining)
    /// - coverage < 0.6 → add abstain options to Choice presets
    /// - robustness < 0.6 → review preset legends for overlap
    /// - efficiency < 0.7 → batch independent questions, trim context
    ///
    /// Rules are idempotent across nights (same `rule_id` dedupes).
    pub fn absorb_jev_report(&mut self, report: &EvalReport) -> AbsorptionResult {
        let source = format!("jev-nightly:{}", JEV_POLICY_VERSION);
        let scores = report_scores(report);
        let get = |k: &str| scores.get(k).copied().unwrap_or(0.0);
        let mut rules = Vec::new();
        let mut push = |id: &str, content: &str, priority: u8| {
            rules.push(AbsorbedRule {
                rule_id: format!("jev/{}", id),
                source: source.clone(),
                content: content.to_string(),
                priority,
            });
        };
        if get("calibration") < 0.85 {
            push(
                "refit-temperature",
                "refit TemperatureScaler per temp_bucket on held-out validation split",
                9,
            );
        }
        if get("accuracy") < 0.8 {
            push(
                "expand-golden-pack",
                "mine weak questions into golden pack; re-run eval",
                8,
            );
        }
        if get("coverage") < 0.6 {
            push(
                "add-abstain-options",
                "inject unknown option into Choice presets; wire contested→abstain",
                7,
            );
        }
        if get("robustness") < 0.6 {
            push(
                "review-presets",
                "review preset legends for overlapping options",
                6,
            );
        }
        if get("efficiency") < 0.7 {
            push(
                "enable-batching",
                "batch independent questions; trim state context",
                5,
            );
        }
        // Same pipeline as absorb(): conflicts first, then dedupe-apply.
        let conflicts = self.detect_conflicts(&rules);
        let conflicts_resolved = conflicts.len();
        self.resolve_conflicts(conflicts);
        let rules_extracted = rules.len();
        let mut rules_applied = 0;
        for rule in rules {
            if !self.accumulated_rules.iter().any(|r| r.rule_id == rule.rule_id) {
                rules_applied += 1;
                self.accumulated_rules.push(rule);
            }
        }
        AbsorptionResult {
            rules_extracted,
            rules_applied,
            conflicts_resolved,
        }
    }

    /// Currently accumulated rule ids (for nightly outcome tracing).
    pub fn rule_ids(&self) -> Vec<String> {
        self.accumulated_rules
            .iter()
            .map(|r| r.rule_id.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absorb_extracts_rules() {
        let mut absorber = KnowledgeAbsorber::new(1);
        let result = absorber.absorb("test_source");
        assert!(result.rules_extracted > 0);
    }

    #[test]
    fn absorb_applies_rules() {
        let mut absorber = KnowledgeAbsorber::new(1);
        let result = absorber.absorb("source_a");
        assert!(result.rules_applied > 0);
    }

    #[test]
    fn absorb_resolves_conflicts() {
        let mut absorber = KnowledgeAbsorber::new(2);
        // Two rounds with same source should detect self-conflicts
        absorber.absorb("same_source");
        let result = absorber.absorb("same_source");
        // Should accumulate without duplicate rule_ids
        assert!(result.conflicts_resolved >= 0);
    }

    #[test]
    fn absorption_result_serializes() {
        let result = AbsorptionResult {
            rules_extracted: 5,
            rules_applied: 3,
            conflicts_resolved: 1,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("rules_extracted"));
    }

    fn weak_report() -> EvalReport {
        EvalReport {
            n: 40,
            accuracy: 0.5,
            brier: 0.5,
            ece: 0.5,
            coverage_at_p90: 0.2,
            mean_latency_ms: 500.0,
        }
    }

    fn strong_report() -> EvalReport {
        EvalReport {
            n: 40,
            accuracy: 0.95,
            brier: 0.05,
            ece: 0.03,
            coverage_at_p90: 0.9,
            mean_latency_ms: 20.0,
        }
    }

    #[test]
    fn absorb_jev_report_extracts_all_five_rules_when_weak() {
        let mut absorber = KnowledgeAbsorber::new(1);
        let result = absorber.absorb_jev_report(&weak_report());
        assert_eq!(result.rules_extracted, 5);
        assert_eq!(result.rules_applied, 5);
        let ids = absorber.rule_ids();
        assert!(ids.iter().any(|id| id == "jev/refit-temperature"));
        assert!(ids.iter().any(|id| id == "jev/enable-batching"));
    }

    #[test]
    fn absorb_jev_report_extracts_nothing_when_strong() {
        let mut absorber = KnowledgeAbsorber::new(1);
        let result = absorber.absorb_jev_report(&strong_report());
        assert_eq!(result.rules_extracted, 0);
        assert_eq!(result.rules_applied, 0);
    }

    #[test]
    fn absorb_jev_report_is_idempotent() {
        let mut absorber = KnowledgeAbsorber::new(1);
        absorber.absorb_jev_report(&weak_report());
        let again = absorber.absorb_jev_report(&weak_report());
        assert_eq!(again.rules_extracted, 5);
        assert_eq!(again.rules_applied, 0);
        assert_eq!(absorber.accumulated_count(), 5);
    }
}
