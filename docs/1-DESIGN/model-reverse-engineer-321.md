# Model Reverse Engineering — Cycle 321

**Date**: 2026-09-11
**Focus**: Hierarchical memory routing, decentralized multi-agent, error-bounded attention, content-routed state, communication control

---

## 5 Models/Papers for Reverse Engineering

### 1. MKA: Memory-Keyed Attention for Efficient Long-Context Reasoning
- **Paper**: https://arxiv.org/abs/2603.20586 (Mar 2026)
- **Key Innovation**: Hierarchical attention mechanism that routes queries across three memory tiers — local (L1), session (L2), and long-term (L3) — via learned lightweight routing gates. Each query token dynamically selects which memory level to attend. FastMKA variant fuses memory sources before attention computation: single KV projection + single attention pass. Caches fused (routed) KV instead of raw token KV. 5x training throughput and 1.86x decode speedup at 256K context with only 1.2% perplexity loss (3.26 vs 3.22 PPL).
- **Architecture Pattern**: Three-tier memory hierarchy with learned routing gates. Route-fusion before attention avoids multiple attention paths. Fused KV caching reduces memory bandwidth. Sublinear complexity scaling. Compatible with standard Transformer pipelines as drop-in replacement.
- **NeoTrix Domain Mapping**:
  - **NT-MEMORY**: Direct analog to our KB tiered architecture — L1 = working memory (KV cache), L2 = session memory (conversational context), L3 = long-term memory (KB experience hub). The learned routing gates map to our GWT salience scoring for memory access. Route-fused KV caching parallels our KB query result caching strategy
  - **GWT (NT-CORE)**: The routing gate that dynamically selects memory tier per query = GWT attention allocation across domains. Query-dependent routing = salience-based attention modulation. The "route-fuse-attend" pipeline is a hardware-friendly implementation of our GWT broadcast-then-focus pattern
  - **Axiom A2 (Context as Scarce)**: MKA's 5x training speedup at 256K context directly validates our KVMem paged KV strategy. The route-fusion pattern reduces memory bandwidth pressure — critical for our context window management
  - **KVMem**: MKA's three-tier memory (local/session/long-term) maps to our GPU/Host/NVMe tiering. The fused KV caching is compatible with our paged KV virtualization

### 2. DeAR: Decentralized Agentic Reasoning via Capability Grounding and Collaborative Thought Navigation
- **Paper**: https://arxiv.org/abs/2608.17282 (Aug 2026)
- **Key Innovation**: Shifts from centralized multi-agent orchestration to autonomous peer-to-peer collaboration. Three mechanisms: (1) Decentralized Capability Grounding — agents are not assigned rigid roles but ground identity in verifiable linguistic benchmarks from model cards; (2) Thought Map Navigation — agents maintain a shared topological map of reasoning space, enabling progressive reasoning (pivot from dead ends without losing progress); (3) Topology Update — adaptive error correction via localized progressive backtracking. Collaboration Propensity Matrix (dynamic adjacency matrix) defines peer interaction probabilities. Dynamic termination when propensities fall below threshold.
- **Architecture Pattern**: Decentralized reasoning via local graph navigation. Capability grounding replaces static role assignment. Thought map as shared reasoning substrate — agents navigate a topology rather than following linear chains. Progressive refinement: dead ends trigger local backtracking, not full restart. Topology pruning via edge penalty in the propensity matrix.
- **NeoTrix Domain Mapping**:
  - **ConsciousnessTree**: DeAR's thought map = our ConsciousnessTree branches. Progressive reasoning (pivot from dead ends) = our 6-stage feedback loop where failed stages trigger re-evaluation of prior stages, not full restart. The dynamic termination based on propensity thresholds = our health-score-based cycle continuation
  - **GWT (NT-CORE)**: Collaboration Propensity Matrix = GWT salience matrix across NT-* domains. Decentralized routing (agent selects peer via local traversal) = domain-internal routing without central coordinator. The "capability grounding" pattern validates our domain-identity model (each NT-* domain has self-defined capabilities)
  - **NT-REPAIR**: Progressive backtracking from dead ends = our self-healing repair pattern. Localized edge pruning (penalize failing edges, not rewrite entire topology) = surgical repair over wholesale replacement. Topology Update via failure evidence = experience-tree failure feedback
  - **Dual Specialization**: DeAR's decentralized agents parallel our dual specialization — agents autonomously select collaboration partners based on current task context, not fixed topology

