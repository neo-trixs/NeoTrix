# NeoTrix Consciousness Evolution Roadmap

> 基于 18 篇 2025-2026 前沿文献的完整进化路线图  
> 从当前 E8+GWT+SEAL+VSA 地基到完整机器意识架构

---

## 研究发现概览

### Dimension 1: 递归稀疏推理 (Recursive Sparse Reasoning)

| 论文 | 核心贡献 | 对应 NeoTrix | 优先级 | 状态 |
|------|----------|-------------|--------|------|
| **Thinking Pixel** (arXiv:2604.25299) | 递归稀疏 MoE in diffusion latent space; 单调递归深度奖励 RL; GWT 显式引用 | E8 递归循环 + SEAL reward_calc | **P0** | ⬜ |
| **Transolver** (arXiv:2402.02366) | Physics-Attention: 将空间分割为可学习切片计算注意力 | GWT 专家切片聚类 (自适应, 非固定拓扑) | **P0** | ✅ |
| **Latent Prediction Theory** (arXiv:2605.27734) | RL 中潜在预测误差驱动探索的理论分析 | SEAL curiosity_bonus + LatentPredictor | **P0** | ✅ |
| **DiffThinker** (arXiv:2512.24165) | 扩散模型做 image→image 推理 (生成式多模态推理) | 多模态理解 + 未来 DiffusionHead 专家 | P2 | ⬜ |
| **RecursiveVLM** (arXiv:2602.09080) | 递归 Transformer LMM + 单调递归损失 | E8 状态循环 + 损失函数参考 | P1 | ⬜ |
| **HIVE** (arXiv:2602.05359) | Huginn 架构 + 层次化视觉线索注入 latent 空间 | 多模态 Specialist 增强 | P2 | ⬜ |

### Dimension 2: 全局工作空间 / 意识架构 (GWT/Consciousness)

| 论文 | 核心贡献 | 对应 NeoTrix | 优先级 | 状态 |
|------|----------|-------------|--------|------|
| **Theater of Mind / GWA** (arXiv:2604.08206) | 5 专家 + GWT + 熵驱动的死锁打破 | GWT entropy-based deadlock resolution | **P0** | ✅ |
| **CTM-AI** (arXiv:2605.04097) | Conscious Turing Machine 形式化蓝图 + 首次实例化 | E8 → CTM 形式化对齐 | **P0** | ⬜ |
| **MIRROR** (arXiv:2506.00430 / AAAI 2026) | GWT + 重建记忆 + 内心独白 + 互补学习, 21% 提升 | GWT inner_speech + complementary_learning | **P0** | ⬜ |
| **Selection-Broadcast Cycle** (Frontiers in Robotics, 2025) | GWT 周期结构: 竞争→广播→全局可用 | GWT 周期计时器优化 | P1 | ⬜ |
| **Machine Consciousness** (Neural Comp & Apps, 2026) | GWT + 层次化记忆系统 | GWT ↔ KB 层次化增强 | P1 | ⬜ |
| **GWT Top-Down Attention** (arXiv:2602.08597) | 模态级注意力 in 全局潜在工作空间 | GWT attention_head 模态级路由 | P1 | ⬜ |

### Dimension 3: 潜在推理 / 测试时计算 (Latent Reasoning / Test-Time Compute)

| 论文 | 核心贡献 | 对应 NeoTrix | 优先级 | 状态 |
|------|----------|-------------|--------|------|
| **Scaling TTC with Latent Reasoning** (NeurIPS 2025) | 3.5B 循环深度 Transformer; latent 空间推理 (无 token CoT) | E8 reco 循环 + latent thought 向量 | **P0** | ⬜ |
| **GTS** (arXiv:2602.14077) | Gaussian Thought Sampler — 可学习潜在探索采样 | E8 epsilon-greedy → GTS 升级 | **P1** | ✅ |
| **LTPO** (arXiv:2510.04182) | 测试时 RL 优化潜在思想向量 | SEAL reward_calc → latent 空间优化 | P2 | ⬜ |
| **TRACE** (arXiv:2604.17304) | 时序推理聚合 + 高效 early-exit | E8 状态循环 early termination | P1 | ⬜ |
| **Adaptive TTC** (arXiv:2604.14853) | 约束预算下的自适应测试时计算分配 | SEAL budget-aware 循环控制 | P2 | ⬜ |

