# Mac 训练加速方案（2026-09-24，资料+实测环境合成）

> 现状：ftv6 跑在 **MPS fp32**（Trainer 自动选 mps；MiniMind-3 = Qwen3 原生建模，
> 无自定义代码，`model_type: qwen3` 已验）。torch 2.12.1 / transformers 5.13 / macOS 27。
> 本文件只给方案不动现跑；下次跑（D3 LoRA / 新 full-ft）直接用。

## 1. 诊断（当前配置的 4 个慢因）

| # | 现状 | 代价 |
|---|------|------|
| 1 | `fp16=False` 全 fp32 | MPS 上 bf16 matmul 快 ~1.6×，内存省一半；现在全额付 |
| 2 | micro-batch=2（accum 8） | GPU 吃不饱；同样有效 batch 下大 micro-batch 更快 |
| 3 | 注意力走 eager（默认） | Qwen3 原生支持 SDPA，MPS 上 eager 纯 eager 开销 |
| 4 | collate 按 batch 最大长 padding（8–256 任意形状） | **MPS graph cache 无驱逐**（torch 2.12 不支持清 graph cache，要 2.13+），5.8 万步形状越积越多——日志里 step 耗时 1.3s↔3.2s 跳就是它 |

## 2. 处方（按收益排序，下次跑直接上）

1. **`bf16=True`**（macOS 27 ✓，HF 官方 Apple Silicon 文档确认 MPS 支持 bf16）——最大单项；
   NaN 史在，故 NanGuard 保留 + 先 `--smoke` 100 步验。
2. **micro-batch 2→8，accum 8→2**（有效 batch 保持 16，lr/ warmup 口径不动）——bf16 省下的内存供它。
3. **`attn_implementation="sdpa"`**（Qwen3 原生支持，`from_pretrained(..., attn_implementation="sdpa")` 一行）。
4. **定形 batch**：collate 改 pad 到 8 的倍数（`m=((max+7)//8)*8`，上限 256）——形状从 ~250 种压到 32 种，
   graph cache 不再无限涨。预期合计 **~2–2.5×**。
   - [x] 2026-09-24 已落地（`finetune.py:collate`，`lora_finetune` 同享；50 组随机验证过）。
     现跑 ftv6 已 import 旧代码，本 fix 从下次跑生效。
5. 可选：torch 升 2.13+ 开 `torch_empty_cache_steps`（清 graph cache）；`dataloader_num_workers=2`
   （小，tokenize 已前置）；`mps-sdpa` 第三方扩展（attention 训练再 ~2×，要动建模调用，排最后）。
6. 不做：`torch.compile`（inductor 无 MPS 后端）；fp16（bf16 更稳）；8-bit Adam（bitsandbytes 不支持 MPS）。

## 3. 落到 D3

D3 LoRA（rank-16，只训 <1% 参数）天然再快一截：optimizer 状态小一个量级，
batch 可更大；500 步按新配方约十几分钟。`lora_finetune.py` 改 4 行
（bf16/micro-batch/sdpa/定形 collate）+ `--smoke` 先验。

## 4. 来源
- HF `perf_train_special`（Apple Silicon：MPS 自动检测、bf16 要 macOS 14+、graph cache 说明）
- Apple Metal PyTorch 文档（MPS 后端要求）
- `crlandsc/mps-sdpa`（stock SDPA 在 MPS 上慢 2× 的根因：`sdpa_general_mps` 图开销；训练加速 ~2×）
- pytorch/pytorch#178545（MPS SDPA fast path 讨论，S≥1024 才显著——我们 256 以内，stock SDPA 够用）
- PyTorch 加速 LLM 博文（SDPA + vocab pad 思路；pad 思想复用到 batch 定形）

## 5. GitHub 项目技术（2026-09-24 实搜+实装验证）

