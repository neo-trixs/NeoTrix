# Model Reverse Engineer — Cycle 414

> **Date**: 2026-09-12
> **Sources**: arXiv (2603-2607), ACL 2026, AAAI 2026, ICML 2026, ICLR 2026
> **Prior baseline**: cycles 318-413 (no duplicates)
> **Focus**: Efficient inference, attention mechanisms, agent coordination, reasoning collapse

---

## Paper 1: GLIDE — Guided Layerwise Hybrid Attention for Efficient LLM Inference

| Field | Value |
|-------|-------|
| **Paper** | [arXiv:2607.24788](https://arxiv.org/abs/2607.24788) |
| **Date** | Jun 26, 2026 |
| **Tested on** | Long-context generation benchmarks |
| **Key metric** | Reduced end-to-end latency for long-context generation without quality compromise |

### Core Idea

GLIDE exploits **layer-wise heterogeneity** in attention sensitivity: early layers are highly sensitive to softmax removal, while deeper layers tolerate aggressive replacement by linear recurrence. Rather than uniform hybrid attention, GLIDE non-uniformly compresses the softmax footprint across the model.

### Architecture Analysis

| Component | Detail |
|-----------|--------|
| **Mechanism** | Layer-wise adaptive: each layer balances linear recurrence + variable-sized softmax window |
| **Key Insight** | Early layers need softmax (high sensitivity); deep layers are redundant (tolerate linear) |
| **KV Savings** | Non-uniform compression — aggressive in deep layers, conservative in early layers |
| **Training** | Layer-wise adaptive mechanism; variable window sizes per layer |
| **Scaling** | Maintains quality where most vital (early layers), compresses where redundant (deep) |

### Why This Matters

This is the first method to recognize that **attention sensitivity is not uniform across layers**. Uniform hybrid methods (e.g., sliding window everywhere) waste computation on layers that need it and under-compress layers that don't. GLIDE's layer-aware approach is a fundamental shift.

### NeoTrix Domain Mapping

| Domain | Integration | Priority |
|--------|-------------|----------|
| **NT-CORE** (E8 引导者) | Map GLIDE's layer sensitivity to GWT salience layers — early layers = high-salience broadcast, deep layers = compressed background | P0 |
| **NT-MEMORY** (知识守护者) | Tiered KV cache: early-layer KV = hot (full softmax), deep-layer KV = cold (linear recurrence + compressed window) | P0 |
| **NT-IO** (界面使徒) | Implement GLIDE's layer-wise router in inference engine; dynamic window sizing per layer | P1 |
| **NT-MIND** (进化工匠) | Evolve layer sensitivity profiles per task domain — code tasks may have different optimal layer compression ratios | P2 |

### NeoTrix Integration Point

```rust
// NT-CORE + NT-MEMORY: Layer-aware KV cache management
struct GlideRouter {
    layer_configs: Vec<LayerConfig>,  // per-layer softmax/linear ratio
}

struct LayerConfig {
    softmax_window: usize,  // variable per layer
    linear_ratio: f32,      // fraction of attention using linear recurrence
    sensitivity: f32,       // learned layer sensitivity score
}

impl GlideRouter {
    fn route(&self, layer: usize, kv_cache: &KvCache) -> AttentionBudget {
        let config = &self.layer_configs[layer];
        AttentionBudget {
            softmax_tokens: config.softmax_window,
            linear_tokens: kv_cache.total_len() - config.softmax_window,
            // Early layers: high softmax budget
            // Deep layers: high linear budget
        }
    }
}
```

---

## Paper 2: SparDA — Sparse Decoupled Attention with Forecast Projections

| Field | Value |
|-------|-------|
| **Paper** | [arXiv:2606.04511](https://arxiv.org/abs/2606.04511) |
| **Date** | Jun 3, 2026 |
| **Tested on** | Sparse-pretrained 8B models |
| **Key metric** | 1.25× prefill speedup, 1.7× decode speedup, up to 5.3× decode throughput |

### Core Idea

SparDA introduces a **fourth per-layer projection** — Forecast — alongside Query, Key, and Value. The Forecast projection predicts which KV blocks the next layer will need, enabling **lookahead selection** that overlaps CPU-to-GPU prefetch with current-layer execution. Decoupled from attention query, so GQA uses one Forecast head per GQA group.

### Architecture Analysis

| Component | Detail |
|-----------|--------|
| **New Projection** | Forecast (4th per-layer projection) predicts next-layer KV block needs |
| **Lookahead Selection** | CPU-to-GPU prefetch overlaps with current-layer execution |
| **GQA Efficiency** | One Forecast head per GQA group (vs multi-head selector) |
| **Parameter Overhead** | <0.5% additional parameters |
| **Training** | Only Forecast projections trained; match original selector's attention distribution |
| **Speedup** | 1.25× prefill, 1.7× decode, 5.3× throughput vs non-offload baseline |

### Why This Matters

The key insight is **decoupling selection from attention**: instead of using the same Q for both selecting tokens and attending to them, SparDA uses a dedicated lightweight projection to predict future needs. This enables prefetching without the O(T²) cost of the selection step itself.

### NeoTrix Domain Mapping

| Domain | Integration | Priority |
|--------|-------------|----------|
| **NT-CORE** (E8 引导者) | Forecast projection = GWT anticipatory attention — predict what to attend to before attention computation | P0 |
| **NT-MEMORY** (知识守护者) | KV block prefetching — predict which memory blocks will be needed, preload from cold storage | P0 |
| **NT-PHYSICAL** (具身骨架) | CPU↔GPU overlap scheduling — prefetch sensor data while processing current sensor batch | P1 |
| **NT-WORLD** (虚空探索者) | Predictive content fetching — forecast which web pages/APIs will be needed next based on current reasoning | P1 |

---

## Paper 3: RAGEN-2 — Reasoning Collapse in Agentic RL

| Field | Value |
|-------|-------|
| **Paper** | [arXiv:2604.06268](https://arxiv.org/abs/2604.06268) |
| **Date** | Apr 7, 2026 (ICML 2026 Oral) |
| **Tested on** | Planning (Sokoban), math reasoning (MetaMathQA), web navigation (WebShop), code execution |
| **Key metric** | Mutual Information (MI) correlates with performance 3-5× more strongly than entropy |

### Core Idea

RAGEN-2 identifies **template collapse**: a failure mode where RL-trained agents produce reasoning that is diverse within each input (high entropy) but input-agnostic (low mutual information). The model learns to produce fluent, templated reasoning that ignores the actual input — like a student who memorizes a fill-in-the-blank essay structure.

### Architecture Analysis

| Component | Detail |
|-----------|--------|
| **Decomposition** | Reasoning = H(Z|X) (within-input diversity) + I(X;Z) (cross-input distinguishability) |
| **Failure Mode** | Template Collapse: high H(Z|X), low I(X;Z) — diverse-looking but generic |
| **Diagnostic** | MI proxy: treat each reasoning trace as query, retrieve source X from minibatch |
| **Root Cause** | Low reward variance → weak task gradients → input-agnostic regularizers dominate |
| **Mitigation** | SNR-Aware Filtering: prioritize high-variance prompts per iteration |
| **Correlation** | MI correlates with performance 3-5× more strongly than entropy |

### Why This Matters

This is the first paper to **formally decompose reasoning quality** into within-input diversity and cross-input dependence. Entropy monitoring (standard practice) is blind to template collapse because it only measures diversity within a single input. MI captures whether reasoning actually adapts to different inputs.

### NeoTrix Domain Mapping

| Domain | Integration | Priority |
|--------|-------------|----------|
| **NT-CORE** (E8 引导者) | Template collapse = E8 hexagram degeneration — reasoning patterns become input-agnostic | P0 |
| **NT-MIND** (进化工匠) | SNR-Aware Filtering for SEAL pipeline — prioritize high-signal training examples | P0 |
| **NT-REPAIR** (自愈工程师) | Detect template collapse via MI monitoring — alert when reasoning becomes templated | P1 |
| **NT-GOVERNANCE** (架构仲裁者) | Reasoning quality gates — MI threshold as production readiness criterion | P1 |

### NeoTrix Integration Point

```rust
// NT-MIND: SNR-Aware training filter
struct SnrAwareFilter {
    min_reward_variance: f32,  // threshold for signal strength
}

impl SnrAwareFilter {
    fn filter_batch(&self, batch: &[TrainingExample]) -> Vec<TrainingExample> {
        batch.iter()
            .filter(|ex| {
                let reward_variance = ex.rewards.iter().map(|r| r).variance();
                reward_variance >= self.min_reward_variance
            })
            .cloned()
            .collect()
        // High-variance examples have strong task signal
        // Low-variance examples are filtered out (risk of template collapse)
    }
}

// NT-REPAIR: MI-based reasoning health monitor
fn check_template_collapse(
    traces: &[ReasoningTrace],
    inputs: &[Input],
) -> CollapseDiagnosis {
    let mi = mutual_information(traces, inputs);
    let entropy = conditional_entropy(traces, inputs);

    if entropy > HIGH_THRESHOLD && mi < LOW_THRESHOLD {
        CollapseDiagnosis::TemplateCollapse { severity: 1.0 - mi }
    } else {
        CollapseDiagnosis::Healthy
    }
}
```

---

## Paper 4: Adaptive Theory of Mind for Multi-Agent Coordination (A-ToM)

| Field | Value |
|-------|-------|
| **Paper** | [arXiv:2603.16264](https://arxiv.org/abs/2603.16264) |
| **Date** | Mar 17, 2026 (AAAI 2026) |
| **Tested on** | Repeated matrix game, grid navigation, Overcooked |
| **Key metric** | A-ToM outperforms fixed-order ToM agents by 15-25% in coordination tasks |

### Core Idea

Higher-order Theory of Mind (ToM) — reasoning about others' reasoning about your reasoning — is not always better. **Misaligned ToM orders** between agents cause either insufficient or excessive reasoning about others. A-ToM agents **adapt their ToM depth** based on partner behavior, aligning their reasoning depth dynamically.

### Architecture Analysis

| Component | Detail |
|-----------|--------|
| **ToM Orders** | ToM-0 (no modeling), ToM-1 (model beliefs), ToM-2 (model beliefs about beliefs) |
| **Misalignment Problem** | ToM-2 agent interacting with ToM-0 agent over-reasons; ToM-0 with ToM-2 under-reasons |
| **Adaptation Mechanism** | Estimate partner's ToM order from interaction history |
| **Prediction** | Use estimated ToM order to predict partner's action |
| **Coordination** | Align own ToM order with partner's for successful coordination |
| **Generalizability** | Works without task-specific training; applicable to non-LLM agents |

### Why This Matters

Previous work assumed deeper ToM is always better. This paper proves the opposite: **ToM depth must match the partner**. A ToM-2 agent coordinating with a ToM-0 agent wastes computation reasoning about beliefs that the partner doesn't model. The key is alignment, not depth.

### NeoTrix Domain Mapping

| Domain | Integration | Priority |
|--------|-------------|----------|
| **NT-CORE** (E8 引导者) | GWT salience for ToM depth — adjust reasoning depth based on partner sophistication | P0 |
| **NT-ACT** (行动执行者) | Adaptive agent coordination — detect partner's reasoning level, match depth | P0 |
| **NT-FEEL** (情感中枢) | Social emotion modeling — partner's ToM depth affects trust and coordination quality | P1 |
| **NT-GOVERNANCE** (架构仲裁者) | Multi-agent governance — different agents may need different ToM depths for different tasks | P1 |

---

## Paper 5: RL Conductor — Learning to Orchestrate Agents in Natural Language

| Field | Value |
|-------|-------|
| **Paper** | [arXiv:2512.04388](https://arxiv.org/abs/2512.04388) |
| **Date** | May 2026 (ICLR 2026) |
| **Tested on** | Multi-agent coordination, complex problem decomposition |
| **Key metric** | Recursive topology enables new tunable axis of inference-time scaling |

### Core Idea

A **Conductor LLM** trained with RL (GRPO) to dynamically divide problems, delegate subtasks, and design communication topologies for worker LLM agents. The Conductor outputs full agentic workflows: which agent gets which instruction, and what each agent can see of the others. Crucially, the Conductor can **specify itself as a worker**, creating recursive topologies.

### Architecture Analysis

| Component | Detail |
|-----------|--------|
| **Conductor** | RL-trained LLM that outputs workflow steps (instruction + agent + visibility) |
| **Workers** | More capable LLMs that execute subtasks |
| **Recursive Topology** | Conductor can instantiate itself as a worker, enabling hierarchical delegation |
| **Communication Design** | Conductor defines which agents can see which other agents' outputs |
| **Training** | GRPO with rewards for task completion + communication efficiency |
| **Scaling Axis** | Recursive depth as new inference-time scaling dimension |

### Why This Matters

This is the first framework where **the orchestrator is itself an LLM** that can be recursive. Traditional multi-agent systems have fixed coordinator-worker relationships. The RL Conductor discovers communication topologies and delegation strategies through training, and the recursive capability enables hierarchical decomposition that scales with problem complexity.

### NeoTrix Domain Mapping

| Domain | Integration | Priority |
|--------|-------------|----------|
| **NT-CORE** (E8 引导者) | Conductor = E8 hexagram decision engine — route tasks based on problem structure | P0 |
| **NT-ACT** (行动执行者) | Recursive agent delegation — Conductor can spawn sub-conductors for complex tasks | P0 |
| **NT-GOVERNANCE** (架构仲裁者) | Communication topology design — control information flow between agents | P1 |
| **NT-MIND** (进化工匠) | Evolve conductor policies per domain — different task types need different delegation patterns | P1 |

### NeoTrix Integration Point

```rust
// NT-CORE + NT-ACT: Recursive conductor pattern
struct ConductorTopology {
    instruction: String,
    assigned_agent: AgentId,
    visibility: Vec<AgentId>,  // which agents this agent can observe
}

impl Conductor {
    fn plan_workflow(&self, task: &Task) -> Vec<ConductorTopology> {
        // RL-trained policy outputs:
        // 1. Task decomposition
        // 2. Agent assignment
        // 3. Communication topology
        // 4. Visibility constraints
        self.policy.generate(task)
    }

    fn recursive_plan(&self, task: &Task, depth: usize) -> Workflow {
        if depth == 0 || task.is_simple() {
            // Leaf: assign to worker
            Workflow::Leaf(self.plan_single(task))
        } else {
            // Recursive: conductor delegates to sub-conductors
            let subtasks = self.decompose(task);
            let subtopologies = subtasks.iter()
                .map(|st| self.recursive_plan(st, depth - 1))
                .collect();
            Workflow::Branch(subtopologies)
        }
    }
}
```

---

## Cross-Paper Synthesis

### Pattern 1: Layer-Aware Compression

| Paper | Mechanism | NeoTrix Integration |
|-------|-----------|-------------------|
| GLIDE | Non-uniform softmax compression across layers | GWT salience layers: early=high, deep=compressed |
| SparDA | Forecast projection for lookahead KV selection | Predictive memory prefetching |

**Synthesis**: Both methods recognize that not all layers/tokens need equal attention. GLIDE works at the architectural level (layer-wise), SparDA at the projection level (forecast). NeoTrix can combine both: layer-aware compression (GLIDE) with predictive prefetching (SparDA).

### Pattern 2: Reasoning Quality Monitoring

| Paper | Metric | NeoTrix Integration |
|-------|--------|-------------------|
| RAGEN-2 | Mutual Information (MI) for template collapse detection | SEAL pipeline health monitoring |
| A-ToM | ToM order alignment for coordination quality | Multi-agent coordination quality gates |

**Synthesis**: Both papers address reasoning quality degradation through different lenses. RAGEN-2 monitors individual reasoning quality (MI vs entropy), A-ToM monitors interpersonal coordination quality (ToM alignment). NeoTrix needs both: individual reasoning health + coordination health.

### Pattern 3: Recursive Agent Architecture

| Paper | Mechanism | NeoTrix Integration |
|-------|-----------|-------------------|
| RL Conductor | Recursive topology — conductor as worker | Hierarchical agent delegation |
| A-ToM | Adaptive ToM depth — match partner's reasoning level | Dynamic agent sophistication matching |

**Synthesis**: RL Conductor shows that recursion enables scaling; A-ToM shows that recursion depth must be adaptive. NeoTrix should implement recursive delegation with adaptive depth — not fixed-depth recursion, but depth that matches problem complexity and partner capability.

---

## Signal Strength

| Signal | Interpretation |
|--------|---------------|
| **GLIDE + SparDA** | Layer-aware and projection-aware attention compression are converging — the future is non-uniform, predictive KV management |
| **RAGEN-2 (ICML Oral)** | Template collapse is a fundamental failure mode of agent RL — MI monitoring will become standard practice |
| **A-ToM (AAAI)** | ToM alignment > ToM depth — multi-agent systems need matching, not maximizing, reasoning depth |
| **RL Conductor (ICLR)** | Recursive agent orchestration with RL is production-viable — new scaling dimension beyond model size |
