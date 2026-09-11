# Model Reverse Engineering — 252 (2025-2026 Frontier Batch)

> 逆向推理 10 个前沿模型的架构创新，提取可迁移模式，映射 NeoTrix 能力网。

## TL;DR

2025-2026 前沿模型呈现 5 大收敛趋势：**MoE 细粒度专家路由**、**混合注意力压缩**、**多模态原生融合**、**推理时计算可控分配**、**KV Cache 极限压缩**。NeoTrix 已具备同构能力（GWT 注意力路由、VSA HyperCube、SEAL Pipeline），但缺少**运行时稀疏激活调度**和**1M+ 上下文 KV 虚拟化**。

---

## 1. GPT-4o (OpenAI, May 2024)

### 架构参数
| 维度 | 值 |
|------|----|
| 总参数 | ~200B (估计) |
| 架构 | Undisclosed (推测 Dense Transformer) |
| 上下文窗口 | 128K |
| 模态 | Text + Audio + Image → Text + Audio + Image |
| 训练方式 | 端到端多模态联合训练 |

### 核心创新
1. **端到端 Omni-Modal 统一网络**: 消除 ASR→LLM→TTS 管线链，单一神经网络处理所有模态，音频响应延迟从 5.4s 降至 320ms
2. **模态原生融合**: 不是"接驳"独立模态编码器，而是从预训练阶段就跨文本/视觉/音频联合优化
3. **情感/语调透传**: 音频模态直接感知情绪、多说话人、背景噪声，无需额外标注

### NeoTrix 映射
| 创新 | NeoTrix 同构 | 差距 |
|------|-------------|------|
| 端到端多模态 | NT-WORLD UnifiedCrawler + NT-IO 多模态输入 | NeoTrix 是能力网架构，非端到端训练；需要 **PerceptionBridge** 打通 L2→L5 多模态流 |
| 低延迟响应 | GWT 注意力路由 (Axiom A1 Cost-Aware) | 需要推理时 token 预算分配机制 |
| 情感透传 | NT-FEEL EmotionEngine (11 variants) | 已有 EmotionLabel 统一枚举，但缺乏**音频流式情感识别** |

---

## 2. Claude 3.5 Sonnet (Anthropic, Jun 2024)

### 架构参数
| 维度 | 值 |
|------|----|
| 总参数 | Undisclosed (推测 ~175B) |
| 架构 | Dense Transformer + Constitutional AI |
| 上下文窗口 | 200K |
| 模态 | Text + Image → Text |
| 特色 | 2x 速度 vs Opus，5x 成本优势 |

### 核心创新
1. **速度-智能 Pareto 前沿**: 中等规模模型超越旗舰 Opus，证明"数据质量 > 参数规模"
2. **Agentic Coding 能力**: SWE-bench 49.0% (升级版)，自主编写/编辑/执行代码，错误自纠正
3. **Artifacts 工作区**: 从纯对话到可交互内容生成（代码/文档/可视化），预示 agent-as-OS 范式
4. **Computer Use**: 首个支持浏览器操作的前沿模型，自主导航数字环境

### NeoTrix 映射
| 创新 | NeoTrix 同构 | 差距 |
|------|-------------|------|
| Pareto 优化 | Constellation 成熟度 (C0-C6) + Dual Specialization | 需要运行时**模型路由决策器** (GWT + Cost Weight) |
| Agentic 编码 | NT-ACT Dev-匠 (implementer) + NT-ACT 工具调用 | 已有 MCP tools，需增强**自纠正循环** (SEAL Phase-4) |
| Artifacts | NT-IO 界面使徒 | 需要 **ArtifactRegistry** 实现交互式内容工作区 |
| Computer Use | NT-WORLD 感知 + NT-ACT 行动 | 需要 **BrowserAgent** 能力节点 |

---

## 3. Gemini 2.5 Pro (Google DeepMind, Mar 2025)

### 架构参数
| 维度 | 值 |
|------|----|
| 总参数 | Undisclosed |
| 架构 | Dense Transformer + Thinking Model |
| 上下文窗口 | **1,048,576** (1M tokens) |
| 模态 | Text + Image + Audio + Video → Text |
| 特色 | 可控 Thinking Budget + Deep Think 模式 |

