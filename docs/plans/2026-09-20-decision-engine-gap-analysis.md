# Decision Engine 架构差距分析

> 评估当前实现 vs Qwen2.5-0.5B 集成需求
> 日期: 2026-09-20

---

## 当前架构 vs 目标架构

```
当前实现 (Mock)                    目标实现 (Qwen2.5-0.5B)
─────────────────                  ─────────────────────────
DecisionEngine                     DecisionEngine
    │                                  │
    ├── MockBackend                 ├── CandleBackend
    │   └── 返回固定 JSON           │   ├── Tokenizer
    │                              │   ├── Model (SafeTensors)
    └── 无推理能力                 │   ├── Device (CPU/Metal/CUDA)
                                   │   └── 推理引擎
```

---

## 10 个关键缺陷

### 缺陷 1: 没有 Tokenizer

**现状**: MockBackend 不需要 tokenizer
**需求**: Qwen2.5 需要特定的 tokenizer（基于 tiktoken）

```rust
// 缺失
pub struct Tokenizer {
    vocab: HashMap<String, u32>,
    merges: Vec<(String, String)>,
}

impl Tokenizer {
    pub fn encode(&self, text: &str) -> Vec<u32> { ... }
    pub fn decode(&self, tokens: &[u32]) -> String { ... }
}
```

**影响**: 无法将文本转换为模型输入

---

### 缺陷 2: 没有模型加载

**现状**: MockBackend 不加载任何模型
**需求**: 加载 SafeTensors 格式的 Qwen2.5

```rust
// 缺失
pub struct ModelLoader;

impl ModelLoader {
    pub fn from_bytes(bytes: &[u8]) -> Result<Model> { ... }
    pub fn from_file(path: &Path) -> Result<Model> { ... }
}
```

**影响**: 无法加载真实模型

---

### 缺陷 3: 没有推理引擎

**现状**: MockBackend 直接返回固定结果
**需求**: 实际的神经网络推理

```rust
// 缺失
pub struct InferenceEngine {
    model: Model,
    device: Device,
}

impl InferenceEngine {
    pub fn forward(&self, input: &Tensor) -> Result<Tensor> { ... }
    pub fn generate(&self, tokens: &[u32], max_tokens: usize) -> Result<Vec<u32>> { ... }
}
```

**影响**: 无法进行实际推理

---

### 缺陷 4: 没有设备管理

**现状**: 无设备概念
**需求**: 支持 CPU/Metal/CUDA

```rust
// 缺失
pub enum DeviceType {
    Cpu,
    Metal,  // macOS
    Cuda,   // NVIDIA
}

pub struct DeviceManager {
    device: Device,
    memory_limit: usize,
}
```

**影响**: 无法利用 GPU 加速

---

### 缺陷 5: 没有 KV Cache

**现状**: 每次推理独立
**需求**: 自回归生成需要 KV Cache

```rust
// 缺失
pub struct KVCache {
    key_cache: Tensor,
    value_cache: Tensor,
    position: usize,
}

impl KVCache {
    pub fn update(&mut self, new_key: Tensor, new_value: Tensor) { ... }
}
```

**影响**: 生成速度慢，无法复用之前计算

---

### 缺陷 6: 没有概率校准

**现状**: MockBackend 返回任意概率
**需求**: 模型输出需要校准

```rust
// 缺失
pub struct ProbabilityCalibrator {
    temperature: f64,
    top_p: f64,
    top_k: usize,
}

impl ProbabilityCalibrator {
    pub fn calibrate(&self, logits: &Tensor) -> Vec<f64> { ... }
}
```

**影响**: 概率不可靠，置信度无意义

---

### 缺陷 7: 没有并行问题评估

**现状**: 问题串行处理
**需求**: JEV 风格的并行评估

```rust
// 缺失
impl DecisionEngine {
    pub fn evaluate_parallel(
        &self,
        state: &State,
        questions: &[Question],
    ) -> Result<Vec<Answer>> {
        // 所有问题共享同一个 prompt
        // 并行评估，不互相依赖
    }
}
```

