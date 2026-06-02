# OmniAgent — NeoTrix Fusion Design

> **Created**: 2026-05-28
> **Status**: Design doc — awaiting implementation
> **Source**: https://github.com/YeQing17-2026/OmniAgent (v1.0, Python, 1.5k stars)
> **License**: GPL-3.0

## 1. Project Overview

**OmniAgent** is an open-source self-evolving AI agent framework (Python, 86.7% Python + 13.3% HTML) implementing **OmniEvolve** — full-dimensional self-evolution across Skill, Context, and BrainModel axes. Developed by YeQing17-2026, it's positioned as the first agent framework to achieve real-time self-evolution during execution (vs periodic evolution in Hermes, static skills in OpenClaw).

### Key Technologies
- **Proactive Memory**: Dual-path alignment (explicit user feedback + implicit LLM induction) for self-evolving user profiles
- **Skill Self-Evolution**: Pattern extraction from high-frequency action sequences → native skill auto-generation; dual-path feedback (user + LLM diagnosis) → automatic skill repair
- **Context Self-Evolution**: Multi-layer information stack with real-time preference signal capture → adaptive Personalization Context
- **BrainModel Self-Evolution**: Online RL (GRPO + PRM) feedback loop → closed-loop model self-evolution during interactive use
- **Hyper-Harness**: Progressive context loading (L0/L1/L2), Dynamic Multi-Agent (Sentinel + Guardian), Dynamic Concurrent Tool Execution, 4-Layer Dynamic Security Scanning
- **Deep Reflexion**: Inner-outer dual-layer reflective architecture — inner failure prevention (trajectory repetition, error action repetition, loop pseudo-termination) + outer failure-to-insight conversion (LLM-driven RCA + heuristic strategy extraction)

### Architecture
```
Channels: CLI · Web UI · Feishu · Discord · Telegram
        ↕
Gateway (WebSocket + HTTP · Session Management)
        ↕
Reflexion Agent-Loop
  ├── Deep Reflexion (dual-layer)
  ├── Hyper-Harness (progressive loading, 4-layer security, concurrent tools)
  ├── Sentinel Agent (planning)
  ├── Guardian Agent (safety review)
  └── OmniEvolve
        ├── Proactive Memory
        ├── Skill Self-Evolution
        ├── Personalization Context
        └── BrainModel Self-Evolution (GRPO + PRM RL)
        ↕
LLM Providers: DeepSeek · OpenAI · Anthropic · Ollama · Gemini · OpenRouter · vLLM · SGLang
```

### Project Structure (omniagent/)
```
omniagent/
├── agents/       # Core: reflexion loop, sentinel, guardian, skill/memory evolution, context
├── security/     # Policy engine, approval, audit, sandbox
├── tools/        # Built-in tools
├── channels/     # Feishu, Discord, Telegram, Webhook
├── config/       # OmniAgentConfig + sub-configs
├── gateway/      # WebSocket + HTTP server
└── rl/           # GRPO + PRM training pipeline
```

---

## 2. Gap Matrix

