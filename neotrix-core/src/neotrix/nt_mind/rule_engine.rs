//! # RuleEngine — 声明式注意力路由规则引擎
//!
//! 替代 `AttentionRouter::compute_salience()` 中的硬编码关键词匹配，
//! 提供可配置、可组合的路由规则系统。
//!
//! ## 集成指南 (AttentionRouter)
//!
//! 在 `AttentionRouter` 中可选使用 `RuleEngine` 替代 `compute_salience()`:
//!
//! ```rust,ignore
//! impl AttentionRouter {
//!     /// 替代方案：基于 RuleEngine 的路由（可切换）
//!     pub fn route_with_rules(&mut self, context: &str) -> RoutedContext {
//!         let rule_engine = RuleEngine::new();
//!         let matches = rule_engine.evaluate(context);
//!
//!         // 从匹配的 RouteTo 动作构建 salience 分数
//!         let mut scores: Vec<(SpecialistType, f64)> = Vec::new();
//!         for (specialist, action) in &matches {
//!             if let RuleAction::RouteTo { priority, .. } = action {
//!                 if let Ok(st) = specialist.parse::<SpecialistType>() {
//!                     scores.push((st, *priority as f64 / 100.0));
//!                 }
//!             }
//!         }
//!
//!         // 回退: 无规则匹配时使用旧 compute_salience
//!         if scores.is_empty() {
//!             return self.route(context);
//!         }
//!
//!         scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
//!
//!         for (st, salience) in &scores {
//!             if let Some(s) = self.workspace.specialist_by_type_mut(st) {
//!                 s.activate(*salience);
//!             }
//!         }
//!
//!         let winner = scores.first().map(|(st, _)| st.name().to_string()).unwrap_or_default();
//!         self.workspace.active_content = Some(winner.clone());
//!         self.workspace.broadcast_history.push(winner.clone());
//!
//!         let active: Vec<SpecialistType> = self.workspace.active_specialists()
//!             .iter().map(|m| m.module_type).collect();
//!
//!         // 处理 InjectKnowledge 动作
//!         let mut knowledge_lines = Vec::new();
//!         for (_, action) in &matches {
//!             if let RuleAction::InjectKnowledge { domain, query } = action {
//!                 knowledge_lines.push(format!("[INJECT] {} :: {}", domain, query));
//!             }
//!         }
//!
//!         if knowledge_lines.is_empty() {
//!             for st in &active {
//!                 let entries = self.retrieve_for_specialist(*st, context);
//!                 for e in entries {
//!                     knowledge_lines.push(format!("[{}] {} ({})", st.short_name(), e.label, e.source));
//!                 }
//!             }
//!         }
//!
//!         self.workspace.decay_all(0.3);
//!
//!         RoutedContext {
//!             winning_topic: winner,
//!             active_specialists: active,
//!             knowledge_lines,
//!             salience_report: scores,
//!             stream_summary: None,
//!             prompt: None,
//!             budget: None,
//!         }
//!     }
//! }
//! ```
//!
//! 切换方式: 将 `AttentionRouter::route()` 中的 `self.compute_salience(&lower)` 调用
//! 替换为 `RuleEngine::evaluate()` + 上表映射，无需改动上游调用者。
//! `RuleEngine` 也可通过 `set_enabled()` 动态启停规则，或 `load_from_config()` 从 JSON 加载。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Core Types
// ============================================================================

/// 规则匹配后的动作
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum RuleAction {
    #[serde(rename = "route_to")]
    RouteTo { specialist: String, priority: u8 },
    #[serde(rename = "inject_knowledge")]
    InjectKnowledge { domain: String, query: String },
    #[serde(rename = "skip")]
    Skip,
    #[serde(rename = "log")]
    Log { message: String },
    #[serde(rename = "adjust_salience")]
    AdjustSalience { delta: f64 },
}

/// 规则模式（匹配条件）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum RulePattern {
    #[serde(rename = "keyword")]
    Keyword { words: Vec<String>, match_all: bool },
    #[serde(rename = "regex")]
    Regex { pattern: String },
    #[serde(rename = "source_type")]
    SourceType { source: String },
    #[serde(rename = "all_of")]
    AllOf(Vec<RulePattern>),
    #[serde(rename = "any_of")]
    AnyOf(Vec<RulePattern>),
    #[serde(rename = "not")]
    Not(Box<RulePattern>),
}

