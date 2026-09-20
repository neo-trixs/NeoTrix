//! Laya model composition

pub mod encoder;
pub mod type_emb;
pub mod scorer;
pub mod act_head;
pub mod decision_head;

use candle_core::{Tensor, Device};
use candle_nn::VarBuilder;
use serde::{Deserialize, Serialize};
use crate::error::Result;

use self::encoder::{ModernBERTEncoder, BertConfig};
use self::type_emb::TypeEmbedding;
use self::scorer::ScorerHead;
use self::act_head::ActHead;
use self::decision_head::DecisionHead;

/// Complete Laya model
pub struct LayaModel {
    encoder: ModernBERTEncoder,
    type_emb: TypeEmbedding,
    decision_head: DecisionHead,
    scorer: ScorerHead,
    act_head: ActHead,
}

/// Model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub hidden_size: usize,
    pub num_types: usize,
    pub head_layers: usize,
    pub encoder_hidden_size: usize,
    pub encoder_num_layers: usize,
    pub encoder_num_heads: usize,
    pub encoder_intermediate_size: usize,
    pub encoder_max_position_embeddings: usize,
    pub encoder_vocab_size: usize,
    /// Weight dtype for loading: "f32" (default) or "f16" (faster on Metal)
    #[serde(default = "default_dtype")]
    pub weight_dtype: String,
}

fn default_dtype() -> String {
    "f32".to_string()
}

impl ModelConfig {
    /// Default config for ModernBERT-large
    pub fn modernbert_large() -> Self {
        Self {
            hidden_size: 1024,
            num_types: 3,
            head_layers: 2,
            encoder_hidden_size: 1024,
            encoder_num_layers: 28,
            encoder_num_heads: 16,
            encoder_intermediate_size: 2624,
            encoder_max_position_embeddings: 8192,
            encoder_vocab_size: 50368,
            weight_dtype: "f32".to_string(),
        }
    }

    /// Load from a HuggingFace-style config.json in the model directory
    pub fn from_config_json(model_dir: &std::path::Path) -> Result<Self> {
        let config_path = model_dir.join("config.json");
        let data = std::fs::read_to_string(&config_path)
            .map_err(|e| crate::error::Error::WeightLoadError(format!(
                "Failed to read {}: {}", config_path.display(), e
            )))?;
        let raw: serde_json::Value = serde_json::from_str(&data)
            .map_err(|e| crate::error::Error::WeightLoadError(format!(
                "Failed to parse config.json: {}", e
            )))?;

        let hidden_size = raw["hidden_size"].as_u64().unwrap_or(1024) as usize;
        let num_layers = raw["num_hidden_layers"].as_u64().unwrap_or(28) as usize;
        let num_heads = raw["num_attention_heads"].as_u64().unwrap_or(16) as usize;
        let intermediate = raw["intermediate_size"].as_u64().unwrap_or(2624) as usize;
        let max_pos = raw["max_position_embeddings"].as_u64().unwrap_or(8192) as usize;
        let vocab = raw["vocab_size"].as_u64().unwrap_or(50368) as usize;

        Ok(Self {
            hidden_size,
            num_types: 3,
            head_layers: 2,
            encoder_hidden_size: hidden_size,
            encoder_num_layers: num_layers,
            encoder_num_heads: num_heads,
            encoder_intermediate_size: intermediate,
            encoder_max_position_embeddings: max_pos,
            encoder_vocab_size: vocab,
            weight_dtype: raw["weight_dtype"].as_str().unwrap_or("f32").to_string(),
        })
    }

    /// Build the encoder BertConfig from this model config
    fn encoder_config(&self) -> BertConfig {
        BertConfig {
            hidden_size: self.encoder_hidden_size,
            num_attention_heads: self.encoder_num_heads,
            num_hidden_layers: self.encoder_num_layers,
            intermediate_size: self.encoder_intermediate_size,
            max_position_embeddings: self.encoder_max_position_embeddings,
            vocab_size: self.encoder_vocab_size,
        }
    }

