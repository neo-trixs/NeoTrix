# 破限制技术第49批 — 位置编码/注意力稀疏化/长序列建模/多模态融合/检索增强

> 日期: 2026-09-11 | 5 主题 × 3-5 来源 = 22 来源 | 突破点 + NeoTrix 融合

---

## 1. 位置编码改进 (Positional Encoding)

### 1.1 ALiBi 2D-Aware — ViT 位置偏差消除

**来源**: arXiv:2603.16840 (Aug 2026), "What DINO saw: ALiBi positional encoding reduces positional bias in Vision Transformers"

**突破点**:
- 首次在 ViT (DINOv2/DINOv3) 中引入 **2D-aware ALiBi 位置编码**: 圆柱边界条件 + 归一化, 支持位置编码插值
- **位置偏差 (positional bias)** 在自监督 ViT 中普遍存在: 影响可训练分割性能, ALiBi 通过在注意力分数上施加线性偏差而非编码到隐状态中来消除
- 跨分辨率泛化: ALiBi 的 interpolation trick 使位置编码可外推到未见图像尺寸
- **关键发现**: RoPE/DINOv2 的 learned PE 在 edge effects 上有系统性偏差, ALiBi 通过相对距离编码消除这一问题

**NeoTrix 融合**:
- **VSA HyperCube 感知对齐**: ALiBi 的 2D-aware 位置编码 = VSA 空间中 token 位置感知的增强, 可用于 NT-WORLD 的图像理解管线中增强位置敏感性
- **PerceptionBridge 注意力调制**: ALiBi 的相对距离偏差 → PerceptionBridge 的 awareness_score 计算中引入距离衰减因子, 过滤低相关性感知事件

### 1.2 ALiBi 数值失效 — bfloat16 精度陷阱

**来源**: arXiv:2608.03994 (Aug 2026), "When Attention Goes Blind: Numerical Failure in ALiBi Positional Encodings"

**突破点**:
- 发现 ALiBi 的 **线性偏差缩放在 bfloat16 精度下 underflow**, 导致大量注意力权重为零, 注意力头部分失明
- **量化分析**: 在 16 头层中, 距离 >128 token 时, 大部分 bias 值 underflow 到零
- 尽管存在该问题, 默认 ALiBi slopes 仍是强基线 (尤其在 needle-in-a-haystack 检索任务上)
- 提出具体训练建议: 使用 float32 累加 bias、动态 slope 选择、head-specific 精度策略

**NeoTrix 融合**:
- **NT-SHIELD 精度监控**: ALiBi 数值失效 = GWT 注意力路由的潜在盲区 → NT-SHIELD 可监控推理精度, 在 bfloat16 场景下自动切换 float32 累加
- **HeartbeatAggregator 精度信号**: 将注意力权重 underflow 比例作为系统健康指标, 纳入 SystemHealthSnapshot

### 1.3 位置编码综述 — 从绝对到旋转的统一视角

**来源**: arXiv:2608.10021 (Aug 2026), "Position Encoding in Transformers: From Absolute and Relative Methods to RoPE and Long-Context Scaling"

**突破点**:
- 统一推导 RoPE 如何将绝对位置索引转换为 QK 内积中的 **相对相位差**
- 系统比较: ALiBi/RoPE/APE 在 KV cache 兼容性、长度外推上的差异
- **核心结论**: 能计算超出训练长度的位置特征 ≠ 可靠的长上下文泛化, 需通过短上下文保留、位置困惑度、检索、推理等多维评估
- 涵盖 Position Interpolation → NTK-aware → YaRN → LongRoPE → LongRoPE2 的完整演进线

**NeoTrix 融合**:
- **GWT 注意力路由**: RoPE 的相位差机制可用于 GWT 的 salience 计算 — 位置相关的注意力衰减曲线影响信息广播范围
- **SEAL Pipeline 评估维度**: 位置编码的多维评估框架可作为 SEAL 的自我测试维度 — 模块间位置信息传递的可靠性