**影响**: 多问题时性能差

---

### 缺陷 8: 没有流式输出

**现状**: 等待完整响应
**需求**: 实时输出

```rust
// 缺失
impl DecisionEngine {
    pub fn evaluate_streaming(
        &self,
        state: &State,
        questions: &[Question],
    ) -> impl Stream<Item = PartialAnswer> { ... }
}
```

**影响**: 用户体验差，延迟高

---

### 缺陷 9: 没有 Prompt 模板

**现状**: 硬编码 prompt 格式
**需求**: 可配置的 prompt 模板

```rust
// 缺失
pub struct PromptTemplate {
    system: String,
    user: String,
    assistant: String,
}

impl PromptTemplate {
    pub fn render(&self, state: &State, questions: &[Question]) -> String { ... }
}
```

**影响**: 难以优化和实验

---

### 缺陷 10: 没有模型量化支持

**现状**: 无量化支持
**需求**: 支持 Q4/Q8 量化以减小体积

```rust
// 缺失
pub enum Quantization {
    F16,
    Q8_0,
    Q4_K_M,
    Q4_0,
}

impl Model {
    pub fn quantize(&self, quant: Quantization) -> Result<Model> { ... }
}
```

**影响**: 模型体积大，无法内置

---

## 依赖关系图

```
                    ┌─────────────────┐
                    │ DecisionEngine  │
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
              ▼              ▼              ▼
        ┌──────────┐  ┌──────────┐  ┌──────────┐
        │ Tokenizer│  │ Inference│  │Prompt    │
        │          │  │ Engine   │  │Template  │
        └────┬─────┘  └────┬─────┘  └──────────┘
             │              │
             ▼              ▼
        ┌──────────┐  ┌──────────┐
        │  Model   │  │  Device  │
        │(SafeTensors)│ │(CPU/Metal)│
        └──────────┘  └──────────┘
```

---

## 优先级排序

| 优先级 | 缺陷 | 工作量 | 依赖 |
|--------|------|--------|------|
| **P0** | Tokenizer | 1-2 周 | 无 |
| **P0** | 模型加载 | 1-2 周 | Tokenizer |
| **P0** | 推理引擎 | 2-3 周 | Model |
| **P1** | 设备管理 | 1 周 | 推理引擎 |
| **P1** | KV Cache | 1-2 周 | 推理引擎 |
| **P1** | 概率校准 | 1 周 | 推理引擎 |
| **P2** | 并行评估 | 1 周 | 无 |
| **P2** | 流式输出 | 1-2 周 | 推理引擎 |
| **P2** | Prompt 模板 | 3 天 | 无 |
| **P3** | 量化支持 | 2-3 周 | 模型 |

---

## 推荐实现路径

### Phase 1: 基础推理 (2-3 周)

```
1. 集成 tokenizers crate (Hugging Face tokenizer)
2. 集成 candle-core (模型加载)
3. 集成 candle-transformers (Qwen2.5 模型)
4. 实现基本推理流程
```

### Phase 2: 优化 (1-2 周)

```
1. 添加 KV Cache
2. 实现设备管理 (CPU/Metal)
3. 添加概率校准
```

### Phase 3: 生产就绪 (1-2 周)

```
1. 并行问题评估
2. 流式输出
3. Prompt 模板
4. 错误处理和恢复
```

---

## 技术选型建议

| 组件 | 推荐 | 备选 |
|------|------|------|
| Tokenizer | `tokenizers` crate | 自实现 |
| 模型格式 | SafeTensors | GGUF (llama.cpp) |
| 推理引擎 | `candle` | `ort` (ONNX) |
| 设备 | CPU + Metal | + CUDA |
| 量化 | Q4_K_M | Q8_0 |

---

## 立即可做

1. 添加 `tokenizers` 依赖
2. 添加 `candle-core` + `candle-transformers` 依赖
3. 下载 Qwen2.5-0.5B 模型
4. 实现基本推理流程
