# External Intelligence Report: Knowledge Graph Brain-Inspired Systems

**Date**: 2026-09-11
**Agent**: External Research Intelligence Agent
**Scope**: Trending projects, academic papers, algorithms, integration recommendations for NeoTrix

---

## 1. Executive Summary

The landscape of knowledge graph brain-inspired memory systems has undergone a paradigm shift in 2025-2026. Three converging trends dominate:

1. **Temporal Knowledge Graphs** (Graphiti/Zep, RoMem) — bi-temporal fact tracking with invalidation windows, achieving 94.8% on DMR benchmarks
2. **Bio-Inspired Memory Architectures** (BMAM, HeLa-Mem, HippoRAG2) — hippocampal indexing theory applied to agent memory, with Hebbian learning dynamics on memory graphs
3. **Sparse Attention Mechanisms** (NSA, SSA) — hardware-aligned natively trainable sparse attention enabling 100K+ context windows

**Key finding**: NeoTrix's existing architecture (KB + SEAL pipeline + GWT + VSA HyperCube) is remarkably well-positioned. The gap is in **temporal awareness**, **Hebbian edge weighting**, and **memory consolidation sleep cycles**. These map directly to existing NeoTrix components with small-to-medium effort.

---

## 2. Project Catalog

### 2.1 High-Priority Projects (Direct NeoTrix Relevance)

| # | Project | URL | Stars | Key Architecture | License | NeoTrix Mapping |
|---|---------|-----|-------|-----------------|---------|-----------------|
| 1 | **Graphiti** (Zep AI) | github.com/getzep/graphiti | ~15K+ | Bi-temporal KG + Neo4j/FalkorDB + hybrid search (semantic+BM25+graph traversal) | Apache-2.0 | NT-MEMORY temporal layer |
| 2 | **BMAM** | github.com/innovation64/BMAM | 6 | 5 brain-region agents + StoryArc timeline + hybrid retrieval + soul portability | MIT | NT-MEMORY + NT-FEEL memory consolidation |
| 3 | **HeLa-Mem** | github.com/ReinerBRO/HeLa-Mem | ~200 | Hebbian learning dynamics on episodic graph + semantic distillation via reflective agent | Apache-2.0 | NT-MEMORY Hebbian edge weights |
| 4 | **HippoRAG 2** | github.com/OSU-NLP-Group/HippoRAG | ~3K | Hippocampal indexing theory + Personalized PageRank + KG-based associative memory | Apache-2.0 | NT-MEMORY retrieval + GWT attention |
| 5 | **LightRAG** | github.com/HKUDS/LightRAG | ~35K | Dual-level retrieval (low-level entity + high-level topic) + graph-based indexing | MIT | NT-MEMORY + NT-WORLD indexing |
| 6 | **Mem0** | github.com/mem0ai/mem0 | 64.8K | Extract→Compare→ADD/UPDATE/DELETE/NOOP + graph memory variant | Apache-2.0 | NT-MEMORY write path |
| 7 | **Anda Brain** | github.com/ldclabs/anda-brain | 78 | Graph-native memory + sleep-based consolidation (NREM/REM) + type system in graph | Apache-2.0 | NT-MEMORY + NT-FEEL sleep cycle |
| 8 | **Open-Engram** | github.com/Open-Nucleus/open-engram | 1 | 4-store bio architecture (sensory→working→episodic→semantic) + 7-stage consolidation | Apache-2.0 | NT-MEMORY tiered storage |
| 9 | **LatticeDB** | github.com/jeffhajewski/latticedb | trending | Embedded graph DB: vector similarity + full-text + relationship traversal in one query | - | NT-MEMORY embedded graph backend |
| 10 | **Graphify** | github.com/Graphify-Labs/graphify | 19K | Code→knowledge graph conversion for AI coding assistants | - | NT-WORLD code understanding |
| 11 | **CodeGraph** | github.com/codegraph-ai/CodeGraph | ~5K | Semantic code graph with 42 MCP tools, 38 languages, persistent memory | - | NT-ACT code intelligence |
| 12 | **OpenViking** | github.com/volcengine/OpenViking | 35.9K | Self-evolving context DB: memory + knowledge RAG + skills unified | - | NT-MEMORY context management |

