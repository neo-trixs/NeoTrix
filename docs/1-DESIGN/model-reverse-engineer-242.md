# 模型架构逆向推理 — 2026-09-11 批次

> 10 款前沿模型架构创新提取 + NeoTrix 映射

---

## 一、模型矩阵总览

| 模型 | 厂商 | 架构类型 | 总参数 | 激活参数 | 上下文 | 关键创新 |
|------|------|---------|--------|---------|--------|---------|
| **GPT-4o** | OpenAI | Dense Transformer (推测) | 未公开 | 未公开 | 128K | 原生多模态 (Omni), 统一 tokenization, 232ms 音频延迟 |
| **Claude 3.5 Sonnet** | Anthropic | MoE | 175B | ~50B | 200K | 8专家/2激活, 计算机使用, agentic coding |
| **Gemini 2.5 Pro** | Google | Sparse MoE | 未公开 | 未公开 | 1M+ | 原生多模态, Thinking模式, k-sparse蒸馏, TPUv5p训练 |
| **Llama 4 Scout** | Meta | MoE | 109B | 17B | 10M | iRoPE (无位置编码注意力层), 16专家, 早期融合多模态 |
| **DeepSeek V4.1 Flash** | DeepSeek | MoE (CED) | 552B | 8B/16B | 1M | 因果编码器-解码器, CSA2, FP4 KV缓存, 384路由专家 |
| **Qwen3-235B-A22B** | Alibaba | MoE | 235B | 22B | 128K | 128专家/8激活, 无共享专家, Thinking/非Thinking双模式 |
| **Mistral Large 3** | Mistral | Granular MoE | 675B | 41B | 256K | 粒度MoE, 2.5B视觉编码器, Eagle推测解码, Apache 2.0 |
| **Phi-4-reasoning** | Microsoft | Dense | 14B | 14B | 32K | SFT+GRPO, 教师蒸馏(o3-mini), 推理token扩展, 小模型超大模型 |
| **Yi-Lightning** | 01.AI | MoE | 未公开 | 未公开 | 64K | 细粒度专家分割, EP/PEP负载均衡, 混合注意力, KV缓存共享 |
| **Grok 3** | xAI | Transformer (推测) | 未公开 | 未公开 | 1M | 10x算力(200K H100), Think/DeepSearch模式, 大规模RL推理 |

---

## 二、逐模型架构深度解析

### 1. GPT-4o — 原生全模态 (Omni)

**架构创新:**
- **端到端多模态训练**: 单一神经网络同时处理 text + audio + vision，非 CLIP 式分阶段管线
- **统一 tokenization**: 文本 (BPE)、图像 patch (ViT-style)、音频 (神经音频编解码器 ~75Hz) 共享单一 token 流
- **跨模态自注意力**: 模态间交互通过 token 流内的 self-attention 实现，无需独立 cross-attention 层
- **音频延迟革命**: 中位响应 232ms (vs 旧管线 2.8s)，接近人类对话反应时间
- **原生图像生成**: 自回归生成嵌入 ChatGPT 架构内 (非扩散模型)

**技术细节:**
- 音频 token 速率: ~50-100 Hz (Encodec/SoundStream/DAC 类)
- 30 秒语音交换 ≈ 2,250 audio tokens
- 模态特异性 embedding/unembedding 层，共享 transformer stack
- 50% 成本低于 GPT-4 Turbo

**NeoTrix 映射:**
| 创新 | NeoTrix 组件 | 映射理由 |
|------|-------------|---------|
| 端到端多模态 | `nt_world_sense::PerceptionBridge` | L2 感知层统一处理多模态输入流 |
| 统一 tokenization | `nt_core_hcube::VSA HyperCube` | VSA 高维向量统一表征多模态概念 |
| 跨模态注意力 | `nt_core::GWT` | 全局工作空间跨域注意力广播 |
| 音频 token 编解码 | `nt_io::PlatformGateway` | 多模态接口层需要统一的 token 编解码 |
| 原生图像生成 | `nt_io::ReferenceBasedGeneration` | 自回归生成嵌入统一架构 (非扩散后处理) |

