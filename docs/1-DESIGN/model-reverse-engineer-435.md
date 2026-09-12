# Model Reverse Engineering — Cycle 435

**Date**: 2026-09-12
**Scope**: 5 recent papers/models on efficient inference, attention, agent coordination
**Method**: Pattern extraction → NeoTrix 7-domain mapping
**Exclusion**: Papers covered in cycles 318–434

---

## 1. ConvMem — Hierarchical Convolutional Memory for Long-Context (arXiv:2609.10441)

**Paper**: *ConvMem: Convolutional Memory for Long-Context Reasoning*
**Published**: 2026-09-09

### Core Insight
Sequential memory update (read segments → iteratively update fixed-size memory) is slow and requires RL training. ConvMem reformulates long-context reasoning as **hierarchical convolution** — the LLM + query becomes a convolutional kernel that summarizes text segments in parallel, reducing reasoning path from linear chain to logarithmic tree.

### Mechanism
- **LLM-as-kernel**: Query-prompted LLM acts as convolutional kernel over text segments
- **Hierarchical summarization**: Segments summarized in parallel at each level, like CNN pooling layers
- **Configurable Strides**: Control overlap between segments to balance coverage vs redundancy
- **Skip Connections**: Shortcuts from lower levels to higher levels propagate evidence across layers
- **Multi-Kernel Convolution**: Multiple parallel kernels decompose complex queries into disentangled semantic channels
- **Training-free**: No RL required, works on off-the-shelf models

### Results
- Outperforms training-free baselines on RULER-HotpotQA and RULER-2WikiMultiHopQA
- Avoids overfitting to parametric priors common in RL-trained models on OOD tasks
- Parallelizable across both text segments and reasoning threads
- Logarithmic depth vs linear depth of sequential approaches

### NeoTrix Domain Mapping

| Domain | Mapping |
|--------|---------|
| **NT-CORE** | Hierarchical convolution → E8 hexagram's multi-level reasoning. Each convolution level = one hexagram layer (Soil→Roots→Trunk→Branches→Fruits→Core). Query-prompted LLM-as-kernel = E8's context-activated reasoning state. Skip connections = cross-layer attention in GWT. |
| **NT-MEMORY** | Training-free memory compression → KB's lazy node expansion. Summarize first (L1), expand only when multi-hop requires deeper evidence (L2+). Configurable strides → KB retrieval chunk overlap control. Skip connections → experience-tree cross-cycle references. |
| **NT-WORLD** | Multi-kernel decomposition → crawl pipeline's multi-perspective content extraction. Each kernel extracts different semantic channel (entities, relationships, temporal). Parallel processing across channels. |
| **NT-MIND** | Logarithmic reasoning depth → SEAL pipeline's graduated analysis. Phase-0 = shallow summary, Phase-N = deep evidence chains. Skip connections = early termination when lower-level evidence suffices. |

### Actionable Pattern for NeoTrix
**Logarithmic Experience Retrieval**: When querying experience namespace, use hierarchical summarization: (1) summarize all experiences at L1, (2) only expand branches that score above threshold, (3) use skip connections to propagate evidence from lower summaries directly to final answer. This gives O(log N) retrieval depth instead of O(N) full scan, with configurable strides controlling detail granularity.

---

## 2. MemoryLACE — Memory Lifecycle-Aware Consolidation (arXiv:2609.03201)

**Paper**: *MemoryLACE: Memory Lifecycle-Aware Consolidation and Evidence Retrieval*
**Published**: 2026-09-02

### Core Insight
Existing memory systems treat memories as independent documents. MemoryLACE models the **lifecycle** of textual evidence through three relation types: **merge** (combine supporting evidence), **supersession** (newer replaces older), and **contradiction** (flag conflicts). Atomic memories preserved; relations reconstructed at query time.