### 3. CEDAR: Error-Bounded Residual Routing for Efficient Long-Context Attention
- **Paper**: https://arxiv.org/abs/2609.07237 (Sep 2026)
- **Key Innovation**: Coarse-to-fine error-aware attention routing. Each semantic chunk contributes a cheap key-value summary to a residual attention path. Chunks with high estimated approximation error are expanded to exact token attention. Exact and summarized contributions combined in single softmax normalization — refinement replaces (not duplicates) coarse evidence. Output-error bound derived from within-chunk key/value dispersion. Variable refinement budget allocated per-query based on error estimate. Reduces reconstruction error by 98%+ vs hard dropping at equal exact-chunk budgets. Maintains ~3x kernel speedup at 128K context.
- **Architecture Pattern**: Residual attention with error-bounded refinement. Cheap summaries provide coverage; exact attention provides precision. Error bound from key/value dispersion determines refinement budget. Variable per-query refinement (not fixed top-k). Single softmax normalization prevents evidence duplication.
- **NeoTrix Domain Mapping**:
  - **GWT (NT-CORE)**: Error-bounded refinement = GWT salience confidence scoring. Chunks with high approximation error = domains/topics where attention is uncertain. Variable refinement budget = cost-aware attention allocation (spend more compute where uncertainty is high). The "refinement replaces, not duplicates" principle prevents redundant attention across GWT cycles
  - **NT-MEMORY**: Residual summaries = compressed KB embeddings. Error-bounded expansion = on-demand full-text retrieval when embedding approximation is uncertain. Variable refinement budget = adaptive query depth based on KB confidence scores
  - **Axiom A2 (Context as Scarce)**: CEDAR's key insight — spend more compute on uncertain queries, less on easy ones — directly validates our cost-aware routing axiom. The 98% error reduction at equal budget shows that error-aware allocation outperforms fixed-top-k
  - **ConsciousnessTree**: CEDAR's coarse-to-fine refinement = our 6-stage loop where early stages provide coarse assessment and later stages refine uncertain branches. Error bounds determine which branches get deeper attention