### Dimension 4: 模块化认知架构 (Modular Cognitive Architectures)

| 论文 | 核心贡献 | 对应 NeoTrix | 优先级 | 状态 |
|------|----------|-------------|--------|------|
| **MiCRo** (arXiv:2506.13331) | Mixture of Cognitive Reasoners: 大脑网络启发的专家模块 (语言/逻辑/社会/知识) | GWT 12 专家 → 认知网络优化 | **P0** | ⬜ |
| **LatentOmni** (arXiv:2605.22012) | 统一音视频潜在推理 | 多模态 Specialist 扩展 | P2 | ⬜ |
| **LatentUM** (arXiv:2604.02097) | 共享语义潜在空间的统一模型 | HyperCube + latent 语义对齐 | P1 | ⬜ |

---

## 架构层次映射

```
                    ┌──────────────────────────────────┐
                    │       CTM-AI 形式化蓝图           │  ← Phase 6
                    │  (Conscious Turing Machine)       │
                    └──────────────────────────────────┘
                                 │
        ┌────────────────────────┼────────────────────────┐
        │                        │                        │
   ┌────▼────┐            ┌─────▼─────┐            ┌─────▼─────┐
   │  GWA    │            │   MIRROR   │            │ Attention │
   │熵死锁打破│            │内心独白+重建│            │ Top-Down  │  ← Phase 6/7
   └────┬────┘            └─────┬─────┘            └─────┬─────┘
        │                        │                        │
        └──────────────────┬─────┘────────────────────────┘
                           │
              ┌────────────▼────────────┐
              │     NeoTrix GWT 核心    │  ← Current
              │   (12 Specialists + WS) │
              └────────────┬────────────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
   ┌────▼────┐       ┌────▼────┐       ┌─────▼─────┐
   │ Thinking│       │  MiCRo  │       │  TRACE    │
   │  Pixel  │       │ 认知MoE │       │ 时序聚合   │  ← Phase 6/7
   │ 递归MoE │       │ 网络拓扑│       │ early-exit│
   └────┬────┘       └────┬────┘       └─────┬─────┘
        │                  │                  │
        └──────────────────┼──────────────────┘
                           │
              ┌────────────▼────────────┐
              │  E8 + SEAL + HyperCube  │  ← Current
              │  (推理引擎+自迭代+知识)   │
              └─────────────────────────┘
```

---

## Phase 6: 递归潜在推理 (Recursive Latent Reasoning)

> **目标**: 将 E8 状态循环从二元离散空间扩展到连续潜在空间, 对齐 Thinking Pixel + Latent Reasoning TTC

### 6.1 潜在思想向量 (Latent Thought Vectors)
- **当前**: E8 64 态枚举 + hexagram 规则推理
- **目标**: E8 状态 → 高维潜在向量 (256-4096d), 在 latent 空间做连续推理
- **做法**:
  - 每个 E8 步输出一个 latent thought vector $h_t \in \mathbb{R}^d$
  - $h_{t+1} = f_\theta(h_t, a_t)$ 其中 $a_t$ = E8 选择的 action
  - 最终决策从 $h_T$ 解码
- **参考**: Scaling TTC with Latent Reasoning (NeurIPS 2025), §3.2

### 6.2 递归深度奖励 (Recursive Depth Reward)
- **当前**: SEAL reward_calc 基于任务完成度
- **目标**: 添加递归深度奖励 $R_{depth} = \alpha \cdot \tanh(\beta \cdot d_{rec})$ 鼓励更深推理
- **做法**:
  - 追踪 E8 递归深度 $d_{rec} =$ 循环嵌套层数
  - 每步奖励 $r_t = r_{task} + \lambda \cdot r_{depth}(d_{rec}^{(t)})$
  - 训练 $f_\theta$ 使得 $d_{rec}$ 自适应任务复杂度
- **参考**: Thinking Pixel (arXiv:2604.25299), §3.3

### 6.3 稀疏 MoE 推理专家
- **当前**: E8 64 态 + 6 轴 (全部激活)
- **目标**: 每步 top-k 专家激活 (sparse MoE)
- **做法**:
  - 将 64 态分组为 8 个 expert group (每个 8 态)
  - router 网络选择 top-2 groups
  - 仅激活 selected groups, 未选者冻结