### 2.2 Trending Ecosystem Projects (August 2026)

| # | Project | Stars | Relevance |
|---|---------|-------|-----------|
| 1 | PrimeIntellect-ai/prime-agent | 20K | Self-improving RLM agent for autonomous tasks |
| 2 | diegosouzapw/OmniRoute | 59.4K | AI gateway: 352 providers, 1200+ models, quota-aware routing |
| 3 | DeusData/codebase-memory-mcp | 6.3K | Code intelligence as persistent knowledge graph |
| 4 | tirth8205/code-review-graph | trending | Local-first code intelligence graph for MCP |
| 5 | semantica-agi/semantica | 275 | Graph-native infrastructure for accountable AI |
| 6 | TencentCloud/TencentDB-Agent-Memory | trending | Team-level memory hub converting conversations/docs/code to reusable assets |
| 7 | chopratejas/headroom | 15.4K | Compress tool outputs/logs/RAG chunks: 60-95% fewer tokens |
| 8 | obulsa/Agentic-Memory | trending | Implementing cognitive memory architecture |

---

## 3. Paper Catalog

### 3.1 Temporal Knowledge Graph Papers

| # | Paper | Venue | Key Innovation | NeoTrix Relevance |
|---|-------|-------|----------------|-------------------|
| 1 | **Graphiti/Zep** (Rasmussen et al.) | arXiv 2025 | Bi-temporal data model: valid_at + invalid_at for every edge. Hybrid retrieval (semantic+BM25+graph). 94.8% DMR. | P0 — NT-MEMORY temporal layer |
| 2 | **RoMem** (Tencent) | EMNLP 2026 | Continuous phase rotation for temporal KGs and agentic memory | P1 — Temporal representation |
| 3 | **MemoTime** | WWW 2026 | Tree of Time decomposition + self-evolving experience memory for temporal QA | P1 — Temporal reasoning |
| 4 | **MemGraphRAG** | KDD 2026 | Three-layer memory (schema→fact→passage) with conflict-aware construction | P1 — Multi-layer knowledge |

### 3.2 Brain-Inspired Memory Papers

| # | Paper | Venue | Key Innovation | NeoTrix Relevance |
|---|-------|-------|----------------|-------------------|
| 5 | **HippoRAG** (Gutiérrez et al.) | NeurIPS 2024 | Hippocampal indexing theory + KG + Personalized PageRank. +20% over SOTA multi-hop QA. | P0 — Retrieval architecture |
| 6 | **HippoRAG 2** | arXiv 2025 | Non-parametric continual learning via hippocampal indexing | P1 — Continual learning |
| 7 | **BMAM** (Li et al.) | ACL 2026 | 5 brain-region agents + StoryArc timeline. 78.45% LoCoMo. Soul portability 87.5%. | P0 — Memory architecture |
| 8 | **HeLa-Mem** (Zhu et al.) | ACL 2026 | Hebbian learning dynamics on episodic graph + semantic distillation via reflective agent | P0 — Edge weighting |
| 9 | **BrainMem** | arXiv 2026 | Training-free hierarchical memory: working + episodic + semantic for embodied agents | P1 — Embodied memory |
| 10 | **SCM** (Sleep-Consolidated Memory) | arXiv 2026 | Working memory limits + NREM/REM consolidation + intentional forgetting + self-model | P0 — Sleep cycle |
| 11 | **Open-Engram** | arXiv 2026 | 4-store bio architecture with 7-stage consolidation pipeline | P1 — Memory lifecycle |

### 3.3 RAG & Retrieval Papers

