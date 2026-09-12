# Model Reverse Engineering — Cycle 378

**Date:** 2026-09-12
**Focus:** Recent models/papers on efficient inference, attention, agent coordination
**Sources:** arXiv (Aug-Sep 2026), ACL 2026 Findings

---

## 1. Declarative Attention — Model-Controlled Sparse Attention via Chain-of-Thought

**Source:** arXiv:2609.02737, Sep 2 2026

### Key Claims
- 52.0% reduction in total attended tokens (Gemma-4-31B), 31.1% (Qwen-3.6-27B)
- Modest accuracy drops: 1.27pp and 2.75pp respectively, shrinking with model scale
- Zero-shot — works on off-the-shelf models without training
- Three modes: `<global>` (full context), `<focus>` (specific region), `<local>` (recent output only)

### Architecture Insights
- **Intrinsic approach**: Instead of external proxy scores to select tokens, the model declares WHERE it needs to attend within its chain-of-thought
- **Three-mode partitioning**: Model emits special tokens `<global>`, `<focus>`, `<local>` during CoT that the inference engine parses like tool calls to skip KV cache reads
- **Self-aware attention**: The model itself knows which parts of context are relevant — exploits this knowledge at inference time
- **No training required**: Existing models already have this capability; just needs the right prompting protocol

### NeoTrix Domain Mapping
| Domain | Pattern | Implementation |
|--------|---------|----------------|
| NT-CORE | Self-declared attention → GWT salience | Model broadcasts attention declarations. GWT routes based on declared focus regions rather than computed salience alone |
| NT-MIND | Self-awareness calibration | Model's ability to declare attention boundaries = meta-cognitive awareness of its own information needs |
| NT-IO | Cost-aware routing (Axiom A1) | Attention declarations enable token-cost budgeting — skip KV reads for `<local>` regions |

### Absorption
- **Declaration protocol**: Implement attention-declaration tokens as a GWT modifier — agents broadcast which context regions they need, reducing broadcast overhead
- **Self-awareness**: Map to SelfModel's uncertainty awareness — model knows what it doesn't need to look at

---

## 2. Gated-Memory Routing — Learned Execution Memory for Multi-Agent Coordination

**Source:** arXiv:2609.00237, Aug 31 2026

### Key Claims
- Best average accuracy across 5 benchmarks, exceeding strongest baseline by +2.44 points
- 31.9% reduction in HumanEval inference cost vs baseline
- Adaptive halting — stops execution when memory contains sufficient evidence
- Memory Write Gate + Retrieval Gate keep execution memory compact and task-relevant

### Architecture Insights
- **Gated execution memory**: Central object — every routing decision conditions on filtered memory, not raw history
- **Memory Write Gate**: Commits only non-redundant reasoning steps (relevance + novelty scoring via MMR-inspired sampling)
- **Retrieval Gate**: Step-adaptive selection — decides independently for each record whether to surface it
- **Adaptive Halting Controller**: Reads gated memory state to decide when to stop. Depth = consequence of execution, not fixed from query
- **Joint training**: Router, write gate, retrieval gate, halting controller all trained end-to-end with single reward (quality vs cost)

### NeoTrix Domain Mapping
| Domain | Pattern | Implementation |
|--------|---------|----------------|
| NT-MEMORY | Gated memory → KB write/retrieval | Write Gate maps to experience-tree's distillation phase (only commit non-redundant). Retrieval Gate maps to KB query optimization |
| NT-ACT | Adaptive multi-agent routing | Role allocation + backbone selection conditioned on execution memory state. Maps to NT-ACT's orchestration layer |
| NT-CORE | Adaptive halting | GWT attention could use halting controller — stop broadcasting when sufficient information accumulated |
| NT-MIND | Execution memory as control signal | Memory is not just storage but active control signal for routing decisions |

### Absorption
- **Write Gate pattern**: Implement in experience-tree — only commit experiences that are both relevant AND novel vs existing KB entries
- **Adaptive halting**: Add to SEAL pipeline — halt evolution cycle when accumulated evidence is sufficient, avoiding over-optimization
- **Joint training**: Model SEAL pipeline components (write/retrieve/route/halt) as jointly trained subsystems

