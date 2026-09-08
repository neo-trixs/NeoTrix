# Iteration Batch #475 — NLP Pipeline Research & Defect Analysis

**Date:** 2026-09-06
**Research Areas:** Coreference Resolution, Named Entity Recognition, Relation Extraction
**Iteration:** 475 / 10000+

---

## 1. Sources Cited

| # | Source | Year | Topic | Key Finding |
|---|--------|------|-------|-------------|
| S1 | Plum et al. "Do LLMs Judge Distantly Supervised Named Entity Labels Well?" (arXiv:2601.00411, LREC 2026) | 2026 | NER | LLM-as-judge pipeline for weakly-supervised NER; Wikipedia+Wikidata weak supervision yields 5× larger corpora with balanced entity coverage for low-resource languages |
| S2 | Esser & Dörpinghaus. "Noise-Aware NER for Historical VET Documents" (arXiv:2601.00488, VISAPP 2026) | 2026 | NER | Noise-Aware Training (NAT) with synthetic OCR error injection + multi-stage fine-tuning; domain-specific noise robustness transfers across languages |
| S3 | Dmonte et al. "Exploring LLMs on Subjective Span Identification" (arXiv:2601.00736) | 2026 | NER/Spans | LLMs (instruction-tuned, ICL, CoT) outperform BERT on subjective span tasks (ABSA, offensive language, claim verification); underlying relationships aid span precision |
| S4 | arXiv 2026 cs.CL listings (19,103 papers YTD 2026) | 2026 | All | Trend: LLM-based NER/parsing replacing pipeline systems; tool-augmented agents performing structured extraction; knowledge graph construction via LLM chains |
| S5 | HuggingFace Daily Papers (Sep 2026) | 2026 | Agents/Memory | VoiceMem dual-brain streaming memory; StateM durable agent states; Apodex 1.1 long-horizon agent coordination; COLLEAGUE.SKILL skill distillation; Mem0 graph-based memory consolidation |
| S6 | Zep temporal knowledge graph (arXiv, 2025, trending) | 2025-26 | KG/Memory | Temporal KG with persistent entity extraction and relationship tracking across sessions; outperforms MemGPT on DMR/LongMemEval |
| S7 | Mem0 production-ready agent memory (arXiv, 2025, trending) | 2025-26 | Memory | Graph-based memory with extraction → consolidation → retrieval pipeline; production-grade entity/relation extraction for agent coherence |

---

## 2. Codebase Audit — Current NLP Pipeline State

**Primary file:** `neotrix-core/src/unified/layers/perception/nt_world/nt_world_semantic_extract.rs`

### Current Capabilities
- `SemanticExtractionPipeline` with `EntityExtractor` + `RelationExtractor` + `EmbeddingGenerator` + `KnowledgeGraph`
- `EntityPattern` with 4 types: Regex, Keyword, NER, Dependency
- 4 entity types: Person, Organization, Location, Event, Concept, Technology, Code, File, Module, Function
- 2 relation patterns: `depends_on`, `implements`
- Basic cosine similarity search
- Graph construction from extraction results

### Critical Gaps Identified

| ID | Gap | Severity | Evidence |
|----|-----|----------|----------|
| G1 | **Zero coreference resolution** | CRITICAL | No `coref`, `pronoun`, `anaphora`, `cataphora`, or `antecedent` logic in the NLP pipeline. The only `antecedent` field exists in `nt_core_knowledge_mgmt.rs:105` for reasoning rules (IF-THEN logic), not linguistic coreference. |
| G2 | **NER is keyword/regex only** | HIGH | `EntityExtractor` only handles `PatternType::Keyword` and `PatternType::Regex`. The `PatternType::NER` variant exists but has zero implementation — the `extract_entities` match arm falls through with `_ => {}` (line 263). |
| G3 | **Relation extraction is positional, not semantic** | HIGH | `extract_relations` blindly creates a relation between `entities[0]` and `entities[1]` when any keyword is found (lines 278-287). No dependency parsing, no subject-object identification, no span alignment. |
| G4 | **No document-level context** | MEDIUM | Each `extract()` call processes a single string independently. No cross-sentence entity chaining, no discourse structure, no session-level entity tracking. |
| G5 | **Embedding is pseudo-hash, not semantic** | MEDIUM | `generate_embedding` uses character-index hashing (`c as u32 / 1000.0`), not a real embedding model. Breaks all downstream semantic similarity. |
| G6 | **No nested entity support** | MEDIUM | `EntityType` is flat; no hierarchy. "Google Research Lab" would be tagged as Organization but not Organization→Org:Division. |
| G7 | **No confidence calibration** | LOW | All entities assigned fixed confidence (0.8 or 0.9). No model-based uncertainty estimation. |

