# Model Reverse Engineer — Cycle 347

**Date**: 2026-09-11
**Focus**: Efficient inference, attention mechanisms, agent coordination, memory architectures
**Sources**: arXiv, ACL 2026, ACL Findings 2026

## 5 New Papers (Not in Cycles 318-346)

### 1. AGAO: Adaptive Goal-aware Attention Orchestration for Multi-Agent Graph Systems (arXiv:2607.23678)
- **URL**: https://arxiv.org/abs/2607.23678
- **Category**: Multi-Agent Attention Routing
- **Core Mechanism**: Extends attention from token-level representation learning to workflow-level agent coordination. Three complementary attention mechanisms:
  - **Goal-aware Attention**: Measures semantic alignment between user objectives and agent capabilities
  - **Topology-aware Attention**: Incorporates graph structural dependencies and execution structure
  - **Resource-aware Attention**: Translates attention scores into practical execution decisions (model selection, token budget, priority)
- **Key Insight**: Treats agents as dynamically selectable computational units rather than fixed workflow operators. Transforms static agent graphs into adaptive execution systems that focus on goal-critical reasoning paths.
- **Results**: Improves task effectiveness while reducing unnecessary computation, latency, and token consumption across diverse multi-agent workloads. Introduces "Attention Engineering" as a new paradigm.
- **NeoTrix Domain Mapping**:
  - **NT-CORE (GWT)**: Attention Orchestration = GWT salience routing extended to multi-agent graph level. Goal-aware attention = task-conditioned salience scoring. Resource-aware attention = cost-aware routing (Axiom A1).
  - **NT-ACT**: Dynamic agent selection based on attention scores = NT-ACT orchestration with GWT-modulated routing.
  - **NT-MIND**: Attention-driven execution = SEAL pipeline stage importance weighting.

### 2. Gated-Memory Routing: Efficient Collaboration in Multi-Agent LLM Systems (arXiv:2609.00237)
- **URL**: https://arxiv.org/abs/2609.00237
- **Category**: Memory-Gated Agent Coordination
- **Core Mechanism**: Conditions each routing decision on a learned, gated execution memory rather than raw history. Four complementary components:
  - **Memory Write Gate**: Commits only non-redundant reasoning steps (novelty + relevance filtering)
  - **Retrieval Gate**: Supplies each agent a compact, relevant subset of memory
  - **History-Aware Role Allocator**: Selects next agent from gated state
  - **Adaptive Halting Controller**: Stops execution when memory contains sufficient evidence
- **Key Insight**: Memory as active control signal, not passive storage. Write and retrieval decisions trained jointly with role allocation, backbone routing, and halting under task-level reward. "Memory not only supplies context but also determines who acts next, what they read, and when collaboration stops."
- **Results**: Best average accuracy across 5 benchmarks, exceeding strongest baseline by 2.44 points. Reduces HumanEval inference cost by 31.9%. Adaptive halting terminates easy queries early.
- **NeoTrix Domain Mapping**:
  - **NT-MEMORY**: Gated memory = experience-tree with active write/retrieval gates. Write gate = experience-tree distillation phase (novelty filtering). Retrieval gate = experience-tree branch loading with route table matching.
  - **NT-CORE (GWT)**: Role allocation from gated memory = GWT salience routing conditioned on execution state. Adaptive halting = GWT attention budget exhaustion detection.
  - **NT-ACT**: Jointly trained orchestration = NT-ACT task delegation with memory-conditioned routing.

### 3. Dual-Layer Agentic Memory with Fast Write Routing and Slow Consolidation (arXiv:2608.22215)
- **URL**: https://arxiv.org/abs/2608.22215
- **Category**: Complementary Learning Systems for Agent Memory
- **Core Mechanism**: Inspired by neuroscience Complementary Learning Systems (CLS) theory. Two memory layers:
  - **Fast Write Routing**: Cost-aware epistemic routing via small-to-large model cascade (1.7B→8B). Incoming information categorized as non-write/write-new/write-update. Cascade minimizes routing overhead while filtering redundant memories.
  - **Slow Consolidation**: Periodic parametric consolidation — high-value external memories selectively consolidated into model parameters via supervised fine-tuning.
- **Key Insight**: "The core challenge is not retrieval alone, but managing the knowledge lifecycle: deciding what to externalize, update, or ultimately internalize." Unified paradigm: selective externalization followed by selective internalization.
- **Results**: 1.7B/8B cascade prunes up to 68% redundant external memory while escalating <50% of inputs. Retains >98% downstream QA EM. Periodic consolidation enables adaptive suppression of redundant writes as model's epistemic boundaries evolve.
- **NeoTrix Domain Mapping**:
  - **NT-MEMORY**: CLS-inspired memory = experience-tree two-tier design (fast write to KB, slow consolidation to model). Write routing cascade = experience-tree absorption with cost-aware routing. Consolidation = experience-tree distillation phase with model parameter updates.
  - **NT-MIND**: Parametric consolidation = SEAL pipeline experience crystallization (external KB → internal model parameters). Epistemic boundary evolution = SelfModel uncertainty tracking.
  - **NT-CORE**: Epistemic routing = GWT salience scoring for memory write decisions.