/// 一条完整路由规则
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoutingRule {
    pub id: String,
    pub name: String,
    pub priority: u8,
    pub pattern: RulePattern,
    pub action: RuleAction,
    pub enabled: bool,
    pub description: String,
}

// ============================================================================
// RuleEngine
// ============================================================================

/// 规则引擎 — 声明式注意力路由
#[derive(Debug)]
pub struct RuleEngine {
    rules: Vec<RoutingRule>,
    compiled_regex: HashMap<String, regex::Regex>,
}

impl RuleEngine {
    pub fn new() -> Self {
        Self {
            rules: Self::default_rules(),
            compiled_regex: HashMap::new(),
        }
    }

    /// 添加规则
    pub fn add_rule(&mut self, rule: RoutingRule) {
        if let RulePattern::Regex { ref pattern } = rule.pattern {
            if let Ok(re) = regex::Regex::new(pattern) {
                self.compiled_regex.insert(rule.id.clone(), re);
            }
        }
        self.rules.push(rule);
    }

    /// 移除规则
    pub fn remove_rule(&mut self, id: &str) {
        self.rules.retain(|r| r.id != id);
        self.compiled_regex.remove(id);
    }

    /// 评估上下文，返回匹配的 (specialist_name, action) 列表
    pub fn evaluate(&self, context: &str) -> Vec<(String, RuleAction)> {
        let mut sorted: Vec<&RoutingRule> = self.rules.iter().filter(|r| r.enabled).collect();
        sorted.sort_by_key(|r| r.priority);

        let mut results = Vec::new();
        for rule in sorted {
            if rule.pattern.matches(context) {
                match &rule.action {
                    RuleAction::RouteTo {
                        specialist,
                        priority,
                    } => {
                        results.push((
                            specialist.clone(),
                            RuleAction::RouteTo {
                                specialist: specialist.clone(),
                                priority: *priority,
                            },
                        ));
                    }
                    RuleAction::InjectKnowledge { domain, query } => {
                        results.push((
                            String::new(),
                            RuleAction::InjectKnowledge {
                                domain: domain.clone(),
                                query: query.clone(),
                            },
                        ));
                    }
                    RuleAction::Log { message } => {
                        results.push((
                            String::new(),
                            RuleAction::Log {
                                message: message.clone(),
                            },
                        ));
                    }
                    RuleAction::AdjustSalience { delta } => {
                        results.push((String::new(), RuleAction::AdjustSalience { delta: *delta }));
                    }
                    RuleAction::Skip => {}
                }
            }
        }
        results
    }

    /// 从 JSON 字符串加载规则
    pub fn load_from_config(&mut self, config_str: &str) -> Result<(), String> {
        let rules: Vec<RoutingRule> =
            serde_json::from_str(config_str).map_err(|e| format!("JSON parse error: {}", e))?;
        for rule in rules {
            self.add_rule(rule);
        }
        Ok(())
    }

