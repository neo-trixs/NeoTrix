# NeoTrix Consciousness Evolution Roadmap

> 基于 18 篇 2025-2026 前沿文献的完整进化路线图  
> 从当前 E8+GWT+SEAL+VSA 地基到完整机器意识架构

---

## 研究发现概览

### Dimension 1: 递归稀疏推理 (Recursive Sparse Reasoning)

| 论文 | 核心贡献 | 对应 NeoTrix | 优先级 | 状态 |
|------|----------|-------------|--------|------|
| **Thinking Pixel** (arXiv:2604.25299) | 递归稀疏 MoE in diffusion latent space; 单调递归深度奖励 RL; GWT 显式引用 | E8 递归循环 + SEAL reward_calc | **P0** | ✅ RewardCalculationStage 已接线 |
| **Transolver** (arXiv:2402.02366) | Physics-Attention: 将空间分割为可学习切片计算注意力 | GWT 专家切片聚类 (自适应, 非固定拓扑) | **P0** | ✅ |
| **Latent Prediction Theory** (arXiv:2605.27734) | RL 中潜在预测误差驱动探索的理论分析 | SEAL curiosity_bonus + LatentPredictor | **P0** | ✅ |
| **DiffThinker** (arXiv:2512.24165) | 扩散模型做 image→image 推理 (生成式多模态推理) | 多模态理解 + 未来 DiffusionHead 专家 | P2 | ⬜ |
| **RecursiveVLM** (arXiv:2602.09080) | 递归 Transformer LMM + 单调递归损失 | E8 状态循环 + 损失函数参考 | P1 | ⬜ |
| **HIVE** (arXiv:2602.05359) | Huginn 架构 + 层次化视觉线索注入 latent 空间 | 多模态 Specialist 增强 | P2 | ⬜ |

### Dimension 2: 全局工作空间 / 意识架构 (GWT/Consciousness)

| 论文 | 核心贡献 | 对应 NeoTrix | 优先级 | 状态 |
|------|----------|-------------|--------|------|
| **Theater of Mind / GWA** (arXiv:2604.08206) | 5 专家 + GWT + 熵驱动的死锁打破 | GWT entropy-based deadlock resolution | **P0** | ✅ |
| **CTM-AI** (arXiv:2605.04097) | Conscious Turing Machine 形式化蓝图 + 首次实例化 | E8 → CTM 形式化对齐 | **P0** | ✅ CtmVerifier 已接线 |
| **MIRROR** (arXiv:2506.00430 / AAAI 2026) | GWT + 重建记忆 + 内心独白 + 互补学习, 21% 提升 | GWT inner_speech + complementary_learning | **P0** | ✅ InnerSpeech + 熵监控 |
| **Selection-Broadcast Cycle** (Frontiers in Robotics, 2025) | GWT 周期结构: 竞争→广播→全局可用 | GWT 周期计时器优化 | P1 | ✅ 周期 + monitor 熵死锁 |
| **Machine Consciousness** (Neural Comp & Apps, 2026) | GWT + 层次化记忆系统 | GWT ↔ KB 层次化增强 | P1 | ✅ MemoryGraphSpecialist |
| **GWT Top-Down Attention** (arXiv:2602.08597) | 模态级注意力 in 全局潜在工作空间 | GWT attention_head 模态级路由 | P1 | ✅ ModalityRouter |

### Dimension 3: 潜在推理 / 测试时计算 (Latent Reasoning / Test-Time Compute)

| 论文 | 核心贡献 | 对应 NeoTrix | 优先级 | 状态 |
|------|----------|-------------|--------|------|
| **Scaling TTC with Latent Reasoning** (NeurIPS 2025) | 3.5B 循环深度 Transformer; latent 空间推理 (无 token CoT) | E8 reco 循环 + latent thought 向量 | **P0** | ✅ LatentThoughtVector + unified_latent |
| **GTS** (arXiv:2602.14077) | Gaussian Thought Sampler — 可学习潜在探索采样 | E8 epsilon-greedy → GTS 升级 | **P1** | ✅ |
| **LTPO** (arXiv:2510.04182) | 测试时 RL 优化潜在思想向量 | SEAL reward_calc → latent 空间优化 | P2 | ⬜ |
| **TRACE** (arXiv:2604.17304) | 时序推理聚合 + 高效 early-exit | E8 状态循环 early termination | P1 | ⬜ |
| **Adaptive TTC** (arXiv:2604.14853) | 约束预算下的自适应测试时计算分配 | SEAL budget-aware 循环控制 | P2 | ⬜ |

