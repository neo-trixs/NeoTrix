# 逆向推理新模型 — 2026-09-11 架构逆向工程

> 10 个前沿 LLM 架构创新提取 + NeoTrix 映射。Cycle 244.

## 总览矩阵

| 模型 | 架构基座 | 参数量 | 激活量 | 关键创新 | NeoTrix 映射 |
|------|---------|--------|--------|---------|-------------|
| GPT-4o | Decoder-only Transformer | 未公开 | 全量 | 原生多模态统一 tokenizer, 232ms 语音延迟 | NT-IO 多模态融合, PerceptionBridge |
| Claude 3.5 Sonnet | Dense Transformer | 未公开 | 全量 | Constitutional AI, 计算机视觉操控, Agentic Coding | NT-GOVERNANCE 合规引擎, NT-ACT agentic loop |
| Gemini 2.5 Pro | Sparse MoE Transformer | 未公开 | 部分 | 原生多模态 MoE, 可控 Thinking Budget, Deep Think 并行推理 | GWT salience 路由, SEAL pipeline 预算控制 |
| Llama 4 Scout | MoE + Early Fusion | 109B (17B active) | 17B | iRoPE 无限上下文, 10M token 窗口, 早期融合多模态 | NT-MEMORY 超长记忆, PerceptionBridge 早期融合 |
| DeepSeek V4.1 Flash | Causal Encoder-Decoder MoE | 552B (8B/16B active) | 8B/16B | CED 架构, CSA2 稀疏注意力, FP4 KV 缓存, Engram 条件记忆 | kv_cache_optimizer, NT-MEMORY 分级存储 |
| Qwen 3 | Dense + MoE 混合 | 0.6B–235B | 3B–22B | Thinking/Non-thinking 双模, QK-Norm, 思考预算控制 | NT-CORE 双过程, GWT 注意力预算 |
| Mistral Large 3 | Granular MoE | 675B (41B active) | 41B | 粒度化 MoE, 原生多模态视觉编码器, 256K 上下文 | NT-ACT 粒度化路由, NT-WORLD 视觉感知 |
| Phi-4 Reasoning | Dense Transformer | 14B | 全量 | 小模型推理蒸馏, GRPO 强化学习, 可复用推理元技能 | NT-MIND 蒸馏流水线, SEAL 自进化 |
| Yi-Lightning | Fine-grained MoE | 未公开 | 部分 | 细粒度专家分割, 混合注意力(3 SWA + 1 Full), 跨层 KV 共享 | GWT 混合注意力, kv_cache_optimizer 跨层复用 |
| Grok 3 | Transformer MoE | ~1.5T (估) | 部分 | Think 模式+DeepSearch 代理, 1M 上下文, 大规模 RL 推理 | NT-CORE 推理模式切换, NT-ACT 代理搜索 |

---

## 1. GPT-4o (OpenAI, 2024-05)

### 架构创新

**原生多模态统一架构**
- 单一神经网络端到端训练 text + vision + audio，非分阶段 CLIP 管线
- 统一 tokenizer：BPE 文本 + ViT 图像 patch + 神经音频编解码器 (Encodec/SoundStream 类)
- 语音中位延迟 232ms（前代 GPT-4 Turbo 管线 2.8s，降低 12x）
- 模态特定 embedding/unembedding 层，但跨模态通过 self-attention 原生交互

**非英语 tokenizer 优化**
- 新 tokenizer 对非英语语言 (Hindi/Arabic/Korean/Tamil) 压缩率提升 1.1x–4.4x
- 128K 上下文窗口，16K 最大输出

**训练方法**
- 多万亿 token 混合：网页文本 + 代码 + 书籍 + 图像-文本对 + 音频
- RLHF + model-graded rewards + 红队测试 (Preparedness Framework)

### NeoTrix 映射

| GPT-4o 创新 | NeoTrix 组件 | 吸收路径 |
|-------------|-------------|---------|
| 统一多模态 tokenizer | `PerceptionBridge` 感知融合 | L2 感知层：多模态 token 统一到 SelectiveState |
| 端到端跨模态训练 | GWT 跨模态注意力广播 | L5 认知层：GWT salience 扩展到 audio token |
| 音频 token 50-100 Hz | NT-IO 音频接口 | L1 行动层：nt_io 音频 tokenizer 适配器 |
| 128K 上下文 | kv_cache_optimizer 窗口管理 | NT-MEMORY：KV 缓存 paged 管理 |