| Feature | OmniAgent | NeoTrix | Priority | Notes |
|---------|-----------|---------|----------|-------|
| **Skill Self-Evolution** | Auto-create skills from action sequences; auto-repair via dual-path feedback | SkillCrystallizer (basic: reward-thresholded crystallization from tasks) + SkillsEngine (executor/registry) | **P2** | NeoTrix can crystallize skills but lacks: (a) action-sequence mining, (b) self-diagnosis, (c) auto-repair with LLM feedback |
| **Context Self-Evolution** | Multi-layer info stack (L0/L1/L2), real-time preference capture → Personalization Context | Static context window (ContextWindow w/ capacity 512); no adaptive context strategy | **P2** | NeoTrix has ContextWindow with attention masking but no personalization or multi-layer progressive loading |
| **BrainModel Self-Evolution** | Online RL (GRPO + PRM) → model self-evolution during interaction | CapabilityVector updates via `absorb()` + SEAL loop; no underlying model training | **P3** | NeoTrix evolves meta-cognitive vectors, not the LLM itself. Full online RL requires model hosting |
| **Deep Reflexion** | Dual-layer: inner failure prevention (3 mechanisms) + outer failure-to-insight (RCA + heuristic extraction) | SiliconSelf reflection cycles (`run_reflection_cycle()`) with ThinkingTrace + ReflectionGrade | **P2** | NeoTrix has single-layer reflection with grade evaluation, but no inner failure prevention or RCA-to-heuristic-conversion pipeline |
| **Hyper-Harness** | Progressive context loading, dynamic multi-agent (Sentinel + Guardian), concurrent tool execution, 4-layer security scanning | Basic tool execution harness; AgentTeam has multi-agent orchestration | **P3** | AgentTeam provides architecture but no progressive loading, concurrent tool execution, or layered security scanning |
| **Proactive Memory** | Dual-path alignment: explicit user feedback + implicit LLM induction → self-evolving profiles | CortexMemory + ReasoningBank with auto-memory-iteration; no alignment mechanism | **P2** | NeoTrix stores memories but doesn't align them via dual-path signals |
| **Multi-Agent Safety** | Sentinel (planning) + Guardian (safety) dynamically activated based on task risk | AgentTeam with Coordinator; no dynamic safety agents | **P3** | AgentTeam architecture exists but Sentinel/Guardian role pattern is absent |
| **4-Layer Security** | LLM review → Policy engine → Interactive approval → Execution sandbox; trust-level classification | Basic sandbox mentions; no layered security pipeline | **P3** | Needs integration with HashCortxSecurity prototype |
| **Concurrent Tool Execution** | Auto-resolves inter-tool dependencies, async parallel invocation | Serial tool execution | **P3** | Orchestrator has ParallelExecutor but tools execute sequentially |
| **Self-Deployed Model RL** | GRPO + PRM training pipeline (`rl/` dir) | No equivalent — all evolution is prompt/vector-level | **P4** | Requires external compute; out of scope for current phase |
| **Multi-Channel Gateway** | Feishu, Discord, Telegram, Webhook, CLI, Web | CLI (headless) + Web (Tauri/Svelte) | **P4** | NeoTrix has fewer channel integrations |

### Priority Definitions
- **P2**: Directly enhances NeoTrix core self-evolution loop — implement in current phase
- **P3**: Enhances safety/harness/infrastructure — implement after P2
- **P4**: Requires external infrastructure or architectural shift — deferred

---

## 3. P2-4: Full-Dimension Evolution Framework (spec for implementation)

### 3.1 Design

The goal is to integrate OmniAgent's 3-axis evolution (Skill + Context + Brain) into NeoTrix's existing SEAL loop, extending rather than replacing the current architecture.

#### Skill Evolution (P2)
**Current NeoTrix**: SkillCrystallizer creates skills from tasks with reward > threshold; SkillsEngine executes + tracks stats.
**Target**: Add self-diagnosis, auto-repair, and action-sequence-mining on top.

**Flow**:
```
User task → SEAL loop execution → SkillCrystallizer attempts crystallization
         ↕ (dual-path feedback)
Skill Diagnoser ← user feedback + LLM diagnosis → Skill Repairer → updated Skill
         ↕
ActionSequenceMiner ← tool_call_traces → pattern extraction → new Skill proposals
```

Key additions:
1. **ActionSequenceMiner**: Reads `tool_traces: Vec<(String, u64, bool)>` from SelfIteratingBrain, clusters repeated patterns, proposes new Skills
2. **SkillDiagnoser**: Takes user feedback (explicit rating) + LLM evaluation (implicit diagnosis) → determines if Skill needs repair
3. **SkillRepairer**: Generates MicroEdit sequence targeting the skill's step definitions or confidence vector

#### Context Evolution (P2)
**Current NeoTrix**: ContextWindow with fixed 512 capacity, attention masking for relevance filtering.
**Target**: Multi-layer progressive loading + personalization context.

**Design**:
```
ContextEvolver
  ├── LayerManager (L0: system prompt, L1: active task, L2: archive/memory)
  ├── PreferenceTracker (captures multi-dimensional signals from user interactions)
  └── ContextStrategy (selects layer depth + window size based on task pattern)
```

