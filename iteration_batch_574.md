# Iteration Batch 574 — Search Engine / IR / Ranking Research

## Context
Batch 573 proved: (1) twin model drift without formal state spec, (2) FMI 3.0 causality loops unresolved, (3) no formal verification of domain randomization coverage, (4) sim-to-real policy degradation detection absent, (5) formal specs are universal structural gap in AI-native systems.

Batch 574 focuses on **search engine architecture, information retrieval fusion, and ranking algorithms** in 2026 — probing whether NeoTrix's NT-MEMORY (KB search), NT-WORLD (crawl retrieval), and NT-ACT (tool routing) share the same formal-spec deficits found in Batch 573.

---

## 1. Search Engines (2026)

### 1.1 Meilisearch v1.36–1.38 — Replicated Sharding + Vector Store Migration
- **Source**: https://www.meilisearch.com/blog/March-2026-updates (2026-03-12)
- **Finding**: Meilisearch shipped replicated sharding directly in-engine (v1.37), eliminating the single-node ceiling for community edition. v1.38 stabilized HNSW-backed vector store (Hannoy), removed legacy `vectorStoreSetting`, and eliminated full database scans for embedding updates.
- **NeoTrix defect**: NeoTrix's KB uses SQLite FTS5 + sqlite-vec. There is **no formal migration path specification** for when vector store backends change (e.g., HNSW → PQ → DiskANN). The KB schema versioning exists but no **contract** guarantees embedding compatibility across engine upgrades. → **DEFECT-574-1: No vector store backend migration contract.**

### 1.2 Elasticsearch 9.5 — Columnar Mode + VectorDB Auto-Calibration
- **Source**: https://www.elastic.co/blog/whats-new-elastic-9-5-0 (2026-08-04)
- **Finding**: Elastic 9.5 introduced Columnar Mode (column store with no inverted index by default, 20% storage reduction) and VectorDB index mode with auto-calibration for DiskBBQ quantization. Auto-calibration uses statistical analysis of indexed vectors to configure quantization depth, preconditioning, and oversampling automatically.
- **NeoTrix defect**: NeoTrix has no equivalent auto-calibration for its vector embeddings. BM25 parameters (k1, b) and vector quantization settings are manually configured. There is **no formal feedback loop** that observes query performance and auto-tunes retrieval parameters. → **DEFECT-574-2: No auto-calibration for retrieval parameters (BM25 k1/b, vector quantization depth).**

### 1.3 Meilisearch vs Elasticsearch — Scale Ceiling Formalization
- **Source**: https://apiscout.dev/guides/elasticsearch-vs-meilisearch-api-2026 (2026-03-08)
- **Finding**: Meilisearch: sub-50ms latency, ~50-100MB idle RAM, single-node, LMDB storage. Elasticsearch: 1-2GB JVM heap minimum, distributed sharding. Decision matrix: <10M docs → Meilisearch, >100M → Elasticsearch. The "weakest link" is operational: teams use Elasticsearch for application search because they already run it for logs.
- **NeoTrix defect**: NeoTrix's NT-MEMORY KB has no **formal capacity planning model** — no specification of when to migrate from single-node SQLite to distributed search. The KB growth trajectory is untracked against any capacity boundary. → **DEFECT-574-3: No formal KB capacity model or migration trigger specification.**

---

## 2. Information Retrieval (2026)

### 2.1 Unified Bayesian Framework for Hybrid Search (Jeong, 2026)
- **Source**: https://doi.org/10.5281/zenodo.20768748 (2026-06-12)
- **Finding**: Bayesian BM25 transforms raw BM25 scores into calibrated probabilities via sigmoid likelihood + corpus-level base rate prior, reducing expected calibration error by 68-77% without relevance labels. Fusion is additive in log-odds space: each signal's evidence sums, prior added once. Preserves WAND/BMW safe pruning with modified upper bounds.
- **NeoTrix defect**: NeoTrix's KB search combines BM25 and vector similarity via ad-hoc weighted sum or RRF with no probabilistic calibration. Scores from different signals are on incompatible scales and fused without calibration. → **DEFECT-574-4: Hybrid search fusion lacks probabilistic calibration — BM25 and vector scores fused without log-odds transformation.**