### 4. MARCH: Scaling Recurrent Memory with Content-Routed State Anchors
- **Paper**: https://arxiv.org/abs/2608.12435 (Aug 2026)
- **Key Innovation**: Augments recurrent models with content-routed state anchors. Periodically checkpoints cumulative recurrent state as state anchors, each associated with compact content-conditioned anchor key. At each token, produces an anchor query to attend all causally available state anchors. Output = attention-style aggregation over historical anchors + current state readout. Null route allows model to bypass historical memory when current state suffices. Residual fusion preserves original recurrent path. Outperforms linear attention variants across commonsense reasoning, LongBench, and in-context retrieval.
- **Architecture Pattern**: Periodic state checkpointing creates a growing memory bank. Content-conditioned routing selects relevant historical states. Null route for bypass when current state is sufficient. Residual fusion preserves original recurrence. Fused reader implementation exceeds FlashAttention-2 throughput at 64K+ context.
- **NeoTrix Domain Mapping**:
  - **NT-MEMORY (KB)**: State anchors = KB experience snapshots (periodic checkpoints of accumulated knowledge). Content-conditioned routing = KB query routing (semantic search to find relevant past experiences). Null route = KB bypass when working memory suffices (don't query KB if context window has the answer). The growing memory bank = our KB experience index that grows with sessions
  - **ConsciousnessTree**: State anchors = cycle snapshots (each growth cycle produces a checkpoint). Content routing between anchors = cross-cycle learning (new cycle queries relevant prior cycles). Null route = skip cross-cycle learning when current cycle's context is sufficient. Residual fusion = carry forward insights from prior cycles
  - **GWT (NT-CORE)**: Content-routed state attention = GWT salience over historical context. The anchor query mechanism = GWT broadcast to historical knowledge. Residual fusion preserves current reasoning while augmenting with historical context — this is our GWT attention pattern with explicit historical grounding
  - **KVMem**: MARCH's state anchors parallel our paged KV snapshots. Content routing = KV page selection based on semantic relevance. The growing memory bank maps to our session → cold storage tiering

### 5. Consilience: Conformally Calibrated Communication Control for Hidden-Profile Multi-Agent Reasoning
- **Paper**: https://arxiv.org/abs/2608.20564 (Aug 2026)
- **Key Innovation**: Inference-time orchestration framework that steers and certifies multi-agent communication under distributed private information. At each turn, summarizes discussion using compact state capturing: uncertainty, disagreement, evidence gain, redundancy, premature consensus. Selects communication intervention (challenge, clarify, seek evidence, route) and appropriate speaker. Central contribution: round-wise conformal calibration providing distribution-free, finite-sample guarantee — one-step regret bounded by calibrated threshold with probability ≥ 1-α. Acceptance mechanism enforces guarantee for executed action. Outperforms full-information baseline on HiddenBench-style tasks.
- **Architecture Pattern**: State-based discussion summarization (5 dimensions: uncertainty, disagreement, evidence gain, redundancy, premature consensus). Conformal calibration provides statistical guarantees on action quality. Intervention selection (challenge/clarify/evidence/route) based on state. Speaker selection based on state. Surpasses full-information baseline — certified communication control > information availability.
- **NeoTrix Domain Mapping**:
  - **GWT (NT-CORE)**: The 5-dimensional discussion state = GWT salience context (uncertainty → attention needed, disagreement → conflict resolution needed, evidence gain → new information detected, redundancy → attention can be reduced, premature consensus → validation needed). The intervention selection (challenge/clarify/evidence/route) = GWT attention modulation modes. Consilience's finding that certified communication > information availability validates our GWT architecture — better attention routing beats more data
  - **ConsciousnessTree**: The conformal calibration guarantee = our cycle quality assurance. Round-wise calibration = per-cycle health assessment with statistical confidence. The "surpasses full-information baseline" result means our ConsciousnessTree's selective attention routing can outperform systems that process everything
  - **NT-SHIELD**: Conformal calibration for action quality = our RiskAssessor scoring. The acceptance mechanism (reject inadmissible proposals) = our tool-call interception (Harden AIF pattern). Distribution-free guarantees are valuable for security-critical decisions
  - **Axiom A1 (Cost-Aware)**: Consilience's certified communication reduces wasted rounds — each intervention is calibrated to provide maximum information gain. This validates cost-aware attention: spend communication budget where it has highest marginal value

---

## Synthesis: Cross-Paper Patterns

### Pattern 1: Hierarchical Memory with Learned Routing (MKA, MARCH)
Both papers organize memory into hierarchical tiers with content-dependent routing. MKA: local/session/long-term with routing gates. MARCH: state anchors with content-conditioned routing. Both use null routes for bypass when current state suffices.

**NeoTrix Mapping**: Our KB should implement a three-tier hierarchy with content-routed access. Tier 1: working memory (current session context). Tier 2: recent session summaries (last N sessions). Tier 3: long-term experience hub (all historical). Routing gates score each tier per query and dynamically allocate attention. Null routes skip lower tiers when working memory suffices — critical for Axiom A2 (Context as Scarce).

### Pattern 2: Decentralized Reasoning with Progressive Refinement (DeAR, Consilience)
Both papers move away from centralized orchestration. DeAR: decentralized agents navigate a thought map via local graph traversal. Consilience: certified communication control with conformal calibration. Both enable progressive refinement — pivot from dead ends without full restart.

**NeoTrix Mapping**: Our ConsciousnessTree should support decentralized domain communication — NT-* domains communicate via EventBus, not central coordinator. The thought map navigation pattern = our experience-graph traversal. Consilience's conformal calibration = our cycle quality assurance with statistical confidence bounds. Progressive refinement = our NT-REPAIR self-healing via selective suffix replacement.

### Pattern 3: Error-Bounded Attention Refinement (CEDAR, MKA)
Both papers replace fixed top-k with error-aware variable refinement. CEDAR: error bound from key/value dispersion determines per-query refinement budget. MKA: routing gates dynamically allocate attention across memory tiers. Both avoid information loss from hard selection.

**NeoTrix Mapping**: GWT salience should be error-aware — assign higher attention to topics where the model's approximation error is high. Variable refinement budget = cost-aware attention allocation. This validates our GWT architecture: attention is not uniform but varies by uncertainty. The "refinement replaces, not duplicates" principle prevents redundant attention across cycles.

### Pattern 4: State Checkpointing with Content Routing (MARCH, MKA)
Both papers checkpoint state and route queries to relevant checkpoints. MARCH: periodic state anchors with content-conditioned keys. MKA: session-level summaries with routing gates. Both use residual fusion to preserve current reasoning.

**NeoTrix Mapping**: Our experience-tree should produce periodic state checkpoints (cycle summaries) with content-conditioned keys. Cross-cycle learning routes new cycle queries to relevant prior checkpoints. Residual fusion = carry forward prior insights while maintaining current reasoning. This is our KB experience hub architecture validated by MARCH.

### Pattern 5: Certified Guarantees for Multi-Agent Coordination (Consilience, DeAR)
Both papers provide formal guarantees for agent coordination. Consilience: conformal calibration bounds one-step regret with probability ≥ 1-α. DeAR: progressive backtracking guarantees convergence via topology update. Both avoid worst-case coordination failures.

**NeoTrix Mapping**: Our SEAL pipeline should include certified quality gates — conformal calibration for stage transitions, progressive backtracking for failed stages. The statistical guarantees are valuable for production deployment. DeAR's topology update pattern = our experience-tree self-evolution with failure feedback.

---

## Implementation Candidates

| Priority | Paper | Action | NeoTrix Component |
|----------|-------|--------|-------------------|
| P0 | MKA | Implement three-tier memory hierarchy with routing gates | nt_memory (KB tiering) |
| P0 | Consilience | Add conformal calibration to GWT salience scoring | nt_core (GWT) |
| P1 | MARCH | Add periodic state checkpointing to experience-tree | nt_memory (experience hub) |
| P1 | DeAR | Implement decentralized domain communication via EventBus | nt_core (inter-domain) |
| P2 | CEDAR | Add error-bounded refinement to attention allocation | nt_core (GWT) |
| P2 | DeAR | Add progressive backtracking to NT-REPAIR workflows | nt_repair |
