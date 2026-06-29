# VLM Document Parsing Evolution — Architecture & Roadmap

> **Phase**: 26 Wave A · VLM Document Parsing  
> **Status**: Phase 3.5 + Phase 4 Complete ✅ · Classification-Aware VLM Routing  
> **Next**: Native Inference Crate (Phase 3.4)

## Current State Audit

### ALIVE — Production Wired

| Module | File | Lines | Wiring Point |
|--------|------|-------|-------------|
| ImagePipeline | `nt_world_vision/mod.rs` | ~400 | consciousness_cycle GATHER (LLM-based image desc → VSA) |
| ImageCache | `nt_world_vision/image_cache.rs` | ~250 | GATHER dhash dedup |
| DocumentPerceptionModule | `nt_world_document/file_perception.rs` | ~350 | GATHER document file path detection |
| WebContentExtractor | `nt_core_experience/web_content_extractor.rs` | 681 | GATHER URL→structured content |
| PdfExtractor | `nt_core_input/pdf_extractor.rs` | 969 | Used by document_pipeline.rs |

### DEAD — Registered, Never Constructed/Run

| Module | File | Lines | Gap |
|--------|------|-------|-----|
| **PixelPerceptionPipeline** | `pixel_perception.rs` | 509 | Registered + exported, no field in ConsciousnessCycle |
| **LongHorizonOcr** | `long_horizon_ocr.rs` | 791 | Registered, NOT exported, no field in ConsciousnessCycle |
| **VisualEmbeddingFrontend** | `visual_embedding_frontend.rs` | 338 | Registered in hcube/ but no pub use |

## Phase 1 — Dead Module Wiring ✅ (This Session)

### 1.1 PixelPerceptionPipeline → ConsciousnessCycle (509 LOC revived)
- **Import**: `use super::pixel_perception::PixelPerceptionPipeline`
- **Struct field**: `pixel_perception: Option<PixelPerceptionPipeline>`
- **Constructor**: `Some(PixelPerceptionPipeline::new())`
- **Clone**: `self.pixel_perception.clone()`
- **Config flag**: `enable_pixel_perception: bool`
- **Builder**: `with_pixel_perception()`
- **GATHER step**: If visual input detected, create VisualTile proxy → `process_tile()` → VSA buffer. Stats available via `scene_buffer()`.

### 1.2 LongHorizonOcr → ConsciousnessCycle (791 LOC revived)
- **Import**: `use super::long_horizon_ocr::{LongHorizonOcr, OcrConfig}`
- **Struct field**: `long_horizon_ocr: Option<LongHorizonOcr>`
- **Constructor**: `Some(LongHorizonOcr::new(OcrConfig::default()))`
- **Clone**: `self.long_horizon_ocr.clone()`
- **Config flag**: `enable_long_horizon_ocr: bool`
- **Builder**: `with_long_horizon_ocr()`
- **GATHER step**: If Document perception → `DocumentSource` → `ocr.process()` → structured doc.

### 1.3 VisualEmbeddingFrontend Export (338 LOC accessible)
- **hcube/mod.rs**: Add `pub use visual_embedding_frontend::VisualEmbeddingFrontend;`

### Revival Summary

| Module | Revived LOC | Wiring LOC | ROI |
|--------|-------------|------------|:---:|
| PixelPerceptionPipeline | 509 | ~15 | 34:1 |
| LongHorizonOcr | 791 | ~15 | 53:1 |
| VisualEmbeddingFrontend | 338 | 1 | 338:1 |
| **Total** | **1,638** | **~31** | **53:1** |

## Phase 2 — VLM Document Model Integration ✅ (Current Session)

### 2.1 DocumentParser Trait

```rust
pub trait DocumentParser: Send {
    fn parse(&self, source: &DocumentSource) -> Result<ParsedDocument, DocumentError>;
    fn supported_formats(&self) -> Vec<&'static str>;
    fn backend_name(&self) -> &'static str;
}
```

**File**: `neotrix-core/src/core/nt_core_input/document_parser.rs` (323 LOC, 13 tests)

### 2.2 PdfExtractorBackend

Wraps existing PdfExtractor into a DocumentParser. Zero external dependencies, text-only PDF extraction.

**File**: `neotrix-core/src/core/nt_core_input/pdf_extractor_backend.rs` (109 LOC, 3 tests)

