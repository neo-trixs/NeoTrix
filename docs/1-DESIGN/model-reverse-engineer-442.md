# Model Reverse Engineering — Cycle 442

**Date**: 2026-09-12
**Focus**: 5 recent AI models/papers mapped to NeoTrix 7 domains
**Sources**: arXiv, ACL 2026, ICLR 2026, EMNLP 2026

---

## Model 1: Attention-MoA (Mixture-of-Agents via Inter-Agent Semantic Attention)

**Paper**: arXiv:2601.16596 (Jan 2026)
**Venue**: Preprint
**Impact**: 91.15% LC Win Rate on AlpacaEval 2.0; small ensemble beats Claude-4.5-Sonnet

### Architecture
```
Agent Layer 1: [Model A] [Model B] [Model C]  → outputs
        ↓ Inter-Agent Semantic Attention
Agent Layer 2: [Attended A] [Attended B] [Attended C]  → outputs
        ↓ Inter-layer Residual + Adaptive Early Stopping
Final Synthesis
```

### Core Innovation
- **Inter-Agent Semantic Attention**: Not concatenation or voting — each agent's output attends to all other agents' outputs via learned attention. Deep semantic interaction enables active hallucination correction.
- **Adaptive Early Stopping**: Monitors information degradation across layers; stops before residual connections cause collapse.
- **Residual Synthesis**: Cross-layer residual connections preserve early-layer insights.

### NeoTrix Domain Mapping

| Domain | Integration Point | Pattern |
|--------|-------------------|---------|
| **NT-CORE** | GWT salience broadcast | Semantic attention between specialist modules — each domain module attends to others' outputs before decision |
| **NT-MIND** | SEAL pipeline ensemble | Multiple reasoning paths with semantic attention for consensus, not voting |
| **NT-ACT** | Multi-agent orchestration | Agent teams with inter-agent attention for coordinated action |
| **NT-IO** | LLM provider routing | Provider outputs attend to each other for quality estimation before selection |
| **NT-WORLD** | Content classification | Multiple classifiers attend to each other's confidence signals |
| **NT-SHIELD** | Security screening | Multiple detectors with semantic attention for anomaly correlation |
| **NT-FEEL** | Emotion synthesis | Multiple emotion signals with attention-based integration |

### Implementation Sketch (NT-CORE GWT Refinement)
```
// In nt_core_gwt — replace voting with semantic attention
pub struct SemanticAttentionRouter {
    // Each module's output is a vector
    // Attention matrix: module_i attends to module_j
    attention: Vec<Vec<f32>>,  // [N_modules x N_modules]
    // Adaptive early stop based on information entropy
    entropy_threshold: f32,
}

impl SemanticAttentionRouter {
    pub fn route(&mut self, module_outputs: &[ModuleOutput]) -> GwtDecision {
        // 1. Compute attention weights between all module pairs
        // 2. Apply attention to create attended representations
        // 3. Check entropy — stop if degradation detected
        // 4. Residual connection from previous layer
    }
}
```

### Risk Assessment
- **Complexity**: Medium — attention matrix is O(N²) where N = number of modules (small)
- **Benefit**: High — active hallucination correction is valuable for GWT reliability
- **Priority**: P1 — foundational improvement to attention routing

---

## Model 2: ODAR (Active Inference for Adaptive Routing)

**Paper**: arXiv:2602.23681 (Feb 2026)
**Venue**: Preprint
**Impact**: 98.2% MATH accuracy; 82% cost reduction on open-source stack

### Architecture
```
Query → Difficulty Estimator (amortized active inference)
        ↓
   [Easy] → Fast Agent (heuristic, cheap)
   [Hard] → Slow Agent (deliberative, expensive)
        ↓
   Risk-Sensitive Fusion:
     Minimize Free Energy = -E[log p(y)] + β·Varentropy(q(y))
        ↓
   Final Answer
```

### Core Innovation
- **Amortized Active Inference Difficulty Estimator**: Predicts query difficulty via free-energy minimization, not just classification.
- **Fast/Slow Agent Routing**: Two-tier system — cheap heuristic for easy queries, expensive deliberative for hard ones.
- **Free-Energy Fusion**: Selects answers by minimizing variational free energy = negative log-likelihood + varentropy (epistemic uncertainty). Principled alternative to voting.
- **Varentropy**: Measures spread of answer distribution — high varentropy = agent is uncertain = route to Slow Agent.

### NeoTrix Domain Mapping

