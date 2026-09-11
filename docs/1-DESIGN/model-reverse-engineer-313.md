# 逆向推理: 10大前沿模型架构分析

> Cycle 313 | 2026-09-11 | 10 Models × Architecture × NeoTrix Mapping

---

## 总览矩阵

| # | 模型 | 厂商 | 架构类型 | 总参数 | 激活参数 | 上下文 | 核心创新 |
|---|------|------|----------|--------|----------|--------|----------|
| 1 | GPT-4o | OpenAI | Dense/MoE(未公开) | ~1.8T(推测) | 未公开 | 128K | 统一多模态端到端 |
| 2 | Claude 3.5 Sonnet | Anthropic | Dense Transformer | 未公开 | 未公开 | 200K | Constitutional AI + 计算机使用 |
| 3 | Gemini 2.5 Pro | Google DeepMind | Sparse MoE | 未公开 | 未公开 | 1M | Thinking模式 + 原生多模态 |
| 4 | Llama 4 Scout | Meta | Sparse MoE | 109B | 17B | 10M | iRoPE + Early Fusion |
| 5 | DeepSeek V4.1 Flash | DeepSeek | CED + Sparse MoE | 552B+196B Engram | 8B/16B | 1M | CSA2 + FP4 KV缓存 |
| 6 | Qwen 3 | Alibaba | Dense + Sparse MoE | 235B(MoE旗舰) | 22B | 128K | 双模式 + Thinking Budget |
| 7 | Mistral Large 3 | Mistral AI | Granular MoE | 675B | 41B | 256K | 细粒度MoE + Apache 2.0 |
| 8 | Phi-4 reasoning | Microsoft | Dense Transformer | 14B | 14B | 32K | 数据策展蒸馏 + GRPO |
| 9 | Yi-Lightning | 01.AI | Enhanced MoE | 未公开 | 未公开 | 64K | 细粒度专家分割 + PEP路由 |
| 10 | Grok 3 | xAI | Transformer + MoE | ~1.5T(推测) | 未公开 | 1M | RL规模化推理 + DeepSearch |

---

## 1. GPT-4o (OpenAI, 2024-05)

### 架构逆向

**核心声明**: 端到端自回归全模态模型, 单一神经网络处理文本/图像/音频/视频的所有输入输出.

**推断架构**:
- **统一Token流**: BPE文本token + ViT风格图像patch token + 神经音频编解码器(Encodec/SoundStream类)token
- **单一Transformer堆栈**: 跨模态自注意力, 无独立跨模态层
- **模态特定嵌入/解嵌入层**: 输入嵌入表按模态分离, 输出头按上下文产生对应模态token
- **训练**: 交织多模态数据(转录对话+音频+字幕图像+视频)端到端联合训练
- **图像解码**: 扩散头(diffusion-based head), 非VAR架构, 使用连续视觉tokenizer

**性能指标**:
- 音频延迟: 中位232ms, 平均320ms (对比旧管线2.8s, 12x提升)
- 词汇表: 199,997 tokens (GPT-4的2倍)
- 上下文: 128K tokens
- 定价: $2.50/$10 per M tokens (GPT-4 Turbo的50%)

### 关键创新

| 创新 | 描述 |
|------|------|
| **端到端多模态** | 消除ASR→LLM→TTS三段管线, 单模型处理全模态 |
| **统一Token化** | 文本/图像/音频共享同一token空间, 通过自注意力学习跨模态关系 |
| **扩散图像解码** | AR+扩散混合架构, 扩散头生成高质量图像 |
| **大幅词汇扩展** | 200K词汇表, 非英语语言效率提升6x (中文: 12 tokens→2 tokens) |

### NeoTrix 映射

