# NeoTrix KB Brain Architecture — Restructuring Plan

**Date**: 2026-09-11
**Scope**: `nt_memory_kb/` module (78+ submodules, ~30K lines)
**Audit Agent**: KB Brain Restructuring

---

## 1. Architecture Diagram

```
nt_memory_kb/mod.rs (KnowledgeBase — god struct, 28 RwLock fields)
│
├── Storage Layer ──────────────────────────────────────────────
│   nt_memory_store.rs        CRUD + crawl queue + procedural memory
│   nt_memory_types.rs        Re-export from nt_core_kb_types (D3下沉)
│   nt_memory_schema.rs       DDL / migration
│   nt_memory_unify.rs        kv_store abstraction
│   knowledge_storage.rs      JSON migration helper
│
├── Search Layer ───────────────────────────────────────────────
│   nt_memory_search.rs       FTS5 + hybrid + Walsh + RRF fusion
│   bm25.rs                   In-memory BM25 index
│   kb_vector_index.rs        HNSW ANN (instant-distance crate)
│   nt_memory_embed.rs        Embedding (local hash-kernel + HTTP)
│   vector_adapter.rs         KB→VectorStore adapter
│
├── Graph Layer ────────────────────────────────────────────────
│   nt_memory_graph.rs        BFS shortest_path / subgraph / community
│   nt_memory_graph_cache.rs  In-memory adjacency cache + Dijkstra
│   nt_memory_graphrag/       GraphRAG (entity extraction + LightRAG)
│   nt_memory_community.rs    Hierarchical Leiden community detection
│
├── Brain Mechanisms ───────────────────────────────────────────
│   nt_memory_brain.rs        SynapticPlasticity / ForgettingCurve / PPR
│   nt_memory_confidence.rs   Epistemic confidence + decay + contradiction
│   nt_memory_sweep_20260815/ FreshnessLedger (A1 recall)
│   nt_temporal_audit.rs      Temporal fact ledger
│
├── Retrieval Intelligence ────────────────────────────────────
│   nt_memory_adaptive_rag.rs Adaptive retrieval routing
│   nt_memory_gwt_router.rs   GWT attention-gated routing
│   nt_memory_e8_agent.rs     E8 phase agent loop
│   nt_memory_vsa_expand.rs   VSA associative expansion
│   nt_memory_decompose.rs    Query decomposition
│   nt_memory_diversity.rs    Result diversity
│   nt_memory_gwtq.rs         GWT queue
│   nt_memory_distill.rs      DistilVDR student model
│
├── Ingestion Pipeline ────────────────────────────────────────
│   nt_memory_crawl.rs        Crawl cycle
│   nt_memory_ingest.rs       Unified ingestion bus
│   nt_memory_pipeline.rs     Ingestion pipeline stages
│   nt_memory_resource_ingest.rs
│   nt_memory_zim_absorber.rs ZIM file absorption
│   nt_memory_seed.rs         Seed data
│
├── Quality Gates ─────────────────────────────────────────────
│   nt_memory_svaf_gate.rs    SVAF write gate
│   nt_memory_write_guard.rs  Write guard verdict
│   nt_memory_curation.rs     Conflict detection + supersede
│   nt_memory_confidence.rs   Epistemic scoring
│   nt_memory_commitment.rs   Embedding commitment store
│
├── Cross-Session ─────────────────────────────────────────────
│   nt_memory_agent_driven.rs Agent memory (tiered)
│   nt_memory_agent_session.rs Agent session manager
│   nt_memory_proficiency.rs  Memory proficiency tracking
│   nt_memory_feedback.rs     Feedback signals
│
├── Meta ──────────────────────────────────────────────────────
│   nt_memory_hierarchical.rs
│   nt_memory_panorama.rs
│   nt_memory_snapshot.rs     KB diff snapshots
│   nt_memory_knowledge_assets.rs  Skills library
│   nt_memory_tech_reserve.rs Tech reserve indexing
│   nt_absorb_mapper.rs       Absorption mapping
│   nt_memory_geo.rs          Geographic
│   nt_memory_cortex_sync.rs  Cortex sync
│   nt_field_ledger.rs        Field ledger
│   nt_memory_weave.rs        Cross-session weaving
│   nt_memory_visibility.rs   Visibility scoring
│   nt_memory_provenance.rs   PROV-O decision provenance
│   nt_memory_wiki.rs         Wiki sync
│   nt_memory_commit_tracker.rs
│   nt_memory_coeffect.rs     Co-effect tracking
│   nt_memory_galaxy_hygiene.rs Galaxy hygiene
│   nt_memory_primitives.rs   Memory primitives
│   nt_memory_integration.rs  Integration layer
│   nt_memory_domain_adapter.rs
│   nt_memory_setting_consistency.rs  (commented out)
│   nt_normalizer.rs          Text normalization
│   nt_memory_pack.rs / nt_memory_pack_chunked.rs  Packing
│
└── Privacy ───────────────────────────────────────────────────
    privacy.rs                Data sovereignty + encryption
```

