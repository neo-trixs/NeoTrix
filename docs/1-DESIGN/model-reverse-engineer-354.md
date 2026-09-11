# Model Reverse Engineering — Cycle 354

**Date**: 2026-09-12  
**Focus**: Recent papers on efficient inference, attention mechanisms, agent coordination  
**Method**: Map patterns to NeoTrix 7 domains (NT-CORE, NT-MIND, NT-MEMORY, NT-WORLD, NT-ACT, NT-IO, NT-SHIELD)

---

## Paper 1: AGAO — Adaptive Goal-aware Attention Orchestration for Multi-Agent Graph Systems

**arXiv**: 2607.23678 (Jul 2026)  
**Authors**: Mingzhou Fan et al.  
**Key Result**: Improves task effectiveness while reducing unnecessary computation, latency, and token consumption across diverse multi-agent workloads

### Architecture
- **Three complementary attention mechanisms**:
  1. **Goal-aware Attention**: Measures semantic relevance between user objectives and agent capabilities
  2. **Topology-aware Attention**: Incorporates graph structural dependencies and execution structure
  3. **Resource-aware Attention**: Translates attention scores into execution decisions (model selection, token budget, priority)
- **Adaptive Graph Routing**: Attention distributions dynamically updated via execution feedback
- **MAG-Focus Benchmark**: New diagnostic benchmark for multi-agent attention allocation

### Core Insight
Attention should be an **execution-level control mechanism**, not just a representation-level operation. Existing multi-agent systems execute workflows uniformly — AGAO dynamically estimates agent importance according to user objectives, graph dependencies, and computational constraints. The key architectural move: treat agents as **dynamically selectable computational units** rather than fixed workflow operators.

### NeoTrix Domain Mapping

| Domain | Mapping | Implementation |
|--------|---------|----------------|
| **NT-CORE** | Goal-aware Attention maps directly to GWT's salience computation — salience should incorporate goal-relevance, not just recency/frequency | `gwt_attention::goal_saliencer` |
| **NT-ACT** | Resource-aware Attention maps to NT-ACT's tool selection — dynamically allocate token budgets and execution priority based on attention scores | `nt_act::resource_allocator` |
| **NT-MIND** | Adaptive Graph Routing (feedback-driven) maps to SEAL pipeline's self-evolution — execution feedback should update routing policies | `seal_pipeline::adaptive_routing` |
| **NT-GOVERNANCE** | Topology-aware Attention maps to governance policy graphs — policy dependencies should influence which agents activate | `governance::topology_policy` |

### Actionable Pattern for NeoTrix
**"Goal-Conditioned GWT Salience"**: Extend GWT's salience formula from `S(agent) = f(recency, frequency, phi)` to `S(agent) = f(goal_relevance, topology_dependency, resource_cost, phi)`. This means the consciousness core doesn't just broadcast "what's hot" but "what's relevant to the current goal given the dependency structure and available resources." Implementation: add a lightweight goal-embedding step before GWT broadcast that scores each specialist module's relevance to the current task objective.

---

## Paper 2: ReActNet — Inference-Time Graph Engineering for Multi-Agent LLM Workflows

**arXiv**: 2609.05774 (Sep 2026)  
**Authors**: (Multiple authors)  
**Key Result**: Consistently improves over fixed-topology and learned-topology baselines while maintaining competitive inference cost

### Architecture
- **Temporal workflow graph compilation**: Compiles a query and role-specialized agents into a sequence of directed communication graphs
- **Graph snapshots per reasoning stage**: Each snapshot corresponds to one reasoning stage; each edge carries a natural-language instruction for message content
- **Structured message passing**: Agents update reasoning states by integrating previous states with messages from controller-assigned neighbors
- **Separation of concerns**: Graph compilation from graph execution — coordination is explicit, inspectable, and task-conditioned