| NeoTrix组件 | 映射 | 优先级 |
|-------------|------|--------|
| **GWT注意力路由** | 统一token流的跨模态注意力 = GWT广播机制的底层实现 | P0 |
| **NT-WORLD感知层** | 端到端多模态 = PerceptionBridge的终极形态, 消除模态转换瓶颈 | P1 |
| **E8推理引擎** | 扩散+AR混合解码 = 双过程推理(快/慢)的生成范式 | P1 |
| **NT-IO界面层** | 232ms延迟目标 = IO层实时交互设计的基准线 | P2 |

---

## 2. Claude 3.5 Sonnet (Anthropic, 2024-06)

### 架构逆向

**已知信息**: Dense Transformer, Constitutional AI训练, 未公开参数量.

**推断架构**:
- **基础**: 大规模Dense Transformer (推测200B+参数)
- **训练方法**: 无监督学习 + Constitutional AI (CAI) — 基于原则的自我修正
- **多模态**: 文本+图像输入 (早期融合), 无原生音频
- **对齐**: HHH (Helpful, Honest, Harmless) + RSP (Responsible Scaling Policy)
- **推理增强**: 计算机使用能力 (截图理解→GUI操作)

**性能指标**:
- 上下文: 200K tokens
- 速度: Claude 3 Opus的2x
- 成本: $3/$15 per M tokens
- SWE-bench Verified: 49.0% (升级版78% agentic coding)
- ASL-2安全等级

### 关键创新

| 创新 | 描述 |
|------|------|
| **Constitutional AI** | 基于原则的自我监督训练, 减少人工标注依赖 |
| **计算机使用** | 截图→理解→生成GUI操作, 开创agent交互新范式 |
| **Agentic Coding** | 理解代码库→实现PR→自纠正循环, 64%→78%问题解决率 |
| **负责任扩展策略** | ASL分级安全框架, 按能力级别递增安全措施 |

### NeoTrix 映射

| NeoTrix组件 | 映射 | 优先级 |
|-------------|------|--------|
| **NT-SHIELD安全域** | Constitutional AI = NT-SHIELD的治理宪法同构; ASL分级 = RiskAssessor分级框架 | P0 |
| **NT-ACT行动域** | 计算机使用 = NT-ACT的GUI自动化工具; Agentic coding = dev-implementer技能 | P0 |
| **ConsciousnessTree** | HHH原则 = ConsciousnessTree的道德分支约束条件 | P1 |
| **NT-MIND进化域** | 自我修正循环 = SEAL pipeline的自校准阶段 | P2 |

---

## 3. Gemini 2.5 Pro (Google DeepMind, 2025-06)

### 架构逆向

**已知信息**: Sparse MoE Transformer, 原生多模态 (文本/视觉/音频), TPUv5p训练.

**详细架构**:
- **核心**: Sparse MoE Transformer, 每token激活参数子集
- **训练基础设施**: TPUv5p, 8960-chip pods, 跨数据中心同步数据并行
- **弹性训练**: Slice-Granularity Elasticity — 故障时自动切片降级, 97%吞吐保持
- **SDC检测**: Split-Phase SDC Detection — 轻量确定性重放, 分钟级硬件故障定位
- **蒸馏**: k-sparse分布近似教师模型输出, Flash系列使用

**Thinking模型**:
- **可控Thinking Budget**: 用户设置内部计算token上限, 性能随budget提升
- **原生多模态Thinking**: 图像/文本/视频/音频输入均可触发thinking
- **模型自决**: 模型自行决定思考时长

**性能指标**:
- 上下文: 1M tokens (2M即将推出)
- 视频理解: 最长3小时视频
- SWE-bench: 63.8% (定制agent)
- 训练数据截止: 2025年1月

### 关键创新

| 创新 | 描述 |
|------|------|
| **Thinking Budget可控** | 用户指定推理计算量, 性能-成本精确权衡 |
| **k-sparse蒸馏** | 用top-k稀疏分布近似教师输出, 蒸馏存储开销降低k倍 |
| **弹性训练容错** | 片粒度弹性 + 分阶段SDC检测, 大规模训练中断恢复秒级 |
| **3小时视频理解** | 突破视频长度限制, 支持完整电影/课程分析 |

