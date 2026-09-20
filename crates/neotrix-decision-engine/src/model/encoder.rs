//! ModernBERT Encoder implementation
//!
//! Uses candle-transformers' built-in ModernBERT implementation.
//! The base `ModernBert` struct provides encoder-only forward pass.
//! Weight key prefixes (`model.embeddings.*`, `model.layers.*`, `model.final_norm.*`)
//! match the answerdotai/ModernBERT-large safetensors layout exactly.

use candle_core::{Tensor, Device};
use candle_nn::VarBuilder;
use crate::error::Result;

// Re-export candle-transformers ModernBERT types
pub use candle_transformers::models::modernbert;

/// ModernBERT Encoder wrapper around candle-transformers base model
pub struct ModernBERTEncoder {
    model: modernbert::ModernBert,
}

#[derive(Debug, Clone)]
pub struct BertConfig {
    pub hidden_size: usize,
    pub num_attention_heads: usize,
    pub num_hidden_layers: usize,
    pub intermediate_size: usize,
    pub max_position_embeddings: usize,
    pub vocab_size: usize,
}

impl BertConfig {
    /// ModernBERT-large configuration (answerdotai/ModernBERT-large)
    pub fn modernbert_large() -> Self {
        Self {
            hidden_size: 1024,
            num_attention_heads: 16,
            num_hidden_layers: 28,
            intermediate_size: 2624,
            max_position_embeddings: 8192,
            vocab_size: 50368,
        }
    }

    /// Convert to candle-transformers Config
    fn to_candle_config(&self) -> modernbert::Config {
        modernbert::Config {
            vocab_size: self.vocab_size,
            hidden_size: self.hidden_size,
            num_hidden_layers: self.num_hidden_layers,
            num_attention_heads: self.num_attention_heads,
            intermediate_size: self.intermediate_size,
            max_position_embeddings: self.max_position_embeddings,
            layer_norm_eps: 1e-5,
            pad_token_id: 50283,
            global_attn_every_n_layers: 3,
            global_rope_theta: 160000.0,
            local_attention: 128,
            local_rope_theta: 10000.0,
            classifier_config: None,
        }
    }
}

impl ModernBERTEncoder {
    /// Load from weights using VarBuilder
    pub fn from_weights(
        weights: &VarBuilder,
        config: &BertConfig,
        _device: &Device,
    ) -> Result<Self> {
        let candle_config = config.to_candle_config();
        let model = modernbert::ModernBert::load(weights.clone(), &candle_config)?;
        Ok(Self { model })
    }

    /// Forward pass — returns last hidden state [batch, seq_len, hidden_size]
    pub fn forward(
        &self,
        input_ids: &Tensor,
        attention_mask: &Tensor,
    ) -> Result<Tensor> {
        let output = self.model.forward(input_ids, attention_mask)?;
        Ok(output)
    }
}
