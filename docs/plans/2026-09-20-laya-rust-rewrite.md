# Laya Rust 重构计划

> 将 Laya (Python/PyTorch) 重构为 Rust (candle)
> 日期: 2026-09-20

---

## Laya 架构分析

### 核心组件

```
┌─────────────────────────────────────────────────────────────┐
│                    Laya Inference Pipeline                   │
├─────────────────────────────────────────────────────────────┤
│ 1. Tokenizer (HuggingFace tokenizers)                       │
│    └── Encode state + questions → input_ids                 │
├─────────────────────────────────────────────────────────────┤
│ 2. Sequence Builder                                          │
│    └── build_sequence(): 插入 markers 标记问题位置           │
├─────────────────────────────────────────────────────────────┤
│ 3. Encoder (ModernBERT-large, 421M params)                  │
│    └── 单次前向传播 → hidden states                         │
├─────────────────────────────────────────────────────────────┤
│ 4. Type Embeddings                                           │
│    └── 为 Noul/Choice/Score 添加类型信息                     │
├─────────────────────────────────────────────────────────────┤
│ 5. Scorer Heads                                              │
│    └── 从 marker 位置提取 logits                             │
├─────────────────────────────────────────────────────────────┤
│ 6. Temperature Scaling                                       │
│    └── 按问题类型和选项数缩放 logits                         │
├─────────────────────────────────────────────────────────────┤
│ 7. Softmax → Probabilities                                   │
│    └── 计算概率分布 + 置信度                                 │
└─────────────────────────────────────────────────────────────┘
```

### 模型权重结构

```python
# rl_agent_config.json
{
    "encoder": "ModernBERT-large",
    "head_layers": 2,
    "max_len": 512,
    "head_max_len": 192,
    "temperature": [1.0, 1.0, 1.0],
    "temperature_by_options": {}
}

# model.safetensors 包含:
# - encoder.* (ModernBERT 权重)
# - type_emb.* (类型嵌入)
# - scorer.* (评分头)
# - act_head.* (动作头)
```

---

## Rust 实现方案

### 技术选型

| 组件 | Python (Laya) | Rust (重构) | 理由 |
|------|---------------|-------------|------|
| Encoder | ModernBERT (transformers) | candle-transformers BERT | 纯 Rust，无 Python 依赖 |
| Tokenizer | tokenizers | tokenizers crate | 同一个库，直接用 |
| 权重加载 | safetensors | safetensors crate | 直接加载 SafeTensors |
| 矩阵运算 | PyTorch | candle-core | GPU 加速 |

### 文件结构

```
crates/neotrix-decision-engine/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── types.rs           # Question, Answer, State
    ├── engine.rs          # DecisionEngine 核心
    ├── error.rs           # Error 类型
    ├── tokenizer.rs       # Tokenizer 封装
    ├── sequence.rs        # build_sequence (marker 插入)
    ├── model/
    │   ├── mod.rs
    │   ├── encoder.rs     # ModernBERT encoder
    │   ├── type_emb.rs    # 类型嵌入
    │   ├── scorer.rs      # 评分头
    │   └── act_head.rs    # 动作头
    ├── router.rs          # 语言/脚本检测
    └── backends/
        └── candle_backend.rs
```

---

## 关键实现

### 1. Sequence Builder (核心)

