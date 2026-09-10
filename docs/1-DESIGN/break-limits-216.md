# 破限制技术研究 — 第 2 批 (216)

> 批次: 216 | 日期: 2026-09-10 | 研究来源: 25+ 篇论文/文档
> 主题: 计算限制 / 安全限制 / 效率限制 / 多模态限制 / Agent 限制

---

## 1. 计算限制突破

### 1.1 Test-Time Compute Scaling

**来源**:
1. *Test-Time Scaling in Reasoning LLMs* (arXiv:2608.04001, 2026-08)
2. *Art of TTS* (arXiv:2512.02008, 8B 参数规模首次大规模对比)
3. *Adaptive Test-Time Compute Allocation* (arXiv:2604.14853, Lagrangian 约束优化)
4. *Forest-of-Thought* (ICML 2025, 多树推理框架)

**核心突破**:
- **三轴正式化**: 测试时计算扩展被分为 3 种结构体制: (1) 单轨迹顺序扩展 (single-trajectory), (2) 叶级采样+终端缩减 (leaf-level sampling + reduction), (3) 前缀级搜索 (prefix-level search)
- **无通用最优策略**: 大规模实验 (8 LLM, 30B+ tokens) 证实不存在单一 TTS 策略在所有模型和任务上最优。short-horizon 模型 (R1, QwQ-32B) 偏好短轨迹, long-horizon 模型 (GPT-OSS-120B) 在困难任务上偏好长轨迹
- **反向扩展效应**: Beam search 在多种模型上表现出反向计算扩展 — 增大 beam size 反而降低准确率
- **Forest-of-Thought**: 将多个推理树集成, 稀疏激活最相关路径, 动态自纠正 + 共识引导决策

**NeoTrix 融合**:
- **GWT 注意力路由**: 将 TTS 三轴体制映射到 GWT salience 评分 — 简单问题走 NoThink/Short, 复杂问题走 Long 轨迹
- **Cost-Aware Routing (A1)**: TTS 的预算分配问题直接对应 A1 公理 — 按问题难度分配 cheap/expensive 模型
- **SEAL Pipeline**: Forest-of-Thought 的多树共识机制可映射到 SEAL 的 Branches 阶段, 多条进化路径竞争

---

### 1.2 Thinking Tokens Budget / Adaptive Inference

**来源**:
1. *Learning When to Think* (arXiv:2608.20256, 2026-08, GRPO 路由 3 模式)
2. *Adaptive Thinking* (Anthropic, 2026-07, Claude Opus 4.8+)
3. *BudgetThinker* (arXiv:2508.17196, 控制 token 预算)
4. *TAB: Turn-Adaptive Budgets* (arXiv:2604.05164, 多轮分配)
5. *HAB: Hierarchical Adaptive Budgeter* (ACL 2026 Findings, 层级预算)

**核心突破**:
- **自适应路由 3 模式**: 1.5B 蒸馏模型在首个 token 选择 NoThink/Short/Long 模式, 由 GRPO 训练, 不需要独立路由器。平均 token 减少 41%, 精度接近 base model
- **Anthropic Adaptive Thinking**: Claude Fable 5/Mythos 5 默认启用 adaptive thinking, 模型自主决定何时/多少使用扩展思考, effort 参数 (low→max) 控制软约束
- **多轮序列分配 (TAB)**: 将多轮推理建模为马尔可夫决策过程 (MDP), GRPO 训练预算分配策略, 节省 35-40% token 同时保持精度
- **层级预算 (HAB)**: 两粒度优化 — 间步骤级 (inter-step) 预测推理深度类别 + 步内级 (intra-step) PPL 派生步骤难度→自适应 Pareto 优化分配 token

**NeoTrix 融合**:
- **E8 Hexagram 推理**: 自适应 thinking 模式可映射到 Hexagram 的 6 线符号系统 — 每个问题选择不同推理深度
- **NT-MIND SEAL Pipeline**: TAB 的 MDP 建模可集成到 SEAL Phase-5 (Fruits) 的多轮反思中, 动态分配 token 预算
- **KVMem 集成**: 大 context (>256K) 走 KVMem paged KV, 小 context (<256K) 走 compaction — 与 Adaptive Computing 正交