| # | Paper | Venue | Key Innovation | NeoTrix Relevance |
|---|-------|-------|----------------|-------------------|
| 12 | **LightRAG** (Guo et al.) | EMNLP 2025 | Dual-level retrieval (entity + topic) + graph-based indexing + incremental update | P0 — KB retrieval |
| 13 | **Mem0** (Chhikara et al.) | arXiv 2025 | Extract→Compare→ADD/UPDATE/DELETE/NOOP pipeline. 26% over OpenAI memory. | P1 — Write path |
| 14 | **Memori** | arXiv 2026 | LLM-agnostic persistent memory: dialogue→semantic triples. 81.95% LoCoMo. | P1 — Memory extraction |
| 15 | **Infini Memory** | arXiv 2026 | Topic-structured documents for maintainable long-term memory | P1 — Memory organization |
| 16 | **Stable-RAG** | ACL 2026 | DPO alignment to mitigate retrieval-permutation hallucinations | P2 — Hallucination mitigation |

### 3.4 Algorithm Foundations Papers

| # | Paper | Venue | Key Innovation | NeoTrix Relevance |
|---|-------|-------|----------------|-------------------|
| 17 | **BambooKG** | arXiv 2025 | Hebbian "fire together, wire together" frequency-weighted KG edges | P0 — Edge weighting formula |
| 18 | **FOREVER** | arXiv 2026 | Ebbinghaus forgetting curve-inspired memory replay for continual learning | P0 — Decay mechanism |
| 19 | **NSA** (Yuan et al.) | ACL 2025 | Natively trainable sparse attention: 3-branch (compressed+selected+sliding). 414 citations. | P1 — Long context |
| 20 | **SSA** | ICML 2026 | Bidirectional alignment of full and sparse attention in feature space | P2 — Attention optimization |
| 21 | **Spreading Activation** (SpreadPy) | arXiv 2025 | Python tool for spreading activation in cognitive multiplex networks | P1 — Retrieval dynamics |
| 22 | **Contradiction Detection** (Kontrast) | arXiv 2026 | Cross-modal knowledge inconsistency detection across text/tables/KGs | P1 — KB consistency |
| 23 | **SparseCL** | ICML 2025 | Contrastive learning with sparsity for contradiction retrieval | P1 — Contradiction detection |

---

## 4. Algorithm Catalog

### 4.1 Personalized PageRank (PPR)

**Core Formula**:
```
π_s(t) = α * Σ_{v→t} π_s(v)/deg(v) + (1-α) * δ_{s,t}
```
Where α = damping factor (typically 0.85), δ is teleport distribution.

**Key Insight**: PPR is the backbone of HippoRAG's "hippocampal index" — it computes associative relevance from a query node through the knowledge graph, mimicking how the hippocampus retrieves memories via partial cues.

**Rust Implementation Reference**: frankmcsherry/pagerank (timely dataflow, distributed PageRank)

**NeoTrix Mapping**: NT-MEMORY retrieval layer. The GWT salience mechanism can use PPR scores as attention weights across the knowledge graph.

**Integration Path**: Add PPR computation to `nt_memory` as a retrieval strategy alongside BM25 + vector search. Existing SQLite graph can serve as the adjacency matrix.

**Priority**: P0 | **Effort**: Medium (~200 lines)

### 4.2 Hebbian Edge Weighting

**Core Formula** (from BambooKG/HeLa-Mem):
```
w_ij(t+1) = w_ij(t) + η * co_activation(i,j) * (1 - w_ij(t)/w_max)
```
Where:
- η = learning rate
- co_activation(i,j) = frequency of co-occurrence in shared contexts
- w_max = maximum weight (prevents saturation)

**Key Insight**: "Neurons that fire together, wire together." Edge weights in the knowledge graph strengthen with repeated co-activation, creating a dynamic associative network.

**NeoTrix Mapping**: NT-MEMORY edge weights. Currently edges in the KB are static. Hebbian weighting would make frequently co-accessed entity pairs more relevant in retrieval.

**Integration Path**: Add weight accumulation to KB edge operations. Each retrieval event updates edge weights between co-retrieved entities.

**Priority**: P0 | **Effort**: Small (~80 lines)

### 4.3 Ebbinghaus Forgetting Curve (Memory Decay)