### 1.4 隐状态通道缩放 — 单通道消除位置偏差

**来源**: ACL 2025 Findings, "Mitigate Position Bias in LLMs via Scaling a Single Hidden States Channel"

**突破点**:
- 发现位置偏差不仅来自位置编码, 还来自 **隐状态的特定通道 (positional hidden states)**
- 仅修改 **一个通道** 的缩放因子 (乘以 s), 即可在 "lost in the middle" 基准上提升 15.2%
- 适用于多种模型: RoPE 模型、上下文扩展模型、ALiBi 模型均有效
- **微调级方法**: 无需重训, 仅在校准集上搜索最佳通道和缩放因子

**NeoTrix 融合**:
- **NT-MIND 微调策略**: 单通道缩放 = 极低成本的注意力偏差修正, 可集成到 NT-MIND 的 opportunistic fine-tuning 窗口中
- **Axiom A2 (Context as Scarce Resource)**: 该方法通过极小干预提升长上下文利用率, 直接体现上下文稀缺性原则

---

## 2. 注意力稀疏化 (Sparse Attention)

### 2.1 SSE — 线性注意力的稀疏状态扩展

**来源**: ICLR 2026, "Scaling Linear Attention Capacity with Sparse State Expansion" (arXiv:2507.16577)

**突破点**:
- **Sparse State Expansion (SSE)**: 将线性注意力的状态更新重新概念化为 **信息分类** — softmax top-k 行稀疏选择
- 状态扩展为 N 个分区, 参数共享 + 稀疏行选择 → 计算/参数开销近似常数
- **2B SSE-H 模型**: AIME24 64.5 / AIME25 50.2, 超越同规模开源 Transformer
- 长上下文检索: SSE 持续优于其他线性注意力模型, 混合变体显著缩小与 softmax attention 的差距
- **关键洞察**: 线性注意力的固定状态大小 (c=128) 仅等效于 64 token 的 softmax 注意力窗口, 扩展状态容量是核心方向

**NeoTrix 融合**:
- **NT-MEMORY 状态压缩**: SSE 的行稀疏更新 = KB embedding 的稀疏存储策略 — 仅保留 top-k 相关状态, 减少 KV cache 压力
- **KV Cache Optimizer 扩展**: SSE 的状态分区 + 参数共享 = kv_cache_optimizer.rs 的稀疏状态扩展范式
- **Axiom A1 (Cost-Aware Routing)**: SSE 以线性复杂度达到 Transformer 级性能, 直接支持低成本推理路由

### 2.2 Sparse Feature Attention — 特征维度稀疏化

**来源**: ICLR 2026, "Scaling Attention via Feature Sparsity" (arXiv:2603.22300)

**突破点**:
- **Sparse Feature Attention (SFA)**: Q/K 表示为 k-sparse codes, 在 **特征维度** 上稀疏化 (非 token 维度)
- 将注意力成本从 Θ(n²d) 降至 Θ(n²k²/d), 保留高维表达力
- **稀疏矩阵乘法**: 仅在重叠活跃坐标上计算注意力分数
- 8k 上下文时 1.9x 解码加速, KV cache 缩小常数因子 ≥2
- 与 token 级稀疏化和 paging **正交可组合**

**NeoTrix 融合**:
- **GWT 注意力广播优化**: SFA 的特征级稀疏化 = GWT 广播时仅传输高 salience 特征通道, 降低跨模块通信成本
- **HyperCube 维度裁剪**: SFA 的 top-k 特征选择 = VSA 空间的维度剪枝 — 仅保留最高信息量的向量维度

### 2.3 稀疏前沿 — 稀疏注意力的大规模评估

**来源**: ACL 2026 Findings, "The Sparse Frontier: Sparse Attention Trade-offs in Transformer LLMs"

