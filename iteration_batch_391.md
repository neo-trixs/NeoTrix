# Iteration Batch 391 — Document AI / OCR / Information Extraction Research

**Date**: 2026-09-06  
**Focus**: nt_file_ability domain gap analysis against 2026 Document AI, OCR, and KIE research  
**Sources**: 18 papers/reports across CVPR 2026, ACL 2026, EACL 2026, ICDAR 2026, arXiv 2026

---

## Sources Cited

1. Yu et al., "MACT: Visual Document Understanding and Reasoning with Multi-Agent Collaboration and Adaptive Test-Time Scaling," *CVPR 2026* — multi-agent procedural scaling for document understanding
2. Cui et al., "PaddleOCR-VL: Boosting Document Parsing Efficiency with Coarse-to-Fine Visual Processing," *CVPR 2026* — 0.9B VLM with Valid Region Focus Module, SOTA on OmniDocBench v1.5
3. "PARL: Position-Aware Relation Learning Network for OCR-Free Document Layout Analysis," *arXiv:2601.07620* — 65M vision-only layout analysis surpassing multimodal models
4. "MinerU2.5: A Decoupled Vision-Language Model for Efficient High-Resolution Document Parsing," *ACL Industry 2026* — 1.2B two-stage coarse-to-fine parsing
5. "Qianfan-OCR: End-to-End Document Intelligence with Layout-as-Thought," *arXiv:2603.13398* — 4B unified OCR+understanding, OmniDocBench v1.5 #1 (93.12)
6. "GLM-OCR: Efficient 0.9B Compact Multimodal Model for Document Understanding," *arXiv:2603.10910* — 0.9B model with Multi-Token Prediction, OmniDocBench 94.6
7. "PP-OCRv6: From 1.5M to 34.5M Parameters, Surpassing Billion-Scale VLMs on OCR Tasks," *arXiv:2606.13108* — lightweight 50-language OCR family
8. "OmniHandwritingOCR: Diagnostic Benchmark for Evaluating MLLMs in Handwritten OCR," *CIKM 2026* — handwritten OCR failure modes across 77K images
9. "Unlimited-OCR: One-shot Long-horizon Parsing," *arXiv:2606.23050* — multi-page PDF parsing with ngram-window decoding
10. IBM Research, "Identify, Locate, Link: End-to-End Key-Value Extraction from Document Images," *ICDAR 2026* — 256M VLM end-to-end KIE without OCR preprocessing
11. "PaDoc: Layout-Guided Parallel Decoding for Document Parsing," *arXiv:2608.06146* — layout-grounded parallel branch decoding, 67-118% throughput improvement
12. "TDATR: Table Detail-Aware Table Recognition with Cell-Level Visual Alignment," *CVPR 2026* — perceive-then-fuse strategy for end-to-end table recognition
13. "TASER: Table Agents for Schema-guided Extraction and Recommendation," *EACL Industry 2026* — continuously learning agentic table extraction, 10.1% improvement over Table Transformer
14. "Infinity-Parser: Layout-Aware Reinforcement Learning with High-quality Document Parsing Dataset," *ACL Findings 2026* — RL framework with composite rewards for document parsing
15. Shihab et al., "LLM-Guided Probabilistic Fusion for Label-Efficient Document Layout Analysis," *CVPR 2026* — LLM structural priors for semi-supervised layout detection
16. "Deep Learning Based Visually Rich Document Content Understanding," *Artificial Intelligence Review* (2026-02) — comprehensive VRD-CU survey covering KIE, QA, entity linking
17. "PP-OCRv6 Introduction," *PaddleOCR Docs* (2026-06) — 50-language unified model family, production benchmarks
18. "GLM-OCR Documentation," *Z.AI* (2026) — real-world scenario benchmarks for code/handwriting/seal/receipt OCR

---

## Defects Found

