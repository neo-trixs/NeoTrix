# Model Reverse Engineering — Cycle 324

**Date**: 2026-09-11
**Focus**: Exponential decay memory for sparse attention, emergent multi-agent coordination, agentic reasoning with structured memory, adaptive theory-of-mind alignment, centroid-scoring attention for reusable contexts

---

## 5 Models/Papers for Reverse Engineering

### 1. RAT+: Exponentially Decaying Memory for Query-Aware Sparse Attention
- **Paper**: https://arxiv.org/html/2605.28640v1 (May 2026)
- **Key Innovation**: Augments standard attention with a lightweight recurrence that applies exponential decay over KV states. At each time step, key and value states are updated with a decay factor, creating an effective recurrence length of ~64 tokens. When combined with query-aware sparse inference methods (Quest, MoBA, SnapKV), the decay memory consistently improves accuracy across all sparse budgets on needle-in-a-haystack tasks. Validated on released checkpoints AND by continuing pretraining OLMo2-7B with the memory module for 10B tokens. Two hypotheses: (1) decay memory provides a compressed "gist" of distant context that guides sparse selection, (2) it smooths the distribution shift between training (full attention) and inference (sparse attention).
- **Architecture Pattern**: **Memory-augmented sparse attention** — the exponential decay acts as a learned summary of historical context that improves the quality of sparse token selection. The key insight: sparse attention methods lose accuracy because they lack global context when selecting which tokens to attend to. The decay memory provides that global context at near-zero cost (one linear projection per layer). The memory is NOT used for the actual attention computation — it only guides which tokens to select.
- **NeoTrix Domain Mapping**:
  - **NT-CORE (GWT Attention Routing)**: The decay memory pattern directly maps to GWT salience computation — instead of raw token importance, use a decay-augmented summary to determine what broadcasts across the workspace. The "memory guides selection, not computation" principle validates our GWT refinement approach
  - **NT-MEMORY (KB Retrieval)**: Exponential decay as a forgetting mechanism — not all historical context deserves equal weight. The decay factor could modulate experience relevance in our experience-tree, where recent experiences are weighted more heavily for routing decisions
  - **NT-WORLD (Perception Filtering)**: The sparse selection guidance pattern could enhance our PerceptionBridge — use decay memory to filter sensory events based on historical relevance, not just current salience
  - **KVMem Integration**: RAT+ provides a complementary approach to KVMem's paged KV — decay memory reduces the quality degradation from aggressive KV sparsification, potentially enabling even smaller working sets

#### Fusion Opportunities
- **Action**: Integrate exponential decay memory into GWT salience computation — the decay factor becomes a learned parameter that modulates attention broadcast weight
- **Action**: Apply decay-based forgetting to experience-tree relevance scoring — recent experiences get higher routing weight, old experiences fade unless re-activated
- **Action**: Study the "memory guides selection, not computation" pattern as a general principle for lightweight augmentation of existing systems

---