**突破点**:
- **最大规模训练无关稀疏注意力评估**: 6 方法 × 多模型 × 128K 序列 × 0.95 稀疏度
- **四维分类法**: 结构单元 (token/block/row/threshold) × 估计方式 × 保留策略 × 分配策略
- **核心发现**:
  - 稀疏注意力有效: 更大稀疏模型在等成本下优于更小稠密模型 (改善 Pareto 前沿)
  - 细粒度 per-query 重要性估计在 prefilling 时仍不实用 (成本过高)
  - 简单的 top-k key block 选择 ≈ 复杂方法性能
- 混合注意力架构 (如 Gemma 3 的 sliding window + global) 天然适配稀疏化

**NeoTrix 融合**:
- **NT-ACT 任务路由**: 稀疏注意力的 Pareto 改进 → NT-ACT 按任务复杂度选择稀疏/稠密注意力路径
- **NT-SHIELD 混合架构**: sliding window + global attention = NT-SHIELD 的多层安全检查模式 — 本地窗口快速检查 + 全局深度审计

### 2.4 MiniCPM-SALA — 稀疏+线性混合注意力

**来源**: arXiv:2602.11761 (2026), "MiniCPM-SALA: Hybridizing Sparse and Linear Attention for Efficient Ultra-Long Sequence Modeling"

**突破点**:
- **混合架构**: 稀疏注意力 + 线性注意力的协同设计, 专为超长序列建模
- 稀疏层处理局部精细交互, 线性层处理全局上下文压缩
- 在 128K+ 序列长度上实现效率-质量的最优平衡

**NeoTrix 融合**:
- **Six-Layer Architecture 混合策略**: L1-L2 用线性注意力 (快速感知), L5-L6 用稀疏注意力 (精细推理), 分层匹配计算预算
- **SEAL Pipeline 序列处理**: 超长经验序列的混合注意力 = SEAL 处理历史经验时的分层策略

---

## 3. 长序列建模 (Long Sequence)

### 3.1 FCP — 灵活上下文并行

**来源**: arXiv:2602.21788 (2026), "Efficient Scaling of LLM Training with Flexible Context Parallelism"

**突破点**:
- **Flexible Context Parallelism (FCP)**: 自适应重配置通信组和上下文并行度, 处理异构序列长度
- 动态 mesh 设计: 按微批次自适应调整并行度, 消除长尾序列的 straggler 效应
- **性能**: 比 Megatron-LM/DeepSpeed 提升 1.46x 平均吞吐, 极端不平衡批次 2.24x 加速
- 基于 Ring-style CP, 支持任意整数并行度

**NeoTrix 融合**:
- **SEAL Pipeline 异构处理**: FCP 的动态并行度 = SEAL 处理不同长度经验序列时的自适应策略 — 短经验快速处理, 长深度分析分配更多资源
- **NT-ACT 资源调度**: FCP 的 straggler 消除 = NT-ACT 的负载均衡, 避免长任务阻塞短任务

### 3.2 ByteScale — 2048K 上下文训练

**来源**: arXiv:2502.21231 (SIGCOMM 2025), "ByteScale: Efficient Scaling of LLM Training with a 2048K Context Length on More Than 12,000 GPUs"

**突破点**:
- **Hybrid Data Parallelism (HDP)**: 统一 inter-data 和 intra-data 分区, 动态 mesh 设计
- **通信优化器**: data-aware sharding + selective offloading, 消除短序列冗余通信
- **平衡调度器**: 并行度感知的数据分配, 缓解计算不平衡
- 7B-141B 模型, 256K-2048K 上下文, 12000+ GPU 验证
- **比 SOTA 训练系统提升 7.89x**

**NeoTrix 融合**:
- **NT-MEMORY 超长上下文**: ByteScale 的 2048K 训练 = NT-MEMORY 支持超长会话历史的持久化和检索
- **ConsciousnessTree 跨会话记忆**: HDP 的动态分区 = ConsciousnessTree 跨会话经验的自适应存储策略

### 3.3 AutoSP — 编译器自动化序列并行

