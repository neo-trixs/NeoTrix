# Model Reverse Engineering — Cycle 335 (2026-09-11)

## Summary

5 new AI models/papers reverse-engineered for pattern extraction and NeoTrix domain mapping. Focus: efficient KV cache management, multi-agent attention coordination, speculative decoding for agents.

---

## Paper 1: LycheeCluster — Structure-Aware KV Cache Management

**Source**: ACL 2026 Findings (arXiv:2603.08453)
**Authors**: Dongfang Li, Zixuan Liu, Gang Lin, Baotian Hu, Min Zhang

### Core Innovation
LycheeCluster transforms KV cache retrieval from linear scan into logarithmic-time pruning via:
1. **Boundary-aware chunking** — preserves local semantic coherence (not fixed-size chunks)
2. **Recursive hierarchical index** — rooted in triangle inequality for theoretically bounded pruning
3. **Lazy update strategy** — supports efficient streaming generation without re-clustering

### Key Results
- **3.6× end-to-end inference speedup** at 64K context length
- Negligible model performance degradation
- Outperforms Quest, ClusterKV, ArkVale, RaaS, ShadowKV, RazorAttention

### Pattern Extraction
| Pattern | Mechanism | NeoTrix Mapping |
|---------|-----------|-----------------|
| Semantic chunking | Boundary-aware splitting preserves coherence | KB chunking for experience-tree |
| Hierarchical index | Triangle inequality → O(log n) pruning | KB node hierarchy, ConsciousnessTree branching |
| Lazy updates | Streaming without re-clustering | KB pipeline incremental indexing |

### NeoTrix Domain Mapping
- **NT-MEMORY**: Direct — KB KV cache optimization for long-context experience retrieval
- **NT-CORE**: GWT attention routing can use hierarchical index for salience scoring
- **NT-IO**: LLM context window management for provider calls

### Absorption Verdict
**ABSORB** — Their boundary-aware chunking is directly applicable to our KB experience storage. The hierarchical index pattern can enhance our ConsciousnessTree branch navigation. Lazy update strategy maps to our SEAL pipeline's incremental knowledge absorption.

---

## Paper 2: Attention-MoA — Inter-Agent Semantic Attention

**Source**: arXiv:2601.16596 (Jan 2026)
**Authors**: Jianyu Wen, Yang Wei, Xiongxi Yu, Changxuan Xiao, Ke Zeng

### Core Innovation
Attention-MoA redefines Mixture-of-Agents collaboration through:
1. **Inter-agent Semantic Attention** — agents critique and refine each other using natural language (not concatenation)
2. **Inter-layer Residual Module** — accumulates historical context to prevent information degradation
3. **Adaptive Early Stopping** — dynamically controls inference depth

### Key Results
- **91.15% LC Win Rate** on AlpacaEval 2.0
- Small open-source ensemble (12B-32B) outperforms Claude-4.5-Sonnet and GPT-4.1
- MT-Bench score of 8.83

### Pattern Extraction
| Pattern | Mechanism | NeoTrix Mapping |
|---------|-----------|-----------------|
| Semantic critique-refine | Natural language peer feedback within layer | GWT broadcast + specialist refinement |
| Residual synthesis | Historical context accumulation across layers | SEAL pipeline stage output carry-forward |
| Adaptive depth | Dynamic termination based on quality assessment | ConsciousnessTree cycle depth modulation |

### NeoTrix Domain Mapping
- **NT-CORE**: GWT attention — their inter-agent semantic attention is a multi-model analog of our GWT broadcast
- **NT-MIND**: SEAL pipeline — residual synthesis maps to cross-stage knowledge carry-forward
- **NT-IO**: Provider routing — small model ensembles outperforming large single models validates cost-aware routing

### Absorption Verdict
**ABSORB** — Their critique-refine pattern is the multi-model version of our GWT specialist refinement. The adaptive early stopping mechanism should inform our SEAL pipeline's stage termination logic. The ensemble-beats-frontier result reinforces A1 (Cost-Aware Routing).

