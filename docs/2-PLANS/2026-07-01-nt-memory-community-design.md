# Hierarchical Community Retrieval for nt_memory_kb

**Blind Spot**: No hierarchical community detection or multi-granular retrieval
**Source**: Microsoft GraphRAG (34k★), LightRAG (37k★), ArchRAG
**Target**: nt_memory_kb (L3 Memory — currently flat entity graph only)
**Priority**: P1
**Estimated Effort**: 9 days (4 phase)

---

## 1. Problem Statement

The existing KB entity graph (`nt_memory_kb`) has a flat topology. `nt_memory_graph.rs` implements a basic **Louvain** community detection (binary weights, no refinement phase, single-level). This means:

- **No hierarchy**: Every query is either entity-level or full-KB scan. No topic-cluster-level retrieval.
- **No community summaries**: Entities must be fetched individually; no pre-computed thematic abstractions.
- **Global queries are O(n)**: Answering "What are the main themes?" requires traversing the entire graph.
- **LLM cost is unbounded**: Every multi-entity reasoning pass must fetch raw nodes rather than distilled summaries.

Microsoft GraphRAG (Edge et al., 2024) showed that hierarchical community detection + summary generation reduces global query latency by **10-50×** while improving answer comprehensiveness by **30%+** over flat vector RAG on the head-to-head benchmark.

---

## 2. Architecture

### 2.1 Three-Layer Retrieval Stack

```
                    ┌──────────────────────────┐
                    │   CommunityAwareKB       │
                    │   (facade: search, query) │
                    └────┬──────┬──────┬───────┘
                         │      │      │
              ┌──────────┘      │      └──────────┐
              ▼                 ▼                 ▼
     ┌──────────────┐  ┌──────────────┐  ┌──────────────┐
     │ Community    │  │ Community    │  │ Entity-Level │
     │ Detector    │  │ Store/Index  │  │ Graph        │
     │ (Leiden)    │  │ (SQLite + LRU)│  │ (existing)   │
     └──────────────┘  └──────────────┘  └──────────────┘
```

### 2.2 Key Data Structures

```rust
// ─── Leiden Community Detection ──────────────────────────────────

pub struct CommunityDetector {
    /// Leiden resolution parameter (default 1.0).
    /// Higher values → more/finer communities.
    pub resolution: f64,
    /// Max nodes per community before recursive split.
    /// Communities exceeding this are re-clustered at next level.
    pub max_cluster_size: usize,
    /// Min nodes to form a community.
    /// Smaller groups are assigned to parent.
    pub min_cluster_size: usize,
    /// Random seed for deterministic output.
    pub seed: u64,
    /// Use largest connected component only.
    pub use_lcc: bool,
}

impl CommunityDetector {
    /// Run hierarchical Leiden algorithm on the KB entity graph.
    /// Returns a CommunityHierarchy with N levels (0..L-1).
    pub fn detect(&self, conn: &Connection) -> Result<CommunityHierarchy, String>;
}

/// Multi-level community hierarchy.
/// levels[0] = leaf (finest granularity, 5-20 entities each)
/// levels[L-1] = root (entire KB as one community)
pub struct CommunityHierarchy {
    /// Hierarchy levels. levels[l][i] = i-th community at level l.
    pub levels: Vec<Vec<Community>>,
    /// Entity ID → all communities it belongs to (one per level).
    pub entity_to_communities: HashMap<String, Vec<CommunityId>>,
    /// Community ID → parent community ID (None for root-level).
    pub parent_map: HashMap<CommunityId, Option<CommunityId>>,
}

/// A single community at a specific hierarchy level.
pub struct Community {
    pub id: CommunityId,
    pub level: usize,
    pub members: Vec<String>,
    pub size: usize,
    pub internal_edge_count: usize,
    pub cohesion: f64,           // internal / max_possible_edges
    pub parent: Option<CommunityId>,
    pub children: Vec<CommunityId>,
    pub summary: Option<String>,   // LLM-generated, lazy-computed
    pub summary_embedding: Option<Vec<f32>>,
    pub dominant_types: Vec<(NodeType, usize)>,
    pub created_at: i64,
    pub updated_at: i64,
}

pub type CommunityId = String;   // Format: "level_{l}_comm_{idx}"
```