**来源**: arXiv:2604.27089 (Apr 2026), "AutoSP: Unlocking Long-Context LLM Training Via Compiler-Based Sequence Parallelism"

**突破点**:
- **首个编译器自动化序列并行方案**: 将 SP 作为 PyTorch 2.0 编译器 pass
- **两个关键 pass**: (1) 自动序列并行变换 — 插入通信 collective + 重塑激活; (2) SP 感知激活检查点 — 利用长上下文训练的计算-内存特性
- **NVIDIA + AMD 双平台验证**: 上下文长度提升 2.7x (NVIDIA) / 2.5x (AMD)
- 几行代码即可将标准 PyTorch 模型编译为分布式长上下文训练管线

**NeoTrix 融合**:
- **SEAL Pipeline 自动优化**: AutoSP 的编译器自动化 = SEAL 的自动化管线优化 — 声明式配置 → 自动插入并行策略
- **NT-IO 推理优化**: AutoSP 的 SP-aware checkpointing 可用于 NT-IO 的推理管线, 降低长上下文推理的内存开销

### 3.4 InfiniPipe — 弹性流水线并行

**来源**: arXiv:2509.21275 (NAACL 2025/updated Apr 2026), "InfiniPipe: Elastic Pipeline Parallelism for Efficient Variable-Length Long-Context LLM Training"

**突破点**:
- **Elastic Pipeline Parallelism (EPP)**: 编排 token-level PP + batch-level PP, 自适应资源和工作负载异构性
- **Stage-Aware Chunk-Level Adaptive Checkpointing**: 与 EPP 集成的梯度检查点策略
- 解决: batch-level PP 高内存 + token-level PP 硬件利用不足的矛盾
- **1.69x 加速** over SOTA, 开源代码

**NeoTrix 融合**:
- **NT-ACT 弹性调度**: EPP 的弹性并行 = NT-ACT 的自适应任务调度 — 按序列长度动态选择 token-level 或 batch-level 处理
- **HeartbeatAggregator 资源监控**: EPP 的资源自适应 = Heartbeat 的计算资源信号, 驱动调度决策

### 3.5 LoongServe — 推理弹性序列并行

**来源**: SOSP 2024, "LoongServe: Efficiently Serving Long-Context Large Language Models with Elastic Sequence Parallelism"

**突破点**:
- **Elastic Sequence Parallelism (ESP)**: 推理时动态调整序列并行度, 处理变长请求
- 解决静态并行策略无法适应混合长度请求的问题
- 跨节点弹性扩缩容, KV cache 分布式管理
- SOSP 级系统论文, 工业级验证

**NeoTrix 融合**:
- **NT-IO 推理服务**: LoongServe 的 ESP = NT-IO 的 LLM 推理服务优化 — 按请求长度动态分配 GPU 资源
- **Axiom A2 (Context as Scarce Resource)**: ESP 的 KV cache 分布式管理 = 上下文稀缺性原则的系统级实现

---

## 4. 多模态融合 (Multimodal Fusion)

### 4.1 LLM-Centric 多模态融合综述

**来源**: arXiv:2506.04788 (Jun 2025), "Towards LLM-Centric Multimodal Fusion: A Survey on Integration Strategies and Techniques"

**突破点**:
- **三维分类框架**: 架构策略 (融合机制 × 融合层级) × 表示学习 (联合/协调) × 训练范式
- 融合机制: Abstraction / Projection / Semantic Embedding / Cross-attention
- 融合层级: Early (LLM 前) / Intermediate (LLM 层内) / Hybrid (两者结合)
- 分析 125 个 MLLM (2021-2025), 识别出 **Intermediate Fusion 增长最快**
- **Cross-attention adapter** 成为主流: 保持核心 LLM 冻结, 通过适配器注入模态信息

