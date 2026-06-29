# V21 架构: 三循环自进化 (Three-Loop Self-Evolution)

> **Build**: 2026-06-23 | **Previous**: v20 (Qualia + Data Quality + AST Gate)
> **Status Audit**: 2026-06-24 — 10 papers/projects absorbed (Rounds 1-4), 3-loop + 4 external integrations planned
> **Design Principle**: "Evolve the optimizer that evolves the tasks"

---

## 0. Executive Summary

v21 升级核心: 从**双循环(小+大)**升级为**三循环(小+大+元)**。小循环管理每 tick 的 5 条反馈桥接。大循环管理每 10-30 tick 的 SEPL 进化管线。**元循环**引入 Escher-Loop 双种群共进化 — TaskPopulation (当前任务) 和 OptimizerPopulation (可自修改的优化器策略) 相互进化。

| 循环 | 频率 | 现有状态 | 新组件 |
|------|------|----------|--------|
| 小循环 (Small Loop) | 每 tick | ✅ SelfEvolutionMetaLayer::tick() | — |
| 大循环 (Big Loop) | 每 10-30 ticks | ✅ SelfEvolutionPipeline + EvolutionTaskSystem | — |
| **元循环 (Meta Loop)** | **每 50-200 ticks** | ❌ 不存在 | **EscherLoopEngine + SubAgentAccumulator + ArchiveManager** |

### 外部论文吸收

| 论文 | 缺口 | 映射 |
|------|------|------|
| **Escher-Loop** (arXiv 2604.23472) | 双种群共进化, 优化器可进化 | Meta Loop: TaskPopulation + OptimizerPopulation |
| **DGM-H** (arXiv 2505.22954) | 存档树 + 性能加权父选择 | ArchiveManager: tree + Pareto selection |
| **Gödel Agent** (ACL 2025) | 运行时自引用代码变异 | MutationOp::SelfModifyProposal (已存在) + 运行时接线 |
| **AgentFactory** (arXiv 2603.18000) | 3 阶段子 agent 生命周期 | SubAgentAccumulator: accumulate/retrieve/refine |
| **Autogenesis SEPL** (arXiv 2604.15034) | 算子代数 + 原子审计 | 5 SEPL 算子映射 (ρ,σ,ι,ε,κ) |
| **yoyo-evolve** (GitHub, 2k★) | Two-phase evolution: PlanningAgent→ImplementationAgent + RLM dispatch | Wave A2: PlanningAgent 阶段 + SharedState |
| **yoagent** (base library) | Event stream protocol + SubAgentTool delegation | Wave B2: 6-event 状态机 + 委托原语 |
| **clawREFORM** (GitHub) | Self-Rewrite: compile→test→clippy→commit/rollback | Wave B2: 三阶段验证环 + 自动 rollback |
| **HIVE** (GitHub, Rust) | Anti-spiral recovery + preference captures | Wave A2: DeadEndDetector 复活 + C2: 偏好学习 |
| **symbiont** (GitHub, Rust) | Hot-swappable compiled code + `evolvable!` macro | Wave C2 探索: 运行时 binary swap |

---

## 1. 三循环架构总览

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         SelfEvolutionMetaLayer (大脑)                        │
│                                                                             │
│  ┌────────────────── 小循环 (每 tick, <5ms) ──────────────────┐             │
│  │  回路1: Calibration(ECE) → MetaCognitiveLoop                │             │
│  │  回路2: LossFunction → SelfModifyAgent                      │              │
│  │  回路3: MetaPlan → SelfEvolution (MutationOp 生成)          │              │
│  │  回路4: GuardActivator(4层) + AstSafetyGate                 │              │
│  │  回路5: ConsciousnessCycle(12步) → 真实认知                  │              │
│  │  GEPA: trace_buffer → reflective_bonus → task execution     │              │
│  └──────────────────────────────────────────────────────────────┘             │
│                              │                                               │
│                              ▼                                               │
│  ┌────────────────── 大循环 (每 10-30 tick, <100ms) ───────────┐             │
│  │  SEPL 代数: ρ(Reflect)→σ(Select)→ι(Improve)→ε(Evaluate)→κ(Commit)│      │
│  │  SelfArchAudit → EvolutionTaskSystem → TaskEngine → Gate → Archive│      │
│  │  GEPA 4阶段: Read→Mutate→Evaluate→Select                   │              │
│  │  DGM-H: archive → select → sandbox → merge                  │              │
│  └──────────────────────────────────────────────────────────────┘             │
│                              │                                               │
│                              ▼                                               │
│  ┌────────────────── 元循环 (每 50-200 tick, <500ms) ───────────┐             │
│  │  EscherLoopEngine:                                            │              │
│  │    ├─ TaskPopulation<T>: 当前进化任务池                        │              │
│  │    ├─ OptimizerPopulation<O>: 可进化的优化器策略库              │              │
│  │    └─ co_evolve_step(): 双种群共进化迭代                       │              │
│  │  SubAgentAccumulator:                                          │              │
│  │    ├─ accumulate(): 执行迹 → 可复用子 agent                    │              │
│  │    ├─ retrieve(): VSA 相似度检索子 agent                       │              │
│  │    └─ refine(): 执行反馈 → 子 agent 改进                       │              │
│  │  ArchiveManager:                                               │              │
│  │    ├─ DGM-H 存档树 (非线性)                                    │              │
│  │    ├─ parent_selection(): 性能加权/子代倒数                      │              │
│  │    └─ pareto_frontier(): 多目标前缘修剪                         │              │
│  └──────────────────────────────────────────────────────────────┘             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. 小循环 — 每 tick (✅ 已实现)

### 2.1 组件状态

| 组件 | 文件 | LOC | 测试 | 接线状态 |
|------|------|:---:|:----:|:--------:|
| SelfEvolutionMetaLayer | `self_evolution_meta_layer.rs` | 1787 | 33+ | ✅ `core.rs` tick |
| FeedbackBridge (3 bridges) | internal | ~120 | 12 | ✅ |
| LoopCoordinator | internal | ~70 | 6 | ✅ |
| GuardActivator (4层) | internal | ~90 | 5 | ✅ |
| GEPA trace_buffer (50-cycle) | internal | ~80 | 4 | ✅ |
| RecoveryRecipeManager | `recovery_recipe.rs` | 283 | 8 | ✅ |

### 2.2 5 回路闭合状态 (每次审计验证)

| 回路 | 路径 | 验证方式 | 状态 |
|------|------|----------|:----:|
| 1 | Calibration(ECE) → MetaCognitiveLoop | grep "bridge_calibration_to_meta" | ✅ |
| 2 | LossFunction → SelfModifyAgent | grep "bridge_loss_to_self_modify" | ✅ |
| 3 | MetaPlan → SelfEvolution (真实 MutationOp) | grep "archive.add" | ✅ |
| 4 | GuardActivator(4层) + AstSafetyGate | grep "activate_guard_layers" | ✅ |
| 5 | ConsciousnessCycle 12步 → 真实认知 | grep "run_cycle" | ✅ |

### 2.3 关键代码路径