### 2.2 Query-Adaptive Hybrid Search (Posokhov et al., 2026)
- **Source**: https://doi.org/10.3390/make8040091 (2026-04-05)
- **Finding**: Adaptive hybrid retrieval with query-driven alpha prediction dynamically calibrates sparse/dense mixing weights per query. Antagonist negative sampling trains dense encoder to resolve BM25's systematic failures. Achieves 92.5% of oracle performance, nDCG@10 of 74.3 on long-document retrieval across 16 languages.
- **NeoTrix defect**: NeoTrix's hybrid search uses static mixing weights (vec=0.5, fts=0.5 or similar). No per-query adaptation. No training signal from retrieval disagreements. → **DEFECT-574-5: Static hybrid mixing weights — no query-adaptive alpha prediction.**

### 2.3 "Weakest Link" Phenomenon in Multi-Path Search (VLDB 2026)
- **Source**: https://www.vldb.org/pvldb/vol19/p1715-gao.pdf (2026)
- **Finding**: A weak retrieval path can substantially degrade overall hybrid accuracy. Optimal configurations depend on resource constraints and data characteristics — no one-size-fits-all. Tensor-based Re-ranking Fusion (TRF) outperforms RRF at fraction of cost of full tensor search.
- **NeoTrix defect**: NeoTrix does not assess per-path quality before fusion. If one retrieval path (e.g., FTS) returns garbage, it poisons the fused result. No path-wise quality gate exists. → **DEFECT-574-6: No per-path quality assessment before hybrid fusion — weakest link unguarded.**

### 2.4 Exact Adaptive Hybrid Retrieval (EAHR, 2026)
- **Source**: https://arxiv.org/html/2608.07152v1 (2026)
- **Finding**: EAHR fixes complete-list weighted RRF as retrieval target, treats channel depth as execution state. Eliminates fixed Top-L cutoff problem. Achieves 23-30x speedup over exhaustive batch execution while reproducing exact complete-list Top-20 in all 150 query-snapshot combinations. Fixed Top-L depths from historical queries do not transfer reliably.
- **NeoTrix defect**: NeoTrix's KB search uses fixed Top-K retrieval depth. No adaptive depth mechanism. No formal guarantee that fixed K produces equivalent results to complete-list fusion. → **DEFECT-574-7: Fixed retrieval depth K — no formal equivalence guarantee to complete-list fusion.**

### 2.5 vstash: Local-First Hybrid with Self-Supervised Refinement (2026)
- **Source**: https://arxiv.org/abs/2604.15484 (2026)
- **Finding**: 74.5% of queries produce top-10 disagreement between vector-heavy and FTS-heavy search. This disagreement is a free training signal (no human labels). Fine-tuning BGE-small (33M params) on 76K disagreement triples matches or exceeds ColBERTv2 (110M params) on 3/5 BEIR datasets. Negative result: post-RRF scoring strategies all failed.
- **NeoTrix defect**: NeoTrix does not harvest retrieval disagreement as training signal. The KB embeds documents once and never refines embeddings based on retrieval failures. → **DEFECT-574-8: No self-supervised embedding refinement from retrieval disagreement signals.**

### 2.6 BM25 Outperforms Dense on Financial Documents (2026)
- **Source**: https://arxiv.org/html/2604.01733v1 (2026-04-02)
- **Finding**: On T2-RAGBench (23K queries, 7.3K docs with text+tables), BM25 outperforms text-embedding-3-large on every metric except Recall@20. Domain-specific terminology (ticker symbols, metric labels) is captured by lexical matching, not semantic search. Hybrid + cross-encoder reranking achieves Recall@5=0.816 (+39% over dense alone).
- **NeoTrix defect**: NeoTrix assumes vector search is always superior to BM25 for knowledge retrieval. No domain-aware retrieval strategy selection. Financial/technical domains with precise terminology may need lexical-first retrieval. → **DEFECT-574-9: No domain-aware retrieval strategy — assumes vector always dominates BM25.**

---

## 3. Ranking Algorithms (2026)

### 3.1 SlimPer — Iterative Refinement for Personalized Ranking (Meta, 2026)
- **Source**: https://arxiv.org/html/2607.12281v1 (2026-07-14)
- **Finding**: SlimPer reformulates personalized ranking as iterative refinement over compact <user,item> knowledge base. O(N) per-layer cost, fixed-size intermediate representation. Model depth decoupled from user history length. Request-only optimization shares user-side tokens across all candidates. Deployed on Instagram Reels and Feed, enables 10k+ event modeling.
- **NeoTrix defect**: NeoTrix's tool routing (NT-ACT) uses static capability matching. No iterative refinement of user-task relevance. No O(N) fixed-cost ranking model. → **DEFECT-574-10: Tool routing lacks iterative refinement — static match vs. O(N) adaptive ranking.**

