# 第96批 破限制技术 — 5大主题

> 采集时间: 2026-09-11 | 每主题 3-5 来源

---

## 主题1: 模型量化 (Model Quantization)

### 核心发现

- **AF1 (All for 1-Bit)**: 真正的 1-bit PTQ 框架，NABF 空间感知二值分解 + HiSA 分层 Shapley 分配，严格 1.0-BPW 预算下 LLaMA/Qwen/Gemma 全系列 SOTA，2.5× 推理加速 + 90%+ 内存削减 (arXiv:2609.06161, EMNLP 2026)
- **HyperQuant**: 率失真最优量化管线，per-tile RHT + E8/D4/A2 格量化 + Rice 熵编码，权重 3.9× 压缩 + KV cache 3.79× 压缩，int8 优于 fp8，支持 Blackwell nvfp4/mxfp4 (arXiv:2606.23406)
- **KronQ**: Kronecker 分解 Hessian 量化，引入梯度协方差 H_G 双向不相干处理 + 子层混合精度分配，LLaMA-3-70B W2 下 GPTQ/GPTAQ 发散而 KronQ 仅 7.93 PPL (arXiv:2607.07964)
- **REAL-Q**: 端到端对齐动态量化，聚合 Fisher MSE 目标 + Block-GD 每 128 列动态修正 + 滑动窗口平滑，W4A16 KL 散度降低达 49% (arXiv:2609.00049)
- **QTEA**: 亚 2-bit 三元 PTQ，三元基 + 列半稀疏 FP8 残差补偿 + 列级 rescale 交替优化 + 误差衰减，Qwen3-14B 有效 1.7 bpw，LUT kernel 7.2× 加速 (arXiv:2609.00224)

### 来源

| # | 来源 | 标题 | 时间 |
|---|------|------|------|
| 1 | arXiv:2609.06161 | All for 1-Bit: Towards Genuine 1-Bit Post-Training Quantization for LLMs | 2026-09 |
| 2 | arXiv:2606.23406 | HyperQuant: A Rate–Distortion-Optimal Quantization Pipeline | 2026-06 |
| 3 | arXiv:2607.07964 | KronQ: LLM Quantization via Kronecker-Factored Hessian | 2026-07 |
| 4 | arXiv:2609.00049 | REAL-Q: E2E LLM Quantization via Dynamic Gradient Descent | 2026-08 |
| 5 | arXiv:2609.00224 | QTEA: Ternary LLMs with Sparse Residual Salient Weight | 2026-08 |

---

## 主题2: 推测解码 (Speculative Decoding)

### 核心发现

- **Saguaro (SSD)**: 推测的推测解码，并行化推测与验证，draft 模型预测验证结果并提前准备，缓存命中策略 + 几何扇出，比最强 SD 基线快 30%，比自回归快 5× (NeurIPS 2026/ICML 2026)
- **AdaptiveSpec**: 训练无关自适应推测，per-step margin 规则放宽 token 匹配 + per-step 树策略自适应深度/宽度，EAGLE-3 吞吐量提升达 56%，恢复 93% 全无损精度 (2026)
- **UNISPEC**: 训练无关跨语言跨硬件推测解码，设备感知校准最优 draft 长度 + 置信度评分树构建 + 增强首层扩展，7 语言 7 任务加速达 2.6× (ACL 2026)
- **Speculative Verification (SV)**: 轻量伴随模型 + 信息增益预测推测精度 + 动态调整验证长度，大 batch 下比标准 SD 快 1.9×，100+ 组合验证 (2026)
- **SparseSpec-L**: 训练无关自推测解码，动态稀疏 KV cache + 注意力统计可召回索引 + 熵引导自适应推测长度，长上下文推理加速达 2.79× (2026)

### 来源

| # | 来源 | 标题 | 时间 |
|---|------|------|------|
| 1 | NeurIPS 2026 | Speculative Speculative Decoding (Saguaro) | 2026 |
| 2 | arXiv:2609.02897 | AdaptiveSpec: Training-Free Per-Step Lossy Speculative Decoding | 2026-07 |
| 3 | ACL 2026 | UNISPEC: Training-Free Speculative Decoding for Robust LLM Acceleration | 2026 |
| 4 | ACL Findings 2026 | Speculative Verification: Exploiting Information Gain for SD | 2026 |
| 5 | arXiv:2607.27735 | SparseSpec-L: Train-Free Self-Speculative Decoding | 2026-07 |

---

## 主题3: KV 缓存优化 (KV Cache Optimization)

### 核心发现

- **Attention Matching (AM)**: 潜空间快速 KV 压缩，匹配注意力输出 + 注意力质量闭式解，OMP 键选择 + 非均匀头预算，50× 压缩秒级完成，质量接近 Cartridges (arXiv:2602.16284, 2026)
- **DeepSeek V4 Compressed Attention**: 序列维度压缩新范式，CSA (4 token 组) + HCA (128 token 全局摘要) + 低秩投影，KV cache 仅占标准 2%，支持 100 万 token 上下文 (2026)
- **CCA/CCGQA**: 压缩卷积注意力，Q/K/V 共享潜空间全注意力计算，RoPE 无缝集成，8× KV cache 压缩零性能损失，H100 prefill 加速 1.7× (arXiv:2510.04476v2, 2026)
- **GSA (Gist Sparse Attention)**: Gist token 作为路由信号的选择性展开，压缩→选择→恢复粗到细机制，8×-32× 压缩比，层次化 gist-of-gist 实现 O(log n) 解码 (arXiv:2604.20920, 2026)

### 来源