```rust
// SelfEvolutionMetaLayer::tick() (1787 行, 核心入口)
pub fn tick(&mut self, cycle: u64, meta_result: Option<&MetaCycleResult>,
            cal: Option<&CalibrationEngine>, loss: Option<&LossFunction>) {
    // 1. 迹收集 (GEPA Read)
    self.collect_trace_snapshot(cycle, cal, loss);
    // 2. 回路 1-3 (FeedbackBridge)
    self.run_feedback_bridges(meta_result, cal, loss);
    // 3. 任务执行 (GEPA Mutate + Select)
    self.run_gepa_cycle(cycle);
    // 4. 日志 + 统计
    self.intervention_log.push(/* ... */);
}
```

---

## 3. 大循环 — 每 10-30 tick (✅ 已实现)

### 3.1 组件状态

| 组件 | 文件 | LOC | 测试 | 接线状态 |
|------|------|:---:|:----:|:--------:|
| SelfEvolutionPipeline | `self_evolution_pipeline.rs` | 1122 | 12+ | ✅ |
| EvolutionTaskSystem | `evolution_task_system.rs` | 833 | 12+ | ✅ |
| SelfEvolutionTaskEngine | `self_evolution_task_engine.rs` | 406 | 5+ | ✅ |
| SelfArchAudit | `self_arch_audit.rs` | 161 | 6 | ✅ (36 模块预注册) |
| AstSafetyGate | `ast_safety_gate.rs` | 660 | 14 | ✅ (imported) |
| WeaknessMiner | `self_evolution_engine.rs` | ~120 | 8 | ✅ |
| SealProposalBridge | `seal_proposal_bridge.rs` | 792 | 24 | 🟡 (Phase C wired) |
| SelfEvolutionLoop | `self_evolution_loop/core.rs` | 2622 | ~40 | ✅ |

### 3.2 SEPL 算子映射

| SEPL 算子 | NeoTrix 组件 | 输入→输出 | LOC | 测试 | 状态 |
|-----------|-------------|-----------|:---:|:----:|:----:|
| **ρ (Reflect)** | SelfArchAudit::audit() + WeaknessMiner | State→ArchReport | 161+120 | 6+8 | ✅ |
| **σ (Select)** | EvolutionTaskSystem::auto_discover + next_ready | Report→Task | 833 | 12 | ✅ |
| **ι (Improve)** | SelfEvolutionTaskEngine::process_system_task | Task→Proposal | 406 | 5 | ✅ |
| **ε (Evaluate)** | GuardActivator(4层) + AstSafetyGate + VerificationGate | Proposal→Result | ~200+660 | 5+14 | ✅ |
| **κ (Commit)** | SelfEvolutionStep archiving + EvolutionArchive | Result→Archive | ~100 | 5 | ✅ |

### 3.3 DGM-H 存档 (当前状态)

当前实现使用简单的线性 Vec 存档:

```rust
// ⚠️ 当前: 线性 Vec (v20 遗留)
pub struct EvolutionArchive {
    entries: Vec<ArchiveEntry>,    // 线性, 无树结构
    config: SelectionConfig,
}
// 选择策略: select_by_best() — 仅取最高分 (latest-wins)
```

**升级方向**: 存档树 + 前缘选择 (元循环, Wave B).

---

## 4. 元循环 — 每 50-200 tick (🆕 新建)

### 4.1 EscherLoopEngine — 双种群共进化

Escher-Loop (arXiv 2604.23472) 的核心创新: 两个种群相互进化 — TaskPopulation (要执行的任务) 和 OptimizerPopulation (如何执行任务的策略)。**优化器本身可以自我改进**, 这是 RSI 的关键。

#### 4.1.1 架构

```rust
/// 元循环引擎: 双种群共进化
pub struct EscherLoopEngine {
    /// 当前进化任务池 (待执行/执行中/已完成)
    task_pop: TaskPopulation<EvolutionTask>,
    /// 可进化的优化器策略库
    opt_pop: OptimizerPopulation<OptimizerStrategy>,
    /// 共进化迭代计数器
    epoch: u64,
    /// 历史统计
    stats: EscherLoopStats,
}

/// 任务种群: 包裹 EvolutionTaskSystem
pub struct TaskPopulation<T> {
    tasks: Vec<T>,
    fitness_history: Vec<f64>,
}

/// 优化器种群: 可自修改的策略
pub struct OptimizerPopulation<O> {
    strategies: Vec<O>,
    /// 每个策略的: {child_count, performance_history}
    lineage: HashMap<u64, LineageStats>,
}

/// 优化器策略: 决定"如何选择和执行任务"
pub struct OptimizerStrategy {
    pub id: u64,
    pub parent_id: Option<u64>,
    /// 选择策略偏好 (冒险/保守/均衡)
    pub selection_bias: f64,     // 0.0=保守, 1.0=冒险
    /// 变异幅度偏好
    pub mutation_scale: f64,     // 0.0=微调, 1.0=大幅
    /// 评估严格度偏好
    pub evaluation_strictness: f64, // 0.0=宽松, 1.0=严格
    /// 此策略产生的子代优化器数量
    pub child_count: u64,
    /// 滚动性能
    pub performance: Vec<f64>,
    pub generation: u32,
}
```

#### 4.1.2 共进化步骤

```rust
impl EscherLoopEngine {
    /// 一步共进化迭代
    pub fn co_evolve_step(&mut self, 
                          trace_buffer: &VecDeque<TraceSnapshot>,
                          task_system: &mut EvolutionTaskSystem) -> EscherLoopResult {
        self.epoch += 1;

        // Phase 1: 从当前策略池选择最优策略
        let parent_opt = self.sample_parent_strategy()?;  // DGM-H 风格选择
        let parent_task = self.task_pop.select_by_strategy(&parent_opt);

        // Phase 2: 执行任务 + 评估结果
        let result = self.execute_with_strategy(parent_task, parent_opt, task_system);

        // Phase 3: 更新种群适应度
        self.opt_pop.record_fitness(parent_opt.id, result.fitness);
        self.task_pop.record_fitness(parent_task.id, result.fitness);

        // Phase 4: 生成新策略变体 (Gödel Agent 自引用)
        if self.should_mutate_strategy(&result) {
            let child_opt = self.generate_variant_strategy(&parent_opt, &result);
            self.opt_pop.strategies.push(child_opt);
        }

        // Phase 5: Pareto 前缘修剪
        if self.opt_pop.strategies.len() > self.max_strategies {
            self.opt_pop.prune_to_pareto_frontier();
        }

        EscherLoopResult {
            epoch: self.epoch,
            parent_strategy: parent_opt.id,
            task_completed: parent_task.id,
            fitness_delta: result.fitness,
            new_strategies: self.opt_pop.strategies.len(),
        }
    }

    /// DGM-H 风格父选择: 性能加权 / 子代数量倒数
    fn sample_parent_strategy(&self) -> Option<&OptimizerStrategy> {
        let scores: Vec<f64> = self.opt_pop.strategies.iter().map(|s| {
            let perf = s.performance.last().copied().unwrap_or(0.5);
            let child_penalty = 1.0 / (1.0 + s.child_count as f64);  // 子代多→惩罚
            perf * child_penalty
        }).collect();
        let total: f64 = scores.iter().sum();
        if total <= 0.0 { return self.opt_pop.strategies.first(); }
        let mut pick = fastrand::f64() * total;
        for (i, s) in scores.iter().enumerate() {
            pick -= s;
            if pick <= 0.0 { return Some(&self.opt_pop.strategies[i]); }
        }
        self.opt_pop.strategies.last()
    }

    /// Gödel Agent 自引用变异: 优化器修改自身参数
    fn generate_variant_strategy(&self, parent: &OptimizerStrategy,
                                 result: &TaskResult) -> OptimizerStrategy {
        // 基于执行结果调整策略参数
        let mutation = if result.fitness > 0.3 {
            // 成功 → 在当前方向微调
            0.0  // conservative
        } else {
            // 失败 → 大幅探索
            0.3  // explore
        };
        OptimizerStrategy {
            id: fastrand::u64(..),
            parent_id: Some(parent.id),
            selection_bias: (parent.selection_bias + fastrand::f64() * mutation - mutation/2.0).clamp(0.0, 1.0),
            mutation_scale: (parent.mutation_scale + fastrand::f64() * mutation - mutation/2.0).clamp(0.0, 1.0),
            evaluation_strictness: (parent.evaluation_strictness + fastrand::f64() * mutation - mutation/2.0).clamp(0.0, 1.0),
            child_count: 0,
            performance: vec![result.fitness],
            generation: parent.generation + 1,
        }
    }
}
```

