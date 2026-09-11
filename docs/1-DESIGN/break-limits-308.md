# 第94批破限制技术 (Batch 308)

> 研究日期: 2026-09-11
> 主题: 强化学习 / 逆强化学习 / 多目标优化 / 贝叶斯优化 / 进化策略

---

## 1. 强化学习 Scaling

### 1.1 ScaleRL: 首个 LLM RL 计算 Scaling 预测框架

**来源**: "The Art of Scaling Reinforcement Learning Compute for LLMs" (arXiv:2510.13786, ICLR 2026)

**突破点**: 首个超过 400,000 GPU-hour 的系统性 RL scaling 研究, 建立了 RL 训练的可预测 scaling 方法学。发现 sigmoidal compute-performance 曲线, 并通过最小规模运行 (16K GPU-h) 的 extrapolation 成功预测 100K GPU-h 的性能。

**关键发现**:
- 并非所有方法都收敛到相同的渐近性能; 渐近性能由训练策略决定
- Loss 聚合/归一化/课程/off-policy 等细节主要影响 compute efficiency, 不改变渐近
- ScaleRL recipe: PipelineRL (8-step off-policyness) + interruption-based truncation + FP32 logits + prompt-level loss aggregation
- 从 16K GPU-h extrapolation 到 100K GPU-h 的预测误差极小

**NeoTrix 融合**: SEAL pipeline 的 scaling 评估可借鉴 ScaleRL 的 sigmoidal 拟合方法。对 NeoTrix 模块性能进行 compute-performance curve fitting, 从 C0→C5 的演进可预测性提升。GWT 注意力路由的 compute allocation 可用 ScaleRL 的渐近性能框架进行资源分配优化。

---

### 1.2 MBDPO: 扩散策略突破 Model-Based RL Scaling 瓶颈

**来源**: "Scaling World-Model Reinforcement Learning Through Diffusion Policy Optimization" (arXiv:2605.26282)

**突破点**: 引入 Model-Based Diffusion Policy Optimization (MBDPO), 解决了 model-based RL 中策略搜索与值学习之间的结构性错位问题。在 offline regime 中, 模型容量从 1.7M 扩展到 340M 时观察到单调递增的 scaling curve。

**关键数据**:
- 多任务 offline pretraining: 150M 参数的 JOWA agent 在 Atari 上仅用 10% subsampled 数据达到 78.9% 人类水平
- 比现有 SOTA large-scale offline RL baselines 平均高出 31.6%
- 仅用 5k offline fine-tuning 数据 (~4 trajectories) 即可零样本迁移到新游戏
- 关键洞察: 模型精度 alone 不决定 planning 效率; 高精度模型仍可能产生弱策略

**NeoTrix 融合**: NT-MIND 的 SEAL pipeline 扩散可引入 MBDPO 的 action-chunk 范式。在模块进化过程中, 将 "状态-动作序列" 作为 action chunk 进行扩散, 减少进化决策中的 compounding error, 提升跨模块学习效率。

---

### 1.3 Adaptive Batch Scaling: 挑战 RL 中大 batch 不兼容的教条

**来源**: "Scalable Reinforcement Learning via Adaptive Batch Scaling" (ICML 2026 Poster)

**突破点**: 挑战了 "大 batch 与 RL 不兼容" 的传统观点。发现非平稳性不是 RL 的固定属性, 而是在训练过程中演化的: 早期需要小 batch 保持 plasticity, 后期大 batch 可实现精确收敛。

**关键数据**:
- Behavioral Divergence 指标: 通过 action-level shifts 测量策略非平稳性
- ABS (Adaptive Batch Scaling) 动态调整有效 batch size, 反比于策略波动性
- 结合 PQN 算法在 ALE 上: 大网络 + 大 batch 达到最佳性能
- 这种 scaling 行为在 RL 中被认为是不可实现的, 现通过自适应 batch 控制解锁

**NeoTrix 融合**: NT-CORE 的 SelfModel 性能训练可引入 ABS 思想。在模块进化训练的早期阶段使用小 batch 快速探索, 后期使用大 batch 精确收敛。Behavioral Divergence 可作为 NT-META 的进化稳定性指标, 指导 SEAL pipeline 的 batch 调度。

---

### 1.4 JustRL: 极简 RL Recipe 的 Scaling 力量

**来源**: "JustRL: Scaling a 1.5B LLM with a Simple RL Recipe" (ICLR 2026 Poster)

