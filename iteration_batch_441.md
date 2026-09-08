# Iteration Batch 441 — External Research: Vector DB / Dense Retrieval / Hybrid Search (2026-09-06)

## Sources Cited

### Vector Databases
1. **Milvus 3.0** (Jul 2026) — Lake-native architecture, External Collections (Parquet/Lance/Iceberg/Vortex), StructArray for late-interaction ColBERT vectors, SINDI sparse algorithm (10x QPS vs MaxScore), server-side ORDER BY/aggregation/facets, Entity TTL, MinHash DIDO, nullable vectors, Function Chain reranking with XGBoost L0 scoring.
   - https://milvus.io/blog/announcing-milvus-3-lake-native-vector-search-and-a-more-powerful-retrieval-engine.md
2. **Qdrant 1.19** (Aug 2026) — TurboQuant 4-bit datatype (9x storage reduction, no full-precision copy), unified `memory` parameter (pinned/cached/cold tiers), per-tenant IDF for multi-tenant BM25, prefix matching on keyword fields, slice filtering, ACORN filtered vector search with query planner routing.
   - https://qdrant.tech/blog/qdrant-1.19.x/
3. **Qdrant 1.18** (May 2026) — TurboQuant (Hadamard rotation + 2x compression over SQ), memory monitoring per component, audit logging improvements.
   - https://qdrant.tech/blog/qdrant-1.18.x/
4. **Weaviate 1.39** (Aug 2026) — MMR diversity selection GA on hybrid+vector, Boost API GA for query-time rescoring, 4-bit Rotational Quantization (preview), Search REST API (experimental), gRPC-Web, automatic HNSW snapshots, namespaces, cross-property keyword AND.
   - https://weaviate.io/blog/weaviate-1-39-release
5. **Weaviate 1.38** (Jun 2026) — HFresh disk-based vector index (SPFresh-inspired) GA, MCP Server GA, async replication rebuilt cluster-wide, Boost API preview, nested object filtering.
   - https://weaviate.io/blog/weaviate-1-38-release
6. **ByteX (ByteDance)** (Aug 2026) — SymRaBitQ quantization-aware vector kernel, hybrid storage (memory/hybrid/SSD-resident), trillion-vector scale, 80% index memory reduction, 86% cost reduction. arXiv:2608.30607.
7. **Vextra Middleware** (Jan 2026) — Unified vector DB API middleware addressing API fragmentation.
8. **HoneyBee** (Jan 2026) — Dynamic RBAC-aligned vector partitioning, 6x query latency reduction.
9. **PostgreSQL-V 2.0** (2026) — Decoupled vector index inside PostgreSQL, 36.4x throughput over v1.0, 20ms crash recovery independent of index size, physical replication support.
10. **AWS S3 Vectors** (2026) — Object-storage vector search, 2B vectors per index, 90% TCO reduction, >100ms latency trade-off.
11. **pgvectorscale benchmarks** — 471 QPS vs Qdrant 41 QPS at 99% recall on 50M vectors (PostgreSQL extension).

### Dense Retrieval & Late Interaction
12. **DenseOn + LateOn** (Jul 2026) — Open recipe: DenseOn (single-vector, 56.20 nDCG@10 on BEIR), LateOn (ColBERT-style, 57.22 nDCG@10), both 149M params. Late-interaction generalizes better to unseen languages. arXiv:2607.27178.
13. **ColBERT-Att** (Mar 2026) — Explicit attention weights in MaxSim for term importance, +5% nDCG@10 on Quora via attention regularizer. arXiv:2603.25248.
14. **Sentence Transformers v6.0** (Aug 2026) — MultiVectorEncoder (ColBERT-style) native support, loads PyLate/Stanford-NLP checkpoints, integrates with existing dense/sparse/reranker API.
15. **MUVERA** (NeurIPS 2024, cited in 2026) — Fixed Dimensional Encodings approximating MaxSim with epsilon guarantee, 2-5x fewer candidates, 10% better recall at 90% lower latency.
16. **Late Interaction Dynamics** (Mar 2026) — Causal multi-vector models suffer strict monotonic length bias; bi-directional models mitigate but not eliminate at extremes. arXiv:2603.26259.
17. **TurkColBERT** (2026) — 1M-param ColBERT model at 600x smaller than 600M dense encoder retains 71% mAP. MUVERA+Rerank 3.33x faster than PLAID with +1.7% mAP.
18. **Vector Interaction Architectures Survey** (Jul 2026) — 25-cell design grid (document encoder output × interaction operator), production pipelines use cascaded stages: cheap high-recall retriever → progressive compute spend.

