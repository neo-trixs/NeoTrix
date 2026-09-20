//! Laya Candle backend — pure Rust inference via LayaEngine
//!
//! Wraps the LayaEngine (ModernBERT encoder) behind the DecisionBackend trait
//! so the Router can use it for multi-language routing.

use std::path::Path;
use crate::router::DecisionBackend;
use crate::error::Result;
use crate::types::*;
use crate::laya_engine::LayaEngine;
use crate::model::ModelConfig;
use crate::temperature::TemperatureScaler;
use candle_core::Device;

/// Pure Rust Laya backend using ModernBERT encoder
pub struct LayaCandleBackend {
    engine: LayaEngine,
}

impl LayaCandleBackend {
    /// Create from model files
    pub fn from_files(
        weights_path: &Path,
        tokenizer_path: &Path,
    ) -> Result<Self> {
        let config = ModelConfig::modernbert_large();
        let temperature = TemperatureScaler::default();
        let device = Device::Cpu;

        let engine = LayaEngine::from_files(
            weights_path,
            tokenizer_path,
            config,
            temperature,
            device,
        )?;

        Ok(Self { engine })
    }

    /// Create from files with custom config
    pub fn from_files_with_config(
        weights_path: &Path,
        tokenizer_path: &Path,
        config: ModelConfig,
        temperature: TemperatureScaler,
        device: Device,
    ) -> Result<Self> {
        let engine = LayaEngine::from_files(
            weights_path,
            tokenizer_path,
            config,
            temperature,
            device,
        )?;

        Ok(Self { engine })
    }

    /// Get a reference to the inner engine
    pub fn engine(&self) -> &LayaEngine {
        &self.engine
    }
}

impl DecisionBackend for LayaCandleBackend {
    fn predict(&self, state: &State, questions: &[Question]) -> Result<EvaluationResult> {
        self.engine.evaluate(state, questions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn project_root() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.pop();
        path.pop();
        path
    }

    #[test]
    fn test_laya_candle_backend_creation() {
        let root = project_root();
        let weights = root.join("models/modernbert-large/model.safetensors");
        let tokenizer = root.join("models/modernbert-large/tokenizer.json");

        if !weights.exists() || !tokenizer.exists() {
            eprintln!("Skipping: model files not found");
            return;
        }

        let backend = LayaCandleBackend::from_files(&weights, &tokenizer).unwrap();
        assert_eq!(backend.engine().evaluate(
            &"test".into(),
            &[Question::noul("q", "test?")],
        ).unwrap().answers.len(), 1);
    }
}