The ContextEvolver sits between the Agent-Loop and the ContextWindow:
- **LayerManager**: Implements progressive disclosure — L0 always loaded, L1 loaded on task start, L2 loaded on demand (when relevance score > threshold)
- **PreferenceTracker**: Monitors `tool_traces`, user corrections, task type distribution → builds user preference profile
- **ContextStrategy**: Uses task type + preference profile to select optimal context configuration

Integration with existing `ContextWindow` (in `core/thinking_model/context_window.rs`):
- Add `layer_config: ContextLayerConfig` field
- Implement `progressive_load()` that returns tokens in layers
- PreferenceTracker writes to `ReasoningBank` as special memory type

#### Brain Evolution (P3)
**Current NeoTrix**: CapabilityVector updated via `absorb()` with learning_rate; ReasoningBank stores knowledge.
**Target**: Add online RL feedback loop (simplified — no GRPO/PRM, but RL signal from capability reward).

**Design**:
```
BrainEvolver
  ├── RewardCollector (aggregates external rewards from tool verification + user feedback)
  ├── PolicyUpdater (updates CapabilityVector update policy based on reward history)
  └── StrategyOptimizer (selects reasoning strategy based on task type + capability profile)
```

This is a lightweight substitute for full model RL:
- `RewardCollector` computes EMA of external rewards per task type
- `PolicyUpdater` adjusts `learning_rate` and `regularization_weight` based on reward trends
- `StrategyOptimizer` updates the `ReasoningStrategyRegistry` weights

### 3.2 Integration Points

| Module | Integration | File |
|--------|------------|------|
| **SelfIteratingBrain** | Add `skill_evolver`, `context_evolver`, `brain_evolver` fields; wire into `run_seal_loop()` after tool execution phase | `neotrix/reasoning_brain/self_iterating/loop_impl/core.rs` |
| **SkillCrystallizer** | Extend with `ActionSequenceMiner` + `SkillDiagnoser` + `SkillRepairer` | `neotrix/reasoning_brain/self_iterating/skill_crystallizer.rs` |
| **SkillsEngine** | Add `self_diagnose(&mut self, skill_id, feedback)` and `auto_repair(&mut self, skill_id)` methods | `agent/skills/execution.rs` |
| **ContextWindow** | Add `layer_config` + `progressive_load()` method | `core/thinking_model/context_window.rs` |
| **ContextEvolver** (new) | New module; owns `LayerManager` + `PreferenceTracker` + `ContextStrategy` | `neotrix/reasoning_brain/context_evolver.rs` |
| **BrainEvolver** (new) | New module; owns `RewardCollector` + `PolicyUpdater` + `StrategyOptimizer` | `neotrix/reasoning_brain/brain_evolver.rs` |
| **CapabilityVector** | Add `adjust_learning_rate(reward_trend)` method | `neotrix/reasoning_brain/core.rs` |
| **ReasoningBank** | Add `MemoryType::PreferenceProfile` variant for preference tracking | `neotrix/reasoning_brain/memory.rs` |
| **ReasoningStrategyRegistry** | Add `weight` field per strategy; update via `StrategyOptimizer` | `core/thinking_model/reasoning_strategy.rs` |
| **Hyper-Harness** (new) | New module: `ProgressiveLoader` + `ConcurrentExecutor` + `SecurityScanner` | `neotrix/harness/` |
| **DeepReflexion** (new) | New module: inner failure prevention + outer RCA pipeline | `neotrix/reasoning_brain/deep_reflexion.rs` |
| **AgentTeam/Coordinator** | Add Sentinel/Guardian role templates | `agent/team.rs` |
| **SelfIteratingBrain** | Wire `ConcurrentExecutor` into seal loop tool execution phase | `self_iterating/loop_impl/seal_loop.rs` |

### 3.3 Interface Design