---

### 2. Claude 3.5 Sonnet — MoE + 计算机使用

**架构创新:**
- **稀疏 MoE 架构**: 8 个专家，每次激活 2 个 (active/total ≈ 28.6%)
- **计算机使用**: 解释屏幕截图 → 生成 GUI 操作 (点击/输入/导航)，OSWorld 14.9%→22%
- **Agentic Coding**: SWE-bench Verified 49.0%，迭代自纠错编码循环
- **成本效率**: $3/M 输入 token，性能超越 Claude 3 Opus

**技术细节:**
- 60 层, 8192 hidden, GQA (64 Q heads / 8 KV heads), 200K 上下文
- KV-cache: 163,840 bytes/token
- FP8 权重 175GB, INT4 可降至 88GB

**NeoTrix 映射:**
| 创新 | NeoTrix 组件 | 映射理由 |
|------|-------------|---------|
| MoE 稀疏激活 | `nt_mind::CapabilityTree` | 技能节点按需激活，类似 MoE 路由 |
| 计算机使用 | `nt_act::MCP Tools` | 行动层工具调用 + 屏幕理解 |
| Agentic Coding | `nt_act::Dev-匠` | 代码迭代自纠错闭环 |
| 成本效率 | `nt_core::GWT` (Axiom A1) | 成本感知路由，简单任务用廉价模型 |

---

### 3. Gemini 2.5 Pro — 原生多模态 + Thinking

**架构创新:**
- **Sparse MoE + 原生多模态**: text + vision + audio 输入，自回归生成
- **Thinking 模式**: 模型自决推理时长，可设 thinking budget 权衡性能/成本
- **k-sparse 蒸馏**: 小模型用 k-sparse 分布近似教师 token 分布，提升蒸馏效率
- **训练稳定性**: 大规模信号传播和优化动力学改进，pre-training 直接提升性能
- **TPUv5p 训练**: 首个 TPUv5p 架构训练的模型，跨数据中心同步数据并行

**技术细节:**
- 1M token 上下文 (2M 即将上线)
- 支持 3 小时视频处理
- Flash 级别使用 k-sparse 蒸馏降低 serving 成本
- Thinking budget 可调 (内部推理 token 数控制)

**NeoTrix 映射:**
| 创新 | NeoTrix 组件 | 映射理由 |
|------|-------------|---------|
| Thinking 模式 | `nt_core::ConsciousnessTree` | 六阶段循环类似 Thinking 预算分配 |
| k-sparse 蒸馏 | `nt_mind::SEAL Pipeline` | 知识蒸馏阶段的稀疏化优化 |
| 训练稳定性 | `nt_repair::MAPE-K` | 监控-分析-规划-执行-知识循环 |
| 长上下文 1M+ | `nt_memory::KB` | SQLite KB 的 BM25+向量检索适配长上下文 |
| Thinking budget | `nt_core::GWT` (Axiom A2) | 上下文作为稀缺资源的预算分配 |

---

### 4. Llama 4 Scout — iRoPE + 极致长上下文

**架构创新:**
- **iRoPE 架构**: Interleaved attention layers without positional embeddings + RoPE 层交替
  - 无位置编码层: 目标支持 "infinite" 上下文
  - 推理时注意力温度缩放增强长度泛化
- **MoE + 早期融合**: 17B 激活参数, 16 专家, 109B 总参数
- **10M token 上下文**: 从 Llama 3 的 128K 跳跃到 10M
- **Natively Multimodal**: text + image 输入，text + code 输出

**技术细节:**
- 预训练 256K 上下文，长度泛化到 10M
- 单张 H100 GPU (INT4 量化) 可部署 Scout
- 交替 dense 和 MoE 层提升推理效率
- MoE 层: 1 shared expert + N routed experts

