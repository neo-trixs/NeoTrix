# Iteration Batch 472 — NLP Research Gap Analysis

**Date**: 2026-09-06
**Focus**: Text Summarization, Information Extraction, Knowledge Base Population
**Method**: External research search → current codebase audit → defect identification

---

## 1. Sources Cited

### Text Summarization
1. **MASF: Multi-Model Adaptive Selection Framework** (arXiv:2606.05494, Jun 2026) — Multi-model ensemble for abstractive summarization, selecting best model per article.
2. **MAMBA-Transformer Hybrids for Extractive Summarization** (arXiv:2603.01288, Mar 2026) — Linear-time state-space models (Mamba) hybridized with transformers for long-document extractive summarization without truncation.
3. **Hybrid Extractive-Abstractive Summarization via Deep Clustering** (Alexandria Engineering Journal, Feb 2026) — Deep clustering-based extractive scoring + attention-guided abstractive generator for scientific texts.
4. **RL-Optimized Abstractive Summarization** (J. Supercomputing, Apr 2026) — Reinforcement learning for summary length and semantics optimization.
5. **TriFusionRank: Multi-Factor Sentence Scoring** (J. Intelligent & Fuzzy Systems, May 2026) — Lightweight extractive summarization with TF-IDF + semantic similarity + positional weighting fusion.
6. **Understanding LLM Reasoning for Abstractive Summarization** (ACL Findings 2026) — Cite, QAG, Plan, SC strategies adapted for abstractive summarization via prompts.
7. **Current Trends in Extractive Text Summarization** (IEEE Access, Feb 2025/2026) — Comprehensive review of 145 papers; highlights stopping criteria, multimodal expansion, evaluation metrics.
8. **Abstractive Summarizers are Excellent Extractive Summarizers** (ACL 2023) — Single model producing both extractive and abstractive summaries, questioning extractive-only design.

### Information Extraction
9. **Assessment of Generative NER in the Era of LLMs** (arXiv:2601.17898, Jan 2026) — NER evolving from sequence labeling to generative paradigm; +32 F1 points with entity understanding enhancement.
10. **ReCoT-NER: Zero-Shot NER via Chain-of-Thought** (ACL Findings 2026) — 77M-parameter model achieving competitive zero-shot NER with CoT prompting + recall-oriented loss.
11. **From RE to Triplet Extraction: Low-Resource Survey** (Computer Science Review, Jun 2026) — Shift from RE (pair classification) to TE (joint subject-predicate-object generation); generative frameworks dominating.
12. **ERE-LLM: Entity-Relation Joint Extraction** (IEEE, 2026) — Three-stage (Germination→Growth→Maturation) LLM-based joint entity-relation extraction for specialized fields.
13. **Cutting-Edge RE Techniques Survey** (AI Review, Jul 2025, 137 papers) — BERT-based dominance; LLMs (T5) excelling in few-shot RE; document-level RE gaining traction.
14. **Low-Resource Joint Entity and RE Using LLMs** (ACM TKDD, Mar 2026) — Semi-supervised LLM approach for joint NER+RE in low-resource settings.

### Knowledge Base Population
15. **LELA: LLM-based Entity Linking with Zero-Shot Domain Adaptation** (arXiv:2601.05192, Jan 2026; IJCAI demo 2026) — Coarse-to-fine pipeline: candidate generation → pointwise re-ranking → LLM reasoning for final selection. True zero-shot, no fine-tuning needed.
16. **MHEL-LLaMo: Multilingual Historical Entity Linking** (EACL 2026) — Unsupervised ensemble: SLM confidence scoring for easy cases + LLM for hard cases; 6 languages, no fine-tuning.
17. **Universal Entity Linking** (ScienceDirect, Dec 2025) — LLM + embedding model + vector store; DPO training for disambiguation; extension to new KBs.
18. **LLM-based KG Construction** (Nature Scientific Reports, Feb 2026) — Multimodal knowledge integration: rule-based + ontological + LLM; Knowledge Routing Network + hierarchical LoRA.
19. **Shadowfax: KBP from Text** (SIGIR 2024) — Interactive platform for knowledge base population; entity disambiguation + relation extraction pipeline.
20. **Joint Inference for End-to-End KBP** (Springer, May 2026) — Joint NER + coreference + RE + entity linking; pipeline vs joint trade-offs.
21. **DeepKE: Knowledge Extraction Toolkit** (EMNLP 2022, extended) — Open-source toolkit: NER + RE + attribute extraction; low-resource, document-level, multimodal.
22. **Portuguese OIE Benchmarking** (LREC 2026) — Rule-based systems strong on general text but fail on specialized corpora; F1 ~40% average.
23. **Normalization-First OIE Framework** (ACL SRW 2026) — Argues OIE outputs are semantically unsound; proposes normalization-first paradigm.

---

## 2. Defects Found in NeoTrix Design