    /// Try to create a device based on preference string
    ///
    /// Supported values: `"auto"`, `"metal"`, `"gpu"`, `"cpu"`
    pub fn resolve_device(preference: &str) -> candle_core::Device {
        match preference {
            "metal" | "gpu" | "auto" => {
                #[cfg(target_os = "macos")]
                {
                    match candle_core::Device::new_metal(0) {
                        Ok(dev) => {
                            tracing::info!("Using Metal device");
                            dev
                        }
                        Err(e) => {
                            tracing::warn!("Metal unavailable ({}), falling back to CPU", e);
                            candle_core::Device::Cpu
                        }
                    }
                }
                #[cfg(not(target_os = "macos"))]
                {
                    tracing::info!("Metal only available on macOS, using CPU");
                    candle_core::Device::Cpu
                }
            }
            _ => {
                tracing::info!("Using CPU device");
                candle_core::Device::Cpu
            }
        }
    }
}

impl LayaModel {
    /// Load from weights
    pub fn from_weights(
        weights: &VarBuilder,
        config: &ModelConfig,
        device: &Device,
    ) -> Result<Self> {
        let encoder = ModernBERTEncoder::from_weights(weights, &config.encoder_config(), device)?;
        let type_emb = TypeEmbedding::new_random(config.hidden_size, config.num_types, device)?;
        let decision_head = DecisionHead::new(config.hidden_size, config.head_layers, device)?;
        let scorer = ScorerHead::new(config.hidden_size, device)?;
        let act_head = ActHead::new(config.hidden_size, 2, device)?;

        Ok(Self {
            encoder,
            type_emb,
            decision_head,
            scorer,
            act_head,
        })
    }

    /// Forward pass — matches upstream Python exactly:
    /// ```python
    /// h = self.encoder(input_ids, attention_mask).last_hidden_state
    /// h = h + self.type_emb(qtype)[:, None, :]
    /// h = self.head(h, src_key_padding_mask=pad)
    /// m = h.gather(1, marker_pos.clamp(0, h.shape[1]-1).unsqueeze(-1).expand(-1,-1,h.shape[-1]))
    /// logits = self.scorer(m).squeeze(-1)
    /// logits = logits.masked_fill(~marker_mask, -1e4)
    /// feats = torch.stack([x, y1, y2, self.k / 255], dim=-1)
    /// act_logits = self.act_head(cat([h[:, 0], feats]))
    /// ```
    pub fn forward(
        &self,
        input_ids: &Tensor,
        attention_mask: &Tensor,
        marker_pos: &Tensor,
        marker_mask: &Tensor,
        qtype: &Tensor,
    ) -> Result<(Tensor, Tensor)> {
        // 1. Encoder
        let hidden = self.encoder.forward(input_ids, attention_mask)?;

        // 2. Type embedding broadcast
        let type_emb = self.type_emb.forward(qtype)?.unsqueeze(1)?;
        let hidden = hidden.broadcast_add(&type_emb)?;

        // 3. Decision head (2-layer TransformerEncoder)
        let hidden = self.decision_head.forward(&hidden, attention_mask)?;

        // 4. Extract marker representations: gather along seq dim
        let marker_repr = self.extract_markers(&hidden, marker_pos)?;

        // 5. Scorer: [batch, num_markers, hidden] -> [batch, num_markers]
        let logits = self.scorer.forward(&marker_repr)?;

        // 6. Mask invalid markers: logits.masked_fill(~marker_mask, -1e4)
        let logits = self.mask_logits(&logits, marker_mask)?;

        // 7. Act head: [CLS + top1 + gap + entropy + k/255]
        let act_logits = self.compute_act_head(&hidden, &logits, marker_mask)?;

        Ok((logits, act_logits))
    }