**Core Formula**:
```
R(t) = e^(-t/S)
```
Where:
- R(t) = retention probability at time t
- S = memory stability (increases with each review)
- t = time since last access/review

**Enhanced Formula** (multi-dimensional):
```
R(t) = e^(-t/S) * novelty_factor * emotional_intensity * access_frequency
```

**Key Insight**: Memories that are not accessed decay rapidly at first, then stabilize. Each review increases stability S, flattening the curve. This prevents catastrophic forgetting while allowing natural cleanup of obsolete information.

**NeoTrix Mapping**: NT-MEMORY node/edge TTL + consolidation. The existing `kv_store` can implement decay scores. The SEAL sleep cycle can trigger "review" of high-stability memories.

**Integration Path**: Add `last_accessed` and `stability` fields to KB nodes/edges. Background process applies decay function. Memories below threshold trigger consolidation or archival.

**Priority**: P0 | **Effort**: Small (~60 lines)

### 4.4 Spreading Activation

**Core Model**:
```
A_j(t+1) = A_j(t) * retention + Σ_{i→j} A_i(t) * w_ij * (1 - retention)
```
Where:
- A_j = activation level of node j
- retention = proportion kept at current node (typically 0.5)
- w_ij = edge weight

**Key Insight**: Activation spreads from a source node through the graph, decaying with distance and edge weights. This is how human semantic memory works — thinking of "dog" activates "cat", "pet", "bark" etc.

**NeoTrix Mapping**: GWT attention routing. Currently GWT uses salience scores. Spreading activation provides a graph-native mechanism for attention propagation.

**Integration Path**: Implement as a retrieval strategy in NT-MEMORY. Query node activates, spreading activation through KB graph, returning activated nodes as context.

**Priority**: P1 | **Effort**: Medium (~150 lines)

### 4.5 Memory Consolidation (Sleep Cycle)

**Core Architecture** (from SCM/Anda Brain/BMAM):
```
Wake Phase:
  - Encode new experiences → Working Memory (limited capacity)
  - Tag with importance/novelty/emotional salience

Sleep Phase (NREM):
  - Replay working memory episodes
  - Strengthen co-occurring concept pairs (Hebbian)
  - Extract "essence" from episodic fragments → semantic knowledge
  - Deduplicate facts

Sleep Phase (REM):
  - Generate novel associations from consolidated knowledge
  - Dream-like recombination for creative insight
```

**Key Insight**: Biological memory consolidation happens during sleep, not during wake. Working memory is limited capacity, forcing prioritization. Important memories get consolidated to long-term storage; unimportant ones decay.

**NeoTrix Mapping**: SEAL pipeline sleep cycle + NT-FEEL emotion tagging. The existing SEAL pipeline already has exploration/distillation stages. Adding explicit NREM/REM phases with Hebbian replay would make it biologically grounded.

**Integration Path**: Extend SEAL pipeline with:
1. Working memory buffer (bounded queue)
2. Importance tagging during encoding
3. NREM consolidation (Hebbian replay + essence extraction)
4. REM dreaming (novel association generation)
5. Decay/cleanup of low-stability memories

**Priority**: P0 | **Effort**: Large (~400 lines)

### 4.6 Contradiction Detection

**Approach** (from Kontrast/SparseCL):
```
1. Extract candidate facts as triples
2. For each new triple, retrieve similar existing triples
3. Classify relationship: ADD / UPDATE / DELETE / NOOP
4. Temporal resolution: newer facts supersede older ones
5. Conflict logging with provenance tracking
```

**Key Insight**: When new information contradicts existing knowledge, the system must decide: is the new fact an update, a correction, or a perspective shift? Graphiti's bi-temporal model handles this elegantly by invalidating (not deleting) old edges.

**NeoTrix Mapping**: NT-MEMORY write path + NT-SHIELD audit. Currently KB writes may overwrite without temporal tracking. Adding contradiction detection prevents silent knowledge corruption.

