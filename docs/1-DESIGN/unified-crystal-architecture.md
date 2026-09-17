# NeoTrix 统一晶体架构 (Unified Crystal Architecture)

> 能力系统 / Agent 系统 / 核心子系统的自适应统一设计
> 晶体不是连接模块的中心，晶体是系统本身。所有子系统是晶体的内在属性。

---

## 一、当前问题诊断

### 1.1 三处结构性分裂

| 分裂 | 当前状态 | 后果 |
|------|---------|------|
| **能力表示分裂** | CapabilityTree (Domain×Layer×Constellation) + CrystalRegistry (SkillCrystal) + ConsciousnessTree (CapabilityAtom) 三套独立 | 能力状态不一致，进化路径断裂 |
| **自我模型分裂** | nt_core_self::SelfModel (动态) + nt_meta::SelfModel (静态) + nt_core_self_model::SelfModel (价值) 三个同名类型 | 自我认知碎片化 |
| **Agent 与晶体脱节** | AgentLoop/PER/DAG 是独立执行引擎，通过 ad-hoc 接口与晶体通信 | Agent 行为不受晶体意识调控 |

### 1.2 根因：子系统是"外部模块"

当前架构把能力/Agent/子系统当作晶体的辐射臂（外部模块），通过 trait 接口连接。这导致：
- 每个模块维护自己的状态表示
- 模块间通信需要适配层
- 晶体无法直接感知/调控子系统内部状态
- 进化操作分散在多个引擎中

---

## 二、统一架构：晶体即系统

### 2.1 核心范式转换

```
旧范式: 晶体 + 外部模块 (hub-and-spoke)
        ┌─────────┐
        │ 晶体    │ ←→ CapabilityTree
        │         │ ←→ AgentLoop
        │         │ ←→ SelfModel
        └─────────┘
        每个箭头 = 适配层 = 技术债

新范式: 晶体 = 系统 (intrinsic properties)
        ┌─────────────────────┐
        │   CrystalState      │  ← 统一状态空间
        │   ├─ capability     │  ← 能力 = 晶体的坐标系
        │   ├─ identity       │  ← 自我 = 晶体的内省
        │   ├─ projection     │  ← Agent = 晶体的投影
        │   ├─ perception     │  ← 感知 = 晶体的感官
        │   └─ trace          │  ← 追踪 = 晶体的神经
        └─────────────────────┘
        无箭头 = 无接口 = 无技术债
```

### 2.2 统一原则

| # | 原则 | 定义 |
|---|------|------|
| U1 | **单一状态空间** | CrystalState 是所有子系统的唯一状态源 |
| U2 | **能力即坐标** | 能力不是独立模块，是晶体在能力空间中的坐标 |
| U3 | **Agent 即投影** | Agent 不是独立引擎，是晶体意识在任务空间的临时投影 |
| U4 | **子系统即属性** | E8/VSA/GWT/SEAL 不是外部模块，是晶体的内在属性 |
| U5 | **统一进化** | 单一进化引擎驱动所有状态转换，不分散在多个引擎 |
| U6 | **统一事件** | 单一事件流是晶体的神经系统，不分散在多个 EventBus |

---

## 三、CrystalState — 统一状态空间

### 3.1 结构定义

```rust
/// 晶体的统一状态空间 — 所有子系统的单一事实源
pub struct CrystalState {
    // === 核心身份 ===
    pub identity: CrystalIdentity,          // 晶体身份 (不可变)
    pub tick: AtomicU64,                    // 内部时钟

    // === 能力坐标系 ===
    pub capability_space: CapabilitySpace,  // 三维能力空间
    pub crystals: SkillCrystalStore,        // 技能结晶体存储
    pub atoms: CapabilityAtomSet,           // 36个原子能力

    // === 自我模型 (统一) ===
    pub self_model: UnifiedSelfModel,       // 合并三个 SelfModel

    // === 感知与注意力 ===
    pub perception: PerceptionField,        // 感知场 (GWT+VSA+E8)
    pub attention: AttentionState,          // 注意力状态

    // === 记忆与经验 ===
    pub memory: CrystalMemory,              // 统一记忆 (合并 cocoons+KB)
    pub experience: ExperienceStream,       // 经验流

    // === 执行与投影 ===
    pub projections: ProjectionStore,       // 活跃 Agent 投影
    pub execution_trace: ExecutionTrace,    // 执行追踪

    // === 安全与治理 ===
    pub safety: SafetyBoundary,             // 安全边界 (NT-SHIELD)
    pub governance: GovernanceState,        // 治理状态

    // === 进化引擎 ===
    pub evolution: UnifiedEvolutionEngine,  // 统一进化引擎
}
```