### 核心创新
1. **Thinking Budget 可控分配**: 开发者通过 API 参数控制模型"思考"时长，动态平衡推理质量与延迟/成本
2. **Deep Think 增强推理**: 多假设探索模式，在 USAMO 2025 数学竞赛达到 SOTA
3. **1M Token 上下文**: 突破性长上下文能力，可处理整小时视频、完整代码仓库
4. **Thought Summaries**: 推理过程透明化，用户可查看模型思考链摘要
5. **Flash-Lite 极致效率**: 同系列最低延迟成本版本，thinking 默认关闭

### NeoTrix 映射
| 创新 | NeoTrix 同构 | 差距 |
|------|-------------|------|
| Thinking Budget | GWT salience + Axiom A2 (Context as Scarce Resource) | 需实现 **ThinkingBudgetManager**，动态分配 NT-CORE 推理资源 |
| Deep Think | E8 Hexagram 多路径推理 | 已有多分支推理，需增强**假设并行探索**能力 |
| 1M 上下文 | KVMem paged KV (Axiom A2) | NeoTrix 已有 Axiom 共识，需实现 **paged KV virtualization** |
| Thought Summaries | ConsciousnessTree 6-stage feedback | 已有阶段反馈，需增加**推理过程可解释性输出** |

---

## 4. Llama 4 Scout (Meta, Apr 2025)

### 架构参数
| 维度 | 值 |
|------|----|
| 总参数 | 109B (Scout) / 400B (Maverick) |
| 激活参数 | **17B** (两者相同) |
| 架构 | **MoE** — 16 experts (Scout) / 128 experts (Maverick) |
| 上下文窗口 | **10M** (Scout) / 1M (Maverick) |
| 模态 | 原生多模态 (Text + Image → Text + Code) |
| 预训练 | ~40T tokens (Scout), ~22T (Maverick) |

### 核心创新
1. **极端 MoE 稀疏激活**: 109B 总参数仅激活 17B (15.6%)，单 H100 GPU 可部署 (Int4)
2. **iRope 长上下文**: 10M token 上下文窗口，通过 RoPE 位置编码创新 + L2 归一化实现
3. **Early Fusion 原生多模态**: 图像/视频在预训练阶段就与文本联合训练，非后接适配
4. **MoE Interleaving**: Scout 全 MoE 层，Maverick MoE 与 Dense 交替 (专家仅在一半层激活)
5. **Behemoth 蒸馏**: 288B 活跃参数的教师模型蒸馏到 Scout/Maverick

### NeoTrix 映射
| 创新 | NeoTrix 同构 | 差距 |
|------|-------------|------|
| MoE 稀疏路由 | GWT 注意力路由 + Rune Socketing 5 槽 | 需实现 **运行时稀疏激活调度器** (CapabilityBridge → 动态能力网) |
| 10M 上下文 | KVMem paged KV (A2) | 需实现 **KV Cache 分页虚拟化** + GPU/Host/NVMe 三级存储 |
| Early Fusion | PerceptionBridge (L2→L5) | 已有注意力门控桥，需增强**预训练级多模态融合** |
| 蒸馏 | SEAL Pipeline distillation (NT-MIND) | 已有蒸馏阶段，需增强**跨模型知识蒸馏**能力 |

---

## 5. DeepSeek V4.1 Flash (DeepSeek, Apr 2026)

### 架构参数
| 维度 | 值 |
|------|----|
| 总参数 | 522B (V4.1-Flash) / 284B (V4-Flash) |
| 激活参数 | 8-16B per token |
| 架构 | **MoE + Hybrid Attention (CSA + HCA)** |
| 上下文窗口 | **1M** |
| 特色 | 三级推理模式 + CED 架构 + Engram n-gram 记忆 |

### 核心创新
1. **混合注意力架构 (CSA + HCA)**:
   - **CSA (Compressed Sparse Attention)**: 沿序列维度压缩 KV Cache + DeepSeek Sparse Attention
   - **HCA (Heavily Compressed Attention)**: 更重压缩 + 密集注意力，仅 4 层实际压缩 KV，其余层读共享缓存
2. **Manifold-Constrained Hyper-Connections (mHC)**: 将残差映射约束在双随机矩阵流形 (Birkhoff polytope)，增强信号传播稳定性
3. **三档推理模式**: Non-think (快速) / Think High (逻辑分析) / Think Max (完整推理)
4. **Causal Encoder-Decoder (CED)**: 20 层编码器 + 20 层解码器，编码器最终隐藏状态投影构建解码器全局 KV Cache
5. **Engram n-gram 记忆**: 层 1 和 14 各拥有 ~384M 行 × 256 维哈希表，通过 4-gram 哈希查找写入残差流