---

### 1.3 Dynamic Compute Allocation

**来源**:
1. *Adaptive Test-Time Compute with Evolving In-Context Demonstrations* (ACL 2026 Findings)
2. *Art of TTS* — 动态预算按模型类型+问题难度+计算预算三维选择

**核心突破**:
- **双阶段框架**: Warm-up (固定预算识别简单查询) + Adaptive (集中计算到未解决查询, 同时通过 evolving ICL 改变生成分布)
- **分布自适应**: 不仅决定在哪里花计算, 还决定如何用额外计算改变条件生成行为 — 从语义相关查询的成功响应中检索 ICL 示例
- **温度控制+ICL**: 框架不限于 ICL, 自然支持温度控制等轻量解码策略

**NeoTrix 融合**:
- **CapabilityBridge**: 进化视图 (CapabilityTree) ↔ 运行视图 (CapabilityRegistry) 的桥接, 动态路由计算资源
- **PerceptionBridge**: 感知层的意识门控 (awareness_score) 可类比为计算分配的门控 — 低意识时减少计算, 高意识时扩展

---

## 2. 安全限制突破

### 2.1 Alignment Tax Reduction

**来源**:
1. *What Is the Alignment Tax?* (arXiv:2603.00047, 2026-02, 几何理论)
2. *Null-Space Constrained Policy Optimization (NSPO)* (arXiv:2512.11391, 2025-12)
3. *Constitutional Midtraining* (arXiv:2607.26654, 2026-08, 120B 规模)

