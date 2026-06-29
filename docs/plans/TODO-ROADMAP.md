# TODO: NeoTrix 进化路线图 — Phase 0-240 可执行任务清单

> 基准: 2026-06-22 | 更新: 2026-06-22 | 6 P0 已完成 → 进入 P1 阶段
> 参考: EVOLUTION_ROADMAP_v6.md, CLIX 并行实施
> 已完成: P105 FPE / P108 CCIPCA / P129 CRUD / P150 因果追踪 / P123 RL巩固 / P156 叠加验证

---

## Phase 0-100: 认知架构基础 (CLII P0 缺口)

> 2026-06-22 四路并行实施，全部编译通过，0 错误 0 新警告。

### P0-R1 — System 1 直觉系统 ✅

**缺口**: CLII #1 — 快思考通道 (模式匹配/联想/自动化)
**参考**: DPT-Agent (ACL 2025), System 1→2 Survey (arXiv 2502.17419)
**文件**: 📄 `neotrix-core/src/core/nt_core_consciousness/system1.rs`

状态:
```text
✅ FastPatternMatcher — VSA cosine similarity, <1ms 目标
✅ IntuitionBuffer — 100 条环形缓存 (situation→action)
✅ ConflictDetector — S1 vs S2 矛盾检测 + 加权裁决
✅ 5 种直觉启发式: Similarity/Frequency/Proximity/Recency/Emotion
✅ CognitiveRouter 接线 (nt_io_router.rs)
✅ 22 测试
```

### P0-R2 — Hierarchical World Model ✅

**缺口**: CLII #2 — 3 层预测编码世界模型
**参考**: PrediRep (Neural Networks 2025), Millidge & Seth PC Review
**文件**: 📄 `neotrix-core/src/core/nt_core_consciousness/hierarchical_world_model.rs`

状态:
```text
✅ Perception(100ms) / Action(1s) / Narrative(10s+) 三层
✅ 每层: VSA latent → predict → prediction error (1 - VSA similarity)
✅ 层间: top-down 预测 / bottom-up error 传播
✅ ImaginationEngine — 未来轨迹 rollout
✅ 22 测试
```

### P0-R3 — Counterfactual Reasoner ✅

**缺口**: CLII #3 — 反事实推理 (Pearl Ladder Step 3)
**参考**: CounterBench (arXiv 2502.11008, 2026), Pearl's Ladder
**文件**: 📄 `neotrix-core/src/core/nt_core_consciousness/counterfactual.rs`

状态:
```text
✅ StructuralCausalModel — VSA 因果 DAG
✅ CounterfactualGenerator — 最小修改反事实场景
✅ CoInReasoner — 迭代回溯反事实空间
✅ 3 种反问: 不做X / 做了Y / 条件不同
✅ 15 测试
```

### P0-R4 — Active Inference Engine ✅

**缺口**: CLII #4 — Expected Free Energy 闭环
**参考**: Active Inference for Physical AI (2025), RxInfer.jl, AIF for Multi-LLM (arXiv 2412.10425)
**文件**: 📄 `neotrix-core/src/core/nt_core_consciousness/active_inference.rs`

状态:
```text
✅ GenerativeModel — POMDP (hidden states + observations + actions)
✅ ExpectedFreeEnergy — risk(KL) + ambiguity(entropy)
✅ PolicySelector — EFE 最小化 + softmax 策略选择
✅ BeliefUpdater — VFE 最小化消息传递
✅ 26 测试
```

---

## 符号说明

- `cargo check -p neotrix --lib` = 编译验证命令
- `cargo test -p neotrix --lib -- <test_name>` = 测试验证命令
- 📄 = 新建文件 | ✏️ = 编辑现有文件 | 🔗 = 外部参考
- 依赖: 前置任务编号

---

## Phase 105-156: P0 任务全部完成 ✅ (2026-06-22)

### P105 — FPE 连续标量编码器 ✅

**缺口**: V1 — Fractional Power Encoding / Spatial Semantic Pointer
**参考**: holon-rs `scalar.rs` encode_scalar/log/circular; SSP (Spatial Semantic Pointer) FHRR 实现; arXiv 2604.22863 Wave-Geometric Duality; arXiv 2412.00488 Bremer & Orchard MLE+CLE 解码
**命令**: `cargo check -p neotrix --lib`
**文件**: 📄 `neotrix-core/src/core/nt_core_hcube/fpe.rs` (426 行, 15 测试)

