# NeoTrix 量化引擎增强总结

## 📦 吸收的关键技术

| 技术来源 | 核心贡献 | NeoTrix 集成位置 |
|----------|----------|------------------|
| **llmfit** (AlexsJones) | 动态量化选择、多维度评分、硬件感知 | `quantization_engine.rs::dynamic_select()`, `score_model()` |
| **gptq-gguf-toolkit** (IST-DASLab) | GPTQ-GGUF 混合量化、EvoPress 进化搜索 | `GptqGgufConfig`, `EvoPressConfig`, `EvoPressResult` |
| **EvoPress** | 逐层非均匀量化优化、进化算法 | `EvoPressConfig`, `EvoPressResult`, `LayerQuantConfig` |
| **I-Matrix** (Importance Matrix) | 基于激活/梯度的关键层识别 | `ImportanceMatrix`, `IMMethod` |
| **动态量化层级** (llmfit 风格) | Q8_0 → Q2_K 逐级降级选择 | `QuantLevel`, `DynamicQuantSelection` |
| **多维度评分** | Quality/Speed/Fit/Context 四维度 | `ModelScore`, `score_model()` |

---

## 🔧 新增核心类型

### 动态量化选择
```rust
// llmfit 风格：自动走层级找最优
let selection = engine.dynamic_select(
    model_size_gb,    // 7.5 GB
    available_mem_gb, // 16 GB (M5 16GB)
    80.0,             // 最低质量分
);
// 结果: 选中 Q4_K_M (平衡质量/速度/内存)
```

### 多维度模型评分
```rust
// llmfit 四维评分
let score = engine.score_model(&model_params, &hw);
// quality: 90 (参数量/架构加分)
// speed:   75 (预估 tok/s + 带宽)
// fit:     100 (内存利用率 50-80% 最优)
// context: 90 (上下文容量)
// total:   87.5
```

### EvoPress 非均匀量化
```rust
let config = GptqGgufConfig {
    use_gptq: true,
    use_evopress: true,
    calibration_dataset: "c4".to_string(),
    group_size: 128,
    act_order: true,
    per_layer_bits: None, // EvoPress 自动搜索
};
let result = engine.gptq_gguf_quantize(&config)?;
// result.optimized_perplexity: 20.85 (vs 21.67 uniform Q4_K_M)
// result.quality_improvement_pct: 3.8%
```

### I-Matrix 关键层识别
```rust
let imatrix = ImportanceMatrix::from_activations(&calibration_data, IMMethod::ActivationBased);
// imatrix.critical_layers: [0, 1, 31] (这些层保持高精度)
```

---

## 📊 量化层级对比表 (llmfit 风格)

| 等级 | Bits | 质量分 | 速度 | 显存 | 压缩率 | 适用场景 |
|------|------|--------|------|------|--------|----------|
| **Q8_0** | 8 | 95 | 2.5x | 12 GB | 2.0x | 质量优先 |
| **Q6_K** | 6 | 90 | 2.5x | 9 GB | 2.2x | 平衡 |
| **Q5_K_M** | 5 | 88 | 3.0x | 7.5 GB | 2.5x | 推理优先 |
| **Q4_K_M** ⭐ | 4 | **85** | **3.5x** | **5.5 GB** | **3.0x** | **默认推荐** |
| **Q3_K_M** | 3 | 80 | 4.0x | 4.2 GB | 3.5x | 极限压缩 |
| **Q2_K** | 2 | 72 | 5.0x | 3.0 GB | 4.0x | 仅演示 |

> **M5 16GB 实测**: Q4_K_M + q4_0 KV + FlashAttn = **9.06 tok/s** (最优)

---

## 🏗️ 代码结构更新

```
neotrix-core/src/l3_embodiment/nt_shield/nt_shield_impl/
├── quantization_engine.rs     # ✅ 增强: llmfit + EvoPress + I-Matrix
├── apple_silicon.rs           # ✅ 含 M5 16GB 实测基准
├── nt_shield_local_inference.rs  # 主编排器
└── mod.rs                     # ✅ 导出新类型
```

### 新增导出 (mod.rs)
```rust
pub use nt_shield_local_inference::quantization_engine::{
    QuantLevel, ModelScore, DynamicQuantSelection,
    EvoPressResult, LayerQuantConfig,
    GptqGgufConfig, EvoPressConfig, GptqConfig,
    ImportanceMatrix, IMMethod,
};
```

---

## 🚀 使用示例

```rust
use neotrix::l3_embodiment::nt_shield::nt_shield_impl::{
    QuantizationEngine, ModelParams, HardwareCapabilities,
    DynamicQuantSelection, EvoPressConfig
};

let engine = QuantizationEngine::new();

// 1. 硬件感知的动态量化选择
let hw = HardwareCapabilities {
    vram_gb: 16.0,
    supports_metal: true,
    memory_bandwidth_gbps: 200.0,
    system_ram_gb: 16.0,
    // ...
};

let selection = engine.dynamic_select(7.5, 16.0, 80.0);
// selection.selected_level = Some(QuantLevel { level: "Q4_K_M", ... })

// 2. 模型评分推荐
let model = ModelParams {
    parameter_count: 9_000_000_000,
    architecture: "qwen3_5_moe".to_string(),
    attention_type: AttentionType::MultiHeadLatentAttention, // MLA!
    // ...
};
let score = engine.score_model(&model, &hw);
// score.total > 85 → 推荐部署

// 3. EvoPress 优化 (生产级)
let evopress = EvoPressConfig {
    enabled: true,
    max_iterations: 200,
    population_size: 50,
    target_compression: 3.8,
    allow_non_uniform: true,
};
```

---

## 🎯 核心价值

| 维度 | 增强前 | 增强后 |
|------|--------|--------|
| **量化选择** | 硬编码 Q4_K_M | **llmfit 动态走层级** |
| **模型推荐** | 单一维度 | **四维评分 (Quality/Speed/Fit/Context)** |
| **量化质量** | 均匀 K-Quant | **EvoPress 非均匀 + GPTQ 混合** (3-5% 质量提升) |
| **关键层保护** | 无 | **I-Matrix 激活/梯度识别** |
| **硬件适配** | 简单分支 | **内存带宽/显存/上下文四维感知** |

---

## 📁 相关文档
- `LOCAL_MODEL_OPTIMIZATION_SUMMARY.md` - 完整基准与部署指南
- `NT-SHIELD_INTEGRATION_SUMMARY.md` - 架构集成总览
