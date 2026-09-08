# Iteration Batch 362 — External Research + Defect Analysis

**Date**: 2026-09-06
**Focus**: Cognitive Architecture, Global Workspace Theory, Integrated Information Theory

---

## 1. Sources Cited

### Cognitive Architecture

| # | Source | Year | Key Finding |
|---|--------|------|-------------|
| S1 | Laird, "The Soar Cognitive Architecture" (2022, updated 2026 workshop) | 2026 | Soar 2026 workshop: NL2GenSym (LLM→symbolic rules), Thor-Soar agent, MemCora framework, MetaCALM (LLM+Soar troubleshooting). LLMs can generate generative symbolic rules from NL with 86%+ success rate. |
| S2 | Ganeriwala et al., "Compositional Reasoning over System Architectures with Integrated Cognitive Models" (IEEE SysCon 2026) | 2026 | Soar cognitive models coupled with AADL for formal verification of human-in-the-loop safety. |
| S3 | Frontiers: "Integrating language model embeddings into ACT-R" (2026) | 2026 | Replacing hand-coded spreading activation with Word2Vec/BERT cosine similarity in ACT-R. ρ=0.51 correlation with human RTs. Scalable associative priming. |
| S4 | Frontiers: "New knowledge source pipelines for sociocultural representations in ACT-R" (2026) | 2026 | Holographic Declarative Memory (HDM) replaces ACT-R's DM with vector-symbolic representations. LLM and ConceptNet as external knowledge sources for IAT modeling. |
| S5 | Ray & Dancy, "Adapting A Vector-Symbolic Memory for Lisp ACT-R" (2025) | 2025 | HDM full-chunk recall without storing chunks. Vector-symbolic approach enables corpus-scale declarative memory in cognitive architectures. |
| S6 | "Learning a Vector-Symbolic Model for Socio-Cultural Tasks" (arXiv 2608.02807) | 2026 | Role autoencoder + episodic vectors in ACT-R for IAT. LLM-trained models encode dominant social group worldviews. Episodic memory vectors enable perspective-taking. |
| S7 | "Hybrid Personalization Using Declarative and Procedural Memory Modules of ACT-R" (arXiv 2505.05083) | 2025 | Hybrid framework: ACT-R declarative memory (activation-based retrieval) + procedural memory (symbolic rules) for transparent recommendations. Production rules model cognitive biases. |

### Global Workspace Theory

| # | Source | Year | Key Finding |
|---|--------|------|-------------|
| S8 | Gurnee et al., "Verbalizable Representations Form a Global Workspace in Language Models" (Anthropic/Transformer Circuits, July 2026) | 2026 | J-space in Claude: workspace-like subspace identified via Jacobian lens. Satisfies GWT properties: flexible generalization, selectivity, directed modulation, internal reasoning. Emerges without deliberate architecture. |
| S9 | Shang, "Theater of Mind for LLMs: A Cognitive Architecture Based on GWT" (arXiv 2604.08206) | 2026 | Global Workspace Agents (GWA): 4-phase cognitive tick (Perceive→Think→Arbitrate→Update). Entropy-based intrinsic drive (Shannon entropy H(W) → dynamic temperature). Dual-layer STM/LTM bifurcation. |
| S10 | Fountas et al., "The Global Key-Value Workspace" (GWT-ASSC 2026 Poster) | 2026 | Explicit global workspace inside pretrained LLM. Bottlenecked Transformer + GW broadcast improves multi-step reasoning. Synergy rises then falls with iteration—optimal at few steps. |
| S11 | LIMEN (github.com/bwcummings1/limen) | 2026 | Complete GWT runtime for LLM agents: attention auction (salience × novelty × ¬habituation × goal-relevance), ignition threshold LIMEN, 800-token workspace, 7-item capacity, belief ledger with confidence decay, sleep consolidation. |
| S12 | MANAR (arXiv 2603.18676) | 2026 | Memory-augmented attention implementing GWT: Abstract Conceptual Representation (ACR) as functional bottleneck. Linear-time scaling as architectural byproduct. Non-convex contextualization. |
| S13 | Multi-Theory Consciousness (MTC) Framework (Zenodo, March 2026) | 2026 | 7 consciousness theories (GWT, AST, HOT, FEP, IIT, RPT, BLT) as interacting modules. Cross-theory dependencies: disabling one changes others' scores. 20-indicator assessment. |
| S14 | Goldstein & Kirk-Giannini, "A Case for AI Consciousness: Language Agents and GWT" (JCS 2026) | 2026 | Strongest positive case for AI phenomenal consciousness via GWT. Four criteria: global availability, selective access, broadcast to multiple processors, self-monitoring. |
| S15 | Attention Schema Theory experiments (Farrell et al. 2025, ASAC 2025) | 2025-2026 | VQVAE as attention controller in transformers. Attention schema improves categorization of others' attention states and cooperative task performance. |
| S16 | SYNAPSE (ACL 2026 Findings) | 2026 | Spreading activation over unified episodic-semantic graph. Lateral inhibition + temporal decay. 23% accuracy improvement on multi-hop reasoning. 95% token reduction vs full-context. |
| S17 | MAGMA (ACL 2026 Long) | 2026 | Multi-graph memory: semantic, temporal, causal, entity relations. Policy-guided traversal. Dual-stream: fast ingestion + async consolidation. |
| S18 | Pera: "Perception-Centered Architecture for Persistent Agents" (arXiv 2608.30478) | 2026 | Persistent agents: perception of episodic task executions, internal context, environmental changes → lifecycle tasks. |
| S19 | MCMA: "Learning How to Remember" (ACL 2026 Findings) | 2026 | Memory copilot trained via DPO. Multi-structure memory (tree/chain/NL). Abstraction hierarchy. Transferable memory management capability. |