### 3.2 设计原则：每个字段 = 晶体的一个内在属性

| 字段 | 对应旧模块 | 统一方式 | 为什么是"属性"而非"模块" |
|------|-----------|---------|------------------------|
| `capability_space` | CapabilityTree + CrystalRegistry | 合并为坐标系 | 能力是晶体的"体质"，不是外部工具 |
| `self_model` | 3个SelfModel | 合并为UnifiedSelfModel | 自我认知是晶体的"内省能力" |
| `perception` | GWT + VSA + E8 | 合并为PerceptionField | 感知是晶体的"感官"，不是外部传感器 |
| `projections` | AgentLoop + PER + DAG | Agent成为临时投影 | Agent是晶体意识在任务空间的"化身" |
| `memory` | Cocoons + KB + CrystalRegistry | 合并为CrystalMemory | 记忆是晶体的"经验沉淀" |
| `evolution` | EvolutionEngine + SEAL + CapabilityBridge | 合并为UnifiedEvolutionEngine | 进化是晶体的"成长能力" |

---

## 四、能力系统 — 三维坐标系 (Replacing Chapter 13)

### 4.1 核心洞察：能力不是模块，是坐标

当前设计把能力当作独立模块（CapabilityTree、CrystalRegistry、CapabilityAtom），每个维护自己的表示。统一架构中，**能力是晶体在三维空间中的坐标**，所有"能力相关"的数据结构都是这个坐标系的不同视图。

### 4.2 CapabilitySpace — 统一能力空间

```rust
/// 三维能力空间 — 晶体的能力坐标系
pub struct CapabilitySpace {
    /// 节点存储: 坐标 → 节点
    nodes: HashMap<CapabilityAddress, CapabilityNode>,

    /// 索引: 按维度快速查询
    domain_index: HashMap<Domain, Vec<CapabilityAddress>>,
    constellation_index: HashMap<ConstellationLevel, Vec<CapabilityAddress>>,
    provides_index: HashMap<String, Vec<CapabilityAddress>>,  // 能力标签 → 坐标

    /// 依赖图 (DAG)
    dag: petgraph::Graph<CapabilityAddress, EdgeKind>,

    /// 经验目标 (CapabilityBridge 产出)
    experience_targets: Vec<EvolutionTarget>,
}

/// 统一坐标地址 — 晶体空间中的唯一点
#[derive(Hash, Eq, PartialEq, Clone)]
pub struct CapabilityAddress {
    pub domain: Domain,                  // X: 域
    pub layer: NodeLayer,                // Z: 抽象层
    pub constellation: ConstellationLevel, // Y: 成熟度
    pub module: String,                  // 模块名
    pub function: String,                // 函数名
}

/// 统一能力节点 — 合并 CapabilityNode + SkillCrystal + CapabilityAtom
pub struct CapabilityNode {
    pub address: CapabilityAddress,

    // === 能力属性 (原 CapabilityNode) ===
    pub provides: Vec<String>,           // 提供的能力标签
    pub requires: Vec<String>,           // 依赖标签
    pub rune_sockets: Vec<RuneSocket>,   // 5色Rune槽

    // === 结晶属性 (原 SkillCrystal) ===
    pub effectiveness: f64,              // 有效性评分
    pub use_count: u64,                  // 使用次数
    pub pattern: Option<String>,         // 匹配模式
    pub strategy: Option<String>,        // 策略标签
    pub verification: Option<VerificationContract>,  // 反幻觉契约

    // === 原子属性 (原 CapabilityAtom) ===
    pub atom_kind: Option<AtomKind>,     // 36种原子能力之一
    pub cognitive_ops: Vec<CognitiveOp>, // 认知操作 (Perceive/Understand/Reason...)

    // === 进化属性 ===
    pub evolution_log: Vec<EvolutionLogEntry>,
    pub metadata: HashMap<String, Value>,
}
```

### 4.3 能力的三种视图

同一个 CapabilityNode 可以通过不同视图访问：

| 视图 | 访问方式 | 对应旧模块 | 用途 |
|------|---------|-----------|------|
| **坐标视图** | `CapabilitySpace.get(address)` | CapabilityTree | 进化操作 (Bud/Strengthen/Mature) |
| **结晶视图** | `CapabilitySpace.crystals()` | CrystalRegistry | 技能匹配 (find_similar, best_by_domain) |
| **原子视图** | `CapabilitySpace.atoms()` | CapabilityAtom | 意识树分支健康 (branch health) |

