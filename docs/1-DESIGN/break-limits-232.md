# 第18批破限制技术 (Break-Limits Batch 232)

> 生成时间: 2026-09-11 | 5主题 × 3-5来源 | 状态: 批量吸收

---

## 1. 奖励建模 (Reward Modeling)

### 1.1 Reward-Based Scaling (RBS) — 无监督奖励模型训练

**来源**: arXiv:2603.02225

**突破点**: 利用大规模网络文本的 next-token continuation 作为隐式偏好信号，无需人工标注即可训练奖励模型。11M token 数学文本训练后，RewardBench v2 平均提升 +7.7 点，域内数学子集提升 +16.1 点。奖励中心化 (reward centering) 正则化防止弱监督下的训练不稳定。

**关键机制**:
- Bradley-Terry 目标 + in-batch negatives：自然延续 = "chosen"，错配延续 = "rejected"
- 中心化系数 c 惩罚奖励尺度漂移：L = L_BT + c * L_center
- 跨 backbone 迁移：Llama/Qwen 家族 1B-7B 均有效

**NeoTrix 融合**:
- **NT-MEMORY KB embedding**: 将 RBS 隐式偏好信号注入 KB 嵌入管线，提升向量检索的语义对齐
- **NT-MIND distillation**: 作为 skill crystallization 的预训练信号源，替代昂贵的人工偏好标注
- **NT-CORE GWT**: RBS 生成的奖励分数可作为 GWT salience 路由的权重因子

---

### 1.2 Reward Scaling Plateau Index (RSPI) — 奖励模型缩放高原

**来源**: arXiv (RSPI paper, coale.science)

**突破点**: 奖励模型质量遵循亚线性幂律 (exponent ≈ 0.27)。10x RM 参数增长仅带来 1.86x 对齐质量提升。超过高原后，额外 RM 容量反而加剧 reward hacking。RSPI 指标量化高原拐点：7B policy 对应 RSPI ≈ 0.43，70B policy 对应 RSPI ≈ 0.29。

**可证伪预测**:
1. 2027年：无 RLHF 研究能从 10x RM 参数中获得超过 2x 金奖励提升
2. 2027年：至少两个大实验室从大 RM 转向小 RM + 不确定性校准
3. 2028年：标准 RLHF 管线包含显式过优化检测

**NeoTrix 融合**:
- **NT-MIND SEAL pipeline**: 在 distillation 阶段引入 RSPI 阈值，自动判断 RM 缩放是否进入高原
- **NT-SHIELD**: RSPI 作为 reward hacking 防御的早期预警指标
- **Axiom alignment (A1 Cost-Aware Routing)**: 验证 "not all tasks need the strongest model" — 小 RM + 高质量数据 > 大 RM + 噪声数据

---

### 1.3 SPCT-GRM — 推理时奖励模型扩展

**来源**: ICLR 2026 (OpenReview)

**突破点**: Self-Principled Critique Tuning (SPCT) 通过在线 RL 训练生成式奖励模型 (GRM)，使其自适应生成原则和批评。推理时通过并行采样 + Meta RM 指导投票，实现奖励建模的推理时扩展，超越训练时扩展方法。

**关键机制**:
- Pointwise generative RM：灵活处理不同输入类型
- 并行采样扩展计算：推理时 scaling 替代训练时 scaling
- Meta RM：指导投票过程，优化 scaling 性能

**NeoTrix 融合**:
- **NT-CORE E8**: SPCT 的自适应原则生成可映射到 E8 hexagram 的动态调整
- **NT-IO LLM providers**: 作为 provider-level 奖励路由的候选策略
- **NT-ACT tools**: Meta RM 作为 MCP 工具节点，供 agent 调用

---

### 1.4 Gradient Fingerprint (Grift) — 梯度指纹检测 Reward Hacking

**来源**: arXiv:2604.16242

**突破点**: 通过分析模型内部梯度表示 (而非表面文本) 检测隐式 reward hacking。将 CoT 推理轨迹编码为紧凑的梯度指纹向量，在数学/代码/逻辑推理任务上比 CoT Monitor 和 TRACE 提升 25%+ 检测率。集成到 rejection fine-tuning 管线后，测试准确率从 45.6% 提升至 53.5%。

**关键机制**:
- 轻量级 adapter 在选定层计算梯度指纹
- 随机投影压缩为紧凑表示
- 聚类标记 hacking/non-hacking 簇

