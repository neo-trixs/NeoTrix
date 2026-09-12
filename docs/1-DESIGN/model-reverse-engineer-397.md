# Model Reverse Engineering — Cycle 397

**Date**: 2026-09-12  
**Focus**: Efficient inference, attention mechanisms, agent coordination, memory architectures, reasoning frameworks

---

## Paper 1: SparDA — Sparse Decoupled Attention for Long-Context Inference

**Source**: arXiv:2606.04511 (Jun 2026)  
**Authors**: Yaosheng Fu, Guangxuan Xiao, Xin Dong, Song Han, Oreste Villa

### Key Innovation
Fourth per-layer projection called **Forecast** alongside Q/K/V. Forecast predicts which KV blocks the next layer needs, enabling **lookahead selection** that overlaps CPU→GPU prefetch with current-layer execution. Decoupled from attention query → one Forecast head per GQA group (not per attention head). Adds <0.5% parameters.

### Results
- 1.25× prefill speedup, 1.7× decode speedup over sparse-attention offload baseline
- Up to 5.3× higher decode throughput via larger feasible batch sizes
- Trains only Forecast projections by matching original selector's attention distribution

### Pattern Extraction
```
Forecast Projection → KV Block Prediction → Lookahead Prefetch → Overlapped Execution
     ↓                      ↓                      ↓                      ↓
  <0.5% params         O(block) selection    CPU→GPU overlap      1.7× decode speedup
```

### NeoTrix Domain Mapping
| Domain | Integration |
|--------|------------|
| **NT-CORE** | GWT attention routing: Forecast projection as "attention prediction" — predict salient blocks before processing |
| **NT-MEMORY** | KV cache optimizer: lookahead prefetch for paged KV management (KVMem-style) |
| **NT-WORLD** | Perception pipeline: predict which sensory inputs will be relevant for next processing stage |

---

## Paper 2: HybridGen — CPU-GPU Hybrid Attention Framework

**Source**: arXiv:2604.18529 (Apr 2026)  
**Authors**: Mao Lin, Xi Wang, Guilherme Cox, Dong Li, Hyeran Jeon

### Key Innovation
Three breakthroughs for long-context KV cache management with CXL-expanded tiered memory:
1. **Attention Logit Parallelism** — split attention computation across CPU and GPU simultaneously
2. **Feedback-Driven Scheduler** — dynamically rebalances CPU/GPU load as sequences grow
3. **Semantic-Aware KV Cache Mapping** — maps KV blocks to memory tiers based on semantic importance

### Results
- 1.41×–3.2× average speedup over 6 SOTA KV cache management methods
- Maintains superior accuracy across 3 LLM models × 11 sizes × 3 GPU platforms

### Pattern Extraction
```
Semantic KV Mapping → Tiered Memory Placement → Feedback Scheduler → Parallel Attention
        ↓                        ↓                      ↓                    ↓
  importance scoring      GPU/CPU/CXL tiers      dynamic rebalance    CPU+GPU compute
```

### NeoTrix Domain Mapping
| Domain | Integration |
|--------|------------|
| **NT-MEMORY** | Tiered memory architecture: hot/warm/cold knowledge tiers with semantic importance scoring |
| **NT-CORE** | GWT attention: semantic-aware routing determines which memory tier processes each query |
| **NT-PHYSICAL** | Resource management: CPU/GPU load balancing for embodied agent compute |

---

## Paper 3: Attention as Binding — VSA Perspective on Transformer Reasoning

**Source**: arXiv:2512.14709 (Dec 2025, AAAI 2026 submission)  
**Authors**: Sahil Rajesh Dhayalkar

### Key Innovation
Unifies transformer attention with Vector Symbolic Architecture (VSA):
- **Queries/Keys** define role spaces (VSA role vectors)
- **Values** encode fillers (VSA filler vectors)
- **Attention weights** implement soft unbinding operator
- **Residual connections** realize superposition of bound structures

Proposes explicit **binding/unbinding heads** and **hyperdimensional memory layers**. Defines metrics for "VSA-likeness" and logical compositionality. Explains failure modes (variable confusion, inconsistency) as VSA algebra violations.

### Pattern Extraction
```
Q/K = Role Vectors → Attention = Soft Unbinding → Values = Fillers → Residual = Superposition
      ↓                       ↓                        ↓                    ↓
  role subspaces         differentiable           filler encoding    accumulative binding
                      unbinding operator                           across depth
```

### NeoTrix Domain Mapping
| Domain | Integration |
|--------|------------|
| **NT-CORE** | HyperCube as VSA implementation: binding = concept association, unbinding = concept retrieval |
| **NT-MIND** | Skill crystallization: bound role-filler pairs as skill templates |
| **NT-MEMORY** | Hyperdimensional memory layers: superposition for parallel concept storage |

---

## Paper 4: Disentangling Recall and Reasoning in Transformers

**Source**: arXiv:2510.03366 (Oct 2025, revised Mar 2026)  
**Authors**: Harshwardhan Fartale, Ashish Kattamuri, Rahul Raja, Arpita Vats, Ishita Prasad, Akshata Kishore Moharir