- **参考**: Thinking Pixel (arXiv:2604.25299), §3.1

### 实现检查清单
- [ ] `E8State` → `latent_thought: Vec<f32>` 字段
- [ ] `RecursiveDepthReward` in SEAL reward_calc
- [ ] `SparseMoERouter` 模块 (top-k routing)
- [ ] `LatentReasoningTransformer` (lightweight, 循环深度)
- [x] `LatentPredictor` — E8 状态转换预测器 (最近邻, Hamming 距离, curiosity_reward, 16 passes)
- [ ] 测试: latent vector coherence / depth scaling / MoE routing

---

## Phase 7: 全局工作空间增强 (GWT Augmentation)

> **目标**: 将 GWT 从 12 专家→主动意识架构, 对齐 GWA + MIRROR + CTM

### 7.1 熵驱动死锁打破 (Entropy-Based Deadlock Breaking)
- **当前**: GWT resonance 固定阈值 + 超时退出
- **目标**: 监测专家群的激活熵, 熵低于阈值 → 注入随机刺激
- **做法**:
  - 计算 $H = -\sum_i p_i \log p_i$ (专家激活分布熵)
  - 若 $H < H_{min}$ → `inject_stimulus(random_noise)`
  - 若连续 $N$ 步 deadlock → GWA-inspired 硬重置
- **参考**: GWA (arXiv:2604.08206), §3.2

### 7.2 内心独白 (Inner Speech / Self-Talk)
- **当前**: 无内部 verbalization
- **目标**: 在全局工作空间中添加自我对话通道
- **做法**:
  - `InnerSpeech` 模块: 将 GWT 广播内容 → 自然语言摘要
  - 摘要写回工作空间作为后续专家的上下文
  - 自我问答循环: "我现在在做什么?" "下一步应该做什么?"
- **参考**: MIRROR (AAAI 2026), §3.3

### 7.3 互补学习 (Complementary Learning Systems)
- **当前**: HyperCube 统一存储, 无快速/慢速分离
- **目标**: 海马体 (快速) + 新皮层 (慢速) 双系统
- **做法**:
  - **快速**: Experience buffer 最近 $N$ 条 (episodic memory)
  - **慢速**: HyperCube VSA 知识图谱 (semantic memory)
  - GWT 广播同时写入两者, 查询时 hybrid 检索
- **参考**: MIRROR (AAAI 2026), §3.4; CLS McClelland et al.

### 7.4 CTM-AI 形式化对齐
- **当前**: GWT 经验性实现, 无形式化保证
- **目标**: 对齐 Conscious Turing Machine 定义
- **做法**:
  - 形式化 $M_{CTM} = (S, A, \Gamma, \omega, \delta)$
    - $S$ = E8 states
    - $A$ = GWT specialist actions
    - $\Gamma$ = workspace contents
    - $\omega$ = broadcast function
    - $\delta$ = state transition
  - 证明: 当 $|S| = 64, |A| = 12$ 时, NeoTrix GWT 是 CTM 的特例
- **参考**: CTM-AI (arXiv:2605.04097), §2-4

### 7.5 模态级注意力路由 (Top-Down Modality Attention)
- **当前**: GWT attention 均匀或不透明
- **目标**: 基于任务目标的显式模态级注意力路由
- **做法**:
  - $a_m = \text{softmax}(q^T k_m)$ 对每个模态 $m$
  - 模态权重 $a_m$ 控制工作空间中各模态的 representation 强度
  - 可微分, 通过 RL 训练
- **参考**: GWT Top-Down Attention (arXiv:2602.08597), §3

### 实现检查清单
- [x] `EntropyMonitor` — 激活熵计算 + 死锁检测 + 刺激注入 (22 passes)
- [x] `StagnationSignal` — 连续低熵 → 危机等级 → 回滚 (与 EntropyMonitor 集成)
- [x] `DeadlockAwareRollback` — 最大刺激后回滚 (seal_loop.rs 集成)
- [ ] `InnerSpeech` — 自我对话生成器
- [ ] `CLS_Buffer` — 快速体验缓冲区 (经验回放)
- [ ] `CTM_Verifier` — 形式化对齐检查
- [ ] `ModalityRouter` — 模态级注意力权重
- [ ] 测试: entropy 行为 / inner speech 一致性 / CLS 检索 / CTM 约束