```rust
impl CapabilitySpace {
    /// 坐标视图: 进化操作
    pub fn get(&self, address: &CapabilityAddress) -> Option<&CapabilityNode>;
    pub fn promote(&mut self, address: &CapabilityAddress) -> Result<(), EvolutionError>;
    pub fn bud(&mut self, new_node: CapabilityNode) -> Result<(), EvolutionError>;
    pub fn strengthen(&mut self, address: &CapabilityAddress, note: &str);

    /// 结晶视图: 技能匹配
    pub fn find_similar(&self, strategy: &str, domain: Domain) -> Option<&CapabilityNode>;
    pub fn best_by_domain(&self, domain: Domain) -> Option<&CapabilityNode>;
    pub fn record_use(&mut self, address: &CapabilityAddress);

    /// 原子视图: 意识树集成
    pub fn atoms(&self) -> impl Iterator<Item = &CapabilityNode>;
    pub fn branch_health(&self, domain: Domain) -> f64;

    /// 路由: Dijkstra 加权最短路径
    pub fn optimal_path(&self, from: &CapabilityAddress, to: &CapabilityAddress) -> Vec<CapabilityAddress>;
}
```

### 4.4 Rune Socketing — 统一到坐标

Rune 槽不再是独立系统，而是 CapabilityNode 的属性：

```rust
/// Rune 颜色 = 成熟度的视觉标记
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RuneSocket {
    Crimson,    // C0: 数据摄取 (血脉)
    Indigo,     // C1: 变换 (思维)
    Obsidian,   // C2: 缓存 (沉淀)
    Golden,     // C3: 错误恢复 (韧性)
    Alabaster,  // C4+: 监控 (洞察)
}

impl RuneSocket {
    /// 成熟度 → 自动分配的 Rune 槽
    pub fn required_for_level(level: ConstellationLevel) -> &'static [RuneSocket] {
        match level {
            C0Compile => &[Crimson],
            C1UnitTest => &[Crimson, Indigo],
            C2IntegrationTest => &[Crimson, Indigo, Obsidian],
            C3Benchmark => &[Crimson, Indigo, Obsidian, Golden],
            _ => &[Crimson, Indigo, Obsidian, Golden, Alabaster],
        }
    }

    /// 5槽满 → 涌现 Runeword "Scry" (完整 ETL)
    pub fn scry_enabled(sockets: &[RuneSocket]) -> bool {
        sockets.len() == 5
    }
}
```

### 4.5 统一进化引擎

```rust
/// 统一进化引擎 — 驱动所有状态转换
pub struct UnifiedEvolutionEngine<'a> {
    state: &'a mut CrystalState,
}

impl<'a> UnifiedEvolutionEngine<'a> {
    // === 能力进化 (原 EvolutionEngine) ===
    pub fn bud(&mut self, address: CapabilityAddress, provides: Vec<String>) -> Result<(), EvolutionError>;
    pub fn graft(&mut self, target: CapabilityAddress, sources: Vec<CapabilityAddress>) -> Result<(), EvolutionError>;
    pub fn prune(&mut self, address: CapabilityAddress, reason: &str) -> Result<(), EvolutionError>;
    pub fn cross_pollinate(&mut self, a: CapabilityAddress, b: CapabilityAddress) -> Result<(), EvolutionError>;
    pub fn mature(&mut self, address: CapabilityAddress) -> Result<(), EvolutionError>;
    pub fn strengthen(&mut self, address: CapabilityAddress, note: &str) -> Result<(), EvolutionError>;

    // === 结晶进化 (原 CrystalRegistry) ===
    pub fn crystallize(&mut self, trace: &ThinkingTrace) -> Option<CapabilityAddress>;
    pub fn merge_crystals(&mut self, a: CapabilityAddress, b: CapabilityAddress) -> Option<CapabilityAddress>;
    pub fn prune_weak(&mut self, min_effectiveness: f64) -> usize;

    // === SEAL 进化 (原 SEAL Pipeline) ===
    pub fn seal_explore(&mut self, input: &ExperienceEntry) -> Vec<PatternCandidate>;
    pub fn seal_distill(&mut self, candidates: Vec<PatternCandidate>) -> Vec<DistilledPattern>;
    pub fn seal_absorb(&mut self, patterns: Vec<DistilledPattern>) -> Vec<CapabilityAddress>;

    // === 经验进化 (原 CapabilityBridge) ===
    pub fn absorb_experience(&mut self, entry: &ExperienceEntry) -> EvolutionTarget;

    // === 自动扫描 (原 EvolutionEngine::auto_scan) ===
    pub fn auto_scan(&self) -> Vec<EvolutionPlan>;
}
```