### DEFECT-001: OCR Engine Is a Fake Placeholder — Zero Real Vision Recognition
**Severity**: CRITICAL  
**Location**: `neotrix-core/src/neotrix/nt_file_ability/ocr.rs:37-55`  
**Research**: PP-OCRv6 (2026) demonstrates that a 34.5M-parameter lightweight model achieves 83.2% recognition accuracy and 86.2% detection Hmean, surpassing billion-parameter VLMs like Qwen3-VL-235B and GPT-5.5. GLM-OCR (0.9B) achieves OmniDocBench 94.6. Even tiny models (1.5M params) are production-viable.  
**Current**: `RuleBasedOcr` extracts the **filename stem** as OCR text with confidence 0.05. This is not OCR — it's a file-naming heuristic. No actual image-to-text recognition exists.  
**Gap**: NeoTrix has zero real OCR capability. Any image/PDF-with-scanned-pages processing returns garbage. The `OcrEngine` trait is correctly designed but has no production implementation.  
**Suggestion**: Integrate PP-OCRv6 (tiny tier: 1.5M params, edge-viable) as the default `OcrEngine` implementation. The trait abstraction already exists — add `PaddleOcrEngine` wrapping the ONNX export. For VLM-quality, expose GLM-OCR or PaddleOCR-VL via the `nt_io::platform_gateway` pattern (local Ollama → cloud API fallback).

---

### DEFECT-002: Document Parsing Has No Layout Awareness — Flat Text Dump
**Severity**: CRITICAL  
**Location**: `neotrix-core/src/neotrix/nt_file_ability/doc_parse.rs:34-45` (`document_to_filemodel`)  
**Research**: MinerU2.5 (ACL 2026) demonstrates that decoupled coarse-to-fine layout analysis + content recognition achieves SOTA by preserving spatial structure. PaddleOCR-VL's Valid Region Focus Module (CVPR 2026) identifies that only 39-60% of document regions are semantically valid — processing everything is wasteful. Infinity-Parser (ACL 2026) uses layout-aware RL with reading order preservation.  
**Current**: `document_to_filemodel()` calls `blocks_to_text()` which joins all blocks with `\n\n`, discarding all spatial information. The `FileModel` struct has no bounding boxes, no reading order, no element type labels. Tables are extracted but layout structure is lost.  
**Gap**: NeoTrix cannot answer "where is X on the page?" or "what's the reading order?" — critical for multi-column documents, forms, invoices. No spatial grounding means no layout-aware KIE.  
**Suggestion**: Extend `FileModel` with `Vec<LayoutRegion> { bbox: Rect, region_type: RegionType, reading_order: usize, content: String }`. Implement a `LayoutAnalysisEngine` trait analogous to `OcrEngine` — default `RuleBasedLayout` (bounding box heuristic), production `VlmLayout` (GLM-OCR/PaddleOCR-VL). The `CoarseToFine` pattern from MinerU2.5 should guide the architecture: Stage 1 layout detection → Stage 2 content recognition per region.

---

### DEFECT-003: No End-to-End Key-Value Extraction — Missing KIE Pipeline
**Severity**: HIGH  
**Location**: Missing entirely from `nt_file_ability`. No KIE trait, no extraction pipeline.  
**Research**: IBM Research (ICDAR 2026) demonstrates a 256M VLM performing end-to-end KIE (identify, locate, link) without OCR preprocessing, outperforming zero-shot Qwen2.5-VL (7B) under layout-aware evaluation. TASER (EACL 2026) shows continuously learning agentic extraction from schema-guided multi-page tables. The VRD-CU survey (2026) categorizes KIE into 5 framework types: feature-driven, joint-learning, relation-aware, few/zero-shot, prompt-based.  
**Current**: `FileModel` has `tables: Option<Vec<Value>>` but no key-value structure, no field labels, no link graph. Document parsing produces flat text + raw table JSON.  
**Gap**: NeoTrix cannot extract structured fields from invoices, receipts, forms, contracts — the majority of enterprise document use cases. No spatial key-value linking means no disambiguation of repeated fields.  
**Suggestion**: Add `KieEngine` trait with `extract_kv(&self, doc: &FileModel) -> Vec<KvPair { key: String, value: String, key_bbox: Rect, value_bbox: Rect, confidence: f64 }>`. Default implementation: LLM-prompt-based extraction using the existing `nt_io::llm_provider`. Production: integrate SmolDocling-style DocTags with `<key_i>`, `<value_i>`, `<link_i>` tags for many-to-many KV relationships.