**NeoTrix 融合**:
- **NT-WORLD 感知融合**: 三层融合框架直接映射到 NT-WORLD 的感知管线 — Early Fusion (原始信号合并) / Intermediate Fusion (层间交互) / Hybrid (自适应选择)
- **PerceptionBridge 融合策略**: Cross-attention adapter = PerceptionBridge 的 attention-gated 桥接机制的多模态扩展
- **CapabilityBridge 模态映射**: 融合层级选择 = CapabilityBridge 的能力路由策略

### 4.2 原生多模态模型缩放定律

**来源**: ICCV 2025 (Oral), "Scaling Laws for Native Multimodal Models" (arXiv:2504.07951)

**突破点**:
- **457 个模型的缩放定律研究**: Early-fusion vs Late-fusion 的系统比较
- **核心发现**: Early-fusion 在低参数量时更强, 训练更高效, 部署更简单
- Late-fusion 无固有优势 — 之前认为的优势来自预训练组件的复用
- **MoE + Early-fusion**: Mixture of Experts 允许学习模态特定权重, 显著提升性能
- 多模态模型的缩放指数与文本 LLM 类似, 略有变化

**NeoTrix 融合**:
- **NT-WORLD 感知架构**: Early-fusion + MoE = NT-WORLD 的感知管线设计 — 原始信号直接融合, MoE 路由不同模态
- **GWT 注意力路由**: MoE 的模态特定权重 = GWT 的 salience 计算中模态权重的自适应调整
- **Axiom A1 (Cost-Aware Routing)**: Early-fusion 低参数效率 = 低成本模态融合路径, 适合 I/O 类任务

### 4.3 Meta Fusion — 统一多模态融合框架

**来源**: arXiv:2507.20089 (Jul 2025), "Meta Fusion: A Unified Framework For Multimodality Fusion with Mutual Learning"

**突破点**:
- **统一框架**: Early/Intermediate/Late fusion 均为 Meta Fusion 的特例
- **互学习 (Mutual Learning)**: 多个单模态 student 通过 disagreement penalty 协同学习
- **PCA 生成潜在表示 + cohort 构建**: 自适应互学习 + top-performer 聚合
- 在互补信息场景下 Early Fusion 一致优于 Late Fusion
- **Diverse Margin Loss**: 集合级目标, 强制 ground-truth 互补证据链主导冗余替代

**NeoTrix 融合**:
- **NT-MIND 多模态蒸馏**: Meta Fusion 的互学习 = NT-MIND 的跨域知识蒸馏 — 各域 (NT-WORLD/NT-ACT/NT-MEMORY) 作为单模态 student 协同学习
- **ConsciousnessTree 共识**: 互学习的 disagreement penalty = ConsciousnessTree 的跨分支一致性检查
- **SEAL Pipeline 融合质量**: Diverse Margin Loss 可用于 SEAL 的经验融合 — 确保互补经验主导冗余

### 4.4 多模态融合策略比较

**来源**: arXiv:2511.21889 (Nov 2025), "Exploring Fusion Strategies for Multimodal Vision-Language Systems"

**突破点**:
- **BERT + MobileNetV2 混合框架**: 系统比较 Early/Intermediate/Late fusion 在情感分析上的表现
- **权衡发现**: Late fusion 精度最高, Early fusion 推理延迟最低
- **边缘部署**: Early fusion 更适合边缘设备 (低延迟)
- Intermediate fusion 在精度-延迟之间取得平衡

**NeoTrix 融合**:
- **NT-SHIELD 安全检查**: Late fusion (高精度) 用于安全审计, Early fusion (低延迟) 用于实时过滤 — 按场景选择融合策略
- **NT-PHYSICAL 具身融合**: 边缘部署的 Early fusion = NT-PHYSICAL 的传感器融合策略 — 低延迟感知

---

## 5. 检索增强 (RAG)

### 5.1 A-RAG — 层级检索接口的 Agentic RAG

**来源**: arXiv:2602.03442 (Feb 2026), "A-RAG: Scaling Agentic Retrieval-Augmented Generation via Hierarchical Retrieval Interfaces"