---

## 2. Claude 3.5 Sonnet (Anthropic, 2024-06)

### 架构创新

**Constitutional AI 合规训练**
- 基于 UN 人权宣言等原则的 Constitutional AI 对齐
- HHH (Helpful, Honest, Harmless) 三目标训练
- Collective Constitutional AI：加入残障权利原则
- ASL-2 安全级别分类 + RSP (Responsible Scaling Policy) 门槛评估

**Agentic Coding 能力**
- 内部 agentic coding 评估：64% → 78% (升级版)
- SWE-bench Verified 49.0% pass@1
- 真实 GitHub PR 级任务：搜索、查看、编辑多文件，自主循环纠错
- 沙箱环境无网络访问下执行

**计算机视觉操控**
- 升级版引入 GUI 截图理解 → 生成鼠标/键盘操作
- OSWorld 14.9% (纯截图输入) → 22% (50 步交互)
- 开创性地将视觉感知与工具调用闭环

**多模态输入 + 文本输出**
- 支持 JPEG/PNG/GIF/WebP (≤10MB, 8000x8000px)
- 视觉理解：MathVista, ChartQA, DocVQA, AI2D 达 SOTA

### NeoTrix 映射

| Claude 3.5 创新 | NeoTrix 组件 | 吸收路径 |
|----------------|-------------|---------|
| Constitutional AI | NT-GOVERNANCE 合规引擎 | Gov-衡：宪法级规则 + 行为护栏 |
| Agentic coding loop | NT-ACT 代理执行 | Dev-匠：agentic self-correct loop |
| GUI 截图操控 | NT-IO 计算机操控 | L1 行动层：screenshot → action 闭环 |
| RSP 安全评估 | NT-SHIELD 安全审计 | Rev-明：风险分级 + 门槛评估 |
| HHH 三目标对齐 | NT-FEEL 情感对齐 | L4 情感层：helpful/harmless 平衡 |

---

## 3. Gemini 2.5 Pro (Google DeepMind, 2025-03)

### 架构创新

**Sparse MoE + 原生多模态**
- 稀疏 MoE Transformer，原生支持 text + vision + audio
- MoE 解耦总参数容量与每 token 计算成本
- 训练于 TPUv5p，8960 芯片 pod，跨数据中心同步数据并行
- Slice-Granularity 弹性：局部故障自动恢复，仅损失数十秒

**可控制 Thinking Budget**
- Thinking 作为原生能力，非独立模型
- 用户可设置 thinking token 预算，平衡性能与成本
- 推理时计算可扩展：budget 越大 → 准确率越高

**Deep Think 并行推理**
- 并行生成多个假设，相互批判后得出最终答案
- USAMO 2025 (数学)、LiveCodeBench (编程)、MMMU (多模态) 达 SOTA

**1M+ 上下文 + 3 小时视频**
- 处理整本《白鲸记》或《堂吉诃德》
- 3 小时视频理解 + 视频→交互式代码应用

**Distillation 到小模型**
- Flash-Lite 使用 k-sparse 分布近似教师模型 token 预测
- 训练数据吞吐量增加 k 倍，质量显著提升

**训练稳定性突破**
- 信号传播和优化动态大幅改进
- SDC (静默数据腐蚀) 检测：轻量确定性重放，数分钟内定位

### NeoTrix 映射

| Gemini 2.5 创新 | NeoTrix 组件 | 吸收路径 |
|----------------|-------------|---------|
| MoE 路由 | GWT salience + cost weight | A1: 成本感知路由，便宜任务用便宜模型 |
| Thinking Budget | SEAL pipeline 预算控制 | L5 认知层：推理深度可调 |
| Deep Think 并行假设 | NT-CORE E8 并行推理 | E8 hexagram 多路径探索 |
| 1M 上下文 | KVMem paged KV | A2: 上下文稀缺资源，>256K 用 paged KV |
| 弹性训练恢复 | NT-REPAIR 自愈 | MAPE-K 循环：监控→分析→计划→执行→知识 |
| Distillation | NT-MIND 蒸馏流水线 | 蒸馏到小模型，保持质量 |

---

## 4. Llama 4 Scout (Meta, 2025-04)

### 架构创新

