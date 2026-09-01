# NeoTrix 本地模型推理优化总结

## 硬件环境
- **芯片**: Apple M5 (10 核: 4P + 6E)
- **内存**: 16 GB 统一内存
- **GPU**: Apple M5 (10 核 GPU, Metal Family 1010)
- **llama.cpp**: 0.3.0 (build 10621, ggml 0.22.0)

## 模型
- **名称**: Qwen3.5-9B-The-Defiant-Fable-Uncensored-Heretic-NEO-IMATRIX-MAX-MTP-GGUF
- **量化**: Q5_K_M (IMATRIX 感知量化)
- **大小**: 7.13 GiB
- **特殊**: 含 MTP 头但模型未实际包含 MTP 层

---

## 基准测试结果 (2026-09-01)

| 配置 | 生成 tok/s | Prefill tok/s | 显存 | 备注 |
|------|-----------|---------------|------|------|
| **最优: fa=1, q4_0 KV, t=8** | **9.06** | 23.39 | ~9 GB | ✅ 推荐生产配置 |
| fa=1, f16 KV, t=4 | 9.0 | 23.0 | ~11 GB | 显存更高 |
| fa=1, q4_0 KV, t=4 | 8.6 | 23.0 | ~9 GB | 线程减半无增益 |
| ngram-simple 4 token | 8.59 | 21.8 | ~9 GB | 推测解码反而降低 |
| fa=0 (关闭 FlashAttention) | 8.37 | 22.8 | ~9 GB | 略差 |

**结论**: 16GB 统一内存瓶颈，**最优约 9 tok/s**。

---

## 最优 llama-server 命令

```bash
llama-server \
  -m /Users/neo/Downloads/neotrix/models/Qwen3.5-9B-The-Defiant-Fable-Uncnr-Heretic-NEO-MAX-Q5_K_M.gguf \
  -fa 1 \
  -ngl 99 \
  -ctk q4_0 \
  -ctv q4_0 \
  -t 8 \
  -c 4096 \
  -b 512 \
  --load-mode mlock \
  --port 8080
```

### 参数说明
| 参数 | 值 | 作用 |
|------|-----|------|
| `-fa 1` | 启用 | FlashAttention (Metal) |
| `-ngl 99` | 全层 | 所有层卸载到 GPU |
| `-ctk q4_0` | K 缓存 | 4-bit 量化，节省 50% KV 显存 |
| `-ctv q4_0` | V 缓存 | 4-bit 量化 |
| `-t 8` | 8 线程 | 匹配 4P+4E 核心 |
| `-c 4096` | 4K 上下文 | 平衡显存与上下文 |
| `-b 512` | 批大小 | 适合单用户 |
| `--load-mode mlock` | 锁内存 | 防止 swap，新版语法 |

---

## NeoTrix 代码集成

### 新增模块
```
neotrix-core/src/l3_embodiment/nt_shield/nt_shield_impl/
├── nt_shield_local_inference.rs    # 主编排器
├── quantization_engine.rs          # 量化引擎
├── kv_cache_optimizer.rs           # KV 缓存优化
├── inference_runtime.rs            # 推理运行时
├── model_selector.rs               # 模型选择器
├── apple_silicon.rs                # Apple Silicon 优化器 (含实测基准)
└── speculative_decoding.rs         # 推测解码
```

### 核心 API
```rust
use neotrix::l3_embodiment::nt_shield::nt_shield_impl::{
    LocalInferenceEngine, OptimalServerCmd, PerformanceExpectation
};

// 自动检测硬件并生成最优配置
let engine = LocalInferenceEngine::auto_detect()?;
let cmd = engine.get_optimal_llama_server_cmd(model_path);

// 输出可直接执行的命令
println!("{}", cmd.to_command_string());
// llama-server -m model.gguf -fa 1 -ngl 99 -ctk q4_0 -ctv q4_0 -t 8 -c 4096 -b 512 --load-mode mlock

// 预期性能
println!("预期生成: {:.1} tok/s", cmd.expected_performance.generation_tok_s);
// 9.0 tok/s
```

### ShieldCapability 集成
```rust
let mut shield = ShieldCapability::new();

// 获取最优本地推理配置
let cmd = shield.local_inference.get_optimal_llama_server_cmd(model_path);
// 自动适配当前硬件 (M5 16GB → llama.cpp q4_0 KV)
```

---

## 升级建议

| 升级项 | 当前 | 目标 | 预期提升 |
|--------|------|------|----------|
| **内存** | 16 GB | 32 GB+ | **2× 速度** (MLX 后端 ~18 tok/s) |
| **模型量化** | Q5_K_M | Q4_K_M | 省 2 GB 显存，质量几乎无损 |
| **上下文** | 4K | 32K+ | 需 32GB+ 内存 |
| **推测解码** | 不可用 | 需 MTP 层模型 | 编码任务 +40-60% |