**突破点**:
- **层级检索接口**: keyword_search (精确词汇匹配) + semantic_search (稠密检索) + chunk_read (完整文档块)
- **Agent 自主决策**: 何时检索、检索什么、如何检索 — 模型参与检索决策
- **Test-Time Scaling**: 性能随计算资源增加稳步提升, 框架随模型能力进步高效扩展
- **SOTA 性能**: MuSiQue 74.1%, HotpotQA 94.5%, 2Wiki 89.7% (GPT-5-mini)
- **关键突破**: 打破 "检索-拼接-生成" 范式, 让模型自主控制检索过程

**NeoTrix 融合**:
- **NT-MEMORY KB 检索**: A-RAG 的层级接口 = NT-MEMORY 的多粒度检索 — BM25 (keyword) + 向量 (semantic) + 全文 (chunk_read)
- **GWT 注意力路由**: Agent 自主检索决策 = GWT 的 salience 驱动信息获取 — 按需检索而非预加载
- **Axiom A2 (Context as Scarce Resource)**: 按需检索 + 层级粒度 = 最小化上下文窗口占用

### 5.2 ScalDPP — 密度×多样性检索

**来源**: arXiv:2604.03240 (Feb 2026), "Scaling DPPs for RAG: Density Meets Diversity"

**突破点**:
- **ScalDPP**: Determinantal Point Processes 用于 RAG 检索, 通过 P-Adapter 实现可扩展的 chunk 间依赖建模
- **Diverse Margin Loss (DML)**: 集合级目标, 确保 ground-truth 互补证据链主导冗余替代
- 解决标准 RAG 的 **冗余上下文问题**: point-wise 评分忽略 chunk 间交互
- **联合优化密度+多样性**: 检索结果信息密集且覆盖广泛

**NeoTrix 融合**:
- **NT-MEMORY 经验检索**: ScalDPP = KB 经验检索的去重+多样性策略 — 避免冗余经验稀释检索质量
- **SEAL Pipeline 经验融合**: DML 的集合级目标 = SEAL 吸收阶段的经验去重 — 互补经验优先, 冗余抑制
- **HyperCube 向量去重**: DPP 的 repulsion 核 = VSA 空间的向量去重, 避免相似概念重复存储

### 5.3 Agentic RAG 综述

**来源**: arXiv:2501.09136 (Jan 2025, updated 2026), "Agentic Retrieval-Augmented Generation: A Survey on Agentic RAG"

**突破点**:
- **RAG 范式演进**: Naive RAG → Advanced RAG → Modular RAG → Graph RAG → Agentic RAG
- **Agentic RAG 核心能力**: 自主规划、多工具调用、验证反馈、多 agent 协作
- **五类 Agent**: Routing Agent / Query Planning Agent / Retrieval Agent / Validation Agent / Response Synthesis Agent
- **多模态扩展**: Agentic RAG 支持图像/音频等多模态数据类型
- **挑战**: 输出不一致、过度泛化、幻觉验证

**NeoTrix 融合**:
- **NT-ACT 工具编排**: Agentic RAG 的五类 Agent = NT-ACT 的工具调用编排模式 — routing/planning/retrieval/validation/synthesis 分工
- **NT-MEMORY 知识图谱**: Graph RAG 的图结构 = NT-MEMORY KB 的图索引增强
- **NT-SHIELD 幻觉验证**: Validation Agent = NT-SHIELD 的输出验证层, 防止幻觉传播

### 5.4 高级 RAG 技术实践

**来源**: Neo4j Blog (2026), "Advanced RAG techniques for high-performance LLM applications"

**突破点**:
- **知识图谱 + Agentic Loop**: 图感知检索 + plan → route → act → verify → stop 循环
- **混合检索**: BM25 (稀疏) + 向量 (稠密) + 图查询 (结构化) 的三路融合
- **Corrective RAG (CRAG)**: 检索后验证 → 低质量结果触发 web 搜索补充
- **评估指标**: 检索质量 (recall@k) + 回答质量 (faithfulness) + 运营指标 (延迟/成本)

