# Laya 完整 Rust 重写计划

> 将 Laya (Python/PyTorch) 完整重构为纯 Rust (candle)
> 日期: 2026-09-20
> 目标: 零 Python 依赖，本地推理，嵌入 NeoTrix

---

## 一、架构总览

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        Laya Rust Architecture                           │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐              │
│  │  Tokenizer   │    │   Router     │    │   Config     │              │
│  │  (tokenizers │    │  (language   │    │  (model      │              │
│  │   crate)     │    │   detection) │    │   settings)  │              │
│  └──────┬───────┘    └──────┬───────┘    └──────┬───────┘              │
│         │                   │                   │                       │
│         └───────────────────┼───────────────────┘                       │
│                             │                                           │
│                             ▼                                           │
│  ┌──────────────────────────────────────────────────────────────┐      │
│  │                    Sequence Builder                           │      │
│  │  state + questions → [CLS] state [SEP] opts [SEP]           │      │
│  │  + marker_positions + question_types                          │      │
│  └──────────────────────────┬───────────────────────────────────┘      │
│                             │                                           │
│                             ▼                                           │
│  ┌──────────────────────────────────────────────────────────────┐      │
│  │                    Model Forward                              │      │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │      │
│  │  │  ModernBERT  │  │  Type Emb   │  │  Scorer     │         │      │
│  │  │  Encoder     │→ │  (3 types)  │→ │  Head       │         │      │
│  │  │  (421M)      │  │             │  │             │         │      │
│  │  └─────────────┘  └─────────────┘  └─────────────┘         │      │
│  └──────────────────────────┬───────────────────────────────────┘      │
│                             │                                           │
│                             ▼                                           │
│  ┌──────────────────────────────────────────────────────────────┐      │
│  │                    Post-Processing                            │      │
│  │  Temperature Scaling → Softmax → Confidence → Answer         │      │
│  └──────────────────────────┬───────────────────────────────────┘      │
│                             │                                           │
│                             ▼                                           │
│  ┌──────────────────────────────────────────────────────────────┐      │
│  │                    EvaluationResult                           │      │
│  │  { answers: { qid: Answer }, model, usage }                  │      │
│  └──────────────────────────────────────────────────────────────┘      │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 二、模块分解

### 模块列表

| 模块 | 文件 | 职责 | 依赖 |
|------|------|------|------|
| `types` | `types.rs` | Question, Answer, State 定义 | 无 |
| `error` | `error.rs` | Error 类型 | thiserror |
| `tokenizer` | `tokenizer.rs` | Tokenizer 封装 | tokenizers |
| `sequence` | `sequence.rs` | 序列构建 + markers | tokenizer, types |
| `encoder` | `model/encoder.rs` | ModernBERT encoder | candle-transformers |
| `type_emb` | `model/type_emb.rs` | 类型嵌入 | candle-nn |
| `scorer` | `model/scorer.rs` | 评分头 | candle-nn |
| `act_head` | `model/act_head.rs` | 动作头 | candle-nn |
| `model` | `model/mod.rs` | 模型组合 | 所有模型模块 |
| `router` | `router.rs` | 语言/脚本检测 | 无 |
| `temperature` | `temperature.rs` | 温度缩放 | 无 |
| `engine` | `engine.rs` | DecisionEngine 核心 | 所有模块 |
| `weights` | `weights.rs` | SafeTensors 加载 | safetensors |

---

## 三、详细实现

### 3.1 types.rs — 类型定义

```rust
//! Question and Answer type definitions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// State sent to the decision engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Question type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum QuestionType {
    Noul { instructions: String },
    Choice { instructions: String, criteria: HashMap<String, Option<String>> },
    Score { instructions: String, criteria: Vec<String> },
}

/// A question
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub id: String,
    #[serde(flatten)]
    pub question_type: QuestionType,
}

/// Answer types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Answer {
    Noul(NoulAnswer),
    Choice(ChoiceAnswer),
    Score(ScoreAnswer),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoulAnswer { pub noul: f64 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChoiceAnswer {
    pub choice: String,
    pub confidence: f64,
    pub probabilities: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreAnswer {
    pub score: f64,
    pub confidence: f64,
    pub legend: HashMap<String, String>,
    pub probabilities: HashMap<String, f64>,
}

/// Evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    pub answers: HashMap<String, Answer>,
    pub model: Option<String>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}
```

### 3.2 error.rs — 错误类型

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Tokenizer error: {0}")]
    Tokenizer(String),
    
    #[error("Model error: {0}")]
    Model(String),
    
    #[error("Weight loading error: {0}")]
    WeightLoad(String),
    
    #[error("Inference error: {0}")]
    Inference(String),
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
```

### 3.3 tokenizer.rs — Tokenizer 封装

```rust
use tokenizers::{Tokenizer, Encoding};
use crate::error::{Error, Result};