#### 4.1.3 与 EvolutionTaskSystem 的集成

```
EscherLoopEngine ←── reads ── EvolutionTaskSystem.stats()
    │                           (total/completed/pending tasks by type)
    │
    ├── co_evolve_step() ──→ 修改 task_system 的任务优先级
    │                           (基于当前最优策略的 selection_bias)
    │
    └── generate_variant() ──→ 注册新策略到 opt_pop
                                (EscherLoopEngine 持有策略, 不侵入 task_system)
```

### 4.2 SubAgentAccumulator — AgentFactory 子 agent 积累

AgentFactory (arXiv 2603.18000) 的三阶段生命周期: Install → Self-Evolve → Deploy. 成功执行迹自动结晶为可复用子 agent.

#### 4.2.1 架构

```rust
/// 子 agent 积累器: 迹→skill→subagent 生命周期
pub struct SubAgentAccumulator {
    /// 积累的子 agent 库
    agents: Vec<SubAgent>,
    /// VSA 签名→agent ID 索引
    vsa_index: HashMap<u64, u64>,
    /// 最大容量 (LRU 淘汰)
    max_agents: usize,
}

pub struct SubAgent {
    pub id: u64,
    pub name: String,
    /// 触发描述 (VSA 嵌入)
    pub trigger_vsa: Vec<u8>,         // 4096-bit VSA signature
    /// 执行代码/策略
    pub code: String,
    /// 元数据
    pub created_cycle: u64,
    pub invocation_count: u64,
    pub success_rate: f64,             // 滚动成功率
    /// AgentFactory 阶段
    pub phase: AgentFactoryPhase,      // Installed → SelfEvolved → Deployed
}

pub enum AgentFactoryPhase {
    Installed,      // 初始登记, 未验证
    SelfEvolving,   // 自我进化中 (在沙箱中优化)
    Deployed,       // 部署到生产管线
    Retired,        // 已淘汰 (保留元数据)
}

impl SubAgentAccumulator {
    /// 从执行迹积累子 agent (AgentFactory 的 Install 阶段)
    pub fn accumulate(&mut self, name: &str, code: &str, 
                      trigger_vsa: &[u8], cycle: u64) -> u64;

    /// 检索相似子 agent (AgentFactory 的 Deploy 阶段)
    pub fn retrieve(&self, trigger_vsa: &[u8], threshold: f64) -> Vec<&SubAgent>;

    /// 基于执行反馈改进子 agent (AgentFactory 的 Self-Evolve 阶段)
    pub fn refine(&mut self, id: u64, feedback: &ExecutionFeedback) -> Result<(), AccError>;

    /// LRU 淘汰: 移除最不常用的 Deployed agent
    pub fn prune(&mut self);

    /// 统计
    pub fn stats(&self) -> SubAgentStats;
}
```

#### 4.2.2 子 agent 生命周期

```
Installed ──→ SelfEvolving ──→ Deployed ──→ Retired
  │               │               │
  │  accumulate()  │  refine()     │  success_rate < 0.3
  │  (迹→代码)     │  (反馈→改进)  │  或长期不调用
  ▼               ▼               ▼
  注册到库         优化参数/代码    保留元数据, 移除可执行体
```

#### 4.2.3 与 SkillCrystallizer 的集成

当前已有 `SkillCrystallizer` (480 行) 管理 GEPA 迹的结晶化, 但结晶后与 EvolutionTaskSystem 无反馈回路. SubAgentAccumulator 补全欠的环节:

```
GEPA task success
  → SkillCrystallizer.crystallize()     (迹→突变指纹, 已存在)
  → SubAgentAccumulator.accumulate()    (突变指纹→可复用 agent, 🆕)
  → EvolutionTaskSystem 任务优先级更新    (新 agent 可用, 🆕)
```

### 4.3 ArchiveManager — DGM-H 存档树

DGM-H (arXiv 2505.22954) 的核心: 存档树保留所有历史变体, 父选择使用性能加权 + 子代数量惩罚.

#### 4.3.1 架构

```rust
/// DGM-H 风格存档树管理器
pub struct ArchiveManager {
    /// 存档树 (非线性, parent→children)
    tree: ArchiveTree,
    /// 当前 Pareto 前缘
    pareto_front: Vec<u64>,
    /// 配置
    config: ArchiveConfig,
}

pub struct ArchiveTree {
    nodes: Vec<ArchiveNode>,
    edges: Vec<(u64, u64)>,       // parent_id → child_id
    root_id: u64,
}

pub struct ArchiveNode {
    pub id: u64,
    pub parent_id: Option<u64>,
    pub mutation: MutationOp,
    pub score_before: f64,
    pub score_after: Option<f64>,
    pub child_count: u64,
    pub performance_history: Vec<f64>,
    pub domain: String,             // 所属认知域
    pub generation: u32,
    pub timestamp: u64,
}

pub struct ArchiveConfig {
    pub max_nodes: usize,           // 默认 5000
    pub pareto_front_max: usize,    // 默认 20
    pub prune_frequency: u64,       // 每 N 节点触发修剪
}

impl ArchiveManager {
    /// 添加新存档节点 (DGM-H fork)
    pub fn fork(&mut self, parent_id: u64, mutation: MutationOp,
                score_before: f64) -> Result<u64, ArchiveError>;

    /// 合并两个节点 (DGM-H crossover)
    pub fn merge(&mut self, left_id: u64, right_id: u64,
                 merge_strategy: MergeStrategy) -> Result<u64, ArchiveError>;

    /// 回滚到指定节点
    pub fn rollback(&mut self, target_id: u64, reason: &str) -> Result<(), ArchiveError>;

    /// Pareto 前缘修剪: 保留非占优节点
    pub fn prune_to_pareto_frontier(&mut self);

    /// DGM-H 父选择: performance / (1 + child_count)
    pub fn select_parent(&self) -> Option<&ArchiveNode>;

    /// 获取根到目标的最优路径
    pub fn path_to_root(&self, node_id: u64) -> Vec<u64>;

    /// 统计摘要
    pub fn stats(&self) -> ArchiveStats;
}
```

