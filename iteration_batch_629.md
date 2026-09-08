# Iteration Batch 629 — Tokenization / Parsing / Text Processing

**Date**: 2026-09-06
**Iteration**: 629 / 10000+
**Previous Batch**: 628 (AI tool orchestration, code debt tracker, cost optimization, local inference, MCP server mode, multi-agent parallel review)

---

## 1. Tokenization Findings

### D629-1: Incremental BPE Tokenization — O(n log²t) Streaming Algorithm
**Source**: arxiv.org/html/2605.30813v1 (2026-05-29)
**Finding**: A novel incremental BPE algorithm processes each byte in O(log²t) time, achieving ~3x speedup over HuggingFace tokenizers. Critically, tiktoken exhibits O(n²) behavior on pathological inputs — the incremental algorithm avoids this entirely. Enables streaming tokenization: tokens emitted as soon as boundaries are determined.
**NeoTrix Defect**: NT-IO token estimation for LLM gateway is currently batch-oriented (encode entire prompt, then send). No streaming tokenization path exists. For large prompts hitting context limits, the O(n²) pathological case in tiktoken can cause latency spikes. **Improvement**: Implement incremental BPE streaming in `nt_io` token estimation layer, using the Aho-Corasick + Centroid Decomposition approach from this paper.

### D629-2: Language Tax — 6x Cost Penalty for Non-English via Tokenizer Vocabulary
**Source**: iotdigitaltwinplm.com/llm-tokenization-bpe-sentencepiece-tiktoken-2026/ (2026-05-26), dreaming.press (2026-06-24)
**Finding**: Tamil pays 6x more tokens than English for identical meaning. Arabic 10-15x for under-resourced scripts. The penalty is structural — frozen into vocabulary at training time. `o200k_base` reduced Hindi from ~8x to ~4.6x but did not eliminate it. Gemma 3's 256k vocab targets <3x for Indic/African languages.
**NeoTrix Defect**: NT-IO cost estimation assumes uniform token-to-cost ratio across languages. No per-language token efficiency tracking exists. Multilingual users get silently overcharged. **Improvement**: Add `language_tax_multiplier` metric to NT-IO gateway cost dashboard. Track tokens-per-character per language as tokenizer-quality metric. Flag when >2x English baseline.

### D629-3: Tokenizer Incompatibility Across Model Families — Measurement Trap
**Source**: dreaming.press (2026-06-24), futureagi.com (2026-04-25)
**Finding**: GPT-4o uses `o200k_base` (~200k tokens), Llama 3 uses 128k BPE, Gemma uses 256k SentencePiece Unigram. Same text: 1200 tokens in `cl100k_base` = 800 in different vocab. Prompt portability between models is a myth. Counting with wrong tokenizer = confidently incorrect cost.
**NeoTrix Defect**: NT-IO token counting appears model-agnostic but must be model-specific. If gateway routes to different providers, token estimates can drift 30%+. **Improvement**: Gateway must use `model->tokenizer` mapping table (GPT-4o→`o200k_base`, Llama 3→128k BPE, Gemma→SentencePiece). Token count must be validated per-model before cost calculation.

### D629-4: SentencePiece ▁ Convention Breaks Naive String Matching
**Source**: iotdigitaltwinplm.com (2026-05-26)
**Finding**: SentencePiece escapes spaces as `▁` (U+2581). Searching for `"hello"` in token stream misses `▁hello`. Detokenization required before any string matching. This is a silent correctness bug in retrieval, span extraction, and RAG systems.
**NeoTrix Defect**: NT-MEMORY RAG retrieval may perform token-level matching. If any code path matches raw strings against token IDs without detokenizing first, results silently fail. **Improvement**: Audit NT-MEMORY retrieval paths for `▁` mismatch. Add `detokenize_before_match` invariant check.

---

## 2. Parsing Findings

