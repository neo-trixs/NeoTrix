# Model Reverse Engineering — Cycle 333

**Date**: 2026-09-11  
**Focus**: Efficient inference, attention mechanisms, agent coordination, memory architectures  
**Previous cycles**: 318–332 (excluded)

---

## Paper 1: AGAO — Adaptive Goal-aware Attention Orchestration for Multi-Agent Graph Systems

**URL**: https://arxiv.org/html/2607.23678v1  
**Authors**: Fan et al.  
**Date**: 2026-07-26

### Core Idea
Attention Engineering as a new paradigm — extending attention from token-level representation learning to workflow-level agent coordination. Three complementary mechanisms:
- **Goal-aware Attention**: Measures semantic relevance between user objectives and agent capabilities
- **Topology-aware Attention**: Incorporates graph structural dependencies and execution paths into attention estimation
- **Resource-aware Attention**: Translates attention scores into execution decisions — model selection, token budget allocation, agent activation priority

Transforms static agent graphs into adaptive execution systems that focus on goal-critical reasoning paths.

### Key Results
- Improves task effectiveness while reducing unnecessary computation, latency, and token consumption
- Establishes "Attention Engineering" as a new direction for multi-agent graph systems
- Dynamic execution graphs instead of static topology

### NeoTrix Domain Mapping

| Domain | Integration |
|--------|------------|
| **NT-CORE (GWT)** | Goal-aware attention → salience scoring incorporates user objective alignment; topology-aware attention → GWT routing considers module dependency graph |
| **NT-ACT** | Resource-aware attention → dynamic capability activation based on task demands |
| **NT-MIND** | Attention as execution control → SEAL pipeline stage-dependent resource allocation |
| **NT-MEMORY** | Dynamic graph execution → experience-tree branch activation based on goal relevance |

### NeoTrix Application
**GWT with goal-aware salience**: Currently GWT computes salience from content alone. AGAO's goal-aware attention adds a second signal: how well does this information serve the current objective? This maps to the `instruction` parameter in `consciousness_task` — GWT should weight broadcasts by goal alignment, not just content salience.

**Attention as execution control**: AGAO's key insight — attention at the orchestration layer, not just the representation layer. NeoTrix's GWT currently operates at representation level. Extending it to workflow-level would enable dynamic capability tree traversal based on attention scores, rather than static traversal.

---

## Paper 2: PSMAS — Phase-Scheduled Multi-Agent Systems for Token-Efficient Coordination

**URL**: https://arxiv.org/abs/2604.17400  
**Authors**: Dubey  
**Date**: 2026-04-19

### Core Idea
Reconceptualizes multi-agent coordination as continuous control over shared attention space on the circular manifold S¹. Each agent assigned a fixed angular phase θᵢ ∈ [0, 2π). A global sweep signal φ(t) rotates at velocity ω, activating only agents within angular window ε. Idle agents receive compressed context summaries.

Key theoretical contribution: S¹ is the **necessary** topology (not convenient) because:
1. Boundary effects and aperiodicity — linear scheduling has hard endpoints
2. No natural proximity metric on linear scheduling
3. No connection to stability theory — S¹ connects to Kuramoto model for synchronization analysis

### Key Results
- Mean token reduction of 27.3% (range 21.4–34.8%) across 4 benchmarks
- Within 2.1pp of full-activation baseline performance
- 5.6pp better token reduction than strongest learned routing baseline (RouterLLM-RA)
- Proven stability (Theorem 7.3) and convergence (Theorem 7.5) guarantees

### NeoTrix Domain Mapping

| Domain | Integration |
|--------|------------|
| **NT-CORE (GWT)** | Phase-scheduled activation → GWT attention windows; sweep signal → consciousness cycle timing |
| **NT-ACT** | Agent activation priority based on phase — only activated agents consume tokens |
| **NT-MEMORY** | Idle agent context compression → experience-tree branch summaries for inactive domains |
| **NT-MIND** | Continuous control system → SEAL pipeline timing as continuous, not discrete |
| **NT-PHYSICAL** | Circular manifold → periodic task scheduling for physical embodiment loops |

### NeoTrix Application
**Consciousness cycle as sweep signal**: PSMAS's φ(t) maps directly to NeoTrix's `consciousness_tick` cycle. Currently each cycle processes all branches uniformly. PSMAS shows that activating only phase-relevant branches per cycle, with compressed summaries for idle branches, preserves performance at 27% lower token cost.

