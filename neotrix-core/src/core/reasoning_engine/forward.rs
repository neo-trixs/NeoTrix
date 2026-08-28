//! 前向推理 — 从 premise 经因果规则与推理边推出结论.

use super::chain_executor::{ChainExecutor, CausalRule, ReasoningResult, ReasoningStepNode};
use std::collections::HashSet;

/// 前向推理: premise -> (匹配因果规则) + (沿 leads_to/synthesizes/analogizes 边遍历) -> 结论.
pub fn forward(ce: &ChainExecutor, premise: &str, max_depth: u8) -> ReasoningResult {
    let lower = premise.to_lowercase();
    let mut steps: Vec<ReasoningStepNode> = Vec::new();
    let mut rules_used: Vec<CausalRule> = Vec::new();
    let mut confidence = 1.0f64;

    // 1. FTS 搜索 premise 相关种子节点
    let seeds = ce.find_nodes(premise, 8);
    if seeds.is_empty() {
        return ReasoningResult {
            mode: "forward".into(),
            query: premise.into(),
            steps,
            conclusion: None,
            confidence: 0.0,
            rules_used,
        };
    }

    // 2. 读取 causal_rules, 匹配 condition 含 premise 或某 seed 标签的规则
    let rules = ce.query_causal_rules();
    for r in &rules {
        let rc = r.condition.to_lowercase();
        if rc.contains(&lower)
            || seeds
                .iter()
                .any(|s| rc.contains(&s.label.to_lowercase()))
        {
            confidence *= r.confidence.max(0.01);
            rules_used.push(r.clone());
        }
    }

    // 3. 沿推理关系正向遍历边
    let mut visited = HashSet::new();
    for s in &seeds {
        visited.insert(s.node_id.clone());
    }
    let edges = ce.traverse_edges(
        &seeds.iter().map(|s| s.node_id.clone()).collect::<Vec<_>>(),
        max_depth,
    );
    for e in &edges {
        if !visited.insert(e.to.clone()) {
            continue;
        }
        let label = ce.node_label(&e.to);
        confidence *= e.weight.max(0.01);
        steps.push(ReasoningStepNode {
            node_id: e.to.clone(),
            label,
            relation: Some(e.relation.clone()),
            confidence: e.weight,
        });
    }

    // 4. 结论
    let conclusion = if !steps.is_empty() {
        Some(steps.last().unwrap().label.clone())
    } else if !rules_used.is_empty() {
        Some(rules_used.last().unwrap().outcome.clone())
    } else {
        Some(seeds[0].label.clone())
    };

    if !steps.is_empty() {
        confidence /= steps.len() as f64 + 1.0;
    }

    ReasoningResult {
        mode: "forward".into(),
        query: premise.into(),
        steps,
        conclusion,
        confidence: confidence.clamp(0.0, 1.0),
        rules_used,
    }
}