### Hybrid Search & Fusion
19. **Balancing the Blend (VLDB 2026)** — First systematic analysis of 4 retrieval paradigms (FTS/SVS/DVS/Tensor) × all combinations × 3 reranking strategies across 11 datasets. Key: "weakest link" phenomenon degrades fusion; Tensor-based Re-ranking Fusion (TRF) outperforms RRF.
20. **From BM25 to Corrective RAG** (Apr 2026) — Hybrid+reranker Recall@5=0.816 dominates all single-stage. BM25 outperforms dense on financial docs. Convex Combination α=0.5 beats RRF k=60 on this benchmark (0.726 vs 0.695).
21. **EAHR (Exact Adaptive Hybrid Retrieval)** (Aug 2026) — Eliminates fixed Top-L cutoff; PVS+PBM generate resumable exact rankings; complete-list weighted RRF as target. Up to 30x speedup with identical results.
22. **QARF (Query Adaptive Rank Fusion)** (Jul 2026) — Training-free per-query weighting from IDF statistics; +1.74 to +8.61 nDCG@10 over equal-weight RRF on BEIR.
23. **DESA (Dense Expansion + Sparse Anchoring)** (Aug 2026) — Channel-asymmetric query expansion: LLM-generated passages expand dense semantically while anchoring sparse lexically. +3.82% nDCG@10, -36.9% access depth.
24. **Hybrid Search Reference 2026** (May 2026) — Weaviate changed default fusion from RRF to Relative Score Fusion in v1.24. Qdrant added server-side RRF in v1.10. BM25+Dense RRF baseline: 0.7068 NDCG on WANDS, tuned hybrid: 0.7497.

---

## Defects Found in NeoTrix KB/Retrieval Design

### DEF-441-01: No Late-Interaction (ColBERT) Support in KB Pipeline
**Severity**: High | **Domain**: NT-MEMORY + NT-CORE
**Finding**: Milvus 3.0 now ships StructArray with native ColBERT/ColPali support (EmbList + DISKANN + Muvera/Lemur acceleration). Weaviate 1.39 supports ColBERT via HFresh. Qdrant 1.19's Turbo4 datatype explicitly optimizes multi-vector collections for late-interaction. NeoTrix KB uses only single-vector dense embeddings + BM25. The VSA HyperCube architecture maps concepts to high-dimensional vectors but cannot represent token-level interactions.
**Impact**: On multi-aspect queries (common in code search, multi-hop reasoning), NeoTrix's bi-encoder KB retrieval hits the Eckart-Young rank bound and cannot represent relevance matrices with rank > k.
**Suggestion**: Add a `ColBERTLateIndex` variant to NT-MEMORY KB. Use Sentence Transformers v6.0's MultiVectorEncoder with PyLate checkpoints. Store token-level embeddings via StructArray (Milvus) or per-token vectors. MUVERA can project back to single-vector for first-stage, with MaxSim as reranker. This aligns with the 2026 production pattern: hybrid BM25+dense as first stage → late-interaction reranking.

### DEF-441-02: No Lake-Native / Zero-Copy Index Architecture
**Severity**: Medium | **Domain**: NT-MEMORY
**Finding**: Milvus 3.0 External Collections search Parquet/Lance/Iceberg/Vortex without copying data. Loon storage engine reduces point-read amplification. Weaviate 1.37 Collection Export to Parquet exists. NeoTrix KB is a monolithic SQLite store that requires all data to be ingested into its own format.
**Impact**: When NeoTrix crawls external data (NT-WORLD), it must copy everything into KB, doubling storage costs and creating ETL sync burden. Data governance stays with NeoTrix, not the source.
**Suggestion**: Design a `LakeView` adapter in NT-MEMORY that can reference external Parquet/Lance files and build vector indexes over them in-place. Use Milvus External Collections as the retrieval backend for large-scale NT-WORLD crawl results, keeping only metadata and indexes in SQLite.