| Domain | Integration Point | Pattern |
|--------|-------------------|---------|
| **NT-CORE** | Dual Specialization routing | Free-energy scoring for fast/slow weapon set switching; varentropy for uncertainty-aware attention |
| **NT-CORE** | Axiom A1 (Cost-Aware) | Difficulty estimation → cheap model for easy tasks, expensive for hard |
| **NT-MIND** | SEAL cycle routing | Route exploration tasks to expensive models, exploitation to cheap ones |
| **NT-ACT** | Task orchestration | Difficulty-estimated task routing across agent pool |
| **NT-IO** | Provider selection | Free-energy-based provider fusion instead of voting |
| **NT-FEEL** | Emotional decision-making | Varentropy as emotional uncertainty signal |

### Implementation Sketch (NT-CORE Dual Specialization)
```
// In nt_core_self::attention_manager — replace fixed routing with free-energy
pub struct FreeEnergyRouter {
    // Difficulty estimator: query features → difficulty score
    difficulty_estimator: DifficultyEstimator,
    // Fast/slow agent pool
    fast_agents: Vec<Box<dyn Agent>>,
    slow_agents: Vec<Box<dyn Agent>>,
    // Free-energy fusion parameters
    beta: f32,  // uncertainty weight
}

impl FreeEnergyRouter {
    pub fn route_and_fuse(&self, query: &Query) -> AgentResponse {
        let difficulty = self.difficulty_estimator.estimate(query);

        let responses = if difficulty < self.threshold {
            // Fast path
            self.fast_agents.iter().map(|a| a.run(query)).collect()
        } else {
            // Slow path
            self.slow_agents.iter().map(|a| a.run(query)).collect()
        };

        // Free-energy fusion: minimize -log p(y) + β * varentropy
        self.free_energy_select(responses)
    }
}
```

### Risk Assessment
- **Complexity**: Medium — difficulty estimator needs training, fusion is analytical
- **Benefit**: Very High — principled cost control directly addresses Axiom A1
- **Priority**: P0 — core to cost-aware routing axiom

---

## Model 3: Gated-Memory Routing (Multi-Agent Collaboration)

**Paper**: arXiv:2609.00237 (Aug 2026, EMNLP 2026)
**Venue**: EMNLP 2026 Main Conference
**Impact**: Best average accuracy across 5 benchmarks; 31.9% cost reduction on HumanEval

### Architecture
```
Query + Current Memory State
        ↓
   Memory Write Gate → commits only non-redundant reasoning steps
        ↓
   Execution Memory (compact, informative state)
        ↓
   Retrieval Gate → supplies each agent a relevant subset
        ↓
   Agent Selection (role + backbone) + Execution
        ↓
   Adaptive Halting Controller → stops when evidence sufficient
        ↓
   Answer
```

### Core Innovation
- **Memory Write Gate**: Learns to commit only non-redundant information to execution memory. Prevents history overload.
- **Retrieval Gate**: Provides each agent a compact, relevant subset of memory. No full-history processing.
- **Adaptive Halting Controller**: Monitors memory sufficiency — stops when enough evidence accumulated, not at fixed steps.
- **Evidence Sufficiency Metric**: Information-theoretic measure of whether accumulated memory can answer the query.

### NeoTrix Domain Mapping

| Domain | Integration Point | Pattern |
|--------|-------------------|---------|
| **NT-MEMORY** | Experience-tree write | Gated write: only commit non-redundant experience to KB; prevent noise accumulation |
| **NT-MEMORY** | Session memory retrieval | Retrieval gate: supply each SEAL cycle stage with relevant context subset |
| **NT-CORE** | Reasoning budget | Adaptive halting: stop reasoning when sufficient evidence, don't waste tokens |
| **NT-MIND** | SEAL cycle control | Halting controller: detect when distillation has enough signal to crystallize |
| **NT-ACT** | Multi-agent coordination | Role + backbone selection based on execution memory state |
| **NT-SHIELD** | Threat detection | Evidence accumulation model: halt when threat confidence sufficient |

### Implementation Sketch (NT-MEMORY Experience Tree)
```
// In experience-tree — add gated write
pub struct GatedExperienceWriter {
    // Learns to filter redundant experiences
    write_gate: WriteGate,
    // Tracks what's already in memory
    memory_state: MemoryState,
}

impl GatedExperienceWriter {
    pub fn absorb(&mut self, experience: &Experience) -> AbsorptionDecision {
        // 1. Compare experience against memory_state
        // 2. Write gate decides: commit / skip / merge
        let gate_output = self.write_gate.forward(experience, &self.memory_state);

        match gate_output {
            GateAction::Commit(exp) => {
                self.memory_state.update(&exp);
                AbsorptionDecision::Stored
            }
            GateAction::Merge(exp) => {
                self.memory_state.merge(&exp);
                AbsorptionDecision::Merged
            }
            GateAction::Skip => AbsorptionDecision::Redundant,
        }
    }
}
```

### Risk Assessment
- **Complexity**: Low-Medium — gates are simple learned filters; halting is threshold-based
- **Benefit**: Very High — directly reduces noise in experience-tree, improves retrieval quality
- **Priority**: P0 — fundamental improvement to memory write path