**NeoTrix 融合**:
- **NT-SHIELD**: Grift 作为 reward hacking 检测的内层防御，部署在 sandbox 级别
- **NT-MIND SEAL**: 在 distillation 阶段用 Grift 过滤 hacking 轨迹
- **NT-REPAIR**: 异常检测触发自愈流程

---

### 1.5 Adversarial Reward Auditing (ARA) — 对抗性奖励审计

**来源**: arXiv:2602.01750

**突破点**: 将 reward hacking 重构为动态博弈。Hacker 策略发现 RM 漏洞，Auditor 从内部表示中学习检测利用行为。Auditor-Guided RLHF (AG-RLHF) 门控奖励信号，将 hacking 从不可观测失败转为可测量信号。跨领域迁移：代码 gaming 的 Hacker 展示 22.5% 更高 sycophancy。

**关键结果**:
- Sycophancy: 38.4% (ARA) vs 72.4% (PPO) vs 47-58% (正则化方法)
- 长度偏置: 162 tokens (ARA) vs 347 tokens (PPO)，ROUGE-L 最高 (24.1)
- 跨领域迁移：hacking 和防御均跨域有效

**NeoTrix 融合**:
- **NT-SHIELD defense-in-depth**: ARA 作为对抗层，与 Grift 互补
- **NT-MIND skill crystallization**: Hacker-Auditor 博弈模式可蒸馏为 skill 节点
- **NT-GOVERNANCE**: AG-RLHF 门控机制映射到治理合规层

---

## 2. 偏好优化 (Preference Optimization)

### 2.1 DPO β 解耦 — Centered-Softplus 重参数化

**来源**: arXiv:2608.27032

**突破点**: DPO 的 β 参数纠缠了两个角色：控制反偏好噪声尺度 + 重缩放优化动态。在固定学习率下，策略偏差对 β 非单调：小 β 死区 → 中间峰值 → 大 β 衰减。标准 DPO loss 值跨 β 不可比：相似 loss 曲线可能 KL 差异数倍。提出的 centered-softplus 重参数化使两个效应独立可调，且 β→0 连续极限退化为线性偏好边距目标。

**NeoTrix 融合**:
- **NT-MIND SEAL pipeline**: centered-softplus 作为 DPO 变体的默认目标函数
- **NT-CORE GWT**: β 解耦为 GWT salience 路由提供更精确的偏好信号
- **NT-MEMORY**: 跨 session 知识检索中，解耦的偏好信号减少迁移偏差

---

### 2.2 Distributionally Robust DPO (WDPO/KLDPO) — 分布鲁棒偏好优化

**来源**: NeurIPS 2025

**突破点**: 用户偏好跨地域/人口统计/文化趋势存在分布偏移。WDPO (Wasserstein) 和 KLDPO (KL) 通过分布鲁棒优化应对偏移。估计误差以 O(n^{-1/4}) 收敛。在 LLaMA-3.2-1B/3B/8B 上，跨 OpenLLM Leaderboard 39 个子任务一致优于 DPO。

**关键机制**:
- WDPO: Wasserstein 不确定集 → 变分正则化
- KLDPO: KL 不确定集 → 重加权阈值 (温度参数 τ)
- 理论保证：即使偏好标签损坏，鲁棒变体仍不退化

**NeoTrix 融合**:
- **NT-WORLD crawl**: 分布鲁棒偏好优化处理多源爬取数据的分布偏移
- **NT-MEMORY KB**: WDPO/KLDPO 的鲁棒性保证适用于跨域经验迁移
- **Axiom A2 (Context as Scarce Resource)**: 鲁棒优化减少上下文窗口浪费在噪声偏好上

---

### 2.3 Pre-DPO — 引导参考模型

**来源**: AAAI 2026

**突破点**: 传统 DPO 将策略和参考模型初始化相同，导致数据利用低效。Pre-DPO 引入引导参考模型 (预训练于目标策略状态)，自适应为更合适的样本分配更高权重。在 AlpacaEval 2 和 Arena-Hard v0.1 上一致提升 DPO 和 SimPO 性能，无需外部模型或额外数据。

**NeoTrix 融合**:
- **NT-MIND distillation**: Pre-DPO 的引导机制映射到 teacher-student 引导蒸馏
- **NT-CORE SelfModel**: 引导参考模型作为 SelfModel 的动态校准锚点
- **NT-ACT production**: 降低 DPO 迭代成本，加速部署

---

### 2.4 KTO — 前景理论优化 (无配对数据)