**Integration Path**: Add to the KB write pipeline:
1. Before INSERT/UPDATE, retrieve K similar existing facts
2. LLM-based contradiction classification
3. If contradiction: invalidate old edge (set invalid_at), insert new edge
4. Log all conflict resolution events

**Priority**: P1 | **Effort**: Medium (~200 lines)

---

## 5. Integration Roadmap

### Phase 1: Core Memory Enhancements (P0, 2-4 weeks)

| # | Integration | Source | Target | Effort | Impact |
|---|------------|--------|--------|--------|--------|
| 1 | **Hebbian Edge Weighting** | BambooKG, HeLa-Mem | NT-MEMORY KB edges | Small | Dynamic associative retrieval |
| 2 | **Ebbinghaus Decay** | FOREVER, Ebbinghaus-LLM | NT-MEMORY node/edge TTL | Small | Natural memory cleanup |
| 3 | **Personalized PageRank** | HippoRAG | NT-MEMORY retrieval | Medium | Associative multi-hop retrieval |
| 4 | **Bi-temporal Model** | Graphiti/Zep | NT-MEMORY edges | Medium | Temporal fact tracking |

### Phase 2: Memory Lifecycle (P0, 4-6 weeks)

| # | Integration | Source | Target | Effort | Impact |
|---|------------|--------|--------|--------|--------|
| 5 | **Sleep Consolidation** | SCM, Anda Brain | SEAL pipeline | Large | Working→long-term memory transfer |
| 6 | **Importance Tagging** | BMAM, Mem0 | NT-FEEL + NT-MEMORY | Medium | Salience-aware memory encoding |
| 7 | **Contradiction Detection** | Graphiti, Kontrast | NT-MEMORY write path | Medium | Knowledge consistency |

### Phase 3: Retrieval Optimization (P1, 2-4 weeks)

| # | Integration | Source | Target | Effort | Impact |
|---|------------|--------|--------|--------|--------|
| 8 | **Spreading Activation** | SpreadPy, HeLa-Mem | GWT attention routing | Medium | Graph-native attention |
| 9 | **Dual-level Retrieval** | LightRAG | NT-MEMORY query | Medium | Entity + topic retrieval |
| 10 | **Experience Memory** | MemoTime | NT-NEXUS | Medium | Reasoning trace reuse |

### Phase 4: Advanced (P2, ongoing)

| # | Integration | Source | Target | Effort | Impact |
|---|------------|--------|--------|--------|--------|
| 11 | **NSA Sparse Attention** | DeepSeek NSA | NT-CORE inference | Large | Long context efficiency |
| 12 | **Soul Portability** | BMAM | NT-MEMORY export | Medium | Cross-session memory transfer |
| 13 | **Dream Phase** | SCM REM | NT-MIND SEAL | Large | Creative association generation |

---

## 6. Architecture Recommendations

### 6.1 Immediate: Temporal KB Layer

The single highest-impact integration is adding bi-temporal tracking to the NeoTrix KB. Graphiti's model is the gold standard:

```
Edge {
    source_id: EntityId,
    relation: String,
    target_id: EntityId,
    valid_at: Timestamp,     // When this became true
    invalid_at: Option<Timestamp>,  // When this was superseded
    episode_id: EpisodeId,   // Provenance
    confidence: f64,         // Extraction confidence
    weight: f64,             // Hebbian weight (starts at 1.0)
}
```

