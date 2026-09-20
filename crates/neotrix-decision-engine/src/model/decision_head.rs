//! Decision head — 2-layer TransformerEncoder between backbone and decision heads
//!
//! Matches upstream Python:
//! ```python
//! layer = nn.TransformerEncoderLayer(d, nhead, 4*d, dropout, batch_first=True, norm_first=True)
//! self.head = nn.TransformerEncoder(layer, head_layers, enable_nested_tensor=False)
//! ```

use candle_core::{Tensor, Device, DType};
use candle_nn::{VarMap, VarBuilder, Linear, LayerNorm, Module};
use crate::error::Result;

/// Single TransformerEncoderLayer: LayerNorm -> SelfAttention -> residual -> LayerNorm -> FFN -> residual
struct TransformerEncoderLayer {
    self_attn_norm: LayerNorm,
    self_attn_qkv: Linear,
    self_attn_out: Linear,
    ffn_norm: LayerNorm,
    ffn_up: Linear,
    ffn_down: Linear,
    head_dim: usize,
    num_heads: usize,
}

impl TransformerEncoderLayer {
    fn new(hidden_size: usize, num_heads: usize, device: &Device) -> Result<Self> {
        let head_dim = hidden_size / num_heads;
        let varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, device);

        let self_attn_norm = LayerNorm::new(
            vb.get((hidden_size,), "self_attn_norm.weight")?,
            vb.get((hidden_size,), "self_attn_norm.bias")?,
            1e-5,
        );

        let self_attn_qkv = Linear::new(
            vb.get((hidden_size * 3, hidden_size), "self_attn_qkv.weight")?,
            Some(vb.get((hidden_size * 3,), "self_attn_qkv.bias")?),
        );

        let self_attn_out = Linear::new(
            vb.get((hidden_size, hidden_size), "self_attn_out.weight")?,
            Some(vb.get((hidden_size,), "self_attn_out.bias")?),
        );

        let ffn_norm = LayerNorm::new(
            vb.get((hidden_size,), "ffn_norm.weight")?,
            vb.get((hidden_size,), "ffn_norm.bias")?,
            1e-5,
        );

        let ffn_up = Linear::new(
            vb.get((hidden_size * 4, hidden_size), "ffn_up.weight")?,
            Some(vb.get((hidden_size * 4,), "ffn_up.bias")?),
        );

        let ffn_down = Linear::new(
            vb.get((hidden_size, hidden_size * 4), "ffn_down.weight")?,
            Some(vb.get((hidden_size,), "ffn_down.bias")?),
        );

        Ok(Self {
            self_attn_norm,
            self_attn_qkv,
            self_attn_out,
            ffn_norm,
            ffn_up,
            ffn_down,
            head_dim,
            num_heads,
        })
    }

    fn forward(&self, x: &Tensor, _attention_mask: &Tensor) -> Result<Tensor> {
        // Self-attention with pre-norm (norm_first=True)
        let residual = x.clone();
        let x_norm = self.self_attn_norm.forward(x)?;

        // QKV projection
        let qkv = self.self_attn_qkv.forward(&x_norm)?;
        let batch_size = qkv.dim(0)?;
        let seq_len = qkv.dim(1)?;
        let total_dim = self.head_dim * self.num_heads;

        // Reshape QKV: [batch, seq, 3*hidden] -> [batch, seq, 3, num_heads, head_dim]
        let qkv = qkv.reshape((batch_size, seq_len, 3, self.num_heads, self.head_dim))?;
        let q = qkv.get_on_dim(2, 0)?; // [batch, seq, num_heads, head_dim]
        let k = qkv.get_on_dim(2, 1)?;
        let v = qkv.get_on_dim(2, 2)?;

        // Transpose for attention: [batch, num_heads, seq, head_dim]
        let q = q.permute((0, 2, 1, 3))?;
        let k = k.permute((0, 2, 1, 3))?;
        let v = v.permute((0, 2, 1, 3))?;

        // Attention scores
        let scale = (self.head_dim as f64).sqrt();
        let scores = (q.matmul(&k.transpose(candle_core::D::Minus2, candle_core::D::Minus1)?)? / scale)?;
        let attn = candle_nn::ops::softmax(&scores, candle_core::D::Minus1)?;
        let attn_out = attn.matmul(&v)?; // [batch, num_heads, seq, head_dim]

        // Transpose back and reshape: [batch, seq, hidden]
        let attn_out = attn_out.permute((0, 2, 1, 3))?;
        let attn_out = attn_out.reshape((batch_size, seq_len, total_dim))?;

        let attn_out = self.self_attn_out.forward(&attn_out)?;
        let x = (residual + attn_out)?;

        // FFN with pre-norm
        let residual = x.clone();
        let x_norm = self.ffn_norm.forward(&x)?;
        let ffn_out = self.ffn_up.forward(&x_norm)?;
        let ffn_out = ffn_out.gelu()?;
        let ffn_out = self.ffn_down.forward(&ffn_out)?;
        let x = (residual + ffn_out)?;

        Ok(x)
    }
}

/// Decision head: stack of TransformerEncoderLayers
pub struct DecisionHead {
    layers: Vec<TransformerEncoderLayer>,
}

impl DecisionHead {
    /// Create decision head with specified number of layers
    pub fn new(hidden_size: usize, num_layers: usize, device: &Device) -> Result<Self> {
        let num_heads = (hidden_size / 64).max(1);
        let mut layers = Vec::with_capacity(num_layers);
        for _ in 0..num_layers {
            layers.push(TransformerEncoderLayer::new(hidden_size, num_heads, device)?);
        }
        Ok(Self { layers })
    }

    /// Forward pass
    /// hidden: [batch, seq_len, hidden_size]
    /// attention_mask: [batch, seq_len] (1=valid, 0=padding)
    pub fn forward(&self, hidden: &Tensor, attention_mask: &Tensor) -> Result<Tensor> {
        let mut x = hidden.clone();
        for layer in &self.layers {
            x = layer.forward(&x, attention_mask)?;
        }
        Ok(x)
    }
}