任务:
```text
1. [x] FPE 核心: encode_scalar(value, dims, length_scale) → VsaVector<DIM>
    算法: φ(x) = F^{-1}{e^{iAx/ℓ}} (逆DFT, A=相位矩阵, ℓ=长度尺度)
2. [x] encode_scalar_log(value, base) — 对数缩放编码
3. [x] encode_scalar_circular(value, period) — 循环值编码 (角度/时间)
4. [x] SSP = encode_spatial(x, y, z) — 3D 空间语义指针
5. [x] decode_fpe(vec) → f64 — MLE 解码 (arXiv 2412.00488)
6. [x] 测试: scalar 相似度随数值差异单调递减, 循环编码周期性
```

### P108 — CCIPCA 在线子空间学习 ✅

**缺口**: V3 — Online Subspace Learning
**参考**: holon-rs `subspace.rs`; Weng et al. 2003 "Candid Covariance-free IPCA"; arXiv 1901.07922 精确增量 PCA
**命令**: `cargo check -p neotrix --lib`
**文件**: 📄 `neotrix-core/src/core/nt_core_hcube/subspace.rs` (339 行, 9 测试)

任务:
```text
1. [x] OnlineSubspace 结构体: eigenvalues, eigenvectors, n_observations
2. [x] observe(vec) — CCIPCA 增量更新 (遗忘参数 l=2..4)
3. [x] residual(vec) → f64 — 距流形距离 = 异常分数
4. [x] anomalous_component(vec) → Vec<usize> — 逐字段归因
5. [x] 测试: 随机向量序列后流形稳定; 异常向量 residual 显著增大
```

### P111 — 多 VSA 模型统一抽象

**缺口**: V2 — 多模型支持 (FHRR, B-SBC, VTB)
**参考**: torchhd 8 模型架构; AVSAD ICLR 26 WS 自动 VSA 发现 (Lean 4 形式验证)
**命令**: `cargo check -p neotrix --lib`
**文件**: 📄 `neotrix-core/src/core/nt_core_hcube/models.rs`

任务:
```text
1. [ ] VsaModel trait: bind/bundle/permute/similarity/unbind/similarity_matrix
2. [ ] FHRVsa: 复数域绑定 (相位旋转) + FPE 原生支持
3. [ ] BsbcVsa: 稀疏块编码 (k=32/4096 激活)  
4. [ ] MAPVsa: 现有 bipolar 封装到 trait
5. [ ] HRRVsa: 现有 FFT circ-conv 封装到 trait
6. [ ] ModelRegistry: 按需选择模型 (类型参数或动态分发)
7. [ ] 测试: 每个模型 bind->unbind 近似恒等; 跨模型相似度矩阵
```

### P114 — Cleanup & Resonator Networks

**缺口**: V4 — VSA 噪声清理
**参考**: Bremer & Orchard 2024 (MLE+CLE); Frady et al. resonator network; torchhd cleanup
**命令**: `cargo check -p neotrix --lib`
**文件**: 📄 `neotrix-core/src/core/nt_core_hcube/cleanup.rs`

任务:
```text
1. [ ] CodebookCleanup: 词表 → codebook → noisy_vec → nearest_clean
2. [ ] FHRR MLE+CLE: 迭代优化解码 FPE 向量 (arXiv 2412.00488)
3. [ ] ResonatorNetwork: 分解 A⊗B⊗C 复合向量 (交替投影)
4. [ ] 测试: 3-hop 复合向量 cleanup 后保真度 > 0.9; 噪声鲁棒性
```

### P117 — 高级 VSA 原语

**缺口**: V5 — analogy/attend/resonance/blend/conditional_bind
**参考**: holon-rs `primitives.rs` (17 个高级原语)
**命令**: `cargo check -p neotrix --lib`
**文件**: ✏️ `neotrix-core/src/core/nt_core_hcube/primitives.rs`

