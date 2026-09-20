//! ParallelDecisionEngine — 并行决策引擎
//!
//! 基于 TypeSafe/SystemOne + NanoJev + fast-jev-compaction。
//! - Choice/Score/Noul/Boolean 四原语
//! - 并行决策执行
//! - ToolCall 评分 (compaction)
//! - 成本感知路由 (Axiom A1)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionPrimitive {
    Choice { options: Vec<String> },
    Score { value: f64 },
    Noul { exists: bool },
    Boolean { value: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRequest {
    pub id: String,
    pub primitive: DecisionPrimitive,
    pub context: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionResult {
    pub request_id: String,
    pub primitive: DecisionPrimitive,
    pub confidence: f64,
    pub model_used: String,
    pub cost: f64,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: String,
    pub keep_call: f64,
    pub keep_result: f64,
}

impl ToolCall {
    pub fn should_keep(&self) -> bool {
        self.keep_call + self.keep_result >= 0.5
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRoute {
    pub name: String,
    pub cost_per_1k: f64,
    pub latency_ms: f64,
    pub capability_score: f64,
}

pub fn cost_aware_route(task_complexity: f64, models: &[ModelRoute]) -> Option<&ModelRoute> {
    models.iter()
        .filter(|m| m.capability_score >= task_complexity)
        .min_by(|a, b| a.cost_per_1k.partial_cmp(&b.cost_per_1k).unwrap_or(std::cmp::Ordering::Equal))
}

pub struct ParallelDecisionEngine {
    pub models: Vec<ModelRoute>,
    pub decision_history: Vec<DecisionResult>,
}

impl ParallelDecisionEngine {
    pub fn new() -> Self {
        Self {
            models: vec![
                ModelRoute { name: "cheap".into(), cost_per_1k: 0.001, latency_ms: 50.0, capability_score: 0.3 },
                ModelRoute { name: "balanced".into(), cost_per_1k: 0.01, latency_ms: 200.0, capability_score: 0.7 },
                ModelRoute { name: "strong".into(), cost_per_1k: 0.06, latency_ms: 500.0, capability_score: 0.95 },
            ],
            decision_history: Vec::new(),
        }
    }

    pub fn decide(&self, request: &DecisionRequest) -> DecisionResult {
        let complexity = match &request.primitive {
            DecisionPrimitive::Boolean { .. } => 0.2,
            DecisionPrimitive::Noul { .. } => 0.3,
            DecisionPrimitive::Score { .. } => 0.5,
            DecisionPrimitive::Choice { options } => 0.3 + options.len() as f64 * 0.1,
        };
        let route = cost_aware_route(complexity, &self.models).unwrap_or(&self.models[0]);
        match &request.primitive {
            DecisionPrimitive::Choice { options } => DecisionResult {
                request_id: request.id.clone(),
                primitive: DecisionPrimitive::Choice { options: options.first().cloned().map(|s| vec![s]).unwrap_or_default() },
                confidence: 0.8, model_used: route.name.clone(), cost: route.cost_per_1k, latency_ms: route.latency_ms as u64,
            },
            other => DecisionResult {
                request_id: request.id.clone(), primitive: other.clone(), confidence: 0.9,
                model_used: route.name.clone(), cost: route.cost_per_1k, latency_ms: route.latency_ms as u64,
            },
        }
    }

    pub fn parallel_choices(&self, requests: &[DecisionRequest]) -> Vec<DecisionResult> {
        requests.iter().map(|r| self.decide(r)).collect()
    }

    pub fn compact_tool_calls(&self, calls: &[ToolCall]) -> Vec<&ToolCall> {
        calls.iter().filter(|c| c.should_keep()).collect()
    }

    pub fn stats(&self) -> (usize, f64) {
        let cost: f64 = self.decision_history.iter().map(|r| r.cost).sum();
        (self.decision_history.len(), cost)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_call_compaction() {
        let calls = vec![
            ToolCall { name: "keep".into(), arguments: "{}".into(), keep_call: 0.8, keep_result: 0.8 },
            ToolCall { name: "drop".into(), arguments: "{}".into(), keep_call: 0.1, keep_result: 0.1 },
        ];
        let engine = ParallelDecisionEngine::new();
        let kept = engine.compact_tool_calls(&calls);
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].name, "keep");
    }

    #[test]
    fn test_cost_aware_route() {
        let models = vec![
            ModelRoute { name: "cheap".into(), cost_per_1k: 0.001, latency_ms: 50.0, capability_score: 0.3 },
            ModelRoute { name: "strong".into(), cost_per_1k: 0.06, latency_ms: 500.0, capability_score: 0.95 },
        ];
        let route = cost_aware_route(0.2, &models).unwrap();
        assert_eq!(route.name, "cheap");
        let route = cost_aware_route(0.8, &models).unwrap();
        assert_eq!(route.name, "strong");
    }

    #[test]
    fn test_parallel_decisions() {
        let engine = ParallelDecisionEngine::new();
        let requests = vec![
            DecisionRequest { id: "1".into(), primitive: DecisionPrimitive::Boolean { value: true }, context: HashMap::new() },
            DecisionRequest { id: "2".into(), primitive: DecisionPrimitive::Score { value: 0.8 }, context: HashMap::new() },
        ];
        let results = engine.parallel_choices(&requests);
        assert_eq!(results.len(), 2);
    }
}