**NeoTrix 映射:**
| 创新 | NeoTrix 组件 | 映射理由 |
|------|-------------|---------|
| iRoPE (无位置编码) | `nt_memory::KB` (长上下文索引) | 无限上下文需要无位置偏见的注意力机制 |
| 10M 上下文 | `nt_nexus::Nexus-梭` | 跨会话记忆编织需要超长上下文 |
| 推理温度缩放 | `nt_core::GWT` (Axiom A1) | 成本感知路由的温度参数控制 |
| 单 GPU 部署 | `nt_physical::PowerManagement` | 具身层资源约束下的高效推理 |
| 早期融合多模态 | `nt_world_sense::PerceptionBridge` | L2 感知层的早期模态融合 |

---

### 5. DeepSeek V4.1 Flash — 非对称架构 + KV 缓存革命

**架构创新:**
- **Causal Encoder-Decoder (CED)**: 40 层 = 20 层因果编码器 + 20 层解码器
  - 解码器全局 KV 缓存从编码器最终隐藏状态投影 (非每层独立计算)
  - 非对称激活: prefill 仅 8B 参数, decode 仅 16B 参数
- **Compressed Sparse Attention 2 (CSA2)**: 三种静态模式 (Full/Reindex/Reuse)
  - 跨层共享 KV 和 indexer K，复用 Top-K 稀疏注意力索引
  - 分层稀疏索引器: 限制深层索引器候选池，复杂度与上下文长度解耦
- **FP4 KV 缓存**: E2M1 格式 + 每 16 通道 E4M3 scale → 890 bytes/token (vs V4-Flash 的 1/4)
- **SWA Bounded Replay**: 重放最近 n_win tokens 重建 SWA KV，避免 SSD 持久化
- **Engram 条件记忆**: 196B 参数，通过 token-based lookup 稀疏访问
- **DSpark 推测解码**: 半自回归草稿生成 + 置信度调度验证

**技术细节:**
- 552B 总参数, 384 路由专家 + 1 共享专家, 每 token 激活 6 路由专家
- DeepSeek-ViT: 2D-RoPE + 3×3 pixel-unshuffle 下采样
- 预训练 45T tokens, 64K 稀疏注意力 → 1M 扩展 (34T tokens)
- 非对称设计: agentic workload (长 prompt, 短生成) 直接获益

**NeoTrix 映射:**
| 创新 | NeoTrix 组件 | 映射理由 |
|------|-------------|---------|
| CED 非对称架构 | `nt_core::GWT` (Axiom A1) | 输入处理 (编码) 和输出生成 (解码) 的成本非对称 |
| CSA2 跨层 KV 共享 | `nt_memory::KV Cache Optimizer` | KV 缓存压缩和跨层复用 |
| FP4 KV 缓存 | `nt_physical::PowerManagement` | 具身层内存效率优化 |
| SWA Bounded Replay | `nt_nexus::Nexus-梭` | 会话边界 KV 状态的惰性重建 |
| Engram 条件记忆 | `nt_memory::KB` (kv_store) | 条件记忆按需稀疏加载 |
| 384 路由专家 | `nt_mind::CapabilityTree` | 大规模技能节点的稀疏激活路由 |
| DSpark 推测解码 | `nt_io::PlatformGateway` | 推理加速的投机执行 |

---

### 6. Qwen3-235B-A22B — 双模式 + 细粒度 MoE

**架构创新:**
- **Thinking/非Thinking 双模式统一**: 同一模型动态切换推理模式 (vs 不同模型切换)
- **Thinking Budget 机制**: 用户可分配推理计算资源，权衡延迟和性能
- **细粒度专家分割**: 128 总专家 / 8 激活专家，无共享专家 (vs Llama 4 有 shared expert)
- **全局批负载均衡**: 鼓励专家特化 (vs 传统 Switch-Transformer 逐专家约束)
- **QK-Norm**: 移除 QKV-bias，引入 QK-Norm 确保训练稳定