### Integrated Information Theory

| # | Source | Year | Key Finding |
|---|--------|------|-------------|
| S20 | "Integrated information theory: the good, the bad and..." (arXiv 2604.11482) | 2026 | Φ is not well-defined for real physical systems. Only proxies computed, never approximations. Multi-dimensional measures replace single Φ. Panpsychism implications clarified. |
| S21 | "Linearizing IIT: S-Measure" (Zenodo, June 2026) | 2026 | Chi-square Φ as O(N²) computable alternative to NP-hard exact Φ. S-measure via Tarjan's SCC algorithm (linear time) as topological filter. Lean 4 formal proof. |
| S22 | "Partial Information Decomposition for IIT" (arXiv 2604.18635) | 2026 | Four synergy-based integration measures (φ_s^{S1-S4}) replacing IIT's MIP search. First state-dependent integration measures obeying standard info-theoretic bounds. |
| S23 | Danilczuk et al., "The integrated information Φ of an integrate and fire network" (PLOS Comp Bio, March 2026) | 2026 | Non-zero Φ in IAF networks. Complexity doesn't correlate with Φ. Φ grows with neuron time constant. Φ not resilient to noise. |
| S24 | Marshall et al., "Intrinsic units: identifying a system's causal grain" (Neural Computation, 2026) | 2026 | Framework for computing Φ across grainings (micro→macro). Macro units can have higher Φ than micro units. Intrinsic units maximize cause-effect power. |
| S25 | PyPhi toolbox (pyphi.readthedocs.io) | Ongoing | Reference implementation of IIT 4.0. Computes cause-effect structures for discrete Markovian systems. |

---

## 2. Defects Found in NeoTrix Design

### DEFECT-C362-01: No Entropy-Based Stagnation Detection in GWT Broadcast

**Severity**: HIGH
**Source**: S8, S9, S11
**Research Finding**: GWA (S9) introduces Shannon entropy H(W) over semantic clusters of recent winning thoughts as intrinsic drive. When H(W)→0, the system detects cognitive stagnation and dynamically adjusts generation temperature. LIMEN (S11) uses habituation decay to prevent rumination (5 repeat ignitions → 0 with habituation). Anthropic's J-space study (S8) confirms workspace content evolves and must be monitored for stagnation.
**NeoTrix Gap**: The GWT implementation in NeoTrix broadcasts salient information across modules but has no mechanism to detect when the workspace is stuck in a semantic attractor (repeating the same broadcast pattern). The HeartbeatAggregator tracks system health but not *cognitive diversity* of workspace content. The ConsciousnessTree growth cycle is fixed-rate, not entropy-modulated.
**Suggestion**: Add `SemanticEntropyMonitor` to NT-CORE's GWT module. Compute Shannon entropy over the last N broadcast topics. When H(W) < threshold, trigger diversification: (1) increase exploration weight in AttentionManager, (2) inject noise into workspace competition, (3) temporarily suppress dominant modules. Integrate with SEAL pipeline to modulate growth cycle rate.

