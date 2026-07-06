use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub enum ErrorType {
    RateLimit { retry_after: Option<u64> },
    ServerError { code: u16 },
    Timeout { elapsed_ms: u64 },
    InvalidOutput { details: String, raw: String },
    Hallucination { details: String },
    InfiniteLoop { tool_name: String, count: usize },
    ContextOverflow { tokens: usize, limit: usize },
    BudgetExceeded { spent: f64, limit: f64 },
    Unknown(String),
}

#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub error_type: ErrorType,
    pub attempt: usize,
    pub max_retries: usize,
    pub model: String,
    pub available_models: Vec<String>,
    pub prompt: String,
    pub prompt_variants: Vec<String>,
    pub state_snapshot: Option<Vec<u8>>,
    pub token_budget_remaining: u64,
    pub elapsed_ms: u64,
    pub metadata: HashMap<String, String>,
}

impl ErrorContext {
    pub fn new(error_type: ErrorType, model: &str, prompt: &str) -> Self {
        Self {
            error_type,
            attempt: 0,
            max_retries: 3,
            model: model.to_string(),
            available_models: Vec::new(),
            prompt: prompt.to_string(),
            prompt_variants: Vec::new(),
            state_snapshot: None,
            token_budget_remaining: 0,
            elapsed_ms: 0,
            metadata: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum RecoveryAction {
    Retry { delay_ms: u64, reason: String },
    FallbackToModel(String),
    FallbackToPrompt(String),
    RestoreFromCheckpoint(Vec<u8>),
    EscalateToHuman(String),
    Abort(String),
}

#[derive(Debug, Clone)]
pub struct AgentError {
    pub kind: ErrorType,
    pub message: String,
    pub source: String,
}

pub trait ErrorRecoveryStrategy: Send + Sync {
    fn name(&self) -> &str;
    fn layer(&self) -> usize;
    fn can_handle(&self, ctx: &ErrorContext) -> bool;
    fn recover(&self, ctx: &ErrorContext) -> Option<RecoveryAction>;
}

pub struct ExponentialBackoffStrategy {
    base_delay_ms: u64,
    max_delay_ms: u64,
    jitter_factor: f64,
}

impl ExponentialBackoffStrategy {
    pub fn new(base_delay_ms: u64, max_delay_ms: u64, jitter_factor: f64) -> Self {
        Self { base_delay_ms, max_delay_ms, jitter_factor }
    }
}

impl ErrorRecoveryStrategy for ExponentialBackoffStrategy {
    fn name(&self) -> &str { "exponential_backoff" }
    fn layer(&self) -> usize { 1 }

    fn can_handle(&self, ctx: &ErrorContext) -> bool {
        matches!(ctx.error_type, ErrorType::RateLimit { .. } | ErrorType::ServerError { .. } | ErrorType::Timeout { .. })
    }

    fn recover(&self, ctx: &ErrorContext) -> Option<RecoveryAction> {
        if ctx.attempt >= ctx.max_retries {
            return None;
        }
        if let ErrorType::RateLimit { retry_after: Some(ra) } = &ctx.error_type {
            return Some(RecoveryAction::Retry {
                delay_ms: *ra,
                reason: format!("rate limit: retry after {}ms", ra),
            });
        }
        let delay = compute_backoff(self.base_delay_ms, ctx.attempt, self.max_delay_ms, self.jitter_factor);
        Some(RecoveryAction::Retry {
            delay_ms: delay,
            reason: format!("backoff retry attempt {}", ctx.attempt + 1),
        })
    }
}

fn compute_backoff(base: u64, attempt: usize, max_delay: u64, jitter: f64) -> u64 {
    let exp_delay = base.saturating_mul(1 << attempt.min(30));
    let jitter_val = if jitter > 0.0 {
        let j = (jitter * base as f64) as u64;
        j / 2
    } else {
        0
    };
    exp_delay.saturating_add(jitter_val).min(max_delay)
}

pub struct CircuitBreakerStrategy {
    threshold: usize,
    cooldown_ms: u64,
    failure_counts: HashMap<String, usize>,
    last_failure: HashMap<String, Instant>,
}

impl CircuitBreakerStrategy {
    pub fn new(threshold: usize, cooldown_ms: u64) -> Self {
        Self { threshold, cooldown_ms, failure_counts: HashMap::new(), last_failure: HashMap::new() }
    }
}

impl ErrorRecoveryStrategy for CircuitBreakerStrategy {
    fn name(&self) -> &str { "circuit_breaker" }
    fn layer(&self) -> usize { 2 }

    fn can_handle(&self, ctx: &ErrorContext) -> bool {
        let key = &ctx.model;
        let count = self.failure_counts.get(key).copied().unwrap_or(0);
        let cooldown_ok = self.last_failure.get(key)
            .map(|t| t.elapsed() >= Duration::from_millis(self.cooldown_ms))
            .unwrap_or(true);
        count >= self.threshold && cooldown_ok
    }

    fn recover(&self, ctx: &ErrorContext) -> Option<RecoveryAction> {
        if ctx.available_models.is_empty() {
            return Some(RecoveryAction::Retry {
                delay_ms: self.cooldown_ms,
                reason: "circuit breaker cooldown".to_string(),
            });
        }
        let next = ctx.available_models.iter()
            .find(|m| *m != &ctx.model)?;
        Some(RecoveryAction::FallbackToModel(next.clone()))
    }
}

pub struct ModelFallbackStrategy;

impl ErrorRecoveryStrategy for ModelFallbackStrategy {
    fn name(&self) -> &str { "model_fallback" }
    fn layer(&self) -> usize { 3 }

    fn can_handle(&self, ctx: &ErrorContext) -> bool {
        matches!(ctx.error_type, ErrorType::ServerError { .. } | ErrorType::Timeout { .. } | ErrorType::RateLimit { .. })
            && ctx.available_models.iter().any(|m| *m != ctx.model)
    }

    fn recover(&self, ctx: &ErrorContext) -> Option<RecoveryAction> {
        // Use a simple priority: prefer different provider
        let current_family = model_family(&ctx.model);
        let best = ctx.available_models.iter()
            .find(|m| model_family(m) != current_family)
            .or_else(|| ctx.available_models.iter().find(|m| *m != &ctx.model))?;
        Some(RecoveryAction::FallbackToModel(best.clone()))
    }
}

fn model_family(model: &str) -> &str {
    if model.contains("claude") || model.contains("sonnet") || model.contains("opus") || model.contains("haiku") { "anthropic" }
    else if model.contains("gpt") || model.contains("o1") || model.contains("o3") { "openai" }
    else if model.contains("gemini") { "gemini" }
    else if model.contains("deepseek") { "deepseek" }
    else if model.contains("llama") || model.contains("mixtral") { "open_source" }
    else { "other" }
}

pub struct SemanticFallbackStrategy;

impl ErrorRecoveryStrategy for SemanticFallbackStrategy {
    fn name(&self) -> &str { "semantic_fallback" }
    fn layer(&self) -> usize { 4 }

    fn can_handle(&self, ctx: &ErrorContext) -> bool {
        matches!(ctx.error_type, ErrorType::InvalidOutput { .. } | ErrorType::Hallucination { .. })
            && !ctx.prompt_variants.is_empty()
    }

    fn recover(&self, ctx: &ErrorContext) -> Option<RecoveryAction> {
        let variant_idx = ctx.attempt.min(ctx.prompt_variants.len().saturating_sub(1));
        let variant = ctx.prompt_variants.get(variant_idx)?;
        Some(RecoveryAction::FallbackToPrompt(variant.clone()))
    }
}

pub struct ValidationGateStrategy;

impl ErrorRecoveryStrategy for ValidationGateStrategy {
    fn name(&self) -> &str { "validation_gate" }
    fn layer(&self) -> usize { 5 }

    fn can_handle(&self, ctx: &ErrorContext) -> bool {
        matches!(ctx.error_type, ErrorType::InvalidOutput { .. })
    }

    fn recover(&self, ctx: &ErrorContext) -> Option<RecoveryAction> {
        let msg = if let ErrorType::InvalidOutput { details, raw: _ } = &ctx.error_type {
            format!("validation failed: {}. retry with stricter constraints", details)
        } else {
            "output validation failed".to_string()
        };
        Some(RecoveryAction::Retry { delay_ms: 0, reason: msg })
    }
}

pub struct CheckpointResumeStrategy;

impl ErrorRecoveryStrategy for CheckpointResumeStrategy {
    fn name(&self) -> &str { "checkpoint_resume" }
    fn layer(&self) -> usize { 6 }

    fn can_handle(&self, ctx: &ErrorContext) -> bool {
        matches!(ctx.error_type, ErrorType::Timeout { .. } | ErrorType::ContextOverflow { .. })
            && ctx.state_snapshot.is_some()
    }

    fn recover(&self, ctx: &ErrorContext) -> Option<RecoveryAction> {
        let snapshot = ctx.state_snapshot.clone()?;
        Some(RecoveryAction::RestoreFromCheckpoint(snapshot))
    }
}

pub struct HumanEscalationStrategy {
    threshold: usize,
}

impl HumanEscalationStrategy {
    pub fn new(threshold: usize) -> Self { Self { threshold } }
}

impl ErrorRecoveryStrategy for HumanEscalationStrategy {
    fn name(&self) -> &str { "human_escalation" }
    fn layer(&self) -> usize { 7 }

    fn can_handle(&self, ctx: &ErrorContext) -> bool {
        ctx.attempt >= self.threshold
    }

    fn recover(&self, ctx: &ErrorContext) -> Option<RecoveryAction> {
        Some(RecoveryAction::EscalateToHuman(format!(
            "failed after {} attempts on model '{}': {:?}",
            ctx.attempt, ctx.model, ctx.error_type
        )))
    }
}

#[derive(Debug, Clone)]
pub struct RecoveryConfig {
    pub max_retries: usize,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub jitter_factor: f64,
    pub enable_semantic_fallback: bool,
    pub enable_checkpoint: bool,
    pub human_escalation_threshold: usize,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 1000,
            max_delay_ms: 60_000,
            jitter_factor: 0.1,
            enable_semantic_fallback: true,
            enable_checkpoint: true,
            human_escalation_threshold: 5,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RecoveryStats {
    pub total_errors: u64,
    pub retries: u64,
    pub model_fallbacks: u64,
    pub semantic_fallbacks: u64,
    pub checkpoint_restores: u64,
    pub human_escalations: u64,
    pub aborts: u64,
    pub total_recovery_time_ms: u64,
    pub successful_recoveries: u64,
}

impl RecoveryStats {
    fn new() -> Self {
        Self {
            total_errors: 0,
            retries: 0,
            model_fallbacks: 0,
            semantic_fallbacks: 0,
            checkpoint_restores: 0,
            human_escalations: 0,
            aborts: 0,
            total_recovery_time_ms: 0,
            successful_recoveries: 0,
        }
    }

    pub fn recovery_rate(&self) -> f64 {
        if self.total_errors == 0 { return 1.0; }
        self.successful_recoveries as f64 / self.total_errors as f64
    }

    pub fn avg_recovery_time_ms(&self) -> f64 {
        if self.successful_recoveries == 0 { return 0.0; }
        self.total_recovery_time_ms as f64 / self.successful_recoveries as f64
    }
}

pub struct RecoveryOrchestrator {
    strategies: Vec<Box<dyn ErrorRecoveryStrategy>>,
    config: RecoveryConfig,
    stats: RecoveryStats,
}

impl RecoveryOrchestrator {
    pub fn new(config: RecoveryConfig) -> Self {
        let strategies: Vec<Box<dyn ErrorRecoveryStrategy>> = vec![
            Box::new(ExponentialBackoffStrategy::new(config.base_delay_ms, config.max_delay_ms, config.jitter_factor)),
            Box::new(CircuitBreakerStrategy::new(5, 60_000)),
            Box::new(ModelFallbackStrategy),
            Box::new(SemanticFallbackStrategy),
            Box::new(ValidationGateStrategy),
            Box::new(CheckpointResumeStrategy),
            Box::new(HumanEscalationStrategy::new(config.human_escalation_threshold)),
        ];
        Self { strategies, config, stats: RecoveryStats::new() }
    }

    pub fn add_strategy(&mut self, strategy: Box<dyn ErrorRecoveryStrategy>) {
        self.strategies.push(strategy);
        self.strategies.sort_by_key(|s| s.layer());
    }

    pub fn handle(&mut self, ctx: &ErrorContext) -> RecoveryAction {
        self.stats.total_errors += 1;
        let start = Instant::now();

        for strategy in &self.strategies {
            if strategy.can_handle(ctx) {
                if let Some(action) = strategy.recover(ctx) {
                    self.record_action(&action);
                    self.stats.total_recovery_time_ms += start.elapsed().as_millis() as u64;
                    self.stats.successful_recoveries += 1;
                    return action;
                }
            }
        }
        self.stats.aborts += 1;
        RecoveryAction::Abort(format!("no recovery strategy for error after {} attempts", ctx.attempt))
    }

    fn record_action(&mut self, action: &RecoveryAction) {
        match action {
            RecoveryAction::Retry { .. } => self.stats.retries += 1,
            RecoveryAction::FallbackToModel(_) => self.stats.model_fallbacks += 1,
            RecoveryAction::FallbackToPrompt(_) => self.stats.semantic_fallbacks += 1,
            RecoveryAction::RestoreFromCheckpoint(_) => self.stats.checkpoint_restores += 1,
            RecoveryAction::EscalateToHuman(_) => self.stats.human_escalations += 1,
            RecoveryAction::Abort(_) => self.stats.aborts += 1,
        }
    }

    pub fn stats(&self) -> &RecoveryStats { &self.stats }
    pub fn config(&self) -> &RecoveryConfig { &self.config }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ctx(error_type: ErrorType, model: &str, attempt: usize) -> ErrorContext {
        ErrorContext {
            error_type,
            attempt,
            max_retries: 3,
            model: model.to_string(),
            available_models: vec!["gpt-4o".to_string(), "gemini-2.5-pro".to_string()],
            prompt: "hello".to_string(),
            prompt_variants: vec!["variant A".to_string(), "variant B".to_string()],
            state_snapshot: Some(vec![1, 2, 3]),
            token_budget_remaining: 1000,
            elapsed_ms: 0,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_backoff_layer1_rate_limit() {
        let s = ExponentialBackoffStrategy::new(1000, 60000, 0.1);
        let ctx = make_ctx(ErrorType::RateLimit { retry_after: None }, "claude-sonnet-4", 0);
        assert!(s.can_handle(&ctx));
        let action = s.recover(&ctx);
        assert!(matches!(action, Some(RecoveryAction::Retry { .. })));
    }

    #[test]
    fn test_backoff_exhausted_retries() {
        let s = ExponentialBackoffStrategy::new(1000, 60000, 0.1);
        let ctx = make_ctx(ErrorType::RateLimit { retry_after: None }, "claude-sonnet-4", 5);
        assert!(s.recover(&ctx).is_none());
    }

    #[test]
    fn test_backoff_respects_retry_after() {
        let s = ExponentialBackoffStrategy::new(1000, 60000, 0.1);
        let ctx = make_ctx(ErrorType::RateLimit { retry_after: Some(5000) }, "claude-sonnet-4", 0);
        if let Some(RecoveryAction::Retry { delay_ms, .. }) = s.recover(&ctx) {
            assert_eq!(delay_ms, 5000);
        } else { panic!("expected retry with retry_after"); }
    }

    #[test]
    fn test_model_fallback_layer3_different_family() {
        let s = ModelFallbackStrategy;
        let ctx = make_ctx(ErrorType::ServerError { code: 503 }, "claude-sonnet-4", 0);
        assert!(s.can_handle(&ctx));
        let action = s.recover(&ctx);
        assert!(matches!(action, Some(RecoveryAction::FallbackToModel(_))));
    }

    #[test]
    fn test_model_fallback_no_alternatives() {
        let s = ModelFallbackStrategy;
        let mut ctx = make_ctx(ErrorType::ServerError { code: 503 }, "claude-sonnet-4", 0);
        ctx.available_models = vec!["claude-sonnet-4".to_string()];
        assert!(!s.can_handle(&ctx));
    }

    #[test]
    fn test_compute_backoff_values() {
        let d = compute_backoff(1000, 0, 60000, 0.0);
        assert!(d >= 1000 && d <= 2000);
        let d2 = compute_backoff(1000, 1, 60000, 0.0);
        assert!(d2 >= 2000 && d2 <= 4000);
    }

    #[test]
    fn test_compute_backoff_caps_at_max() {
        let d = compute_backoff(1000, 100, 5000, 0.0);
        assert_eq!(d, 5000);
    }

    #[test]
    fn test_semantic_fallback_layer4() {
        let s = SemanticFallbackStrategy;
        let ctx = make_ctx(ErrorType::InvalidOutput { details: "bad json".into(), raw: "{{{".into() }, "gpt-4o", 0);
        assert!(s.can_handle(&ctx));
        let action = s.recover(&ctx);
        assert!(matches!(action, Some(RecoveryAction::FallbackToPrompt(_))));
    }

    #[test]
    fn test_semantic_fallback_no_variants() {
        let s = SemanticFallbackStrategy;
        let mut ctx = make_ctx(ErrorType::InvalidOutput { details: "bad json".into(), raw: "{{{".into() }, "gpt-4o", 0);
        ctx.prompt_variants.clear();
        assert!(!s.can_handle(&ctx));
    }

    #[test]
    fn test_validation_gate_layer5() {
        let s = ValidationGateStrategy;
        let ctx = make_ctx(ErrorType::InvalidOutput { details: "schema violation".into(), raw: "{}".into() }, "gpt-4o", 0);
        assert!(s.can_handle(&ctx));
        assert!(matches!(s.recover(&ctx), Some(RecoveryAction::Retry { .. })));
    }

    #[test]
    fn test_validation_gate_wrong_type() {
        let s = ValidationGateStrategy;
        let ctx = make_ctx(ErrorType::ServerError { code: 500 }, "gpt-4o", 0);
        assert!(!s.can_handle(&ctx));
    }

    #[test]
    fn test_checkpoint_resume_layer6() {
        let s = CheckpointResumeStrategy;
        let ctx = make_ctx(ErrorType::Timeout { elapsed_ms: 30000 }, "claude-sonnet-4", 0);
        assert!(s.can_handle(&ctx));
        assert!(matches!(s.recover(&ctx), Some(RecoveryAction::RestoreFromCheckpoint(_))));
    }

    #[test]
    fn test_checkpoint_no_snapshot() {
        let s = CheckpointResumeStrategy;
        let mut ctx = make_ctx(ErrorType::Timeout { elapsed_ms: 30000 }, "claude-sonnet-4", 0);
        ctx.state_snapshot = None;
        assert!(!s.can_handle(&ctx));
    }

    #[test]
    fn test_human_escalation_layer7() {
        let s = HumanEscalationStrategy::new(3);
        let ctx = make_ctx(ErrorType::Unknown("critical".into()), "gpt-4o", 3);
        assert!(s.can_handle(&ctx));
        assert!(matches!(s.recover(&ctx), Some(RecoveryAction::EscalateToHuman(_))));
    }

    #[test]
    fn test_human_escalation_below_threshold() {
        let s = HumanEscalationStrategy::new(5);
        let ctx = make_ctx(ErrorType::Unknown("minor".into()), "gpt-4o", 2);
        assert!(!s.can_handle(&ctx));
    }

    #[test]
    fn test_orchestrator_handles_rate_limit() {
        let mut orch = RecoveryOrchestrator::new(RecoveryConfig::default());
        let ctx = make_ctx(ErrorType::RateLimit { retry_after: Some(2000) }, "claude-sonnet-4", 0);
        let action = orch.handle(&ctx);
        assert!(matches!(action, RecoveryAction::Retry { .. }));
        assert_eq!(orch.stats().retries, 1);
        assert_eq!(orch.stats().total_errors, 1);
    }

    #[test]
    fn test_orchestrator_escalates_after_many_failures() {
        let mut orch = RecoveryOrchestrator::new(RecoveryConfig {
            human_escalation_threshold: 2, ..Default::default()
        });
        let ctx = make_ctx(ErrorType::Unknown("persistent".into()), "gpt-4o", 5);
        let action = orch.handle(&ctx);
        assert!(matches!(action, RecoveryAction::EscalateToHuman(_)));
    }

    #[test]
    fn test_orchestrator_no_strategies_match_aborts() {
        let mut orch = RecoveryOrchestrator::new(RecoveryConfig::default());
        let mut ctx = make_ctx(ErrorType::Unknown("weird".into()), "gpt-4o", 0);
        ctx.available_models.clear();
        ctx.prompt_variants.clear();
        ctx.state_snapshot = None;
        let action = orch.handle(&ctx);
        assert!(matches!(action, RecoveryAction::Abort(_)));
    }

    #[test]
    fn test_orchestrator_stats_tracking() {
        let mut orch = RecoveryOrchestrator::new(RecoveryConfig::default());
        let ctx1 = make_ctx(ErrorType::RateLimit { retry_after: None }, "claude-sonnet-4", 0);
        orch.handle(&ctx1);
        let mut ctx2 = make_ctx(ErrorType::InvalidOutput { details: "bad".into(), raw: "x".into() }, "gpt-4o", 0);
        ctx2.prompt_variants.clear();
        let ctx3 = make_ctx(ErrorType::ServerError { code: 503 }, "gpt-4o", 1);
        orch.handle(&ctx2);
        orch.handle(&ctx3);
        assert_eq!(orch.stats().total_errors, 3);
        assert!(orch.stats().retries >= 1);
    }

    #[test]
    fn test_model_family_classification() {
        assert_eq!(model_family("claude-sonnet-4"), "anthropic");
        assert_eq!(model_family("gpt-4o"), "openai");
        assert_eq!(model_family("gemini-2.5-pro"), "gemini");
        assert_eq!(model_family("deepseek-v3"), "deepseek");
        assert_eq!(model_family("llama-3-70b"), "open_source");
        assert_eq!(model_family("unknown-model"), "other");
    }
}
