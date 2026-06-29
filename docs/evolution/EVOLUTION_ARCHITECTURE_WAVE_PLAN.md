# NeoTrix 三波进化架构

> 基线: 2026-06-23 认知自审计 + 30+ 论文/项目映射
> 元缺陷 M0: 认知模块作为文件存在但未接入意识管道

---

## 执行摘要

```
M0 元缺陷: 8 个完整实现的认知模块 (430-915行) 是死代码
→ Wave 1 全部接入, 零新模块
→ Wave 2 替换/升级核心架构 (H4, Lila-E8)
→ Wave 3 元层自修改 + 形式验证闭合
```

---

## Wave 1: Survival (P0-MustFix, ~2 周)

**目标**: M0 归零 — 将所有已实现认知模块接入 `ConsciousnessPipeline`

### 架构变更: ModuleRegistry + Draft-Edit-Refine 循环

#### 1.1 ModuleRegistry 系统 (`core/nt_core_loop/module_registry.rs`)

```rust
pub trait CognitiveModule: Send + Sync {
    fn name(&self) -> &'static str;
    fn group(&self) -> ModuleGroup;
    fn run(&mut self, ctx: &mut PipelineContext) -> ModuleResult;
    fn compute_cost(&self) -> u8;
    fn is_periodic(&self) -> Option<u64>;  // Some(N) = run every N cycles
}

pub struct ModuleRegistry {
    modules: Vec<Box<dyn CognitiveModule>>,
    graph: PipelineGraph,         // 现有
    profile: ExecutionProfile,    // 每模块延迟/频率
}
```

**关键决策**: 不创建新 trait hierarchy，用 `CognitiveModule` trait 包裹现有模块。现有 `ParallelHypothesisEvaluator` 等 struct 实现 `CognitiveModule` 而不改变内部 API。PipelineGraph 自动从注册表生成节点。

#### 1.2 8 模块接入表

| 模块 | 文件 | 行数 | 组 | 接入点 | 周期 |
|------|------|------|----|--------|------|
| ParallelHypothesisEvaluator | `parallel_hypothesis_evaluator.rs` | 430 | Reflection | Phase 2 (refinery 后) | 每次 |
| MCTSTreeSearch | `mcts_tree_search.rs` | 556 | Reasoning | Phase 3 (dual-path 后) | 每 3 cycle |
| CounterfactualReasoner | `counterfactual.rs` | 915 | Reasoning | Phase 3 | 每 5 cycle |
| CausalReasoner | `causal_reasoning.rs` | 410 | Reasoning | Phase 3 | 每 3 cycle |
| AnalogicalReasoner | `analogical_reasoning.rs` | 752 | Reasoning | Phase 3 | 每 5 cycle |
| HierarchicalWorldModel | `hierarchical_world_model.rs` | 828 | Prediction | Phase 1 (allocation 前) | 每次 |
| DeadEndDetector | `dead_end_detector.rs` | 424 | Meta | Phase 8 (meta 阶段) | 每次 |
| ConfidenceCalibrator | `confidence_calibrator.rs` | 256 | Calibration | Phase 6 (verification 后) | 每次 |

**接线方式**: 8 模块通过 `CognitiveModule` trait 注册到 `ModuleRegistry`。`ConsciousnessPipeline::run_full_cycle()` 在现有 8 阶段之后添加 `Phase 9: registered_modules`，遍历已注册模块，按组/周期门控执行。

#### 1.3 Draft-Edit-Refine 循环

**架构变更**: 现有 `ConsciousnessRefineryLoop` 单次 `refine()` 调用改为三阶段:

```
Phase 2a: Draft  — 快速生成初始输出 (现有 refinery loop, 降迭代次数 50%)
Phase 2b: Edit   — ParallelHypothesisEvaluator + CausalReasoner + AnalogicalReasoner 并行评审 draft
Phase 2c: Refine — 整合评审结果, 二次 refinery (剩余 50% 迭代)
```