```rust
// sequence.rs
pub struct SequenceBuilder {
    tokenizer: Tokenizer,
    max_len: usize,
    head_max_len: usize,
}

impl SequenceBuilder {
    /// 构建带 markers 的序列
    /// 
    /// 输入: state + question
    /// 输出: (input_ids, marker_positions, question_type)
    pub fn build(
        &self,
        state: &str,
        question: &Question,
    ) -> Result<(Vec<u32>, Vec<usize>, QuestionType)> {
        // 1. Tokenize state
        let state_tokens = self.tokenizer.encode(state)?;
        
        // 2. Tokenize question options
        let (option_tokens, markers) = self.tokenize_options(question)?;
        
        // 3. 组合: [CLS] state [SEP] options [SEP]
        let mut input_ids = vec![self.cls_token()];
        input_ids.extend(&state_tokens);
        input_ids.push(self.sep_token());
        
        let marker_offset = input_ids.len();
        input_ids.extend(&option_tokens);
        input_ids.push(self.sep_token());
        
        // 4. 调整 marker 位置
        let marker_positions: Vec<usize> = markers
            .iter()
            .map(|m| m + marker_offset)
            .collect();
        
        // 5. 截断到 max_len
        input_ids.truncate(self.max_len);
        
        Ok((input_ids, marker_positions, question.question_type.clone()))
    }
    
    /// Tokenize 选项并记录 marker 位置
    fn tokenize_options(&self, question: &Question) -> Result<(Vec<u32>, Vec<usize>)> {
        let mut tokens = Vec::new();
        let mut markers = Vec::new();
        
        match &question.question_type {
            QuestionType::Choice { criteria, .. } => {
                for (opt, _desc) in criteria {
                    markers.push(tokens.len());
                    let opt_tokens = self.tokenizer.encode(opt)?;
                    tokens.extend(opt_tokens);
                    tokens.push(self.sep_token());
                }
            }
            QuestionType::Score { criteria, .. } => {
                for (i, level) in criteria.iter().enumerate() {
                    markers.push(tokens.len());
                    let level_tokens = self.tokenizer.encode(level)?;
                    tokens.extend(level_tokens);
                    if i < criteria.len() - 1 {
                        tokens.push(self.sep_token());
                    }
                }
            }
            QuestionType::Noul { .. } => {
                markers.push(tokens.len());
                tokens.extend(self.tokenizer.encode("true")?);
                tokens.push(self.sep_token());
                markers.push(tokens.len());
                tokens.extend(self.tokenizer.encode("false")?);
            }
        }
        
        Ok((tokens, markers))
    }
}
```

### 2. Model 结构

```rust
// model/mod.rs
pub struct LayaModel {
    encoder: ModernBERTEncoder,
    type_emb: TypeEmbedding,
    scorer: ScorerHead,
    act_head: ActHead,
}

impl LayaModel {
    pub fn forward(
        &self,
        input_ids: &Tensor,
        attention_mask: &Tensor,
        marker_pos: &Tensor,
        marker_mask: &Tensor,
        qtype: &Tensor,
    ) -> Result<(Tensor, Tensor)> {
        // 1. Encoder forward
        let hidden = self.encoder.forward(input_ids, attention_mask)?;
        
        // 2. Add type embeddings
        let type_emb = self.type_emb.forward(qtype)?;
        let hidden = hidden + type_emb;
        
        // 3. Extract marker representations
        let marker_repr = self.extract_markers(&hidden, marker_pos)?;
        
        // 4. Score
        let logits = self.scorer.forward(&marker_repr)?;
        
        // 5. Action prediction
        let act_logits = self.act_head.forward(&hidden)?;
        
        Ok((logits, act_logits))
    }
    
    /// 从 hidden states 中提取 marker 位置的表示
    fn extract_markers(
        &self,
        hidden: &Tensor,
        marker_pos: &Tensor,
    ) -> Result<Tensor> {
        // hidden: [batch, seq_len, hidden_dim]
        // marker_pos: [batch, num_markers]
        // 输出: [batch, num_markers, hidden_dim]
        
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
```

### 3. Temperature Scaling

```rust
// engine.rs
pub struct TemperatureScaler {
    default: [f64; 3],  // [noul, choice, score]
    by_options: HashMap<(usize, usize), f64>,  // (qtype, num_options) -> temp
}

impl TemperatureScaler {
    pub fn scale(&self, logits: &Tensor, qtype: QuestionType, num_options: usize) -> Tensor {
        let temp = self.get_temperature(qtype, num_options);
        logits / temp
    }
    
    fn get_temperature(&self, qtype: QuestionType, num_options: usize) -> f64 {
        let bucket = (qtype as usize, num_options);
        self.by_options.get(&bucket)
            .copied()
            .unwrap_or(self.default[qtype as usize])
    }
}
```