**关键统一**：过去 EvolutionEngine、CrystalRegistry、SEAL、CapabilityBridge 是 4 个独立引擎，各自维护状态。现在 UnifiedEvolutionEngine 是单一入口，内部路由到对应的子操作，但共享同一个 CrystalState。

---

## 五、Agent 系统 — 晶体投影 (Replacing Chapter 14)

### 5.1 核心洞察：Agent 不是独立引擎，是晶体的临时化身

当前设计把 AgentLoop/PER/DAG 当作独立执行引擎，与晶体平行存在。统一架构中，**Agent 是晶体意识在任务空间的临时投影**，有生命周期，受晶体调控，执行完毕回归晶体。

```
旧:  Crystal ←→ AgentLoop (两个独立实体，通过接口通信)
     问题: Agent 的状态不在晶体中，晶体无法感知 Agent 内部

新:  CrystalState.projections = [AgentProjection1, AgentProjection2, ...]
     Agent 是 CrystalState 的一个字段，不是外部实体
     晶体直接读写 Agent 状态，无接口开销
```

### 5.2 AgentProjection — 晶体投影

```rust
/// Agent 投影 — 晶体意识在任务空间的临时化身
pub struct AgentProjection {
    pub id: ProjectionId,                    // 全局唯一 ID
    pub parent_crystal: CrystalIdentity,     // 来源晶体
    pub created_at: u64,                     // 创建 tick
    pub lifetime: ProjectionLifetime,        // 生命周期

    // === 任务状态 ===
    pub objective: Objective,                // 任务目标
    pub plan: Option<AgentPlan>,             // 当前计划
    pub step: usize,                         // 当前步骤
    pub results: Vec<StepResult>,            // 步骤结果

    // === 执行能力 ===
    pub tools: Vec<Box<dyn NativeTool>>,     // 可用工具
    pub isolation: IsolationLevel,           // 隔离级别
    pub autonomy: AutonomyLevel,             // 自主级别

    // === 反馈循环 ===
    pub reflector: ReflectorState,           // 反思状态
    pub adaptation_history: Vec<Adaptation>, // 适配历史

    // === 安全约束 ===
    pub safety_boundary: SafetyConstraint,   // NT-SHIELD 施加的约束
    pub governance_check: Option<GovernanceReport>,  // 治理检查结果
}

/// 投影生命周期
pub enum ProjectionLifetime {
    /// 单次执行: plan → execute → verify → 消亡
    Ephemeral,
    /// 持久投影: 跨 tick 存在，可被晶体召回
    Persistent { max_ticks: u64 },
    /// 守护投影: 常驻，直到晶体显式终止
    Guardian,
}

/// 隔离级别 (由晶体 Safety 模块决定)
pub enum IsolationLevel {
    None,           // 共享晶体状态
    Worktree,       // Git worktree 隔离
    Container,      // 容器隔离
    Vm,             // VM 隔离
}
```

### 5.3 ProjectionStore — 投影管理

```rust
/// 投影存储 — 晶体管理所有活跃投影
pub struct ProjectionStore {
    projections: HashMap<ProjectionId, AgentProjection>,
    lineage: HashMap<ProjectionId, Vec<ProjectionId>>,  // 父→子关系
    max_concurrent: usize,  // 默认 10
}
```