    /// Mask logits: valid positions keep value, invalid get -1e4
    fn mask_logits(&self, logits: &Tensor, marker_mask: &Tensor) -> Result<Tensor> {
        let neg_inf = Tensor::new(-1e4f32, logits.device())?;
        let mask_f32 = marker_mask.to_dtype(candle_core::DType::F32)?;
        let one_minus_mask = Tensor::new(1.0f32, logits.device())?.broadcast_sub(&mask_f32)?;
        let masked = logits.broadcast_mul(&mask_f32)?
            + neg_inf.broadcast_mul(&one_minus_mask)?;
        Ok(masked?)
    }

    /// Compute act head features matching upstream:
    /// Input: cat([CLS_token (d), top1_prob, prob_gap, entropy, k/255])
    fn compute_act_head(
        &self,
        hidden: &Tensor,   // [batch, seq, hidden]
        logits: &Tensor,   // [batch, num_markers]
        marker_mask: &Tensor,
    ) -> Result<Tensor> {
        let device = logits.device();

        // CLS token: hidden[:, 0]
        let cls = hidden.get_on_dim(1, 0)?; // [batch, hidden]

        // Probabilities from logits
        let probs = candle_nn::ops::softmax(logits, candle_core::D::Minus1)?;

        // top1 probability (max along markers dim)
        let top1 = probs.max(candle_core::D::Minus1)?; // [batch]

        // top2: get 2nd-best via topk
        use candle_transformers::models::deepseek2::TopKLastDimOp;
        let top2_vals = probs.topk(2)?;
        let top2_values = top2_vals.values; // [batch, 2]
        let second_best = top2_values.get_on_dim(1, 1)?;
        // gap = best - 2nd_best
        let gap = top1.sub(&second_best)?;

        // Entropy: -sum(p * log(p))
        let log_probs = probs.clamp(1e-12f32, 1.0f32)?.log()?;
        let entropy = probs.mul(&log_probs)?.sum(candle_core::D::Minus1)?.neg()?; // [batch]

        // k = number of valid markers per sample
        let k = marker_mask.to_dtype(candle_core::DType::F32)?
            .sum(candle_core::D::Minus1)?; // [batch]
        let k_norm = k.broadcast_div(&Tensor::new(255.0f32, device)?)?; // [batch]

        // Stack features: [batch, 4]
        let feats = Tensor::cat(&[
            &top1.unsqueeze(1)?,
            &gap.unsqueeze(1)?,
            &entropy.unsqueeze(1)?,
            &k_norm.unsqueeze(1)?,
        ], 1)?;

        // Concatenate CLS + features: [batch, hidden + 4]
        let input = Tensor::cat(&[&cls, &feats], 1)?;

        self.act_head.forward(&input)
    }

    /// Extract marker representations via gather (device-native, no CPU roundtrip)
    ///
    /// Matches upstream: `h.gather(1, marker_pos.clamp(0, h.shape[1]-1).unsqueeze(-1).expand(-1,-1,h.shape[-1]))`
    fn extract_markers(
        &self,
        hidden: &Tensor,
        marker_pos: &Tensor,
    ) -> Result<Tensor> {
        let seq_len = hidden.dim(1)?;
        let hidden_size = hidden.dim(2)?;
        let batch_size = hidden.dim(0)?;
        let num_markers = marker_pos.dim(1)?;

        // Clamp marker positions to valid range [0, seq_len-1]
        let seq_len_m1 = (seq_len - 1) as u32;
        let marker_pos_clamped = marker_pos.clamp(0u32, seq_len_m1)?;

        // Expand: [batch, num_markers] -> [batch, num_markers, 1] -> [batch, num_markers, hidden_size]
        let idx = marker_pos_clamped.unsqueeze(2)?
            .expand(&[batch_size, num_markers, hidden_size])?
            .contiguous()?;

        // Gather along seq dim — stays on device (Metal/CPU), no data copy
        Ok(hidden.gather(&idx, 1)?)
    }
}