pub struct LayaTokenizer {
    inner: Tokenizer,
    cls_token_id: u32,
    sep_token_id: u32,
    pad_token_id: u32,
}

impl LayaTokenizer {
    /// 从文件加载
    pub fn from_file(path: &std::path::Path) -> Result<Self> {
        let tokenizer = Tokenizer::from_file(path)
            .map_err(|e| Error::Tokenizer(e.to_string()))?;
        
        let cls_token_id = tokenizer.token_to_id("[CLS]")
            .ok_or_else(|| Error::Tokenizer("Missing [CLS] token".into()))?;
        let sep_token_id = tokenizer.token_to_id("[SEP]")
            .ok_or_else(|| Error::Tokenizer("Missing [SEP] token".into()))?;
        let pad_token_id = tokenizer.token_to_id("[PAD]")
            .unwrap_or(0);
        
        Ok(Self {
            inner: tokenizer,
            cls_token_id,
            sep_token_id,
            pad_token_id,
        })
    }
    
    /// 从字节加载
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let tokenizer = Tokenizer::from_bytes(bytes)
            .map_err(|e| Error::Tokenizer(e.to_string()))?;
        
        // Same as from_file
        let cls_token_id = tokenizer.token_to_id("[CLS]")
            .ok_or_else(|| Error::Tokenizer("Missing [CLS] token".into()))?;
        let sep_token_id = tokenizer.token_to_id("[SEP]")
            .ok_or_else(|| Error::Tokenizer("Missing [SEP] token".into()))?;
        let pad_token_id = tokenizer.token_to_id("[PAD]")
            .unwrap_or(0);
        
        Ok(Self {
            inner: tokenizer,
            cls_token_id,
            sep_token_id,
            pad_token_id,
        })
    }
    
    /// Encode 文本
    pub fn encode(&self, text: &str) -> Result<Vec<u32>> {
        let encoding = self.inner.encode(text, true)
            .map_err(|e| Error::Tokenizer(e.to_string()))?;
        Ok(encoding.get_ids().to_vec())
    }
    
    /// Decode tokens
    pub fn decode(&self, tokens: &[u32]) -> Result<String> {
        self.inner.decode(tokens, false)
            .map_err(|e| Error::Tokenizer(e.to_string()))
    }
    
    /// 获取特殊 token IDs
    pub fn cls_token_id(&self) -> u32 { self.cls_token_id }
    pub fn sep_token_id(&self) -> u32 { self.sep_token_id }
    pub fn pad_token_id(&self) -> u32 { self.pad_token_id }
}
```

### 3.4 sequence.rs — 序列构建

```rust
use crate::types::*;
use crate::tokenizer::LayaTokenizer;
use crate::error::Result;

pub struct SequenceBuilder {
    tokenizer: LayaTokenizer,
    max_len: usize,
    head_max_len: usize,
}

/// 构建的序列
pub struct BuiltSequence {
    pub input_ids: Vec<u32>,
    pub attention_mask: Vec<u32>,
    pub marker_positions: Vec<usize>,
    pub question_type: QuestionType,
}

impl SequenceBuilder {
    pub fn new(tokenizer: LayaTokenizer, max_len: usize, head_max_len: usize) -> Self {
        Self {
            tokenizer,
            max_len,
            head_max_len,
        }
    }
    
    /// 构建带 markers 的序列
    pub fn build(&self, state: &State, question: &Question) -> Result<BuiltSequence> {
        // 1. Tokenize state
        let state_tokens = self.tokenizer.encode(&state.content)?;
        
        // 2. Tokenize options
        let (option_tokens, markers) = self.tokenize_options(question)?;
        
        // 3. 组合: [CLS] state [SEP] options [SEP]
        let mut input_ids = vec![self.tokenizer.cls_token_id()];
        input_ids.extend(&state_tokens);
        input_ids.push(self.tokenizer.sep_token_id());
        
        let marker_offset = input_ids.len();
        input_ids.extend(&option_tokens);
        input_ids.push(self.tokenizer.sep_token_id());
        
        // 4. 调整 marker 位置
        let marker_positions: Vec<usize> = markers
            .iter()
            .map(|m| m + marker_offset)
            .collect();
        
        // 5. 截断到 max_len
        if input_ids.len() > self.max_len {
            input_ids.truncate(self.max_len);
        }
        
        // 6. 创建 attention mask
        let attention_mask = vec![1; input_ids.len()];
        
        Ok(BuiltSequence {
            input_ids,
            attention_mask,
            marker_positions,
            question_type: question.question_type.clone(),
        })
    }
    