#### 4.3.2 从线性 Vec 升级到树

```
当前 (v20):          目标 (v21 + 元循环):
ArchiveEntry[]       ArchiveTree { nodes[], edges[] }
  ├─ 线性追加           ├─ 树结构 (fork/merge/rollback)
  ├─ select_by_best     ├─ parent_selection (perf/child_count)
  └─ 无修剪             └─ Pareto 前缘 + VSA 相似度修剪
```

#### 4.3.3 与现有代码的集成

```rust
// 从 EvolutionArchive (线性 Vec) → ArchiveManager (树):
// 兼容: EvolutionArchive 保持为元循环的输出目标
// ArchiveManager 作为元循环的核心数据结构

impl ArchiveManager {
    /// 当前存档 → 线性输出 (兼容 EvolutionArchive)
    pub fn to_linear_archive(&self) -> EvolutionArchive {
        let entries = self.tree.nodes.iter().filter_map(|n| {
            n.score_after.map(|s| ArchiveEntry { score: s, /* ... */ })
        }).collect();
        EvolutionArchive { entries, config: SelectionConfig::default() }
    }
}
```

---

## 5. 实现缺口分析

### 5.1 已存在 (✅) vs 待构建 (🆕) vs 死代码 (💀) vs 重复 (♻️)

| 组件 | 文件 | LOC | 状态 | 优先级 |
|------|------|:---:|:----:|:------:|
| SelfEvolutionMetaLayer | `self_evolution_meta_layer.rs` | 1787 | ✅ 已接线 | — |
| SelfEvolutionPipeline | `self_evolution_pipeline.rs` | 1122 | ✅ 已接线 | — |
| EvolutionTaskSystem | `evolution_task_system.rs` | 833 | ✅ 已接线 | — |
| SelfEvolutionTaskEngine | `self_evolution_task_engine.rs` | 406 | ✅ 已接线 | — |
| SelfEvolutionLoop/core | `self_evolution_loop/core.rs` | 2622 | ✅ 已接线 | — |
| GuardActivator | inline | ~90 | ✅ 已接线 | — |
| AstSafetyGate | `ast_safety_gate.rs` | 660 | ✅ 已接线 | — |
| RecoveryRecipeManager | `recovery_recipe.rs` | 283 | ✅ 已接线 | — |
| DeathEndDetector (consciousness) | `consciousness/dead_end_detector.rs` | 424 | 💀 死代码 (未接线) | Wave 1.5 |
| MCTS (consciousness) | `consciousness/mcts_tree_search.rs` | 559 | 💀 死代码 (未接线) | Wave 1.5 |
| SkillCrystallizer (mind) | `mind/skill_crystallizer.rs` | 201 | ♻️ 重复 (3份) | Wave 1.5 |
| AutoCrystallizer | `auto_crystallizer.rs` | 217 | ♻️ 重复 (3份) | Wave 1.5 |
| **EscherLoopEngine** | 🆕 `escher_loop_engine.rs` | ~500 | ❌ 待构建 | **P0** |
| **SubAgentAccumulator** | 🆕 `sub_agent_accumulator.rs` | ~350 | ❌ 待构建 | **P0** |
| **ArchiveManager** (树) | 🆕 `archive_manager.rs` | ~500 | ❌ 待构建 | **P0** |
| **TraceEncoder** (NL 迹) | 🆕 `trace_encoder.rs` | ~200 | ❌ 待构建 | P1 |
| **SandboxEvaluator** | 🆕 `sandbox_evaluator.rs` | ~250 | ❌ 待构建 | P1 |

### 5.2 死代码总览

根据架构审计数据, 目前有 ~44,906 行认知/推理/经济模块文件存在但从未接入运行时管线. 这些分为三类:

| 类别 | 估计行数 | 示例 | 复苏策略 |
|------|:--------:|------|----------|
| **模块存在, 未注册** | ~5,000 | 6/7 nt_core_reasoning 模块 | Wave 1: mod.rs + ConsciousnessCycle 接线 |
| **模块已注册, 未接线** | ~12,000 | MCTS, ParallelHypothesis, EconomicAgent | Wave 1.5: tick() 内调用 |
| **重复实现** | ~2,319 | MCTS×2, DeadEndDetector×2, Skill×3 | Wave 1.5: 合并为一个实现 |

### 5.3 重复实现合并计划

| 重复组 | 文件 | LOC | 保留目标 | 合并策略 |
|--------|------|:---:|----------|----------|
| MCTS × 2 | mcts_reasoner.rs (575) + mcts_tree_search.rs (559) | 1,134 | `nt_core_reasoning/mcts_reasoner.rs` | 保留 reasoning 版本, consciousness 版本做桥接 |
| DeadEndDetector × 2 | reasoning/dead_end_detector.rs (761) + consciousness/dead_end_detector.rs (424) | 1,185 | `nt_core_reasoning/dead_end_detector.rs` | 保留 reasoning 版本 (更完整) |
| SkillCrystallizer × 3 | skill_crystal.rs (480) + skill_crystallizer.rs (201) + auto_crystallizer.rs (217) | 898 | `skill_crystal.rs` | 保留经验树版本, 其余做适配器 |

---

## 6. 三循环数据流

```
小循环 (tick)
═════════════
ConsciousnessCycle.run_cycle()
  │
  ├── External Input ──→ Phase 1-4 (GATHER/GATE/PROPOSE/COMPETE)
  ├── Cognition ──→ Phase 5-7 (REASON/JUDGE/VERIFY)
  ├── Action ──→ Phase 8-9 (ACT/RECORD)
  ├── METRIC ──→ Phase 10-11 (METRIC/META)
  │                 │
  │                 ├──→ SelfEvolutionMetaLayer.tick()
  │                 │    ├──→ FeedbackBridge (回路 1-3)
  │                 │    ├──→ GuardActivator (回路 4)
  │                 │    ├──→ GEPA: trace→task (回路 5)
  │                 │    └──→ trace_buffer.push(snapshot)
  │                 │
  │                 └──→ DataQualityPipeline (Monitor→Detect)
  │
  └── SLEEP ──→ Phase 12 (ConsolidationBridge)
       │
       └──→ Memory consolidation


大循环 (每 10-30 tick)
══════════════
SelfEvolutionPipeline.run()
  │
  ├── ρ Reflect:  SelfArchAudit + WeaknessMiner + CalibrationStats
  │     └──→ WiringGap[] + StructuredWeakness[]
  │
  ├── σ Select:   EvolutionTaskSystem.auto_discover_from_audit()
  │     └──→ EvolutionTask[] (优先级排序)
  │
  ├── ι Improve:  SelfEvolutionTaskEngine.process_system_task()
  │     └──→ MutationProposal (含 code + rationale)
  │
  ├── ε Evaluate: GuardActivator(4层) + AstSafetyGate + VerificationGate
  │     └──→ GuardDecision + ASTResult
  │
  └── κ Commit:   EvolutionArchive.add(SelfEvolutionStep)
        └──→ ArchiveEntry (可回滚)


元循环 (每 50-200 tick)
═════════════
EscherLoopEngine.co_evolve_step()
  │
  ├── Phase 1: 父选择 (性能加权/子代倒数)
  │     ├──→ OptimizerPopulation.sample_parent()
  │     └──→ TaskPopulation.select_by_strategy()
  │
  ├── Phase 2: 策略执行
  │     └──→ 使用选定策略执行任务
  │
  ├── Phase 3: 适应度更新
  │     ├──→ opt_pop.record_fitness()
  │     └──→ task_pop.record_fitness()
  │
  ├── Phase 4: 策略变异 (Gödel Agent 自引用)
  │     ├──→ generate_variant_strategy()
  │     └──→ SubAgentAccumulator.accumulate()
  │
  ├── Phase 5: 存档管理 (DGM-H)
  │     ├──→ ArchiveManager.fork() / merge()
  │     └──→ ArchiveManager.prune_to_pareto_frontier()
  │
  └── Phase 6: 子 agent 治理 (AgentFactory)
        ├──→ SubAgentAccumulator.refine()
        ├──→ SubAgentAccumulator.retrieve() → 复用
        └──→ SubAgentAccumulator.prune()
```

