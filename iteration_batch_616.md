# Iteration Batch 616 — Agent Memory, Vector DB, RAG

**Date**: 2026-09-06
**Previous batch**: 615 (Apollo Federation, agent retry, RFC 9457, spec fragmentation)
**Status**: Research-only — no code changes

---

## 1. Agent Memory — NEW Findings

### 1.1 AgeMem (ACL 2026) — Unified LTM/STM via RL
**Source**: https://aclanthology.org/2026.acl-long.981/
- Proposes unified LTM+STM management as tool-based actions exposed to the agent policy
- Three-stage progressive RL + step-wise GRPO for sparse reward from memory operations
- **NEW vs 615**: Memory operations are now first-class **actions** in the agent's RL policy, not external middleware. This means memory management becomes part of the reward signal, not a side effect.
- **DEFECT-616-01**: No mechanism for memory cost accounting. Each memory operation (store/retrieve/summarize/discard) consumes inference tokens but there is no budget constraint. Agent can "spend" unlimited memory operations to boost task reward — overfitting to memory overhead.
- **DEFECT-616-02**: Step-wise GRPO requires dense per-step reward. For memory, reward is only available at task completion. The gap between memory-action and task-outcome creates a credit assignment horizon that scales with context length — effectively unbounded.

### 1.2 EARM — Experience-Amortized Reranking
**Source**: https://arxiv.org/abs/2608.22767
- Uses previously acquired LLM relevance scores as reusable experience via online matrix completion
- Reduces scoring budget as experience accumulates: LLM reranking cost decreases over agent lifetime
- **NEW vs 615**: Addresses the "retriever should remember too" — the retriever itself learns from past queries, not just the memory content.
- **DEFECT-616-03**: Matrix completion assumes past query-memory relevance structure persists. Novel queries (out-of-distribution) get estimated scores that may be worse than raw embedding similarity. No fallback mechanism documented.
- **DEFECT-616-04**: Online matrix grows unbounded with query count. The "experience accumulation" is also memory accumulation — there is no pruning of stale relevance scores, contradicting the goal of reducing overhead.

### 1.3 Mem0 Benchmark Landscape (April 2026)
**Source**: https://mem0.ai/blog/state-of-ai-agent-memory-2026
- LoCoMo 92.5, LongMemEval 94.4, BEAM 1M 64.1, BEAM 10M 48.6
- Temporal +29.6pt, multi-hop +23.1pt over 2025 baseline
- Multi-signal retrieval: semantic + BM25 + entity matching fused
- 21 frameworks, 20 vector stores integrated
- **NEW vs 615**: Token efficiency metric crystallized — 6,956 tokens/query vs 26,000 full-context. This is the cost axis.
- **DEFECT-616-05**: BEAM 1M→10M shows 64.1→48.6 (25% loss at 10x scale). Temporal abstraction degrades catastrophically at scale. No architecture proposed to close this gap.
- **DEFECT-616-06**: Memory staleness remains unsolved. Highly-retrieved memory about user's employer is "accurately wrong" after job change. Decay handles low-relevance; staleness in high-relevance memories is an open problem. **Direct risk to NeoTrix KB**: any high-confidence cached fact can become confidently incorrect.

### 1.4 MAGMA — Multi-Graph Agentic Memory (ACL 2026)
**Source**: https://aclanthology.org/2026.acl-long.1709.pdf
- Four orthogonal graphs: semantic, temporal, causal, entity
- Policy-guided traversal over relational views, query-adaptive
- Dual-stream: fast ingestion + async structural consolidation
- **NEW vs 615**: Separation of memory representation from retrieval logic. Causal graph is the new dimension — previous systems only modeled semantic + temporal.
- **DEFECT-616-07**: Graph quality depends on LLM reasoning fidelity during async consolidation. Extraction errors propagate as hallucinated causal links. No self-consistency check or adversarial validation of graph structure.
- **DEFECT-616-08**: Dual-stream architecture creates temporal window where ingestion sees stale graph. Concurrent reads during async consolidation may return incomplete or inconsistent causal paths.

