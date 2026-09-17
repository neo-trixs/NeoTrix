//! Cost-Aware Router — GWT + token cost weighting (Axiom A1)
//!
//! Routes tasks to the cheapest model that can handle them.
//! Uses salience scoring with cost penalty:
//!   `final_score = relevance_score * (1.0 - cost_weight * cost_per_token)`
//!
//! Models are ranked by: `capability_score / cost_per_token`
//! This ensures cheap models get simple tasks, expensive models get complex tasks.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Token pricing tier for a model provider.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum PricingTier {
    /// Free tier (local models, community endpoints)
    Free,
    /// Budget tier ($0.01-0.10 per 1K tokens)
    Budget,
    /// Standard tier ($0.10-1.00 per 1K tokens)
    Standard,
    /// Premium tier ($1.00+ per 1K tokens)
    Premium,
}

/// A model's cost and capability profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProfile {
    /// Unique model identifier (e.g., "gpt-4o", "claude-sonnet-4-20250514", "llama-3-8b")
    pub model_id: String,
    /// Provider name (e.g., "openai", "anthropic", "local")
    pub provider: String,
    /// Cost per input token in USD (e.g., 0.0000025 for $2.50/1M tokens)
    pub cost_per_input_token: f64,
    /// Cost per output token in USD
    pub cost_per_output_token: f64,
    /// Capability score (0.0-1.0): how capable this model is for general tasks
    pub capability_score: f64,
    /// Median latency in milliseconds (for latency-sensitive routing)
    pub latency_p50_ms: u64,
    /// Maximum context window in tokens
    pub context_window: usize,
    /// Pricing tier (derived from cost_per_input_token)
    pub tier: PricingTier,
}

impl ModelProfile {
    /// Create a new model profile with automatic tier classification.
    pub fn new(
        model_id: impl Into<String>,
        provider: impl Into<String>,
        cost_per_input_token: f64,
        cost_per_output_token: f64,
        capability_score: f64,
        latency_p50_ms: u64,
        context_window: usize,
    ) -> Self {
        let tier = Self::classify_tier(cost_per_input_token);
        Self {
            model_id: model_id.into(),
            provider: provider.into(),
            cost_per_input_token,
            cost_per_output_token,
            capability_score: capability_score.clamp(0.0, 1.0),
            latency_p50_ms,
            context_window,
            tier,
        }
    }

    /// Cost efficiency ratio: capability per dollar.
    /// Higher = more bang for the buck.
    pub fn cost_efficiency(&self) -> f64 {
        if self.cost_per_input_token <= 0.0 {
            return f64::INFINITY;
        }
        self.capability_score / self.cost_per_input_token
    }

    /// Weighted cost combining input and output token prices.
    /// `output_ratio` is the expected output-to-input ratio (default ~0.3).
    pub fn blended_cost(&self, output_ratio: f64) -> f64 {
        let output_weight = output_ratio.min(1.0);
        let input_weight = 1.0 - output_weight;
        self.cost_per_input_token * input_weight + self.cost_per_output_token * output_weight
    }

    /// Can this model fit the given context length?
    pub fn fits_context(&self, tokens: usize) -> bool {
        tokens <= self.context_window
    }

    fn classify_tier(cost_per_input_token: f64) -> PricingTier {
        // Cost per 1K tokens
        let cost_per_1k = cost_per_input_token * 1000.0;
        if cost_per_1k < 0.001 {
            PricingTier::Free
        } else if cost_per_1k < 0.10 {
            PricingTier::Budget
        } else if cost_per_1k < 1.00 {
            PricingTier::Standard
        } else {
            PricingTier::Premium
        }
    }
}

/// Task complexity classification for cost-aware routing.
///
/// Maps to the appropriate model tier:
/// - Trivial/Simple → use cheapest available (Budget or Free)
/// - Medium → Standard tier
/// - Complex/Critical → Premium tier if needed, Standard if budget-constrained
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskComplexity {
    /// I/O, formatting, trivial transforms — use cheapest model
    Trivial,
    /// Search, lookup, simple classification — cheap model
    Simple,
    /// Analysis, generation, moderate reasoning — mid-tier model
    Medium,
    /// Multi-step reasoning, planning, architecture — capable model
    Complex,
    /// Security audit, safety-critical decisions — best available model
    Critical,
}