---

### DEFECT-004: No Multi-Page Document Reasoning — Single-Page Silo
**Severity**: HIGH  
**Location**: `neotrix-core/src/neotrix/nt_file_ability/doc_parse.rs:291-303` (`parse_pdf_bytes_enhanced_with_config`)  
**Research**: Unlimited-OCR (arXiv 2026) introduces multi-page PDF parsing with ngram-window decoding for cross-page continuity. PaDoc (arXiv 2026) implements layout-grounded parallel decoding where content branches share a page-image prefix. The VRD-CU survey (2026) notes that single-page models with 512-token constraints fail on multi-page documents — recent work uses long-sequence transformers and hierarchical encoders.  
**Current**: `parse_pdf_enhanced_with_config` processes each PDF page independently via `to_document()`. There is no cross-page content linking, no multi-page table merging, no reading order across pages. `max_pages` config exists but is ignored (no `_config` usage).  
**Gap**: Financial filings (44-page tables per TASER), legal contracts, and multi-page invoices cannot be parsed coherently. Page-level processing breaks entity continuity across page boundaries.  
**Suggestion**: Implement `MultiPageDocumentModel` with: (1) per-page `LayoutRegion` list, (2) cross-page `ReadingOrderGraph` connecting last region of page N to first of page N+1, (3) `MergeStrategy` for tables spanning pages (entity resolution on headers). Use the `BatchProcessor` pattern already in `nt_file_ability` but add inter-page state.

---

### DEFECT-005: PDF Parsing Is Pure Fallback — No VLM Integration
**Severity**: HIGH  
**Location**: `neotrix-core/src/neotrix/nt_file_ability/doc_parse.rs:291-303`  
**Research**: Qianfan-OCR (2026) shows end-to-end VLM achieves 93.12 on OmniDocBench, while two-stage OCR+LLM pipelines degrade to zero on chart interpretation. GLM-OCR demonstrates 94.6 with 0.9B parameters. The "Layout-as-Thought" mechanism shows VLMs can recover layout analysis within end-to-end generation.  
**Current**: `parse_pdf_bytes_enhanced_with_config` accepts `_config` (ignored), always falls back to `to_document()` with metadata `"fallback_reason": "marker crate not yet integrated"`. The `PdfParseConfig.vlm_model` and `llm_model` fields are never read. No VLM call is ever made.  
**Gap**: The entire PDF enhancement pipeline is a stub. Config promises VLM integration but delivers basic text extraction. Scanned PDFs, image-heavy PDFs, and complex-layout PDFs produce garbage.  
**Suggestion**: Implement the VLM pipeline: (1) rasterize PDF pages to images, (2) dispatch to `VlmOcrEngine` (GLM-OCR via Ollama or cloud API), (3) use `Layout-as-Thought` prompting for complex layouts, (4) merge per-page results with cross-page entity linking. The `PdfParseConfig` struct already has all needed fields — wire them up.

---

### DEFECT-006: Table Extraction Loses Structural Semantics
**Severity**: MEDIUM  
**Location**: `neotrix-core/src/neotrix/nt_file_ability/doc_parse.rs:131-139` (`table_to_json`)  
**Research**: TDATR (CVPR 2026) proposes cell-level visual alignment and structure-guided cell localization for accurate table recognition. PubTabNet benchmarks show that span cells (merged cells) are a major challenge — current models struggle with hierarchical table structures. TASER (EACL 2026) handles heterogeneous financial tables where 99.4% lack bounding boxes and the largest spans 44 pages.  
**Current**: `table_to_json` extracts `headers` (first row) and `rows` (remaining rows) as flat string arrays. Span cells (merged cells via `CellSlot::Covered`) are silently dropped to empty strings. No cell bounding boxes, no row/column spans, no HTML structure.  
**Gap**: Complex tables with merged cells (invoices, financial reports, academic papers) lose critical structure. A merged header spanning 3 columns becomes 3 empty cells + 1 populated cell, destroying the semantic meaning.  
**Suggestion**: Extend table output to include: `span_cells: Vec<SpanCell { row_span, col_span, content, bbox }>` and `html: String` (rendered HTML preserving merge structure). Use TDATR's "perceive-then-fuse" pattern: structure understanding (cell detection + span prediction) → content recognition → HTML generation.