### 1.5 CoEvo-Mem — Co-Evolving Retrieval + Memory Bank
**Source**: https://arxiv.org/abs/2608.01739
- Closed-loop: retrieval determines which memories get feedback; memory updates reshape retrieval
- Phase-wise block update: router and memory alternate fixed/moving
- **NEW vs 615**: Recognizes retrieval and memory as **coupled** systems, not independent layers. This is architecturally correct — batch 615 treated them as separate problems.
- **DEFECT-616-09**: Phase-wise alternation assumes ergodicity — that alternating updates converge. For non-stationary memory (user context changes over time), the alternation can oscillate rather than converge.
- **DEFECT-616-10**: Seven benchmarks tested, but all are conversational QA. No evaluation on agent-tool-use, autonomous planning, or cross-session persistence scenarios relevant to NeoTrix.

### 1.6 HERO — Human-profile Enhanced Retrieval
**Source**: https://arxiv.org/abs/2608.22310
- Converts dialogue to heterogeneous memory graph preserving raw text as evidence
- Iterative graph traversal with human profile anchors
- **NEW vs 615**: Preserves raw dialogue text as evidence, not compressed summaries. Addresses information loss from compression and semantic drift from rewriting.
- **DEFECT-616-11**: Raw text preservation scales linearly with conversation length. No mechanism to bound storage growth for long-lived agents.

### 1.7 RippleMem — Adaptive Associative Recollection
**Source**: https://arxiv.org/abs/2608.13334
- Replaces one-shot retrieval with cue-dependent expansion along semantic + structural associations
- +3.95% on LoCoMo, +11.87% on LongMemEval-S
- **NEW vs 615**: Retrieval is not just query→results but query→anchor→expansion→evidence completion. Matches cognitive episodic memory models.
- **DEFECT-616-12**: Expansion is unbounded — no hop limit or relevance decay along association edges. Risk of "retrieval explosion" where expanding associations pull in increasingly tangential memories.

### 1.8 LiCoMemory — Lightweight Cognitive Memory
**Source**: https://aclanthology.org/2026.findings-acl.1835.pdf
- CogniGraph as semantic indexing layer, not static repository
- Real-time updating + retrieval, temporal + hierarchy-aware search
- **NEW vs 615**: Graph-as-index rather than graph-as-storage. Lightweight, incremental construction.
- **DEFECT-616-13**: "Lightweight" = simplified graph. Loses the causal and entity-resolution depth of MAGMA. Trade-off between update latency and memory fidelity not quantified.

---

## 2. Vector Database — NEW Findings

### 2.1 Filtered Search Is the Real Differentiator (July 2026)
**Source**: https://dreaming.press/posts/qdrant-vs-milvus-vs-weaviate.html
- Scale axis has collapsed: all three run laptop→cluster, all do hybrid search
- The real differentiator: **how** filtered search is implemented
  - Qdrant: payload indexes fused into HNSW traversal (filterable HNSW)
  - Weaviate: ACORN predicate-agnostic traversal (default v1.34)
  - Milvus: iterative filtering + partition keys
- **NEW vs 615**: Batch 615 compared by scale/performance. The 2026 reality is: filtered search composition with ANN is the ballgame. Pre-filter → graph fragmentation; post-filter → over-fetch/topK miss.
- **DEFECT-616-14**: None of the three engines handle **compound filters across heterogeneous field types** (e.g., temporal range + geo proximity + text match) gracefully. Each engine has one dominant filter strategy; compound predicates degrade to sequential evaluation.