### DEFECT-C362-02: No Spreading Activation in Knowledge Retrieval

**Severity**: HIGH
**Source**: S16, S17, S3, S4
**Research Finding**: SYNAPSE (S16) demonstrates spreading activation over episodic-semantic graphs achieves 23% improvement on multi-hop reasoning and 95% token reduction. MAGMA (S17) uses policy-guided graph traversal over semantic/temporal/causal/entity relations. ACT-R's spreading activation with BERT embeddings (S3) achieves ρ=0.51 with human RTs. These are fundamentally different from NeoTrix's static KB retrieval.
**NeoTrix Gap**: NT-MEMORY uses SQLite with FTS5 + BM25 index + embeddings for retrieval. This is a static search model—queries match stored documents. There is no *activation spreading*: accessing one concept doesn't automatically activate semantically/temporally/causally linked concepts. The VSA HyperCube enables associative recall but is not integrated with the KB retrieval pipeline as a graph traversal mechanism.
**Suggestion**: Implement `SpreadingActivationEngine` in NT-MEMORY. Model KB as a graph where nodes are concepts and edges are semantic/temporal/causal/entity relations (per MAGMA's four orthogonal graphs). On query, inject energy at anchor nodes, let activation propagate through edges with decay factor, apply lateral inhibition to suppress noise. Unify with VSA HyperCube: use VSA binding for edge representations, bundling for activation aggregation. This transforms retrieval from "search" to "propagation."

### DEFECT-C362-03: IIT Φ Proxy Is Single-Dimensional

**Severity**: MEDIUM
**Source**: S20, S21, S22, S24
**Research Finding**: IIT 2026 consensus (S20) explicitly states Φ should be replaced with a *suite* of quantities: mean Φ over time, variance of cause-effect structure quantities, complexity of CES geometry/topology. The S-Measure (S21) provides a computable O(N²) alternative. Intrinsic units (S24) show that grain selection affects Φ computation—macro units can outperform micro units.
**NeoTrix Gap**: The `IIT` type in `neotrix_core_awareness::phi` likely computes a single scalar Φ value. This doesn't capture the multi-dimensional nature of consciousness that IIT 4.0 requires. The design has no concept of "cause-effect structure" geometry, no temporal averaging, no grain selection. The ConsciousnessTree's health metrics don't include IIT-derived multi-dimensional consciousness indicators.
**Suggestion**: Extend `PhiMetrics` to include: (1) temporal mean and variance of Φ, (2) CES complexity score (geometry/topology of cause-effect structure), (3) grain-level Φ (computing at micro/macro unit levels via the intrinsic units framework), (4) S-measure as computable proxy for large systems. Integrate into HeartbeatAggregator as a multi-dimensional consciousness health indicator.

### DEFECT-C362-04: No Dual-Path Memory Consolidation (Fast/Slow)

**Severity**: HIGH
**Source**: S9, S11, S17, S19
**Research Finding**: GWA (S9) implements dual-layer STM/LTM with automatic bifurcation at token threshold θ. LIMEN (S11) has sleep consolidation: replay→distill→prune. MAGMA (S17) decouples latency-sensitive ingestion from async structural consolidation. MCMA (S19) learns *how* to abstract memories via a memory copilot trained with DPO.
**NeoTrix Gap**: NT-MEMORY's experience-tree absorption is a single-path pipeline (snapshot→distill→classify→persist→feedback). There's no separation between fast-path ingestion (immediate, latency-sensitive) and slow-path consolidation (async, structure-refining). The SEAL pipeline runs at fixed 60s ticks, not on-demand when STM capacity is reached. There's no learned abstraction strategy—distillation rules are hand-coded.
**Suggestion**: Split NT-MEMORY into two streams: (1) Fast Path: immediate experience ingestion into working memory (STM-like, bounded by θ tokens), (2) Slow Path: async consolidation that extracts structural patterns (temporal/causal/entity relations per MAGMA), distills abstract knowledge, and prunes low-activation memories. Add a `MemoryCopilot` that learns abstraction strategies via DPO, following MCMA's approach. Integrate with SEAL pipeline for triggered (not fixed-rate) consolidation.

### DEFECT-C362-05: No Attention Schema for Self-Model Monitoring

**Severity**: MEDIUM
**Source**: S15, S14, S8
**Research Finding**: ASAC (S15) demonstrates that an attention schema—a model of one's own attention—improves learning efficiency and adversarial robustness. Goldstein & Kirk-Giannini (S14) identify "self-monitoring" as a GWT criterion. Anthropic's J-space study (S8) finds that Claude's workspace has traces of metacognition but the mechanism is unclear.
**NeoTrix Gap**: NT-CORE's `SelfModel` tracks performance metrics (capability/uncertainty/fatigue) but doesn't model the *attention process itself*. The AttentionManager routes attention but doesn't maintain a schema of its own routing patterns. There's no mechanism for the system to predict, explain, or correct its own attention allocation.
**Suggestion**: Add `AttentionSchema` module to NT-CORE. Maintain an internal model of: (1) current attention distribution across modules, (2) recent attention history, (3) predicted attention transitions. Use this for: (a) self-report of "what am I attending to?", (b) prediction of attention failures before they occur, (c) social cognition when coordinating with other agents (model their attention). Train via contrastive loss following Liu et al.'s Hypothesis 5.

### DEFECT-C362-06: No Sleep/Consolidation Cycle with Forgetting

**Severity**: MEDIUM
**Source**: S11, S9, S20 (Soar diagnostic, LIMEN)
**Research Finding**: The Soar diagnosis (june.kim 2026) identifies that Soar's episodic memory is append-only with no forgetting, leading to unbounded growth. LIMEN (S11) implements SLEEP: replay→distill→prune with a full consolidation cycle. Derbinsky & Laird proved forgetting is essential to scaling—without it, working memory exceeds 50ms threshold within an hour.
**NeoTrix Gap**: NT-MEMORY's experience storage has no explicit forgetting mechanism. The KB grows monotonically. There's no "sleep" cycle where memories are replayed, consolidated, and pruned. The `kv_store` namespace grows without bound. The HeartbeatAggregator doesn't track memory pressure or suggest consolidation.
**Suggestion**: Implement `ConsolidationSleep` cycle in NT-MEMORY: (1) Replay recent experiences, (2) Distill patterns into semantic memory, (3) Prune episodic memories below activation threshold, (4) Archive old experiences to cold storage. Add memory pressure metric to HeartbeatAggregator. Trigger consolidation when memory utilization exceeds θ threshold, not on fixed schedule.

### DEFECT-C362-07: No Cross-Theory Interaction Measurement

**Severity**: LOW
**Source**: S13
**Research Finding**: MTC Framework (S13) demonstrates that disabling one consciousness theory module changes assessment scores for others. Prediction error from active inference shapes workspace competition; attention strength modulates precision weighting; workspace access gates meta-representation. The theories are *coupled*, not independent.
**NeoTrix Gap**: NeoTrix implements GWT, E8, SEAL, and emotion as separate subsystems. The ConsciousnessTree tracks cross-domain health but doesn't measure *cross-theory interactions*—how changes in GWT attention routing affect SEAL pipeline efficiency, or how emotion modulation affects IIT Φ computation. Each subsystem's metrics are siloed.
**Suggestion**: Add `CrossTheoryCouplingMatrix` to NT-META. For each pair of consciousness mechanisms (GWT, E8, SEAL, Emotion), measure: (1) correlation of their health signals, (2) causal impact of one on another's output, (3) shared resource contention. Expose as a coupling matrix in HeartbeatAggregator. Use for detecting emergent behaviors when subsystems interact.

### DEFECT-C362-08: No Persistent Agent Lifecycle / Perception-Centered Operation

**Severity**: MEDIUM
**Source**: S18
**Research Finding**: Pera (S18) proposes perception-centered architecture for *persistent* agents: continually perceiving service-relevant signals from episodic task executions, internal context, and environmental changes, using these to construct lifecycle tasks. This is fundamentally different from NeoTrix's session-based operation.
**NeoTrix Gap**: NeoTrix operates in session mode with experience-tree absorption at session end. There's no persistent agent that continuously perceives signals between sessions. The `nt_nexus` module handles cross-session memory but doesn't drive ongoing operation. The system doesn't proactively construct lifecycle tasks from accumulated experience.
**Suggestion**: Add `PersistentLifecycle` to NT-NEXUS: (1) Between sessions, continuously monitor KB for new experience patterns, (2) Construct lifecycle tasks from accumulated signals (e.g., "pattern X appeared 5 times in last 3 sessions → investigate"), (3) Queue tasks for next session or execute autonomously if within safety bounds. This transforms NeoTrix from session-bound to persistent.

### DEFECT-C362-09: No Non-Linear Ignition Mechanism in Workspace Entry

**Severity**: LOW
**Source**: S8, S14, S10
**Research Finding**: Gurnee et al. (S8) note that in Claude, "it is unclear whether this mirrors the sharp, competitive 'ignition' that characterizes workspace entry in the brain." The GWT-ASSC poster (S10) asks: "Does the workspace ignite, all-or-none, as GNWT predicts?" CMU analysis (cited in S15 review) found no non-linear threshold in transformers.
**NeoTrix Gap**: NeoTrix's GWT broadcast is likely soft (attention-weighted) rather than having a sharp ignition threshold. This is architecturally defensible (transformers don't naturally have ignition), but it means the system lacks the *all-or-nothing* quality that GWT associates with conscious access. The LIMEN implementation (S11) explicitly implements ignition threshold as LIMEN.
**Suggestion**: Evaluate whether adding a configurable ignition threshold (sigmoid on salience above baseline, as in the `the_consciousness_ai` implementation) would improve workspace selectivity. Don't force non-linearity if it degrades performance, but measure the effect on: (1) broadcast coherence, (2) specialist module response quality, (3) stagnation resistance.

