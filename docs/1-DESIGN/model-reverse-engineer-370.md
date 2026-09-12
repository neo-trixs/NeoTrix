# Model Reverse Engineering — Cycle 370 (2026-09-12)

## 5 New AI Models/Papers → NeoTrix Domain Mapping

---

## 1. RouteMoA — Dynamic Routing without Pre-Inference
**Paper**: arXiv:2601.18130 | **Venue**: ACL 2026
**Authors**: Wang et al.

### Core Idea
Mixture-of-Agents (MoA) routing traditionally requires all models to run inference before judging. RouteMoA introduces a lightweight scorer that predicts coarse-grained performance from the query alone, narrowing candidates to a high-potential subset **without any inference**. A mixture of judges then refines scores via self/cross-assessment on existing outputs.

### Key Results
- 89.8% cost reduction, 63.6% latency reduction vs MoA baseline
- Matches or exceeds MoA accuracy across diverse tasks
- Handles large model pools (67+ models) without context overflow

### Pattern Extracted
**Pre-Inference Filtering**: A cheap model scores query complexity before routing to expensive models. This is a 2-stage filter: (1) coarse screening (zero inference), (2) refinement from partial results.

### NeoTrix Mapping

| Domain | Integration |
|--------|-------------|
| **NT-CORE (GWT)** | RouteMoA's scorer maps directly to GWT salience calculation. Add "pre-inference filter" as a GWT attention gate before model dispatch. |
| **NT-ACT (load balancer)** | Total_calls ascending rotation + RouteMoA's coarse scorer = adaptive load balancing. Route easy tasks to cheap models, hard to expensive. |
| **NT-IO (provider routing)** | Ordered Backend Router extended: add lightweight complexity scorer as routing decision layer. |
| **Axiom A1** | Cost-Aware Routing validated: RouteMoA proves 89% cost savings by not running inference on models that won't help. |

### Action Item
Integrate RouteMoA's coarse scorer concept into GWT salience. When a task arrives, score it with a tiny model first (O(100 tokens)), then route based on score + cost budget.

---

## 2. Flux Attention — Context-Aware Hybrid Attention
**Paper**: arXiv:2604.07394
**Authors**: Qiu et al.

### Core Idea
Standard attention is quadratic. Existing hybrid methods (Full Attention + Sparse Attention) use static allocation ratios. Flux Attention introduces a **Layer Router** that dynamically routes each layer to FA or SA based on input context. Layer-wise routing (not head-level) preserves contiguous memory access.

### Key Results
- 2.8x prefill speedup, 2.0x decode speedup
- Only 12 hours training on 8×A800 GPUs (parameter-efficient)
- Preserves long-context retrieval accuracy

### Pattern Extracted
**Layer-Level Dynamic Routing**: Instead of head-level sparsity (which causes load imbalance), route entire layers based on input complexity. Simple layers → sparse, complex layers → full attention.

### NeoTrix Mapping

| Domain | Integration |
|--------|-------------|
| **NT-CORE (GWT)** | GWT attention routing operates at layer level. Flux Attention validates layer-level granularity over head-level. |
| **NT-MEMORY (KVMem)** | Flux's layer-level sparsity maps to KVMem's paged KV optimization. Simple layers → compressed KV, complex layers → full KV cache. |
| **Axiom A2** | Context as Scarce Resource: Flux proves layer-level dynamic allocation saves compute while preserving accuracy. |
| **Skill Tree** | Could implement "Adaptive Attention" as a Keystone node under NT-CORE cognition branch. |

### Action Item
Design a Layer Router for NeoTrix's inference path: classify each layer's attention demand per-query, route to appropriate computation strategy.

---

## 3. Latent Action Reparameterization (LAR)
**Paper**: arXiv:2605.18597
**Authors**: Huang et al.

### Core Idea
LLM agents use long sequences of low-level textual actions, creating large decision horizons. LAR learns a compact **latent action space** where each latent action maps to multi-step semantic behavior. Planning and execution operate over abstract action representations.

### Key Results
- Significant reduction in action tokens and wall-clock inference time
- Maintains or improves task success rates across benchmarks
- Complementary to model architecture improvements

### Pattern Extracted
**Semantic Action Compression**: Replace multi-step text actions with learned latent tokens. A "write function" latent = "parse function signature → open file → write body → close". Compresses 20 tokens → 1 latent.

### NeoTrix Mapping

| Domain | Integration |
|--------|-------------|
| **NT-ACT (tool calling)** | LAR maps to tool-call macro compression. Instead of 5-step tool chains, use 1 latent action. |
| **NT-IO (PTC)** | Programmatic Tool Calling gains a "latent macro" mode: pre-compiled action sequences as single typed stubs. |
| **SEAL pipeline** | LAR's trajectory learning maps to SEAL skill crystallization: frequently-used action sequences crystallize into latent macros. |
| **Axiom A2** | Context savings: latent actions reduce token usage by ~60% per action sequence. |