### NeoTrix 映射

| NeoTrix组件 | 映射 | 优先级 |
|-------------|------|--------|
| **GWT注意力路由** | Thinking Budget = GWT salience的显式控制旋钮; 成本感知路由(A1公理) | P0 |
| **E8推理引擎** | Thinking模式 = E8六阶段推理循环的token级实现 | P0 |
| **NT-MEMORY知识域** | 1M上下文 = KVMem paged KV的对标场景; 弹性 = KB弹性伸缩 | P1 |
| **SEAL Pipeline** | Thinking budget = SEAL Phase耗时控制; 蒸馏 = NT-MIND知识蒸馏 | P1 |

---

## 4. Llama 4 Scout (Meta, 2025-04)

### 架构逆向

**已知信息**: 17B激活参数, 16专家MoE, 109B总参数, 开源.

**详细架构**:
- **MoE设计**: 交替Dense层和MoE层; 每token发送到1个共享专家 + 1个路由专家
- **专家配置**: 16路由专家 + 1共享专家 (Scout); 128路由专家 + 1共享专家 (Maverick)
- **iRoPE架构**: Interleaved attention layers — 部分层无位置编码 + RoPE层交替
  - "i" = infinite context目标
  - 推理时温度缩放注意力增强长度泛化
- **Early Fusion**: 文本+图像在输入层即融合, 非后期拼接
- **训练**: ~40T tokens, 200+语言, FP8精度, 390 TFLOPs/GPU
- **MetaP**: 自动选择每层学习率和初始化尺度, 跨batch/深度/token预算泛化

**性能指标**:
- 上下文: 10M tokens (业界最长)
- 预训练: 40T tokens (Scout), 22T tokens (Maverick)
- 部署: 单H100 GPU (Int4量化)

### 关键创新

| 创新 | 描述 |
|------|------|
| **iRoPE** | 交替注意力+RoPE, 部分层无位置编码, 实现10M超长上下文 |
| **10M上下文** | 从128K→10M, 78x提升, 支持多文档摘要/全代码库推理 |
| **Early Fusion** | 多模态在输入层融合, 避免后期融合的信息瓶颈 |
| **MetaP超参选择** | 自动化超参数选择, 跨配置泛化, 减少人工调参 |

### NeoTrix 映射

| NeoTrix组件 | 映射 | 优先级 |
|-------------|------|--------|
| **NT-NEXUS跨会话记忆** | 10M上下文 = 跨会话记忆的硬件级支撑; iRoPE = 无限记忆架构 | P0 |
| **KVMem分页KV** | 10M上下文需要分页KV虚拟化, Llama 4验证了需求真实性 | P0 |
| **NT-WORLD感知层** | Early Fusion = PerceptionBridge的输入融合模式 | P1 |
| **E8推理引擎** | iRoPE交替注意力 = E8长程推理的位置编码策略 | P2 |

---

## 5. DeepSeek V4.1 Flash (DeepSeek, 2026-09)

### 架构逆向

**已知信息**: 552B MoE + 196B Engram, CED架构, 1M上下文, MIT开源.

**详细架构**:
- **Causal Encoder-Decoder (CED)**: 40层Transformer = 20层因果编码器 + 20层解码器
  - 解码器全局KV从编码器最终隐藏状态投影, 非每层独立计算
  - **非对称激活**: 8B/token(prefill) vs 16B/token(decode)
- **CSA2 (Compressed Sparse Attention 2)**: 三种静态模式
  - **Full**: 计算主KV+索引器K+Top-K位置
  - **Reindex**: 复用主KV+索引器K, 重新计算自身Top-K
  - **Reuse**: 复用主KV+索引器K+Top-K索引
  - 层级稀疏索引器: 首Full层选Top-512, 后续层从缩减池中选
