# Model Reverse Engineering — Cycle 329

**Date**: 2026-09-11  
**Focus**: Efficient inference, attention mechanisms, agent coordination, memory architectures  
**Previous cycles**: 318–328 (excluded)

---

## Paper 1: Gated-Memory Routing for Multi-Agent LLM Systems

**URL**: https://arxiv.org/html/2609.00237v1  
**Authors**: Hasan et al.  
**Date**: 2026-08-31

### Core Idea
Multi-agent routing conditioned on learned execution memory rather than query-only or full-history. Two gates:
- **Memory Write Gate**: Commits only non-redundant reasoning steps (relevant + novel relative to stored)
- **Retrieval Gate**: Surfaces compact, step-relevant subset for each agent

Adaptive Halting Controller stops execution when gated memory contains sufficient evidence.

### Key Results
- Best average accuracy across 5 reasoning/code benchmarks (+2.44pp over strongest baseline)
- 31.9% inference cost reduction on HumanEval vs baseline
- Dynamic interaction structure emerges from gated memory, not predefined

### NeoTrix Domain Mapping

| Domain | Integration |
|--------|------------|
| **NT-MEMORY** | Memory Write Gate → KB write-side filtering; Retrieval Gate → on-demand branch loading (experience-tree) |
| **NT-CORE (GWT)** | Adaptive Halting Controller → GWT attention routing with termination signal |
| **NT-ACT** | History-Aware Role Allocator → dynamic agent selection based on execution state |
| **NT-MIND** | Joint training of memory + routing under task-level reward → SEAL evolution feedback |

### NeoTrix Application
**Experience-tree absorption optimization**: The Memory Write Gate pattern directly applies to `neotrix-experience absorb` — filter redundant experiences before writing to KB. The Retrieval Gate pattern applies to session-start hub index loading — load only step-relevant branches, not all branches.

**GWT enhancement**: Add halting signal to GWT attention routing. Currently GWT broadcasts salient info but doesn't model when to stop accumulating context. The Adaptive Halting Controller provides a principled termination mechanism based on gated memory sufficiency.

---

## Paper 2: CEDAR — Error-Bounded Residual Routing for Long-Context Attention

**URL**: https://arxiv.org/abs/2609.07237  
**Date**: 2026-09-07

### Core Idea
Coarse-to-fine Error-aware Dynamic Attention Routing for long-context prefill. Key innovations:
- **Residual attention path**: Each semantic chunk contributes cheap KV summary alongside hard selection
- **Error-bounded refinement**: Variable refinement budget allocated by within-chunk key/value dispersion
- **Single softmax normalization**: Exact and summarized contributions combined — refinement replaces, not duplicates, coarse evidence

### Key Results
- 98%+ reconstruction error reduction vs hard dropping at equal exact-chunk budgets
- ~3× kernel speedup at 128K context
- Frozen LM (no retraining required)

### NeoTrix Domain Mapping

| Domain | Integration |
|--------|------------|
| **NT-CORE (GWT)** | Error-bounded attention routing → salience-based context window management |
| **NT-MEMORY** | Residual KV summaries → KB embedding tiered storage (hot/warm/cold) |
| **NT-WORLD** | 128K context efficiency → long-document crawl parsing |
| **NT-IO** | Frozen LM compatibility → model-agnostic inference optimization |

### NeoTrix Application
**GWT attention refinement**: CEDAR's error-bounded approach can optimize GWT's attention routing. Instead of binary salient/non-salient, assign error bounds to each broadcast. Low-error summaries pass through residual path; high-error regions get exact attention. This creates a natural "what to compute exactly vs summarize" decision for GWT.

**KB query optimization**: For long-document retrieval, use residual summaries for first-pass screening, then exact attention only for high-dispersion chunks. Reduces query latency without sacrificing recall.

---

## Paper 3: HeRo — History-Aware Routing for Efficient Dynamic LLM Inference

**URL**: https://arxiv.org/abs/2609.08189  
**Date**: 2026-09-08

### Core Idea
Dynamic layer routing with explicit router memory across model depth:
- **Router memory via linear attention**: Incrementally aggregates preceding routing scores and induced residual updates into compact history
- **Joint conditioning**: Each routed layer conditions on accumulated state + current hidden representation
- **Frozen backbone**: Only lightweight routers and adapters trained, no pretrained parameter modification

### Key Results
- Llama 3.1-8B: bypasses 26.87% parameters while achieving 100.24% dense performance (7 benchmarks)
- Retains 97.01% while bypassing 38.82% under tighter budget
- Removing routing history consistently degrades multistep reasoning and code generation most

### NeoTrix Domain Mapping

| Domain | Integration |
|--------|------------|
| **NT-CORE (GWT)** | Router memory → attention state persistence across reasoning steps |
| **NT-MIND** | History-aware routing → SEAL pipeline stage-dependent routing |
| **NT-IO** | 26-39% parameter bypass → cost-aware model execution |
| **NT-PHYSICAL** | Linear attention memory → lightweight state on constrained hardware |

### NeoTrix Application
**GWT with persistent attention state**: HeRo's router memory pattern can be applied to GWT. Currently GWT's attention routing is stateless across steps. Adding linear-attention-based router memory would let GWT accumulate routing decisions over a session, enabling path-dependent attention allocation.

**Cost-aware routing (Axiom A1)**: The 26-39% parameter bypass directly implements cost-aware routing. For simple tasks (I/O, formatting), bypass expensive reasoning layers. For complex tasks (architecture, debugging), activate full depth. HeRo's history mechanism ensures the bypass decisions accumulate correctly.

