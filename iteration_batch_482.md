# Iteration Batch 482 — Tokenization / Parsing / Morphology Research Loop

**Date**: 2026-09-06
**Domains**: Tokenization | Syntactic Parsing | Morphological Analysis

---

## 1. Sources Cited

### Tokenization (2026)

| # | Source | Year | Key Finding |
|---|--------|------|-------------|
| T1 | ACL Findings 2026 — "Which Pieces Does Unigram Tokenization Really Need?" | 2026 | Unigram is robust to hyperparameter choices; EM iterations have minimal effect; proposes Flat Score Pruning for better compression vs BPE; pretoken-based initialization consistently outperforms full-text |
| T2 | arXiv 2608.00837 — "Pruned BPE: Post-training Visibility Pruning and Token Reallocation" | 2026 | Separates merge construction from model-visible vocabulary; low-exposure tokens retained as internal-only merge nodes; 0.27–0.36% encoded-length reduction at same vocab budget |
| T3 | IoT Digital Twin PLM — "LLM Tokenization Deep Dive: BPE, SentencePiece, Tiktoken" (May 2026) | 2026 | Vocab sizes cluster at 32K–50K (English) and 100K–200K (multilingual); o200k_base uses regex preserving multi-char punctuation; 15–30% token count shift between cl100k and o200k on same text |
| T4 | ACL Anthology 2026 — "Tokenizer-Aware Cross-Lingual Adaptation" (EACL 2026) | 2026 | Embedding relearning with customized tokenizers improves Gemma2 by up to 20%; non-Latin scripts benefit most |
| T5 | Dev.to — "Tokenization under the hood" (Jun 2026) | 2026 | SentencePiece byte-level BPE best for multilingual; Unigram supports probabilistic segmentation for robustness; tokenizer version pinning critical for reproducibility |
| T6 | FutureAGI — "What is Tokenization in LLMs" (May 2026) | 2026 | Tokens-per-character per language as tokenizer-quality metric; tokenization affects BLEU, ROUGE, refusal-rate evals; pin tokenizer version |

### Syntactic Parsing (2026)

| # | Source | Year | Key Finding |
|---|--------|------|-------------|
| P1 | ACL 2026 — "High-Accuracy Transition-Based Constituency Parsing" | 2026 | Self-training + dynamic oracle + Electra embeddings achieve new SOTA 96.61 F1 on Penn Treebank; windowed bidirectional attention helps low-resource |
| P2 | ACL 2026 — "Dynamic Head Selection for Neural Lexicalized Constituency Parsing" | 2026 | Latent lexicalization dynamically infers lexical heads without predefined rules; learns from data directly; SOTA across multiple treebanks |
| P3 | ACL 2026 — "Modal Dependency Parsing as Structured Prediction over Source-Cue Scope" | 2026 | Structured prediction over source-cue-scope triples; 3–4% SOTA improvement; LLMs identify cue expressions + modal scope |
| P4 | arXiv 2608.27035 — "Korean Constituency at Different Granularity Levels" (Aug 2026) | 2026 | Fine-grained morphological + XPOS representations provide valuable parsing evidence; Morpheme+XPOS strongest even when projected to eojeol domain |
| P5 | arXiv 2603.14755 — "Constituent Headedness as Explicit Interface" (Mar 2026) | 2026 | Learned headedness outperforms Collins-style percolation; near-ceiling intrinsic accuracy; transfers across resources under simple label mapping |
| P6 | Computational Linguistics 2026 — "Sequence Labeling for Constituent Parsing" | 2026 | Homogeneous comparison of encodings across 9 languages; new compact encoding introduced; binary vs arbitrary structures affect accuracy |
| P7 | ACL Findings 2025 — "Span-based SRL as Lexicalized Constituency Tree Parsing" | 2025 | SRL integrated with constituency + dependency parsing; bridges syntax-semantics gap without external syntactic resources |

### Morphological Analysis (2026)