- **FP4 KV缓存**: E2M1格式, 每16通道1个E4M3缩放因子 → 890 bytes/token
- **SWA Bounded Replay**: 重放最近n_win个token重建SWA KV, 避免SSD持久化
- **Engram条件记忆**: 196B参数, token n-gram(2/3/4-gram), 8哈希头, 上下文感知门控
- **DSpark推测解码**: 3个Transformer块, 128-token滑动窗口, 5位置并行提议

**MoE配置**: 1共享专家 + 384路由专家, 每token激活6路由专家

**性能指标**:
- KV缓存: 890 bytes/token (V4 Flash的1/4, V1的1/437)
- 持久化KV: V4 Flash的1/8
- 上下文: 1M tokens
- 训练: 45T multimodal tokens
- 推理effort: 1-100可调

### 关键创新

| 创新 | 描述 |
|------|------|
| **CED非对称架构** | 编码器一次投影KV到解码器, prefill仅8B激活, 大幅降低长输入成本 |
| **CSA2跨层共享** | KV+索引在Full/Reindex/Reuse三层间共享, KV足迹压缩4x |
| **FP4 KV量化** | E2M1格式890 bytes/token, 推动KV缓存进入4-bit时代 |
| **Engram条件记忆** | 196B参数稀疏访问, 基于token n-gram查找, 分离记忆与计算 |
| **DSpark推测解码** | 置信度调度验证长度, 系统负载感知的推测解码 |

### NeoTrix 映射

| NeoTrix组件 | 映射 | 优先级 |
|-------------|------|--------|
| **KVMem分页KV** | CSA2+FP4 = KV缓存压缩的工程标杆; CED = prefill/decode分离优化 | P0 |
| **NT-MEMORY知识域** | Engram = KB条件记忆的同构设计; n-gram查找 = KB检索优化 | P0 |
| **GWT注意力路由** | CSA2三层模式 = GWT注意力的硬件级实现; 成本感知(A1公理) | P0 |
| **NT-NEXUS跨会话记忆** | Engram = 跨会话模式记忆的token级实现 | P1 |
| **SEAL Pipeline** | DSpark推测解码 = SEAL阶段加速的推理优化 | P2 |

---

## 6. Qwen 3 (Alibaba, 2025-05)

### 架构逆向

**已知信息**: Dense + MoE双系列, 0.6B-235B, Apache 2.0开源.

**详细架构**:
- **Dense系列**: 类Qwen2.5架构
  - GQA (Grouped Query Attention) + SwiGLU + RoPE + RMSNorm
  - **移除QKV-bias**, 引入**QK-Norm**确保训练稳定
- **MoE系列**: 
  - 128总专家, 8激活专家/token (无共享专家)
  - **全局批量负载均衡损失** (非per-expert) 鼓励专家特化
  - 细粒度专家分割 (受DeepSeekMoE启发)
- **双模式统一框架**:
  - **Thinking模式**: 多步推理, 类o1
  - **Non-Thinking模式**: 快速响应, 类GPT-4o
  - 模型根据查询/模板动态切换
- **Thinking Budget**: 用户可分配推理计算资源
- **4阶段训练Pipeline**:
  1. 长CoT冷启动
  2. 推理RL (rule-based rewards)
  3. Thinking模式融合
  4. 通用RL (20+任务)
- **Strong-to-Weak蒸馏**: 旗舰→小模型知识传递

**性能指标**:
- Qwen3-235B-A22B: 235B总参/22B激活, 128K上下文
- Qwen3-30B-A3B: 30B总参/3B激活, 超越QwQ-32B
- 119种语言 (Qwen2.5的4x)
- 词汇: 151,669 BBPE tokens

### 关键创新

