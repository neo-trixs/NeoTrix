//! Semantic Router — 语义路由 (confidence-based dispatch)
//!
//! 基于任务分类器的置信度决定路由:
//! - 高置信度: 路由到首选模型
//! - 低置信度: 升级到更强模型 (兜底)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 路由决策
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticRouteDecision {
    pub model: String,
    pub confidence: f64,
    pub reason: String,
    pub task_type: String,
}

/// 语义路由器
pub struct SemanticRouter {
    /// 置信度阈值 (低于此值升级到强模型)
    confidence_threshold: f64,
    /// 路由表: task_type → preferred_model
    route_table: HashMap<String, String>,
    /// 降级模型 (低置信度时使用)
    fallback_model: String,
}

impl SemanticRouter {
    pub fn new() -> Self {
        Self::default()
    }

    /// 路由决策
    pub fn route(&self, _input: &str, task_type: &str, confidence: f64) -> SemanticRouteDecision {
        if confidence >= self.confidence_threshold {
            let model = self
                .route_table
                .get(task_type)
                .cloned()
                .unwrap_or_else(|| self.fallback_model.clone());
            SemanticRouteDecision {
                model,
                confidence,
                reason: format!(
                    "high confidence ({:.2} >= {:.2}), using preferred model",
                    confidence, self.confidence_threshold
                ),
                task_type: task_type.to_string(),
            }
        } else {
            SemanticRouteDecision {
                model: self.fallback_model.clone(),
                confidence,
                reason: format!(
                    "low confidence ({:.2} < {:.2}), upgrading to fallback",
                    confidence, self.confidence_threshold
                ),
                task_type: task_type.to_string(),
            }
        }
    }

    /// 添加路由规则
    pub fn add_route(&mut self, task_type: &str, model: &str) {
        self.route_table
            .insert(task_type.to_string(), model.to_string());
    }

    /// 设置置信度阈值
    pub fn set_threshold(&mut self, threshold: f64) {
        self.confidence_threshold = threshold;
    }
}

impl Default for SemanticRouter {
    fn default() -> Self {
        let mut route_table = HashMap::new();
        route_table.insert("coding".to_string(), "claude-code".to_string());
        route_table.insert("reasoning".to_string(), "gpt-4o".to_string());
        route_table.insert("simple".to_string(), "gemini-flash".to_string());

        Self {
            confidence_threshold: 0.7,
            route_table,
            fallback_model: "gpt-4o".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_high_confidence() {
        let router = SemanticRouter::default();
        let decision = router.route("fn hello()", "coding", 0.9);
        assert_eq!(decision.model, "claude-code");
        assert!(decision.confidence >= router.confidence_threshold);
        assert!(decision.reason.contains("high confidence"));
    }

    #[test]
    fn test_route_low_confidence() {
        let router = SemanticRouter::default();
        let decision = router.route("fn hello()", "coding", 0.3);
        assert_eq!(decision.model, "gpt-4o");
        assert!(decision.confidence < router.confidence_threshold);
        assert!(decision.reason.contains("low confidence"));
    }

    #[test]
    fn test_add_route() {
        let mut router = SemanticRouter::default();
        router.add_route("creative", "gemini-pro");
        let decision = router.route("write a poem", "creative", 0.95);
        assert_eq!(decision.model, "gemini-pro");
    }

    #[test]
    fn test_default_routes() {
        let router = SemanticRouter::default();
        assert_eq!(
            router.route_table.get("coding").unwrap(),
            "claude-code"
        );
        assert_eq!(
            router.route_table.get("reasoning").unwrap(),
            "gpt-4o"
        );
        assert_eq!(
            router.route_table.get("simple").unwrap(),
            "gemini-flash"
        );
        assert_eq!(router.fallback_model, "gpt-4o");
        assert!((router.confidence_threshold - 0.7).abs() < f64::EPSILON);
    }
}