任务:
```text
1. [ ] analogy(A, B, C) = bind(A, unbind(B, C)) — "A is to B as C is to ?"
2. [ ] attend(query, keys, values) — 注意力加权绑定  
3. [ ] resonance(A, B) — 迭代绑定→unbind 收敛
4. [ ] blend(A, B, alpha) — 软混合
5. [ ] conditional_bind(condition, A, B) — if cond then bind(A,B) else A
6. [ ] project(vec, subspace) — 向量到子空间投影
7. [ ] 测试: analogy 正确性; attention 权重分布; resonance 收敛性
```

### P120 — VSA 梯度压缩桥

**缺口**: V6 — Gradient Compression
**参考**: vsa-optim-rs `VSAGradientCompressor` (90% 压缩, 2.9x 加速)
**命令**: `cargo check -p neotrix --lib`
**文件**: 📄 `neotrix-core/src/core/nt_core_hcube/gradient.rs`

任务:
```text
1. [ ] VSAGradientCompressor: 梯度 → VSA 绑定+打包 → 压缩
2. [ ] DeterministicPhaseTrainer: Warmup/Full/Predict/Correct 相位
3. [ ] 测试: 压缩率 > 80%; 解压后余弦相似度 > 0.95
```

---

## Phase 120-150: 记忆系统重构

### P123 — RL 驱动的记忆巩固 ✅

**缺口**: M1 — 用 RL 策略替代 FIFO/LRU 逐出
**参考**: MEM1 ICLR 2026 (end-to-end RL, 3.5x 提升, 3.7x 节省); arXiv 2506.15841
**命令**: `cargo test -p neotrix --lib decent_mem`
**文件**: ✏️ `neotrix-core/src/core/nt_core_experience/decent_mem.rs` (922 行, 6 新测试)

任务:
```text
1. [x] SurpriseScorer: 惊讶度分数 = 加权组合(访问频率, 近期性, 内容新颖度)
2. [x] RL 逐出策略: Q-learning, 状态=离散特征(访问桶/年龄桶/惊讶桶/长度桶), 动作=keep/evict
3. [x] 训练信号: 检索成功正奖励 + 默认负奖励
4. [x] 加权惊讶巩固: 惊讶度 > 阈值 → 晋升到 X-pool
5. [x] 测试: RL 策略学习正确逐出目标; 现有 API 完全向后兼容
```

### P126 — 时间分层记忆 TiMem (M2)

**缺口**: M2 — 5 层 Temporal Memory Tree
**参考**: TiMem ACL 2026 Findings (arXiv 2601.02845; GitHub TiMEM-AI/TiMEM)
        5 层: Segment→Session→Daily→Weekly→Persona; 复杂度感知检索
**命令**: `cargo test -p neotrix --lib timem`
**文件**: 📄 `neotrix-core/src/core/nt_core_experience/timem.rs`

任务:
```text
1. [ ] TMT 节点结构: level(0-4), content, timestamp, parent, children
2. [ ] insert(conversation) → TMT.update() 自动路由到正确层级
3. [ ] consolidate(): 低层 → 高层 VSA 摘要 (语义引导, 无需微调)
4. [ ] recall(query, complexity): 低复杂度→低层精细事实; 高复杂度→高层画像
5. [ ] 测试: 层级插入一致性; 复杂度感知检索准确率; 跨会话头像稳定性
```

### P129 — 记忆 CRUD 协议 ✅

**缺口**: M3 — Add/Update/Delete/Noop 操作
**参考**: Mem0 (arXiv 2504.19413, 50k+ 开发者); VSA 冲突检测代替 LLM 判断
**命令**: `cargo test -p neotrix --lib memory_ops`
**文件**: 📄 `neotrix-core/src/core/nt_core_experience/memory_ops.rs` (546 行, 15 测试)

任务:
```text
1. [x] MemoryOperation enum: Add, Update {old, new}, Delete, Noop
2. [x] MemoryStore trait + VsaMemoryStore 实现
3. [x] VSA 冲突检测: similarity(new, old) < threshold → 标记冲突
4. [x] VersionedFact: 每个事实带版本号 + 时间戳 + 源
5. [x] 测试: 矛盾检测; 更新后旧版本可回溯; 并发无冲突
```

### P132 — CRDT 一致性 (M4)

