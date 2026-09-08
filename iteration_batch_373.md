# Iteration Batch 373 — Recommender Systems, Personalization, Multi-Objective Optimization (2026)

**Date**: 2026-09-06
**Research Domains**: (1) Recommender systems, (2) Personalization, (3) Multi-objective optimization

---

## Sources Consulted

### Recommender Systems (2026)

| ID | Source | Key Contribution |
|----|--------|-----------------|
| S1 | Mehta et al., "LLM Enhanced Embeddings for Knowledge Aware RecSys" (DOI:10.1007/s44163-026-01404-y, May 2026) | LLM-induced KG from item text, KG-derived embeddings (translational + bilinear) integrated into CF/GNN/feature-interaction models; consistent Recall@K/NDCG@K gains across models |
| S2 | KGERA, Nature Scientific Reports (Mar 2026) | Test-time reasoning over KG: profile similarity, genre/creator affinity, complementarity, substitutability, diversity-aware penalization. ~10ms/user inference. 52.83% NDCG@10 improvement. Heterogeneous stacking ensemble with interpretable coefficients |
| S3 | DCGL, arXiv:2605.07314 (May 2026) | Dual-channel graph learning: structurally decoupling LLM semantics from ID behavioral signals. Multi-level contrastive learning (intra-view + inter-view). Frequency-aware gated fusion adapts to interaction frequency |
| S4 | DAIGNN, Nature Scientific Reports (Jun 2026) | Dual-adaptive imputation GNN: similarity-driven imputation via pseudo-ratings, multiple auxiliary info sources, dual-adaptive feature fusion. 3.1% AUC improvement |
| S5 | TAGCF, arXiv:2602.21099 | Topology-augmented graph CF: LLM infers interaction intents as intermediate attribute nodes in U-A-I graph. Adaptive Relation-weighted Graph Convolution (ARGC). No textual embeddings needed |
| S6 | MixRAGRec, arXiv:2605.28175 (May 2026) | Multi-agent KG-RAG: MoE retrieval agent (query-aware granularity), Knowledge Preference Alignment agent (KG→text), Contrastive Learning-reinforced recommendation agent. MMAPO training |
| S7 | KnowSA, arXiv:2604.07825 (Apr 2026) | Selective knowledge augmentation: estimates LLM's internal knowledge per item via Comparative Knowledge Probing, injects external info only where needed. No fine-tuning |

### Personalization (2026)

| ID | Source | Key Contribution |
|----|--------|-----------------|
| S8 | Sager (Self-Evolving Agent for Personalized Rec), arXiv:2604.14972 | Per-user self-evolving policy skill: personalizes *how* agent reasons, not just what it knows. Incremental contrastive chain-of-thought engine. Two-representation skill architecture (full + slim injection) |
| S9 | HypReflect, arXiv:2609.00251 (Aug 2026) | Hypotheses-guided self-distillation for continual personalization: explicit uncertainty-aware preference hypotheses, reflective refinement, generalizes to unseen users and cross-domain |
| S10 | NP-MiSR, AAAI 2026 | Neural Process-based multi-interest for SBR: adaptive interest count per session (no pre-defined K), cross-session context fusion. 38.8% avg improvement. 10% data → 95% performance |
| S11 | GraphFine, AAAI 2026 | Multi-granular graph learning for SBR: position-aware graph (short-term transitions) + segmented co-occurrence hypergraphs (high-order semantics) + multi-view intent readout |
| S12 | SPRINT, arXiv:2508.00570 | Scalable intent refinement: global intent pool + predict-and-correct loop + lightweight intent predictor. No LLM dependency at inference. Uncertainty-aware selective LLM invocation |
| S13 | Persona-driven SBR via HKG, arXiv:2604.06928 | Heterogeneous KG for persona modeling: HDGI objective over KG initialized with LLM-derived item embeddings. Two-stage: persona extraction + utilization with reranking |
| S14 | Dimos, EAAI 2026 | Diffusion model for SBR: dual-branch (exploring implicit + exploiting explicit). Bi-MaKAN backbone. 94.32% GPU memory reduction, 98.80% inference time reduction |