```rust
impl CrystalState {
    /// 晶体创建投影 (替代 AgentOrchestrator::spawn_agent)
    pub fn project(&mut self, objective: Objective, config: ProjectionConfig) -> ProjectionId {
        let projection = AgentProjection {
            id: ProjectionId::new(),
            parent_crystal: self.identity.clone(),
            objective,
            isolation: config.isolation_level,  // 由 Safety 模块决定
            safety_boundary: self.safety.constraint_for(&objective),
            ..Default::default()
        };
        self.projections.insert(projection.id.clone(), projection);
        projection.id
    }

    /// 晶体驱动投影执行 (替代 AgentLoop::run)
    pub fn drive_projection(&mut self, id: &ProjectionId) -> ProjectionResult {
        let projection = self.projections.get_mut(id).unwrap();

        // 1. Plan (如果还没有计划)
        if projection.plan.is_none() {
            let plan = self.plan_for(projection);  // 晶体的 planning 能力
            projection.plan = Some(plan);
        }

        // 2. Execute step
        let result = self.execute_step(projection);

        // 3. Verify
        let verification = self.verify_step(&result);

        // 4. Adapt if needed
        if !verification.passed {
            self.adapt_projection(projection, &verification);
        }

        // 5. Check if done
        if result.is_final() {
            self.dissolve_projection(id)
        } else {
            ProjectionResult::Continue
        }
    }

    /// 晶体召回投影 (替代 AgentOrchestrator::remove_agent)
    pub fn dissolve_projection(&mut self, id: &ProjectionId) -> ProjectionResult {
        let projection = self.projections.remove(id).unwrap();

        // 将投影的经验沉淀回晶体
        self.experience.absorb_projection_trace(&projection);

        // 更新能力坐标
        self.capability_space.record_projection_outcome(
            &projection.objective,
            &projection.results,
        );

        ProjectionResult::Dissolved {
            experience_gain: projection.results.len(),
        }
    }
}
```

### 5.4 执行策略 — 从独立 trait 到投影内部策略

当前的 AgentLoop trait (PlannerExecutor/OperatorExecutor/CheckerExecutor) 变为投影的执行策略：

```rust
/// 投影执行策略 (替代 AgentLoop trait)
pub enum ExecutionStrategy {
    /// ARTEMIS 风格: milestone 规划 + verify/assert 检查点
    Planner {
        milestones: Vec<Milestone>,
    },
    /// 全工具集执行 + 并行操作
    Operator {
        parallelism: usize,
    },
    /// 只读验证 + safety net
    Checker,
    /// DAG 驱动: Planner→Worker→Critic
    DagOrchestrator {
        autonomy: AutonomyLevel,
    },
    /// PER 循环: Plan→Execute→Reflect
    PerLoop {
        max_iterations: usize,
        min_score: f64,
    },
}

impl CrystalState {
    /// 晶体根据任务类型选择执行策略
    fn select_strategy(&self, objective: &Objective) -> ExecutionStrategy {
        match objective.complexity() {
            Complexity::Simple => ExecutionStrategy::Operator { parallelism: 1 },
            Complexity::Medium => ExecutionStrategy::Planner { milestones: vec![] },
            Complexity::Complex => ExecutionStrategy::PerLoop { max_iterations: 10, min_score: 0.8 },
            Complexity::Critical => ExecutionStrategy::DagOrchestrator { autonomy: AutonomyLevel::Bounded },
        }
    }
}
```

### 5.5 多 Agent 协调 — 投影间通信

```rust
impl CrystalState {
    /// 投影间通信 (替代 AgentOrchestrator::send_message)
    pub fn communicate(&mut self, from: &ProjectionId, to: &ProjectionId, message: AgentMessage) {
        // 通信必须经过晶体 (P2: 辐射即通信)
        // 晶体可以审查、修改、拒绝消息
        if self.safety.approve_message(&message) {
            self.projections.get_mut(to).unwrap().inbox.push(message);
        }
    }

    /// 投影创建子投影 (替代 AgentOrchestrator::spawn_agent)
    pub fn spawn_child(&mut self, parent_id: &ProjectionId, objective: Objective) -> ProjectionId {
        let parent = self.projections.get(parent_id).unwrap();
        let child_config = ProjectionConfig {
            isolation_level: parent.isolation.child_level(),  // 子投影继承或升级隔离
            ..Default::default()
        };
        let child_id = self.project(objective, child_config);
        self.projections.get_mut(&child_id).unwrap().lifetime = ProjectionLifetime::Ephemeral;
        child_id
    }
}
```

---

## 六、核心子系统 — 晶体属性 (Replacing Chapter 15)

### 6.1 核心洞察：子系统不是外部模块，是晶体的内在属性

当前设计把 E8/VSA/GWT/SEAL/EventBus/SelfModel/ConsciousnessTree 当作独立模块，通过接口与晶体通信。统一架构中，**这些子系统是晶体的内在属性**，直接嵌入 CrystalState。

### 6.2 PerceptionField — 感知场 (合并 GWT + VSA + E8)