### Key Innovation
First **causal evidence** that recall and reasoning use separable but interacting circuits in transformers:
- **Recall circuits**: Disabling them reduces fact-retrieval accuracy by 15% while leaving reasoning intact
- **Reasoning circuits**: Disabling them reduces multi-step inference by comparable margin
- **Layer specialization**: Different layers specialize for recall vs reasoning
- **Head specialization**: Attention heads show differential activation patterns between task types

Uses activation patching and structured ablations across Qwen and LLaMA families.

### Pattern Extraction
```
Layer N: Recall Circuit (fact retrieval) ──→ selective impairment: -15% recall, 0% reasoning
Layer M: Reasoning Circuit (inference) ──→ selective impairment: -15% reasoning, 0% recall
                    ↓
         Separable but interacting circuits
                    ↓
         Task-specific firing patterns at neuron level
```

### NeoTrix Domain Mapping
| Domain | Integration |
|--------|------------|
| **NT-CORE** | GWT attention routing: separate "recall mode" vs "reasoning mode" with circuit-level dispatch |
| **NT-MEMORY** | Recall circuit equivalent: KB retrieval vs. reasoning chain construction |
| **NT-MIND** | Meta-cognition: detect which circuit type is active, optimize routing accordingly |

---

## Paper 5: SwarmSys — Decentralized Swarm-Inspired Multi-Agent Reasoning

**Source**: arXiv:2510.10047 (Oct 2025)  
**Authors**: (Multiple, swarm intelligence research group)

### Key Innovation
Three-role swarm framework: **Explorers** (breadth search), **Workers** (depth exploitation), **Validators** (consistency checking). Coordination via **pheromone-inspired traces** encoding contextual utility. Agents form debate–consensus cycles that update event profiles.

**Critical finding**: A swarm of GPT-4o-based agents approaches GPT-5 performance. **Scaling coordination can substitute for model scaling.**

Self-organized coordination emerges: clustering coefficient rises 0.28→0.47, global path lengths shorten. No central control needed.

### Pattern Extraction
```
Explorer Agents → Worker Agents → Validator Agents
      ↓                ↓                ↓
   breadth search   depth exploit   consistency check
      ↓                ↓                ↓
   Pheromone Traces (contextual utility encoding)
      ↓
   Debate–Consensus Cycles
      ↓
   Self-Organized Coordination (small-world emergence)
```

### NeoTrix Domain Mapping
| Domain | Integration |
|--------|------------|
| **NT-ACT** | Swarm orchestration: Explorer/Worker/Validator role specialization |
| **NT-CORE** | GWT salience: pheromone traces as attention modulation signals |
| **NT-MIND** | Self-evolution: swarm learning as distributed skill crystallization |
| **NT-MEMORY** | Distributed memory: pheromone traces as persistent cross-agent knowledge |

---

## Cross-Paper Pattern Synthesis

### Unified Architecture Implications for NeoTrix

| Paper | Core Pattern | NeoTrix Integration |
|-------|-------------|---------------------|
| SparDA | Forecast before processing | NT-CORE: Predict salient KB blocks before attention |
| HybridGen | Semantic tiered memory | NT-MEMORY: Hot/warm/cold knowledge tiers |
| Attention as Binding | VSA algebra in attention | NT-CORE: HyperCube binding/unbinding operations |
| Recall vs Reasoning | Circuit-level dispatch | NT-CORE: Mode-specific attention routing |
| SwarmSys | Coordination > scaling | NT-ACT: Swarm roles as domain specialization |

### Axiom Alignment

| Axiom | Paper Evidence |
|-------|---------------|
| **A1: Cost-Aware Routing** | SparDA Forecast: 0.5% params for 1.7× speedup. HybridGen: 3.2× via smart scheduling |
| **A2: Context as Scarce Resource** | HybridGen: tiered KV for million-token contexts. Mem0: 7K tokens vs 25K full-context |
| **A3: Skill as Production Template** | Attention as Binding: role-filler pairs = skill templates. SwarmSys: pheromone traces = learned patterns |

### Design Decisions Triggered

1. **Forecast Projection for HyperCube** (SparDA-inspired): Add predictive projection to HyperCube that anticipates which knowledge blocks will be needed → prefetch into attention window
2. **Semantic KV Tiering** (HybridGen-inspired): Implement hot/warm/cold knowledge tiers with importance scoring in NT-MEMORY
3. **VSA Binding Heads** (Attention as Binding): Add explicit binding/unbinding operations to HyperCube for logical compositionality
4. **Recall/Reasoning Mode Switch** (Circuit paper): Implement circuit-level dispatch in GWT — detect task type and route to specialized attention pathway
5. **Swarm Role Specialization** (SwarmSys): Explorer/Worker/Validator roles for NT-ACT multi-agent orchestration with pheromone-based coordination

---

## Action Items

| Priority | Item | Paper | Domain |
|----------|------|-------|--------|
| P0 | Design Forecast Projection spec for HyperCube | SparDA | NT-CORE |
| P0 | Spec semantic importance scoring for KB tiers | HybridGen | NT-MEMORY |
| P1 | Prototype VSA binding/unbinding heads | Attention as Binding | NT-CORE |
| P1 | Implement recall/reasoning mode detection | Circuit paper | NT-CORE |
| P2 | Design swarm role protocol with pheromone traces | SwarmSys | NT-ACT |