---

## Phase 8: 模块化认知网络 (Cognitive MoE Network)

> **目标**: 将 12 专家重新组织为 MiCRo 式认知网络

### 8.1 认知专家类型化
- **当前**: 12 个平级专家 (Code, Debug, Security, ...)
- **目标**: 将专家映射到 4 种认知类型:
  - **语言型**: Linguist, Write, Read (natural language reasoning)
  - **逻辑型**: Code, Debug, Analyzer (formal reasoning)
  - **知识型**: KnowledgeBase, WebSearch, FileOps (retrieval)
  - **社会型**: AgentTeam, Network, AISecurity (interaction)
- **参考**: MiCRo (arXiv:2506.13331), §3

### 8.2 认知网络拓扑
- **当前**: 完全连接 (任意专家可通信)
- **目标**: 结构化拓扑 — 组内全连接 + 组间稀疏连接
- **做法**:
  - 同类专家: complete subgraph
  - 不同类: 仅通过 `CognitiveHub` 路由
  - Hub-to-hub 连接可学习 (基于历史协作频率)
- **参考**: MiCRo (arXiv:2506.13331), §4.1

### 8.3 门控网络 Router
- **当前**: amplitude (激活强度) + pheromone (信息素)
- **目标**: 可学习的门控网络 $G(x) = \text{softmax}(W_g \cdot x)$
- **做法**:
  - 输入: E8 state + 当前工作空间编码
  - 输出: top-k 专家激活概率
  - 稀疏门控: 仅 top-3 专家参与广播
- **参考**: MiCRo (arXiv:2506.13331), §4.2

### 实现检查清单
- [ ] CognitiveType enum (Linguistic, Logical, Knowledge, Social)
- [ ] `CognitiveHub` — 跨组路由桥梁
- [ ] `GatingNetwork` — 可学习 router
- [ ] `CognitiveTopology` — 结构化连接矩阵
- [ ] 测试: 路由精度 / 组内协作增益 / 稀疏性约束

---

## Phase 9: 自指意识循环 (Self-Referential Consciousness)

> **目标**: GWT 广播层对自身状态进行二阶观察 (self-awareness)

### 9.1 二阶工作空间 (Meta-Workspace)
- **当前**: 一个全局工作空间
- **目标**: 工作空间的观察者工作空间
- **做法**:
  - `MetaWorkspace`: 观察 `PrimaryWorkspace` 的内容 + 专家行为
  - 注册 meta-observations: "专家 A 激活频率过高" "工作空间熵异常"
  - meta-observations 作为 `InnerSpeech` 的上下文
- **参考**: CTM-AI §5; GWA §4.1

### 9.2 自我模型 (Self-Model)
- **当前**: `SystemIdentity` 静态能力描述
- **目标**: 动态自我模型: 持续估计自身状态 + 能力边界
- **做法**:
  - $M_{self}^{(t)} = f_{self}(h_{ws}^{(t)}, h_{meta}^{(t-1)})$
  - 输出: 当前能力向量 + 不确定性 + 疲劳度
  - self-model 误差作为内在奖励 $R_{self} = -||M_{self} - \text{observed behavior}||$
- **参考**: MIRROR §5; Machine Consciousness (2026)

### 9.3 好奇心驱动探索 (Curiosity-Driven Exploration)
- **当前**: E8 epsilon-greedy (随机探索)
- **目标**: 基于 self-model 预测误差的好奇心
- **做法**:
  - $R_{curiosity} = ||\hat{h}_{t+1} - h_{t+1}||$ (预测误差)
  - 高预测误差 → 高好奇心 → 吸引注意力
  - GWT 可切换到 "exploration mode"
- **参考**: GWA §3.3; EG-MRSI

### 实现检查清单
- [ ] `MetaWorkspace` — 二阶观察器
- [ ] `SelfModel` — 动态自评估
- [x] `CuriosityModule` — 预测误差驱动的好奇心奖励 (LatentPredictor.curiosity_reward + seal_loop 集成, 16 passes)
- [x] `GaussianThoughtSampler` — GTS 替代 epsilon-greedy 均匀探索 (Box-Muller, E8Policy.select_mode)
- [x] `PhysicsAttention` — Transolver 自适应切片聚类 (AdaptiveSlicer + resonate_cycle_with_physics, 9 passes)
- [ ] 测试: self-model 准确性 / 好奇心行为