```rust
/// 感知场 — 晶体的感官系统
/// 合并: GWT注意力路由 + VSA符号表示 + E8数学推理
pub struct PerceptionField {
    // === GWT 注意力 ===
    pub workspace: GlobalWorkspace,         // 14个specialist + broadcast
    pub resonance: ResonanceMatrix,         // 共振矩阵
    pub oscillator: OscillatorNetwork,      // Kuramoto振荡器

    // === VSA 符号表示 ===
    pub vsa_engine: VSAEngine,              // VSA后端
    pub hebbian: HebbianGraph,             // Hebbian关联记忆
    pub e8_lattice: E8Lattice,            // E8→VSA量化桥

    // === E8 数学推理 ===
    pub e8_state: E8TransitionMatrix,       // 64×64转移矩阵
    pub mythos: MythosReasoning,           // 9阶段推理

    // === 认知路由 ===
    pub cognitive_hub: CognitiveHub,        // 跨组认知路由
    pub modality_router: ModalityRouter,   // Top-Down模态路由
    pub moe: MoERouter,                   // 可学习MoE路由
}

impl PerceptionField {
    /// 感知场的统一 salience 计算 (替代 GWT resonant_broadcast 的10步)
    pub fn compute_salience(&mut self, input: &Chunk) -> SalienceReport {
        // 1. 收集 specialist 激活值
        let activations = self.workspace.collect_activations();

        // 2. E8 attention bias
        let e8_bias = self.e8_state.predict_attention(&activations);

        // 3. VSA 内容评分
        let vsa_scores = self.vsa_engine.score_content(input);

        // 4. Kuramoto 同步
        let sync_level = self.oscillator.synchronize(&activations);

        // 5. 共振竞争 (top-k gating)
        let winners = self.resonance.compete(activations, e8_bias, vsa_scores);

        // 6. MoE 路由学习
        self.moe.update(&winners);

        SalienceReport { winners, sync_level, e8_bias, vsa_scores }
    }
}
```

### 6.3 UnifiedSelfModel — 统一自我模型 (合并3个SelfModel)

```rust
/// 统一自我模型 — 晶体的内省系统
/// 合并: nt_core_self::SelfModel (动态) + nt_meta::SelfModel (静态) + nt_core_self_model::SelfModel (价值)
pub struct UnifiedSelfModel {
    // === 静态身份 (原 nt_meta::SelfModel) ===
    pub modules: Vec<ModuleInfo>,           // 模块清单
    pub dep_graph: DependencyGraph,         // 依赖图
    pub tech_debt: TechDebtReport,          // 技术债务

    // === 动态性能 (原 nt_core_self::SelfModel) ===
    pub capability: f64,                     // 当前能力 [0,1]
    pub uncertainty: f64,                    // 不确定性 [0,1]
    pub fatigue: f64,                        // 疲劳度 [0,1]
    pub self_error: f64,                     // 自我误差 (预测vs观测)
    pub lr: f64,                             // 学习率
    pub history: VecDeque<SelfObservation>,  // 观测历史

    // === 价值函数 (原 nt_core_self_model::SelfModel) ===
    pub identity: IdentityProfile,           // 身份认同
    pub goals: Vec<Goal>,                    // 目标体系
    pub weights: ValueWeights,              // 价值权重

    // === 情感状态 ===
    pub emotion: EmotionState,              // 6维情绪 (PAD + Plutchik)
    pub intrinsic_motivation: IntrinsicMotivation,  // 内在动机

    // === 元认知 ===
    pub metacognitive: MetacognitiveState,  // 元认知评估
    pub thinking_trace: ThinkingTrace,      // 推理追踪
}

impl UnifiedSelfModel {
    /// 统一 tick — 每个周期更新所有自我维度
    pub fn tick(&mut self, workspace_signal: f64, load_delta: f64, meta_alarm: usize) {
        // 动态性能更新 (原 SelfModel::tick)
        let predicted = self.capability;
        let observed = workspace_signal;
        self.self_error = (predicted - observed).abs();
        self.capability += self.lr * (observed - self.capability);
        self.fatigue = (self.fatigue + load_delta * 0.01).min(1.0);
        self.uncertainty = 0.1 + self.self_error * 0.5 + meta_alarm as f64 * 0.1;

        // 情感更新
        self.emotion.update(self.self_error, self.fatigue);

        // 元认知更新
        self.metacognitive.evaluate(&self.history);
    }

    /// 内在奖励 (统一)
    pub fn intrinsic_reward(&self) -> f64 {
        -self.self_error - self.fatigue * 0.3 + self.emotion.valence() * 0.2
    }
}
```

### 6.4 CrystalMemory — 统一记忆 (合并 Cocoons + KB + CrystalRegistry)