**突破点**: 证明复杂多阶段 pipeline (动态调度、课程学习) 对 1.5B 基础模型并非必要。JustRL 使用固定超参数在两个 1.5B 基础模型上分别达到 54.5% 和 64.3% (9 个数学 benchmark), 同时仅用 2× 计算量。

**关键发现**:
- 固定超参数跨模型迁移, 无需调优
- 训练在数千步内保持稳定, 无需干预
- 字段可能在用复杂性解决随规模增长而消失的问题
- 暗示: 稳定、可扩展的 baseline 可能比复杂方法更有效

**NeoTrix 融合**: NeoTrix 的模块进化应优先建立稳定 baseline, 再考虑复杂调度。JustRL 的 "固定超参数跨域迁移" 暗示 NT-* 域间可共享进化超参数。对 NT-MIND 的 distillation 流程, 简单 stable recipe 可能优于复杂 multi-stage pipeline。

---

### 1.5 RL Post-Training Scaling: 数据规模驱动的可预测幂律

**来源**: "Scaling Behaviors of LLM Reinforcement Learning Post-Training" (ACL 2026, arXiv:2412.05265)

**突破点**: 系统研究 LLM RL post-training 的 scaling 行为, 覆盖 Qwen2.5 系列 (0.5B 到 72B)。发现模型性能与训练资源之间存在可预测的 power-law 关系, 且该关系在基础模型和 instruction-tuned 模型上均成立。

**关键发现**:
- 大模型始终展示更优的 compute 和 data efficiency
- RL learning efficiency 随模型规模增长呈现 latent saturation trend
- 在 data-constrained 场景中, 性能主要由训练数据总量驱动, 而非样本唯一性
- 提供了通过 RL post-training 扩展推理能力的实用指南

**NeoTrix 融合**: NT-CORE 的 E8 推理引擎可借鉴 power-law scaling 分析。对不同规模的 SelfModel 进行 scaling curve fitting, 预测进化投入的边际收益。在 NT-MEMORY 的 KB embedding 训练中, 利用 data scaling law 指导数据采集策略。

---

## 2. 逆强化学习

### 2.1 Interactionless IRL: 无交互的可复用奖励制品

**来源**: "Interactionless Inverse Reinforcement Learning: A Data-Efficient Framework" (ACM, 2026)

**突破点**: 提出 Interactionless IRL 框架, 从专家演示中学习可检查、可编辑、可复用的奖励制品, 与策略分离。通过 sparse autoencoders 发现语言模型中的高度可解释特征, 将 IRL 从策略学习中解耦。

**关键创新**:
- 奖励函数与策略解耦: 奖励制品可跨策略、跨任务复用
- 可检查性: 奖励制品可被人类审查和编辑
- 数据效率: 无需与环境交互即可从演示中提取奖励信号
- 与 GAIL 等对抗方法相比, 训练更稳定且样本效率更高

**NeoTrix 融合**: NT-MIND 的技能蒸馏可引入 Interactionless IRL 范式。将专家技能演示中的隐式奖励函数提取为可复用的 "技能奖励制品", 存储在 KB `experience` namespace 中。这些奖励制品可跨 NT-* 域迁移, 作为新模块进化的目标函数。

---

### 2.2 Rethinking IRL: 从数据对齐到任务对齐

**来源**: "Rethinking Inverse Reinforcement Learning: from Data Alignment to Task Alignment" (NeurIPS 2024)

**突破点**: 提出任务对齐优先于数据对齐的 IRL 框架。使用专家演示作为弱监督, 推导一组与任务对齐 (而非仅与数据对齐) 的候选奖励函数, 然后通过对抗机制用这组奖励函数训练策略。

**关键发现**:
- 传统 IRL 推断的奖励函数常无法捕捉底层任务目标
- 半监督方法: 专家演示作为弱监督 → 候选奖励函数集 → 对抗验证
- 在复杂和迁移学习场景中优于传统 IL baselines
- 理论证明: 任务-奖励错配可通过候选集的集体验证缓解

**NeoTrix 融合**: NT-GOVERNANCE 的策略验证可借鉴任务对齐范式。将 NT-* 域的治理规则作为 "任务目标", 将 agent 行为作为 "专家演示", 通过任务对齐 IRL 推断隐式治理奖励函数, 用于跨域合规验证。

---

