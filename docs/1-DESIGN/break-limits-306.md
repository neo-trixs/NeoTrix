# 第92批破限制技术

> 搜索时间: 2026-09-11 | 来源数: 10组搜索, 25+候选

---

## 主题1: 大规模图学习

| # | 来源 | 核心贡献 | 链接 |
|---|------|---------|------|
| 1 | **Graph Learning at Scale** (MLSys 2025) | PP-GNN 解耦传播与训练, 优化数据加载, 吞吐量提升15x, 比采样方法快2个数量级 | https://arxiv.org/abs/2504.13266 |
| 2 | **PyG 2.0** (KDD 2025 Workshop) | 统一图存储/采样/异构处理, 支持十亿节点图, GPU加速+模型编译 | https://arxiv.org/abs/2507.16991 |
| 3 | **ScaleGNN** (WWW 2026) | 自适应高阶邻居特征融合, LCS-based剪枝解决过平滑, 准确率+效率双SOTA | https://arxiv.org/abs/2504.15920 |
| 4 | **GraphScale** (TikTok Production) | 混合并行(data+model), 已部署生产, 训练时间减少40%+, 支持十亿节点 | https://arxiv.org/abs/2407.15452 |
| 5 | **SWIFT** (ACM 2025) | 首个二级存储T-GNN训练系统, bucket-based流水线并行, 单机大规模时序图 | https://dl.acm.org/doi/10.1145/3749184 |

**关键趋势**: 预传播GNN解耦特征传播→训练, 混合并行(data+model), LCS自适应剪枝防过平滑

---

## 主题2: 多智能体协作

| # | 来源 | 核心贡献 | 链接 |
|---|------|---------|------|
| 1 | **LatentMAS** (ICML 2026 Spotlight) | 潜空间协作, 无文本中介, token减少70-83%, 推理加速4x, 表达力更高 | https://arxiv.org/abs/2511.20639 |
| 2 | **Multi-Agent Collaboration Mechanisms Survey** (arXiv 2025) | 框架: actors/types/structures/strategies/protocols, 覆盖5G/工业5.0/社会文化 | https://arxiv.org/abs/2501.06322 |
| 3 | **VIKI-R** (NeurIPS 2025 D&B) | VLM+RL具身多智能体协作, 层级benchmark(agent激活→任务规划→轨迹感知) | https://arxiv.org/abs/2506.09049 |
| 4 | **ACP Protocol** (IBM/Linux Foundation) | Agent通信协议TCP/IP级, REST原生, 零信任安全, 联邦发现, 延迟降低40% | https://arxiv.org/abs/2602.15055 |
| 5 | **Multi-Agent Collaboration via Evolving Orchestration** (NeurIPS 2025) | Puppeteer范式: RL训练编排器动态调度agents, 涌现紧凑循环推理结构 | https://proceedings.neurips.cc/paper_files/paper/2025/hash/f1320d2e2842169c6fc89dcbd80e94d0 |

**关键趋势**: 潜空间协作(无文本中介), 协议标准化(A2A/ACP/MCP), RL动态编排, 具身多模态协作

---

## 主题3: 神经架构搜索

| # | 来源 | 核心贡献 | 链接 |
|---|------|---------|------|
| 1 | **MODNAS** (ICLR 2025) | 多目标可微NAS, Pareto前沿profiling, 跨设备硬件感知优化 | https://arxiv.org/abs/2402.18213 |
| 2 | **SEKI** (arXiv 2025) | LLM驱动NAS: 自进化+知识蒸馏, CoT范式, 自动设计架构无需人工 | https://arxiv.org/abs/2502.20422 |
| 3 | **Efficient Global NAS** (CAIP 2023) | 宏-微搜索空间解耦, 架构感知近似评估, 比最快全局搜索方法快2-4x | https://arxiv.org/abs/2502.03553 |
| 4 | **GNE-NAS** (Nature Scientific Reports 2025) | 代理模型+网络嵌入+GAN数据增强, NASBench上SOTA | https://www.nature.com/articles/s41598-025-30012-6 |
| 5 | **NAS-GAN Survey** (arXiv 2026) | 进化算法+梯度方法在GAN-NAS中的优越性, 综合评估框架 | https://arxiv.org/abs/2606.26169 |