### 2. Emergent Coordination in Multi-Agent Language Models
- **Paper**: https://arxiv.org/abs/2510.05174 (ICLR 2026)
- **Key Innovation**: Information-theoretic framework to test whether multi-agent LLM systems show higher-order structure. Uses mutual information decomposition to measure synergistic information and coordinated differentiation across agents. Three conditions tested: (1) no persona, no ToM — strong temporal synergy but little cross-agent coordination; (2) persona only — introduces stable identity-linked differentiation; (3) persona + ToM prompt ("think about what other agents might do") — shows both identity-linked differentiation AND goal-directed complementarity. Key finding: multi-agent systems can be steered from "mere aggregates" to "genuine collectives" through prompt design. The framework distinguishes spurious temporal coupling from performance-relevant cross-agent synergy.
- **Architecture Pattern**: **Steerable coordination regimes** — the quality of multi-agent coordination isn't fixed; it's a function of prompt design. Three levers: (1) identity assignment (personas), (2) perspective-taking prompts (ToM), (3) task framing. The information-theoretic decomposition provides a principled way to measure whether coordination is real or illusory. Groups differ in variance, stability, and adaptability — properties that matter for deployment reliability.
- **NeoTrix Domain Mapping**:
  - **NT-ACT (Multi-Agent Orchestration)**: The three coordination levers map directly to our faction design — personas = domain faction identity, ToM = cross-faction awareness, task framing = SEAL pipeline goals. The "steerable regimes" concept validates our approach of designing coordination through architecture, not just prompting
  - **NT-CORE (GWT Attention)**: The mutual information decomposition provides a principled measure for GWT salience — not just "is this signal strong?" but "does this signal carry synergistic information across the workspace?" This could replace ad-hoc salience heuristics with information-theoretic grounding
  - **NT-MEMORY (Experience Absorption)**: The "mere aggregates vs genuine collectives" distinction applies to our experience-tree — absorbed experiences should carry coordination-relevant information (how did this work across domains?) not just factual summaries
  - **NT-SHIELD (Audit)**: The information decomposition framework could detect coordination failures — if cross-agent mutual information drops below threshold, the system may be in "aggregate" mode rather than "collective" mode

#### Fusion Opportunities
- **Action**: Implement information-theoretic coordination health metrics — measure synergistic information across faction domains to detect degraded coordination
- **Action**: Use the three-lever model (identity, perspective, framing) as a template for our multi-agent task specification
- **Action**: Apply the variance/stability/adaptability properties to assess faction reliability in production

---

### 3. Agentic Reasoning: Streamlined Framework for LLM Reasoning with Agentic Tools
- **Paper**: https://arxiv.org/abs/2502.04644 (ACL 2025)
- **Key Innovation**: Framework that enhances LLM reasoning by integrating external tool-using agents. Three core agents: (1) Web-Search agent with highly effective search mechanism surpassing prior approaches, (2) Code-Execution agent for computation, (3) Mind-Map agent — constructs structured knowledge graph to store reasoning context and track logical relationships. The Mind-Map agent is the key innovation: it builds a structured representation of the reasoning chain, ensuring coherence in long reasoning with extensive tool usage. Ablation studies validate optimal tool selection. The Mind-Map prevents reasoning drift in chains spanning 10+ tool calls.
- **Architecture Pattern**: **Structured reasoning memory as coherence guarantee** — the Mind-Map agent acts as an external working memory that maintains logical structure during long reasoning chains. Without it, the LLM loses track of what it's proven, what it's assumed, and what it still needs to verify. The Mind-Map is a structured knowledge graph, not just a text summary — it captures relationships between claims, evidence, and conclusions.
- **NeoTrix Domain Mapping**:
  - **NT-CORE (E8 Reasoning)**: The Mind-Map pattern directly addresses reasoning coherence in our E8 Hexagram engine — as reasoning chains extend across multiple branches, a structured representation prevents coherence loss. The Mind-Map could serve as the working memory layer for our 6-stage feedback loop
  - **NT-MEMORY (Experience Tree)**: The Mind-Map is a structured reasoning trace — exactly what our experience-tree should capture. Not just "what happened" but "what was the logical structure of the reasoning that led to this conclusion." This makes experiences reusable for similar reasoning patterns
  - **NT-ACT (Tool Orchestration)**: The ablation-validated optimal tool selection pattern could enhance our capability registry — instead of fixed tool routing, use reasoning-structure-aware selection based on what the current Mind-Map needs
  - **NT-WORLD (Research)**: The Web-Search agent's superior search mechanism could enhance our NT-WORLD perception pipeline — the search strategy is task-structure-dependent, not just query-dependent

#### Fusion Opportunities
- **Action**: Implement Mind-Map-style structured reasoning traces in our SEAL pipeline — each stage produces a structured representation of its reasoning, not just output
- **Action**: Use Mind-Map structure to guide tool selection in capability registry — the current reasoning graph determines which capability to invoke next
- **Action**: Capture reasoning structure in experience-tree entries — enables pattern-matching across similar reasoning chains in future sessions