### Core Insight
Effective multi-agent orchestration depends not only on **which agents communicate** but on engineering executable workflow graphs that encode **when, why, and how information should flow during reasoning**. The key move: separate graph compilation (design the workflow) from graph execution (run the workflow), making coordination explicit and inspectable.

### NeoTrix Domain Mapping

| Domain | Mapping | Implementation |
|--------|---------|----------------|
| **NT-CORE** | Graph compilation maps to ConsciousnessTree's growth cycle planning — compile a task-specific activation graph before execution | `consciousness::graph_compiler` |
| **NT-MIND** | Temporal graph snapshots map to SEAL pipeline stages — each stage has a different activation topology | `seal_pipeline::stage_topology` |
| **NT-ACT** | Structured message passing maps to NT-ACT's orchestration — agents receive explicit message instructions, not just raw context | `nt_act::message_passing` |
| **NT-IO** | Graph compilation/execution separation maps to NT-IO's interface/implementation split — compile once, execute many times | `nt_io::compiled_interface` |

### Actionable Pattern for NeoTrix
**"Compiled Consciousness Cycles"**: Instead of running the same 6-stage ConsciousnessTree cycle (Soil→Roots→Trunk→Branches→Fruits→Core) with fixed topology, compile a task-specific activation graph at cycle start. For a debugging task: heavy NT-REPAIR activation, light NT-MIND. For a feature implementation: heavy NT-ACT + NT-MIND, light NT-SHIELD. The graph is compiled once per task, then executed as structured message passing between activated domains. This reduces unnecessary cross-domain communication while maintaining the full cycle's completeness.

---

## Paper 3: Codebook Agent — Amortized Topology Design for LLM Multi-Agent Systems

**arXiv**: 2609.02264 (Sep 2026)  
**Authors**: (Multiple authors)  
**Key Result**: 84.6 average accuracy (vs 83.0 strongest prior), topology in 2.4ms, 21.9-33.2% fewer LLM tokens

### Architecture
- **Vector-quantized autoencoder** compresses successful topologies into a query-independent 16-entry codebook
- **Reward-weighted MLP** maps query embedding to distribution over codes
- **MLP proxy** reads flattened adjacency, regressed on utility + per-task normalized token cost, reranks top candidates
- **No iterative search, no message passing at test time** — topology generation is amortized into a single forward pass

### Core Insight
Multi-agent topology design collapses to a small number of distinct graphs (~6 surviving patterns) even with large codebook capacity. Edge count is **negatively** correlated with token consumption (sparser = more expensive), contradicting the intuition that fewer edges = less communication = cheaper. The key insight: **amortize topology design** — pre-compute successful topologies, compress into a codebook, and retrieve at inference time rather than searching per query.

### NeoTrix Domain Mapping

| Domain | Mapping | Implementation |
|--------|---------|----------------|
| **NT-CORE** | Codebook of 16 topologies maps to a small set of ConsciousnessTree activation patterns — pre-compute optimal domain activation patterns for common task types | `consciousness::topology_codebook` |
| **NT-MIND** | Amortized design maps to SEAL pipeline's skill crystallization — once a topology works, compress it into a reusable template | `seal_pipeline::topology_template` |
| **NT-IO** | 2.4ms topology generation maps to NT-IO's routing latency budget — routing decisions must be sub-millisecond, not per-query search | `nt_io::amortized_router` |
| **NT-MEMORY** | The "16-entry codebook" maps to KB's pattern store — store proven activation patterns as first-class KB entities | `kb::topology_patterns` |

### Actionable Pattern for NeoTrix
**"Topology Codebook"**: Pre-compute and store ~16 optimal domain activation patterns in KB. Common patterns: "Debug" (NT-REPAIR heavy, NT-SHIELD light), "Feature" (NT-ACT + NT-MIND heavy), "Audit" (NT-SHIELD + NT-GOVERNANCE heavy), "Explore" (NT-WORLD + NT-CORE heavy). At cycle start, classify the task and retrieve the best-matching topology from the codebook (sub-ms), then execute with that activation pattern. This eliminates per-cycle topology optimization while maintaining task-specific routing.