### 4. Router

```rust
// router.rs
pub struct Router {
    english: LayaModel,
    multilingual: LayaModel,
    typed: LayaModel,
}

impl Router {
    pub fn predict(&self, state: &State, questions: &[Question]) -> Result<EvaluationResult> {
        // 1. 检测脚本
        let script = detect_script(&state.content);
        
        // 2. 选择模型
        let model = match script {
            Script::Latin => &self.english,
            Script::Devanagari | Script::Arabic | Script::CJK => &self.multilingual,
            Script::TypedDecisions => &self.typed,
        };
        
        // 3. 推理
        model.predict(state, questions)
    }
}

/// 检测文本脚本
pub fn detect_script(text: &str) -> Script {
    let chars: Vec<char> = text.chars().collect();
    let total = chars.len() as f64;
    
    if total == 0 {
        return Script::Unknown;
    }
    
    // 统计各类脚本的字符比例
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
```

---

## 实现优先级

| 阶段 | 任务 | 工作量 | 依赖 |
|------|------|--------|------|
| **P0** | Tokenizer 集成 | 3 天 | tokenizers crate |
| **P0** | Sequence Builder | 1 周 | Tokenizer |
| **P0** | 权重加载 (SafeTensors) | 3 天 | safetensors crate |
| **P1** | Encoder (ModernBERT) | 2-3 周 | candle-transformers |
| **P1** | Type Embeddings | 3 天 | Encoder |
| **P1** | Scorer + Act Head | 1 周 | Encoder |
| **P2** | Router | 3 天 | 模型完成 |
| **P2** | Temperature Scaling | 2 天 | 模型完成 |
| **P3** | GPU 加速优化 | 1-2 周 | 全部完成 |

---

## 替代方案

如果完整重构太复杂，可以：

### 方案 A: Python FFI

```rust
// 用 pyo3 调用 Python Laya
use pyo3::prelude::*;

pub struct LayaFFI {
    py_model: Py<PyAny>,
}

impl LayaFFI {
    pub fn new(model_path: &str) -> PyResult<Self> {
        Python::with_gil(|py| {
            let laya = py.import("laya")?;
            let agent = laya.call_method1("load", (model_path,))?;
            Ok(Self { py_model: agent.into() })
        })
    }
    
    pub fn predict(&self, state: &str, questions: &str) -> PyResult<String> {
        Python::with_gil(|py| {
            let result = self.py_model.call_method1(py, "predict", (state, questions))?;
            Ok(result.to_string())
        })
    }
}
```

### 方案 B: ONNX Runtime

```rust
// 导出 ONNX 模型，用 ort 加载
use ort::{Session, SessionBuilder, inputs};

pub struct OnnxBackend {
    session: Session,
}

impl OnnxBackend {
    pub fn from_file(path: &Path) -> Result<Self> {
        let session = SessionBuilder::new()?.commit_from_file(path)?;
        Ok(Self { session })
    }
    
    pub fn run(&self, input_ids: &Tensor) -> Result<Tensor> {
        let outputs = self.session.run(inputs![input_ids]?)?;
        Ok(outputs[0].try_extract_tensor()?)
    }
}
```

---

## 结论

| 方案 | 工作量 | 优先级 |
|------|--------|--------|
| 完整 Rust 重写 | 2-3 个月 | 长期目标 |
| Python FFI | 1-2 周 | 短期可行 |
| ONNX Runtime | 1 周 | 中期可行 |
| 直接用 Python Laya | 今天 | 立即可用 |

**建议**: 先用 Python Laya 验证效果，再决定是否用 Rust 重写。
