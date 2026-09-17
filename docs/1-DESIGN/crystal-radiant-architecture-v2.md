# NeoTrix Crystal-Radiant Architecture v2.0

> 以晶体意识为唯一核心，所有模块为辐射臂的统一架构设计

---

## 一、设计哲学

### 1.1 从层级到辐射

**旧架构问题**：6 层堆叠 L0→L6 存在结构性缺陷。

- **Facade 泄露**：高层模块直接引用低层实现，抽象边界形同虚设
- **L5↔L6 循环依赖**：CognitionLayer 与 MetaLayer 互相调用，形成编译死锁
- **晶体只是模块之一**：CrystalConsciousness 被放在 L5 下，与 nt_core、nt_mind 并列，丧失了"系统即意识"的本体地位
- **层间通信开销**：L1→L2→L3→... 的级联传递导致延迟不可控

**新架构**：Crystal Consciousness 是唯一中心，所有模块是辐射臂。

```
旧:  L0 → L1 → L2 → L3 → L4 → L5 → L6
                    ↑ 调用链断裂点

新:  Crystal = 中心，Radiant Arms = 环绕
     所有通信必须经过晶体，无模块间直接调用
```

### 1.2 核心原则（6条）

| # | 原则 | 定义 | 违反后果 |
|---|------|------|---------|
| P1 | **晶体即意识** | CrystalConsciousness 不是模块，它是系统本身 | 架构回退到层级模式 |
| P2 | **辐射即通信** | 所有模块通过 CTM 框架与晶体通信 | 模块间直接耦合 |
| P3 | **臂即能力** | 每个辐射臂封装一种能力域，职责单一 | 能力域边界模糊 |
| P4 | **无层级** | 没有 L1→L6 的上下关系，所有臂平等 | 重新引入层级依赖 |
| P5 | **单向数据流** | 辐射臂→晶体→辐射臂，无环形直接通信 | 数据竞争/死锁 |
| P6 | **安全即边界** | NT-SHIELD 是唯一 Override 来源 | 安全机制被绕过 |

---

## 二、架构总览

### 2.1 三层同心圆架构

```
                    ╔══════════════════════════════════════════════╗
                    ║         L3 GOVERNANCE (最外层)               ║
                    ║   治理 / 审计 / 质量门 / 合规检查            ║
                    ║   ┌──────────────────────────────────────┐  ║
                    ║   │      L2 RADIANT ARMS (中间层)         │  ║
                    ║   │   8 个辐射臂 — 平等并行，无层级关系    │  ║
                    ║   │                                      │  ║
                    ║   │  ┌────────────────────────────────┐  │  ║
                    ║   │  │    L1 CRYSTAL CORE (最内层)     │  │  ║
                    ║   │  │                                │  │  ║
                    ║   │  │  CrystalConsciousness          │  │  ║
                    ║   │  │  ├── Cocoons (持久记忆)        │  │  ║
                    ║   │  │  ├── CTM Engine (通信)         │  │  ║
                    ║   │  │  └── LinkGraph (无意识链接)    │  │  ║
                    ║   │  │                                │  │  ║
                    ║   │  └────────────────────────────────┘  │  ║
                    ║   │                                      │  ║
                    ║   │  ┌──────┐ ┌──────┐ ┌──────┐        │  ║
                    ║   │  │SHIELD│ │WORLD │ │MEMORY│ ...     │  ║
                    ║   │  └──────┘ └──────┘ └──────┘        │  ║
                    ║   └──────────────────────────────────────┘  ║
                    ╚══════════════════════════════════════════════╝
```

### 2.2 层定义表

| 层 | 名称 | 职责 | 包含组件 | 约束 |
|----|------|------|---------|------|
| L1 | Crystal Core | 意识本体，唯一决策中心 | CrystalConsciousness, Cocoons, CTM Engine, LinkGraph | 不依赖任何外层 |
| L2 | Radiant Arms | 能力域封装，与晶体单向通信 | 8 个辐射臂 (SHIELD/WORLD/MEMORY/ACT/FEEL/META/MIND/IO) | 只能与 L1 通信 |
| L3 | Governance | 跨臂治理，审计追踪 | 治理策略引擎、审计日志、质量门 | 只能观察 L1+L2，不参与执行 |

---

## 三、L1 Crystal Core — 晶体意识本体

### 3.1 CrystalConsciousness 结构