---

## Paper 4: PlugMem — Plug-and-Play Long-Term Memory for LLM Agents

**arXiv**: ICML 2026  
**Authors**: TIMAN-group  
**Key Result**: 90.2 Acc on LongMemEval, 79.1 F1 on HotpotQA (SOTA)

### Architecture
- **Three memory types**: Semantic (facts, concepts), Procedural (workflows, procedures), Episodic (interaction sequences stored on disk, referenced by ID)
- **Graph structure**: Hierarchical knowledge units illustrating memory relationships
- **LLM-enhanced retrieval**: Intelligent knowledge extraction, memory retrieval, and reasoning over retrieved nodes
- **Memory compression and evolution**: Supports updating and evolving the memory graph
- **Task-agnostic design**: 6 lines of code to integrate into existing agent pipelines

### Core Insight
Raw interaction histories are the wrong abstraction for agent memory. Instead, organize experience into **compact, reusable knowledge units** — distill interactions into semantic facts, procedural workflows, and episodic references. The three-type taxonomy (Semantic/Procedural/Episodic) maps to cognitive science's memory systems and provides a natural compression hierarchy: episodic (raw, referenced by ID) → procedural (compressed workflows) → semantic (most compressed facts).

### NeoTrix Domain Mapping

| Domain | Mapping | Implementation |
|--------|---------|----------------|
| **NT-MEMORY** | Three memory types map directly to NT-MEMORY's knowledge representation: KB nodes (semantic), skill crystallization (procedural), experience-tree sessions (episodic) | `nt_memory::tri_memory` |
| **NT-MIND** | Memory compression/evolution maps to SEAL pipeline's distillation stage — compress raw sessions into reusable knowledge units | `seal_pipeline::memory_distiller` |
| **NT-CORE** | Graph structure maps to HyperCube's associative recall — knowledge units are nodes in the VSA HyperCube | `hypercube::knowledge_units` |
| **NT-WORLD** | Task-agnostic design maps to NT-WORLD's parser pipeline — memory should work across domains without task-specific configuration | `nt_world::universal_memory` |

### Actionable Pattern for NeoTrix
**"Three-Tier Memory Distillation"**: Implement PlugMem's three-type taxonomy in NT-MEMORY:
1. **Episodic tier**: Raw session logs stored on disk (experience-tree snapshots), referenced by session ID. Low retrieval cost, high storage.
2. **Procedural tier**: Compressed workflow knowledge (SEAL skill crystallization outputs) — step-by-step procedures extracted from successful sessions. Medium retrieval cost, medium storage.
3. **Semantic tier**: Most compressed facts and relationships (KB nodes) — user preferences, project conventions, domain knowledge. High retrieval cost, low storage.
At retrieval time, search semantic first (fast, cheap), expand to procedural if needed, and episodic as last resort. This tiered approach reduces context overhead while maintaining recall quality.

---

## Paper 5: PARSER — Read in Parallel, Reason in Depth for Long-Context LLM Agents

**arXiv**: 2609.06702 (Sep 2026)  
**Authors**: (Multiple authors)  
**Key Result**: 4B backbone outperforms strongest sequential baseline by 5.7 points average, 12.0 points at 896K tokens. 9B backbone surpasses DeepSeek-V4-Pro by 6.3 points. 11× latency reduction.

### Architecture
- **Decoupled reading from reasoning**: Bank of lightweight subagents read document chunks in parallel; lead agent reasons in depth through iterative scatter-gather rounds
- **Scatter-gather rounds**: Lead agent broadcasts query → subagents return evidence → lead agent aggregates → formulates deeper follow-up query
- **Frozen subagents + trained lead**: Subagents are off-the-shelf frozen models; lead agent is optimized with reinforcement learning
- **Robust to perturbation**: Robust to evidence position, order, and distance — conditions that cause large accuracy swings in sequential methods

