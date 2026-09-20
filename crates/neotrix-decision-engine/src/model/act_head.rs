//! Action prediction head
//!
//! Matches upstream: nn.Sequential(nn.Linear(d + 4, 256), nn.GELU(), nn.Linear(256, n_act))
//! Input: [CLS token (d)] + [top1_prob, prob_gap, entropy_norm, k/255] = d + 4 features

use candle_core::{Tensor, Device, DType};
use candle_nn::{Linear, VarBuilder, VarMap, Module};
use crate::error::Result;

/// Action head: Linear(d+4, 256) -> GELU -> Linear(256, n_act)
pub struct ActHead {
    dense1: Linear,
    dense2: Linear,
}

impl ActHead {
    /// Initialize (task-specific head, not in pretrained weights)
    pub fn new(hidden_size: usize, n_act: usize, device: &Device) -> Result<Self> {
        let varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, device);

        let input_dim = hidden_size + 4; // CLS + 4 features

        let w1 = vb.get((256, input_dim), "dense.weight")?;
        let b1 = vb.get(256, "dense.bias")?;
        let dense1 = Linear::new(w1, Some(b1));

        let w2 = vb.get((n_act, 256), "dense2.weight")?;
        let b2 = vb.get(n_act, "dense2.bias")?;
        let dense2 = Linear::new(w2, Some(b2));

        Ok(Self { dense1, dense2 })
    }

    /// Forward pass
    /// input: [batch, hidden_size + 4] (CLS + features concatenated)
    /// output: [batch, n_act]
    pub fn forward(&self, input: &Tensor) -> Result<Tensor> {
        let hidden = self.dense1.forward(input)?;
        let hidden = hidden.gelu()?;
        let logits = self.dense2.forward(&hidden)?;
        Ok(logits)
    }
}
