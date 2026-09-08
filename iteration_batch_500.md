# Iteration Batch 500 — Research Loop

**Date**: 2026-09-06
**Domains**: Autonomous Agents | Multi-Agent Coordination | Agent Memory
**Method**: External research → Gap analysis → Defect identification → Suggestions

---

## 1. Sources Cited

### Autonomous Agents
1. **Holistic Review of Agentic AI** (Springer, 2026-06-15) — https://link.springer.com/article/10.1007/s11831-026-10675-8
2. **Auton Agentic AI Framework** (arXiv:2602.23720) — https://www.arxiv.org/pdf/2602.23720 — Cognitive Blueprint/Runtime Engine separation, POMDP+latent reasoning, constraint manifold safety, hierarchical memory consolidation
3. **System-Theoretic Agent Architecture** (arXiv:2601.19752) — https://arxiv.org/pdf/2601.19752v1 — 5 functional subsystems, 12 Agentic Design Patterns
4. **InfiAgent** (ACL 2026 Findings) — https://aclanthology.org/2026.findings-acl.1787.pdf — File-centric state externalization, bounded reasoning, hierarchical DAG decomposition
5. **AutoAgent (zero-code)** (ACL 2026 Findings) — https://aclanthology.org/2026.findings-acl.2129.pdf — Self-managing file system, self-play agent customization
6. **Life-inspired Interoceptive AI** (Nature Machine Intelligence, 2026-08-26) — https://www.nature.com/articles/s42256-026-01296-8 — Interoception for homeostasis, EVAAA benchmark
7. **AutoAgent (self-evolving)** (arXiv:2603.09716) — https://arxiv.org/pdf/2603.09716 — Evolving cognition, on-the-fly contextual decisions, Elastic Memory Orchestrator, cognitive evolution loop

### Multi-Agent Coordination
8. **LLM-GNCF** (Springer, 2026-06-08) — https://link.springer.com/article/10.1007/s40747-026-02356-7 — LLM-guided Team-Adaptive Coordination Graph, latent reward shaping
9. **Codebook Agent** (arXiv:2609.02264, 2026-09-02) — https://arxiv.org/abs/2609.02264 — Vector-quantized topology codebook, 21-33% token savings
10. **HHC: Hierarchical Hypergraph Communication** (UAI 2026) — https://proceedings.mlr.press/v337/liang26a.html — Dual-layer hypergraph, overlapping multi-group membership
11. **Guided Topology Diffusion (GTD)** (ACL 2026) — https://aclanthology.org/2026.acl-long.1764.pdf — Conditional graph diffusion for dynamic topology synthesis
12. **CAIC: Congestion-Aware Intent Communication** (UAI 2026) — https://proceedings.mlr.press/v337/li26h.html — Queueing-aware communication, temporal masked autoencoder
13. **Civilization Framework** (arXiv:2609.03425, 2026-09-03) — https://arxiv.org/abs/2609.03425 — Sovereign-anchored inter-system communication, temporal-weight hazard
14. **NeuralFSM** (ACL 2026) — https://aclanthology.org/2026.acl-long.1543.pdf — FSM-based coordination, TGN temporal controller, trust-aware message attenuation
15. **Consilience** (arXiv:2608.20564, 2026-08-20) — https://arxiv.org/abs/2608.20564 — Conformal calibration for communication control, certified regret bounds

### Agent Memory
16. **AgeMem: Agentic Memory** (ACL 2026) — https://aclanthology.org/2026.acl-long.981/ — Unified LTM+STM via tool actions, 3-stage progressive RL, step-wise GRPO
17. **EverMemOS** (ACL 2026) — https://aclanthology.org/2026.acl-long.2125.pdf — Engram lifecycle: MemCell→MemScene→Reconstructive Recollection
18. **RippleMem** (arXiv:2608.13334) — https://arxiv.org/html/2608.13334v1 — Cue-dependent episodic retrieval, associative recollection, event-centric memory graph
19. **SEEM: Structured Episodic Event Memory** (ACL 2026) — https://aclanthology.org/2026.acl-long.277.pdf — Episodic Event Frames + Graph Memory Layer, Reverse Provenance Expansion
20. **VerMem: Verifiable Memory** (arXiv:2608.03137) — https://arxiv.org/html/2608.03137 — 7 atomic operations, local+global verifiers, 3-stage RL curriculum
21. **MemWeaver** (ACL 2026 Findings) — https://aclanthology.org/2026.findings-acl.630.pdf — Tri-layer: graph+experience+passage, dual-channel retrieval

---

## 2. Defects Identified in NeoTrix Design

