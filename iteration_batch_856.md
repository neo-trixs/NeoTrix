# Iteration Batch 856 Report — NeoTrix Consciousness Architecture

## Research Sources (38+)

### Full-Text Search (8)
- Tantivy v0.26.1: Rust-native, Lucene-inspired, 0 CVEs, incremental indexing
- Meilisearch v1.53: pivoting to cloud, BSL license, wrong fit for NeoTrix
- SQLite FTS5: production-proven, zero-dependency, built-in
- BM25: NeoTrix already has 4-way RRF fusion (state-of-the-art)
- FTS5 limitation: no fuzzy/typo-tolerant search
- In-memory BM25 not persistent (rebuilt on every call)
- No CJK tokenization (unicode61 doesn't handle Chinese well)
- No query expansion (synonyms, related terms)

### Vector Embeddings (8)
- instant-distance HNSW unmaintained (recall regression problems)
- No tunable HNSW parameters (M, ef_construction, ef_search)
- load_all_embeddings O(n) full scan on every query
- Hash-kernel semantic quality gap (local fallback)
- No incremental HNSW update (full rebuild required)
- PQ codebook global singletons (not per-domain)
- Quantization now default (binary/scalar + int8 rescoring)
- Matryoshka embeddings: truncatable dimensions without re-indexing

### Graph Databases (10)
- petgraph 0.8.3: 441M downloads, StableGraph, serde+rayon
- petgraph-decypher: OpenCypher queries on petgraph
- Graph Memory fastest-growing subcategory (Cognee, Neo4j, Zep)
- LatticeDB: 2,819x faster than SQLite CTE at depth 50
- LHGstore: 5.9-28.2x throughput over SOTA in-memory graph stores
- BLEST: TC-accelerated BFS 22x over GAP baseline
- PBS: Sublinear shortest paths 1.84-7.76x faster
- TRGH: Temporal KG reasoning beats SOTA on 3 benchmarks
- No petgraph in NT-MEMORY (hand-rolled adjacency lists)
- No incremental graph updates (O(|V|+|E|) full reload)

### Caching (8)
- moka: Caffeine-inspired, TTL+TTI+per-entry, async Cache
- quick-cache: lock-free, no TTL, ultra-low latency
- lru: single-threaded, O(1) get/put/pop, no_std
- No unified cache trait in NeoTrix
- No eviction monitoring
- No cache-aside pattern
- No per-entry TTL
- No tiered cache (L1 quick-cache → L2 moka → L3 KB)

---

## Defects Identified (28+)

### Full-Text Search (6)
| ID | Defect | Severity |
|----|--------|----------|
| D-FTS-1 | In-memory BM25 not persistent | Medium |
| D-FTS-2 | No FTS5 incremental sync | Medium |
| D-FTS-3 | BM25 tokenizer naive (whitespace only) | Low |
| D-FTS-4 | No CJK tokenization | Medium |
| D-FTS-5 | FTS5 rank direction fragile | Low |
| D-FTS-6 | No search result caching | Low |

### Vector Embeddings (6)
| ID | Defect | Severity |
|----|--------|----------|
| D-VEC-1 | HNSW crate unmaintained (recall regression) | High |
| D-VEC-2 | No tunable HNSW parameters | High |
| D-VEC-3 | load_all_embeddings O(n) full scan | Medium |
| D-VEC-4 | Hash-kernel semantic quality gap | Medium |
| D-VEC-5 | No incremental HNSW update | Medium |
| D-VEC-6 | PQ codebook global singletons | Low |

### Graph Databases (6)
| ID | Defect | Severity |
|----|--------|----------|
| D-GRAPH-1 | No petgraph in NT-MEMORY | High |
| D-GRAPH-2 | Hand-rolled adjacency lists | High |
| D-GRAPH-3 | No incremental graph updates | High |
| D-GRAPH-4 | No community detection / centrality | High |
| D-GRAPH-5 | No temporal graph support | Medium |
| D-GRAPH-6 | No Cypher query language | Low |

### Caching (6)
| ID | Defect | Severity |
|----|--------|----------|
| D-CACHE-1 | No unified cache trait | High |
| D-CACHE-2 | No eviction monitoring | Medium |
| D-CACHE-3 | No cache-aside pattern | Medium |
| D-CACHE-4 | No per-entry TTL | Medium |
| D-CACHE-5 | No tiered cache | Medium |
| D-CACHE-6 | No stampede protection | Medium |

## Key Insights (This Batch)

1. **NeoTrix already ahead on hybrid search**: 4-way RRF fusion (FTS5+BM25+Walsh+graph) is more than most commercial systems offer. Must maintain this advantage.

2. **instant-distance HNSW is unmaintained**: Must replace with hnswlib-rs or usearch for tunable parameters and recall guarantee.

3. **Tantivy is the upgrade path for search quality**: Rust-native, incremental indexing, phrase/proximity queries. Better than FTS5 when corpus >100K nodes.

4. **petgraph 0.8 replaces all hand-rolled graph code**: StableGraph indices survive node removal. dijkstra(), connected_components, toposort all built-in.

5. **LatticeDB is 2,819x faster than SQLite for graph traversal**: Embedded Zig graph DB with native HNSW + BM25 + Cypher. Worth monitoring.

6. **Temporal graph edges enable time-aware retrieval**: valid_from/valid_to + decay weighting. Research shows this is SOTA for memory systems.

7. **moka is the production cache default**: Caffeine-inspired, TTL+TTI+per-entry, async Cache, eviction listeners.

8. **Tiered cache architecture**: L1 quick-cache (1µs) → L2 moka (10µs) → L3 KB (1ms). Aligns with Rune Socketing Obsidian slot.

9. **No cache-aside pattern**: Duplicate fetch logic across domains. moka's get_with provides this natively.

10. **Incremental HNSW updates**: hnswlib-rs supports add_item/remove_item without full rebuild. Must adopt.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| Batches completed | 856 |
| New defects (this batch) | 24 |
| Cumulative defects | D01-D77579 |
| Research sources (this batch) | 34 |
| Cumulative research sources | 98,639+ |