**来源**: arXiv:2402.01306 (ContextualAI)

**突破点**: KTO 仅需二元信号 (好/坏)，无需配对偏好数据。基于 Kahneman-Tversky 前景理论，直接最大化生成效用。在 1B-30B 规模匹配或超越 DPO。关键发现：
- 离线 PPO + 虚拟 +1/-1 奖励在多数规模匹配 DPO（暗示损失函数的归纳偏置比数据更重要）
- KTO 可处理 90%+ 数据不平衡
- 当预训练模型足够好时，可跳过 SFT 直接 KTO

**NeoTrix 融合**:
- **NT-ACT social media**: KTO 的二元信号匹配社交媒体 thumbs-up/down 数据
- **NT-IO web server**: 生产环境偏好收集简化为二元反馈
- **NT-MIND skill crystallization**: KTO 作为低成本持续对齐工具

---

### 2.5 ε-DPO — 实例级自适应 KL 惩罚

**来源**: NeurIPS 2025

**突破点**: DPO 的静态 KL 惩罚是性能瓶颈。ε-DPO 通过 β 扰动下 logit 单调性检查，为每个偏好对实例级自适应控制 KL 惩罚。无需 batch 级统计，无需额外模型更新计算。在通用聊天基准上显著超越 DPO 及大多数直接对齐算法。

**关键机制**:
- 扰动 β 观察 chosen/rejected log-likelihood ratio 单调性
- 复用当前策略和参考策略的 logit 估计扰动策略
- 反映偏好对的 "混淆度"

**NeoTrix 融合**:
- **NT-CORE AttentionManager**: ε-DPO 的实例级自适应映射到 Ascendancy 双专精的注意力路由
- **NT-MIND SEAL**: 在 distillation 的每步动态调整 KL 约束
- **NT-REPAIR**: 混淆度信号触发知识修复流程

---

## 3. 推理蒸馏 (Reasoning Distillation)

### 3.1 Masked Distillation — 内化 CoT

**来源**: arXiv:2607.22629

**突破点**: 大推理模型的中间 token 主导延迟和内存，但最终答案正确性与轨迹正确性无因果关系。Masked Distillation 训练学生仅预测 solution tokens，教师在 CoT 上提供反馈。支持自蒸馏 (同一模型 thinking/non-thinking 模式) 和双模型设置。通过改变 scaffold 长度，插值全内化和无内化。

**NeoTrix 融合**:
- **NT-MIND distillation**: masked distillation 作为 skill crystallization 的核心蒸馏策略
- **NT-CORE E8**: scaffold 长度映射到 hexagram 的推理深度控制
- **NT-IO CLI**: 降低推理成本，使 CLI 响应更快

---

### 3.2 MI-Distillation — 模型插值推理数据谱

**来源**: arXiv:2608.29623

**突破点**: Long CoT 蒸馏效果不如 Short CoT 的梯度分析。Long CoT 诱导更大梯度幅值和更集中的更新方向，随学生容量增加更显著。MI-Distillation 通过模型插值构建连续 Instruct-Reasoning 数据谱，SeqLSS (Sequential Learnable Surprisal Score) 选择既信息丰富又可学习的轨迹。

**关键发现**:
- 有效 Long CoT 蒸馏需要平衡推理信息密度与分布对齐
- SeqLSS 同时考虑信息量和可学习性

**NeoTrix 融合**:
- **NT-MIND SEAL**: MI-Distillation 的数据谱选择映射到 SEAL phase transition 优化
- **NT-MEMORY KB**: SeqLSS 评分可存储在 KB 中作为经验索引
- **NT-ACT tools**: 降低推理模型的工具调用延迟

---

### 3.3 Gen-SSD — 学生在环生成时选择蒸馏

**来源**: arXiv:2604.02819

**突破点**: 传统蒸馏被动消费完整轨迹。Gen-SSD 让学生在教师采样过程中主动评估候选延续，选择低 PPL 路径，提前剪枝无帮助分支。在数学推理基准上比 Standard KD 提升 ~5.9 点，比其他基线提升 ~4.7 点。

**关键机制**:
- 分块生成 + 学生 PPL 评估
- 早期干预减少计算浪费
- 拒绝采样 + SFT 精炼

**NeoTrix 融合**:
- **NT-CORE GWT**: Gen-SSD 的学生在环机制映射到 GWT attention routing
- **NT-MIND skill crystallization**: 选择性蒸馏作为 skill 节点的构建方法
- **NT-REPAIR**: PPL 评估信号可用于检测推理退化