### D472-1: Pattern-Based Entity Extraction (Critical)
**Location**: `nt_world_semantic_extract.rs:23-35`, `EntityExtractor`
**Gap**: NeoTrix uses regex/keyword patterns (`"Mr."`, `"Dr."`) for NER. 2026 research shows generative NER with LLMs achieves +32 F1 points (arXiv:2601.17898). Even lightweight 77M CoT models outperform pattern-based approaches (ACL 2026). The `PatternType::NER` variant exists but has no transformer backbone.
**Impact**: Entity extraction quality severely limited for domain-specific, multilingual, or nested entity scenarios. Cannot handle "continued entities" (multi-span) or open-vocabulary entities.

### D472-2: No Entity Linking (Critical)
**Location**: `nt_world_semantic_extract.rs` — entity types defined but no linking step
**Gap**: NeoTrix extracts entities but never links them to a canonical KB (Wikidata, internal KB). 2026 work (LELA, MHEL-LLaMo, Universal EL) shows true zero-shot entity linking is now practical via LLM reasoning. Without entity linking, the KB accumulates duplicate/ambiguous entities.
**Impact**: Knowledge graph suffers from entity fragmentation. "Apple" (company) vs "apple" (fruit) vs "Apple" (record label) all stored as separate nodes. KB population quality degrades.

### D472-3: Binary Relation Extraction Only (High)
**Location**: `nt_world_semantic_extract.rs:82-98`, `RelationExtractor`
**Gap**: NeoTrix extracts only binary (subject, relation, object) tuples. 2026 survey (Computer Science Review, Jun 2026) shows the field shifting to Triplet Extraction (TE) — joint subject-predicate-object generation. Additionally, open n-ary extraction (ONIEX, 2025) handles relations involving 3+ entities (e.g., "Microsoft was founded by Bill Gates and Paul Allen in 1975").
**Impact**: Cannot capture complex multi-entity facts. Loses information in sentences like "X acquired Y for $Z in date D" where 4 entities participate.

### D472-4: No Open Information Extraction (High)
**Location**: `nt_world_semantic_extract.rs` — no OIE component
**Gap**: NeoTrix's `RelationPattern` requires predefined relation types. Open IE systems (TextRunner → ReVerb → OLLIE → LLM-based) extract relations without predefined schemas. 2026 LREC benchmark shows even ~40% F1 OIE is useful for KB population. NeoTrix has no mechanism for schema-free relation discovery.
**Impact**: Cannot discover new relation types from crawled content. Every new domain requires manual pattern engineering.

### D472-5: No Text Summarization Module (High)
**Location**: No summarization code found in codebase
**Gap**: Zero summarization capability. 2026 state-of-the-art includes: hybrid extractive-abstractive (Alexandria Eng J), multi-model adaptive selection (MASF), MAMBA-transformer hybrids for long docs, RL-optimized length/semantics. The SEAL pipeline distills session patterns but never summarizes crawled documents.
**Impact**: Knowledge distillation operates on raw text without compression. Long documents (papers, articles) not condensed before KB storage, wasting storage and reducing retrieval quality.

### D472-6: No Joint NER+RE Pipeline (Medium)
**Location**: `EntityExtractor` and `RelationExtractor` are separate structs with no shared state
**Gap**: 2026 research (ERE-LLM, Joint Inference for KBP) shows joint models avoid error propagation from pipeline approaches. NeoTrix's sequential entity→relation pipeline suffers from cascading failures — if entity extraction misses an entity, relation extraction cannot recover.
**Impact**: Estimated 15-25% relation recall loss from pipeline error propagation.

### D472-7: No Continual/Lifelong NER (Medium)
**Location**: `EntityExtractor` has fixed `patterns: Vec<EntityPattern>`
**Gap**: 2025 survey (AI Review) identifies continual learning NER as a major frontier. NeoTrix cannot learn new entity types incrementally without retraining. The `KnowledgeMiner` adds new sources but entity patterns remain static.
**Impact**: As NeoTrix encounters new domains (e.g., legal, biomedical, financial), it cannot adapt its entity recognition without manual pattern updates.

### D472-8: No Multimodal Knowledge Integration (Medium)
**Location**: `nt_world_semantic_extract.rs` — text-only pipeline
**Gap**: 2026 Nature paper shows multimodal KG construction combining rule-based + ontological + LLM from diverse data sources. NeoTrix's pipeline processes text only; cannot extract entities from images, tables, code blocks, or audio transcriptions.
**Impact**: Misses entity information in diagrams, architecture images, code structures, and tabular data that are common in technical documentation.

### D472-9: Knowledge Distiller Uses Keyword Heuristics (Medium)
**Location**: `knowledge_distiller.rs:133-147`, Rule 3: "should always"
**Gap**: Principle extraction from sessions relies on string matching (`"should always"`, `"refactor"`, `"optimize"`). 2026 RE research shows LLM-based extraction with CoT prompting far exceeds keyword heuristics.
**Impact**: High-quality user instructions that don't match keyword patterns are silently dropped. Estimated 40-60% of actionable principles lost.