    /// Tokenize 选项
    fn tokenize_options(&self, question: &Question) -> Result<(Vec<u32>, Vec<usize>)> {
        let mut tokens = Vec::new();
        let mut markers = Vec::new();
        
        match &question.question_type {
            QuestionType::Choice { criteria, .. } => {
                for (opt, _desc) in criteria {
                    markers.push(tokens.len());
                    let opt_tokens = self.tokenizer.encode(opt)?;
                    tokens.extend(opt_tokens);
                    tokens.push(self.tokenizer.sep_token_id());
                }
            }
            QuestionType::Score { criteria, .. } => {
                for (i, level) in criteria.iter().enumerate() {
                    markers.push(tokens.len());
                    let level_tokens = self.tokenizer.encode(level)?;
                    tokens.extend(level_tokens);
                    if i < criteria.len() - 1 {
                        tokens.push(self.tokenizer.sep_token_id());
                    }
                }
            }
            QuestionType::Noul { .. } => {
                markers.push(tokens.len());
                tokens.extend(self.tokenizer.encode("true")?);
                tokens.push(self.tokenizer.sep_token_id());
                markers.push(tokens.len());
                tokens.extend(self.tokenizer.encode("false")?);
            }
        }
        
        Ok((tokens, markers))
    }
    
    /// 批量构建序列
    pub fn build_batch(
        &self,
        state: &State,
        questions: &[Question],
    ) -> Result<Vec<BuiltSequence>> {
        questions.iter()
            .map(|q| self.build(state, q))
            .collect()
    }
}
```

### 3.5 model/encoder.rs — ModernBERT Encoder

```rust
use candle_core::{Module, Tensor, Device, DType};
use candle_nn::{VarBuilder, VarMap};
use crate::error::Result;

/// ModernBERT Encoder wrapper
pub struct ModernBERTEncoder {
    // BERT layers
    layers: Vec<BertLayer>,
    // Layer norm
    layer_norm: LayerNorm,
    // Config
    config: BertConfig,
}

#[derive(Debug, Clone)]
pub struct BertConfig {
    pub hidden_size: usize,      // 1024 for large
    pub num_attention_heads: usize,  // 16
    pub num_hidden_layers: usize,    // 24
    pub intermediate_size: usize,    // 4096
    pub max_position_embeddings: usize,
}

impl ModernBERTEncoder {
    /// 从权重加载
    pub fn from_weights(
        weights: &VarBuilder,
        config: &BertConfig,
        device: &Device,
    ) -> Result<Self> {
        let mut layers = Vec::new();
        
        for i in 0..config.num_hidden_layers {
            let layer = BertLayer::load(weights, i, config, device)?;
            layers.push(layer);
        }
        
        let layer_norm = LayerNorm::load(weights, config, device)?;
        
        Ok(Self {
            layers,
            layer_norm,
            config: config.clone(),
        })
    }
    
    /// Forward pass
    pub fn forward(
        &self,
        input_ids: &Tensor,
        attention_mask: &Tensor,
    ) -> Result<Tensor> {
        let mut hidden = self.embeddings(input_ids)?;
        
        for layer in &self.layers {
            hidden = layer.forward(&hidden, attention_mask)?;
        }
        
        let hidden = self.layer_norm.forward(&hidden)?;
        
        Ok(hidden)
    }
    
    /// Embeddings (token + position + type)
    fn embeddings(&self, input_ids: &Tensor) -> Result<Tensor> {
        // TODO: 实现 embedding 层
        // 包括: token_embedding, position_embedding, layer_norm, dropout
        todo!()
    }
}

/// BERT Layer
struct BertLayer {
    attention: MultiHeadAttention,
    intermediate: Intermediate,
    output: LayerOutput,
}

impl BertLayer {
    fn load(
        weights: &VarBuilder,
        layer_idx: usize,
        config: &BertConfig,
        device: &Device,
    ) -> Result<Self> {
        let prefix = format!("encoder.layer.{}", layer_idx);
        
        let attention = MultiHeadAttention::load(weights, &prefix, config, device)?;
        let intermediate = Intermediate::load(weights, &prefix, config, device)?;
        let output = LayerOutput::load(weights, &prefix, config, device)?;
        
        Ok(Self {
            attention,
            intermediate,
            output,
        })
    }
    
    fn forward(&self, hidden: &Tensor, attention_mask: &Tensor) -> Result<Tensor> {
        let attn_output = self.attention.forward(hidden, attention_mask)?;
        let intermediate_output = self.intermediate.forward(&attn_output)?;
        let output = self.output.forward(&attn_output, &intermediate_output)?;
        Ok(output)
    }
}

/// Multi-Head Attention
struct MultiHeadAttention {
    query: Linear,
    key: Linear,
    value: Linear,
    output: Linear,
    num_heads: usize,
    head_dim: usize,
}

impl MultiHeadAttention {
    fn load(
        weights: &VarBuilder,
        prefix: &str,
        config: &BertConfig,
        device: &Device,
    ) -> Result<Self> {
        let head_dim = config.hidden_size / config.num_attention_heads;
        
        let query = Linear::load(weights, &format!("{}.attention.self.query", prefix), device)?;
        let key = Linear::load(weights, &format!("{}.attention.self.key", prefix), device)?;
        let value = Linear::load(weights, &format!("{}.attention.self.value", prefix), device)?;
        let output = Linear::load(weights, &format!("{}.attention.output.dense", prefix), device)?;
        
        Ok(Self {
            query,
            key,
            value,
            output,
            num_heads: config.num_attention_heads,
            head_dim,
        })
    }
    
