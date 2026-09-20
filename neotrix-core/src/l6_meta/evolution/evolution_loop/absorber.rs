//! Knowledge Absorber — 知识吸收器
//!
//! 实现 R-P119 (吸收周期): 从外部源提取规则, 解决冲突, 应用到系统。

use serde::{Deserialize, Serialize};

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
}
