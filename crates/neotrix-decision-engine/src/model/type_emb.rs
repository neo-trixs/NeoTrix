//! Type embedding for question types
//!
//! This is a task-specific head — NOT pretrained in ModernBERT.
//! Initialized randomly and fine-tuned during training.

use candle_core::{Tensor, Module, Device, DType};
use candle_nn::{Embedding, VarBuilder, VarMap};
use crate::error::Result;

/// Question type embedding
/// Maps question types (Noul=0, Choice=1, Score=2) to embeddings
pub struct TypeEmbedding {
    embedding: Embedding,
}

impl TypeEmbedding {
    /// Initialize randomly (task-specific head, not in pretrained weights)
    pub fn new_random(hidden_size: usize, num_types: usize, device: &Device) -> Result<Self> {
        let varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, device);
        let embedding = Embedding::new(
            vb.get((num_types, hidden_size), "weight")?,
            hidden_size,
        );
        Ok(Self { embedding })
    }

    /// Forward pass
    /// qtype: [batch] - question type IDs (0=Noul, 1=Choice, 2=Score)
    /// output: [batch, hidden_size]
    pub fn forward(&self, qtype: &Tensor) -> Result<Tensor> {
        Ok(self.embedding.forward(qtype)?)
    }
}
