# Cognition Modules (nt_core/nt_mind)

> L5 Cognition — Reasoning Brain, Knowledge Engine, Evolution, Consciousness

---

## Overview

The cognition layer implements NeoTrix's reasoning, knowledge management, and self-evolution capabilities. Architecture: `CapabilityVector + ReasoningBrain + SelfIteratingBrain + SelfEvolver`.

```text
nt_mind
├── nt_mind/             — Unified reasoning brain
│   ├── core             — CapabilityVector, KnowledgeSource
│   ├── reason/          — ReasoningEngine, AttentionRouter
│   ├── knowledge/       — KnowledgeEngine, CortexMemory
│   ├── evolution/       — SelfEvolver, AutoCrystallizer
│   ├── consciousness/   — CuriosityDrive, PredictiveCortex
│   ├── infrastructure/  — CodeGraph, CodeReview
│   └── experience_tree/ — ExperienceEngine
├── nt_cognition_facade  — L5 external facade
├── nt_core_consciousness_core — ConsciousnessCore
├── nt_core_consciousness_tree — ConsciousnessTree
├── nt_core_gwt          — Global Workspace Theory
├── nt_core_context      — Context management
├── nt_core_reasoning    — Reasoning pipeline
├── nt_core_dispatch     — Task dispatch
├── nt_core_model_gateway — Model gateway
└── nt_core_semantic_router — Semantic routing
```

---

## Key Types

### `ReasoningBrain`

Core reasoning brain with self-iteration capability.

```rust
pub struct ReasoningBrain {
    capability_vector: CapabilityVector,
    knowledge: KnowledgeEngine,
    memory: ReasoningBank,
    stats: BrainStats,
}

pub struct CapabilityVector {
    pub dimensions: Vec<f64>,
    pub labels: Vec<String>,
    pub version: u32,
}

pub struct BrainStats {
    pub iterations: u64,
    pub total_tokens: u64,
    pub avg_latency_ms: f64,
    pub success_rate: f64,
    pub memory_usage: usize,
}
```

**Public Methods:**

```rust
impl ReasoningBrain {
    pub fn new() -> Self;
    pub fn reason(&mut self, input: &str) -> Result<ReasoningOutput>;
    pub fn iterate(&mut self) -> IterationResult;
    pub fn stats(&self) -> &BrainStats;
    pub fn reset(&mut self);
}
```

---

### `SelfIteratingBrain`

Extended brain with self-improvement loops.

```rust
pub struct SelfIteratingBrain {
    brain: ReasoningBrain,
    iteration_count: usize,
    evaluation_history: Vec<EvaluationRecord>,
    evo_stats: EvoStats,
}

pub struct EvaluationRecord {
    pub iteration: usize,
    pub input: String,
    pub output: String,
    pub score: f64,
    pub improvement: f64,
    pub timestamp: DateTime<Utc>,
}

pub struct EvoStats {
    pub total_iterations: u64,
    pub avg_improvement: f64,
    pub best_score: f64,
    pub stagnation_count: usize,
}
```

**Public Methods:**

```rust
impl SelfIteratingBrain {
    pub fn new(brain: ReasoningBrain) -> Self;
    pub fn iterate(&mut self) -> IterationResult;
    pub fn evaluate(&self) -> EvaluationRecord;
    pub fn should_stop(&self) -> bool;
    pub fn stats(&self) -> &EvoStats;
    pub fn history(&self) -> &[EvaluationRecord];
}
```

---

### `SelfEvolver`

External information self-evolution (S-06).

```rust
pub struct SelfEvolver {
    knowledge_sources: Vec<Box<dyn KnowledgeSource>>,
    evolution_history: Vec<EvolutionEvent>,
    config: EvolverConfig,
}

pub struct EvolverConfig {
    pub max_sources: usize,
    pub min_confidence: f64,
    pub auto_absorb: bool,
    pub crystallize: bool,
}

pub struct EvolutionEvent {
    pub source: String,
    pub event_type: EventType,
    pub impact: f64,
    pub timestamp: DateTime<Utc>,
}

pub enum EventType {
    KnowledgeAbsorbed,
    SkillCrystallized,
    CapabilityExpanded,
    ContradictionResolved,
}
```

**Public Methods:**

```rust
impl SelfEvolver {
    pub fn new(config: EvolverConfig) -> Self;
    pub fn evolve(&mut self) -> Result<Vec<EvolutionEvent>>;
    pub fn add_source(&mut self, source: Box<dyn KnowledgeSource>);
    pub fn absorb(&mut self, url: &str) -> Result<EvolutionEvent>;
    pub fn crystallize(&mut self) -> Result<Option<SkillCrystal>>;
    pub fn history(&self) -> &[EvolutionEvent];
}
```

---

### `AttentionRouter`

GWT-based attention routing with HyperCube recall.