| 项目 | 结论 | 依据 |
|------|------|------|
| `ml-explore/mlx-lm`（官方，7.1k★，内置 LoRA/full-ft） | **P0 采用（D3）**：venv `~/.venvs/mlx` 已装好，`mlx_lm.lora` CLI + qwen3 建模双验通过；Qwen3-0.6B SFT ~4.7it/s/2GB vs 现 MPS ~0.6 → **~7–8×** | 本窗实装验证 |
| `Goekdeniz-Guelmez/mlx-lm-lora`（12 种算法，DPO/GRPO 俱全） | P1：D3 首版用官方即可；偏好优化阶段再引 | 第三方 benchmark 表 |
| `crlandsc/mps-sdpa` | P1 备选（PyTorch 路）：attention 训练 ~2×，JIT 即装，monkeypatch 接入 + smoke | 作者 benchmark（macOS 15+、torch≥2.11，我方 27/2.12.1 符合） |
| `linkedin/Liger-Kernel` | ❌ 不用：Triton 内核**无 Metal 后端**，Mac 跑不了（对 Qwen3 支持再全也与我无关） | Triton 硬件谱（CUDA/ROCm/XPU，无 MPS） |
| `apple/ml-cross-entropy` | ❌ 不用：同上 Triton 系；且作者自认 research code | Liger #391 讨论串 |
| `unsloth` | ❌ 不用：CUDA only | — |
| `M1T8E6/LoRA-MPS-FineTuning` | 佐证：FP32-on-MPS 稳定性警示 → 我方 bf16 必须先 smoke（已在 §2 注明） | 其 README Tips |

MLX 路待办（D3 开工单）：① 权重转换（`mlx_lm.convert` 或 `mlx-community` 现成 Qwen3-0.6B；
注意底座用 ftv4 落盘权重转还是 HF 基模+adapter 二次对齐，二选一）→ ② `jev_labeled.jsonl`
转 `{train,valid}.jsonl` → ③ 首跑 500 步 → ④ `fuse` 回 HF 格式 + `eval_lora` parity 验收。
前提：ftv4 落盘（转权重用）+ adapter 格式与 PEFT 互认验证。

## 6. 残余优化点（2026-09-24 数据实测：中位 88 字符/p90 149/上限 256）

| # | 点 | 收益 | 代价/备注 |
|---|----|------|-----------|
| 1 | **Sequence packing**：短样本拼满 256 再训（现 ~70% pad 空烧） | **最大单项 ~3–5×/epoch** | 改 steps 语义（步内样本数变），只用于新跑；跨样本边界污染对 pretrain 可接受 |
   - [x] 2026-09-24 原型 `models/training/pack_texts.py` 自测过（best-fit-decreasing，2000 条→417 桶，填充率 0.87，省 82% 槽位；SFT/对话禁用，待 segment mask 见 D3）
| 2 | **Chunked CE**：vocab 151936，logits 256×151k fp32≈150MB/样本，手动分块算 loss | 省内存→batch 更大；mlx 侧已有 `chunked_nll` 佐证 | 手写 20 行，数值等价 |
   - [x] 2026-09-24 等价性已验（20 组随机，atol=1e-5 全对；155.6MB→38.9MB/样本；接线等内存窗）
| 3 | **LoRA 段 mask**：`lora_finetune` 现连 user 段一起训（docstring 自认） | 省 loss tokens + 质量正 | mlx 路 `--mask-prompt` 原生，PyTorch 路手加 |
   - [x] 2026-09-24 `--mask-user` 已落地（默认关零变化；假 tokenizer 6 项验证过；真跑等内存窗）
| 4 | 流式 tokenize：46 万条前置全 tokenize（启动慢、占 RAM） | 启动快 | 小 |
| 5 | Muon 优化器（mlx 原生支持 `muon`） | 收敛步数少 | 改动力学，只 smoke |
| 6 | Grad checkpoint | 只换内存不换速度（-20% 计算），当大 batch 跳板 | 中性 |
