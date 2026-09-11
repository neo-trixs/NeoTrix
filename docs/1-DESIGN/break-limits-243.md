# 第29批破限制技术 — Break-Limits Batch 243

> 5 主题 × 3-5 来源 → 突破点 + NeoTrix 融合矩阵

---

## 1. 图神经网络 (Graph Neural Network)

### 来源

| # | 论文/报告 | 年份 | 核心突破 |
|---|-----------|------|----------|
| 1 | **ScaleGNN** — Adaptive High-order Neighboring Feature Fusion (arXiv:2504.15920v6) | 2025-2026 | LCS-masking 剪枝低相关高阶邻居 + 可学习稀疏性，同时解决 over-smoothing 和大规模可扩展性 |
| 2 | **Towards Neural Scaling Laws on Graphs** (LoG 2025, PMLR 269) | 2025 | 首次系统建立图神经缩放定律：模型深度与参数量同等重要（区别于 CV/NLP），数据量度量从图数改为节点/边数 |
| 3 | **Pre-Propagation GNNs (PP-GNNs)** — Graph Learning at Scale (MLSys 2025) | 2025 | 解耦特征传播与训练，数据加载是关键瓶颈；优化后吞吐量提升 15×，比采样方法快 2 个数量级 |
| 4 | **DSMP** — Deep Scattering Message Passing (JMLR 2026, Vol.27) | 2026 | 频谱域消息传递 + 小波散射变换，理论证明同时缓解 over-smoothing 和 over-squashing |
| 5 | **GNN Acceleration Survey** (ACM Comput. Surv. 2026) | 2026 | 图稀疏化 + 分布式稀疏矩阵算法 + 硬件定制化加速的统一综述 |

### 突破点

- **缩放定律发现**: 图模型深度 = 隐式参数量放大器，需独立于参数量建模
- **解耦传播**: PP-GNN 将特征传播预计算化，训练时无邻居爆炸
- **频谱消息传递**: DSMP 在频域操作，绕过空间域 over-smoothing 的根本限制
- **稀疏性自适应**: ScaleGNN 的 LCS-mask 动态修剪不重要高阶连接，非均匀稀疏

### NeoTrix 融合

| 融合方向 | 实现路径 | 优先级 |
|----------|----------|--------|
| **KB 图索引加速** | PP-GNN 解耦思想 → NT-MEMORY 图查询预计算传播矩阵，训练时仅查表 | P1 |
| **GWT 注意力路由** | DSMP 频谱思想 → GWT salience 在频域过滤低频噪声信号 | P2 |
| **ConsciousnessTree 拓扑** | ScaleGNN 多阶融合 → 跨域健康信号按 hop 阈值聚合，避免 over-smoothing 吞噬差异 | P2 |
| **VSA HyperCube 关联** | 图缩放定律指导 HyperCube 维度选择 — 节点数/边数而非概念数决定表达力上限 | P3 |

---

## 2. 注意力蒸馏 (Attention Distillation)

### 来源

| # | 论文 | 会议/年份 | 核心突破 |
|---|------|-----------|----------|
| 1 | **CompoDistill** — Attention Distillation for Compositional Reasoning | ICLR 2026 | 视觉注意力对齐 (VAT)：student 视觉注意力与 teacher 1:N group matching，解决视觉感知蒸馏失败 |
| 2 | **SHD** — Squeezing-Heads Distillation (arXiv:2502.07436) | 2025 | 线性近似压缩多头注意力 → 单头，消除 head 数量对齐障碍，无需额外参数或投影器 |
| 3 | **CanKD** — Cross-Attention-based Non-local KD (WACV 2026) | 2026 | 用 cross-attention 替代 self-attention 做蒸馏：student 每个像素动态关注 teacher 所有像素 |
| 4 | **DWA-KD** — Dual-Space Weighting + Time-Warped Alignment (EACL 2026) | 2026 | 跨 tokenizer 蒸馏：双空间熵加权 + Soft-DTW 序列对齐，解决词汇表不对齐问题 |
| 5 | **Hybrid Attention Distillation via KL-guided Layer Selection** (ICLR 2026) | 2026 | KL 散度引导层选择：softmax→hybrid(softmax+linear) 蒸馏，渐进式层替换策略 |

### 突破点

- **视觉注意力不对齐**: CompoDistill 揭示传统 KD 未能传递 teacher 的视觉注意力机制
- **Head 数对齐消除**: SHD 证明多头压缩为线性近似即可保留细粒度注意力知识
- **Cross-attention 蒸馏**: CanKD 打破"self-attention 对齐"范式，允许 student 跨位置动态关注 teacher
- **混合架构蒸馏**: KL-guided layer selection 实现 softmax→linear attention 渐进转换