**iRoPE 无限上下文架构**
- 交替注意力层无位置编码 + RoPE 位置编码层
- "i" = interleaved，目标支持无限上下文长度
- 推理时温度缩放注意力增强长度泛化
- 256K 预训练 + 10M 推理时扩展

**早期融合多模态**
- Text + image token 早期融合到统一骨干
- 非标注 text/image/video 联合预训练
- 视觉编码器基于 MetaCLIP，与 frozen Llama 联合训练适配

**MoE + 共享专家**
- Scout: 17B active / 109B total, 16 routed experts + 1 shared expert
- Maverick: 17B active / 400B total, 128 routed experts + 1 shared expert
- 每 token 同时路由到共享专家 + 1 个路由专家

**高效量化**
- BF16 权重，Int4 量化后 Scout 适配单 H100
- FP8 量化 Maverick 适配单 H100 DGX

### NeoTrix 映射

| Llama 4 创新 | NeoTrix 组件 | 吸收路径 |
|-------------|-------------|---------|
| iRoPE 无限上下文 | NT-MEMORY 无限记忆 | L1 行动层：位置编码自适应 |
| 早期融合 | PerceptionBridge 早期融合 | L2 感知层：模态 token 统一骨干 |
| MoE + shared expert | GWT 专家路由 | 共享基础能力 + 特化专家 |
| 10M 上下文窗口 | KVMem 超长会话 | A2: >256K paged KV |

---

## 5. DeepSeek V4.1 Flash (DeepSeek, 2026-09)

### 架构创新

**Causal Encoder-Decoder (CED) 架构**
- 40 层 Transformer = 20 层因果编码器 + 20 层解码器
- 解码器全局 KV 缓存从编码器最终隐藏状态投影（非逐层自生）
- 激活参数：prefill 8B, decode 16B（总 552B）
- 输入密集型任务（长上下文/代理）获得廉价 half，生成获得昂贵 half

**Compressed Sparse Attention 2 (CSA2)**
- 三种静态模式：Full / Reindex / Reuse
- 跨层共享 main KV 和 indexer K，复用 Top-K 稀疏注意力索引
- 分层稀疏索引器：后续层索引限制在首个 Full 层构建的候选池内
- FP4 主 KV 缓存 (E2M1 格式)，每 16 通道一个 E4M3 scale

**极致 KV 缓存压缩**
- 全局 KV 缓存：890 bytes/token（约 DeepSeek V4 Flash 的 1/4）
- 4 代累积：V1→V4.1 压缩 437x
- SWA Bounded Replay：重放最近 n_win token 重建 SWA KV，避免 SSD 持久化

**Engram 条件记忆**
- 196B 参数条件记忆组件
- 通过 token-based lookup 稀疏访问，非全量加载

**DSpark 推测解码**
- 半自回归草稿生成 + 置信度调度验证

**MoE 配置**
- 1 shared expert + 384 routed experts/层
- 每 token 激活 6 routed experts

### NeoTrix 映射

| DeepSeek V4.1 创新 | NeoTrix 组件 | 吸收路径 |
|-------------------|-------------|---------|
| CED 非对称架构 | NT-CORE 非对称推理 | L5 认知层：输入/输出非对称计算分配 |
| CSA2 稀疏注意力 | GWT 稀疏注意力路由 | L5 认知层：Full/Reindex/Reuse 三级注意力模式 |
| FP4 KV 缓存压缩 | kv_cache_optimizer 量化 | NT-MEMORY：FP4 + 分层索引 |
| Engram 条件记忆 | NT-NEXUS 条件记忆 | L6 元认知层：稀疏条件加载 |
| SWA Bounded Replay | NT-MEMORY 滑动窗口 | L1 行动层：SWA 重建避免 SSD I/O |
| 8B/16B 非对称激活 | Cost-Aware Routing (A1) | 输入处理用小模型，生成用大模型 |

---

## 6. Qwen 3 (Alibaba, 2025-04)

### 架构创新

**Thinking/Non-thinking 双模统一**
- 单模型通过特殊 token 切换：快速响应 vs 多步推理
- 无需独立推理模型（如 o1/QwQ），同权重不同推理模式
- 思考预算机制：用户指定 thinking token 数量 (K)，控制推理深度

**QK-Norm 训练稳定性**
- RMSNorm 应用于 Q 和 K 向量
- 稳定注意力分数分布，允许更高学习率
- 消除 logit soft-capping 等技巧需求