    /// 序列化为 JSON
    pub fn export_to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(&self.rules)
            .map_err(|e| format!("JSON serialize error: {}", e))
    }

    /// 启停规则
    pub fn set_enabled(&mut self, id: &str, enabled: bool) {
        if let Some(rule) = self.rules.iter_mut().find(|r| r.id == id) {
            rule.enabled = enabled;
        }
    }

    /// 列出所有规则
    pub fn list_rules(&self) -> &[RoutingRule] {
        &self.rules
    }

    /// 清空所有规则
    pub fn clear(&mut self) {
        self.rules.clear();
        self.compiled_regex.clear();
    }

    /// 生成 7 条默认规则，模仿 AttentionRouter 的硬编码匹配
    pub fn default_rules() -> Vec<RoutingRule> {
        vec![
            RoutingRule {
                id: "pattern-matcher".into(),
                name: "Pattern Matcher".into(),
                priority: 10,
                pattern: RulePattern::Keyword {
                    words: vec![
                        "pattern".into(),
                        "repeat".into(),
                        "template".into(),
                        "structure".into(),
                        "trend".into(),
                        "similar".into(),
                        "common".into(),
                        "regular".into(),
                        "cycle".into(),
                        "algorithm".into(),
                    ],
                    match_all: false,
                },
                action: RuleAction::RouteTo {
                    specialist: "PatternMatcher".into(),
                    priority: 70,
                },
                enabled: true,
                description: "Route pattern/anomaly/trend language to PatternMatcher".into(),
            },
            RoutingRule {
                id: "anomaly-detector".into(),
                name: "Anomaly Detector".into(),
                priority: 10,
                pattern: RulePattern::Keyword {
                    words: vec![
                        "error".into(),
                        "bug".into(),
                        "crash".into(),
                        "fail".into(),
                        "unusual".into(),
                        "exception".into(),
                        "unexpected".into(),
                        "wrong".into(),
                        "broken".into(),
                        "issue".into(),
                        "problem".into(),
                    ],
                    match_all: false,
                },
                action: RuleAction::RouteTo {
                    specialist: "AnomalyDetector".into(),
                    priority: 70,
                },
                enabled: true,
                description: "Route error/exception/bug language to AnomalyDetector".into(),
            },
            RoutingRule {
                id: "knowledge-integrator".into(),
                name: "Knowledge Integrator".into(),
                priority: 10,
                pattern: RulePattern::Keyword {
                    words: vec![
                        "knowledge".into(),
                        "learn".into(),
                        "understand".into(),
                        "combine".into(),
                        "integrate".into(),
                        "synthesize".into(),
                        "connect".into(),
                        "relate".into(),
                        "overview".into(),
                        "survey".into(),
                    ],
                    match_all: false,
                },
                action: RuleAction::RouteTo {
                    specialist: "KnowledgeIntegrator".into(),
                    priority: 60,
                },
                enabled: true,
                description: "Route knowledge/synthesis language to KnowledgeIntegrator".into(),
            },
            RoutingRule {
                id: "goal-prioritizer".into(),
                name: "Goal Prioritizer".into(),
                priority: 10,
                pattern: RulePattern::Keyword {
                    words: vec![
                        "goal".into(),
                        "plan".into(),
                        "priority".into(),
                        "objective".into(),
                        "milestone".into(),
                        "strategy".into(),
                        "roadmap".into(),
                        "next".into(),
                        "schedule".into(),
                        "deadline".into(),
                    ],
                    match_all: false,
                },
                action: RuleAction::RouteTo {
                    specialist: "GoalPrioritizer".into(),
                    priority: 70,
                },
                enabled: true,
                description: "Route goal/plan/strategy language to GoalPrioritizer".into(),
            },
            RoutingRule {
                id: "risk-assessor".into(),
                name: "Risk Assessor".into(),
                priority: 10,
                pattern: RulePattern::Keyword {
                    words: vec![
                        "risk".into(),
                        "nt_shield".into(),
                        "danger".into(),
                        "warn".into(),
                        "vulnerability".into(),
                        "threat".into(),
                        "safe".into(),
                        "protect".into(),
                        "audit".into(),
                        "breach".into(),
                    ],
                    match_all: false,
                },
                action: RuleAction::RouteTo {
                    specialist: "RiskAssessor".into(),
                    priority: 70,
                },
                enabled: true,
                description: "Route risk/sec language to RiskAssessor".into(),
            },
            RoutingRule {
                id: "creativity-generator".into(),
                name: "Creativity Generator".into(),
                priority: 10,
                pattern: RulePattern::Keyword {
                    words: vec![
                        "creative".into(),
                        "novel".into(),
                        "innovate".into(),
                        "design".into(),
                        "imagine".into(),
                        "invent".into(),
                        "explore".into(),
                        "possibility".into(),
                        "brainstorm".into(),
                        "idea".into(),
                    ],
                    match_all: false,
                },
                action: RuleAction::RouteTo {
                    specialist: "CreativityGenerator".into(),
                    priority: 60,
                },
                enabled: true,
                description: "Route creative/design language to CreativityGenerator".into(),
            },
            RoutingRule {
                id: "reflection-engine".into(),
                name: "Reflection Engine".into(),
                priority: 10,
                pattern: RulePattern::Keyword {
                    words: vec![
                        "reflect".into(),
                        "review".into(),
                        "improve".into(),
                        "optimize".into(),
                        "evolve".into(),
                        "retrospect".into(),
                        "lesson".into(),
                        "growth".into(),
                        "iterate".into(),
                        "meta".into(),
                    ],
                    match_all: false,
                },
                action: RuleAction::RouteTo {
                    specialist: "ReflectionEngine".into(),
                    priority: 60,
                },
                enabled: true,
                description: "Route reflection/meta language to ReflectionEngine".into(),
            },
        ]
    }
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// RulePattern matching
// ============================================================================