---

## 2. Redundancy List

### R1: Duplicate Path-Finding — BFS vs Dijkstra

| File | Function | Algorithm |
|------|----------|-----------|
| `nt_memory_graph.rs:7` | `shortest_path()` | BFS (unweighted) |
| `nt_memory_graph_cache.rs:82` | `weighted_shortest_path()` | Dijkstra (weighted) |

**Issue**: `shortest_path()` ignores edge weights (pure BFS), while `weighted_shortest_path()` uses Dijkstra. The BFS version is semantically incorrect for a weighted graph. Both exist and callers may use the wrong one.

**Merge**: Delete `nt_memory_graph::shortest_path()`. Redirect all callers to `weighted_shortest_path(&GraphCache, ...)`. The `GraphCache` already provides the in-memory adjacency needed for Dijkstra.

### R2: Duplicate Community Detection — Leiden vs Label Propagation

| File | Algorithm | Trigger |
|------|-----------|---------|
| `nt_memory_community.rs` | Hierarchical Leiden (CPM) | `kb.detect_communities()` |
| `nt_memory_graphrag/mod.rs:854` | Label Propagation | `GraphRagStore::community_summary()` |

**Issue**: Two completely independent community detection algorithms produce two different community structures for the same graph. GraphRAG's label propagation is used for its own `community_query()`, while Leiden is used for `CommunityAwareSearch`. They don't share results.

**Merge**: Unify on Leiden (higher quality, hierarchical). GraphRAG should consume `CommunityHierarchy` from `CommunityAwareSearch` instead of running its own label propagation. GraphRAG's `community_summary()` becomes a thin wrapper around Leiden results.

### R3: Duplicate Entity/Relation Types — GraphRAG vs KB Core

| File | Types |
|------|-------|
| `nt_memory_graphrag/mod.rs` | `EntityNode`, `RelationEdge` (GraphRAG-specific) |
| `nt_memory_types.rs` / `nt_core_kb_types` | `KnowledgeNode`, `KnowledgeEdge` (KB core) |

**Issue**: GraphRAG defines its own `EntityNode`/`RelationEdge` types with different fields (`source_entity`/`target_entity` vs `source_id`/`target_id`, different metadata). This creates a dual-graph problem: KB has one graph in `nodes`/`edges` tables, GraphRAG has another in `EntityGraph`.

**Merge**: GraphRAG should use `KnowledgeNode`/`KnowledgeEdge` as its entity/relation types, or define a thin wrapper that maps 1:1. The `EntityGraph` should be a view over the KB graph, not a parallel structure.

### R4: Duplicate Contradiction Detection

| File | Approach |
|------|----------|
| `nt_memory_brain.rs:392` | `ContradictionDetector` — embedding cosine + metadata assertions |
| `nt_memory_confidence.rs:279` | `detect_simple_contradiction()` — text pattern matching |