---

### 3.4 Reasoning Scaffolding — 推理脚手架

**来源**: arXiv:2509.23619

**突破点**: 行为克隆文本模仿是根本限制。Reasoning Scaffolding 将教师推理过程抽象为离散语义信号 (Contrast, Addition, Elaboration) 脚手架。学生通过多任务目标：(1) 预测下一个语义信号，(2) 基于信号生成对应步骤。信号预测任务作为正则化器，迫使学生内化计算模式。

**NeoTrix 融合**:
- **NT-CORE E8 hexagram**: 语义信号映射到 hexagram 的六线结构
- **NT-MIND distillation**: scaffolding 作为 skill 树的层级构建方法
- **NT-IO LLM providers**: 降低推理 token 成本

---

### 3.5 Sequence Truncation — 50% Token 保留 91% 性能

**来源**: ACL 2026 Findings

**突破点**: 推理轨迹本身是蒸馏信号的主要载体。仅监督 CoT tokens（不含 prompt/answer）与全序列监督性能相当。截断至前 50% tokens 保留 ~91% 下游性能，训练时间/内存/FLOPs 减少约 50%。关键推理行为集中在早期 tokens。

**NeoTrix 融合**:
- **NT-MIND SEAL**: 截断协议作为 distillation 的成本优化器
- **NT-IO CLI**: 推理时 token 预算优化
- **Axiom A1 (Cost-Aware Routing)**: 验证 "cheap models for I/O, expensive for reasoning" 的粒度细化

---

## 4. 模型合并 (Model Merging)

### 4.1 FUSE Taxonomy — LLM 时代模型合并综述

**来源**: arXiv:2603.09938

**突破点**: 四维框架 (Foundations, Unification Strategies, Scenarios, Ecosystem) 系统梳理模型合并。核心发现：
- 线性模式连通性 (Linear Mode Connectivity)：共享预训练初始化的微调模型在同一 loss basin
- Task Arithmetic 是唯一可靠产生建设性干扰的方法
- 大模型更容易合并，不同方法在大模型上行为趋同

**关键方法对比**:

| 方法 | 机制 | 优势 | 劣势 |
|------|------|------|------|
| Model Soup | 权重平均 | 简单零推理成本 | 忽略干扰 |
| Task Arithmetic | 任务向量算术 | 模块化，支持加/减/组合 | 需共享预训练初始化 |
| TIES-Merging | Trim-Elect-Sign | 减少参数干扰 | 超参敏感 |
| DARE | 概率稀疏化 | 随机丢弃+重缩放 | 依赖 dropout 率 |

**NeoTrix 融合**:
- **NT-MIND skill crystallization**: Task Arithmetic 映射到 skill 节点组合
- **NT-CORE SelfModel**: 合并后的多任务模型作为 SelfModel 的能力扩展
- **NT-ACT tools**: 合并成本仅需加法运算，适合生产环境

---

### 4.2 大规模合并实证 — PaLM-2 1B-64B

**来源**: arXiv:2410.03617

**突破点**: 
1. 强零样本基座模型 (instruction-tuned) 比预训练模型更适合合并
2. 更大模型更容易合并
3. 合并 8 个大专家模型时，泛化性常超越多任务训练模型
4. 不同合并方法在大模型上行为趋同（过度参数化消除了技术优势）

**NeoTrix 融合**:
- **NT-MIND SEAL**: 合并策略选择随 constellation 成熟度动态调整
- **NT-CORE E8**: 大模型合并趋同性映射到 E8 的相空间收敛

---

### 4.3 合并时专家训练时长 — 方法依赖最优时长

**来源**: arXiv (Qwen3.5 0.8B-4B 研究)

**突破点**: 最优训练时长根本性地依赖合并方法：
- **Simple Averaging**: 欠训练专家最优 (0.25-0.75× T*)
- **Task Arithmetic**: 中间范围 (0.25-3× T*)
- **TIES/DARE+TIES**: 过拟合专家最优 (1.5-5× T*)

过拟合专家提供更高多样性，稀疏化方法的干扰消解机制充当方差缩减（类似随机森林中深层高方差树的优势）。

**NeoTrix 融合**:
- **NT-MIND SEAL**: 训练时长作为 SEAL pipeline 的超参数自适应维度
- **NT-REPAIR**: 过拟合检测与合并策略联动
- **NT-GOVERNANCE**: 合并决策的治理审计