### NeoTrix 映射
| 创新 | NeoTrix 同构 | 差距 |
|------|-------------|------|
| CSA + HCA 混合注意力 | GWT 分层注意力 + KVMem (A2) | 需实现**两级稀疏注意力**：局部滑动窗口 + 压缩 KV 共享缓存 |
| mHC 流形约束 | E8 Hexagram 几何约束 | 需研究**双随机矩阵约束**对信号稳定性的影响 |
| 三档推理 | Dual Specialization (Weapon Set I/II) | 需实现**运行时推理模式路由器**，根据任务复杂度动态选择 |
| CED 架构 | NT-CORE + NT-MIND 分层 | 需研究**编码器-解码器分离 KV Cache** 在能力网中的应用 |
| Engram 记忆 | NT-MEMORY KB + experience-tree | 需实现**哈希表辅助的快速 n-gram 记忆检索** |

---

## 6. Qwen3 (Alibaba, Apr 2025)

### 架构参数
| 维度 | 值 |
|------|----|
| 总参数 | 0.6B → 235B (dense) / 30B-A3B, 235B-A22B (MoE) |
| 激活参数 | MoE: 3B / 22B |
| 架构 | Dense + MoE 混合系列 |
| 上下文窗口 | 32K → 128K |
| 专家数 | 128 总专家 / 8 激活 (无共享专家) |
| 特色 | Thinking / Non-thinking 模式切换 + 119 语言 |

### 核心创新
1. **Thinking/Non-Thinking 统一框架**: 同一模型支持深度推理 (thinking) 和快速响应 (non-thinking)，无需切换模型
2. **QK-Norm 替代 QKV-bias**: 移除 QKV 偏置，引入 Query-Key 归一化，确保训练稳定性
3. **细粒度专家分割 + 无共享专家**: 128 专家 / 8 激活，排除共享专家 (shared experts)，鼓励专家特化
4. **全局批负载均衡损失**: 促进专家特化而非负载均衡
5. **Thinking Budget 机制**: 用户可分配推理计算资源，动态平衡延迟与性能
6. **蒸馏降成本**: 利用旗舰模型知识显著减少小模型训练资源

### NeoTrix 映射
| 创新 | NeoTrix 同构 | 差距 |
|------|-------------|------|
| Thinking 切换 | Dual Specialization + AttentionManager | 需实现**运行时推理模式切换器** (类似 DeepSeek V4 的三档) |
| QK-Norm | E8 Hexagram 注意力归一化 | 需研究 QK-Norm 对 NeoTrix 内部注意力机制的影响 |
| 无共享专家 MoE | CapabilityTree 能力节点 | 需研究**纯路由 MoE** 在能力网中的应用 (去除固定共享路径) |
| Thinking Budget | Axiom A2 + GWT salience | 需实现**token 预算动态分配器** |
| 跨语言 119 语 | NT-IO 多语言支持 | NeoTrix 本身语言无关，但需增强**多语言 tokenization** |

---

## 7. Mistral Large 3 (Mistral AI, Dec 2025)

### 架构参数
| 维度 | 值 |
|------|----|
| 总参数 | **675B** |
| 激活参数 | **41B** (39B LM + 2.5B Vision) |
| 架构 | **Granular MoE** + Vision Encoder |
| 上下文窗口 | 256K |
| 专家数 | 128 experts per layer |
| 许可 | Apache 2.0 |
| 训练 | 3000 H200 GPU 从零训练 |

### 核心创新
1. **Granular MoE (粒度化 MoE)**: 比标准 MoE 更细粒度的专家划分，41B 激活 / 675B 总计 (6.1% 激活率)
2. **Multi-Latent Attention (MLA)**: 潜空间多头注意力，压缩 KV Cache 到低维潜在空间
3. **Vision Encoder 集成**: 2.5B 视觉编码器 + 673B 语言模型，原生多模态
4. **Eagle 投机解码**: Draft model 加速推理，支持 NVFP4 量化
5. **Prefill/Decode 分离服务**: 支持长上下文高吞吐的分离式推理架构

### NeoTrix 映射
| 创新 | NeoTrix 同构 | 差距 |
|------|-------------|------|
| Granular MoE | Rune Socketing 5 槽 + CapabilityTree | 需实现**粒度化能力节点路由** (更细粒度的专家划分) |
| MLA 压缩注意力 | GWT + KVMem (A2) | 需研究**潜空间 KV 压缩**对 NeoTrix 注意力机制的影响 |
| 投机解码 | SEAL Pipeline 优化 | 需实现 **SpeculativeDecoding 能力节点** |
| 分离服务 | NT-IO 界面使徒 | 需研究**Prefill/Decode 分离架构**在能力网中的应用 |