This replaces the current static edge model and enables:
- Temporal queries ("what was true at time T?")
- Natural contradiction handling (invalidate, don't delete)
- Hebbian weight evolution
- Provenance tracking

### 6.2 Near-term: Hebbian Retrieval Weight

Add a `hebbian_weight` field to all KB edges, updated on every co-retrieval:

```rust
fn update_hebbian(edge: &mut Edge, retrieval_count: f64) {
    let delta = LEARNING_RATE * (1.0 - edge.hebbian_weight / MAX_WEIGHT);
    edge.hebbian_weight += delta * retrieval_count;
}
```

This is a 60-line change to the KB write path that creates a self-organizing retrieval system — frequently co-accessed entities become more relevant.

### 6.3 Medium-term: SEAL Sleep Cycle

Extend the SEAL pipeline with explicit consolidation phases:

```
SEAL Phase 5 (NEW): Sleep Consolidation
├── NREM Consolidation
│   ├── Replay working memory episodes
│   ├── Hebbian edge strengthening
│   ├── Fact essence extraction → semantic knowledge
│   └── Deduplication of redundant facts
├── REM Dreaming (Optional)
│   ├── Novel association generation
│   ├── Cross-domain knowledge synthesis
│   └── Creative hypothesis formation
└── Decay Cleanup
    ├── Apply Ebbinghaus decay to all nodes/edges
    ├── Archive memories below stability threshold
    └── Log consolidation metrics
```

### 6.4 Key Algorithm Integrations

| Algorithm | Formula | Where | Why |
|-----------|---------|-------|-----|
| PPR | `π(t+1) = α * M * π(t) + (1-α) * e_s` | NT-MEMORY retrieval | Associative multi-hop retrieval in 1 step |
| Hebbian | `w += η * co_act * (1 - w/w_max)` | NT-MEMORY edges | Self-organizing relevance |
| Ebbinghaus | `R = e^(-t/S) * novelty * emotion * freq` | NT-MEMORY TTL | Natural memory lifecycle |
| Spreading Activation | `A_j = A_j*r + Σ A_i*w_ij*(1-r)` | GWT attention | Graph-native attention routing |
| Consolidation | Working→NREM→REM→Long-term | SEAL pipeline | Biological memory lifecycle |

---

## 7. Competitive Landscape

| System | Memory Model | Temporal | Hebbian | Consolidation | Language |
|--------|-------------|----------|---------|---------------|----------|
| **NeoTrix** (current) | SQLite KB + embeddings | No | No | SEAL (partial) | Rust |
| **Graphiti/Zep** | Neo4j + bi-temporal | Yes | No | No | Python |
| **BMAM** | 5 brain-region agents | Yes (StoryArc) | No | Yes (6-phase) | Python |
| **HeLa-Mem** | Hebbian graph | No | Yes | Yes (distillation) | Python |
| **Mem0** | Vector + graph | Partial | No | No | Python |
| **Open-Engram** | 4-store bio | No | No | Yes (7-stage) | TypeScript |
| **Anda Brain** | Graph + sleep | Yes | No | Yes (NREM/REM) | Rust |

**NeoTrix advantage**: Rust performance, E8 deterministic reasoning, existing SEAL pipeline, VSA HyperCube for symbolic representation.

**Gap**: No temporal tracking, no Hebbian weighting, no explicit sleep consolidation. All three are addressable with <1000 lines total.

---

## 8. Risk Assessment

| Risk | Severity | Mitigation |
|------|----------|------------|
| Hebbian weights create retrieval bias | Medium | Cap maximum weight, periodic normalization |
| Temporal tracking adds storage overhead | Low | SQLite indexes on valid_at/invalid_at are efficient |
| Sleep consolidation blocks main thread | Medium | Background task, configurable schedule |
| Ebbinghaus decay removes useful memories | Low | Stability threshold tuning, review before archival |
| Contradiction detection false positives | Medium | Confidence threshold, human review for high-stakes |

---

## 9. Next Steps

1. **Immediate**: Implement Hebbian edge weighting in NT-MEMORY (60 lines, P0)
2. **Week 1**: Add bi-temporal model to KB edges (200 lines, P0)
3. **Week 2**: Implement PPR retrieval strategy (200 lines, P0)
4. **Week 3-4**: Ebbinghaus decay + importance tagging (100 lines, P0)
5. **Month 2**: SEAL sleep consolidation phase (400 lines, P0)
6. **Month 3**: Spreading activation + contradiction detection (400 lines, P1)

**Total estimated effort**: ~1,360 lines of Rust for P0 integrations.

---

*Report generated 2026-09-11 by External Research Intelligence Agent*
*Sources: 12 web searches, 23+ papers, 12+ GitHub projects analyzed*
