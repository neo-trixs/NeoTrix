//! Scorer head for question answers
//!
//! Matches upstream: nn.Sequential(nn.LayerNorm(d), nn.Linear(d, d), nn.GELU(), nn.Linear(d, 1))

use candle_core::{Tensor, Device, DType};
use candle_nn::{Linear, VarBuilder, VarMap, LayerNorm, Module};
use crate::error::Result;

/// Scorer head: LayerNorm -> Linear -> GELU -> Linear
pub struct ScorerHead {
    layer_norm: LayerNorm,
    dense1: Linear,
    dense2: Linear,
}

impl ScorerHead {
    /// Initialize (task-specific head, not in pretrained weights)
    pub fn new(hidden_size: usize, device: &Device) -> Result<Self> {
        let varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, device);

        let layer_norm = LayerNorm::new(
            vb.get((hidden_size,), "layer_norm.weight")?,
            vb.get((hidden_size,), "layer_norm.bias")?,
            1e-5,
        );

        let w1 = vb.get((hidden_size, hidden_size), "dense1.weight")?;
        let b1 = vb.get(hidden_size, "dense1.bias")?;
        let dense1 = Linear::new(w1, Some(b1));

        let w2 = vb.get((1, hidden_size), "dense2.weight")?;
        let b2 = vb.get(1, "dense2.bias")?;
        let dense2 = Linear::new(w2, Some(b2));

        Ok(Self { layer_norm, dense1, dense2 })
    }

    /// Forward pass
    /// input: [batch, num_markers, hidden_size]
    /// output: [batch, num_markers]
    pub fn forward(&self, marker_repr: &Tensor) -> Result<Tensor> {
        let hidden = self.layer_norm.forward(marker_repr)?;
        let hidden = self.dense1.forward(&hidden)?;
        let hidden = hidden.gelu()?;
        let logits = self.dense2.forward(&hidden)?;
        let logits = logits.squeeze(2)?;
        Ok(logits)
    }
}