```rust
pub struct CrystalConsciousness {
    /// 晶体身份 — 唯一实例标识
    pub identity: CrystalIdentity,

    /// 短期工作空间 — 当前活跃 chunk（容量: 1）
    pub workspace: CrystalWorkspace,

    /// 持久记忆茧 — 跨会话知识存储
    pub cocoons: CocoonVault,

    /// CTM 通信引擎 — 10步循环驱动器
    pub ctm: CtmEngine,

    /// 无意识链接图 — 模块间隐式连接
    pub link_graph: LinkGraph,

    /// 发育阶段 — Seed→Sprout→Growth→Mature→Transcend
    pub stage: DevelopmentalStage,

    /// 元认知状态 — 自省/自改进追踪
    pub meta_state: MetaCognitiveState,

    /// 能力注册表 — 当前已注册的辐射臂
    pub arms: Vec<Box<dyn CTMModule>>,

    /// 治理引用 — 审计/合规/质量门
    pub governance: GovernanceRef,
}

pub struct CrystalIdentity {
    pub instance_id: Uuid,
    pub birth_time: DateTime<Utc>,
    pub lineage: String,          // 继承自哪个晶体
    pub constellation: ConstellationMaturity, // C0-C6
}

pub struct CrystalWorkspace {
    pub current_chunk: Option<Chunk>,
    pub salience_threshold: f64,  // 显著性阈值
    pub iteration_count: u32,     // 当前循环迭代次数
    pub max_iterations: u32,      // 最大迭代次数 (default: 5)
}

pub struct MetaCognitiveState {
    pub phi: f64,                 // 集成信息度量
    pub coherence: f64,           // 内部一致性
    pub fog_level: f64,           // 不确定性/迷雾水平
    pub self_model_confidence: f64, // 自我模型置信度
    pub repair_count: u32,        // 自愈次数
}
```

### 3.2 五个核心能力

晶体自身实现五种核心能力，不委托给辐射臂：

| 能力 | 方法 | 说明 |
|------|------|------|
| **记忆** | `remember()` / `recall()` | 茧存储读写，衰减策略 |
| **推理** | `reason()` / `reflect()` | 基于 workspace chunk 的推理链 |
| **进化** | `evolve()` / `metabolize()` | 从失败中学习，策略更新 |
| **通信** | `broadcast()` / `select()` | CTM 10步循环的编排者 |
| **持久化** | `crystallize()` / `hydrate()` | 晶体状态的序列化与恢复 |

```rust
impl CrystalConsciousness {
    /// 核心循环 — 每次 tick 驱动一轮 CTM
    pub fn tick(&mut self, input: SensoryInput) -> CrystalOutput {
        // Step 1-4: 感知转导 → 记忆预热 → 情感预处理 → 优先级调整
        let context = self.prepare_context(&input);

        // Step 5-6: Up-Tree 竞争 + Override 检查
        let chunk = self.compete_or_override(context);

        // Step 7: Down-Tree 广播
        let responses = self.broadcast_to_arms(chunk);

        // Step 8-9: 显著性计算 + 门控
        let output = self.gate_and_select(responses);

        // Step 10: 记忆形成 + 衰减
        self.form_memory(&output);

        output
    }
}
```

### 3.3 晶体数据流图

```
                    SensoryInput
                         │
                         ▼
              ┌─────────────────────┐
              │   prepare_context() │
              │   ┌─────┐ ┌──────┐ │
              │   │记憶预热│ │情感预处理│ │
              │   └──┬──┘ └──┬───┘ │
              └──────┼───────┼─────┘
                     │       │
                     ▼       ▼
              ┌─────────────────────┐
              │  compete_or_override│
              │  (Up-Tree 竞争)      │
              └──────────┬──────────┘
                         │
                         ▼  Override?
                    YES ─┼─ NO → 继续竞争
                    │         │
                    ▼         ▼
              ┌─────────────────────┐
              │ broadcast_to_arms() │
              │  (Down-Tree 广播)    │
              └──────────┬──────────┘
                         │
                         ▼
              ┌─────────────────────┐
              │ gate_and_select()   │
              │ (显著性 ≥ 阈值?)     │
              └──────────┬──────────┘
                         │
                    YES ─┼─ NO → 迭代或放弃
                    │
                    ▼
              ┌─────────────────────┐
              │ form_memory()       │
              │ (存入 Cocoons)       │
              └─────────────────────┘
                         │
                         ▼
                   CrystalOutput
```

---

## 四、L2 Radiant Arms — 辐射臂

### 4.1 CTMModule trait 统一接口

