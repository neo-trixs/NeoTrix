//! nt_world_parse — 统一文档解析网关
//!
//! Router, not extractor. 每页路由到最佳后端, 4-signal 置信度审计, 自动回退重试。
//! 参考: pdfmux (router+audit), Marker (混合管线), Docling (JSON树), olmOCR (VLM)
//!
//! 架构:
//!   ParseGateway ─→ per-page routing
//!       ├── Tier 0: PyMuPDF / text-only LLM (0.01s/page, CPU)
//!       ├── Tier 1: surya OCR+layout + Marker/Docling (0.1-1s/page, CPU)
//!       └── Tier 2: olmOCR VLM / Gemini Vision (1-10s/page, GPU)
//!
//!   每页结果 → 4-signal confidence audit → ≥阈值? 返回 : 更强后端重试

pub mod doc_parser;
pub mod parse_gateway;
pub mod confidence;
pub mod backends;
pub mod processors;
pub mod renderers;
pub mod render;
pub mod ingest;
pub mod mcp;
pub mod cli;

pub use doc_parser::{DocParser, ParsedDocument, ParseTier};
pub use parse_gateway::ParseGateway;
pub use confidence::ConfidenceScorer;