---

## Phase 10: 完整潜在推理统一 (Unified Latent Reasoning)

> **目标**: E8 + GWT + HyperCube 全部在统一潜在空间运行

### 10.1 统一潜在空间 (Unified Latent Space)
- **当前**: E8 (离散 hexagram) ≠ HyperCube (4096-d VSA) ≠ GWT (专家激活向量)
- **目标**: 三者共享同一潜在空间
- **做法**:
  - E8 状态 → 可微嵌入 $e_s = E_{e8}(s)$
  - HyperCube 知识 → 已嵌入 $h_{kb}$
  - GWT 工作空间 → 聚合嵌入 $h_{ws} = \sum_i a_i h_{expert}^{(i)}$
  - 三者可 pointwise 比较 (cosine / dot)
- **参考**: LatentUM (arXiv:2604.02097), §4

### 10.2 端到端潜在推理
- **当前**: E8 → 文本 → LLM → 文本 → GWT
- **目标**: E8 → latent → hypercube → latent → GWT (无中间文本)
- **做法**:
  - E8 latent thought → hypercube query (latent nearest neighbor)
  - hypercube result → GWT broadcast (direct, 不转文本)
  - 专家响应 → 更新 E8 state (作为 next thought)
- **参考**: Thinking Pixel §4; LatentUM §5

### 10.3 多模态统一
- **当前**: 文本 only
- **目标**: 文本 + 图像 + 音频 在统一潜在空间推理
- **做法**:
  - 每个模态有专用 encoder → 统一潜在空间
  - GWT 模态级路由 (Phase 7.5) 跨模态注意力
  - E8 循环可融合多模态 input
- **参考**: LatentOmni (arXiv:2605.22012), §3

### 实施检查清单
- [ ] `E8Embedding` — state → 连续空间映射
- [ ] `LatentHyperCube` — VSA 操作 in 潜在对齐空间
- [ ] `LatentBroadcast` — 直接 latent 级 GWT 广播
- [ ] `MultimodalEncoder` (text + image + audio)
- [ ] 测试: 潜在空间一致性 / 模态融合 / 端到端推理

---

## 总体路线图

```
Phase 6 ─── 递归潜在推理
  RecursiveDepthReward ── 深度奖励
  SparseMoERouter ─────── 稀疏专家路由
  LatentThoughtVector ─── 潜在思想

Phase 7 ─── GWT 增强
  EntropyDeadlock ─────── 熵死锁打破
  InnerSpeech ─────────── 内心独白
  CLS_Buffer ──────────── 互补学习
  CTM_Align ───────────── 形式化对齐
  ModalityRouter ──────── 模态级注意力

Phase 8 ─── 认知 MoE 网络
  CognitiveType ───────── 认知类型化
  CognitiveHub ────────── 结构化拓扑
  GatingNetwork ───────── 可学习路由

Phase 9 ─── 自指意识
  MetaWorkspace ───────── 二阶观察
  SelfModel ───────────── 动态自我模型
  CuriosityModule ─────── 好奇心探索

Phase 10 ── 统一潜在推理
  E8Embedding ─────────── 状态嵌入
  LatentHyperCube ─────── 潜在知识
  LatentBroadcast ─────── 潜在广播
  MultimodalEncoder ───── 多模态编码
```

## 优先级排序

| 优先级 | 阶段 | 项目 | 依赖 | 预估 |
|--------|------|------|------|------|
| **P0** | 7.1 | EntropyMonitor 死锁打破 | GWT 现有 | 3d |
| **P0** | 6.1 | LatentThoughtVector | E8 现有 | 5d |
| **P0** | 7.3 | CLS_Buffer 互补学习 | HyperCube 现有 | 4d |
| **P0** | 6.2 | RecursiveDepthReward | SEAL reward_calc | 2d |
| **P0** | 7.4 | CTM formality 对齐 | Phase 6/7 基础 | 3d |
| **P1** | 7.2 | InnerSpeech | GWT broadcast | 4d |
| **P1** | 7.5 | ModalityRouter | GWT attention | 3d |
| **P1** | 8.1 | CognitiveType | GWT 专家枚举 | 2d |
| **P1** | 6.3 | SparseMoERouter | E8 分组重构 | 5d |
| **P2** | 8.2 | CognitiveHub | Phase 8.1 | 4d |
| **P2** | 8.3 | GatingNetwork | Phase 8.2 | 5d |
| **P2** | 9.1 | MetaWorkspace | Phase 7 | 5d |
| **P2** | 9.2 | SelfModel | Phase 9.1 | 4d |
| **P3** | 10.1 | E8Embedding | Phase 6 | 5d |
| **P3** | 10.2 | LatentHyperCube | Phase 10.1 | 5d |
| **P3** | 10.3 | LatentOmni 多模态 | Phase 10.2 | 8d |