---

### DEFECT-007: No Handwriting Recognition — Blind to Handwritten Input
**Severity**: MEDIUM  
**Location**: `neotrix-core/src/neotrix/nt_file_ability/ocr.rs:44-55`  
**Research**: OmniHandwritingOCR (CIKM 2026) evaluates 13 OCR systems across 77K handwritten images — performance drops sharply on complex multi-line formulas, and generative models hallucinate plausible but visually unsupported corrections. PP-OCRv6 achieves 62.1% on handwritten Chinese and 67.8% on handwritten English (real benchmarks). GLM-OCR scores 87.0 on handwritten text recognition.  
**Current**: `RuleBasedOcr` returns filename as text — completely blind to handwritten content in images or scanned documents.  
**Gap**: NeoTrix cannot process any handwritten input: medical prescriptions, handwritten notes, signed forms, whiteboard photos. This is a hard blocker for the `nt_file_ability` branch's doc-parse SKILL.md promise of "任意文件→统一文件模型".  
**Suggestion**: After integrating PP-OCRv6 or GLM-OCR as the base `OcrEngine`, add a `HandwritingMode` enum (`Printed`, `Handwritten`, `Mixed`, `Auto`) to `OcrConfig`. For `Handwritten`/`Mixed`, route to GLM-OCR which specifically optimizes for handwriting. Add confidence-based fallback: if handwriting confidence < threshold, retry with VLM.

---

### DEFECT-008: No Self-Correction Loop — Error Cascading Without Verification
**Severity**: MEDIUM  
**Location**: `neotrix-core/src/neotrix/nt_file_ability/doc_parse.rs:291-303`  
**Research**: MACT (CVPR 2026) demonstrates that a **judgment agent** with self-correction outperforms existing mechanisms while requiring fewer corrections. The key insight: monolithic feed-forward models lack internal verification — initial extraction errors cascade without being challenged. Qianfan-OCR shows Layout-as-Thought improves accuracy on structurally complex documents by forcing structural reasoning before generation.  
**Current**: `parse_pdf_bytes_enhanced_with_config` makes a single pass through `to_document()` with no verification, no confidence scoring, no retry on low-confidence regions. Results are returned as-is.  
**Gap**: Errors in OCR/layout/table extraction propagate silently to downstream consumers (KB ingestion, search indexing). No mechanism exists to detect or correct extraction failures.  
**Suggestion**: Implement `ExtractionVerifier` trait: after initial parse, run a lightweight verification pass (e.g., LLM sanity check on extracted key fields, confidence threshold filtering). Use MACT's judgment-agent pattern: if extraction confidence < threshold, re-parse with higher-compute VLM or flag for human review. Wire into the SEAL quality gate pipeline.

---

### DEFECT-009: No Multilingual Document Support
**Severity**: MEDIUM  
**Location**: `neotrix-core/src/neotrix/nt_file_ability/ocr.rs` and `doc_parse.rs`  
**Research**: PP-OCRv6 (2026) supports **50 languages** in a single unified model (medium/small tiers). GLM-OCR benchmarks multilingual text at 69.3% (vs Gemini-3-Pro at 86.2%). Qianfan-OCR training data includes "15% Specialized OCR tasks including handwriting, formulas, tables, and multilingual text."  
**Current**: No language detection, no multilingual OCR, no script-aware processing. `RuleBasedOcr` operates on filenames only. `FileModel` has no language field.  
**Gap**: NeoTrix cannot process documents in Japanese, Korean, Arabic, Hindi, or any non-Latin script beyond what `anydoc` handles for structured formats. Multilingual document ingestion is blocked.  
**Suggestion**: Add `language: Option<String>` to `FileModel` and `OcrResult`. Integrate PP-OCRv6's 50-language support. Add script detection as a pre-processing step (Unicode range heuristic → language confirmation via OCR confidence). Route to language-specific recognition heads when available.

