# Iteration Batch 368 — IR/Ranking/QA Research Loop

**Date**: 2026-09-06
**Focus**: Information Retrieval, Learning-to-Rank, Multi-hop QA & RAG

---

## Sources Cited

| # | Source | Year | Topic |
|---|--------|------|-------|
| 1 | Li Y. "Understanding and Enhancing Robustness in Dense Information Retrieval" ECIR 2026 | 2026 | Dense retrieval robustness |
| 2 | Saxena et al. "IMRNNs: Interpretable Dense Retrieval via Embedding Modulation" EACL 2026 | 2026 | Interpretability in dense retrieval |
| 3 | Wischounig et al. "Negative Sampling Techniques in Dense Retrieval: A Survey" EACL 2026 | 2026 | Negative sampling taxonomy |
| 4 | DREAM: Dense Retrieval Embeddings via Autoregressive Modeling arXiv 2606.24667 | 2026 | LLM-based embedding generation |
| 5 | Qwen3-Reranker series (0.6B/4B/8B) — SiliconFlow/Redis/DataAspirant | 2026 | Cross-encoder reranking |
| 6 | Liu et al. "Think Parallax: Multi-View KG-RAG" ACL 2026 | 2026 | Multi-hop reasoning, head-specific retrieval spaces |
| 7 | Man et al. "State-Aware RAG: Adaptive Information Management" ACL Findings 2026 | 2026 | Dynamic working memory for multi-hop |
| 8 | Kim et al. "TH-RAG: Topic-Based Hierarchical KGs" ACL 2026 | 2026 | Hierarchical knowledge graph for RAG |
| 9 | RT-RAG: Reasoning Tree Guided RAG arXiv 2601.11255 | 2026 | Hierarchical reasoning structure |
| 10 | EKA: Multi-hop via Early Knowledge Alignment arXiv 2512.20144 | 2025-2026 | RL-augmented iterative retrieval |
| 11 | SA-RAG: Structured and Adaptive RAG ScienceDirect 2026 | 2026 | Adaptive strategies for multi-hop |

---

## Defects Found

### DEFECT-368-1: No Interpretable Retrieval Modulation (CRITICAL)

**Source**: IMRNNs (Saxena et al., EACL 2026)

**Finding**: IMRNNs demonstrate that static embeddings in dense retrievers obscure bidirectional query-document semantic alignment. Dynamic, per-query modulation adapters (+6.35% nDCG, +7.14% recall) make retrieval interpretable AND more effective.

**NeoTrix Gap**: `kb_vector_index.rs:13-24` implements static cosine similarity on fixed embeddings with zero bidirectional modulation. The `FloatVec::distance()` is purely dot-product based with no query-conditioning. There is no mechanism to dynamically modulate document embeddings based on the incoming query at inference time.

**Impact**: NeoTrix cannot explain WHY a document was retrieved, making the "Evidence-First" review methodology impossible for retrieval decisions. The GWT attention system receives retrieval scores without interpretability signals.

**Fix**: Add a `ModulatingRetriever` adapter that wraps `KbVectorIndex`:
- Per-query adapter: conditions document embeddings on the current query before similarity computation
- Per-corpus feedback adapter: uses initially retrieved documents to refine the query embedding
- Output: retrieval explanations as structured `(reason, weight)` alongside scores

---

### DEFECT-368-2: Heuristic-Only Query Classification (HIGH)

**Source**: State-Aware RAG (Man et al., ACL 2026); TH-RAG (Kim et al., 2026)

**Finding**: State-Aware RAG achieves +8.6% over memory-augmented baselines by maintaining an explicit dynamic working memory with Path-Outcome Dual Reward. TH-RAG uses topic-based hierarchical KGs to handle fragmented triplet graphs.

**NeoTrix Gap**: `nt_memory_adaptive_rag.rs:124-162` — `heuristic_classify()` uses only keyword pattern matching ("vs ", "why ", "and ") and capitalized word counting for entity detection. No working memory state is maintained across reasoning hops. The `count_entities()` at line 153 fails on lowercase technical terms, CJK text, and compound identifiers (e.g., `nt_core_self`).

**Impact**: Multi-hop queries involving technical NeoTrix terms (all `nt_*` prefixed, lowercase) are misclassified as Simple, leading to single-pass retrieval that misses cross-domain evidence chains.