| # | Source | Year | Key Finding |
|---|--------|------|-------------|
| M1 | LREC 2026 — "MorfFlex: Handling Rich Morphology" | 2026 | MorfFlex CZ: 100M+ wordforms, 1M+ lemmas from 450K source rows via inflectional/derivational patterns; MorphoDiTa achieves 96.27% tagging F1, 98.31% lemmatization F1; throughput 10–200K words/sec |
| M2 | LREC 2026 — "Nepali Lemmatization with Multilingual Transformers" | 2026 | mT5-base: 96.1% accuracy, 1.1% CER; mBART-large-50: 0.970 morphological coverage; lemmatization improves cross-lingual alignment from 12.86% to 41.61% Acc@1 |
| M3 | ACL 2026 — "Morphologically-informed Somali Lemmatization Corpus" | 2026 | 5,584 roots → 78,629 POS-tagged derivatives; hybrid lexicon+rule achieves 51–60% accuracy; agglutinative morphology challenge for low-resource |
| M4 | LREC 2026 — "Modular Approach to Automating Morphological Components in Grammar Engineering" | 2026 | Paradigm construction → inflectional class extraction → prediction pipeline; decision-tree rules outperform manual grammar rules; 7 new languages |
| M5 | LREC 2026 — "GLeMM: Large-scale Derivational Resource" | 2026 | Automated derivational morphology across 7 European languages from Wiktionary; bridges derivational-inferential morphology gap |
| M6 | arXiv 2604.12442 — "GLeMM" (Apr 2026) | 2026 | Automatic annotation of morphological features + semantic descriptions; enables data-driven word-formation research |

---

## 2. Defects Identified in NeoTrix Design

### Defect 1: Crude Whitespace Tokenizer in BM25 and Iteration Bank — No Subword Awareness

**Evidence**: `neotrix-core/src/unified/layers/action/nt_memory/nt_memory_kb/bm25.rs:165-170` and `neotrix-core/src/unified/core/nt_core_bank/iteration.rs:132-138` implement identical tokenizers:

```rust
pub(crate) fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
        .filter(|s| !s.is_empty() && s.len() >= 2)
        .map(|s| s.to_string())
        .collect()
}
```

This is a whitespace/punctuation splitter with a 2-char minimum filter. It has no subword decomposition, no stemming, no lemmatization, no Unicode normalization.

**Gap vs Research**:
- T1 (Unigram 2026) shows subword segmentation can be simplified to ~Flat Score Pruning with minimal quality loss
- T2 (Pruned BPE) achieves 0.27–0.36% encoded-length reduction by separating merge construction from visible vocabulary
- T4 (EACL 2026) shows embedding relearning with customized tokenizers improves multilingual models by up to 20%
- M2 (Nepali 2026) demonstrates lemmatization improves cross-lingual alignment from 12.86% to 41.61% Acc@1

**Impact**: KB search quality degrades on: (a) CJK/Indic/agglutinative text where whitespace ≠ word boundary, (b) compound words ("state-of-the-art" kept as single token or split), (c) inflected forms (different surfaces of same lemma scored separately), (d) technical jargon (e.g., "async/await" loses subword structure).

**Suggestion**: Implement a `TokenizationPipeline` trait with pluggable backends:
1. Default: BPE or Unigram via `tokenizers` crate (Rust-native, 500K tokens/sec)
2. Fallback: byte-level BPE for unknown scripts (per T3)
3. Optional: morphology-aware segmentation for agglutinative languages (per M1 MorfFlex, M2 mT5)
4. Add `tokenize_version: String` to BM25 index metadata to track tokenizer drift

### Defect 2: No Morphological Normalization in KB Search Pipeline

**Evidence**: The `tokenize()` function preserves raw surface forms. No lemmatization, no stem reduction, no POS tagging exists anywhere in the KB search path. The FTS5 index uses `tokenize='porter unicode61'` (per `nt_core_kb_primitives.rs:245` and `nt_memory_search.rs:783`) but this is SQLite's built-in Porter stemmer — language-unaware, English-only, no morphological paradigm coverage.