    fn forward(&self, hidden: &Tensor, attention_mask: &Tensor) -> Result<Tensor> {
        // TODO: 实现 multi-head attention
        todo!()
    }
}

/// Intermediate layer
struct Intermediate {
    dense: Linear,
}

impl Intermediate {
    fn load(weights: &VarBuilder, prefix: &str, device: &Device) -> Result<Self> {
        let dense = Linear::load(weights, &format!("{}.intermediate.dense", prefix), device)?;
        Ok(Self { dense })
    }
    
    fn forward(&self, hidden: &Tensor) -> Result<Tensor> {
        // GELU activation
        let hidden = self.dense.forward(hidden)?;
        let hidden = hidden.gelu()?;
        Ok(hidden)
    }
}

/// Layer output
struct LayerOutput {
    dense: Linear,
    layer_norm: LayerNorm,
}

impl LayerOutput {
    fn load(weights: &VarBuilder, prefix: &str, device: &Device) -> Result<Self> {
        let dense = Linear::load(weights, &format!("{}.output.dense", prefix), device)?;
        let layer_norm = LayerNorm::load(weights, &format!("{}.output.layer_norm", prefix), device)?;
        Ok(Self { dense, layer_norm })
    }
    
    fn forward(&self, residual: &Tensor, hidden: &Tensor) -> Result<Tensor> {
        let hidden = self.dense.forward(hidden)?;
        let hidden = hidden + residual;
        let hidden = self.layer_norm.forward(&hidden)?;
        Ok(hidden)
    }
}

/// Linear layer wrapper
struct Linear {
    weight: Tensor,
    bias: Option<Tensor>,
}

impl Linear {
    fn load(weights: &VarBuilder, name: &str, device: &Device) -> Result<Self> {
        let weight = weights.get(name, device)?;
        let bias = weights.get(&format!("{}.bias", name), device).ok();
        Ok(Self { weight, bias })
    }
    
    fn forward(&self, input: &Tensor) -> Result<Tensor> {
        let output = input.matmul(&self.weight.t()?)?;
        if let Some(bias) = &self.bias {
            let output = output + bias;
            Ok(output)
        } else {
            Ok(output)
        }
    }
}

/// LayerNorm wrapper
struct LayerNorm {
    weight: Tensor,
    bias: Tensor,
    eps: f64,
}

impl LayerNorm {
    fn load(weights: &VarBuilder, name: &str, device: &Device) -> Result<Self> {
        let weight = weights.get(name, device)?;
        let bias = weights.get(&format!("{}.bias", name), device)?;
        Ok(Self {
            weight,
            bias,
            eps: 1e-12,
        })
    }
    
    fn forward(&self, input: &Tensor) -> Result<Tensor> {
        // TODO: 实现 layer norm
        todo!()
    }
}

/// Embedding layer
struct Embeddings {
    token_embedding: Tensor,
    position_embedding: Tensor,
    layer_norm: LayerNorm,
}

impl Embeddings {
    fn load(weights: &VarBuilder, config: &BertConfig, device: &Device) -> Result<Self> {
        let token_embedding = weights.get("embeddings.word_embeddings.weight", device)?;
        let position_embedding = weights.get("embeddings.position_embeddings.weight", device)?;
        let layer_norm = LayerNorm::load(weights, "embeddings.LayerNorm", device)?;
        
        Ok(Self {
            token_embedding,
            position_embedding,
            layer_norm,
        })
    }
    
    fn forward(&self, input_ids: &Tensor) -> Result<Tensor> {
        let seq_len = input_ids.dim(1)?;
        
        let token_emb = input_ids.unsqueeze(2)?
            .embedding(&self.token_embedding)?;
        
        let position_ids = Tensor::arange(0u32, seq_len as u32, &self.token_embedding.device())?;
        let position_emb = position_ids.unsqueeze(0)?
            .unsqueeze(2)?
            .embedding(&self.position_embedding)?;
        
        let hidden = token_emb + position_emb;
        let hidden = self.layer_norm.forward(&hidden)?;
        
        Ok(hidden)
    }
}
```

### 3.6 model/type_emb.rs — 类型嵌入

```rust
use candle_core::{Tensor, Device};
use candle_nn::{Embedding, VarBuilder};
use crate::error::Result;

/// Question type embedding
pub struct TypeEmbedding {
    embedding: Embedding,
}

/// Question types: Noul=0, Choice=1, Score=2
impl TypeEmbedding {
    pub fn load(weights: &VarBuilder, hidden_size: usize, device: &Device) -> Result<Self> {
        let embedding = Embedding::load(weights, "type_emb.weight", device)?;
        Ok(Self { embedding })
    }
    