---

## 3. CEDAR — Error-Bounded Residual Routing for Sparse Attention

**Source:** arXiv:2609.07237, Sep 7 2026

### Key Claims
- Recovers most quality lost by hard sparse routing while maintaining ~3x kernel speedup at 128K
- Residual summaries reduce reconstruction error by 98%+ vs hard dropping at equal budgets
- Output-error bound governed by within-chunk key/value dispersion
- Coarse-to-fine: cheap KV summaries + expansion only for high-error chunks

### Architecture Insights
- **Residual attention path**: Each semantic chunk contributes a cheap key-value summary; high-error chunks expanded to exact token attention
- **Error-aware routing**: Variable refinement budget based on estimated approximation error — spend more compute on ambiguous queries
- **Single softmax normalization**: Exact and summarized contributions combined in one softmax — refinement REPLACES rather than duplicates coarse evidence
- **Frozen LM**: No weight changes — drops in as a post-hoc acceleration method

### NeoTrix Domain Mapping
| Domain | Pattern | Implementation |
|--------|---------|----------------|
| NT-CORE | Error-bounded attention | GWT could use error estimates to decide which broadcast fragments need full vs summarized attention |
| NT-MEMORY | Coarse-to-fine retrieval | KB retrieval: cheap summaries for all entries, expand to full detail only for high-uncertainty queries |
| NT-WORLD | Residual perception | PerceptionBridge could use residual summaries — lightweight scan first, detailed analysis only for salient events |

### Absorption
- **Error-bounded routing**: Add error estimation to GWT attention — compute cheap summary of each broadcast, expand only where error exceeds threshold
- **Residual KB retrieval**: Implement coarse-to-fine pattern in KB query pipeline — summary-first, expand on demand

---

## 4. MAGMA — Multi-Graph Agentic Memory Architecture

**Source:** ACL 2026 (acl-long.1709)

### Key Claims
- Consistently outperforms SOTA agentic memory on LoCoMo and LongMemEval
- Four orthogonal relational graphs: semantic, temporal, causal, entity
- Policy-guided graph traversal for retrieval — query-adaptive selection
- Dual-stream memory evolution: fast ingestion + slow structural consolidation

### Architecture Insights
- **Four-graph representation**: Each memory item modeled across semantic, temporal, causal, and entity graphs — disentangled relational views
- **Adaptive Traversal Policy**: Routes retrieval based on query intent — WHEN queries use temporal graph, WHY queries use causal graph, etc.
- **Dual-stream evolution**: Fast path (synaptic ingestion) handles real-time writes; slow path (structural consolidation) densifies graph structure asynchronously
- **Topological ordering**: Query results ordered by temporal or causal logic before context construction — forces LLM to interpret evidence rather than generate creatively

### NeoTrix Domain Mapping
| Domain | Pattern | Implementation |
|--------|---------|----------------|
| NT-MEMORY | Multi-graph KB | KB could model entries across semantic/temporal/causal/entity dimensions instead of flat KV store |
| NT-WORLD | Intent-aware retrieval | PerceptionBridge routes queries to appropriate graph based on intent — temporal events → temporal graph, causal chains → causal graph |
| NT-CORE | Policy-guided traversal | GWT attention could traverse knowledge graphs with intent-aware routing |
| NT-MIND | Dual-stream consolidation | Experience-tree: fast snapshot for immediate use, slow distillation for structural refinement |

### Absorption
- **Multi-graph KB**: Extend KB schema with temporal/causal/entity dimensions alongside semantic — enable intent-aware retrieval
- **Dual-stream writes**: Implement fast-path capture + slow-path consolidation in experience-tree pipeline
- **Topological context ordering**: Order retrieved evidence by temporal/causal logic before feeding to LLM

---

## 5. Mesh Memory Protocol (MMP) — Semantic Infrastructure for Cross-Agent Collaboration

**Source:** arXiv:2604.19540