**缺口**: M4 — VectorClock + CmRDT 向量合并
**参考**: ContextFS (arXiv 2603.10062); CRDT 论文 (Shapiro 2011)
**命令**: `cargo test -p neotrix --lib crdt`
**文件**: 📄 `neotrix-core/src/core/nt_core_experience/crdt.rs`

任务:
```text
1. [ ] VectorClock: {agent_id → counter} + merge 函数
2. [ ] VsaCmRDT: 基于操作的 CRDT (bind 作为可交换合并)
3. [ ] ConflictResolution: last-writer-wins + VSA 相似度降级
4. [ ] 测试: 向量时钟因果序; 并发合并收敛; 无限并发无分歧
```

### P135 — 睡眠巩固周期 (M5)

**缺口**: M5 — 异步后台巩固
**参考**: Active Dreaming Memory (觉醒 store → 睡眠 replay); Letta 睡眠时间代理
**命令**: `cargo test -p neotrix --lib sleep`
**文件**: 📄 `neotrix-core/src/core/nt_core_consciousness/sleep.rs`

任务:
```text
1. [ ] SleepCycle 枚举: Awake, NREM, REM, Consolidated
2. [ ] ConsolidationWorker: GWT 空闲时隙触发后台处理
3. [ ] 惊讶子集采样: 只回放高惊讶度轨迹 (10:1 压缩)
4. [ ] 反事实模拟: "如果当时采取了不同行动?" JEPA 预测
5. [ ] 测试: 巩固后记忆总量减少; 关键事实保留; 反事实多样性
```

### P138 — 置信度校准 (M6)

**缺口**: M6 — 每检索 MetaAccuracy
**参考**: Mem0 置信度; 原则#9 MetaAccuracy KPI
**命令**: `cargo test -p neotrix --lib calibration`
**文件**: 📄 `neotrix-core/src/core/nt_core_experience/calibration.rs`

任务:
```text
1. [ ] ConfidenceScorer: VSA 空间线性探测器 → [0,1] 置信度
2. [ ] 训练: 检索→下游任务成功与否 → 校准损失
3. [ ] MetaAccuracy 反馈: |predicted_confidence - actual_accuracy|
4. [ ] 测试: ECE (Expected Calibration Error) < 0.1; 低置信度检索可被过滤
```

### P141 — 过程记忆层 (M7)

**缺口**: M7 — 轨迹→可重用过程模式
**参考**: AgentRR (arXiv 2505.17716) 经验重放 2-3x; CoEvolutionBridge
**命令**: `cargo test -p neotrix --lib procedural`
**文件**: 📄 `neotrix-core/src/core/nt_core_experience/procedural.rs`

任务:
```text
1. [ ] ProceduralMemory: pattern_id, trigger_condition, action_sequence, outcome
2. [ ] extract_pattern(trajectory): 识别成功模式 → VSA 绑定编码
3. [ ] match_pattern(context): 当前状态 → 匹配过程模式 → 推荐行动
4. [ ] integrate(procedural_memory, X-pool): 通过 VSA 绑定关联交换条目
5. [ ] 测试: 模式提取精度; 匹配召回率; 重用后成功率提升
```

### P144 — 统一记忆总线

**缺口**: I3 (部分) — 所有记忆子系统统一 VSA 协议
**参考**: 原则#2 (统一 VSA); DecentMem + KB + Procedural + Emotional + CognitiveMemory
**命令**: `cargo check -p neotrix --lib`
**文件**: ✏️ 跨文件重构

任务:
```text
1. [ ] MemoryBus trait: store/recall/update/delete/consolidate
2. [ ] 所有记忆系统实现 MemoryBus
3. [ ] 统一路由: query → 自动选择最佳记忆子系统
4. [ ] 测试: 所有记忆子系统通过统一接口; 路由选择性
```

---

## Phase 150-180: 自进化管道升级

### P150 — 进化因果追踪 ✅

**缺口**: E1 — 自我修改历史分析 → 预测
**参考**: OpenSpace 级联进化 + GDPVal 基准; DGM 存档树; Ouroboros Git 谱系
**命令**: `cargo test -p neotrix --lib evolution_trace`
**文件**: 📄 `neotrix-core/src/core/nt_core_self/evolution_trace.rs` (675 行, 15 测试)