```rust
// ============================================
// Core types for Full-Dimension Evolution
// ============================================

/// The three evolution axes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EvolutionAxis {
    Skill,
    Context,
    Brain,
}

/// One round of evolution across all axes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionRound {
    pub timestamp: u64,
    pub skill_delta: f64,     // improvement in skill effectiveness
    pub context_delta: f64,   // improvement in context relevance
    pub brain_delta: f64,     // improvement in capability vector
    pub external_reward: Option<f64>,
    pub internal_reward: f64,
}

/// Full-dimension evolution orchestrator
pub struct FullDimensionEvolver {
    pub skill: SkillEvolver,
    pub context: ContextEvolver,
    pub brain: BrainEvolver,
    pub rounds: Vec<EvolutionRound>,
    pub evolution_interval: u64,  // evolve every N iterations
    pub last_evolution: u64,
}

impl FullDimensionEvolver {
    pub fn new() -> Self;
    pub fn evolve(&mut self, brain: &mut ReasoningBrain, traces: &[ToolTrace]) -> EvolutionResult;
    pub fn should_evolve(&self, iteration: u64) -> bool;
}

// ============================================
// Skill Evolution
// ============================================

/// Mines action sequences from tool traces to propose new skills
pub struct ActionSequenceMiner {
    pub min_sequence_length: usize,
    pub min_frequency: usize,
}

impl ActionSequenceMiner {
    pub fn mine(&self, traces: &[ToolTrace]) -> Vec<SkillProposal>;
    pub fn cluster_sequences(&self, sequences: &[Vec<String>>]) -> Vec<Vec<String>>;
}

/// Diagnoses skill issues from dual-path feedback
pub struct SkillDiagnoser {
    pub llm_diagnosis_enabled: bool,
}

impl SkillDiagnoser {
    pub fn diagnose(&self, skill: &Skill, user_feedback: Option<f64>, llm_eval: Option<String>) -> SkillDiagnosis;
}

/// Repairs skills by generating micro-edits
pub struct SkillRepairer;

impl SkillRepairer {
    pub fn repair(&self, skill: &mut Skill, diagnosis: &SkillDiagnosis) -> Vec<MicroEdit>;
}

/// A proposed skill from action sequence mining
pub struct SkillProposal {
    pub name: String,
    pub confidence: f64,
    pub steps: Vec<String>,
    pub source_sequences: Vec<Vec<String>>,
}

pub struct SkillDiagnosis {
    pub needs_repair: bool,
    pub issue_type: SkillIssue,
    pub severity: f64,
    pub suggested_fix: String,
}

pub enum SkillIssue {
    LowConfidence,
    MissingStep,
    IncorrectOrder,
    OutdatedPattern,
    UserRejected,
}

pub struct ToolTrace {
    pub tool_name: String,
    pub args: Vec<String>,
    pub success: bool,
    pub duration_ms: u64,
    pub timestamp: u64,
}

// ============================================
// Context Evolution
// ============================================

/// Multi-layer context loading
pub struct LayerManager {
    pub layers: Vec<ContextLayer>,
    pub active_layers: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextLayer {
    pub level: u8,            // 0, 1, 2
    pub content: String,
    pub max_tokens: usize,
    pub priority: f64,
    pub access_count: u64,
}

impl LayerManager {
    pub fn progressive_load(&self, depth: u8, context_window: &mut ContextWindow) -> usize;
    pub fn promote_layer(&mut self, layer_idx: usize);
    pub fn demote_layer(&mut self, layer_idx: usize);
}

/// Captures user preferences from interaction patterns
pub struct PreferenceTracker {
    pub profile: PreferenceProfile,
    pub signal_history: Vec<PreferenceSignal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferenceProfile {
    pub preferred_task_types: Vec<TaskType>,
    pub context_depth_preference: f64,  // 0.0 = minimal, 1.0 = maximal
    pub verbosity_preference: f64,
    pub safety_tolerance: f64,
    pub last_updated: u64,
}

pub enum PreferenceSignal {
    TaskTypeFrequency(TaskType),
    ContextDepthUsage(u8),
    UserCorrection(String),
    ToolRejection(String),
}

impl PreferenceTracker {
    pub fn record_signal(&mut self, signal: PreferenceSignal);
    pub fn update_profile(&mut self) -> PreferenceProfile;
    pub fn profile(&self) -> &PreferenceProfile;
}

/// Selects context strategy based on task + preference
pub struct ContextStrategy {
    pub current_depth: u8,
    pub window_size_override: Option<usize>,
}

impl ContextStrategy {
    pub fn select(&self, task_type: TaskType, profile: &PreferenceProfile) -> ContextConfig;
}

pub struct ContextConfig {
    pub depth: u8,
    pub window_size: usize,
    pub include_archive: bool,
    pub include_preferences: bool,
}

pub struct ContextEvolver {
    pub layers: LayerManager,
    pub preferences: PreferenceTracker,
    pub strategy: ContextStrategy,
}

impl ContextEvolver {
    pub fn new() -> Self;
    pub fn evolve(&mut self, task_type: TaskType, signals: &[PreferenceSignal]);
    pub fn apply(&mut self, context_window: &mut ContextWindow) -> ContextConfig;
}

// ============================================
// Brain Evolution
// ============================================

/// Collects rewards from external and internal sources
pub struct RewardCollector {
    pub external_rewards: Vec<(f64, u64)>,  // (reward, timestamp)
    pub internal_rewards: Vec<(f64, u64)>,
    pub ema_alpha: f64,
}

impl RewardCollector {
    pub fn record_external(&mut self, reward: f64);
    pub fn record_internal(&mut self, reward: f64);
    pub fn ema_external(&self) -> f64;
    pub fn ema_internal(&self) -> f64;
    pub fn combined_reward(&self, external_weight: f64) -> f64;
}

/// Updates capability vector policy based on reward trends
pub struct PolicyUpdater {
    pub base_learning_rate: f64,
    pub min_learning_rate: f64,
    pub max_learning_rate: f64,
    pub reward_window: VecDeque<f64>,
}

impl PolicyUpdater {
    pub fn update_learning_rate(&mut self, reward: f64) -> f64;
    pub fn update_regularization(&mut self, reward_volatility: f64) -> f64;
}

/// Optimizes reasoning strategy weights
pub struct StrategyOptimizer {
    pub strategy_weights: HashMap<String, f64>,
    pub exploration_rate: f64,
}

impl StrategyOptimizer {
    pub fn select_strategy(&self, task_type: TaskType) -> String;
    pub fn update_weight(&mut self, strategy: &str, delta: f64);
}

pub struct BrainEvolver {
    pub rewards: RewardCollector,
    pub policy: PolicyUpdater,
    pub strategies: StrategyOptimizer,
}

impl BrainEvolver {
    pub fn new() -> Self;
    pub fn evolve(&mut self, brain: &mut ReasoningBrain, external_reward: Option<f64>);
    pub fn stats(&self) -> BrainEvolverStats;
}

pub struct BrainEvolverStats {
    pub current_learning_rate: f64,
    pub external_reward_ema: f64,
    pub internal_reward_ema: f64,
    pub strategy_count: usize,
}

// ============================================
// Deep Reflexion (P2)
// ============================================

/// Inner-outer dual-layer reflective architecture
pub struct DeepReflexion {
    pub inner: InnerFailurePrevention,
    pub outer: OuterFailureConversion,
}

/// Three-layer inner failure prevention
pub struct InnerFailurePrevention {
    pub trajectory_repetition: TrajectoryMonitor,
    pub error_action_repetition: ErrorActionMonitor,
    pub loop_pseudo_termination: LoopTerminationMonitor,
}

impl InnerFailurePrevention {
    pub fn check(&mut self, state: &AgentLoopState) -> Option<FailureRisk>;
    pub fn inject_prevention(&self, risk: &FailureRisk) -> Vec<String>;
}

pub struct TrajectoryMonitor {
    pub recent_trajectories: Vec<Vec<String>>,
    pub repetition_threshold: usize,
}

impl TrajectoryMonitor {
    pub fn detect_loop(&self, current: &[String]) -> bool;
}

pub struct ErrorActionMonitor {
    pub error_history: Vec<(String, u64)>,
    pub max_retries_per_action: usize,
}

impl ErrorActionMonitor {
    pub fn check(&self, action: &str) -> bool;
}

pub struct LoopTerminationMonitor {
    pub stall_detector: StagnationDetector,
    pub max_idle_iterations: usize,
}

/// Outer layer: RCA + heuristic strategy extraction
pub struct OuterFailureConversion {
    pub rca_engine: RcaEngine,
    pub strategy_extractor: HeuristicStrategyExtractor,
}

pub struct RcaEngine;

impl RcaEngine {
    pub fn analyze(&self, trace: &[ToolTrace], final_state: &str) -> RcaResult;
}

pub struct RcaResult {
    pub root_cause: String,
    pub failure_type: FailureType,
    pub severity: f64,
    pub prevention_strategy: String,
}

pub enum FailureType {
    Planning,
    Execution,
    KnowledgeGap,
    SafetyBlocked,
    BudgetExhausted,
}

pub struct HeuristicStrategyExtractor;

impl HeuristicStrategyExtractor {
    pub fn extract(&self, rca: &RcaResult) -> Vec<String>;
}

// ============================================
// Hyper-Harness (P3)
// ============================================

/// Progressive context loading strategy
pub struct ProgressiveLoader {
    pub levels: Vec<LoadLevel>,
}

pub struct LoadLevel {
    pub level: u8,
    pub condition: LoadCondition,
    pub max_tokens: usize,
}

pub enum LoadCondition {
    Always,
    OnTaskStart,
    OnRelevance(f64),
    OnUserRequest,
}

/// Concurrent tool execution with dependency resolution
pub struct ConcurrentExecutor {
    pub dependency_graph: DependencyGraph,
    pub max_concurrent: usize,
}

impl ConcurrentExecutor {
    pub fn resolve_dependencies(&self, tools: &[String]) -> Vec<Vec<String>>;
    pub async fn execute_batch(&self, batch: Vec<ToolInvocation>) -> Vec<ToolResult>;
}

/// Four-layer security scanner
pub struct SecurityScanner {
    pub llm_review: LlmReviewLayer,
    pub policy_engine: PolicyEngineLayer,
    pub interactive_approval: ApprovalLayer,
    pub execution_sandbox: SandboxLayer,
    pub trust_classifier: TrustClassifier,
}

impl SecurityScanner {
    pub fn scan(&self, action: &Action, trust_level: TrustLevel) -> SecurityVerdict;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustLevel {
    Trusted,
    LowRisk,
    MediumRisk,
    HighRisk,
}

pub enum SecurityVerdict {
    Allow,
    Block,
    RequestApproval(String),
    SandboxOnly,
}

// ============================================
// Integration into SelfIteratingBrain
// ============================================

impl SelfIteratingBrain {
    /// Extended seal loop with full-dimension evolution
    pub fn run_seal_loop_full(
        &mut self,
        task: &str,
        validator: Option<&dyn AbsorbValidator>,
        external_reward: Option<f64>,
    ) -> Result<f64, String> {
        // Phase 1: Context evolution — select optimal context config
        if let Some(ref mut evolver) = self.context_evolver {
            let task_type = infer_task_type(task);
            let config = evolver.apply(&mut self.cortex);
            // Apply context config
        }

        // Phase 2: Execute core SEAL loop (existing)
        let result = self.run_seal_loop(task, validator, external_reward)?;

        // Phase 3: Skill evolution — mine traces, diagnose, repair
        if let Some(ref mut evolver) = self.skill_evolver {
            let traces = self.collect_tool_traces();
            evolver.mine_and_propose(&traces);
            if let Some(fb) = external_reward {
                evolver.diagnose_skills(fb);
            }
        }

        // Phase 4: Brain evolution — update policy, optimize strategies
        if let Some(ref mut evolver) = self.brain_evolver {
            evolver.evolve(&mut self.brain, external_reward);
        }

        // Phase 5: Record evolution round
        self.evolution_rounds.push(EvolutionRound {
            timestamp: Utc::now().timestamp() as u64,
            skill_delta: self.skill_evolver.as_ref().map_or(0.0, |e| e.last_delta()),
            context_delta: self.context_evolver.as_ref().map_or(0.0, |e| e.last_delta()),
            brain_delta: self.brain_evolver.as_ref().map_or(0.0, |e| e.policy.last_lr_change()),
            external_reward,
            internal_reward: result,
        });

        Ok(result)
    }
}
```

