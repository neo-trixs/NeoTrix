# Iteration Batch 494 — Research Loop: Information Retrieval, Text Mining, Question Answering

**Date**: 2026-09-06  
**Focus**: External research advances in IR, text mining/topic modeling, and QA  
**Method**: Web search → NeoTrix codebase gap analysis → defect identification → suggestions

---

## 1. Sources Cited

### Information Retrieval (8 sources)
| # | Paper | Venue | Key Advance |
|---|-------|-------|-------------|
| S1 | ByteX (ByteDance) | arXiv 2608.30607 | Trillion-vector scale unified AI search engine. SymRaBitQ quantization-aware vector kernel, hybrid SSD-resident storage, 3x throughput, 80% memory reduction, 86% cost reduction. |
| S2 | SERM (Self-Evolving Relevance Model) | ACL 2026 Findings | Multi-agent self-evolving relevance model. Agent-driven sample mining + two-level agreement annotation. +2.99 NDCG@1 after 3 iterations on industrial search (billions daily queries). |
| S3 | ProRetrieval | arXiv 2608.27017 | LM as retrieval orchestrator: synthesizes executable programs in hybrid DSL (SQL + vector primitives). 4B model surpasses GPT-5.5 on Hit@1 (0.81 vs 0.69 e-commerce). |
| S4 | ITER (Interaction-Aware Retrieval) | arXiv 2608.27912 | Dense retriever trained on agent interaction trajectories (main Q + preceding sub-queries). +7.5% InfoSeek-Eval, +13.5% BrowseComp-Plus over prior trajectory-trained retriever. |
| S5 | MIDR (Multimodal Document Retrieval) | EMNLP 2026 | Index-time multimodal reasoning: MLLM converts pages to verified text fields, indexed with BM25F + dense fusion. 23% nDCG gain over BM25, 9x smaller index than ColQwen2.5. |
| S6 | SPEAR | arXiv 2608.01738 | End-to-end query rewriting + retrieval with dual-embedding isolation + multiplicative gating + dynamic rewrite selector. +99.5% click recall@10, deployed in production. |
| S7 | STAIR (Structure-Aware IR) | arXiv 2609.03874 | ToC-based generative IR via Differentiable Search Index. Low hallucination (<0.05%), 82.6% Recall@1 on SearchTome. |
| S8 | OrLog (Neuro-Symbolic Retrieval) | arXiv 2601.23085 | Decouples predicate plausibility estimation (LLM) from logical reasoning (ProbLog). Handles disjunction/conjunction/negation. 90% token reduction vs monolithic LLM reasoning. |

### Text Mining / Topic Modeling (7 sources)
| # | Paper | Venue | Key Advance |
|---|-------|-------|-------------|
| S9 | "Towards Modern Topic Models" (Survey) | ACL 2026 Findings | Three-way taxonomy: Algorithm-Centric → LLM-Assisted → LLM-Centric. Agentic TM with iterative evaluation loops, knowledge base integration. |
| S10 | TriTopic | arXiv 2602.19079 | Tri-modal graph (semantic + lexical + metadata) + Consensus Leiden Clustering. NMI 0.575 vs BERTopic 0.510, 0% outliers vs BERTopic 18.8%. |
| S11 | COBWEBTM | ACL 2026 Findings | Lifelong hierarchical topic modeling via probabilistic concept formation. Online/streaming, no fixed topic count, no catastrophic forgetting. |
| S12 | PRISM | arXiv 2604.03180 | Student-teacher distillation: LLM supervision → lightweight encoder. Thresholded clustering with unassignment for precise narrative discovery. |
| S13 | MUDY | SIGIR 2026 | Multi-granular dynamic contextualization for keyphrase extraction. Candidate-aware weighting + multi-granular self-attention. +6.52% F1@5 on long docs with topic drift. |
| S14 | Label Semantic Expansion (LGNTM) | arXiv 2608.30216 | Label-aligned topic model enriching sparse label representations with corpus-grounded topic words. Bidirectional label↔topic alignment. |
| S15 | TIDE | arXiv 2601.11762 | LLM-based granular topic modeling framework. Multi-granularity topic decomposition for business applications. |