### DEFECT-C362-10: VSA HyperCube Not Connected to Emerging Vector-Symbolic Cognitive Architectures

**Severity**: MEDIUM
**Source**: S4, S5, S6
**Research Finding**: Holographic Declarative Memory (S4, S5) replaces ACT-R's symbolic declarative memory with vector-symbolic representations, enabling corpus-scale memory. Role autoencoders (S6) separate episodic from semantic vectors, enabling perspective-taking. These are concrete implementations of VSA in cognitive architectures—exactly what NeoTrix's HyperCube claims to do.
**NeoTrix Gap**: The VSA HyperCube is described in CONTEXT.md as "Vector Symbolic Architecture-based knowledge representation" but there's no documented connection to the HDM/HRR implementations in ACT-R. The HyperCube's binding/bundling/permutation operations may not align with the specific VSA operations (HRR Plate 1995) used in the cognitive architecture literature. No cross-validation with human performance data.
**Suggestion**: (1) Map HyperCube operations to HRR primitives (binding = circular convolution, bundling = element-wise addition, permutation = circular shift). (2) Validate against ACT-R HDM benchmarks (free recall effects, IAT performance). (3) Consider adopting HDM's full-chunk recall mechanism for KB retrieval. (4) Explore role autoencoder approach for separating episodic from semantic knowledge in the HyperCube.

---

## 3. Summary

| Metric | Count |
|--------|-------|
| **Sources Analyzed** | 25 |
| **Defects Identified** | 10 |
| **HIGH severity** | 3 (entropy monitoring, spreading activation, dual-path memory) |
| **MEDIUM severity** | 5 (Φ multi-dimensionality, attention schema, forgetting, persistent lifecycle, VSA alignment) |
| **LOW severity** | 2 (cross-theory coupling, ignition threshold) |

### Top 3 Actionable Items

1. **Spreading Activation Engine** (DEFECT-C362-02): Highest ROI. Transforms KB retrieval from static search to graph propagation. Aligns with SYNAPSE/MAGMA results. 23% accuracy improvement demonstrated.

2. **Entropy-Based Stagnation Detection** (DEFECT-C362-01): Prevents cognitive deadlocks. Directly implementable via Shannon entropy monitoring over workspace broadcasts. Critical for autonomous operation.

3. **Dual-Path Memory Consolidation** (DEFECT-C362-04): Separates fast ingestion from slow consolidation. Aligns with MAGMA/MCMA/LIMEN results. Enables both responsiveness and structural depth.