---

## Paper 4: ROAM — Robust Organization of Atomic Memories

**URL**: https://arxiv.org/abs/2609.09778  
**Date**: 2026-09-09

### Core Idea
Relation-guided memory management using atomicity:
- **Relation classification**: Incoming-stored atom pairs classified as independent, equivalent, directionally subsuming, or conflicting
- **Role organization**: Active Primary vs supporting Evidence roles
- **Fusion**: Combines complementary details and temporal changes into compact, potentially non-atomic views
- Only Primary views retrieved for answering (prevents redundant atoms competing independently)

### Key Results
- Up to 29.8 percentage point improvement in answer accuracy
- 15.6-point higher answer-critical source recall
- 11.5-point lower confounder-token share
- Robust across manager scales

### NeoTrix Domain Mapping

| Domain | Integration |
|--------|------------|
| **NT-MEMORY** | Relation-guided KB deduplication; Primary/Evidence role for experience nodes |
| **NT-MEMORY** | Fusion → experience-tree branch merging during absorption |
| **NT-CORE** | Conflict detection → architecture-level contradiction resolution |
| **NT-SHIELD** | Confounder detection → knowledge poisoning defense |

### NeoTrix Application
**KB experience deduplication**: ROAM's relation classification directly applies to KB `experience` namespace. When absorbing new experiences, classify against existing entries:
- Equivalent → merge (don't duplicate)
- Directionally subsuming → update existing, demote to Evidence
- Conflicting → flag for review (don't silently overwrite)
- Independent → add as new Primary

**Experience-tree fusion**: During `close --cycle NNN`, fuse complementary details from related experiences into compact views. Only Primary views load into session start; Evidence loaded on-demand.

---

## Paper 5: MAGMA — Multi-Graph Agentic Memory Architecture

**URL**: https://arxiv.org/abs/2601.03236  
**Date**: 2026-09 (updated)

### Core Idea
Memory items modeled across 4 orthogonal relational graphs:
1. **Semantic graph** — conceptual similarity
2. **Temporal graph** — chronological ordering
3. **Causal graph** — cause-effect chains
4. **Entity graph** — participant relationships

Retrieval as policy-guided graph traversal with Adaptive Traversal Policy. Dual-stream memory evolution:
- **Fast path** (Synaptic Ingestion): latency-sensitive event ingestion
- **Slow path** (Asynchronous Consolidation): compute-intensive structural refinement

### Key Results
- LoCoMo judge score: 0.7 (vs Full Context 0.481, A-MEM 0.58, MemoryOS 0.553)
- 18.6% to 45.5% relative improvement over baselines
- Transparent reasoning paths via graph traversal

### NeoTrix Domain Mapping

| Domain | Integration |
|--------|------------|
| **NT-MEMORY** | 4-graph memory substrate for KB; dual-stream evolution for KB writes |
| **NT-WORLD** | Entity graph → knowledge graph from crawl data |
| **NT-CORE** | Causal graph → E8 reasoning chain tracking |
| **NT-ACT** | Policy-guided traversal → capability routing via graph structure |
| **NT-MIND** | Asynchronous consolidation → background SEAL evolution |

### NeoTrix Application
**KB multi-relational substrate**: MAGMA's 4-graph model provides a blueprint for enriching NeoTrix KB. Currently KB stores nodes + edges + embeddings. Adding explicit temporal, causal, and entity graphs would enable:
- Temporal: "What happened before/after X?" queries
- Causal: "What caused Y?" chain tracing
- Entity: "What entities are involved in Z?" structured lookup

**Dual-stream KB writes**: Fast path for immediate experience ingestion (session-end absorption). Slow path for background consolidation (linking experiences, detecting conflicts, building causal chains). This maps to the existing `pending-absorb.json` → background loop architecture.

---

## Cross-Paper Patterns

| Pattern | Papers | NeoTrix Mapping |
|---------|--------|-----------------|
| **Memory as routing signal** | Gated-Memory, HeRo, MAGMA | KB experiences should influence GWT attention routing, not just be stored |
| **Error-bounded refinement** | CEDAR, Gated-Memory | Add error bounds to GWT salience scores — refine only high-error regions |
| **Dual-stream evolution** | MAGMA, ROAM | Fast ingestion + slow consolidation for KB writes |
| **Adaptive halting** | Gated-Memory, HeRo | GWT should model when to stop accumulating context |
| **Relation-guided deduplication** | ROAM, MAGMA | KB write-side filtering using relation classification |
| **Frozen backbone + lightweight adapters** | HeRo, CEDAR | All memory/routing enhancements should be non-parametric to base models |

---

## Action Items for NeoTrix

| Priority | Action | Source Paper | Target Domain |
|----------|--------|-------------|---------------|
| P0 | Implement Memory Write Gate for experience-tree absorption | Gated-Memory | NT-MEMORY |
| P0 | Add error-bounded attention to GWT routing | CEDAR | NT-CORE |
| P1 | Add router memory (linear attention) to GWT | HeRo | NT-CORE |
| P1 | Implement ROAM relation classification for KB dedup | ROAM | NT-MEMORY |
| P1 | Add causal + temporal graphs to KB | MAGMA | NT-MEMORY |
| P2 | Implement Adaptive Halting Controller for GWT | Gated-Memory | NT-CORE |
| P2 | Dual-stream KB write (fast ingest + slow consolidation) | MAGMA | NT-MEMORY |
| P3 | Entity graph extraction from crawl data | MAGMA | NT-WORLD |