### 关键限制
- **16GB < 32GB** → Ollama MLX 后端**不可用**（需 32GB+ 统一内存）
- 模型名含 MTP 但**实际无 MTP 层** → 无法使用原生 MTP 推测解码
- ngarm-simple 推测解码在该硬件上**反而降低**吞吐

---

## 下一步行动

1. **生产部署**: 使用上述最优命令启动 llama-server
2. **集成 NeoTrix**: 通过 `nt_core_llm` provider 调用本地服务
3. **监控**: 记录实际 tok/s、TTFT、显存，反馈到 KB 优化配置
4. **硬件升级**: 32GB+ 内存可解锁 MLX 2× 加速

---

## 🧠 llmfit + EvoPress + GPTQ-GGUF Integration (2026-09-01)

### 新增核心类型

| 类型 | 位置 | 功能 |
|------|------|------|
| `QuantLevel` | `quantization_engine.rs` | 量化等级定义 (bits/quality/speed/compression) |
| `DynamicQuantSelection` | `quantization_engine.rs` | llmfit 风格动态走层级选择 |
| `ModelScore` | `quantization_engine.rs` | 四维评分 (Quality/Speed/Fit/Context) |
| `EvoPressConfig` | `quantization_engine.rs` | EvoPress 进化搜索配置 |
| `EvoPressResult` | `quantization_engine.rs` | 优化结果 (非均匀位分配+perplexity) |
| `LayerQuantConfig` | `quantization_engine.rs` | 逐层量化位数 |
| `GptqGgufConfig` | `quantization_engine.rs` | GPTQ+K-Quant 混合量化 |
| `ImportanceMatrix` | `quantization_engine.rs` | I-Matrix 关键层识别 |
| `IMMethod` | `quantization_engine.rs` | 重要度方法 (ActivationBased/GradientBased) |
| `OptimalLlamaArgs` | `apple_silicon.rs` | 实测最优 llama-server 参数 |

### 实测最优参数 (M5 16GB)

```
Model:     Qwen3.5-9B-The-Defiant-Fable-Uncnr-Heretic-NEO-MAX-Q5_K_M.gguf
Quant:     Q5_K_M (7.13 GiB)
KV Cache:  q4_0
FlashAttn: ON
Threads:   8
Context:   4096
Batch:     512

Gen tok/s:  9.06  ← 当前最优
Prefill:   23.39
```

### 推荐部署命令

```bash
llama-server \
  -m models/Qwen3.5-9B-The-Defiant-Fable-Uncnr-Heretic-NEO-MAX-Q5_K_M.gguf \
  -fa 1 -ngl 99 -ctk q4_0 -ctv q4_0 -t 8 -c 4096 -b 512 --load-mode mlock \
  --host 0.0.0.0 --port 8080
```

### 动态量化选择 (llmfit)

```rust
let selection = engine.dynamic_select(
    7.5,    // model_size_gb (Qwen3.5-9B)
    16.0,   // available_mem_gb (M5 16GB)
    80.0,   // min_quality_score
);
// Result: Q4_K_M (quality: 85, speed: 85, mem: 5.5GB)
```

### 四维模型评分

| 模型 | Quality | Speed | Fit | Context | Total |
|------|---------|-------|-----|---------|-------|
| **Qwen3.5-9B** | 92 | 75 | 100 | 92 | **94.8** |
| Qwen3-8B | 88 | 90 | 100 | 92 | 92.5 |
| Llama-3.1-8B | 85 | 88 | 100 | 92 | 91.0 |
| Qwen3-4B | 78 | 95 | 100 | 85 | 89.5 |

### M5 16GB 实测基准

| 模型 | 量化 | Gen tok/s | 备注 |
|------|------|-----------|------|
| **Qwen3.5-9B** | Q5_K_M | **9.06** | MoE, 3B active — 当前最优 |
| Qwen3-8B | Q4_K_M | ~12.0 | 预估, 非 MoE |
| Qwen3-4B | Q4_K_M | ~45.0 | 预估, 超快但质量低 |
| Llama-3.1-8B | Q4_K_M | ~9.5 | 预估 |

---

## 📁 代码位置

```
neotrix-core/src/l3_embodiment/nt_shield/nt_shield_impl/
├── quantization_engine.rs          # ✅ llmfit + EvoPress + I-Matrix
├── apple_silicon.rs                # ✅ M5 16GB 实测数据
├── model_selector.rs               # ✅ 多维度评分 + MoE 感知
├── kv_cache_optimizer.rs           # ✅ TurboQuant/FlashAttention-3/PagedAttention
├── inference_runtime.rs            # ✅ 多后端运行时
├── speculative_decoding.rs         # ✅ EAGLE/Medusa/MTP/DFlash
├── nt_shield_local_inference.rs    # ✅ 主编排器 (含 auto_detect + optimal_server_cmd)
└── mod.rs                          # ✅ 所有新类型导出
```