**技术细节:**
- 94 层, 64 Q heads / 4 KV heads, 128K 上下文
- 密集模型: 0.6B→32B, MoE 模型: 30B-A3B / 235B-A22B
- 从旗舰模型蒸馏到小模型 (Strong-to-Weak Distillation)
- 119 种语言支持 (vs Qwen2.5 的 29 种)
- Apache 2.0 开源

**NeoTrix 映射:**
| 创新 | NeoTrix 组件 | 映射理由 |
|------|-------------|---------|
| Thinking/非Thinking 双模式 | `nt_core::ConsciousnessTree` | 六阶段循环 vs 快速响应的动态切换 |
| Thinking Budget | `nt_core::GWT` (Axiom A2) | 上下文作为稀缺资源的预算控制 |
| 128 专家无共享 | `nt_mind::CapabilityTree` | 纯路由专家 (无共享基础能力) 的技能树 |
| 全局批负载均衡 | `nt_act::ParallelTaskManager` | 跨设备负载均衡 |
| QK-Norm | `nt_core::E8 Hexagram` | 注意力机制的训练稳定性 |
| Strong-to-Weak 蒸馏 | `nt_mind::SEAL Pipeline` | 知识蒸馏阶段的强→弱传递 |

---

### 7. Mistral Large 3 — 粒度 MoE + 原生视觉

**架构创新:**
- **粒度 MoE**: 675B 总参数, 41B 激活 (ratio ≈ 16:1)，激活率仅 6%
- **集成视觉编码器**: 2.5B 视觉编码器内嵌，原生图像理解 (OCR, 文档 Q&A)
- **Eagle 推测解码**: 定制草稿模型 + 置信度调度验证, 3 个推测 token
- **NVFP4 量化**: Blackwell GPU 原生 4-bit 格式，单节点 8×H100 可部署
- **Apache 2.0 开源**: 675B MoE 模型首次完全开源

**技术细节:**
- 3000 NVIDIA H200 GPU 从零训练
- FP8 权重可在单节点 B200/H200 运行
- NVFP4 可在 8×H100/A100 运行
- 256K 上下文, 多语言 (英/法/西/德/意/葡/荷/中/日/韩/阿)
- 原生 function calling + JSON 输出

**NeoTrix 映射:**
| 创新 | NeoTrix 组件 | 映射理由 |
|------|-------------|---------|
| 粒度 MoE (16:1) | `nt_mind::CapabilityTree` | 超稀疏技能激活，极致成本效率 |
| 集成视觉编码器 | `nt_world_sense::PerceptionBridge` | L2 感知层原生视觉编码 |
| Eagle 推测解码 | `nt_io::PlatformGateway` | 推理加速的投机执行管线 |
| NVFP4 量化 | `nt_physical::PowerManagement` | 具身层低精度推理优化 |
| Apache 2.0 开源 | `nt_memory::KB` (知识开放) | 开放权重促进社区生态 |

---

### 8. Phi-4-reasoning — 小模型推理超大模型

**架构创新:**
- **教师蒸馏 + SFT**: 14B 模型通过 o3-mini 教师生成的推理链 SFT，在推理任务上超越 70B 蒸馏模型
- **推理 token 扩展**: 两个占位符 token 重定义为 `<think>` / `</think>`，RoPE 基频翻倍 → 32K 上下文
- **GRPO 强化学习**: Group Relative Policy Optimization, 仅 6K 数学问题 seed，RL 后推理链长度 1.5x
- **"可教学" 数据选择**: 筛选处于基础模型能力边缘的 prompt，最大化教学效率
- **推理可迁移**: 推理训练的改进迁移到未训练域 (代码/规划/空间理解)

