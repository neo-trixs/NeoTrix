//! # NeoTrix Decision Engine
//!
//! A local decision engine inspired by TypeSafe JEV.
//! Provides structured decision-making with calibrated probabilities.
//!
//! ## Features
//!
//! - **Three question primitives**: Noul (boolean), Choice (selection), Score (rating)
//! - **Type-safe outputs**: Answers are constrained to declared schemas
//! - **Calibrated probabilities**: Each answer includes confidence scores
//! - **Parallel evaluation**: All questions evaluated independently
//! - **Local inference**: No external API calls required
//! - **Laya backend**: Direct model inference via candle
//! - **Encoder model**: ModernBERT encoder for fast inference (~33ms/question)
//!
//! ## Architecture
//!
//! ```text
//! State + Questions → Sequence Builder → Model Forward → Post-Processing → Answers
//!                           ↓                    ↓
//!                     [CLS] state [SEP]   ModernBERT + TypeEmb
//!                     opts markers [SEP]  → Scorer → Logits
//! ```

pub mod types;
pub mod engine;
pub mod error;
pub mod backends;
pub mod tokenizer;
pub mod sequence;
pub mod temperature;
pub mod router;
pub mod model;
pub mod laya_engine;

// Re-exports
pub use types::*;
pub use engine::{DecisionEngine, InferenceBackend};
pub use error::{Error, Result};
pub use backends::{LayaBackend, LayaCandleBackend};
pub use backends::simulation::{SimulationBackend, SimulationResponse, simulate_answers};
pub use tokenizer::LayaTokenizer;
pub use sequence::{SequenceBuilder, BuiltSequence};
pub use temperature::TemperatureScaler;
pub use router::{Router, detect_script, Script};
pub use model::{LayaModel, ModelConfig};
pub use laya_engine::LayaEngine;
pub use laya_engine::LayaEngineBuilder;

/// Version of the decision engine
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_question_creation() {
        let q = Question::noul("test", "Test question");
        assert_eq!(q.id, "test");
    }

    #[test]
    fn test_state_from_str() {
        let state: State = "test".into();
        assert_eq!(state.content, "test");
    }
}