---

### DEFECT-010: No Formula/Equation Recognition
**Severity**: LOW  
**Location**: `neotrix-core/src/neotrix/nt_file_ability/doc_parse.rs`  
**Research**: Qianfan-OCR achieves 91.92-92.43 Formula CDM scores. MinerU2.5 explicitly handles formula recognition in its second stage. PP-OCRv6 benchmarks include formula recognition. TDATR (CVPR 2026) handles formulas as part of table detail-aware learning.  
**Current**: `Block::Math(tex)` in anydoc preserves raw LaTeX, but this only works for native-digital PDFs with embedded math. Scanned documents with handwritten/printed formulas produce garbled text.  
**Gap**: Academic papers, math textbooks, and scientific documents with formulas cannot be accurately parsed from scans.  
**Suggestion**: Add formula detection to the layout analysis stage (RegionType::Formula). For detected formula regions, use specialized formula recognition (UniMERNet-style) or VLM with math-specific prompting. Store as LaTeX in `FileModel` with a `formula_confidence` field.

---

## Summary Table

| # | Defect | Severity | Current State | 2026 Research Baseline |
|---|--------|----------|--------------|----------------------|
| 1 | Fake OCR placeholder | CRITICAL | Filename extraction | PP-OCRv6 83.2% acc, GLM-OCR 94.6 OmniDocBench |
| 2 | No layout awareness | CRITICAL | Flat text dump | MinerU2.5 coarse-to-fine, PARL 65M vision-only SOTA |
| 3 | No KIE pipeline | HIGH | No key-value extraction | IBM 256M end-to-end KIE, TASER agentic extraction |
| 4 | Single-page silo | HIGH | No cross-page reasoning | Unlimited-OCR multi-page, PaDoc parallel decoding |
| 5 | PDF parsing stub | HIGH | Config fields ignored | Qianfan-OCR 93.12, GLM-OCR 94.6 end-to-end |
| 6 | Table structure loss | MEDIUM | Flat header/rows | TDATR cell-level alignment, span cell handling |
| 7 | No handwriting | MEDIUM | Zero handwriting support | PP-OCRv6 62-67% HW, GLM-OCR 87.0 HW |
| 8 | No self-correction | MEDIUM | Single-pass no verify | MACT judgment agent, Layout-as-Thought |
| 9 | No multilingual | MEDIUM | English-only heuristic | PP-OCRv6 50-language unified model |
| 10 | No formula recognition | LOW | LaTeX passthrough only | Qianfan-OCR 92.4 CDM, MinerU2.5 formula stage |

---

## Prioritized Suggestions

1. **Immediate (P0)**: Replace `RuleBasedOcr` with PP-OCRv6 tiny (1.5M params, edge-viable) — single file change in `ocr.rs`, unlocks real OCR
2. **Short-term (P1)**: Add `LayoutRegion` to `FileModel` + implement `LayoutAnalysisEngine` trait — foundational for all spatial tasks
3. **Short-term (P1)**: Wire up `PdfParseConfig.vlm_model` to actual VLM call — the config exists, the integration doesn't
4. **Medium-term (P2)**: Add `KieEngine` trait with LLM-prompt-based default — covers 80% of enterprise document extraction use cases
5. **Medium-term (P2)**: Implement multi-page document model with cross-page entity linking — blocks financial/legal document processing
6. **Long-term (P3)**: Table structure enhancement (span cells, cell bounding boxes) + formula recognition — academic/financial document quality