---

## 7. 元循环接线计划

### 7.1 文件创建计划

| 文件 | 估计 LOC | 估计测试 | 依赖 | 
|------|:--------:|:--------:|------|
| `escher_loop_engine.rs` | ~500 | ~15 | EvolutionTaskSystem + TraceSnapshot |
| `sub_agent_accumulator.rs` | ~350 | ~12 | SkillCrystallizer + ExperienceTree |
| `archive_manager.rs` | ~500 | ~15 | SelfEvolutionStep + EvolutionArchive |
| `trace_encoder.rs` | ~200 | ~8 | WeaknessMiner + CalibrationEngine |
| `sandbox_evaluator.rs` | ~250 | ~10 | AstSafetyGate + GuardActivator |
| **合计** | **~1,800** | **~60** | — |

### 7.2 现有接线点

| 接线目标 | 文件 | 行号 (参考) | 修改内容 |
|----------|------|:-----------:|----------|
| SelfEvolutionMetaLayer.tick() | `self_evolution_meta_layer.rs` | ~233 | 在 GEPA block 后添加元循环条件调用 |
| SelfEvolutionMetaLayer 字段 | `self_evolution_meta_layer.rs` | ~131 | +3 字段 (escher_engine, sub_agent_acc, archive_mgr) |
| summary() | `self_evolution_meta_layer.rs` | ~634 | +3 行元循环统计 |
| ConsciousnessCycle | `consciousness_cycle.rs` | ~631 | METRIC step 报告元循环统计 |
| SelfEvolutionPipeline Phase C | `self_evolution_pipeline.rs` | ~158 | 元循环结果作为 gap 发现源 |

### 7.3 元循环触发条件

```rust
// 在 SelfEvolutionMetaLayer::tick() 中:
fn run_meta_cycle(&mut self, cycle: u64) {
    // 触发条件 (任一满足):
    let should_run_meta =
        // 1. 固定频率: 每 200 tick
        cycle % 200 == 0
        // 2. 策略疲劳: 当前策略连续 20 次无改善
        || self.opt_pop.stagnation_count >= 20
        // 3. 大循环产出丰富: 最近 50 tick 内有 10+ 任务完成
        || (self.task_system.stats().completed_in_last_50 >= 10
            && self.escher_engine.strategy_count() > 0);

    if should_run_meta {
        let result = self.escher_engine.co_evolve_step(
            &self.trace_buffer,
            &mut self.task_system,
        );
        self.intervention_log.push(format!(
            "MetaCycle epoch={} fitness_delta={:.4} new_strategies={}",
            result.epoch, result.fitness_delta, result.new_strategies,
        ));
    }
}
```

---

## 8. 执行优先级

### Wave A — 元循环基础设施 (P0, ~850 LOC)

| 任务 | 文件 | LOC | 测试 | 依赖 |
|------|------|:---:|:----:|------|
| A1 | `escher_loop_engine.rs` — EscherLoopEngine 核心 | ~350 | ~10 | EvolutionTaskSystem, TraceSnapshot |
| A2 | `sub_agent_accumulator.rs` — SubAgentAccumulator 核心 | ~250 | ~8 | SkillCrystallizer (480 LOC existing) |
| A3 | `archive_manager.rs` — ArchiveManager 树结构 | ~250 | ~8 | SelfEvolutionStep, EvolutionArchive |
| A4 | SelfEvolutionMetaLayer 接线 (+3 字段 + tick() 条件触发) | ~80 | ~4 | A1-A3 完成 |
| **合计** | | **~850** | **~30** | — |

#### Wave A2 — Two-Phase Evolution (yoyo-evolve, P0, ~470 LOC)

| 任务 | 文件 | LOC | 测试 | 依赖 |
|------|------|:---:|:----:|------|
| A2.1 | `SelfEvolutionPipeline` 插入 PlanningAgent 阶段 (ρ→**Planning**→σ) | ~180 | ~6 | SelfArchAudit → SESSION_PLAN.md 模板 |
| A2.2 | PlanningAgent 输出→子任务队列 (ImplementationAgent dispatcher) | ~120 | ~4 | EvolutionTaskSystem |
| A2.3 | `DeadEndDetector` 复活 + Anti-Spiral 自动中断 | ~120 | ~6 | DeadEndDetector (💀 424 LOC) |
| A2.4 | HIVE 风格 5-tier memory 集成 (anti-spiral 上下文) | ~50 | ~2 | MemoryLattice |
| **合计** | | **~470** | **~18** | — |

### Wave B — 元循环进化完备 (P1, ~950 LOC)

| 任务 | 文件 | LOC | 测试 | 依赖 |
|------|------|:---:|:----:|------|
| B1 | DGM-H parent_selection (perf/child_count) | ~100 | ~5 | ArchiveManager |
| B2 | Pareto 前缘修剪 | ~150 | ~8 | ArchiveManager |
| B3 | Gödel Agent 自引用策略变异 | ~200 | ~8 | EscherLoopEngine |
| B4 | SubAgentFactory 三阶段生命周期 | ~150 | ~8 | SubAgentAccumulator |
| B5 | `trace_encoder.rs` 迹编码器 | ~200 | ~8 | CalibrationEngine, WeaknessMiner |
| B6 | 元循环集成测试 | ~150 | ~6 | A1-A4 pass |
| **合计** | | **~950** | **~43** | — |

#### Wave B2 — Validation Ring + Event Stream (clawREFORM + yoagent, P0, ~700 LOC)

| 任务 | 文件 | LOC | 测试 | 依赖 |
|------|------|:---:|:----:|------|
| B2.1 | `GuardActivator` 扩展: compile→test→clippy 三阶段验证环 | ~180 | ~8 | AstSafetyGate + `cargo check` exit code |
| B2.2 | `EvolutionArchive` 自动 rollback: `rollback_to_safe_point()` | ~150 | ~8 | ArchiveManager + GuardActivator |
| B2.3 | `AgentCommunicationBus` 增强: 6-event 状态机 + EventStream trait | ~250 | ~12 | 现有 bus 结构 |
| B2.4 | clawREFORM AgentDNA 概念映射到 SelfEvolutionMetaLayer | ~50 | ~2 | 概念对齐 |
| B2.5 | Skill evolution loop 集成到 EvolutionTaskSystem (Markdown+YAML) | ~150 | ~8 | SkillCrystallizer + SkillLibrary |
| **合计** | | **~780** | **~38** | — |