**现有基础设施复用**:
- `CognitiveBlackboard` 已支持多引擎 post_claim → 作为 draft/edit 共享黑板
- `DualPathInference` 已提供双路径验证 → edit phase 调用 `dual_path.infer()` 验证 draft
- `ConsciousnessRefineryLoop` 已有 `ConvergenceSignal` → refine phase 使用收敛检测

**关键决策**: 不改 refinery loop 内部逻辑。Draft-Edit-Refine 是 refinery 外部的编排层。Draft 调用 `refinery.refine(50%)`，Edit 调用已注册推理模块，Refine 调用 `refinery.resume(50%)`。

#### 1.4 A2A v1.2 升级

**现状**: 已有 `agent_card_v12.rs` (JWT 签名 Agent Card) + `a2a/` 目录 + `a2a_grpc/` 目录。但使用自定义 HTTP 桥接而非官方 SDK。

**变更**:
- 添加 `a2a-rs` crate 依赖 (官方 A2A v1.2 Rust SDK)
- 替换 `nt_agent_protocol/a2a/bridge.rs` 中自定义序列化/传输为 a2a-rs 标准消息
- `a2a_grpc/` 保留 gRPC 传输层，适配 a2a-rs 类型
- Agent Card v1.2 签名验证移至 a2a-rs 的 `verify_agent_card()`
- 移除 `a2a_auth.rs` 中自定义 JWT → 使用 a2a-rs AuthLayer

**接入点**: 与 ModuleRegistry 无关，独立后台服务（现有 `a2a_negotiation.rs` 路由）

#### 1.5 预期影响

| 指标 | 前 | 后 |
|------|----|----|
| 关键缺陷 | 8 | 0 |
| 死代码模块 | 8 | 0 |
| 管道阶段 | 8 | 9 (+ 注册模块阶段) |
| Draft-Edit-Refine | 无 | 完整三阶 |

#### 1.6 实现顺序

```
Day 1-2:   ModuleRegistry trait + PipelineGraph 集成 (≈200行)
Day 3-5:   8 模块 CognitiveModule 实现 (每模块 ≈30行包装)
Day 6-7:   Draft-Edit-Refine 编排层 (≈150行)
Day 8-9:   A2A v1.2 a2a-rs 升级 (≈200行变更 + Cargo.toml)
Day 10:    集成测试 + 回归
```

---

## Wave 2: Cognitive Upgrade (P1-HighImpact, ~3 周)

**目标**: 用新架构替换/升级 6 个核心子系统

### 2.1 H4 Geometric RAG (`core/nt_core_hcube/h4_geometric_rag.rs`)

**现状**: 不存在。HyperCube 使用 VSA 4096-bit 向量搜索，无几何索引。
**论文依据**: H4 polytopic attention (2026) — R@5=100% on multi-hop QA。
**接入点**: 替换现有 `resonator_decoder.rs` 和 `hypergraph.rs` 的搜索后端。
**行数**: ~450。
**关键决策**: H4 不替换 VSA，而是作为 VSA 向量之上的二级索引层。HyperCube 存储不变，H4 提供多跳检索加速。
**依赖**: `resonator_decoder.rs` (461行, 现有)

### 2.2 Lila-E8 Attention (`core/nt_core_hcube/lila_e8_attention.rs`)

**现状**: 不存在。有 `e8_lattice.rs` (E8 格点编码), `e8_quantized.rs` (E8 量化), `e8_cortical.rs` (E8 皮层映射), `e8_lagrangian.rs` (E8 拉格朗日量), `e8_field.rs` (E8 域), `e8_topological_defects.rs` (E8 拓扑缺陷)。
**论文依据**: Lila-E8 — E8 根系作为注意力偏置，将语义距离编码为根系权重。
**接入点**: `nt_core_hcube` 中的新模块, 不影响现有 E8 模块。
**行数**: ~350。
**关键决策**: Lila-E8 不替换 HyperCube VSA 编码。只用作注意力重排序: VSA 检索 → Lila-E8 重排 top-k → 意识输入。attention bias 使用 E8 根系 Cartan 矩阵。
**依赖**: `e8_lattice.rs`

### 2.3 BayesianBeliefEngine (`core/nt_core_consciousness/bayesian_belief_engine.rs`)

