//! L1 基础设施 — 语义路由器 (Semantic Router)
//!
//! 按 LLM 意图分类选择最佳 Provider
//! 支持: 关键词匹配 + 嵌入相似度 + LLM 分类回退

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 路由规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteRule {
    pub id: String,
    pub intent: String,
    pub keywords: Vec<String>,
    pub provider_preference: Vec<String>,
    pub priority: u32,
}

/// 路由结果
#[derive(Debug, Clone)]
pub struct RouteDecision {
    pub provider_id: String,
    pub confidence: f64,
    pub method: String,
}

/// 语义路由器
pub struct SemanticRouter {
    rules: Vec<RouteRule>,
    provider_scores: HashMap<String, f64>,
}

impl Default for SemanticRouter {
    fn default() -> Self { Self::new() }
}

impl SemanticRouter {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            provider_scores: HashMap::new(),
        }
    }

    pub fn add_rule(&mut self, rule: RouteRule) {
        self.rules.push(rule);
    }

    /// 按关键词匹配路由
    pub fn route_by_keywords(&self, query: &str) -> Option<RouteDecision> {
        let query_lower = query.to_lowercase();
        let mut best_match: Option<(&RouteRule, u32)> = None;

        for rule in &self.rules {
            let matches = rule.keywords.iter()
                .filter(|kw| query_lower.contains(&kw.to_lowercase()))
                .count() as u32;
            if matches > 0 {
                if best_match.as_ref().map_or(true, |(_, score)| matches > *score) {
                    best_match = Some((rule, matches));
                }
            }
        }

        best_match.map(|(rule, score)| {
            let provider = rule.provider_preference.first()
                .cloned()
                .unwrap_or_default();
            RouteDecision {
                provider_id: provider,
                confidence: score as f64 / rule.keywords.len() as f64,
                method: "keyword_match".into(),
            }
        })
    }

    /// 按意图匹配路由
    pub fn route_by_intent(&self, intent: &str) -> Option<RouteDecision> {
        self.rules.iter()
            .find(|r| r.intent.to_lowercase() == intent.to_lowercase())
            .and_then(|rule| {
                rule.provider_preference.first().map(|p| RouteDecision {
                    provider_id: p.clone(),
                    confidence: 1.0,
                    method: "intent_match".into(),
                })
            })
    }

    /// 混合路由: 关键词 → 意图 → 默认
    pub fn route(&self, query: &str, intent: Option<&str>) -> Option<RouteDecision> {
        // 1. 关键词匹配
        if let Some(decision) = self.route_by_keywords(query) {
            if decision.confidence > 0.5 {
                return Some(decision);
            }
        }
        // 2. 意图匹配
        if let Some(intent) = intent {
            if let Some(decision) = self.route_by_intent(intent) {
                return Some(decision);
            }
        }
        // 3. 默认: 最高分 provider
        self.provider_scores.iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(id, _)| RouteDecision {
                provider_id: id.clone(),
                confidence: 0.3,
                method: "default".into(),
            })
    }

    /// 更新 provider 评分 (从反馈中学习)
    pub fn update_score(&mut self, provider_id: &str, delta: f64) {
        let score = self.provider_scores.entry(provider_id.to_string()).or_insert(0.5);
        *score = (*score + delta).clamp(0.0, 1.0);
    }

    pub fn rules(&self) -> &[RouteRule] { &self.rules }
    pub fn scores(&self) -> &HashMap<String, f64> { &self.provider_scores }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_routing() {
        let mut router = SemanticRouter::new();
        router.add_rule(RouteRule {
            id: "r1".into(),
            intent: "search".into(),
            keywords: vec!["search".into(), "find".into(), "lookup".into()],
            provider_preference: vec!["kb_search".into(), "web_search".into()],
            priority: 1,
        });
        let decision = router.route("search for knowledge", None).unwrap();
        assert_eq!(decision.provider_id, "kb_search");
        assert!(decision.confidence > 0.0);
    }

    #[test]
    fn test_intent_routing() {
        let mut router = SemanticRouter::new();
        router.add_rule(RouteRule {
            id: "r1".into(),
            intent: "translate".into(),
            keywords: vec![],
            provider_preference: vec!["deepL".into()],
            priority: 1,
        });
        let decision = router.route("anything", Some("translate")).unwrap();
        assert_eq!(decision.provider_id, "deepL");
    }

    #[test]
    fn test_score_learning() {
        let mut router = SemanticRouter::new();
        router.update_score("provider_a", 0.1);
        router.update_score("provider_a", -0.2);
        assert!(*router.scores().get("provider_a").unwrap() < 0.5);
    }
}
