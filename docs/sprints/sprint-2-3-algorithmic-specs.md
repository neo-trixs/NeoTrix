# Sprint 2 & 3 Algorithmic Specifications
## Unified Router + Context Compactor + Config Hierarchy + Circuit Breaker

**Date**: 2026-09-18
**Status**: Draft — ready for implementation
**Existing foundations**: `CostAwareRouter` (cost_router.rs), `RealTimeModelRouter` (nt_core_model_router.rs), `ProviderBreaker` (circuit_breaker.rs), `CompactionPipeline` (compaction.rs), `ContextCompactor` (context_compaction.rs), `NeoTrixConfig` (config.rs)

---

## Task A: UnifiedModelRouter Algorithm

### Research Base

| Source | Key Insight | NeoTrix Mapping |
|--------|-------------|-----------------|
| MTRouter (ACL 2026) | Turn-level selection under cost budgets | Per-turn `RouteDecision` with budget tracking |
| Cross-Attention Router | query_embedding × model_embedding → quality + cost prediction | Heuristic complexity × model capability scoring |
| Cascade Routing (ETH ICML 2025) | Route between "supermodels" with quality estimators | Quality gate → escalate to next tier |
| LLMRouterBench | Simple baselines often match complex methods | Heuristic classifier, no ML |
| Cursor | Per-turn classifier with 75% uplift threshold | Cost savings target: ≥50% vs always-using-best |
| Axiom A1 | Cost-aware routing | Core scoring formula |

### Algorithm Design

```
UNIFIED ROUTER ALGORITHM:

Input:  ModelRequest { task_type, content, cost_budget, latency_tolerance, context_tokens }
Output: RouteDecision { model, estimated_cost, quality_score, reasoning, cascade_level }

PHASE 1: Complexity Classification (heuristic, ~0.1ms)
  complexity = classify_complexity(content, task_type)
  → returns f64 in [0.0, 1.0]

PHASE 2: Candidate Filtering
  candidates = models.filter(m =>
    m.capability >= complexity.min_capability()  AND
    m.context_window >= context_tokens           AND
    m.latency_p50 <= latency_tolerance           AND
    circuit_breaker[m.provider].is_available()   AND
    estimated_cost(m) <= cost_budget
  )

PHASE 3: Score & Select
  for each candidate:
    score = capability × (1 - cost_weight × normalized_cost) × health_penalty
  select model with highest score

PHASE 4: Cascade Quality Gate
  if selected_score < quality_threshold:
    escalate to next tier model (if exists and within budget)
    set cascade_level = 1

PHASE 5: Return RouteDecision
```

### Complexity Classifier (Heuristic)

```rust
fn classify_complexity(content: &str, task_type: &TaskType) -> f64 {
    let mut score: f64 = 0.0;

    // Signal 1: Content length (longer = more complex)
    let token_est = content.len() / 4;
    score += match token_est {
        0..=50 => 0.0,
        51..=200 => 0.1,
        201..=1000 => 0.2,
        _ => 0.3,
    };

    // Signal 2: Code presence
    if content.contains("```") || content.contains("fn ") || content.contains("impl ") {
        score += 0.15;
    }

    // Signal 3: Multi-step indicators
    let step_words = ["step 1", "first", "then", "finally", "1.", "2."];
    if step_words.iter().any(|w| content.to_lowercase().contains(w)) {
        score += 0.15;
    }

    // Signal 4: Reasoning keywords
    let reasoning_words = ["prove", "derive", "analyze", "compare", "architect", "design"];
    if reasoning_words.iter().any(|w| content.to_lowercase().contains(w)) {
        score += 0.2;
    }

    // Signal 5: Task type base complexity
    score += match task_type {
        TaskType::Chat => 0.0,
        TaskType::Coding => 0.15,
        TaskType::Math => 0.2,
        TaskType::Research => 0.2,
        TaskType::Agent => 0.25,
        TaskType::Creative => 0.1,
        _ => 0.1,
    };

    score.clamp(0.0, 1.0)
}
```

### Score Formula

```
score(model, task) = capability × (1 - cost_weight × norm_cost) × health_penalty × latency_bonus

where:
  norm_cost     = (model.cost_per_input_token × 10000).min(1.0)
  health_penalty = circuit_breaker.health_penalty()  // 1.0=closed, 0.5=half-open, 0.0=open
  latency_bonus  = if model.latency_p50 <= request.latency_tolerance { 1.0 } else { 0.7 }
  cost_weight    = config.routing.cost_weight  // default 0.3
```

### Cascade Logic

```
cascade_threshold = config.routing.quality_threshold  // default 0.5

if selected.score < cascade_threshold AND cascade_enabled:
    // Find next tier up
    next_tier = find_cheapest_model(
        capability >= next_complexity_tier(min_capability),
        within_budget,
        available
    )
    if next_tier.is_some():
        selected = next_tier
        cascade_level = 1
        reasoning += " (cascade: quality gate triggered)"
```

### Complete Rust Implementation

```rust
// neotrix-core/src/l5_cognition/nt_core_unified_router.rs

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use crate::l1_action::nt_io::nt_io_provider::health::circuit_breaker::ProviderBreaker;
use crate::config::RoutingConfig;

// ── Types ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TaskType {
    Chat,
    Coding,
    Math,
    Research,
    Creative,
    Agent,
    DataProcessing,
    Multimodal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LatencyTolerance {
    Low,      // < 1s
    Medium,   // 1-5s
    High,     // 5-30s
    VeryHigh, // 30s+
}

