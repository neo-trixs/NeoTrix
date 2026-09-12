# Model Reverse Engineering — Cycle 373 (2026-09-12)

## 5 New AI Models/Papers & NeoTrix Domain Mapping

---

### Paper 1: Flux Attention — Context-Aware Hybrid Attention
**arXiv:2604.07394** (Apr 2026)
**Authors**: Quantong Qiu et al.
**Topic**: Efficient LLM Inference / Attention Optimization

**Core Mechanism**:
- Layer-level dynamic routing between Full Attention (FA) and Sparse Attention (SA)
- Lightweight Layer Router injected into frozen pretrained LLMs
- Input-context-dependent routing (not static allocation ratio)
- Contiguous memory access preserved → theoretical reductions become wall-clock speedups
- 12 hours training on 8×A800 GPUs (parameter-efficient)

**Key Results**: 2.8× prefill speedup, 2.0× decode speedup. Superior performance-speed tradeoff on long-context + math reasoning benchmarks.

**Pattern for NeoTrix**:
- **NT-CORE (GWT)**: Layer Router concept maps to GWT's attention gating — dynamically choosing between full-salience broadcast (FA) and sparse selective attention (SA) based on current consciousness state
- **NT-MEMORY (KB)**: Contiguous memory access pattern → KB index layout optimization for sequential scans vs random access
- **L5 Cognition**: Layer-level routing is a micro-scale version of our Constellation-level routing — decisions made at per-layer granularity, not per-module

**Absorbable Concept**: `LayerRouter` — inject a lightweight router into frozen subsystems to dynamically switch between full-computation and sparse-computation modes based on input context.

---

### Paper 2: SparDA — Sparse Decoupled Attention with Forecast Projections
**arXiv:2606.04511** (Jun 2026)
**Authors**: Yaosheng Fu, Guangxuan Xiao, Xin Dong, Song Han, Oreste Villa
**Topic**: KV Cache Optimization / Long-Context Inference

**Core Mechanism**:
- Fourth per-layer projection: **Forecast** (alongside Q, K, V)
- Forecast predicts KV blocks needed by the next layer → lookahead selection
- CPU-to-GPU prefetch overlaps with current-layer execution
- Decoupled from attention query → one Forecast head per GQA group (reduced overhead)
- <0.5% additional parameters, trains only Forecast projections

**Key Results**: 1.25× prefill speedup, 1.7× decode speedup. 5.3× higher decode throughput with larger feasible batch sizes.

**Pattern for NeoTrix**:
- **NT-MEMORY (KB)**: Forecast projection → predictive prefetching for KB queries. Pre-compute which KV blocks (experience branches, embedding clusters) will be needed next based on current access pattern
- **NT-CORE (E8)**: Forecast as "next state prediction" — E8 hexagram transitions could use a lightweight Forecast to pre-load relevant reasoning states
- **L6 Meta-Cognition**: Lookahead selection mirrors meta-cognition's anticipatory self-monitoring — predict which self-test modules will be needed before they're invoked

**Absorbable Concept**: `ForecastProjection` — add a lightweight prediction head to any subsystem that accesses KV-like caches. The projection predicts future access patterns, enabling prefetch that overlaps with current computation.

---

### Paper 3: CoSA — Proxy-Kernel Co-Designed Sparse Attention
**arXiv:2607.25291** (Jul 2026)
**Authors**: Yufei Xue et al.
**Topic**: Long-Context Inference / Sparse Attention

**Core Mechanism**:
- Two-stage training-free sparse attention:
  - **Stage 1**: Kernel-Aware Proxy (KAP) selects blocks under moderate budget, produces ordered mask
  - **Stage 2**: Ordered-Skipping Kernel (OSK) applies mask, skips more blocks under tightened budget using online-softmax statistics
- Proxy and kernel are co-designed: proxy knows what kernel can skip, kernel knows proxy's ordering
- Tightening budget across stages (moderate → aggressive)

**Key Results**: 4.93× attention speedup, 2.53× TTFT reduction at 128K context. Negligible performance degradation.