**GQA 5:1 配比**
- 40 query heads 共享 8 KV heads
- 平衡 Llama 3 (4:1) 和 Qwen 2.5 (7:1) 之间

**强到弱蒸馏**
- 旗舰模型 (235B/32B) 知识蒸馏到小模型 (14B→0.6B)
- 保持双模能力，减少计算资源

**架构精简**
- 14B: 40 层 + 17,408 FFN（vs Qwen 2.5 14B: 48 层 + 13,824 FFN）
- 更少层数 = 更低延迟，更宽 FFN = 保持容量

### NeoTrix 映射

| Qwen 3 创新 | NeoTrix 组件 | 吸收路径 |
|------------|-------------|---------|
| Thinking/Non-thinking 双模 | NT-CORE 双过程 (MARS) | L5 认知层：快思考/慢思考切换 |
| 思考预算控制 | GWT salience 预算 | 注意力计算预算按任务动态分配 |
| QK-Norm | GWT 注意力归一化 | L5 认知层：QK 稳定性增强 |
| 强到弱蒸馏 | NT-MIND 蒸馏 | 旗舰→小模型知识迁移 |
| GQA 配比优化 | GWT 注意力头配比 | KV 缓存效率 vs 注意力质量平衡 |

---

## 7. Mistral Large 3 (Mistral AI, 2025-12)

### 架构创新

**粒度化 MoE (Granular MoE)**
- 675B total / 41B active（约 16:1 比率）
- "粒度化"：专家更细粒度分割，路由更灵活
- 训练于 3000 × H200 GPU，从零开始预训练

**原生多模态视觉编码器**
- 集成 ~2.5B 参数视觉编码器
- 原生 OCR + 文档理解，无需外部 VLM 接口
- 支持图像/PDF/图表直接输入

**极致推理优化**
- NVFP4 格式：单 8×A100/H100 节点可部署
- Eagle 推测解码：定制 draft model，3 token 投机
- TensorRT-LLM + vLLM + SGLang 推理支持

**256K 上下文 + Apache 2.0**
- 全系列 (3B–675B) 统一 256K 上下文
- Apache 2.0 开源，含 base + instruct 版本

### NeoTrix 映射

| Mistral Large 3 创新 | NeoTrix 组件 | 吸收路径 |
|---------------------|-------------|---------|
| 粒度化 MoE | NT-ACT 粒度化路由 | L1 行动层：细粒度专家路由 |
| 原生视觉编码器 | NT-WORLD 视觉感知 | L2 感知层：统一视觉编码 |
| Eagle 推测解码 | NT-IO 推测生成 | L1 行动层：draft → verify 流水线 |
| NVFP4 量化 | NT-MEMORY 量化存储 | KV 缓存 + 权重量化 |
| 全系列统一上下文 | NT-MEMORY 统一记忆接口 | 不同规模模型共享记忆接口 |

---

## 8. Phi-4 Reasoning (Microsoft, 2025-04)

### 架构创新

**小模型推理蒸馏**
- 14B 参数，SFT on o3-mini 生成的推理链
- 1.4M prompt-response 对，8.3B unique tokens
- 比 DeepSeek-R1-Distill-Llama-70B 更小但更强

**推理元技能可迁移**
- 数学/代码 SFT → 3SAT/TSP/日历规划等未见任务显著提升
- 推理是可迁移的元技能，不仅限于训练域

**GRPO 强化学习**
- Group Relative Policy Optimization
- 72,401 数学种子问题，每迭代 64 个
- 规则奖励模型（非神经网络奖励），避免 reward hacking
- RL 后：准确率提升，推理 token 增加 ~1.5x

**推理 token 机制**
- `<think>` / `</think>` 特殊 token 标记推理块
- RoPE base frequency 翻倍支持 32K 上下文
- 系统消息增强推理一致性

**训练效率**
- 32 × H100 GPU, 2.5 天训练
- 纯 SFT + 短 RL，非大规模预训练

### NeoTrix 映射

| Phi-4 Reasoning 创新 | NeoTrix 组件 | 吸收路径 |
|---------------------|-------------|---------|
| 推理蒸馏 | NT-MIND 蒸馏流水线 | 旗舰→小模型推理能力迁移 |
| 推理元技能迁移 | SEAL 技能泛化 | 跨域推理能力自动泛化 |
| GRPO RL | NT-CORE 强化学习 | 规则奖励 + 群体相对策略 |
| 推理 token 标记 | GWT 推理模式标记 | `<think>` 标记触发深度推理路径 |
| 小模型高效推理 | Cost-Aware Routing (A1) | 14B 模型处理复杂推理任务 |