### 2.3 VLM HTTP Backend

Generic HTTP-based VLM backend supporting Docling, Dolphin, and Nemotron document parsing APIs. Uses existing `reqwest 0.11` (blocking + json) and `base64 0.22`.

**File**: `neotrix-core/src/core/nt_core_input/vlm_backend.rs` (388 LOC, 17 tests)

### 2.4 DocumentParserRegistry

Routes DocumentSource to the best backend based on supported formats.

**File**: `neotrix-core/src/core/nt_core_input/document_router.rs` (~195 LOC, 6 tests)

### 2.5 GATHER Step Wiring

DocumentParserRegistry added as `document_parser_registry` field in ConsciousnessCycle with config flag, builder, constructor init, Clone/Debug, and GATHER step invocation.

**File**: `consciousness_cycle.rs` (+~30 LOC)

### Revival Summary (Phase 1 — Phase 4)

| Module | LOC | Status |
|--------|:---:|:------:|
| PixelPerceptionPipeline | 509 | ✅ Phase 1 |
| LongHorizonOcr | 791 | ✅ Phase 1 |
| VisualEmbeddingFrontend | 338 | ✅ Phase 1 (pub use) |
| DocumentParser trait | 323 | ✅ Phase 2 — NEW |
| PdfExtractorBackend | 109 | ✅ Phase 2 — NEW |
| VlmBackend (HTTP) | 388 | ✅ Phase 2 — NEW |
| DocumentParserRegistry | ~195 | ✅ Phase 2 — NEW |
| DocumentClassifier | 578 | ✅ Phase 3 — NEW |
| FormulaExtractor | 585 | ✅ Phase 3 — NEW |
| classify_and_parse() | ~40 | ✅ Phase 3.5 — NEW |
| VlmBackend::from_env() | ~20 | ✅ Phase 3.5 — NEW |
| init_missing_fields() | ~60 | ✅ Phase 4 — NEW |
| feed_parsed_document() | ~25 | ✅ Phase 4 — NEW |
| try_register_vlm_from_env() | ~30 | ✅ Phase 4 — NEW |
| **Total** | **~3,991** | **All revived/built** |

## Phase 3 — Document Classification + Formula Extraction ✅ (Current Session)

### 3.1 DocumentClassifier

Classifies documents by type before routing: scanned PDF, digital PDF, image, office doc, web page, plain text. Uses magic bytes for format detection and PDF content analysis (/Font vs /Image) for scanned vs digital PDF distinction.

**File**: `neotrix-core/src/core/nt_core_input/document_classifier.rs` (578 LOC, 26 tests)

### 3.2 FormulaExtractor

Post-processing step that detects LaTeX formulas ($...$, $$...$$, \\(...\\), \\[...\\], \\begin{equation}...\\end{equation}) and MathML (<math>...</math>) in parsed documents. Enriches DocumentMetadata with formula_count.

**File**: `neotrix-core/src/core/nt_core_input/formula_extractor.rs` (585 LOC, 17 tests)

### 3.3 GATHER Step Integration

- DocumentClassifier runs before DocumentParserRegistry to classify the incoming document
- FormulaExtractor runs after parsing to detect math content
- Both gated by CycleConfig flags (enable_document_classifier, enable_formula_extractor)

**File**: `consciousness_cycle.rs` (+~40 LOC)

### 3.4 Future: Native Inference Crate

- `neotrix-infer` crate (optional, behind feature flag):
  - ONNX runtime for DocLayNet-v2 layout model (~50MB)
  - Tokenize → layout → VSA region encoding
  - No GPU required for inference (CPU ONNX)

## Phase 3.5 — Classification-Aware VLM Routing ✅ (Current Session)

### 3.5.1 Registry classify_and_parse()

`DocumentParserRegistry.classify_and_parse(source, bytes)` combines classification + backend selection + parsing in one call:

- **ScannedPdf / Image** (confidence > 0.5): prefer VlmBackend over PdfExtractorBackend
- **DigitalPdf / other**: standard format-based routing (PdfExtractorBackend)
- VLM failure → graceful fallback to format-based routing

**File**: `document_router.rs` (+~40 LOC)

### 3.5.2 VlmBackend::from_env()

Auto-constructs VLM backend from env vars: `VLM_BACKEND_TYPE`, `VLM_BACKEND_URL`, `VLM_API_KEY`.