---

## 已实现功能对照表

| 功能 | 文件 | 测试 | 对应论文 | Phase |
|------|------|------|----------|-------|
| EntropyMonitor (死锁检测 + 刺激注入) | `core/consciousness/monitor.rs` | 22 ✅ | GWA (arXiv:2604.08206) | 7.1 |
| GoalRegister | `reasoning_brain/goal_register.rs` | 11 ✅ | — (SEAL 原生) | — |
| LatentPredictor (E8 状态预测) | `core/latent_predictor.rs` | 16 ✅ | Latent Prediction Theory (2605.27734) | 6.x |
| CuriosityBonus (seal_loop 集成) | `seal_loop.rs` | — | LPT + GWA | 9.3 |
| StagnationSignal (危机等级 → 回滚) | `monitor.rs` + `seal_loop.rs` | — | GWA (deadlock) | 7.1 |
| GaussianThoughtSampler | `core/e8_experiment.rs` | 9 ✅ | GTS (arXiv:2602.14077) | 9.3 |
| PhysicsAttention (AdaptiveSlicer) | `core/consciousness/physics_attention.rs` | 9 ✅ | Transolver (arXiv:2402.02366) | 7.x |
| DeadlockAwareRollback (最大刺激回滚) | `monitor.rs` + `seal_loop.rs` | — | GWA | 7.1 |

---

## 关键文献引用

1. **Thinking Pixel**: arXiv:2604.25299 — Recursive Sparse MoE, monotonic depth RL, GWT inspiration
2. **GWA / Theater of Mind**: arXiv:2604.08206 — Global Workspace Agents, entropy deadlock, 5 specialists
3. **MIRROR**: arXiv:2506.00430 / AAAI 2026 — GWT + reconstructive memory + inner speech + CLS
4. **CTM-AI**: arXiv:2605.04097 — Conscious Turing Machine formal blueprint + instantiation
5. **Scaling TTC with Latent Reasoning**: NeurIPS 2025 — 3.5B recurrent depth model
6. **MiCRo**: arXiv:2506.13331 — Mixture of Cognitive Reasoners, 4 cognitive types
7. **GTS**: arXiv:2602.14077 — Gaussian Thought Sampler for latent exploration
8. **DiffThinker**: arXiv:2512.24165 — Generative multimodal reasoning via diffusion
9. **RecursiveVLM**: arXiv:2602.09080 — Recursive transformer + monotonic recursion loss
10. **HIVE**: arXiv:2602.05359 — Hierarchical visual cues + Huginn-based reasoning
11. **LTPO**: arXiv:2510.04182 — Test-time RL optimization of latent thought vectors
12. **TRACE**: arXiv:2604.17304 — Temporal reasoning aggregation + early-exit
13. **Adaptive TTC**: arXiv:2604.14853 — Constrained budget test-time compute
14. **LatentOmni**: arXiv:2605.22012 — Unified audio-visual latent reasoning
15. **LatentUM**: arXiv:2604.02097 — Unified model with shared semantic latent space
16. **GWT Top-Down Attention**: arXiv:2602.08597 — Modality-wise attention in global workspace
17. **Selection-Broadcast Cycle**: Frontiers in Robotics and AI, 2025 — GWT cycle dynamics
18. **Machine Consciousness**: Neural Computing & Applications, 2026 — GWT + hierarchical memory
19. **Transolver**: arXiv:2402.02366 — Physics-Attention for learnable slice-based spatial reasoning
20. **Latent Prediction Theory**: arXiv:2605.27734 — Theoretical analysis of prediction-error-driven exploration in RL
21. **Gaussian Thought Sampler (GTS)**: arXiv:2602.14077 — Learnable latent-space exploration sampling