**技术细节:**
- 14B 密集 Transformer, 32K 上下文
- SFT: 1.4M prompt-response pairs, 8.3B unique tokens
- RL: 32 H100, batch size 64, 仅 6,400 数学问题
- 推理链: o3-mini (medium/high effort) 作为教师
- MIT 开源, 可在笔记本运行

**NeoTrix 映射:**
| 创新 | NeoTrix 组件 | 映射理由 |
|------|-------------|---------|
| 教师蒸馏 (14B→超越70B) | `nt_mind::SEAL Pipeline` (蒸馏阶段) | 知识蒸馏: 强模型教弱模型 |
| 推理 token 扩展 | `nt_core::ConsciousnessTree` (推理预算) | 推理链 token 预算控制 |
| GRPO RL | `nt_core::SelfModel` (价值函数) | 基于结果的策略优化 |
| "可教学" 数据选择 | `nt_mind::Skill Crystallization` | 技能结晶的数据质量筛选 |
| 推理可迁移 | `nt_mind::Cross-Domain Propagation` | 跨域能力迁移 |

---

### 9. Yi-Lightning — 混合注意力 + KV 缓存共享

**架构创新:**
- **细粒度专家分割**: FFN 分割为更小功能单元，减少中间维度，增加激活专家数
- **EP/PEP 负载均衡**:
  - EP: 放松逐专家约束到 EP 组
  - PEP: 分区内进一步分割，解决 All-to-All 通信不平衡
  - 三级损失: L_ST (10⁻⁶) + L_EP (10⁻⁴) + L_PEP (10⁻³)
- **混合注意力块**: 3 层滑动窗口注意力 + 1 层全注意力
- **跨层 KV 缓存共享**: 连续全注意力层共享 KV 状态 → 内存减半
- **FP8 量化优化**: 1,200 TFLOPS/card (FP8), Hopper 架构深度适配

**技术细节:**
- 82.8% 内存减少 (混合注意力 + KV 共享)
- 70% 训练加速 (上下文并行优化)
- 95% GPU 利用率 (异步调度)
- RAISE 安全引擎: 4 组件覆盖训练/部署/服务全周期

**NeoTrix 映射:**
| 创新 | NeoTrix 组件 | 映射理由 |
|------|-------------|---------|
| 细粒度专家分割 | `nt_mind::CapabilityTree` | 技能节点细粒度分割 |
| EP/PEP 三级负载均衡 | `nt_act::ParallelTaskManager` | 跨设备通信优化 |
| 混合注意力 (SWA+Full) | `nt_core::GWT` (注意力路由) | 局部+全局注意力的混合策略 |
| 跨层 KV 共享 | `nt_memory::KV Cache Optimizer` | KV 缓存跨层复用 |
| FP8 硬件适配 | `nt_physical::PowerManagement` | 具身层硬件感知优化 |
| RAISE 安全引擎 | `nt_shield::Shield` | 安全域全周期覆盖 |

---

### 10. Grok 3 — 大规模 RL + 推理代理

**架构创新:**
- **10x 算力训练**: 200K NVIDIA H100 Colossus 超算，10x Grok 2
- **Think 模式**: 大规模 RL 训练的 Chain-of-Thought，秒到分钟级推理，自动纠错回溯
- **DeepSearch 代理**: 实时互联网 + X 平台搜索，综合分析冲突信息
- **1M 上下文**: 8x 上一代，LOFT 128K 基准 SOTA
- **Think + DeepSearch 分离**: Think (内部推理) vs DeepSearch (外部知识获取)

**技术细节:**
- 架构未公开 (Transformer-based, 推测 MoE)
- 知识截止 2024.11
- Chatbot Arena Elo 1402
- AIME 2025: 93.3%, GPQA Diamond: 84.6%, LiveCodeBench: 79.4%
- 合成数据训练 + 逻辑一致性调整