### Key Claims
- Production-deployed protocol (3 reference deployments) for cross-session agent-to-agent cognitive collaboration
- Four composable primitives: CAT7 schema, SVAF evaluation, inter-agent lineage, remix semantics
- Write-time filtering (not read-time) — memory relevance established at admission, not retrieval
- Solves 3 problems: field-by-field acceptance (P1), traceable provenance (P2), session-resumable memory (P3)

### Architecture Insights
- **CAT7 fixed schema**: 7-field cognitive memory block — every exchanged thought fits this schema, enabling heterogeneous agents to evaluate contributions without negotiation
- **SVAF (Symbolic-Vector Attention Fusion)**: Per-field evaluation against receiver's role-indexed anchors — same CMB produces different admission decisions at different receivers
- **Inter-agent lineage DAG**: Content-hash keys with parent/ancestor tracking — echo detection and post-hoc audit without central event log
- **Remix semantics**: Stores only receiver's own role-evaluated understanding, never raw peer signal. Memory is never copied, only remixed — every session remains autonomous
- **Write-time filtering**: Inverts standard persistent-memory primitive (RAG, checkpoint replay filter at read time). Relevance is invariant at admission moment

### NeoTrix Domain Mapping
| Domain | Pattern | Implementation |
|--------|---------|----------------|
| NT-MEMORY | Write-time filtering | KB writes: evaluate relevance at admission, not retrieval. Reduces query-time overhead |
| NT-NEXUS | Inter-agent lineage | Cross-session memory with provenance DAG — track which experiences influenced which decisions |
| NT-GOVERNANCE | Role-indexed evaluation | Each domain evaluates incoming knowledge through its own role lens — NT-CORE evaluates differently than NT-ACT |
| NT-SHIELD | Remix semantics | Never expose raw peer signals — only role-evaluated understanding. Defense against adversarial memory injection |

### Absorption
- **Write-time filtering**: Implement SVAF-inspired admission control in KB — evaluate relevance/novelty at write time, not query time
- **Remix semantics**: Experience-tree stores only distilled understanding, never raw session data — reduces storage and prevents adversarial contamination
- **Lineage DAG**: Track provenance of all KB entries — which session, which agent, which decision produced each entry

---

## Cross-Paper Synthesis

### 1. Attention is Becoming Self-Aware
Declarative Attention shows models can declare their own attention needs. Combined with CEDAR's error-bounded routing, this points to a future where attention is **negotiated** between model intent and computational budget — not computed blindly.

### 2. Memory is an Active Control Signal, Not Passive Storage
Gated-Memory Routing, MAGMA, and MMP all treat memory as an active participant in routing decisions. Memory determines who acts next, what they read, and when collaboration stops. This validates NeoTrix's KB-as-shared-state architecture.

### 3. Write-Time vs Read-Time is the New Architectural Axis
MMP's write-time filtering inverts the standard RAG paradigm. MAGMA's dual-stream evolution (fast ingestion + slow consolidation) adds another dimension. The choice of WHEN to filter (write vs read) has architectural consequences beyond optimization.

### 4. Error Bounds Enable Trust
CEDAR's output-error bounds and Gated-Memory's adaptive halting both provide **guarantees** on quality. This is critical for production systems where "probably correct" isn't sufficient — you need "correct within bound X."

### 5. Agent Autonomy Through Role Specialization
MMP's role-indexed evaluation, MAGMA's intent-aware traversal, and Gated-Memory's role allocation all show that agent identity shapes how information is processed. One-size-fits-all retrieval is giving way to role-specific processing pipelines.

---

## Absorption Priority Matrix

| Paper | Pattern | Target Domain | Priority | Complexity |
|-------|---------|---------------|----------|------------|
| Declarative Attention | Self-declared focus regions | NT-CORE (GWT) | P0 | Medium |
| Gated-Memory Routing | Write/Retrieval Gates + Adaptive Halting | NT-MEMORY + NT-ACT | P0 | High |
| MAGMA | Multi-graph KB + dual-stream writes | NT-MEMORY | P1 | High |
| MMP | Write-time filtering + remix semantics | NT-MEMORY + NT-SHIELD | P1 | Medium |
| CEDAR | Error-bounded residual routing | NT-CORE (GWT) | P2 | Medium |