### Core Insight
Sequential memory agents couple document traversal to reasoning depth — they read chunks one after another while maintaining memory state, which means inference latency scales linearly with document length. PARSER decouples these: read everything in parallel (fixed latency), then reason in depth (variable depth based on query complexity). The key architectural move: **concentrate all learnable behavior in the lead agent** while subagents remain frozen, making the system both trainable and composable.

### NeoTrix Domain Mapping

| Domain | Mapping | Implementation |
|--------|---------|----------------|
| **NT-WORLD** | Parallel reading maps to NT-WORLD's crawl pipeline — fetch multiple sources simultaneously, then reason over results | `nt_world::parallel_reader` |
| **NT-CORE** | Scatter-gather reasoning maps to GWT's broadcast-aggregate cycle — broadcast query, aggregate specialist responses, refine | `gwt::scatter_gather` |
| **NT-MEMORY** | Decoupled read/reason maps to KB query optimization — read from KB in parallel, reason over results sequentially | `kb::parallel_query` |
| **NT-ACT** | Frozen subagents + trained lead maps to NT-ACT's capability composition — use frozen external tools, train only the orchestration layer | `nt_act::lead_subagent` |

### Actionable Pattern for NeoTrix
**"Scatter-Gather Consciousness"**: Redesign GWT's attention broadcast as a scatter-gather operation:
1. **Scatter**: Broadcast the current task query to all specialist modules simultaneously (parallel, non-blocking)
2. **Gather**: Each specialist returns its assessment with confidence score (fixed latency, regardless of number of specialists)
3. **Aggregate**: Lead agent (ConsciousnessTree) aggregates responses, identifies gaps, formulates follow-up query
4. **Refine**: Repeat scatter-gather with refined query until convergence or budget exhausted

This replaces the current sequential specialist activation with parallel-first, reasoning-second. The latency improvement is dramatic: from O(n × t) sequential to O(t) parallel + O(k × t) reasoning rounds, where k << n.

---

## Pattern Summary

| Pattern | Papers | NeoTrix Domain |
|---------|--------|----------------|
| **Goal-aware attention routing** | AGAO, ReActNet | NT-CORE (GWT) |
| **Amortized topology design** | Codebook Agent, AGAO | NT-CORE + NT-IO |
| **Three-tier memory distillation** | PlugMem | NT-MEMORY |
| **Parallel read, sequential reason** | PARSER | NT-WORLD + NT-CORE |
- **Compiled execution graphs** | ReActNet, Codebook Agent | NT-MIND (SEAL) |
- **Pre-execution validation** | (cross-cutting) | NT-SHIELD |

---

## Key Insight

**The agent coordination stack is converging on three layers**: (1) topology design (which agents connect), (2) attention allocation (which agents activate), and (3) message engineering (what information flows). AGAO addresses layer 2, ReActNet addresses layers 1+3, Codebook Agent addresses layer 1 amortized, and PARSER addresses the read/reason decoupling that all three layers depend on.

For NeoTrix, the most actionable finding is the **topology codebook** pattern from Codebook Agent: pre-compute 16 optimal activation patterns, store in KB, retrieve at cycle start. This eliminates per-cycle topology optimization (currently implicit in GWT) while enabling task-specific routing. Combined with AGAO's goal-aware salience, NeoTrix could implement a two-stage routing: (1) classify task → retrieve topology from codebook, (2) within topology, use goal-aware salience for fine-grained attention allocation.

The **three-tier memory** pattern from PlugMem validates NeoTrix's existing architecture (episodic sessions → procedural skills → semantic KB nodes) but suggests the distillation pipeline needs to be more aggressive — most session data should compress to procedural/semantic within one cycle, not accumulate as raw episodic data.