impl TaskComplexity {
    /// Minimum capability score required for this complexity level.
    pub fn min_capability(&self) -> f64 {
        match self {
            TaskComplexity::Trivial => 0.1,
            TaskComplexity::Simple => 0.3,
            TaskComplexity::Medium => 0.5,
            TaskComplexity::Complex => 0.7,
            TaskComplexity::Critical => 0.9,
        }
    }

    /// Preferred pricing tier for this complexity level.
    pub fn preferred_tier(&self) -> PricingTier {
        match self {
            TaskComplexity::Trivial => PricingTier::Free,
            TaskComplexity::Simple => PricingTier::Budget,
            TaskComplexity::Medium => PricingTier::Standard,
            TaskComplexity::Complex => PricingTier::Standard,
            TaskComplexity::Critical => PricingTier::Premium,
        }
    }

    /// Order for comparison (higher = more complex).
    pub fn ordinal(&self) -> u8 {
        match self {
            TaskComplexity::Trivial => 0,
            TaskComplexity::Simple => 1,
            TaskComplexity::Medium => 2,
            TaskComplexity::Complex => 3,
            TaskComplexity::Critical => 4,
        }
    }
}

/// Cost-aware routing result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteResult {
    /// The selected model profile
    pub model: ModelProfile,
    /// Final score used for selection (higher = better)
    pub score: f64,
    /// Estimated cost for this request in USD
    pub estimated_cost_usd: f64,
    /// Estimated tokens (input)
    pub estimated_input_tokens: usize,
    /// Whether the model fits within the context budget
    pub within_context_budget: bool,
    /// Whether the request fits within the cost budget
    within_cost_budget: bool,
    /// All candidate models considered (for debugging/auditing)
    pub candidates_evaluated: usize,
}

impl RouteResult {
    /// Is this route valid (both context and cost budgets met)?
    pub fn is_valid(&self) -> bool {
        self.within_context_budget && self.within_cost_budget
    }
}

/// Default complexity thresholds for automatic task classification.
///
/// Maps task complexity to a minimum acceptable score threshold.
fn default_complexity_thresholds() -> HashMap<TaskComplexity, f64> {
    let mut m = HashMap::new();
    m.insert(TaskComplexity::Trivial, 0.2);
    m.insert(TaskComplexity::Simple, 0.4);
    m.insert(TaskComplexity::Medium, 0.6);
    m.insert(TaskComplexity::Complex, 0.8);
    m.insert(TaskComplexity::Critical, 0.95);
    m
}

/// Cost-Aware Router — routes tasks to the cheapest capable model.
///
/// Implements Axiom A1: "Not all tasks need the strongest model."
///
/// Usage:
/// ```ignore
/// let mut router = CostAwareRouter::new();
/// router.register_model(ModelProfile::new("gpt-4o", "openai", 0.0025/1000.0, 0.01/1000.0, 0.95, 500, 128000));
/// router.register_model(ModelProfile::new("gpt-4o-mini", "openai", 0.00015/1000.0, 0.0006/1000.0, 0.7, 200, 128000));
/// let result = router.route(TaskComplexity::Simple, 1000, None);
/// assert_eq!(result.model.model_id, "gpt-4o-mini");
/// ```
#[derive(Debug, Clone)]
pub struct CostAwareRouter {
    /// Registered model profiles
    pub models: Vec<ModelProfile>,
    /// Per-complexity minimum score thresholds
    pub complexity_thresholds: HashMap<TaskComplexity, f64>,
    /// Default cost weight for scoring (0.0 = ignore cost, 1.0 = maximize savings)
    pub cost_weight: f64,
    /// Default output-to-input token ratio for blended cost estimation
    pub output_ratio: f64,
}

impl Default for CostAwareRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl CostAwareRouter {
    /// Create a new router with default thresholds.
    pub fn new() -> Self {
        Self {
            models: Vec::new(),
            complexity_thresholds: default_complexity_thresholds(),
            cost_weight: 0.3,
            output_ratio: 0.3,
        }
    }

    /// Set the cost weight (0.0 to 1.0). Higher = more cost-sensitive.
    pub fn with_cost_weight(mut self, weight: f64) -> Self {
        self.cost_weight = weight.clamp(0.0, 1.0);
        self
    }