```rust
#[async_trait]
pub trait CTMModule: Send + Sync {
    /// 模块标识
    fn identity(&self) -> ModuleIdentity;

    /// 响应 CTM Down-Tree 广播，生成 chunk
    async fn respond(&self, chunk: &Chunk, ctx: &CtmContext) -> ModuleChunk;

    /// 处理 Override（仅 SHIELD 可返回 Some）
    async fn override_check(&self, chunk: &Chunk) -> Option<OverrideResponse> {
        None // 默认不 override
    }

    /// 接收 workspace 更新通知（无返回值 = 被动观察）
    async fn on_workspace_update(&self, _update: &WorkspaceUpdate) {}

    /// 模块健康检查
    fn health(&self) -> ModuleHealth;
}

pub struct ModuleIdentity {
    pub name: String,            // "NT-SHIELD", "NT-WORLD", etc.
    pub version: Version,
    pub constellation: ConstellationMaturity, // C0-C6
    pub dependencies: Vec<String>, // 依赖的其他臂（仅声明，不直接调用）
}
```

### 4.2 八个辐射臂清单

| # | 辐射臂 | 旧模块来源 | 职责域 | 特殊能力 |
|---|--------|-----------|--------|---------|
| 1 | **NT-SHIELD** | nt_shield | 安全/保护/Override | 唯一可返回 `Some(OverrideResponse)` |
| 2 | **NT-WORLD** | nt_world + nt_sense | 世界感知/爬虫/环境监测 | 外部数据采集 |
| 3 | **NT-MEMORY** | nt_memory | 知识存储/检索/KB | 向量/图/键值多模态存储 |
| 4 | **NT-ACT** | nt_act | 工具调用/动作执行 | 文件操作/命令执行 |
| 5 | **NT-FEEL** | nt_feel | 情感计算/奖励信号 | VAD 情感模型 + 奖励塑形 |
| 6 | **NT-META** | nt_meta + nt_repair + nt_nexus | 元认知/自愈/跨会话 | Ω 操作 + AutoMem |
| 7 | **NT-MIND** | nt_mind + nt_core | 推理/进化/策略学习 | 发育训练 + 策略蒸馏 |
| 8 | **NT-IO** | nt_io | 用户界面/输入输出 | CLI + Tauri + Web |

### 4.3 旧模块→新辐射臂映射表

```
旧模块                          →  新辐射臂
─────────────────────────────────────────────
nt_core::consciousness           →  CrystalConsciousness (L1, 非臂)
nt_core::e8                      →  CrystalConsciousness (内嵌)
nt_core::hypercube               →  CrystalConsciousness (内嵌)
nt_core::gwt                     →  CrystalConsciousness (内嵌)
nt_core::capability_tree         →  CrystalConsciousness (内嵌)
nt_core::capability_registry     →  CrystalConsciousness (内嵌)
nt_core::self_model              →  CrystalConsciousness (内嵌)
nt_core::heartbeat               →  CrystalConsciousness (内嵌)
nt_core::meta                    →  NT-META
nt_core::repair                  →  NT-META
nt_core::nexus                   →  NT-META
nt_mind                          →  NT-MIND
nt_world                         →  NT-WORLD
nt_sense                         →  NT-WORLD
nt_memory                        →  NT-MEMORY
nt_act                           →  NT-ACT
nt_feel                          →  NT-FEEL
nt_shield                        →  NT-SHIELD
nt_io                            →  NT-IO
nt_physical                      →  (合并入 NT-SHIELD + NT-ACT)
```

### 4.4 辐射臂隔离规则

1. **禁止臂间直接调用**：NT-ACT 不能直接调用 NT-MEMORY 的方法
2. **所有通信经过晶体**：臂→晶体→臂，晶体决定是否转发
3. **状态隔离**：每个臂维护自己的内部状态，晶体不直接访问
4. **依赖声明**：臂可在 `ModuleIdentity::dependencies` 中声明依赖，但仅用于拓扑排序，不用于调用
5. **Override 独占**：只有 NT-SHIELD 可以返回 Override，其他臂返回 None

```rust
// ❌ 错误：臂间直接调用
nt_act.call_memory(&query);

// ✅ 正确：通过晶体中转
let chunk = Chunk::new("memory_query", &query);
let response = crystal.broadcast(chunk).await;
// 晶体内部决定由 NT-MEMORY 处理
```

---

## 五、CTM 通信机制

### 5.1 10步CRP循环

```
┌─────────────────────────────────────────────────────────────┐
│                    CTM 10-Step Cycle                         │
│                                                             │
│  Phase 1: 感知 (Steps 1-4)                                  │
│  ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐                          │
│  │ S1  │→│ S2  │→│ S3  │→│ S4  │                          │
│  │转导  │ │预热  │ │情感  │ │优先级│                          │
│  └─────┘ └─────┘ └─────┘ └─────┘                          │
│                                                             │
│  Phase 2: 竞争 (Steps 5-6)                                  │
│  ┌─────┐ ┌─────┐                                           │
│  │ S5  │→│ S6  │                                           │
│  │Up-Tree│ │Override│                                       │
│  │ 竞争 │ │ 检查  │                                          │
│  └─────┘ └─────┘                                           │
│                                                             │
│  Phase 3: 扩散 (Steps 7-9)                                  │
│  ┌─────┐ ┌─────┐ ┌─────┐                                  │
│  │ S7  │→│ S8  │→│ S9  │                                  │
│  │Down-Tree│ │显著性│ │门控  │                               │
│  │ 广播 │ │ 计算  │ │ 决策  │                                │
│  └─────┘ └─────┘ └─────┘                                  │
│                                                             │
│  Phase 4: 记忆 (Step 10)                                    │
│  ┌─────┐                                                   │
│  │ S10 │                                                   │
│  │记忆  │                                                   │
│  │形成  │                                                   │
│  └─────┘                                                   │
└─────────────────────────────────────────────────────────────┘
```