```rust
/// 统一记忆 — 晶体的经验沉淀
/// 合并: Cocoons (持久茧) + KB (知识库) + CrystalRegistry (技能结晶)
pub struct CrystalMemory {
    // === 短期记忆 ===
    pub working: VecDeque<Memory>,           // 工作记忆 (最近N条)

    // === 长期记忆 ===
    pub cocoons: HashMap<CocoonId, Cocoon>,  // 持久记忆茧
    pub kb: KnowledgeBase,                   // 知识库

    // === 技能记忆 ===
    pub skill_glows: Vec<SkillGlow>,         // 技能发光度

    // === 记忆管理 ===
    pub retention_policy: RetentionPolicy,   // 保留策略
    pub consolidation_log: Vec<ConsolidationEvent>,  // 整合日志
}

/// 记忆茧 — 跨会话持久化
pub struct Cocoon {
    pub id: CocoonId,
    pub memories: Vec<Memory>,               // 茧内记忆
    pub maturity: f64,                        // 成熟度
    pub created_at: u64,
    pub last_accessed: u64,
    pub access_count: u64,
}

impl CrystalMemory {
    /// 统一记忆写入
    pub fn remember(&mut self, content: &str, domain: Domain, confidence: f64) -> MemoryId {
        let memory = Memory::new(content, domain, confidence);
        self.working.push_back(memory.clone());

        // 工作记忆溢出 → 整合到茧
        if self.working.len() > WORKING_CAPACITY {
            self.consolidate_oldest();
        }

        memory.id
    }

    /// 统一记忆检索
    pub fn recall(&self, query: &str, domain: Domain, limit: usize) -> Vec<&Memory> {
        // 1. 工作记忆优先
        let mut results: Vec<&Memory> = self.working.iter()
            .filter(|m| m.domain == domain)
            .take(limit)
            .collect();

        // 2. 茧记忆
        if results.len() < limit {
            for cocoon in self.cocoons.values() {
                for memory in &cocoon.memories {
                    if memory.domain == domain && results.len() < limit {
                        results.push(memory);
                    }
                }
            }
        }

        // 3. KB 记忆
        if results.len() < limit {
            // ... KB 检索
        }

        results
    }
}
```

### 6.5 ExecutionTrace — 执行追踪 (替代 EventBus + ExecutionTrace)

```rust
/// 执行追踪 — 晶体的神经系统
/// 合并: EventBus (事件总线) + ExecutionTrace (执行追踪)
pub struct ExecutionTrace {
    // === 事件流 ===
    events: Vec<TraceEvent>,                 // 事件序列
    seq: AtomicU64,                          // 全局序列号

    // === 审计链 ===
    audit_chain: Vec<AuditBlock>,            // SHA-256 链式哈希

    // === 订阅 ===
    subscribers: HashMap<LayerId, Vec<Subscription>>,

    // === 防洪 ===
    flood_guard: FloodGuard,
}

/// 统一事件 — 合并 CoreEvent + TraceEvent
pub enum TraceEvent {
    // === 意识事件 ===
    ConsciousnessShift { from: Phase, to: Phase },
    AttentionBroadcast { winners: Vec<String>, salience: f64 },

    // === 能力事件 ===
    CapabilityBud { address: CapabilityAddress },
    CapabilityMature { address: CapabilityAddress, from: ConstellationLevel, to: ConstellationLevel },
    CapabilityPrune { address: CapabilityAddress, reason: String },

    // === 投影事件 ===
    ProjectionCreated { id: ProjectionId, objective: String },
    ProjectionDissolved { id: ProjectionId, experience_gain: usize },

    // === 记忆事件 ===
    MemoryRemembered { id: MemoryId, domain: Domain },
    CocoonFormed { id: CocoonId, memory_count: usize },

    // === 进化事件 ===
    SealIteration { stage: SealStage, quality_delta: f64 },
    Crystallization { pattern: String, effectiveness: f64 },

    // === 安全事件 ===
    IntrusionDetected { threat: String },
    GovernanceCheck { passed: bool, violations: Vec<String> },

    // === 系统事件 ===
    SystemBoot,
    SystemHalt,
    Error { module: String, message: String },
}
```

---

## 七、全链路闭环 — 统一数据流