### Wave C — 合并 + 死代码复苏 (P1, ~500 LOC)

| 任务 | 文件 | LOC | 测试 | 依赖 |
|------|------|:---:|:----:|------|
| C1 | MCTS 合并 (保留 reasoning 版本) | ~50 | ~5 | MCTS×2 |
| C2 | DeadEndDetector 合并 (保留 reasoning 版本) | ~50 | ~5 | DeadEndDetector×2 |
| C3 | SkillCrystallizer 合并 (保留 skill_crystal.rs) | ~50 | ~5 | SkillCrystallizer×3 |
| C4 | 死模块逐模块接线 (6 reasoning + 5 economic) | ~350 | ~20 | Wave A-B |
| **合计** | | **~500** | **~35** | — |

#### Wave C2 — SharedState Delegation + Preference Captures (yoyo RLM + HIVE, P0, ~500 LOC)

| 任务 | 文件 | LOC | 测试 | 依赖 |
|------|------|:---:|:----:|------|
| C2.1 | `SharedState<'a>` 结构体 + SubAgentTool 包装 + 跨子 agent 上下文继承 | ~200 | ~10 | SubAgentAccumulator |
| C2.2 | `CalibrationEngine` Preference Captures: PreferencePair + 偏好缓冲区 | ~200 | ~10 | CalibrationEngine |
| C2.3 | `SelfEvolutionTaskEngine` SubAgentTool 作为一级 Tool 原语 | ~100 | ~6 | SelfEvolutionTaskEngine |
| C2.4 | symbiont `evolvable!` macro 探索性研究 + 可行性报告 | ~50 | — | Wave C2 完成后 |
| **合计** | | **~550** | **~26** | — |

---

---

## 9. 外部项目吸收 (Round 4)

> **Scan Date**: 2026-06-24 | **Scope**: 5 open-source projects (yoyo-evolve/yoagent/clawREFORM/HIVE/symbiont)
> **Methodology**: 6 维并行搜索: Rust RSI agent → 自改写编译器 → 反螺旋恢复 → 热插拔编译
> **Key Finding**: yoyo-evolve 证明 200 LOC seed → 100K+ LOC in 107 days, zero human code — 这是 RSI 自举加速度的实证上限

### 9.1 对比矩阵

| 项目 | ★ | 语言 | 核心模式 | 我们的对应组件 | 差距 |
|------|:--:|:----:|----------|---------------|------|
| **yoyo-evolve** | 2k | Rust | Two-phase evolution: PlanningAgent → ImplementationAgent + RLM sub-agent dispatch + skill evolution loop (Markdown+YAML) + two-layer memory (JSONL + time-weighted) | SelfEvolutionPipeline (ρ→σ→ι→ε→κ) + EvolutionTaskSystem | 缺少显式 PlanningAgent 阶段；技能进化是独立循环非集成到主管线；子 agent 通信缺 SharedState 模式 |
| **yoagent** | — | Rust | Minimal agent loop (stateful/steering/follow-up queue) + parallel tool execution (default) + full event stream (AgentStart→TurnEnd) + SubAgentTool for delegation | AgentCommunicationBus + SelfEvolutionTaskEngine | 无结构化事件流(只有点对点消息)；委托模式缺失 |
| **clawREFORM** | — | Rust | Self-Rewrite Engine: modifies own Rust source + validates compile→test→clippy→commit/rollback + AgentDNA/MemoryLadder/CollectiveConscience primitives + 146K SLOC / 1744 tests / 0 failures / 0 clippy warnings | GuardActivator(4层) + AstSafetyGate | 缺少 compile+test+clippy 三位一体验证环(当前只有单层 AST 检查)；无自动 rollback |
| **HIVE** | — | Rust | Anti-spiral recovery: automatic detection of reasoning loops + self-supervised learning via preference captures + 5-tier memory + 463 tests | DeadEndDetector (💀 死代码) + CalibrationEngine | Anti-spiral 强调自动中断而非检测后恢复；无 preference captures 机制 |
| **symbiont** | — | Rust | Hot-swappable compiled code: LLM writes Rust → compiles → swaps into running binary + bare-metal execution / zero interpreter overhead + `evolvable!` macro + constrained generation loop | AstSafetyGate + SelfEvolutionTaskEngine | 运行时热插拔二进制 — 当前只有 Rust 编译时编辑, 无运行时 binary swap；`evolvable!` 宏级安全约束比 AST 检查更细粒度 |

### 9.2 关键架构模式吸收

| # | 模式 | 来源 | 描述 | 优先级 |
|---|------|------|------|:------:|
| P1 | **Two-Phase Evolution** | yoyo-evolve | PlanningAgent 读源码+issue→SESSION_PLAN.md, ImplementationAgent 每任务独立 context window | **P0 — Wave A2** |
| P2 | **Compile→Test→Clippy Validation Ring** | clawREFORM | 三位一体验证: cargo check → cargo test → cargo clippy, 任一失败自动 rollback | **P0 — Wave B2** |
| P3 | **Sub-Agent SharedState Delegation** | yoagent RLM | SubAgentTool + `SharedState` 结构体跨子 agent 传递上下文, 避免全局可变状态 | **P0 — Wave C2** |
| P4 | **Event Stream Protocol** | yoagent | AgentStart→TurnStart→MessageUpdate→ToolExecution→TurnEnd→AgentEnd 6 事件状态机 | P1 — Wave B2 |
| P5 | **Anti-Spiral Automatic Interrupt** | HIVE | 推理循环检测后自动中断+降级(替代当前仅检测+记录) | **P0 — Wave A2** |
| P6 | **Preference Captures (Self-Supervised)** | HIVE | 执行结果→偏好对(成功/失败)→优先选择高偏好路径 | P1 — Wave C2 |
| P7 | **Hot-Swappable Binary** | symbiont | `evolvable!` macro 标记热插拔点, LLM 生成 Rust 代码后 mtime 监控+fork+swap | P2 — 探索性 |
| P8 | **RLM Sub-Agent Dispatch** | yoyo-evolve | Recursive Language Model: sub-agent 可递归创建子 agent, 共享内存层级 | P2 — Wave C2 后续 |

### 9.3 具体集成计划