### Question Answering (7 sources)
| # | Paper | Venue | Key Advance |
|---|-------|-------|-------------|
| S16 | Lodestar | arXiv 2608.11922 | RL-trained prompt polarizer fixes entropy-based answer selection. Addresses "confidently wrong" failure mode where misleading passages drive entropy down. +3.71% F1 over entropy baseline. |
| S17 | Hi-Q | arXiv 2608.30468 | Evidence-conditioned hierarchical query refinement for multi-hop QA. Query tree topology driven by corpus support signals, not fixed decomposition. 52.3 EM on 3 benchmarks. |
| S18 | Inferential QA (QUIT dataset) | arXiv 2602.01239 | New QA task: infer answers from indirect clues rather than extract verbatim. Current retrievers/rerankers/LLMs all struggle on this task. |
| S19 | EnSI-RAG | arXiv 2608.21252 | Entity-Structure-Indexed RAG: query-independent entity-centered index with (entity, type, category, value) records. 78.24 avg accuracy on Loong+Oolong. |
| S20 | MINTQA | ACL 2026 | Multi-hop QA benchmark for new/long-tail knowledge. Sharp performance drops on new knowledge + multi-hop reasoning. Sub-question generation is key bottleneck. |
| S21 | LAKEQA | arXiv 2606.10460 | Exploratory QA over 9.5TB/40M file data lake. GPT-5.2 achieves only 18.37% EM. Main bottleneck: document discovery, not reasoning. |
| S22 | GEM (Generative Embedding Model) | arXiv 2608.13200 | Unifies generation + embedding in single model. Reasoning-augmented retrieval: reason about intent first, then encode for retrieval. Test-time compute scaling via prompting. |

---

## 2. Defects Found in NeoTrix Design

### DEFECT-IR-001: No Self-Evolving Relevance Model
**Severity**: HIGH  
**Evidence**: `nt_world_search.rs:184` — `WebSearchEngine::search()` takes a raw query string and passes it directly to DDG/Wikipedia. No relevance model, no feedback loop, no self-evolution. The `EvidenceScore` (line 71) is a static heuristic (selection×absorption+freshness+engine), not a learned relevance model.  
**Gap**: SERM (S2) demonstrates that self-evolving relevance models with multi-agent sample mining achieve +2.99 NDCG@1 over 3 iterations at industrial scale. NeoTrix has no mechanism to learn from query streams and improve retrieval quality over time.  
**Suggestion**: Implement `SelfEvolvingRelevance` in `nt_world_search` that: (1) tracks query→click/satisfaction signals, (2) uses multi-agent sample mining to identify informative samples (distribution shifts), (3) generates pseudo-labels via LLM agreement, (4) fine-tunes a lightweight relevance scorer. This aligns with SEAL's evolution philosophy.

### DEFECT-IR-002: No Hybrid Orchestration (Text + Structured + Vector)
**Severity**: HIGH  
**Evidence**: `nt_world_search.rs:284` — `SearchBackend` trait only defines `fn search(query, count) -> Vec<SearchResult>`. All backends return flat text results. No structured query synthesis, no Boolean logic composition, no vector+text hybrid fusion at query time.  
**Gap**: ProRetrieval (S3) shows that an LM synthesizing an executable program over SQL + vector primitives surpasses GPT-5.5 (0.81 vs 0.69 Hit@1). OrLog (S8) separates predicate plausibility from logical reasoning. NeoTrix's flat text interface cannot express "find documents about X AND published after Y OR from source Z."  
**Suggestion**: Add a `HybridQueryOrchestrator` trait to `nt_world_search` that: (1) decomposes natural language into a structured query plan (conjunction/disjunction/negation), (2) routes predicates to appropriate backends (BM25 for text, SQL for structured fields, dense for semantic), (3) fuses results via ProbLog-style probabilistic reasoning. This extends the existing `OrderedBackendRouter` pattern.

### DEFECT-IR-003: No Interaction-Aware / Trajectory-Aware Retrieval
**Severity**: MEDIUM  
**Evidence**: `nt_world_search.rs:836` — `UnifiedSearch::search()` treats every query independently. No state carries over between searches. `ConversationAwareness` (nt_core_aware:88) tracks topic drift but **does not feed this into search query reformulation**.  
**Gap**: ITER (S4) shows that incorporating main question + preceding sub-queries into the retriever improves agent-based search by 7.5-13.5%. NeoTrix's search is stateless per-query.  
**Suggestion**: Implement `TrajectoryAwareRetriever` in `nt_world_search` that maintains a session-level query trajectory. Each new search reformulates using: (1) current query, (2) previous searches in the session, (3) which results were consumed/rejected. Feed `ConversationAwareness::previous_topics` as signals.