### 2.3 Query Modes (LightRAG + GraphRAG fusion)

```rust
/// Query precision modes, inspired by LightRAG dual-level + GraphRAG global/local
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CommunityQueryMode {
    /// Use community summaries ONLY. Best for thematic/global questions.
    /// "What are the main research areas in this KB?"
    Global,
    /// Use entity-level ONLY. Best for specific factual questions.
    /// "What is the publication date of paper X?"
    Local,
    /// Hybrid: 0.5 × community + 0.5 × entity fusion.
    /// "How does algorithm A relate to the broader field of B?"
    Hybrid,
    /// ALL hierarchy levels, weighted by level depth.
    /// For exploratory queries where granularity is unknown.
    Mix,
    /// Use C-HNSW-style hierarchical index for ANN search over communities.
    /// Fastest for vector-similarity-first queries.
    Hnsw,
}

impl CommunityAwareKB {
    /// Primary search entry point.
    pub fn search_community(
        &self,
        query: &str,
        mode: CommunityQueryMode,
        k: usize,
    ) -> Result<Vec<CommunityResult>>;

    /// Fused search: community + entity results merged and reranked.
    pub fn search_fused(
        &self,
        query: &str,
        k: usize,
    ) -> Result<Vec<CommunityResult>>;
}

pub struct CommunityResult {
    pub community_id: String,
    pub level: usize,
    pub summary: String,
    pub score: f64,             // relevance score 0-1
    pub matched_entities: Vec<String>,
    pub source_type: ResultSource,
}

pub enum ResultSource {
    CommunitySummary(Vec<f32>),   // embedding match
    EntityExpansion,              // entity-in-community match
    DirectEntity(String),         // single entity match
}
```

---

## 3. Hierarchical Leiden Algorithm

### 3.1 Algorithm Summary

Based on Traag, Waltman & van Eck (2019) "From Louvain to Leiden: guaranteed well-connected communities" — the current Louvain implementation (`knowledge_graph.rs:482`) is **missing**:

1. **Refinement phase**: Louvain may merge disconnected communities. Leiden fixes this with a random local refinement + fast local move.
2. **Hierarchical recursion**: Louvain runs once. Leiden recursively aggregates communities and re-clusters.
3. **Resolution parameter**: Louvain has no resolution control. Leiden uses the Constant Potts Model (CPM) for tunable granularity.

### 3.2 Pseudo-code

```
fn hierarchical_leiden(graph, resolution, max_cluster_size, seed):
    level = 0
    hierarchy = []
    current_graph = graph

    while current_graph.num_nodes > 1:
        // Step 1: Local moving
        community_of = current_graph.each_node → own_community
        repeat until no improvement:
            for each node i in random order:
                // Move i to neighbor community that maximizes CPM quality
                best_community = argmax(CPM_gain(i → community))
                if gain > 0: move i

        // Step 2: Refinement (Leiden's key innovation)
        refined = community_of.copy()
        for each community c:
            internal_nodes = nodes_in(c, community_of)
            if is_well_connected(c, internal_nodes):
                continue  // already well-connected
            // Refine: split disconnected subsets
            refine_subset(internal_nodes, current_graph, refined)

        // Step 3: Aggregation
        super_graph = collapse(community_of, refined, current_graph)
        hierarchy[current_level] = extract_communities(community_of)

        // Handle large communities
        for each community c in hierarchy[current_level]:
            if c.size > max_cluster_size:
                sub_hierarchy = hierarchical_leiden(
                    subgraph(c), resolution, max_cluster_size, seed + level
                )
                hierarchy.append(sub_hierarchy with level_offset)

        current_graph = super_graph
        level += 1

    return hierarchy

fn CPM_gain(i, community, resolution):
    // Constant Potts Model modularity
    e_i_c = sum(weights from i to community)
    p_i_c = resolution * degree(i) * size(community)
    return (e_i_c - p_i_c) / total_weight

fn refine_subset(nodes, graph, community_of):
    // Split node subset into well-connected sub-communities
    subgraph = induced_subgraph(graph, nodes)
    communities = leiden_once(subgraph, resolution=low)
    for each refined_community in communities:
        if size > min_cluster_size:
            assign_to_refined(nodes, refined_community, community_of)
```