### D472-10: No Evaluation Metrics for Extraction Quality (Low)
**Location**: No ROUGE, F1, precision/recall computation found in extraction pipeline
**Gap**: 2026 IEEE review highlights that evaluation metrics (ROUGE, BERTScore, MoverScore) are critical for summarization quality. NeoTrix has no mechanism to measure extraction or summarization quality.
**Impact**: Cannot detect degradation in extraction quality over time. No feedback loop for extraction model improvement.

---

## 3. Suggestions

### S472-1: Integrate LLM-Based NER (Priority: P0)
Replace pattern-based `EntityExtractor` with an LLM-backed extractive NER. Use CoT prompting approach from ReCoT-NER (ACL 2026) — lightweight 77M model with recall-oriented loss. For zero-shot domains, route through LLM reasoning. Keep pattern-based approach as fast-path fallback for known entity types.

### S472-2: Add Entity Linking Stage (Priority: P0)
Implement LELA-style coarse-to-fine entity linking: (1) candidate retrieval via FAISS/BM25 against internal KB, (2) pointwise re-ranking, (3) LLM reasoning for disambiguation. True zero-shot — no domain-specific training required. Wire into `nt_world_semantic_extract.rs` pipeline after entity extraction.

### S472-3: Upgrade to Triplet Extraction (Priority: P1)
Extend `RelationExtractor` from binary to open triplet (subject, predicate, object) generation. Use generative approach (T5/BERT-based) per 2026 survey. Support n-ary relations for multi-entity facts. Add canonicalization step to normalize relation phrases.

### S472-4: Add Open IE Module (Priority: P1)
Implement or integrate an Open IE system (e.g., Adapted ReVerb/OLLIE patterns or LLM-based OIE). Allow schema-free relation discovery from crawled content. Post-process with canonicalization to reduce redundancy. Store discovered relations in KB with confidence scores.

### S472-5: Add Document Summarization (Priority: P1)
Implement hybrid extractive-abstractive summarization for crawled documents before KB storage. Extractive phase: TriFusionRank-style multi-factor scoring (TF-IDF + semantic + positional). Abstractive phase: Use LLM to rephrase extracted sentences. For long documents (>4K tokens), use MAMBA-transformer hybrid approach. Store summary alongside full text in KB.

### S472-6: Unify NER+RE into Joint Model (Priority: P2)
Replace separate `EntityExtractor` + `RelationExtractor` with a joint model. Use ERE-LLM's three-stage approach (Germination→Growth→Maturation) or a T5-based joint extraction model. Shared encoder eliminates error propagation. Update `SemanticExtractionPipeline` to use single model call.

### S472-7: Add Continual NER Learning (Priority: P2)
Implement class-incremental NER via knowledge distillation (ExtendNER/AddNER from AAAI 2021). When new entity patterns are discovered, expand classifier dimension without forgetting old types. Store per-domain NER adapters alongside main model.

### S472-8: Add Extraction Quality Metrics (Priority: P2)
Implement ROUGE/BERTScore computation for summaries. Add precision/recall/F1 tracking for entity and relation extraction against gold standards (when available). Wire into ConsciousnessTree health monitoring via HeartbeatAggregator. Track quality trends over time.

### S472-9: Upgrade Knowledge Distiller to LLM-Based (Priority: P3)
Replace keyword heuristics in `knowledge_distiller.rs` with LLM-based principle extraction using Chain-of-Thought prompting. Use the E8 hexagram reasoning to select extraction strategy per session type. Retain keyword approach as fast-path for simple patterns.

### S472-10: Add Multimodal Entity Extraction (Priority: P3)
Extend extraction pipeline to handle: (1) code blocks → extract functions/modules/types as entities, (2) tables → extract rows/columns as structured facts, (3) images → use vision-language models for caption-based entity extraction. Store multimodal entities with source modality tag.

---

## 4. Architecture Impact Matrix

| Defect | Severity | NT-WORLD | NT-MEMORY | NT-MIND | NT-ACT | NT-CORE |
|--------|----------|----------|-----------|---------|--------|---------|
| D472-1 Pattern NER | Critical | X | X | | | |
| D472-2 No Entity Linking | Critical | X | X | | | |
| D472-3 Binary RE | High | X | X | | | |
| D472-4 No Open IE | High | X | X | | | |
| D472-5 No Summarization | High | X | X | X | | |
| D472-6 No Joint NER+RE | Medium | X | | | | |
| D472-7 No Continual NER | Medium | X | | X | | |
| D472-8 No Multimodal | Medium | X | | | | |
| D472-9 Keyword Distiller | Medium | | | X | X | |
| D472-10 No Eval Metrics | Low | X | X | X | | X |

## 5. Priority Queue

1. **P0** — D472-1 + D472-2 (NER upgrade + Entity Linking) — Foundation for all downstream KB quality
2. **P1** — D472-3 + D472-4 + D472-5 (Triplet Extraction + Open IE + Summarization) — Core IE pipeline
3. **P2** — D472-6 + D472-7 + D472-10 (Joint model + Continual learning + Metrics) — Quality assurance
4. **P3** — D472-8 + D472-9 (Multimodal + LLM Distiller) — Future expansion
