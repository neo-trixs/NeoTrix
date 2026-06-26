pub mod column_layout;
pub mod document_classifier;
pub mod document_parser;
pub mod document_pipeline;
pub mod document_router;
pub mod docx_extractor;
pub mod docx_extractor_backend;
pub mod formula_extractor;
pub mod layout_analyzer;
pub mod markdown_table_extractor;
pub mod novelty_detector;
pub mod office_common;
pub mod parallel_decoder;
pub mod pdf_extractor;
pub mod pdf_extractor_backend;
pub mod structured_chunker;
pub mod unified_search;
pub mod vlm_backend;
pub mod vsa_input_pipeline;
pub mod xls_extractor;
pub mod xls_extractor_backend;
pub mod xlsx_extractor;
pub mod xlsx_extractor_backend;
pub use document_classifier::{ClassificationResult, DocumentClassifier, DocumentType};
pub use document_parser::{
    detect_format, source_to_bytes, DocumentError, DocumentMetadata, DocumentParser,
    DocumentSource, ParsedDocument,
};
pub use document_pipeline::{DocFormat, DocPipeline, Document, FormatDetector};
pub use document_router::{create_default_registry, BackendInfo, DocumentParserRegistry};
pub use formula_extractor::{enrich_document_with_formulas, FormulaElement, FormulaExtractor};
pub use layout_analyzer::{LayoutAnalysis, LayoutAnalyzer, LayoutAnalyzerConfig, RegionInfo};
pub use markdown_table_extractor::extract_markdown_tables;
pub use parallel_decoder::ParallelVsaDecoder;
pub use structured_chunker::{
    chunk_document, ChunkRegion, ChunkStats, ChunkStrategy, DocumentChunk,
};
pub use vlm_backend::VlmBackend;

pub use novelty_detector::{NoveltyDetector, NoveltyLevel, NoveltyStats};
pub use pdf_extractor::{extract_text_from_pdf, PdfDocument, PdfError, PdfExtractor, PdfPage};
pub use unified_search::{SearchQuery, SearchResult, SearchSource, UnifiedSearchEngine};
pub use vsa_input_pipeline::{InputSemanticType, NgramVsaEncoder, VsaInput, VsaInputPipeline};
