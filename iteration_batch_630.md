# Iteration Batch 630 — Knowledge Graph Construction, Entity Linking, Relation Extraction

**Date**: 2026-09-06
**Context**: Post-Batch-629 (tiktoken O(n²), token counting drift, no AST parsing, no NFKC, no post-normalization)
**Focus**: External research survey — KG construction, entity linking, relation extraction (2026 SOTA)

---

## 1. Knowledge Graph Construction

### 1.1 AutoSchemaKG (ACL 2026)
- **Source**: https://aclanthology.org/2026.acl-long.942.pdf
- **Key**: Fully autonomous KG construction — no predefined schemas. LLM extracts triples AND induces schemas simultaneously. 900M+ nodes, 5.9B edges. 92% semantic alignment with human schemas. **Event nodes are first-class** alongside entity nodes.
- **NeoTrix Defect #1**: **No dynamic schema induction.** NeoTrix's KB uses static node/edge type definitions. AutoSchemaKG proves schemas should be induced from data via conceptualization (generalizing instances into semantic categories). NeoTrix lacks the ϕ: V → P(C) mapping that assigns conceptual categories to nodes.
- **NeoTrix Defect #2**: **No event modeling.** AutoSchemaKG models events as first-class semantic units (VE ∩ VN = ∅), capturing temporal relationships, causality, and procedural knowledge. NeoTrix only extracts entity-entity triples. 90% content preservation with events vs 70% with entities alone.

### 1.2 SocraticKG (ACL Findings 2026)
- **Source**: https://aclanthology.org/2026.findings-acl.1951.pdf
- **Key**: QA-driven fact extraction. Uses 5W1H-guided QA pairs as intermediate representation before triple extraction. Addresses coverage-connectivity trade-off. Canonicalization via embedding clustering + LLM refinement.
- **NeoTrix Defect #3**: **No QA-mediated semantic decomposition.** NeoTrix extracts triples directly from raw text, losing implicit relational dependencies. SocraticKG proves QA expansion as intermediate step captures contextual dependencies and implicit links lost in direct extraction.
- **NeoTrix Defect #4**: **No graph canonicalization pipeline.** SocraticKG uses K-means clustering + BM25 sparse overlap + LLM refinement to unify synonymous entities/relations. NeoTrix has no deduplication or canonicalization for extracted triples.

### 1.3 Wikontic (EACL 2026)
- **Source**: https://aclanthology.org/2026.eacl-long.388/
- **Key**: Wikidata-aligned ontology-aware KG. Extracts candidate triplets WITH qualifiers. Enforces type/relation constraints. Normalizes entities. 96% correct answer entity in triplets on MuSiQue. 3× fewer output tokens than AriGraph, <1/20 of GraphRAG.
- **NeoTrix Defect #5**: **No qualifier extraction.** NeoTrix extracts flat (s, r, o) triples. Wikontic shows qualifiers (time, location, manner) dramatically improve downstream QA. Missing qualifiers = missing context for disambiguation.

### 1.4 GPTKB 2.0 (2026)
- **Source**: https://arxiv.org/html/2608.03729
- **Key**: 1M+ disambiguated entities, 38.4M triples. On-the-fly disambiguation during construction. Guarded parallelization strategy to prevent duplication. 36.8% novel entities not in Wikidata. $6,992 cost for full build.
- **NeoTrix Defect #6**: **No guarded parallelization for entity disambiguation.** When processing entities concurrently, duplicates arise if candidates don't yet exist. GPTKB 2.0 defers entities whose duplicates may not be established, parallelizing only safe batches. NeoTrix has no such concurrency control.

### 1.5 TFreeKGGen (2026)
- **Source**: https://www.sciopen.com/local/article_pdf/10.26599/TST.2026.9010071.pdf
- **Key**: Training-free KG generation. Hierarchical prompting. Global semantic entity alignment. **Two-phase evidence-grounded validation-correction** — each triple verified against verbatim textual evidence.
- **NeoTrix Defect #7**: **No evidence-grounded validation.** TFreeKGGen requires verbatim textual evidence for every extracted triple, with no rewriting or inference allowed. NeoTrix has no mechanism to verify extracted triples against source text, enabling hallucinated facts in KB.

---

## 2. Entity Linking

### 2.1 Select, Don't Train (ISWC 2026)
- **Source**: https://arxiv.org/abs/2608.27470
- **Key**: Decouples retrieval from selection. BM25 (training-free) + LLM selector reaches 86.3 F1 on ZELDA. Trained dense retriever + LLM reaches 88.5. **Abstention when retrieval failure detected** → 90.7 F1 when rewarded.
- **NeoTrix Defect #8**: **No retrieval-selection decoupling.** NeoTrix conflates candidate retrieval and disambiguation into a single step. Decoupling allows: (a) training-free retrieval, (b) LLM-based selection, (c) abstention on retrieval failure. NeoTrix forces a link even when uncertain.