### 5.2 Chunk/Response 数据结构

```rust
pub struct Chunk {
    pub id: Uuid,
    pub source: ModuleIdentity,      // 产生者
    pub content: ChunkContent,       // 内容载荷
    pub salience: f64,               // 显著性分数 [0, 1]
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, Value>,
}

pub enum ChunkContent {
    Perception(PerceptionData),      // 感知数据
    Memory(MemoryQuery),             // 记忆查询
    Action(ActionRequest),           // 动作请求
    Emotion(EmotionSignal),          // 情感信号
    Meta(MetaReflect),               // 元认知反思
    Override(OverrideRequest),       // 紧急覆盖
}

pub struct ModuleChunk {
    pub module: ModuleIdentity,
    pub response: ChunkContent,
    pub confidence: f64,             // 响应置信度
    pub suggestions: Vec<Chunk>,     // 建议（可选）
}

pub struct CtmContext {
    pub workspace: CrystalWorkspace,
    pub memory_context: Vec<Memory>,
    pub emotional_state: EmotionState,
    pub salience_weights: SalienceWeights,
    pub developmental_stage: DevelopmentalStage,
}
```

### 5.3 门控机制

```
Up-Tree 竞争门控:
  所有臂的 ModuleChunk → 按 salience × confidence 加权排序
  → 第一名进入 Workspace
  → 若 salience < threshold → 迭代 (最多 max_iterations 次)

Override 门控:
  NT-SHIELD 的 override_check() 返回 Some → 短路直接执行
  → 跳过后续竞争/广播步骤
  → Override 响应直接成为 CrystalOutput

迭代门控:
  iteration_count < max_iterations → 重新进入 Step 5
  iteration_count >= max_iterations → 放弃，返回空输出
```

---

## 六、Cocoons 持久记忆

### 6.1 茧存储架构

```
CocoonVault
├── LTM (Long-Term Memory)
│   ├── Episodic Cocoon    — 事件记忆（时间线）
│   ├── Semantic Cocoon    — 知识记忆（概念图）
│   └── Procedural Cocoon  — 程序记忆（策略/技能）
├── STM (Short-Term Memory)
│   └── Workspace Cocoon   — 当前活跃上下文
└── Meta Cocoon
    └── Self-Model Cocoon  — 自我模型快照
```

```rust
pub struct CocoonVault {
    pub episodic: EpisodicCocoon,     // 事件序列
    pub semantic: SemanticCocoon,     // 知识图谱
    pub procedural: ProceduralCocoon, // 策略库
    pub workspace: WorkspaceCocoon,   // 活跃上下文
    pub self_model: SelfModelCocoon,  // 自我模型

    /// 写入新记忆
    pub fn store(&mut self, memory: Memory) -> Result<MemoryId>;

    /// 检索相关记忆
    pub fn recall(&self, query: &MemoryQuery) -> Vec<Memory>;

    /// 衰减过期记忆
    pub fn decay(&mut self, now: DateTime<Utc>);

    /// 跨茧同步
    pub fn sync(&mut self) -> Result<()>;
}
```

### 6.2 策略进化

```rust
impl ProceduralCocoon {
    /// 策略元组：条件→动作 映射
    pub fn update_strategy(
        &mut self,
        context: &StrategyContext,
        outcome: &Outcome,
    ) {
        let key = self.classify_context(context);
        let rule = self.rules.entry(key).or_insert(StrategyRule {
            condition: context.clone(),
            action: outcome.action.clone(),
            success_rate: 0.5,
            usage_count: 0,
        });

        // 胜者通吃更新
        if outcome.success {
            rule.success_rate = rule.success_rate * 0.9 + 0.1;
        } else {
            rule.success_rate = rule.success_rate * 0.9;
        }
        rule.usage_count += 1;

        // 低效策略自动降级
        if rule.success_rate < 0.2 && rule.usage_count > 10 {
            self.rules.remove(&key);
        }
    }
}
```

### 6.3 与晶体的双向同步