### DEF-441-03: No Per-Tenant IDF for Multi-Tenant BM25
**Severity**: Medium | **Domain**: NT-MEMORY
**Finding**: Qdrant 1.19 adds per-tenant IDF statistics — narrow the IDF corpus to a specific tenant so term rarity reflects that tenant's vocabulary. NeoTrix BM25 index uses global IDF across all domains/factions.
**Impact**: In multi-tenant or multi-domain scenarios (e.g., NT-SHIELD security docs vs NT-ACT code docs), common terms in one domain become rare in another. Global IDF misranks results.
**Suggestion**: Implement tenant-scoped IDF in the BM25 index. Partition the FTS5 index by domain namespace (`domain_nt_*`) and compute IDF within each partition. This is a small structural change to the existing SQLite FTS5 layer.

### DEF-441-04: No Server-Side Reranking / Rescoring Pipeline
**Severity**: High | **Domain**: NT-MEMORY + NT-CORE (GWT attention routing)
**Finding**: Milvus 3.0 Function Chain API chains L0 rescoring + L2 reranking in a single search request (XGBoost L0, HuggingFace L2). Weaviate Boost API (GA) promotes/demotes without dropping. Weaviate MMR (GA) diversifies results. VLDB 2026 TRF outperforms RRF. NeoTrix has no server-side reranking — all post-processing is client-side.
**Impact**: GWT attention routing receives raw retrieval scores without relevance refinement. The "weakest link" phenomenon (VLDB 2026) means a weak retrieval path degrades all downstream cognition.
**Suggestion**: Add a `RerankStage` in the KB pipeline between retrieval and GWT attention. Support: (1) Cross-encoder reranking (Cohere Rerank / local BGE-reranker), (2) MMR diversity selection for GWT broadcasting, (3) Boost API for domain-specific score modulation. This routes through the Function Chain pattern.

### DEF-441-05: Static Fusion Weights in Hybrid Search
**Severity**: Medium | **Domain**: NT-MEMORY
**Finding**: NeoTrix hybrid search (if implemented) likely uses fixed α weighting or basic RRF with k=60. QARF (2026) shows training-free per-query adaptive weighting from IDF stats improves +1.74 to +8.61 nDCG@10. DESA shows channel-asymmetric expansion reduces access depth 36%. VLDB 2026 shows no one-size-fits-all fusion.
**Impact**: Fixed fusion fails on queries that are either heavily lexical (codes, IDs) or heavily semantic (paraphrase). The system cannot adapt fusion balance per query.
**Suggestion**: Implement QARF: compute per-query IDF distribution at query time, select regime (per-query weighting vs corpus-level recalibration) automatically. For DESA-style expansion, use the existing SEAL pipeline to generate reference passages, then apply orthogonal residual expansion (dense) + score-product anchoring (sparse).

### DEF-441-06: No Adaptive Hybrid Retrieval (Fixed Top-L Cutoff)
**Severity**: Medium | **Domain**: NT-MEMORY
**Finding**: EAHR (2026) eliminates fixed Top-L cutoff by treating channel depth as execution state. Complete-list weighted RRF as result contract. PVS+PBM produce resumable exact rankings. NeoTrix likely uses fixed Top-K for both BM25 and dense retrievers.
**Impact**: Fixed cutoff either wastes compute (too deep) or misses results (too shallow). The 2026 finding shows this can be eliminated with bound-based stopping.
**Suggestion**: Implement EAHR-style adaptive retrieval: (1) Define complete-list RRF as the target, (2) Use PVS (Per-Vector Scalar Quantization) for dense rank generation with upper-bound pruning, (3) Use PBM (Posting Block-Max) for sparse rank generation, (4) Coordinate both under snapshot isolation for consistent results.

### DEF-441-07: No 4-Bit Quantization for Vector Storage
**Severity**: Low-Medium | **Domain**: NT-MEMORY
**Finding**: Qdrant Turbo4: 9x storage reduction, no full-precision copy. Weaviate RQ4: 7.84x compression. ByteX SymRaBitQ: 80% memory reduction at trillion scale. NeoTrix uses full float32 embeddings.
**Impact**: KB embedding storage grows linearly with crawl volume. At 100M vectors × 1536 dims × 4 bytes = 614 GB. With 4-bit: ~69 GB. The cost difference is material for edge/on-prem deployments (NT-PHYSICAL).
**Suggestion**: Add TurboQuant/RQ4-style quantization as an option in KB embedding storage. Support both modes: (a) 4-bit with rescoring (Qdrant TurboQuant v1.18 path), (b) 4-bit-only (Qdrant Turbo4 v1.19 path) for maximum compression. Make this configurable per collection via Rune Socketing (Crimson rune for data ingestion compression).

