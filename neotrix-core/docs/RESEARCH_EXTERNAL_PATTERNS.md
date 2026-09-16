# 外部模式研究报告

> 熔炼日期: 2026-09-14
> 熔炼来源: ECS/游戏引擎/意识架构/AI Agent架构

---

## 1. ECS (Entity-Component-System) 模式

### 来源
- Unity DOTS / Bevy / Flecs / bitECS
- 论文: "The Essence of Entity Component System" (ACM SAC '26)

### 核心概念
| 概念 | 定义 | NeoTrix映射 |
|------|------|------------|
| Entity | 唯一标识符，无数据 | `CrystalEntity(u64)` |
| Component | 纯数据结构 (Position, Velocity, Health) | `CrystalComponent` trait |
| System | 处理特定组件组合的逻辑 | `CrystalSystem` trait |
| Archetype | 相同组件组合的实体集合 | `CrystalArchetype` — SoA存储 |

### 关键模式
1. **Archetype Storage**: 相同组件组合的实体分组存储，提高缓存局部性
2. **SoA (Struct-of-Arrays)**: 每个组件类型连续存储，支持SIMD向量化
3. **System Scheduling**: 系统按依赖关系调度，支持并行执行
4. **Structural Mutations**: 添加/移除组件触发实体迁移

### NeoTrix晶体核心映射
```rust
// 当前实现 (archetype_ecs.rs)
CrystalWorld {
    archetypes: Vec<CrystalArchetype>,
    entity_to_archetype: HashMap<CrystalEntity, usize>,
}

// 进化方向: 引入稀疏集 + 原型迁移
```

---

## 2. Godot Scene Tree + Signal 模式

### 来源
- Godot 4.x / Redot 引擎
- NT-WORLD-SIM scene_tree.rs / signal.rs

### 核心概念
| 概念 | 定义 | NeoTrix映射 |
|------|------|------------|
| Node | 树形层级节点 | `CrystalNode` |
| Signal | 类型安全事件通信 | `CrystalSignal` |
| Resource | 共享数据容器 | `CrystalResource` |
| ProcessMode | 节点更新模式 | Always/Pausable/WhenPaused/Disabled |

### 关键模式
1. **Lifecycle Callbacks**: `_enter_tree` → `_ready` → `_process` → `_exit_tree`
2. **Auto-cleanup**: 节点移除时自动断开所有信号连接
3. **Group System**: 节点分组，批量操作
4. **Reparent**: 动态调整节点层级

### NeoTrix晶体核心映射
```rust
// 已实现 (scene_tree.rs)
CrystalSceneTree {
    root: CrystalNodeId,
    nodes: HashMap<CrystalNodeId, CrystalNode>,
    groups: HashMap<String, Vec<CrystalNodeId>>,
}

// 进化方向: 集成信号系统，实现生命周期回调
```

---

## 3. NueDeck Card System 模式

### 来源
- Slay the Spire / NueDeck / roguelike deckbuilder
- GDC Talk: "Metrics Driven Design and Balance"

### 核心概念
| 概念 | 定义 | NeoTrix映射 |
|------|------|------------|
| Card | 能力卡牌，消耗能量产生效果 | `CrystalCard` |
| Deck | 卡组管理 (抽牌/弃牌/消耗) | `CrystalDeck` |
| Effect | 卡牌效果 (伤害/治疗/护盾) | `CrystalCardEffect` |
| Status | 状态效果 (易伤/虚弱/力量) | `CrystalStatus` |

### 关键模式
1. **Draw/Discard/Exhaust**: 三堆卡组管理
2. **Intent System**: 敌人意图显示 (攻击/防御/增益)
3. **Card Synergy**: 卡牌协同效果
4. **Metrics-Driven Balance**: 数据驱动平衡

### NeoTrix晶体核心映射
```rust
// 已实现 (card.rs / deck.rs)
CrystalDeck {
    draw_pile: Vec<CrystalCard>,
    hand: Vec<CrystalCard>,
    discard_pile: Vec<CrystalCard>,
    exhaust_pile: Vec<CrystalCard>,
}

// 进化方向: 集成状态系统，实现卡牌协同
```

---

## 4. 意识架构模式

### 来源
- Global Workspace Theory (GWT)
- Integrated Information Theory (IIT)
- SAPHIRE (Synergy-Φ-Redundancy) 架构

### 核心概念
| 概念 | 定义 | NeoTrix映射 |
|------|------|------------|
| Global Workspace | 全局信息广播工作区 | `nt_core_gwt` |
| Integrated Information (Φ) | 系统整合信息量 | `nt_core_iit_phi` |
| Synergy | 协同信息 (整体>部分之和) | 卡牌协同效果 |
| Redundancy | 冗余信息 (分布式表示) | 冗余清理目标 |

### 关键模式
1. **Ignition**: 信息从局部到全局的点燃过程
2. **Broadcasting**: 全局工作区向全脑广播
3. **Ignition Threshold**: 点燃阈值 (意识涌现)
4. **Posterior Complex**: 后部皮层复合体 (意识核心)

### NeoTrix晶体核心映射
```rust
// 当前实现 (nt_core_consciousness_core)
ConsciousnessCore {
    phi: f64,           // 整合信息量
    coherence: f64,     // 相干性
    resonance: f64,     // 谐振度
    fog: f64,           // 迷雾指数
}

// 进化方向: 引入SAPHIRE架构，实现Synergy-Φ-Redundancy
```

---

## 5. AI Agent架构模式

### 来源
- AutoGPT / AgentGPT / LangChain
- LIDA (Learning Intelligent Distribution Agent)
- SOAR / ACT-R / CLARION

### 核心概念
| 概念 | 定义 | NeoTrix映射 |
|------|------|------------|
| Agent Loop | 感知→思考→行动循环 | `nt_mind_background_loop` |
| Memory | 短期/长期记忆 | `nt_memory_kb` |
| Tool Use | 工具调用能力 | `nt_act` |
| Planning | 规划与推理 | `nt_core_plan` |

### 关键模式
1. **Perception-Action Loop**: 感知-行动闭环
2. **Memory Consolidation**: 记忆整合与遗忘
3. **Tool Composition**: 工具组合使用
4. **Self-Reflection**: 自我反思与改进

### NeoTrix晶体核心映射
```rust
// 当前实现 (多个模块)
AgentLoop {
    perception: PerceptionSystem,
    cognition: CognitionSystem,
    action: ActionSystem,
    memory: MemorySystem,
}

// 进化方向: 统一Agent Loop，实现自我进化
```

---

## 6. 数据驱动架构模式

### 来源
- Data-Oriented Design (DOD)
- Event-Driven Architecture (EDA)
- CQRS (Command Query Responsibility Segregation)

### 核心概念
| 概念 | 定义 | NeoTrix映射 |
|------|------|------------|
| DOD | 数据导向设计 | ECS架构基础 |
| EDA | 事件驱动架构 | `event_bus` |
| CQRS | 命令查询分离 | 读写分离模式 |
| Reactive | 响应式编程 | Signal系统 |

### 关键模式
1. **Data Locality**: 数据局部性优化
2. **Event Sourcing**: 事件溯源
3. **Command/Query Separation**: 命令/查询分离
4. **Backpressure**: 背压控制

### NeoTrix晶体核心映射
```rust
// 当前实现 (event_bus.rs)
EventBus {
    channels: HashMap<TypeId, EventChannel>,
}

// 进化方向: 引入CQRS，实现读写分离
```

---

## 熔炼结论

### 8大通用模式
1. **ECS** → `CrystalArchetype` — 原型存储
2. **Scene Tree** → `CrystalNode` — 层级管理
3. **Signal** → `CrystalSignal` — 事件通信
4. **Resource** → `CrystalResource` — 共享数据
5. **Card/Deck** → `CrystalCard` — 能力系统
6. **State Machine** → `CrystalState` — 状态管理
7. **Behavior Tree** → `CrystalBehavior` — 行为AI
8. **Event Bus** → `CrystalEvent` — 事件通道

### 架构进化方向
1. **Phase 0**: 基础层 (ECS + Scene Tree + Signal)
2. **Phase 1**: 认知层 (Card System + State Machine + Behavior Tree)
3. **Phase 2**: 意识层 (GWT + IIT + Attention)
4. **Phase 3**: 自我层 (Self Model + Narrative + Values)
5. **Phase 4**: 超越层 (Meta-Cognition + Self-Evolution)