**关键趋势**: LLM驱动NAS, 多目标Pareto优化, 代理模型+GAN加速搜索, 宏微解耦

---

## 主题4: 自适应学习率

| # | 来源 | 核心贡献 | 链接 |
|---|------|---------|------|
| 1 | **Optimal LRS under FSL** (arXiv 2026) | 功能缩放定律下的最优LR调度, phase transition: power decay vs WSD | https://arxiv.org/abs/2602.06797 |
| 2 | **WSqD** (arXiv 2026) | Horizon-free LR调度, 不依赖训练长度, 收敛率最优, 1/√t调度变体 | https://arxiv.org/abs/2607.10959 |
| 3 | **VolSched** (arXiv 2025) | 波动率自适应调度, Hessian分析: 找到38%更平坦解, CIFAR-100 +1.4% top-1 | https://arxiv.org/abs/2507.10575 |
| 4 | **GreedyLR** (arXiv 2025) | 基于loss变化的自适应调整, 7B参数验证, NLP/CV/LLM全面超越SOTA调度 | https://arxiv.org/abs/2512.14527 |
| 5 | **UBA Schedule** (NeurIPS 2025) | 统一预算感知调度, 理论基础, 跨架构/任务/预算一致优于cosine等 | https://proceedings.neurips.cc/paper_files/paper/2025/file/dce0ad3bd4981fea9a5a5a274a2256d9 |

**关键趋势**: WSD调度成为LLM标准, 波动率/loss变化驱动自适应, horizon-free理论, 预算感知统一框架

---

## 主题5: 正则化技术

| # | 来源 | 核心贡献 | 链接 |
|---|------|---------|------|
| 1 | **Regularisation Survey** (IEEE TAI 2026) | 4类正则化分类(data/arch/training/loss), 10数据集+2架构实证: 效果数据依赖 | https://arxiv.org/abs/2601.23131 |
| 2 | **PENEX** (arXiv 2025) | AdaBoost启发的指数损失正则化, 增大margin, 低数据场景SOTA | https://arxiv.org/abs/2510.02107 |
| 3 | **MDL Regularization** (arXiv 2025) | 最小描述长度原则, 平衡复杂度与数据拟合, 优于L1/L2/无正则化 | https://arxiv.org/abs/2505.13398 |
| 4 | **MoReDrop** (ICML 2026) | 无丢弃的Dropout正则化, 只更新稠密模型, 消除train/infer分布偏移 | https://icml.cc/virtual/2024/37162 |
| 5 | **Scaling Laws for Uncertainty** (arXiv 2026) | MC Dropout不确定性缩放定律, γ=-0.36~-0.44, 数据量与不确定性的幂律关系 | https://arxiv.org/abs/2506.09648 |

**关键趋势**: 正则化效果数据依赖(非通用), 指数损失回归, MDL理论框架, Dropout→无丢弃变体, 不确定性缩放定律

---

## 批次总结

| 主题 | 突破点 | 核心模式 |
|------|--------|---------|
| 大规模图学习 | PP-GNN解耦+15x加速, 混合并行部署十亿节点 | 解耦传播/训练, data+model并行 |
| 多智能体协作 | 潜空间协作(-83% token), 协议标准化(TCP/IP级) | 消除文本中介, 协议层标准化 |
| 神经架构搜索 | LLM驱动NAS, 多目标Pareto, 代理模型2-4x加速 | LLM生成+代理评估+Pareto选择 |
| 自适应学习率 | WSD成LLM标准, horizon-free理论, 波动率自适应 | loss-driven调度, 跨长度迁移 |
| 正则化技术 | 效果数据依赖, MDL理论框架, 无丢弃Dropout | 正则化≠通用改善, 需任务匹配 |