### 3.3 Implementation Mapping

| Leiden Step | `knowledge_graph.rs` status | Changes needed |
|-------------|----------------------------|----------------|
| CPM modularity (vs binary Louvain) | Uses `gain = (k_i_in - sigma_tot × k_i / 2m) / m` | Replace with `gain = (e_i_c - resolution × deg(i) × size(c)) / total_weight` |
| Refinement phase | Missing entirely | New function `refine_communities()` |
| Hierarchical recursion | `louvain_communities()` returns flat mapping | New function `hierarchical_leiden()` with multi-level return |
| Resolution parameter | Hardcoded (binary weight) | New `resolution: f64` field |
| Largest connected component | Not used | Add `stable_lcc()` filter from graspologic |
| Weighted edges | Supported (reads `weight`) | Already works |
| Deterministic seed | Not supported | Add `seed: u64` for reproducible output |

### 3.4 Key Constants

| Parameter | Default | Range | Effect |
|-----------|---------|-------|--------|
| `resolution` | 1.0 | 0.5 - 2.0 | Lower → fewer large communities. Higher → many small communities. |
| `max_cluster_size` | 20 | 10 - 50 | Max entities per leaf community. Dictates hierarchy depth. |
| `min_cluster_size` | 3 | 2 - 10 | Min entities to form a community. |
| `seed` | 42 | any u64 | Reproducibility for deterministic pipelines. |

---

## 4. Community Summary Generation

### 4.1 LLM Prompt Template

```
SYSTEM: You are a knowledge graph community analyst. Your task is to
generate a concise, information-dense summary of a community of
entities.

COMMUNITY STATISTICS:
- Total entities: {N}
- Total relationships: {R} ({R_internal} internal, {R_external} external)
- Cohesion score: {cohesion:.2}
- Dominant types: {types_text}

ENTITIES:
{entities_text}

KEY RELATIONSHIPS:
{relationships_text}

Generate a structured summary covering:
1. OVERVIEW: 2-3 sentences on the community's domain/thematic focus.
2. KEY ENTITIES: Top-5 most central entities with 1-sentence descriptions.
3. KEY RELATIONSHIPS: Top-3 most important connections.
4. NOTABLE CLAIMS: 1-2 facts or insights that emerge from this cluster.
5. CONFIDENCE: Overall confidence score 0.0-1.0 based on entity density
   and relationship completeness.

Respond in JSON format:
{
  "overview": "...",
  "key_entities": [{"id": "...", "name": "...", "description": "..."}],
  "key_relationships": [{"source": "...", "target": "...", "type": "...", "description": "..."}],
  "notable_claims": ["..."],
  "confidence": 0.0-1.0
}
```

### 4.2 Caching Strategy

| Strategy | Level | Cost | Refresh |
|----------|-------|------|---------|
| Eager (bulk on detect) | All levels | N LLM calls (one per community) | On re-detect |
| Lazy (on first query) | Unqueried communities | 0 until first access | On miss + TTL |
| Hybrid (max_cache_size LRU) | Hot communities | Max K cached at any time | On miss, evict cold |

**Recommendation**: Hybrid with max_cache_size=1000, lazy generation, TTL=1 hour. Community summaries are stored as JSON in a new SQLite table `community_summaries` with `(community_id, level, summary_json, embedding BLOB, created_at, accessed_at)`.

### 4.3 Embedding for Community Retrieval

Each community summary is embedded (same embedding model as KB entities) for vector-similarity search at query time.

```
community_summaries table:
  community_id TEXT PRIMARY KEY,
  level INTEGER,
  summary_json TEXT,
  embedding BLOB,         -- f32 vec bytes
  embedding_model TEXT,
  created_at INTEGER,
  accessed_at INTEGER,
  access_count INTEGER DEFAULT 0
```

---

## 5. Multi-Level Query Routing

### 5.1 Route Selection Logic

