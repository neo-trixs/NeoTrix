//! 归因推理 — 为 observation 找出最佳解释 (因果规则的 condition).

use super::chain_executor::{ChainExecutor, CausalRule, ReasoningResult, ReasoningStepNode};

/// 归因推理: observation -> 匹配 outcome 的因果规则 -> 最佳解释 = condition.
pub fn abductive(ce: &ChainExecutor, observation: &str) -> ReasoningResult {
    let lower = observation.to_lowercase();

    // 1. 在所有因果规则中找出 outcome 命中 observation 的, 取置信度最高者
    let rules = ce.query_causal_rules();
    let mut best: Option<&CausalRule> = None;
    let mut best_score = 0.0f64;
    for r in &rules {
        if r.outcome.to_lowercase().contains(&lower) && r.confidence > best_score {
            best_score = r.confidence;
            best = Some(r);
        }
    }

    // 2. 组装解释步骤
    let mut steps: Vec<ReasoningStepNode> = Vec::new();
    let mut rules_used: Vec<CausalRule> = Vec::new();
    if let Some(r) = best {
        rules_used.push(r.clone());
        steps.push(ReasoningStepNode {
            node_id: format!("expl:{}", r.id),
            label: r.condition.clone(),
            relation: Some("explains".into()),
            confidence: r.confidence,
        });
    }

    let conclusion = best.map(|r| r.condition.clone());

    ReasoningResult {
        mode: "abductive".into(),
        query: observation.into(),
        steps,
        conclusion,
        confidence: best_score.clamp(0.0, 1.0),
        rules_used,
    }
}