### Multi-Objective Optimization (2026)

| ID | Source | Key Contribution |
|----|--------|-----------------|
| S15 | CPFR-MOIP, KDD 2025/ACM (Nov 2025) | Multi-sided fairness: similarity-based individual fairness + consistent product-side fairness + variety-seeking levels as moderating factor. Personalized weights for user/product fairness trade-off |
| S16 | SPDD, arXiv:2607.19357 (Jul 2026, Spotify Research) | Stochastic primal-dual decoding for generative RS: inference-time multi-objective control without retraining. Dynamic Lagrangian multiplier based on constraint slack. +1.8% auxiliary objective at zero engagement cost |
| S17 | DualAgent-Rec, arXiv:2601.19121 (Jan 2026) | LLM-coordinated dual-agent: Exploitation Agent (accuracy under hard constraints) + Exploration Agent (unconstrained Pareto search). 100% hard constraint satisfaction. Adaptive ε-relaxation |
| S18 | Semantic Pareto-DQN, arXiv:2606.24042 (Jun 2026) | Breaking filter bubbles: multi-objective RL with engagement/diversity/fairness as distinct non-aggregable reward signals. Hypervolume-based action selection. Sustains high state-trajectory variance |
| S19 | CVaR Two-Sided Fairness, arXiv:2602.10739 (Feb 2026) | CVaR for consumer group fairness: compresses group-level utility disparities. "Free fairness" disappears in multi-item settings (15-25% utility drop at k=10). Moderate fairness can improve business metrics |
| S20 | Multi-Decoder OneRec, arXiv:2607.26500 (Jul 2026, Kwai) | Quota-aware generative retrieval: shared representations + isolated objective LoRA experts + MD-CBS constrained beam search. Kwai26 benchmark: 1.31B records. 1.69-5.62% Recall improvement |
| S21 | Fair Agents, arXiv:2605.02379 (May 2026) | Social choice theory for multi-stakeholder: Borda/Copeland/Ranked Pairs voting for agent aggregation. Stakeholder-centric evaluation. Domain-specific fairness tensions |

---

## Defects Found

### DEFECT-373-01: No Recommendation Engine in NeoTrix

**Location**: Architecture-wide (no `nt_*_recommend` module exists)

**Gap**: NeoTrix is an AI-native developer toolkit with VSA HyperCube knowledge representation and GWT attention routing, but it has **no recommendation subsystem**. The 2026 landscape shows that LLM-enhanced KG recommendation (S1-S7) is a core capability for any knowledge-rich system. NeoTrix's KB contains nodes, edges, embeddings, and BM25 indices — precisely the substrate needed for knowledge-aware recommendation. The system can retrieve knowledge but cannot recommend next actions, relevant skills, or optimal module configurations based on collaborative or content-based signals.

**Impact**: NeoTrix cannot recommend: (1) which skill to load next based on user behavior, (2) which module configuration is optimal for a task type, (3) which historical experience is most relevant to the current context. The SEAL pipeline explores blindly rather than leveraging collaborative signals from past sessions.

**Suggestion**: Implement `nt_mind::recommendation` with: (1) KG-enhanced CF using KB graph structure (S1/S2 pattern), (2) test-time reasoning module that adapts recommendations without retraining (KGERA's 10ms inference), (3) frequency-aware dual-channel fusion (DCGL's S3 pattern) for cold-start vs active module recommendation.

---

### DEFECT-373-02: VSA HyperCube Cannot Represent Heterogeneous Multi-Relational Graphs

**Location**: `nt_core_hcube::vsa` (entire VSA module)