    pub fn forward(&self, qtype: &Tensor) -> Result<Tensor> {
        // qtype: [batch] (0=Noul, 1=Choice, 2=Score)
        // output: [batch, hidden_size]
        self.embedding.forward(qtype)
    }
}
```

### 3.7 model/scorer.rs — 评分头

```rust
use candle_core::{Tensor, Module};
use candle_nn::{Linear, VarBuilder};
use crate::error::Result;

/// Scorer head for question answers
pub struct ScorerHead {
    dense1: Linear,
    dense2: Linear,
}

impl ScorerHead {
    pub fn load(weights: &VarBuilder, hidden_size: usize, device: &Device) -> Result<Self> {
        let dense1 = Linear::load(weights, "scorer.dense1", device)?;
        let dense2 = Linear::load(weights, "scorer.dense2", device)?;
        
        Ok(Self { dense1, dense2 })
    }
    
    /// Forward pass
    /// input: [batch, num_markers, hidden_size]
    /// output: [batch, num_options]
    pub fn forward(&self, marker_repr: &Tensor) -> Result<Tensor> {
        let hidden = self.dense1.forward(marker_repr)?;
        let hidden = hidden.relu()?;
        let logits = self.dense2.forward(&hidden)?;
        
        // 取每个 marker 的第一个输出作为 logits
        let logits = logits.squeeze(2)?;
        
        Ok(logits)
    }
}
```

### 3.8 model/act_head.rs — 动作头

```rust
use candle_core::{Tensor, Module};
use candle_nn::{Linear, VarBuilder};
use crate::error::Result;

/// Action prediction head
pub struct ActHead {
    dense: Linear,
}

impl ActHead {
    pub fn load(weights: &VarBuilder, hidden_size: usize, device: &Device) -> Result<Self> {
        let dense = Linear::load(weights, "act_head.dense", device)?;
        Ok(Self { dense })
    }
    
    /// Forward pass
    /// input: [batch, seq_len, hidden_size]
    /// output: [batch, 1] (probability of taking action)
    pub fn forward(&self, hidden: &Tensor) -> Result<Tensor> {
        // 取 [CLS] 位置的输出
        let cls_hidden = hidden.get(0, 0)?;
        let logits = self.dense.forward(&cls_hidden.unsqueeze(0)?)?;
        let prob = logits.sigmoid()?;
        Ok(prob)
    }
}
```

### 3.9 model/mod.rs — 模型组合

```rust
use candle_core::{Tensor, Device};
use candle_nn::VarBuilder;
use crate::error::Result;

use super::encoder::ModernBERTEncoder;
use super::type_emb::TypeEmbedding;
use super::scorer::ScorerHead;
use super::act_head::ActHead;

/// Complete Laya model
pub struct LayaModel {
    encoder: ModernBERTEncoder,
    type_emb: TypeEmbedding,
    scorer: ScorerHead,
    act_head: ActHead,
}

impl LayaModel {
    /// 从权重加载
    pub fn from_weights(
        weights: &VarBuilder,
        config: &ModelConfig,
        device: &Device,
    ) -> Result<Self> {
        let encoder = ModernBERTEncoder::load(weights, &config.encoder_config, device)?;
        let type_emb = TypeEmbedding::load(weights, config.hidden_size, device)?;
        let scorer = ScorerHead::load(weights, config.hidden_size, device)?;
        let act_head = ActHead::load(weights, config.hidden_size, device)?;
        
        Ok(Self {
            encoder,
            type_emb,
            scorer,
            act_head,
        })
    }
    
    /// Forward pass
    pub fn forward(
        &self,
        input_ids: &Tensor,
        attention_mask: &Tensor,
        marker_pos: &Tensor,
        qtype: &Tensor,
    ) -> Result<(Tensor, Tensor)> {
        // 1. Encoder forward
        let hidden = self.encoder.forward(input_ids, attention_mask)?;
        
        // 2. Add type embeddings
        let type_emb = self.type_emb.forward(qtype)?;
        let hidden = hidden + type_emb.unsqueeze(1)?;
        
        // 3. Extract marker representations
        let marker_repr = self.extract_markers(&hidden, marker_pos)?;
        
        // 4. Score
        let logits = self.scorer.forward(&marker_repr)?;
        
        // 5. Action prediction
        let act_logits = self.act_head.forward(&hidden)?;
        
        Ok((logits, act_logits))
    }
    