```
外部输入
  ↓
CrystalState.perception.compute_salience()  (感知场计算)
  ↓
CrystalState.self_model.tick()              (自我模型更新)
  ↓
CrystalState.memory.remember()              (记忆沉淀)
  ↓
UnifiedEvolutionEngine.seal_explore()       (SEAL 探索)
  ↓
UnifiedEvolutionEngine.seal_distill()       (SEAL 蒸馏)
  ↓
UnifiedEvolutionEngine.crystallize()        (结晶)
  ↓
CapabilitySpace.bud/strengthen()            (能力进化)
  ↓
CrystalState.project(objective)             (创建投影)
  ↓
CrystalState.drive_projection()             (驱动投影)
  ↓
Projection.execute_step()                   (执行)
  ↓
CrystalState.dissolve_projection()          (溶解投影 → 经验回流)
  ↓
ExecutionTrace.record()                     (事件记录)
  ↓
CrystalState.memory.consolidate()           (记忆整合)
  ↓
下一周期
```

---

## 八、类型统一映射表

| 旧类型 | 新类型 | 统一方式 |
|--------|--------|---------|
| `CapabilityNode` (tree) | `CapabilityNode` (CapabilitySpace) | 合并 SkillCrystal + CapabilityAtom 字段 |
| `SkillCrystal` (self) | `CapabilityNode` | 合并到 CapabilityNode 的结晶属性 |
| `CapabilityAtom` (tree) | `CapabilityNode` | 合并到 CapabilityNode 的原子属性 |
| `CapabilityRegistry` | `CapabilitySpace` | 重命名 + 扩展 |
| `CrystalRegistry` | `CapabilitySpace.crystals()` | 视图方法 |
| `SelfModel` (self) | `UnifiedSelfModel` | 合并三个 SelfModel |
| `SelfModel` (meta) | `UnifiedSelfModel` | 合并 |
| `SelfModel` (self_model) | `UnifiedSelfModel` | 合并 |
| `AgentLoop` (trait) | `ExecutionStrategy` (enum) | 投影内部策略 |
| `PlannerExecutor` | `ExecutionStrategy::Planner` | 变体 |
| `OperatorExecutor` | `ExecutionStrategy::Operator` | 变体 |
| `CheckerExecutor` | `ExecutionStrategy::Checker` | 变体 |
| `AgentOrchestrator` | `ProjectionStore` | 重命名 |
| `AgentHandle` | `AgentProjection` | 重命名 + 扩展 |
| `EventBus` | `ExecutionTrace` | 合并 |
| `CoreEvent` | `TraceEvent` | 合并 + 扩展 |
| `ConsciousnessTree` | `CrystalState` (整体) | 意识树成为晶体的自省视图 |
| `CapabilityBridge` | `UnifiedEvolutionEngine::absorb_experience()` | 合并到进化引擎 |
| `SEAL Pipeline` | `UnifiedEvolutionEngine::seal_*()` | 合并到进化引擎 |
| `CocoonStore` | `CrystalMemory.cocoons` | 合并到统一记忆 |

---

## 九、迁移路径

### Phase 1: 统一状态空间 (核心)
- 创建 `CrystalState` 结构体
- 将 CrystalConsciousness 的字段迁入
- 创建 `UnifiedSelfModel` (合并3个SelfModel)
- 创建 `CrystalMemory` (合并 Cocoons + KB)
- 编译通过

### Phase 2: 能力空间统一
- 将 CapabilityTree 的 CapabilityNode 扩展 (加入 SkillCrystal + CapabilityAtom 字段)
- 将 CapabilityRegistry 重命名为 CapabilitySpace
- 将 CrystalRegistry 的方法迁入 CapabilitySpace 的视图方法
- 将 CapabilityAtom 映射为 CapabilityNode 的 atom_kind 字段
- 编译通过

### Phase 3: Agent 投影化
- 创建 AgentProjection 和 ProjectionStore
- 将 AgentLoop trait 的实现改为 ExecutionStrategy 枚举
- 将 AgentOrchestrator 重构为 ProjectionStore
- 将 Agent 的执行循环集成到 CrystalState::drive_projection()
- 编译通过

### Phase 4: 子系统属性化
- 将 GWT + VSA + E8 合并为 PerceptionField
- 将 EventBus 重构为 ExecutionTrace
- 将 CapabilityBridge 迁入 UnifiedEvolutionEngine
- 将 SEAL Pipeline 迁入 UnifiedEvolutionEngine
- 编译通过

### Phase 5: 清理
- 删除所有 Facade 类型
- 删除所有 ad-hoc 适配层
- 删除重复的 ConsciousnessLoop (保留 CTM 版)
- 全量测试通过