```
fn select_query_level(query: &str, mode: CommunityQueryMode, k: usize):
    match mode:
        Global:
            // Embed query, search community summary embeddings
            // Score = cosine(query_embedding, summary_embedding)
            // Return top-k community summaries
            return community_level_only(query, k)

        Local:
            // Existing FTS5 + BM25 + entity embedding search
            // Return entity-level results
            return entity_level_only(query, k)

        Hybrid:
            // Community results (k communities) + entity results (k entities)
            // Fusion: 0.5 × community_score + 0.5 × entity_score
            // Combine, deduplicate, rerank, return top-k
            community_results = community_level_only(query, k)
            entity_results = entity_level_only(query, k * 2)
            return weighted_fusion(community_results, entity_results, alpha=0.5)

        Mix:
            // ALL levels: for level l in 0..L:
            //   search communities at level l
            //   weight = 1.0 / (l + 1)  (higher levels = higher weight = more abstract)
            //   combine all, dedup, rerank
            return all_level_search(query, k)

        Hnsw:
            // ANN search over C-HNSW-like hierarchical index
            // Each level's community summary embeddings indexed in HNSW
            // Search from top level down, refine at each level
            return hnsw_level_search(query, k)
```

### 5.2 Fusion Algorithm (Hybrid mode)

```
fn weighted_fusion(
    community_results: Vec<CommunityResult>,
    entity_results: Vec<CommunityResult>,
    alpha: f64
) -> Vec<CommunityResult>:
    // Normalize both sets to 0-1
    community_results = normalize_scores(community_results)
    entity_results = normalize_scores(entity_results)

    // Interleave: community_results[i] × alpha + entity_results[i] × (1-alpha)
    // Dedup: if an entity belongs to a community already returned, increase
    //   that community's score by entity_score × (1-alpha) × boost
    return fused_reranked
```

### 5.3 Token Cost Analysis

| Mode | Avg tokens/query | Latency | Best for |
|------|------------------|---------|----------|
| Global | ~200 (summary only) | ~50ms | Thematic questions |
| Local | ~500 (entity + context) | ~100ms | Factual questions |
| Hybrid | ~700 (summary + entity) | ~150ms | Mixed questions |
| Mix | ~1000+ (multi-level) | ~200ms | Exploratory topics |
| Hnsw | ~300 (embedding + 1 summary) | ~80ms | Vector-first search |

---

## 6. Integration Points

### 6.1 Module Map

```
neotrix-core/src/neotrix/l3_memory_impl/
├── nt_memory_kb/
│   ├── nt_memory_types.rs       ← NEW: Community, CommunityId, CommunityHierarchy,
│   │                                CommunityQueryMode, CommunityResult, ResultSource
│   ├── nt_memory_graph.rs       ← MODIFY: add hierarchical_leiden(), refine, CPM
│   │                                Keep existing louvain for backward compat
│   ├── nt_memory_community.rs   ← NEW file: CommunityDetector, CommunityAwareKB,
│   │                                query routing, fusion, caching
│   ├── nt_memory_search.rs      ← MODIFY: add search_community(), search_fused()
│   ├── nt_memory_store.rs       ← MODIFY: add community_summaries table CRUD
│   └── mod.rs                   ← MODIFY: add pub mod nt_memory_community, re-exports
```

### 6.2 Dependency Flow

```
CommunityAwareKB
  ├── CommunityDetector (nt_memory_community.rs)
  │     └── hierarchical_leiden() (nt_memory_graph.rs)
  │           └── nt_memory_store.get_edges_for_node()
  ├── CommunitySummaryStore
  │     ├── nt_memory_store (SQLite CRUD)
  │     └── nt_memory_embed (summary embedding)
  ├── search_community()
  │     ├── nt_memory_search (FTS5 + BM25 for entity fallback)
  │     └── CommunitySummaryStore.query_by_embedding()
  └── GWT events
        └── nt_core_gwt (community detection completed → broadcast)
```

### 6.3 GWT Integration Events

```rust
pub enum CommunityEvent {
    CommunitiesUpdated {
        level_count: usize,
        total_communities: usize,
        elapsed_ms: u64,
    },
    SummaryGenerated {
        community_id: CommunityId,
        level: usize,
        confidence: f64,
    },
    CommunityQuery {
        mode: CommunityQueryMode,
        latency_ms: u64,
        results_count: usize,
    },
}
```

---

## 7. Implementation Plan

### Phase 1: Community Data Structures + Leiden Algorithm (3 days)