    /// Extract marker representations
    fn extract_markers(
        &self,
        hidden: &Tensor,
        marker_pos: &Tensor,
    ) -> Result<Tensor> {
        let batch_size = hidden.dim(0)?;
        let num_markers = marker_pos.dim(1)?;
        let hidden_dim = hidden.dim(2)?;
        
        let mut marker_repr = Vec::new();
        
        for b in 0..batch_size {
            let mut batch_markers = Vec::new();
            for m in 0..num_markers {
                let pos = marker_pos.get(b, m)?.to_scalar::<usize>()?;
                let repr = hidden.get(b, pos)?;
                batch_markers.push(repr);
            }
            marker_repr.push(Tensor::stack(&batch_markers, 0)?);
        }
        
        Ok(Tensor::stack(&marker_repr, 0)?)
    }
}

/// Model configuration
pub struct ModelConfig {
    pub hidden_size: usize,
    pub encoder_config: BertConfig,
}
```

### 3.10 weights.rs — 权重加载

```rust
use std::path::Path;
use safetensors::SafeTensors;
use candle_core::{Device, Tensor};
use crate::error::{Error, Result};

pub struct WeightLoader;

impl WeightLoader {
    /// 从 SafeTensors 文件加载权重
    pub fn from_file(path: &Path, device: &Device) -> Result<Vec<(String, Tensor)>> {
        let data = std::fs::read(path)?;
        let tensors = SafeTensors::deserialize(&data)
            .map_err(|e| Error::WeightLoad(e.to_string()))?;
        
        let mut weights = Vec::new();
        
        for (name, tensor_view) in tensors.iter() {
            let tensor = Self::load_tensor(tensor_view, device)?;
            weights.push((name.to_string(), tensor));
        }
        
        Ok(weights)
    }
    
    /// 加载单个 tensor
    fn load_tensor(
        view: &safetensors::TensorView,
        device: &Device,
    ) -> Result<Tensor> {
        let dtype = match view.dtype() {
            safetensors::Dtype::F16 => candle_core::DType::F16,
            safetensors::Dtype::F32 => candle_core::DType::F32,
            safetensors::Dtype::BF16 => candle_core::DType::BF16,
            _ => return Err(Error::WeightLoad(format!("Unsupported dtype: {:?}", view.dtype()))),
        };
        
        let shape = view.shape().to_vec();
        let data = view.data();
        
        Tensor::from_bytes(data, &shape, dtype, device)
            .map_err(|e| Error::WeightLoad(e.to_string()))
    }
    
    /// 按前缀加载权重
    pub fn load_with_prefix(
        path: &Path,
        prefix: &str,
        device: &Device,
    ) -> Result<Vec<(String, Tensor)>> {
        let all_weights = Self::from_file(path, device)?;
        
        Ok(all_weights.into_iter()
            .filter(|(name, _)| name.starts_with(prefix))
            .map(|(name, tensor)| {
                let new_name = name.strip_prefix(prefix)
                    .unwrap_or(&name)
                    .trim_start_matches('.')
                    .to_string();
                (new_name, tensor)
            })
            .collect())
    }
}
```

### 3.11 temperature.rs — 温度缩放

```rust
use crate::types::QuestionType;
use std::collections::HashMap;

pub struct TemperatureScaler {
    default: [f64; 3],  // [noul, choice, score]
    by_options: HashMap<(usize, usize), f64>,
}

impl TemperatureScaler {
    pub fn new(default: [f64; 3], by_options: HashMap<(usize, usize), f64>) -> Self {
        Self {
            default,
            by_options,
        }
    }
    
    pub fn scale(&self, logits: &mut [f64], qtype: QuestionType, num_options: usize) {
        let temp = self.get_temperature(qtype, num_options);
        for logit in logits.iter_mut() {
            *logit /= temp;
        }
    }
    
    fn get_temperature(&self, qtype: QuestionType, num_options: usize) -> f64 {
        let qtype_idx = match qtype {
            QuestionType::Noul { .. } => 0,
            QuestionType::Choice { .. } => 1,
            QuestionType::Score { .. } => 2,
        };
        
        let bucket = (qtype_idx, num_options);
        self.by_options.get(&bucket)
            .copied()
            .unwrap_or(self.default[qtype_idx])
    }
}

impl Default for TemperatureScaler {
    fn default() -> Self {
        Self {
            default: [1.0, 1.0, 1.0],
            by_options: HashMap::new(),
        }
    }
}
```

### 3.12 router.rs — 语言路由

```rust
use crate::types::*;
use crate::error::Result;

pub struct Router {
    english: Box<dyn DecisionBackend>,
    multilingual: Box<dyn DecisionBackend>,
    typed: Box<dyn DecisionBackend>,
}

pub trait DecisionBackend {
    fn predict(&self, state: &State, questions: &[Question]) -> Result<EvaluationResult>;
}

impl Router {
    pub fn new(
        english: Box<dyn DecisionBackend>,
        multilingual: Box<dyn DecisionBackend>,
        typed: Box<dyn DecisionBackend>,
    ) -> Self {
        Self {
            english,
            multilingual,
            typed,
        }
    }
    