**Issue**: Two contradiction detectors with completely different approaches. `ContradictionDetector` requires embeddings and metadata. `detect_simple_contradiction()` works on raw text. Neither calls the other; a fact could pass one but not the other.

**Merge**: Unified `ContradictionPipeline`: text pattern detection (cheap, fast) as first pass → embedding cosine (expensive, precise) as second pass. Single API, single confidence output.

### R5: Duplicate Forgetting/Freshness Mechanisms

| File | Mechanism |
|------|-----------|
| `nt_memory_brain.rs:123` | `ForgettingCurve` — Ebbinghaus retention rate |
| `nt_memory_sweep_20260815/` | `FreshnessLedger` — update timestamps + should_forget flag |
| `nt_memory_confidence.rs:440` | `ConfidenceStore::apply_decay()` — confidence decay |

**Issue**: Three independent "forgetting" mechanisms that don't coordinate. ForgettingCurve marks nodes in metadata. FreshnessLedger tracks update times. ConfidenceStore decays confidence scores. A node could be "forgotten" by one system but still appear in results from another.

**Merge**: Single `MemoryLifecycle` orchestrator that composes: (1) FreshnessLedger for timestamp tracking, (2) ForgettingCurve for retention rate calculation, (3) ConfidenceStore for confidence decay. All three share the same "should this node be retained?" decision.

### R6: Duplicate Search Fusion

| File | Fusion |
|------|--------|
| `nt_memory_search.rs:259` | `hybrid_search()` — FTS5 + BM25 + Walsh via RRF |
| `nt_memory_search.rs:656` | `fuse_signals()` — FTS5 + BM25 + Embed + Graph via weighted linear |

**Issue**: Two fusion functions with different algorithms (RRF vs weighted linear) and different signal combinations. `hybrid_search` is the production path; `fuse_signals` appears unused or used for different query modes.

**Merge**: Single `FusionEngine` that supports pluggable fusion strategies (RRF, weighted linear, etc.). All search paths go through it.

### R7: Duplicate Graph Traversal — GraphCache vs GraphRAG BFS

| File | Traversal |
|------|-----------|
| `nt_memory_graph_cache.rs` | `weighted_shortest_path()`, `all_paths()` |
| `nt_memory_graphrag/mod.rs:395` | `GraphRagStore::query()` — BFS subgraph extraction |

**Issue**: GraphRAG reimplements BFS traversal instead of using GraphCache. Both maintain their own adjacency structures.

**Merge**: GraphRAG query should accept a `&GraphCache` parameter and use its adjacency data. Eliminate `EntityGraph.adjacency` — it's a duplicate of `GraphCache.forward/backward`.

---

## 3. Flat Defect List

### F1: `nt_memory_graph.rs` — Stale BFS Path (Dead Code)

**Lines**: 7-119
**Issue**: `shortest_path()` uses unweighted BFS on a weighted graph. The `GraphCache` version (`weighted_shortest_path`) is the correct implementation. BFS version is semantically wrong and should be removed.
**Fix**: Delete `nt_memory_graph::shortest_path()`. Add `#[deprecated]` or remove entirely.

### F2: `nt_memory_graph.rs` — community_detection is Buggy

**Lines**: 182-227
**Issue**: `community_detection()` limits to 1000 nodes (`LIMIT 1000`), uses simple BFS connected-components (not real community detection), and doesn't handle edge weights. The `CommunityDetector` in `nt_memory_community.rs` is the real implementation.
**Fix**: Delete `nt_memory_graph::community_detection()`. Redirect callers to `CommunityAwareSearch`.

### F3: `nt_memory_store.rs` — Duplicate `insert_or_get_node` Functions

**Lines**: 76-116 vs 513-557
**Issue**: Two `insert_or_get_node` functions exist: `insert_or_get_node_rows` (no transaction) and `insert_or_get_node` (with transaction wrapper). The row-level version copies `summary` to `content` field, the wrapper version does the same but also wraps in a transaction. The logic is duplicated.
**Fix**: Keep `insert_or_get_node` (transactional). Delete `insert_or_get_node_rows` and have callers use the transactional version (or add a `_rows` suffix only for batch callers that manage their own transaction).