| 创新 | 描述 |
|------|------|
| **双模式统一** | Thinking+Non-Thinking在同一模型, 消除模型切换需求 |
| **Thinking Budget** | 用户控制推理深度, 性能平滑可预测 |
| **QK-Norm** | 替代QKV-bias, 训练稳定性显著提升 |
| **全局批量负载均衡** | 鼓励专家特化而非均匀分配, 提升MoE效率 |
| **4阶段训练** | 冷启动→推理RL→模式融合→通用RL, 渐进式能力构建 |

### NeoTrix 映射

| NeoTrix组件 | 映射 | 优先级 |
|-------------|------|--------|
| **GWT注意力路由** | 双模式 = GWT快/慢通道路由; Thinking Budget = salience权重控制 | P0 |
| **E8推理引擎** | 4阶段训练 = SEAL pipeline的训练范式; 推理RL = 自我进化 | P0 |
| **NT-MIND进化域** | Strong-to-Weak蒸馏 = NT-MIND知识蒸馏; 通用RL = 自我进化RL | P1 |
| **NT-SHIELD安全域** | QK-Norm = 训练稳定性 = 安全性基础 | P2 |

---

## 7. Mistral Large 3 (Mistral AI, 2025-12)

### 架构逆向

**已知信息**: Granular MoE, 675B总参/41B激活, 2.5B视觉编码器, Apache 2.0.

**详细架构**:
- **Granular MoE**: 675B总参, 41B活跃 (16:1稀疏比)
- **视觉编码器**: 2.5B参数原生集成, 非外部适配器
- **训练**: 3000x NVIDIA H200 GPU从头训练
- **低精度支持**: NVFP4 (Blackwell), FP8/FP16
- **部署**: 单8×H100节点可运行 (FP8), vLLM+TensorRT-LLM
- **Ministral阶梯**: 3B/8B/14B同许可证同工具链

**性能指标**:
- 上下文: 256K tokens
- 定价: $2/$6 per M tokens (API)
- 多语言: 12+语言原生支持
- 许可: Apache 2.0 (含专利授权)

### 关键创新

| 创新 | 描述 |
|------|------|
| **Granular MoE** | 极致稀疏比(16:1), 675B存储容量+41B推理成本 |
| **原生视觉融合** | 2.5B视觉编码器内置, 布局感知文档理解+OCR |
| **Apache 2.0专利授权** | 含专利授权的真正开源, 欧盟主权AI首选 |
| **Ministral阶梯** | 3B→675B同许可证同工具链, 从边缘到旗舰全覆盖 |

### NeoTrix 映射

| NeoTrix组件 | 映射 | 优先级 |
|-------------|------|--------|
| **NT-ACT行动域** | 原生视觉 = NT-ACT文档处理能力; Apache 2.0 = 工具链集成 | P0 |
| **NT-IO界面层** | Ministrall阶梯 = NT-IO多模型路由 (GWT cost-aware) | P1 |
| **NT-WORLD感知层** | 布局感知OCR = NT-WORLD文档解析增强 | P1 |
| **架构模式** | Granular MoE = NeoTrix能力网(CapabilityNetwork)的底层实现参考 | P2 |

---

## 8. Phi-4 reasoning (Microsoft, 2025-04)

### 架构逆向

**已知信息**: 14B Dense Transformer, 从Phi-4 SFT+RL训练, MIT开源.

**详细架构**:
- **基础模型**: Phi-4 (14B参数Dense Decoder-only Transformer)
- **修改**: 
  - 2个placeholder token重新用作 `<reasoning>`/`</reasoning>` 标记
  - RoPE基频翻倍, 上下文从16K→32K
- **SFT数据**: 1.4M prompt-response对, 8.3B唯一token
  - 来源: 公开网站+数据集+合成生成
  - 教师: o3-mini (medium/high effort)
  - 域: STEM + 代码 + 安全 + 负责任AI
- **训练**: 16K步骤, batch 32, 32K上下文, AdamW lr=1e-5
- **GRPO强化学习** (Phi-4-reasoning-plus):
  - 72,401数学问题, 每迭代采样64
  - 32x H100 GPU, batch 64
  - RL生成1.5x更长响应, 更高准确率
