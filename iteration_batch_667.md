# Iteration 667 — Search/IR/RAG Defect Mining

**Date**: 2026-09-06  
**Batch**: 667/10000  
**Context**: Post-666 (S3-FIFO > LRU, XFetch stampede, cache invalidation dual-write, zero CDN)

---

## Source: Search Engine Landscape 2026

### Finding 1: BM25 Outperforms Dense Retrieval on Financial Documents
- **Source**: Strich et al. (EACL 2026) — arxiv:2604.01733
- **Evidence**: On T2-RAGBench (23,088 queries, 7,318 financial docs with mixed text+tables), BM25 outperformed text-embedding-3-large on every metric except Recall@20
- **Metrics**: BM25 Recall@5=0.644 vs Dense=0.587; BM25 nDCG@10=0.515 vs Dense=0.466
- **DEFECT D-667-1**: NeoTrix `nt_memory` KB search uses dense-only embedding retrieval. On structured/financial/tabular data, this underperforms BM25 by 8-11pp. No BM25 path exists for keyword-exact queries.

### Finding 2: Hybrid Search + RRF is Now Table Stakes
- **Source**: Digital Applied, WANDS benchmark; DenserAI 2026 guide
- **Evidence**: Hybrid RRF achieves NDCG 0.7497 (7.4% lift over either BM25 0.6983 or dense 0.6953 alone on e-commerce)
- **DEFECT D-667-2**: NeoTrix KB has no hybrid retrieval mode. Single embedding search misses exact-match signals. RRF fusion (k=60) is not implemented.

### Finding 3: Cross-Encoder Reranking is Highest-Leverage Upgrade
- **Source**: 1337skills production RAG guide; AppScale hybrid search deep dive
- **Evidence**: Cross-encoder reranking on top-50-200 candidates consistently cited as single highest-ROI improvement. Cohere Rerank v3.5 vs Voyage rerank-2.5: +7.9% gain. Qwen3-Reranker (open-source) is now competitive.
- **DEFECT D-667-3**: NeoTrix has no reranking stage. Retrieval results are final once retrieved. No second-stage neural reranker integrated.

### Finding 4: Adaptive Re-Ranking Depth
- **Source**: SIGIR '26, ICTIR '26 — "Adaptive Re-Ranking" (ACM 10.1145/3805713.3820436)
- **Evidence**: Fixed reranking depth (e.g., always top-100) is suboptimal. Adaptive depth based on query difficulty improves efficiency without sacrificing quality.
- **DEFECT D-667-4**: NeoTrix (if reranking were added) would use fixed depth. No query-difficulty-based adaptive reranking depth.

### Finding 5: Q2Q Query Translation for Agentic Search
- **Source**: Meng et al., SIGIR '26 — "Revisiting Text Ranking in Deep Research" (arxiv:2602.21456)
- **Evidence**: Agent-issued queries mismatch natural document language. Q2Q (query-to-question) translation significantly reduces query mismatch in deep research scenarios.
- **DEFECT D-667-5**: NeoTrix `nt_mind` generates queries that may not match document vocabulary. No Q2Q translation layer.

### Finding 6: Passage-Level > Document-Level Retrieval
- **Source**: Same SIGIR '26 paper
- **Evidence**: Passage retrieval outperforms document retrieval in deep research — more efficient under context-window budgets, avoids length normalization issues, better granularity for neural retrievers.
- **DEFECT D-667-6**: NeoTrix KB operates on document-level nodes. No passage-level indexing or retrieval.

---

## Source: RAG Production 2026

### Finding 7: GraphRAG for Cross-Document Reasoning
- **Source**: 1337skills; Microsoft GraphRAG
- **Evidence**: Single-chunk retrieval cannot answer questions requiring synthesis across documents. GraphRAG adds graph-traversal retrieval alongside chunk-based path.
- **DEFECT D-667-7**: NeoTrix KB has edges (relations) but no graph-based retrieval path. Queries requiring multi-hop reasoning across KB nodes have no dedicated retrieval mechanism.

### Finding 8: Naive RAG Fails 40% in Production
- **Source**: Lushbinary RAG guide 2026
- **Evidence**: Naive embed-and-retrieve RAG fails 40% of the time at retrieval in production. Three structural reasons: dense misses exact terms, bad ordering, no cross-document reasoning.
- **DEFECT D-667-8**: NeoTrix `nt_memory` search is effectively naive RAG — single embedding pass, no reranking, no hybrid. Expected ~40% retrieval failure rate on non-trivial queries.

### Finding 9: Chunking Quality Caps Pipeline Performance
- **Source**: 1337skills production RAG guide
- **Evidence**: Naive fixed-N-character chunking destroys sentence/idea coherence. Structure-aware chunking (headings, paragraphs, semantic boundaries with overlap) pays dividends through entire pipeline. Bad chunking caps everything downstream.
- **DEFECT D-667-9**: NeoTrix KB ingestion has no configurable chunking strategy. Documents are ingested as atomic nodes with no semantic chunking or overlap.

### Finding 10: Agentic RAG — Model Decides Retrieval Strategy
- **Source**: 1337skills; SignitySolutions
- **Evidence**: Instead of fixed retrieve→rerank→generate, agentic RAG lets the model decide: whether to retrieve, what to search, whether results are sufficient, which mode (vector/keyword/graph) suits the query.
- **DEFECT D-667-10**: NeoTrix search is a single fixed-path call. No adaptive retrieval strategy selection based on query characteristics.