### D629-5: Tree-sitter Pure Go Runtime — Zero-CGO Cross-Compilation
**Source**: dev.to/thegdsks (2026-04-10)
**Finding**: `gotreesitter` reimplements tree-sitter runtime in pure Go. Ships 205 embedded grammars as compressed blobs. Sub-millisecond parsing. Cross-compiles to 6+ platforms without C toolchains. Binary size: 32MB with all grammars (vs 12MB regex baseline). Performance: 83ms for 66 Go files, 400ms for 1131 Python files.
**NeoTrix Defect**: NT-ACT code analysis and NT-WORLD content extraction currently use regex-based parsing for code understanding. No AST-level parsing exists for multi-language code comprehension. Tree-sitter's incremental parsing (O(1) amortized updates) is critical for real-time code analysis during LLM code generation. **Improvement**: Integrate tree-sitter Rust bindings (`tree-sitter` crate) into NT-ACT for code structure analysis. Priority languages: Rust (self-hosting), Python, TypeScript, Go.

### D629-6: Tree-sitter 106MB Parser Blowup — Grammar Optimization Required
**Source**: sshadows.dk/blog/tree-sitter-al-v2-rewrite/ (2026-03-24)
**Finding**: Naive grammar design produced 106MB `parser.c` (exceeds GitHub's 100MB limit). Rewrite from 8,500 lines to 3,100 lines reduced it to 10.6MB. Key insight: 291 property rules replaced by 1 generic rule + external scanner. Keywords-as-named-nodes can trigger GLR backtracking failures in `#if`/`#endif` constructs. Some keywords (`begin`/`end`) fundamentally cannot be named nodes.
**NeoTrix Defect**: If NeoTrix develops custom grammars for domain-specific DSLs (e.g., SEAL pipeline config, HyperCube schema), grammar blowup risk is real. No grammar size budget exists. **Improvement**: Add grammar size validation to NT-WORLD grammar pipeline. Cap parser.c at 20MB. Flag grammars with >1000 symbols for external-scanner optimization.

### D629-7: Tree-sitter Language ABI Versioning — Runtime Mismatch Risk
**Source**: tree-sitter-tree-sitter.mintlify.app (2026)
**Finding**: Tree-sitter parsers declare ABI version. Runtime checks compatibility. Mismatch between parser.c compiled with old tree-sitter and new runtime = silent parsing failures. Must recompile parsers when updating tree-sitter.
**NeoTrix Defect**: NT-ACT code tooling may bundle pre-compiled tree-sitter parsers. If tree-sitter crate is upgraded, stale parsers silently break. No version pinning or recompilation trigger exists. **Improvement**: Add `tree_sitter_abi_version` to NT-ACT dependency manifest. Trigger recompilation on tree-sitter major version bump.

### D629-8: LSP + Tree-sitter Integration for DSL Code Intelligence
**Source**: ropensci.org/blog/2026/04/02/tree-sitter-overview/ (2026-04-02)
**Finding**: Ark (R kernel for Positron IDE) uses tree-sitter for autocompletion and hover help. GitHub uses tree-sitter for code search (function definition shows in search results). `ast-grep` built on tree-sitter for code rewriting without regex. `gander` package uses tree-sitter to improve LLM code writing experience.
**NeoTrix Defect**: NT-IO (LLM interface) has no code-aware context injection. When LLM generates code, it lacks AST-level understanding of the user's codebase. No function definition lookup, no import chain resolution, no scope analysis. **Improvement**: Build tree-sitter-backed code intelligence into NT-IO LLM context pipeline. Extract: function signatures, import chains, type definitions, scope boundaries. Inject as structured context into LLM prompts.

---

## 3. Text Processing Findings

### D629-9: TextPraline — Deterministic Post-Extraction Refinement
**Source**: github.com/BadTrL/text-praline (2026-02-10)
**Finding**: Deterministic text normalization engine for post-extraction refinement. Handles: Unicode NFKC normalization, control character removal, zero-width character stripping, BOM removal, PUA removal, typographic quote normalization, list marker stabilization. Three modes: `safe`, `markdown_safe`, `strict`. Debug mode returns `PralineReport` with metrics.
**NeoTrix Defect**: NT-WORLD content extraction feeds raw OCR/HTML text into KB embedding pipeline without intermediate normalization. Invisible Unicode artifacts (zero-width chars, PUA, BOM) corrupt embeddings and chunking. **Improvement**: Insert TextPraline-style normalization layer between NT-WORLD extraction and NT-MEMORY embedding. Mandatory before any text enters KB pipeline.

### D629-10: W-NUT 2026 Multi-Lexnorm — 17-Language Normalization Gap
**Source**: noisy-text.github.io/2026/multi-lexnorm.html (2026)
**Finding**: W-NUT 2026 shared task covers 17 languages for lexical normalization. Vietnamese MFR-ERR baseline: 75.77% (hardest). Japanese: 6.32% (easiest). Key finding: annotation guidelines differ across languages — 1-n/n-1 splits, caps handling, insertion annotation are inconsistent. These differences create silent cross-lingual normalization failures.
**NeoTrix Defect**: NT-WORLD multilingual content processing has no lexical normalization stage. Social media text (abbreviations, slang, misspellings) enters KB without normalization. Cross-lingual consistency is untested. **Improvement**: Add lexnorm module to NT-WORLD. Start with high-impact languages: Vietnamese (75% error rate), Turkish (36%), Dutch (29%). Use MFR baseline as minimum quality bar.

### D629-11: CeluneNorm — Lightweight TTS Text Normalization Model
**Source**: huggingface.co/lunahr/CeluneNorm-0.6B-v2.0-ctx1024 (2026)
**Finding**: 0.6B parameter model for text normalization. Deterministic (no sampling). Preserves domain tokens (URLs, commands). v2.0 supports 1024-token contexts. Trained on mixed dataset: formal text + conversational + synthetic edge cases. 97.53% mean token accuracy.
**NeoTrix Defect**: NT-IO voice/speech interface has no text normalization stage for TTS input. Raw LLM output may contain formatting that degrades TTS quality. **Improvement**: Integrate CeluneNorm or equivalent normalization before TTS pipeline in NT-IO. Normalize: contractions, abbreviations, numbers → words, symbols → words.

### D629-12: flexiPipe — Unicode Normalization Critical for African Languages
**Source**: aclanthology.org/2026.africanlp-main.13/ (2026)
**Finding**: Unicode NFC vs NFD normalization causes 30% accuracy drop in Yoruba NLP. Combined diacritics (NFC) vs decomposed (NFD) treated as different tokens by models. flexiPipe tracks which normalization each model expects and normalizes accordingly. This is "often overlooked" but has "significant impact."
**NeoTrix Defect**: NT-MEMORY KB embedding pipeline has no Unicode normalization standard. Same word in NFC and NFD produces different embeddings, fragmenting knowledge. No NFKC normalization before embedding. **Improvement**: Enforce NFKC normalization at NT-MEMORY write boundary. All text entering KB must be NFKC-normalized. Add validation: reject non-NFKC text at ingestion.

### D629-13: DAG-Driven NLP Pipelines — Parallel Execution Architecture
**Source**: arunbaby.com/ml-system-design/0058-advanced-nlp-pipeline/ (2026-04-18)
**Finding**: Production NLP uses DAG orchestrator for parallel execution of NER, sentiment, coreference. RegEx state machines kill spam before GPU layers. Dynamic batching with length-sorted sentences minimizes padding waste. Model distillation (DistilBERT) delivers 95% accuracy at 10% latency.
**NeoTrix Defect**: NT-WORLD content processing is sequential (extract → classify → embed → store). No DAG orchestration for parallel NLP tasks. Spam filtering happens after expensive LLM processing. **Improvement**: Restructure NT-WORLD pipeline as DAG: parallel branches for NER + sentiment + classification. Insert RegEx spam filter as pre-filter before LLM calls. Implement length-sorted dynamic batching for GPU inference.

### D629-14: gladia-normalization — YAML-Declarative Normalization Pipelines
**Source**: github.com/gladiaio/normalization (2026-03-04)
**Finding**: Deterministic, language-aware normalization via YAML presets. Three-stage pipeline: text_pre → word → text_post. Protect/restore pattern for safe symbol handling. YAML-defined, immutable presets ensure reproducibility. Built for ASR WER computation.
**NeoTrix Defect**: NT-WORLD text processing lacks declarative normalization configuration. Normalization steps are hardcoded, not composable. No protect/restore pattern for safe symbol handling during normalization. **Improvement**: Adopt YAML-preset normalization pipeline for NT-WORLD. Define normalization as data, not code. Enable per-language preset selection.

---

## Summary

| # | Defect ID | Domain | Finding | Severity |
|---|-----------|--------|---------|----------|
| 1 | D629-1 | NT-IO | No incremental/streaming tokenization; O(n²) pathological case | HIGH |
| 2 | D629-2 | NT-IO | No per-language token cost tracking; 6x language tax invisible | MEDIUM |
| 3 | D629-3 | NT-IO | Token counting not model-specific; 30% drift across providers | HIGH |
| 4 | D629-4 | NT-MEMORY | ▁ convention breaks naive token-level string matching | MEDIUM |
| 5 | D629-5 | NT-ACT | No AST-level code parsing; regex-only code comprehension | HIGH |
| 6 | D629-6 | NT-WORLD | Grammar blowup risk for custom DSLs; no size budget | LOW |
| 7 | D629-7 | NT-ACT | Tree-sitter ABI version mismatch risk; no recompilation trigger | MEDIUM |
| 8 | D629-8 | NT-IO | No code-aware context injection into LLM prompts | HIGH |
| 9 | D629-9 | NT-WORLD | No post-extraction text normalization; Unicode artifacts corrupt KB | HIGH |
| 10 | D629-10 | NT-WORLD | No lexical normalization for 17 languages; social media text noisy | MEDIUM |
| 11 | D629-11 | NT-IO | No TTS input normalization; raw LLM output degrades speech | LOW |
| 12 | D629-12 | NT-MEMORY | No Unicode NFKC standard; same word → different embeddings | HIGH |
| 13 | D629-13 | NT-WORLD | Sequential pipeline; no DAG orchestration; spam filter after LLM | MEDIUM |
| 14 | D629-14 | NT-WORLD | No declarative normalization config; hardcoded, not composable | MEDIUM |

---

## Sources Cited

1. arxiv.org/html/2605.30813v1 — Incremental BPE Tokenization (2026-05-29)
2. iotdigitaltwinplm.com — LLM Tokenization Deep Dive: BPE, SentencePiece, Tiktoken (2026-05-26)
3. dreaming.press — tiktoken vs SentencePiece vs Hugging Face Tokenizers (2026-06-24)
4. futureagi.com — What is Tokenization in LLMs (2026-04-25)
5. github.com/google/SentencePiece — SentencePiece repository (last push 2026-06-14)
6. sshadows.dk — I Rewrote My Tree-sitter Grammar From Scratch (2026-03-24)
7. dev.to/thegdsks — Parsing 11 Languages in Pure Go Without CGO (2026-04-10)
8. ropensci.org — A Better R Programming Experience Thanks to Tree-sitter (2026-04-02)
9. tree-sitter-tree-sitter.mintlify.app — Tree-sitter Internal Architecture
10. tree-sitter.github.io — Tree-sitter Introduction
11. github.com/BadTrL/text-praline — TextPraline text refinement engine (2026-02-10)
12. noisy-text.github.io/2026/multi-lexnorm.html — W-NUT 2026 Multi-Lexnorm Shared Task
13. huggingface.co/lunahr/CeluneNorm-0.6B-v2.0-ctx1024 — CeluneNorm TTS normalization model
14. aclanthology.org/2026.africanlp-main.13/ — UDMorph and flexiPipe for African NLP
15. arunbaby.com — Advanced NLP Pipelines at Scale (2026-04-18)
16. github.com/gladiaio/normalization — gladia-normalization YAML pipelines (2026-03-04)
