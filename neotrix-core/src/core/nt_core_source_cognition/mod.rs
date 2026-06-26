//! # SourceCognition — 源认知层
//!
//! 眼耳鼻舌身意 — The six sensory modalities that bridge raw data and conscious VSA experience.
//!
//! ## Architecture
//!
//! ```text
//! [bytes/file] → TypeDetector → ParserRouter → SourceParser → SenseEncoder → VsaTagged<SenseModality>
//! ```
//!
//! Each step is independent and composable. The SourceCognitionEngine orchestrates
//! the full pipeline: detect type → route to parser → extract content → encode as VSA → tag with sense.
//!
//! ## Modality Mapping
//!
//! | Sense | 识 | File Types | Parser |
//! |-------|-----|------------|--------|
//! | Visual | 眼 | PDF, images, HTML | PdfParser |
//! | Auditory | 耳 | (reserved) | — |
//! | Olfactory | 鼻 | Unknown binary (scent) | BinaryParser |
//! | Gustatory | 舌 | (reserved for taste evaluation) | — |
//! | Tactile | 身 | (reserved for haptic/physical) | — |
//! | Mental | 意 | Text, code, JSON, markup | TextParser |

pub mod parser_router;
pub mod parsers;
pub mod sense_encoder;
pub mod sense_modality;
pub mod source_cognition;
pub mod type_detector;

pub use parser_router::ParserRouter;
pub use parsers::binary_parser::BinaryParser;
pub use parsers::pdf_parser::PdfParser;
pub use parsers::text_parser::TextParser;
pub use parsers::{ParseError, ParsedContent, SourceParser};
pub use sense_encoder::SenseEncoder;
pub use sense_modality::SenseModality;
pub use source_cognition::{SourceCognitionEngine, SourceCognitionStats};
pub use type_detector::{DetectedType, TypeDetector};