**Gap**: The VSA HyperCube maps concepts to high-dimensional vectors using bind/bundle/permute operations on undifferentiated vectors. The 2026 research (S2, S3, S5) demonstrates that knowledge-aware recommendation requires **heterogeneous multi-relational graphs**: typed node hierarchies, relation-specific attention, category co-membership, creator-level associations, data-driven similarity links. KGERA (S2) constructs a unified KG encoding diverse item relationships and performs structured reasoning with profile similarity, genre/creator affinity, complementarity, and substitutability — all as distinct relation types. DCGL (S3) structurally decouples semantic information from behavioral patterns into parallel channels. NeoTrix's VSA treats all bindings identically regardless of relation type.

**Impact**: The HyperCube cannot distinguish between "is-a", "causes", "co-occurs-with", "temporal-precedes", or "analogous-to" relations. All knowledge is flattened into a single vector space, losing the structured multi-relational information that makes KG-enhanced recommendation effective.

**Suggestion**: Extend VSA HyperCube with **relation-typed binding**: each edge type gets a distinct binding operator or attribute vector. Implement `TypedBinding { relation: RelationType, subject: V, object: V } → V` where `RelationType` is an enum (IsA, Causes, CoOccurs, TemporalPrecedes, AnalogousTo, ...). This enables KG-style multi-relational reasoning while preserving VSA's compositional properties. Wire to KB edge schema (add `relation_type TEXT` column per iteration_batch_336 D2).

---

### DEFECT-373-03: GWT Has No Per-User Personalization Adaptation

**Location**: `nt_core_gwt` (attention routing)

**Gap**: GWT broadcasts salient information across specialist modules via resonance-based routing. The salience computation is global — identical for all users/tasks. Sager (S8) demonstrates that **personalizing the reasoning process itself** (not just what the system knows) provides qualitatively distinct improvement. S8 shows a "one-policy-fits-all" bottleneck: when recommendation fails, the agent updates memory but never interrogates the decision logic. The 2026 paradigm is per-user policy evolution. NeoTrix's GWT uses a universal attention policy shared identically across all sessions.

**Impact**: NeoTrix allocates attention the same way for every task, regardless of the user's historical preference patterns, expertise level, or working style. A novice and an expert get the same module activation pattern, wasting attention bandwidth on irrelevant specialists.