### 2.3 Scalable Causal Imitation Learning

**来源**: "Scalable Causal Imitation Learning" (arXiv:2607.17003, RL Journal 2026)

**突破点**: 将因果推断框架与 SOTA inverse RL objectives 结合, 解决长 horizon 高维状态-动作空间中的模仿学习问题。引入 Causal SQIL 和 Causal IQ-Learn 两种 off-policy 因果模仿学习算法。

**关键数据**:
- 在有混淆变量的长 horizon 任务中, 因果感知方法大幅超越非因果方法
- Causal SQIL 和 Causal IQ-Learn 在某些任务中甚至超越专家
- 滑动窗口近似: 将全 horizon 因果调整简化为固定大小滑动窗口
- 无因果感知的模仿方法全部无法学到有意义行为

**NeoTrix 融合**: NT-ACT 的工具调用学习可引入因果模仿框架。在 MCP 工具使用中识别混淆变量 (如用户偏好、环境状态), 通过因果调整从非最优专家演示中提取有效策略。滑动窗口近似可降低长序列工具调用的因果计算开销。

---

### 2.4 Fast Rates for Min-Max IRL

**来源**: "Fast Rates for Inverse Reinforcement Learning" (arXiv:2605.14599, 2026)

**突破点**: 为 entropy-regularized min-max IRL 建立了新的结构和统计结果。在线性设置下, 证明了 min-max IRL 的 fast convergence rates, 为大规模 IRL 问题提供了理论保证。

**关键创新**:
- Min-max formulation: 同时优化策略和奖励, 避免传统 IRL 的双层优化
- Entropy regularization: 确保探索-利用平衡, 提升样本效率
- Fast rates: 在线性函数近似下, 收敛速度优于标准 IRL 方法
- 理论与实践的桥梁: 为大规模 IRL 部署提供收敛保证

**NeoTrix 融合**: NT-MIND 的自我进化可借鉴 min-max IRL 框架。将自我进化建模为 min-max game: 策略 (技能选择) 和奖励 (进化目标) 同时优化。Entropy regularization 可防止进化陷入局部最优, fast rates 保证为 C0→C5 演进提供理论收敛性。

---

### 2.5 Robometer: 轨迹比较驱动的通用机器人奖励模型

**来源**: "Robometer: Scaling General-Purpose Robotic Reward Models via Trajectory Comparisons" (RSS 2026)

**突破点**: 结合帧级 progress loss 和轨迹级 preference loss, 从 100 万+ 轨迹 (含大量次优和失败数据) 中学习通用奖励模型。RBM-1M 数据集跨越多种机器人形态和任务。

**关键创新**:
- 双目标训练: 帧级 progress (锚定奖励量级) + 轨迹级 preference (全局排序)
- 次优/失败数据利用: 传统方法仅用专家数据, Robometer 利用所有数据
- 跨具身性泛化: 在不同机器人和任务间学习可迁移的奖励函数
- 1M+ 轨迹的规模化训练, 支持真实世界部署

**NeoTrix 融合**: NT-WORLD 的数据采集可构建类似的轨迹对比数据集。将 NeoTrix 模块的运行轨迹 (成功/失败/次优) 作为训练数据, 学习通用的 "模块健康奖励模型"。该模型可用于 NT-REPAIR 的自动诊断和修复建议生成。

---

## 3. 多目标优化

### 3.1 MODNAS: 超网络驱动的多目标 NAS

**来源**: "Multi-objective Differentiable Neural Architecture Search" (ICLR 2025, ICML 2024 Workshop)

**突破点**: 提出硬件感知多目标可微 NAS 算法, 通过 hypernetwork 参数化跨设备和多目标的联合架构分布。条件化于硬件特征和偏好向量, 实现零样本迁移到新设备。

**关键数据**:
- 单次搜索运行获得多样化 Pareto 最优架构集
- 支持 19 种硬件设备 × 3 个目标的联合优化
- 超越现有 MOO NAS 方法: MobileNetV3/ImageNet-1k, encoder-decoder Transformer/MT, decoder-only Transformer/LM
- MetaHypernetwork 显著优于 RHPN 基线在 Pareto front profiling 上

**NeoTrix 融合**: NT-PHYSICAL 的传感器/执行器配置可引入 MODNAS 范式。将 NeoTrix 模块的性能-功耗-延迟建模为多目标, 通过 hypernetwork 生成跨硬件配置的 Pareto 最优架构。偏好向量可动态调整, 适应不同部署场景 (云端 vs 边缘)。