### DEFECT-IR-004: No Multimodal Document Retrieval
**Severity**: MEDIUM  
**Evidence**: `nt_world_search.rs` only returns text snippets. `nt_memory_search.rs:11` uses FTS5 (text-only) + cosine similarity on text embeddings. No table/chart/figure extraction. No layout-aware document indexing.  
**Gap**: MIDR (S5) shows index-time multimodal reasoning achieves 23% nDCG gain and 9x smaller index than visual retrievers. NeoTrix's KB is purely text-based.  
**Suggestion**: Add `MultimodalIndexer` capability to `nt_memory` that: (1) uses a VLM to convert rendered pages into verified text fields (tables, charts, layout), (2) indexes via BM25F + dense fusion, (3) stores page-image patches for visual retrieval. This bridges the gap between NeoTrix's text-only KB and multimodal document understanding.

### DEFECT-IR-005: No End-to-End Query Rewriting with Retrieval Feedback
**Severity**: MEDIUM  
**Evidence**: The `OrderedBackendRouter` (nt_world_crawl) does backend selection but **no query rewriting**. The `WebSearchTool::search()` passes the raw user query directly to DDG. No attribution-guided rewriting, no intent disambiguation.  
**Gap**: SPEAR (S6) achieves +99.5% click recall by jointly optimizing query rewriting and retrieval. Attribution-guided rewriting (arXiv 2602.11841) shows consistent gains by using token-level retriever attributions to guide LLM reformulation.  
**Suggestion**: Add a `QueryRewriter` stage before backend routing that: (1) computes token-level attributions from the retriever, (2) uses an LLM to clarify weak/misleading tokens while preserving intent, (3) uses multiplicative gating to ensure the rewritten query stays faithful to the original. This plugs into the existing `WebSearchRouter::search()` flow.

### DEFECT-TM-006: No LLM-Centric Topic Modeling Capability
**Severity**: HIGH  
**Evidence**: `nt_core_aware/mod.rs:90` has `topic_coherence` and `topic_drift` fields, but these are **scalar metrics** — no actual topic model exists. `previous_topics` is `Vec<String>` with no hierarchical structure. No topic extraction, no topic evolution tracking, no LLM-assisted labeling.  
**Gap**: The ACL 2026 survey (S9) documents a paradigm shift from algorithm-centric to LLM-centric topic modeling. COBWEBTM (S11) enables lifelong streaming topic modeling. TriTopic (S10) achieves 0% outliers with tri-modal fusion. NeoTrix has zero topic modeling capability — only a naive drift metric.  
**Suggestion**: Implement `LifelongTopicModel` in `nt_mind` (self-evolution domain, appropriate for SEAL integration) that: (1) uses COBWEBTM-style incremental concept formation for streaming corpora, (2) supports LLM-assisted labeling via TopicGPT pattern, (3) feeds topic hierarchy into `ConsciousnessAwareness::previous_topics` as structured topic trees rather than flat strings, (4) integrates with SEAL pipeline for topic-aware evolution.

### DEFECT-TM-007: No Multi-Granular Keyphrase Extraction
**Severity**: MEDIUM  
**Evidence**: No keyphrase extraction module exists anywhere in the codebase. `EvidenceScore` (nt_world_search.rs:85) has crude feature flags (`has_numbers`, `has_definition`, `has_comparison`, `has_disclose`) but no semantic keyphrase extraction.  
**Gap**: MUDY (S13) achieves +6.52% F1@5 over SOTA by combining prompt-based scoring with multi-granular self-attention for keyphrase extraction. Critical for long documents with topic drift.  
**Suggestion**: Implement `KeyphraseExtractor` in `nt_memory` that uses MUDY's candidate-aware weighting + multi-granular attention. Feed extracted keyphrases into: (1) `EvidenceScore` as semantic density features, (2) KB node tagging for improved FTS5 search, (3) topic model seeding.

