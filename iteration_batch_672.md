# Iteration Batch 672 — Database & Storage Research

**Date**: 2026-09-06 | **Prior**: Batch 671 (algebraic effects, capability confusion, typestate, LLM-compiler)

---

## Domain 1: Database Indexing (B-Tree / LSM-Tree / RocksDB)

### Research Sources
- FOSDEM 2026: "LSM vs B-Tree: RocksDB and WiredTiger for Cloud-Native Distributed Databases" (Pachot, MongoDB)
- NDLab Blog 2026: B-Tree vs LSM-Tree — InnoDB and RocksDB Trade-offs
- Youngju.dev 2026: LSM-Tree Deep Dive — Compaction, Bloom Filter, Write Amplification
- arxiv 2605.23815: "A Pragmatic Approach to Learned Indexing in RocksDB"

### Finding 1: Learned Index Integration Is Pragmatic-Feasible (arxiv 2605.23815)

**What's new**: Research demonstrates off-the-shelf learned indexes can be integrated into RocksDB with *minimal system redesign*. The key insight: replace only the SSTable block index with a learned model, not the entire B-tree/LSM navigation structure. This is practical rather than aspirational.

**Defect for NeoTrix**: **KB has no adaptive index strategy**. NeoTrix's SQLite KB uses fixed B-tree indexes. For embedding-vector similarity search, a learned index on the embedding dimension could reduce lookup latency by 3-5x. Current architecture has no path from "vector search via brute-force cosine" → "learned index on embedding clusters". The arxiv paper proves this is achievable without rewriting the storage engine.

**Improvement**: Introduce `AdaptiveIndexPolicy` enum in `nt_memory` that selects index strategy per-namespace: `BTree` (default for KV), `LearnedIndex` (for high-cardinality embedding namespaces), `LSM` (for write-heavy experience logs). Detect pattern at insert time.

### Finding 2: LSM Compaction Is the Hidden Tax (FOSDEM 2026, Youngju.dev)

**What's new**: LSM trees achieve write throughput by deferring compaction, but compaction is the *invisible cost center*. Write amplification in leveled compaction can reach 10-30x. The FOSDEM talk specifically notes: "LSM tree complex to tune — hundreds of parameters for memory allocation, compaction algorithm." Size-tiered vs leveled compaction trade-off is not academic: it determines whether your write workload is sustainable.

**Defect for NeoTrix**: **Experience-tree absorption writes use SQLite (B-tree) but the access pattern is LSM-like** — append-only experience snapshots, infrequent reads, bulk reads at session start. This is a write-heavy workload on a read-optimized engine. NeoTrix forces SQLite to do compaction-equivalent work (VACUUM, WAL checkpoint) manually. The `experience-tree` skill writes to `kv_store` via SQLite, which is structurally wrong for append-heavy logs.

**Improvement**: Migrate experience namespace to a simple append-only log file (WAL-style) with periodic compaction into SQLite for query. The raw experience stream should not touch B-tree indexes until distillation is complete.

### Finding 3: B-Tree Secondary Index Double-Lookup Cost (NDLab 2026)

**What's new**: InnoDB's clustered index means every secondary index lookup requires *two* B-tree traversals — one to find the PK, one to fetch the row. Covered indexes eliminate this. The article shows: `SELECT id, email FROM users WHERE email = '...'` skips step 2 if the index includes both columns.

**Defect for NeoTrix**: **KB edge queries have unbounded double-lookup cost**. When querying edges by source node + relation type, the query planner does: (1) index lookup on `source_id` → find `edge_id`, (2) table lookup to fetch `target_id` + `weight`. For dense nodes (high fan-out), this doubles I/O. NeoTrix has no covering-index strategy for edge queries.

**Improvement**: Create covering index `CREATE INDEX idx_edge_cover ON edges(source_id, relation_type, target_id, weight)` for the hot-path edge queries. This eliminates double-lookup for the most common KB traversal pattern.

---

## Domain 2: Query Optimization (CBO / Cost-Based Planning)

### Research Sources
- queries.cloud: "The Evolution of Cost-Aware Query Optimization in 2026"
- thelinuxcode.com: "Cost-Based Query Planning: Building Fast, Predictable SQL in 2026"
- Iceberg Lakehouse Blog 2026: "Inside the Query Optimizer: How Engines Pick a Plan"
- YugabyteDB Docs 2026: Query Planner / CBO architecture