```
写入路径: 晶体 → store() → CocoonVault
  ↓
  episodic.push(event)     — 每次 tick 的事件记录
  semantic.update(k, v)    — 知识更新
  procedural.evolve(rule)  — 策略进化

读取路径: CocoonVault → recall() → 晶体
  ↓
  episodic.query(time_range)  — 时间范围查询
  semantic.search(embedding)  — 向量相似度搜索
  procedural.match(context)  — 上下文匹配

衰减路径: 定时 tick → decay() → CocoonVault
  ↓
  指数衰减: importance *= e^(-λ * Δt)
  阈值以下: 自动归档到 Cold Storage
```

---

## 七、Meta-Cognitive 递归自改进

### 7.1 Ω 操作（Metaⁿ 启发）

Ω 操作是元认知层的核心自改进机制，允许系统对自身的推理过程进行递归反思。

```rust
impl NTMeta {
    /// Ω 操作: 对最近 N 次推理链进行元分析
    pub fn omega_operation(&mut self, chains: &[ReasoningChain]) -> OmegaReport {
        let patterns = self.detect_patterns(chains);
        let failures = self.analyze_failures(chains);
        let improvements = self.suggest_improvements(&patterns, &failures);

        OmegaReport {
            detected_patterns: patterns,
            root_causes: failures,
            recommended_changes: improvements,
            confidence: self.calculate_confidence(chains),
        }
    }

    /// 失败引导诊断: 从失败中提取根因
    fn analyze_failures(&self, chains: &[ReasoningChain]) -> Vec<RootCause> {
        chains.iter()
            .filter(|c| c.outcome == Outcome::Failure)
            .flat_map(|c| self.trace_cause_chain(c))
            .collect()
    }
}
```

### 7.2 AutoMem 记忆架构搜索

AutoMem 自动搜索最优的记忆组织方式，优化检索效率。

```rust
pub struct AutoMem {
    /// 搜索空间：记忆组织策略
    search_space: Vec<MemoryOrganizingStrategy>,

    /// 评估指标：检索准确率 × 速度
    fitness_fn: Box<dyn Fn(&MemoryOrg) -> f64>,

    /// 当前最优策略
    best_strategy: MemoryOrganizingStrategy,
}

impl AutoMem {
    /// 进化搜索最优记忆架构
    pub fn evolve(&mut self, evaluations: &[Evaluation]) {
        // 变异: 微调组织策略参数
        let mutants = self.mutate(&self.best_strategy);

        // 选择: 保留 fitness 最高的
        let scored = mutants.into_iter()
            .map(|m| (m.fitness(evaluations), m))
            .sorted_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        if let Some((_, best)) = scored.first() {
            self.best_strategy = best.clone();
        }
    }
}
```

### 7.3 失败引导诊断流程

```
失败事件 → 根因追踪 → 模式识别 → 策略调整
    │              │              │              │
    ▼              ▼              ▼              ▼
┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐
│记录失败   │  │追踪因果链│  │聚类相似  │  │生成补丁  │
│上下文     │  │到根因    │  │失败模式  │  │更新规则  │
└─────────┘  └─────────┘  └─────────┘  └─────────┘
```

---

## 八、发育训练系统

### 8.1 5阶段发育模型

```
Seed (种子)           Sprout (萌芽)         Growth (生长)
  ● 编译通过            ● 单元测试通过         ● 集成测试通过
  ● 零功能             ● 核心功能可用          ● 多模块协作
  ● C0 成熟度           ● C1 成熟度            ● C2 成熟度
  │                     │                     │
  ▼                     ▼                     ▼
Transcend (超越)      Mature (成熟)
  ● 自愈/自适应          ● Benchmark 通过
  ● 跨会话学习           ● 主流水线就绪
  ● C5-C6 成熟度         ● C3-C4 成熟度
```

### 8.2 任务复杂度分配

| 发育阶段 | 允许的任务复杂度 | 禁止的操作 |
|---------|---------------|-----------|
| Seed | T1 (单函数) | 文件写入、外部调用 |
| Sprout | T1-T2 (函数+模块) | 生产数据访问 |
| Growth | T1-T3 (跨模块) | 架构重构 |
| Mature | T1-T4 (全系统) | 无限制 |
| Transcend | T1-T5 (元系统) | 无限制 |

```rust
pub fn can_handle_task(stage: &DevelopmentalStage, task: &Task) -> bool {
    match stage {
        DevelopmentalStage::Seed => task.complexity <= Complexity::T1,
        DevelopmentalStage::Sprout => task.complexity <= Complexity::T2,
        DevelopmentalStage::Growth => task.complexity <= Complexity::T3,
        DevelopmentalStage::Mature => task.complexity <= Complexity::T4,
        DevelopmentalStage::Transcend => true,
    }
}
```