---

## 8. Phi-4 Reasoning (Microsoft, Apr 2025)

### 架构参数
| 维度 | 值 |
|------|----|
| 总参数 | **14B** (Dense) |
| 架构 | Dense Decoder-only Transformer |
| 上下文窗口 | 32K |
| 训练数据 | 1.4M prompts + o3-mini 蒸馏 |
| 训练时间 | 2.5 天 / 32 H100 |
| 特色 | 小模型 × 大推理能力 |

### 核心创新
1. **数据质量 > 参数规模**: 14B 模型在 AIME 2025 达到 81.3% (Phi-4-reasoning-plus)，接近 671B DeepSeek-R1
2. **o3-mini 蒸馏推理链**: 使用 o3-mini 作为教师生成高质量推理 trace，SFT 训练
3. **<think> / </think> 推理标记**: 复用 base model 占位符作为推理标记，区分推理链与最终答案
4. **Pivotal Token Search DPO**: 新型 DPO 对构建方法，基于关键 token 搜索
5. **合成数据闭环**: 多 agent prompting + 自修订工作流 + 指令反转生成训练数据
6. **Phi-4-reasoning-plus**: 短期 RL 增强版，通过更长推理 trace 提升性能

### NeoTrix 映射
| 创新 | NeoTrix 同构 | 差距 |
|------|-------------|------|
| 数据质量优先 | SEAL Pipeline 数据工程 + exp-藏 (experience-tree) | 已有数据质量理念，需实现**合成数据生成管线** |
| 蒸馏推理链 | SEAL distillation + NT-MIND 蒸馏 | 已有蒸馏能力，需增强**跨模型推理 trace 蒸馏** |
| 推理标记 | ConsciousnessTree 6-stage | 需研究**显式推理标记**对 NT-CORE 推理链的影响 |
| 小模型大能力 | Constellation C0-C6 成熟度 | 证明**小节点可达到域级突破** (Notable Passive) |

---

## 9. Yi-Lightning (01.AI, Dec 2024)

### 架构参数
| 维度 | 值 |
|------|----|
| 总参数 | ~100B (MoE) |
| 架构 | Enhanced MoE |
| 上下文窗口 | 128K |
| 排名 | Chatbot Arena #6 (与 Grok-2 持平) |
| 特色 | 中国模型 LMSYS 最佳成绩 |

### 核心创新
1. **Fine-grained Expert Segmentation**: 细粒度专家分割 + 平衡路由策略
2. **Cross-layer KV Cache Sharing**: 跨层 KV Cache 共享设计，减少推理显存占用
3. **FP8 训练优化**: 针对 GPU 硬件特性的 FP8 量化训练
4. **RAISE 安全引擎**: 4 组件安全框架 (预训练/后训练/推理全覆盖)
5. **多阶段训练策略**: 分阶段数据混合优化，紧密集成训练进度

### NeoTrix 映射
| 创新 | NeoTrix 同构 | 差距 |
|------|-------------|------|
| KV Cache 跨层共享 | KVMem paged KV (A2) | 需实现**跨层 KV Cache 共享机制** |
| RAISE 安全 | NT-SHIELD 影卫 + NT-GOVERNANCE | 已有安全架构，需增强**推理时安全审计** |
| FP8 优化 | Constellation C3 benchmark | 需研究**低精度训练**对 NeoTrix 模型性能的影响 |
| 多阶段训练 | SEAL Pipeline 多阶段 | 已有多阶段进化，需增强**训练进度感知的数据混合** |

---

## 10. Grok 3 (xAI, Feb 2025)

### 架构参数
| 维度 | 值 |
|------|----|
| 总参数 | ~1.2T (推测，混合 Dense/MoE) |
| 架构 | Hybrid Dense/MoE + Neuro-Symbolic |
| 训练集群 | **100,000 H100 GPU** (Colossus) |
| 训练成本 | ~$420M (估计) |
| 上下文窗口 | 131K → 200K |
| 特色 | DeepSearch + Big Brain 模式 |