---

### 3.2 MOO-VARI: 物理信息神经网络的自适应权重

**来源**: "A Multi-Objective Optimization Framework for Adaptive Weighting in Physics-Informed Machine Learning" (AAAI-26)

**突破点**: 将 PINN 训练建模为多任务优化, 使用 NSGA-II 探索多个损失项的 Pareto front, 然后通过 VARI (Variance-Aware Relative Improvement) 权重方法将 Pareto-optimal 信息转化为自适应损失权重。

**关键创新**:
- 自动平衡竞争损失: 数据驱动 vs 物理驱动 loss 的动态权衡
- VARI 权重: 基于方差感知的相对改进, 比固定权重更鲁棒
- 在收敛速度、预测精度、参数估计上均优于标准 PINN 和 SOTA 自适应权重策略
- 适用于逆问题中的参数不确定性场景

**NeoTrix 融合**: NT-MIND 的 SEAL pipeline 多目标优化 (精度/速度/资源) 可引入 MOO-VARI。将模块进化的多个目标 (编译成功率/测试通过率/性能指标) 视为竞争损失项, 通过 NSGA-II 探索 Pareto front, VARI 动态调整权重, 实现自适应进化平衡。

---

### 3.3 GNN-MOGWO: 图神经网络引导的多目标灰狼优化

**来源**: "GNN-MOGWO: A Graph Neural Network-Guided Multi-Objective Grey Wolf Optimization" (ACM, 2026)

**突破点**: 两阶段框架: GNN encoder 学习多模态网络的低维嵌入, 然后用改进的多目标灰狼优化 (MOGWO) 在嵌入空间中搜索。在 DMTSP 上达到最优 GD 和 IGD, 解方案最接近且最覆盖真实 Pareto front。

**关键创新**:
- GNN 嵌入: 捕获多模态网络的拓扑结构和节点关系
- MOGWO 优化: 在低维嵌入空间中高效搜索, 避免高维诅咒
- 两阶段解耦: 表示学习与优化分离, 各自可独立改进
- 超越现有 MOCO 方法在 Pareto front 质量上

**NeoTrix 融合**: NT-CORE 的 E8 hexagram 推理可引入 GNN-MOGWO 范式。将 hexagram 状态空间建模为图结构, GNN 学习状态嵌入, MOGWO 在嵌入空间中搜索多目标 Pareto 最优推理路径 (准确率/推理深度/计算成本)。

---

### 3.4 NHDE: 神经多目标组合优化的多样性增强

**来源**: "Neural Multi-Objective Combinatorial Optimization with Diversity Enhancement" (NeurIPS 2023)

**突破点**: 超越纯分解方法, 引入 multiple Pareto optima (MPO) 策略, 为每个子问题发现多个解。结合多样性增强, 生成更高多样性的 Pareto front。

**关键数据**:
- 在 MOTSP、MOCVRP、MOKP 上超越 PMOCO、MDRL、DRL-MOA
- MPO 策略: 利用 Pareto optimality 为每个权重向量发现多个非支配解
- 多样性增强: 避免分解方法的重复解问题
- 可作为插件应用于不同神经 MOCO backbone

**NeoTrix 融合**: NT-ACT 的多任务编排可引入 NHDE 的 MPO 策略。在多任务调度中, 为每个任务权重配置发现多个 Pareto 最优调度方案, 通过多样性增强避免调度模式固化, 提升系统对任务变化的适应性。

---

### 3.5 ParetoQ: 极低比特 LLM 量化的 Scaling Law

**来源**: "ParetoQ: Scaling Laws in Extremely Low-bit LLM Quantization" (PyTorch Blog, 2026)

**突破点**: 首个统一 binary、ternary、2-to-4 bit 量化感知训练的算法。在所有比特宽度上达到 SOTA, 且 scaling law 分析揭示: binary 严重损害精度, 但 ternary/2-bit/3-bit 性能持平, 常超越 4-bit。

**关键发现**:
- 统一公式 ParetoQ = Elastic Binarization (1-bit) + LSQ (3/4-bit) + SEQ (1.58/2-bit)
- Scaling law: 给定总训练预算 B = B_FPT + B_QAT, 最优分配策略随比特宽度变化
- MobileLLM 低比特模型集合: 最小 1-bit 125M 仅 ~16MB
- 所有 SOTA 点均在 Pareto front 上, 确保 scaling law 比较的一致性