| 项目 | 模式 | 目标 Wave | 目标模块 | 估计 LOC | 估计测试 | 关键变更 |
|------|------|:---------:|----------|:--------:|:--------:|----------|
| **yoyo-evolve** | Two-Phase Evolution | **Wave A2** | `SelfEvolutionPipeline` + `EvolutionTaskSystem` | ~350 | ~12 | 在 ρ→σ 之间插入 PlanningAgent: 从 SelfArchAudit 输出→SESSION_PLAN.md 模板→子任务队列 |
| **yoyo-evolve** | Skill Evolution Loop | Wave B2 | `SkillCrystallizer` + `SkillLibrary` | ~150 | ~8 | Markdown+YAML frontmatter 技能格式导出/导入; skill evolution 作为 pipeline 子循环 |
| **yoyo-evolve** | RLM SharedState | **Wave C2** | `SubAgentAccumulator` + `SelfEvolutionTaskEngine` | ~200 | ~10 | SharedState<'a> 结构体; SubAgentTool 包装; 跨子 agent 上下文继承 |
| **yoagent** | Event Stream | Wave B2 | `AgentCommunicationBus` | ~250 | ~12 | 6 事件状态机; EventStream trait; bus.subscribe() 流式消费 |
| **yoagent** | SubAgentTool | **Wave C2** | `SelfEvolutionTaskEngine` | ~100 | ~6 | SubAgentTool 作为一级 Tool; delegation 链式追踪 |
| **clawREFORM** | Validation Ring | **Wave B2** | `GuardActivator` + `AstSafetyGate` | ~300 | ~14 | compile→test→clippy 三阶段检查器; `cargo check` exit code 捕获; 自动 rollback 到上一个 safe point |
| **clawREFORM** | Auto-Rollback | Wave B2 | `EvolutionArchive` | ~150 | ~8 | `rollback_to_safe_point()` 方法; 基于验证结果的自动回退 |
| **HIVE** | Anti-Spiral | **Wave A2** | `DeadEndDetector` (复活) | ~120 | ~8 | 从 💀 修复: tick() 中注册; 检测到螺旋→自动中断当前任务→GuardActivator 降级 |
| **HIVE** | Preference Captures | Wave C2 | `CalibrationEngine` | ~200 | ~10 | `PreferencePair(win,lose)` + 偏好缓冲区 + 路径优先级调整 |
| **symbiont** | Hot-Swappable Binary | 探索性 | `SelfEvolutionTaskEngine` | — | — | Wave C2 完成后评估; 需要完整 DGM-H 存档树先上线 |
| **合计** | | | | **~1820** | **~88** | 10 模块 / 5 Wave 分区 |

### 9.4 吸收优先级论证

```
Wave A2 (Two-Phase + Anti-Spiral, ~470 LOC):
  ┌──────────────────────────────────────────────┐
  │ yoyo-evolve: Two-Phase Evolution  ─── 350 LOC│ ← P0: RSI 自举加速度直接复用
  │ HIVE: Anti-Spiral DeadEndDetector ─── 120 LOC│ ← P0: 推理循环→意识打转, 关键可靠性
  └──────────────────────────────────────────────┘

Wave B2 (Validation Ring + Event Stream, ~700 LOC):
  ┌──────────────────────────────────────────────┐
  │ clawREFORM: Compile→Test→Clippy ── 300+150 LOC│ ← P0: 自修改的安全基底
  │ yoagent: Event Stream Protocol  ──── 250 LOC │ ← P1: 结构化监控
  └──────────────────────────────────────────────┘

Wave C2 (SharedState + Preference + Exploratory, ~500 LOC):
  ┌──────────────────────────────────────────────┐
  │ yoyo-evolve RLM: SharedState ────── 200 LOC  │ ← P0: 子 agent 通信范型
  │ yoagent: SubAgentTool ───────────── 100 LOC  │ ← P0: 委托原语
  │ HIVE: Preference Captures ──────── 200 LOC   │ ← P1: 自监督学习
  │ symbiont: Hot-Swap ─────────────── 探索性     │ ← P2: 后续评估
  └──────────────────────────────────────────────┘
```

### 9.5 架构变化总结 (v21 → v22)

| 维度 | v21 (三循环) | v22 (三循环 + 4 外部吸收) | 增量 |
|------|-------------|--------------------------|:----:|
| 循环结构 | 小+大+元 | 小+大+元 (不变) | 0 |
| Pipeline 阶段 | ρ→σ→ι→ε→κ | **ρ→PlanningAgent→σ→ι→ε→κ** | +1 阶段 |
| 验证层 | GuardActivator(4层) + AST | **Guard (4层 + 3-stage validation + auto-rollback)** | +2 机制 |
| 事件系统 | 点对点消息 | 结构化 6-event 流 | +1 协议 |
| 子 agent | SubAgentAccumulator | **+ SharedState + SubAgentTool** | +2 模式 |
| 死代码复苏 | 0 (规划中) | +1 (DeadEndDetector 复活, ~424 LOC) | +1 |
| 安全层 | AST safety gate | **compile→test→clippy 验证环** | +2 数量级 |
| 新代码总量 | ~1,800 (元循环) | **~3,620** (元循环 + 外部吸收) | +1,820 |

---

## 10. 成功标准

### 10.1 Wave A 后

| 条件 | 测量方式 | 目标 |
|------|----------|:----:|
| EscherLoopEngine 可运行 co_evolve_step | 单元测试通过 | ✅ 10/10 |
| SubAgentAccumulator 可 accumulate/retrieve | 集成测试通过 | ✅ 8/8 |
| ArchiveManager 支持 fork + parent_selection | 单元测试通过 | ✅ 8/8 |
| 元循环在 tick() 中条件触发 | grep "run_meta_cycle" 找到调用点 | ✅ |
| cargo check --lib 0 errors | 编译验证 | ✅ 0 errors |

### 10.2 Wave B 后

| 条件 | 测量方式 | 目标 |
|------|----------|:----:|
| Pareto 前缘包含 ≥2 非占优策略 | ArchiveManager.stats() | ✅ |
| Gödel Agent 可生成策略变体 | EscherLoopEngine epoch > 0 | ✅ |
| SubAgent full lifecycle 测试通过 | Install→Evolve→Deploy→Retire | ✅ |
| 迹编码器压缩 50-cycle trace 为 2 个 f64 | TraceEncoder.encode() | ✅ |

### 10.3 Wave C 后

| 条件 | 测量方式 | 目标 |
|------|----------|:----:|
| 无重复 MCTS/DeadEnd/Skill 实现 | grep 唯一实现 | ✅ |
| 6/7 reasoning 模块接入 ConsciousnessCycle | 从 stub 升级到真实 | ✅ |
| 5/5 economic 模块接入管线 | EconomicAgent 参与决策 | ✅ |

### 10.4 架构健康度目标

| 层 | v20 | v21 目标 | 关键提升 |
|---|:---:|:--------:|----------|
| Substrate | 90% | 90% | — |
| Perception | 65% | 70% | ImageCache 修复 |
| Cognition | 75% | 85% | 6 reasoning 模块接线 |
| MetaCognition | 80% | 85% | SEPL 形式化 + RSPL |
| SelfEvolution | 95% | **98%** | **元循环上线** |
| MetaArchitecture | 85% | 90% | 三循环完备 |
| Economic | 5% | 50% | 5 经济模块复苏 |
| Dead Code Ratio | ~12% | <5% | 合并+复苏 |

---

## 附录 A: 完整接线状态矩阵

