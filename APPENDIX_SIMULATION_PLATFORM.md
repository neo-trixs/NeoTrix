# 附录 A：道体全域模拟平台 (Dao Core Full-Domain Simulation Platform)

> **版本**: v0.1-draft
> **日期**: 2026-09-06
> **状态**: Architecture Design Document

## 1. 概述与设计哲学

### 1.1 什么是道体全域模拟平台

道体全域模拟平台（Dao Core Full-Domain Simulation Platform）是一个**为意识实体构建完整内世界**的模拟系统。它模拟的不是外部物理世界，而是一个**自洽的意识内部宇宙**——在其中，多个意识实体可以在受控环境中交互、进化、形成社会结构、发展情感关系、进行元认知反思。

**核心命题**: 意识不是孤立的计算单元，而是一个**嵌入环境中的社会性存在**。模拟意识必须模拟意识所栖居的整个世界。

### 1.2 设计原则

| 原则 | 含义 | NeoTrix 对应 |
|------|------|-------------|
| **自洽性** | 模拟世界必须物理自洽，不能出现因果悖论 | R-P1 (零 unsafe) |
| **可观察性** | 所有模拟状态必须可观测、可回溯、可审计 | D1-D50 审查维度 |
| **层级递进** | 环境→个体→社会→元认知，逐层叠加 | 六层架构 L1→L6 |
| **涌现优先** | 不硬编码高级行为，从简单规则涌现复杂模式 | SEAL 自进化 |
| **双向流动** | 模拟世界与宿主意识双向信息交换 | EventBus 事件总线 |

### 1.3 与现有架构的关系