    pub fn predict(&self, state: &State, questions: &[Question]) -> Result<EvaluationResult> {
        let script = detect_script(&state.content);
        
        let backend = match script {
            Script::Latin => &self.english,
            Script::Devanagari | Script::Arabic | Script::CJK => &self.multilingual,
            Script::TypedDecisions => &self.typed,
        };
        
        backend.predict(state, questions)
    }
    
    pub fn route(&self, state: &State) -> RouteDecision {
        let script = detect_script(&state.content);
        
        match script {
            Script::Latin => RouteDecision {
                model: "english".to_string(),
                reason: "Latin script detected".to_string(),
            },
            Script::Devanagari => RouteDecision {
                model: "multilingual".to_string(),
                reason: "Devanagari script detected".to_string(),
            },
            Script::Arabic => RouteDecision {
                model: "multilingual".to_string(),
                reason: "Arabic script detected".to_string(),
            },
            Script::CJK => RouteDecision {
                model: "multilingual".to_string(),
                reason: "CJK script detected".to_string(),
            },
            Script::TypedDecisions => RouteDecision {
                model: "typed".to_string(),
                reason: "Typed decisions workflow".to_string(),
            },
            Script::Unknown => RouteDecision {
                model: "english".to_string(),
                reason: "Unknown script, defaulting to English".to_string(),
            },
        }
    }
}

pub struct RouteDecision {
    pub model: String,
    pub reason: String,
}

#[derive(Debug, Clone, Copy)]
pub enum Script {
    Latin,
    Devanagari,
    Arabic,
    CJK,
    TypedDecisions,
    Unknown,
}

pub fn detect_script(text: &str) -> Script {
    let chars: Vec<char> = text.chars().collect();
    let total = chars.len() as f64;
    
    if total == 0 {
        return Script::Unknown;
    }
    
    let latin = chars.iter().filter(|c| is_latin(**c)).count() as f64 / total;
    let devanagari = chars.iter().filter(|c| is_devanagari(**c)).count() as f64 / total;
    let arabic = chars.iter().filter(|c| is_arabic(**c)).count() as f64 / total;
    let cjk = chars.iter().filter(|c| is_cjk(**c)).count() as f64 / total;
    
    if latin > 0.5 {
        Script::Latin
    } else if devanagari > 0.3 {
        Script::Devanagari
    } else if arabic > 0.3 {
        Script::Arabic
    } else if cjk > 0.3 {
        Script::CJK
    } else {
        Script::Unknown
    }
}