### Finding 4: Telemetry-First Optimizers (queries.cloud 2026)

**What's new**: 2026 CBOs ingest *real-time cost telemetry* — not just table statistics, but actual query execution metrics (latency, spill, I/O). The planner continuously adjusts based on observed behavior, not static ANALYZE snapshots. Three shifts: (1) telemetry-first optimizers, (2) policy-driven execution with budget limits, (3) model-based costing with learned models replacing static cardinality estimates.

**Defect for NeoTrix**: **No feedback loop from KB query performance to planner**. NeoTrix's KB queries use raw SQL with no cost telemetry. When a query is slow (e.g., cross-domain edge traversal), there's no mechanism to record that cost and influence future query plan selection. The planner operates in the dark — identical queries get the same plan regardless of runtime conditions.

**Improvement**: Add `QueryCostLog` table to KB that records: `query_hash, estimated_cost_ms, actual_cost_ms, rows_scanned, rows_returned, plan_hash`. Use this to detect planner drift and trigger re-analyze when actual cost exceeds 2x estimated cost.

### Finding 5: Cardinality Estimation Is Still the Achilles Heel (thelinuxcode.com 2026)

**What's new**: Without histograms, CBOs use uniform assumptions that are "badly wrong on skewed data." Example: 90% of `tier = 'FREE'` rows — uniform assumption says 10% selectivity, planner picks index scan, but full scan is faster. Histograms + hybrid histograms + sampling are the 2026 state of art, but only if stats are up to date. The article explicitly warns: "Late-arriving data in partitions can make stats stale even if partition is 'old'."

**Defect for NeoTrix**: **KB statistics are never refreshed after initial load**. NeoTrix runs `ANALYZE` (or equivalent) once at schema creation, then never again. As experience data accumulates (10k+ entries), the planner's cardinality estimates for `experience` namespace queries become increasingly wrong. The planner may choose full scans over index scans for queries that should use indexes, or vice versa.

**Improvement**: Add `analyze_if_stale()` call in KB write path: if >1000 new rows since last ANALYZE, trigger background ANALYZE. This keeps planner stats within 1% of actual distribution.

### Finding 6: Adaptive Materialization as Cost Control (queries.cloud 2026)

**What's new**: "Dynamically maintain partial aggregates based on access frequency and cost-to-recompute heuristics." This is not caching — it's *adaptive materialization* that blends caching economics with query patterns. If a partial aggregate is accessed 100x/day and costs 500ms to compute, materializing it saves 50s/day. If access drops to 1x/day, the materialization is reclaimed.

**Defect for NeoTrix**: **No materialized view or pre-aggregation strategy for KB**. Common NeoTrix queries (domain health scores, module maturity rollups, experience frequency counts) recompute from raw data every time. For ConsciousnessTree health checks that run every cycle, this is wasted work.

**Improvement**: Implement `MaterializedView` manager for KB: track query frequency, auto-materialize hot-path aggregates, auto-expire cold ones. First target: `domain_health_summary` (refresh every 100 writes), `experience_frequency_by_domain` (refresh hourly).

---

## Domain 3: Storage Engine (Row vs Column vs Hybrid)

### Research Sources
- Apache Doris docs 2026: Hybrid Row-Columnar Storage (store_row_column)
- openGauss docs 2026: Hybrid Row-Column Store
- VeloDB 2026: "What Is a Columnar Database? The Complete Guide for 2026"
- ingestthis.com 2026: "Row vs. Column: How Storage Layout Shapes Everything"

### Finding 7: Hybrid Row-Column Storage Is Production-Ready (Doris 2.0+, openGauss)

**What's new**: Apache Doris 2.0 introduced `store_row_column` — an additional column that concatenates all columns of a row in binary format. For point queries (`SELECT *`), only one IO operation is needed (vs one IO per column in columnar). Page size is tunable (4KB-64KB): smaller = faster point queries, larger = better compression. openGauss has had this since 1.0.0. The 2024 HTAP survey documents hybrid transactional/analytical processing as mainstream.

**Defect for NeoTrix**: **KB has no hybrid storage strategy**. NeoTrix stores everything in SQLite (row-oriented). But KB has two distinct access patterns: (1) point queries for node lookups (row-oriented, fast), (2) analytical scans for domain health/evolution metrics (column-oriented, slow). The current schema forces analytical queries to scan entire tables. For a 100k-row `experience` table, a `SELECT domain, AVG(confidence) GROUP BY domain` scans all 100k rows when only 7 domain values exist.