### D1: No Declarative Agent Specification (Cognitive Blueprint gap)
**Source**: Auton Framework (#2), AutoAgent (#5)
**Evidence**: NeoTrix agents are defined imperatively in Rust code. No YAML/JSON "Cognitive Blueprint" exists that declaratively specifies agent identity, tools, memory config, safety constraints, I/O contracts.
**Impact**: Agents cannot be versioned, diffed, audited, or ported across runtimes. Cross-language portability impossible. No formal auditability trail for agent configurations.
**Defect Location**: `neotrix-core/src/` — no agent specification schema layer.

### D2: No Latent Reasoning Space (Think-before-Act missing at architecture level)
**Source**: Auton Framework (#2), System-Theoretic Architecture (#3)
**Evidence**: NeoTrix has no explicit separation between internal deliberation and external action. The E8 Hexagram provides reasoning states but not a factorized policy that enforces a think-before-act invariant. The system-theoretic architecture demands 5 subsystems (Reasoning & World Model, Perception & Grounding, Action Execution, Learning & Adaptation, Inter-Agent Communication) — NeoTrix maps partially but has no formal separation.
**Impact**: Actions may be emitted without deliberation. No POMDP formalism for analyzing agent behavior. Latent reasoning cannot be introspected or audited.
**Defect Location**: `l5_cognition/nt_core/` — reasoning is stateless per step, no latent buffer.

### D3: No Interoceptive State (Homeostatic drive missing)
**Source**: Life-inspired Interoceptive AI (#6)
**Evidence**: NeoTrix has EmotionLabel (11 variants) and NT-FEEL for emotional regulation, but no interoceptive state factorization — no internal homeostatic variables (energy, fatigue, uncertainty, novelty) that serve as universally available reference signals modulating learning and behavior.
**Impact**: Emotions are reactive, not grounded in internal state. No intrinsic motivation signal. Adaptivity limited to external task demands, not self-regulated homeostasis.
**Defect Location**: `l4_emotion/nt_feel/` — EmotionLabel is categorical, not grounded in continuous state variables.

### D4: No Adaptive Communication Topology (Static GWT routing)
**Source**: Codebook Agent (#9), GTD (#11), NeuralFSM (#14)
**Evidence**: NeoTrix's GWT uses fixed resonance-based routing. 2026 research shows dynamic topology generation (codebook/vector-quantized, diffusion-based, FSM-driven) significantly outperforms static topologies by 21-33% token savings and 2-8% accuracy gains.
**Impact**: Broadcast routing wastes tokens on irrelevant modules. No task-adaptive specialization. Communication cost scales linearly with module count instead of optimizing critical path.
**Defect Location**: `core/gwt/` — routing is resonance-based, not query-conditioned.

### D5: No Congestion-Aware Communication (Queueing delay ignored)
**Source**: CAIC (#12)
**Evidence**: NeoTrix assumes instantaneous message delivery via EventBus. Real multi-agent systems face shared-channel queueing delays. Stale messages can be worse than no communication (CAIC ablation).
**Impact**: Under load, EventBus messages arrive stale, potentially causing incorrect coordination. No mechanism to predict future trajectory or encode delay-robust intent.
**Defect Location**: `core/nt_core_heartbeat.rs` — assumes synchronous delivery; no temporal masking.

### D6: No Hypergraph Communication (Flat pairwise only)
**Source**: HHC (#10)
**Evidence**: NeoTrix inter-domain communication is pairwise (domain→domain). HHC shows agents can simultaneously belong to multiple collaborative groups, and hypergraph encoding captures overlapping memberships that pairwise graphs miss.
**Impact**: Cross-domain coordination that involves 3+ domains simultaneously (e.g., NT-WORLD+NT-ACT+NT-MEMORY pipeline) cannot be expressed as a single hyperedge. Forces sequential pairwise calls, losing parallelism.
**Defect Location**: `l6_meta/nt_nexus/` — Nexus tracks pairwise domain relations, not hyperedges.

### D7: No File-Centric State Externalization (Context-centric reasoning)
**Source**: InfiAgent (#4)
**Evidence**: NeoTrix stores state in KB (SQLite) and in-memory. InfiAgent demonstrates that file-centric state externalization with periodic consolidation keeps reasoning context strictly bounded regardless of task duration. NeoTrix has no bounded-context reconstruction mechanism — context grows with session length.
**Impact**: Long-horizon tasks degrade as context accumulates. No checkpoint/recovery mechanism. Agent cannot reconstruct reasoning from a fixed-size workspace snapshot.
**Defect Location**: `l1_action/nt_memory/` — no file-centric state abstraction layer.

### D8: No Hierarchical Memory Consolidation (No engram lifecycle)
**Source**: EverMemOS (#17), AgeMem (#16), SEEM (#19), VerMem (#20), MemWeaver (#21)
**Evidence**: NeoTrix KB stores entities/edges/embeddings but lacks the three-phase memory lifecycle now standard in 2026:
- **Episodic Trace Formation** → MemCells (atomic episodic units with time-bounded foresight)
- **Semantic Consolidation** → MemScenes (thematic clusters with conflict detection)
- **Reconstructive Recollection** → necessity-and-sufficiency guided retrieval

NeoTrix's experience-tree absorption is a write-path process but has no structured read-path lifecycle.
**Impact**: Memory retrieval is flat (BM25/embedding similarity). No association chains for evidence completion. No conflict detection between episodic memories. No time-bounded foresight expiry.
**Defect Location**: `nt_memory/` — no MemCell/MemScene structures; KB lacks engram lifecycle.

### D9: No Associative Recollection (One-shot retrieval only)
**Source**: RippleMem (#18), SEEM (#19)
**Evidence**: NeoTrix uses one-shot query→retrieval. RippleMem shows that adaptive associative recollection — where initially recalled memories serve as cues for finding additional evidence — significantly outperforms one-shot retrieval (+3.95% to +11.87%).
**Impact**: When evidence is distributed across multiple interactions, NeoTrix retrieves isolated fragments instead of completing the evidence chain. Multi-hop temporal reasoning fails.
**Defect Location**: `nt_memory/kb/` — no cue-dependent expansion mechanism.

### D10: No Memory Operation Policy (Heuristic memory management)
**Source**: AgeMem (#16), VerMem (#20)
**Evidence**: NeoTrix memory operations (store/retrieve/delete) are triggered by code, not learned. AgeMem and VerMem demonstrate that exposing memory as 7 atomic tool actions trained via RL (SFT warmup + 3-stage progressive RL) significantly outperforms heuristic memory management.
**Impact**: Memory operations cannot adapt to task structure. No learning signal connects early storage decisions with later reasoning quality. Context management is suboptimal.
**Defect Location**: `nt_memory/` — operations are imperative, not policy-learned.

### D11: No Sovereign-Anchored Inter-System Communication
**Source**: Civilization Framework (#13)
**Evidence**: NeoTrix has no protocol for sovereign-to-sovereign communication between independent NeoTrix instances. The temporal-weight hazard (first-arriving claim acquires unearned authority, 54.2% capture rate) is not addressed.
**Impact**: Multi-instance coordination is vulnerable to temporal authority bias. No commitment-state ledger for async inter-civilization messaging. No signed credential authority cap.
**Defect Location**: `l6_meta/nt_nexus/` — no inter-instance protocol; no temporal authority handling.

### D12: No Certified Communication Control (No formal regret bounds)
**Source**: Consilience (#15)
**Evidence**: NeoTrix has no mechanism to certify that a communication action is appropriate before execution. Consilience provides distribution-free, finite-sample regret bounds via conformal calibration. Stale messages under delay are worse than no communication.
**Impact**: No formal guarantee that inter-domain messages are useful. Potential for communication to degrade rather than improve coordination.
**Defect Location**: `l6_meta/nt_meta/` — no conformal calibration layer for communication decisions.

### D13: No Cognitive Evolution Loop (No structured self-improvement from decisions)
**Source**: AutoAgent self-evolving (#7)
**Evidence**: AutoAgent demonstrates a closed-loop: evolving cognition → contextual decisions → memory orchestration → cognitive evolution. NeoTrix has SEAL pipeline for evolution but it operates at the module/macro level, not at per-decision cognitive refinement.
**Impact**: Agent-level decision quality doesn't improve from individual experience. SEAL evolves architecture; nothing evolves decision-making policy.
**Defect Location**: `l5_cognition/nt_mind/` — SEAL is macro-evolutionary, not per-decision.

### D14: No Constraint Manifold Safety (Post-hoc filtering vs projection)
**Source**: Auton Framework (#2)
**Evidence**: NeoTrix's safety is post-hoc (NT-SHIELD audit after actions). Auton shows policy projection onto a Constraint Manifold is architecturally superior — unsafe actions get zero probability during generation, not filtered after.
**Impact**: Safety violations can occur and be caught late. No formal safe subspace enforcement at the policy level.
**Defect Location**: `l3_embodiment/nt_shield/` — post-hoc audit, not policy projection.

---

## 3. Suggestions (Prioritized)

### High Priority (Architectural Gaps)

| # | Suggestion | Addresses | Effort |
|---|-----------|-----------|--------|
| S1 | Introduce `AgentBlueprint` struct — a YAML/JSON-serializable declarative spec for agent identity, tools, memory config, safety constraints. Store in KB `agent_spec` namespace. | D1 | Medium |
| S2 | Factorize agent policy into think/act sub-policies within `nt_core`. Add a `LatentBuffer` trait that holds deliberation state before action emission. | D2 | High |
| S3 | Add `InteroceptiveState` to NT-FEEL: continuous variables (energy, fatigue, novelty, uncertainty) that modulate EmotionLabel thresholds and GWT attention weights. | D3 | Medium |
| S4 | Replace static GWT resonance routing with a `CodebookRouter`: vector-quantized topology lookup conditioned on task embedding. Start with 16-entry codebook. | D4 | Medium |

### Medium Priority (Communication & Safety)

| # | Suggestion | Addresses | Effort |
|---|-----------|-----------|--------|
| S5 | Add temporal masking to EventBus: messages include predicted-validity window; stale messages auto-expire. Implement intent prediction for delay-robust communication. | D5 | Low |
| S6 | Extend Nexus to support hypergraph edges: `HyperEdge { participants: Vec<DomainId>, shared_context: ... }` for multi-domain coordination. | D6 | Medium |
| S7 | Add file-centric state abstraction: `WorkspaceSnapshot` trait that serializes task state to structured files, enabling bounded-context reconstruction. | D7 | Medium |
| S11 | Design inter-instance `EmbassyProtocol`: async message arrival at ledger endpoints, commitment-state tracking, signed credential authority caps, temporal-weight mitigation (sealed answers). | D11 | High |
| S14 | Implement constraint manifold projection in NT-SHIELD: define safe subspaces per action type, project policy before emission instead of post-hoc filtering. | D14 | High |

### Medium Priority (Memory Lifecycle)

| # | Suggestion | Addresses | Effort |
|---|-----------|-----------|--------|
| S8 | Implement engram lifecycle in NT-MEMORY: MemCell (atomic episodic unit with temporal bounds) → MemScene (thematic cluster with conflict detection) → Reconstructive Recollection (necessity-and-sufficiency retrieval). | D8 | High |
| S9 | Add associative recollection: after initial retrieval, expand along semantic+structural edges in an event-centric memory graph. Implement as `RecallExpansion` pass in retrieval pipeline. | D9 | Medium |
| S10 | Expose memory as 7 atomic tool actions (ADD, UPDATE, DELETE, RETRIEVE, SUMMARY, FILTER, SELECT_EPISODE). Train via SFT + 3-stage progressive RL with local/global verifiers. | D10 | High |

### Lower Priority (Evolution & Certification)

| # | Suggestion | Addresses | Effort |
|---|-----------|-----------|--------|
| S12 | Add conformal calibration to communication decisions: at each inter-domain message, bound one-step regret with distribution-free guarantee. Replace ad-hoc message routing with certified interventions. | D12 | Medium |
| S13 | Add per-decision cognitive evolution loop (separate from SEAL macro-evolution): analyze action→outcome pairs, update prompt-level cognition (tool knowledge, self-capabilities, peer expertise). | D13 | Medium |

---

## 4. Summary Statistics

| Metric | Count |
|--------|-------|
| Sources cited | 21 |
| Defects identified | 14 (D1-D14) |
| Suggestions generated | 14 (S1-S14) |
| High-priority suggestions | 5 (S2, S11, S14, S8, S10) |
| Medium-priority suggestions | 6 (S1, S3, S4, S6, S7, S9, S12, S13) |
| Low-priority suggestions | 1 (S5) |

---

## 5. Cross-Domain Impact Matrix

| Defect | NT-CORE | NT-MIND | NT-MEMORY | NT-WORLD | NT-ACT | NT-IO | NT-SHIELD | NT-FEEL |
|--------|---------|---------|-----------|----------|--------|-------|-----------|---------|
| D1 (Blueprint) | ● | ● | | | ● | ● | ● | |
| D2 (Latent) | ● | | | | | | | |
| D3 (Interoception) | | | | | | | | ● |
| D4 (Topology) | ● | | | | | | | |
| D5 (Congestion) | | | | | | | | |
| D6 (Hypergraph) | | | ● | ● | ● | | | |
| D7 (File-state) | | | ● | | ● | | | |
| D8 (Engram) | | | ● | | | | | |
| D9 (Associative) | | | ● | | | | | |
| D10 (Policy) | | ● | ● | | | | | |
| D11 (Sovereign) | | | ● | | | ● | ● | |
| D12 (Certified) | | | | | | | ● | |
| D13 (Cognitive evo) | ● | ● | | | | | | |
| D14 (Constraint) | | | | | | | ● | |

**Most impacted domains**: NT-MEMORY (8 defects), NT-CORE (4), NT-SHIELD (3)