---

## 4. KnowledgeSource Registration

| Source | Core Capability | Provenance |
|--------|----------------|------------|
| OmniAgent | `full_dim_evolution`: 0.95, `skill_self_evolution`: 0.94, `context_self_evolution`: 0.92, `brain_model_evolution`: 0.85, `deep_reflexion_inner`: 0.90, `deep_reflexion_outer`: 0.88, `proactive_memory`: 0.90, `hyper_harness`: 0.82, `progressive_context`: 0.88, `concurrent_tool_exec`: 0.80, `four_layer_security`: 0.85, `sentinel_guardian`: 0.78, `multi_channel`: 0.75 | `YeQing17-2026/OmniAgent` — self-evolving agent framework with OmniEvolve |

### CapabilityVector Extension Map

```rust
// To be added in core/knowledge/sources.rs
KnowledgeSource::OmniAgent => {
    let mut cv = CapabilityVector::from_values(
        0.3, 0.4, 0.3, 0.4,     // base reasoning dimensions
        0.4, 0.5, 0.3, 0.4,     // perception dimensions
        0.7, 0.6, 0.6, 0.8, 0.75, // memory/knowledge dimensions
        0.5, 0.6, 0.5,          // execution dimensions
        0.6, 0.7, 0.5,          // adaptation dimensions
        0.85, 0.9, 0.88, 0.92,  // meta dimensions (strong: evolution)
    );
    cv.extend_named(&[
        ("full_dim_evolution".into(), 0.95),
        ("skill_self_evolution".into(), 0.94),
        ("context_self_evolution".into(), 0.92),
        ("brain_model_evolution".into(), 0.85),
        ("deep_reflexion_inner".into(), 0.90),
        ("deep_reflexion_outer".into(), 0.88),
        ("proactive_memory".into(), 0.90),
        ("hyper_harness".into(), 0.82),
        ("progressive_context".into(), 0.88),
        ("concurrent_tool_exec".into(), 0.80),
        ("four_layer_security".into(), 0.85),
        ("sentinel_guardian".into(), 0.78),
        ("multi_channel_gateway".into(), 0.75),
        ("dual_path_feedback".into(), 0.92),
        ("action_sequence_mining".into(), 0.91),
        ("preference_tracking".into(), 0.89),
    ]);
    SourceEntry {
        name: "OmniAgent",
        capability: cv,
        weight: 0.90,
    }
}
```