### 3.2 GenRec — LLM-Backed Recommendation Ranker (Netflix, 2026)
- **Source**: https://arxiv.org/html/2608.10257v1 (2026)
- **Finding**: Netflix deploys LLM-backed ranker using prefill-only configuration (no autoregressive decoding). Phase 1 adapts OSS LLM to Netflix data; Phase 2 post-trains with ranking-specific labels + multiple reward signals. Achieves significant improvement over production ranker on both short-term and long-term metrics.
- **NeoTrix defect**: NeoTrix's LLM tool selection (NT-ACT gateway) does not use reward-weighted ranking. No Phase 1/Phase 2 adaptation for tool selection. No long-term satisfaction signal in tool routing. → **DEFECT-574-11: Tool selection lacks reward-weighted ranking and long-term adaptation.**

### 3.3 TGR — Generative Recommendation with Reasoning (Tencent, 2026)
- **Source**: https://arxiv.org/abs/2609.00986 (2026-09-01)
- **Finding**: TGR combines unified feature tokenization, feature-field separated cross attention, hierarchical sequence compression, and end-to-end generation. Deployed across Tencent surfaces serving hundreds of millions. CCFormer: +3.57% CTR, +1.71% ad revenue. Cold-start new-user Hit@1 +477.8% via reasoning tokens.
- **NeoTrix defect**: NeoTrix has no cold-start reasoning for new capabilities or new users. No offline-generated reason tokens for online decoding. New modules lack initial ranking boost. → **DEFECT-574-12: No cold-start reasoning mechanism for new capabilities/modules.**

### 3.4 PSAD — Personalized Semi-Autoregressive Reranking (2026)
- **Source**: https://arxiv.org/pdf/2603.07107 (2026)
- **Finding**: Semi-autoregressive generation balances quality and latency. Online knowledge distillation trains lightweight student from teacher on-the-fly (no pre-trained teacher needed). User Profile Network with personalized gates and personalized position encoding captures interest dynamics.
- **NeoTrix defect**: NeoTrix has no distillation from heavy reranker to lightweight scorer. All ranking decisions use the same model depth regardless of latency budget. → **DEFECT-574-13: No teacher-student distillation for latency-adaptive ranking.**

### 3.5 hLLM — Hungarian Algorithm for Generative Reranking (2026)
- **Source**: https://arxiv.org/abs/2609.01807 (2026-09-01)
- **Finding**: hLLM reads item-position score matrix from prefill hidden states, decodes ordinals via Hungarian algorithm (optimal bipartite assignment). Valid permutation by construction. 28ms end-to-end inference — 10x+ speedup over autoregressive decoding while maintaining quality.
- **NeoTrix defect**: NeoTrix's ranking uses softmax-based scoring with no combinatorial optimization. No guaranteed-valid permutation output. Potential for invalid rankings (duplicate items, missing items). → **DEFECT-574-14: No combinatorial optimization for guaranteed-valid ranking permutations.**

### 3.6 Exposure-Based RL for Learning to Rank (2026)
- **Source**: https://arxiv.org/html/2607.18689v1 (2026)
- **Finding**: Exposure-based RL for LTR avoids custom gradients by abstracting behind exposure distribution. Seamless integration with auto-differentiation. Converges faster and at higher performance than PL-Rank. PL-Rank has severe stability issues with 32-bit floats over many epochs (first observation).
- **NeoTrix defect**: NeoTrix's ranking optimization (if any) would use standard gradients. No exposure-based abstraction. No fairness-of-exposure consideration in tool/capability ranking. → **DEFECT-574-15: No exposure-based ranking optimization — missing fairness dimension.**

### 3.7 Think-to-Personalize (TTP) — Reasoning-Driven Dense Retrieval (2026)
- **Source**: https://arxiv.org/html/2608.18855 (2026-08-19)
- **Finding**: TTP unifies explicit user-centric intent reasoning with dense retrieval. Two-stage training: SFT for cold-start + RL (GRPO) for retrieval-aligned reasoning. Generates intent-enhanced queries from user history. Online A/B: +0.46% order volume. Bridges intent gap for ambiguous/long-tail queries.
- **NeoTrix defect**: NeoTrix's query understanding is static — no reasoning over user history to disambiguate intent. Long-tail queries to the KB may fail due to underspecified intent. → **DEFECT-574-16: No reasoning-driven query disambiguation for KB search — long-tail queries underserved.**