### F4: `nt_memory_brain.rs` — SynapticPlasticity UPSERT Schema Mismatch

**Line**: 91-98
**Issue**: `strengthen_coactivated()` uses `INSERT INTO edges ... ON CONFLICT(source_id, target_id)` but the `edges` table has a `UNIQUE` constraint on `id`, not on `(source_id, target_id)`. This UPSERT will fail silently or create orphan edges.
**Fix**: Use `upsert_edge()` from `nt_memory_store` which correctly handles the edge schema.

### F5: `nt_memory_brain.rs` — ForgettingCurve.update_freshness Uses `datetime('now')` (SQLite Timezone)

**Line**: 218
**Issue**: `UPDATE nodes SET ... updated_at < datetime('now', '-7 days')` uses SQLite's `datetime()` which returns UTC string, but `updated_at` is stored as Unix timestamp (i64). String comparison against integer is undefined behavior in SQLite.
**Fix**: Use `strftime('%s', 'now') - 7*86400` for consistent Unix timestamp comparison.

### F6: `nt_memory_graphrag/mod.rs` — Unused `Entity` and `Relation` Structs

**Lines**: 1920-1935
**Issue**: `Entity` and `Relation` are defined but only used in `GraphExtractor::extract()` which returns them. They duplicate `EntityNode`/`RelationEdge` with slightly different fields. `GraphExtractor` is a standalone extractor that doesn't integrate with the main `GraphRagStore::extract_entities()`.
**Fix**: Consolidate extraction into `GraphRagStore::extract_entities()`. Remove `GraphExtractor` or make it a thin wrapper.

### F7: `nt_memory_confidence.rs` — `search_with_confidence` Unused Weighted Score

**Lines**: 603-619
**Issue**: `RetrievalStrategy::ConfidenceWeighted` computes a weighted score but discards it (`let _ = ...`). Results are sorted by `aggregate()` regardless of the custom weights.
**Fix**: Apply the computed weighted score to re-rank results, or remove the dead computation.

### F8: `nt_memory_community.rs` — `query_cache` Field Never Used

**Line**: 724
**Issue**: `CommunityAwareSearch` has a `query_cache: HashMap<String, Vec<CommunityResult>>` field that is never read from or written to (only cleared in `clear_cache()`).
**Fix**: Remove the field, or implement cache lookup/store in `search_community()`.

### F9: `mod.rs` — `_from_conn` Duplicates `open()` Initialization

**Lines**: 260-302
**Issue**: `_from_conn()` is a private fallback constructor that duplicates ~30 lines of field initialization from `open()`. Any new field added to `KnowledgeBase` must be updated in both places.
**Fix**: Extract a `KnowledgeBase::init_fields(conn, db_path)` helper and call it from both `open()` and `_from_conn()`.

### F10: `mod.rs` — `clone_connection()` Opens Full KB + Drops Original

**Lines**: 429-441
**Issue**: `clone_connection()` calls `Self::open()` which re-initializes schema, rebuilds all stores, etc. It's not a "clone" — it's a second full open. The fallback chain (try path → try default → in-memory) is complex and may mask real errors.
**Fix**: Document clearly that this opens a new connection to the same DB (not a clone). Consider using SQLite's WAL mode + shared cache instead of opening separate connections.

---

## 4. Cross-Domain Misalignment List

### X1: L1 nt_memory_graph imports from nt_memory_store (Correct)

`nt_memory_graph.rs` calls `super::nt_memory_store::get_node()` and `get_edges_for_node()`. This is correct — L1 module calling L1 sibling.

### X2: GraphRAG Defines Parallel Entity Types (Wrong)

`nt_memory_graphrag/mod.rs` defines `EntityNode`/`RelationEdge` which are structurally parallel to `KnowledgeNode`/`KnowledgeEdge` from `nt_core_kb_types`. This creates two incompatible graph representations within the same L1 module.