    /// Set the output-to-input token ratio for cost estimation.
    pub fn with_output_ratio(mut self, ratio: f64) -> Self {
        self.output_ratio = ratio.clamp(0.0, 1.0);
        self
    }

    /// Override the complexity threshold for a specific level.
    pub fn with_threshold(mut self, complexity: TaskComplexity, threshold: f64) -> Self {
        self.complexity_thresholds
            .insert(complexity, threshold.clamp(0.0, 1.0));
        self
    }

    /// Register a model profile.
    pub fn register_model(&mut self, model: ModelProfile) {
        self.models.push(model);
    }

    /// Register multiple model profiles at once.
    pub fn register_models(&mut self, models: Vec<ModelProfile>) {
        self.models.extend(models);
    }

    /// Get all registered models.
    pub fn models(&self) -> &[ModelProfile] {
        &self.models
    }

    /// Route a task to the cheapest capable model.
    ///
    /// # Arguments
    /// * `task_complexity` — classified complexity of the task
    /// * `estimated_input_tokens` — rough estimate of input token count
    /// * `budget_usd` — optional cost budget ceiling in USD (None = unlimited)
    ///
    /// # Returns
    /// A `RouteResult` with the selected model, score, and cost estimate.
    /// If no model qualifies, returns the cheapest model that fits the context
    /// budget (graceful degradation).
    pub fn route(
        &self,
        task_complexity: TaskComplexity,
        estimated_input_tokens: usize,
        budget_usd: Option<f64>,
    ) -> RouteResult {
        let min_cap = task_complexity.min_capability();
        let threshold = self
            .complexity_thresholds
            .get(&task_complexity)
            .copied()
            .unwrap_or(0.5);

        // Filter models that meet capability threshold and fit context
        let mut candidates: Vec<(&ModelProfile, f64)> = self
            .models
            .iter()
            .filter(|m| {
                m.capability_score >= min_cap
                    && m.fits_context(estimated_input_tokens)
                    && budget_usd
                        .map(|b| self.estimate_cost(m, estimated_input_tokens) <= b)
                        .unwrap_or(true)
            })
            .map(|m| {
                let score = self.score_model(m, task_complexity, threshold);
                (m, score)
            })
            .collect();

        // Sort by score descending (highest score wins)
        candidates.sort_by(|a, b| b.1.total_cmp(&a.1));

        if let Some((model, score)) = candidates.first() {
            let est_cost = self.estimate_cost(model, estimated_input_tokens);
            return RouteResult {
                model: (*model).clone(),
                score: *score,
                estimated_cost_usd: est_cost,
                estimated_input_tokens,
                within_context_budget: true,
                within_cost_budget: budget_usd.map(|b| est_cost <= b).unwrap_or(true),
                candidates_evaluated: candidates.len(),
            };
        }

        // Graceful degradation: pick the cheapest model that fits context
        let fallback = self
            .models
            .iter()
            .filter(|m| m.fits_context(estimated_input_tokens))
            .min_by(|a, b| {
                a.cost_per_input_token
                    .partial_cmp(&b.cost_per_input_token)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

        if let Some(model) = fallback {
            let est_cost = self.estimate_cost(model, estimated_input_tokens);
            RouteResult {
                model: model.clone(),
                score: 0.0, // degraded score
                estimated_cost_usd: est_cost,
                estimated_input_tokens,
                within_context_budget: true,
                within_cost_budget: budget_usd.map(|b| est_cost <= b).unwrap_or(true),
                candidates_evaluated: 0,
            }
        } else {
            // No model fits — return a synthetic "no model" result
            RouteResult {
                model: ModelProfile {
                    model_id: "__no_model__".into(),
                    provider: "none".into(),
                    cost_per_input_token: 0.0,
                    cost_per_output_token: 0.0,
                    capability_score: 0.0,
                    latency_p50_ms: 0,
                    context_window: 0,
                    tier: PricingTier::Free,
                },
                score: -1.0,
                estimated_cost_usd: 0.0,
                estimated_input_tokens,
                within_context_budget: false,
                within_cost_budget: false,
                candidates_evaluated: 0,
            }
        }
    }

    /// Score a model for a given task complexity.
    ///
    /// Formula: `score = capability * (1.0 - cost_weight * normalized_cost) * tier_bonus`
    ///
    /// - `capability` rewards high-capability models
    /// - `cost_weight * normalized_cost` penalizes expensive models
    /// - `tier_bonus` gives a small boost to models matching the preferred tier
    pub fn score_model(
        &self,
        model: &ModelProfile,
        task_complexity: TaskComplexity,
        threshold: f64,
    ) -> f64 {
        let capability = model.capability_score;

        // Normalize cost to 0-1 range (sigmoid-like mapping)
        // $0/token → 0.0, $0.01/token → ~1.0
        let normalized_cost = (model.cost_per_input_token * 10000.0).min(1.0);

        // Core score: capability minus cost penalty
        let raw_score = capability * (1.0 - self.cost_weight * normalized_cost);

        // Tier bonus: small reward for matching preferred tier
        let tier_bonus = if model.tier == task_complexity.preferred_tier() {
            0.05
        } else {
            0.0
        };

        // Threshold gate: penalize models below the complexity threshold
        let threshold_factor = if capability >= threshold {
            1.0
        } else {
            // Linear penalty for under-threshold models
            capability / threshold
        };

        (raw_score + tier_bonus) * threshold_factor
    }

    /// Estimate the cost of processing `input_tokens` with a given model.
    /// Uses the blended cost (input + estimated output).
    pub fn estimate_cost(&self, model: &ModelProfile, input_tokens: usize) -> f64 {
        let input_cost = model.cost_per_input_token * input_tokens as f64;
        let output_tokens = (input_tokens as f64 * self.output_ratio) as usize;
        let output_cost = model.cost_per_output_token * output_tokens as f64;
        input_cost + output_cost
    }

    /// Rough token estimation for text.
    /// Uses the ~4 chars per token heuristic (works for English; adjust for other languages).
    pub fn estimate_tokens(text: &str) -> usize {
        // English: ~4 chars per token. Also count newlines and spaces.
        let char_count = text.len();
        let word_count = text.split_whitespace().count();
        // Heuristic: max(char_count/4, word_count * 1.3)
        let by_chars = char_count / 4;
        let by_words = (word_count as f64 * 1.3) as usize;
        by_chars.max(by_words).max(1)
    }

    /// Check if a model can handle `estimated_tokens` within a USD budget.
    pub fn budget_check(
        &self,
        model: &ModelProfile,
        estimated_tokens: usize,
        budget_usd: f64,
    ) -> bool {
        if !model.fits_context(estimated_tokens) {
            return false;
        }
        self.estimate_cost(model, estimated_tokens) <= budget_usd
    }

    /// Find the cheapest model that can handle the given complexity and token count.
    /// Returns None if no model qualifies.
    pub fn cheapest_capable(
        &self,
        task_complexity: TaskComplexity,
        estimated_tokens: usize,
    ) -> Option<&ModelProfile> {
        let min_cap = task_complexity.min_capability();
        self.models
            .iter()
            .filter(|m| m.capability_score >= min_cap && m.fits_context(estimated_tokens))
            .min_by(|a, b| {
                a.cost_per_input_token
                    .partial_cmp(&b.cost_per_input_token)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Find the most capable model within a budget.
    pub fn best_within_budget(
        &self,
        estimated_tokens: usize,
        budget_usd: f64,
    ) -> Option<&ModelProfile> {
        self.models
            .iter()
            .filter(|m| {
                m.fits_context(estimated_tokens)
                    && self.estimate_cost(m, estimated_tokens) <= budget_usd
            })
            .max_by(|a, b| {
                a.capability_score
                    .partial_cmp(&b.capability_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// List models by cost efficiency (capability per dollar), descending.
    pub fn ranked_by_efficiency(&self) -> Vec<(&ModelProfile, f64)> {
        let mut ranked: Vec<(&ModelProfile, f64)> = self
            .models
            .iter()
            .map(|m| (m, m.cost_efficiency()))
            .collect();
        ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
        ranked
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gpt4o() -> ModelProfile {
        ModelProfile::new("gpt-4o", "openai", 2.5e-6, 10.0e-6, 0.95, 500, 128_000)
    }

    fn gpt4o_mini() -> ModelProfile {
        ModelProfile::new("gpt-4o-mini", "openai", 0.15e-6, 0.6e-6, 0.70, 200, 128_000)
    }

    fn claude_sonnet() -> ModelProfile {
        ModelProfile::new(
            "claude-sonnet-4-20250514",
            "anthropic",
            3.0e-6,
            15.0e-6,
            0.92,
            600,
            200_000,
        )
    }

    fn llama_local() -> ModelProfile {
        ModelProfile::new("llama-3-8b", "local", 0.0, 0.0, 0.50, 50, 8_000)
    }

    fn make_router() -> CostAwareRouter {
        let mut r = CostAwareRouter::new();
        r.register_models(vec![gpt4o(), gpt4o_mini(), claude_sonnet(), llama_local()]);
        r
    }

    #[test]
    fn test_trivial_task_uses_cheapest() {
        let router = make_router();
        let result = router.route(TaskComplexity::Trivial, 100, None);
        // Trivial tasks should prefer the cheapest capable model (llama or gpt-4o-mini)
        assert!(result.model.capability_score >= TaskComplexity::Trivial.min_capability());
        assert!(result.score > 0.0);
    }

    #[test]
    fn test_critical_task_uses_capable_model() {
        let router = make_router();
        let result = router.route(TaskComplexity::Critical, 1000, None);
        // Critical tasks need high capability
        assert!(result.model.capability_score >= TaskComplexity::Critical.min_capability());
    }

    #[test]
    fn test_budget_constraint_respected() {
        let router = make_router();
        // Very tight budget — should pick cheapest model
        let result = router.route(TaskComplexity::Medium, 1000, Some(0.0001));
        assert!(result.estimated_cost_usd <= 0.0001 + 1e-10);
    }

    #[test]
    fn test_context_window_respected() {
        let router = make_router();
        // Llama has 8K context — request 10K should skip it
        let result = router.route(TaskComplexity::Simple, 10_000, None);
        assert_ne!(result.model.model_id, "llama-3-8b");
    }

    #[test]
    fn test_score_model_higher_capability_higher_score() {
        let router = CostAwareRouter::new().with_cost_weight(0.0); // no cost penalty
        let cheap = ModelProfile::new("cheap", "test", 1.0e-6, 1.0e-6, 0.3, 100, 8_000);
        let expensive = ModelProfile::new("expensive", "test", 10.0e-6, 10.0e-6, 0.9, 500, 128_000);

        let s_cheap = router.score_model(&cheap, TaskComplexity::Medium, 0.5);
        let s_expensive = router.score_model(&expensive, TaskComplexity::Medium, 0.5);
        assert!(
            s_expensive > s_cheap,
            "higher capability should score higher when cost weight is 0"
        );
    }

    #[test]
    fn test_score_model_cost_penalty() {
        let router = CostAwareRouter::new().with_cost_weight(1.0); // max cost penalty
        let cheap = ModelProfile::new("cheap", "test", 1.0e-6, 1.0e-6, 0.7, 100, 8_000);
        let expensive =
            ModelProfile::new("expensive", "test", 100.0e-6, 100.0e-6, 0.9, 500, 128_000);

        let s_cheap = router.score_model(&cheap, TaskComplexity::Medium, 0.0);
        let s_expensive = router.score_model(&expensive, TaskComplexity::Medium, 0.0);
        // With max cost weight, the cheaper model should win despite lower capability
        assert!(
            s_cheap > s_expensive,
            "cheaper model should win with max cost weight: cheap={s_cheap}, exp={s_expensive}"
        );
    }

    #[test]
    fn test_estimate_tokens() {
        let tokens = CostAwareRouter::estimate_tokens("hello world");
        assert!(tokens >= 1 && tokens <= 10);
    }

    #[test]
    fn test_budget_check() {
        let model = gpt4o_mini();
        let router = CostAwareRouter::new();
        assert!(router.budget_check(&model, 1000, 0.01));
        assert!(!router.budget_check(&model, 1000, 0.0000001));
    }

    #[test]
    fn test_cheapest_capable() {
        let router = make_router();
        let cheapest = router.cheapest_capable(TaskComplexity::Simple, 1000);
        assert!(cheapest.is_some());
        // Should be the local model (free) or gpt-4o-mini (cheapest paid)
        let m = cheapest.unwrap();
        assert!(m.capability_score >= TaskComplexity::Simple.min_capability());
    }

    #[test]
    fn test_best_within_budget() {
        let router = make_router();
        let best = router.best_within_budget(1000, 0.001);
        assert!(best.is_some());
        let m = best.unwrap();
        assert!(m.capability_score >= 0.5); // should get at least a mid-tier model
    }

    #[test]
    fn test_ranked_by_efficiency() {
        let router = make_router();
        let ranked = router.ranked_by_efficiency();
        assert_eq!(ranked.len(), 4);
        // Local model (free) should have infinite efficiency
        assert_eq!(ranked[0].0.model_id, "llama-3-8b");
    }

    #[test]
    fn test_tier_classification() {
        let free = ModelProfile::new("free", "test", 0.0, 0.0, 0.5, 100, 8_000);
        assert_eq!(free.tier, PricingTier::Free);

        let budget = ModelProfile::new("budget", "test", 0.05e-6, 0.1e-6, 0.6, 200, 32_000);
        assert_eq!(budget.tier, PricingTier::Budget);

        let standard = ModelProfile::new("standard", "test", 2.5e-6, 10.0e-6, 0.9, 500, 128_000);
        assert_eq!(standard.tier, PricingTier::Standard);

        let premium = ModelProfile::new("premium", "test", 15.0e-6, 75.0e-6, 0.99, 1000, 200_000);
        assert_eq!(premium.tier, PricingTier::Premium);
    }

    #[test]
    fn test_model_profile_cost_efficiency() {
        let cheap = ModelProfile::new("cheap", "test", 1.0e-6, 1.0e-6, 0.5, 100, 8_000);
        let expensive = ModelProfile::new("exp", "test", 10.0e-6, 10.0e-6, 0.9, 500, 128_000);
        assert!(cheap.cost_efficiency() > expensive.cost_efficiency());
    }

    #[test]
    fn test_fallback_when_no_model_qualifies() {
        let mut router = CostAwareRouter::new();
        // Only register a low-capability model
        router.register_model(ModelProfile::new(
            "weak", "test", 1.0e-6, 1.0e-6, 0.2, 100, 8_000,
        ));
        // Critical task needs 0.9 capability — no model qualifies
        let result = router.route(TaskComplexity::Critical, 1000, None);
        // Should fall back to the weak model (graceful degradation)
        assert_eq!(result.model.model_id, "weak");
        assert_eq!(result.score, 0.0);
    }

    #[test]
    fn test_no_model_at_all() {
        let router = CostAwareRouter::new();
        let result = router.route(TaskComplexity::Simple, 1000, None);
        assert_eq!(result.model.model_id, "__no_model__");
        assert_eq!(result.score, -1.0);
        assert!(!result.is_valid());
    }

    #[test]
    fn test_route_result_is_valid() {
        let router = make_router();
        let result = router.route(TaskComplexity::Medium, 1000, Some(1.0));
        assert!(result.is_valid());
    }

    #[test]
    fn test_complexity_min_capability() {
        assert!(TaskComplexity::Trivial.min_capability() < TaskComplexity::Simple.min_capability());
        assert!(TaskComplexity::Simple.min_capability() < TaskComplexity::Medium.min_capability());
        assert!(TaskComplexity::Medium.min_capability() < TaskComplexity::Complex.min_capability());
        assert!(
            TaskComplexity::Complex.min_capability() < TaskComplexity::Critical.min_capability()
        );
    }

    #[test]
    fn test_complexity_preferred_tier() {
        assert_eq!(TaskComplexity::Trivial.preferred_tier(), PricingTier::Free);
        assert_eq!(TaskComplexity::Simple.preferred_tier(), PricingTier::Budget);
        assert_eq!(
            TaskComplexity::Medium.preferred_tier(),
            PricingTier::Standard
        );
        assert_eq!(
            TaskComplexity::Critical.preferred_tier(),
            PricingTier::Premium
        );
    }

    #[test]
    fn test_blend_cost() {
        let model = ModelProfile::new("test", "test", 0.001, 0.005, 0.8, 100, 8_000);
        // 70% input, 30% output
        let blended = model.blended_cost(0.3);
        let expected = 0.001 * 0.7 + 0.005 * 0.3;
        assert!((blended - expected).abs() < 1e-10);
    }

    #[test]
    fn test_register_models_batch() {
        let mut router = CostAwareRouter::new();
        router.register_models(vec![gpt4o(), gpt4o_mini(), claude_sonnet()]);
        assert_eq!(router.models().len(), 3);
    }
}
