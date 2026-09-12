# Model Reverse Engineering — Cycle 420 (2026-09-12)

## 5 New Papers — Pattern Extraction & NeoTrix Mapping

---

### Paper 1: AGAO — Adaptive Goal-aware Attention Orchestration

**Source**: arXiv:2607.23678 (2026)
**Core Idea**: Extends Transformer attention from token-level to workflow-level agent coordination. Three complementary mechanisms:
1. **Goal-aware Attention**: Semantic relevance between user objectives and agent capabilities
2. **Topology-aware Attention**: Graph dependencies + execution structure into attention estimation
3. **Resource-aware Attention**: Converts attention scores into concrete execution decisions (urgency + dependency constraints)

**Pattern Extracted**: Attention as execution-level control mechanism, not just representation-level operation.

**NeoTrix Domain Mapping**:
| Domain | Integration | Specific Module |
|--------|------------|-----------------|
| **NT-CORE** | GWT salience refinement — topology-aware routing over module dependency graph | `gwt_attention` |
| **NT-MIND** | SEAL pipeline — goal-aware stage selection based on task-to-capability relevance scoring | `seal_pipeline` |
| **NT-ACT** | Agent orchestration — resource-aware execution under compute budget constraints | `tool_orchestration` |

**Implementation Insight**: AGAO's formula `P_i = alpha_i * w_i` where w_i = urgency/dependency constraint maps directly to GWT salience weights. Extend salience computation to include graph topology (module dependency DAG) and resource state (fatigue/capacity).

---

### Paper 2: ODAR — Active Inference Routing

**Source**: arXiv:2602.23681 (2026-02)
**Core Idea**: Free-energy-principled routing between Fast Agent (heuristic) and Slow Agent (deliberative). Uses amortized active inference for difficulty estimation + variational free energy for answer fusion (balancing log-likelihood with varentropy).

**Pattern Extracted**: Fast/Slow dual-path routing with information-theoretic answer fusion.

**NeoTrix Domain Mapping**:
| Domain | Integration | Specific Module |
|--------|------------|-----------------|
| **NT-CORE** | E8 hexagram reasoning — Fast path for simple hexagrams, Slow for complex multi-line analysis | `e8_engine` |
| **NT-IO** | Provider routing — Fast path to cheap models, Slow to expensive ones | `llm_router` |
| **NT-FEEL** | Emotion regulation — varentropy as confidence measure for emotion expression tuning | `emotion_engine` |

**Implementation Insight**: ODAR achieves 82% cost reduction on open-source stack (Llama 4 + DeepSeek). The free-energy fusion: `F = -E_q[log p(o|s)] + KL(q||p)` directly applicable to GWT salience — salience = expected utility - epistemic uncertainty.

---

### Paper 3: PackInfer — Batched Attention Kernels

**Source**: arXiv:2602.06072 (2026-02)
**Core Idea**: Compute- and I/O-aware attention for heterogeneous batched inference. Orchestrates requests into load-balanced execution groups, constructs attention kernels directly over packed query-key regions, I/O-aware grouping co-locates shared-prefix requests.

**Pattern Extracted**: Kernel-level attention packing for GPU utilization saturation.

**NeoTrix Domain Mapping**:
| Domain | Integration | Specific Module |
|--------|------------|-----------------|
| **NT-IO** | LLM provider optimization — pack multiple inference requests into unified kernel launches | `provider_pool` |
| **NT-MEMORY** | KV cache optimization — group-contiguous layouts reduce memory fragmentation | `kv_cache_optimizer` |
| **NT-PHYSICAL** | GPU resource management — load-balanced execution groups for multi-device inference | `gpu_scheduler` |

**Implementation Insight**: 13-20% latency reduction, 20% throughput improvement over FlashAttention. The I/O-aware grouping pattern (co-locate shared-prefix requests) maps to KB query caching — similar queries share KV cache pages.

---

### Paper 4: H-MEM — Hierarchical Memory

**Source**: ACL Anthology 2026 (EACL)
**Core Idea**: Multi-level memory organized by semantic abstraction degree. Each higher-level vector has positional index encoding pointing to sub-memories. Index-based routing enables layer-by-layer retrieval without exhaustive similarity search.

**Pattern Extracted**: Hierarchical memory with index-based routing (O(log N) vs O(N) retrieval).