---

## 9. Yi-Lightning (01.AI, 2024-10)

### 架构创新

**细粒度专家分割**
- FFN 分割为更小功能单元，减少中间隐藏维度
- 增加每 token 激活专家数，更细粒度知识分解
- 平衡策略：不过度分割以维持训练吞吐量

**三级负载均衡**
1. Switch-Transformer 负载均衡 (per-expert)
2. EP (Expert Parallel) 组级负载均衡
3. PEP (Partitioned EP) 分区级负载均衡
- 解决 All-to-All 通信中的 token 分发不平衡

**混合注意力 + 跨层 KV 共享**
- 3 层 Sliding Window Attention + 1 层 Full Attention
- Full Attention 层间共享 KV 缓存，内存需求减半
- 82.8% 内存缩减，保持长序列性能

**FP8 硬件感知设计**
- 架构精确对齐 GPU 规格
- MoE 算子：expert-parallel 策略，1200 TFLOPS/card (FP8, Hopper)
- 自研高性能算子，>100% 执行效率提升

**RAISE 安全框架**
- 4 组件：预训练安全 + 后训练安全 + 服务安全 + 持续监控

### NeoTrix 映射

| Yi-Lightning 创新 | NeoTrix 组件 | 启示路径 |
|------------------|-------------|---------|
| 细粒度专家分割 | NT-ACT 粒度化路由 | 更灵活的专家组合 |
| 三级负载均衡 | GWT 负载均衡 | Per-expert → EP-group → partitioned |
| 混合注意力 (3 SWA + 1 Full) | GWT 混合注意力模式 | 本地/全局注意力混合策略 |
| 跨层 KV 共享 | kv_cache_optimizer 跨层复用 | 连续层共享 KV，内存减半 |
| FP8 硬件感知 | NT-PHYSICAL 硬件对齐 | 架构设计对齐目标硬件 |
| RAISE 安全框架 | NT-SHIELD 安全框架 | 全生命周期安全 |

---

## 10. Grok 3 (xAI, 2025-02)

### 架构创新

**Think + DeepSearch 双推理模式**
- Think: CoT 链式推理，数秒到数分钟，自我纠错
- DeepSearch: 代理搜索 + 实时网络查询 + 事实综合
- RL 大规模训练优化推理链

**Colossus 超算训练**
- ~200,000 × NVIDIA H100 GPU
- 10x Grok 2 计算量
- NVIDIA Spectrum-X Ethernet 互联

**1M 上下文窗口**
- 8x 前代模型
- LOFT (128K) 长上下文 RAG 达 SOTA

**MoE + 稀疏注意力**
- 疑似 ~1.5T 参数，MoE 架构
- 选择性注意力：减少计算开销
- 高级位置编码：可学习位置嵌入

**多模态能力**
- 文本 + 图像 + 音频 + 视频理解
- 集成语音模式

### NeoTrix 映射

| Grok 3 创新 | NeoTrix 组件 | 吸收路径 |
|------------|-------------|---------|
| Think 模式 | NT-CORE 推理模式 | L5 认知层：CoT 深度推理 |
| DeepSearch 代理 | NT-ACT 代理搜索 | L1 行动层：实时网络搜索代理 |
| 1M 上下文 | KVMem 超长会话 | A2: paged KV for >256K |
| 大规模 RL | NT-CORE 强化学习 | 推理策略自我优化 |
| 稀疏注意力 | GWT 稀疏注意力 | 选择性关注关键信息 |

---

## 跨模型架构趋势总结

### 趋势 1: MoE 成为主流
- **采用者**: Gemini 2.5, Llama 4, DeepSeek V4.1, Qwen 3, Mistral Large 3, Yi-Lightning, Grok 3
- **NeoTrix 吸收**: GWT salience 路由 + Cost-Aware Routing (Axiom A1)
- **启示**: 每 token 激活子集参数，解耦容量与计算成本

### 趋势 2: 原生多模态融合
- **采用者**: GPT-4o, Gemini 2.5, Llama 4, Mistral Large 3, Grok 3
- **NeoTrix 吸收**: PerceptionBridge 统一多模态感知
- **启示**: 从 CLIP 分阶段 → 端到端联合训练