### 2.2 Milvus 3.0 — Lake-Native Vector Search (July 2026)
**Source**: https://milvus.io/blog/announcing-milvus-3-lake-native-vector-search-and-a-more-powerful-retrieval-engine.md
- External Collections over Parquet/Lance/Iceberg/Vortex — zero-copy, read-only
- Server-side sorting, aggregation, faceted search, StructArray, ColBERT vectors
- SINDI sparse retrieval: 10× QPS over MaxScore for SPLADE embeddings
- Loon (Storage v3): manifest-based columnar, 135× less I/O per point read than Parquet
- **NEW vs 615**: Vector data no longer needs to be copied into a separate database. External Collections over open formats are the third path (vs copy-to-DB or brute-force lake scan).
- **DEFECT-616-15**: External Collections are read-only. No write path for streaming/real-time ingestion. This creates a two-tier architecture: lake-resident historical data (read-only) + separate writable serving copy. The "zero-copy" promise is partial.
- **DEFECT-616-16**: SINDI's SIMD optimization assumes CPU vector instructions. No GPU path documented for SPLADE/sparse workloads. GPU acceleration only for dense indexing.

### 2.3 Weaviate 1.39 (August 2026)
**Source**: https://weaviate.io/blog/weaviate-1-39-release
- MMR diversity selection GA on hybrid search
- 4-bit Rotational Quantization (preview): 784 bytes per 1536-dim vector (7.84× compression)
- Boost API for query-time rescoring
- Search REST API (experimental): JSON over HTTP/1.1, documented by OpenAPI — **explicitly designed for LLM tool calling**
- Automatic HNSW snapshots GA
- **NEW vs 615**: Weaviate's Search REST API is designed for LLM tool calling — agent can call `POST /v1/search/{collection}/near-text` as a tool. This is a direct integration point for NeoTrix agent memory.
- **DEFECT-616-17**: Search REST API is experimental and off by default. Only 5 endpoints, no write/search-delete/update. LLM tool calling needs a complete CRUD surface, not just search.

### 2.4 Qdrant 1.17-1.18 Updates (2026)
**Source**: https://blog.elest.io/qdrant-vs-weaviate-vs-milvus-which-vector-database-for-your-rag-pipeline/
- TurboQuant: ~8× compression at near-baseline recall (from Google Research)
- Low-memory mode, dynamic CPU pooling
- GPU indexing, Multi-AZ replication, audit logging in Cloud
- Payload-filtered multitenancy (collection-per-tenant is outdated — use payload filtering)
- **NEW vs 615**: Qdrant's approach to multitenancy (payload-based filtering, not collection-per-tenant) is directly applicable to NeoTrix's multi-domain architecture (7 domains sharing one vector store).
- **DEFECT-616-18**: TurboQuant's "near-baseline recall" claim has no published recall-vs-compression curve. The 8× figure is stated without confidence intervals or workload dependencies.

### 2.5 Benchmark Convergence (July 2026)
**Source**: https://lushbinary.com/blog/vector-database-benchmarks-production-selection-guide-2026/
- At 10M vectors: Qdrant ~4ms p99, ~3200 QPS; Milvus ~5ms, ~2900 QPS; Weaviate ~9ms, ~1800 QPS
- At 100M vectors (distributed): Qdrant ~12ms; Milvus ~15ms; Weaviate ~25ms; pgvector ~85ms
- pgvector with StreamingDiskANN: feasible but 5-7× latency penalty vs dedicated engines
- **NEW vs 615**: pgvector's StreamingDiskANN makes it feasible for 100M+ vectors at the cost of 5-7× latency. For NeoTrix (if already on Postgres), pgvector is viable for non-latency-critical paths.
- **DEFECT-616-19**: Benchmark numbers are vendor-influenced. All three engines shipped major releases in 2026; numbers from early 2026 are already stale. No independent cross-version benchmark exists.

---

## 3. RAG — NEW Findings