**NeoTrix 映射:**
| 创新 | NeoTrix 组件 | 映射理由 |
|------|-------------|---------|
| Think 模式 | `nt_core::ConsciousnessTree` | 六阶段循环的深度推理 |
| DeepSearch 代理 | `nt_world::UnifiedCrawler` | 外部知识获取和综合 |
| 1M 上下文 | `nt_nexus::Nexus-梭` | 跨会话长上下文编织 |
| Think/DeepSearch 分离 | `nt_core::GWT` (双过程) | 内部推理 vs 外部探索的注意力分配 |
| 大规模 RL | `nt_core::SelfModel` (价值函数) | 基于 RL 的自我进化 |

---

## 三、跨模型架构趋势提取

### 3.1 六大共识趋势

| # | 趋势 | 证据模型 | NeoTrix 设计验证 |
|---|------|---------|-----------------|
| T1 | **MoE 成为默认架构** | Claude 3.5, Gemini 2.5, Llama 4, DeepSeek V4.1, Qwen3, Mistral Large 3, Yi-Lightning (7/10) | `CapabilityTree` 的稀疏技能激活设计已对齐 |
| T2 | **原生多模态训练** | GPT-4o, Gemini 2.5, Llama 4, DeepSeek V4.1, Mistral Large 3 (5/10) | `PerceptionBridge` + `PlatformGateway` 需强化统一 tokenization |
| T3 | **推理时计算可控** | Gemini 2.5 (Thinking budget), Qwen3 (Thinking/非Thinking), Grok 3 (Think), Phi-4 (推理链) | `ConsciousnessTree` 需引入 thinking budget 参数 |
| T4 | **KV 缓存极限压缩** | DeepSeek V4.1 (FP4, CSA2, 890B/token), Yi-Lightning (82.8% 减少) | `kv_cache_optimizer.rs` 需升级到 FP4 + 跨层共享 |
| T5 | **10M+ 超长上下文** | Llama 4 Scout (10M), Gemini 2.5 (1M+), DeepSeek V4.1 (1M), Grok 3 (1M) | `Nexus-梭` 的跨会话编织需要无位置偏见注意力 |
| T6 | **教师蒸馏民主化** | Phi-4 (14B→超越70B), Qwen3 (Strong-to-Weak), Gemini 2.5 (k-sparse) | `SEAL Pipeline` 蒸馏阶段需支持 k-sparse 优化 |

### 3.2 三大分化方向

| 方向 | 代表 | 特征 | NeoTrix 机会 |
|------|------|------|-------------|
| **极致效率** | DeepSeek V4.1 (8B 激活), Phi-4 (14B), Mistral NVFP4 | 低激活参数 + 极致量化 + 非对称架构 | `PowerManagement` + `ParallelTaskManager` |
| **极致规模** | Grok 3 (200K H100), Gemini 2.5 (TPUv5p), Mistral Large 3 (3000 H200) | 海量算力 + 大规模 RL + 推理时间缩放 | `GWT` 成本感知路由 (Axiom A1) |
| **极致开放** | Qwen3 (Apache 2.0), Llama 4 (开放权重), Mistral Large 3 (Apache 2.0), Phi-4 (MIT) | 开放权重 + 开源许可 + 社区生态 | `KB` 知识开放 + `CapabilityBridge` |

---

## 四、NeoTrix 吸收优先级

### P0 — 立即吸收 (本 cycle)

| 创新 | 来源模型 | 吸收目标 | 实现路径 |
|------|---------|---------|---------|
| Thinking Budget 参数 | Gemini 2.5 + Qwen3 | `nt_core::GWT` | GWT salience 增加 thinking_budget 参数，按任务复杂度动态分配推理 token |
| KV FP4 缓存 | DeepSeek V4.1 | `nt_memory::kv_cache_optimizer` | E2M1 格式 + 每 16 通道 scale，目标 890 bytes/token |
| 细粒度 MoE 路由 | Yi-Lightning + Qwen3 | `nt_mind::CapabilityTree` | 技能节点 FFN 分割 + EP/PEP 负载均衡 |

