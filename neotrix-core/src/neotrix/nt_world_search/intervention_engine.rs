use std::hash::{Hash, Hasher};
use std::time::Instant;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InterventionType {
    ContextAugmentation,
    ToolReordering,
    PromptPatch,
    ErrorHandler,
    RetryStrategy,
    ParameterOverride,
    PreconditionCheck,
}

impl InterventionType {
    pub fn label(&self) -> &'static str {
        match self {
            InterventionType::ContextAugmentation => "ContextAugmentation",
            InterventionType::ToolReordering => "ToolReordering",
            InterventionType::PromptPatch => "PromptPatch",
            InterventionType::ErrorHandler => "ErrorHandler",
            InterventionType::RetryStrategy => "RetryStrategy",
            InterventionType::ParameterOverride => "ParameterOverride",
            InterventionType::PreconditionCheck => "PreconditionCheck",
        }
    }
}

#[derive(Clone, Debug)]
pub struct InterventionTemplate {
    pub id: String,
    pub intervention_type: InterventionType,
    pub trigger_pattern: String,
    pub action: String,
    pub success_count: u64,
    pub failure_count: u64,
    pub effectiveness: f64,
    pub created_at: Instant,
    pub last_applied: Option<Instant>,
    pub tool_name: String,
    pub confidence_threshold: f64,
}

#[derive(Clone, Debug)]
pub struct FailurePattern {
    pub tool_name: String,
    pub error_type: String,
    pub context_snippet: String,
    pub frequency: u64,
    pub first_seen: Instant,
    pub last_seen: Instant,
    pub suggested_intervention: InterventionType,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn suggested_intervention_for(error_type: &str) -> InterventionType {
    match error_type {
        e if e.contains("timeout") => InterventionType::RetryStrategy,
        e if e.contains("parse") || e.contains("format") => InterventionType::PromptPatch,
        e if e.contains("auth") || e.contains("permission") => InterventionType::PreconditionCheck,
        e if e.contains("empty") || e.contains("not_found") => InterventionType::ToolReordering,
        _ => InterventionType::ErrorHandler,
    }
}

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

pub struct InterventionEngine {
    pub templates: Vec<InterventionTemplate>,
    pub failure_patterns: Vec<FailurePattern>,
    pub total_interventions_applied: u64,
    pub total_interventions_attempted: u64,
    pub pattern_discovery_enabled: bool,
    pub auto_apply_threshold: f64,
}

impl Default for InterventionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl InterventionEngine {
    const MAX_FAILURE_PATTERNS: usize = 1000;
    const MAX_TEMPLATES: usize = 500;

    pub fn new() -> Self {
        Self {
            templates: Vec::new(),
            failure_patterns: Vec::new(),
            total_interventions_applied: 0,
            total_interventions_attempted: 0,
            pattern_discovery_enabled: true,
            auto_apply_threshold: 0.8,
        }
    }

    pub fn discover_pattern(&mut self, tool_name: &str, error_type: &str, context: &str) {
        if !self.pattern_discovery_enabled {
            return;
        }

        let now = Instant::now();
        let tn = tool_name.to_string();
        let et = error_type.to_string();

        let idx = self
            .failure_patterns
            .iter()
            .position(|p| p.tool_name == tn && p.error_type == et);

        if let Some(i) = idx {
            let freq = {
                let pattern = &mut self.failure_patterns[i];
                pattern.frequency += 1;
                pattern.last_seen = now;
                pattern.frequency
            };
            if freq > 3 && !self.has_template_for(&tn, &et) {
                let pattern = self.failure_patterns[i].clone();
                let template = self.create_template(&pattern);
                if self.templates.len() >= Self::MAX_TEMPLATES {
                    let remove = self.templates.len() / 5;
                    self.templates.drain(..remove);
                }
                self.templates.push(template);
            }
            return;
        }

        let suggested = suggested_intervention_for(error_type);

        if self.failure_patterns.len() >= Self::MAX_FAILURE_PATTERNS {
            let remove = self.failure_patterns.len() / 5;
            self.failure_patterns.drain(..remove);
        }
        self.failure_patterns.push(FailurePattern {
            tool_name: tn,
            error_type: et,
            context_snippet: context.chars().take(200).collect(),
            frequency: 1,
            first_seen: now,
            last_seen: now,
            suggested_intervention: suggested,
        });
    }