任务:
```text
1. [x] EvolutionEvent: time, action, context, outcome, causal_parents
2. [x] EvolutionTrace: 时间序列 → TemporalAttention 编码
3. [x] CausalGraph: 从序列学习因果关系 (Granger + 注意力权重)
4. [x] predict_modification_outcome(proposal) → success_probability
5. [x] 测试: 因果图召回已知因果关系; 预测准确率 > 基准
```

### P153 — 好奇心驱动探索 (E2)

**缺口**: E2 — 知识缺口 → 好奇心信号 → 主动探索
**参考**: Evolver 知识差距检测; DGM 开放式存档; 原则#7 (内在驱动)
**命令**: `cargo test -p neotrix --lib curiosity`
**文件**: 📄 `neotrix-core/src/core/nt_core_experience/curiosity.rs`

任务:
```text
1. [ ] KnowledgeGapDetector: 在 HyperCube 中识别低密度区域
2. [ ] CuriositySignal: prediction_error + knowledge_gap → curiosity_score
3. [ ] ExplorationTrigger: curiosity > threshold → 发起探索行动
4. [ ] ExplorationAction: 自动搜索/爬取/实验
5. [ ] 测试: 好奇心分数与知识增长正相关; 探索行动多样性
```

### P156 — 叠加验证管线 ✅

**缺口**: E3 — 六层一体化验证 (语法→类型→安全→自洽→回归→基准)
**参考**: MARIA OS δM + Lyapunov; DGM 基准验证; OpenSpace 级联降级
**命令**: `cargo test -p neotrix --lib stacked_validation`
**文件**: 📄 `neotrix-core/src/core/nt_core_experience/stacked_validation.rs` (655 行, 14 测试)

任务:
```text
1. [x] ValidationLayer enum: Syntax, TypeSafety, SelfConsistency, Regression, Benchmark, Meta
2. [x] StackedValidationPipeline: 逐层执行, fail-fast 短路
3. [x] Layer 1-3: Gödel 检查器复用 (3 层)
4. [x] Layer 4 Regression: 后向兼容分数阈值检查
5. [x] Layer 5 Benchmark: 基准分数阈值检查
6. [x] Layer 6 Meta: StatisticalSafetyGate + meta-accuracy
7. [x] 测试: 14 测试, 各层独立可测, fail-fast 短路, 全通过场景
```

### P159 — 优雅降级集成 (E4)

**缺口**: E4 — 原则 8 集成到 SEAL
**参考**: 原则#8 (优雅降级); DGM 崩溃处理; Ouroboros 僵尸进程预防
**命令**: `cargo check -p neotrix --lib`
**文件**: 📄 `neotrix-core/src/core/nt_core_experience/graceful.rs`

任务:
```text
1. [ ] DegradationLevel: Full, Reduced, Minimal, Safe
2. [ ] SubsystemHealth: 每个子系统的心跳 + 延迟 + 错误率
3. [ ] DegradationTable: 子系统失效 → 能力范围缩小 → 保持连贯
4. [ ] 测试: JEPA 模拟故障 → 意识降级到纯 HyperCube; 对话无中断
```

### P162 — DGM-H 形式化 (E5)

**缺口**: E5 — δM 算子 + 修改边界 + 收敛保证
**参考**: MARIA OS δM + Lyapunov; MUE-X AST 变异策略
**命令**: `cargo test -p neotrix --lib dgmh_formal`
**文件**: 📄 `neotrix-core/src/core/nt_core_experience/dgmh_formal.rs`

任务:
```text
1. [ ] DeltaM 算子: M(t+1) = M(t) + δM (形式化修改表示)
2. [ ] ModificationFrontier: 元层可修改范围 (不可修改: 安全门, 身份, 收敛性)
3. [ ] LyapunovFunction: 单调递减能量函数 → 收敛保证
4. [ ] ResponsibilityGate: 路由器 × 门 → 联合约束优化
5. [ ] 测试: 所有修改在 Lyapunov 函数下收敛; 边界不可突破
```

### P165 — 主动推理回路 (E6) — 已由 P0-R4 覆盖 ✅