### DEFECT-TM-008: No Tri-Modal Topic Fusion (Semantic + Lexical + Metadata)
**Severity**: MEDIUM  
**Evidence**: `Bm25Index` (nt_core_bank/iteration.rs:33) operates on tokenized text only. `cosine_similarity` (nt_memory_embed) uses dense embeddings only. No fusion of lexical precision with semantic abstraction.  
**Gap**: TriTopic (S10) proves that fusing semantic + lexical + metadata into a heterogeneous information network achieves NMI 0.575 vs BERTopic's 0.510, with 100% corpus coverage. NeoTrix's BM25 and embedding search are completely independent — no fusion.  
**Suggestion**: Extend `nt_memory_search` with a `FusedRetrieval` that combines: (1) BM25 lexical scores, (2) dense embedding cosine similarity, (3) metadata-based features (node_type, domain, temporal). Use weighted linear combination with learned weights (α·BM25 + β·dense + γ·metadata) and RRF for late fusion.

### DEFECT-QA-009: No Multi-Hop Reasoning / Query Decomposition
**Severity**: HIGH  
**Evidence**: `nt_core_task_dispatcher.rs:536` has a simple heuristic: `if lower.contains("search") || lower.contains("research")` → route to research. No question decomposition, no iterative retrieval, no evidence aggregation across hops. `nt_core_plan.rs:50` has `question: String` but it's a plan tracker, not a QA reasoning engine.  
**Gap**: Hi-Q (S17) achieves 52.3 EM on multi-hop QA by growing a query tree driven by corpus support signals. MINTQA (S20) reveals that sub-question generation is the key bottleneck in multi-hop QA. NeoTrix cannot decompose a complex question into sub-questions and iteratively retrieve evidence.  
**Suggestion**: Implement `MultiHopReasoner` in `nt_core` that: (1) decomposes complex queries into a query tree (Hi-Q pattern), (2) for each node, retrieves evidence and checks semantic coverage, (3) expands unresolved nodes via dependency-preserving binary decomposition, (4) aggregates evidence into a final answer. Feed results into GWT for attention routing.

### DEFECT-QA-010: No Entity-Structure-Indexed Retrieval for Long Documents
**Severity**: MEDIUM  
**Evidence**: KB nodes (`KnowledgeNode` in nt_memory_types) have flat `content` field. No entity-centric indexing, no relation extraction, no structured evidence records. Long document retrieval relies on chunk-based FTS5 which loses entity-relation structure.  
**Gap**: EnSI-RAG (S19) achieves 78.24 accuracy by constructing query-independent entity-centered indices with (entity, type, category, value) records. LAKEQA (S21) shows that document discovery is the main bottleneck — models fail to find relevant documents in large collections.  
**Suggestion**: Add `EntityStructureIndex` to `nt_memory` that: (1) extracts entities and relations during KB ingestion, (2) creates (entity, type, category, value) records linked to source passages, (3) serves as an additional retrieval handle alongside FTS5 and dense search.

### DEFECT-QA-011: No Entropy-Based Answer Validation / Confidence Calibration
**Severity**: MEDIUM  
**Evidence**: No answer confidence estimation exists in the codebase. When multiple search results return conflicting information, there is no mechanism to determine which is trustworthy. `EvidenceScore` is a pre-retrieval heuristic, not a post-retrieval answer validation.  
**Gap**: Lodestar (S16) shows that entropy-based selection improves F1 by +3.71% over top-ranked passage, and that misleading passages drive entropy **down** (confidently wrong). The polarizer approach fixes this without retraining.  
**Suggestion**: Implement `AnswerConfidenceCalibrator` in `nt_core` that: (1) for each candidate answer, measures entropy of the respondent model's output distribution, (2) detects misleading passages (low entropy + wrong answer), (3) uses a learned polarizer string to steer entropy in the correct direction. This is a lightweight post-retrieval validation layer.

### DEFECT-QA-012: No Inferential Question Answering Capability
**Severity**: LOW  
**Evidence**: NeoTrix's search is purely extractive — match query terms to documents. No capability to infer answers from indirect evidence or background clues.  
**Gap**: Inferential QA (S18) reveals that current QA pipelines fail on questions where answers must be inferred from clues rather than extracted verbatim. Even reasoning-oriented LLMs don't outperform smaller models on this task.  
**Suggestion**: While full inferential QA is research-stage, add a `InferentialRetrieval` mode to `nt_world_search` that: (1) detects when a query cannot be directly matched to document content, (2) expands retrieval to include semantically related but not lexically matching documents, (3) uses chain-of-thought prompting to infer answers from indirect evidence.