---

## Paper 3: AgentInfer — Co-Design of Inference Architecture and System

**Source**: arXiv:2512.18337v2 (Feb 2026)
**Authors**: Weizhe Lin et al. (Huawei)

### Core Innovation
AgentInfer bridges inference optimization and architectural design through 4 synergistic components:
1. **AgentCollab** — hierarchical dual-model reasoning (large + small model with dynamic role assignment)
2. **AgentSched** — cache-aware hybrid scheduler for heterogeneous request patterns
3. **AgentSAM** — suffix-automaton speculative decoding reusing multi-session semantic memory
4. **AgentCompress** — async semantic compression of agent memory without disrupting reasoning

### Key Results
- **50%+ reduction** in ineffective token consumption
- **1.8-2.5× speedup** with preserved accuracy
- Self-Evolution Engine for long-horizon reasoning tasks

### Pattern Extraction
| Pattern | Mechanism | NeoTrix Mapping |
|---------|-----------|-----------------|
| Dynamic role assignment | Large/small model routing per task | GWT salience + cost weight routing |
| Cross-session memory reuse | Suffix automaton for token prediction | NT-NEXUS cross-session experience |
| Async compression | Memory distillation without reasoning disruption | SEAL pipeline background absorption |
| Cache-aware scheduling | Hybrid scheduler for heterogeneous loads | NT-ACT task scheduling with resource awareness |

### NeoTrix Domain Mapping
- **NT-IO**: Provider routing — AgentCollab's dynamic role assignment is exactly our GWT routing
- **NT-MEMORY**: AgentCompress's async compression maps to SEAL pipeline's background knowledge distillation
- **NT-NEXUS**: AgentSAM's cross-session memory reuse is our experience-tree's lazy branch loading
- **NT-ACT**: AgentSched's cache-aware scheduling is our parallel_task with resource budget

### Absorption Verdict
**ABSORB (HIGH PRIORITY)** — This paper is a systems-level blueprint that validates multiple NeoTrix design decisions simultaneously. Their "Self-Evolution Engine" concept directly parallels our SEAL pipeline. AgentSAM's cross-session memory reuse is the inference-level analog of our NT-NEXUS experience bridges.

---

## Paper 4: Explicit Trait Inference for Multi-Agent Coordination (ETI)

**Source**: ACL 2026 Main Conference (arXiv:2604.19278)
**Authors**: Suhaib Abdurahman et al. (Amazon)

### Core Innovation
ETI enables agents to infer and track partner characteristics along two psychological dimensions:
1. **Warmth** (trust, credibility) — inferred from interaction quality
2. **Competence** (skill, reliability) — inferred from task performance

Agents use these trait profiles to guide coordination decisions.

### Key Results
- **45-77% reduction** in payoff loss in economic games
- **3-29% improvement** on MultiAgentBench
- First systematic evidence that LLM agents can infer others' traits from interaction histories

### Pattern Extraction
| Pattern | Mechanism | NeoTrix Mapping |
|---------|-----------|-----------------|
| Trait inference | Warmth + Competence profiles from history | SelfModel performance tracking |
| Profile-guided coordination | Use traits to select collaboration partners | GWT routing based on module health |
| Lightweight adaptation | No fine-tuning, inference-time only | SEAL pipeline runtime adaptation |

### NeoTrix Domain Mapping
- **NT-CORE**: SelfModel — trait inference is a formalized version of our module health monitoring
- **NT-MIND**: SEAL pipeline — profile-guided adaptation maps to self-evolution strategy selection
- **NT-GOVERNANCE**: Warmth/competence profiles for module trust scoring

### Absorption Verdict
**ABSORB** — ETI's warmth/competence dual-axis profiling is a psychologically grounded approach to module health assessment. Our SelfModel's capability/uncertainty/fatigue tracking is the module-level analog. ETI's lightweight (no fine-tuning) approach validates our runtime adaptation philosophy.

---