### Action Item
Implement "action macros" in NT-ACT: detect repeated 3+ step tool sequences, offer to compress into single typed operations. Track compression ratio and success rate.

---

## 4. Explicit Trait Inference (ETI) for Multi-Agent Coordination
**Paper**: arXiv:2604.19278 | **Venue**: ACL 2026
**Authors**: Abdurahman et al.

### Core Idea
LLM multi-agent systems fail from goal drift, error cascades, and misaligned behaviors. ETI enables agents to infer partner characteristics along **warmth** (trust) and **competence** (skill) dimensions from interaction histories. These trait profiles guide coordination decisions.

### Key Results
- 45-77% payoff loss reduction in economic games
- 3-29% performance improvement on MultiAgentBench
- Lightweight: works with interaction history alone, no model modification

### Pattern Extracted
**Trait-Based Agent Coordination**: Each agent maintains a trait profile (warmth/competence) of its partners. Before delegating work, check the partner's competence trait. Before trusting output, check warmth trait.

### NeoTrix Mapping

| Domain | Integration |
|--------|-------------|
| **NT-ACT (orchestration)** | ETI trait profiles as agent selection criteria. When NT-ACT dispatches subtasks, weight by agent competence trait. |
| **NT-CORE (SelfModel)** | SelfModel extension: add warmth/competence tracking for external agents. Maps to DynamicParams (speed/amplitude/frequency). |
| **NT-FEEL** | Warmth dimension maps to trust emotion in EmotionLabel. Competence maps to confidence. |
| **Cross-module** | ETI validates P3 (Profile-Driven Adaptation): persistent trait profiles shape behavior across sessions. |

### Action Item
Add `AgentTraitProfile { warmth: f32, competence: f32, history: Vec<Interaction> }` to NT-ACT orchestration. Update task dispatch to consider partner traits.

---

## 5. HeteroPanacea — Disaggregated Agentic Inference
**Paper**: arXiv:2608.03741
**Authors**: Forys et al.

### Core Idea
Agentic inference (multi-turn, tool-calling) has fundamentally different workload characteristics than static inference. Prefill and decode stages exhibit different compute/memory demands. HeteroPanacea simulates **4-way disaggregation**: Prefill, Decode, Attention, FFN as separate specialized hardware units.

### Key Results
- 75% throughput increase over traditional serving
- 4-way PDAF disaggregation most consistent across model architectures
- Addresses the "agentic inference gap" where homogeneous GPUs fail

### Pattern Extracted
**Inference Stage Specialization**: Different inference stages need different hardware. Prefill = compute-bound, Decode = memory-bound, Attention = bandwidth-bound, FFN = compute-bound. Route each stage to specialized silicon.

### NeoTrix Mapping

| Domain | Integration |
|--------|-------------|
| **NT-IO (provider routing)** | Route inference stages to different backends: prefill → GPU cloud, decode → LPUs, attention → CPU offload. |
| **NT-MEMORY (KVMem)** | KVMem's paged KV is a software-level analog of decode specialization. Extend to hardware-aware tiering. |
| **NT-PHYSICAL** | For embodied agents: different inference stages map to different compute devices (edge GPU, cloud TPU, local NPU). |
| **Axiom A2** | HeteroPanacea validates context-as-scarce-resource at hardware level: disaggregation increases capacity without proportional cost. |

### Action Item
Design an inference stage router in NT-IO: classify incoming request stage requirements (prefill-heavy vs decode-heavy), route to appropriate provider/hardware tier.

---

## Cross-Paper Synthesis

### Meta-Pattern M1: 2-Stage Filtering
Both RouteMoA (pre-inference scoring) and Flux Attention (layer-level routing) apply the same principle: **cheap filter first, expensive computation second**. This is NeoTrix's GWT salience in action.

### Meta-Pattern M2: Latent Compression
LAR (latent actions) and Flux Attention (layer-level sparsity) both compress information before expensive processing. LAR compresses actions, Flux compresses attention. Both achieve 2-3x speedup.

### Meta-Pattern M3: Trait-Aware Coordination
ETI (warmth/competence traits) and HeteroPanacea (hardware specialization) both recognize that **not all agents/resources are equal**. Matching task difficulty to agent/resource capability is the key to efficiency.

### Meta-Pattern M4: Disaggregation
HeteroPanacea disaggregates inference stages. RouteMoA disaggregates routing into pre-filter + refinement. Both prove that **monolithic processing is suboptimal** — decompose and specialize.

---

## Priority Absorption Queue

| Priority | Item | Domain | Effort |
|----------|------|--------|--------|
| P0 | RouteMoA pre-inference scorer → GWT salience | NT-CORE | Medium |
| P0 | ETI trait profiles → agent dispatch | NT-ACT | Low |
| P1 | LAR latent actions → tool-call macros | NT-ACT + NT-IO | High |
| P1 | Flux Attention layer router → inference path | NT-IO | Medium |
| P2 | HeteroPanacea stage routing → provider selection | NT-IO | High |