```rust
pub struct AttentionRouter {
    workspace: GlobalWorkspace,
    hypercube: KnowledgeHyperCube,
    attention_weights: Vec<f64>,
}

pub struct GlobalWorkspace {
    pub active_context: Vec<ContextItem>,
    pub broadcast: Vec<Broadcast>,
    pub competition: Vec<Competition>,
}

pub struct ContextItem {
    pub id: String,
    pub content: String,
    pub salience: f64,
    pub source: String,
}
```

**Public Methods:**

```rust
impl AttentionRouter {
    pub fn new(workspace: GlobalWorkspace, hypercube: KnowledgeHyperCube) -> Self;
    pub fn route(&self, input: &str) -> Vec<ContextItem>;
    pub fn broadcast(&mut self, content: &str, origin: &str);
    pub fn compete(&mut self, candidates: Vec<String>) -> Option<String>;
    pub fn attention_weights(&self) -> &[f64];
}
```

---

### `ConsciousnessBridge`

HC-06: GWT attention router <-> SEAL loop bridge.

```rust
pub struct ConsciousnessBridge {
    gwt: AttentionRouter,
    seal: SealLoop,
    bridge_state: BridgeState,
}

pub struct BridgeState {
    pub active: bool,
    pub last_sync: DateTime<Utc>,
    pub sync_count: u64,
}
```

**Public Methods:**

```rust
impl ConsciousnessBridge {
    pub fn new(gwt: AttentionRouter, seal: SealLoop) -> Self;
    pub fn sync(&mut self) -> Result<SyncResult>;
    pub fn state(&self) -> &BridgeState;
    pub fn activate(&mut self);
    pub fn deactivate(&mut self);
}
```

---

### `ReasoningEngine`

Multi-method reasoning engine.

```rust
pub struct ReasoningEngine {
    methods: Vec<ReasoningMethod>,
    traces: Vec<ReasoningTrace>,
}

pub enum ReasoningMethod {
    ChainOfThought,
    TreeOfThought,
    Reflection,
    Analogy,
    Counterfactual,
}

pub struct ReasoningTrace {
    pub method: ReasoningMethod,
    pub input: String,
    pub steps: Vec<ReasoningStep>,
    pub output: String,
    pub confidence: f64,
    pub duration: Duration,
}
```

**Public Methods:**

```rust
impl ReasoningEngine {
    pub fn new() -> Self;
    pub fn reason(&mut self, input: &str, method: ReasoningMethod) -> Result<ReasoningTrace>;
    pub fn reason_multi(&mut self, input: &str) -> Vec<ReasoningTrace>;
    pub fn best_trace(&self) -> Option<&ReasoningTrace>;
    pub fn traces(&self) -> &[ReasoningTrace];
}
```

---

## Subsystems

| Module | Description |
|--------|-------------|
| `nt_consciousness` | GWT, IIT-phi, consciousness tree |
| `nt_core_dispatch` | Task dispatch and orchestration |
| `nt_core_model_gateway` | Unified model gateway with cost-aware routing |
| `nt_core_semantic_router` | Confidence-based semantic routing |
| `nt_core_hybrid_search` | Hybrid code search retriever |
| `nt_core_multi_agent` | Multi-agent coordination |
| `nt_resonator_network` | VSA Resonator Network (Frady et al., 2020) |
| `nt_decision_engine` | Multi-layer decision engine (Stimulus->GOAP->BT->Utility) |

---

## Experience Tree

Five-stage absorption protocol: snapshot -> distill -> classify -> persist -> feedback.

```rust
pub struct ExperienceEngine {
    entries: Vec<ExperienceEntry>,
}

pub struct ExperienceEntry {
    pub id: String,
    pub domain: Domain,
    pub source: Source,
    pub content: String,
    pub metadata: HashMap<String, Value>,
}

pub enum Domain { Code, System, Architecture, Business, Research }
pub enum Source { Session, External, Inferred }
pub enum EntryType { Snapshot, Distill, Classify, Persist, Feedback }
```

**Public Methods:**

```rust
impl ExperienceEngine {
    pub fn new() -> Self;
    pub fn snapshot(&mut self, data: &str) -> Result<ExperienceEntry>;
    pub fn distill(&self, entry: &ExperienceEntry) -> Result<DistillResult>;
    pub fn classify(&self, entry: &ExperienceEntry) -> Result<ClassifyResult>;
    pub fn persist(&mut self, entry: &ExperienceEntry) -> Result<AbsorptionResult>;
    pub fn query(&self, query: &ExperienceQuery) -> Vec<ExperienceEntry>;
}
```

---

## Related Modules

| Module | Description |
|--------|-------------|
| `nt_core_panic_recovery` | Panic recovery |
| `nt_core_hive` | Hive coordination protocol |
| `nt_core_byoa` | Bring Your Own Agent |
| `nt_core_agent_circuit_breaker` | Agent behavior control |
| `nt_core_quantum_fusion` | Quantum fusion processing |
| `nt_core_scoring_substrate` | Scoring substrate |
| `nt_core_sae` | Sparse Autoencoder |
| `nt_core_arch_diagram` | Architecture diagram generation |
