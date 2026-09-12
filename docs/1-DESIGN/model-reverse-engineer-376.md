# Model Reverse Engineering — Cycle 376

**Date**: 2026-09-12
**Focus**: Recent papers on efficient inference, attention, agent coordination, self-evolution
**Sources**: arXiv (2026), ICML 2026, ACL 2026, ICLR 2026

---

## Paper 1: Token Sparse Attention — Interleaved Token Selection for Long-Context

**Paper**: [arXiv:2602.03216](https://arxiv.org/abs/2602.03216) (Feb 2026, ICML 2026)
**Authors**: Jo et al.

### Core Idea
Quadratic attention complexity is the central bottleneck for long-context LLMs. Existing methods either use structured sparse patterns or permanently evict tokens at specific layers — both retain irrelevant tokens or make irreversible early decisions. Token Sparse Attention introduces **dynamic token-level sparsification** that compresses per-head Q/K/V to a reduced token set during attention, then decompresses the output back to the original sequence. Token information can be reconsidered in subsequent layers.

### Key Mechanism
- **Dynamic per-head token selection**: Each attention head independently selects which tokens to attend to
- **Compress-decompress cycle**: Q/K/V compressed during attention, output decompressed back to full sequence
- **Interleaved across layers**: Tokens evicted in one layer can be reconsidered in the next
- **Compatible with Flash Attention**: Fully composable with existing dense attention implementations
- **Per-head granularity**: Different heads select different token subsets

### Results
- **3.23× attention speedup** at 128K context
- **<1% accuracy degradation**
- Composable with existing sparse attention kernels
- New design point at intersection of token selection and sparse attention

### NeoTrix Mapping

| Component | NeoTrix Domain | Pattern |
|-----------|---------------|---------|
| Dynamic Token Selection | NT-CORE (GWT) | Per-head attention gating based on relevance |
| Compress-Decompress | NT-MEMORY (KV cache) | Selective KV cache with re-admission |
| Interleaved Reconsideration | NT-MIND (self-correction) | Second-chance token admission across layers |
| Flash Attention Compat | NT-IO (inference) | Hardware-friendly sparse patterns |

**Key Insight for NeoTrix**: The GWT salience mechanism operates at the task/module level. Token Sparse Attention shows the same principle at the token level — each attention head acts as an independent "salience router" selecting which tokens matter. The interleaved reconsideration pattern (evicted tokens can return) maps to GWT's ability to re-attend to previously ignored signals when context changes.

**Integration Point**: The `kv_cache_optimizer.rs` could implement token-level sparsification with re-admission. Instead of hard eviction, maintain a "second chance" buffer where evicted tokens can be reconsidered in later layers — mirroring how GWT can re-allocate attention to previously low-salience signals.

---

## Paper 2: Sheaf-ADMM — Multi-Agent Coordination via Algebraic Topology

**Paper**: [arXiv:2605.31005](https://arxiv.org/abs/2605.31005) (May 2026, ICML 2026)
**Authors**: Seely, Cupiał, Jones

### Core Idea
Multi-agent coordination requires agents with individually insufficient local views to produce correct global outputs. Sheaf-ADMM decomposes inputs into overlapping local views, each processed by an agent solving a convex subparameterized by a neural encoder. Agents coordinate through ADMM with inter-agent constraints specified by a **cellular sheaf** — a mathematical structure from algebraic topology that specifies which aspects of neighboring solutions must agree.

### Key Mechanism
- **Cellular sheaf constraints**: Mathematical structure specifying agreement requirements between agents
- **ADMM coordination**: Alternating Direction Method of Multipliers with sheaf-specified inter-agent constraints
- **Overlapping local views**: Each agent sees a partial view; sheaf defines how views must agree
- **Heterogeneous consensus**: Different aspects can have different agreement requirements
- **Transparent dynamics**: ADMM exposes primal, consensus, and dual state variables — directly analyzable and intervenable

### Results
- **Improved robustness to distribution shifts** on MNIST vs standard CNN
- **Higher solve rates** on Sudoku vs parameter-matched message-passing baselines
- **Analyzeable coordination**: Primal/consensus/dual variables enable direct inspection of coordination dynamics
- Agents with individually insufficient views learn to coordinate

### NeoTrix Mapping

| Component | NeoTrix Domain | Pattern |
|-----------|---------------|---------|
| Cellular Sheaf | NT-GOVERNANCE (policy constraints) | Mathematical structure for inter-domain agreement |
| ADMM Coordination | NT-ACT (multi-domain coordination) | Iterative consensus with decomposed subproblems |
| Overlapping Views | NT-CORE (GWT) | Partial perception with consensus constraints |
| Transparent Variables | NT-SHIELD (auditability) | Inspectable coordination state |
| Heterogeneous Consensus | NT-GOVERNANCE (domain policies) | Different domains have different agreement rules |

**Key Insight for NeoTrix**: The 7 NeoTrex domains (NT-CORE, NT-MIND, NT-MEMORY, etc.) each have partial views of system state. The cellular sheaf formalism provides a mathematically rigorous way to specify which aspects of neighboring domain states must agree — not "all domains agree on everything" but "NT-CORE's reasoning output must be consistent with NT-MEMORY's knowledge state, but not necessarily with NT-PHYSICAL's sensor readings."

**Integration Point**: The ConsciousnessTree's cross-domain health monitoring could adopt sheaf-theoretic constraints. Instead of ad-hoc consistency checks, define formal sheaf constraints between domain pairs: "NT-MIND's evolution velocity must agree with NT-CORE's capability assessment" (high coupling) vs "NT-FEEL's emotion state must be loosely correlated with NT-PHYSICAL's power level" (low coupling). The ADMM structure exposes exactly where consensus breaks down.

---

## Paper 3: SparDA — Sparse Decoupled Attention with Forecast Projections

**Paper**: [arXiv:2606.04511](https://arxiv.org/abs/2606.04511) (Jun 2026)
**Authors**: Fu et al.

### Core Idea
Sparse attention reduces compute but KV cache still grows with sequence length, and offloading to CPU introduces PCIe bottleneck. The sparse selection step itself retains O(T²) complexity. SparDA introduces a **fourth per-layer projection (Forecast)** alongside Q/K/V. The Forecast predicts which KV blocks the next layer will need, enabling lookahead selection that overlaps CPU-to-GPU prefetch with current-layer execution.

### Key Mechanism
- **Forecast projection**: Fourth per-layer projection that predicts next-layer KV needs
- **Lookahead selection**: Prefetch predicted KV blocks while computing current layer
- **Decoupled from attention query**: Forecast is independent of the attention query, reducing selection overhead
- **GQA-compatible**: One Forecast head per GQA group (not per attention head)
- **Minimal overhead**: <0.5% additional parameters
- **Only Forecast projections trained**: Lightweight training to match original selector's attention distribution

### Results
- **1.25× prefill speedup**, **1.7× decode speedup** over sparse-attention offload baseline
- **5.3× higher decode throughput** by enabling larger batch sizes
- Matches or slightly improves accuracy
- <0.5% parameter overhead

### NeoTrix Mapping

| Component | NeoTrix Domain | Pattern |
|-----------|---------------|---------|
| Forecast Projection | NT-CORE (predictive attention) | Predict which knowledge/modules will be needed next |
| Lookahead Prefetch | NT-MEMORY (predictive cache) | Pre-load knowledge before it's requested |
| Decoupled Selection | NT-IO (async I/O) | Selection logic independent of computation |
| GQA Compatibility | NT-IO (model efficiency) | Resource-efficient prediction per group |

**Key Insight for NeoTrix**: The GWT attention mechanism currently reacts to salience — it allocates attention after determining what's important. SparDA's Forecast projection shows a **predictive** alternative: predict what the next layer will need, then prefetch. This maps to predictive knowledge loading — before NT-CORE needs certain KB knowledge, NT-MEMORY prefetches it based on the current reasoning trajectory.

**Integration Point**: The `kv_cache_optimizer.rs` could implement a Forecast-style predictive prefetch. When processing layer N, predict which KV blocks layer N+1 will need and overlap the prefetch with current-layer computation. For NeoTrix's KB, this means predicting which knowledge nodes the next reasoning step will query and pre-loading them.

---

## Paper 4: OrgAgent — Hierarchical Multi-Agent Organization

**Paper**: [arXiv:2604.01020](https://arxiv.org/abs/2604.01020) (Apr 2026)
**Authors**: Wang et al.

### Core Idea
How to organize multiple LLM agents? Flat collaboration wastes tokens and causes coordination failures. OrgAgent introduces a **company-style hierarchical framework** with three layers: governance (planning/resource allocation), execution (task solving/review), and compliance (final answer control).

### Key Mechanism
- **Three-layer hierarchy**:
  - **Governance layer**: Planning, resource allocation, strategy
  - **Execution layer**: Task solving, review, iteration
  - **Compliance layer**: Final answer validation, quality control
- **Controlled information flow**: Information flows through hierarchy, not all-to-all
- **Layered verification**: Each layer verifies the layer below
- **Stable skill assignment**: Hierarchy enables consistent role specialization

### Results
- **102.73% performance improvement** over flat multi-agent (GPT-OSS-120B on SQuAD 2.0)
- **74.52% token reduction** vs flat collaboration
- Hierarchy helps most when tasks benefit from stable skill assignment and controlled information flow
- Organizational structure shapes effectiveness, cost, AND coordination behavior

### NeoTrix Mapping

| Component | NeoTrix Domain | Pattern |
|-----------|---------------|---------|
| Governance Layer | NT-GOVERNANCE (policy) | Strategic planning and resource allocation |
| Execution Layer | NT-ACT (task execution) | Task solving with review loops |
| Compliance Layer | NT-SHIELD (validation) | Final quality control and safety checks |
| Controlled Information Flow | NT-CORE (GWT) | Selective information routing through hierarchy |
| Stable Skill Assignment | NT-MIND (capability mapping) | Consistent role specialization |

**Key Insight for NeoTrix**: NeoTrix's 7 domains already form a natural hierarchy — but the hierarchy is implicit. OrgAgent shows that **explicit** hierarchical organization with controlled information flow reduces token cost by 74% while improving performance. The three layers map to NeoTrix's architecture: L6 Meta (governance), L5 Cognition + L4 Emotion (execution), L3-L1 (compliance/verification).

**Integration Point**: The ConsciousnessTree's domain coordination could adopt explicit three-layer hierarchy. Governance decisions (which domain gets attention) flow through NT-GOVERNANCE → execution (task solving) flows through NT-CORE/NT-MIND → compliance (safety/quality) flows through NT-SHIELD. Information doesn't flow all-to-all; it flows through the hierarchy.

---

## Paper 5: Evolving-RL — Experience-Driven Self-Evolving Capability

**Paper**: [arXiv:2605.10663](https://arxiv.org/abs/2605.10663) (May 2026)
**Authors**: Fan et al.

### Core Idea
Self-evolving agents extract reusable experience from past interactions to adapt at deployment time. Most work focuses on system-level design (how experience is stored), neglecting the model's inherent extraction/utilization capabilities. Evolving-RL jointly optimizes both experience extraction AND utilization through coordinated co-evolution.

### Key Mechanism
- **Dual optimization**: Jointly trains experience extractor and experience solver
- **Evaluation-driven signals**: Two supervisory signals from evaluation optimize extractor and solver separately
- **Coordinated co-evolution**: Extractor and solver improve each other iteratively
- **Internalized experience**: By training on extracted experience, models internalize reusable patterns into parameters
- **Two-stage**: Experience extraction → evaluation → separate optimization of extractor and solver

### Results
- **98.7% relative improvement** over GRPO baseline on ALFWorld unseen tasks
- **35.8% improvement** on Mind2Web
- Gains fully unlocked only through coordinated co-evolution
- Works even without test-time experience accumulation (experience internalized into weights)

### NeoTrix Mapping

| Component | NeoTrix Domain | Pattern |
|-----------|---------------|---------|
| Experience Extractor | NT-MIND (experience distillation) | Extract reusable patterns from trajectories |
| Experience Solver | NT-CORE (reasoning) | Apply extracted patterns to new tasks |
| Coordinated Co-Evolution | NT-MIND (self-evolution) | Extractor and solver improve each other |
| Internalized Experience | NT-MEMORY (knowledge internalization) | Patterns become implicit model capability |
| Evaluation Signals | NT-GOVERNANCE (quality metrics) | Supervision from evaluation, not human labels |

**Key Insight for NeoTrix**: The SEAL pipeline currently separates experience extraction (experience-tree absorption) from capability application (skill crystallization). Evolving-RL shows these must be **jointly optimized** — the extractor learns to extract what the solver can use, and the solver learns to use what the extractor produces. The co-evolution signal comes from evaluation, not human labels.

**Integration Point**: Extend `nt_mind_skill_engine` with Evolving-RL's dual-optimization pattern. The experience extractor (currently experience-tree) and the skill solver (currently skill crystallization) should be jointly trained — the extractor learns to produce experience formats that the solver can most effectively internalize. The evaluation signal (task success on held-out tasks) drives both components simultaneously.

---

## Cross-Paper Synthesis: NeoTrix Integration Matrix

### Pattern 1: Predictive Attention (Token Sparse Attention + SparDA Forecast)
**Combined**: GWT doesn't just react to salience — it predicts future salience and prefetches. Token Sparse Attention shows per-head dynamic selection; SparDA shows lookahead prefetching. Together: predict what the next reasoning step needs, prefetch it, and select at token granularity.

### Pattern 2: Sheaf-Constrained Domain Coordination (Sheaf-ADMM + OrgAgent)
**Combined**: The 7 NeoTrix domains have formal agreement constraints (sheaf) AND hierarchical organization (OrgAgent). The sheaf specifies which domain states must agree; the hierarchy controls information flow. Together: mathematically rigorous inter-domain consistency with controlled information routing.

### Pattern 3: Co-Evolving Experience Systems (Evolving-RL + ARISE-RL from trending)
**Combined**: Experience extraction and utilization must co-evolve (Evolving-RL). Verification via rubrics replaces gold answers (ARISE-RL). Together: extract experience → create verification rubrics → solve → evaluate → refine extraction. The SEAL pipeline becomes a co-evolution loop.

### Priority Integration Path

| Phase | Pattern | Target Module | Complexity |
|-------|---------|---------------|------------|
| P1 | Predictive KV Prefetch | kv_cache_optimizer.rs | Medium |
| P1 | Sheaf Constraints for Domain Pairs | ConsciousnessTree | Medium |
| P2 | Hierarchical Domain Coordination | NT-GOVERNANCE | Medium |
| P2 | Dual-Optimization Experience Loop | nt_mind_skill_engine | High |
| P3 | Token-Level GWT Salience | GWT attention | High |
| P3 | Forecast-Style Predictive Attention | nt_core_self | High |

---

## NeoTrix Domain Impact (Papers 1-5)

| Domain | Papers | New Capability |
|--------|--------|---------------|
| NT-CORE | Token Sparse Attention, SparDA, OrgAgent | Per-head dynamic attention, predictive KV routing, hierarchical information flow |
| NT-MEMORY | Token Sparse Attention, SparDA | Token-level KV re-admission, predictive cache prefetch |
| NT-MIND | Evolving-RL | Co-evolving extraction/utilization, internalized experience |
| NT-GOVERNANCE | Sheaf-ADMM, OrgAgent | Mathematically rigorous domain constraints, three-layer hierarchy |
| NT-ACT | Sheaf-ADMM, OrgAgent | ADMM coordination, execution layer with review loops |
| NT-SHIELD | OrgAgent | Compliance layer, layered verification |
| NT-IO | Token Sparse Attention, SparDA | Hardware-compatible sparse patterns, async prefetch |