**Pattern for NeoTrix**:
- **GWT Attention**: Two-stage routing — coarse-grained salience (KAP-like) selects candidate modules, fine-grained evaluation (OSK-like) refines selection with runtime statistics. Budget tightening = progressive attention narrowing as session deepens
- **SEAL Pipeline**: KAP/OSK two-stage pattern maps to SEAL's exploration → distillation — first broad scan (moderate budget), then focused refinement (tightened budget)
- **NT-SHIELD**: Ordered-skipping for security scans — fast proxy identifies high-risk areas, detailed kernel inspects only those areas

**Absorbable Concept**: `ProxyKernelCoupling` — design proxy and kernel as a co-optimized pair. The proxy's selection order is consumed by the kernel's skip logic. Progressive budget tightening across processing stages.

---

### Paper 4: RAGEN / StarPO — Trajectory-Level Agent RL
**arXiv:2504.20073** (Apr 2025, updated May 2025) + **RAGEN-2** arXiv:2604.06268 (Apr 2026)
**Authors**: Zihan Wang et al. (Stanford, Microsoft)
**Topic**: Agent Training / Reinforcement Learning / Self-Evolution

**Core Mechanism (StarPO)**:
- MDP formulation: states and actions are token sequences
- Rollout stage: generate N trajectories, each with reasoning-guided actions (`<think>...</think><ans>action</ans>`)
- Update stage: optimize entire trajectories via PPO (token-level advantage) or GRPO (normalized reward)
- Trajectory-level reward assignment (not per-turn)

**Core Mechanism (RAGEN-2)**:
- **Template Collapse**: entropy looks fine but reasoning is input-agnostic (fixed templates across different inputs)
- **Diagnosis**: decompose reasoning into within-input diversity (Entropy) + cross-input distinguishability (Mutual Information)
- **SNR-Aware Filtering**: use reward variance as lightweight proxy to select high-signal prompts per iteration
- Mutual Information correlates with performance much more strongly than entropy

**Key Findings**:
1. Echo Trap: reward variance cliffs + gradient spikes → StarPO-S stabilizes with trajectory filtering + critic incorporation
2. Diverse initial states + medium interaction granularity + more frequent sampling improves RL rollouts
3. Without fine-grained reasoning-aware rewards, agent reasoning barely emerges

**Pattern for NeoTrix**:
- **SEAL Pipeline**: StarPO's trajectory-level optimization maps to SEAL's cycle-level optimization — don't optimize per-phase, optimize across the full cycle trajectory
- **ConsciousnessTree**: Template Collapse diagnosis → meta-cognition self-deception detection. MI proxy could measure whether consciousness tree branches actually respond to different system states
- **NT-MIND (Self-Evolution)**: SNR-Aware Filtering for evolution iterations — filter training signals by reward variance to prevent gradient degradation
- **Experience-Tree**: Trajectory-level reward = cycle-level outcome assessment, not per-stage metric

**Absorbable Concept**: `TrajectoryRL` — optimize across full task trajectories, not per-step. Use MI (not entropy) to detect reasoning quality. SNR filtering to select high-signal evolution signals.

---

### Paper 5: Agentic AI Frameworks — Unified Design Patterns
**arXiv:2508.10146** (Aug 2025, comprehensive survey)
**Topic**: Agent Architecture / Design Patterns / Memory Taxonomy

**Core Framework**:
- **Two-dimensional classification**: Cognitive Function (7) × Execution Topology (6-8)
- Cognitive Functions: Perception, Memory, Reasoning, Action, Reflection, Collaboration, Governance
- Execution Topologies: Chain, Route, Parallel, Orchestrate, Loop, Hierarchy

**Memory Taxonomy** (12 systems evaluated):
| Type | Description | Example |
|------|-------------|---------|
| Short-term | In-context registers, session state | LangGraph stateful nodes |
| Long-term | Persistent storage, indexed retrieval | Mem0, Letta |
| Semantic | Concept-level knowledge | CrewAI entity memory |
| Procedural | Task flows, strategies | Semantic Kernel skills |
| Episodic | Contextual snapshots of interactions | AutoGen shared context |

**Key Insight**: "Agent performance depends on system-level coordination choices, not only on the base model." Coordination as a first-class design dimension.

