//! 解析后端实现
//!
//! Tier 0: PyMuPDF (lopdf), text-only LLM (Pollinations/Groq via GatewayV2)
//! Tier 1: surya OCR+layout, Marker, Docling (Python subprocess)
//! Tier 2: olmOCR, Gemini Vision, GPT-4V (GatewayV2 VLM)

pub mod pymupdf_backend;
pub mod text_only_backend;
pub mod surya_backend;
pub mod marker_backend;
pub mod docling_backend;
pub mod olmocr_backend;
pub mod vision_api_backend;
#[cfg(feature = "office")]
pub mod office_oxide_backend;