**Suggestion**: Implement **GWT policy skills** per user/session (S8 pattern): (1) Per-session `AttentionPolicy` struct encoding personalized routing weights, (2) incremental contrastive update from task outcomes (accepted vs rejected module activations), (3) slim injection variant for inference-time efficiency (S8's two-representation architecture). Wire to `nt_core_self::AttentionManager` (dual specialization routing).

---

### DEFECT-373-04: No Session-Based Recommendation Architecture

**Location**: NT-MEMORY (knowledge retrieval) + NT-IO (user interaction)

**Gap**: NeoTrix processes sessions but has no session-based recommendation (SBR) architecture. The 2026 SBR research is rich: NP-MiSR (S10) learns adaptive multi-interest per session via Neural Processes; GraphFine (S11) captures multi-granular behavioral patterns; SPRINT (S12) achieves scalable intent refinement without LLM dependency at inference; Dimos (S14) achieves 98.80% inference time reduction via dual-branch diffusion. These methods handle the fundamental SBR challenges: anonymous sessions, data sparsity, cold-start, and short interaction sequences. NeoTrix treats each session independently with no cross-session knowledge transfer.

**Impact**: When a user starts a new session, NeoTrix has no mechanism to: (1) infer latent personas from session history, (2) model multiple concurrent interests within a session, (3) transfer behavioral patterns from similar past sessions, (4) adaptively determine how many interests to model per session.

**Suggestion**: Implement `nt_memory::session_recommender` with: (1) Neural Process-based interest modeling (S10) for adaptive interest count, (2) cross-session context fusion via similar session retrieval from KB, (3) uncertainty-aware intent inference (SPRINT's S12 global intent pool pattern) for cold-start sessions. Wire to `nt_core_self::AttentionManager` for session-aware routing.

---

### DEFECT-373-05: No Multi-Objective Optimization for Attention/Resource Allocation

**Location**: GWT attention routing + ResourceBudgetManager

**Gap**: NeoTrix's GWT optimizes for a single salience score (urgency × novelty × coherence). The ResourceBudgetManager handles cost but has no multi-objective formulation. The 2026 research demonstrates that production recommendation systems must simultaneously optimize accuracy, diversity, fairness, and novelty: CPFR-MOIP (S15) introduces personalized multi-sided fairness with variety-seeking moderation; SPDD (S16) achieves inference-time multi-objective control via stochastic primal-dual decoding; DualAgent-Rec (S17) guarantees 100% hard constraint satisfaction via dual-agent architecture; Pareto-DQN (S18) treats engagement/diversity/fairness as non-aggregable reward signals.

**Impact**: NeoTrix's attention routing is single-objective. When multiple modules compete for attention, the system has no principled way to balance: (1) immediate task relevance vs long-term capability building, (2) accuracy vs exploration of novel approaches, (3) efficiency vs diversity of perspectives. The "Dark Forest" rule (compile+test+connect or delete) is binary, not Pareto-optimal.

**Suggestion**: Implement `nt_core_gwt::multi_objective_router` with: (1) Pareto front tracking for competing objectives (relevance, diversity, novelty, cost), (2) stochastic primal-dual decoding (S16) for inference-time trade-off adjustment without retraining, (3) hard constraint support (S17's adaptive ε-relaxation) for non-negotiable requirements (R-P1 safety, budget limits). Replace single salience scalar with multi-objective reward vector.

---

### DEFECT-373-06: No Fairness/Diversity Mechanisms in Knowledge Allocation

**Location**: KB namespace allocation + GWT specialist activation

**Gap**: NeoTrix allocates KB namespaces per domain and activates specialists via GWT, but has no fairness or diversity guarantees. The 2026 research shows: CVaR optimization (S19) compresses group-level utility disparities that mean/max-min objectives miss; "free fairness" disappears in multi-item settings (15-25% utility drop); moderate fairness constraints can actually improve business metrics by diversifying exposure. Social choice theory (S21) provides formally verified aggregation mechanisms (Borda, Copeland, Ranked Pairs) for multi-stakeholder systems. NeoTrix's 7 domains are stakeholders with competing interests but no formal fairness mechanism.

**Impact**: A single domain (e.g., NT-CORE) could dominate GWT attention, starving other domains of activation. The KB could accumulate knowledge unevenly, with well-represented domains having rich embeddings while underrepresented domains remain sparse. No mechanism detects or corrects this imbalance.

**Suggestion**: Implement `nt_governance::fairness_allocator` with: (1) CVaR-based group fairness (S19) for domain-level attention allocation, (2) social choice aggregation (S21) for multi-domain priority voting, (3) variety-seeking-aware personalization (S15) that adjusts fairness-accuracy trade-offs per user. Add diversity metrics to HeartbeatAggregator: domain activation Gini coefficient, KB embedding density per namespace.

---

### DEFECT-373-07: E8 Reasoning Lacks Test-Time Adaptation

**Location**: `nt_core_e8` (E8 Hexagram reasoning engine)

**Gap**: The E8 Hexagram engine produces deterministic state transitions (+1 Observer classifies as Productive/Oscillating/Stuck/DeadEnd). KGERA (S2) demonstrates that **test-time reasoning** — performing structured reasoning at inference without retraining — achieves 52.83% NDCG@10 improvement. The key insight: reasoning should adapt dynamically to evolving signals without model updates. NeoTrix's E8 transitions are fixed-state machines; the Observer recommends meta-state transitions but cannot modify the reasoning strategy based on accumulated evidence within a session.

**Impact**: When the E8 engine enters a "Stuck" state, the Observer recommends a state transition but cannot dynamically adjust the reasoning parameters (abstraction level, scope, depth) based on what worked in similar past situations. The engine restarts from the new state rather than adapting its transition rules.

**Suggestion**: Add **test-time adaptation** to E8 (KGERA pattern): (1) Maintain a lightweight reasoning ensemble (multiple E8 parameter configurations), (2) at each step, select the configuration that best fits the current context via a learned meta-learner (KGERA's stacking ensemble), (3) the meta-learner weights are interpretable (positive = primary contributor, negative = debiaser). This enables dynamic adaptation without retraining the base E8 model.

---

### DEFECT-373-08: SEAL Pipeline Lacks Selective Knowledge Augmentation

**Location**: `nt_mind` (SEAL pipeline)

**Gap**: The SEAL pipeline runs exploration → distillation → self-test → absorption uniformly. KnowSA (S7) demonstrates that **selective augmentation** — estimating the system's internal knowledge per item and injecting external information only where needed — outperforms uniform augmentation. KnowSA introduces Comparative Knowledge Probing: quantifying the model's capability to comparatively rank items based on collaborative patterns, not just semantic familiarity. NeoTrix's SEAL pipeline treats all knowledge gaps equally, wasting exploration budget on areas where the system already has sufficient knowledge.

**Impact**: SEAL explores the entire capability space uniformly. Resources are wasted exploring well-understood domains while under-explored domains receive insufficient attention. The pipeline cannot distinguish between "I know this well" and "I have a knowledge gap here."

**Suggestion**: Implement **Knowledge Gap Detection** in SEAL (KnowSA pattern): (1) For each potential exploration target, estimate NeoTrix's internal knowledge via comparative probing (can the system correctly rank related capabilities?), (2) prioritize exploration of low-knowledge targets, (3) skip exploration of well-understood areas. This transforms SEAL from uniform exploration to targeted knowledge-gap filling.

---

## Priority Matrix

| Defect | Severity | Effort | Priority |
|--------|----------|--------|----------|
| D373-01: No recommendation engine | High | High | P1 — Foundation for all other improvements |
| D373-02: VSA heterogeneous graphs | High | Medium | P1 — Required for KG-enhanced reasoning |
| D373-03: GWT personalization | Medium | Medium | P2 — Enables user-adaptive attention |
| D373-04: Session-based rec | Medium | High | P2 — Critical for cross-session learning |
| D373-05: Multi-objective optimization | High | High | P1 — Required for production-grade routing |
| D373-06: Fairness/diversity | Medium | Medium | P2 — Required for multi-domain equity |
| D373-07: E8 test-time adaptation | Medium | Low | P2 — High ROI, low implementation cost |
| D373-08: Selective knowledge augmentation | Medium | Low | P2 — Direct SEAL efficiency improvement |

---

## Strategic Synthesis

The 2026 research reveals a convergent theme: **the shift from static, one-size-fits-all systems to adaptive, personalized, multi-objective architectures**. NeoTrix's VSA HyperCube + GWT + E8 foundation is architecturally sound but operates in a single-objective, user-agnostic, relation-agnostic regime. The three research streams (recommender systems, personalization, multi-objective) share a common pattern:

1. **Heterogeneous multi-relational graphs** replace flat vector spaces (S2, S3, S5, S13)
2. **Per-user/per-session policy evolution** replaces universal static policies (S8, S9, S12)
3. **Inference-time adaptation** replaces train-then-deploy (S2, S16, S17)
4. **Formal fairness constraints** replace ad-hoc regularization (S15, S17, S19, S21)

These four patterns map directly to NeoTrix's four architectural gaps: VSA needs relation typing (D373-02), GWT needs personalization (D373-03), E8 needs test-time adaptation (D373-07), and resource allocation needs multi-objective fairness (D373-05, D373-06). Implementing these would transform NeoTrix from a static knowledge engine to an adaptive, personalized, fair reasoning platform.