**Fix**: Replace heuristic classifier with a lightweight embedding-based classifier or integrate with GWT's `SelectiveState` to leverage attention signals for complexity estimation. Add a `WorkingMemory` state object that persists intermediate reasoning across retrieval iterations.

---

### DEFECT-368-3: No Cross-Encoder Reranking Layer (HIGH)

**Source**: Redis/DataAspirant/SiliconFlow 2026 reranking surveys; Nemorize 2026 RAG Roadmap

**Finding**: The 2026 consensus is a 2-stage architecture: bi-encoder retrieval (high recall) followed by cross-encoder reranking (high precision). Qwen3-Reranker-8B achieves SOTA by jointly attending to query+document tokens. Cross-encoders lift rank-8 passages to rank-1 in real-world scenarios.

**NeoTrix Gap**: `nt_memory_search.rs:259-280` — `hybrid_search()` fuses FTS5 + BM25 via Reciprocal Rank Fusion but has NO cross-encoder reranking stage. The `rerank_weight_fts` and `rerank_weight_embed` config fields in `AdaptiveRagConfig:61-62` are dead configuration — they are never used in any reranking computation. The fusion is purely score-based, not relevance-jointly-modeled.

**Impact**: Top-K retrieval may bury the correct answer (e.g., at rank 8) under keyword-matching noise, exactly the scenario documented in DataAspirant 2026. The `max_iterations: 3` in adaptive RAG compensates by re-retrieving, but never reranks the shortlist.

**Fix**: Add a `CrossEncoderReranker` stage between retrieval and generation:
- Use an ONNX-quantized cross-encoder (e.g., MiniLM-L-6 or Qwen3-Reranker-0.6B)
- Operate on top-50 candidates from hybrid search
- Output reranked top-10 with joint query-document relevance scores
- Wire into `AdaptiveRetrieval::execute_pipeline` as Stage 2.5

---

### DEFECT-368-4: Flat Embedding Space for Multi-Hop Reasoning (HIGH)

**Source**: Think Parallax (Liu et al., ACL 2026)

**Finding**: ParallaxRAG identifies that "multi-hop reasoning is inherently multi-view" — Transformer attention heads specialize in distinct semantic relations across reasoning stages. Collapsing all hops into a single flat embedding space suppresses this structure, causing noisy/drifted path exploration.

**NeoTrix Gap**: `kb_vector_index.rs` and the entire `nt_memory_search` pipeline operate on a single flat vector space. There is no hop-aligned or relation-specific retrieval. The `KbVectorIndex::search()` at line 70 performs a single ANN query regardless of the number of reasoning hops needed. The E8 Hexagram reasoning engine has 64 states but no mechanism to condition retrieval on the current reasoning state.

**Impact**: When answering questions requiring 2+ hops (e.g., "What domain does the module that implements State-Aware RAG belong to in NeoTrix?"), the system retrieves flat semantic neighbors rather than hop-specific evidence, leading to incomplete reasoning chains.

**Fix**: Implement multi-view retrieval aligned with E8 reasoning states:
- Map E8 hexagram states to retrieval head spaces (each head specializes in different relation types)
- For multi-hop queries, decompose into hop-specific sub-queries
- Retrieve per-hop, then join with graph traversal (`get_related` at line 174)
- This directly leverages the existing E8 infrastructure for retrieval routing

---

### DEFECT-368-5: No Dynamic Working Memory for Reasoning Chains (MEDIUM)

**Source**: State-Aware RAG (Man et al., 2026); RT-RAG (arXiv 2601.11255)

**Finding**: State-Aware RAG maintains an explicit working memory that serves as a dynamic cognitive workspace. RT-RAG uses hierarchical tree structures with parent-child relationships for multi-hop reasoning. Both achieve SOTA by managing intermediate reasoning state explicitly.

**NeoTrix Gap**: The `AdaptiveRetrieval` struct at `nt_memory_adaptive_rag.rs:84-87` only caches query complexity classification (LRU cache of 200 entries). There is no working memory for intermediate reasoning states, no reasoning tree, and no mechanism to consolidate evidence across iterations. The `max_iterations: 3` loop re-retrieves from scratch each time without building on prior findings.

**Impact**: Each iteration of adaptive retrieval is stateless — it cannot say "I found X in hop 1, now I need Y that relates to X." This wastes retrieval budget and produces fragmented evidence.