---

## 3. Defect Analysis — Mapping Research to NeoTrix Gaps

### Defect D1: Zero Coreference Resolution (from S5, S6, S7)

**Research signal:** VoiceMem (S5) uses dual-brain architecture with streaming entity extraction + consolidation. Mem0 (S7) explicitly extracts entities and consolidates duplicates across sessions. Zep (S6) maintains persistent entity extraction with relationship tracking across sessions. All three require coreference resolution as a foundational capability.

**NeoTrix defect:** The `SemanticExtractionPipeline` processes text as isolated strings. When NeoTrix reads "NeoTrix is an AI toolkit. It uses E8..." — "It" is never resolved to "NeoTrix". The `KnowledgeGraph` accumulates duplicate entity nodes without deduplication.

**Impact:** Knowledge graph becomes a garbage heap of duplicate entities. GWT attention routing cannot track salient entities across discourse. ConsciousnessTree's cross-session entity tracking is blind to pronoun references.

### Defect D2: NER PatternType::NER Is Unimplemented (from S1, S2, S3)

**Research signal:** JudgeWEL (S1) shows LLM-as-judge NER achieving 5× data yield for low-resource languages. Noise-Aware NER (S2) demonstrates that robust NER requires domain-specific fine-tuning + synthetic noise injection. LLM span identification (S3) shows instruction-tuned models outperform BERT on subjective spans.

**NeoTrix defect:** `PatternType::NER` enum variant exists but the match arm in `extract_entities` is `_ => {}` — the implementation is literally empty. The "NER" keyword in `EntityPattern` is decorative.

**Impact:** NeoTrix cannot recognize entities beyond keyword/regex patterns. Cannot handle "Dr. Sarah Chen founded Acme Corp in Berlin" — no real NER exists.

### Defect D3: Relation Extraction Is Semantically Wrong (from S4, S6, S7)

**Research signal:** Modern KG construction (S4, S6, S7) uses dependency parsing + subject-object extraction + LLM-based relation classification. The pattern is: entity extraction → dependency parse → candidate relation generation → relation classification.

**NeoTrix defect:** `extract_relations` creates a relation between `entities[0]` and `entities[1]` whenever ANY keyword matches anywhere in the text. If text contains 3 entities and the keyword "uses", it creates a relation between the FIRST two entities regardless of whether they actually have a "uses" relationship.

**Impact:** Generated relations are random, not truthful. The `KnowledgeGraph.edges` are unreliable. Downstream reasoning (`ReasoningEngine` in `nt_core_knowledge_mgmt.rs`) receives false premises.

### Defect D4: No Cross-Sentence / Document-Level Entity Resolution (from S5, S6)

**Research signal:** Apodex 1.1 (S5) handles "long-horizon work with state maintenance." Zep (S6) specifically tracks entities across sessions. COLLEAGUE.SKILL (S5) distills operational knowledge into inspectable packages.

**NeoTrix defect:** `extract()` takes a single `&str` and returns a flat `ExtractionResult`. No entity linking across calls. The `build_graph` method deduplicates by entity ID, but entities from different texts with the same name get different UUIDs.

**Impact:** "The README mentions Docker" and "Docker is used in the build" produce two different Docker entities with no cross-reference. KB node deduplication is UUID-based, not name/resolution-based.

### Defect D5: Pseudo-Embedding Breaks Semantic Search (from S4, S7)

**Research signal:** Mem0 (S7) and WeMM-Embedding (HuggingFace trending) use real neural embeddings for semantic retrieval. Cosine similarity over random vectors returns noise.

**NeoTrix defect:** `generate_embedding` at line 296-304 produces a vector where `embedding[i] = (text_char_at_i as u32) / 1000.0`. This is not a semantic embedding — it's a character-position histogram.

**Impact:** `SemanticExtractionPipeline::search()` returns results sorted by meaningless scores. The `KnowledgeGraph.embeddings` map stores garbage vectors. VSA HyperCube symbolic representation cannot ground to actual semantic space.

### Defect D6: No Noise-Aware / Robust Extraction (from S2)

**Research signal:** Esser & Dörpinghaus (S2) demonstrate that OCR-noisy text requires explicit noise-aware training + synthetic error injection for robust NER.

**NeoTrix defect:** NT-WORLD crawls web content and OCR'd documents. The `EntityExtractor` has zero noise tolerance — keyword matching is case-insensitive but has no fuzzy matching, no edit-distance tolerance, no character normalization.

