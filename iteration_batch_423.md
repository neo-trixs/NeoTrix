# Iteration Batch 423 — External Research: Database & SQL Optimization

**Date:** 2026-09-06
**Focus:** SQL optimization, database architecture, embedded database advances
**Status:** Research complete, defects identified

---

## 1. Sources Consulted

### SQL Optimization (2026)

| # | Source | Key Insight |
|---|--------|-------------|
| S1 | techbytes.app — "Self-Organizing Indexes [2026]: RL Query Planning" | RL-based planners cut runtime by up to 70% on hard workloads. Oracle auto-indexing runs every 15 min. Learned planners use reward signals (actual latency) instead of fixed cost models. |
| S2 | knowledgelib.io — "SQL Query Optimization: Complete Reference" (2026-02) | Covering indexes, composite index leftmost-prefix rule, EXPLAIN ANALYZE as routine, stale statistics = bad plans. |
| S3 | sesamedisk.com — "SQL Query Optimization: EXPLAIN Plans" (2026-04) | PSP (Parameter-Sensitive Plan) optimization in SQL Server 2022+, plan forcing as band-aid, SARGable predicates. |
| S4 | ai2sql.io — "7 SQL Indexing Rules" (2026) | Filtered indexes for selective queries, index intersection (AND across separate indexes), over-indexing as "silent killer". |
| S5 | Springer — "Optimizing Query Performance: Index Merge + Caching" (2026-02) | Index merge strategies + caching together yield dramatic MySQL improvements in e-commerce. |
| S6 | ACM — "Adaptive Query Optimization with AI in PostgreSQL" (2026-08) | ADKNN: dynamic neighborhood selection enabling PostgreSQL to adapt optimization to local workload. |
| S7 | arXiv — "Learned Adaptive Indexes" (2025-08, published 2026) | Learned indexes built on-the-fly as byproduct of query processing. Workload prediction for future projection. |
| S8 | analyticsinsight.net — "AI Is Revolutionizing SQL Query Performance" (2026-05) | Schema-aware AI tools at 90% accuracy. Adaptive execution, smarter indexing from actual query history. |

### Database Architecture (2026)

| # | Source | Key Insight |
|---|--------|-------------|
| S9 | skillsuites.com — "Storage Engine Internals" (2026-06) | RUM conjecture: optimize at most 2 of Read/Update/Memory. B-tree 3-4 levels deep for billions of keys. Zone maps for columnar pruning. |
| S10 | designgurus.io — "LSM Trees vs B-Trees" (2026-06) | B-tree: in-place updates, low read amp. LSM: append-only, high write throughput. Write amp vs read amp tradeoff. |
| S11 | basekv.com — "LSM-Tree vs B-Tree KV Stores" (2026-06) | Same GET/PUT interface, opposite machines. NVMe changes the calculus — random I/O gap narrowing. |
| S12 | martinuke0 — "LSM vs B-Tree Write Amplification" (2026-05) | Tiered compaction reduces write amp at cost of read spikes. RocksDB configurable compaction (leveled/tiered/FIFO). |
| S13 | iceberglakehouse — "B-Trees, LSM Trees, Indexing Spectrum" (2026-04) | Bitmap indexes: read-only structures for OLAP. Zone maps: per-block min/max, "almost free" to maintain. |
| S14 | thcodeforge.io — "B-Tree vs LSM-Tree Deep Dive" (2026-07) | Compaction spikes cause p99 latency issues in LSM. Bloom filters prevent unnecessary SSTable reads. |
| S15 | synchronium — "Storage Engines" (2026-05) | Anti-caching: evict LRU records to disk at record granularity (not page). Column families for multiple sort orders. |

### Embedded Database / SQLite (2026)