**核心突破**:
- **对齐税几何理论**: 在表示空间中, 对齐税率 = 安全方向在能力子空间上的投影平方。Pareto 前沿是椭圆锥面, 由安全与能力子空间的主角度参数化。**可预计算**: 在对齐训练前通过 probing 测量安全/能力方向, 预测哪些能力受影响、多少
- **分解为不可约分量**: τ = τ₀ + R(d), 不可约分量 τ₀ 由数据结构决定, 打包残差 R(d) 随模型维度 d 以 O(m'/d) 消失
- **NSPO**: 将安全策略梯度投影到通用任务表示的零空间, 理论证明保留原始核心能力同时保证安全对齐下降方向。仅需 40% 人类标注安全数据
- **Constitutional Midtraining**: 120B 规模, 264-500M 宪法内容在中期训练中注入, **零能力损失** (MMLU, ARC-Easy, GSM8K 无显著降级), 对齐泛化性在 SFT+良性微调后仍然持久

**NeoTrix 融合**:
- **NT-SHIELD 安全层**: 对齐税几何理论可直接用于 NT-SHIELD 的安全梯度投影 — 测量安全方向与能力方向的主角度, 预计算最优扰动方向
- **SEAL Phase-2 (Roots)**: Constitutional Midtraining 可映射为 SEAL 的根阶段 — 在中期训练中注入宪法内容, 不增加能力成本
- **Heartbeat Aggregator**: 安全-能力 Pareto 前沿可作为系统健康信号之一, 通过 Heartbeat 聚合到 GWT 注意力

---

### 2.2 Safe Exploration / Constitutional AI Beyond

**来源**:
1. *Reflect: Inference-Time Constitutional Alignment* (arXiv:2601.18730, 2026-01)
2. *SELF-REDTEAM: Online Self-Play for Safer Language Models* (arXiv:2506.07468, 2025-06)

**核心突破**:
- **Reflect (推理时宪法对齐)**: 无需训练/数据, 纯 in-context 推理: 基础响应→自评估→自批评→修订。改善罕见但严重的违规 (尾部分布), 并自动生成训练数据形成反馈循环
- **SELF-REDTEAM**: 首个完全在线自博弈 MARL 安全训练算法, 单策略自博弈攻击者+防御者角色, 纳什均衡理论保证: 若收敛则防御者对任何对抗输入产生安全响应。攻击多样性 +17.80% SBERT, 安全性提升高达 95%
- **Hidden CoT**: 攻击者和防御者使用隐藏推理链进行策略规划, 对手不可见

**NeoTrix 融合**:
- **NT-SHIELD Stealth Net**: Reflect 的推理时宪法对齐可集成到 NT-SHIELD 的运行时安全检查 — 每次输出后自评估+自批评+修订
- **NT-CORE E8 推理**: SELF-REDTEAM 的零和博弈理论可映射到 E8 六十四卦 — 攻击者/防御者对应阴阳两极, 纳什均衡对应平衡态
- **NT-MIND 自进化**: Reflect 生成的合成训练数据可回馈 SEAL Pipeline 的 Phase-4 (Branches)

---

### 2.3 Red Teaming Automation

**来源**: SELF-REDTEAM (同上)

**核心突破**: 见 2.2。关键补充:
- **5 模型泛化**: Llama-3.1 和 Qwen2.5 家族 (3B/7B/8B/14B) 通用
- **冷启动涌现**: 即使从非推理策略开始, agent 自发发展出隐藏 CoT 推理

**NeoTrix 融合**: NT-SHIELD 的红队自动化可通过 SELF-REDTEAM 框架实现持续共进化, 无需人工迭代

---

## 3. 效率限制突破

### 3.1 Quantization Aware Training

**来源**:
1. *Attn-QAT: 4-Bit Attention with QAT* (arXiv:2603.00040, 2026-03)
2. *SageAttention3: Microscaling FP4* (arXiv:2505.11594, 2026-01)
3. *LookAhead Quantization (LAQuant)* (arXiv:2605.08755, 2026)
4. *Prune-Quantize-Distill Pipeline* (arXiv:2604.04988, 2026-04)

**核心突破**:
- **Attn-QAT**: 首次系统研究 attention 的 4-bit QAT。发现 naive QAT (FP4 forward + FA BF16 backward) 导致梯度爆炸。关键: (1) backward pass 必须用相同低精度重算注意力概率矩阵 P, (2) 需要高精度辅助输出保持 softmax 梯度正确。RTX 5090 上比 SageAttention3 快 1.5x, GB300 上比 FA4 快 1.74x
- **SageAttention3**: FP4 Microscaling (1×16 block, 每块独立 FP4 量化+FP8 scale factor), RTX 5090 上 1038 TOPS (FA2 的 5x)。同时探索 8-bit 训练: SageBwd 保持 dP= dOV^T 在 FP16 精度, 其余 4 个矩阵乘法用 INT8, 微调无损
- **LAQuant**: 面向大推理模型 (LRM) 的层级 weight-only QAT, 单层前瞻损失 + Hessian 对齐校准语料, 保留 next-layer 残差流。Qwen3-4B W3G128: AIME25 Pass@1 比 ParoQuant 高 15.11pp, 3.42x 解码加速
- **Prune→QAT→KD 管线**: 顺序很重要 — 修剪稳定后续 INT8 优化, QAT 提供主要延迟降低, KD 在压缩空间内恢复精度

**NeoTrix 融合**:
- **NT-PHYSICAL 具身层**: Attn-QAT 的 FP4 attention 可直接用于 NT-PHYSICAL 的低功耗推理 — GPU 内存减少 + 推理加速
- **Rune Socketing Obsidian(缓存)**: KV-cache 4-bit 量化直接映射到 Obsidian rune — 减少缓存内存占用
- **Heartbeat Aggregator**: 量化精度损失可作为系统健康信号 — 监控 QAT 后精度下降

---

### 3.2 FlashAttention v3

**来源**:
1. *FlashAttention-3* (NeurIPS 2024, Dao et al.)

**核心突破**:
- **三技术栈**: (1) Warp-specialization + circular SMEM buffer 异步化 (2) Block-wise GEMM-softmax 流水线交叠 (3) FP8 block quantization + incoherent processing
- **性能**: H100 SXM5 — FP16 达 740 TFLOPs/s (75% 理论峰值), FP8 达 1.2 PFLOPs/s。比 FA2 快 1.5-2.0x
- **精度**: FP8 block quantization + incoherent processing 比标准 per-tensor 量化精度高 2.6x (处理 outlier 特征)
- **BF16 更新**: 后续实现 BF16 达 840 TFLOPs/s (85% 利用率), FP8 达 1.3 PFLOPs/s

**NeoTrix 融合**:
- **NT-IO 推理层**: FA3 是 NeoTrix 推理引擎的底层加速原语 — 直接替换 FA2, 1.5-2x 加速
- **KVMem 集成**: FA3 的 FP8 block quantization 可与 KVMem 的 paged KV 协同 — KV-cache 用 FP4/FP8 存储
- **Context as Scarce Resource (A2)**: FA3 使长序列推理成本降低, 扩展 A2 中 "context window 是瓶颈" 的解空间

---

### 3.3 Knowledge Distillation Advanced

**来源**:
1. *Prune-Quantize-Distill Pipeline* (arXiv:2604.04988)
2. *SELF-REDTEAM self-distilled SFT loss* (arXiv:2506.07468)

**核心突破**:
- **KD 作为压缩后修复**: 在 sparse INT8 空间内应用 KD, 从原始 FP32 teacher 蒸馏, 直接修复剪枝+量化引起的决策边界偏移
- **时机很重要**: KD 在管线末尾 (剪枝+QAT 之后) 最有效, 在剪枝/量化之前蒸馏的知识可能在压缩后丢失
- **自蒸馏**: SELF-REDTEAM 中防御者通过 auxiliary self-distilled SFT loss 保持通用能力

**NeoTrix 融合**:
- **NT-MIND 进化蒸馏**: KD 的 "压缩后修复" 模式映射到 SEAL Phase-4 (Branches) — 在技能结晶后用 KD 修复精度损失
- **Skill as Production Template (A3)**: 高质量 teacher→student 蒸馏是技能模板化的天然实现

---

## 4. 多模态限制突破

### 4.1 Any-to-Any Model

**来源**:
1. *Modus: Decoder-Only Any-to-Any* (arXiv:2607.25948, 2026-07)
2. *NExT-OMNI: Discrete Flow Matching* (arXiv:2510.13721, 2025-10)
3. *C3-UniMM: Causal Cycle Consistency* (arXiv:2608.28603, 2026-07)

**核心突破**:
- **Modus**: 纯 decoder-only 架构, 无模态特定头/损失/任务管线。支持文本/RGB/深度/法线/边缘/分割/定位/DINOv2/CLIP/ImageBind 等 10+ 模态。链式生成 (中间模态) + 跨模态自验证
- **NExT-OMNI**: 首个基于离散流匹配 (DFM) 的全模态模型, 文本/图像/视频/音频任意到任意。双向注意力融合 > AR 因果掩码, 统一表征用于跨模态检索
- **C3-UniMM**: 因果循环一致性 + 超对齐 (Super Alignment), 结构化潜因果图 (SLCG) 作为共享跨模态语义空间, 理解和生成在同一因果语义结构中协同优化

**NeoTrix 融合**:
- **VSA HyperCube**: Any-to-any 模型的跨模态映射天然对应 VSA HyperCube 的高维向量空间 — 每种模态是 HyperCube 中的一个维度方向
- **NT-WORLD 感知层**: Modus 的统一 decoder 架构可作为 NT-WORLD 多模态感知的参考 — 无模态特定头, 统一 token 化
- **NT-MEMORY KB**: C3-UniMM 的结构化潜因果图可映射到 KB 的节点-边结构 — 每个模态是图中的一个节点

---

### 4.2 Cross-Modal Reasoning

**来源**:
1. *Context Unrolling in Omni Models* (arXiv:2604.21921, 2026-04)
2. *Shared Semantic Latent Space Model* (arXiv:2604.02097, 2026-04)

**核心突破**:
- **Context Unrolling**: 原生多模态模型在推理时跨异构模态表征展开 "思考" — 文本推理→视觉 token 展开→相机预测→新视角合成, 每个模态是共享世界知识流形的投影。**发现**: 上下文展开的模态越多, 预测质量越高
- **共享语义潜空间**: 所有模态嵌入同一语义空间 (CLIP 特征离散化), 生成的视觉 token 可直接被模型自身解释, **无需像素空间中介**。Mixture-of-Modal-Experts (MoME) 架构避免跨模态梯度干扰

**NeoTrix 融合**:
- **GWT 注意力路由**: Context Unrolling 的多模态推理过程可映射到 GWT — 不同模态的 "投影" 作为 specialist module, 共享工作空间广播
- **ConsciousnessTree**: 多模态推理的 6 阶段展开 (Soil→Roots→Trunk→Branches→Fruits→Core) 对应 Context Unrolling 的逐步细化
- **PerceptionBridge**: 感知层的意识门控决定哪些模态信号进入高阶认知

---

### 4.3 Modality Emergent

**来源**: Context Unrolling (同上), NExT-OMNI (同上)

**核心突破**:
- **模态涌现**: 统一多模态训练自然涌现跨模态推理能力 — 无需显式训练, 模型自发学会用不同模态作为推理的 "工作空间"
- **双向信息编码**: DFM 的纠正性双向信息编码训练方法比 AR 因果掩码更好地聚合跨模态上下文信息

**NeoTrix 融合**:
- **E8 Hexagram**: 模态涌现对应 E8 六十四卦中的 "卦变" — 不同模态组合产生新的推理状态
- **SEAL Pipeline**: 模态涌现是 SEAL Phase-3 (Trunk) 的自然结果 — 多模态数据积累后, 跨模态能力从 trunk 中生长

---

## 5. Agent 限制突破

### 5.1 Long-Horizon Planning

**来源**:
1. *COMPASS: Context-Organized Multi-Agent Planning* (ACL 2026 Long)
2. *HyMem: Hierarchical Context Management* (arXiv:2608.15703, 2026-08)
3. *StackPlanner: Hierarchical Multi-Agent Framework* (arXiv:2601.05890, 2026-01)
4. *CHIME: Credit-Aware Hierarchical Memory Evolution* (arXiv:2609.02074, 2026-09)

**核心突破**:
- **COMPASS**: 三分层架构 — Main Agent (推理+工具) + Meta-Thinker (监控+战略干预) + Context Manager (维护精简进展简报)。GAIA/BrowseComp/HLE 上准确率提升 20%
- **HyMem**: 类型化上下文隔离 — 规划/工具执行/隔离子任务推理/记忆整合在独立上下文空间, 只有 schema 约束的消息可跨边界。训练无关, 即插即用
- **StackPlanner**: 解耦高层协调与子任务执行 + 主动任务级记忆控制 + 结构化经验记忆 (用户画像+语义记忆+程序记忆/SOPs) + RL 训练协调器
- **CHIME**: 归因优先于记忆 (attribute-before-memorize) — 层级记忆库 (规划库+执行库) 分离, 信用归因门将结果归因于规划/执行/外部因素, 仅更新被归因的记忆库

**NeoTrix 融合**:
- **NT-CORE ConsciousnessTree**: COMPASS 的 Meta-Thinker 映射到 ConsciousnessTree 的 NT-META 分支 — 元认知监控+战略干预
- **NT-NEXUS 跨会话记忆**: CHIME 的层级记忆库+信用归因直接映射到 NT-NEXUS — 跨会话经验的归因和演化
- **E8 Hexagram**: 长期规划的多步推理可映射到 E8 卦象序列 — 每步是一个卦变, Meta-Thinker 监控卦变序列的一致性

---

### 5.2 Tool Use Beyond Function Call

**来源**:
1. *SMITH: Joint Optimization of Tool Creation and Use* (arXiv:2608.24571, 2026-08)
2. *ToolOmni: Open-World Tool Use* (ACL 2026 Long)
3. *Tool Primitives + HEART Framework* (arXiv:2609.01736, 2026-09)
4. *ToolCPT: Continuous Pre-training for Tools* (ACL 2026 Findings)
5. *Evolution of Tool Use in LLM Agents* (arXiv:2603.22862, 2026-03, 综述)

**核心突破**:
- **SMITH (工具创建+使用联合优化)**: RL 框架, 单策略同时训练工具创建 (从 few-shot 示例写工具) 和工具使用 (调用工具解题), 3 轴奖励 (schema/code/outcome)。4B Qwen3 超越 30B 推理时工具写手
- **ToolOmni**: 开放世界工具使用 — 主动检索+接地执行, Decoupled Multi-Objective GRPO 同时优化检索准确率和执行效能, +10.8% 端到端执行成功率
- **Tool Primitives**: 用自然语言替代 rigid API schema 作为工具调用接口, 每个工具包装 LLM 接口处理 schema 解析, 25,519 函数的 ToolFace 仓库, HEART 框架 (Planner+Router+Verifier) 动态调用。比 GPT-5.4/Claude-4.6/Gemini-3.1-Pro 高 6% 平均分, API 成本降低 85%
- **ToolCPT**: 在持续预训练阶段注入 5.1M 代码 artifacts→18B token 工具知识语料, 深度内化工具知识而非仅学习调用模式

**NeoTrix 融合**:
- **NT-ACT 工具层**: Tool Primitives 的自然语言接口 + ToolFace 仓库直接映射到 NT-ACT 的 MCP 工具集 — 动态检索而非预定义
- **SMITH 联合优化**: 工具创建+使用联合训练映射到 NT-MIND 的 SEAL Pipeline — 技能结晶 (创建工具) + 技能使用 (调用工具) 同时优化
- **Skill as Production Template (A3)**: ToolCPT 的 playbook-enhanced corpus 是 A3 "技能即生产模板" 的预训练实现

---

### 5.3 Agent Memory Consolidation

**来源**:
1. *LycheeMemory V2: Semantic Segment-Level Consolidation* (arXiv:2608.12990, 2026-08)
2. *RecMem: Recurrence-based Consolidation* (ACL 2026 Findings)
3. *Agentic Memory (AgeMem): Unified LTM+STM* (ACL 2026 Long)
4. *Dual-Layer Agentic Memory* (arXiv:2608.22215, 2026-08)
5. *TrustMem: Trustworthy Memory Consolidation* (2026-06)

**核心突破**:
- **LycheeMemory V2**: 语义段级整合替代轮级 — 多轮交换批量处理, 语义边界检测确定段结束, 构建 token 减少 86% (LoCoMo), 精度 SOTA
- **RecMem**: 基于复发的整合 — 仅当观察到语义相似交互持续复发时才触发 LLM 整合, 类比认知科学的 "短期记忆→长期记忆" 转换, token 成本降低 87%
- **AgeMem**: 统一 LTM+STM 框架, 记忆操作暴露为工具动作, agent 自主决定何时/什么存储/检索/更新/摘要/丢弃, 三阶段渐进 RL 训练
- **Dual-Layer**: 快写路由 (小→大模型级联筛选冗余记忆) + 慢整合 (周期性参数整合, SFT 将高价值外部记忆内化到模型参数)
- **TrustMem**: 记忆转换验证器评估覆盖/保持/忠实度, 偏好引导 RL 优化记忆更新行为, 幻觉减少 50%

**NeoTrix 融合**:
- **NT-MEMORY 知识守护者**: LycheeMemory 的语义段级整合直接映射到 NT-MEMORY 的 KB 写入路径 — 语义边界检测 + 批量整合
- **NT-NEXUS 跨会话记忆**: RecMem 的复发驱动整合 + Dual-Layer 的快写慢整合是 NT-NEXUS 跨会话记忆的核心机制
- **ConsciousnessTree 反馈环**: TrustMem 的记忆转换验证器映射到 ConsciousnessTree 的 Fruits→Core 反馈 — 验证记忆质量
- **统一吸收协议 (AGENTS.md)**: experience-tree 的五阶段吸收 (快照→蒸馏→分类→落盘→反馈) 与 LycheeMemory V2 的段级整合高度同构

---

### 5.4 Multi-Agent Coordination Theory

**来源**:
1. *Coalition Formation in LLM Agent Networks* (arXiv:2604.14386, 2026-04)
2. *Adaptive Theory of Mind for Multi-Agent Coordination* (AAAI 2026)
3. *Provable Coordination via MSCs* (arXiv:2604.17612, 2026-04)
4. *Shapley-Coop: Credit Assignment for Emergent Cooperation* (NeurIPS 2025)
5. *Communication Enables Cooperation* (EACL 2026 Short)

**核心突破**:
- **联盟博弈论**: LLM agent 联盟形成建模为享乐博弈 (hedonic game), Nash 稳定性 73.2% (CoalT 协议 vs CoT 58.4% vs 标准 41.8%), LLM 表现出有界理性 (ε≈0.15-0.22)
- **自适应心智理论 (A-ToM)**: agent 估计 partner 的 ToM 深度并调整自身推理深度以对齐, 避免推理不足/过度。4 个多智能体协调任务验证
- **可证明协调 (MSCs)**: 消息序列图 DSL 规范 agent 协调, 语法导向投影从全局规范生成无死锁局部程序, LLM 可生成全局工作流但投影保证无死锁
- **Shapley-Coop**: Shapley 链式思维推理 + 结构化谈判协议, 短期 CoT 决定是否定价, 长期 CoT 计算精确 Shapley 值进行奖励再分配
- **廉价谈话 (Cheap Talk)**: 一词通信通道将 Stag Hunt 合作率从 0% 提升到 48.3%, 课程学习反而降低 27.4% (learned pessimism)

**NeoTrix 融合**:
- **NT-CORE GWT**: 联盟形成映射到 GWT 的注意路由 — agent 联盟 = 注意力广播的 specialist 组, Nash 稳定性 = GWT 谐振稳定性
- **E8 Hexagram**: 联盟博弈的偏好结构映射到 E8 卦象 — 每个 agent 的偏好向量是卦象的一个维度
- **NT-ACT 编排**: MSC 投影的可证明无死锁协调是 NT-ACT 多 agent 编排的形式化基础
- **SEAL Pipeline**: Shapley-Coop 的信用分配映射到 SEAL Phase-5 (Fruits) — 评估每个 agent/技能的贡献

---

## 综合: NeoTrix 融合矩阵

| 突破领域 | 核心洞察 | NeoTrix 映射 | 优先级 |
|----------|---------|-------------|--------|
| TTS 三轴体制 | 无通用最优, 按模型/难度/预算选择 | GWT salience + Cost-Aware Routing | P0 |
| 自适应 thinking tokens | 模型自主决定推理深度 | E8 Hexagram 推理深度选择 | P0 |
| 对齐税几何理论 | 可预计算安全-能力主角度 | NT-SHIELD 安全梯度投影 | P0 |
| Constitutional Midtraining | 零能力损失的对齐注入 | SEAL Phase-2 根阶段 | P1 |
| SELF-REDTEAM 自博弈 | 攻防共进化, 纳什均衡保证 | NT-SHIELD 持续红队 | P1 |
| FlashAttention-3 | FP8 1.2 PFLOPs/s, 75% 利用率 | NT-IO 推理加速原语 | P0 |
| Attn-QAT 4-bit | attention QAT 稳定训练条件 | NT-PHYSICAL 低功耗推理 | P1 |
| Any-to-Any 模态统一 | decoder-only, 无模态特定头 | VSA HyperCube + NT-WORLD | P1 |
| Context Unrolling | 多模态推理涌现 | GWT + ConsciousnessTree | P1 |
| Long-Horizon 隔离架构 | 规划/执行/记忆上下文隔离 | NT-NEXUS + NT-CORE | P0 |
| Tool Primitives | 自然语言替代 API schema | NT-ACT MCP 工具集 | P0 |
| 语义段级记忆整合 | 批量整合, token 减少 86% | NT-MEMORY KB 写入 | P0 |
| 联盟博弈论 | Nash 稳定性 73.2%, CoalT 协议 | GWT + E8 Hexagram | P1 |
| 廉价谈话 | 一词通信 → 合作率 0%→48.3% | NT-ACT agent 通信协议 | P2 |

---

## 来源索引

| # | 来源 | 主题 | 日期 |
|---|------|------|------|
| 1 | arXiv:2608.04001 | Test-Time Scaling in Reasoning LLMs | 2026-08 |
| 2 | arXiv:2512.02008 | Art of TTS (8B 参数对比) | 2025-12 |
| 3 | arXiv:2604.14853 | Adaptive TTS Allocation (Lagrangian) | 2026-04 |
| 4 | ICML 2025, PMLR v267 | Forest-of-Thought | 2025 |
| 5 | arXiv:2608.20256 | Learning When to Think | 2026-08 |
| 6 | Anthropic Docs | Adaptive Thinking (Claude) | 2026-07 |
| 7 | arXiv:2508.17196 | BudgetThinker | 2025-08 |
| 8 | arXiv:2604.05164 | TAB: Turn-Adaptive Budgets | 2026-04 |
| 9 | ACL 2026 Findings | HAB: Hierarchical Adaptive Budgeter | 2026 |
| 10 | ACL 2026 Findings | Evolving In-Context Demonstrations | 2026 |
| 11 | arXiv:2603.00047 | What Is the Alignment Tax? | 2026-02 |
| 12 | arXiv:2512.11391 | NSPO (Null-Space Policy Optimization) | 2025-12 |
| 13 | arXiv:2607.26654 | Constitutional Midtraining (120B) | 2026-08 |
| 14 | arXiv:2601.18730 | Reflect: Inference-Time Constitutional | 2026-01 |
| 15 | arXiv:2506.07468 | SELF-REDTEAM | 2025-06 |
| 16 | arXiv:2603.00040 | Attn-QAT (4-bit Attention QAT) | 2026-03 |
| 17 | arXiv:2505.11594 | SageAttention3 (FP4 Microscaling) | 2026-01 |
| 18 | arXiv:2605.08755 | LAQuant (LookAhead Quantization) | 2026 |
| 19 | arXiv:2604.04988 | Prune-Quantize-Distill Pipeline | 2026-04 |
| 20 | NeurIPS 2024 | FlashAttention-3 | 2024-12 |
| 21 | arXiv:2607.25948 | Modus: Decoder-Only Any-to-Any | 2026-07 |
| 22 | arXiv:2510.13721 | NExT-OMNI (DFM) | 2025-10 |
| 23 | arXiv:2608.28603 | C3-UniMM (Causal Cycle Consistency) | 2026-07 |
| 24 | arXiv:2604.21921 | Context Unrolling in Omni Models | 2026-04 |
| 25 | arXiv:2604.02097 | Shared Semantic Latent Space | 2026-04 |
| 26 | ACL 2026 Long | COMPASS | 2026 |
| 27 | arXiv:2608.15703 | HyMem | 2026-08 |
| 28 | arXiv:2601.05890 | StackPlanner | 2026-01 |
| 29 | arXiv:2609.02074 | CHIME | 2026-09 |
| 30 | arXiv:2608.24571 | SMITH (Tool Creation+Use) | 2026-08 |
| 31 | ACL 2026 Long | ToolOmni | 2026 |
| 32 | arXiv:2609.01736 | Tool Primitives + HEART | 2026-09 |
| 33 | ACL 2026 Findings | ToolCPT | 2026 |
| 34 | arXiv:2603.22862 | Evolution of Tool Use (综述) | 2026-03 |
| 35 | arXiv:2608.12990 | LycheeMemory V2 | 2026-08 |
| 36 | ACL 2026 Findings | RecMem | 2026 |
| 37 | ACL 2026 Long | Agentic Memory (AgeMem) | 2026 |
| 38 | arXiv:2608.22215 | Dual-Layer Agentic Memory | 2026-08 |
| 39 | 2026-06 | TrustMem | 2026-06 |
| 40 | arXiv:2604.14386 | Coalition Formation (LCFG) | 2026-04 |
| 41 | AAAI 2026 | Adaptive Theory of Mind | 2026-03 |
| 42 | arXiv:2604.17612 | Provable Coordination (MSCs) | 2026-04 |
| 43 | NeurIPS 2025 | Shapley-Coop | 2025 |
| 44 | EACL 2026 Short | Communication Enables Cooperation | 2026 |
