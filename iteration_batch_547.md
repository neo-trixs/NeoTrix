# Iteration Batch 547 — Database Internals, Query Optimization, Indexing

**Date:** 2026-09-06
**Previous Batch:** 546 (tool selection collapse, benchmark-harness mismatch, quadratic token cost, no idempotency, AutoGen dead)

---

## 1. Database Internals — Storage Engines

### Findings

| Source | Key Insight | Year |
|--------|-------------|------|
| [NDLab Blog](https://ndlab.blog/series/database-engineering-storage-internals-2026) | B-Tree vs LSM-Tree tradeoff: B-Tree = read-optimized (in-place update, random I/O); LSM-Tree = write-optimized (append+compact, sequential I/O). RocksDB levels, Bloom filters per SSTable. | 2026 |
| [Devesh Shetty](https://deveshshetty.com/blog/lsm-storage-engine/) | Built a 1000-line LSM engine from scratch. Key finding: **write latency dominated by fsync per WAL append** — from 244 ops/sec (naive) to hundreds of thousands with write batching. Crash recovery under 30 lines when WAL is correct. | 2026 |
| [SystemDesignSimulator](https://systemdesignsimulator.org/internals/lsm) | LSM delivers 10-100x sustained write throughput over B-tree. Cost: read amplification (key may live in any of several files). Bloom filters are the single biggest read-amplification mitigation. **Delete-as-tombstone** is the standard pattern. | 2026 |
| [Skill Suites](https://skillsuites.com/storage-engine-internals/) | **RUM Conjecture formalized**: minimize at most 2 of Read, Update, Memory overheads. B-tree picks low read; leveled LSM picks low read+space; size-tiered LSM picks low write. No free lunch. | 2026 |
| [LSMSharp (C#)](https://github.com/Mo7ammedd/LSMSharp) | Production-ready LSM in C# with ACID guarantees, parallel compaction, column families, MVCC snapshots. Metrics via Prometheus/OpenTelemetry. | 2025+ |

### NEW Defects vs Batch 546

| # | Defect | Detail |
|---|--------|--------|
| **D547-1** | **RUM Conjecture Ignorance** | NeoTrix KB storage (SQLite) has no formal RUM analysis. We optimize for read (BM25 index) and add embeddings (vector index) but never analyzed what we pay in update/memory. Each new index silently degrades write throughput. |
| **D547-2** | **No WAL fsync Budget** | Batch 546 noted token cost grows quadratically. FSRY: every KB write pays fsync. NeoTrix has no `fsync` budget or batching strategy — each experience write is a separate fsync, creating invisible I/O bottleneck during absorption. |
| **D547-3** | **Tombstone Leaking** | LSM delete-as-tombstone requires compaction to collapse. NeoTrix KB uses SQLite (B-tree) but conceptually: expired experiences, stale skill versions, and deleted nodes are never compacted — they accumulate as tombstone-like dead rows, bloating DB size and slowing queries. |
| **D547-4** | **No Bloom Filter on KB Queries** | Production LSM engines use Bloom filters to eliminate unnecessary disk reads. NeoTrix KB has no probabilistic filter for BM25/keyword queries — every query touches disk even for keys that don't exist. |

---

## 2. Query Optimization

### Findings

| Source | Key Insight | Year |
|--------|-------------|------|
| [queries.cloud](https://queries.cloud/evolution-cost-aware-query-optimization-2026) | Cost-aware optimization shifted from heuristic knobs to **continuous, model-driven policies**. Three pillars: telemetry-first optimizers, policy-driven execution, model-based costing. Teams cut query spend 38%. | 2026 |
| [Analytics Insight](https://www.analyticsinsight.net/artificial-intelligence/how-ai-is-transforming-sql-query-performance-in-2026) | AI2SQL achieved 90% accuracy. **Adaptive query execution** adjusts plans at runtime based on actual data sizes. Schema-aware tools outperform general LLMs on complex queries. | 2026 |
| [Alex Merced / Lakehouse Blog](https://iceberglakehouse.com/posts/2026-04-29-query-engine-05/) | **Cardinality estimation is CBO's Achilles heel**. Correlated columns, stale stats, complex expressions cause 100-1000x plan degradation. Spark AQE switches join strategy mid-flight. | 2026 |
| [arXiv 2409.17136](https://arxiv.org/abs/2409.17136) | **Adaptive Cost Model (ACM)**: lightweight ML models monitor query execution statistics and adjust CPU/IO cost parameters dynamically — no manual DBA tuning. | 2024 |
| [arXiv 2510.20082](https://arxiv.org/html/2510.20082v1) | **Practical QO Trends 2026**: QE+QO collaboration — runtime statistics trigger re-optimization. Adaptive join (SQL Server, Oracle), AQE (Spark/Databricks), BigQuery broadcast switching. | 2025-2026 |
| [Linux Code](https://thelinuxcode.com/costbased-query-planning-building-fast-predictable-sql-in 2026/) | In 2026, workloads are messy: streaming ingest + hybrid OLTP/analytics + AI features on same data. Plans need to adapt without becoming a full-time babysitter. | 2026 |
| [MoldStud](https://moldstud.com/articles/p-unlock-the-power-of-adaptive-query-optimization-top-techniques-tools) | Dynamic index selection based on workload: 30% execution time reduction. ML models predict query performance: 40% improvement. In-memory processing: 50% acceleration for complex queries. | 2025 |

### NEW Defects vs Batch 546

| # | Defect | Detail |
|---|--------|--------|
| **D547-5** | **No Cardinality Estimation for KB Queries** | NeoTrix KB query planner (SQLite) uses static statistics. No `ANALYZE` equivalent runs after experience absorption or skill updates. Query plans degrade silently as data distribution shifts. |
| **D547-6** | **No Adaptive Query Execution** | Batch 546 found token cost quadratic. But also: NeoTrix has no mechanism to switch query strategies at runtime. When KB grows, point lookups that were fast become slow sequential scans — no adaptive join/scan switching exists. |
| **D547-7** | **Policy-Driven Execution Absent** | Modern systems declare SLOs and budget policies that guide pruning, sampling, and materialization. NeoTrix has no query budget system — a single expensive recursive KB query can block all other operations. |
| **D547-8** | **No Cost-to-Recompute in Catalog** | 2026 best practice: annotate datasets with recompute cost. NeoTrix KB has no metadata about how expensive each node/edge is to re-generate, making absorption prioritization purely heuristic. |

---

## 3. Indexing — Learned Index & Vector Index

### Findings

| Source | Key Insight | Year |
|--------|-------------|------|
| [ALEX (Microsoft/Princeton)](https://dl.acm.org/doi/10.1145/3318464.3389711) | **Updatable learned index**: beats B+tree by 4.1x on read-write, 2000x smaller index size. Gapped Array node layout + adaptive RMI structure. Key: model predicts position, not traversal. | 2020 (still foundational) |
| [ACM Cited-By (2026)](https://dl.acm.org/doi/10.1145/3318464.3389711) | **FineStore-SL** (2026): efficient verifiable learning index based on skip list. **Read-optimized learned index** for high-frequency sensor data streams. ML in modern DB: architectures + deployment challenges. | 2026 |
| [Cosmos DB Vector Index](https://learn.microsoft.com/en-us/cosmos-db/index-vector-data) | Three vector index types: `flat` (exact, ≤505 dims), `quantizedFlat` (compressed brute-force, ≤4096 dims), `diskANN` (graph+SSD, ≤4096 dims). Vectors colocated with data — eliminates separate vector DB. | 2026 |
| [ContentWave](https://contentwave.net/article/enterprise-vector-search-at-100m-scale-updated-tradeoffs-june-2026) | At 100M+ vectors: HNSW needs TBs of RAM unless aggressive quantization. **DiskANN-style SSD-first** hybrids are the production answer. IVF+PQ for filtered queries needs attribute-aligned sharding. | 2026 |
| [BestAIWeb ANN Benchmarks](https://www.bestaiweb.ai/scann-diskann-and-glass-the-2026-ann-benchmarks-race-andwhere-vector-indexing-is-heading/) | **Quantization is now mandatory** — every top ANN algorithm depends on it. Graph-vs-clustering debate collapsed. DiskANN v0.49.1 (Rust rewrite): 1B vectors in 64GB RAM, <5ms recall@1. Google ScaNN SOAR: sub-10ms at billion scale. SymphonyQG: least sensitive to query difficulty. | 2026 |
| [Technspire](https://technspire.com/en/blog/vector-search-2026-azure-pgvector-managed) | pgvector HNSW reached production maturity (2024). Under 5M vectors: pgvector wins. 5M-100M: Azure AI Search with hybrid BM25+vector. Over 100M: dedicated platforms. **Market settled in 2026.** | 2026 |
| [npblue.com](https://npblue.com/ai/rag/vector-indexing/) | 2026 trend: **GPU-accelerated indexing** via NVIDIA RAFT (IVF-Flat, IVF-PQ, CAGRA). Build times from hours to minutes for hundreds of millions of vectors. | 2026 |

### NEW Defects vs Batch 546

| # | Defect | Detail |
|---|--------|--------|
| **D547-9** | **No Learned Index on KB** | NeoTrix KB uses plain B-tree (SQLite). At scale, a learned index (ALEX-style) could predict key positions with 2000x smaller index footprint. Current approach wastes memory on full tree traversal for every query. |
| **D547-10** | **Vector Index Not Quantized** | Cosmos DB and 2026 best practice: quantizedFlat or DiskANN for vectors >505 dims. NeoTrix VSA HyperCube embeddings are likely full float32 — no product quantization, no spherical quantization. Memory cost is 4-8x higher than necessary. |
| **D547-11** | **HNSW Memory Wall Ignored** | At 100M vectors × 1536 dims = 600GB raw. HNSW graph edges add 500GB-1TB. NeoTrix has no DiskANN-style SSD-offload strategy. When KB embeddings exceed RAM, performance cliff is imminent. |
| **D547-12** | **No GPU-Accelerated Index Build** | 2026 standard: NVIDIA RAFT for batch ingestion. NeoTrix rebuilds indexes on CPU only. For absorption cycles that ingest thousands of documents, index rebuild time is unmeasured and unoptimized. |
| **D547-13** | **No Hybrid Search (BM25+Vector)** | 2026 consensus: pure vector search underperforms hybrid on enterprise corpora. NeoTrix separates BM25 and vector search — never combines rankings. Acronyms, product codes, specific names (like domain terms) are missed by pure semantic search. |
| **D547-14** | **No Per-Query Recall Observability** | DiskANN v0.49.1 and ScaNN provide per-query recall estimates and dynamic probe tuning. NeoTrix has no way to know if a vector search returned 95% or 60% recall — correctness is assumed, never measured. |
| **D547-15** | **Stale Index After Absorption** | Common LSM/B-tree bug: index not rebuilt after bulk insert. NeoTrix absorption writes to KB but may not trigger `REINDEX` or `ANALYZE`, leaving query optimizer with stale statistics and suboptimal plans. |

---

## Summary: What's NEW vs Batch 546

| Category | Batch 546 Findings | Batch 547 NEW Findings |
|----------|--------------------|------------------------|
| **Agent/Tool** | Tool selection collapse at ~30 tools | (no change — carry forward) |
| **Benchmarking** | Benchmarks measure harness not model | (no change — carry forward) |
| **Token Economics** | Cost grows quadratically with steps | D547-2: fsync cost also grows per KB write |
| **Resilience** | No idempotency in retries | D547-3: tombstone leaking, D547-15: stale index post-absorption |
| **Framework** | AutoGen dead, MS Agent Framework 1.0 | (no change — carry forward) |
| **Storage Engine** | (not covered in 546) | D547-1: RUM ignorance, D547-4: no Bloom filters on KB |
| **Query Optimization** | (not covered in 546) | D547-5: no cardinality estimation, D547-6: no adaptive execution, D547-7: no policy-driven execution, D547-8: no cost-to-recompute metadata |
| **Indexing** | (not covered in 546) | D547-9: no learned index, D547-10: no quantized vectors, D547-11: HNSW memory wall, D547-12: no GPU index build, D547-13: no hybrid search, D547-14: no recall observability |

**Total NEW defects: 15** (D547-1 through D547-15)
**Carried forward from 546: 5** (tool collapse, benchmark mismatch, quadratic tokens, no idempotency, AutoGen dead)

---

## Sources Cited

1. NDLab Blog — Database Engineering Storage Internals 2026 (ndlab.blog)
2. Devesh Shetty — Building a Storage Engine: WAL, LSM Trees, SSTables from Scratch (deveshshetty.com, Feb 2026)
3. SystemDesignSimulator — LSM-Tree Explained (systemdesignsimulator.org, May 2026)
4. Skill Suites — Storage Engine Internals: B-Trees, LSM-Trees & WAL (skillsuites.com, Jun 2026)
5. LSMSharp — LSM-Tree Storage Engine in C# (github.com/Mo7ammedd, 2025+)
6. queries.cloud — The Evolution of Cost-Aware Query Optimization in 2026 (Dec 2025)
7. Analytics Insight — How AI is Transforming SQL Query Performance in 2026 (May 2026)
8. Alex Merced / IcebergLakehouse — Inside the Query Optimizer (Apr 2026)
9. arXiv 2409.17136 — Adaptive Cost Model for Query Optimization (Sep 2024)
10. arXiv 2510.20082 — Query Optimization in the Wild: Realities and Trends (Oct 2025)
11. Linux Code — Cost-Based Query Planning: Building Fast, Predictable SQL in 2026 (Jan 2026)
12. MoldStud — Adaptive Query Optimization Techniques and Tools Guide (Jun 2025)
13. ACM DL 10.1145/3318464.3389711 — ALEX: An Updatable Adaptive Learned Index (SIGMOD 2020)
14. Microsoft Learn — Index Vector Data in Cosmos DB (May 2026)
15. ContentWave — Vector Search at 100M+ Scale (Jul 2026)
16. BestAIWeb — ScaNN, DiskANN, and Glass: The 2026 ANN-Benchmarks Race (Mar 2026)
17. Technspire — Vector Search 2026: Azure AI Search vs pgvector vs Pinecone (May 2026)
18. npblue.com — Vector Indexing: HNSW, IVF, and DiskANN (Updated 2026)
19. CallSphere — Vector Index Algorithms Compared: HNSW, IVF, ScaNN, DiskANN (Jun 2026)