**缺口**: E6 — Active Inference 策略选择
**参考**: pymdp 1.0 JAX (arXiv 2603.13110); Friston Free Energy Principle
**文件**: 📄 `neotrix-core/src/core/nt_core_consciousness/active_inference.rs` (已实现)
**说明**: 由 P0-R4 在 nt_core_consciousness 中实现，无需重复构建。如需集成到好奇心回路，见 P153 接线。
**状态**: `cargo test -p neotrix --lib active_inference — 26 测试`

### P168 — 进化基准套件 (I2)

**缺口**: I2 — 标准化进化基准
**参考**: DGM SWE-bench + Polyglot; OpenSpace GDPVal
**命令**: `cargo test -p neotrix --lib evolution_bench`
**文件**: 📄 `tests/evolution_bench.rs`

任务:
```text
1. [ ] EvolBench trait: run(), score(), compare()
2. [ ] SelfEvolutionBench: 自我修改后编译通过率 + 测试通过率
3. [ ] KnowledgeGrowthBench: HyperCube 密度增长
4. [ ] ReasoningDepthBench: 多跳推理链长度增长
5. [ ] GDPVal 移植: 经济价值度量 (OpenSpace)
6. [ ] 测试: 基准套件可重复运行; 分数单调递增应是改进信号
```

### P171 — 轨迹压缩 (I1)

**缺口**: I1 — 进化日志压缩
**参考**: HarnessX trajectory compression; MEM1 恒定上下文
**命令**: `cargo test -p neotrix --lib trace_compress`
**文件**: 📄 `neotrix-core/src/core/nt_core_experience/trace_compress.rs`

任务:
```text
1. [ ] TrajectoryCompressor: 原始 JSON → VSA 摘要 → 压缩
2. [ ] 压缩率 > 80% 且关键事件可重建
3. [ ] 集成到 SelfEvolutionArchive
4. [ ] 测试: 压缩+解压后因果链完整性; 压缩率
```

---

## Phase 180-210: 认知架构融合

### P180 — 预测编码集成 (C1)

**缺口**: C1 — Prediction Error → GWT AttentionBoost
**参考**: PyHGF 层级高斯过滤; xagent 7 阶段预测处理; Cortex-rs GWT 集成
**命令**: `cargo test -p neotrix --lib predictive_coding`
**文件**: 📄 `neotrix-core/src/core/nt_core_prediction/predictive_coding.rs`

任务:
```text
1. [ ] VsaPredictionError: 精度加权预测误差 (计算置信度缩放)
2. [ ] AttentionBoost: PE > threshold → GWT 优先广播
3. [ ] BeliefUpdate: E8 态作为"先验-似然"算子
4. [ ] 测试: 高 PE 捕获注意力; 信念随观测逐步更新
```

### P183 — 层级高斯过滤 (C2)

**缺口**: C2 — HyperCube 上 6 层信念传播
**参考**: PyHGF gHGF 可微树; Nengo 层级皮层; Friston 2019
**命令**: `cargo test -p neotrix --lib hierarchical_filter`
**文件**: 📄 `neotrix-core/src/core/nt_core_prediction/hierarchical.rs`

任务:
```text
1. [ ] VSAHierarchicalFilter: 6 层 (感知层 0→抽象层5)
2. [ ] PrecisionWeightedNode: 自上而下预测 + 自下而上误差
3. [ ] VSAErrorUnit: 精度加权 PE 在 VSA 子空间传播
4. [ ] 测试: 层级一致; 顶层先验约束底层感知; 底层感知更新顶层信念
```

### P186 — 脉冲神经元接口 (C3)

**缺口**: C3 — SNN Bridge
**参考**: xagent WGSL GPU 内核; Nengo LIF 神经元
**命令**: `cargo check -p neotrix --lib --features gpu`
**文件**: 📄 `neotrix-core/src/neotrix/nt_world_sense/snn.rs`

任务:
```text
1. [ ] NeuralInterface trait: VsaState → SpikePattern → MotorCommand
2. [ ] LIFNeuron: WGSL 计算着色器 (可选 CUDA)
3. [ ] STDP: 脉冲时间可塑性 → VSA 绑定权重
4. [ ] 测试: 脉冲编码/解码保真度; STDP 学习模式
```

### P189 — 可微元学习 (C4)