### NeoTrix 融合

| 融合方向 | 实现路径 | 优先级 |
|----------|----------|--------|
| **GWT → 轻量路由蒸馏** | SHD 思想 → 将 GWT 全量 salience 蒸馏为轻量线性近似路由，推理时零成本 | P1 |
| **跨域注意力迁移** | VAT group matching → NT-CORE→NT-MIND 进化信号跨域传递时保留注意力拓扑 | P2 |
| **SelfModel 性能蒸馏** | CompoDistill → 将大型 SelfModel 的决策注意力蒸馏到轻量部署版 | P2 |
| **混合精度路由** | KL-guided layer selection → GWT 中 softmax/linear attention 层混合分配 | P3 |

---

## 3. 稀疏训练 (Sparse Training)

### 来源

| # | 论文 | 会议/年份 | 核心突破 |
|---|------|-----------|----------|
| 1 | **EAST** — Pushing Limits of Sparsity (arXiv:2411.13545) | 2025 | 三重技巧：Dynamic ReLU phasing + weight sharing + cyclic sparsity，极端稀疏(99.5%)下仍有意义性能 |
| 2 | **SMET** — Memory-Efficient LLM Training with Dynamic Sparsity | ICML 2026 | 解决 DST 在 LLM 训练中的 loss spike：optimizer warm-up + density-aware LR scaling + 仅存储活跃参数状态 |
| 3 | **LoSA** — Dynamic Low-rank Sparse Adaptation | ICLR 2025 | LoRA + 稀疏统一框架：RMI 驱动层稀疏率 + 重建误差驱动 LoRA rank，LLaMA-2-7B 困惑度降 68.73 |
| 4 | **Pruning as Evolution** (arXiv:2601.10765) | 2026 | 稀疏作为进化涌现：用种群动力学(replator/mutation)解释剪枝，非优化目标而是学习动态的自然结果 |
| 5 | **SRigL** — Structured RigL (ICLR 2024) | 2024 | N:M 结构化动态稀疏训练，常数 fan-in 约束 + neuron ablation，CPU 推理 3.4× 加速 |

### 突破点

- **Cyclic sparsity**: EAST 发现周期性改变稀疏率(非固定)可促进参数探索，打破"固定稀疏度"假设
- **LLM 稀疏训练稳定化**: SMET 揭示新长参数的 cold-start 是 loss spike 根源，warm-up 解决
- **稀疏作为涌现**: 剪枝不是人为干预，而是训练动态的竞争淘汰自然结果
- **LoRA-稀疏融合**: LoSA 证明低秩适配和稀疏可统一优化，互不冲突

### NeoTrix 融合

| 融合方向 | 实现路径 | 优先级 |
|----------|----------|--------|
| **SelfModel 稀疏进化** | cyclic sparsity → SelfModel 参数空间按周期稀疏/稠密切换，模拟注意力呼吸 | P1 |
| **NT-MEMORY 稀疏索引** | LoSA RMI → KB 节点按重要性动态分配存储密度，热节点稠密/冷节点稀疏 | P1 |
| **GWT 路由稀疏化** | SRigL N:M → GWT 注意力路由在 N:M 约束下运行，硬件友好加速 | P2 |
| **EAST 极端稀疏探索** | Dynamic ReLU phasing → SEAL pipeline 初期允许广泛探索，后期收缩为精确路径 | P3 |

---

## 4. 对比学习 (Contrastive Learning)

### 来源