**S¹ topology for experience cycles**: The experience-tree 5-stage absorption runs linearly. PSMAS's circular manifold argument suggests modeling it as periodic — each stage's output naturally feeds into the next cycle's input without restart overhead.

---

## Paper 3: Codebook Agent — Amortized Topology Design for LLM Multi-Agent Systems

**URL**: https://arxiv.org/abs/2609.02264  
**Date**: 2026-09-02

### Core Idea
Three empirical findings that invalidate current topology design approaches:
1. Topologies collapse to ~6 distinct graphs even with codebook capacity 8→64
2. Edge count is **negatively** correlated with token consumption (r ≈ −0.4) — sparsifying makes inference MORE expensive
3. Message-passing scorers are adjacency-invariant when agents share profiles — cannot rank candidates

Codebook Agent: vector-quantized autoencoder compresses successful topologies into query-independent 16-entry codebook. Reward-weighted MLP maps query embedding to code distribution. MLP proxy reranks top decoded candidates.

### Key Results
- 84.6 average accuracy (vs 83.0 strongest prior)
- Topology emitted in 2.4ms (no iterative search, no message passing)
- 21.9–33.2% fewer LLM tokens than baselines
- 16-entry codebook sufficient for all evaluated tasks

### NeoTrix Domain Mapping

| Domain | Integration |
|--------|------------|
| **NT-CORE** | Codebook as capability topology memory — reusable execution patterns |
| **NT-ACT** | Query-to-topology mapping → capability routing via learned pattern library |
| **NT-MIND** | Evidence-backed design rules (Preserve/Modify/Avoid) → SEAL evolution rules |
| **NT-MEMORY** | 16-entry codebook → compact experience index (not full-text, just topology patterns) |

### NeoTrix Application
**Experience index as codebook**: Codebook Agent's finding — 16 entries suffice for all evaluated tasks — suggests the experience-tree hub index can be extremely compact. Instead of loading full experience summaries, load 16 canonical execution patterns. The reward-weighted MLP maps current task to the most relevant pattern.

**Topology as retrievable skill**: Codebook Agent treats topology as a "retrievable and self-improving design skill." This maps to NeoTrix's skill tree concept — each skill is not just capability, but a retrievable execution topology. The Preserved/Modified/Avoided actions from QueenBee (related work) map directly to experience-tree branch management.

---

## Paper 4: NeuralFSM — Adaptive Multi-Agent Coordination via Finite-State Execution Policy

**URL**: https://aclanthology.org/2026.acl-long.1543.pdf  
**Date**: 2026 (ACL)

### Core Idea
Multi-agent problem solving as finite-state execution process. Key innovations:
- **Temporal Coordination Controller**: Uses Temporal Graph Networks (TGN) to learn state transitions and inter-agent communication weights from interaction traces
- **FSM-based coordination**: Reusable finite state machine structure captures shared execution regularities across tasks
- **Dual-Defense Protection**: Training-time graph regularization + runtime trust-aware message attenuation against adversarial agents

### Key Results
- 6.74%–19.39% improvement over baselines across 6 benchmarks
- Substantial token reduction via sparse routing
- Only 1.82% performance drop under adversarial attack (vs significant degradation without protection)
- Strong inherent robustness even without protection layer

### NeoTrix Domain Mapping

| Domain | Integration |
|--------|------------|
| **NT-CORE** | FSM state transitions → consciousness tree stage transitions with learned policies |
| **NT-ACT** | Communication routing weights → capability activation probability |
| **NT-SHIELD** | Dual-defense protection → trust-aware message filtering; adversarial agent detection |
| **NT-MEMORY** | TGN node memories → persistent module interaction history in KB |
| **NT-MIND** | Reusable FSM structure → SEAL pipeline coordination patterns |

### NeoTrix Application
**Consciousness tree as FSM**: NeuralFSM's reusable FSM structure maps to NeoTrix's ConsciousnessTree 6-stage loop (Soil→Roots→Trunk→Branches→Fruits→Core). Currently transitions are deterministic. NeuralFSM shows learned transition distributions improve performance significantly.

**Trust-aware communication in GWT**: The dual-defense protection layer (centrality analysis + online anomaly scoring) directly applies to GWT broadcast filtering. Not all module broadcasts should be equally weighted — trust-aware attenuation prevents adversarial or low-quality modules from polluting the global workspace.