### 3.1 UniversalRAG — Any-to-Any Modality RAG (ACL 2026)
**Source**: https://aclanthology.org/2026.acl-long.177/
- Modality-aware routing: dynamically identifies appropriate modality-specific corpus
- Multi-granularity retrieval within each modality
- Theoretical justification: forced unified representation causes "modality gap"
- **NEW vs 615**: Batch 615 didn't address multimodal RAG. UniversalRAG proves single-corpus approach causes modality gap. NeoTrix (with NT-WORLD crawling diverse content) needs modality-aware routing.
- **DEFECT-616-20**: Routing requires knowing query modality at dispatch time. Ambiguous queries ("show me the research on X") can't be cleanly routed — the system must either guess or query multiple modalities.

### 3.2 A-RAG — Agentic RAG with Hierarchical Interfaces (August 2026)
**Source**: https://arxiv.org/pdf/2602.03442
- Exposes keyword_search, semantic_search, chunk_read as tools to the agent
- Agent autonomously chooses retrieval strategy per query
- Scales with model capability: GPT-5-mini outperforms GPT-4o-mini more on complex tasks
- **NEW vs 615**: RAG as agent tool-use, not algorithm. The model participates in retrieval decisions. This aligns with batch 615's finding that "agents are primary API consumers" — agents should also be primary retrieval decision-makers.
- **DEFECT-616-21**: Three tools (keyword/semantic/chunk_read) is insufficient for NeoTrix's 7-domain architecture. Each domain has different retrieval semantics (KB embedding vs VSA embedding vs BM25). A-RAG's tool set is domain-agnostic.
- **DEFECT-616-22**: Test-time scaling shows diminishing returns above 20 steps. The "agentic" overhead (LLM calls for retrieval decisions) can exceed the retrieval cost itself.

### 3.3 LAnR — Latent Abstraction for RAG
**Source**: https://arxiv.org/abs/2604.17866
- Retrieval entirely in LLM latent space — no text query generation
- MLP control head for adaptive retrieval stopping
- 30× fewer output tokens, 1.5-2.7× wall-clock speedup
- **NEW vs 615**: Eliminates the retriever-generator architectural boundary. Single model does both. This is the architectural endgame for RAG — the "federation" between retriever and generator disappears.
- **DEFECT-616-23**: Latent-space retrieval requires the embedding model and the LLM to share a representation space. Currently requires LoRA fine-tuning per model family. Not transferable across model architectures.
- **DEFECT-616-24**: MLP stopping head is a binary decision (retrieve more / stop). Cannot express "retrieve from a different corpus" or "switch retrieval modality." The control signal is too low-dimensional for multi-domain retrieval.

### 3.4 SARA — Selective Adaptive RAG with Compression
**Source**: https://aclanthology.org/2026.acl-long.661.pdf
- Hybrid: natural language snippets + semantic compression vectors
- Iterative evidence reranking with embedding-based novelty + conditional self-information
- +17.71 answer relevance, +13.72 correctness across 9 datasets
- **NEW vs 615**: Addresses the "answer was in the context but ranked too low" failure mode. Compression vectors enable global context without token budget explosion.
- **DEFECT-616-25**: Compression vectors are opaque — no interpretability. Cannot trace which compressed evidence contributed to the answer. Violates NeoTrix's evidence-first principle.

### 3.5 NeuRAG — Neuralized RAG via Hyper-Neurons
**Source**: https://aclanthology.org/2026.findings-acl.1516/
- Each document encoded as a LoRA module ("knowledge neuron")
- Hyper-Layer dynamically activates knowledge neurons via attention
- Single forward pass retrieves + reasons
- **NEW vs 615**: Eliminates retrieval latency entirely — knowledge is in the forward pass. For NeoTrix: this could replace KB retrieval with parameter-space knowledge for hot paths.
- **DEFECT-616-26**: LoRA modules per document don't scale to millions of documents. Each knowledge neuron adds parameters. Memory footprint = O(documents × LoRA_rank). At 1M documents with rank-16, that's 16M additional parameters per model — feasible but growing.
- **DEFECT-616-27**: No mechanism for knowledge update without re-encoding. Changing one document requires re-training its LoRA module. For NeoTrix's constantly evolving KB, this is a significant operational cost.