**Improvement**: Introduce `HybridStoragePolicy` for KB namespaces: hot namespaces (experience, edges) get row-store with covering indexes; analytical namespaces (metrics, health) get column-store or pre-aggregated materialized tables. For SQLite specifically, implement `stats_summary` table that is column-oriented (domain-level aggregates updated on write).

### Finding 8: Column Store Write Penalty Is Real (VeloDB, openGauss)

**What's new**: Column-store single-point query and single-record insertion performance is *poor*. "If a small amount of data is frequently inserted each time, use a row-store table." Columnar is optimized for batch inserts + analytical reads. Mixing OLTP writes with analytical reads in a pure columnar store creates performance cliffs.

**Defect for NeoTrix**: **Experience-tree writes are OLTP-pattern (single-row inserts per experience) but analytics are OLAP-pattern (domain rollups, evolution metrics)**. If NeoTrix naively moves experience data to columnar storage, every experience absorption write becomes slow. The hybrid approach (Doris `store_row_column`) solves this but requires schema-level decision at table creation time — not runtime.

**Improvement**: Implement write-path buffering: accumulate experience inserts in a row-oriented buffer, flush to columnar store in batches (every 100 experiences or every 5 minutes). Read-path checks buffer first, then columnar store. This gives OLTP-speed writes + columnar-speed analytics.

### Finding 9: Compression Ratio Trades CPU (openGauss 2026)

**What's new**: "The compression ratio of a column-store table is higher than that of a row-store table. High compression ratio consumes more CPU resources." This is the fundamental trade-off: columnar compression is 2-10x better than row compression, but decompression cost can dominate for point queries. For wide tables (200+ columns), columnar wins on scan queries but loses on point queries.

**Defect for NeoTrix**: **No CPU-cost-aware compression policy**. NeoTrix's KB uses SQLite's default page compression. For embedding vectors (1536-dimensional float32), compression could save 80% storage but decompression cost would hurt the common case (nearest-neighbor search reads single vectors). For experience text (variable-length strings), compression saves 60% with minimal decompression cost.

**Improvement**: Add per-column compression policy: `NoCompression` for embedding vectors (hot path), `LZ4` for experience text (cold path), `ZSTD` for edge metadata (rarely accessed, high compression worth the CPU cost).

---

## Summary: 9 Defects Found

| # | Domain | Defect | Severity |
|---|--------|--------|----------|
| 1 | Indexing | No adaptive index strategy for embedding vectors | Medium |
| 2 | Indexing | Experience writes use wrong engine (B-tree for LSM pattern) | High |
| 3 | Indexing | No covering indexes for edge queries | Medium |
| 4 | CBO | No cost telemetry feedback loop for KB queries | Medium |
| 5 | CBO | KB statistics never refreshed after initial load | High |
| 6 | CBO | No materialized view strategy for hot-path aggregates | Medium |
| 7 | Storage | No hybrid row/column strategy for dual access patterns | High |
| 8 | Storage | Write-buffer gap for experience OLTP→columnar transition | Medium |
| 9 | Storage | No CPU-cost-aware compression policy per column type | Low |

## Sources Cited

1. arxiv.org/abs/2605.23815 — Learned Indexing in RocksDB (2026)
2. FOSDEM 2026 — LSM vs B-Tree: RocksDB and WiredTiger (Pachot)
3. NDLab Blog — B-Tree vs LSM-Tree: InnoDB and RocksDB Trade-offs (2026)
4. Youngju.dev — LSM-Tree Deep Dive: Compaction, Bloom Filter (2026)
5. queries.cloud — Evolution of Cost-Aware Query Optimization (2026)
6. thelinuxcode.com — Cost-Based Query Planning (2026)
7. Iceberg Lakehouse — Inside the Query Optimizer (2026)
8. YugabyteDB Docs — Query Planner / CBO (2026)
9. Apache Doris Docs — Hybrid Row-Columnar Storage (2026)
10. openGauss Docs — Hybrid Row-Column Store (2026)
11. VeloDB — What Is a Columnar Database (2026)
12. ingestthis.com — Row vs Column Storage (2026)