### 2.2 LELA (IJCAI 2026)
- **Source**: https://arxiv.org/html/2601.05192v1
- **Key**: True zero-shot entity linking — no fine-tuning, no training data. Coarse-to-fine: candidate generation → pointwise reranking → LLM reasoning with self-consistency voting. Outperforms fine-tuned methods on multiple benchmarks.
- **NeoTrix Defect #9**: **No zero-shot entity linking capability.** LELA proves LLMs can do entity linking without any training data or domain adaptation. NeoTrix's KB ingestion pipeline has no entity linking step — mentions are stored as-is without disambiguation against existing KB entities.

### 2.3 ThinkLinker (ACL Findings 2026)
- **Source**: https://aclanthology.org/2026.findings-acl.1248.pdf
- **Key**: Multimodal entity linking. Low-rank fusion for multi-granular + multimodal features. Bidirectional retrieval-verification with LLM dialogue-style verification. +5.64 H@1 on WikiMEL.
- **NeoTrix Defect #10**: **No multimodal entity disambiguation.** When NeoTrix ingests documents with images/diagrams, visual context is discarded. ThinkLinker shows visual information significantly improves disambiguation, especially when textual context is weak.

### 2.4 Sci-ZSEL (EMNLP Findings 2026)
- **Source**: https://arxiv.org/abs/2609.00228
- **Key**: Selective LLM alias generation + ontology-aware filtering. Addresses low lexical overlap in scientific domains. Controls computational cost by selective generation.
- **NeoTrix Defect #11**: **No alias generation for KB entities.** Sci-ZSEL shows LLMs can generate aliases that bridge lexical divergence between mentions and canonical entity names. NeoTrix KB stores entities by single label with no alias set, causing missed links when surface forms vary.

---

## 3. Relation Extraction

### 3.1 ReaORE (2026)
- **Source**: https://arxiv.org/html/2606.26986
- **Key**: Reasoning-guided open RE. Coarse-to-fine relation reasoning. Relation filtering (multi-aspect reasoning) → relation prediction (fine-grained comparative reasoning with pairwise judgment evidence). Handles unseen relation types.
- **NeoTrix Defect #12**: **No open-world relation extraction.** NeoTrix only extracts relations from a fixed predefined set. ReaORE shows LRMs can discover and reason about novel relation types through explicit comparative reasoning. NeoTrix misses relations not in its schema.

### 3.2 DiffIE (2026)
- **Source**: https://arxiv.org/abs/2609.02315
- **Key**: Diffusion-based OpenIE. Treats multi-output extraction as independent reverse-diffusion trajectories. Pool size and return count are inference-time tunable. New SOTA on CaRB F1 and AUC.
- **NeoTrix Defect #13**: **No multi-hypothesis extraction.** A single sentence often expresses multiple valid relational triplets. DiffIE generates a pool of candidates and ranks them. NeoTrix extracts one interpretation per sentence, discarding valid alternative relations.

### 3.3 DSE-RE (Springer 2026)
- **Source**: https://link.springer.com/article/10.1007/s44230-026-00158-1
- **Key**: Dynamic Schema Evolution. Closed-loop feedback: dynamically constructs/optimizes entity type tables and relation type tables. Graph-based semantic clustering merges synonymous redundancies. +3% F1 on FOBIE, +6.8% on HSA.
- **NeoTrix Defect #14**: **No schema evolution feedback loop.** DSE-RE dynamically adapts extraction schema based on observed data. NeoTrix's schema is static — new entity/relation types discovered during extraction are either forced into existing categories or discarded.

### 3.4 D2G (ACL 2026)
- **Source**: https://aclanthology.org/2026.acl-long.2093.pdf
- **Key**: Discriminative-to-Generative framework. Discriminative model produces top-k candidate relations, injected into generative model via ICL or prompt learning. Significant gains on long-tailed relations.
- **NeoTrix Defect #15**: **No discriminative-guided generation.** NeoTrix uses purely generative extraction. D2G shows discriminative models provide structured prior that helps generative models, especially for rare/long-tailed relation types. NeoTrix has no such hybrid approach.