---

## Model 4: REMem (Hybrid Episodic Memory Graph)

**Paper**: arXiv:2602.13530 (Feb 2026, ICLR 2026)
**Venue**: ICLR 2026 Poster
**Impact**: 13.4% absolute gain on episodic reasoning; >90% EM on Test of Time

### Architecture
```
Phase 1: Offline Indexing
  Experience → Gist Extraction (concise summary + parsed timestamp)
             → Fact Extraction (time-scoped triples)
             → Hybrid Memory Graph
                [Gist Nodes] ← temporal links → [Fact Triples]
                [Situational Dimensions: who/where/what/when]
                [Positional Index Encoding]

Phase 2: Online Agentic Inference
  Query → Tool-augmented retriever
        → Iterative graph traversal
        → Episodic recollection OR episodic reasoning
        → Answer
```

### Core Innovation
- **Hybrid Memory Graph**: Gists (human-readable summaries) + Facts (structured triples) + temporal indices. Not just vector store or knowledge graph — unified hybrid.
- **Situational Dimensions**: Each episode grounded in who/where/what/when. Richer than vector similarity.
- **Tool-Augmented Graph Traversal**: Agent uses custom tools (search, traverse, count) to navigate memory graph. Not single retrieval call.
- **Temporal Awareness**: Time-scoped facts enable temporal reasoning (before/after/during).

### NeoTrix Domain Mapping

| Domain | Integration Point | Pattern |
|--------|-------------------|---------|
| **NT-MEMORY** | KB evolution | Hybrid graph (KV + embeddings + temporal) replacing flat KV store |
| **NT-MEMORY** | Experience-tree | Gist + fact dual representation for experiences |
| **NT-CORE** | ConsciousnessTree | Temporal links between growth cycles; situational grounding |
| **NT-WORLD** | Content indexing | Situational dimensions for crawled content |
| **NT-MIND** | Distillation | Gist extraction as distillation mechanism |
| **NT-IO** | Session memory | Tool-augmented retrieval for session context |

### Implementation Sketch (NT-MEMORY Hybrid Evolution)
```
// Evolution path for KB: KV → Hybrid Graph
pub struct HybridMemoryGraph {
    // Existing: KV store + embeddings
    kv_store: KvStore,
    embeddings: EmbeddingIndex,

    // New: Gist nodes + fact triples + temporal links
    gists: Vec<GistNode>,        // human-readable summaries with timestamps
    facts: Vec<FactTriple>,       // (subject, predicate, object, time_range)
    temporal_index: TemporalIndex, // time-based lookup
    situational_dims: SituationalDimensions, // who/where/what/when
}

pub struct GistNode {
    pub content: String,          // concise summary
    pub timestamp: DateTime,      // when this happened
    pub situational: SituationalDimensions,
    pub links: Vec<GistLink>,     // temporal ordering links
}

pub struct FactTriple {
    pub subject: Entity,
    pub predicate: Relation,
    pub object: Entity,
    pub time_range: Option<(DateTime, DateTime)>,  // time-scoped validity
}
```

### Risk Assessment
- **Complexity**: High — requires significant KB schema evolution
- **Benefit**: Very High — temporal reasoning + hybrid graph is a generational leap for KB
- **Priority**: P1 — foundational KB evolution, but large scope; implement incrementally

---

## Model 5: DCPM (Dual-Process Cognitive Memory)

**Paper**: arXiv:2606.09483 (Jun 2026)
**Venue**: Preprint
**Impact**: +5.20 on PersonaMem-v2 for cross-session inference

### Architecture
```
Capability Hierarchy:
  L0: Raw Inputs (perception)
  L1: Atomic Facts (structured extraction)
  L2: Diachronic Belief Trajectories (supersedes chains)
  L3: Identity (who am I over time)
  L4: Domain Schemas (knowledge structures)
  L5: Latent Intentions (implicit goals)
  L6: Cross-Domain Core Patterns (meta-schemas)

Dual Processes:
  System1 (Daytime Writer):
    - Synchronous, fast
    - Records belief revisions as doubly-linked supersedes chains
    - "Event X supersedes Event Y in belief B"

  System2 (Nighttime Engine):
    - Asynchronous, slow
    - Induces schemas from accumulated trajectories
    - Sweeps for cross-domain collisions
    - Abstracts into higher-level core schemas
```

### Core Innovation
- **Supersedes Chains**: Doubly-linked lists tracking belief evolution. "I believed X, then Y supersedes X, then Z supersedes Y." Full provenance.
- **Dual-Process Writer/Consolidator**: Daytime = fast recording; Nighttime = slow schema induction. Matches biological sleep-wake cycle.
- **Cross-Domain Collision Detection**: System2 identifies when domain schemas conflict or reinforce each other.
- **Capability Hierarchy**: 7 levels from raw perception to cross-domain meta-patterns. Progressive abstraction.