### 核心创新
1. **Neuro-Symbolic 集成**: Transformer + 符号推理模块混合，在时序推理达到 84% (vs GPT-4 79%)
2. **TTCS (Test-Time Compute at Scale)**: 推理时大规模计算分配，通过 RL 训练 chain-of-thought 过程
3. **DeepSearch**: 实时信息检索 + 推理融合，弥补静态训练数据不足
4. **Big Brain 模式**: 额外计算资源分配给复杂任务，优先推理质量而非速度
5. **对抗去偏**: 从中间表示层移除敏感模式，而非仅在最终输出过滤

### NeoTrix 映射
| 创新 | NeoTrix 同构 | 差距 |
|------|-------------|------|
| Neuro-Symbolic | E8 Hexagram (符号) + VSA HyperCube (向量) | 已有符号-向量混合，需增强**符号推理模块**的运行时集成 |
| TTCS | ThinkingBudget + GWT salience (A1, A2) | 需实现**推理时计算预算动态分配** |
| DeepSearch | NT-WORLD 探索 + NT-MEMORY 检索 | 已有世界感知+记忆检索，需增强**实时信息融合** |
| Big Brain | Dual Specialization 高级模式 | 需实现**BigBrainMode** — 高复杂度任务自动升级推理资源 |
| 对抗去偏 | NT-SHIELD 安全 + NT-GOVERNANCE 治理 | 需研究**中间表示层去偏**技术 |

---

## 横向收敛趋势 (Cross-Model Patterns)

### Trend 1: MoE 细粒度路由 — 从"大而全"到"稀疏激活"

| 模型 | 总参数 | 激活参数 | 激活率 | 专家数 |
|------|--------|----------|--------|--------|
| Llama 4 Scout | 109B | 17B | 15.6% | 16 |
| Llama 4 Maverick | 400B | 17B | 4.3% | 128 |
| DeepSeek V4-Flash | 284B | 13B | 4.6% | 256 |
| Mistral Large 3 | 675B | 41B | 6.1% | 128 |
| Qwen3-235B | 235B | 22B | 9.4% | 128 |

**NeoTrix 启示**: CapabilityTree 能力节点应支持**动态稀疏激活** — 根据任务复杂度选择性激活能力子集，而非全量加载。GWT salience 需要增加 **MoE 路由权重**。

### Trend 2: KV Cache 极限压缩 — 从 128K 到 10M

| 模型 | 上下文 | KV 压缩技术 |
|------|--------|-------------|
| GPT-4o | 128K | Undisclosed |
| Claude 3.5 | 200K | Undisclosed |
| Gemini 2.5 Pro | **1M** | Thinking Budget 控制 |
| Llama 4 Scout | **10M** | iRope + L2 归一化 |
| DeepSeek V4.1 | 1M | CSA + HCA + CED |
| Mistral Large 3 | 256K | Multi-Latent Attention |
| Yi-Lightning | 128K | Cross-layer KV Sharing |

**NeoTrix 启示**: Axiom A2 (Context as Scarce Resource) 已预见到这一趋势。需要实现 **KVMem paged KV virtualization** — GPU→Host→NVMe 三级 KV 存储，保持 GPU 内存恒定 (~35 GiB) 不随上下文增长。

### Trend 3: 推理时计算可控 — 从固定到动态分配

| 模型 | 推理模式 |
|------|---------|
| Gemini 2.5 | Thinking Budget + Deep Think |
| DeepSeek V4.1 | Non-think / Think High / Think Max |
| Qwen3 | Thinking / Non-thinking |
| Grok 3 | Standard / Think / Big Brain |
| Phi-4-reasoning | Reasoning trace + Summarization |

**NeoTrix 启示**: Dual Specialization (Weapon Set I/II) 应扩展为**多档推理模式路由器**。GWT salience 需要增加 **task_complexity_score** 维度，动态分配 NT-CORE 推理资源。

### Trend 4: 原生多模态 — 从"接驳"到"融合"

| 模型 | 多模态方式 |
|------|-----------|
| GPT-4o | 端到端 Omni-Modal |
| Llama 4 | Early Fusion 预训练 |
| Mistral Large 3 | Vision Encoder 集成 |
| DeepSeek V4.1 | ViT + 文本模型融合 |

**NeoTrix 启示**: PerceptionBridge (L2→L5) 已实现注意力门控桥接，但需要增强**预训练级多模态融合**能力，使 NT-WORLD 的多模态感知与 NT-CORE 的推理更紧密耦合。

### Trend 5: 小模型大能力 — 数据质量颠覆规模定律