    fn has_template_for(&self, tool_name: &str, error_type: &str) -> bool {
        self.templates
            .iter()
            .any(|t| t.tool_name == tool_name && t.trigger_pattern.contains(error_type))
    }

    pub fn create_template(&mut self, pattern: &FailurePattern) -> InterventionTemplate {
        let hash_str = format!("{}/{}", pattern.tool_name, pattern.error_type);
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        hash_str.hash(&mut hasher);
        let id = format!("int_{:016x}", hasher.finish());

        let (action, confidence) = match pattern.suggested_intervention {
            InterventionType::RetryStrategy => (
                format!(
                    "Apply exponential backoff retry for {} on {}",
                    pattern.tool_name, pattern.error_type
                ),
                0.75,
            ),
            InterventionType::PromptPatch => (
                format!(
                    "Fix output format parsing for {} after {}",
                    pattern.tool_name, pattern.error_type
                ),
                0.7,
            ),
            InterventionType::PreconditionCheck => (
                format!(
                    "Validate authentication/connection before calling {}",
                    pattern.tool_name
                ),
                0.85,
            ),
            InterventionType::ToolReordering => (
                format!(
                    "Try alternative tool before {} when {} occurs",
                    pattern.tool_name, pattern.error_type
                ),
                0.65,
            ),
            InterventionType::ErrorHandler => (
                format!(
                    "Add custom error recovery logic for {} on {}",
                    pattern.tool_name, pattern.error_type
                ),
                0.6,
            ),
            InterventionType::ContextAugmentation => (
                format!("Add missing context before calling {}", pattern.tool_name),
                0.7,
            ),
            InterventionType::ParameterOverride => (
                format!(
                    "Override default parameters for {} to avoid {}",
                    pattern.tool_name, pattern.error_type
                ),
                0.7,
            ),
        };

        InterventionTemplate {
            id,
            intervention_type: pattern.suggested_intervention,
            trigger_pattern: format!("{} when {}", pattern.tool_name, pattern.error_type),
            action,
            success_count: 0,
            failure_count: 0,
            effectiveness: 0.0,
            created_at: Instant::now(),
            last_applied: None,
            tool_name: pattern.tool_name.clone(),
            confidence_threshold: confidence,
        }
    }

    pub fn apply_intervention(&mut self, template_id: &str, success: bool) {
        self.total_interventions_attempted += 1;
        if let Some(template) = self.templates.iter_mut().find(|t| t.id == template_id) {
            if success {
                template.success_count += 1;
            } else {
                template.failure_count += 1;
            }
            let total = template.success_count + template.failure_count;
            template.effectiveness = if total > 0 {
                template.success_count as f64 / total as f64
            } else {
                0.0
            };
            template.last_applied = Some(Instant::now());
            if success {
                self.total_interventions_applied += 1;
            }
        }
    }