---

### 4. Adaptive Theory of Mind for LLM-Based Multi-Agent Coordination
- **Paper**: https://arxiv.org/abs/2603.16264 (AAAI 2026)
- **Key Innovation**: Addresses the "ToM order misalignment" problem in multi-agent coordination. A ToM-k agent assumes its partner is ToM-(k-1). When agents have mismatched ToM orders (e.g., ToM-2 paired with ToM-0), coordination fails due to either insufficient or excessive reasoning about others. The Adaptive ToM (A-ToM) agent detects its partner's ToM order and aligns its own reasoning depth. Three ToM orders: ToM-0 (no modeling of others), ToM-1 (model others' beliefs), ToM-2 (model others' modeling of you). A-ToM uses a calibration signal to detect partner complexity and adjusts. Transforms coordination from policy-space problem to ToM-order alignment problem, reducing dimensionality.
- **Architecture Pattern**: **ToM-order alignment as coordination primitive** — the key insight is that coordination failures aren't about capability but about alignment depth. Too shallow (ToM-0 with ToM-2 partner) → can't anticipate partner actions. Too deep (ToM-2 with ToM-0 partner) → over-reasoning creates noise. A-ToM detects and aligns, turning a continuous coordination problem into a discrete alignment problem. The three-order model provides a clean taxonomy for agent interaction design.
- **NeoTrix Domain Mapping**:
  - **NT-ACT (Cross-Domain Coordination)**: ToM-order alignment directly maps to our faction coordination — different domains have different "reasoning depth" requirements. NT-CORE (deep reasoning) interacting with NT-IO (fast I/O) needs ToM-order alignment, not just message passing. A-ToM could detect when a domain is over-thinking or under-thinking its partner's needs
  - **NT-CORE (GWT)**: The ToM-order concept could enhance GWT salience — the salience of a signal depends on the ToM-order of the receiving module. A ToM-0 module (no partner modeling) needs explicit instructions; a ToM-2 module needs high-level goals
  - **NT-SHIELD (Security)**: ToM-order detection could identify adversarial agents — an agent claiming ToM-2 but operating at ToM-0 (no actual partner modeling) is a security risk. The calibration signal could serve as a trust verification mechanism
  - **NT-MEMORY (Session Continuity)**: ToM-order persistence across sessions — if a module's reasoning depth is stable, the coordination framework can cache the detected order rather than re-detecting each interaction

#### Fusion Opportunities
- **Action**: Implement ToM-order detection for cross-domain coordination — each domain module declares its reasoning depth, and the coordination framework aligns message complexity
- **Action**: Use ToM-order as a trust signal — modules that consistently operate at their declared depth are trusted; mismatches trigger security review
- **Action**: Apply the "alignment as dimension reduction" principle — instead of solving coordination in full policy space, detect and align ToM-orders first, then coordinate within the aligned subspace

---

### 5. CSAttention: Centroid-Scoring Attention for High-Throughput Serving
- **Paper**: https://arxiv.org/abs/2604.08584 (Apr 2026)
- **Key Innovation**: Training-free sparse attention method optimized for high-throughput serving of reusable contexts. Storage-for-computation strategy: front-loads computation into one-time offline prefill phase (amortized across multiple queries), then aggressively optimizes per-step decoding latency. Constructs query-centric lookup tables during offline prefill (fixed size during decoding), enabling online decoding to replace full-context scans with efficient table lookups and GPU-friendly score accumulation. At 95% sparsity and 32K-128K context, achieves up to 4.6x speedup over best baseline while maintaining near-identical accuracy. Specifically designed for the reusable-prompt pattern (agents, domain Q&A) where the same context is queried many times.
- **Architecture Pattern**: **Offline prefill + online decode asymmetry** — the key insight is that many agent use cases involve reusable contexts (system prompts, knowledge bases, conversation history) that are queried repeatedly. By front-loading the expensive computation into a one-time prefill, subsequent queries become table lookups. The lookup table is the amortized representation of the context — computed once, used many times.
- **NeoTrix Domain Mapping**:
  - **NT-IO (Provider Routing)**: CSAttention's amortized prefill pattern directly maps to our LLM provider optimization — system prompts and knowledge contexts could be pre-computed once and reused across sessions, reducing per-query cost. The lookup table becomes a cached context representation
  - **NT-MEMORY (KB Embedding)**: The amortized computation pattern validates our KB caching approach — frequently-accessed knowledge nodes should be pre-computed and cached, not re-embedded each query. The fixed-size lookup table maps to our vector index structure
  - **NT-CORE (GWT)**: The query-centric lookup pattern could enhance GWT salience computation — instead of computing salience from scratch each cycle, use pre-computed salience tables that are updated incrementally
  - **Dual Specialization**: The offline/online split maps directly to our weapon set design — Weapon Set I (offline preparation, full computation) and Weapon Set II (online execution, table lookups)

#### Fusion Opportunities
- **Action**: Implement amortized context precomputation for high-frequency queries — system prompts and knowledge contexts become pre-computed lookup tables
- **Action**: Apply the offline/online asymmetry to our SEAL pipeline — expensive analysis happens offline, runtime decisions use pre-computed tables
- **Action**: Study the "reusable context" pattern as a general optimization — identify which NeoTrix computations are amortizable across sessions

---

## Key Patterns Across Papers

### Pattern 1: Memory as Selection Guide, Not Computation
**RAT+** — The most efficient memory systems don't participate in the main computation; they guide what gets computed. The decay memory guides sparse selection but doesn't do attention. The Mind-Map guides tool selection but doesn't do reasoning. This separation of "memory for guidance" vs "memory for computation" is a fundamental architectural principle.

### Pattern 2: Coordination Through Alignment, Not Capability
**A-ToM, Emergent Coordination** — Multi-agent systems fail not because agents are weak, but because they're misaligned. ToM-order alignment and identity-linked differentiation are alignment mechanisms. The solution is better coordination protocols, not stronger individual agents. This validates our domain-faction isolation principle.

### Pattern 3: Amortized Computation for Reusable Contexts
**CSAttention, Agentic Reasoning Mind-Map** — When the same context is queried repeatedly, amortize the expensive computation. CSAttention does it with lookup tables; Mind-Map does it with structured reasoning traces. The pattern: compute once, use many times, update incrementally.

### Pattern 4: Structured Representation Over Text Summaries
**Mind-Map, Bitemporal Graph** — Text summaries lose structure. Knowledge graphs, reasoning traces, and bitemporal graphs preserve relationships. The shift from "summarize what happened" to "represent the structure of what happened" is enabling more sophisticated reuse and reasoning.

### Pattern 5: Information-Theoretic Coordination Metrics
**Emergent Coordination** — Measuring coordination quality requires principled metrics, not just task success rates. Mutual information decomposition distinguishes real synergy from spurious coupling. This provides a foundation for production monitoring of multi-agent systems.

---

## NeoTrix Fusion Summary

| Paper | Primary Pattern | NeoTrix Integration | Priority |
|-------|----------------|---------------------|----------|
| RAT+ | Decay memory for selection guidance | GWT salience augmentation + experience-tree forgetting | P0 — immediate integration path |
| Emergent Coordination | Steerable coordination regimes | Faction health metrics + information-theoretic audit | P1 — production monitoring |
| Agentic Reasoning | Mind-Map structured reasoning | SEAL pipeline reasoning traces + experience-tree structure | P0 — direct experience quality improvement |
| A-ToM | ToM-order alignment | Cross-domain coordination depth alignment | P1 — coordination reliability |
| CSAttention | Amortized precomputed contexts | KB caching + provider optimization + weapon set asymmetry | P0 — cost reduction for high-frequency queries |