**缺口**: C4 — 梯度优化核心超参数
**参考**: pymdp 1.0 JAX (完全可微); Candle (Rust torch)
**命令**: `cargo test -p neotrix --lib meta_learning`
**文件**: 📄 `neotrix-core/src/core/nt_core_meta/meta_learning.rs`

任务:
```text
1. [ ] MetaParam enum: BROADCAST_THRESHOLD, DECAY_RATE, E8_TRANSITION_PROBS, BINDING_STRENGTH
2. [ ] MetaLearningOptimizer: 元图 + 自动微分 (Candle/JAX)
3. [ ] MetaAccuracy loss: |predicted - actual| → 梯度
4. [ ] 安全约束: 梯度不能禁用安全门控
5. [ ] 测试: 元梯度有效; 安全约束不可突破; MetaAccuracy 单调递减
```

### P192 — 9D 编辑分类 (C5)

**缺口**: C5 — EditSurface 枚举
**参考**: HarnessX 9 维编辑表面; MARIA OS 修改边界
**命令**: `cargo check -p neotrix --lib`
**文件**: 📄 `neotrix-core/src/core/nt_core_self_modify/edit_surface.rs`

任务:
```text
1. [ ] EditSurface enum: Local/Global, Safe/Dangerous, Reversible/Irreversible, ...
2. [ ] classify(edit) → EditSurface
3. [ ] safety_score(surface) → 安全等级 (阻止高危险不可逆)
4. [ ] 测试: 分类准确率; 安全等级保护
```

---

## Phase 210-240: 工程基础设施

### P210-p222 — 工程清理

**参考**: C4 (unwrap), Cargo.toml 冲突, CI, 沙箱, 覆盖率, 插件
**命令**: `cargo check -p neotrix --lib && cargo clippy -p neotrix --lib`

任务:
```text
1. [ ] P210 - DecentMem CRDT 多 agent 部署
2. [ ] P213 - unwrap() 清理 1200→200 (80% 覆盖率)
3. [ ] P216 - reqwest 0.11 vs 0.12/0.13 冲突解决
4. [ ] P219 - GitHub CI + Release 管道
5. [ ] P222 - wasmtime OS 沙箱集成
6. [ ] P225 - 代码覆盖率 + clippy 零警告
7. [ ] P228 - 插件系统实现
8. [ ] P231 - 第一版公开文档 + README 更新
```

---

## 依赖图

```
P105 FPE ──→ P108 CCIPCA ──→ P111 多模型 ──→ P114 Cleanup ──→ P117 高级原语
  │                                                              │
  └──→ P120 梯度压缩                                              │
                                                                  ↓
Phase 105-156 (P0 全部完成 ✅) ──→ Phase 123-144 (记忆重构 P1)
                                        │
                                        ↓
                              P123 RL巩固 ─→ P126 TiMem ─→ P129 CRUD
                                │                              │
                                ↓                              ↓
                             P132 CRDT ←── P135 睡眠 ──→ P138 校准
                                                              │
                                                              ↓
                                              P141 过程记忆 ←── P144 统一总线
                                                              │
                                                              ↓
                                              Phase 150-180 (自进化)
                                                              │
                       P150 因果追踪 ──→ P153 好奇心 ──→ P156 叠加验证
                         │                                      │
                         ↓                                      ↓
                    P159 优雅降级 ──→ P162 DGM-H ──→ P165 主动推理
                         │
                         ↓
           P168 基准套件 ←── P171 轨迹压缩
                              │
                              ↓
            Phase 180-210 (认知架构) ──→ Phase 210-240 (工程)
```

---

## 执行策略

1. **Phase 105-108 (FPE + CCIPCA) 立即启动**: 无依赖, 独立可并行
2. **P0 缺口优先**: V1, V3, M3, E1, E3, M1 — 对核心推理有直接提升
3. **每阶段验证**: `cargo check` 通过 + 新增模块测试 > 6 个
4. **并行路径**: VSA 进化 (P105-P120) 与 记忆重构 (P123-P144) 无共享依赖, 可并行
5. **基准前置**: 每个模块实施前定义测试标准 (避免"不知道改了多少")

---

## Phase 241-400: CLX 自审查 31 缺口 (2026-06-22)