**Fix**: GraphRAG should use `KnowledgeNode`/`KnowledgeEdge` or define types that convert 1:1.

### X3: nt_memory_brain.rs Imports from `nt_core_math` (Correct)

`nt_memory_brain.rs:15` imports `cosine_similarity_f64` from `crate::core::nt_core_math`. L1 importing from L5 core is the correct direction per the architecture.

### X4: nt_memory_confidence.rs Imports from `nt_memory_types` (Correct)

Uses `super::nt_memory_types::*` which re-exports from `nt_core_kb_types`. Correct layering.

### X5: GraphRAG `community_summary()` Bypasses `CommunityAwareSearch` (Wrong)

`GraphRagStore::community_summary()` implements its own label propagation instead of using `CommunityAwareSearch` from `nt_memory_community.rs`. Two community detection systems in the same module with no shared state.

**Fix**: GraphRAG should call `kb.community_search.read().hierarchy()` and consume Leiden results.

### X6: `nt_memory_search.rs` Calls `nt_memory_pipeline::compile_ingest_index` (Tight Coupling)

Line 97: `search_fts()` directly calls `compile_ingest_index()` from the pipeline module. This creates a dependency from search → ingestion pipeline, violating the principle that search should be independent of how data was ingested.

**Fix**: Move concept indexing to a shared utility or pass pre-compiled concepts as a parameter.

---

## 5. Data Flow Diagram

### Write Path

```
User/CLI/Crawl
    │
    ▼
write_memory_entry() ──────────────────────── mod.rs:955
    │
    ├── insert_or_get_node() ──→ nodes + nodes_fts (FTS5)
    │
    ├── record_node_fact() ──→ temporal_facts (append-only)
    │
    ├── generation stamp ──→ metadata.generation++
    │
    ├── block_stats ──→ metadata.block_types
    │
    ├── evidence ──→ metadata.evidence
    │
    ├── evaluate_write_gate() ──→ SVAF decision → metadata.svaf
    │
    ├── conflict_detect_for_write() ──→ supersede old nodes
    │       │
    │       └── record_decision_provenance() ──→ provenance log
    │
    └── mark_bm25_dirty() ──→ BM25 reindex on next search
```

### Read Path

```
User Query
    │
    ▼
search_fused() / hybrid_search() ────────── mod.rs (search methods)
    │
    ├── search_fts() ──→ FTS5 MATCH + title boost
    │
    ├── BM25 index ──→ in-memory BM25 search
    │
    ├── Walsh ranklist ──→ Hadamard orthogonal encoding
    │
    ├── RRF fusion ──→ rank Reciprocal Fusion
    │
    ├── title boost post-fusion
    │
    ├── confidence × decay rerank (adjacent results only)
    │
    ├── LIKE fallback (if insufficient results)
    │
    └── embedding cosine rerank (Tier 3)
            │
            └── DistilVDR student (if trained)
```

### Update Path

```
update_node() ──────────────────────────── mod.rs:909
    │
    ├── nt_memory_store::update_node() ──→ nodes + nodes_fts
    │
    ├── mark_bm25_dirty()
    │
    └── FreshnessLedger::note_updated()

update_node_content() ──────────────────── mod.rs:924
    │
    ├── get_node() → modify content → update_node()
    │
    └── mark_bm25_dirty()
```

### Delete Path

```
delete_node() ──────────────────────────── mod.rs:824
    │
    ├── nt_memory_store::delete_node() ──→ nodes_fts + nodes
    │
    ├── mark_bm25_dirty()
    │
    └── FreshnessLedger::mark_should_forget()

compact() ──────────────────────────────── mod.rs:387
    │
    ├── DELETE stale nodes (access_count=0, old)
    │
    ├── DELETE orphan edges
    │
    └── VACUUM
```

### Identified Flow Breaks