impl RulePattern {
    /// 判断一个上下文是否匹配此模式
    pub fn matches(&self, context: &str) -> bool {
        let lower = &context.to_lowercase();
        match self {
            RulePattern::Keyword { words, match_all } => {
                if *match_all {
                    words.iter().all(|w| lower.contains(&w.to_lowercase()))
                } else {
                    words.iter().any(|w| lower.contains(&w.to_lowercase()))
                }
            }
            RulePattern::Regex { pattern } => regex::Regex::new(pattern)
                .map(|re| re.is_match(lower))
                .unwrap_or(false),
            RulePattern::SourceType { source } => lower.contains(&source.to_lowercase()),
            RulePattern::AllOf(patterns) => patterns.iter().all(|p| p.matches(context)),
            RulePattern::AnyOf(patterns) => patterns.iter().any(|p| p.matches(context)),
            RulePattern::Not(pattern) => !pattern.matches(context),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- RulePattern::matches tests ---

    #[test]
    fn test_keyword_match_any() {
        let pattern = RulePattern::Keyword {
            words: vec!["error".into(), "bug".into()],
            match_all: false,
        };
        assert!(pattern.matches("found a bug in the code"));
        assert!(pattern.matches("error occurred"));
        assert!(!pattern.matches("everything is fine"));
    }

    #[test]
    fn test_keyword_match_all() {
        let pattern = RulePattern::Keyword {
            words: vec!["error".into(), "fix".into()],
            match_all: true,
        };
        assert!(pattern.matches("need to fix this error"));
        assert!(!pattern.matches("only error no fix"));
        assert!(!pattern.matches("nothing here"));
    }

    #[test]
    fn test_regex_pattern() {
        let pattern = RulePattern::Regex {
            pattern: r"bug\s+#\d+".into(),
        };
        assert!(pattern.matches("fix bug #1234 in main"));
        assert!(!pattern.matches("bug report"));
    }

    #[test]
    fn test_regex_invalid_pattern_returns_false() {
        let pattern = RulePattern::Regex {
            pattern: r"[invalid".into(),
        };
        assert!(!pattern.matches("anything"));
    }

    #[test]
    fn test_source_type_match() {
        let pattern = RulePattern::SourceType {
            source: "wikipedia".into(),
        };
        assert!(pattern.matches("source: wikipedia article"));
        assert!(!pattern.matches("source: github"));
    }

    #[test]
    fn test_all_of_compound() {
        let pattern = RulePattern::AllOf(vec![
            RulePattern::Keyword {
                words: vec!["rust".into()],
                match_all: false,
            },
            RulePattern::Regex {
                pattern: r"unsafe".into(),
            },
        ]);
        assert!(pattern.matches("rust code with unsafe block"));
        assert!(!pattern.matches("rust code is safe"));
    }

    #[test]
    fn test_any_of_compound() {
        let pattern = RulePattern::AnyOf(vec![
            RulePattern::SourceType {
                source: "arxiv".into(),
            },
            RulePattern::Keyword {
                words: vec!["paper".into()],
                match_all: false,
            },
        ]);
        assert!(pattern.matches("read the paper on attention"));
        assert!(pattern.matches("source: arxiv"));
        assert!(!pattern.matches("just a blog post"));
    }

    #[test]
    fn test_not_pattern() {
        let pattern = RulePattern::Not(Box::new(RulePattern::Keyword {
            words: vec!["error".into()],
            match_all: false,
        }));
        assert!(pattern.matches("everything is fine"));
        assert!(!pattern.matches("an error occurred"));
    }

    #[test]
    fn test_nested_compound() {
        let pattern = RulePattern::AllOf(vec![
            RulePattern::Keyword {
                words: vec!["security".into()],
                match_all: false,
            },
            RulePattern::AnyOf(vec![
                RulePattern::Keyword {
                    words: vec!["audit".into(), "review".into()],
                    match_all: false,
                },
                RulePattern::Not(Box::new(RulePattern::Keyword {
                    words: vec!["critical".into()],
                    match_all: false,
                })),
            ]),
        ]);
        assert!(pattern.matches("security audit required"));
        assert!(pattern.matches("security review"));
        assert!(pattern.matches("security check"));
        assert!(!pattern.matches("security critical situation"));
    }

    // --- RuleEngine tests ---

    #[test]
    fn test_new_has_seven_default_rules() {
        let engine = RuleEngine::new();
        assert_eq!(engine.rules.len(), 7);
    }

    #[test]
    fn test_default_rules_pattern_matcher() {
        let engine = RuleEngine::new();
        let results = engine.evaluate("find the pattern in this structure");
        assert!(results.iter().any(|(s, _)| s == "PatternMatcher"));
    }

    #[test]
    fn test_default_rules_anomaly_detector() {
        let engine = RuleEngine::new();
        let results = engine.evaluate("critical bug in production, crash detected");
        assert!(results.iter().any(|(s, _)| s == "AnomalyDetector"));
    }

    #[test]
    fn test_default_rules_knowledge_integrator() {
        let engine = RuleEngine::new();
        let results = engine.evaluate("synthesize knowledge from multiple sources");
        assert!(results.iter().any(|(s, _)| s == "KnowledgeIntegrator"));
    }

    #[test]
    fn test_default_rules_goal_prioritizer() {
        let engine = RuleEngine::new();
        let results = engine.evaluate("set roadmap milestones for next quarter");
        assert!(results.iter().any(|(s, _)| s == "GoalPrioritizer"));
    }

    #[test]
    fn test_default_rules_risk_assessor() {
        let engine = RuleEngine::new();
        let results = engine.evaluate("security vulnerability threat detected");
        assert!(results.iter().any(|(s, _)| s == "RiskAssessor"));
    }

    #[test]
    fn test_default_rules_creativity_generator() {
        let engine = RuleEngine::new();
        let results = engine.evaluate("brainstorm novel design ideas");
        assert!(results.iter().any(|(s, _)| s == "CreativityGenerator"));
    }

    #[test]
    fn test_default_rules_reflection_engine() {
        let engine = RuleEngine::new();
        let results = engine.evaluate("reflect on lessons learned and optimize");
        assert!(results.iter().any(|(s, _)| s == "ReflectionEngine"));
    }

    #[test]
    fn test_add_and_remove_rule() {
        let mut engine = RuleEngine::new();
        let rule = RoutingRule {
            id: "custom-test".into(),
            name: "Test Rule".into(),
            priority: 5,
            pattern: RulePattern::Keyword {
                words: vec!["custom".into()],
                match_all: false,
            },
            action: RuleAction::RouteTo {
                specialist: "CustomHandler".into(),
                priority: 80,
            },
            enabled: true,
            description: "test".into(),
        };
        engine.add_rule(rule);
        assert_eq!(engine.rules.len(), 8);

        let results = engine.evaluate("custom context");
        assert!(results.iter().any(|(s, _)| s == "CustomHandler"));

        engine.remove_rule("custom-test");
        assert_eq!(engine.rules.len(), 7);
    }

    #[test]
    fn test_set_enabled() {
        let mut engine = RuleEngine::new();
        engine.set_enabled("pattern-matcher", false);
        let results = engine.evaluate("pattern in structure");
        assert!(!results.iter().any(|(s, _)| s == "PatternMatcher"));
    }

    #[test]
    fn test_clear() {
        let mut engine = RuleEngine::new();
        engine.clear();
        assert_eq!(engine.rules.len(), 0);
    }

    #[test]
    fn test_list_rules() {
        let engine = RuleEngine::new();
        assert_eq!(engine.list_rules().len(), 7);
    }

    #[test]
    fn test_evaluate_multiple_matches() {
        let engine = RuleEngine::new();
        let results = engine.evaluate("bug in the pattern, need to reflect");
        assert!(results.iter().any(|(s, _)| s == "PatternMatcher"));
        assert!(results.iter().any(|(s, _)| s == "AnomalyDetector"));
        assert!(results.iter().any(|(s, _)| s == "ReflectionEngine"));
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_evaluate_no_match() {
        let engine = RuleEngine::new();
        let results = engine.evaluate("xyzzzz qwerty plugh");
        assert!(results.is_empty());
    }

    #[test]
    fn test_skip_action_does_not_return_result() {
        let mut engine = RuleEngine::new();
        engine.add_rule(RoutingRule {
            id: "skip-test".into(),
            name: "Skip Rule".into(),
            priority: 1,
            pattern: RulePattern::Keyword {
                words: vec!["trigger".into()],
                match_all: false,
            },
            action: RuleAction::Skip,
            enabled: true,
            description: "test skip".into(),
        });
        let results = engine.evaluate("trigger the skip");
        assert!(results.is_empty());
    }

    #[test]
    fn test_log_action() {
        let mut engine = RuleEngine::new();
        engine.add_rule(RoutingRule {
            id: "log-test".into(),
            name: "Log Rule".into(),
            priority: 1,
            pattern: RulePattern::Keyword {
                words: vec!["xyzlogmarker".into()],
                match_all: false,
            },
            action: RuleAction::Log {
                message: "log marker encountered".into(),
            },
            enabled: true,
            description: "test log".into(),
        });
        let results = engine.evaluate("here comes the xyzlogmarker");
        assert_eq!(results.len(), 1);
        assert_eq!(
            results[0].1,
            RuleAction::Log {
                message: "log marker encountered".into()
            }
        );
    }

    #[test]
    fn test_adjust_salience_action() {
        let mut engine = RuleEngine::new();
        engine.add_rule(RoutingRule {
            id: "adjust-test".into(),
            name: "Adjust Rule".into(),
            priority: 1,
            pattern: RulePattern::Regex {
                pattern: r"xyzurgent|critical".into(),
            },
            action: RuleAction::AdjustSalience { delta: 0.2 },
            enabled: true,
            description: "boost urgent contexts".into(),
        });
        let results = engine.evaluate("this is xyzurgent");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1, RuleAction::AdjustSalience { delta: 0.2 });
    }

    #[test]
    fn test_inject_knowledge_action() {
        let mut engine = RuleEngine::new();
        engine.add_rule(RoutingRule {
            id: "inject-test".into(),
            name: "Inject Rule".into(),
            priority: 1,
            pattern: RulePattern::SourceType {
                source: "wikipedia".into(),
            },
            action: RuleAction::InjectKnowledge {
                domain: "science".into(),
                query: "quantum mechanics".into(),
            },
            enabled: true,
            description: "inject physics knowledge".into(),
        });
        let results = engine.evaluate("from wikipedia article");
        assert_eq!(results.len(), 1);
        match &results[0].1 {
            RuleAction::InjectKnowledge { domain, query } => {
                assert_eq!(domain, "science");
                assert_eq!(query, "quantum mechanics");
            }
            _ => panic!("expected InjectKnowledge"),
        }
    }

    #[test]
    fn test_load_from_json() {
        let json_str = r#"[
  {
    "id": "json-rule",
    "name": "JSON Rule",
    "priority": 5,
    "pattern": { "type": "keyword", "words": ["json"], "match_all": false },
    "action": { "type": "route_to", "specialist": "ConfigHandler", "priority": 90 },
    "enabled": true,
    "description": "loaded from json"
  },
  {
    "id": "json-regex",
    "name": "Regex Rule",
    "priority": 10,
    "pattern": { "type": "regex", "pattern": "v\\d+\\.\\d+" },
    "action": { "type": "log", "message": "version found" },
    "enabled": true,
    "description": "regex from json"
  }
]"#;

        let mut engine = RuleEngine::new();
        engine.load_from_config(json_str).unwrap();
        assert_eq!(engine.rules.len(), 9);

        let results = engine.evaluate("load from json config");
        assert!(results.iter().any(|(s, _)| s == "ConfigHandler"));
    }

    #[test]
    fn test_export_to_json() {
        let mut engine = RuleEngine::new();
        engine.add_rule(RoutingRule {
            id: "export-test".into(),
            name: "Export Test".into(),
            priority: 1,
            pattern: RulePattern::Keyword {
                words: vec!["export".into()],
                match_all: false,
            },
            action: RuleAction::RouteTo {
                specialist: "ExportHandler".into(),
                priority: 50,
            },
            enabled: true,
            description: "export test".into(),
        });
        let json_out = engine.export_to_json().unwrap();
        assert!(json_out.contains("export-test"));
        assert!(json_out.contains("ExportHandler"));
    }

    #[test]
    fn test_invalid_json_returns_error() {
        let mut engine = RuleEngine::new();
        let result = engine.load_from_config("not valid json {{{");
        assert!(result.is_err());
    }

    #[test]
    fn test_disabled_rule_not_evaluated() {
        let mut engine = RuleEngine::new();
        engine.set_enabled("anomaly-detector", false);
        let results = engine.evaluate("critical bug crash error");
        assert!(!results.iter().any(|(s, _)| s == "AnomalyDetector"));
    }

    #[test]
    fn test_priority_order() {
        let mut engine = RuleEngine::new();
        engine.add_rule(RoutingRule {
            id: "high-pri".into(),
            name: "High Priority".into(),
            priority: 1,
            pattern: RulePattern::Keyword {
                words: vec!["test".into()],
                match_all: false,
            },
            action: RuleAction::RouteTo {
                specialist: "FirstHandler".into(),
                priority: 95,
            },
            enabled: true,
            description: "high priority".into(),
        });
        engine.add_rule(RoutingRule {
            id: "low-pri".into(),
            name: "Low Priority".into(),
            priority: 100,
            pattern: RulePattern::Keyword {
                words: vec!["test".into()],
                match_all: false,
            },
            action: RuleAction::RouteTo {
                specialist: "LastHandler".into(),
                priority: 10,
            },
            enabled: true,
            description: "low priority".into(),
        });
        let results = engine.evaluate("test context");
        assert_eq!(results[0].0, "FirstHandler");
    }

    #[test]
    fn test_not_rule_excludes_specialist() {
        let mut engine = RuleEngine::new();
        engine.set_enabled("pattern-matcher", false);
        engine.set_enabled("anomaly-detector", false);
        engine.set_enabled("knowledge-integrator", false);
        engine.set_enabled("goal-prioritizer", false);
        engine.set_enabled("risk-assessor", false);
        engine.set_enabled("creativity-generator", false);
        engine.set_enabled("reflection-engine", false);

        engine.add_rule(RoutingRule {
            id: "reflection-but-not-meta".into(),
            name: "Reflection without meta".into(),
            priority: 1,
            pattern: RulePattern::AllOf(vec![
                RulePattern::Keyword {
                    words: vec!["reflect".into()],
                    match_all: false,
                },
                RulePattern::Not(Box::new(RulePattern::Keyword {
                    words: vec!["meta".into()],
                    match_all: false,
                })),
            ]),
            action: RuleAction::RouteTo {
                specialist: "PureReflection".into(),
                priority: 80,
            },
            enabled: true,
            description: "reflect but not meta".into(),
        });

        assert!(engine
            .evaluate("reflect on the code")
            .iter()
            .any(|(s, _)| s == "PureReflection"));
        assert!(!engine
            .evaluate("meta reflection")
            .iter()
            .any(|(s, _)| s == "PureReflection"));
    }

    #[test]
    fn test_all_of_matches_only_when_all_satisfied() {
        let pattern = RulePattern::AllOf(vec![
            RulePattern::Keyword {
                words: vec!["performance".into()],
                match_all: false,
            },
            RulePattern::Regex {
                pattern: r"O\(n\)".into(),
            },
        ]);
        assert!(pattern.matches("need O(n) performance"));
        assert!(!pattern.matches("good performance"));
    }
}