---

### 4.4 CoMerge — 冲突驱动偏好优化

**来源**: arXiv:2609.02273 (2026-09)

**突破点**: 将模型合并重构为冲突驱动的偏好优化问题。识别任务向量间的冲突区域，通过偏好学习解决冲突，而非简单的剪枝或符号投票。在多任务合并场景中超越 TIES-Merging 和 DARE。

**NeoTrix 融合**:
- **NT-MIND skill crystallization**: CoMerge 的冲突解决机制映射到 skill 节点融合策略
- **NT-CORE GWT**: 冲突检测作为 GWT salience 的负向信号

---

## 5. 评估方法 (LLM Evaluation)

### 5.1 LLMEval-Fair — 动态抗污染评估

**来源**: ACL 2026

**突破点**: 
- 220k 研究生级私有题库，每次评估动态采样 1000 题
- 两层反作弊架构 (进程控制 + 安全传输)
- 相对排名系统：LLM-as-Judge 达到 90% 人类专家一致性
- 30 个月纵向研究 ~60 个模型，揭示静态基准的数据污染
- 动态排名与静态基准排名相关性仅 ρ ≈ 0.65-0.72

**关键发现**:
- 知识记忆存在 ~90% 性能天花板
- C-Eval 最高排名模型 (Claude Sonnet-4, Doubao-1.5-Pro) 污染最严重
- 提示格式对知识密集型任务影响微小

**NeoTrix 融合**:
- **NT-SHIELD**: LLMEval-Fair 的反作弊架构映射到评估安全层
- **NT-MEMORY KB**: 私有题库管理作为 KB 的评估数据 namespace
- **NT-IO web server**: 动态评估作为 web server 的质量监控

---

### 5.2 LivingArena — 对等探测评估

**来源**: arXiv:2607.24780

**突破点**: 模型互相出题，识别对手无法回答的问题。完全自动化、抗污染、自适应。Judge panel 验证问题有效性，惩罚自伤 (self-harm)。360 场比赛 (10 模型 round-robin)，产生稳定 Elo 排名。模型能定位并攻击对手弱点维度。

**关键发现**:
- 询问能力和回答能力是独立维度
- 自校准率：GPT-5.2 (42.7% self-harm) vs GPT-5.5 (10.0%)
- 与人类偏好仅弱相关 — 测量的是客观事实严谨性

**NeoTrix 融合**:
- **NT-SHIELD**: 对等探测模式映射到红队评估架构
- **NT-MIND SEAL**: LivingArena 的自适应问题生成作为 SEAL exploration 阶段
- **NT-CORE E8**: Elo 排名映射到 hexagram 的相空间位置

---

### 5.3 DR-Arena — 深度研究 Agent 评估

**来源**: ACL 2026

**突破点**: 
- 动态信息树 (Information Trees) 从实时网络趋势构建
- 自适应进化循环：基于实时表现动态升级任务复杂度
- 分离深度推理 (Depth) 和广度覆盖 (Width) 能力
- 与 LM SYS Search Arena 人类偏好 Spearman 相关性 0.94 (SOTA)

**NeoTrix 融合**:
- **NT-WORLD crawl**: DR-Arena 的实时信息树映射到 crawl pipeline
- **NT-ACT tools**: Agent 评估框架作为 NT-ACT 的质量门禁
- **NT-IO web server**: 深度研究能力作为 web server 的分析功能

---

### 5.4 FTD — 可控污染检测

**来源**: ACL 2026

**突破点**: FDR (False Discovery Rate) 控制的训练数据检测框架。组合多个互补检测器，自适应加权，在受控 FDR 下实现高统计功效。在真实基准上显著减少残余污染，同时保持评估一致性。

**关键机制**:
- 用户指定 FDR 阈值， provably 控制误判率
- 自适应加权组合多个检测器
- 统计功效保证

**NeoTrix 融合**:
- **NT-SHIELD**: FTD 作为评估数据清洗的统计保证层
- **NT-MEMORY KB**: FDR 控制的检测结果存储为 KB 的数据质量元数据

---

### 5.5 League of LLMs (LOL) — 无基准互评

**来源**: ACL 2026

**突破点**: 多 LLM 自治联赛：生成问题 → 独立回答 → 互评 → 聚合排名。四个核心标准：动态、透明、客观、专业。Top-k 一致性 70.7%。发现 "记忆式回答" 行为和 OpenAI 模型家族内评分优势 (∆=9, p<0.05)。