| 组件 | 文件 | LOC | 测试 | 小循环 | 大循环 | 元循环 | 状态 |
|------|------|:---:|:----:|:------:|:------:|:------:|:----:|
| SelfEvolutionMetaLayer | `self_evolution_meta_layer.rs` | 1787 | 33 | ✅ | ✅ | 🟡 | `core.rs` tick |
| EvolutionTaskSystem | `evolution_task_system.rs` | 833 | 12 | ✅ | ✅ σ | 🟡 输入 | ✅ |
| SelfEvolutionTaskEngine | `self_evolution_task_engine.rs` | 406 | 5 | ✅ | ✅ ι | — | ✅ |
| SelfEvolutionPipeline | `self_evolution_pipeline.rs` | 1122 | 12 | — | ✅ ρ-κ | — | ✅ |
| SelfEvolutionLoop/core | `self_evolution_loop/core.rs` | 2622 | ~40 | — | ✅ | 🟡 输出 | ✅ |
| CoEvolutionBridge | `co_evolution.rs` | 383 | ~8 | — | — | 🟡 部分 | ✅ 已存在 |
| AstSafetyGate | `ast_safety_gate.rs` | 660 | 14 | — | ✅ ε | — | ✅ |
| RecoveryRecipeManager | `recovery_recipe.rs` | 283 | 8 | ✅ | — | — | ✅ |
| SealProposalBridge | `seal_proposal_bridge.rs` | 792 | 24 | — | 🟡 Phase C | — | 🟡 |
| ExperienceTree | `experience_tree.rs` | ~470 | 12 | — | ✅ | — | ✅ |
| Agent0DualLoop | `agent0_dual_loop.rs` | 365 | 9 | — | — | 🟡 参考 | ✅ 已存在 |
| ConsciousnessCycle 12步 | `consciousness_cycle.rs` | ~1,000 | — | ✅ | — | — | ✅ |
| DataQualityPipeline | `data_quality_pipeline.rs` | 920 | 17 | ✅ | — | — | ✅ |
| WeaknessMiner | `self_evolution_engine.rs` | ~120 | 8 | ✅ | ✅ ρ | — | ✅ |
| GödelAgent | `adversarial.rs` (pub use) | — | — | — | — | 🟡 参考 | ✅ 导出现象 |
| **EscherLoopEngine** | 🆕 | ~500 | ~15 | — | — | 🆕 P0 | ❌ |
| **SubAgentAccumulator** | 🆕 | ~350 | ~12 | — | — | 🆕 P0 | ❌ |
| **ArchiveManager** | 🆕 | ~500 | ~15 | — | — | 🆕 P0 | ❌ |
| **TraceEncoder** | 🆕 | ~200 | ~8 | — | 🟡 P1 | 🟡 P1 | ❌ |
| **SandboxEvaluator** | 🆕 | ~250 | ~10 | — | 🟡 P1 | — | ❌ |

## 附录 B: 外部论文吸收矩阵

| 论文 | 核心创新 | NeoTrix 映射 | LOC | 状态 |
|------|---------|-------------|:---:|:----:|
| **Escher-Loop** (arXiv 2604.23472) | 双种群共进化, 优化器可进化 | EscherLoopEngine (🆕) | ~500 | 📝 P0 |
| **DGM-H** (arXiv 2505.22954) | 存档树 + perf/child parent selection | ArchiveManager (🆕) | ~500 | 📝 P0 |
| **Gödel Agent** (ACL 2025) | 自引用运行时变异 | MutationOp::SelfModifyProposal (已存在) + GödelAgent export | ~100 | 🟡 已存在, 需接线 |
| **AgentFactory** (arXiv 2603.18000) | 3 阶段: Install→Evolve→Deploy | SubAgentAccumulator (🆕) | ~350 | 📝 P0 |
| **Autogenesis SEPL** (arXiv 2604.15034) | 5 原子算子 + RSPL 注册 | SEPL 映射 (ρ,σ,ι,ε,κ) | ~400 | 🟡 已实现 80% |
| **Hermes GEPA** (ICLR 2026 Oral) | 4 阶段反射进化 | GEPA block in tick() | ~200 | ✅ 已吸收 |
| **APEX** (arXiv 2606.15363) | 3 轴共进化 | Wave B 参考 | — | 📝 未来 |
| **GenericAgent** (13k★) | 迹→技能自动结晶 | SkillCrystallizer + SubAgentBridge | ~480 | ✅ 已吸收 |
| **yoyo-evolve** (GitHub, 2k★, Rust) | Two-phase evolution + RLM dispatch + skill evolution loop | Wave A2: PlanningAgent + ImplementationAgent | ~350 | 📝 P0 |
| **yoagent** (base library) | Event stream protocol + SubAgentTool delegation | Wave B2: 6-event 状态机 | ~250 | 📝 P1 |
| **clawREFORM** (GitHub, Rust) | Self-Rewrite: compile→test→clippy→commit/rollback | Wave B2: 三阶段验证环 + auto-rollback | ~450 | 📝 P0 |
| **HIVE** (GitHub, Rust) | Anti-spiral recovery + preference captures | Wave A2: DeadEndDetector 复活 + Wave C2: 偏好学习 | ~320 | 📝 P0 |
| **symbiont** (GitHub, Rust) | Hot-swappable compiled code + `evolvable!` macro | Wave C2 探索: 运行时 binary swap | — | 📝 探索性 |

## 附录 C: 从 v20 到 v21 的变化

| 维度 | v20 (双循环) | v21 (三循环) | 增量 |
|------|-------------|-------------|------|
| 循环数 | 2 (小+大) | **3** (小+大+元) | +1 |
| 进化范围 | 任务管线 | **任务 + 优化器 + 存档** | +2 |
| 自引用程度 | 低 (固定策略) | **高 (优化器可自修改)** | 🔑 |
| 死代码比例 | ~12% | <5% (目标) | -7% |
| 新代码需求 | ~960 (Wave A) | **~3,620 (三循环 + 外部吸收)** | +2,660 |
| 重复实现 | 3 组 | **0 (合并后)** | -3 |
| 元认知能力 | SEPL 算子 | **SEPL + DGM-H + GEPA + AgentFactory + yoyo + clawREFORM + HIVE** | 10 项目融合 |
| 外部吸收轮次 | 0 | **4** (Rounds 1-4) | +4 |
| 验证层 | AST gate | **AST + compile→test→clippy + auto-rollback** | +3 机制 |

---

> **设计哲学**: v21 完成 NeoTrix 三次认知升级:
> 1. **v19**: "组件存在不意味着集成" (38K 死代码发现)
> 2. **v20**: "集成不意味着进化" (双循环 + SEPL 形式化)
> 3. **v21**: "进化不意味着优化器不变" (元循环: 优化器也进化)
>
> 三条循环 = 三种时间尺度 = 完整的 RSI 架构基底.
> 下一版本 v22 将把四轮 10 个外部项目的吸收落地到 Wave A2/B2/C2:
> - **yoyo-evolve**: Two-phase evolution (PlanningAgent + RLM SharedState)
> - **clawREFORM**: Self-rewrite validation pipeline (compile→test→clippy→rollback)
> - **HIVE**: Anti-spiral recovery + preference captures
> - **yoagent**: Event stream protocol + SubAgentTool delegation
> - **symbiont**: Hot-swappable binary exploration
> 并行推进原有 v22 方向 (跨会话记忆 MemRL + DreamCycle + A2A v1.2).