---

## Summary: New Defects vs. Batch 573

| ID | Defect | Domain | Severity |
|---|---|---|---|
| DEFECT-574-1 | No vector store backend migration contract | NT-MEMORY | HIGH |
| DEFECT-574-2 | No auto-calibration for retrieval parameters | NT-MEMORY | HIGH |
| DEFECT-574-3 | No formal KB capacity model or migration trigger | NT-MEMORY | MEDIUM |
| DEFECT-574-4 | Hybrid search fusion lacks probabilistic calibration | NT-MEMORY | HIGH |
| DEFECT-574-5 | Static hybrid mixing weights — no query-adaptive alpha | NT-MEMORY | HIGH |
| DEFECT-574-6 | No per-path quality assessment before fusion | NT-MEMORY | HIGH |
| DEFECT-574-7 | Fixed retrieval depth — no equivalence guarantee | NT-MEMORY | MEDIUM |
| DEFECT-574-8 | No self-supervised embedding refinement | NT-MEMORY | MEDIUM |
| DEFECT-574-9 | No domain-aware retrieval strategy selection | NT-MEMORY+WORLD | HIGH |
| DEFECT-574-10 | Tool routing lacks iterative refinement | NT-ACT | MEDIUM |
| DEFECT-574-11 | Tool selection lacks reward-weighted ranking | NT-ACT | MEDIUM |
| DEFECT-574-12 | No cold-start reasoning for new capabilities | NT-ACT | MEDIUM |
| DEFECT-574-13 | No teacher-student distillation for latency-adaptive ranking | NT-ACT | LOW |
| DEFECT-574-14 | No combinatorial optimization for valid permutations | NT-ACT | LOW |
| DEFECT-574-15 | No exposure-based ranking optimization | NT-ACT | LOW |
| DEFECT-574-16 | No reasoning-driven query disambiguation | NT-MEMORY+WORLD | HIGH |

**Total new defects: 16** (9 NT-MEMORY, 5 NT-ACT, 2 cross-domain)

## Sources Cited
1. Meilisearch March 2026 updates — https://www.meilisearch.com/blog/March-2026-updates
2. Elastic 9.5 announcement — https://www.elastic.co/blog/whats-new-elastic-9-5-0
3. Meilisearch vs Elasticsearch API 2026 — https://apiscout.dev/guides/elasticsearch-vs-meilisearch-api-2026
4. Unified Bayesian Framework for Hybrid Search — https://doi.org/10.5281/zenodo.20768748
5. Query-Adaptive Hybrid Search — https://doi.org/10.3390/make8040091
6. VLDB Hybrid Search Trade-offs — https://www.vldb.org/pvldb/vol19/p1715-gao.pdf
7. EAHR — https://arxiv.org/html/2608.07152v1
8. vstash — https://arxiv.org/abs/2604.15484
9. T2-RAGBench — https://arxiv.org/html/2604.01733v1
10. SlimPer (Meta) — https://arxiv.org/html/2607.12281v1
11. GenRec (Netflix) — https://arxiv.org/html/2608.10257v1
12. TGR (Tencent) — https://arxiv.org/abs/2609.00986
13. PSAD — https://arxiv.org/pdf/2603.07107
14. hLLM — https://arxiv.org/abs/2609.01807
15. Exposure-Based RL for LTR — https://arxiv.org/html/2607.18689v1
16. Think-to-Personalize — https://arxiv.org/html/2608.18855

## Cross-Batch Meta-Defect (573→574 Continuity)

**Batch 573's core finding — "formal specs are a universal structural gap in AI-native systems" — is confirmed and extended by Batch 574.** The search/IR/ranking domain shows the same pattern: production systems (Elasticsearch, Meilisearch, Netflix GenRec, Tencent TGR, Meta SlimPer) are shipping adaptive, calibrated, reward-aligned ranking — while NeoTrix's NT-MEMORY and NT-ACT use static, uncalibrated, single-signal retrieval and routing. The gap is not just missing formal specs but missing **adaptive feedback loops** that production systems now treat as table stakes.