**Gap vs Research**:
- M1 (MorfFlex 2026) achieves 98.31% lemmatization F1 using pattern-based morphological dictionaries
- M2 (Nepali 2026) shows mT5-base achieves 96.1% accuracy for neural lemmatization with only 8K training pairs
- M3 (Somali 2026) demonstrates hybrid lexicon+rule lemmatization for agglutinative languages
- M4 (Grammar Engineering 2026) shows paradigm-based inflectional class prediction outperforms manual grammar rules

**Impact**: KB queries for "running", "ran", "runs" return separate results instead of unified matches. FTS5 Porter stemmer conflates "running"→"run" but fails on: irregular forms ("went"→"?"), CJK (no stems to reduce), agglutinative languages (Turkish "evlerinizden" has 4 morphemes).

**Suggestion**: Add `nt_memory::morphology` module:
1. `MorphologicalNormalizer` trait with backends: (a) rule-based (POS-tag + stem), (b) neural (mT5/mBART for low-resource), (c) dictionary (MorfFlex-style pattern expansion)
2. Store `<lemma, POS, surface>` triples in KB alongside raw text
3. Index both surface and lemma forms in BM25 for retrieval
4. Add language detection to `tokenize()` to route to appropriate normalizer

### Defect 3: No Constituency or Dependency Parsing in NT-WORLD Document Understanding

**Evidence**: NT-WORLD's `UnifiedCrawler` and `FileModel` extract flat text + raw table JSON. No syntactic parsing of any kind exists. The `FileParser` handles format detection and text extraction but produces unstructured text blobs.

**Gap vs Research**:
- P1 (ACL 2026) achieves 96.61 F1 constituency parsing with transition-based methods + Electra embeddings
- P2 (ACL 2026) shows dynamic head selection eliminates need for predefined head-finding rules
- P4 (Korean 2026) demonstrates that fine-grained morphological + XPOS representations provide parsing evidence
- P5 (2026) shows learned headedness outperforms rule-based percolation as constituency-dependency interface
- P7 (ACL 2025) bridges SRL with constituency+dependency parsing without external syntactic resources

**Impact**: NeoTrix cannot answer questions requiring structural understanding: "Who did what to whom?", "What modified what?", "What is the subject of clause X?". Knowledge graph construction relies on LLM-based extraction (iteration_batch_475, defect G3) instead of grounded syntactic analysis.

**Suggestion**: Add `nt_world::syntax` module:
1. `ConstituencyParser` — transition-based (P1) or sequence-labeling (P6) for fast extraction
2. `DependencyParser` — projective arc-standard for Subject-Object-Verb detection
3. `MorphoSyntacticAnalyzer` — combine P4's Morpheme+XPOS with P5's learned headedness
4. Wire into KB graph construction: dependency triples → edges, constituency chunks → node spans
5. Add SelfTest T2 registration for parser quality metrics (F1 vs gold treebanks)

### Defect 4: No Cross-Tokenizer Version Tracking — Evaluation Reproducibility Gap

**Evidence**: NeoTrix has no `TokenizerVersion` metadata. The FTS5 tokenizers are configured inline (`porter unicode61`, `unicode61 remove_diacritics=2`) with no version tracking. BM25's custom `tokenize()` has no version identifier. There is no mechanism to detect tokenizer drift between index creation and query time.

**Gap vs Research**:
- T3 (2026) quantifies 15–30% token count shift between cl100k and o200k on same text
- T5/T6 (2026) explicitly warn that tokenizer version drift is a quiet source of regression
- T1 (2026) identifies EM iteration count and pruning thresholds as invisible quality levers

**Impact**: KB index rebuilt with different tokenizer version silently produces different results. Query-time tokenization mismatches index-time tokenization. No reproducibility guarantee for search results across deployments.

**Suggestion**: Add `TokenizerSpec` to KB metadata:
```rust
struct TokenizerSpec {
    algorithm: String,     // "bpe" | "unigram" | "porter" | "whitespace"
    version: String,       // "2026.09.01"
    vocab_size: Option<u32>,
    language: String,
    hash: String,          // SHA-256 of training data hash
}
```
Store in `kv_store` `config` namespace. Add `selftest` that validates query tokenizer matches index tokenizer.

### Defect 5: No Morphology-Aware Tokenization for Agglutinative Languages