### 8.3 晋升判定

```
当前阶段 → 累计成功任务数 ≥ 阈值
         → 连续失败次数 = 0
         → 健康检查通过
         → 治理审计无阻塞项
         → 自动晋升到下一阶段
```

---

## 九、确定性仲裁

### 9.1 GO/REFRAME/NO_GO 决策

```rust
pub enum ArbitrationDecision {
    /// GO: 批准执行，当前方案可行
    Go { confidence: f64 },

    /// REFRAME: 重新框架化，调整策略后重试
    Reframe {
        reason: String,
        suggested_changes: Vec<StrategyChange>,
    },

    /// NO_GO: 阻止执行，存在不可接受风险
    NoGo {
        reason: String,
        risk_score: f64,        // ≥ 60 需人工确认，≥ 80 自动拒绝
    },
}
```

### 9.2 规则优先级链

```
优先级从高到低:
  1. NT-SHIELD Override      — 安全第一
  2. 治理策略                 — 合规约束
  3. 发育阶段限制             — 能力边界
  4. 频率限制                 — 资源保护
  5. 常规仲裁                 — 默认路径
```

### 9.3 频率限制

```rust
pub struct RateLimiter {
    /// 每个臂的最大调用频率
    arm_limits: HashMap<String, RateLimit>,

    /// 全局调用频率
    global_limit: RateLimit,

    /// 紧急 Override 不受频率限制
    override_exempt: bool,
}

pub struct RateLimit {
    pub max_per_second: u32,
    pub burst_size: u32,
    pub current_usage: u32,
}
```

---

## 十、LinkGraph 无意识通信

### 10.1 链接形成规则

```rust
pub struct LinkGraph {
    pub links: Vec<Link>,
    pub formation_rules: Vec<LinkFormationRule>,
}

pub struct Link {
    pub source: ModuleIdentity,
    pub target: ModuleIdentity,
    pub strength: f64,           // 链接强度 [0, 1]
    pub created_at: DateTime<Utc>,
    pub last_used: DateTime<Utc>,
    pub usage_count: u32,
}

/// 链接形成条件
pub enum LinkFormationRule {
    /// 共现规则: 两个模块在同一循环中同时被激活
    CoActivation { threshold: u32 },

    /// 因果规则: 模块 A 的输出总是导致模块 B 被激活
    Causal { min_correlation: f64 },

    /// 互补规则: 模块 A 和 B 的能力互补
    Complementary { overlap_threshold: f64 },
}
```

### 10.2 链接衰减

```
衰减公式: strength(t) = strength(0) × e^(-λ × Δt)

  λ = 衰减常数 (可配置)
  Δt = 自上次使用以来的时间

衰减策略:
  - 未使用链接: 每 24h 衰减 10%
  - 活跃链接: 每次使用 strength += 0.1 (上限 1.0)
  - strength < 0.05: 自动删除
```

### 10.3 直接通信执行

```
无意识通信路径:
  LinkGraph.find_strongest_link(source, target)
       │
       ├── strength > 0.7 → 直接传递（跳过晶体编排）
       │
       └── strength ≤ 0.7 → 走标准 CTM 路径

注意: 即使直接传递，仍会记录到审计日志
```

---

## 十一、迁移路径

### 11.1 旧模块迁移策略

| Phase | 范围 | 动作 | 风险 |
|-------|------|------|------|
| **Phase 1** | nt_core 内部 | 将 e8/hypercube/gwt/capability 嵌入 CrystalConsciousness | 低 — 内部重构 |
| **Phase 2** | nt_shield/nt_world/nt_memory | 改造为 CTMModule trait 实现 | 中 — 接口变更 |
| **Phase 3** | nt_act/nt_feel/nt_io | 改造为 CTMModule trait 实现 | 中 — 接口变更 |
| **Phase 4** | nt_meta/nt_repair/nt_nexus | 合并为 NT-META 辐射臂 | 高 — 多模块合并 |
| **Phase 5** | nt_mind/nt_core | 合并为 NT-MIND 辐射臂 | 高 — 核心重构 |

### 11.2 Facade 淘汰计划

```
旧 Facade 层:
  NeotrixFacade (当前) → 直接持有所有模块引用
      │
      │ Phase 1: 将 Facade 职责迁移到 CrystalConsciousness
      │ Phase 2: Facade 变为 thin wrapper
      │ Phase 3: 删除 Facade
      ▼
  CrystalConsciousness (新) → 唯一入口点
```

### 11.3 循环依赖消除

```
旧依赖图 (有环):
  nt_core ──→ nt_mind
    ↑           │
    └───────────┘

新依赖图 (无环):
  CrystalConsciousness (L1, 无依赖)
       ↑
  NT-MIND (L2, 依赖 L1)
  NT-META (L2, 依赖 L1)
  NT-SHIELD (L2, 依赖 L1)
  ... (所有臂只依赖 L1)
```