---

## 5. Testing Strategy

| # | Test Name | What It Tests | Expected |
|---|-----------|--------------|----------|
| 1 | `test_action_sequence_miner_empty` | Empty traces produce no proposals | `mine()` returns `vec![]` |
| 2 | `test_action_sequence_miner_pattern` | Repeated tool call sequences are detected | Returns ≥1 SkillProposal with correct steps |
| 3 | `test_skill_diagnoser_user_feedback` | Low user feedback triggers repair diagnosis | `diagnose()` returns `needs_repair: true` |
| 4 | `test_skill_diagnoser_llm_eval` | LLM eval identifying missing step | Returns `issue_type: MissingStep` |
| 5 | `test_context_evolver_progressive_load` | LayerManager loads L1 on task start, L2 on demand | L0 always loaded; L2 only when relevance > threshold |
| 6 | `test_preference_tracker_profile_update` | Recording signals updates preference profile | Profile reflects task type distribution after 10 signals |
| 7 | `test_brain_evolver_reward_learning_rate` | Positive reward trend increases learning rate | `update_learning_rate(0.9)` > `update_learning_rate(0.3)` |
| 8 | `test_deep_reflexion_inner_trajectory_loop` | Repeated trajectory detected by InnerFailurePrevention | `check()` returns `Some(FailureRisk)` after 3 identical paths |
| 9 | `test_deep_reflexion_outer_rca` | Failed execution trace produces RCA result | `rca_engine.analyze()` returns `RcaResult` with non-empty `root_cause` |
| 10 | `test_full_dimension_evolver_round_tracking` | Evolution round recorded after evolve() | `rounds.len()` increments, each round has non-zero deltas |
| 11 | `test_omniagent_knowledge_source_registration` | CapabilityVector has OmniAgent extension dims | `cv.get_named("full_dim_evolution") == Some(0.95)` |
| 12 | `test_skill_proposal_crystallization` | SkillProposal can be converted to Skill + registered | Registry contains skill with matching steps |
| 13 | `test_concurrent_executor_dependency_order` | Dependency graph correctly orders parallel batches | `resolve_dependencies()` returns independent tools in same batch |
| 14 | `test_inner_failure_prevention_context_injection` | Prevention strings are generated for detected risk | `inject_prevention()` returns non-empty prevention context |