**Pattern for NeoTrix**:
- **Six-Layer Architecture**: 7 Cognitive Functions map directly to our L1-L6 layers. The paper's Governance function = NT-GOVERNANCE. Reflection = NT-META.
- **Skill Domain Mapping**: Execution Topologies → our Skill Routing table. Chain = SEAL pipeline stages. Route = GWT attention routing. Parallel = worktree isolation. Hierarchy = ConsciousnessTree branching.
- **Memory Types**: Our KB already implements short-term (KV store), long-term (experience hub), semantic (embeddings). Missing: procedural memory (task flow templates) and episodic memory (contextual snapshots with full interaction traces).
- **AIN (Agentic Intent Network)**: Internet-scale agent coordination via shared capability routing infrastructure. Reduces per-agent integration from O(N) to O(1).

**Absorbable Concept**: `CognitiveTopologyMatrix` — explicitly classify every NeoTrix module by Cognitive Function × Execution Topology. Identify gaps (e.g., procedural memory, governance topology). Use as architecture review checklist.

---

## Cross-Paper Synthesis

### Pattern 1: Dynamic Routing at Every Scale
| Scale | Paper | NeoTrix Equivalent |
|-------|-------|-------------------|
| Layer-level | Flux Attention (FA vs SA) | GWT attention gating |
| Token-level | SparDA (Forecast projection) | KB predictive prefetch |
| Block-level | CoSA (KAP + OSK) | SEAL exploration → distillation |
| Agent-level | Switchyard / EvoRoute | Cost-Aware Routing (A1) |
| System-level | AIN (Agentic Intent Network) | Cross-domain capability routing |

### Pattern 2: Predictive Prefetch as Universal Accelerator
- SparDA: Forecast predicts next-layer KV blocks
- Flux Attention: Layer Router predicts per-layer attention mode
- CoSA: KAP predicts which blocks kernel will need
- **NeoTrix Application**: Predict which experience branches, KB nodes, or self-test modules will be needed based on current trajectory → preload into active context

### Pattern 3: Quality Metrics Beyond Entropy
- RAGEN-2: Mutual Information > Entropy for reasoning quality
- SparDA: Forecast accuracy as proxy for KV cache efficiency
- CoSA: Online-softmax statistics as runtime quality signal
- **NeoTrix Application**: Replace single-metric health monitoring with multi-axis quality: within-module diversity (entropy) + cross-module distinguishability (MI) + trajectory-level outcome (reward)

### Pattern 4: Two-Stage Co-Design
- CoSA: Proxy (selection) + Kernel (execution) co-designed
- RAGEN: Rollout (exploration) + Update (exploitation) interleaved
- Switchyard: Classifier (initial routing) + Escalation (adaptive re-routing)
- **NeoTrix Application**: Design SEAL phases as co-optimized pairs, not independent stages. Phase N's output format should be co-designed with Phase N+1's input requirements.

---

## Actionable Absorption Targets

| # | Concept | Source | Target Domain | Priority |
|---|---------|--------|---------------|----------|
| 1 | LayerRouter (FA/SA dynamic switch) | Flux Attention | NT-CORE GWT | P1 |
| 2 | ForecastProjection (predictive prefetch) | SparDA | NT-MEMORY KB | P1 |
| 3 | ProxyKernelCoupling (co-designed sparse) | CoSA | SEAL Pipeline | P2 |
| 4 | TrajectoryRL (MI-based quality) | RAGEN/StarPO | NT-MIND Evolution | P1 |
| 5 | CognitiveTopologyMatrix (design checklist) | Agentic Survey | Architecture Review | P2 |
| 6 | SNR-AwareFiltering (high-signal selection) | RAGEN-2 | Experience-Tree | P1 |
| 7 | ProceduralMemory (task flow templates) | Survey | NT-MEMORY KB | P2 |
| 8 | Agent-as-Participant (room-scoped context) | Switch | NT-IO ACP | P3 |

## Sources
- arXiv:2604.07394 (Flux Attention)
- arXiv:2606.04511 (SparDA)
- arXiv:2607.25291 (CoSA)
- arXiv:2504.20073 + arXiv:2604.06268 (RAGEN/StarPO/RAGEN-2)
- arXiv:2508.10146 (Agentic AI Frameworks Survey)
- NVIDIA NeMo Switchyard (GitHub + blog)
- A-MEM (NeurIPS 2025)