### Finding 11: SPLADE Learned Sparse Retrieval
- **Source**: Digital Applied hybrid search reference
- **Evidence**: SPLADE maps text to 30K+ dimensional sparse vectors, outperforms BM25 on BEIR benchmarks. Requires GPU inference (unlike CPU-only BM25 inverted index).
- **DEFECT D-667-11**: NeoTrix has no sparse learned retrieval path. BM25 is not implemented. SPLADE not considered.

---

## Source: Search Engine Comparisons 2026

### Finding 12: Meilisearch Built-In Hybrid Search
- **Source**: APIScout 2026; Meilisearch docs
- **Evidence**: Meilisearch now has built-in vector search + hybrid (semantic + keyword) in a single query — no external vector DB required. Sub-50ms responses with zero config.
- **DEFECT D-667-12**: NeoTrix KB search is custom-built with no external search engine integration. Meilisearch or Typesense could provide hybrid search out-of-box.

### Finding 13: Vespa First-Class Tensor Ranking
- **Source**: youngju.dev search engines deep dive
- **Evidence**: Vespa treats tensors as first-class citizens. Embeddings + multi-dimensional tensors in index. ONNX/TF models run inside ranking step. First-phase/second-phase/global-phase ranking pipelines. ColBERTv2 late-interaction is native.
- **DEFECT D-667-13**: NeoTrix has no ML-model-in-the-loop ranking. Ranking is purely score-based with no learned ranking function.

### Finding 14: ParadeDB — Postgres-Native Hybrid
- **Source**: youngju.dev search engines deep dive
- **Evidence**: ParadeDB = Postgres + pg_search + pgvector. Metadata, full-text, and vectors all in one DB. Under 100K docs, ParadeDB or Meilisearch usually suffices for RAG.
- **DEFECT D-667-14**: NeoTrix KB (SQLite-backed) lacks vector search + FTS fusion. ParadeDB model shows this can be unified in one engine.

---

## Defect Summary

| ID | Category | Severity | Description |
|----|----------|----------|-------------|
| D-667-1 | Retrieval | HIGH | Dense-only KB search; BM25 outperforms on structured/financial data by 8-11pp |
| D-667-2 | Retrieval | HIGH | No hybrid retrieval (BM25 + dense via RRF). Missing 7.4% NDCG lift |
| D-667-3 | Ranking | HIGH | No cross-encoder reranking stage. Highest-leverage upgrade absent |
| D-667-4 | Ranking | MEDIUM | No adaptive reranking depth based on query difficulty |
| D-667-5 | Query | MEDIUM | No Q2Q query-to-question translation for agent-issued queries |
| D-667-6 | Indexing | HIGH | Document-level only; no passage-level indexing or retrieval |
| D-667-7 | Retrieval | HIGH | No graph-based retrieval for multi-hop cross-document reasoning |
| D-667-8 | System | CRITICAL | Naive RAG pattern — expected ~40% retrieval failure rate |
| D-667-9 | Ingestion | MEDIUM | No configurable semantic chunking strategy |
| D-667-10 | Retrieval | MEDIUM | Fixed single-path retrieval; no adaptive strategy selection |
| D-667-11 | Retrieval | LOW | No learned sparse retrieval (SPLADE); no BM25 implementation |
| D-667-12 | Architecture | MEDIUM | Custom KB search vs using Meilisearch/Typesense with built-in hybrid |
| D-667-13 | Ranking | MEDIUM | No ML-model-in-the-loop ranking (Vespa-style tensor ranking) |
| D-667-14 | Storage | MEDIUM | SQLite KB lacks vector+FTS fusion (ParadeDB model) |

---

## Sources Cited

1. Strich et al. (2026). "From BM25 to Corrective RAG: Benchmarking Retrieval Strategies for Text-and-Table Documents." arXiv:2604.01733, EACL 2026.
2. Meng et al. (2026). "Revisiting Text Ranking in Deep Research." SIGIR '26, arXiv:2602.21456.
3. ICTIR '26. "Adaptive Re-Ranking." ACM 10.1145/3805713.3820436.
4. Digital Applied (2025/2026). "Hybrid Search: BM25, Vector & Reranking Reference 2026."
5. 1337skills (2026). "Production RAG in 2026: Hybrid Search, Reranking, and GraphRAG."
6. Lushbinary (2026). "RAG in 2026: The Complete Production Guide."
7. youngju.dev (2026). "Self-Hosted Search Engines 2026 Deep Dive."
8. APIScout (2026). "Elasticsearch vs Meilisearch: When to Use Which 2026."
9. DenserAI (2026). "Hybrid Search for RAG: Combining BM25 and Dense Vector Search."
10. AppScale (2026). "Hybrid Search and Re-ranking in Production RAG 2026."

---

## What's NEW vs Batch 666

| Dimension | 666 | 667 |
|-----------|-----|-----|
| Search paradigm | (not covered) | Hybrid BM25+dense is 2026 default; NeoTrix has neither |
| Ranking | (not covered) | Cross-encoder reranking is highest-leverage upgrade; absent |
| Retrieval depth | (not covered) | Adaptive reranking depth (SIGIR '26) not considered |
| Query processing | (not covered) | Q2Q translation for agent queries missing |
| Index granularity | (not covered) | Document-level only; passage-level retrieval absent |
| Cross-doc reasoning | (not covered) | No GraphRAG or graph traversal retrieval |
| Production RAG failure | (not covered) | 40% failure rate for naive RAG; NeoTrix is naive RAG |
| Chunking | (not covered) | No semantic chunking strategy |
| Retrieval strategy | (not covered) | Fixed single-path; no adaptive strategy selection |
| Learned sparse | (not covered) | No SPLADE or BM25 implementation |
| Engine integration | (not covered) | Custom search vs Meilisearch/ParadeDB with built-in hybrid |