---

## 6. Risk Assessment

| # | Risk | Impact | Likelihood | Mitigation |
|---|------|--------|------------|------------|
| 1 | **Skill evolution overfitting**: ActionSequenceMiner proposes too many low-quality skills, bloating SkillRegistry | Medium | Medium | Set `min_frequency ≥ 3` and `min_confidence ≥ 0.6`; require external reward > 0.7 before auto-registration |
| 2 | **Context evolution memory explosion**: PreferenceTracker accumulates unbounded signal history | Medium | Low | Cap `signal_history` at 1000 entries; old signals are weighted into profile then discarded |
| 3 | **Deep reflexion slows agent loop**: Inner failure prevention checks add latency to each iteration | Medium | High | Make inner checks optional (`reflexion_enabled: bool`); use async check with timeout; benchmark target < 5ms |
| 4 | **Concurrent tool execution race conditions**: Dependency resolution misses implicit ordering constraints | High | Medium | Start with sequential execution fallback; promote concurrent only after safe-dependency-annotation API is stable |
| 5 | **BrainEvolver policy oscillation**: Learning rate oscillates wildly under noisy reward signal | Medium | Low | Apply EMA smoothing (alpha=0.3) before policy updates; clamp learning rate to [0.001, 0.1] |

---

## 7. Implementation Sequence