---

## 3. Cross-Cutting Defects

### DEFECT-XC-013: Search ↔ Awareness Disconnect
**Severity**: HIGH  
**Evidence**: `ConsciousnessAwareness` (nt_core_aware:52) has `topic_coherence`, `topic_drift`, `previous_topics` — but these are **never consumed** by `UnifiedSearch`. The search system is completely unaware of the conversation's topic trajectory. GWT (Global Workspace Theory) routes attention but **does not route search queries**.  
**Gap**: All 2026 IR research converges on the principle that retrieval should be context-aware (ITER's trajectory-aware retrieval, SERM's stream-aware adaptation, SPEAR's personalized rewriting). NeoTrix's search is context-free.  
**Suggestion**: Create a `SearchConsciousnessBridge` that: (1) feeds `ConversationAwareness` state into `UnifiedSearch::search()` as context, (2) uses GWT attention scores to prioritize which search results to surface, (3) uses topic drift signals to trigger query reformulation.

### DEFECT-XC-014: No Unified Query Understanding Pipeline
**Severity**: HIGH  
**Evidence**: Query handling is fragmented: `WebSearchEngine` does raw DDG queries, `WebSearchRouter` does backend selection, `Bm25Index` does text matching, `nt_memory_search` does FTS5 + embedding. No unified query understanding that jointly handles intent classification, entity extraction, constraint parsing, and query rewriting.  
**Gap**: LinkedIn's unified query understanding (KDD 2026, arXiv 2605.27441) consolidates routing/tagging/rewriting/facets into a single SLM, reducing failures by 17% and improving engagement. NeoTrix's fragmented approach mirrors the pre-2026 architecture that industry has moved away from.  
**Suggestion**: Implement `UnifiedQueryUnderstanding` in `nt_world_search` as a single pipeline that: (1) classifies query intent (factual/navigational/explorational/inferential), (2) extracts entities and constraints, (3) determines optimal backend(s), (4) rewrites query for the selected backend, (5) routes to the appropriate `SearchBackend`. This replaces the current ad-hoc routing.

---

## 4. Summary

| Category | Defects | Critical | High | Medium | Low |
|----------|---------|----------|------|--------|-----|
| Information Retrieval | 5 | 0 | 3 | 2 | 0 |
| Text Mining | 3 | 0 | 1 | 2 | 0 |
| Question Answering | 4 | 0 | 1 | 2 | 1 |
| Cross-Cutting | 2 | 0 | 2 | 0 | 0 |
| **Total** | **14** | **0** | **7** | **6** | **1** |

### Priority Recommendations

1. **DEFECT-IR-001** (Self-Evolving Relevance) — Aligns with SEAL pipeline evolution philosophy. Implement first.
2. **DEFECT-XC-014** (Unified Query Understanding) — Foundational; all other IR improvements depend on this.
3. **DEFECT-XC-013** (Search ↔ Awareness Bridge) — Enables GWT-routed retrieval, core NeoTrix differentiator.
4. **DEFECT-TM-006** (Lifelong Topic Modeling) — Integrates with NT-MIND SEAL domain; enables topic-aware evolution.
5. **DEFECT-QA-009** (Multi-Hop Reasoning) — Critical for complex task handling; extends existing plan system.

### Key External Advances Not Captured by NeoTrix

- **Self-evolution of retrieval** (SERM): NeoTrix's SEAL could evolve its own relevance models, but doesn't.
- **Executable program synthesis for retrieval** (ProRetrieval): NeoTrix treats search as text-in/text-out; modern systems synthesize programs.
- **Agent trajectory awareness** (ITER): NeoTrix's multi-turn conversations have no search trajectory memory.
- **Lifelong topic modeling** (COBWEBTM): NeoTrix's topic tracking is a flat scalar, not a growing hierarchy.
- **Confidently-wrong detection** (Lodestar): NeoTrix has no mechanism to detect when search results are confidently misleading.