- **关键洞察**:
  - "Teachable" prompts — 选择模型能力边界的题目
  - 推理是可迁移的元技能 — 非训练域也有提升
  - 合成数据显著提升最终答案质量

**性能指标**:
- 参数: 14B (可本地运行于笔记本)
- 上下文: 32K tokens
- AIME 2025: 超越DeepSeek-R1 (671B)
- LiveCodeBench: o1-mini水平
- 训练时间: 2.5天 (32x H100)

### 关键创新

| 创新 | 描述 |
|------|------|
| **"Teachable"数据策展** | 选择模型能力边界的prompt, 最大化每样本信息增益 |
| **推理元技能迁移** | STEM推理训练迁移到日历规划/指令跟随等非训练域 |
| **小模型大能力** | 14B超越70B蒸馏模型, 接近671B完整模型 |
| **SFT→RL渐进** | SFT建立推理基础, RL提升1.5x token使用换取更高准确率 |

### NeoTrix 映射

| NeoTrix组件 | 映射 | 优先级 |
|-------------|------|--------|
| **NT-MIND进化域** | 可教导数据策展 = NT-MIND技能蒸馏的数据策略; RL = 自我进化 | P0 |
| **SEAL Pipeline** | SFT→RL渐进 = SEAL阶段的训练范式 | P0 |
| **E8推理引擎** | 推理元技能 = E8推理的可迁移性验证 | P1 |
| **NT-PHYSICAL具身域** | 14B本地运行 = 具身设备推理的可行性验证 | P1 |

---

## 9. Yi-Lightning (01.AI, 2024-12)

### 架构逆向

**已知信息**: Enhanced MoE, 细粒度专家分割+PEP路由+跨层KV共享.

**详细架构**:
- **Enhanced MoE**:
  - 细粒度专家分割: FFN分成更小单元, 减少中间隐藏维度, 增加每token激活专家数
  - 平衡点: 分割到训练效率最优为止, 非最大化分割
- **三层路由负载均衡**:
  1. **Switch-Transformer (L_ST)**: per-expert均匀分布
  2. **EP Load Balancing (L_EP)**: Expert Parallel组级均衡
  3. **Partitioned EP (L_PEP)**: 组内再分区, 解决All-to-All通信不均衡
  - 调优: α_PEP=1e-3, α_EP=1e-4, α_ST=1e-6
- **KV缓存缩减**:
  - **混合注意力**: 3层滑动窗口 + 1层全注意力
  - **跨层KV共享**: 连续全注意力层共享KV, 内存减半
  - 总效果: **82.8%内存缩减**
- **FP8硬件感知**: 架构对齐Hopper GPU, MoE算子1200 TFLOPS/card
- **并行策略**: EP+PP混合, 70%训练加速

**性能指标**:
- Chatbot Arena: 第6名 (中文/数学/编码第2-4名)
- 82.8% KV内存缩减
- 1200 TFLOPS/card (FP8, Hopper)
- 95% GPU利用率 (高并发)

### 关键创新

| 创新 | 描述 |
|------|------|
| **PEP三层路由** | per-expert→EP组→分区, 解决MoE通信不均衡的三级方案 |
| **82.8% KV缩减** | 混合注意力+跨层共享, 长上下文推理成本剧降 |
| **硬件感知架构** | 模型设计对齐GPU硬件规格, FP8原生优化 |
| **Benchmark差距观察** | 静态benchmark与动态人类偏好存在显著差异, 呼吁重新评估方法 |

### NeoTrix 映射