### DEF-441-08: No Entity TTL / Compliance Expiry
**Severity**: Medium | **Domain**: NT-MEMORY + NT-SHIELD
**Finding**: Milvus 3.0 Entity TTL: per-entity TIMESTAMPTZ field, automatic expiry + garbage collection. NeoTrix KB has no built-in entity expiry mechanism.
**Impact**: Crawl data accumulates indefinitely. Right-to-be-forgotten compliance, session data expiry, and bounded conversation history require manual cleanup. NT-SHIELD audit trail grows unbounded.
**Suggestion**: Add `expires_at` field to KB nodes. Implement TTL-aware compaction in KB pipeline. For NT-WORLD crawl data, auto-expire by default. For NT-NEXUS experience data, use longer TTL with manual override. Expiry runs during SEAL pipeline cycles.

### DEF-441-09: No MinHash Deduplication in Crawl Pipeline
**Severity**: Low-Medium | **Domain**: NT-WORLD + NT-MEMORY
**Finding**: Milvus 3.0 MinHash DIDO: server-side MinHash generation + MINHASH_LSH index for deduplication. NeoTrix crawl pipeline has no deduplication mechanism.
**Impact**: NT-WORLD crawls duplicate content across sources. KB storage and embedding compute waste on near-duplicate documents. Search results include redundant entries.
**Suggestion**: Add MinHash signature generation in NT-WORLD ingest path. Use MINHASH_LSH for near-duplicate detection before KB write. Dedup threshold configurable per domain (e.g., strict for NT-SHIELD audit logs, relaxed for NT-WORLD knowledge).

### DEF-441-10: No Disk-Resident Vector Index (HFresh/Streaming)
**Severity**: Low | **Domain**: NT-MEMORY + NT-PHYSICAL
**Finding**: Weaviate HFresh (GA): SPFresh-inspired disk-based index for streaming workloads, low memory with predictable latency. Qdrant supports mmap-based segments. NeoTrix KB loads all embeddings into RAM.
**Impact**: For edge deployments (NT-PHYSICAL) or very large corpora, full RAM loading is infeasible. AWS S3 Vectors shows 90% TCO reduction with >100ms latency trade-off.
**Suggestion**: For NT-PHYSICAL edge deployments, implement HFresh-style disk-resident indexing. For cloud deployments, support mmap-backed segments (Qdrant model). Add a `StorageTier` configuration per collection: `full_ram`, `mmap_cached`, `disk_resident` — aligning with Qdrant's pinned/cached/cold tiers.

---

## Summary

| # | Defect | Severity | Source |
|---|--------|----------|--------|
| 441-01 | No late-interaction (ColBERT) support | High | Milvus 3.0, Weaviate 1.39, DenseOn/LateOn |
| 441-02 | No lake-native zero-copy index | Medium | Milvus 3.0 External Collections |
| 441-03 | No per-tenant IDF | Medium | Qdrant 1.19 |
| 441-04 | No server-side reranking pipeline | High | Milvus 3.0 Function Chain, Weaviate Boost/MMR, VLDB 2026 TRF |
| 441-05 | Static fusion weights | Medium | QARF, DESA, VLDB 2026 |
| 441-06 | Fixed Top-L cutoff in hybrid | Medium | EAHR 2026 |
| 441-07 | No 4-bit quantization | Low-Med | Qdrant Turbo4, Weaviate RQ4, ByteX SymRaBitQ |
| 441-08 | No entity TTL | Medium | Milvus 3.0 Entity TTL |
| 441-09 | No MinHash deduplication | Low-Med | Milvus 3.0 MinHash DIDO |
| 441-10 | No disk-resident vector index | Low | Weaviate HFresh, Qdrant mmap, AWS S3 Vectors |

**Top Priority**: DEF-441-01 (ColBERT) and DEF-441-04 (Server-side reranking) — these directly bottleneck GWT attention routing quality and align with the 2026 consensus that hybrid first-stage + late-interaction/cross-encoder reranking is the production-standard architecture.