### NeoTrix Domain Mapping

| Domain | Integration Point | Pattern |
|--------|-------------------|---------|
| **NT-MEMORY** | Experience-tree evolution | Daytime writer = session capture; Nighttime engine = absorption consolidation |
| **NT-MEMORY** | Belief tracking | Supersedes chains for knowledge versioning |
| **NT-MIND** | Self-evolution | Schema induction as meta-learning |
| **NT-CORE** | ConsciousnessTree | System1/System2 split mirrors fast/slow cognition |
| **NT-FEEL** | Emotional trajectory | Belief evolution tracking for emotional state changes |
| **NT-GOVERNANCE** | Policy evolution | Supersedes chains for policy versioning and provenance |

### Implementation Sketch (NT-MEMORY Dual-Process)
```
// Nighttime consolidation engine
pub struct NighttimeConsolidator {
    // Accumulated trajectories from daytime sessions
    trajectory_buffer: Vec<BeliefTrajectory>,
    // Induced schemas
    schemas: Vec<DomainSchema>,
    // Cross-domain collision detector
    collision_detector: CollisionDetector,
}

impl NighttimeConsolidator {
    pub fn consolidate(&mut self) -> ConsolidationReport {
        // 1. Cluster trajectories by domain
        let clusters = self.cluster_by_domain();

        // 2. For each cluster, induce schema
        for cluster in clusters {
            let schema = self.induce_schema(&cluster);
            self.schemas.push(schema);
        }

        // 3. Detect cross-domain collisions
        let collisions = self.collision_detector.scan(&self.schemas);

        // 4. Abstract into core patterns
        let core_patterns = self.abstract_core_patterns(&self.schemas, &collisions);

        ConsolidationReport { schemas, collisions, core_patterns }
    }
}
```

### Risk Assessment
- **Complexity**: High — schema induction requires significant machinery
- **Benefit**: High — provides provenance tracking and cross-domain synthesis
- **Priority**: P2 — important but complex; implement after gated-memory (Model 3) proves value

---

## Synthesis: Cross-Paper Patterns

### 1. Memory is Becoming Cognitive
All three memory papers (Gated-Memory, REMem, DCPM) move beyond flat KV/vector stores toward **structured, temporal, hierarchical** memory systems. NeoTrix KB should evolve in this direction.

**Action**: Plan NT-MEMORY evolution toward hybrid graph (KV + gists + facts + temporal + supersedes chains)

### 2. Routing is Becoming Deliberative
ODAR and Attention-MoA both replace reactive routing (vote/concatenate) with **deliberative routing** (attention/free-energy). This aligns with NeoTrix Axiom A1.

**Action**: Implement free-energy routing in NT-CORE GWT before adding more models

### 3. Dual-Process is the New Default
DCPM (System1/System2), ODAR (Fast/Slow Agent), AgentInfer (AgentCollab/AgentCompress) all use fast/slow splits. NeoTrix already has Dual Specialization — now extend to memory and consolidation.

**Action**: Add nighttime consolidation to experience-tree absorption protocol

### 4. Adaptive Halting Replaces Fixed Budgets
Gated-Memory's halting controller and ODAR's difficulty-based routing both replace fixed token budgets with **adaptive stopping criteria**. More efficient.

**Action**: Add evidence sufficiency metric to SEAL pipeline stage transitions

---

## Priority Matrix

| Priority | Paper | Domain | Action |
|----------|-------|--------|--------|
| P0 | ODAR | NT-CORE | Free-energy routing for Dual Specialization |
| P0 | Gated-Memory | NT-MEMORY | Gated write for experience-tree |
| P1 | Attention-MoA | NT-CORE | Semantic attention for GWT |
| P1 | REMem | NT-MEMORY | Hybrid graph for KB evolution |
| P2 | DCPM | NT-MEMORY | Nighttime consolidation for absorption |

---

## References

1. Wen et al. "Attention-MoA: Enhancing Mixture-of-Agents via Inter-Agent Semantic Attention." arXiv:2601.16596, Jan 2026.
2. Ma et al. "ODAR: Principled Adaptive Routing for LLM Reasoning via Active Inference." arXiv:2602.23681, Feb 2026.
3. Rajib et al. "Gated-Memory Routing for Efficient Collaboration in Multi-Agent LLM Systems." arXiv:2609.00237, Aug 2026. EMNLP 2026.
4. Shu et al. "REMem: Reasoning with Episodic Memory in Language Agent." arXiv:2602.13530, Feb 2026. ICLR 2026.
5. Fei et al. "Memory Beyond Recall: A Dual-Process Cognitive Memory System for Self-Evolving LLM Agents." arXiv:2606.09483, Jun 2026.