> 来源: SELF_AUDIT_CLX.md — 11 GitHub 项目深度分析 + 6 维度自我审查
> 识别: 31 缺口 (12 P0 + 11 P1 + 8 P2)
> 路径: 6 路径并行 (认知/VSA/自进化/记忆/接口/去中心化)

### Phase 241-280: P0 核心 (6 路并行)

| Phase | 缺口 | 模块 | 文件 | 预估行数 |
|-------|------|------|------|---------|
| P241 | G01 注意力图式 | 认知 | `nt_core_consciousness/attention_schema.rs` | ~800 |
| P244 | G02 认知灵活性 | 认知 | `nt_core_consciousness/cognitive_flexibility.rs` | ~600 |
| P247 | G03 多 VSA 模型 | VSA | `nt_core_hcube/vsa_models.rs` | ~1200 |
| P250 | G04 GPU 加速 VSA | VSA | `nt_core_hcube/gpu_vsa.rs` | ~900 |
| P253 | G05 可验证 RSI | 自进化 | `nt_core_self/verified_rsi.rs` | ~1000 |
| P256 | G06 SleepGate 睡眠 | 记忆 | `nt_core_experience/sleep_gate.rs` | ~800 |
| P259 | G07 统一搜索 API | 接口 | `nt_core_perception/unified_search.rs` | ~700 |
| P262 | G08 Document 管道 | 接口 | `nt_core_perception/document_pipeline.rs` | ~600 |
| P265 | G09 AI 浏览器导航 | 接口 | `nt_core_perception/ai_browser_agent.rs` | ~1200 |
| P268 | G10 BM25 内容过滤 | 优化 | `nt_core_crawler/content_filter.rs` | ~500 |
| P271 | G11 去中心化 AGI | 基础设施 | `nt_infra/agi_bridge.rs` | ~2000 |
| P274 | G12 自适应元素追踪 | 爬虫 | `nt_core_crawler/adaptive_tracker.rs` | ~700 |
| P277-280 | 集成测试 + 编译清零 | — | — | — |

### Phase 281-340: P1 深度进化

| Phase | 缺口 | 类型 | 预估行数 |
|-------|------|------|---------|
| P281-290 | G13-G15, G23 认知模块 | 认知 | ~3,200 |
| P291-300 | G16-G18 VSA 深度 | VSA | ~1,900 |
| P301-310 | G19-G22 自进化+记忆 | 自进化/记忆 | ~2,800 |
| P311-320 | 并行加速+集成 | — | — |
| P321-340 | 全模块调优+编译清零 | — | — |

### Phase 341-400: P2 基础设施

| Phase | 缺口 | 类型 | 预估行数 |
|-------|------|------|---------|
| P341-360 | G26-G28, G31 去中心化 | 基础设施 | ~8,000 |
| P361-380 | G29-G30 接口管道 | 接口 | ~1,200 |
| P381-400 | 基准+校准+清零 | 基础设施 | ~2,000 |

### Wave 1 任务 (立即启动, 8 P0 并行)

- **W1-01**: AttentionSchema — 注意力焦点自表征 (`attention_schema.rs`)
- **W1-02**: CognitiveFlexibility — 任务切换+认知控制 (`cognitive_flexibility.rs`)
- **W1-03**: MultiVsaModels — 8 模型统一 API (`vsa_models.rs`)
- **W1-04**: GpuVsa — Metal/CUDA 加速 (`gpu_vsa.rs`)
- **W1-05**: VerifiedRSI — 可验证递归自我改进 (`verified_rsi.rs`)
- **W1-06**: SleepGate — NREM/REM 睡眠微循环 (`sleep_gate.rs`)
- **W1-07**: UnifiedSearch — Search+Scrape 统一 API (`unified_search.rs`)
- **W1-08**: DocumentPipeline — PDF/Office→Markdown (`document_pipeline.rs`)

### 依赖图扩展

```
Phase 241-280 (P0 CLX) ──→ Phase 281-340 (P1 CLX) ──→ Phase 341-400 (P2 CLX)
    6 路并行:                             │
    认知 + VSA + 自进化                    ↓
    + 记忆 + 接口 + 去中心化           P1 深度进化
    │                                    |
    ↓                                    ↓
  P0 编译清零                        P2 基础设施
                                      + 进化基准
```