### Phase 1: Foundation (P2 — Skill Evolution + Context Evolution)
1. Register `KnowledgeSource::OmniAgent` with capability vector
2. Implement `ActionSequenceMiner` — read `tool_traces`, cluster patterns, propose skills
3. Extend `SkillCrystallizer` with `SkillDiagnoser` + `SkillRepairer`
4. Wire skill evolution into `SelfIteratingBrain.run_seal_loop()` post-execution phase
5. Implement `LayerManager` + `PreferenceTracker` + `ContextStrategy`
6. Wire context evolution into SEAL loop pre-execution phase
7. Add `FullDimensionEvolver` as orchestrator

### Phase 2: Deep Reflexion (P2)
1. Implement `InnerFailurePrevention` (trajectory, error action, loop termination monitors)
2. Implement `OuterFailureConversion` (RCA engine + heuristic strategy extraction)
3. Wire DeepReflexion into agent loop as optional plugin

### Phase 3: Brain Evolution (P3)
1. Implement `RewardCollector` + `PolicyUpdater` + `StrategyOptimizer`
2. Connect brain evolution to capability vector updates
3. Add learning rate auto-tuning based on reward trends

### Phase 4: Hyper-Harness (P3)
1. Implement `ProgressiveLoader` — multi-level context loading
2. Implement `ConcurrentExecutor` — dependency graph + async execution
3. Implement `SecurityScanner` — 4-layer scanning (basic version)
4. Add Sentinel/Guardian role templates to AgentTeam

---

## 8. Comparison Table: OmniAgent vs NeoTrix (Post-Fusion)

| Dimension | OmniAgent | NeoTrix (Current) | NeoTrix (Post-Fusion) |
|-----------|-----------|-------------------|----------------------|
| **Skill Evolution** | Real-time auto-create + repair | Basic crystallization | Full auto-create + diagnose + repair |
| **Context Evolution** | Multi-layer + personalization | Static context window | Progressive loading + preference-adaptive |
| **Brain Evolution** | Online RL (GRPO + PRM) | CapabilityVector absorb | Policy-optimized vector evolution |
| **Reflexion** | Dual-layer (inner+outer) | Single-layer reflection | Dual-layer with RCA |
| **Safety** | 4-layer dynamic scanning | Basic sandbox | Progressive 4-layer (phased) |
| **Tool Execution** | Concurrent w/ dep resolution | Serial | Concurrent (phased) |
| **Memory** | Proactive dual-path | CortexMemory + Bank | Preference-aligned memory |
| **Capability Model** | Python-based | Rust typed vectors | Rust typed vectors + RL policy |

---

*Generated from analysis of [YeQing17-2026/OmniAgent](https://github.com/YeQing17-2026/OmniAgent) — 1.5k stars, 244 forks, GPL-3.0*