**NeoTrix Domain Mapping**:
| Domain | Integration | Specific Module |
|--------|------------|-----------------|
| **NT-MEMORY** | KB hierarchy — coarse-to-fine retrieval: domain → module → experience | `kb_search` |
| **NT-NEXUS** | Cross-session memory — high-level session summaries point to detailed episode chains | `nexus_weaver` |
| **NT-MIND** | Experience-tree — hub index (L1) points to branch summaries, which point to full experiences | `experience_tree` |

**Implementation Insight**: H-MEM's positional index encoding is isomorphic to experience-tree's route table matching. The hierarchical structure (abstract→concrete) maps to KB namespace hierarchy (domain → module → skill → experience).

---

### Paper 5: Latent Action Reparameterization (LAR)

**Source**: arXiv:2605.18597 (2026-05)
**Core Idea**: Improves LLM-agent efficiency by treating action representation as first-class modeling choice. Reduces effective action horizon (not just token count). Selective action reparameterization preserves semantic correctness while improving inference speed.

**Pattern Extracted**: Action abstraction — compress multi-token actions into latent representations.

**NeoTrix Domain Mapping**:
| Domain | Integration | Specific Module |
|--------|------------|-----------------|
| **NT-ACT** | Tool calling — compress multi-step tool sequences into atomic latent actions | `tool_registry` |
| **NT-MIND** | SEAL pipeline — compress multi-stage evolution into latent phase transitions | `seal_pipeline` |
| **NT-CORE** | E8 reasoning — compress multi-line hexagram analysis into latent state transitions | `e8_engine` |

**Implementation Insight**: LAR shows effective decision horizon (not token count) governs agent efficiency. Maps to NeoTrix's SEAL pipeline stages — instead of explicit Stage 0→1→2 transitions, model as latent state machine with learned transitions.

---

## Cross-Paper Synthesis: 3 Meta-Patterns

### Meta-Pattern 1: Attention Hierarchy
AGAO (token→workflow) + PackInfer (kernel-level) + H-MEM (memory-level) = **Three-level attention**:
- L1: Token attention (standard Transformer)
- L2: Task/workflow attention (AGAO goal-aware)
- L3: Memory/episode attention (H-MEM index-based)

**NeoTrix Action**: Implement 3-level GWT salience — salience = token_relevance * task_alignment * memory_freshness.

### Meta-Pattern 2: Fast-Slow Dual Path
ODAR (Fast/Slow Agent) + LAR (latent actions) = **Efficiency through abstraction**:
- Fast path: heuristic + latent actions (low latency)
- Slow path: deliberative + full actions (high accuracy)
- Fusion: free-energy principle (information-theoretic)

**NeoTrix Action**: Extend GWT with difficulty-aware routing — cheap models for routine tasks, expensive for novel/complex.

### Meta-Pattern 3: Gated Memory Architecture
Gated-Memory Routing + H-MEM = **Write-gate + Read-gate**:
- Write gate: only non-redundant state persists
- Read gate: hierarchical index-based retrieval
- Adaptive halting: stop when memory sufficient

**NeoTrix Action**: KB write filter — deduplicate experiences before persistence. Experience-tree hub index = H-MEM's top-level vectors.

---

## Priority Matrix

| Priority | Paper | Impact | Effort | NeoTrix Value |
|----------|-------|--------|--------|---------------|
| P0 | Gated-Memory Routing | High | Medium | KB write deduplication, experience-tree optimization |
| P0 | R2-Router (from trending) | High | Low | Cost-aware routing (Axiom A1) |
| P1 | ODAR | High | Medium | Fast/Slow GWT routing, provider cost optimization |
| P1 | H-MEM | Medium | Low | Experience-tree hub indexing |
| P2 | AGAO | Medium | High | Topology-aware GWT salience |
| P2 | PackInfer | Low | High | KV cache optimization (deferred — NT-IO focus) |
| P2 | LAR | Medium | Medium | SEAL stage compression |

---

## Sources

1. AGAO: arXiv:2607.23678 (2026)
2. ODAR: arXiv:2602.23681 (2026-02)
3. PackInfer: arXiv:2602.06072 (2026-02)
4. H-MEM: ACL Anthology 2026 (EACL-Long-15)
5. LAR: arXiv:2605.18597 (2026-05)