**NeoTrix 融合**: NT-PHYSICAL 的边缘部署可引入 ParetoQ 的量化策略。对 NeoTrix 模型进行 Pareto-front 分析, 在精度和部署成本间找到最优平衡。Scaling law 可预测不同量化级别对模块性能的影响, 指导资源受限场景的模型选择。

---

## 4. 贝叶斯优化

### 4.1 LCBO: 高维约束贝叶斯优化

**来源**: "Local Constrained Bayesian Optimization" (arXiv:2603.07965, 2026)

**突破点**: 提出 Local Constrained Bayesian Optimization (LCBO), 专门针对高维约束问题。利用约束惩罚代理模型的可微景观, 在快速局部下降和不确定性驱动探索间交替。理论证明 KKT 残差收敛率对维度 d 为多项式依赖, 而非全局 BO 的指数依赖。

**关键数据**:
- 在高维 benchmark (最高 100D) 上一致超越 SOTA baselines
- 理论保证: KKT residual 收敛率 = poly(d) (常见核函数, 温和假设)
- 对比 trust-region 方法: 避免面对紧密/复杂约束时的过早收缩
- 约束惩罚代理模型: 可微景观允许梯度信息引导搜索

**NeoTrix 融合**: NT-CORE 的 E8 hexagram 搜索空间是高维约束优化。LCBO 的 local descent + uncertainty-driven exploration 可替代当前的全局搜索, 将搜索从 100D+ 的 hexagram 空间分解为局部可管理的子问题, 理论保证收敛率从指数降为多项式。

---

### 4.2 Local Preferential BO: 偏好反馈驱动的高维优化

**来源**: "Local Preferential Bayesian Optimization" (arXiv:2606.02351, 2026)

**突破点**: 将高维 BO 的 trust-region 和 derivative-informed local search 思想迁移到 preferential setting。通过成对人类偏好反馈学习, 无需显式目标函数。在 GP 标准 benchmark 和策略搜索任务上大幅降低累积 regret。

**关键创新**:
- 无需目标函数: 从 pairwise preference 反馈中学习
- Local PBO: 信任域 + 导数信息 → 成对偏好反馈
- 高维有效性: 在高维和复杂景观中尤其有效
- 策略搜索应用: 适用于真实世界偏好驱动的优化任务

**NeoTrix 融合**: NT-GOVERNANCE 的策略验证可引入 Local PBO。将人类对 NT-* 域行为的偏好反馈 (如 "这个模块进化方向更好") 作为 pairwise preference, 无需定义显式治理目标函数, 直接从偏好中学习最优策略。

---

### 4.3 Safe BO with Monotonicity Constraints

**来源**: "No-Regret Algorithms for Safe Bayesian Optimization with Monotonicity Constraints" (AISTATS 2024)

**突破点**: 在安全约束下进行 BO, 同时利用目标函数和约束函数的单调性结构。引入 expd 规则: 仅当扩展到新点可能乐观地获得更好 f 值时才扩展安全集。

**关键创新**:
- 安全+单调性: 双重结构利用
- No-regret 保证: 在安全约束下仍能达到渐近最优
- Expd 规则: 乐观扩展安全集, 避免过度保守
- 简化: 当 f 和 g 对 s 均单调时, 算法可进一步简化

**NeoTrix 融合**: NT-SHIELD 的安全验证可引入安全 BO 框架。将模块行为的安全约束建模为单调性约束 (如资源使用量随任务复杂度单调递增), 在安全边界内进行 BO 搜索, 保证 no-regret 的安全优化。

---

### 4.4 Surrogate-Guided Scaling Law Estimation

**来源**: "Active Budget Allocation for Efficient Scaling Law Estimation via Surrogate-Guided Pruning" (arXiv:2605.17234, 2026)

**突破点**: 使用 GP surrogate model 预测学习曲线的未来走势, 在 Successive Halving 中进行更智能的预算分配。Surrogate 预测允许提前终止不 promissing 的模型, 将预算集中在有前途的方向。

**关键创新**:
- Surrogate-guided pruning: GP 预测 learning curve 的可能延续
- 从 partial 学习曲线中 extrapolate 完整 scaling law
- 比 plain SH 更低的最低 loss (选择不同且更优的模型)
- 概率 surrogate 提供 scaling law 的置信上下界