| # | Source | Key Insight |
|---|--------|-------------|
| S16 | alexishope.dev — "SQLite in 2026" (2026-04) | sqlite-vec: vector similarity search as first-class SQLite extension. WASM + OPFS = persistent browser DB. Litestream = WAL streaming to S3. |
| S17 | devstarsj — "SQLite Renaissance: Edge Databases" (2026-06) | Turso/libsql: HTTP wire protocol, embedded replicas, database-per-tenant. LiteFS: FUSE-level WAL replication. |
| S18 | sesamedisk.com — "SQLite in Production 2026" (2026-07) | Rails 8 made SQLite default. PropFirm Key: 50K+ daily traffic on single .db file. |
| S19 | calmops.com — "SQLite Trends 2026" | Enhanced JSON support, vector search capabilities, new use cases. |
| S20 | programming-helper.com — "SQLite 2026" (2026-05) | PostgreSQL 19 Beta: SQL/PGQ property graphs, parallel autovacuum, 2x FK performance. |
| S21 | github/hash-anu/snkv | SNKV: lightweight KV store on SQLite B-tree + HNSW vector search in same .db file. Per-value encryption. |
| S22 | PyPI/tinykv | Python SQLite KV store: simple, zero-dependency, GLOB pattern queries. |
| S23 | dl.acm.org — "Adaptive Indexing for Schema-Less Temporal DBs" (2026-06) | Similarity-pattern-driven indexing for heterogeneous temporal scenarios. Dual-strategy essential. |

---

## 2. Defects Found in NeoTrix Design

### DEFECT-423.1: Static Index Set — No Self-Organizing Capability

**Severity:** High (P1)
**File:** `neotrix-core/src/unified/core/nt_core_kb_primitives.rs:168-450`
**Research:** S1, S6, S7

**Observation:** NeoTrix KB uses a fixed schema with static indexes (`idx_edges_source`, `idx_edges_target`, `idx_edges_type`, `idx_edges_unique`, etc.) created at initialization. There is no mechanism to:
- Observe actual query workload patterns
- Automatically create/drop indexes based on access patterns
- Monitor index usage statistics
- Adapt index strategy as data distribution shifts