**Fix**: Add a `ReasoningWorkspace` struct:
```rust
pub struct ReasoningWorkspace {
    gathered_evidence: Vec<GradedDocument>,
    reasoning_tree: Vec<HopNode>,  // parent-child subproblem decomposition
    current_hop: usize,
    confidence_trajectory: Vec<f64>,
}
```
Wire into `execute_pipeline` to accumulate evidence across iterations.

---

### DEFECT-368-6: No Robustness Against Adversarial/OOD Queries (MEDIUM)

**Source**: Li Y. ECIR 2026; ACM TOIS 2026 "Robust Neural Information Retrieval"

**Finding**: Dense retrieval models are vulnerable to adversarial perturbations (typos, OOD distributions, performance variance). ECIR 2026 provides systematic robustness enhancement techniques.

**NeoTrix Gap**: `search_fts` at `nt_memory_search.rs:11-136` has no adversarial robustness layer. FTS5 `MATCH` syntax is vulnerable to injection via special characters (`*`, `"`, `OR`, `NEAR`). The `heuristic_classify` in adaptive RAG treats short queries (<5 chars) as Reject without sanitization. No OOD detection exists for embeddings.

**Impact**: Malformed queries could cause FTS5 crashes or injection. Out-of-domain queries produce garbage retrieval without warning.

**Fix**: 
- Sanitize FTS5 query input (escape special chars, enforce query length bounds)
- Add OOD detection on query embeddings (Mahalanobis distance from centroid of known distribution)
- Implement fallback to BM25-only when embedding confidence is low

---

### DEFECT-368-7: Missing Negative Sampling Strategy for Embedding Training (MEDIUM)

**Source**: Wischounig et al. "Negative Sampling Techniques in Dense Retrieval: A Survey" EACL 2026

**Finding**: The survey of 35 papers shows negative sampling is central to dense retriever training quality. LLM-driven synthetic negatives represent a new frontier. Dynamic hard-negative mining significantly outperforms random negatives.

**NeoTrix Gap**: The KB embedding pipeline (`nt_memory_embed`) uses no documented negative sampling strategy. The `Bm25Index::build` at `bm25.rs:39-77` treats all documents uniformly with `recall_weight: 1.0`. No contrastive training loop exists for domain-specific embedding fine-tuning on NeoTrix's own KB data.

**Impact**: Embeddings are not optimized for NeoTrix's domain-specific vocabulary (nt_* modules, E8, GWT, VSA), leading to suboptimal retrieval precision for project-specific queries.

**Fix**: Implement a contrastive fine-tuning pipeline:
- Mine hard negatives from BM25-high-score-but-low-semantic-match pairs
- Use LLM-generated synthetic negatives for domain-specific terms
- Fine-tune embeddings on NeoTrix KB data quarterly

---

### DEFECT-368-8: No Hierarchical Knowledge Graph for Multi-Hop (LOW)

**Source**: TH-RAG (Kim et al., ACL 2026)

**Finding**: TH-RAG organizes fragmented triplets into subtopics and topics, enhancing connectivity and supporting robust multi-hop reasoning. This addresses the fundamental limitation of flat KG representations.

**NeoTrix Gap**: The KB graph (`nt_memory_graph.rs`, `get_related` at search.rs:174) operates on flat edge traversal with `weight DESC` ordering. No hierarchical topic organization exists. The `edges` table has `relation_type` but no topic/subtopic hierarchy.

**Impact**: Graph traversal for multi-hop reasoning follows flat adjacency rather than hierarchical topic structure, leading to noisy paths.

**Fix**: Add a `topic_hierarchy` table and route graph traversal through topic boundaries. This complements the proposed E8 multi-view retrieval (DEFECT-368-4).

---

## Summary

| Severity | Count | Key Themes |
|----------|-------|------------|
| CRITICAL | 1 | No interpretable retrieval modulation |
| HIGH | 3 | Heuristic-only classification, no cross-encoder reranking, flat embedding for multi-hop |
| MEDIUM | 3 | No working memory, no adversarial robustness, no negative sampling |
| LOW | 1 | No hierarchical KG |

**Total defects**: 8
**Recommended priority**: Fix DEFECT-368-3 (cross-encoder reranking) first — highest ROI, most aligned with 2026 industry consensus. Then DEFECT-368-2 + DEFECT-368-5 (working memory + better classification) together as they are coupled.