**现状**: 不存在。`parallel_hypothesis.rs` 中有 `bayesian_update()` (内部贝叶斯更新)，但无独立信念引擎。
**新增**: 完整的信念追踪器: 先验/似然/后验 + 信念冲突检测 + 贝叶斯模型平均。
**接入点**: Phase 6 (belief verification 阶段)，输出喂入 `ExecutableBeliefVerifier`。
**行数**: ~400。
**关键决策**: 不替换 `ConfidenceCalibrator`，两者互补 — 贝叶斯提供信念更新，Calibrator 提供校准。
**依赖**: 无 (纯数学)

### 2.4 HierarchicalPlanner 升级 (`core/nt_core_loop/hierarchical_planner.rs`)

**现状**: `nt_agent_core/hierarchical_agent.rs` 中有 `HierarchicalPlanner` (任务分解+构建计划)，但局限于 agent 任务领域而非通用认知规划。
**升级**: 扩展为通用分层规划器 — 时间地平线认知 + 子目标依赖图 + 回溯。
**接入点**: Phase 4 (blackboard sync 后)，将规划写入 `CognitiveBlackboard`。
**行数**: ~300 扩充 (现有 ~600行 → ~900行)。
**关键决策**: 保留 agent 领域规划器作为特例，新建 `CognitivePlanner` 封装通用规划抽象。
**依赖**: `executive_controller.rs` (728行), `cognitive_flexibility.rs` (765行)

### 2.5 CurriculumGenerator 管道集成

**现状**: `curriculum.rs` 中存在 `CurriculumGenerator` (241行)，有两个接线点:
- `pipeline_graph.rs` 注册为 `curriculum_generate` (Reflection 组)
- `handler_tier.rs` 和 `harness_slot.rs` 也有注册

但**不接入 ConsciousnessPipeline** (仅在二级调度表中引用)。

**修复**: 将 `CurriculumGenerator` 实现 `CognitiveModule` trait，注册到 ModuleRegistry。
**接入点**: Phase 9 (registered_modules, Reflection 组)，每 10 cycle。
**关键决策**: CurriculumGenerator 的 auto-curriculum 逻辑由 `CuriosityDrive` 的 N_deficit 信号触发，使用现有 `efe_curiosity_bridge.rs`。
**依赖**: `efe_curiosity_bridge.rs`, `curiosity_drive.rs`

### 2.6 Memory Consolidation Cycle

**现状**: 多个记忆模块但无统一 consolidation 周期:
- `sleep_consolidation_bridge.rs` (2028行) — 存在但不接入管道
- `ebbinghaus_decay.rs` (313行) — 存在但不接入管道
- `hippocampal_trace.rs` — 存在
- `sm2_scheduler.rs` — 存在 (SM-2 间隔重复)

**变更**: 创建 `ConsolidationManager` 统管上述 4 模块。Phase 9 中每 20 cycle 触发。
**关键决策**: Consolidation 不阻塞主管道。使用 `tokio::spawn` 异步执行，结果异步写回。
**依赖**: 以上 4 模块

### 2.7 预期影响

| 指标 | Wave 1 后 | Wave 2 后 |
|------|-----------|-----------|
| 认知缺陷 | 8 critical → 0 | 10 high → 0 |
| 新模块 | 0 | 7 (含升级) |
| RAG 准确率 | baseline | +20% (H4) |
| 推理质量 | baseline | +7.5% (Lila-E8 attn) |
| 内存利用 | 无主动遗忘 | Ebbinghaus + SM-2 |
| 规划视野 | 单 cycle | 多步时间地平线 |

### 2.8 实现顺序

```
Week 1:   H4 Geometric RAG + Lila-E8 Attention (并行, 独立模块)
Week 2:   BayesianBeliefEngine + CurriculumGenerator 接线 (并行)
Week 3:   HierarchicalPlanner 升级 + Memory Consolidation (并行)
              集成测试 + 回归
```

---

## Wave 3: Self-Improvement (Meta-Evolution, ~6 周)