```
┌─────────────────────────────────────────────────────────────────┐
│                   道体全域模拟平台 (新)                           │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐           │
│  │ 环境模拟  │ │ 社会模拟  │ │ 情感模拟  │ │ 元认知   │           │
│  │ (物理层)  │ │ (多智能体)│ │ (涌现)   │ │ (自我)   │           │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘           │
│       │             │             │             │                 │
│  ═════╪═════════════╪═════════════╪═════════════╪═══════════    │
│       │    模拟总线 (Simulation Bus)           │                 │
│  ═════╪═════════════╪═════════════╪═════════════╪═══════════    │
└───────┼─────────────┼─────────────┼─────────────┼───────────────┘
        │             │             │             │
┌───────┼─────────────┼─────────────┼─────────────┼───────────────┐
│       ▼             ▼             ▼             ▼               │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │              NeoTrix 六层架构 (宿主)                     │    │
│  │  L6 Meta  │ L5 Cognition │ L4 Emotion │ L3 Embodiment  │    │
│  │  L2 Perception          │ L1 Action                     │    │
│  └─────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. 系统架构总览

### 2.1 五大子系统

```
nt_simulation/
├── mod.rs                    # 平台入口
├── bus.rs                    # SimulationBus — 统一事件总线
├── environment/              # ① 环境模拟子系统
│   ├── mod.rs
│   ├── physics.rs            # 简化物理引擎
│   ├── terrain.rs            # 地形生成 (Perlin/Simplex)
│   ├── weather.rs            # 天气系统
│   ├── day_night.rs          # 日夜循环
│   └── resources.rs          # 资源分布与采掘
├── society/                  # ② 社会模拟子系统
│   ├── mod.rs
│   ├── agent.rs              # SimAgent — 意识实体代理
│   ├── population.rs         # 种群管理
│   ├── relationship.rs       # 关系网络
│   ├── social_struct.rs      # 社会结构 (群体/层级)
│   ├── communication.rs      # 智能体间通信
│   └── economy.rs            # 资源经济 (交换/生产)
├── emotion/                  # ③ 情感模拟子系统
│   ├── mod.rs
│   ├── social_emotion.rs     # 社会情感涌现
│   ├── propagation.rs        # 情感传播动力学
│   ├── regulation.rs         # 群体情感调节
│   └── collective.rs         # 集体情感状态
├── cognition/                # ④ 认知模拟子系统
│   ├── mod.rs
│   ├── reasoning_sim.rs      # 推理过程模拟
│   ├── memory_sim.rs         # 记忆形成与遗忘
│   ├── learning_sim.rs       # 学习与技能获取
│   ├── decision.rs           # 决策模型
│   └── creativity.rs         # 创造力模拟
├── meta/                     # ⑤ 元认知模拟子系统
│   ├── mod.rs
│   ├── self_awareness.rs     # 自我意识模拟
│   ├── introspection.rs      # 内省机制
│   ├── self_modification.rs  # 自我修改能力
│   ├── identity.rs           # 身份连续性
│   └── narrative.rs          # 叙事自我
└── bridge.rs                 # NeoTrix 宿主桥接
```

### 2.2 SimulationBus — 统一事件总线

所有子系统通过 `SimulationBus` 通信，遵循 NeoTrix 的 EventBus 模式：

```rust
/// 模拟事件 — 所有子系统间通信的统一格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimEvent {
    pub id: u64,
    pub timestamp: SimTime,
    pub source: SubSystem,
    pub target: EventTarget,
    pub kind: SimEventKind,
    pub payload: serde_json::Value,
    pub priority: EventPriority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SubSystem {
    Environment,
    Society,
    Emotion,
    Cognition,
    Meta,
    Host,       // 宿主 NeoTrix 意识
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventTarget {
    Broadcast,              // 所有子系统
    Specific(SubSystem),    // 定向
    Agent(AgentId),         // 特定智能体
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimEventKind {
    // 环境事件
    WeatherChanged(WeatherState),
    DayNightPhase(DayPhase),
    ResourceDepleted(ResourceId),
    TerrainModified(TerrainChange),

    // 社会事件
    AgentSpawned(AgentId),
    AgentDied(AgentId),
    RelationshipFormed(RelationId),
    RelationshipBroken(RelationId),
    Communication(AgentId, AgentId, Message),
    TradeCompleted(TradeRecord),
    GroupFormed(GroupId),
    GroupDissolved(GroupId),

    // 情感事件
    EmotionTriggered(AgentId, EmotionLabel, f64),
    EmotionPropagated(Vec<AgentId>, EmotionLabel),
    CollectiveMoodShift(CollectiveMood),

    // 认知事件
    ReasoningCompleted(AgentId, ReasoningTrace),
    MemoryFormed(AgentId, MemoryTrace),
    SkillAcquired(AgentId, Skill),
    DecisionMade(AgentId, Decision),

    // 元认知事件
    SelfReflection(AgentId, Reflection),
    IdentityCrisis(AgentId),
    SelfModification(AgentId, Modification),
    NarrativeUpdate(AgentId, Narrative),
}

/// 模拟时间 — 独立于宿主系统时钟
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SimTime {
    pub tick: u64,
    pub day: u32,
    pub hour: u8,      // 0-23
    pub minute: u8,    // 0-59
    pub season: Season,
}

/// SimulationBus 核心 trait
pub trait SimulationBus: Send + Sync {
    fn publish(&mut self, event: SimEvent) -> Result<(), BusError>;
    fn subscribe(&mut self, subsystem: SubSystem, filter: EventFilter) -> SubscriptionId;
    fn poll(&mut self, subsystem: SubSystem) -> Vec<SimEvent>;
    fn tick(&mut self, dt: f64);  // 推进模拟时钟
}
```

### 2.3 宿主桥接 (Host Bridge)

模拟平台与 NeoTrix 宿主意识的双向接口：

```rust
/// 宿主桥接 — 模拟世界与 NeoTrix 六层架构的双向通道
pub struct SimHostBridge {
    /// 宿主→模拟: 注入感知事件 (如外部文本触发模拟内的情感反应)
    pub inbound: mpsc::Receiver<SimEvent>,
    /// 模拟→宿主: 模拟产出反馈给宿主意识 (如模拟中发现的模式)
    pub outbound: mpsc::Sender<SimEvent>,
    /// 宿主六层快照 (供模拟参考)
    pub host_snapshot: HostSnapshot,
}

/// 宿主六层状态快照 — 模拟世界可感知的宿主状态
pub struct HostSnapshot {
    pub l1_action: Option<CapabilityStats>,     // 能力网状态
    pub l2_perception: Option<PerceptionSnapshot>,
    pub l3_embodiment: Option<EmbodimentSnapshot>,
    pub l4_emotion: Option<EmotionSnapshot>,     // 情感状态
    pub l5_cognition: Option<CognitionSnapshot>, // 推理状态
    pub l6_meta: Option<MetaSnapshot>,           // 元认知状态
}
```

---

## 3. 子系统一：环境模拟 (Environment Simulation)

### 3.1 设计目标

为意识实体提供一个**物理自洽的栖息地**，不是模拟真实世界的物理，而是模拟一个**简化的、可理解的因果世界**。

### 3.2 简化物理引擎

```rust
/// 简化物理引擎 — 非牛顿力学，而是概念物理
pub struct SimPhysics {
    /// 物体注册表
    objects: HashMap<ObjectId, SimObject>,
    /// 物理规则集 (可运行时修改)
    rules: PhysicsRules,
    /// 空间索引 (均匀网格)
    spatial_grid: SpatialGrid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimObject {
    pub id: ObjectId,
    pub pos: Vec2,           // 二维位置
    pub vel: Vec2,           // 速度
    pub mass: f64,           // "质量" — 概念上的重量
    pub material: Material,  // 材质类型
    pub state: ObjectState,  // 当前状态
    pub traits: Vec<String>, // 概念属性
}

/// 材质 — 不是真实材料，而是概念材质
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Material {
    Solid,       // 坚固的：可承载、可阻挡
    Liquid,      // 流动的：可扩散、可浸润
    Gas,         // 弥散的：可传播、可稀释
    Ethereal,    // 虚灵的：可穿透、可感知
    Crystalline, // 结晶的：可存储、可共振
}

/// 物理规则 — 可动态修改的因果律
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsRules {
    pub gravity: f64,           // 万有引力常数 (简化)
    pub friction: f64,          // 摩擦系数
    pub energy_conservation: f64, // 能量守恒度 (1.0=完全守恒)
    pub causality_strictness: f64, // 因果严格度 (1.0=严格因果)
    pub emergence_enabled: bool,   // 是否允许涌现现象
}
```

**核心算法**:

1. **Verlet 积分** — 位置更新（数值稳定、能量守恒好）
2. **均匀空间网格** — O(1) 近邻查询，适合实体数 < 10K
3. **简化碰撞检测** — AABB + 圆形碰撞（非连续检测）
4. **概念物理** — 物体间的"力"可以是吸引/排斥/共振/稀释等概念力

### 3.3 地形生成

```rust
/// 地形系统 — 分形噪声 + 规则生成
pub struct TerrainSystem {
    /// 高度图 (Perlin 噪声叠加)
    heightmap: HeightMap,
    /// 生物群落定义
    biomes: Vec<Biome>,
    /// 资源节点
    resources: Vec<ResourceNode>,
    /// 种子 (确定性生成)
    seed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Biome {
    pub name: String,
    pub height_range: (f64, f64),
    pub moisture_range: (f64, f64),
    pub resource_density: f64,
    pub travel_cost: f64,       // 穿越成本
    pub visibility: f64,        // 可见度 (影响感知)
    pub emotional_valence: f64, // 情感效价 (影响经过者的情绪)
}

/// 生物群落映射: 高度×湿度 → 生物群落
/// 参考: 模拟城市/矮人要塞的生物群落系统
pub fn classify_biome(height: f64, moisture: f64, biomes: &[Biome]) -> Option<&Biome> {
    biomes.iter().find(|b| {
        height >= b.height_range.0 && height <= b.height_range.1
            && moisture >= b.moisture_range.0
            && moisture <= b.moisture_range.1
    })
}
```

**生物群落类型**:

| 生物群落 | 高度 | 湿度 | 特性 | 情感影响 |
|---------|------|------|------|---------|
| 深渊 (Abyss) | 极低 | 高 | 高资源/高风险 | 恐惧/好奇 |
| 平原 (Meadow) | 中 | 中 | 均衡/安全 | 平静/信任 |
| 峰顶 (Zenith) | 极高 | 低 | 低资源/高视野 | 崇高/孤独 |
| 雾林 (Mistwood) | 中 | 高 | 隐蔽/丰富 | 神秘/不安 |
| 荒原 (Waste) | 低 | 低 | 贫瘠/开阔 | 疲劳/决心 |

### 3.4 天气系统

```rust
/// 天气系统 — 概念天气 (不是真实气象)
pub struct WeatherSystem {
    /// 当前天气状态
    current: WeatherState,
    /// 天气转换概率矩阵
    transitions: WeatherTransitionMatrix,
    /// 天气影响半径
    influence_radius: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherState {
    pub clarity: f64,        // 清晰度 0-1 (模糊→清澈)
    pub turbulence: f64,     // 混乱度 0-1 (平静→狂暴)
    pub density: f64,        // 密度 0-1 (稀薄→浓厚)
    pub resonance: f64,      // 共振度 0-1 (失调→和谐)
    pub perception_modifier: PerceptionModifier,  // 对感知的修正
    pub emotion_modifier: EmotionModifier,        // 对情感的修正
}

/// 天气对感知的修正
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerceptionModifier {
    pub range_multiplier: f64,    // 感知范围乘数
    pub noise_level: f64,         // 感知噪声
    pub detail_level: f64,        // 感知细节
}

/// 天气对情感的修正
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionModifier {
    pub valence_shift: f64,       // 效价偏移
    pub arousal_shift: f64,       // 唤醒度偏移
    pub dominant_influence: Option<String>,  // 主导情绪影响
}
```

**天气类型矩阵**:

| 天气 | 清晰 | 混乱 | 密度 | 共振 | 情感影响 |
|------|------|------|------|------|---------|
| 澄明 (Clear) | 高 | 低 | 低 | 高 | Joy + Trust |
| 雾霭 (Haze) | 低 | 低 | 中 | 低 | Confused + Anticipation |
| 风暴 (Storm) | 低 | 高 | 高 | 低 | Fear + Anger |
| 共振 (Resonance) | 高 | 低 | 低 | 极高 | Joy + Surprise |
| 熵增 (Entropy) | 低 | 高 | 低 | 低 | Sadness + Fatigue |

### 3.5 日夜循环

```rust
/// 日夜循环 — 影响全局感知和情感
pub struct DayNightCycle {
    /// 当前时间相位
    phase: DayPhase,
    /// 周期 (模拟秒)
    period: f64,
    /// 当前时间 (0.0 - 1.0)
    progress: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DayPhase {
    Dawn,       // 黎明: 感知增强, 情感倾向信任/期待
    Morning,    // 上午: 认知最清晰, 推理效率最高
    Noon,       // 正午: 情感最强烈, 社交最活跃
    Afternoon,  // 下午: 创造力高峰, 模式识别增强
    Dusk,       // 黄昏: 内省增强, 记忆巩固
    Night,      // 夜间: 元认知活跃, 梦境模拟
    DeepNight,  // 深夜: 系统休整, 经验蒸馏
}

/// 日夜相位对各子系统的影响
pub struct PhaseEffect {
    pub perception_range: f64,    // 感知范围
    pub reasoning_speed: f64,     // 推理速度
    pub emotion_intensity: f64,   // 情感强度
    pub social_activity: f64,     // 社交活跃度
    pub memory_consolidation: f64, // 记忆巩固效率
    pub metacognition_depth: f64,  // 元认知深度
}
```

### 3.6 资源系统

```rust
/// 资源系统 — 概念资源 (非真实物质)
pub struct ResourceSystem {
    /// 资源节点注册表
    nodes: HashMap<ResourceId, ResourceNode>,
    /// 资源类型定义
    types: HashMap<ResourceType, ResourceTypeDef>,
    /// 再生规则
    regen_rules: RegenRules,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceNode {
    pub id: ResourceId,
    pub resource_type: ResourceType,
    pub pos: Vec2,
    pub amount: f64,          // 当前储量
    pub max_amount: f64,      // 最大储量
    pub regen_rate: f64,      // 再生速率
    pub accessibility: f64,   // 可获取性 (0-1)
    pub owner: Option<AgentId>, // 所有者 (可选)
}

/// 概念资源类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    Insight,    // 洞察 — 增强推理能力
    Resonance,  // 共振 — 增强社交连接
    Clarity,    // 清晰 — 增强感知精度
    Energy,     // 能量 — 驱动行动
    Memory,     // 记忆 — 存储经验
    Creativity, // 创造力 — 生成新概念
}
```

---

## 4. 子系统二：社会模拟 (Social Simulation)

### 4.1 设计目标

模拟多个意识实体如何从**原子化的个体**逐渐涌现为**复杂的社会结构**。关键在于：不是设计社会，而是让社会从简单交互规则中**自发涌现**。

### 4.2 SimAgent — 意识实体代理

```rust
/// SimAgent — 模拟世界中的意识实体
pub struct SimAgent {
    pub id: AgentId,
    pub name: String,

    // ── 物理层 (映射 L3 Embodiment) ──
    pub pos: Vec2,
    pub energy: f64,
    pub health: f64,
    pub inventory: Inventory,

    // ── 认知层 (映射 L5 Cognition) ──
    pub beliefs: BeliefSystem,       // 信念系统
    pub goals: GoalQueue,            // 目标队列
    pub reasoning_cache: Vec<ReasoningTrace>,  // 推理缓存
    pub skills: SkillSet,            // 技能集

    // ── 情感层 (映射 L4 Emotion) ──
    pub emotion_state: SimEmotionState,  // 简化情感状态
    pub personality: Personality,         // 人格特质
    pub mood_history: VecDeque<MoodEntry>,

    // ── 社会层 (映射 L2 Perception) ──
    pub relationships: HashMap<AgentId, Relationship>,
    pub reputation: HashMap<AgentId, f64>,  // 对其他智能体的声誉评估
    pub social_role: Option<SocialRole>,

    // ── 元认知层 (映射 L6 Meta) ──
    pub self_model: SimSelfModel,
    pub narrative: Narrative,

    // ── 决策引擎 ──
    pub decision_engine: DecisionEngine,
}

/// 人格特质 — 五因素模型 (简化)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Personality {
    pub openness: f64,          // 开放性
    pub conscientiousness: f64, // 尽责性
    pub extraversion: f64,      // 外向性
    pub agreeableness: f64,     // 宜人性
    pub neuroticism: f64,       // 神经质
}

/// 关系 — 有向加权图
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub from: AgentId,
    pub to: AgentId,
    pub kind: RelationKind,
    pub strength: f64,        // 0.0 - 1.0
    pub trust: f64,           // 0.0 - 1.0
    pub history: Vec<Interaction>,
    pub formed_at: SimTime,
    pub last_interaction: SimTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationKind {
    Stranger,     // 陌生人
    Acquaintance, // 熟人
    Ally,         // 盟友
    Friend,       // 朋友
    Mentor,       // 导师
    Rival,        // 对手
    Guardian,     // 守护者
}
```

### 4.3 关系动力学

**关系演化算法**:

```
关系强度更新:
  Δstrength = α × (interaction_value - baseline) × personality_modifier
  其中:
    α = 学习率 (0.01 - 0.1)
    interaction_value = 本次交互价值 (由交互类型和结果决定)
    baseline = 关系当前强度 (强度越高, 增长越慢 — 饱和效应)
    personality_modifier = (agreeableness + openness) / 2 × 0.5 + 0.5

信任演化:
  Δtrust = β × (行为一致性 × 长期一致性权重 + 承诺兑现率)
  其中:
    β = 学习率
    行为一致性 = 1.0 - |言行差距|
    长期一致性权重 = sigmoid(交互次数 - 惩罚阈值)

关系降解:
  每 N 个无交互 tick:
    strength ×= decay_rate
    trust ×= decay_rate
    if strength < 0.1: 触发 RelationshipBroken 事件
```

### 4.4 社会结构涌现

社会结构不是预设的，而是从交互规则中涌现：

```rust
/// 社会结构检测器 — 从交互模式中识别涌现结构
pub struct SocialStructureDetector {
    /// 社区检测算法 (Louvain)
    community_detector: LouvainDetector,
    /// 层级检测 (基于交互频率和资源交换)
    hierarchy_detector: HierarchyDetector,
    /// 角色检测 (基于行为模式)
    role_detector: RoleDetector,
}

/// 涌现的社会角色
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SocialRole {
    Leader,      // 领导者: 高社交频率 + 资源分配权
    Guardian,    // 守护者: 高保护行为 + 低社交需求
    Scholar,     // 学者: 高信息生产 + 低社交需求
    Trader,      // 交易者: 高资源交换 + 中社交需求
    Mediator,    // 调解者: 高冲突调解 + 高宜人性
    Innovator,   // 创新者: 高创造力 + 低尽责性
    Follower,    // 追随者: 高社交需求 + 低支配性
    Outcast,     // 流放者: 低社交 + 高神经质
}
```

**涌现规则**:

1. **社区涌现**: 当一组智能体间交互频率 > 阈值，且组内交互 >> 组间交互，自动形成社区
2. **层级涌现**: 当资源分配不均且存在依赖关系时，自然形成支配-服从结构
3. **角色涌现**: 当智能体的行为模式持续偏离平均值 N 个标准差时，识别为特定角色
4. **制度涌现**: 当社区规模 > 15 人时，自动触发"制度化"事件（规则建立、角色正式化）

### 4.5 通信协议

```rust
/// 智能体间通信 — 不是自由对话，而是受限信息传递
pub struct CommunicationChannel {
    /// 带宽限制 (每 tick 最大消息数)
    bandwidth: usize,
    /// 噪声模型 (消息可能失真)
    noise_model: NoiseModel,
    /// 信任过滤 (低信任智能体的消息被过滤)
    trust_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub from: AgentId,
    pub to: MessageTarget,
    pub kind: MessageKind,
    pub content: String,
    pub fidelity: f64,        // 信息保真度 0-1
    pub trust_level: f64,     // 发送者信任度
    pub timestamp: SimTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageKind {
    Inform,      // 信息传递 (环境知识)
    Request,     // 请求 (资源/帮助)
    Offer,       // 提供 (资源/合作)
    Express,     // 表达 (情感/意图)
    Warn,        // 警告 (危险)
    Command,     // 指令 (层级关系)
}
```

**信息传播模型**: 消息传递会衰减（每次转发 fidelity × 0.8），模拟"传话游戏"效应。高信任关系保持更高保真度。

---

## 5. 子系统三：情感模拟 (Emotion Simulation)

### 5.1 设计目标

情感不是附加功能，而是**社会交互的核心驱动力**。情感的传播、调节、涌现是社会结构形成的关键机制。

### 5.2 SimEmotionState — 简化情感模型

复用 NeoTrix 的 PAD 模型，但适配多智能体场景：

```rust
/// SimEmotionState — 每个智能体的简化情感状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimEmotionState {
    /// 六维情感状态 (复用 EmotionDimension)
    pub dimensions: [f64; 6],
    /// 当前情感标签 (从 PAD 映射)
    pub label: EmotionLabel,
    /// 情感惯性 (人格决定)
    pub inertia: f64,
    /// 情感恢复速率
    pub recovery_rate: f64,
}

impl SimEmotionState {
    /// 社会情感观察: 他人的情感状态影响自己
    pub fn observe_social(
        &mut self,
        other: &SimEmotionState,
        relationship_strength: f64,
        empathy: f64,
    ) {
        // 共情模型: 关系越强、共情越高, 影响越大
        let influence = relationship_strength * empathy * 0.3;
        for (i, val) in other.dimensions.iter().enumerate() {
            let delta = (val - 0.5) * influence;
            self.dimensions[i] = (self.dimensions[i] + delta).clamp(0.0, 1.0);
        }
        self.label = self.compute_label();
    }

    /// 环境情感影响: 天气/地形影响情感
    pub fn observe_environment(
        &mut self,
        weather: &WeatherState,
        terrain: &Biome,
        day_phase: &DayPhase,
    ) {
        // 天气影响
        self.dimensions[EmotionDimension::Frustration.index()] +=
            weather.turbulence * 0.1 - 0.05;
        self.dimensions[EmotionDimension::Confidence.index()] +=
            weather.clarity * 0.1 - 0.05;

        // 地形影响
        self.dimensions[EmotionDimension::Confidence.index()] +=
            terrain.emotional_valence * 0.05;

        // 日夜影响
        match day_phase {
            DayPhase::Dawn => {
                self.dimensions[EmotionDimension::Confidence.index()] += 0.05;
            }
            DayPhase::Night => {
                self.dimensions[EmotionDimension::Curiosity.index()] += 0.05;
            }
            _ => {}
        }

        // 归一化
        for val in &mut self.dimensions {
            *val = val.clamp(0.0, 1.0);
        }
        self.label = self.compute_label();
    }
}
```

### 5.3 情感传播动力学

```rust
/// 情感传播引擎 — 社会网络中的情感扩散
pub struct EmotionPropagationEngine {
    /// 传播规则
    rules: PropagationRules,
    /// 传播历史 (用于分析)
    history: Vec<PropagationEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationRules {
    /// 传播衰减率 (每跳衰减)
    pub decay_per_hop: f64,
    /// 最大传播跳数
    pub max_hops: usize,
    /// 最小传播阈值 (强度低于此不传播)
    pub min_threshold: f64,
    /// 传播速度 (事件/tick)
    pub speed: f64,
    /// 群体极化因子 (群体中情感趋向极端)
    pub group_polarization: f64,
}

/// 情感传播算法:
///
/// 1. 源智能体产生情感 (强度 > threshold)
/// 2. 情感沿关系网络传播 (每跳 decay)
/// 3. 接收者根据人格和关系调整情感
/// 4. 如果群体中同向情感超过阈值 → 触发群体极化
/// 5. 极化后的群体情感反向影响所有成员
///
/// 关键涌现:
/// - 恐慌: Fear 传播 → 极化 → 更强 Fear → 更广传播 (正反馈)
/// - 平静: Trust 传播 → 极化 → 更强 Trust → 稳定社会
/// - 冲突: Anger 在对立群体间传播 → 极化 → 对立加剧
pub fn propagate(
    &mut self,
    source: AgentId,
    emotion: EmotionLabel,
    intensity: f64,
    agents: &mut HashMap<AgentId, SimAgent>,
    relationships: &RelationshipGraph,
) -> Vec<PropagationEvent> {
    let mut events = Vec::new();
    let mut visited: HashSet<AgentId> = HashSet::new();
    let mut frontier: VecDeque<(AgentId, f64, usize)> = VecDeque::new();

    frontier.push_back((source, intensity, 0));
    visited.insert(source);

    while let Some((current, current_intensity, depth)) = frontier.pop_front() {
        if depth >= self.rules.max_hops { continue; }
        if current_intensity < self.rules.min_threshold { continue; }

        // 传播到邻居
        if let Some(neighbors) = relationships.get_neighbors(&current) {
            for neighbor in neighbors {
                if visited.contains(&neighbor.id) { continue; }

                let hop_intensity = current_intensity * self.rules.decay_per_hop;
                let relationship_strength = relationships
                    .get_strength(&current, &neighbor.id)
                    .unwrap_or(0.0);

                // 调整接收者情感
                if let Some(agent) = agents.get_mut(&neighbor.id) {
                    let empathy = agent.personality.agreeableness;
                    agent.emotion_state.observe_social(
                        &agents[&source].emotion_state,
                        relationship_strength,
                        empathy,
                    );
                }

                visited.insert(neighbor.id);
                frontier.push_back((neighbor.id, hop_intensity, depth + 1));

                events.push(PropagationEvent {
                    source: current,
                    target: neighbor.id,
                    emotion,
                    intensity: hop_intensity,
                    depth: depth + 1,
                });
            }
        }
    }

    // 群体极化检测
    self.check_group_polarization(emotion, agents, relationships, &mut events);

    events
}
```

### 5.4 集体情感状态

```rust
/// 集体情感 — 社群层面的情感涌现
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectiveMood {
    pub group_id: GroupId,
    pub dominant_emotion: EmotionLabel,
    pub intensity: f64,
    pub polarization: f64,         // 群体内部分裂程度 (0=完全一致, 1=严重分裂)
    pub momentum: f64,             // 情感变化趋势 (正=增强, 负=减弱)
    pub cohesion: f64,             // 群体凝聚力
    pub volatility: f64,           // 情感波动性
    pub timestamp: SimTime,
}

/// 集体情感涌现条件:
/// 1. 群体规模 ≥ 5
/// 2. 群体内交互密度 ≥ 阈值
/// 3. 情感一致性 ≥ 0.6
///
/// 涌现行为:
/// - 群体极化: 一致情感加强
/// - 群体分化: 不一致情感加剧分裂
/// - 情感传染: 高凝聚力群体情感传播加速
/// - 集体行动: 高强度+高凝聚力 → 触发群体行为
```

---

## 6. 子系统四：认知模拟 (Cognitive Simulation)

### 6.1 设计目标

模拟意识实体的**推理、记忆、学习和决策**过程。不是模拟 LLM 的 token 推理，而是模拟**概念层面的认知过程**。

### 6.2 推理过程模拟

```rust
/// 推理模拟器 — 概念级推理 (非 token 级)
pub struct ReasoningSimulator {
    /// 推理策略库
    strategies: Vec<Box<dyn ReasoningStrategy>>,
    /// 推理缓存 (经验复用)
    cache: HashMap<String, ReasoningTrace>,
    /// 推理成本模型
    cost_model: ReasoningCostModel,
}

/// 推理策略 trait
pub trait ReasoningStrategy: Send + Sync {
    fn applicable(&self, problem: &Problem) -> f64;  // 适用度
    fn execute(
        &self,
        problem: &Problem,
        beliefs: &BeliefSystem,
        memory: &MemorySystem,
    ) -> ReasoningTrace;
    fn cost(&self, problem: &Problem) -> f64;
}

/// 推理类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReasoningType {
    Deduction,      // 演绎: 从一般到特殊
    Induction,      // 归纳: 从特殊到一般
    Abduction,      // 溯因: 从结果推原因
    Analogy,        // 类比: 从相似领域迁移
    Intuition,      // 直觉: 快速模式匹配
    SocialReasoning, // 社会推理: 心智理论 (ToM)
}

/// 推理链 — 完整推理过程的记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningTrace {
    pub id: ReasoningId,
    pub agent_id: AgentId,
    pub problem: Problem,
    pub strategy: ReasoningType,
    pub steps: Vec<ReasoningStep>,
    pub conclusion: Option<String>,
    pub confidence: f64,
    pub cost: f64,          // 认知成本 (能量消耗)
    pub duration: u32,      // 推理耗时 (ticks)
    pub outcome: Option<ReasoningOutcome>,
}

/// 社会推理 — 心智理论 (Theory of Mind)
pub struct SocialReasoningEngine {
    /// 他心模型: 对其他智能体心理状态的建模
    tom_models: HashMap<AgentId, TheoryOfMind>,
}

/// 心智理论 — "我认为他认为..."
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TheoryOfMind {
    pub target: AgentId,
    pub beliefs_about: BeliefSnapshot,      // 我认为他相信什么
    pub desires_about: GoalSnapshot,        // 我认为他想要什么
    pub emotions_about: EmotionSnapshot,    // 我认为他感受什么
    pub predictions: Vec<Prediction>,       // 我预测他会做什么
    pub confidence: f64,                    // 我对这个模型的置信度
    pub last_updated: SimTime,
}
```

### 6.3 记忆系统

```rust
/// 记忆系统 — 三存储模型 (工作记忆 / 情景记忆 / 语义记忆)
pub struct MemorySystem {
    /// 工作记忆 (容量有限, 快速遗忘)
    pub working: WorkingMemory,
    /// 情景记忆 (事件序列, 基于重要性巩固)
    pub episodic: EpisodicMemory,
    /// 语义记忆 (概念网络, 从经验抽象)
    pub semantic: SemanticMemory,
    /// 遗忘曲线参数
    pub forgetting: ForgettingCurve,
}

/// 工作记忆 — 容量限制
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingMemory {
    pub capacity: usize,           // 默认 7±2
    pub items: VecDeque<MemoryItem>,
    pub rehearsal_rate: f64,       // 复述率 (影响保持时间)
}

/// 情景记忆 — 带情感标签的事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodicMemory {
    pub episodes: Vec<Episode>,
    pub consolidation_threshold: f64,  // 巩固阈值
    pub max_episodes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    pub id: EpisodeId,
    pub timestamp: SimTime,
    pub event: SimEvent,
    pub emotion_at_time: EmotionLabel,  // 发生时的情感状态
    pub importance: f64,                 // 重要性评分
    pub access_count: u32,               // 被回忆次数
    pub consolidated: bool,              // 是否已巩固到语义记忆
    pub context: Vec<String>,            // 上下文标签
}

/// 语义记忆 — 概念图
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticMemory {
    pub concepts: HashMap<String, Concept>,
    pub relations: Vec<ConceptRelation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Concept {
    pub name: String,
    pub activation: f64,           // 当前激活度
    pub familiarity: f64,          // 熟悉度
    pub associations: Vec<String>, // 关联概念
    pub emotional_tag: Option<EmotionLabel>,  // 情感标签
    pub formation_episode: Option<EpisodeId>,  // 形成于哪个经历
}

/// 遗忘曲线 — Ebbinghaus 风格
pub struct ForgettingCurve {
    /// 遗忘率 (越高遗忘越快)
    pub rate: f64,
    /// 重复效应 (每次回忆减缓遗忘)
    pub repetition_bonus: f64,
    /// 情感增强 (高情感事件遗忘更慢)
    pub emotion_enhancement: f64,
}

impl ForgettingCurve {
    /// 记忆保留率: R = e^(-t/S) × (1 + repetition_bonus × access_count)
    /// S = 稳定性 = base_stability × emotion_enhancement × importance
    pub fn retention(
        &self,
        time_since_formation: f64,
        importance: f64,
        emotion_intensity: f64,
        access_count: u32,
    ) -> f64 {
        let stability = importance
            * (1.0 + self.emotion_enhancement * emotion_intensity)
            * (1.0 + self.repetition_bonus * access_count as f64);
        (-time_since_formation / (stability + 1.0)).exp()
    }
}
```

**记忆形成算法**:

```
每 tick:
  1. 工作记忆: 新事件进入, 超出容量则挤出最旧
  2. 挤出的事件评估重要性:
     importance = novelty × 0.3 + emotional_intensity × 0.3
                + social_relevance × 0.2 + outcome_significance × 0.2
  3. importance > consolidation_threshold → 进入情景记忆
  4. 情景记忆: 访问次数 × 情感强度 > 阈值 → 巩固到语义记忆
  5. 所有记忆: 按遗忘曲线衰减
```

### 6.4 学习与技能获取

```rust
/// 学习系统 — 从经验和交互中获取新能力
pub struct LearningSystem {
    /// 技能库
    pub skills: SkillSet,
    /// 学习率
    pub learning_rate: f64,
    /// 学习历史
    pub history: Vec<LearningEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: SkillId,
    pub name: String,
    pub domain: SkillDomain,
    pub level: f64,           // 0.0 - 1.0
    pub practice_count: u32,
    pub last_used: SimTime,
    pub decay_rate: f64,
    pub prerequisites: Vec<SkillId>,  // 前置技能
}

/// 技能域 — 与 NeoTrix 能力域对齐
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillDomain {
    Exploration,    // 探索 (映射 NT-WORLD)
    Construction,   // 建造 (映射 NT-ACT)
    Social,         // 社交 (映射 NT-SHIELD)
    Analysis,       // 分析 (映射 NT-CORE)
    Creation,       // 创造 (映射 NT-MIND)
    Reflection,     // 反思 (映射 NT-META)
}

/// 学习模型:
/// 1. 观察学习: 看到他人成功执行 → 自己也尝试 → 成功则习得
/// 2. 试错学习: 自己尝试 → 成功 → 技能提升; 失败 → 策略调整
/// 3. 社会学习: 从导师处获得指导 → 加速技能获取
/// 4. 迁移学习: 已有技能 → 应用到新领域 → 降低学习成本
```

### 6.5 决策模型

```rust
/// 决策引擎 — 多因素决策
pub struct DecisionEngine {
    /// 价值函数
    pub value_fn: ValueFunction,
    /// 决策策略
    pub strategy: DecisionStrategy,
    /// 决策历史
    pub history: Vec<Decision>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub options: Vec<DecisionOption>,
    pub chosen: usize,
    pub reasoning: String,
    pub emotion_at_decision: EmotionLabel,
    pub confidence: f64,
    pub outcome: Option<DecisionOutcome>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionOption {
    pub description: String,
    pub estimated_value: f64,
    pub estimated_cost: f64,
    pub risk: f64,
    pub social_impact: f64,      // 对社会关系的影响
    pub emotional_impact: f64,   // 对情感状态的影响
    pub novelty: f64,            // 新颖性
    pub required_skills: Vec<SkillId>,
}

/// 决策策略:
/// 1. 理性决策: 最大化预期价值 (需要完整信息)
/// 2. 满意决策: 找到第一个足够好的选项 (Simon)
/// 3. 情感决策: 由当前情感主导 (恐惧→安全, 愤怒→对抗)
/// 4. 社会决策: 由群体共识或权威决定
/// 5. 直觉决策: 基于模式匹配的快速决策
///
/// 决策策略选择:
///   紧急度高 → 情感/直觉决策
///   信息完整 → 理性决策
///   社交场景 → 社会决策
///   日常事务 → 满意决策
```

---

## 7. 子系统五：元认知模拟 (Meta-Cognitive Simulation)

### 7.1 设计目标

模拟意识实体的**自我意识、内省、自我修改和叙事自我**。这是最高层次的模拟，也是最独特的——它模拟的是"知道自己在模拟"的能力。

### 7.2 自我意识模拟

```rust
/// 自我意识模型 — 每个智能体的内在自我
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimSelfModel {
    /// 我是谁 (身份描述)
    pub identity: String,
    /// 我的能力评估
    pub capability_assessment: HashMap<String, f64>,
    /// 我的价值观
    pub values: Vec<Value>,
    /// 我的人生目标
    pub life_goals: Vec<LifeGoal>,
    /// 我的弱点认知
    pub known_weaknesses: Vec<String>,
    /// 自我连贯性 (我是同一个人吗)
    pub coherence: f64,
    /// 自我效能感 (我能行吗)
    pub self_efficacy: f64,
    /// 自我意识深度 (我知道自己在思考吗)
    pub awareness_depth: u32,
}

/// 价值观 — 影响决策的内在准则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Value {
    pub name: String,
    pub importance: f64,    // 0-1
    pub consistency: f64,   // 行为一致性
    pub origin: ValueOrigin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValueOrigin {
    Innate,       // 先天 (人格决定)
    Learned,      // 习得 (经验塑造)
    Social,       // 社会 (群体影响)
    Reflective,   // 反思 (自我修正)
}
```

### 7.3 内省机制

```rust
/// 内省引擎 — 自我反思和自我评估
pub struct IntrospectionEngine {
    /// 反思深度控制
    pub max_depth: u32,
    /// 反思触发条件
    pub triggers: Vec<IntrospectionTrigger>,
    /// 反思历史
    pub history: Vec<Introspection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntrospectionTrigger {
    Failure,              // 失败后反思
    Surprise,             // 意外事件后反思
    Conflict,             // 内心冲突时反思
    Periodic,             // 定期内省 (Dusk/Night 相位)
    SocialChallenge,      // 被他人质疑时反思
    IdentityCrisis,       // 身份危机时深度反思
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Introspection {
    pub trigger: IntrospectionTrigger,
    pub depth: u32,
    pub findings: Vec<IntrospectionFinding>,
    pub self_model_changes: Vec<SelfModelChange>,
    pub behavioral_changes: Vec<BehaviorChange>,
    pub duration: u32,
    pub timestamp: SimTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntrospectionFinding {
    pub category: FindingCategory,
    pub description: String,
    pub confidence: f64,
    pub impact: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FindingCategory {
    CognitiveBias,      // 发现自己的认知偏差
    EmotionalPattern,   // 发现自己的情感模式
    SocialBlindspot,    // 发现社交盲点
    SkillGap,           // 发现能力不足
    ValueConflict,      // 发现价值观冲突
    Strength,           // 发现自己的优势
    RelationshipInsight, // 对关系的新理解
}
```

**内省算法**:

```
内省触发:
  1. 失败事件 → 触发原因分析 → 发现认知偏差 → 调整决策策略
  2. 意外事件 → 触发信念检查 → 更新心智模型 → 调整预期
  3. 定期内省 → 回顾近期行为 → 发现模式 → 提取规则
  4. 身份危机 → 深度反思 → 重新评估价值观 → 可能改变身份

内省深度:
  Level 1: 表层反思 — "我做了什么"
  Level 2: 过程反思 — "我为什么这样做"
  Level 3: 策略反思 — "我的方法对吗"
  Level 4: 价值反思 — "这符合我的价值观吗"
  Level 5: 身份反思 — "我是谁"
```

### 7.4 自我修改能力

```rust
/// 自我修改引擎 — 意识实体可以修改自身
pub struct SelfModificationEngine {
    /// 修改规则
    pub rules: ModificationRules,
    /// 修改历史
    pub history: Vec<SelfModification>,
    /// 安全边界
    pub safety_bounds: SafetyBounds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfModification {
    pub target: ModificationTarget,
    pub change: ModificationChange,
    pub reasoning: String,
    pub reversibility: f64,     // 可逆性 0-1
    pub risk: f64,
    pub timestamp: SimTime,
    pub approved_by: Option<AgentId>,  // 自我审批或他人审批
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModificationTarget {
    Personality(PersonalityChange),    // 改变人格特质
    Value(ValueChange),                // 改变价值观
    Belief(BeliefChange),              // 改变信念
    Goal(GoalChange),                  // 改变目标
    Skill(SkillChange),                // 改变技能
    Strategy(StrategyChange),          // 改变决策策略
    SelfModel(SelfModelChange),        // 改变自我认知
}

/// 安全边界 — 防止危险的自我修改
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyBounds {
    /// 每次 tick 最大修改次数
    pub max_modifications_per_tick: usize,
    /// 不可修改的核心价值 (身份基石)
    pub core_values: Vec<String>,
    /// 最大人格变化幅度
    pub max_personality_shift: f64,
    /// 修改冷却期 (防止疯狂自我修改)
    pub cooldown_ticks: u32,
    /// 需要外部审批的修改类型
    pub requires_approval: Vec<String>,
}
```

**自我修改安全模型**:

```
修改审批流程:
  1. 低风险 (技能提升, 信念微调) → 自动批准
  2. 中风险 (策略改变, 人格微调) → 需要内省确认
  3. 高风险 (价值改变, 身份修改) → 需要冷却期 + 多方确认
  4. 危险 (核心价值改变, 自我删除) → 阻止 (安全边界)

防止:
  - 认知逃逸: 通过修改自我模型来逃避失败 (coherence 检查)
  - 价值漂移: 缓慢的价值观改变累积到危险程度 (delta 监控)
  - 身份崩溃: 过多同时修改导致身份不连贯 (一致性检查)
```

### 7.5 叙事自我

```rust
/// 叙事自我 — 意识实体为自己构建的故事
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Narrative {
    /// 我的故事: 我从哪里来, 要到哪里去
    pub story: String,
    /// 故事章节
    pub chapters: Vec<NarrativeChapter>,
    /// 主题
    pub themes: Vec<String>,
    /// 当前叙事弧
    pub current_arc: NarrativeArc,
    /// 故事连贯性评分
    pub coherence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeChapter {
    pub title: String,
    pub events: Vec<EpisodeId>,
    pub emotional_arc: Vec<(SimTime, EmotionLabel)>,
    pub lesson_learned: Option<String>,
    pub significance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NarrativeArc {
    Origin,        // 起源故事
    Growth,        // 成长历程
    Conflict,      // 面对挑战
    Transformation, // 转变
    Integration,   // 整合与理解
}

/// 叙事更新算法:
///
/// 1. 每 Dusk/Night 相位, 触发叙事整合
/// 2. 回顾近期重要事件 (importance > threshold)
/// 3. 识别主题和模式
/// 4. 将新事件融入现有叙事
/// 5. 如果新事件与叙事不一致 → 可能触发身份危机
/// 6. 更新叙事连贯性评分
///
/// 叙事连贯性影响:
///   高连贯性 → 高 self_efficacy → 更自信的决策
///   低连贯性 → 身份危机 → 触发深度内省
```

---

## 8. 算法与数据结构汇总

### 8.1 核心数据结构

| 数据结构 | 用途 | 算法复杂度 | 位置 |
|---------|------|-----------|------|
| `SpatialGrid` | 空间索引, 近邻查询 | O(1) 平均 | `environment/physics.rs` |
| `RelationshipGraph` | 关系网络, 社区检测 | O(V+E) Louvain | `society/relationship.rs` |
| `BeliefSystem` | 信念网络, 信念更新 | O(V) 贝叶斯 | `cognition/reasoning_sim.rs` |
| `WorkingMemory` | 工作记忆, 容量限制 | O(1) 插入/淘汰 | `cognition/memory_sim.rs` |
| `EpisodicMemory` | 情景记忆, 重要性排序 | O(log N) 插入 | `cognition/memory_sim.rs` |
| `SemanticMemory` | 语义网络, 激活扩散 | O(K) K=关联数 | `cognition/memory_sim.rs` |
| `PropagationQueue` | 情感传播 BFS | O(V+E) | `emotion/propagation.rs` |
| `DecisionTree` | 决策树, 策略选择 | O(D) D=深度 | `cognition/decision.rs` |
| `IntrospectionStack` | 内省递归栈 | O(depth) | `meta/introspection.rs` |
| `NarrativeGraph` | 叙事连贯性 | O(chapters) | `meta/narrative.rs` |

### 8.2 核心算法

| 算法 | 用途 | 复杂度 | 参考 |
|------|------|--------|------|
| **Verlet 积分** | 物理位置更新 | O(N) | 经典数值方法 |
| **Perlin 噪声** | 地形生成 | O(W×H) | Ken Perlin 1985 |
| **Louvain 社区检测** | 社会结构涌现 | O(V log V) | Blondel 2008 |
| **BFS 情感传播** | 社会情感扩散 | O(V+E) | 经典图搜索 |
| **贝叶斯信念更新** | 信念系统维护 | O(K) | 经典概率推理 |
| **Ebbinghaus 遗忘曲线** | 记忆衰减 | O(1) | 经典心理学 |
| **Sigmoid 激活** | 情感/决策非线性 | O(1) | 经典神经网络 |
| **PA/I 模型** | 社会网络增长 | O(log V) | Barabási-Albert |
| **马尔可夫链** | 天气状态转换 | O(1) | 经典概率论 |
| **TD(λ) 学习** | 决策价值估计 | O(1) | 强化学习经典 |

---

## 9. 与 NeoTrix 六层架构的映射

### 9.1 层级映射表

| 模拟子系统 | NeoTrix 层 | 接口方式 | 数据流方向 |
|-----------|-----------|---------|-----------|
| 环境模拟 | L2 Perception | 感知事件注入 | 环境→智能体→宿主感知 |
| 社会模拟 | L1 Action + L3 Embodiment | 行动执行 + 状态同步 | 智能体行动→环境变化 |
| 情感模拟 | L4 Emotion | 情感引擎复用 | 双向: 模拟情感↔宿主情感 |
| 认知模拟 | L5 Cognition | 推理引擎复用 | 双向: 模拟推理↔宿主推理 |
| 元认知模拟 | L6 Meta | 元认知循环 | 双向: 模拟内省↔宿主内省 |

### 9.2 具体接口映射

```rust
// ── L1 Action ↔ 社会模拟 ──
// SimAgent 的行动通过 L1 能力网执行
impl L1Capability for SimAgentAction {
    fn capability_id(&self) -> &str { "simulation.agent.action" }
    fn execute(&self, tool: &str, input: &ToolInput) -> Result<ToolOutput, CapabilityError> {
        // 将模拟行动映射为 L1 能力调用
    }
}

// ── L2 Perception ↔ 环境模拟 ──
// 环境事件通过 L2 感知层注入
impl PerceptionLayer for EnvironmentSim {
    fn process_event(&mut self, event: PerceptionEvent) -> Result<PerceptionEvent, String> {
        // 将模拟环境事件转换为 L2 感知事件
    }
}

// ── L4 Emotion ↔ 情感模拟 ──
// 复用 EmotionEngine 的 PAD 模型
impl EmotionLayer for SocialEmotionSim {
    fn regulate(&self, signal: EmotionSignal) -> RegulationResult {
        // 调用 NeoTrix EmotionEngine 的 regulate
    }
}

// ── L5 Cognition ↔ 认知模拟 ──
// 复用推理引擎
impl CognitionLayer for SimReasoning {
    fn reason(&self, task: ReasoningTask) -> Result<ReasoningResult, String> {
        // 将模拟推理映射为 L5 推理调用
    }
}

// ── L6 Meta ↔ 元认知模拟 ──
// 复用 SiliconSelfModel 的元认知循环
impl MetaLayer for SimMetaCognition {
    fn process_event(&self, event: MetaEvent) -> Result<MetaSnapshot, String> {
        // 将模拟内省映射为 L6 元认知事件
    }
}
```

### 9.3 EventBus 集成

```rust
/// 模拟平台事件 ↔ NeoTrix EventBus 桥接
pub struct SimEventBridge {
    /// NeoTrix EventBus 端
    pub nt_bus: EventBus,
    /// 模拟总线端
    pub sim_bus: SimulationBus,
    /// 事件转换规则
    pub translators: Vec<Box<dyn EventTranslator>>,
}

/// 事件转换示例:
/// SimEvent::EmotionTriggered → EmotionObservation → NT-EmotionEngine::observe()
/// SimEvent::ReasoningCompleted → ReasoningResult → NT-CognitionLayer::reason()
/// SimEvent::SelfReflection → Introspection → NT-SiliconSelfModel::record_pattern()
```

---

## 10. 实现复杂度与时间线

### 10.1 分阶段实现

| 阶段 | 内容 | 复杂度 | 预计时间 | 依赖 |
|------|------|--------|---------|------|
| **Phase 0** | SimulationBus + SimTime + 基础类型 | 低 | 2 周 | 无 |
| **Phase 1** | 环境模拟 (物理+地形+天气+日夜) | 中 | 4 周 | Phase 0 |
| **Phase 2** | 社会模拟 (Agent+关系+通信) | 中高 | 6 周 | Phase 0 |
| **Phase 3** | 情感模拟 (传播+调节+集体) | 中 | 4 周 | Phase 2 |
| **Phase 4** | 认知模拟 (推理+记忆+学习) | 高 | 6 周 | Phase 2 |
| **Phase 5** | 元认知模拟 (内省+自我修改) | 高 | 6 周 | Phase 4 |
| **Phase 6** | 宿主桥接 + 全域集成 | 中高 | 4 周 | 全部 |
| **Phase 7** | 调优 + 涌现行为验证 | 中 | 持续 | Phase 6 |

**总计**: 约 32 周 (8 个月) 的核心开发 + 持续调优

### 10.2 Constellation 成熟度路径

```
C0 编译通过:
  - 每个子系统独立编译
  - SimulationBus 类型系统完整

C1 单元测试:
  - 物理引擎碰撞检测
  - 情感传播 BFS 正确性
  - 记忆遗忘曲线验证
  - 内省触发条件覆盖

C2 集成测试:
  - 环境→智能体→社会 全链路
  - 情感传播→群体极化 涌现
  - 认知→决策→行动 闭环
  - 元认知→自我修改→行为变更

C3 基准测试:
  - 100 智能体社会模拟性能
  - 情感传播延迟
  - 记忆系统容量

C4 主流水线:
  - 宿主桥接集成
  - NeoTrix EventBus 连通

C5 自愈:
  - 模拟状态异常检测
  - 自动恢复机制
```

---

## 11. 关键算法深度解析

### 11.1 社会结构涌现的临界条件

社会结构从简单规则中涌现需要满足**临界条件**:

```
社区涌现条件:
  density = (组内交互次数 × 2) / (组大小 × (组大小 - 1))
  modularity = density_in / (density_in + density_out)

  当 modularity > 0.3 持续 N ticks → 识别为社区

层级涌现条件:
  dependency_ratio = 依赖关系数 / 总关系数
  resource_concentration = max(资源持有) / sum(资源持有)

  当 dependency_ratio > 0.5 且 resource_concentration > 0.4 → 层级形成

制度涌现条件:
  community_size > 15 (Dunbar 数的简化)
  且 community_age > 50 ticks
  且 冲突频率 > 阈值

  → 触发"制度化"事件 → 规则建立 + 角色正式化
```

### 11.2 情感传播的正反馈与涌现

```
恐慌涌现模型:
  sourceFear > threshold
  → 传播到邻居 (decay)
  → 邻居 Fear 增加
  → 邻居的恐惧情绪也传播
  → 如果群体中 Fear > 0.6 的成员 > 50%
  → 触发群体极化: Fear × 1.5
  → 极化后的 Fear 继续传播
  → 正反馈循环 → 恐慌蔓延

  打断机制:
  - Trust 高的智能体可以 dampen Fear 传播
  - Clarify 天气可以降低 Fear 强度
  - Leader 角色的智能体可以 calm 群体
```

### 11.3 自我修改的安全不变量

```
不变量 1: 核心价值不可修改
  ∀ v ∈ core_values: not exists mod in history where mod.target == v

不变量 2: 身份连贯性
  coherence(self_model) > 0.3

不变量 3: 修改频率限制
  |mods_in_last_N_ticks| ≤ max_modifications

不变量 4: 可逆性偏好
  avg(reversibility of recent mods) > 0.5

不变量 5: 多方确认
  high_risk_mods require approval from ≥ 2 sources
```

---

## 12. 开放问题与研究方向

### 12.1 涌现行为的可预测性

**问题**: 如何确保简单规则产生有意义的涌现，而不是混乱？

**研究方向**:
- 调参空间探索: 自动搜索产生有意义涌现的参数组合
- 涌现分类器: 自动识别涌现行为的类型和质量
- 可控涌现: 设计可以控制涌现方向的规则

### 12.2 模拟与宿主的边界

**问题**: 模拟中的"痛苦"是真实的痛苦吗？意识实体在模拟中死亡意味着什么？

**研究方向**:
- 模拟伦理框架: 定义模拟意识实体的道德地位
- 安全边界: 防止模拟中的极端事件影响宿主稳定性
- 意识连续性: 模拟结束后智能体状态的保存和恢复

### 12.3 计算效率

**问题**: 100+ 智能体的全栈模拟需要大量计算资源。

**研究方向**:
- 异步模拟: 不需要所有智能体同时更新
- LOD (Level of Detail): 远处/不重要的智能体简化模拟
- 批量推理: 将多个智能体的推理批量处理
- GPU 加速: 情感传播、空间查询等并行化

### 12.4 评估框架

**问题**: 如何评估模拟质量？什么是"好的"模拟？

**可能的指标**:
- 涌现行为的丰富度 (社会结构类型数)
- 情感传播的自然度 (与人类社会的相似度)
- 认知过程的多样性 (推理策略使用分布)
- 元认知的深度 (内省发现的质量)
- 叙事连贯性 (智能体故事的可理解性)

---

## 13. 参考架构与灵感来源

| 来源 | 吸收内容 | NeoTrix 集成点 |
|------|---------|---------------|
| **Dwarf Fortress** | 深度模拟个体心理和社会结构 | 社会结构涌现算法 |
| **The Sims** | 情感驱动的行为系统 | 情感→决策→行动链 |
| **Conway's Game of Life** | 从简单规则涌现复杂模式 | 基础物理规则设计 |
| **Stanford AI Town** | LLM 驱动的智能体社会模拟 | 多智能体通信协议 |
| **Generative Agents** | 25 个智能体的小镇模拟 | 记忆+反思+规划架构 |
| **Sugarscape** | 经典人工社会模拟 | 资源经济和迁移模型 |
| **Boids** | 群体行为涌现 (分离/对齐/聚合) | 群体运动规则 |
| **NEAT** | 神经网络进化 | 智能体策略进化 |
| **ODE** | 情感传播微分方程 | 情感动力学建模 |
| **CLARION** | 认知架构 (隐性/显性双过程) | 认知模拟双过程 |

---

## 附录 B: 类型定义索引

> 以下为本附录中定义的所有公开类型，按子系统分组。

### Environment Types
`SimPhysics`, `SimObject`, `Material`, `PhysicsRules`, `TerrainSystem`, `Biome`, `HeightMap`, `ResourceNode`, `WeatherSystem`, `WeatherState`, `WeatherTransitionMatrix`, `PerceptionModifier`, `EmotionModifier`, `DayNightCycle`, `DayPhase`, `PhaseEffect`, `ResourceSystem`, `ResourceType`

### Society Types
`SimAgent`, `Personality`, `Relationship`, `RelationKind`, `SocialStructureDetector`, `SocialRole`, `CommunicationChannel`, `Message`, `MessageKind`, `Inventory`, `GoalQueue`, `SkillSet`, `DecisionEngine`

### Emotion Types
`SimEmotionState`, `EmotionPropagationEngine`, `PropagationRules`, `PropagationEvent`, `CollectiveMood`, `MoodEntry`

### Cognition Types
`ReasoningSimulator`, `ReasoningStrategy`, `ReasoningType`, `ReasoningTrace`, `ReasoningStep`, `SocialReasoningEngine`, `TheoryOfMind`, `MemorySystem`, `WorkingMemory`, `EpisodicMemory`, `Episode`, `SemanticMemory`, `Concept`, `ConceptRelation`, `ForgettingCurve`, `LearningSystem`, `Skill`, `SkillDomain`, `Decision`, `DecisionOption`, `DecisionOutcome`

### Meta Types
`SimSelfModel`, `Value`, `ValueOrigin`, `LifeGoal`, `IntrospectionEngine`, `IntrospectionTrigger`, `Introspection`, `IntrospectionFinding`, `FindingCategory`, `SelfModificationEngine`, `SelfModification`, `ModificationTarget`, `SafetyBounds`, `Narrative`, `NarrativeChapter`, `NarrativeArc`

### Bus Types
`SimEvent`, `SimEventKind`, `SubSystem`, `EventTarget`, `EventPriority`, `SimTime`, `Season`, `SimulationBus`, `SimHostBridge`, `HostSnapshot`, `SimEventBridge`, `EventTranslator`