| 模型 | 参数 | 对标 | 关键 |
|------|------|------|------|
| Phi-4-reasoning | 14B | DeepSeek-R1 (671B) | 数据质量 + 蒸馏 |
| Qwen3-30B-A3B | 30B (3B active) | Qwen2.5-72B | MoE + 数据策略 |
| Llama 4 Scout | 17B active | Llama 3.1-405B | MoE + 蒸馏 |

**NeoTrix 启示**: Constellation 成熟度证明 **Notable Passive 节点可达到域级突破**。NeoTrix 的 SEAL Pipeline distillation + experience-tree 吸收协议与此趋势一致。

---

## NeoTrix 能力差距清单 (Gap Analysis)

| # | 差距 | 优先级 | 来源趋势 | 建议能力节点 |
|---|------|--------|---------|-------------|
| G1 | 运行时稀疏激活调度 | **P0** | MoE 路由 | `nt_act::sparse_activation_scheduler` |
| G2 | KV Cache 分页虚拟化 | **P0** | 10M 上下文 | `nt_memory::kvmem_paged_kv` |
| G3 | 多档推理模式路由器 | **P1** | 推理时计算 | `nt_core::inference_mode_router` |
| G4 | 混合注意力压缩 (CSA+HCA) | **P1** | KV 压缩 | `nt_core::hybrid_attention` |
| G5 | 原生多模态融合增强 | **P1** | 多模态融合 | `nt_world::native_multimodal_fusion` |
| G6 | 推理 trace 蒸馏管线 | **P2** | 小模型大能力 | `nt_mind::reasoning_distillation` |
| G7 | Neuro-Symbolic 运行时集成 | **P2** | Grok 3 | `nt_core::neuro_symbolic_bridge` |
| G8 | 投机解码能力节点 | **P2** | Mistral | `nt_act::speculative_decoding` |
| G9 | 合成数据生成管线 | **P2** | Phi-4 | `nt_mind::synthetic_data_factory` |
| G10 | 实时信息融合 | **P3** | Grok DeepSearch | `nt_world::realtime_info_fusion` |

---

## 架构决策记录 (ADR-252)

### ADR-252-1: NeoTrix 是否需要原生 MoE 架构?

**决策**: 不采用原生 MoE，而是通过 **CapabilityBridge 动态稀疏激活** 实现等效效果。

**理由**:
- NeoTrix 是能力网架构 (Capability Network)，非单一神经网络
- GWT 注意力路由 ≈ MoE 路由器的抽象层
- CapabilityTree 的节点选择 ≈ 专家激活决策
- 优势：无需重训练，运行时动态调整

### ADR-252-2: 10M 上下文的 KV Cache 策略

**决策**: 实现 **KVMem paged KV virtualization**，GPU→Host→NVMe 三级存储。

**理由**:
- Axiom A2 已预见到 Context as Scarce Resource
- Llama 4 iRope 和 DeepSeek CSA/HCA 证明了工程可行性
- NeoTrix 需要支持长会话 (experience-tree 跨 cycle 吸收)
- 实现路径：扩展 `kv_cache_optimizer.rs`

### ADR-252-3: 推理模式路由的架构位置

**决策**: 在 **NT-CORE GWT salience** 中增加 `task_complexity_score` 维度，路由到不同推理深度。

**理由**:
- GWT 已是 NeoTrix 的注意力路由中心
- Dual Specialization 提供了双模式基础
- 扩展为多档 (Non-think/Think/Deep Think) 是自然演进
- 与 Axiom A1 (Cost-Aware Routing) 一致

---

## 参考来源

| 模型 | 关键来源 |
|------|---------|
| GPT-4o | OpenAI Hello GPT-4o (2024-05) |
| Claude 3.5 Sonnet | Anthropic Model Card Addendum (2024-06) |
| Gemini 2.5 Pro | Google DeepMind Blog (2025-03, 2026-01) |
| Llama 4 Scout | Meta AI Blog + HuggingFace Model Card (2025-04) |
| DeepSeek V4.1 Flash | DeepSeek Technical Report + NVIDIA NIM (2026-04) |
| Qwen3 | Alibaba Qwen3 Technical Report arXiv:2505.09388 (2025-05) |
| Mistral Large 3 | Mistral AI Blog + NVIDIA Model Card (2025-12) |
| Phi-4 Reasoning | Microsoft Research Technical Report (2025-04) |
| Yi-Lightning | 01.AI Technical Report arXiv:2412.01253 (2024-12) |
| Grok 3 | xAI Blog + PerplexityAI Analysis (2025-02) |