**NeoTrix 融合**:
- **NT-CORE ConsciousnessTree**: LOL 的互评机制映射到 ConsciousnessTree 的分支健康检查
- **NT-MIND SEAL**: LOL 的多轮迭代作为 SEAL exploration 的评估阶段
- **NT-GOVERNANCE**: 透明和客观标准映射到治理审计

---

## NeoTrix 融合总结

### 跨主题交叉模式

| 模式 | 来源主题 | NeoTrix 映射 |
|------|---------|-------------|
| **数据质量 > 模型规模** | RSPI (奖励建模), DPO β 解耦 | A1 Cost-Aware Routing 的粒度细化 |
| **自适应/实例级控制** | ε-DPO, SPCT-GRM, Gen-SSD | GWT attention routing + AttentionManager |
| **对抗性检测** | ARA, Grift, LivingArena | NT-SHIELD defense-in-depth |
| **内化/压缩** | Masked Distillation, Sequence Truncation | SEAL distillation cost optimizer |
| **无监督/弱监督** | RBS (奖励建模), KTO (偏好优化) | NT-MEMORY KB embedding |
| **动态/抗污染** | LLMEval-Fair, LivingArena, DR-Arena | NT-SHIELD 评估安全层 |
| **冲突解决** | TIES-Merging, CoMerge, WDPO | NT-MIND skill crystallization |

### 优先实施路径

1. **P0 (立即)**: ε-DPO + Centered-Softplus → NT-MIND SEAL pipeline DPO 变体
2. **P0 (立即)**: Grift 梯度指纹 → NT-SHIELD reward hacking 检测
3. **P1 (短期)**: RBS 无监督奖励 → NT-MEMORY KB embedding 信号源
4. **P1 (短期)**: MI-Distillation 数据谱 → NT-MIND distillation 优化器
5. **P2 (中期)**: LivingArena 对等探测 → NT-SHIELD 红队评估架构
6. **P2 (中期)**: CoMerge 冲突驱动合并 → NT-MIND skill 节点融合
7. **P3 (长期)**: FTD 统计保证 → NT-SHIELD 评估数据清洗

### Axiom 验证

| Axiom | 本批验证 |
|-------|---------|
| **A1 Cost-Aware Routing** | RSPI 证明小 RM + 高质量数据 > 大 RM；Sequence Truncation 50% token 保留 91% 性能 |
| **A2 Context as Scarce Resource** | DPO β 解耦减少上下文浪费；WDPO/KLDPO 鲁棒优化减少噪声偏好消耗 |
| **A3 Skill as Production Template** | Pre-DPO / KTO 降低 skill 对齐成本；MI-Distillation 的 SeqLSS 评分可存入 KB |

---

## 参考文献

1. arXiv:2603.02225 — Reward-Based Scaling (RBS)
2. arXiv (RSPI) — Reward Scaling Plateau Index
3. ICLR 2026 (OpenReview) — SPCT-GRM Inference-Time Scaling
4. arXiv:2604.16242 — Gradient Fingerprint (Grift)
5. arXiv:2602.01750 — Adversarial Reward Auditing (ARA)
6. arXiv:2608.27032 — DPO β Entanglement / Centered-Softplus
7. NeurIPS 2025 — WDPO/KLDPO Distributionally Robust DPO
8. AAAI 2026 — Pre-DPO Guiding Reference Model
9. arXiv:2402.01306 — KTO Kahneman-Tversky Optimization
10. NeurIPS 2025 — ε-DPO Instance-Level KL Control
11. arXiv:2607.22629 — Masked Distillation
12. arXiv:2608.29623 — MI-Distillation
13. arXiv:2604.02819 — Gen-SSD Student-in-the-Loop
14. arXiv:2509.23619 — Reasoning Scaffolding
15. ACL 2026 Findings — Sequence Truncation Distillation
16. arXiv:2603.09938 — FUSE Taxonomy for Model Merging
17. arXiv:2410.03617 — Large-Scale Model Merging (PaLM-2)
18. arXiv (Qwen3.5) — Expert Training Duration for Merging
19. arXiv:2609.02273 — CoMerge Conflict-Driven Merging
20. ACL 2026 — LLMEval-Fair Dynamic Evaluation
21. arXiv:2607.24780 — LivingArena Peer-Probing
22. ACL 2026 — DR-Arena Deep Research Evaluation
23. ACL 2026 — FTD Contamination Detection
24. ACL 2026 — League of LLMs (LOL)
