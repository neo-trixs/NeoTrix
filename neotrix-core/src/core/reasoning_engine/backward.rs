//! 反向推理 — 从 goal 反推支撑 premise.

use super::chain_executor::{ChainExecutor, CausalRule, ReasoningResult, ReasoningStepNode};
use std::collections::HashSet;

/// 反向推理: goal -> (匹配 outcome 因果规则) + (沿推理边反向遍历命中 source) -> premise 链.
pub fn backward(ce: &ChainExecutor, goal: &str) -> ReasoningResult {
    let lower = goal.to_lowercase();
    let mut steps: Vec<ReasoningStepNode> = Vec::new();
    let mut rules_used: Vec<CausalRule> = Vec::new();
    let mut confidence = 1.0f64;

    // 1. 定位 goal 节点
    let goals = ce.find_nodes(goal, 4);

    // 2. 匹配 outcome 命中 goal 的因果规则
    let rules = ce.query_causal_rules();
    for r in &rules {
        let ro = r.outcome.to_lowercase();
        if ro.contains(&lower)
            || goals.iter().any(|g| ro.contains(&g.label.to_lowercase()))
        {
            confidence *= r.confidence.max(0.01);
            rules_used.push(r.clone());
        }
    }

    // 3. 反向遍历: 以 goal 节点为目标, 取 source 作为支撑 premise
    let goal_ids: Vec<String> = goals.iter().map(|g| g.node_id.clone()).collect();
    let mut visited = HashSet::new();
    for g in &goal_ids {
        visited.insert(g.clone());
    }
    let edges = ce.traverse_edges_rev(&goal_ids, 8);
    for e in &edges {
        if e.from == e.to || !visited.insert(e.from.clone()) {
            continue;
        }
        let label = ce.node_label(&e.from);
        confidence *= e.weight.max(0.01);
        steps.push(ReasoningStepNode {
            node_id: e.from.clone(),
            label,
            relation: Some(e.relation.clone()),
            confidence: e.weight,
        });
    }

    // 4. 结论 = 原 goal
    let conclusion = if !goals.is_empty() {
        Some(goals[0].label.clone())
    } else {
        Some(goal.to_string())
    };

    if !steps.is_empty() {
        confidence /= steps.len() as f64 + 1.0;
    }

    ReasoningResult {
        mode: "backward".into(),
        query: goal.into(),
        steps,
        conclusion,
        confidence: confidence.clamp(0.0, 1.0),
        rules_used,
    }
}