**Day 1**: Types + Schema
- Add `Community`, `CommunityId`, `CommunityHierarchy` to `nt_memory_types.rs`
- Add `community_summaries` SQLite table creation in `nt_memory_store.rs`
- Add `store_community_summary()`, `get_community_summary()`, `delete_community_summary()`

**Day 2**: Hierarchical Leiden
- Create `nt_memory_community.rs` with `CommunityDetector` struct
- Implement `hierarchical_leiden()` function:
  - Step 1: Local moving with CPM modularity
  - Step 2: Refinement phase (key difference from Louvain)
  - Step 3: Aggregation + recursion
- Add `resolution`, `max_cluster_size`, `min_cluster_size`, `seed` params
- Add `stable_lcc()`: filter to largest connected component

**Day 3**: Tests
- Unit tests for each Leiden step on synthetic graphs
- Integration test: detect on KB → verify hierarchy structure
- Benchmark: 1000-node graph, compare Louvain vs Leiden runtime + quality

### Phase 2: Community Summary Generation (2 days)

**Day 4**: LLM Integration
- Build community → prompt builder
- Extract entities + relationships for community context
- Parse LLM JSON response
- Handle partial failures (retry 2×, use template fallback)

**Day 5**: Caching + Embedding
- Hybrid cache (LRU + TTL)
- Embed summaries on store
- Add `query_community_summaries_by_embedding()`

### Phase 3: Multi-Level Query Routing (2 days)

**Day 6**: Query Modes
- Implement `CommunityAwareKB::search_community()` with all 5 modes
- Implement `weighted_fusion()` for Hybrid
- Implement `all_level_search()` for Mix

**Day 7**: Integration
- Wire into `nt_memory_search::search_fused()`
- Add GWT event emission
- Unit tests for each query mode

### Phase 4: Integration with KB Search API (2 days)

**Day 8**: Frontend Integration
- Expose `CommunityQueryMode` in search API
- Add CLI flag `--community-mode global|local|hybrid|mix|hnsw`
- Wire into REPL if applicable

**Day 9**: System Tests
- End-to-end: ingest → detect → summarize → query at all levels
- Token cost measurement for each mode
- Latency benchmarks

---

## 8. Risks & Mitigations

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Leiden on 100K+ node graph exceeds memory | High | Low | Use `use_lcc=true`; stream processing with `ChunkedCommunityDetector` |
| LLM summary generation cost for large graphs ($0.50/community on GPT-4) | Medium | Medium | Lazy generation; use GPT-4o-mini for summaries; max 2000 communities |
| Community drift (graph changes between detections) | Medium | High | Incremental Leiden update; mark changed subgraphs; partial re-cluster |
| CPM resolution parameter tuning | Low | Medium | Auto-tuning via silhouette score; expose in config |
| Embedding dimension mismatch with entities | Low | Low | Use same `nt_memory_embed` service; verify dimension at load time |

---

## 9. Success Criteria

1. **Detection quality**: Leiden communities have higher internal cohesion (σ within / σ between > 2.0) than existing Louvain (currently ~1.5)
2. **Query latency**: Global mode < 100ms for 10K-node KB (vs entity-level traversal > 500ms)
3. **Token savings**: Hybrid mode uses < 700 tokens per query (vs entity-only > 2000 tokens for same coverage)
4. **Answer quality**: Global query comprehensiveness score > 0.8 on a held-out set of thematic questions
5. **Determinism**: Same graph + same seed → identical hierarchy (verified in CI)

---

## 10. References

- Traag, V. A., Waltman, L., & van Eck, N. J. (2019). "From Louvain to Leiden: guaranteeing well-connected communities." *Scientific Reports*, 9(1), 5233. [arXiv:1810.08473](https://arxiv.org/abs/1810.08473)
- Edge, D., et al. (2024). "GraphRAG: Unlocking LLM Discovery on Narrative Private Data." *Microsoft Research*. [arxiv:2404.16130](https://arxiv.org/abs/2404.16130)
- Guo, Z. et al. (2024). "LightRAG: Simple and Fast Retrieval-Augmented Generation." [arxiv:2410.05779](https://arxiv.org/abs/2410.05779)
- Huang, S. et al. (2025). "ArchRAG: Attributed Community-based Hierarchical Retrieval-Augmented Generation." [arxiv:2502.09891](https://arxiv.org/abs/2502.09891)