### Mechanism
- **Atomic memories**: Each fact stored as independent natural-language unit with provenance
- **Lifecycle relations**: Merge (A+B → AB), Supersession (A' replaces A), Contradiction (A ≠ B)
- **Relation-aware retrieval**: At query time, reconstruct evidence units exposing current, historical, supporting, and conflicting evidence
- **No global graph**: Local lifecycle relations only — no expensive graph construction
- **Lightweight**: 66.6% runtime reduction vs Hindsight (strongest reflective-memory baseline)

### Results
- Highest overall performance on BEAM and StructMemEval across open-weight and proprietary LLM backends
- 66.6% runtime reduction vs Hindsight
- Lifecycle expansion and temporal awareness are principal contributors

### NeoTrix Domain Mapping

| Domain | Mapping |
|--------|---------|
| **NT-MEMORY** | Lifecycle relations = KB node edge types. Merge → edge_merge, Supersession → edge_replaces, Contradiction → edge_conflicts. Atomic memories = KB nodes. Relation-aware retrieval = KB graph traversal with lifecycle metadata. |
| **NT-CORE** | Contradiction detection → E8 hexagram's conflicting-perspective analysis. When evidence contradicts, E8 explores both paths. Merge → hexagram synthesis of supporting perspectives. |
| **NT-MIND** | Supersession → SEAL pipeline's skill evolution. New skill versions supersede old ones. Lifecycle tracking enables rollback to any previous skill version. |
| **NT-REPAIR** | Contradiction flagging → NT-REPAIR's degradation detection. When new evidence contradicts stored knowledge, trigger self-healing review. |

### Actionable Pattern for NeoTrix
**Lifecycle-Aware Experience Hub**: Add lifecycle relation types to experience nodes. When new experience contradicts existing, don't overwrite — add contradiction edge and flag for review. When experience adds supporting evidence, merge into richer node. When experience supersedes old, mark old as historical. At retrieval time, expose full lifecycle: current state + historical context + supporting evidence + contradictions.

---

## 3. Metis — Memory Foundation Model (arXiv:2607.26760)

**Paper**: *Metis: Memory Foundation Model*
**Published**: 2026-07-29

### Core Insight
Agent memory is still external modules (vector DB, knowledge graphs). Metis internalizes memory into the model itself — **native memory state** persists across inferences as dynamic parameters, and **native memory procedures** (store/forget/update) happen during forward computation.

### Mechanism
- **Native Memory State**: Persistent dynamic parameters inside backbone (not external DB). Updated during forward pass, not separate write operation.
- **Metis Blocks**: Each Transformer block extended with local memory block (dense representation) + hyper memory block (function space for memory operations)
- **Memory Attention**: Parallel to original attention — both branches run on same input, no sequential dependency
- **Memory Procedures**: Storage = hyper block updates local block from intermediate activations. Utilization = local block incorporates memory into forward computation
- **Gradient-free online update**: Memory maintenance requires only forward pass, no backpropagation
- **Frozen weights at inference**: All learned weights stay fixed; only native memory states transform through computation

### Results
- Memory attention runs in parallel with original attention (minimal overhead)
- Memory update is forward-pass-only (no training at inference)
- Removes textual memory from context entirely (no RAG needed)
- Native memory competes with and outperforms external memory on long-context tasks

### NeoTrix Domain Mapping

| Domain | Mapping |
|--------|---------|
| **NT-CORE** | Native memory state = ConsciousnessTree's persistent consciousness state. Memory attention parallel to original attention = GWT's parallel salience computation. Hyper memory block = E8's meta-reasoning layer that updates reasoning parameters. |
| **NT-MEMORY** | Native memory = KB state internalized into model computation. Removes external retrieval step. Memory procedures (store/forget/update) = KB CRUD but embedded in inference, not API calls. |
| **NT-MIND** | Gradient-free online update = SEAL pipeline's zero-training skill evolution. Skills update through forward pass (observation → skill refinement) without retraining. |
| **NT-IO** | Memory attention parallel to original attention = NT-IO's dual-path processing (model response + memory retrieval in parallel, not sequential). |

### Actionable Pattern for NeoTrix
**In-Process Memory Attention**: Instead of sequential (1) query KB, (2) inject into prompt, (3) generate response, implement parallel path: memory retrieval runs concurrently with model reasoning. Model's intermediate activations update KB state in the same forward pass. This eliminates the retrieval latency overhead and enables gradient-free memory updates during normal inference.

---

## 4. AGAO — Adaptive Goal-aware Attention Orchestration (arXiv:2607.23678)

**Paper**: *Focus Is All You Need: Adaptive Goal-aware Attention Orchestration for Multi-Agent Graph Systems*
**Published**: 2026-07-26

### Core Insight
Multi-agent graph systems execute most of the graph uniformly — wasting computation on irrelevant agents. AGAO extends attention from token-level to **workflow-level agent coordination**: dynamically estimate each agent's importance according to user goals, graph dependencies, and computational constraints.

### Mechanism
- **Goal-aware Attention**: Semantic alignment between user objective and agent capabilities (cosine similarity of embeddings)
- **Topology-aware Attention**: Graph structural dependencies — agents with high betweenness centrality and prerequisite edges get higher attention
- **Resource-aware Attention**: Translates attention scores into execution decisions — model selection (cheap vs expensive), token budget allocation, agent activation priority
- **Adaptive Graph Routing**: Attention distributions updated by execution feedback — each iteration recalculates agent importance based on intermediate results
- **Three components work hierarchically**: Goal → Topology → Resource, with each level refining the previous

### Results
- Improves task effectiveness while reducing unnecessary computation
- Reduces latency and token consumption vs static graph execution
- Establishes "Attention Engineering" as new direction for multi-agent systems
- Competitive on both coding (MBPP) and QA (HotpotQA) tasks

### NeoTrix Domain Mapping

| Domain | Mapping |
|--------|---------|
| **NT-CORE** | Goal-aware attention = GWT salience computation. Topology-aware = E8 hexagram dependency graph (which reasoning states depend on others). Resource-aware = Axiom A1 cost-aware routing. Adaptive routing = ConsciousnessTree's dynamic attention reallocation based on cycle feedback. |
| **NT-ACT** | Agent importance estimation → NT-ACT's worker selection based on task-agent capability matching. Resource-aware execution → budget allocation across NT-ACT worker pool. Adaptive routing → dynamic task reassignment based on intermediate results. |
| **NT-MIND** | Attention Engineering = SEAL pipeline's resource-aware evolution. Allocate more evolution budget to high-impact skills, less to mature/stable ones. |
| **NT-SHIELD** | Topology-aware attention → NT-SHIELD's risk-weighted execution priority. High-risk agents (security-sensitive) get more attention (more verification). |

### Actionable Pattern for NeoTrix
**GWT Attention Orchestration**: Extend GWT's salience computation to include: (1) goal-aware scoring (how aligned is this domain with the current task?), (2) topology-aware scoring (what are the dependency relationships between domains?), (3) resource-aware scoring (what's the token/compute budget for this domain?). Dynamic routing updates salience at each SEAL pipeline stage based on intermediate results. This makes GWT not just attention allocation but **computational focus orchestration**.