### 4. Mnemis: Dual-Route Retrieval on Hierarchical Graphs for Long-Term LLM Memory (ACL 2026)
- **URL**: https://aclanthology.org/2026.acl-long.1096.pdf
- **Category**: System-1/System-2 Memory Retrieval
- **Core Mechanism**: Integrates System-1 similarity search with System-2 global selection for memory retrieval:
  - **System-1 Similarity Search**: Base graph for fast semantic retrieval (embedding search + BM25 + RRF re-ranking)
  - **System-2 Global Selection**: Hierarchical graph for deliberate top-down traversal. Bottom-up category construction with three principles: Minimum Concept Abstraction, Many-to-Many Mapping, Compression Efficiency Constraint.
- **Key Insight**: Real-world queries benefit from combining fast similarity (System-1) with deliberate hierarchical traversal (System-2). "System-1 provides fine-grained semantic similarity evidence, while System-2 retrieves structurally relevant items that may be semantically distant yet relationally important."
- **Results**: SOTA on LoCoMo (93.9) and LongMemEval-S (91.6) using GPT-4.1-mini. Hierarchical graph with many-to-many mapping enables multi-perspective retrieval.
- **NeoTrix Domain Mapping**:
  - **NT-MEMORY**: Dual-route retrieval = experience-tree with route table matching (System-1) + hierarchical branch traversal (System-2). Hierarchical graph = experience-tree hub index with layered category structure. Many-to-many mapping = experience-tree branch cross-references.
  - **NT-CORE (GWT)**: System-1/System-2 split = GWT fast-path (salience-driven) + slow-path (deliberative reasoning) attention routing.
  - **NT-WORLD**: Category construction = NT-WORLD domain knowledge organization.

### 5. HELENA: Hierarchical Sparse Coordination over Union of Complementary Topologies (arXiv:2608.04634)
- **URL**: https://arxiv.org/abs/2608.04634
- **Category**: Topology Diversity with Noise Control
- **Core Mechanism**: Addresses the dilemma that single topology restricts reasoning while composite topologies propagate noise. Three-stage solution:
  - **Union Graph Construction**: Monte Carlo Tree Search explores topology space, selects complementary subset via Determinantal Point Process, merges into union graph preserving diverse reasoning paths
  - **Hierarchical Sparse Coordination**: Activates only sparse subgraph at each step. Node-Level Memory Composer removes irrelevant records. Edge-Level Sparse Activation restricts communication to selected paths. Agents exchange compressed latent briefs.
  - **Local Self-Refinement**: Identifies decision units with discrepancy evidence, rewrites only when contrastive evidence confirms reliable failure + challenger improvement.
- **Key Insight**: "Fusing complementary topologies captures diverse reasoning perspectives, while restricting irrelevant node communication prevents noise propagation." Sparse activation over union graph achieves diversity without noise.
- **Results**: SOTA on all 8 benchmarks. Average gain 3.47% over strongest baseline. Up to 10.34% on MMLU-Pro. Larger improvements on harder benchmarks at reasonable cost. Difficulty-adaptive token efficiency.
- **NeoTrix Domain Mapping**:
  - **NT-CORE (GWT)**: Union graph = GWT attention routing over multiple parallel reasoning paths. Sparse activation = GWT salience-based attention gating (only top-salience agents activate). Compressed latent briefs = VSA HyperCube compressed state representations.
  - **NT-ACT**: Topology search = NT-ACT orchestration with dynamic topology selection. Hierarchical sparse coordination = NT-ACT resource-aware task delegation.
  - **NT-MEMORY**: Node-Level Memory Composer = experience-tree branch pruning. Compressed briefs = experience-tree entry compression for cross-agent state sharing.

## Cross-Cutting Patterns (Cycle 347)

| Pattern | Papers | NeoTrix Mapping |
|---------|--------|-----------------|
| **Attention as Orchestration Primitive** | AGAO | GWT salience routing extended to workflow-level agent coordination — attention engineering as new paradigm |
| **Memory as Active Control Signal** | Gated-Memory Routing, Dual-Layer Memory | Memory determines who acts next, what they read, when to stop — not passive storage but active routing control |
| **CLS-Inspired Two-Tier Memory** | Dual-Layer Memory, Mnemis | Fast write + slow consolidation; System-1 similarity + System-2 hierarchy — complementary memory pathways |
| **Sparse Activation over Dense Topology** | HELENA, Gated-Memory Routing | Activate only what's needed from a rich topology — diversity without noise, efficiency without loss |
| **Epistemic Routing with Cost Awareness** | Dual-Layer Memory, AGAO | Model cascade for routing decisions, resource-aware attention — cheapest capable model for each subtask |

## NeoTrix Absorption Priority

| Priority | Paper | Pattern | Target Domain |
|----------|-------|---------|---------------|
| P0 | AGAO | Attention Engineering for multi-agent orchestration | NT-CORE (GWT) |
| P0 | Gated-Memory Routing | Memory-gated routing with adaptive halting | NT-MEMORY + NT-ACT |
| P0 | Dual-Layer Memory | CLS-inspired fast write + slow consolidation | NT-MEMORY |
| P1 | Mnemis | System-1/System-2 dual-route memory retrieval | NT-MEMORY |
| P1 | HELENA | Sparse activation over union topology | NT-CORE (GWT) + NT-ACT |