### Dimension 4: 模块化认知架构 (Modular Cognitive Architectures)

| 论文 | 核心贡献 | 对应 NeoTrix | 优先级 | 状态 |
|------|----------|-------------|--------|------|
| **MiCRo** (arXiv:2506.13331) | Mixture of Cognitive Reasoners: 大脑网络启发的专家模块 (语言/逻辑/社会/知识) | GWT 12 专家 → 认知网络优化 | **P0** | ✅ GWT 15 个 SpecialistType |
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
- [x] `E8State` → `latent_thought` 嵌入 (nt_latent_thought.rs, 10 tests, Gauss-kernel 可微 embed/interpolate/nearest_state) ✅ iter192
- [x] `RecursiveDepthReward` in SEAL reward_calc (recursive_depth_reward.rs, 6 tests, seal_loop 融合点接线) ✅ iter192
- [x] `SparseMoERouter` 模块 (sparse_moe.rs, 13 tests, top-k=2 稀疏路由 + 质量守恒掩码) ✅ iter193
- [x] `LatentReasoningTransformer` (nt_latent_transformer.rs, 10 tests, 循环深度 + 潜在相干性 + 收敛检测) ✅ 2026-08-13
- [x] `LatentPredictor` — E8 状态转换预测器; 语义由已有 `E8PredictionEnsemble`/`E8MctsPredictor`/`E8PredictionOracle` (nt_core_e8_prediction.rs, engine_core.rs 接线) 承担; curiosity_reward 已接线 seal_loop: 好奇心融合真实 E8 预测不确定性 (1-last_e8_confidence), R_curiosity = ||ĥ-h|| (seal_loop.rs #5) ✅ 2026-08-14
- [x] 测试: latent vector coherence / depth scaling / MoE routing (sparse_moe.rs 路由精度/稀疏性 + nt_latent_transformer.rs 深度增益) ✅ 2026-08-13

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
- [x] `InnerSpeech` — 自我对话生成器 (inner_speech.rs, 11 tests, GWT 广播→独白→context 写回)
- [x] `ModalityRouter` — 模态级注意力权重 (modality_router.rs, 21 tests, $a_m=\text{softmax}(q^T k_m)$ + REINFORCE, resonant_broadcast Step 4c 接线)
- [x] `CLS_Buffer` — 快速体验缓冲区 (cls_buffer.rs, 10+2 tests, ring buffer + hybrid 检索, resonant_broadcast Step 4d 写入)
- [x] `CTM_Verifier` — 形式化对齐检查 (ctm_verifier.rs, 7 tests, 5 公理: finite-state/finite-action/globality/deterministic-delta/bounded-tape, Step 4e 接线)
- [x] 测试: entropy 行为 / inner speech 一致性 / CLS 检索 / CTM 约束

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
- [x] CognitiveType enum (Linguistic, Logical, Knowledge, Social) — cognitive_type.rs, 10 tests, Step 4f 接线 ✅ iter192
- [x] `CognitiveHub` — 跨组路由桥梁 (cognitive_hub.rs, 8 tests, 4×4 可学习 hub 权重 + top-2 稀疏路由, broadcast 协作记录) ✅ iter193
- [x] `GatingNetwork` — 可学习 router (MoERouter::sparse_gate, 4 tests, G(x)=softmax(W_g·x) top-3 广播门控) ✅ iter193
- [x] `CognitiveTopology` — 结构化连接矩阵 (组内全连接 + 组间仅经 hub, 由 CognitiveHub 覆盖) ✅ iter193
- [x] 测试: 路由精度 / 组内协作增益 / 稀疏性约束 (sparse_moe.rs 路由精度 + 稀疏性约束, 8.1/6.3 覆盖) ✅ 2026-08-13

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
- [x] `MetaWorkspace` — 二阶观察器 (meta_workspace.rs, 9 tests, overactivation/entropy-anomaly/gate-fixation, 接线 InnerSpeech) ✅ iter193
- [x] `SelfModel` — 动态自评估 (self_model.rs, 8 tests, capability+uncertainty+fatigue, self-error 内在奖励接线 SEAL) ✅ iter193
- [x] `CuriosityModule` — 预测误差驱动的好奇心奖励; 等价信号由 seal_loop 承担并已升级为融合 E8 预测不确定性 (1-last_e8_confidence), 非独立模块 (seal_loop.rs #5) ✅ 2026-08-14
- [x] `GaussianThoughtSampler` — GTS 思想已吸收进 E8Policy 探索: select_mode 用 Box-Muller Gaussian 采样替代均匀 epsilon-greedy, sigma ∝ epsilon, 中心为最佳已知模式 (nt_core_policy.rs:101-123) ✅ 2026-08-14 修正
- [x] `PhysicsAttention` — Transolver 自适应切片聚类 ✅ 代码存在 (nt_core_gwt/physics_attention.rs, AdaptiveSlicer 274 行, 接线 workspace.rs + resonance.rs) 2026-08-14 修正
- [x] 测试: self-model 准确性 / 好奇心行为 (self_model.rs 准确性收敛 2 组) ✅ 2026-08-13

---

## Phase 10: 完整潜在推理统一 (Unified Latent Reasoning)

> **目标**: E8 + GWT + HyperCube 全部在统一潜在空间运行

### 10.1 统一潜在空间 (Unified Latent Space) ✅
- **当前**: E8 (离散 hexagram) ≠ HyperCube (4096-d VSA) ≠ GWT (专家激活向量)
- **目标**: 三者共享同一潜在空间
- **做法**:
  - E8 状态 → 可微嵌入 $e_s = E_{e8}(s)$
  - HyperCube 知识 → 已嵌入 $h_{kb}$
  - GWT 工作空间 → 聚合嵌入 $h_{ws} = \sum_i a_i h_{expert}^{(i)}$
  - 三者可 pointwise 比较 (cosine / dot)
- **参考**: LatentUM (arXiv:2604.02097), §4

### 10.2 端到端潜在推理 ✅
- **当前**: E8 → 文本 → LLM → 文本 → GWT
- **目标**: E8 → latent → hypercube → latent → GWT (无中间文本)
- **做法**:
  - E8 latent thought → hypercube query (latent nearest neighbor)
  - hypercube result → GWT broadcast (direct, 不转文本)
  - 专家响应 → 更新 E8 state (作为 next thought)
- **参考**: Thinking Pixel §4; LatentUM §5

### 10.3 多模态统一 ✅
- **当前**: 文本 only
- **目标**: 文本 + 图像 + 音频 在统一潜在空间推理
- **做法**:
  - 每个模态有专用 encoder → 统一潜在空间
  - GWT 模态级路由 (Phase 7.5) 跨模态注意力
  - E8 循环可融合多模态 input
- **参考**: LatentOmni (arXiv:2605.22012), §3

### 实施检查清单
- [x] `E8Embedding` — state → 连续空间映射 (unified_latent.rs 的 project_e8_state + SeededProjection JL 式确定性投影, 8 tests) ✅ iter193
- [x] `LatentHyperCube` — VSA 操作 in 潜在对齐空间 (unified_latent.rs 的 project_vsa, 2048→256 保 cosine) ✅ iter193
- [x] `LatentBroadcast` — 直接 latent 级 GWT 广播 (nt_latent_reasoning.rs 的 to_gwt_attention → set_e8_attention_weights, 无中间文本, 8 tests) ✅ iter193
- [x] `MultimodalEncoder` (text + image + audio) (nt_multimodal.rs, 7 tests, char n-gram hash kernel + ModalityRouter 融合 → E8 mode) ✅ iter193
- [x] `UnifiedLatentSpace` — 跨域共享空间 (unified_latent.rs, project_e8/project_workspace/project_vsa + cosine/dot, 接线 engine_core) ✅ iter193
- [x] `LatentReasoningPipeline` — 潜在 episodic 检索 (nt_latent_reasoning.rs, LATENT_MEMORY_SIZE=256, outcome 加权 top-k 注意力) ✅ iter193
- [x] 测试: 潜在空间一致性 / 模态融合 / 端到端推理 (跨域 cosine/确定性/融合主导模态 + `test_end_to_end_latent_reasoning_flow` 端到端检索→GWT 注意力) ✅ 2026-08-13

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
| **P0** | 7.1 | EntropyMonitor 死锁打破 | GWT 现有 | 3d | ✅ iter191 |
| **P0** | 6.1 | LatentThoughtVector | E8 现有 | 5d | ✅ iter192 |
| **P0** | 7.3 | CLS_Buffer 互补学习 | HyperCube 现有 | 4d | ✅ iter191 |
| **P0** | 6.2 | RecursiveDepthReward | SEAL reward_calc | 2d | ✅ iter192 |
| **P0** | 7.4 | CTM formality 对齐 | Phase 6/7 基础 | 3d | ✅ iter191 |
| **P1** | 7.2 | InnerSpeech | GWT broadcast | 4d | ✅ iter190 |
| **P1** | 7.5 | ModalityRouter | GWT attention | 3d | ✅ iter191 |
| **P1** | 8.1 | CognitiveType | GWT 专家枚举 | 2d | ✅ iter192 |
| **P1** | 6.3 | SparseMoERouter | E8 分组重构 | 5d | ✅ iter193 |
| **P2** | 8.2 | CognitiveHub | Phase 8.1 | 4d | ✅ iter193 |
| **P2** | 8.3 | GatingNetwork | Phase 8.2 | 5d | ✅ iter193 |
| **P2** | 9.1 | MetaWorkspace | Phase 7 | 5d | ✅ iter193 |
| **P2** | 9.2 | SelfModel | Phase 9.1 | 4d | ✅ iter193 |
| **P3** | 10.1 | E8Embedding | Phase 6 | 5d | ✅ iter193 |
| **P3** | 10.2 | LatentHyperCube | Phase 10.1 | 5d | ✅ iter193 |
| **P3** | 10.3 | LatentOmni 多模态 | Phase 10.2 | 8d | ✅ iter193 |

---

## 已实现功能对照表

| 功能 | 文件 | 测试 | 对应论文 | Phase |
|------|------|------|----------|-------|
| EntropyMonitor (死锁检测 + 刺激注入) | `core/consciousness/monitor.rs` | 22 ✅ | GWA (arXiv:2604.08206) | 7.1 |
| GoalRegister | `l8_autonomic_impl/nt_mind/goal_contract.rs` | 11 ✅ | — (SEAL 原生) | — |
| LatentPredictor (E8 状态预测) | 已有 `nt_core_e8_prediction.rs` (Ensemble/MCTS/Oracle, engine_core.rs 接线) + seal_loop #5 融合 E8 预测不确定性 | ✅ | Latent Prediction Theory (2605.27734) | 6.x |
| CuriosityBonus (seal_loop 集成) | `loop_impl/seal_loop.rs` (reward_gap + E8 预测不确定性融合) | ✅ | LPT + GWA | 9.3 |
| StagnationSignal (危机等级 → 回滚) | `monitor.rs` + `seal_loop.rs` | — | GWA (deadlock) | 7.1 |
| GaussianThoughtSampler | 已吸收进 `nt_core_policy.rs` select_mode (Box-Muller Gaussian 探索, sigma ∝ epsilon) | ✅ | GTS (arXiv:2602.14077) | 9.3 |
| PhysicsAttention (AdaptiveSlicer) | `nt_core_gwt/physics_attention.rs` (接线 workspace.rs + resonance.rs) | ✅ | Transolver (arXiv:2402.02366) | 7.x |
| DeadlockAwareRollback (最大刺激回滚) | `monitor.rs` + `seal_loop.rs` | — | GWA | 7.1 |
| InnerSpeech (内心独白 + 上下文写回) | `core/nt_core_gwt/inner_speech.rs` | 11 ✅ | MIRROR (AAAI 2026) §3.3 | 7.2 |
| SparseMoERouter (top-2 专家组路由) | `core/nt_core_e8/sparse_moe.rs` | 13 ✅ | Thinking Pixel §3.1 | 6.3 |
| LatentReasoningTransformer (轻量循环推理) | `core/nt_core_e8/nt_latent_transformer.rs` | 10 ✅ | Thinking Pixel §4 | 6.3 |
| CognitiveHub (跨组路由桥梁) | `core/nt_core_gwt/cognitive_hub.rs` | 8 ✅ | MiCRo §4.1 | 8.2 |
| SparseGate (top-3 广播门控) | `core/nt_core_gwt/moe_router.rs` | 4 ✅ | MiCRo §4.2 | 8.3 |
| MetaWorkspace (二阶观察器) | `core/nt_core_gwt/meta_workspace.rs` | 9 ✅ | CTM-AI §5 / GWA §4.1 | 9.1 |
| SelfModel (动态自评估) | `core/nt_core_self/self_model.rs` | 8 ✅ | MIRROR §5 | 9.2 |
| UnifiedLatentSpace (跨域共享空间) | `core/nt_core_e8/unified_latent.rs` | 8 ✅ | LatentUM (2604.02097) §4 | 10.1 |
| LatentReasoningPipeline (端到端潜在推理) | `core/nt_core_e8/nt_latent_reasoning.rs` | 8 ✅ | Thinking Pixel §4 / LatentUM §5 | 10.2 |
| MultimodalEncoder (多模态统一) | `core/nt_core_e8/nt_multimodal.rs` | 7 ✅ | LatentOmni (2605.22012) §3 | 10.3 |

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