### 3.5 ProUIE (ACL Findings 2026)
- **Source**: https://aclanthology.org/2026.findings-acl.1093.pdf
- **Key**: Macro-to-micro progressive learning for UIE. Three stages: Complete Modeling → Streamlined Alignment → Deep Exploration (GRPO with stepwise fine-grained rewards). Qwen3-4B backbone. Outperforms larger models.
- **NeoTrix Defect #16**: **No progressive extraction refinement.** ProUIE shows extracting NER → RE → EE in difficulty order with progressive reward shaping improves all tasks. NeoTrix treats extraction as single-pass, missing iterative refinement opportunities.

---

## Summary: 16 NEW Defects Found

| # | Category | Defect | Source |
|---|----------|--------|--------|
| 1 | KG Construction | No dynamic schema induction | AutoSchemaKG |
| 2 | KG Construction | No event modeling as first-class units | AutoSchemaKG |
| 3 | KG Construction | No QA-mediated semantic decomposition | SocraticKG |
| 4 | KG Construction | No graph canonicalization pipeline | SocraticKG |
| 5 | KG Construction | No qualifier extraction | Wikontic |
| 6 | KG Construction | No guarded parallelization for disambiguation | GPTKB 2.0 |
| 7 | KG Construction | No evidence-grounded triple validation | TFreeKGGen |
| 8 | Entity Linking | No retrieval-selection decoupling | Select Don't Train |
| 9 | Entity Linking | No zero-shot entity linking | LELA |
| 10 | Entity Linking | No multimodal disambiguation | ThinkLinker |
| 11 | Entity Linking | No alias generation for KB entities | Sci-ZSEL |
| 12 | Relation Extraction | No open-world relation extraction | ReaORE |
| 13 | Relation Extraction | No multi-hypothesis extraction | DiffIE |
| 14 | Relation Extraction | No schema evolution feedback loop | DSE-RE |
| 15 | Relation Extraction | No discriminative-guided generation | D2G |
| 16 | Relation Extraction | No progressive extraction refinement | ProUIE |

---

## Cross-Batch Evolution (629 → 630)

Batch 629 found **input-layer defects** (tokenization, normalization, code parsing).
Batch 630 finds **extraction-layer defects** (schema, disambiguation, validation, open-world).

**Pattern**: NeoTrix's KB pipeline is a "closed-world static extraction" system in an era of "open-world dynamic extraction." Every 2026 SOTA system features:
- Dynamic/induced schemas (not predefined)
- Open-world extraction (novel types discovered at runtime)
- Evidence-grounded validation (triples verified against source)
- Disambiguation-first architecture (link before store)
- Multi-output/hypothesis extraction (not single-pass)

**Priority Fix**: Defects #7 (evidence validation), #9 (zero-shot EL), and #14 (schema evolution) are highest impact — they gate all downstream quality.

---

## Sources Cited

1. AutoSchemaKG — ACL 2026, https://aclanthology.org/2026.acl-long.942.pdf
2. SocraticKG — ACL Findings 2026, https://aclanthology.org/2026.findings-acl.1951.pdf
3. Wikontic — EACL 2026, https://aclanthology.org/2026.eacl-long.388/
4. GPTKB 2.0 — 2026, https://arxiv.org/html/2608.03729
5. TFreeKGGen — 2026, https://www.sciopen.com/local/article_pdf/10.26599/TST.2026.9010071.pdf
6. Select Don't Train — ISWC 2026, https://arxiv.org/abs/2608.27470
7. LELA — IJCAI 2026, https://arxiv.org/html/2601.05192v1
8. ThinkLinker — ACL Findings 2026, https://aclanthology.org/2026.findings-acl.1248.pdf
9. Sci-ZSEL — EMNLP Findings 2026, https://arxiv.org/abs/2609.00228
10. ReaORE — 2026, https://arxiv.org/html/2606.26986
11. DiffIE — 2026, https://arxiv.org/abs/2609.02315
12. DSE-RE — Springer 2026, https://link.springer.com/article/10.1007/s44230-026-00158-1
13. D2G — ACL 2026, https://aclanthology.org/2026.acl-long.2093.pdf
14. ProUIE — ACL Findings 2026, https://aclanthology.org/2026.findings-acl.1093.pdf
15. HYDRE — ACL 2026, https://aclanthology.org/2026.acl-long.2109.pdf
16. From RE to TE Survey — Computer Science Review 2026, https://www.sciencedirect.com/science/article/pii/S1574013726000626
17. Nature Specialized KG — Scientific Reports 2026, https://www.nature.com/articles/s41598-026-38066-w
18. SPHERE/HGNet — 2026, https://arxiv.org/pdf/2603.23136
19. DocZSRE-SI — EACL 2026, https://aclanthology.org/2026.eacl-long.216/
20. Reap AKBC — 2026, https://arxiv.org/html/2608.10963