Returns `None` if required vars are missing. Supports `docling`, `dolphin`, `nemotron` types.

**File**: `vlm_backend.rs` (+~20 LOC, +3 tests)

### 3.5.3 GATHER Wiring: VLM Fallback

The GATHER step now:
1. Calls `dpr.classify_and_parse()` — classification-aware routing
2. If that fails → `VlmBackend::from_env()` as last-resort VLM fallback
3. On success → FormulaExtractor → DocumentPerceptionModule.feed_parsed_document()

**File**: `consciousness_cycle.rs` (replaced ~20 LOC)

## Phase 4 — Self-Healing Module Registry + Env Registration ✅ (Current Session)

### 4.1 init_missing_fields()

Adds `init_missing_fields()` to ConsciousnessCycle that auto-initializes any `None` subsystem fields with their defaults at the start of `run_cycle()`. Prevents wiring regressions when new `Option<T>` fields are added but forgotten in the constructor.

- Heals ~40 subsystem fields (all `Option<T>` fields in the struct)
- Only logs at `info!` if any field was actually healed
- Zero behavioral change when all fields are already initialized

**File**: `consciousness_cycle.rs` (+~60 LOC for method + call site)

### 4.2 feed_parsed_document()

`DocumentPerceptionModule.feed_parsed_document(parsed: &ParsedDocument)` stores parsed document results (markdown, table/figure/formula counts) for downstream VSA/knowledge pipeline access.

**File**: `file_perception.rs` (+~25 LOC, +4 fields)

### 4.3 try_register_vlm_from_env()

`DocumentParserRegistry.try_register_vlm_from_env()` checks env vars and constructs+registers the appropriate VlmBackend. `create_default_registry()` now auto-invokes this (no-op when env vars absent).

**File**: `document_router.rs` (+~30 LOC, +2 tests)

## Architecture Diagram (Phase 3.5 + Phase 4 ✅)

```
User Input (file path / URL / bytes)
  │
  ├─ Web URL ──→ WebContentExtractor ──→ Structured text ──→ VSA
  │
  └─ Document ──→ DocumentParserRegistry.classify_and_parse()
                    ├─ Scanned PDF / Image (conf > 0.5) ──→ VLM Backend
                    │   ├─ Docling/Dolphin/Nemotron (from_env)
                    │   ├─ Table extraction + Formula recognition
                    │   └─ Markdown output + spatial metadata
                    │
                    ├─ Digital PDF ──→ PdfExtractorBackend ──→ raw text
                    │
                    └─ Falls back to format-based routing
                         └─ If all fail → VlmBackend::from_env() last-resort

All success paths → FormulaExtractor → DocumentPerceptionModule.feed_parsed_document()
                                                    └─ VSA grounding bridge

ConsciousnessCycle GATHER:
  ├─ PixelPerceptionPipeline ──── tile → VSA embedding → scene buffer
  ├─ LongHorizonOcr ───────────── document source → OCR tiles → structured doc → VSA grounding
  ├─ DocumentParserRegistry ───── classify_and_parse() → VLM/text → FormulaExtract → DPM feed
  ├─ ImagePipeline ────────────── image → LLM desc → VSA encode
  ├─ DocumentPerception ───────── file path → extraction
  └─ WebContentExtractor ──────── URL → structured content

Self-Healing:
  └─ init_missing_fields() ───── auto-initializes 43 Option<T> fields at cycle start
```

## Decision Log

| Decision | Rationale |
|----------|-----------|
| DocumentParser trait for VLM backends | Keep neotrix-core inference-agnostic; swap backends via config |
| Phase 1 wiring before VLM integration | Revive 1,638 LOC dead code with ~31 LOC wiring before building new capabilities |
| PdfExtractorBackend as native fallback | Zero-cost path for text-only PDFs; VLM only when layout needed |
| Docling + Dolphin as primary VLM refs | Docling: best structure (unified model), Dolphin: most element types (15) |
| MinerU as subprocess fallback | Best-known open-source pipeline but Python-only; CLI bridge acceptable |
| Nemotron as commercial option | NVIDIA-backed, production-ready spatial grounding |
| FormulaExtractor runs on parsed text, not raw PDF | ParsedDocument.markdown is the natural input for LaTeX detection; no need for PDF stream parsing |
