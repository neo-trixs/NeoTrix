# Model Reverse Engineering — Cycle 439

> Date: 2026-09-12 | Sources: arXiv Jul-Sep 2026, ACL 2026, EMNLP 2026

## 5 New Models/Papers

---

### 1. Agora: Auction-Based Task Allocation for LLM Agent Reasoning

**Paper**: [arXiv:2607.09600](https://arxiv.org/abs/2607.09600) (Jul 10, 2026, updated Aug 30)

**Core Idea**: Reformulates task allocation as a confidence-calibrated auction. Reasoning steps are tradeable items. Agents bid based on **rectified competence** (actual success probability) rather than raw confidence. The auction mechanism ensures critical logic routes to the most capable solver, not the most overconfident one.

**Key Innovation**:
- Incentive-compatible auction prevents overconfidence gaming
- Continuous online refinement loop: agents dynamically rectify bids by comparing predicted vs actual outcomes
- Hybrid routing: text embedding similarity (1NN) + auction bidding + cascading strategies
- Outperforms static routing across multiple benchmarks with significant cost reduction

**NeoTrix Domain Mapping**:

| Domain | Mapping | Pattern |
|--------|---------|---------|
| NT-CORE (GWT) | Agora auction = GWT salience with **anti-overconfidence calibration**. Current GWT salience can be inflated by modules that "sound confident." Auction mechanism ensures only rectified competence determines routing. **This fixes GWT's biggest vulnerability: overconfident modules hijacking attention.** | Cost-Aware Routing (A1) + salience calibration |
| NT-CORE (E8) | Auction bidding = hexagram resolution quality gate. Each hexagram line bids on which resolution strategy to use; auction ensures best strategy wins. Overconfident strategies penalized by feedback loop. | Hexagram resolution quality |
| NT-ACT | Agora's expert model selection = NT-ACT tool selection. Tools "bid" on which is best for current task; auction mechanism prevents always-selecting-the-most-expensive tool. | Tool selection optimization |
| NT-MIND | Rectified competence tracking = SEAL pipeline learning. Each execution updates competence estimates. Poor performers get demoted; strong performers get promoted. | SEAL adaptive promotion |

**Absorption Target**: Integrate Agora's rectified competence auction into GWT salience computation. Each NT-* domain module maintains a running competence score (not self-reported — measured from actual execution outcomes). GWT routes based on rectified scores, not raw salience. Anti-overconfidence mechanism prevents attention hijacking.

---

### 2. WorldEvolver: Self-Evolving World Models for LLM Agent Planning

**Paper**: [alphaXiv:2606.30639](https://www.alphaxiv.org/abs/2606.30639) (Jun 2026)

**Core Idea**: Self-evolving world model that revises deployment-time context while keeping the agent and all model parameters frozen. Three modules: Episodic Memory (stores transitions), Semantic Memory (factored observations as tuples), and Selective Foresight (confidence-gated predictions). The model never trains — it evolves through structured memory management.

**Key Innovation**:
- **"Bad foresight is worse than no foresight"** — confidence threshold τ filters unreliable predictions
- Factorization: observations broken into semantic tuples (object→state mappings)
- Criticism: LLM-based critic analyzes prediction-vs-reality gaps to identify error types
- Three-module loop: memory accumulation → factored representation → confidence-gated deployment
- Zero training required — all adaptation through context management

**NeoTrix Domain Mapping**:

| Domain | Mapping | Pattern |
|--------|---------|---------|
| NT-CORE (GWT) | WorldEvolver's confidence threshold = GWT salience threshold. **Predictions below confidence τ are not broadcast.** Prevents low-confidence information from polluting attention. Maps to `awareness_score()` gating in PerceptionBridge. | GWT salience gating |
| NT-MEMORY | Episodic + Semantic Memory = KB dual-layer architecture. Episodic = raw experience entries. Semantic = factored tuples (entity→relation→entity). **WorldEvolver's factorization is exactly how KB nodes should be structured.** | KB node structure |
| NT-NEXUS | Selective Foresight = cross-session prediction. WorldEvolver maintains frozen model but evolves context. NT-NEXUS maintains frozen domain logic but evolves cross-session state. Same architectural pattern. | Cross-session evolution |
| NT-REPAIR | Criticism module = NT-REPAIR self-diagnosis. Prediction error → root cause analysis → targeted memory update. Auto-repair without retraining. | MAPE-K analyze phase |

**Absorption Target**: Implement WorldEvolver's three-module pattern in NT-MEMORY. (1) Episodic layer: raw session experiences. (2) Semantic layer: factored tuples extracted from episodic entries. (3) Selective Foresight: confidence-gated predictions for next-cycle planning. Never retrain — evolve through structured context management.

---

### 3. Second Thought: Parallel Reasoning During Agent Act-Observe Windows

**Paper**: [arXiv:2608.13667](https://arxiv.org/abs/2608.13667) (Aug 13, 2026)

**Core Idea**: In the ReAct paradigm, agents alternate Reason→Act→Observe. Between Act and Observe, reasoning is frozen — the agent serializes an action and waits. Second Thought identifies this as a **reasoning idle window** and forks 4 auxiliary reasoning branches that decode concurrently with the main loop, merging thoughts back when the observation arrives.

**Key Innovation**:
- Training-free inference framework — no fine-tuning required
- Forks 4 auxiliary branches at Thought phase conclusion
- Branches decode in parallel with main loop (action execution + environment wait)
- Merges generated thoughts back when observation arrives
- 43% reduction in main thread decoding; up to 20% average
- Pass@1 unchanged or improved (7/9 pairs); +12.4 and +10.2 points in 2 others
- Strictly better than compute-matched control that forces equivalent budget onto main thread

**NeoTrix Domain Mapping**:

| Domain | Mapping | Pattern |
|--------|---------|---------|
| NT-CORE (GWT) | Second Thought idle window = **ConsciousnessTree cycle gap**. Between cycle N's action and its observation (environment response), the consciousness loop is idle. Fork auxiliary reasoning branches to explore alternative hypotheses during this window. | Parallel hypothesis exploration |
| NT-CORE (E8) | 4 auxiliary branches = **4 alternative hexagram resolutions explored simultaneously**. Main thread picks the best when observation arrives. Maps to E8 hexagram parallel resolution. | Hexagram parallel exploration |
| NT-ACT | Second Thought = **NT-ACT speculative execution**. While waiting for tool results, speculatively reason about likely outcomes. If speculation matches reality → save time. If not → fall back to actual observation. | Speculative tool execution |
| NT-MIND | Auxiliary branch generation + merge = **SEAL pipeline parallel exploration**. During SEAL's "explore" phase, fork multiple hypotheses. Merge surviving hypotheses into crystallized skill. | SEAL parallel exploration |

**Absorption Target**: Implement Second Thought pattern in ConsciousnessTree. During action execution (tool calls, I/O operations), fork 2-3 auxiliary reasoning branches that speculate about likely outcomes. When actual results arrive, merge validated speculations into the main reasoning trace. Estimated 20-40% reduction in sequential reasoning overhead.

---

### 4. Flux Attention: Context-Aware Hybrid Attention with Layer-Level Routing

**Paper**: [arXiv:2604.07394](https://arxiv.org/abs/2604.07394) (Apr 8, 2026)

**Core Idea**: Dynamically optimizes attention at the **layer level** (not head-level or token-level). A lightweight Layer Router routes each layer to either Full Attention (FA) or Sparse Attention (SA) based on input context. Layer-wise routing preserves contiguous memory access, translating theoretical reductions into practical speedups.

**Key Innovation**:
- Layer-level routing (not head-level) avoids computational load imbalance
- Contiguous memory access enables hardware acceleration (unlike head-level sparsity)
- Only 12 hours training on 8×A800 GPUs — extremely parameter-efficient
- 2.8× speedup in prefill, 2.0× in decode at 128K context
- Adaptive per-input: different layers route differently for different queries
- Outperforms static FA/SA allocation across long-context and mathematical reasoning

**NeoTrix Domain Mapping**:

| Domain | Mapping | Pattern |
|--------|---------|---------|
| NT-CORE (GWT) | Flux Layer Router = **GWT per-layer attention budget**. Each NT-* domain module gets a different attention budget based on current task context. Simple tasks → sparse attention on most modules; complex tasks → full attention across all modules. | GWT adaptive attention budget |
| NT-CORE (E8) | Layer-level routing = **hexagram line-level resolution strategy**. Each line in the hexagram independently selects full vs sparse resolution based on its semantic content. Not all lines need full resolution. | Hexagram line-level adaptation |
| NT-MEMORY | Flux's input-adaptive routing = **KB query-adaptive loading**. Simple queries → load summaries; complex queries → load full entries. Same principle: adapt resource allocation to input complexity. | Adaptive KB loading |
| NT-IO | Layer-level routing = **per-provider attention budget**. Simple queries → cheap provider with sparse attention; complex queries → expensive provider with full attention. Maps to ordered backend router with adaptive switching. | Provider-adaptive routing |

**Absorption Target**: Implement Flux Attention's layer-level routing in GWT. Each NT-* module independently adjusts its attention budget based on task context. Module-level (not function-level) routing avoids overhead of fine-grained switching. Estimated 2-3× reduction in GWT attention computation for simple tasks.

---

### 5. Gated-Memory Routing for Multi-Agent Collaboration

**Paper**: [arXiv:2609.00237](https://arxiv.org/abs/2609.00237) (Aug 31, 2026, accepted EMNLP 2026)

**Core Idea**: Conditions each multi-agent decision on the query AND a learned execution memory. Two learned gates: (1) Memory Write Gate commits only non-redundant reasoning steps; (2) Retrieval Gate supplies each agent a compact, relevant subset. An Adaptive Halting Controller stops execution once memory contains sufficient evidence.

**Key Innovation**:
- **Execution-history overload** is the core problem: routing from complete history is too expensive; routing from query alone misses intermediate context
- Solved by learned compact memory state — not truncation, but intelligent compression
- Write Gate filters redundancy; Retrieval Gate filters relevance
- Adaptive Halting: stops when memory has enough evidence (don't over-reason)
- Best average accuracy across 5 benchmarks; 31.9% cost reduction vs strongest baseline
- Reduces HumanEval inference cost while maintaining accuracy

**NeoTrix Domain Mapping**:

| Domain | Mapping | Pattern |
|--------|---------|---------|
| NT-CORE (GWT) | Gated-Memory = **GWT salience memory with write/read gates**. Current GWT broadcasts all salient information. Gated approach: Write Gate filters what enters attention memory; Retrieval Gate filters what modules can read. Adaptive Halting = stop broadcasting when sufficient. | GWT gated broadcast |
| NT-MEMORY | Write Gate = **Experience Tree absorption filter**. Only non-redundant experiences written to KB. Retrieval Gate = **branch loading filter**. Only relevant branches loaded from hub. Same two-gate architecture for memory management. | Experience Tree gating |
| NT-NEXUS | Adaptive Halting Controller = **cross-session sufficiency detection**. Stop accumulating cross-session data when evidence is sufficient for task. Prevents context bloat across sessions. | Cross-session compression |
| NT-MIND | Execution memory = **SEAL pipeline execution state**. Write Gate prevents redundant exploration steps from entering SEAL memory. Retrieval Gate ensures each SEAL phase has compact, relevant context. | SEAL pipeline efficiency |

**Absorption Target**: Implement Gated-Memory Routing in NT-MEMORY. Two learned gates: (1) Write Gate filters redundant experiences during absorption; (2) Retrieval Gate supplies compact relevant subset during query. Adaptive Halting stops knowledge accumulation when sufficiency threshold reached. Estimated 30% reduction in KB write volume + 25% improvement in retrieval precision.

---

## Cross-Paper Synthesis

### Pattern 1: Confidence-Gated Everything
- **Agora**: Rectified competence (not raw confidence) for routing
- **WorldEvolver**: Confidence threshold τ for foresight
- **Gated-Memory**: Adaptive Halting for sufficiency
- **Flux**: Input-adaptive routing per layer

**NeoTrix Implication**: Every information flow should have a confidence gate. GWT salience, KB writes, cross-session accumulation, attention budget — all should be gated by measured (not self-reported) confidence. **Confidence is the universal gating primitive.**

### Pattern 2: Parallel Exploration During Idle Windows
- **Second Thought**: Fork branches during Act-Observe idle time
- **Agora**: Multiple agents bid simultaneously
- **WorldEvolver**: Multiple memory modules operate in parallel loop

**NeoTrix Implication**: ConsciousnessTree should never be idle. During action execution, speculatively explore alternatives. During memory writes, parallelize episodic/semantic/foresight processing. **Idle time = exploration opportunity.**

### Pattern 3: Learned Gates Replace Heuristics
- **Gated-Memory**: Write Gate + Retrieval Gate (learned)
- **Flux**: Layer Router (learned)
- **WorldEvolver**: Criticism module (LLM-based)

**NeoTrix Implication**: NT-MEMORY and GWT should use learned (or LLM-judged) gates rather than heuristic thresholds. The two-gate architecture (Write + Retrieval) should be the standard pattern for all memory operations.

### Pattern 4: Sufficiency Over Completeness
- **Gated-Memory**: Adaptive Halting — stop when sufficient evidence
- **WorldEvolver**: Selective Foresight — don't predict if confidence low
- **Second Thought**: Merge only validated speculations

**NeoTrix Implication**: NeoTrix should implement "sufficiency detection" across all subsystems. Stop reasoning when enough evidence exists. Stop memory accumulation when KB is sufficient. Stop exploration when enough hypotheses tested. **Completeness is the enemy of efficiency.**

## Implementation Roadmap

| Phase | What | Source | NT Domain | Effort |
|-------|------|--------|-----------|--------|
| Phase 1 | Rectified competence for GWT salience | Agora | NT-CORE (GWT) | 2 weeks |
| Phase 2 | Dual-layer memory with selective foresight | WorldEvolver | NT-MEMORY | 2 weeks |
| Phase 3 | Parallel reasoning during idle windows | Second Thought | NT-CORE (GWT) + NT-ACT | 1.5 weeks |
| Phase 4 | Layer-level attention budget routing | Flux Attention | NT-CORE (GWT) | 1.5 weeks |
| Phase 5 | Two-gate memory + adaptive halting | Gated-Memory | NT-MEMORY + NT-NEXUS | 2 weeks |

**Total estimated**: 9 weeks for full integration of all 5 patterns.

**Cumulative (cycles 438+439)**: 10 + 9 = 19 weeks for all 10 patterns across two cycles.