fn is_latin(c: char) -> bool {
    (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z')
}

fn is_devanagari(c: char) -> bool {
    c >= '\u{0900}' && c <= '\u{097F}'
}

fn is_arabic(c: char) -> bool {
    c >= '\u{0600}' && c <= '\u{06FF}'
}

fn is_cjk(c: char) -> bool {
    (c >= '\u{4E00}' && c <= '\u{9FFF}') ||
    (c >= '\u{3400}' && c <= '\u{4DBF}') ||
    (c >= '\u{F900}' && c <= '\u{FAFF}')
}
```

### 3.13 engine.rs — DecisionEngine 核心

```rust
use crate::types::*;
use crate::error::Result;
use crate::sequence::SequenceBuilder;
use crate::model::LayaModel;
use crate::temperature::TemperatureScaler;

pub struct DecisionEngine {
    model: LayaModel,
    sequence_builder: SequenceBuilder,
    temperature: TemperatureScaler,
}

impl DecisionEngine {
    pub fn new(
        model: LayaModel,
        sequence_builder: SequenceBuilder,
        temperature: TemperatureScaler,
    ) -> Self {
        Self {
            model,
            sequence_builder,
            temperature,
        }
    }
    
    pub fn evaluate(&self, state: &State, questions: &[Question]) -> Result<EvaluationResult> {
        // 1. 构建序列
        let sequences = self.sequence_builder.build_batch(state, questions)?;
        
        // 2. 准备输入
        let (input_ids, attention_mask, marker_pos, qtype) = self.prepare_inputs(&sequences)?;
        
        // 3. 模型前向传播
        let (logits, act_logits) = self.model.forward(
            &input_ids,
            &attention_mask,
            &marker_pos,
            &qtype,
        )?;
        
        // 4. 后处理
        let answers = self.post_process(&logits, questions)?;
        
        // 5. 计算 token 使用量
        let usage = Usage {
            input_tokens: attention_mask.sum()?.to_scalar::<u32>()?,
            output_tokens: 0,
        };
        
        Ok(EvaluationResult {
            answers,
            model: Some("laya-rust".to_string()),
            usage: Some(usage),
        })
    }
    
    fn prepare_inputs(
        &self,
        sequences: &[BuiltSequence],
    ) -> Result<(Tensor, Tensor, Tensor, Tensor)> {
        // TODO: 实现 batch 准备
        todo!()
    }
    
    fn post_process(
        &self,
        logits: &Tensor,
        questions: &[Question],
    ) -> Result<HashMap<String, Answer>> {
        let mut answers = HashMap::new();
        
        for (i, question) in questions.iter().enumerate() {
            let answer = self.process_question(logits, i, question)?;
            answers.insert(question.id.clone(), answer);
        }
        
        Ok(answers)
    }
    
    fn process_question(
        &self,
        logits: &Tensor,
        idx: usize,
        question: &Question,
    ) -> Result<Answer> {
        let logits_row = logits.get(idx)?;
        let mut logits_vec: Vec<f64> = logits_row.to_vec1()?;
        
        let num_options = logits_vec.len();
        self.temperature.scale(&mut logits_vec, question.question_type.clone(), num_options);
        
        // Softmax
        let probs = softmax(&logits_vec);
        
        match &question.question_type {
            QuestionType::Noul { .. } => {
                Ok(Answer::Noul(NoulAnswer {
                    noul: probs[1],
                }))
            }
            QuestionType::Choice { criteria, .. } => {
                let keys: Vec<String> = criteria.keys().cloned().collect();
                let max_idx = probs.iter().enumerate()
                    .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                
                let probabilities: HashMap<String, f64> = keys.iter().zip(probs.iter())
                    .map(|(k, v)| (k.clone(), *v))
                    .collect();
                
                let confidence = confidence_from_probs(&probs);
                
                Ok(Answer::Choice(ChoiceAnswer {
                    choice: keys[max_idx].clone(),
                    confidence,
                    probabilities,
                }))
            }
            QuestionType::Score { criteria, .. } => {
                let exp_score: f64 = probs.iter().enumerate()
                    .map(|(i, p)| i as f64 * p)
                    .sum();
                
                let legend: HashMap<String, String> = criteria.iter().enumerate()
                    .map(|(i, c)| (i.to_string(), c.clone()))
                    .collect();
                
                let probabilities: HashMap<String, f64> = probs.iter().enumerate()
                    .map(|(i, p)| (i.to_string(), *p))
                    .collect();
                
                let confidence = confidence_from_probs(&probs);
                
                Ok(Answer::Score(ScoreAnswer {
                    score: exp_score,
                    confidence,
                    legend,
                    probabilities,
                }))
            }
        }
    }
}

/// Softmax 函数
fn softmax(logits: &[f64]) -> Vec<f64> {
    let max = logits.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let exps: Vec<f64> = logits.iter().map(|l| (l - max).exp()).collect();
    let sum: f64 = exps.iter().sum();
    exps.iter().map(|e| e / sum).collect()
}

/// 从概率分布计算置信度
fn confidence_from_probs(probs: &[f64]) -> f64 {
    let max_prob = probs.iter().cloned().fold(0.0, f64::max);
    let entropy: f64 = probs.iter()
        .filter(|&&p| p > 0.0)
        .map(|&p| -p * p.ln())
        .sum();
    let max_entropy = -(1.0 / probs.len() as f64).ln();
    let normalized_entropy = if max_entropy > 0.0 {
        entropy / max_entropy
    } else {
        0.0
    };
    max_prob * (1.0 - normalized_entropy)
}
```

---

## 四、实现路线图

### Phase 1: 基础框架 (1-2 周)

```
Week 1:
- [x] types.rs - 类型定义
- [x] error.rs - 错误类型
- [ ] tokenizer.rs - Tokenizer 封装
- [ ] sequence.rs - 序列构建

Week 2:
- [ ] weights.rs - 权重加载
- [ ] temperature.rs - 温度缩放
- [ ] router.rs - 语言路由
```

### Phase 2: 模型实现 (2-3 周)

```
Week 3-4:
- [ ] encoder.rs - ModernBERT Encoder
- [ ] type_emb.rs - 类型嵌入
- [ ] scorer.rs - 评分头
- [ ] act_head.rs - 动作头

Week 5:
- [ ] model/mod.rs - 模型组合
- [ ] engine.rs - DecisionEngine 核心
```

### Phase 3: 优化 (1-2 周)

```
Week 6-7:
- [ ] GPU 加速优化
- [ ] 批量推理优化
- [ ] 内存优化
- [ ] 嵌入模型文件
```

---

## 五、依赖清单

```toml
[dependencies]
candle-core = "0.8"
candle-nn = "0.8"
candle-transformers = "0.8"
tokenizers = "0.21"
safetensors = "0.4"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
```

---

## 六、测试策略

| 测试类型 | 覆盖范围 | 工具 |
|---------|---------|------|
| 单元测试 | 每个模块 | `#[test]` |
| 集成测试 | 模块间交互 | `tests/` |
| 基准测试 | 性能 | `criterion` |
| 精度测试 | 与 Python Laya 对比 | 对比测试 |

---

## 七、成功标准

| 指标 | 目标 |
|------|------|
| 延迟 | < 50ms/question |
| 精度 | 与 Python Laya 相差 < 1% |
| 体积 | 模型 < 500MB |
| 依赖 | 零 Python 依赖 |