### 3.6 Disco-RAG — Discourse-Aware RAG (ACL 2026)
**Source**: https://aclanthology.org/2026.acl-long.189/
- Intra-chunk discourse trees + inter-chunk rhetorical graphs
- Planning blueprint conditions generation on discourse structure
- **NEW vs 615**: Structure-aware retrieval, not just content-aware. Discourse signals (cause, elaboration, contrast) improve synthesis from dispersed evidence.
- **DEFECT-616-28**: Discourse tree construction requires a parsing model. Overhead not quantified against the retrieval+generation latency.

### 3.7 MegaRAG — Multimodal KG-RAG (ACL 2026)
**Source**: https://aclanthology.org/2026.acl-long.2218/
- Visual cues incorporated into KG construction, retrieval, and generation
- Cross-modal reasoning for full-book comprehension
- **NEW vs 615**: Extends KG-RAG to visual documents. For NeoTrix's NT-WORLD (crawling diverse content types), this addresses the visual modality gap.

---

## 4. Consolidated Defects (616-01 through 616-28)

| ID | Domain | Defect | Severity | Applicable to NeoTrix |
|---|---|---|---|---|
| 616-01 | Agent Memory | No memory operation cost budget | High | KB memory operations |
| 616-02 | Agent Memory | GRPO credit assignment horizon unbounded | Medium | SEAL pipeline memory ops |
| 616-03 | Agent Memory | EARM matrix completion OOD failure | Medium | Cross-session retrieval |
| 616-04 | Agent Memory | Experience matrix unbounded growth | High | Long-lived agent memory |
| 616-05 | Agent Memory | BEAM 10M: 25% degradation at scale | High | Large KB queries |
| 616-06 | Agent Memory | Staleness in high-confidence memories | Critical | KB cached facts |
| 616-07 | Agent Memory | MAGMA causal graph hallucination | High | Graph-based memory |
| 616-08 | Agent Memory | Dual-stream consistency window | Medium | Async consolidation |
| 616-09 | Agent Memory | CoEvo-Mem alternation non-convergence | Medium | Memory-retrieval co-evolution |
| 616-10 | Agent Memory | CoEvo-Mem tested only on conversational QA | Medium | Agent-tool-use scenarios |
| 616-11 | Agent Memory | HERO raw text linear scaling | Medium | Long-term storage |
| 616-12 | Agent Memory | RippleMem unbounded association expansion | High | Retrieval explosion |
| 616-13 | Agent Memory | LiCoMemory graph simplification trade-off | Low | Fidelity vs update speed |
| 616-14 | Vector DB | Compound heterogeneous filters unhandled | High | Multi-field queries |
| 616-15 | Vector DB | Milvus External Collections read-only | Medium | Streaming ingestion |
| 616-16 | Vector DB | SINDI CPU-only, no GPU sparse path | Low | GPU workloads |
| 616-17 | Vector DB | Weaviate Search REST API incomplete CRUD | Medium | LLM tool calling |
| 616-18 | Vector DB | TurboQuant recall curve not published | Medium | Quantization selection |
| 616-19 | Vector DB | No independent cross-version benchmarks | Medium | Vendor-neutral evaluation |
| 616-20 | RAG | UniversalRAG query modality ambiguity | High | Multimodal routing |
| 616-21 | RAG | A-RAG tool set too simple for 7 domains | High | Multi-domain retrieval |
| 616-22 | RAG | A-RAG agentic overhead exceeds retrieval cost | Medium | Complex queries |
| 616-23 | RAG | LAnR latent space not transferable across models | Medium | Model switching |
| 616-24 | RAG | LAnR control head too low-dimensional | Medium | Multi-corpus routing |
| 616-25 | RAG | SARA compression vectors opaque | Medium | Evidence tracing |
| 616-26 | RAG | NeuRAG LoRA-per-doc scaling | Low | Large document corpora |
| 616-27 | RAG | NeuRAG no incremental knowledge update | High | Evolving KB |
| 616-28 | RAG | Disco-RAG discourse parsing overhead unquantified | Low | Latency budgets |