1. **GraphRAG writes bypass main KB**: `GraphRagStore::extract_entities()` writes to its own `EntityGraph` in memory, not to the `nodes`/`edges` SQLite tables. No BM25/embedding/freshness updates.
2. **Community detection not triggered on write**: `insert_node()` doesn't trigger community re-detection. Communities become stale until explicitly rebuilt.
3. **Embedding commitment not auto-triggered**: `insert_node()` doesn't auto-embed. Embeddings are a separate manual step (`/kb embed`).
4. **Confidence store not auto-populated**: New nodes get no confidence score until explicitly set via `store_node_confidence()`.

---

## 6. Priority Roadmap

### P0 — Critical (Fix Immediately)

| ID | Issue | Impact | Effort |
|----|-------|--------|--------|
| R1 | BFS `shortest_path` ignores weights | Incorrect shortest paths in production | 1h |
| F4 | SynapticPlasticity UPSERT schema mismatch | Edge strengthening silently fails | 1h |
| F5 | ForgettingCurve datetime() vs Unix timestamp | Stale node detection broken | 1h |
| F7 | ConfidenceWeighted strategy discards score | Weighted retrieval returns wrong order | 30m |

### P1 — High (This Sprint)

| ID | Issue | Impact | Effort |
|----|-------|--------|--------|
| R2 | Dual community detection (Leiden vs Label Prop) | Two conflicting community structures | 4h |
| R3 | Dual entity types (GraphRAG vs KB core) | Two parallel graphs, data inconsistency | 8h |
| R5 | Triple forgetting mechanism | Inconsistent node lifecycle | 4h |
| F1 | Dead BFS `shortest_path` | Misleading API surface | 30m |
| F2 | Buggy `community_detection` in graph.rs | Wrong community results for callers | 30m |
| F6 | Unused `Entity`/`Relation` structs in GraphRAG | Dead code, confusion | 1h |
| F8 | Unused `query_cache` in CommunityAwareSearch | Dead code | 15m |
| F9 | Duplicated `_from_conn` initialization | Maintenance burden | 1h |

### P2 — Medium (Next Sprint)

| ID | Issue | Impact | Effort |
|----|-------|--------|--------|
| R4 | Duplicate contradiction detection | Inconsistent conflict detection | 3h |
| R6 | Duplicate search fusion functions | Maintenance burden, unclear API | 2h |
| R7 | GraphRAG BFS bypasses GraphCache | Duplicate adjacency structures | 3h |
| F3 | Duplicate `insert_or_get_node` functions | Maintenance burden | 1h |
| F10 | `clone_connection` semantics misleading | Potential connection leaks | 2h |
| X6 | Search→Pipeline tight coupling | Hard to test search independently | 2h |

### P3 — Low (Backlog)

| ID | Issue | Impact | Effort |
|----|-------|--------|--------|
| GraphRAG writes bypass main KB | GraphRAG data invisible to BM25/embedding | Incomplete search coverage | 8h |
| Community detection not auto-triggered | Stale communities after writes | May return wrong clusters | 4h |
| Embedding not auto-triggered on write | Manual `/kb embed` required | Poor UX | 2h |
| Confidence store not auto-populated | New nodes have no confidence | Degrades retrieval quality | 2h |

---

## Appendix: Module Count Summary

| Category | Count | Lines (est.) |
|----------|-------|-------------|
| Storage | 5 | ~2,500 |
| Search | 4 | ~2,000 |
| Graph | 4 | ~3,000 |
| Brain | 3 | ~2,000 |
| Retrieval Intelligence | 8 | ~3,000 |
| Ingestion | 5 | ~2,000 |
| Quality Gates | 5 | ~2,500 |
| Cross-Session | 4 | ~1,500 |
| Meta | 20+ | ~8,000 |
| Privacy | 1 | ~500 |
| **Total** | **~78** | **~27,000** |

The `mod.rs` god struct has **28 RwLock/Mutex fields**. Each new subsystem adds a field. This is the root cause of most initialization duplication (F9) and the maintenance burden.
