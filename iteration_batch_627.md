# Iteration Batch 627 — Feature Engineering, Dimensionality Reduction, Embedding

**Date**: 2026-09-06
**Previous**: Batch 626 (event-sourced decision log, optimistic concurrency, idempotency, event envelope, upcasting)
**Focus**: Feature engineering, dimensionality reduction, embedding — 2026 state-of-the-art

---

## 1. FEATURE ENGINEERING

### Sources
- PromptFE (EACL 2026) — LLM-driven automated feature construction via prompts ([aclanthology.org/2026.eacl-long.28](https://aclanthology.org/2026.eacl-long.28/))
- FAMOSE (arXiv 2602.17641) — ReAct agent for iterative feature engineering with mRMR selection
- TOPOFE (arXiv 2607.23286) — Graph-of-Islands evolutionary AutoFE with LLM mutation/crossover
- Panama Feature-Store (ICHORA 2026) — YAML contract language for offline-online consistency
- RisingWave real-time feature store (2026) — Streaming SQL materialized views replacing batch+online split
- Jumio/AWS feature store (Aug 2026) — Kinesis→Flink→SageMaker Feature Store with tiered storage
- Enterprise Feature Store guide (Jul 2026) — Architecture patterns, OpenLineage, streaming CDC

### NEW Defect: No Feature Store Layer in NeoTrix

NeoTrix has `nt_world` for data ingestion and `nt_memory` for KB storage, but **no dedicated feature engineering/store abstraction**. The 2026 landscape shows:

1. **Streaming feature freshness is table stakes** — AI agents need sub-second feature freshness, not batch recomputed features. RisingWave and Jumio show streaming-first architectures are mandatory for real-time agent decisions.
2. **No offline-online consistency mechanism** — Panama's YAML contract language and point-in-time correctness checks prevent training-serving skew. NeoTrix's KB has no concept of `as_of_date` partitioning for temporal feature joins.
3. **No feature reuse across domains** — Enterprise feature stores show 20-25% of features reused across teams. NeoTrix's 7 domains (NT-CORE through NT-FEEL) each compute features independently with no shared feature catalog.

**Impact**: When NeoTrix agents (NT-ACT) make real-time decisions, they cannot serve fresh features. When NT-MIND trains on historical data, it risks training-serving skew. Feature definitions are duplicated across domains.

### NEW Defect: No Feature Contract / Schema Evolution for Feature Definitions

Panama (2026) uses a YAML-based contract language that defines feature schemas, source mappings, and materialization rules. When schemas evolve, the contract provides versioning and migration. NeoTrix's KB nodes have no equivalent:
- Feature definitions are implicit in code, not declarative
- No versioning when feature computation logic changes
- No migration path when upstream data schemas change

---

## 2. DIMENSIONALITY REDUCTION

### Sources
- DiRe-RAPIDS (arXiv 2604.25209, Apr 2026) — Topology-faithful DR at scale, Pareto-dominates UMAP on 7/11 datasets
- UMAP spectral clustering proof (arXiv 2602.11662, Feb 2026) — UMAP = spectral clustering on fuzzy k-NN graph
- LMC/MiCS (Pattern Anal. Applic., Jan 2026) — Landmark Mantel Correlation for global structure preservation, GSP increased from 0.49 to 0.92
- PCA/t-SNE/UMAP comparison guide (Mar 2026) — scikit-learn 1.8 benchmarks, PCA→UMAP pipeline recommended
- Biology DR guide (Jul 2026) — UMAP global structure claims depend on initialization, not algorithm itself

### NEW Defect: No Topology-Aware Dimensionality Reduction for VSA HyperCube

NeoTrix uses VSA HyperCube for knowledge representation (high-dimensional vector symbolic architecture). The 2026 findings reveal:

1. **DiRe preserves topological structure 3-4x better than UMAP** on large corpora (723K arXiv papers). UMAP's k-NN metric rewards reproduction of sampling noise as well as manifold structure. For VSA HyperCube, where topological relationships between concepts matter, UMAP is actively harmful.
2. **UMAP is spectral clustering** (proven Feb 2026). Its global structure preservation is an artifact of initialization, not the algorithm. NeoTrix's use of UMAP for embedding visualization loses inter-cluster distance meaning.
3. **LMC achieves Mantel-Pearson GSP of 0.92** vs UMAP's 0.49 — a 2x improvement in global structure preservation. NeoTrix has no mechanism for landmark-based global structure preservation.

**Impact**: VSA HyperCube embeddings lose topological fidelity during dimensionality reduction. Concept relationships that span multiple clusters are distorted or destroyed. The GWT attention routing receives degraded signal from reduced embeddings.

### NEW Defect: No Adaptive Hyperparameter Selection for DR Methods

TOPOFE (Jul 2026) shows that multi-objective optimization (NSGA-II) over DR hyperparameters discovers configurations that Pareto-dominate defaults on 7/11 datasets. DiRe's default `spread=1.0` is off by a factor of 3.6 for topology-faithful embedding. NeoTrix applies dimensionality reduction with static parameters, never adapting to dataset-specific topology.

---

## 3. EMBEDDING

### Sources
- KV-Embedding (ACL 2026) — Training-free text embedding via internal KV re-routing, +10% over baselines
- OmniSONAR (Meta, Aug 2026) — Omnilingual 4200-language cross-modal sentence embeddings
- pplx-embed (ACL Industry 2026) — Diffusion-pretrained bidirectional embeddings, INT8 quantization-aware
- SEMPA (ACL Findings 2026) — Semantic preference alignment via DPO for LLM embeddings
- Giga-Embeddings (arXiv 2608.23806, Aug 2026) — 10B MoE encoder, 114.5k tokens/sec throughput
- Gorse Embedding Benchmark (Feb 2026) — Qwen3-embedding:4b is cost-efficiency king
- Milvus RAG Benchmark (Mar 2026) — 10 models tested, Gemini Embedding 2 best all-rounder

### NEW Defect: No Bidirectional Embedding for Causal LLM Backbones

KV-Embedding (ACL 2026) proves that frozen decoder-only LLMs can produce high-quality embeddings by re-routing internal KV states. NeoTrix's LLM-based perception (nt_world) uses causal attention for all tasks including embedding generation. This means:
- Early tokens cannot access subsequent context
- Next-token prediction objective biases representations toward generation, not semantic compression
- Training-free embedding quality is 10% worse than it could be

**Impact**: NeoTrix's text understanding pipeline produces suboptimal embeddings for retrieval, classification, and clustering tasks without any parameter updates.

### NEW Defect: No Quantization-Aware Embedding Pipeline

pplx-embed (2026) demonstrates native INT8 quantization-aware training producing compact embeddings with minimal performance loss. At binary quantization, they store 3,125 docs/MB vs float32's 390 docs/MB — an 8x storage reduction. NeoTrix stores all embeddings in float32 with no quantization strategy:
- KB embedding storage grows unboundedly
- No tiered storage (hot/cold) for embeddings
- No Matryoshka Representation Learning (MRL) support for dimension truncation

**Impact**: Embedding storage costs scale linearly with knowledge base size. No ability to trade off precision for storage/memory at query time.

### NEW Defect: No Cross-Modal Embedding Unification

OmniSONAR (Meta, Aug 2026) establishes a unified semantic space for text, speech, code, and math across 4,200 languages. NeoTrix's embedding pipeline is text-only:
- No speech embedding integration (NT-FEEL cannot process vocal emotion)
- No code embedding for NT-ACT's tool understanding
- No cross-modal alignment between VSA HyperCube symbolic representations and neural embeddings

**Impact**: NeoTrix's 7 domains cannot share a common semantic space. NT-WORLD's text perception, NT-FEEL's emotional analysis, and NT-ACT's code understanding operate in disconnected embedding spaces.

### NEW Defect: No Embedding Model Lifecycle / Benchmarking

Milvus (2026) and Gorse (2026) show that no single embedding model wins across all dimensions. The field requires:
- Per-task model selection (cross-modal vs cross-lingual vs key-info)
- Dimension compression budgets (512 vs 1024 vs 3072)
- Latency/throughput requirements

NeoTrix has no embedding model registry, no A/B testing for embedding quality, and no mechanism to swap embedding backends based on task requirements.

---

## Summary: 8 NEW Defects Found

| # | Defect | Domain | Severity |
|---|--------|--------|----------|
| 1 | No feature store layer (streaming freshness, offline-online consistency, feature reuse) | NT-WORLD + NT-ACT | CRITICAL |
| 2 | No feature contract / schema evolution for feature definitions | NT-MEMORY | HIGH |
| 3 | No topology-aware dimensionality reduction (DiRe/LMC for VSA HyperCube) | NT-CORE | CRITICAL |
| 4 | No adaptive hyperparameter selection for DR methods | NT-CORE | HIGH |
| 5 | No bidirectional embedding for causal LLM backbones (KV re-routing) | NT-WORLD | HIGH |
| 6 | No quantization-aware embedding pipeline (INT8/binary/MRL) | NT-MEMORY | HIGH |
| 7 | No cross-modal embedding unification (text/speech/code/math) | ALL domains | CRITICAL |
| 8 | No embedding model lifecycle / benchmarking | NT-IO | MEDIUM |

---

## Sources Cited

1. PromptFE — ACL 2026 EACL Long Paper 28
2. FAMOSE — arXiv 2602.17641
3. TOPOFE — arXiv 2607.23286
4. Panama Feature-Store — ICHORA 2026, DOI:10.1109/ichora69329.2026.11537084
5. RisingWave real-time feature store — risingwave.com/blog/real-time-feature-store-2026/
6. Jumio/AWS feature store — aws.amazon.com/blogs/machine-learning/how-jumio-built-a-real-time-feature-store-on-aws/
7. Enterprise Feature Store guide — enterprise-software-review.contentwave.net
8. DiRe-RAPIDS — arXiv 2604.25209v2
9. UMAP spectral clustering proof — arXiv 2602.11662
10. LMC/MiCS — Pattern Anal. Applic. 29, 38 (2026), DOI:10.1007/s10044-025-01585-9
11. PCA/t-SNE/UMAP comparison — pythondatabench.com
12. Biology DR guide — technologynetworks.com/informatics/articles/dimensionality-reduction-in-biology
13. KV-Embedding — ACL 2026 Long Paper 540
14. OmniSONAR — Meta, arXiv 2603.16606v3
15. pplx-embed — ACL 2026 Industry Paper 69
16. SEMPA — ACL Findings 2026, Paper 1858
17. Giga-Embeddings — arXiv 2608.23806
18. Gorse Embedding Benchmark — gorse.io/posts/embedding-benchmark-2026
19. Milvus RAG Benchmark — milvus.io/blog/choose-embedding-model-rag-2026.md