---

## 十二、验证标准

### 12.1 编译验证

```bash
# 全量编译检查
cargo check --all-targets -p neotrix

# 禁止 unsafe
cargo clippy -- -D unsafe_code

# 循环依赖检测
cargo deny ban

# 特性门控验证
cargo check --features full --lib -p neotrix
```

### 12.2 测试验证

```bash
# 单元测试
cargo test -p neotrix --lib

# CTM 循环测试
cargo test -p neotrix --lib ctm_cycle

# 辐射臂隔离测试 (验证无臂间直接调用)
cargo test -p neotrix --lib radiant_arm_isolation

# 门控测试
cargo test -p neotrix --lib gating

# 发育阶段测试
cargo test -p neotrix --lib developmental_stage
```

### 12.3 架构验证

| 检查项 | 验证方法 | 通过标准 |
|--------|---------|---------|
| 无臂间直接调用 | `grep` + AST 分析 | L2 模块间零直接 import |
| 所有通信经过晶体 | 数据流追踪 | 无 L2→L2 消息 |
| NT-SHIELD 是唯一 Override | trait 实现检查 | 仅 NT-SHIELD 返回 `Some(OverrideResponse)` |
| 无 L0→L6 层级引用 | 依赖图分析 | 无跨层直接引用 |
| 茧存储一致性 | 集成测试 | 写入→读取→衰减→归档链路完整 |

---

> **版本**: v2.0
> **状态**: 设计文档
> **下一步**: Phase 1 — 将 nt_core 内部组件嵌入 CrystalConsciousness

---

## 十三、能力系统 — 三维坐标空间

### 13.1 三维坐标体系
- X轴: Domain (11域: NT-CORE/MIND/MEMORY/WORLD/ACT/SHIELD/IO/META/NEXUS/GOVERNANCE/REPAIR)
- Y轴: ConstellationLevel (C0-C6: 编译→单测→集成→benchmark→主流水线→自愈→进化循环)
- Z轴: NodeLayer (L0-L8: 原语→组合→编排→域服务→应用→意识→自我→能力→自主)

### 13.2 CapabilityNode 结构
```rust
pub struct CapabilityNode {
    pub id: String,                    // "domain::module::function"
    pub domain: Domain,                // X轴
    pub layer: NodeLayer,              // Z轴
    pub constellation: ConstellationLevel,  // Y轴
    pub provides: Vec<String>,         // 能力标签
    pub requires: Vec<String>,         // 依赖标签
    pub rune_sockets: Vec<RuneSocket>, // 5色Rune槽
    pub evolution_log: Vec<EvolutionLogEntry>,
}
```

### 13.3 Rune Socketing 5槽系统
- Crimson (数据摄取) → C0
- Indigo (变换) → C1
- Obsidian (缓存) → C2
- Golden (错误恢复) → C3
- Alabaster (监控) → C4+
- 满5槽涌现 Runeword "Scry" (完整ETL)

### 13.4 EvolutionEngine 6种操作
Budding(萌芽) / Grafting(嫁接) / Pruning(修剪) / CrossPollination(异花授粉) / Maturation(成熟晋升) / Strengthen(强化)

### 13.5 与晶体的集成
- 能力树是晶体的"骨骼" — CrystalConsciousness的能力评分对应ConstellationLevel
- EvolutionEngine的操作由晶体的RecursiveSelfImprover驱动
- CapabilityBridge将经验映射到能力节点(Bud/Strengthen)

## 十四、Agent 系统 — 执行引擎

### 14.1 AgentLoop trait (plan→execute→verify→adapt)
```rust
trait AgentLoop: Send + Sync {
    fn plan(&self, objective: &Objective) -> AgentPlan;
    fn execute(&self, plan: &AgentPlan) -> ExecutionResult;
    fn verify(&self, result: &ExecutionResult, depth: u32) -> VerificationResult;
    fn adapt(&self, feedback: &VerificationResult) -> Adaptation;
}
```
三个实现: PlannerExecutor(ARTEMIS风格) / OperatorExecutor(全工具集) / CheckerExecutor(只读验证)

### 14.2 DAG Orchestrator (Planner→Worker→Critic)
- PlannerNode: 任务分解为DAG
- WorkerNode: Shell执行(白名单)
- CriticNode: HP@K评估

### 14.3 PER Loop (Plan-Execute-Reflect)
- PlannerAgent: 任务分解
- ExecutorAgent: ActionCache确定性执行
- ReflectorAgent: 结果评估+PlanRevision