---

## 5. Summary: What's NEW vs Batch 615

| Dimension | Batch 615 | Batch 616 |
|---|---|---|
| Agent Memory | Retry loops, idempotency | Memory as RL action (AgeMem), co-evolution (CoEvo-Mem), staleness crisis, multi-graph architecture (MAGMA) |
| Vector DB | Apollo Federation 27× speedup | Filtered search composition is differentiator; Milvus 3.0 lake-native; Weaviate 1.39 Search REST for LLM tool calling; Qdrant TurboQuant 8× compression |
| RAG | Spec fragmentation, RFC 9457 gap | Agentic RAG (A-RAG), latent-space RAG (LAnR), neuralized RAG (NeuRAG), multimodal routing (UniversalRAG), discourse-aware (Disco-RAG) |
| Critical new defect | Agent retry double-charge | Memory staleness in high-confidence memories (616-06) — **direct risk to NeoTrix KB** |
| NeoTrix integration | Federation needs | (1) Memory ops as RL actions for SEAL; (2) Weaviate Search REST API for agent tool calling; (3) Modality-aware routing for NT-WORLD; (4) CoEvo-Mem pattern for KB+retrieval co-evolution |

---

## 6. Sources Cited

1. Yu et al. "Agentic Memory" ACL 2026 — https://aclanthology.org/2026.acl-long.981/
2. EARM arXiv 2608.22767 — https://arxiv.org/abs/2608.22767
3. Mem0 State of Agent Memory 2026 — https://mem0.ai/blog/state-of-ai-agent-memory-2026
4. MAGMA ACL 2026 — https://aclanthology.org/2026.acl-long.1709.pdf
5. CoEvo-Mem arXiv 2608.01739 — https://arxiv.org/abs/2608.01739
6. HERO arXiv 2608.22310 — https://arxiv.org/abs/2608.22310
7. RippleMem arXiv 2608.13334 — https://arxiv.org/abs/2608.13334
8. LiCoMemory ACL Findings 2026 — https://aclanthology.org/2026.findings-acl.1835.pdf
9. Qdrant vs Milvus vs Weaviate (July 2026) — https://dreaming.press/posts/qdrant-vs-milvus-vs-weaviate.html
10. Milvus 3.0 Blog — https://milvus.io/blog/announcing-milvus-3-lake-native-vector-search-and-a-more-powerful-retrieval-engine.md
11. Weaviate 1.39 Release — https://weaviate.io/blog/weaviate-1-39-release
12. Vector DB Benchmarks 2026 — https://lushbinary.com/blog/vector-database-benchmarks-production-selection-guide-2026/
13. Elest.io Comparison — https://blog.elest.io/qdrant-vs-weaviate-vs-milvus-which-vector-database-for-your-rag-pipeline/
14. UniversalRAG ACL 2026 — https://aclanthology.org/2026.acl-long.177/
15. A-RAG arXiv 2602.03442 — https://arxiv.org/pdf/2602.03442
16. LAnR arXiv 2604.17866 — https://arxiv.org/abs/2604.17866
17. SARA ACL 2026 — https://aclanthology.org/2026.acl-long.661.pdf
18. NeuRAG ACL Findings 2026 — https://aclanthology.org/2026.findings-acl.1516/
19. Disco-RAG ACL 2026 — https://aclanthology.org/2026.acl-long.189/
20. MegaRAG ACL 2026 — https://aclanthology.org/2026.acl-long.2218/