**NeoTrix 融合**: NT-MIND 的 SEAL pipeline 可引入 surrogate-guided scaling law estimation。用 GP surrogate 预测模块进化曲线, 在 C0→C5 演进中提前终止不 promissing 的进化路径, 将 compute 预算集中在有前途的模块变体上。

---

### 4.5 Time-Varying GP Bandits with Constant Exploration

**来源**: "Sharper Regret Bounds for Time-Varying Gaussian Process Bandits with Constant Exploration" (arXiv, 2026)

**突破点**: 在时变环境中进行 BO, 目标函数按 GP drift 模型演化。现有 GP-UCB 分析要求 exploration parameter 随 horizon 增长, 本工作证明 constant exploration 即可获得更紧的 regret bound。

**关键创新**:
- Constant exploration: 无需随时间增长的 exploration parameter
- 更紧的 regret bound: 比现有时变 GP-UCB 分析更优
- GP drift model: 目标函数平滑演化, 非完全随机变化
- 实际意义: 减少 tuning 开销, 适应动态环境

**NeoTrix 融合**: NT-WORLD 的在线学习可引入时变 GP bandit。在爬虫/感知模块的在线优化中, 目标函数 (数据质量/延迟) 随时间变化。Constant exploration 减少 hyperparameter tuning, 适应动态 web 环境。

---

## 5. 进化策略

### 5.1 ES-AHD: 进化策略驱动的自动启发式设计

**来源**: "ES-AHD: An Evolution Strategy Framework for Automatic Heuristic Design" (arXiv:2609.00023, ICIST 2026)

**突破点**: 将进化策略 (ES) 框架应用于自动启发式设计, 通过种群搜索发现组合优化问题的高性能启发式。ES 的无梯度特性使其适用于不可微的启发式搜索空间。

**关键创新**:
- 无梯度搜索: 无需目标函数可微, 适用于离散/组合空间
- 种群多样性: ES 天然维护解的多样性
- 自动设计: 从人工设计启发式转向自动发现
- 可扩展: ES 的并行评估能力支持大规模搜索

**NeoTrix 融合**: NT-ACT 的工具调用策略可引入 ES-AHD 范式。将 MCP 工具调用的启发式 (如重试策略、错误处理模式) 作为可进化对象, 通过 ES 自动发现高性能工具调用策略, 替代人工设计的固定规则。

---

### 5.2 Gradient-Free SNN Training via Low-Rank ES

**来源**: "Gradient-Free Training of Spiking Neural Networks via Low-Rank Evolution Strategies" (arXiv:2605.30361, 2026)

**突破点**: 使用低秩进化策略训练脉冲神经网络 (SNN), 绕过 spike threshold 的不可微性。低秩约束减少参数空间, 使 ES 在高维 SNN 训练中可行。

**关键创新**:
- 低秩 ES: 通过低秩约束将参数空间降维, 解决 ES 在高维中的 scaling 问题
- 无梯度: 完全绕过 SNN 的非可微 spike 操作
- 神经形态硬件: SNN 在 neuromorphic hardware 上的能效优势
- 可扩展性: 低秩 ES 比 full-rank ES 在高维中更高效

**NeoTrix 融合**: NT-PHYSICAL 的传感器融合可引入低秩 ES 训练 SNN。在边缘设备上用 SNN 处理传感器数据, 低秩 ES 训练绕过梯度计算限制, 适应 neuromorphic hardware 的能效约束。

---

### 5.3 Neural ES for Black-box Pareto Set Learning

**来源**: "Neural Evolution Strategy for Black-box Pareto Set Learning" (NeurIPS 2025 Poster)

**突破点**: 在 PSL (Pareto Set Learning) 框架中引入 Evolution Strategy, 学习黑盒多目标函数的整个 Pareto set。ES 的分布模型 (多元高斯) 适合在复杂适应度景观中搜索。

**关键创新**:
- 黑盒优化: 无需目标函数梯度或显式形式
- 整个 Pareto set: 不仅找到单个 Pareto 最优解, 而是整个集合
- 分布模型: 多元高斯捕获 Pareto set 的结构
- 与神经网络结合: 用 NN 参数化 ES 的分布模型

**NeoTrix 融合**: NT-CORE 的 SelfModel 多目标优化可引入 Neural ES PSL。学习 SelfModel 参数的整个 Pareto set (性能/稳定性/资源消耗), 在不同部署场景下快速切换, 而非每次重新优化。