impl LatencyTolerance {
    pub fn max_ms(&self) -> u64 {
        match self {
            Self::Low => 1_000,
            Self::Medium => 5_000,
            Self::High => 30_000,
            Self::VeryHigh => 120_000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRequest {
    pub task_type: TaskType,
    pub content: String,
    pub cost_budget_usd: Option<f64>,
    pub latency_tolerance: LatencyTolerance,
    pub context_tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProfile {
    pub model_id: String,
    pub provider: String,
    pub capability_score: f64,        // 0.0-1.0
    pub cost_per_input_token: f64,    // USD
    pub cost_per_output_token: f64,   // USD
    pub latency_p50_ms: u64,
    pub context_window: usize,
    pub requires_api_key: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteDecision {
    pub model: ModelProfile,
    pub estimated_cost_usd: f64,
    pub estimated_latency_ms: u64,
    pub quality_score: f64,
    pub cascade_level: u8,
    pub reasoning: String,
    pub candidates_evaluated: usize,
}

// ── Complexity Classifier ──────────────────────────────────────

pub fn classify_complexity(content: &str, task_type: &TaskType) -> f64 {
    let mut score: f64 = 0.0;
    let lower = content.to_lowercase();

    // Signal 1: Content length
    let token_est = content.len() / 4;
    score += match token_est {
        0..=50 => 0.0,
        51..=200 => 0.1,
        201..=1000 => 0.2,
        _ => 0.3,
    };

    // Signal 2: Code presence
    if content.contains("```") || content.contains("fn ") || content.contains("impl ") {
        score += 0.15;
    }

    // Signal 3: Multi-step indicators
    let step_markers = ["step 1", "first,", "then,", "finally,", "1.", "2.", "3."];
    if step_markers.iter().any(|m| lower.contains(m)) {
        score += 0.15;
    }

    // Signal 4: Reasoning keywords
    let reasoning_words = ["prove", "derive", "analyze", "compare", "architect", "design", "optimize"];
    if reasoning_words.iter().any(|w| lower.contains(w)) {
        score += 0.2;
    }

    // Signal 5: Task type base complexity
    score += match task_type {
        TaskType::Chat => 0.0,
        TaskType::Coding => 0.15,
        TaskType::Math => 0.2,
        TaskType::Research => 0.2,
        TaskType::Agent => 0.25,
        TaskType::Creative => 0.1,
        TaskType::DataProcessing => 0.1,
        TaskType::Multimodal => 0.15,
    };

    score.clamp(0.0, 1.0)
}

// ── Unified Router ─────────────────────────────────────────────

pub struct UnifiedModelRouter {
    models: Vec<ModelProfile>,
    breakers: HashMap<String, ProviderBreaker>,
    cost_weight: f64,
    quality_threshold: f64,
    cascade_enabled: bool,
    output_ratio: f64,
}

impl UnifiedModelRouter {
    pub fn new(config: &RoutingConfig) -> Self {
        Self {
            models: Vec::new(),
            breakers: HashMap::new(),
            cost_weight: config.cost_weight,
            quality_threshold: config.quality_threshold,
            cascade_enabled: config.cascade_enabled,
            output_ratio: 0.3,
        }
    }

    pub fn register_model(&mut self, model: ModelProfile) {
        self.models.push(model);
    }

    pub fn register_breaker(&mut self, provider: String, breaker: ProviderBreaker) {
        self.breakers.insert(provider, breaker);
    }

    /// Core routing function — the single entry point.
    pub fn route(&self, request: &ModelRequest) -> RouteDecision {
        // Phase 1: Classify complexity
        let complexity = classify_complexity(&request.content, &request.task_type);

        // Phase 2: Filter candidates
        let mut candidates: Vec<(&ModelProfile, f64)> = self.models
            .iter()
            .filter(|m| self.model_is_viable(m, request, complexity))
            .map(|m| {
                let score = self.score_model(m, complexity);
                (m, score)
            })
            .collect();

        // Phase 3: Sort by score descending
        candidates.sort_by(|a, b| b.1.total_cmp(&a.1));

        let candidates_evaluated = candidates.len();

        // Phase 4: Select best candidate
        let (selected_model, selected_score) = candidates
            .first()
            .cloned()
            .unwrap_or_else(|| {
                // Graceful degradation: cheapest available model
                let fallback = self.models.iter()
                    .filter(|m| m.context_window >= request.context_tokens)
                    .min_by(|a, b| {
                        a.cost_per_input_token
                            .partial_cmp(&b.cost_per_input_token)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                match fallback {
                    Some(m) => (m, 0.0),
                    None => return self.no_model_decision(request, candidates_evaluated),
                }
            });

        // Phase 5: Cascade quality gate
        let mut cascade_level = 0u8;
        let mut reasoning = format!(
            "complexity={:.2}, score={:.2}, task={:?}",
            complexity, selected_score, request.task_type
        );

        let mut final_model = selected_model;
        let mut final_score = selected_score;

        if self.cascade_enabled && selected_score < self.quality_threshold {
            if let Some(upgraded) = self.find_escalation_target(request, complexity, &final_model) {
                let upgraded_score = self.score_model(&upgraded, complexity);
                if upgraded_score > final_score {
                    reasoning.push_str(&format!(
                        " (cascade L1: {:.2} < {:.2} threshold, escalated to {})",
                        selected_score, self.quality_threshold, upgraded.model_id
                    ));
                    final_model = upgraded;
                    final_score = upgraded_score;
                    cascade_level = 1;
                }
            }
        }

        // Phase 6: Compute estimates
        let est_input_tokens = request.context_tokens;
        let est_output_tokens = (est_input_tokens as f64 * self.output_ratio) as usize;
        let est_cost = final_model.cost_per_input_token * est_input_tokens as f64
            + final_model.cost_per_output_token * est_output_tokens as f64;
        let est_latency = final_model.latency_p50_ms;

        RouteDecision {
            model: final_model.clone(),
            estimated_cost_usd: est_cost,
            estimated_latency_ms: est_latency,
            quality_score: final_score,
            cascade_level,
            reasoning,
            candidates_evaluated,
        }
    }

    fn model_is_viable(
        &self,
        model: &ModelProfile,
        request: &ModelRequest,
        complexity: f64,
    ) -> bool {
        // Capability threshold
        let min_cap = self.complexity_to_min_capability(complexity);
        if model.capability_score < min_cap {
            return false;
        }

        // Context window
        if model.context_window < request.context_tokens {
            return false;
        }

        // Latency
        if model.latency_p50_ms > request.latency_tolerance.max_ms() {
            return false;
        }

        // Circuit breaker
        if let Some(breaker) = self.breakers.get(&model.provider) {
            if !breaker.is_available() {
                return false;
            }
        }

        // Cost budget
        if let Some(budget) = request.cost_budget_usd {
            let est = self.estimate_cost(model, request.context_tokens);
            if est > budget {
                return false;
            }
        }

        true
    }

    fn score_model(&self, model: &ModelProfile, complexity: f64) -> f64 {
        let norm_cost = (model.cost_per_input_token * 10_000.0).min(1.0);
        let cost_penalty = 1.0 - self.cost_weight * norm_cost;
        let capability = model.capability_score;

        // Health penalty from circuit breaker
        let health = self.breakers
            .get(&model.provider)
            .map(|b| b.health_penalty())
            .unwrap_or(1.0);

        // Latency bonus: penalize if slower than ideal for complexity
        let ideal_latency = self.ideal_latency_for_complexity(complexity);
        let latency_bonus = if model.latency_p50_ms <= ideal_latency {
            1.0
        } else {
            0.7
        };

        capability * cost_penalty * health * latency_bonus
    }

    fn estimate_cost(&self, model: &ModelProfile, input_tokens: usize) -> f64 {
        let output_tokens = (input_tokens as f64 * self.output_ratio) as usize;
        model.cost_per_input_token * input_tokens as f64
            + model.cost_per_output_token * output_tokens as f64
    }

    fn complexity_to_min_capability(&self, complexity: f64) -> f64 {
        // Linear mapping: complexity 0.0→0.1, 1.0→0.95
        0.1 + complexity * 0.85
    }

    fn ideal_latency_for_complexity(&self, complexity: f64) -> u64 {
        // Simple tasks: prefer < 500ms, complex: tolerate 2s+
        (500.0 + complexity * 1500.0) as u64
    }

    fn find_escalation_target(
        &self,
        request: &ModelRequest,
        complexity: f64,
        current: &ModelProfile,
    ) -> Option<ModelProfile> {
        let min_cap = self.complexity_to_min_capability(complexity) + 0.15; // Step up
        self.models
            .iter()
            .filter(|m| {
                m.model_id != current.model_id
                    && m.capability_score >= min_cap
                    && m.context_window >= request.context_tokens
                    && m.latency_p50_ms <= request.latency_tolerance.max_ms()
                    && self.breakers.get(&m.provider).map_or(true, |b| b.is_available())
            })
            .min_by(|a, b| {
                a.cost_per_input_token
                    .partial_cmp(&b.cost_per_input_token)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()
    }

    fn no_model_decision(&self, request: &ModelRequest, evaluated: usize) -> RouteDecision {
        RouteDecision {
            model: ModelProfile {
                model_id: "__no_model__".into(),
                provider: "none".into(),
                capability_score: 0.0,
                cost_per_input_token: 0.0,
                cost_per_output_token: 0.0,
                latency_p50_ms: 0,
                context_window: 0,
                requires_api_key: false,
            },
            estimated_cost_usd: 0.0,
            estimated_latency_ms: 0,
            quality_score: -1.0,
            cascade_level: 0,
            reasoning: "No viable model found".into(),
            candidates_evaluated: evaluated,
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn test_model(id: &str, cap: f64, cost: f64, ctx: usize) -> ModelProfile {
        ModelProfile {
            model_id: id.into(),
            provider: "test".into(),
            capability_score: cap,
            cost_per_input_token: cost,
            cost_per_output_token: cost * 5.0,
            latency_p50_ms: 200,
            context_window: ctx,
            requires_api_key: false,
        }
    }

    fn default_config() -> RoutingConfig {
        RoutingConfig {
            strategy: "cost_optimized".into(),
            cost_weight: 0.3,
            quality_threshold: 0.5,
            cascade_enabled: true,
        }
    }

    #[test]
    fn test_simple_chat_routes_to_cheap() {
        let mut router = UnifiedModelRouter::new(&default_config());
        router.register_model(test_model("gpt-4o-mini", 0.7, 0.00015 / 1000.0, 128_000));
        router.register_model(test_model("gpt-4o", 0.95, 0.0025 / 1000.0, 128_000));

        let decision = router.route(&ModelRequest {
            task_type: TaskType::Chat,
            content: "Hello".into(),
            cost_budget_usd: None,
            latency_tolerance: LatencyTolerance::Low,
            context_tokens: 100,
        });

        assert_eq!(decision.model.model_id, "gpt-4o-mini");
        assert_eq!(decision.cascade_level, 0);
    }

    #[test]
    fn test_complex_coding_escalates() {
        let mut router = UnifiedModelRouter::new(&default_config());
        router.register_model(test_model("gpt-4o-mini", 0.5, 0.00015 / 1000.0, 128_000));
        router.register_model(test_model("gpt-4o", 0.95, 0.0025 / 1000.0, 128_000));

        let decision = router.route(&ModelRequest {
            task_type: TaskType::Coding,
            content: "Implement a concurrent hash map with lock-free reads and fine-grained \
                      locking for writes. Include proper memory ordering and handle ABA problem. \
                      Step 1: design the data structure. Step 2: implement compare-and-swap. \
                      Step 3: add memory barriers.".into(),
            cost_budget_usd: None,
            latency_tolerance: LatencyTolerance::High,
            context_tokens: 500,
        });

        // Should pick gpt-4o due to complexity + cascade
        assert_eq!(decision.model.model_id, "gpt-4o");
    }

    #[test]
    fn test_budget_constraint_respected() {
        let mut router = UnifiedModelRouter::new(&default_config());
        router.register_model(test_model("gpt-4o-mini", 0.7, 0.00015 / 1000.0, 128_000));
        router.register_model(test_model("gpt-4o", 0.95, 0.0025 / 1000.0, 128_000));

        let decision = router.route(&ModelRequest {
            task_type: TaskType::Chat,
            content: "Hello".into(),
            cost_budget_usd: Some(0.00001), // Very tight
            latency_tolerance: LatencyTolerance::Low,
            context_tokens: 100,
        });

        assert!(decision.estimated_cost_usd <= 0.00001 + 1e-10);
    }

    #[test]
    fn test_circuit_breaker_excludes_provider() {
        let mut router = UnifiedModelRouter::new(&default_config());
        router.register_model(test_model("gpt-4o", 0.95, 0.0025 / 1000.0, 128_000));

        let mut breaker = ProviderBreaker::new(3, 60, 20);
        breaker.force_open(); // Provider is down
        router.register_breaker("test".into(), breaker);

        let decision = router.route(&ModelRequest {
            task_type: TaskType::Chat,
            content: "Hello".into(),
            cost_budget_usd: None,
            latency_tolerance: LatencyTolerance::Low,
            context_tokens: 100,
        });

        assert_eq!(decision.model.model_id, "__no_model__");
    }

    #[test]
    fn test_complexity_classification_range() {
        let c_simple = classify_complexity("hi", &TaskType::Chat);
        let c_complex = classify_complexity(
            "Prove that P≠NP using the following approach: Step 1. Analyze the \
             polynomial hierarchy. Step 2. Design an algorithm. Step 3. Optimize.",
            &TaskType::Research,
        );
        assert!(c_simple < c_complex);
        assert!(c_simple >= 0.0 && c_simple <= 1.0);
        assert!(c_complex >= 0.0 && c_complex <= 1.0);
    }
}
```

---

## Task B: Context Compactor Algorithm

### 3-Level Pipeline Design

```
INPUT: messages[], current_tokens, context_limit
OUTPUT: messages[], compaction_report

MONITOR: usage_ratio = current_tokens / context_limit

IF usage_ratio < threshold (default 0.80):
    return messages (no compaction needed)

LEVEL 1: Tool Result Pruning  [COST: 0]
  FOR each message:
    IF message.role == "tool" AND message.content.len > 10240:
      message.content = first_500_chars + "\n... [truncated, N chars total]\n" + last_200_chars
  RECALCULATE tokens
  IF usage_ratio < threshold: return

LEVEL 2: History Snip  [COST: 0]
  KEEP: system_messages + last_N_turns (default 6) + all_tool_call_messages
  REMOVE: middle conversation turns older than threshold
  PRESERVE: any message containing "decision", "important", "must", "critical"
  RECALCULATE tokens
  IF usage_ratio < threshold: return

LEVEL 3: Summary Compaction  [COST: 1 cheap model call]
  SELECT oldest non-critical messages
  CALL cheap_model(summary_prompt, messages)
  STRUCTURED OUTPUT:
    {
      context: "what we're working on",
      decisions_made: ["decision 1", "decision 2"],
      current_state: "current status",
      next_steps: ["step 1", "step 2"]
    }
  REPLACE old messages with summary message
  RECALCULATE tokens
```

### Complete Rust Implementation

```rust
// neotrix-core/src/l1_action/nt_memory/unified_compactor.rs

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

// ── Types ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
    pub is_tool_call: bool,
    pub timestamp: Option<u64>,
}

impl Message {
    pub fn token_estimate(&self) -> usize {
        // ~4 chars per token
        (self.content.len() / 4).max(1)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionConfig {
    /// Token limit for context window
    pub context_limit: usize,
    /// Trigger threshold (0.0-1.0). Default 0.80
    pub threshold: f64,
    /// Number of recent turns to preserve in Level 2
    pub keep_recent_turns: usize,
    /// Tool result size threshold for pruning (bytes). Default 10240
    pub tool_prune_threshold: usize,
    /// Summary model identifier (cheap model for Level 3)
    pub summary_model: String,
}

impl Default for CompactionConfig {
    fn default() -> Self {
        Self {
            context_limit: 128_000,
            threshold: 0.80,
            keep_recent_turns: 6,
            tool_prune_threshold: 10_240,
            summary_model: "gpt-4o-mini".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionReport {
    pub messages_before: usize,
    pub messages_after: usize,
    pub tokens_before: usize,
    pub tokens_after: usize,
    pub levels_applied: Vec<CompactionLevel>,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompactionLevel {
    ToolPruning { messages_trimmed: usize },
    HistorySnip { messages_removed: usize },
    SummaryCompaction { messages_replaced: usize, summary_tokens: usize },
}

// ── Compactor ──────────────────────────────────────────────────

pub struct ContextCompactor {
    config: CompactionConfig,
}

impl ContextCompactor {
    pub fn new(config: CompactionConfig) -> Self {
        Self { config }
    }

    /// Main entry point — runs the 3-level pipeline.
    pub fn compact(&self, messages: &[Message]) -> (Vec<Message>, CompactionReport) {
        let mut msgs: Vec<Message> = messages.to_vec();
        let tokens_before = Self::total_tokens(&msgs);
        let msgs_before = msgs.len();
        let mut levels_applied = Vec::new();

        let usage = tokens_before as f64 / self.config.context_limit as f64;
        if usage < self.config.threshold {
            return (msgs, CompactionReport {
                messages_before: msgs_before,
                messages_after: msgs_before,
                tokens_before,
                tokens_after: tokens_before,
                levels_applied,
                summary: None,
            });
        }

        // Level 1: Tool Result Pruning (zero cost)
        let (trimmed, pruned_count) = self.level1_tool_pruning(&msgs);
        if pruned_count > 0 {
            msgs = trimmed;
            levels_applied.push(CompactionLevel::ToolPruning {
                messages_trimmed: pruned_count,
            });
        }

        let tokens_after_l1 = Self::total_tokens(&msgs);
        if (tokens_after_l1 as f64 / self.config.context_limit as f64) < self.config.threshold {
            return self.report(msgs_before, tokens_before, msgs, levels_applied);
        }

        // Level 2: History Snip (zero cost)
        let (snipped, removed_count) = self.level2_history_snip(&msgs);
        if removed_count > 0 {
            msgs = snipped;
            levels_applied.push(CompactionLevel::HistorySnip {
                messages_removed: removed_count,
            });
        }

        let tokens_after_l2 = Self::total_tokens(&msgs);
        if (tokens_after_l2 as f64 / self.config.context_limit as f64) < self.config.threshold {
            return self.report(msgs_before, tokens_before, msgs, levels_applied);
        }

        // Level 3: Summary Compaction (1 cheap model call)
        let (compacted, summary) = self.level3_summary_compaction(&msgs);
        let summary_tokens = summary.as_ref().map(|s| s.len() / 4).unwrap_or(0);
        msgs = compacted;
        levels_applied.push(CompactionLevel::SummaryCompaction {
            messages_replaced: 1,
            summary_tokens,
        });

        self.report(msgs_before, tokens_before, msgs, levels_applied)
    }

    // ── Level 1: Tool Result Pruning ──────────────────────────

    fn level1_tool_pruning(&self, messages: &[Message]) -> (Vec<Message>, usize) {
        let mut result = Vec::with_capacity(messages.len());
        let mut pruned = 0;

        for msg in messages {
            if msg.role == Role::Tool && msg.content.len() > self.config.tool_prune_threshold {
                let total_len = msg.content.len();
                let head = &msg.content[..500.min(total_len)];
                let tail_start = total_len.saturating_sub(200);
                let tail = &msg.content[tail_start..];

                let pruned_content = if tail_start > 500 {
                    format!(
                        "{}\n... [truncated, {} chars total]\n{}",
                        head, total_len, tail
                    )
                } else {
                    msg.content.clone() // Already small enough after head
                };

                let mut new_msg = msg.clone();
                new_msg.content = pruned_content;
                result.push(new_msg);
                pruned += 1;
            } else {
                result.push(msg.clone());
            }
        }

        (result, pruned)
    }

    // ── Level 2: History Snip ─────────────────────────────────

    fn level2_history_snip(&self, messages: &[Message]) -> (Vec<Message>, usize) {
        if messages.len() <= self.config.keep_recent_turns {
            return (messages.to_vec(), 0);
        }

        let original_len = messages.len();
        let mut result: Vec<Message> = Vec::new();

        // Always keep system messages at the start
        let system_msgs: Vec<&Message> = messages
            .iter()
            .filter(|m| m.role == Role::System)
            .collect();
        for m in &system_msgs {
            result.push((*m).clone());
        }

        // Keep the last N turns (user+assistant pairs)
        let recent_start = messages.len().saturating_sub(self.config.keep_recent_turns * 2);
        let recent: Vec<Message> = messages[recent_start..].to_vec();

        // Also preserve critical messages from the middle
        let critical_keywords = ["decision", "important", "must", "critical", "never", "always"];
        let middle = &messages[system_msgs.len()..recent_start];
        let preserved: Vec<Message> = middle
            .iter()
            .filter(|m| {
                let lower = m.content.to_lowercase();
                critical_keywords.iter().any(|kw| lower.contains(kw))
            })
            .cloned()
            .collect();

        // Add a separator marker
        if !preserved.is_empty() {
            result.push(Message {
                role: Role::System,
                content: "[... earlier conversation ...]".into(),
                is_tool_call: false,
                timestamp: None,
            });
            result.extend(preserved);
        }

        // Add recent turns
        result.extend(recent);

        let removed = original_len - result.len();
        (result, removed)
    }

    // ── Level 3: Summary Compaction ───────────────────────────

    fn level3_summary_compaction(&self, messages: &[Message]) -> (Vec<Message>, Option<String>) {
        // Identify messages to summarize (everything except system and last 2 turns)
        let system_end = messages
            .iter()
            .position(|m| m.role != Role::System)
            .unwrap_or(0);

        let keep_from = messages.len().saturating_sub(4); // Keep last 2 user+assistant pairs
        if keep_from <= system_end {
            return (messages.to_vec(), None);
        }

        let to_summarize = &messages[system_end..keep_from];
        let to_keep_after = &messages[keep_from..];

        // Build summary prompt (this would call a cheap model in production)
        let summary = self.generate_summary(to_summarize);

        let mut result: Vec<Message> = messages[..system_end].to_vec();
        result.push(Message {
            role: Role::System,
            content: format!(
                "[Context Summary]\n{}\n[/Context Summary]",
                summary
            ),
            is_tool_call: false,
            timestamp: None,
        });
        result.extend_from_slice(to_keep_after);

        (result, Some(summary))
    }

    fn generate_summary(&self, messages: &[Message]) -> String {
        // In production: call self.config.summary_model with structured prompt
        // For now: heuristic summary
        let user_msgs: Vec<&str> = messages
            .iter()
            .filter(|m| m.role == Role::User)
            .map(|m| m.content.as_str())
            .collect();

        let assistant_msgs: Vec<&str> = messages
            .iter()
            .filter(|m| m.role == Role::Assistant)
            .map(|m| m.content.as_str())
            .collect();

        let tool_count = messages.iter().filter(|m| m.role == Role::Tool).count();

        format!(
            "Conversation summary ({} messages, {} tool calls):\n\
             User topics: {}\n\
             Assistant responses covered: {} turns",
            messages.len(),
            tool_count,
            user_msgs.iter().take(3).fold(String::new(), |acc, m| {
                if acc.is_empty() {
                    m.chars().take(100).collect()
                } else {
                    format!("{}, {}", acc, m.chars().take(80).collect::<String>())
                }
            }),
            assistant_msgs.len()
        )
    }

    // ── Helpers ────────────────────────────────────────────────

    fn total_tokens(messages: &[Message]) -> usize {
        messages.iter().map(|m| m.token_estimate()).sum()
    }

    fn report(
        msgs_before: usize,
        tokens_before: usize,
        msgs: Vec<Message>,
        levels_applied: Vec<CompactionLevel>,
    ) -> (Vec<Message>, CompactionReport) {
        let tokens_after = Self::total_tokens(&msgs);
        (msgs, CompactionReport {
            messages_before: msgs_before,
            messages_after: msgs.len(),
            tokens_before,
            tokens_after,
            levels_applied,
            summary: None,
        })
    }
}

// ── Tests ──────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn tool_message(size: usize) -> Message {
        Message {
            role: Role::Tool,
            content: "x".repeat(size),
            is_tool_call: false,
            timestamp: None,
        }
    }

    fn user_message(content: &str) -> Message {
        Message {
            role: Role::User,
            content: content.into(),
            is_tool_call: false,
            timestamp: None,
        }
    }

    fn assistant_message(content: &str) -> Message {
        Message {
            role: Role::Assistant,
            content: content.into(),
            is_tool_call: false,
            timestamp: None,
        }
    }

    #[test]
    fn test_level1_truncates_large_tool_results() {
        let compactor = ContextCompactor::new(CompactionConfig {
            tool_prune_threshold: 100,
            ..Default::default()
        });

        let messages = vec![tool_message(500)];
        let (result, report) = compactor.compact(&messages);

        assert_eq!(result.len(), 1);
        assert!(result[0].content.len() < 500);
        assert!(result[0].content.contains("truncated"));
        assert!(!report.levels_applied.is_empty());
    }

    #[test]
    fn test_level1_preserves_small_tool_results() {
        let compactor = ContextCompactor::new(CompactionConfig {
            tool_prune_threshold: 10_240,
            ..Default::default()
        });

        let messages = vec![tool_message(500)];
        let (result, _report) = compactor.compact(&messages);
        assert_eq!(result[0].content.len(), 500);
    }

    #[test]
    fn test_level2_snips_old_messages() {
        let compactor = ContextCompactor::new(CompactionConfig {
            context_limit: 1000,
            threshold: 0.5,
            keep_recent_turns: 2,
            ..Default::default()
        });

        let mut messages = vec![];
        for i in 0..20 {
            messages.push(user_message(&format!("user message {}", i)));
            messages.push(assistant_message(&format!("assistant reply {}", i)));
        }

        let (result, report) = compactor.compact(&messages);
        // Should have fewer messages after snipping
        assert!(result.len() < messages.len());
        assert!(report.levels_applied.iter().any(|l| matches!(l, CompactionLevel::HistorySnip { .. })));
    }

    #[test]
    fn test_level2_preserves_critical_messages() {
        let compactor = ContextCompactor::new(CompactionConfig {
            context_limit: 1000,
            threshold: 0.5,
            keep_recent_turns: 2,
            ..Default::default()
        });

        let mut messages = vec![];
        messages.push(user_message("normal message"));
        messages.push(assistant_message("CRITICAL: Never modify the database schema without approval"));
        messages.push(user_message("another normal message"));
        messages.push(assistant_message("ok"));

        let (result, _report) = compactor.compact(&messages);
        let has_critical = result.iter().any(|m| m.content.contains("CRITICAL"));
        assert!(has_critical);
    }

    #[test]
    fn test_no_compaction_when_under_threshold() {
        let compactor = ContextCompactor::new(CompactionConfig {
            context_limit: 100_000,
            threshold: 0.80,
            ..Default::default()
        });

        let messages = vec![user_message("hi")];
        let (result, report) = compactor.compact(&messages);
        assert_eq!(result.len(), 1);
        assert!(report.levels_applied.is_empty());
    }

    #[test]
    fn test_total_tokens_estimation() {
        let messages = vec![
            user_message("hello world"),         // 2 tokens
            assistant_message("goodbye world"),   // 2 tokens
        ];
        let tokens = ContextCompactor::total_tokens(&messages);
        assert!(tokens >= 2 && tokens <= 6);
    }
}
```

---

## Task C: Config Hierarchy

### Loading Order (lowest → highest priority)

```
1. Global:   ~/.config/neotrix/config.toml
2. Project:  ./neotrix.toml
3. Local:    ./.neotrix.toml  (gitignored)
4. Env vars: NEOTRIX_*        (highest priority)

Merge strategy: deep merge, later layers override earlier layers.
Env vars: NEOTRIX_ROUTING_COST_WEIGHT=0.5 overrides [routing] cost_weight.
```

### TOML Schema

```toml
# ~/.config/neotrix/config.toml (Global defaults)

[model]
provider = "openai"
model = "gpt-4o-mini"
# api_key = "sk-..."            # or encrypted: "enc:..."
base_url = "https://api.openai.com/v1"

[routing]
strategy = "cost_optimized"       # cost_optimized | quality_optimized | latency_optimized
cost_weight = 0.3                 # 0.0-1.0, higher = more cost-sensitive
quality_threshold = 0.5           # cascade trigger threshold
cascade_enabled = true
output_ratio = 0.3                # expected output/input token ratio

[compaction]
enabled = true
threshold = 0.80                  # trigger when context usage > 80%
keep_recent_turns = 6
tool_prune_threshold = 10240      # bytes
summary_model = "gpt-4o-mini"

[providers.openai]
api_key = "sk-..."
base_url = "https://api.openai.com/v1"
models = ["gpt-4o", "gpt-4o-mini"]

[providers.anthropic]
api_key = "sk-ant-..."
base_url = "https://api.anthropic.com"
models = ["claude-sonnet-4-20250514", "claude-opus-4-20250514"]

[providers.local]
base_url = "http://localhost:11434"
models = ["llama-3-8b", "deepseek-r1:7b"]
```

### Rust Config Struct

```rust
// neotrix-core/src/config/unified_config.rs

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ── Config Types ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedConfig {
    #[serde(default)]
    pub model: ModelSection,
    #[serde(default)]
    pub routing: RoutingConfig,
    #[serde(default)]
    pub compaction: CompactionSection,
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSection {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
}

impl Default for ModelSection {
    fn default() -> Self {
        Self {
            provider: Some("openai".into()),
            model: Some("gpt-4o-mini".into()),
            api_key: None,
            base_url: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConfig {
    pub strategy: String,
    pub cost_weight: f64,
    pub quality_threshold: f64,
    pub cascade_enabled: bool,
    pub output_ratio: f64,
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            strategy: "cost_optimized".into(),
            cost_weight: 0.3,
            quality_threshold: 0.5,
            cascade_enabled: true,
            output_ratio: 0.3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionSection {
    pub enabled: bool,
    pub threshold: f64,
    pub keep_recent_turns: usize,
    pub tool_prune_threshold: usize,
    pub summary_model: String,
}

impl Default for CompactionSection {
    fn default() -> Self {
        Self {
            enabled: true,
            threshold: 0.80,
            keep_recent_turns: 6,
            tool_prune_threshold: 10_240,
            summary_model: "gpt-4o-mini".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub models: Vec<String>,
}

// ── Config Loader ──────────────────────────────────────────────

impl UnifiedConfig {
    /// Load config with 4-layer hierarchy.
    pub fn load() -> Self {
        let mut config = Self::load_global();
        config.merge(Self::load_project());
        config.merge(Self::load_local());
        config.apply_env_overrides();
        config
    }

    fn load_global() -> Self {
        let path = Self::global_path();
        Self::load_file(&path).unwrap_or_default()
    }

    fn load_project() -> Self {
        let path = PathBuf::from("neotrix.toml");
        Self::load_file(&path).unwrap_or_default()
    }

    fn load_local() -> Self {
        let path = PathBuf::from(".neotrix.toml");
        Self::load_file(&path).unwrap_or_default()
    }

    fn load_file(path: &Path) -> Option<Self> {
        if !path.exists() {
            return None;
        }
        let content = std::fs::read_to_string(path).ok()?;
        match toml::from_str(&content) {
            Ok(config) => {
                eprintln!("[config] loaded {}", path.display());
                Some(config)
            }
            Err(e) => {
                eprintln!("[config] parse error in {}: {}", path.display(), e);
                None
            }
        }
    }

    /// Deep merge: other overrides self (only non-default values).
    fn merge(&mut self, other: Self) {
        // Model section
        if other.model.provider.is_some() {
            self.model.provider = other.model.provider;
        }
        if other.model.model.is_some() {
            self.model.model = other.model.model;
        }
        if other.model.api_key.is_some() {
            self.model.api_key = other.model.api_key;
        }
        if other.model.base_url.is_some() {
            self.model.base_url = other.model.base_url;
        }

        // Routing section (merge non-default)
        if other.routing.strategy != "cost_optimized" {
            self.routing.strategy = other.routing.strategy;
        }
        if (other.routing.cost_weight - 0.3).abs() > f64::EPSILON {
            self.routing.cost_weight = other.routing.cost_weight;
        }
        if (other.routing.quality_threshold - 0.5).abs() > f64::EPSILON {
            self.routing.quality_threshold = other.routing.quality_threshold;
        }
        if !other.routing.cascade_enabled {
            self.routing.cascade_enabled = other.routing.cascade_enabled;
        }

        // Compaction section
        if (other.compaction.threshold - 0.80).abs() > f64::EPSILON {
            self.compaction.threshold = other.compaction.threshold;
        }
        if other.compaction.keep_recent_turns != 6 {
            self.compaction.keep_recent_turns = other.compaction.keep_recent_turns;
        }
        if other.compaction.summary_model != "gpt-4o-mini" {
            self.compaction.summary_model = other.compaction.summary_model;
        }

        // Providers: merge by key
        for (key, provider) in other.providers {
            self.providers.insert(key, provider);
        }
    }

    /// Apply NEOTRIX_* environment variable overrides.
    fn apply_env_overrides(&mut self) {
        if let Ok(v) = std::env::var("NEOTRIX_MODEL_PROVIDER") {
            self.model.provider = Some(v);
        }
        if let Ok(v) = std::env::var("NEOTRIX_MODEL_MODEL") {
            self.model.model = Some(v);
        }
        if let Ok(v) = std::env::var("NEOTRIX_MODEL_API_KEY") {
            self.model.api_key = Some(v);
        }
        if let Ok(v) = std::env::var("NEOTRIX_MODEL_BASE_URL") {
            self.model.base_url = Some(v);
        }
        if let Ok(v) = std::env::var("NEOTRIX_ROUTING_STRATEGY") {
            self.routing.strategy = v;
        }
        if let Ok(v) = std::env::var("NEOTRIX_ROUTING_COST_WEIGHT") {
            if let Ok(f) = v.parse() {
                self.routing.cost_weight = f;
            }
        }
        if let Ok(v) = std::env::var("NEOTRIX_ROUTING_QUALITY_THRESHOLD") {
            if let Ok(f) = v.parse() {
                self.routing.quality_threshold = f;
            }
        }
        if let Ok(v) = std::env::var("NEOTRIX_COMPACTION_THRESHOLD") {
            if let Ok(f) = v.parse() {
                self.compaction.threshold = f;
            }
        }
        if let Ok(v) = std::env::var("NEOTRIX_COMPACTION_SUMMARY_MODEL") {
            self.compaction.summary_model = v;
        }
    }

    pub fn global_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".config")
            .join("neotrix")
            .join("config.toml")
    }
}

impl Default for UnifiedConfig {
    fn default() -> Self {
        Self {
            model: ModelSection::default(),
            routing: RoutingConfig::default(),
            compaction: CompactionSection::default(),
            providers: HashMap::new(),
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = UnifiedConfig::default();
        assert_eq!(config.model.provider.as_deref(), Some("openai"));
        assert_eq!(config.routing.cost_weight, 0.3);
        assert_eq!(config.compaction.threshold, 0.80);
    }

    #[test]
    fn test_merge_prefers_later_values() {
        let mut base = UnifiedConfig::default();
        let override_config = UnifiedConfig {
            model: ModelSection {
                provider: Some("anthropic".into()),
                ..Default::default()
            },
            ..Default::default()
        };
        base.merge(override_config);
        assert_eq!(base.model.provider.as_deref(), Some("anthropic"));
    }

    #[test]
    fn test_toml_roundtrip() {
        let config = UnifiedConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: UnifiedConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.routing.cost_weight, 0.3);
    }
}
```

---

## Task D: Circuit Breaker Design

### State Machine

```
           ┌─────────────────────────────────────┐
           │                                     │
           ▼                                     │
      ┌─────────┐    N failures     ┌──────────┐ │  success
      │ CLOSED  │──────────────────▶│   OPEN   │ │◀─┐
      │ (normal)│                    │ (failing)│   │
      └─────────┘                    └──────────┘   │
           ▲                          │             │
           │                          │ timeout     │
           │                          ▼             │
           │   success          ┌──────────┐        │
           └────────────────────│HALF_OPEN │────────┘
                                │ (testing)│  failure
                                └──────────┘

Parameters:
  failure_threshold: 3    (CLOSED → OPEN after 3 consecutive failures)
  cooldown_secs: 60       (OPEN → HALF_OPEN after 60 seconds)
  half_open_max_probes: 1 (allow 1 probe in HALF_OPEN)
```

### Complete Rust Implementation

```rust
// neotrix-core/src/l1_action/nt_io/nt_io_provider/health/unified_circuit_breaker.rs

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

// ── Types ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BreakerState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures to trip the breaker (CLOSED → OPEN)
    pub failure_threshold: u32,
    /// Duration to wait before trying again (OPEN → HALF_OPEN)
    pub cooldown_secs: u64,
    /// Max probe requests allowed in HALF_OPEN state
    pub half_open_max_probes: u32,
    /// Sliding window size for failure rate calculation
    pub window_size: usize,
    /// Minimum calls before failure rate is meaningful
    pub min_calls_for_rate: usize,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 3,
            cooldown_secs: 60,
            half_open_max_probes: 1,
            window_size: 20,
            min_calls_for_rate: 5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerMetrics {
    pub total_calls: u64,
    pub total_failures: u64,
    pub total_successes: u64,
    pub consecutive_failures: u32,
    pub last_failure_time: Option<u64>,
    pub state_changes: u64,
    pub half_open_probes_used: u32,
}

// ── Circuit Breaker ────────────────────────────────────────────

pub struct UnifiedCircuitBreaker {
    config: CircuitBreakerConfig,
    state: BreakerState,
    consecutive_failures: u32,
    last_state_change: Option<Instant>,
    half_open_probes_used: u32,
    sliding_window: VecDeque<bool>,
    metrics: CircuitBreakerMetrics,
}

impl UnifiedCircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: BreakerState::Closed,
            consecutive_failures: 0,
            last_state_change: None,
            half_open_probes_used: 0,
            sliding_window: VecDeque::with_capacity(config.window_size),
            metrics: CircuitBreakerMetrics {
                total_calls: 0,
                total_failures: 0,
                total_successes: 0,
                consecutive_failures: 0,
                last_failure_time: None,
                state_changes: 0,
                half_open_probes_used: 0,
            },
        }
    }

    /// Check if the circuit breaker allows a request.
    pub fn is_available(&self) -> bool {
        match self.state {
            BreakerState::Closed => true,
            BreakerState::HalfOpen => {
                self.half_open_probes_used < self.config.half_open_max_probes
            }
            BreakerState::Open => {
                // Check if cooldown has elapsed
                if let Some(t) = self.last_state_change {
                    t.elapsed() >= Duration::from_secs(self.config.cooldown_secs)
                } else {
                    true
                }
            }
        }
    }

    /// Record a successful call.
    pub fn on_success(&mut self) {
        self.metrics.total_calls += 1;
        self.metrics.total_successes += 1;

        self.sliding_window.push_back(true);
        if self.sliding_window.len() > self.config.window_size {
            self.sliding_window.pop_front();
        }

        match self.state {
            BreakerState::Closed => {
                // Reset failure count on success
                self.consecutive_failures = 0;
            }
            BreakerState::HalfOpen => {
                // Success in HALF_OPEN → transition to CLOSED
                self.state = BreakerState::Closed;
                self.consecutive_failures = 0;
                self.half_open_probes_used = 0;
                self.last_state_change = Some(Instant::now());
                self.metrics.state_changes += 1;
            }
            BreakerState::Open => {
                // Unexpected success while open (shouldn't happen if is_available checked)
                // Treat as HALF_OPEN probe success
                self.state = BreakerState::Closed;
                self.consecutive_failures = 0;
                self.last_state_change = Some(Instant::now());
                self.metrics.state_changes += 1;
            }
        }
    }

    /// Record a failed call.
    pub fn on_failure(&mut self) {
        self.metrics.total_calls += 1;
        self.metrics.total_failures += 1;
        self.consecutive_failures += 1;
        self.metrics.consecutive_failures = self.consecutive_failures;
        self.metrics.last_failure_time = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );

        self.sliding_window.push_back(false);
        if self.sliding_window.len() > self.config.window_size {
            self.sliding_window.pop_front();
        }

        match self.state {
            BreakerState::Closed => {
                // Check if we should trip
                if self.consecutive_failures >= self.config.failure_threshold {
                    self.state = BreakerState::Open;
                    self.last_state_change = Some(Instant::now());
                    self.metrics.state_changes += 1;
                }
            }
            BreakerState::HalfOpen => {
                // Failure in HALF_OPEN → back to OPEN
                self.state = BreakerState::Open;
                self.last_state_change = Some(Instant::now());
                self.half_open_probes_used = 0;
                self.metrics.state_changes += 1;
            }
            BreakerState::Open => {
                // Already open, just update timestamp
                self.last_state_change = Some(Instant::now());
            }
        }
    }

    /// Attempt to transition from OPEN → HALF_OPEN.
    /// Should be called periodically (e.g., health check loop).
    pub fn tick(&mut self) {
        if self.state == BreakerState::Open {
            if let Some(t) = self.last_state_change {
                if t.elapsed() >= Duration::from_secs(self.config.cooldown_secs) {
                    self.state = BreakerState::HalfOpen;
                    self.half_open_probes_used = 0;
                    self.last_state_change = Some(Instant::now());
                    self.metrics.state_changes += 1;
                }
            }
        }
    }

    /// Health penalty score for routing (1.0 = healthy, 0.0 = down).
    pub fn health_penalty(&self) -> f64 {
        match self.state {
            BreakerState::Closed => {
                // Partial penalty based on recent failure rate
                let rate = self.failure_rate();
                1.0 - (rate * 0.3) // Max 30% penalty in closed state
            }
            BreakerState::HalfOpen => 0.3,
            BreakerState::Open => 0.0,
        }
    }

    /// Current failure rate from sliding window.
    pub fn failure_rate(&self) -> f64 {
        let len = self.sliding_window.len();
        if len < self.config.min_calls_for_rate {
            return 0.0;
        }
        let failures = self.sliding_window.iter().filter(|&&s| !s).count();
        failures as f64 / len as f64
    }

    /// Force the breaker open (e.g., for manual intervention).
    pub fn force_open(&mut self) {
        self.state = BreakerState::Open;
        self.last_state_change = Some(Instant::now());
        self.metrics.state_changes += 1;
    }

    /// Force reset to closed (e.g., after manual recovery).
    pub fn force_reset(&mut self) {
        self.state = BreakerState::Closed;
        self.consecutive_failures = 0;
        self.half_open_probes_used = 0;
        self.sliding_window.clear();
    }

    /// Get current state.
    pub fn state(&self) -> BreakerState {
        self.state
    }

    /// Get metrics snapshot.
    pub fn metrics(&self) -> CircuitBreakerMetrics {
        self.metrics.clone()
    }
}

// ── Tests ──────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn default_breaker() -> UnifiedCircuitBreaker {
        UnifiedCircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 3,
            cooldown_secs: 1,
            half_open_max_probes: 1,
            window_size: 10,
            min_calls_for_rate: 3,
        })
    }

    #[test]
    fn test_initial_state_is_closed() {
        let cb = default_breaker();
        assert_eq!(cb.state(), BreakerState::Closed);
        assert!(cb.is_available());
    }

    #[test]
    fn test_trip_after_threshold_failures() {
        let mut cb = default_breaker();
        cb.on_failure();
        cb.on_failure();
        assert_eq!(cb.state(), BreakerState::Closed); // Not yet

        cb.on_failure();
        assert_eq!(cb.state(), BreakerState::Open);
        assert!(!cb.is_available());
    }

    #[test]
    fn test_success_resets_failure_count() {
        let mut cb = default_breaker();
        cb.on_failure();
        cb.on_failure();
        cb.on_success(); // Reset

        cb.on_failure();
        cb.on_failure();
        assert_eq!(cb.state(), BreakerState::Closed); // Should not trip
    }

    #[test]
    fn test_open_to_half_open_after_cooldown() {
        let mut cb = default_breaker();
        cb.on_failure();
        cb.on_failure();
        cb.on_failure(); // Trip to Open
        assert_eq!(cb.state(), BreakerState::Open);

        // Wait for cooldown (1 second in test config)
        std::thread::sleep(Duration::from_millis(1100));
        cb.tick();
        assert_eq!(cb.state(), BreakerState::HalfOpen);
        assert!(cb.is_available());
    }

    #[test]
    fn test_half_open_success_closes() {
        let mut cb = default_breaker();
        cb.on_failure();
        cb.on_failure();
        cb.on_failure(); // Trip
        std::thread::sleep(Duration::from_millis(1100));
        cb.tick(); // → HalfOpen
        cb.on_success(); // → Closed
        assert_eq!(cb.state(), BreakerState::Closed);
        assert_eq!(cb.consecutive_failures, 0);
    }

    #[test]
    fn test_half_open_failure_reopens() {
        let mut cb = default_breaker();
        cb.on_failure();
        cb.on_failure();
        cb.on_failure(); // Trip
        std::thread::sleep(Duration::from_millis(1100));
        cb.tick(); // → HalfOpen
        cb.on_failure(); // → Open again
        assert_eq!(cb.state(), BreakerState::Open);
    }

    #[test]
    fn test_health_penalty_values() {
        let mut cb = default_breaker();
        assert_eq!(cb.health_penalty(), 1.0); // Closed, no failures

        cb.on_failure();
        cb.on_failure();
        cb.on_failure(); // Open
        assert_eq!(cb.health_penalty(), 0.0);

        std::thread::sleep(Duration::from_millis(1100));
        cb.tick(); // HalfOpen
        assert_eq!(cb.health_penalty(), 0.3);
    }

    #[test]
    fn test_failure_rate_calculation() {
        let mut cb = default_breaker();
        // Need min_calls_for_rate (3) calls
        cb.on_success();
        cb.on_success();
        cb.on_failure(); // 1/3 = 33%
        assert!((cb.failure_rate() - 0.333).abs() < 0.01);
    }

    #[test]
    fn test_force_open_and_reset() {
        let mut cb = default_breaker();
        cb.force_open();
        assert_eq!(cb.state(), BreakerState::Open);
        assert!(!cb.is_available());

        cb.force_reset();
        assert_eq!(cb.state(), BreakerState::Closed);
        assert!(cb.is_available());
    }

    #[test]
    fn test_metrics_tracking() {
        let mut cb = default_breaker();
        cb.on_success();
        cb.on_failure();
        cb.on_failure();

        let metrics = cb.metrics();
        assert_eq!(metrics.total_calls, 3);
        assert_eq!(metrics.total_successes, 1);
        assert_eq!(metrics.total_failures, 2);
    }
}
```

---

## Integration: How Components Connect

```
UnifiedConfig
  │
  ├─→ UnifiedModelRouter
  │     ├─ reads: routing.{strategy, cost_weight, quality_threshold, cascade_enabled}
  │     ├─ reads: model.{provider, model, api_key, base_url}
  │     ├─ uses: ProviderBreaker per provider (from health module)
  │     └─ output: RouteDecision
  │
  ├─→ ContextCompactor
  │     ├─ reads: compaction.{enabled, threshold, keep_recent_turns, summary_model}
  │     └─ output: (Vec<Message>, CompactionReport)
  │
  └─→ UnifiedCircuitBreaker (per provider)
        ├─ config: failure_threshold, cooldown_secs, half_open_max_probes
        └─ used by: UnifiedModelRouter.model_is_viable()
```

### Config Loading Flow

```
CLI启动
  │
  ├─ UnifiedConfig::load()
  │   ├─ load_global(~/.config/neotrix/config.toml)
  │   ├─ load_project(./neotrix.toml)
  │   ├─ load_local(./.neotrix.toml)
  │   ├─ apply_env_overrides(NEOTRIX_*)
  │   └─ return UnifiedConfig
  │
  ├─ UnifiedModelRouter::new(&config.routing)
  │   └─ register models from config.providers
  │
  ├─ ContextCompactor::new(&config.compaction)
  │
  └─ UnifiedCircuitBreaker per provider
```

---

## File Locations

| File | Path |
|------|------|
| Unified Router | `neotrix-core/src/l5_cognition/nt_core_unified_router.rs` |
| Unified Compactor | `neotrix-core/src/l1_action/nt_memory/unified_compactor.rs` |
| Unified Config | `neotrix-core/src/config/unified_config.rs` |
| Unified Circuit Breaker | `neotrix-core/src/l1_action/nt_io/nt_io_provider/health/unified_circuit_breaker.rs` |

These files are **new additions** — they extend (not replace) existing implementations. The existing `cost_router.rs`, `context_compaction.rs`, `circuit_breaker.rs`, and `config.rs` remain as backward-compatible modules.