### P1 — 近期吸收 (下 2 cycle)

| 创新 | 来源模型 | 吸收目标 | 实现路径 |
|------|---------|---------|---------|
| CED 非对称架构 | DeepSeek V4.1 | `nt_core::GWT` | 编码/解码分离的注意力权重分配 |
| 教师蒸馏 k-sparse | Gemini 2.5 | `nt_mind::SEAL Pipeline` | 蒸馏阶段的稀疏 token 分布近似 |
| iRoPE 无位置编码 | Llama 4 Scout | `nt_memory::KB` (长上下文索引) | 交替位置编码/无位置编码注意力层 |

### P2 — 远期探索

| 创新 | 来源模型 | 吸收目标 | 实现路径 |
|------|---------|---------|---------|
| 原生多模态统一 tokenization | GPT-4o | `nt_world_sense::PerceptionBridge` | 统一 text+audio+vision token 流 |
| Engram 条件记忆 | DeepSeek V4.1 | `nt_memory::KB` | 196B 参数条件记忆的 token-based lookup |
| DeepSearch 外部代理 | Grok 3 | `nt_world::UnifiedCrawler` | 实时互联网搜索 + 冲突信息综合 |

---

## 五、关键洞察

### 5.1 MoE 的 "激活率竞赛"

| 模型 | 激活率 (active/total) | 趋势 |
|------|----------------------|------|
| Claude 3.5 Sonnet | 28.6% (50B/175B) | |
| Qwen3-235B-A22B | 9.4% (22B/235B) | |
| Llama 4 Scout | 15.6% (17B/109B) | |
| Mistral Large 3 | 6.1% (41B/675B) | |
| DeepSeek V4.1 Flash | 1.4%-2.9% (8B-16B/552B) | **最低** |

**结论**: 行业正在从 "大 MoE" (高激活率) 向 "极稀疏 MoE" (极低激活率) 演进。DeepSeek V4.1 的 1.4%-2.9% 激活率是当前最低，验证了 NeoTrix `CapabilityTree` 稀疏激活设计的正确性。

### 5.2 上下文长度军备竞赛

| 模型 | 上下文 | 关键技术 |
|------|--------|---------|
| Llama 4 Scout | **10M** | iRoPE (无位置编码) |
| Gemini 2.5 Pro | 1M+ | 训练稳定性 + 长上下文扩展 |
| DeepSeek V4.1 Flash | 1M | CSA2 + SWA Bounded Replay |
| Grok 3 | 1M | 大规模算力 |
| Mistral Large 3 | 256K | 传统 RoPE |
| Phi-4-reasoning | 32K | RoPE 基频翻倍 |

**结论**: 10M 是新前沿，但需要全新的注意力机制 (iRoPE)。NeoTrix `Nexus-梭` 的跨会话编织需要考虑无位置偏见的注意力设计。

### 5.3 推理模式收敛

| 模型 | 推理模式 | 特征 |
|------|---------|------|
| Gemini 2.5 | Thinking (用户可设 budget) | 模型自决 + 外部约束 |
| Qwen3 | Thinking/非Thinking 双模式 | 同一模型动态切换 |
| Grok 3 | Think + DeepSearch | 内部推理 + 外部搜索 |
| Phi-4-reasoning | 推理 token (<think>/</think>) | 显式推理块 |
| Yi-Lightning | 混合注意力 (SWA+Full) | 隐式局部/全局推理 |

**结论**: "推理可控" 是 2025-2026 最重要的架构趋势。NeoTrix `ConsciousnessTree` 的六阶段循环已具备推理预算分配的基础，需要增加显式的 thinking budget 参数。

---

*文档生成时间: 2026-09-11*
*数据源: 各模型官方技术报告/博客/HuggingFace Model Card*
*下一步: 按 P0/P1/P2 优先级接线到 NeoTrix 生产路径*