## Paper 5: AgentSpec — Speculative Decoding for Batch Agent Inference

**Source**: EMNLP 2026 (arXiv:2608.24004)
**Authors**: Xin Wang et al. (Ohio State + Microsoft Research)

### Core Innovation
AgentSpec addresses speculative decoding failures in batch agent inference:
1. **Structure-isolated drafting** — constrains speculation to semantically coherent agent workflow segments
2. **Redundancy-aware budget allocation** — uses agent-level info to allocate dynamic token budgets
3. **Cached token-to-string mapping** — avoids tokenizer invocation during online conversion

### Key Results
- Addresses two dominant bottlenecks: high rejection rate + under-utilized dynamic budgets
- Consistently faster than autoregressive decoding in batch settings (up to 2.02× speedup)
- Works across 5 workloads and 4 LLM families in vLLM

### Pattern Extraction
| Pattern | Mechanism | NeoTrix Mapping |
|---------|-----------|-----------------|
| Structure-isolated speculation | Constrain to coherent workflow segments | SEAL pipeline stage boundaries |
| Redundancy-aware budgets | Agent-level info guides token allocation | Resource budget per task type |
| Cached token mapping | Avoid repeated tokenization | KB token cache for experience retrieval |

### NeoTrix Domain Mapping
- **NT-IO**: Inference optimization — speculative decoding for our LLM provider calls
- **NT-ACT**: Task scheduling — redundancy-aware budgets map to our parallel_task resource allocation
- **NT-MEMORY**: Token caching for KB experience retrieval

### Absorption Verdict
**ABSORB** — AgentSpec's structure-isolated drafting maps to our SEAL pipeline's stage boundaries (don't speculate across stages). Redundancy-aware budget allocation is directly applicable to our ResourceBudgetManager. The batch inference optimization is critical for multi-agent NeoTrix scenarios.

---

## Cross-Paper Synthesis

### Emergent Pattern: Self-Evolution at Three Levels

| Level | Paper | Mechanism | NeoTrix Mapping |
|-------|-------|-----------|-----------------|
| **Inference** | AgentInfer | Cross-session memory reuse + async compression | NT-NEXUS experience bridges |
| **Collaboration** | Attention-MoA | Semantic critique-refine + adaptive depth | GWT broadcast + SEAL stage termination |
| **Knowledge** | LycheeCluster | Hierarchical KV index + boundary-aware chunking | KB node hierarchy + experience-tree chunking |

### Key Insight: Agent Coordination Converges on Trait Profiling

ETI (warmth/competence) + Attention-MoA (semantic attention) + AgentInfer (dynamic role assignment) all converge on the same principle: **agents must model each other to coordinate effectively**. This validates NeoTrix's SelfModel as a first-class architectural component.

### Absorption Priority Matrix

| Priority | Paper | Domain | Action |
|----------|-------|--------|--------|
| **P0** | AgentInfer | NT-IO + NT-MEMORY | Integrate cross-session memory reuse into NT-NEXUS |
| **P1** | Attention-MoA | NT-CORE | Adapt semantic critique-refine for GWT specialist broadcast |
| **P1** | LycheeCluster | NT-MEMORY | Implement boundary-aware chunking for KB experience storage |
| **P2** | ETI | NT-CORE + NT-GOVERNANCE | Formalize warmth/competence profiling for module health |
| **P2** | AgentSpec | NT-ACT | Apply structure-isolated speculation to SEAL pipeline stages |

### Contradictions & Resolutions

| Tension | Resolution |
|---------|-----------|
| LycheeCluster's hierarchical index vs. NeoTrix's flat KB nodes | Adopt hierarchical index as a retrieval optimization; keep flat nodes as the canonical storage model |
| Attention-MoA's small ensemble vs. NeoTrix's single-model routing | Ensemble approach is for quality-critical paths; routing is for cost optimization. Both coexist. |
| ETI's trait inference overhead vs. real-time constraints | Run trait inference asynchronously, cache results in SelfModel. Not per-turn. |