### 趋势 3: 推理时计算可控
- **采用者**: Gemini 2.5 (Thinking Budget), Qwen 3 (思考预算), DeepSeek V4.1 (CED 非对称), Grok 3 (Think 模式)
- **NeoTrix 吸收**: GWT salience 预算 + SEAL pipeline 预算控制
- **启示**: 推理深度成为可调参数

### 趋势 4: KV 缓存极致压缩
- **采用者**: DeepSeek V4.1 (890B/tok, FP4), Yi-Lightning (跨层共享), Llama 4 (iRoPE)
- **NeoTrix 吸收**: kv_cache_optimizer 量化 + 分层存储
- **启示**: 长上下文的关键瓶颈在 KV 缓存

### 趋势 5: 小模型推理能力涌现
- **采用者**: Phi-4 Reasoning (14B), Qwen 3 (0.6B-32B), Llama 4 Scout (17B active)
- **NeoTrix 吸收**: NT-MIND 蒸馏 + Cost-Aware Routing
- **启示**: 蒸馏 + RL 让小模型获得大模型推理能力

### 趋势 6: 安全与对齐内化
- **采用者**: Claude 3.5 (Constitutional AI), Yi-Lightning (RAISE), GPT-4o (Preparedness Framework)
- **NeoTrix 吸收**: NT-GOVERNANCE 合规引擎 + NT-SHIELD 安全框架
- **启示**: 安全从后处理 → 训练时内化

---

## NeoTrix 吸收优先级

### P0 — 立即吸收
1. **Thinking Budget 机制** (Gemini 2.5 + Qwen 3) → GWT salience 预算控制
2. **KV 缓存极致压缩** (DeepSeek V4.1 CSA2 + FP4) → kv_cache_optimizer 升级
3. **CED 非对称架构** (DeepSeek V4.1) → 输入/输出非对称计算

### P1 — 近期吸收
4. **细粒度 MoE 路由** (Yi-Lightning 三级均衡 + Mistral 粒度化) → GWT 路由升级
5. **混合注意力模式** (Yi-Lightning 3 SWA + 1 Full) → GWT 注意力模式
6. **推理元技能迁移** (Phi-4 Reasoning) → SEAL 蒸馏流水线

### P2 — 中期吸收
7. **早期多模态融合** (Llama 4 Early Fusion) → PerceptionBridge 升级
8. **iRoPE 无限上下文** (Llama 4) → 位置编码自适应
9. **DeepSearch 代理** (Grok 3) → NT-ACT 代理搜索

### P3 — 远期吸收
10. **Engram 条件记忆** (DeepSeek V4.1) → NT-NEXUS 条件加载
11. **RAISE 全生命周期安全** (Yi-Lightning) → NT-SHIELD 安全框架
12. **DSpark 推测解码** (DeepSeek V4.1) → NT-IO 推测生成

---

## 关键数值对比

| 模型 | 最大上下文 | KV 缓存效率 | 激活/总参数比 | 推理延迟标杆 |
|------|-----------|------------|-------------|------------|
| GPT-4o | 128K | 未公开 | 全量/全量 | 232ms (语音) |
| Claude 3.5 Sonnet | 200K | 未公开 | 全量/全量 | 未公开 |
| Gemini 2.5 Pro | 1M+ | 未公开 | 部分/部分 | 未公开 |
| Llama 4 Scout | 10M | 未公开 | 17B/109B (15.6%) | 单 H100 可运行 |
| DeepSeek V4.1 Flash | 1M | 890 B/tok | 8B-16B/552B (1.5-2.9%) | 极低 (输入密集型) |
| Qwen 3 235B | 256K | 未公开 | 22B/235B (9.4%) | 未公开 |
| Mistral Large 3 | 256K | 未公开 | 41B/675B (6.1%) | 单 8×H200 |
| Phi-4 Reasoning | 32K | 未公开 | 全量/14B | 32×H100 2.5 天 |
| Yi-Lightning | 16K | 82.8% 内存缩减 | 部分/未公开 | 1200 TFLOPS/card |
| Grok 3 | 1M (声称) | 未公开 | 部分/~1.5T | 未公开 |

---

*Generated: 2026-09-11 | Cycle: 244 | Sources: 10 模型技术报告 + 公开论文*