---

### 5.4 CMA-ES with Adaptive Sampling for Noisy Robot Optimization

**来源**: "Improving CMA-ES Convergence Speed, Efficiency, and Reliability in Noisy Robot Optimization Problems" (Evolutionary Computation, 2026)

**突破点**: 提出 AS-CMA (Adaptive Sampling CMA-ES), 根据预测排序难度分配采样时间, 实现一致精度。在四个模拟 cost landscape 上: 98% 运行无需调参即收敛, 比最佳静态采样 CMA-ES 快 24-65%, 总成本低 29-76%。

**关键数据**:
- 适应性采样: 基于排序难度预测分配评估资源
- 无调参收敛: 98% 运行无需调整 tunable parameter
- 比 Bayesian optimization: 在复杂 landscape 上更高效可靠
- 真实部署: 在外骨骼优化实验中行为符合预期

**NeoTrix 融合**: NT-PHYSICAL 的机器人控制优化可引入 AS-CMA。在真实机器人策略优化中, AS-CMA 的自适应采样减少昂贵的物理评估次数, 一致性精度保证控制策略的可靠性。

---

### 5.5 Neuroevolution for Biological Neural Computation

**来源**: "Neuroevolution insights into biological neural computation" (Science, 2026)

**突破点**: 通过神经进化实验揭示生物神经回路、行为和认知过程的进化起源。进化发现了比人工设计更有效的任务分工: 一个模块控制近距离威胁, 另一个处理其他一切。

**关键发现**:
- 进化压力发现非直觉的模块化分工
- 对称和重复变异是生物结构的标志
- 模块化编码比整体编码更鲁棒
- 为人工神经网络设计提供生物学启发

**NeoTrix 融合**: NT-CORE 的模块化架构可借鉴神经进化的模块化发现。通过 neuroevolution 压力自动发现 NT-* 域间的最优模块分工, 替代当前的手工架构设计。模块化编码提升系统鲁棒性, 符合 "Dark Forest" 存活法则。

---

## 跨主题洞察

### 核心张力

| 维度 | 研究趋势 | NeoTrix 映射 |
|------|---------|-------------|
| **Scaling 预测** | RL training 可从小规模 extrapolate 到大规模 | SEAL pipeline 进化投入的可预测性 |
| **无梯度优化** | ES/CMA-ES 在不可微空间中持续突破 | SelfModel 的无梯度进化路径 |
| **任务对齐** | IRL 从数据对齐转向任务对齐 | 治理规则的隐式奖励函数提取 |
| **约束 BO** | 高维+约束+安全性三重挑战 | E8 hexagram 搜索空间的约束优化 |
| **Pareto 通用性** | 从单目标到 Pareto set 的范式转变 | SelfModel 的多目标 Pareto 最优 |

### 批次共识

1. **Scaling 可预测性是 2026 年 RL 研究的核心主题**: ScaleRL, MBDPO, RL Post-Training Scaling 均关注从小规模预测大规模性能
2. **无梯度方法在高维黑盒优化中持续获得理论突破**: LCBO 的多项式收敛率, ES-AHD 的自动启发式设计
3. **因果推断正在重塑模仿学习**: Scalable Causal IML 解决了长 horizon 中的 compounding error
4. **自适应采样/批大小是效率提升的关键杠杆**: AS-CMA, Adaptive Batch Scaling 均通过自适应分配资源实现效率跃升
5. **模块化是进化压力的自然结果**: 从 neuroevolution 到 GNN-MOGWO, 模块化结构反复出现

### NeoTrix 行动项

- [ ] **P0**: 将 ScaleRL sigmoidal curve fitting 集成到 SEAL pipeline 的 scaling 评估中
- [ ] **P0**: 引入 LCBO 替代 E8 hexagram 全局搜索, 将收敛率从指数降为多项式
- [ ] **P1**: 用 Interactionless IRL 从专家技能演示中提取可复用奖励制品
- [ ] **P1**: 实现 Adaptive Batch Scaling 指导 NT-MIND 进化训练的 batch 调度
- [ ] **P2**: 用 Neural ES PSL 学习 SelfModel 的整个 Pareto set
- [ ] **P2**: 引入 MOO-VARI 平衡 SEAL pipeline 的多目标竞争损失