---

## Paper 5: QueenBee Planner — Skill-Evolving Communication Topologies for Token-Efficient Multi-Agent Systems

**URL**: https://arxiv.org/abs/2606.27492  
**Date**: 2026-06-25

### Core Idea
Treats inter-agent communication topology as a **retrievable and self-improving design skill**. Outer LLM planner generates temporal communication DAGs (who sends to whom, in which round, who merges, who emits). Execution traces distilled into evidence-backed design rules with three actions:
- **Preserve**: Keep successful patterns
- **Modify**: Adapt patterns for new contexts
- **Avoid**: Reject patterns that failed

Self-evolution prevention: held-out acceptance gates, variance-aware credit, motif-level attribution, transfer trust, insight falsification, structural deduplication.

### Key Results
- Best generated graph reduces RMSE from 12.53 to 7.87 vs strongest fixed topology
- Simultaneously reduces messages, model calls, and token cost
- Self-evolved graph generation outperforms cold generation and fixed topologies

### NeoTrix Domain Mapping

| Domain | Integration |
|--------|------------|
| **NT-MIND** | Design rules (Preserve/Modify/Avoid) → SEAL evolution rules for skill crystallization |
| **NT-MEMORY** | Retrievable topology as skill → experience-tree branches as reusable execution patterns |
| **NT-ACT** | Temporal DAG generation → dynamic capability routing per task |
| **NT-GOVERNANCE** | Self-evolution prevention mechanisms → prevent "lucky run" overfitting in experience absorption |
| **NT-CORE** | Motif-level attribution → E8 reasoning pattern attribution |

### NeoTrix Application
**Experience-tree as skill-evolving topology**: QueenBee's core insight — topology as a retrievable, self-improving skill — directly maps to NeoTrix's skill tree concept. Each skill node is not just a capability, but a communication topology (which modules talk to which, in what sequence). The Preserve/Modify/Avoid actions map to experience-tree branch management.

**Anti-self-deception for experience absorption**: QueenBee's self-evolution prevention (acceptance gates, variance-aware credit, falsification) directly addresses a real risk in NeoTrix's experience-tree absorption: "lucky runs" being crystallized as skills. The experience吸收协议 needs these mechanisms to prevent false pattern recognition.

---

## Cross-Paper Patterns

| Pattern | Papers | NeoTrix Mapping |
|---------|--------|-----------------|
| **Attention as orchestration control** | AGAO, PSMAS | GWT should operate at both representation AND workflow levels |
| **Topology as retrievable skill** | Codebook Agent, QueenBee | Skill tree nodes are execution topologies, not just capabilities |
| **Continuous control over discrete scheduling** | PSMAS, NeuralFSM | ConsciousnessTree transitions should be continuous/learned, not deterministic |
| **Anti-self-deception in evolution** | QueenBee, NeuralFSM | Experience absorption needs acceptance gates to prevent false patterns |
| **Phase-based activation** | PSMAS, AGAO | GWT attention windows activated by phase, not blanket broadcasting |
| **Trust-aware communication** | NeuralFSM, AGAO | Module broadcasts filtered by trust scores and goal relevance |

---

## Action Items for NeoTrix

| Priority | Action | Source Paper | Target Domain |
|----------|--------|-------------|---------------|
| P0 | Add goal-aware salience to GWT (objective alignment weighting) | AGAO | NT-CORE |
| P0 | Implement Preserve/Modify/Avoid rules for experience-tree absorption | QueenBee | NT-MIND |
| P1 | Phase-scheduled consciousness cycle (activate only phase-relevant branches) | PSMAS | NT-CORE |
| P1 | Compact 16-entry codebook for experience hub index | Codebook Agent | NT-MEMORY |
| P1 | Trust-aware GWT broadcast filtering | NeuralFSM | NT-SHIELD |
| P2 | FSM-based consciousness tree transitions (learned, not deterministic) | NeuralFSM | NT-CORE |
| P2 | Anti-self-deception gates for experience absorption | QueenBee | NT-MIND |
| P2 | S¹ topology for experience cycle modeling | PSMAS | NT-MEMORY |
| P3 | Resource-aware attention for dynamic capability activation | AGAO | NT-ACT |