**NeoTrix 融合**:
- **NT-MEMORY 混合检索**: 三路融合 = NT-MEMORY 的 ordered backend fallback 模式 (DDG → Wikipedia → KB)
- **SEAL Pipeline 验证循环**: CRAG 的验证-补充 = SEAL 的 converge_check 自我审计
- **HeartbeatAggregator RAG 健康**: 检索质量 + 回答质量 = 系统健康信号的 RAG 维度

---

## 跨主题融合矩阵

| 主题 | NeoTrix 域 | 具体映射 |
|------|-----------|---------|
| 位置编码偏差修正 | NT-SHIELD + GWT | 精度监控 + 注意力衰减曲线 |
| 线性注意力稀疏扩展 | NT-MEMORY + KV Cache | 状态压缩 + 稀疏存储 |
| 特征级稀疏化 | GWT + VSA | 广播优化 + 维度裁剪 |
| 编译器自动化并行 | SEAL + NT-IO | 管线优化 + 推理加速 |
| 弹性序列并行 | NT-ACT + Heartbeat | 自适应调度 + 资源监控 |
| Early-fusion + MoE | NT-WORLD + GWT | 感知融合 + 模态路由 |
| 层级 Agentic RAG | NT-MEMORY + NT-ACT | 多粒度检索 + 工具编排 |
| 密度×多样性检索 | NT-MEMORY + SEAL | 经验去重 + 互补吸收 |

## 新增 Absorbed 术语

| 术语 | 定义 | NeoTrix 映射 |
|------|------|-------------|
| **SSE (Sparse State Expansion)** | 线性注意力的行稀疏状态扩展, 通过 softmax top-k 分类实现稀疏状态更新, 解耦参数大小与状态容量 | NT-MEMORY 稀疏状态存储 |
| **SFA (Sparse Feature Attention)** | 特征维度稀疏注意力, Q/K 转为 k-sparse codes, 在特征轴上稀疏化, 与 token 级稀疏正交 | GWT 特征级广播优化 |
| **FCP (Flexible Context Parallelism)** | 自适应重配置通信组和并行度, 处理异构序列长度, 消除 straggler | SEAL 异构经验处理 |
| **HDP (Hybrid Data Parallelism)** | 统一 inter/intra-data 分区的动态 mesh 设计, 消除短序列冗余通信 | NT-ACT 动态任务分区 |
| **AutoSP** | 编译器自动化序列并行, 作为 PyTorch 编译器 pass 实现, 自动插入通信 collective | SEAL 管线自动化优化 |
| **EPP (Elastic Pipeline Parallelism)** | 编排 token-level + batch-level PP, 自适应资源和工作负载异构性 | NT-ACT 弹性调度 |
| **ESP (Elastic Sequence Parallelism)** | 推理时动态调整序列并行度, 处理变长请求 | NT-IO 推理服务优化 |
| **A-RAG** | 层级检索接口的 Agentic RAG, 模型自主决策何时/检索什么/如何检索 | NT-MEMORY 多粒度检索 |
| **ScalDPP** | DPP 用于 RAG 检索, 联合优化密度+多样性, P-Adapter 实现可扩展 chunk 间依赖建模 | NT-MEMORY 检索去重 |
| **AdaGroPE** | 自适应分组位置编码, training-free 的 LLM 上下文窗口外推, 渐进增加远距离 token 的位置复用计数 | GWT 位置感知扩展 |
| **CRAG (Corrective RAG)** | 检索后验证, 低质量结果触发 web 搜索补充的自纠正机制 | SEAL converge_check 自审 |
| **Meta Fusion** | 统一多模态融合框架, Early/Intermediate/Late fusion 均为其特例, 互学习协同 | NT-MIND 跨域蒸馏 |