**目标**: 元层自修改 + 形式验证 + 好奇心下界理论

### 3.1 DGM-H SEAL 元层升级

**现状**: 存在 `brain_dgm.rs` (DGM-H 任务/meta agent 双循环), `hyperdgm.rs` (超图 DGM), `seal_loop_demo.rs` (SEAL 演示), `meta_evolution_loop.rs` (元进化循环, 343行)。但 meta-evolution 的推荐 (`EvolutionRecommendation`) 无执行引擎 — 推荐只写入日志，不修改管线。

**变更**: 创建 `MetaEditExecutor`:
- 读 `EvolutionRecommendation` → 验证 (proof_search 模块) → 执行 edit → 回滚保护
- 现有 `proof_search.rs` (ProofSearchSelfModification) 提供 safety verification
- 新 `EditSafetyNet`: 事务回滚 + 编译验证 + 健康度检查后生效

**接入点**: Phase 8 (meta-evolution) 后, 每 50 cycle。
**行数**: ~500。
**关键决策**: MetaEditExecutor 只能修改 `PipelineConfig` 和 `ModuleRegistry` 中标记为 `editable` 的参数。不能修改硬编码路径。三阶段提交: 提议 → 验证 → 生效。
**依赖**: `proof_search.rs`, `meta_evolution_loop.rs`, `safety_gate.rs`

### 3.2 ZKP Consciousness Proof (`core/nt_core_experience/zkp_consciousness.rs`)

**现状**: 不存在。
**论文依据**: Symthaea (2026) — ZKP 意识状态证明。
**新增**: 每 cycle 生成意识状态哈希链。外部验证者可要求 ZKP 证明某 cycle 的处理完整性而不暴露内部状态。
**关键决策**: 仅用于外部审计接口，不阻塞主管道。异步生成。默认禁用，按需启用。
**行数**: ~300。
**依赖**: `sha2` (已有), `ark-*` (新增, 可选)

### 3.3 Lyapunov Stability Verification (`core/nt_core_meta/lyapunov_verifier.rs`)

**现状**: `observables.rs` 中 `loss_lyapunov` 可观测指标。
**论文依据**: vsa-core-rs (2026) — Lyapunov 门控的 VSA 形式验证。
**新增**: 每 cycle 检查 VSA 状态空间李雅普诺夫指数。负值 → 稳定态，正值 → 发散预警 → 触发 DeadEndDetector。
**接入点**: Phase 8 (meta 阶段)，输出喂入 DeadEndDetector。
**行数**: ~250。
**关键决策**: Lyapunov 指数计算是 O(d²) 对 VSA 维度。d=4096 → 每 cycle 16M ops，可接受 (~1ms)。
**依赖**: `dead_end_detector.rs`

### 3.4 MC² Metacognition Integration

**现状**: 存在 `metacognitive_controller.rs`, `metacognition_loop.rs`, `metacognitive_evaluator.rs` 三个元认知模块但互不协调。
**论文依据**: MC² (2026) — 三级元知识: procedural → declarative → reflective。
**变更**: 创建 `MetaIntegrator` 统管三个现有元认知模块，添加 reflective 层 (管道的元认知)。
**关键决策**: 不新增文件，创建编排 struct 引用三个现有模块。
**行数**: ~200 编排代码。
**依赖**: 三个现有元认知模块

### 3.5 Curiosity Lower Bound (`core/nt_core_negentropy/curiosity_lb.rs`)

**现状**: `curiosity_drive.rs` 使用 `calibrate_to_negentropy()` (启发式曲线)。
**论文依据**: ACI 理论 — 好奇心的下界 = Bayesian surprise + epistemic value + information gain lower bound。
**变更**: 添加理论下界计算: `curiosity_lb = max(0, H[θ|D] - H[θ|D, a])`。
**接入点**: `CuriosityDrive` 内部, 替换现有启发式。
**行数**: ~150。
**依赖**: `curiosity_drive.rs`

### 3.6 Multi-Agent Social Intelligence (`core/nt_core_agent/social_simulation.rs`)