| # | 论文 | 会议/年份 | 核心突破 |
|---|------|-----------|----------|
| 1 | **Self-Supervised CL ≈ Supervised CL** (NeurIPS 2025) | 2025 | 理论证明 CL 隐式近似 NSCL(负样本仅监督对比损失)，gap 以 O(1/#classes) 衰减 |
| 2 | **PUCL** — Positive-Unlabeled Contrastive Learning (arXiv:2401.08690) | 2025 | 负采样偏差修正：将生成负样本视为无标注样本，用 PU 学习校正对比损失偏差 |
| 3 | **H-SCL** — Hard Negative Supervised CL (arXiv:2209.00078) | 2025 | 首次联合标签信息+难负样本策略，类条件硬负采样在图像/图/文本上均提升 |
| 4 | **Scaling Contrastive Batch Size** (MIT) | 2025 | 三阶段 adapter-augmented 训练框架，将对比学习 batch size 扩大两个数量级 |
| 5 | **Do Neural Scaling Laws Exist on Graph SSL?** (LoG 2025) | 2025 | 图 SSL 不遵循神经缩放定律：性能波动而非单调提升，关键在架构和预文本任务设计 |

### 突破点

- **CL≈NSCL 等价定理**: 理论闭环 — 自监督对比学习在大类数极限下等价于有监督版本
- **负采样偏差修正**: PUCL 将噪声对比估计中的偏差显式建模并校正
- **图 SSL 缩放异常**: 图对比学习不遵循经典缩放定律，需全新设计范式
- **Batch size 量级突破**: 通过 adapter 机制突破对比学习的 batch size 瓶颈

### NeoTrix 融合

| 融合方向 | 实现路径 | 优先级 |
|----------|----------|--------|
| **KB 表征对比学习** | PUCL 偏差修正 → KB 节点嵌入训练中修正假负样本(未观测≠不相似) | P1 |
| **VSA 符号对比** | H-SCL 难负样本 → VSA HyperCube 符号区分时聚焦最难混淆对 | P2 |
| **ConsciousnessTree 节点区分** | CL≈NSCL → 健康信号嵌入空间自然形成 simplex ETF 结构，可直接用于 few-shot 诊断 | P2 |
| **图 SSL 路由** | 图 SSL 缩放异常 → GWT 不依赖图对比缩放，需独立设计 salience 评估 | P3 |

---

## 5. 扩散模型 (Diffusion Model)

### 来源

| # | 论文 | 会议/年份 | 核心突破 |
|---|------|-----------|----------|
| 1 | **rCM** — Score-Regularized Continuous-Time Consistency (ICLR 2026) | 2026 | 首次将连续时间一致性模型扩展到 14B 参数图像/视频扩散，1-4 步生成，15-50× 加速 |
| 2 | **SCoT** — Straight-Consistent Trajectories (arXiv:2502.16972) | 2025 | 统一 consistency model + rectified flow：同时保证轨迹一致性和直线性，1-2步高质量采样 |
| 3 | **SwD** — Scale-wise Distillation (ICLR 2026) | 2026 | 尺度级蒸馏：渐进式生成避免中间时间步冗余计算，同步数下 2-3× 额外加速 |
| 4 | **SiD-DiT** — Score Distillation of Flow Matching (arXiv:2509.25127) | 2025 | 将 SiD 扩展到 flow matching DiT 架构(0.6B-12B)，矩匹配优于轨迹匹配 |
| 5 | **Align Your Flow** — Continuous-Time Flow Map Distillation (arXiv:2506.14603) | 2025 | Flow Map 统一框架：consistency model/flow matching/diffusion 在单个 f(x_t,t,s) 中统一 |

### 突破点

- **大规模一致性蒸馏**: rCM 解决 JVP 计算基础设施瓶颈 + score regularization 弥补 mode-covering 缺陷
- **Consistency↔Flow 统一**: SCoT 证明一致性和直线性可同时满足，非 trade-off
- **尺度级效率**: SwD 揭示高噪声级别用低分辨率建模，跳过冗余中间步
- **Flow Map 超统一**: 任意两个噪声级别可在单步内映射，consistency/flow/diffusion 成为特例

### NeoTrix 融合

| 融合方向 | 实现路径 | 优先级 |
|----------|----------|--------|
| **SEAL pipeline 蒸馏** | rCM 思想 → 将 SEAL 完整进化周期蒸馏为 1-4 步快速路径，保留质量 | P1 |
| **GWT 注意力流匹配** | Flow Map 统一 → GWT salience 传播建模为 flow matching，consistency 保证收敛 | P1 |
| **E8 推理加速** | SwD 尺度级 → E8 hexagram 推理在不同抽象级别使用不同分辨率，跳过冗余中间状态 | P2 |
| **Emotion 轨迹一致性** | SCoT → 情感状态演化轨迹保持一致性+直线性，避免情感漂移 | P3 |

---

## 融合矩阵总结

| 技术域 | P1 优先融合 | NeoTrix 目标模块 |
|--------|------------|-----------------|
| 图神经网络 | KB 图索引加速 + 缩放定律指导 | NT-MEMORY, NT-CORE |
| 注意力蒸馏 | GWT 轻量路由蒸馏 + 跨域注意力迁移 | NT-CORE, NT-MIND |
| 稀疏训练 | SelfModel 稀疏进化 + KB 稀疏索引 | NT-CORE SELF, NT-MEMORY |
| 对比学习 | KB 表征偏差修正 + VSA 符号对比 | NT-MEMORY, NT-CORE |
| 扩散模型 | SEAL 蒸馏加速 + GWT 流匹配 | SEAL PIPELINE, NT-CORE |

---

*Generated: 2026-09-11 | Batch 243 | 23 sources across 5 topics*