---

## 5. RecurTrace — Adaptive Latent Reasoning with Loop-Time Memory (arXiv:2609.03379)

**Paper**: *RecurTrace: Adaptive Latent Reasoning with Loop-Time Memory*
**Published**: 2026-09-03

### Core Insight
Repeating middle layers increases effective inference depth without adding parameters or tokens. But two limitations: (1) each iteration only sees previous output, not earlier computations, and (2) fixed loop count wastes depth on easy inputs while starving hard ones. RecurTrace uses the loop's own trajectory to solve both.

### Mechanism
- **Loop Memory Attention**: Each looped layer attends to its own states from previous iterations along the loop-time axis. Model can revisit earlier computations, not just latest state.
- **Halting Head**: Reads loop state and predicts whether to continue. Supervised by oracle that identifies when additional depth still reduces loss.
- **Loop-time memory**: Persistent state across loop iterations — like a mini working memory that accumulates across repetitions
- **Adaptive compute**: Easy inputs halt early (2 loops), hard inputs get more (4+ loops). Compute budget allocated by difficulty.
- **Same backbone, no training**: Works on frozen models with learned halting head only

### Results
- 56.9% accuracy on MathQA with average 2.0 loops (vs 55.7% best fixed-loop at 3.2 loops)
- 2.2pp improvement over best fixed loop depth at matched compute
- Improves over same-budget fine-tuned baselines at 0.6B, 1.7B, 4B, and 8B scales
- Gain grows with model size (0.6 to 3.4 points)