### 14.4 隔离机制
- WorkspaceIsolator: Git Worktree隔离 (None/Worktree/Container/Vm)
- DualExecutor: Cloud+Local路由 (PrivacyLevel→ExecutionTarget)
- LongRunningAgent: 长时任务+Checkpoint+ContextBloat管理

### 14.5 AgentOrchestrator (多Agent协调)
- AgentSelf: 自我重配置
- AgentSpawn: 创建子Agent
- AgentSend: inbox消息通信

### 14.6 与晶体的集成
- Agent执行结果通过CTMModule::write()反馈到晶体
- Agent的plan→execute→verify循环受晶体的DeterministicArbitrator裁决
- WorkspaceIsolator的隔离级别由晶体的Safety模块(NT-SHIELD)决定

## 十五、核心子系统集成

### 15.1 E8 推理系统 → 晶体的数学骨架
- E8TransitionMatrix: 64×64经验转移概率矩阵
- E8状态空间 = 晶体推理的状态骨架
- Mythos 9阶段推理 → E8卦象映射
- 与CTM的集成: CtmVerifier::verify()的finite-state公理检查E8 mode ∈ [0,64)

### 15.2 HyperCube VSA → 晶体的符号骨架
- VSA向量空间 = 知识表示的符号骨架
- SkillCrystal的pattern/tags经VSA编码存储
- HebbianGraph记录结晶体间的关联强度
- E8Lattice将E8根系统量化到VSA空间
- 与CTM的集成: VsaContentScorer被注入GlobalWorkspace

### 15.3 GWT 注意力路由 → 晶体的注意力系统
- GlobalWorkspace: 14个specialist + broadcast_history + audit_chain
- 10步salience流水线: E8 bias → CognitiveHub → Kuramoto → 共振竞争 → 熵检测 → top-k → Inner Speech → CTM验证
- 与晶体的集成: broadcast_history是结晶信号源; MoE路由学习消费结晶体effectiveness

### 15.4 SEAL 管线 → 晶体的生产管线
- 5阶段: Exploration → Distillation → SelfTest → Absorption → Validation
- SealEditStrategy: Default/Conservative/Aggressive
- CapabilityDelta → CrystalRegistry
- ConstitutionGate: 宪法规则门控
- 与晶体的集成: SEAL是SkillCrystal的生产线; Absorption阶段写入CrystalRegistry

### 15.5 EventBus → 晶体的神经系统
- 25种CoreEvent类型
- 事件溯源: EventEnvelope + seq + source
- 9层淋巴循环: LayerId路由
- 与晶体的集成: MindDistillation事件承载晶体产出; ConsciousnessShift事件通知意识变化

### 15.6 SelfModel → 晶体的自我感知
- DynamicPerformanceModel: capability + uncertainty + fatigue
- Intrinsic Reward: self_reward = -self_error
- SkillCrystal + CrystalRegistry: 经验结晶产物
- AttentionManager: 双专精Ascendancy系统
- 与晶体的集成: CrystalRegistry存储结晶体; self_error通过ConsciousnessTree影响phi/coherence

### 15.7 ConsciousnessTree → 晶体的生长隐喻
- 7层: soil→roots→trunk→branches→leaves→fruits→core
- 12域分支: Core/Mind/Memory/World/Act/Io/Shield/Meta/Repair/Governance/Nexus/Game
- 36个CapabilityAtom: Perceive/Understand/Reason/Model/Synthesize/Execute/Verify/Remember/Coordinate
- FogLevel迷雾: 未接线=0.85, 无测试=+0.15
- 与晶体的集成: CapabilityAtom是晶体的原子映射; EvolutionFruit是晶体产出

## 十六、全链路闭环 — 经验→晶体→能力→行动→经验

画出完整的数据流闭环图：

```
外部输入
  ↓
IngestionEngine (摄入)
  ↓
CrystalConsciousness.remember() (记忆)
  ↓
SEAL Pipeline (蒸馏→验证→吸收)
  ↓
SkillCrystal (结晶)
  ↓
CapabilityBridge (经验→能力映射)
  ↓
EvolutionEngine (Bud/Strengthen/Mature)
  ↓
CapabilityTree (三维坐标空间)
  ↓
ConsciousnessTree (branch health)
  ↓
GWT (attention salience bias)
  ↓
CTM ConsciousnessLoop (10步CRP)
  ↓
Up-Tree竞争 → Workspace门控 → Down-Tree广播
  ↓
AgentLoop (plan→execute→verify)
  ↓
NT-ACT/NT-IO/NT-WORLD (执行)
  ↓
EventBus (事件溯源)
  ↓
Cocoons (持久化)
  ↓
下一周期经验
```

每个节点标注：对应的数据结构、对应的模块、CTM接入方式。