**Impact:** Crawled documents with encoding artifacts, OCR noise, or formatting corruption produce zero entity matches.

### Defect D7: No LLM-Augmented Extraction (from S1, S3)

**Research signal:** JudgeWEL (S1) and subjective span identification (S3) both demonstrate that LLMs dramatically outperform pattern-based NER, especially for novel entities, nested entities, and context-dependent spans.

**NeoTrix defect:** `SemanticExtractionPipeline` is entirely rule-based. No LLM invocation path exists for entity/relation extraction. NT-IO has LLM provider infrastructure but it's not wired to the extraction pipeline.

**Impact:** NeoTrix cannot extract entities from code reviews, technical discussions, or any domain-specific text where entity boundaries are ambiguous.

---

## 4. Suggestions — Prioritized Fixes

### P0 — Critical (blocks KB correctness)

| # | Fix | Module | Est. Effort |
|---|-----|--------|-------------|
| F1 | **Wire LLM-as-NER**: Add `PatternType::LLM` variant. On extraction, call NT-IO LLM provider with entity extraction prompt. Parse JSON output into `Entity` structs. Cache results. | `nt_world_semantic_extract.rs` | 2-3 days |
| F2 | **Coreference resolution stub**: Add `CoreferenceResolver` struct. First pass: detect pronoun patterns (it/they/she/he/this/that). Second pass: resolve to most recent compatible entity in same document window. Store resolved entities in KB with `coref_chain_id`. | New file: `nt_world_coref.rs` | 3-5 days |

### P1 — High (blocks relation quality)

| # | Fix | Module | Est. Effort |
|---|-----|--------|-------------|
| F3 | **Subject-Object extraction**: Replace blind `entities[0]→entities[1]` with dependency-parse-based extraction. Use LLM to identify subject/object for each candidate relation. | `nt_world_semantic_extract.rs` | 2-3 days |
| F4 | **Entity deduplication**: Add entity linking step after extraction. Match new entities against existing KB nodes by name fuzzy matching + type overlap + embedding similarity. | `nt_world_semantic_extract.rs` + `nt_memory_graphrag` | 2-3 days |

### P2 — Medium (blocks robustness)

| # | Fix | Module | Est. Effort |
|---|-----|--------|-------------|
| F5 | **Real embeddings**: Replace pseudo-hash with a local embedding model (e.g., `fastembed` crate or ONNX MiniLM). 384-dim is already declared. | `nt_world_semantic_extract.rs` | 1 day |
| F6 | **Noise-aware normalization**: Add Unicode normalization (NFKC), OCR artifact cleanup, and fuzzy keyword matching (edit distance ≤ 1) to `EntityExtractor`. | `nt_world_semantic_extract.rs` | 1 day |

### P3 — Low (future hardening)

| # | Fix | Module | Est. Effort |
|---|-----|--------|-------------|
| F7 | **Nested entity types**: Extend `EntityType` to support hierarchy (e.g., `Organization { division: Option<String> }`). | `nt_world_semantic_extract.rs` | 1 day |
| F8 | **Document-level context**: Add `DocumentContext` struct that maintains entity state across multiple `extract()` calls within a session. | `nt_world_semantic_extract.rs` | 2 days |

---

## 5. Cross-Domain Impact Assessment

| Domain | Impact | Notes |
|--------|--------|-------|
| **NT-WORLD** | Direct owner | All F1-F8 changes land here |
| **NT-MEMORY** | Entity dedup affects KB graph quality | F4 must coordinate with `nt_memory_graphrag` entity merging |
| **NT-CORE** | GWT attention routing depends on entity salience | Coreference resolution (F2) enables proper salience tracking |
| **NT-MIND** | SEAL pipeline extracts knowledge from code/docs | LLM-augmented extraction (F1) unlocks better distillation |
| **NT-FEEL** | Emotion detection in text needs entity resolution | "They're upset" — who? Coreference needed for agent emotion modeling |
| **NT-ACT** | Agent actions reference entities | Ambiguous entity references lead to wrong tool invocations |

---

## 6. Design Doc Location

**Primary file under review:** `neotrix-core/src/unified/layers/perception/nt_world/nt_world_semantic_extract.rs:1-395`
**Secondary file:** `neotrix-core/src/unified/layers/cognition/nt_core/nt_core_knowledge_mgmt.rs:95-124`
**Layer contract:** `neotrix-core/src/unified/layers/perception/traits.rs:34-72`

---

*Iteration 475 complete. 7 defects identified, 8 fixes suggested. Priority: F1 (LLM-NER) and F2 (coref resolution) are foundational — all downstream quality depends on them.*