| NeoTrix组件 | 映射 | 优先级 |
|-------------|------|--------|
| **GWT注意力路由** | 混合注意力 = GWT本地/全局注意力的实现参考; PEP = 路由优化 | P0 |
| **KVMem分页KV** | 82.8% KV缩减 = NT-MEMORY缓存优化的技术路线 | P0 |
| **NT-SHIELD安全域** | Benchmark差距 = NT-SHIELD评估框架的警示 | P1 |
| **架构模式** | PEP三级路由 = CapabilityBridge跨层优化的同构设计 | P2 |

---

## 10. Grok 3 (xAI, 2025-02)

### 架构逆向

**已知信息**: Transformer, 10x Grok 2算力训练, Colossus超算(200K H100).

**推断架构**:
- **基础**: 大规模Dense/MoE Transformer (推测~1.5T参数)
- **训练**: Colossus超算, 200,000x NVIDIA H100 GPU
- **推理模式**: Think模式 — 显式推理, 可见思维链
- **Agent能力**: DeepSearch — 实时网络+X平台查询, 综合推理
- **多模态**: 文本为主, 图像理解(MMMU), 视频理解(EgoSchema)
- **上下文**: 1M tokens (API标注131K, 市场宣传1M)

**性能指标**:
- Chatbot Arena Elo: 1402
- AIME 2025 (Think): 93.3%
- GPQA Diamond (Think): 84.6%
- LiveCodeBench (Think): 79.4%
- SWE-bench Verified: 83.9%
- 上下文: 1M tokens (宣传) / 131K (API)

### 关键创新

| 创新 | 描述 |
|------|------|
| **规模化RL推理** | 史无前例的RL规模训练推理能力, 秒到分钟级思考 |
| **DeepSearch Agent** | 实时网络+社交平台查询, 综合多源信息推理 |
| **可见思维链** | 推理过程透明化, 用户可审查推理路径 |
| **Colossus基础设施** | 200K H100超算, 验证超大规模训练的工程可行性 |

### NeoTrix 映射

| NeoTrix组件 | 映射 | 优先级 |
|-------------|------|--------|
| **E8推理引擎** | Think模式 = E8六阶段循环的运行时实现; 可见推理 = 推理可审计性 | P0 |
| **NT-WORLD感知层** | DeepSearch = NT-WORLD爬虫+综合推理的同构设计 | P0 |
| **GWT注意力路由** | 1M上下文 = GWT在超长序列的注意力路由挑战 | P1 |
| **NT-ACT行动域** | DeepSearch = NT-ACT的agent搜索编排 | P1 |

---

## 跨模型创新趋势分析

### 趋势1: MoE成为主导架构

| 模型 | MoE类型 | 稀疏比 | 专家数 |
|------|---------|--------|--------|
| Gemini 2.5 Pro | Sparse MoE | 未公开 | 未公开 |
| Llama 4 Scout | Dense+MoE交替 | 6.4:1 | 16+1 |
| DeepSeek V4.1 Flash | CED+MoE | 34.5:1 | 384+1 |
| Qwen 3 MoE | Fine-grained MoE | 10.7:1 | 128 (无共享) |
| Mistral Large 3 | Granular MoE | 16.5:1 | 未公开 |
| Yi-Lightning | Enhanced MoE | 未公开 | 细粒度 |

**NeoTrix启示**: NeoTrix的能力网(CapabilityNetwork)天然适配MoE范式 — 每个能力域=一个专家, GWT路由器=MoE门控.

### 趋势2: KV缓存压缩军备竞赛

| 模型 | KV压缩技术 | 效果 |
|------|-----------|------|
| DeepSeek V4.1 Flash | CSA2+FP4+SWA Replay | 890 bytes/token (437x↓) |
| Yi-Lightning | 混合注意力+跨层共享 | 82.8%内存↓ |
| Llama 4 Scout | iRoPE (无位置编码层) | 10M上下文 |
| Gemini 2.5 Pro | 弹性训练+蒸馏 | 1M上下文 |

**NeoTrix启示**: KVMem分页KV + CSA2式压缩 = NT-MEMORY缓存层的技术路线图.

