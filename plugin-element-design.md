# NeoTrix Element Protocol — Plugin Architecture for ReasoningBrain

> **Fusion Design Document** — 2026-05-24
> Inspired by [Aglet](https://github.com/zyssyz123/agentkit) (zyssyz123/agentkit): a Python agent runtime where every capability is a swappable plugin Element.
> Adapted for Rust / NeoTrix's 83-module ReasoningBrain monolith.

---

## Table of Contents

1. [全景差距图 (Gap Matrix)](#1-全景差距图)
2. [优先级矩阵 (Priority Matrix)](#2-优先级矩阵)
3. [Element 协议设计 (Protocol Design)](#3-element-协议设计)
4. [集成点 (Integration Points)](#4-集成点)
5. [分阶段实现计划 (Phased Plan)](#5-分阶段实现计划)
6. [迁移策略 (Migration Strategy)](#6-迁移策略)
7. [Communication & Event Model](#7-communication--event-model)
8. [Persistence & Lifecycle](#8-persistence--lifecycle)
9. [ADR (Architecture Decision Records)](#9-adr)

---

## 1. 全景差距图

### 1.1 核心维度对比

| 维度 | 当前 (ReasoningBrain 巨石) | 目标 (Element 插件化) |
|------|---------------------------|---------------------|
| **模块注册** | 83 个 `pub mod` 硬编码在 `mod.rs`；编译期固定 | `ElementRegistry` 运行时注册；每个 Element 可独立加载/卸载 |
| **生命周期** | 无统一生命周期；`new()` → 手动调用各 `init_*()` 方法 | `init → start → stop → destroy` 四阶段契约 |
| **通信** | 跨模块通过直接方法调用；`SelfIteratingBrain` 持有一切引用的 God struct | `ElementBus`: 发布/订阅 + 命令通道；Element 之间只知总线，不知具体实现 |
| **依赖** | Cargo.toml 静态；所有 feature 编译时确定 | 每个 Element 声明 `depends_on()`；`ElementRegistry` 解析 DAG 后按序加载 |
| **状态管理** | 单一 `SelfIteratingBrain` struct 承载 ~35 个字段 | 每个 Element 持有独立状态；总线管理跨 Element 共享状态 |
| **发现** | N/A；无运行时发现 | `ElementRegistry::auto_discover()` 扫描已知路径/环境变量 |
| **版本** | Cargo.lock 统一版本 | 每个 Element 独立 `version()`；兼容性检查通过 `compatible_with()` |
| **扩展** | 需在 `mod.rs` 添加 `pub mod` + 在 `loop_impl.rs` 添加字段 + `new()` 初始化 | 第三方提供 `impl Element` 即可；零核心修改 |
| **测试隔离** | 所有测试在 `tests::` 模块；全局状态污染 | 每个 Element 独立测试；`ElementMock` 替代真实依赖 |
| **热加载** | 不支持 | `ElementRegistry::hot_swap()` 停用旧 → 加载新 → 验证 → 切换 |

### 1.2 当前巨石解剖

```
SelfIteratingBrain (loop_impl.rs:19, ~976 行)
├── ReasoningBrain          (brain_impl.rs:58, ~584 行)  ← 核心能力向量
│   ├── capability: CapabilityVector
│   ├── task_affinity: HashMap<TaskType, f64>
│   ├── custom_sources: HashMap<String, CapabilityVector>
│   └── source_access_tracker: SourceAccessTracker
├── reasoning_bank: ReasoningBank
├── cortex: CortexMemory
├── reasoning_engine: Option<ReasoningEngine>
├── attention_router: Option<AttentionRouter>
├── select_operator: Option<SelectableOperator>
├── group_manager: Option<MultiBrainManager>
├── pipeline: BrainPipeline
├── archive: ChangeArchive
├── champion: Option<BrainSnapshot>
├── stagnation: StagnationDetector
├── skills: SkillBridge
├── ... (35 字段总数)
└── 15+ `_scratchpad` 字段 (pipeline 阶段间状态传递)
```

**问题**:
- 35 字段直接耦合，无法独立测试任何子模块
- `new()` 重复初始化，拆装困难
- 新增功能 = 改 `loop_impl.rs` + `mod.rs` + 测试，摩擦大
- SEAL pipeline 通过 `_scratchpad` 字段传递状态 → 隐式契约，容易遗漏

### 1.3 Aglet 架构参考

[Aglet](https://github.com/zyssyz123/agentkit) (zyssyz123/agentkit) 提供二维插件化：

```
Runtime (canonical Loop)
  └── Element (9 protocols)           ← 协议层，不可扩展（但可第三方添加新 Element）
        └── Technique (无限实现)       ← 每个 Element 有多个 Technique，YAML 选择
              └── Plugin Runtime        ← 4 种: in-process / subprocess / HTTP / MCP
```

NeoTrix Adaptation:
- **Element** → Rust `trait Element` (等效 Aglet 协议)
- **Technique** → Rust trait 的具体实现 (等效 Aglet Technique)
- **Plugin Runtime** → 不适用 (Rust 编译期加载；不动态加载 `.so`)
- **Routing** → 借鉴 `all / first_match / parallel_merge` 用于多 Technique 协调
- **Context** → 借鉴 `Immutable AgentContext + ContextPatch` 事件溯源

---

## 2. 优先级矩阵

| Element | Impact | 复杂度 | 模块现状 | 优先级 |
|---------|--------|--------|---------|--------|
| **Capability Element** | 高 (核心 API) | 低 | `CapabilityVector` 已完全独立于 `core/capability.rs` | **P0** |
| **Memory Element** | 高 (ReasoningBank) | 低 | `ReasoningBank` 已完全独立于 `memory.rs`；含 4 种子类注入 | **P0** |
| **Skill Element** | 高 (SkillCrystallization) | 低 | `SkillBridge` + `CrystalRegistry` 已封装；仅 +20 行 Element 壳 | **P0** |
| **Goal Element** | 中 (GoalLoop) | 中 | `goal_loop/` 含 4 文件已模块化；需适配 ElementBus 事件 | P1 |
| **Thinking Element** | 中 (SiliconSelf) | 中 | `ThinkingBridge` 含 ~646 行；需拆分 monitor/archiver | P1 |
| **Monitoring Element** | 中 (CognitiveEvaluator+) | 低 | `StagnationDetector`, `CognitiveEvaluator` 已有；组合为单个 Element | P1 |
| **Planner Element** | 中 (PlanTemplate / GoalLoop) | 低 | `goal_loop/tracker.rs` 含部分规划逻辑 | P1 |
| **Network Element** | 低 (stealth-net) | 高 | 需新建；依赖 `agent_protocol` + UDP/mDNS | P2 |
| **Evolution Element** | 中 (SelfEvolver) | 中 | `self_evolver.rs` 已独立；需接入吸收管道 | P1 |
| **REPL/Ui Element** | 低 | 中 | 现有 `headless` REPL；拆分为独立 Element | P2 |

**排序依据**: `Priority = Impact × (1 / Complexity) × ModuleReadiness`

---

## 3. Element 协议设计

### 3.1 核心 Trait

```rust
/// Core trait for all pluggable Elements.
///
/// Each Element is a self-contained capability registered at runtime.
/// The ElementRegistry resolves the DAG of depends_on(), starts Elements
/// in order, and routes messages via ElementBus.
pub trait Element: Send + Sync + Debug {
    /// Unique identifier (e.g. "element.capability", "element.memory.skill")
    fn id(&self) -> &str;

    /// Human-readable name
    fn name(&self) -> &str;

    /// Semver string (e.g. "0.1.0")
    fn version(&self) -> &str;

    /// Classification for loading behavior
    fn element_type(&self) -> ElementType;

    // ── Lifecycle ──────────────────────────────────────────────

    /// Called after construction. Register event handlers, validate config.
    /// Do NOT start background work here.
    fn init(&mut self, bus: &ElementBus) -> Result<(), ElementError>;

    /// Called after all dependencies are initialized.
    /// Start background loops, timers, listeners.
    fn start(&mut self) -> Result<(), ElementError>;

    /// Called on shutdown or unload. Gracefully stop background work.
    fn stop(&mut self) -> Result<(), ElementError>;

    /// Final cleanup. Release resources, flush state.
    fn destroy(&mut self) -> Result<(), ElementError>;

    // ── Dependency Graph ───────────────────────────────────────

    /// IDs of Elements that must be loaded before this one
    fn depends_on(&self) -> Vec<&str> { vec![] }

    /// Capabilities this Element exposes to other Elements
    fn provides(&self) -> Vec<CapabilityAccess> { vec![] }

    /// Maximum state version this Element understands
    fn state_version(&self) -> u32 { 1 }

    /// Check if this Element's ABI is compatible with a given version
    fn compatible_with(&self, other_version: &str) -> bool { true }
}

/// Classification for startup ordering and UI grouping
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ElementType {
    /// Core: must always load; Capability, Memory
    Core,
    /// Feature: optional capability; Skill, Goal, Planner, Evolution
    Feature,
    /// Monitor: passive observers; CognitiveEvaluator, StagnationDetector
    Monitor,
    /// Network: external communication; stealth-net, agent protocol
    Network,
    /// UI: user interface; REPL, MCP tool server
    Ui,
}

/// Describes a capability point exposed by an Element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityAccess {
    pub name: &'static str,
    pub description: &'static str,
    /// Which operations are available on this capability
    pub operations: Vec<CapabilityOp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CapabilityOp {
    Query,      // read-only
    Command,    // mutate state
    Subscribe,  // event stream
    Provide,    // provides data to other Elements
}

#[derive(Debug)]
pub enum ElementError {
    InitFailed(String),
    StartFailed(String),
    StopFailed(String),
    DependencyNotMet(String),
    VersionMismatch { element: String, required: String, found: String },
    BusError(String),
    RuntimeError(String),
}

impl std::fmt::Display for ElementError { /* ... */ }
impl std::error::Error for ElementError {}
```

### 3.2 ElementRegistry

```rust
/// Registry that manages Element lifecycle, dependency resolution,
/// and provides access to loaded Elements by ID or trait.
pub struct ElementRegistry {
    elements: HashMap<String, Box<dyn Element>>,
    load_order: Vec<String>,
    bus: ElementBus,
    state: RegistryState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RegistryState {
    Constructed,
    Resolved,
    Started,
    Stopped,
}

impl ElementRegistry {
    /// Create a new empty registry with a fresh bus
    pub fn new() -> Self;

    /// Register an Element (does NOT init/start it)
    pub fn register(&mut self, element: Box<dyn Element>) -> Result<(), ElementError>;

    /// Register multiple Elements at once
    pub fn register_all(&mut self, elements: Vec<Box<dyn Element>>) -> Result<(), ElementError>;

    /// Resolve dependencies, compute topological order, call init() on each
    pub fn resolve_and_init(&mut self) -> Result<(), ElementError>;

    /// Start all Elements in dependency order
    pub fn start_all(&mut self) -> Result<(), ElementError>;

    /// Full bootstrap: register → resolve → init → start
    pub fn bootstrap(&mut self, elements: Vec<Box<dyn Element>>) -> Result<(), ElementError> {
        self.register_all(elements)?;
        self.resolve_and_init()?;
        self.start_all()
    }

    /// Graceful shutdown: stop all (reverse order) → destroy
    pub fn shutdown(&mut self) -> Result<(), ElementError>;

    /// Get a reference to an Element by ID (downcast to concrete type)
    pub fn get<T: Element>(&self, id: &str) -> Option<&T>;

    /// Get a mutable reference to an Element by ID
    pub fn get_mut<T: Element>(&mut self, id: &str) -> Option<&mut T>;

    /// Hot-swap: stop old → register new → init → start
    /// Old element's state is transferred via ElementBus snapshot
    pub fn hot_swap(&mut self, id: &str, new: Box<dyn Element>) -> Result<(), ElementError>;

    /// Auto-discover Elements from known paths / environment variables
    pub fn auto_discover(&mut self) -> Result<Vec<String>, ElementError>;

    /// List all registered Element IDs
    pub fn list(&self) -> Vec<&str>;

    /// Current registry state
    pub fn state(&self) -> RegistryState;
}
```

### 3.3 ElementBus (通信总线)

```rust
/// Communication backbone: publish/subscribe + command channel.
///
/// Elements NEVER hold direct references to other Elements.
/// All cross-Element communication goes through the Bus.
#[derive(Clone, Debug)]
pub struct ElementBus {
    tx: mpsc::Sender<BusMessage>,
    rx: Arc<Mutex<mpsc::Receiver<BusMessage>>>,
    subscriptions: Arc<Mutex<HashMap<EventKind, Vec<(ElementId, mpsc::Sender<EventPayload>)>>>>,
}

/// Unique identifier for an Element within the registry
pub type ElementId = String;

/// Kinds of events that can be published/subscribed
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventKind {
    /// Capability vector changed
    CapabilityUpdated,
    /// New memory stored in ReasoningBank
    MemoryStored,
    /// Goal status changed (pursuing → achieved / unmet)
    GoalStateChanged,
    /// Thinking trace completed
    TraceCompleted,
    /// System-level: registry state change
    RegistryStateChanged,
    /// Custom event (Element-defined)
    Custom(String),
}

/// A message on the bus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusMessage {
    pub source: ElementId,
    pub kind: EventKind,
    pub payload: EventPayload,
    pub timestamp: u64,
}

/// Payload of a bus event (type-erased, serializable)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EventPayload {
    Text(String),
    Number(f64),
    CapabilitySnapshot(CapabilityVector),
    MemorySnapshot(Box<ReasoningMemory>),
    GoalSnapshot(Box<GoalState>),
    TraceSnapshot(Box<ThinkingTrace>),
    Map(HashMap<String, serde_json::Value>),
    None,
}

impl ElementBus {
    pub fn new() -> Self;

    /// Publish an event to all subscribers
    pub fn publish(&self, source: ElementId, kind: EventKind, payload: EventPayload);

    /// Subscribe to an event kind, receiving on a channel
    pub fn subscribe(
        &self,
        subscriber: ElementId,
        kind: EventKind,
    ) -> mpsc::Receiver<EventPayload>;

    /// Send a command to a specific Element (point-to-point)
    pub fn send_command(
        &self,
        target: ElementId,
        command: &str,
        payload: EventPayload,
    ) -> Result<EventPayload, ElementError>;

    /// Create a scoped bus for testing (no real channels)
    #[cfg(test)]
    pub fn mock() -> Self;
}
```

### 3.4 生命周期状态机

```
                    register()
                        │
                        ▼
                  ┌──────────────┐
         ┌───────│  CONSTRUCTED  │
         │       └──────┬───────┘
         │              │ resolve_and_init()
         │              ▼
         │       ┌──────────────┐
         │       │  INITIALIZED  │
         │       └──────┬───────┘
         │              │ start()
         │              ▼
         │       ┌──────────────┐
         │       │   STARTED     │◄──────────────┐
         │       └──────┬───────┘                │
         │              │ stop()                  │ hot_swap()
         │              ▼                        │
         │       ┌──────────────┐                │
         │       │   STOPPED     │───────────────┘
         │       └──────┬───────┘
         │              │ destroy()
         │              ▼
         │       ┌──────────────┐
         └───────│   DESTROYED   │
                 └──────────────┘
```

### 3.5 ElementManifest (声明式注册)

```rust
/// Declarative Element manifest (alternative to manual registration).
/// Loaded from `element.toml` or in-memory config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub element_type: ElementType,
    pub depends_on: Vec<String>,
    pub provides: Vec<String>,
    pub config: HashMap<String, serde_json::Value>,
    pub enabled: bool,
}

impl ElementRegistry {
    /// Load Elements from manifests, constructing via a factory function
    pub fn load_from_manifest(
        &mut self,
        manifests: Vec<ElementManifest>,
        factory: &dyn Fn(&ElementManifest) -> Option<Box<dyn Element>>,
    ) -> Result<(), ElementError>;
}
```

### 3.6 Bootstrapping Usage

```rust
// ── Example: Creating a minimal configured brain ──

let mut registry = ElementRegistry::new();

registry.bootstrap(vec![
    Box::new(CapabilityElement::new()),
    Box::new(MemoryElement::new()),
    Box::new(SkillElement::new()),
    Box::new(GoalElement::new()),
    Box::new(MonitoringElement::new()),
])?;

// After bootstrap, all Elements are running.
// Access the Capability Element to absorb knowledge:
let cap = registry.get_mut::<CapabilityElement>("element.capability").unwrap();
cap.absorb(KnowledgeSource::HeroUI);

// The bus notifies subscribers automatically:
// MemoryElement → logs the absorption as a memory
// MonitoringElement → records the capability delta

// Shutdown:
registry.shutdown()?;
```

---

## 4. 集成点

### 4.1 Capability Element

| Property | Value |
|----------|-------|
| **ID** | `element.capability` |
| **Source file** | `reasoning_brain/self_iterating/brain_impl.rs` → **新文件**: `element/capability_element.rs` |
| **Moves from** | `ReasoningBrain` struct 中的 `capability: CapabilityVector`, `task_affinity`, `custom_sources`, `absorption_history`, `source_access_tracker` |
| **Leaves in place** | `AbsorbValidator` trait, `SelfIteration` trait, `EvaluationRecord` |
| **Integration** | 包装现有 `CapabilityVector`, `absorb()`, `apply_micro_edits()`, `generate_self_edit()` |
| **Bus events** | `publish(EventKind::CapabilityUpdated)` after each absorb/apply |
| **Migration** | 并行：Element 内部持有 `ReasoningBrain`，向外暴露 `CapabilityAccess { Query, Command }` |

```rust
pub struct CapabilityElement {
    brain: ReasoningBrain,       // 内部持有，外部不直接访问
    bus: Option<ElementBus>,
}

impl Element for CapabilityElement {
    fn id(&self) -> &str { "element.capability" }
    fn name(&self) -> &str { "Capability Vector Element" }

    fn init(&mut self, bus: &ElementBus) -> Result<(), ElementError> {
        self.bus = Some(bus.clone());
        Ok(())
    }

    fn provides(&self) -> Vec<CapabilityAccess> {
        vec![CapabilityAccess {
            name: "capability_vector",
            description: "Read/write access to the Brain's capability vector",
            operations: vec![CapabilityOp::Query, CapabilityOp::Command],
        }]
    }
}

impl CapabilityElement {
    pub fn absorb(&mut self, source: KnowledgeSource) {
        self.brain.absorb(source);
        if let Some(bus) = &self.bus {
            bus.publish(
                self.id().to_string(),
                EventKind::CapabilityUpdated,
                EventPayload::CapabilitySnapshot(self.brain.capability.clone()),
            );
        }
    }
}
```

### 4.2 Memory Element

| Property | Value |
|----------|-------|
| **ID** | `element.memory` |
| **Source file** | `reasoning_brain/memory.rs` → **新文件**: `element/memory_element.rs` |
| **Moves from** | `SelfIteratingBrain::reasoning_bank: ReasoningBank` |
| **Leaves in place** | `ReasoningMemory`, `MemoryTier`, `TemporalContext`, `MemoryLifecycle` 定义在 `core/memory.rs` |
| **Integration** | 包装 `ReasoningBank`；添加 `MemoryElement` 作为总线订阅者 |
| **Bus events** | `subscribe(EventKind::CapabilityUpdated)` → 自动存储为记忆<br>`publish(EventKind::MemoryStored)` after store |
| **Migration** | 并行：Element 内部持有 `ReasoningBank`，`SelfIteratingBrain` 通过 `registry.get()` 访问 |

```rust
pub struct MemoryElement {
    bank: ReasoningBank,
    bus: Option<ElementBus>,
}
```

### 4.3 Skill Element

| Property | Value |
|----------|-------|
| **ID** | `element.skill` |
| **Source file** | `reasoning_brain/skill_bridge.rs` → **新文件**: `element/skill_element.rs` |
| **Moves from** | `ThinkingBridge::skills: SkillBridge` |
| **Integration** | 包装 `SkillBridge` + `CrystalRegistry`；监听 `TraceCompleted` 事件自动结晶 |
| **Bus events** | `subscribe(EventKind::TraceCompleted)` → `extract_from_recent()` |
| **Migration** | 直接迁移：`SkillBridge` 已是独立模块，+Element 壳即可 |

### 4.4 Goal Element

| Property | Value |
|----------|-------|
| **ID** | `element.goal` |
| **Source file** | `reasoning_brain/goal_loop/` (4 文件) → **新文件**: `element/goal_element.rs` |
| **Moves from** | `SelfIteratingBrain` (当前松散耦合) |
| **Integration** | 包装 `GoalLoop`, `GoalTracker`, `GoalConfig`；通过 Bus 接收能力更新触发自动目标 |
| **Bus events** | `subscribe(EventKind::CapabilityUpdated)` → auto re-evaluate goals<br>`publish(EventKind::GoalStateChanged)` |
| **Migration** | 重构：将 `GoalLoop` 重构为无外部引用的独立模块；通过 `ElementBus` 获取 `CapabilityAccess` |

### 4.5 Thinking Element

| Property | Value |
|----------|-------|
| **ID** | `element.thinking` |
| **Source file** | `reasoning_brain/thinking_bridge.rs` → **新文件**: `element/thinking_element.rs` |
| **Moves from** | `ThinkingBridge` (当前由 `background_loop` 驱动) |
| **Integration** | 包装 `SiliconSelfModel` + `CognitiveEvaluator` + `IntrinsicMotivation` |
| **Bus events** | `subscribe(EventKind::MemoryStored)` → trigger reflection<br>`publish(EventKind::TraceCompleted)` |
| **Migration** | 拆分：`CognitiveEvaluator` 作为独立 `MonitoringElement` 子组件 |

### 4.6 Monitoring Element

| Property | Value |
|----------|-------|
| **ID** | `element.monitor` |
| **Source file** | 新文件：`element/monitor_element.rs` |
| **Moves from** | `StagnationDetector` (现有) + `CognitiveEvaluator` (从 Thinking 拆分) + 新 `Watchdog` |
| **Integration** | 纯被动观察者；订阅所有 `EventKind`，记录统计，触发告警 |
| **Bus events** | `subscribe("*")` → 分析模式，检测瓶颈和退化 |

### 4.7 Planner Element

| Property | Value |
|----------|-------|
| **ID** | `element.planner` |
| **Source file** | 新文件：`element/planner_element.rs` |
| **Moves from** | `goal_loop/` 中的规划逻辑 |
| **Integration** | 接收目标 → 分解子任务 → 通过 Bus 分派给其他 Element |
| **Bus events** | `subscribe(EventKind::GoalStateChanged)` → trigger planning |

### 4.8 Evolution Element

| Property | Value |
|----------|-------|
| **ID** | `element.evolution` |
| **Source file** | `reasoning_brain/self_evolver.rs` → **新文件**: `element/evolution_element.rs` |
| **Moves from** | `SelfEvolver` (现有) |
| **Integration** | 接收外部 URL/知识 → 调用吸收管道 → 触发 `CapabilityUpdated` |
| **Bus events** | 自定义 `EventKind::Custom("evolution.completed")` |

### 4.9 Network Element

| Property | Value |
|----------|-------|
| **ID** | `element.network` |
| **Source file** | 新文件：`element/network_element.rs` |
| **New** | 包装 `agent_protocol` 模块 (UDP 发现 + A2A 协议) |
| **Integration** | 接收远程 Element 的消息并桥接到本地 ElementBus |
| **Bus events** | 自定义 `EventKind::Custom("network.message")` |

---

## 5. 分阶段实现计划

### Phase 1: Core Infrastructure (1-2 sessions)

```
目标: Element trait + ElementRegistry + ElementBus + Capability Element + Memory Element
```

| Task | Effort | Dependencies |
|------|--------|-------------|
| 1.1 Define `Element` trait + `ElementType` + `CapabilityAccess` + `ElementError` in new crate `neotrix-element` | 1h | None |
| 1.2 Implement `ElementRegistry` + `ElementBus` + lifecycle state machine | 2h | 1.1 |
| 1.3 Implement `ElementManifest` + `load_from_manifest()` | 1h | 1.2 |
| 1.4 Create `CapabilityElement` wrapping `ReasoningBrain` | 1h | 1.1, existing `brain_impl.rs` |
| 1.5 Create `MemoryElement` wrapping `ReasoningBank` | 1h | 1.1, existing `memory.rs` |
| 1.6 Verify: Registry bootstraps bus → CapabilityElement publish → MemoryElement subscribe | 1h | 1.4, 1.5 |
| 1.7 Write `element/mod.rs` with module structure + re-exports | 0.5h | 1.1-1.6 |
| ✅ **Gate**: `cargo check --lib` zero errors + 5 integration tests | | |

**Files created**:
- `neotrix-core/src/neotrix/element/mod.rs`
- `neotrix-core/src/neotrix/element/core.rs` (trait definitions)
- `neotrix-core/src/neotrix/element/registry.rs`
- `neotrix-core/src/neotrix/element/bus.rs`
- `neotrix-core/src/neotrix/element/capability.rs`
- `neotrix-core/src/neotrix/element/memory.rs`
- `neotrix-core/src/neotrix/element/manifest.rs`

### Phase 2: Feature Elements (2-3 sessions)

```
目标: Skill Element + Goal Element + Evolution Element
```

| Task | Effort | Dependencies |
|------|--------|-------------|
| 2.1 Create `SkillElement` wrapping `SkillBridge` + `CrystalRegistry` | 1h | Phase 1 |
| 2.2 Refactor `GoalLoop` → bus-driven `GoalElement` | 3h | Phase 1 |
| 2.3 Create `EvolutionElement` wrapping `SelfEvolver` | 1h | Phase 1 |
| 2.4 Integration test: Skill crystalization triggered by Goal completion | 1h | 2.1, 2.2 |
| ✅ **Gate**: All Phase 1 + 2 cargo check + tests | | |

**Files created**:
- `neotrix-core/src/neotrix/element/skill.rs`
- `neotrix-core/src/neotrix/element/goal.rs`
- `neotrix-core/src/neotrix/element/evolution.rs`

### Phase 3: Thinking + Monitoring (2 sessions)

```
目标: Thinking Element + Monitoring Element
```

| Task | Effort | Dependencies |
|------|--------|-------------|
| 3.1 Split `CognitiveEvaluator` from `ThinkingBridge` → `MonitoringElement` | 1h | Phase 1 |
| 3.2 Create `ThinkingElement` wrapping `SiliconSelfModel` + `IntrinsicMotivation` | 2h | 3.1 |
| 3.3 Create `StagnationDetector` → wiring as `MonitorElement` sub-component | 1h | 3.1 |
| 3.4 Integration test: Thinking trace → Monitor alert → auto-repair | 1h | 3.2, 3.3 |
| ✅ **Gate**: cargo check + all existing thinking_bridge tests still pass | | |

**Files created**:
- `neotrix-core/src/neotrix/element/thinking.rs`
- `neotrix-core/src/neotrix/element/monitor.rs`

### Phase 4: Planner + Network (3 sessions)

```
目标: Planner Element + Network Element + full integration
```

| Task | Effort | Dependencies |
|------|--------|-------------|
| 4.1 Create `PlannerElement` extracting planning from `GoalLoop` | 2h | Phase 2 |
| 4.2 Create `NetworkElement` wrapping `agent_protocol` | 3h | Phase 1, external `agent_protocol` |
| 4.3 End-to-end test: All Elements loaded → task → planning → capability update → memory | 2h | 4.1, 4.2 |
| 4.4 Performance benchmark: bus latency, memory throughput | 1h | 4.3 |
| ✅ **Gate**: All Elements loadable via single `bootstrap()` call | | |

**Files created**:
- `neotrix-core/src/neotrix/element/planner.rs`
- `neotrix-core/src/neotrix/element/network.rs`

### Phase 5: Backward Compat Layer + Migration (1-2 sessions)

```
目标: 旧 API 通过 Element 代理，逐步弃用直接字段访问
```

| Task | Effort | Dependencies |
|------|--------|-------------|
| 5.1 Old `SelfIteratingBrain` delegates to `ElementRegistry` internally | 2h | Phase 1-4 |
| 5.2 Deprecate direct field access → `registry.get::<T>()` pattern | 1h | 5.1 |
| 5.3 Update `background_loop.rs` to drive via bus events | 1h | 5.1 |
| 5.4 Remove old `mod.rs` direct module deps where Element provides the feature | 1h | 5.2 |
| ✅ **Gate**: All existing tests pass with identical behavior | | |

---

## 6. 迁移策略

### 6.1 Parallel Run (Phase 1-3)

```
SelfIteratingBrain (monolith)
    └── holds: ReasoningBrain, ReasoningBank, SkillBridge ...
                ↑                         ↑
                │  (wraps existing)        │  (wraps existing)
    ElementRegistry ─── CapabilityElement ─┘
                    └── MemoryElement ──────┘
                    └── SkillElement ───────┘
                    └── GoalElement ────────┘
```

- Monolith 继续正常运行
- Element 层包装现有模块，增加总线通信
- 新代码优先使用 Element API
- 零侵入性：不需要修改现有代码

### 6.2 Gradual Migration (Phase 3-4)

逐步将 monolith 的依赖从"直接字段访问"迁移到"ElementBus 事件驱动"：

```
old: self.reasoning_bank.store(memory);
new: let mem_el = registry.get::<MemoryElement>("element.memory").unwrap();
     mem_el.store(memory);
```

步骤：
1. 为每个访问点添加 `registry.get()` 封装
2. 运行双写验证（旧路径 + Element 路径输出一致）
3. 确认一致后移除旧路径（feature flag 控制）

### 6.3 Backward Compatibility (Phase 5)

```rust
// OLD: Direct field access (deprecated but continues to work)
let bank = &self.reasoning_bank;

// NEW: Element access
let bank = self.registry.get::<MemoryElement>("element.memory").unwrap().bank();
```

Compatibility layer:
```rust
impl SelfIteratingBrain {
    /// [DEPRECATED] Use ElementRegistry instead
    #[deprecated(since = "0.2.0", note = "Use registry.get::<MemoryElement>()")]
    pub fn reasoning_bank(&self) -> &ReasoningBank {
        self.registry.get::<MemoryElement>("element.memory").unwrap().bank()
    }
}
```

### 6.4 并行运行数据流

```
External Event
    │
    ▼
Old Path: SelfIteratingBrain.iterate() ──→ modify ReasoningBrain ──→ store to ReasoningBank
                                                   │                         │
New Path: CapabilityElement.absorb() ───→ publish to Bus ───────────→ MemoryElement.subscribe()
                                                                           │
                                                                           ▼
                                                                    GoalElement re-evaluates
                                                                           │
                                                                           ▼
                                                                    ThinkingElement reflects
```

两个路径同时运行直到验证一致（数据校验在 Phase 5 集成测试中）。

---

## 7. Communication & Event Model

### 7.1 Standard Event Flow

```
┌──────────────┐     publish(CapabilityUpdated)     ┌──────────────┐
│ Capability   │ ──────────────────────────────────► │ Memory       │
│ Element      │                                     │ Element      │
│              │                                     │              │
│ absorb(src)──┼──►bus.publish(...)──────────────────► subscribe()  │
│              │                                     │ → store()    │
└──────────────┘                                     └──────────────┘
       │                                                     │
       │                                     ┌───────────────┘
       ▼                                     ▼
┌──────────────┐                     ┌──────────────┐
│ Monitor      │◄────────────────────│ Goal          │
│ Element      │   GoalStateChanged  │ Element       │
│              │                     │              │
│ detect stall │                     │ re-evaluate  │
└──────────────┘                     └──────────────┘
```

### 7.2 Event Catalog

| Event | Publisher | Subscribers | Payload |
|-------|-----------|-------------|---------|
| `CapabilityUpdated` | Capability | Memory, Monitor, Goal | `CapabilitySnapshot` |
| `MemoryStored` | Memory | Thinking, Monitor | `MemorySnapshot` |
| `GoalStateChanged` | Goal | Planner, Monitor | `GoalSnapshot` |
| `TraceCompleted` | Thinking | Skill, Monitor | `TraceSnapshot` |
| `EvolutionCompleted` | Evolution | Capability, Memory | `Map` |

### 7.3 Wiring Example

```rust
// In a bootstrapper:
let registry = ElementRegistry::new();

// Wire Capability → Memory automatically via bus
let cap = CapabilityElement::new();
let mem = MemoryElement::new();

registry.register(Box::new(cap))?;
registry.register(Box::new(mem))?;
registry.resolve_and_init()?;
registry.start_all()?;

// MemoryElement.init() already called bus.subscribe("element.memory", EventKind::CapabilityUpdated)
// So whenever cap.absorb() is called, mem automatically stores the event.

registry.get_mut::<CapabilityElement>("element.capability")
    .unwrap()
    .absorb(KnowledgeSource::HeroUI);
```

---

## 8. Persistence & Lifecycle

### 8.1 State Serialization

Each Element is responsible for its own persistence:

```rust
pub trait ElementPersistence: Element {
    /// Serialize current state to JSON value
    fn save_state(&self) -> Result<serde_json::Value, ElementError>;

    /// Restore state from JSON value (called after init())
    fn load_state(&mut self, state: &serde_json::Value) -> Result<(), ElementError>;

    /// Path for auto-persist (relative to ~/.neotrix/elements/)
    fn state_path(&self) -> Option<&str> { None }
}
```

Default state directory: `~/.neotrix/elements/{element_id}/state.json`

### 8.2 Registry Snapshot

```rust
impl ElementRegistry {
    /// Snapshot all Elements' states into a single JSON
    pub fn snapshot_all(&self) -> Result<HashMap<String, serde_json::Value>, ElementError>;

    /// Restore all Elements from a snapshot
    pub fn restore_all(&mut self, snapshot: &HashMap<String, serde_json::Value>) -> Result<(), ElementError>;
}
```

### 8.3 Health Check

```rust
pub trait ElementHealth: Element {
    /// Returns a health status for this Element
    fn health(&self) -> ElementHealthStatus;

    /// Last error encountered (if any)
    fn last_error(&self) -> Option<ElementError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementHealthStatus {
    pub id: String,
    pub state: RegistryState,
    pub uptime_secs: u64,
    pub memory_bytes: u64,
    pub last_heartbeat: u64,
    pub ok: bool,
    pub message: String,
}
```

---

## 9. ADR

### ADR-1: Why Not Dynamic Loading (`.so` / `dlopen`)

**Context**: We considered loading Elements as dynamic libraries at runtime.
**Decision**: Compile-time loading only.

**Rationale**:
- Rust has no stable ABI; `dynlib` is fragile across compiler versions
- NeoTrix is a single binary; dynamic loading adds complexity without benefit
- Aglet's subprocess/HTTP/MCP runtimes are Python-ecosystem solutions; Rust doesn't need them
- `#[cfg(feature = "...")]` provides sufficient compile-time selection
- **But**: The `Element` trait + `ElementRegistry` design does NOT preclude future dynamic loading via a `PluginRuntime` enum variant

### ADR-2: Bus vs. Direct Calls

**Context**: Should Elements communicate via bus or direct trait references?
**Decision**: Bus for events; direct access (via `registry.get()`) for commands.

**Rationale**:
- Bus enables loose coupling: Memory Element doesn't need to know about Monitor Element
- Direct access preserves type safety for command-style calls (`cap_element.absorb(src)`)
- Pattern: events for broadcast (many:many), registry.get() for point-to-point
- Aglet's `AgentContext + ContextPatch` pattern is equivalent: immutable context + event sourcing

### ADR-3: Element vs. Technique

**Context**: Should we implement Aglet's two-level (Element + Technique) model?
**Decision**: Element only in v1; Technique considered for v2.

**Rationale**:
- Aglet's Technique is useful when multiple implementations of the same protocol exist (e.g. `planner.echo` vs `planner.react`)
- NeoTrix currently has 1:1 modules per capability (one `GoalLoop`, one `SkillBridge`)
- When we need routing strategies (e.g. parallel goal planners), add `TechniqueRegistry` in v2

### ADR-4: Element Trait vs. Typed Wrapper

**Context**: Should `registry.get::<CapabilityElement>()` return `&dyn Element` or a typed reference?
**Decision**: Typed via generics.

**Rationale**:
- Type safety: callers get the full API of `CapabilityElement`, not just `Element` trait
- Implementation: `HashMap<String, Box<dyn Any>>` internal storage; `downcast_ref()` on get
- This is idiomatic Rust; matches `Arc<Mutex<T>>` pattern already used in codebase

### ADR-5: New Crate vs. In-Place

**Context**: Should Element protocol live in a separate crate `neotrix-element`?
**Decision**: New module `neotrix-core/src/neotrix/element/`.

**Rationale**:
- Placing alongside existing code reduces PR size and migration friction
- If Elements become a general plugin system across the entire NeoTrix workspace, extract to `neotrix-element` crate later
- The `Element` trait itself has zero external dependencies (only `std` + `serde`)

### ADR-6: Bus Channel Backend

**Context**: Which channel implementation for `ElementBus`?
**Decision**: `std::sync::mpsc` for simplicity; upgrade to `tokio::sync::broadcast` if async needed later.

**Rationale**:
- Current codebase is synchronous (no async runtime dependency)
- `mpsc` with `Arc<Mutex<...>>` suffices for current throughput
- Bus is an internal detail; swapping the backend doesn't affect Element implementations
- Aglet's Python async model doesn't map to Rust sync; we use sync channels with polling

### ADR-7: Error Handling Strategy

**Context**: How should Element errors propagate?
**Decision**: `ElementError` enum with `BusError`, `InitFailed`, etc. Registry collects errors and continues.

**Rationale**:
- One Element failing init should not block others (defensive design)
- `Registry.state` tracks partial failures; user can inspect via `health()` API
- Aglet's approach: exceptions per Element; we use Result-based error handling

---

## 10. Appendix: File Map

```
neotrix-core/src/neotrix/
├── element/                          ← NEW: Element plugin system
│   ├── mod.rs                         ← Module exports + re-exports
│   ├── core.rs                        ← Element trait, ElementType, CapabilityAccess
│   ├── registry.rs                    ← ElementRegistry, state machine
│   ├── bus.rs                         ← ElementBus, EventKind, EventPayload
│   ├── manifest.rs                    ← ElementManifest, load_from_manifest()
│   ├── capability.rs                  ← CapabilityElement
│   ├── memory.rs                      ← MemoryElement
│   ├── skill.rs                       ← SkillElement            (Phase 2)
│   ├── goal.rs                        ← GoalElement             (Phase 2)
│   ├── evolution.rs                   ← EvolutionElement        (Phase 2)
│   ├── thinking.rs                    ← ThinkingElement         (Phase 3)
│   ├── monitor.rs                     ← MonitoringElement       (Phase 3)
│   ├── planner.rs                     ← PlannerElement          (Phase 4)
│   └── network.rs                     ← NetworkElement          (Phase 4)
│
├── reasoning_brain/                   ← Existing (unchanged during Phase 1)
│   ├── ... (83 existing modules)
│
└── lib.rs                             ← Add `pub mod element;`
```

## 11. Summary

```
Monolith (83 mod, 35-field SelfIteratingBrain)
    │
    │  Phase 1: Element core infrastructure
    ▼
ElementRegistry (bus-driven, DAG-resolved)
    │
    │  Phase 1-2: Core Elements (Capability, Memory, Skill, Goal)
    ▼
ElementBus (pub/sub event backbone)
    │
    │  Phase 3-4: Full Elements (Thinking, Monitor, Planner, Network)
    ▼
Plugin-compatible Architecture (zero core changes for new Elements)
    │
    │  Phase 5: Backward compat + graduation
    ▼
All modules accessible as Elements, monolith delegates to registry
```

**Key metrics**:
- Lines moved out of `loop_impl.rs`: ~600 (all Element-related fields)
- New public API surface: `Element` trait (7 methods) + `ElementRegistry` (12 methods) + `ElementBus` (5 methods)
- Test isolation: Each Element testable with `ElementBus::mock()` (no real channels)
- Backward compatibility: 100% existing tests pass unchanged (feature-flag gated)