| # | 来源 | 标题 | 时间 |
|---|------|------|------|
| 1 | arXiv:2602.16284 | Fast KV Compaction via Attention Matching | 2026-02 |
| 2 | DeepSeek Blog | DeepSeek V4 Compressed Attention — KV-Cache Explained | 2026-04 |
| 3 | arXiv:2510.04476v2 | Compressed Convolutional Attention: Efficient Attention in Compressed Latent Space | 2026-03 |
| 4 | arXiv:2604.20920 | GSA: Gist Sparse Attention with Selective Unfolding | 2026 |

---

## 主题4: 注意力压缩 (Attention Compression)

### 核心发现

- **gzip 压缩率自适应稀疏注意力**: 无需参数，gzip 压缩比作为长程注意力路由信号，不可压缩块 = 高信息密度目标，PG-19 达 1.71 BPB 超 Dense/BigBird/Longformer (arXiv:2607.21752, 2026)
- **Attn-GS**: 注意力引导上下文压缩个性化 LLM，白盒标记模型 attention 反馈标记关键句 + 压缩模型生成高质量压缩上下文，50× 压缩保持 98.2% 全上下文性能 (ACL 2026)
- **CCA (Compressed Convolutional Attention)**: 潜空间全注意力计算，参数 + KV cache + FLOPs 同时压缩，组合 GQA 实现 8× 压缩，MLA 2× 参数节省，prefill 1.7× 加速 (arXiv:2510.04476v2, 2026)
- **DeepSeek V4 Compressed Attention**: 混合压缩栈 HCA(早期)→交替→全注意力(末层)，CSA 局部精度 + HCA 全局摘要，2% KV cache 占用支持百万 token (2026)
- **Learnable ADSC (Attention Driven Structured Compression)**: 可学习注意力 MLP 计算滤波器重要性，50-60% 压缩零精度损失甚至 CIFAR-100 精度提升 0.68% (Nature Sci Rep, 2026)

### 来源

| # | 来源 | 标题 | 时间 |
|---|------|------|------|
| 1 | arXiv:2607.21752 | Parameter-free Adaptive Sparse Attention via Compression-Based Content Selection | 2026-07 |
| 2 | ACL 2026 | Attn-GS: Attention-Guided Context Compression for Efficient Personalized LLMs | 2026 |
| 3 | arXiv:2510.04476v2 | Compressed Convolutional Attention (CCA/CCGQA) | 2026-03 |
| 4 | DeepSeek Blog | DeepSeek V4 Compressed Attention — KV-Cache Explained | 2026-04 |
| 5 | Nature Sci Rep | Learnable Attention Driven Structured Compression (ADSC) | 2026-04 |

---

## 主题5: 模型剪枝 (Model Pruning)

### 核心发现

- **Wisp/Whisper**: Wasserstein 启发剪枝，输出差异作为显著性信号，MLP up/gate 投影保留成对输出差异，LLaMA-2/3.1 50%-75% 稀疏度全面超越 SparseGPT，组合 RIA/ALPS 进一步提升 (arXiv:2608.06630, 2026)
- **非结构化剪枝增强 TTS**: 10-20% 非结构化剪枝 (Wanda/Magnitude) 在推理 LLM 上反而超越未剪枝模型，结构化剪枝 (ShortGPT) 仍严重退化，挑战剪枝有害论 (arXiv:2604.25098, 2026)
- **99% 稀疏度**: 渐进稀疏化 + 二阶显著性 + 与稀疏度同步的持续训练，LLaMA-2-7B 95% 稀疏度 13.48 PPL + 3.23× 解码加速 + 6.21× 内存节省 (arXiv:2609.06557, 2026)
- **PRUNE&COMP**: 迭代层剪枝 + 幅度补偿，移除层引起隐藏状态幅度缺口，离线权重缩放补偿，LLaMA-3-8B 5 层剪枝 PPL 从 512.78 降至 16.34 (AAAI 2026)
- **GRASPrune**: 全局预算结构化剪枝，轻量门控联合剪枝 FFN 通道 + 注意力 KV 头组，LLaMA-2-7B 移除 50% 参数达 12.18 PPL，无微调 (ACL 2026)

### 来源

| # | 来源 | 标题 | 时间 |
|---|------|------|------|
| 1 | arXiv:2608.06630 | The Sparsity Whisperer (Wisp/Whisper) | 2026-08 |
| 2 | arXiv:2604.25098 | Revisiting LLM Pruning for Test-Time Scaling | 2026-04 |
| 3 | arXiv:2609.06557 | Hidden in Plain Sight: Canonical Elements for Extreme LLM Sparsity | 2026-09 |
| 4 | AAAI 2026 | PRUNE&COMP: Free Lunch for Layer-Pruned LLMs | 2026-03 |
| 5 | ACL 2026 | GRASPrune: Global Gating for Budgeted Structured Pruning | 2026 |

---

## 跨主题趋势

| 趋势 | 表现 |
|------|------|
| **1-bit 成为现实** | AF1 达到严格 1.0 BPW，BitNet b1.58 2B 已开源，1-bit scaling law 被理论证明 |
| **训练无关成为标配** | AdaptiveSpec/UNISPEC/SparseSpec-L 均无需训练，推理时自适应调整 |
| **序列维度压缩崛起** | DeepSeek V4 将压缩从 head 转向 sequence，2% KV cache + 百万 token |
| **非结构化剪枝逆袭** | 10-20% 非结构化剪枝可超越未剪枝模型，99% 稀疏度可达 19.67 PPL |
| **推测解码并行化** | SSD 突破 draft→verify 串行瓶颈，DFlash 6× 无损加速，生产部署成熟 |