### NeoTrix Domain Mapping

| Domain | Mapping |
|--------|---------|
| **NT-CORE** | Loop Memory Attention = E8 hexagram's cross-layer state reuse. Each iteration of ConsciousnessTree's 6-stage loop can attend to previous iterations' states. Halting head = adaptive cycle count — ConsciousnessTree doesn't run fixed N cycles, but adapts based on convergence signal. |
| **NT-MIND** | Loop-time memory = SEAL pipeline's cross-stage memory. Phase-1 results inform Phase-3 decisions without re-deriving. Halting head = convergence detector — stop evolution when gains diminish. |
| **NT-REPAIR** | Adaptive compute = NT-REPAIR's self-healing intensity matching. Simple degradations: quick patch. Complex failures: deep diagnostic loop with accumulated state. |
| **NT-MEMORY** | Loop-time memory = experience-tree's cross-cycle memory. Each cycle's state persists and informs the next cycle's decisions. Halting = experience absorption completion detection. |

### Actionable Pattern for NeoTrix
**Adaptive Cycle Depth**: Replace fixed cycle count in ConsciousnessTree with adaptive halting. Track loop-time memory (previous iterations' key states). Halting head predicts: "Will one more cycle improve the result?" If yes, continue. If no, stop. This prevents wasting computation on already-converged domains while ensuring hard problems get enough depth. Maps to Axiom A2 (Context as Scarce Resource) — don't spend context tokens on cycles that won't improve the outcome.

---

## Cross-Paper Synthesis

| Dimension | ConvMem | MemoryLACE | Metis | AGAO | RecurTrace |
|-----------|---------|------------|-------|------|------------|
| **Memory Architecture** | Hierarchical convolution (parallel) | Lifecycle relations (merge/supersede/contradict) | Native in-model state (gradient-free) | N/A (attention orchestration) | Loop-time persistent state |
| **Attention Mechanism** | LLM-as-kernel over segments | Relation-aware retrieval | Memory attention parallel to original | Goal+Topology+Resource aware | Loop Memory Attention across iterations |
| **Compute Efficiency** | O(log N) parallel depth | 66.6% runtime reduction | Zero retrieval overhead (in-model) | Reduces unnecessary agent execution | Adaptive early stopping |
| **Training Required** | No | No | Mid-training only | No (learned routing) | Halting head only |
| **Primary Domain** | Long-context reasoning | Memory lifecycle management | Native memory capabilities | Multi-agent coordination | Latent reasoning depth |

## NeoTrix Integration Opportunity

The five papers converge on a single architectural insight: **memory and attention are not separate systems — they are two views of the same information flow.**

- ConvMem shows memory = parallel attention over hierarchical summaries
- MemoryLACE shows memory = lifecycle-tracked relations between atomic facts
- Metis shows memory = native state inside the model, not external
- AGAO shows attention = resource allocation across agents, not just tokens
- RecurTrace shows attention = persistent state across loop iterations

**NeoTrix should unify these into a single "Memory-Attention" primitive:**
1. **Hierarchical** (ConvMem) — summaries at multiple granularities
2. **Lifecycle-tracked** (MemoryLACE) — merge/supersede/contradict relations
3. **In-process** (Metis) — no external retrieval step for hot path
4. **Resource-aware** (AGAO) — budget allocation across memory tiers
5. **Adaptive depth** (RecurTrace) — stop when memory is sufficient, continue when not

This unified primitive maps directly to NT-MEMORY's KB retrieval + NT-CORE's GWT attention allocation, suggesting they should be a single subsystem with dual interface: query-by-retrieve (memory) and query-by-allocate (attention).