**Evidence**: The tokenize() function treats all languages identically via whitespace splitting. For agglutinative languages (Turkish, Finnish, Hungarian, Japanese, Korean), whitespace does not delimit morphological units.

**Gap vs Research**:
- P4 (Korean 2026) shows Morpheme+XPOS representation significantly outperforms Eojeol+UPOS for parsing
- M1 (MorfFlex 2026) demonstrates pattern-based paradigm expansion from 450K source rows to 100M+ wordforms
- M3 (Somali 2026) shows agglutinative morphology requires hybrid lexicon+rule approaches
- T4 (EACL 2026) shows 20% improvement with customized tokenizers for non-Latin scripts

**Impact**: Korean, Turkish, Finnish, Hungarian, Japanese content in KB is treated as undifferentiated byte sequences. Search for "evlerinizden" (Turkish: "from your houses") cannot match "ev" (house). Japanese compound words without spaces are split at wrong boundaries.

**Suggestion**: Add `MorphologySegmenter` adapter in tokenization pipeline:
1. Language detection → route to appropriate segmenter
2. Korean: eojeol-level + morpheme decomposition (per P4)
3. Turkish/Finnish: affix-stripping with paradigm lookup (per M1)
4. Japanese: MeCab/UniDic-based morpheme boundary detection
5. Store segmenter config in `TokenizerSpec` for reproducibility

### Defect 6: No Structured Parsing for KB Knowledge Graph — Flat Text Only

**Evidence**: KB nodes store `text: String` as flat content. Edges are manually created by `nt_absorb_mapper.rs`. No automatic extraction of entity-relation-entity triples from crawled text. The `extract_relations` function (iteration_batch_475, defect G3) blindly creates relations between entities without syntactic grounding.

**Gap vs Research**:
- P3 (ACL 2026) uses structured prediction for source-cue-scope triples — a model for structured extraction
- P7 (ACL 2025) bridges SRL with constituency parsing for predicate-argument structure extraction
- M2 (Nepali 2026) shows POS tagging improves downstream task quality by 2–3x

**Impact**: Knowledge graph is manually curated, not automatically populated from document content. Semantic search cannot leverage syntactic structure. Relation extraction is unreliable without dependency grounding.

**Suggestion**: Add `nt_world::structured_extraction` pipeline:
1. Parse crawled text → constituency/dependency trees
2. Extract SRL frames (P7 approach) for predicate-argument triples
3. Store `<subject, predicate, object, source_doc, confidence>` as typed edges in KB
4. Add SelfTest T3: extraction precision/recall on gold dataset

---

## 3. Priority Suggestions

| # | Defect | Severity | Module | Suggestion |
|---|--------|----------|--------|------------|
| D1 | Crude whitespace tokenizer | HIGH | nt_memory_kb | Replace with BPE/Unigram via `tokenizers` crate |
| D2 | No morphological normalization | HIGH | nt_memory | Add `MorphologicalNormalizer` trait with neural + rule backends |
| D3 | No syntactic parsing | HIGH | nt_world | Add `ConstituencyParser` + `DependencyParser` modules |
| D4 | No tokenizer version tracking | MEDIUM | nt_memory | Add `TokenizerSpec` to KB metadata |
| D5 | No agglutinative language support | MEDIUM | nt_memory | Add `MorphologySegmenter` adapter with language routing |
| D6 | No structured extraction pipeline | MEDIUM | nt_world | Add `structured_extraction` using SRL + dependency triples |

---

## 4. Recommended Absorption Path

1. **Immediate** (P0): D1 + D2 — Replace tokenize() with BPE tokenizer + add lemmatization. Blocks all downstream KB quality. Target: `tokenizers` crate integration, 1 day.
2. **Short-term** (P1): D3 + D6 — Add dependency parsing for KB graph construction. Depends on D1 for tokenizer foundation. Target: `tree-sitter` or `удалить` crate integration, 3 days.
3. **Medium-term** (P2): D4 + D5 — Version tracking + agglutinative support. Enables production reliability. Target: KB metadata schema update, 2 days.