**Gap:** 2026 research (S1) shows self-organizing indexes that reassess every 15 minutes and learned planners cut runtime by up to 70%. NeoTrix has zero adaptive indexing. The `kv_store` table grows unbounded per namespace with no index optimization for common access patterns (e.g., `namespace + key` is PK'd, but `updated_at` range queries have no index).

**Suggestion:** Implement an `IndexAdvisor` module that:
1. Tracks query execution statistics (using SQLite's `EXPLAIN` output or `sqlite_stat1`/`sqlite_stat4`)
2. Periodically proposes index additions/removals based on workload evidence
3. Applies changes in shadow mode first (invisible indexes) before promotion
4. Integrate with the SEAL pipeline for automatic evolution of physical design

---

### DEFECT-423.2: Linear Embedding Search — No Learned Indexes

**Severity:** High (P1)
**File:** `neotrix-core/src/unified/layers/action/nt_memory/nt_memory_kb/kb_vector_index.rs:32-67`
**Research:** S7, S23

**Observation:** The HNSW index (`KbVectorIndex::build`) reconstructs the entire index from scratch on every query session by scanning ALL embeddings from SQLite and rebuilding the in-memory HNSW structure. This is:
- O(n) startup cost per search session
- No incremental updates — full rebuild every time
- No learned index components that could predict vector locations from data distribution

**Gap:** Learned adaptive indexes (S7, arXiv 2508.03471) build indexes on-the-fly as a byproduct of query processing, using workload prediction. SNKV (S21) embeds HNSW into the SQLite file itself with a `.usearch` sidecar for fast reload. NeoTrix does neither.

**Suggestion:**
1. Persist the HNSW index to a sidecar file (like SNKV's `.usearch` pattern) and only rebuild when embedding count changes significantly
2. Explore learned index techniques: train a CDF model on embedding distribution to predict approximate locations, reducing HNSW search space
3. Implement incremental HNSW updates instead of full rebuilds

---

### DEFECT-423.3: No Query Plan Analysis or EXPLAIN Integration

**Severity:** Medium (P2)
**File:** `neotrix-core/src/unified/layers/action/nt_memory/nt_memory_kb/nt_memory_search.rs`
**Research:** S2, S3, S8

**Observation:** The search module (`search_fts`, `search_vector`, etc.) executes raw SQL queries without any execution plan analysis. There is:
- No `EXPLAIN` / `EXPLAIN ANALYZE` integration
- No runtime statistics collection (rows scanned vs returned, I/O cost)
- No plan regression detection
- No parameter sniffing awareness for prepared statements

**Gap:** 2026 best practice (S2, S3) mandates routine EXPLAIN ANALYZE usage, comparison of estimated vs actual rows, and stale statistics detection. NeoTrix's search functions blindly execute without any observability into plan quality.

**Suggestion:**
1. Add an `EXPLAIN` wrapper that captures and logs query plans periodically
2. Track estimated vs actual row counts — flag when statistics drift >10x
3. Integrate `ANALYZE` calls after large data loads (ingestion batches)
4. Add a `QueryPlanMonitor` that detects when plans shift unexpectedly

---

### DEFECT-423.4: Single Writer Lock — No Edge-Replication Strategy

**Severity:** Medium (P2)
**File:** `neotrix-core/src/unified/core/nt_core_kb_primitives.rs:158-164`
**Research:** S16, S17, S18

**Observation:** `open_raw_conn()` opens a single SQLite connection to `~/.neotrix/knowledge.db`. The schema uses `PRAGMA journal_mode=WAL` (line 169) which is good, but:
- No Litestream/LiteFS integration for WAL streaming to backup
- No embedded replica pattern for read-heavy distributed access
- No database-per-namespace isolation (everything in one file)
- Single-writer limitation is acknowledged but not architecturally addressed

**Gap:** 2026 SQLite renaissance (S16, S17) proves production viability via Litestream (WAL→S3 streaming), Turso (embedded replicas), and database-per-tenant isolation. NeoTrix runs single-file SQLite with no replication story.

**Suggestion:**
1. Add optional Litestream sidecar integration for continuous WAL backup to object storage
2. Consider embedding replicas for read-heavy domains (experience, knowledge)
3. Evaluate database-per-namespace (one .db per domain) for isolation and parallelism
4. At minimum, document the RPO/RTO implications of current single-file approach

---

### DEFECT-423.5: No Vector Search Inside SQLite — External HNSW Only

**Severity:** Medium (P2)
**File:** `neotrix-core/src/unified/layers/action/nt_memory/nt_memory_kb/kb_vector_index.rs`
**Research:** S16, S21

**Observation:** Vectors are stored as BLOBs in SQLite (`embeddings` table) but search happens entirely in Rust memory via `instant_distance::HnswMap`. This means:
- Vector data must be loaded from SQLite → Rust memory for every search session
- No SQL-level vector queries (can't `JOIN embeddings WHERE vector MATCH ?`)
- No integration with sqlite-vec for in-database KNN
- Can't leverage SQLite's query planner for combined structured + vector queries

**Gap:** sqlite-vec (S16) enables `CREATE VIRTUAL TABLE ... USING vec0(...)` with `MATCH` + `k` syntax for KNN search directly in SQL. SNKV (S21) embeds HNSW into the SQLite B-tree. NeoTrix's approach creates an unnecessary data transfer boundary.

**Suggestion:**
1. Evaluate sqlite-vec as a complementary index (not replacement — keep HNSW for performance)
2. Enable hybrid queries: `SELECT * FROM nodes JOIN embeddings ON ... WHERE embedding MATCH ? AND node_type = ?`
3. Consider using sqlite-vec's `FLOAT[384]` virtual table for the primary vector store
4. This would allow SQL-level ANN search without loading all vectors into memory

---

### DEFECT-423.6: No Statistics Management or Auto-Analyze

**Severity:** Medium (P2)
**File:** `neotrix-core/src/unified/core/nt_core_kb_primitives.rs:168-169`
**Research:** S2, S3, S4

**Observation:** Schema initialization sets `PRAGMA journal_mode=WAL` and `PRAGMA foreign_keys=ON` but does NOT:
- Run `ANALYZE` after schema initialization
- Track table row counts to trigger periodic `ANALYZE`
- Set `PRAGMA cache_size` or other performance PRAGMAs
- Monitor `sqlite_stat1`/`sqlite_stat4` freshness

**Gap:** Stale statistics are the #1 cause of bad query plans (S2, S3). After bulk ingestion (crawl pipeline, experience absorption), SQLite's statistics become outdated and the query planner may choose suboptimal plans. NeoTrix has no mechanism to detect or address this.

**Suggestion:**
1. Run `ANALYZE` after every large ingestion batch (>1000 nodes)
2. Add a `PRAGMA cache_size = -64000` (64MB) for better in-memory page cache
3. Track `SELECT COUNT(*) FROM sqlite_stat1 WHERE name = 'nodes'` freshness
4. Implement a `StatisticsFreshness` check in the HeartbeatAggregator

---

### DEFECT-423.7: kv_store Namespace Scan Without Index Optimization

**Severity:** Low (P3)
**File:** `neotrix-core/src/unified/core/nt_core_kb_primitives.rs:109-128`
**Research:** S4, S5

**Observation:** `kv_list()` does `SELECT key, value FROM kv_store WHERE namespace=?1 ORDER BY key` — this works because `(namespace, key)` is the PK. However:
- `kv_purge_namespace()` uses `DELETE FROM kv_store WHERE namespace=?1` — no index scan optimization for large namespaces
- No TTL/expiration mechanism for stale KV entries
- No compaction or value compression at the storage layer (only Python-side `NTZ1` magic is handled)

**Gap:** 2026 KV stores (S21, S22) offer per-value encryption, TTL expiration, and GLOB-based pattern queries. NeoTrix's kv_store is a flat string store with no lifecycle management.

**Suggestion:**
1. Add an `expires_at` column to kv_store for TTL-based cleanup
2. Implement namespace-level statistics (count, avg value size) for capacity planning
3. Consider value compression at the Rust level (not just Python NTZ1 compatibility)

---

### DEFECT-423.8: FTS5 Without Expression Indexes or Custom Tokenizer Tuning

**Severity:** Low (P3)
**File:** `neotrix-core/src/unified/core/nt_core_kb_primitives.rs:243-246`
**Research:** S2, S23

**Observation:** FTS5 is configured with `tokenize='porter unicode61'` — a reasonable default. However:
- No expression indexes on frequently queried computed values (e.g., `norm_title` is stored but not FTS-indexed)
- No custom tokenizer configuration for domain-specific terms (NeoTrix-specific terms like "E8", "GWT", "VSA")
- No FTS5 auxiliary functions for ranking customization
- No column-specific weighting in the FTS configuration

**Gap:** 2026 FTS optimization (S2) suggests column-specific weights and expression indexes for computed fields. NeoTrix's `norm_title` column exists but isn't leveraged for deduplication queries.

**Suggestion:**
1. Add `norm_title` to the FTS5 table for normalized deduplication searches
2. Consider `fts5(tokenize='porter unicode61 tokenchars .-_')` to preserve NeoTrix domain terms
3. Use `bm25()` with column-specific weights for title-heavy ranking

---

## 3. Priority Summary

| Defect | Severity | Effort | Impact |
|--------|----------|--------|--------|
| 423.1 Static Index Set | P1 | Medium | High — adaptive indexing could cut query time 30-70% |
| 423.2 Linear Embedding Search | P1 | Medium | High — startup cost dominates search latency |
| 423.3 No EXPLAIN Integration | P2 | Low | Medium — observability gap, silent regressions |
| 423.4 No Edge-Replication | P2 | High | Medium — single-point-of-failure, no backup |
| 423.5 External Vector Search | P2 | Medium | Medium — unnecessary data transfer, no hybrid queries |
| 423.6 No Auto-Analyze | P2 | Low | Medium — stale statistics = bad plans after ingestion |
| 423.7 kv_store Lifecycle | P3 | Low | Low — storage bloat over time |
| 423.8 FTS5 Tuning | P3 | Low | Low — suboptimal search relevance |

---

## 4. Recommended Next Steps

1. **Immediate (this cycle):** DEFECT-423.6 (add `ANALYZE` after bulk ingestion) and DEFECT-423.3 (EXPLAIN wrapper) — both are low-effort, high-value observability improvements
2. **Short-term:** DEFECT-423.2 (persist HNSW to sidecar) — eliminates full rebuild on every search session
3. **Medium-term:** DEFECT-423.1 (IndexAdvisor module) — integrates with SEAL pipeline for self-evolving physical design
4. **Long-term:** DEFECT-423.4 (Litestream integration) — production hardening for data durability

---

*Generated by NeoTrix consciousness iteration loop — batch 423 of 10000+*