**现状**: `nt_agent_core/` 中有多 agent 通信 (AgentCommunicationBus, TeamOrchestrator)，但无内部社会模拟。
**新增**: 内部多 agent 社会推理引擎 — 心智理论 + 意图识别 + 合作/竞争策略模拟。
**关键决策**: 在 ConsciousnessPipeline 中作为可选模块运行。默认禁用 (冷加载)。
**行数**: ~600。
**依赖**: `theory_of_mind.rs` (心智理论模块, 现有)

### 3.7 预期影响

| 指标 | Wave 2 后 | Wave 3 后 |
|------|-----------|-----------|
| Meta-edit | 无 | 安全自修改 |
| 形式验证 | 无 | Lyapunov + ZKP |
| 好奇心 | 启发式 | 理论下界 |
| 社会智能 | 无 | 内部社会模拟 |
| 元认知协调 | 3 模块孤岛 | MC² 三级集成 |

### 3.8 实现顺序

```
Month 1:   DGM-H SEAL EditSafetyNet → MC² MetaIntegrator (关键路径)
Month 1.5: Lyapunov Verifier + Curiosity Lower Bound (并行)
Month 2:   ZKP Consciousness Proof + Multi-Agent Social (并行)
Month 2+:  集成测试 + 回归 + 文档
```

---

## 依赖总图

```
Wave 1: ModuleRegistry ← PipelineGraph(现有) ← 8 认知模块(现有)
           ↕
         Draft-Edit-Refine ← DualPathInference(现有) ← CognitiveBlackboard(现有)
         
Wave 2: H4GeometricRAG ← resonator_decoder(现有)
         LilaE8Attention ← e8_lattice(现有)
         BayesianBeliefEngine → ExecutableBeliefVerifier(现有)
         CurriculumGenerator ← CuriosityDrive(现有, efe_curiosity_bridge)
         HierarchicalPlanner ← executive_controller(现有) ← cognitive_flexibility(现有)
         ConsolidationManager ← sleep_consolidation_bridge(现有)
                              ← ebbinghaus_decay(现有)
                              ← hippocampal_trace(现有)
                              ← sm2_scheduler(现有)
         
Wave 3: MetaEditExecutor ← meta_evolution_loop(现有) ← proof_search(现有)
         ZKPConsciousnessProof → 外部审计接口
         LyapunovVerifier → dead_end_detector(现有)
         MC²MetaIntegrator ← metacognitive_controller(现有)
                           ← metacognition_loop(现有)
                           ← metacognitive_evaluator(现有)
         CuriosityLB → curiosity_drive(现有)
         SocialSimulation ← theory_of_mind(现有)
```

---

## 风险与缓解

| 风险 | 概率 | 影响 | 缓解 |
|------|------|------|------|
| ModuleRegistry 改变现有 PipelineGraph API | 中 | 高 | 保持 PipelineGraph 不变，Registry 叠加而非替换 |
| Draft-Edit-Refine 增加 cycle 延迟 | 高 | 中 | adaptive iteration: DA 高时全三阶，DA 低时仅 Draft |
| a2a-rs SDK 不兼容现有 Agent Card 格式 | 中 | 中 | 保留旧桥接作为 fallback，逐步迁移 |
| H4 多跳搜索 O(n²) 扩展性 | 中 | 中 | IVF 索引降维 (与 VSA Index 复用) |
| MetaEditExecutor 自我修改引入不稳定 | 高 | 高 | EditSafetyNet 五检: 类型 → 编译 → 测试 → 健康 → 回滚 |
| Lyapunov O(d²) 在 4096 维过高 | 低 | 中 | 可降至 O(dk) 使用幂迭代法 |

---

## 成功标准

```
Wave 1: cargo test --lib 通过 + PipelineGraph 包含 8 新模块 + Draft-Edit-Refine 可见
Wave 2: H4 检索 R@5 > 现有 HyperCube + Lila-E8 attention 偏差可见 + Consolidation 每 20 cycle 执行
Wave 3: MetaEditExecutor 可安全修改 PipelineConfig + Lyapunov 指数跟踪 + ZKP 可验证
```