### 趋势3: Thinking/Budget可控推理

| 模型 | 思考模式 | Budget控制 |
|------|---------|-----------|
| Gemini 2.5 Pro | Thinking模式 | 用户可设置token上限 |
| Qwen 3 | 双模式统一 | Thinking Budget可调 |
| DeepSeek V4.1 Flash | effort 1-100 | 精确控制推理深度 |
| Grok 3 | Think模式 | 秒到分钟自适应 |
| Phi-4 reasoning | CoT推理链 | 1.5x token换取准确率 |

**NeoTrix启示**: GWT salience权重 + Thinking Budget = 注意力路由的运行时控制机制.

### 趋势4: 数据策展>规模

| 模型 | 数据策略 | 效果 |
|------|---------|------|
| Phi-4 reasoning | "Teachable" prompts + o3-mini蒸馏 | 14B超越70B+ |
| Qwen 3 | 4阶段渐进训练 | 30B-A3B超越QwQ-32B |
| Gemini 2.5 Pro | k-sparse蒸馏 | Flash系列高效 |
| Yi-Lightning | 合成数据+MCTS | Arena第6 |

**NeoTrix启示**: NT-MIND的数据策展能力 = SEAL pipeline的输入质量保证.

---

## NeoTrix吸收优先级矩阵

### P0 — 必须立即接线

| 创新 | 来源模型 | NeoTrix接线点 |
|------|---------|--------------|
| CSA2+FP4 KV压缩 | DeepSeek V4.1 Flash | KVMem分页KV优化 |
| MoE路由门控 | 全模型趋势 | CapabilityNetwork + GWT路由器 |
| Thinking Budget控制 | Gemini 2.5/Qwen 3 | GWT salience权重 |
| Engram条件记忆 | DeepSeek V4.1 Flash | KB条件记忆 + 跨会话模式 |
| 可教导数据策展 | Phi-4 reasoning | NT-MIND蒸馏数据策略 |

### P1 — 吸收强化

| 创新 | 来源模型 | NeoTrix接线点 |
|------|---------|--------------|
| Constitutional AI | Claude 3.5 Sonnet | NT-SHIELD治理宪法 |
| DeepSearch Agent | Grok 3 | NT-WORLD搜索编排 |
| iRoPE无限上下文 | Llama 4 Scout | NT-NEXUS跨会话记忆 |
| 4阶段渐进训练 | Qwen 3 | SEAL pipeline训练范式 |
| PEP三级路由 | Yi-Lightning | GWT路由优化 |

### P2 — 观望跟踪

| 创新 | 来源模型 | 跟踪理由 |
|------|---------|---------|
| 端到端多模态 | GPT-4o | 硬件要求极高, 等待开源验证 |
| 计算机使用 | Claude 3.5 Sonnet | Agent GUI交互需安全框架 |
| Granular MoE | Mistral Large 3 | 等待更多部署数据 |
| 推理元技能迁移 | Phi-4 reasoning | 需要更多泛化验证 |

---

## 附录: 技术报告来源

| 模型 | 来源 |
|------|------|
| GPT-4o | OpenAI System Card (arXiv:2410.21276), GPT-ImgEval |
| Claude 3.5 Sonnet | Anthropic Model Card Addendum |
| Gemini 2.5 Pro | Google DeepMind Technical Report (arXiv:2507.06261) |
| Llama 4 Scout | Meta Model Card + AI Blog |
| DeepSeek V4.1 Flash | DeepSeek Technical Report + HuggingFace |
| Qwen 3 | Alibaba Technical Report (arXiv:2505.09388) |
| Mistral Large 3 | Mistral AI Technical Documentation |
| Phi-4 reasoning | Microsoft Research (arXiv:2504.21318) |
| Yi-Lightning | 01.AI Technical Report (arXiv:2412.01253) |
| Grok 3 | xAI Blog + PerplexityAI Analysis |