    pub fn suggest_interventions(
        &self,
        tool_name: &str,
        error_type: &str,
    ) -> Vec<&InterventionTemplate> {
        let mut matched: Vec<&InterventionTemplate> = self
            .templates
            .iter()
            .filter(|t| t.tool_name == tool_name && t.trigger_pattern.contains(error_type))
            .collect();
        matched.sort_by(|a, b| {
            b.effectiveness
                .partial_cmp(&a.effectiveness)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        matched
    }

    pub fn auto_apply_interventions(
        &self,
        tool_name: &str,
        error_type: &str,
    ) -> Vec<&InterventionTemplate> {
        let threshold = self.auto_apply_threshold;
        let matched: Vec<&InterventionTemplate> = self
            .templates
            .iter()
            .filter(|t| {
                t.tool_name == tool_name
                    && t.trigger_pattern.contains(error_type)
                    && t.effectiveness >= threshold
            })
            .collect();
        matched
    }

    pub fn best_interventions(&self, n: usize) -> Vec<&InterventionTemplate> {
        let mut sorted: Vec<&InterventionTemplate> = self.templates.iter().collect();
        sorted.sort_by(|a, b| {
            b.effectiveness
                .partial_cmp(&a.effectiveness)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted.into_iter().take(n).collect()
    }

    pub fn discovery_summary(&self) -> String {
        let mut parts = Vec::new();
        parts.push(format!("== Intervention Engine Summary =="));
        parts.push(format!(
            "Patterns discovered: {}",
            self.failure_patterns.len()
        ));
        parts.push(format!("Templates created: {}", self.templates.len()));
        parts.push(format!(
            "Interventions applied/attempted: {}/{}",
            self.total_interventions_applied, self.total_interventions_attempted
        ));
        parts.push(format!(
            "Success rate: {:.2}%",
            self.intervention_success_rate() * 100.0
        ));
        parts.push(format!(
            "Pattern diversity: {} types",
            self.pattern_diversity()
        ));

        if !self.failure_patterns.is_empty() {
            parts.push(String::from(""));
            parts.push(String::from("--- Failure Patterns ---"));
            for p in &self.failure_patterns {
                parts.push(format!(
                    "  [{}] {}: freq={}, intervention={:?}",
                    p.tool_name, p.error_type, p.frequency, p.suggested_intervention
                ));
            }
        }

        if !self.templates.is_empty() {
            parts.push(String::from(""));
            parts.push(String::from("--- Intervention Templates ---"));
            for t in &self.templates {
                parts.push(format!(
                    "  [{}] {:?} on {}: effectiveness={:.2}, action={}",
                    t.id, t.intervention_type, t.tool_name, t.effectiveness, t.action
                ));
            }
        }

        parts.join("\n")
    }

    pub fn intervention_success_rate(&self) -> f64 {
        if self.total_interventions_attempted == 0 {
            return 0.0;
        }
        self.total_interventions_applied as f64 / self.total_interventions_attempted as f64
    }

    pub fn pattern_diversity(&self) -> usize {
        self.templates
            .iter()
            .map(|t| t.intervention_type)
            .collect::<std::collections::HashSet<_>>()
            .len()
    }
}

// ---------------------------------------------------------------------------
// Global singleton
// ---------------------------------------------------------------------------

static INTERVENTION_ENGINE: std::sync::OnceLock<std::sync::Mutex<InterventionEngine>> =
    std::sync::OnceLock::new();

pub fn global_intervention_engine() -> &'static std::sync::Mutex<InterventionEngine> {
    INTERVENTION_ENGINE.get_or_init(|| std::sync::Mutex::new(InterventionEngine::new()))
}

pub fn record_failure_pattern(tool_name: &str, error_type: &str, context: &str) {
    if let Ok(mut engine) = global_intervention_engine().lock() {
        engine.discover_pattern(tool_name, error_type, context);
    }
}

pub fn suggest_tool_interventions(tool_name: &str, error_type: &str) -> Vec<String> {
    if let Ok(engine) = global_intervention_engine().lock() {
        engine
            .suggest_interventions(tool_name, error_type)
            .into_iter()
            .map(|t| {
                format!(
                    "[{}] {} (effectiveness: {:.2})",
                    t.id, t.action, t.effectiveness
                )
            })
            .collect()
    } else {
        Vec::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[serial]
    #[test]
    fn test_discover_pattern_first_call_creates() {
        let mut engine = InterventionEngine::new();
        engine.discover_pattern("search", "timeout", "fetching results from API");
        assert_eq!(engine.failure_patterns.len(), 1);
        let p = &engine.failure_patterns[0];
        assert_eq!(p.tool_name, "search");
        assert_eq!(p.error_type, "timeout");
        assert_eq!(p.frequency, 1);
    }

    #[test]
    fn test_discover_pattern_second_call_increments() {
        let mut engine = InterventionEngine::new();
        engine.discover_pattern("search", "timeout", "fetching results");
        engine.discover_pattern("search", "timeout", "fetching results again");
        assert_eq!(engine.failure_patterns.len(), 1);
        assert_eq!(engine.failure_patterns[0].frequency, 2);
    }

    #[test]
    fn test_auto_create_template_at_frequency_above_three() {
        let mut engine = InterventionEngine::new();
        engine.discover_pattern("reader", "timeout", "reading data");
        assert_eq!(engine.templates.len(), 0);
        engine.discover_pattern("reader", "timeout", "reading data");
        assert_eq!(engine.templates.len(), 0);
        engine.discover_pattern("reader", "timeout", "reading data");
        assert_eq!(engine.templates.len(), 0);
        // 4th call: frequency > 3 → creates template
        engine.discover_pattern("reader", "timeout", "reading data");
        assert_eq!(engine.templates.len(), 1);
        assert_eq!(engine.templates[0].tool_name, "reader");
        assert_eq!(
            engine.templates[0].intervention_type,
            InterventionType::RetryStrategy
        );
    }

    #[test]
    fn test_apply_intervention_updates_effectiveness() {
        let mut engine = InterventionEngine::new();
        engine.discover_pattern("tool_a", "parse_error", "bad format");
        engine.discover_pattern("tool_a", "parse_error", "bad format");
        engine.discover_pattern("tool_a", "parse_error", "bad format");
        engine.discover_pattern("tool_a", "parse_error", "bad format");
        let template_id = engine.templates[0].id.clone();
        assert_eq!(engine.templates[0].effectiveness, 0.0);

        engine.apply_intervention(&template_id, true);
        assert_eq!(engine.templates[0].success_count, 1);
        assert_eq!(engine.templates[0].effectiveness, 1.0);

        engine.apply_intervention(&template_id, false);
        assert_eq!(engine.templates[0].failure_count, 1);
        assert_eq!(engine.templates[0].effectiveness, 0.5);
    }

    #[test]
    fn test_suggest_interventions_sorted_by_effectiveness() {
        let mut engine = InterventionEngine::new();

        // Manually create two templates for same tool+error with different effectiveness
        let mut t1 = engine.create_template(&FailurePattern {
            tool_name: "git".into(),
            error_type: "timeout".into(),
            context_snippet: "clone".into(),
            frequency: 5,
            first_seen: Instant::now(),
            last_seen: Instant::now(),
            suggested_intervention: InterventionType::RetryStrategy,
        });
        t1.effectiveness = 0.9;
        t1.id = "t1".into();
        t1.trigger_pattern = "git when timeout".into();
        engine.templates.push(t1);

        let mut t2 = engine.create_template(&FailurePattern {
            tool_name: "git".into(),
            error_type: "timeout".into(),
            context_snippet: "fetch".into(),
            frequency: 5,
            first_seen: Instant::now(),
            last_seen: Instant::now(),
            suggested_intervention: InterventionType::PreconditionCheck,
        });
        t2.effectiveness = 0.5;
        t2.id = "t2".into();
        t2.trigger_pattern = "git when timeout".into();
        engine.templates.push(t2);

        let suggestions = engine.suggest_interventions("git", "timeout");
        assert_eq!(suggestions.len(), 2);
        assert_eq!(suggestions[0].id, "t1");
        assert_eq!(suggestions[1].id, "t2");
    }

    #[test]
    fn test_best_interventions_returns_top_n() {
        let mut engine = InterventionEngine::new();

        let mut t1 = engine.create_template(&FailurePattern {
            tool_name: "a".into(),
            error_type: "timeout".into(),
            context_snippet: "".into(),
            frequency: 5,
            first_seen: Instant::now(),
            last_seen: Instant::now(),
            suggested_intervention: InterventionType::RetryStrategy,
        });
        t1.effectiveness = 0.3;
        t1.id = "t1".into();
        engine.templates.push(t1);

        let mut t2 = engine.create_template(&FailurePattern {
            tool_name: "b".into(),
            error_type: "timeout".into(),
            context_snippet: "".into(),
            frequency: 5,
            first_seen: Instant::now(),
            last_seen: Instant::now(),
            suggested_intervention: InterventionType::RetryStrategy,
        });
        t2.effectiveness = 0.9;
        t2.id = "t2".into();
        engine.templates.push(t2);

        let mut t3 = engine.create_template(&FailurePattern {
            tool_name: "c".into(),
            error_type: "timeout".into(),
            context_snippet: "".into(),
            frequency: 5,
            first_seen: Instant::now(),
            last_seen: Instant::now(),
            suggested_intervention: InterventionType::RetryStrategy,
        });
        t3.effectiveness = 0.6;
        t3.id = "t3".into();
        engine.templates.push(t3);

        let top = engine.best_interventions(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].id, "t2");
        assert_eq!(top[1].id, "t3");
    }

    #[test]
    fn test_intervention_success_rate_calculation() {
        let mut engine = InterventionEngine::new();
        assert_eq!(engine.intervention_success_rate(), 0.0);

        engine.total_interventions_attempted = 10;
        engine.total_interventions_applied = 7;
        assert!((engine.intervention_success_rate() - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn test_pattern_diversity_count() {
        let mut engine = InterventionEngine::new();

        let mut t1 = engine.create_template(&FailurePattern {
            tool_name: "x".into(),
            error_type: "timeout".into(),
            context_snippet: "".into(),
            frequency: 5,
            first_seen: Instant::now(),
            last_seen: Instant::now(),
            suggested_intervention: InterventionType::RetryStrategy,
        });
        t1.id = "t1".into();
        engine.templates.push(t1);

        let mut t2 = engine.create_template(&FailurePattern {
            tool_name: "y".into(),
            error_type: "auth_error".into(),
            context_snippet: "".into(),
            frequency: 5,
            first_seen: Instant::now(),
            last_seen: Instant::now(),
            suggested_intervention: InterventionType::PreconditionCheck,
        });
        t2.id = "t2".into();
        engine.templates.push(t2);

        // Same type as t2 — should not increase diversity
        let mut t3 = engine.create_template(&FailurePattern {
            tool_name: "z".into(),
            error_type: "auth_error".into(),
            context_snippet: "".into(),
            frequency: 5,
            first_seen: Instant::now(),
            last_seen: Instant::now(),
            suggested_intervention: InterventionType::PreconditionCheck,
        });
        t3.id = "t3".into();
        engine.templates.push(t3);

        assert_eq!(engine.pattern_diversity(), 2);
    }

    #[test]
    fn test_discovery_summary_output() {
        let mut engine = InterventionEngine::new();
        let summary = engine.discovery_summary();
        assert!(summary.contains("Patterns discovered: 0"));
        assert!(summary.contains("Templates created: 0"));

        engine.discover_pattern("find", "parse_error", "invalid regex");
        let summary = engine.discovery_summary();
        assert!(summary.contains("Patterns discovered: 1"));
    }

    #[test]
    fn test_create_template_timeout_maps_to_retry() {
        let mut engine = InterventionEngine::new();
        engine.discover_pattern("fetch", "timeout", "slow network");
        engine.discover_pattern("fetch", "timeout", "slow network");
        engine.discover_pattern("fetch", "timeout", "slow network");
        engine.discover_pattern("fetch", "timeout", "slow network");
        assert_eq!(engine.templates.len(), 1);
        assert_eq!(
            engine.templates[0].intervention_type,
            InterventionType::RetryStrategy
        );
    }

    #[test]
    fn test_create_template_auth_maps_to_precondition_check() {
        let mut engine = InterventionEngine::new();
        engine.discover_pattern("api", "auth_error", "invalid token");
        engine.discover_pattern("api", "auth_error", "invalid token");
        engine.discover_pattern("api", "auth_error", "invalid token");
        engine.discover_pattern("api", "auth_error", "invalid token");
        assert_eq!(engine.templates.len(), 1);
        assert_eq!(
            engine.templates[0].intervention_type,
            InterventionType::PreconditionCheck
        );
    }

    #[test]
    fn test_create_template_empty_maps_to_tool_reordering() {
        let mut engine = InterventionEngine::new();
        engine.discover_pattern("search", "empty_results", "no data");
        engine.discover_pattern("search", "empty_results", "no data");
        engine.discover_pattern("search", "empty_results", "no data");
        engine.discover_pattern("search", "empty_results", "no data");
        assert_eq!(engine.templates.len(), 1);
        assert_eq!(
            engine.templates[0].intervention_type,
            InterventionType::ToolReordering
        );
    }

    #[test]
    fn test_create_template_generic_maps_to_error_handler() {
        let mut engine = InterventionEngine::new();
        engine.discover_pattern("cmd", "connection_reset", "broken pipe");
        engine.discover_pattern("cmd", "connection_reset", "broken pipe");
        engine.discover_pattern("cmd", "connection_reset", "broken pipe");
        engine.discover_pattern("cmd", "connection_reset", "broken pipe");
        assert_eq!(engine.templates.len(), 1);
        assert_eq!(
            engine.templates[0].intervention_type,
            InterventionType::ErrorHandler
        );
    }

    #[test]
    fn test_global_engine_singleton() {
        let e1 = global_intervention_engine();
        let e2 = global_intervention_engine();
        assert!(std::ptr::eq(e1, e2));
    }

    #[test]
    fn test_record_failure_pattern_global() {
        record_failure_pattern("global_tool", "timeout", "global context");
        let engine = global_intervention_engine()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        assert!(engine
            .failure_patterns
            .iter()
            .any(|p| p.tool_name == "global_tool"));
    }

    #[test]
    fn test_suggest_tool_interventions_global() {
        record_failure_pattern("gtool", "parse_error", "global parse fail");
        record_failure_pattern("gtool", "parse_error", "global parse fail");
        record_failure_pattern("gtool", "parse_error", "global parse fail");
        record_failure_pattern("gtool", "parse_error", "global parse fail");
        let suggestions = suggest_tool_interventions("gtool", "parse_error");
        assert!(!suggestions.is_empty());
        assert!(suggestions[0].contains("effectiveness"));
    }

    #[test]
    fn test_pattern_discovery_disabled() {
        let mut engine = InterventionEngine::new();
        engine.pattern_discovery_enabled = false;
        engine.discover_pattern("secret", "timeout", "shh");
        assert_eq!(engine.failure_patterns.len(), 0);
    }

    #[test]
    fn test_auto_apply_interventions_respects_threshold() {
        let mut engine = InterventionEngine::new();

        let mut t1 = engine.create_template(&FailurePattern {
            tool_name: "db".into(),
            error_type: "timeout".into(),
            context_snippet: "".into(),
            frequency: 5,
            first_seen: Instant::now(),
            last_seen: Instant::now(),
            suggested_intervention: InterventionType::RetryStrategy,
        });
        t1.effectiveness = 0.95;
        t1.id = "high".into();
        t1.trigger_pattern = "db when timeout".into();
        engine.templates.push(t1);

        let mut t2 = engine.create_template(&FailurePattern {
            tool_name: "db".into(),
            error_type: "timeout".into(),
            context_snippet: "".into(),
            frequency: 5,
            first_seen: Instant::now(),
            last_seen: Instant::now(),
            suggested_intervention: InterventionType::ErrorHandler,
        });
        t2.effectiveness = 0.3;
        t2.id = "low".into();
        t2.trigger_pattern = "db when timeout".into();
        engine.templates.push(t2);

        let auto = engine.auto_apply_interventions("db", "timeout");
        assert_eq!(auto.len(), 1);
        assert_eq!(auto[0].id, "high");
    }

    #[test]
    fn test_deterministic_template_id() {
        let mut engine = InterventionEngine::new();
        engine.discover_pattern("tool_x", "timeout", "ctx");
        engine.discover_pattern("tool_x", "timeout", "ctx");
        engine.discover_pattern("tool_x", "timeout", "ctx");
        engine.discover_pattern("tool_x", "timeout", "ctx");
        let id1 = engine.templates[0].id.clone();

        // Create a fresh engine with same pattern → same deterministic ID
        let mut engine2 = InterventionEngine::new();
        engine2.discover_pattern("tool_x", "timeout", "ctx");
        engine2.discover_pattern("tool_x", "timeout", "ctx");
        engine2.discover_pattern("tool_x", "timeout", "ctx");
        engine2.discover_pattern("tool_x", "timeout", "ctx");
        let id2 = engine2.templates[0].id.clone();

        assert_eq!(id1, id2);
    }
}
