# NT-GAME 引擎特性吸收计划

**日期**: 2026-09-20
**目标**: 逐条吸收 Godot 4 / Unity / UE5 的核心优势，补齐 NT-GAME 引擎的缺陷

---

## 一、NT-GAME 现有能力盘点与缺口分析

### 已有能力（无需重建）

| 模块 | 能力 | 成熟度 |
|------|------|--------|
| `ecs/` | Entity/Component/Store/System 调度 | ⭐⭐ 基础可用 |
| `rpg/` | 角色属性/装备/技能树 | ⭐⭐⭐ 完整 |
| `world/combat/` | 战斗状态机/伤害计算 | ⭐⭐⭐ 完整 |
| `world/npc/` | NPC 状态/关系/对话树 | ⭐⭐⭐ 完整 |
| `world/quest/` | 任务系统/战利品表 | ⭐⭐⭐ 完整 |
| `world/tilemap/` | 瓦片地图/分块/分层 | ⭐⭐⭐ 完整 |
| `world/generation/` | 噪声生成/生态群落/POI | ⭐⭐⭐ 完整 |
| `world/weather/` | 天气状态机 | ⭐⭐ 可用 |
| `world/season/` | 季节循环 | ⭐⭐ 可用 |
| `world/time_system/` | 游戏时钟 | ⭐⭐ 可用 |
| `world/audio/` | 音频管理器 | ⭐ 基础 |
| `world/input/` | 输入管理器 | ⭐ 基础 |
| `world/particles/` | 粒子系统 | ⭐ 基础 |
| `world/minimap/` | 小地图 | ⭐ 基础 |
| `world/notification/` | 通知系统 | ⭐ 基础 |
| `ai/` | 行为树（Sequence/Selector/Inverter） | ⭐⭐ 可用 |
| `consciousness/` | 意识-游戏桥 | ⭐⭐⭐ 独有 |
| `evolution.rs` | 自主训练循环 | ⭐⭐⭐ 独有 |
| `framework.rs` | Actor/Episode/Rubric | ⭐⭐⭐ 独有 |
| `mcp.rs` | MCP 工具接口 | ⭐⭐⭐ 独有 |

### 关键缺口（需从三大引擎吸收）

| 缺口 | 来源引擎 | 优先级 | 预估工作量 |
|------|---------|--------|-----------|
| **渲染系统** | 三者皆有 | P0 | 高 |
| **场景树/节点组合** | Godot | P1 | 中 |
| **信号/事件系统** | Godot | P1 | 低 |
| **资源/资产管线** | Godot + Unity | P1 | 中 |
| **UI 系统** | Godot + Unity | P1 | 中 |
| **状态机（游戏状态）** | Unity | P1 | 低 |
| **动画系统** | Godot + Unity | P2 | 中 |
| **物理/碰撞** | 三者皆有 | P2 | 低 |
| **游戏能力系统（GAS）** | UE5 | P2 | 中 |
| **增强输入** | UE5 | P2 | 低 |
| **存档系统** | Unity + Godot | P1 | 低 |
| **热重载** | Godot | P2 | 中 |
| **定时器/延迟系统** | Unity | P2 | 低 |
| **相机系统** | 三者皆有 | P2 | 低 |
| **光照系统** | Godot + UE5 | P3 | 高 |
| **关卡流** | UE5 | P3 | 中 |
| **本地化** | Godot + Unity | P3 | 低 |

---

## 二、逐项吸收方案

### 1. 渲染系统 — 吸收 Godot 的分层渲染 + macroquad

**Godot 4 做法**: SceneTree → RenderingServer → Drivers（Vulkan/OpenGL），场景树只发命令，渲染在独立线程。

**Unity 做法**: URP/HDRP 通过 Render Graph 管理渲染管线。

**NT-GAME 吸收方案**:

```
┌──────────────────────────────────────┐
│  RenderWorld（新增模块）              │
│  ├── Camera2D（吸收 Godot Camera2D）  │
│  ├── SpriteRenderer（即时模式）       │
│  ├── TileMapRenderer（优化瓦片绘制）  │
│  ├── ParticleRenderer（粒子绘制）     │
│  ├── UIRenderer（egui 集成）         │
│  └── PostProcess（可选后处理）        │
├──────────────────────────────────────┤
│  macroquad（底层渲染）               │
│  └── 即时模式 API，零全局状态        │
└──────────────────────────────────────┘
```

**Rust 实现要点**:
```rust
// render/camera.rs — 吸收 Godot Camera2D 的跟随/缩放
pub struct Camera2D {
    pub position: Vec2,
    pub zoom: f32,
    pub target: Option<Entity>,  // 跟随目标
    pub smoothing: f32,          // 平滑插值
    pub bounds: Option<Rect>,    // 限制范围
}

// render/sprite.rs — 吸收 Godot Sprite2D 的锚点/翻转
pub struct SpriteComponent {
    pub texture: Handle<Texture>,
    pub region: Rect,            // 精灵表区域
    pub anchor: Vec2,            // 锚点
    pub flip_x: bool,
    pub flip_y: bool,
    pub color: Color,
    pub z_index: i32,
    pub visible: bool,
}
```

**验证**: `cargo check` 通过 + macroquad 窗口可渲染精灵

---

### 2. 场景树 / 节点组合 — 吸收 Godot 的 SceneTree

**Godot 4 做法**: 一切都是 Node，Node 组成 SceneTree。Node 可以挂载脚本（GDScript），场景可以实例化为子树。

**NT-GAME 吸收方案**: 不用 Godot 的 Node 继承体系，用 ECS + 组合模式。

```
Godot Node 继承树          NT-GAME ECS 组合
─────────────────          ─────────────────
Node (基类)                Entity（实体 ID）
├── Node2D                 ├── Transform2D 组件
│   ├── Sprite2D           ├── SpriteComponent 组件
│   ├── Camera2D           ├── Camera2D 组件
│   ├── TileMap            ├── TileMap 组件
│   └── Area2D             └── Collider 组件
├── Control (UI)           └── ...
└── AudioStreamPlayer      └── AudioComponent 组件
```

**关键设计 — Prefab 系统（吸收 Godot .tscn）**:
```rust
/// 预制体：一组组件的模板（类似 Godot 的 .tscn 场景文件）
pub struct Prefab {
    pub name: String,
    pub components: Vec<ComponentDescriptor>,
    pub children: Vec<Prefab>,  // 嵌套子实体
}

/// 场景：世界中的一个实例化单元
pub struct SceneInstance {
    pub root: Entity,
    pub children: Vec<Entity>,
    pub prefab_name: Option<String>,
}

/// 世界管理器：吸收 Godot 的 SceneTree
pub struct SceneManager {
    scenes: HashMap<String, Prefab>,
    instances: Vec<SceneInstance>,
    root: Entity,
}
```

**RON 数据格式（吸收 Godot .tscn）**:
```ron
// assets/prefabs/player.ron
Prefab(
    name: "player",
    components: [
        Transform2D(position: (100.0, 200.0), rotation: 0.0),
        SpriteComponent(texture: "player.png", region: (0, 0, 32, 32)),
        Collider(shape: Rectangle((0.0, 0.0, 32.0, 32.0)), sensor: false),
        Health(max: 100.0),
        Movement(speed: 200.0),
    ],
    children: [
        Prefab(
            name: "weapon_point",
            components: [
                Transform2D(position: (16.0, 0.0)),
            ],
        ),
    ],
)
```

**验证**: 能从 RON 文件加载预制体并实例化到世界中

---

### 3. 信号/事件系统 — 吸收 Godot Signals + Unity Events

**Godot 4 做法**: 信号是观察者模式的一等公民。节点 `emit_signal("damage_taken", 50)`，其他节点 `connect("damage_taken", self, "on_damage")`。

**Unity 做法**: `UnityEvent<T>` + `UnityAction<T>`，Inspector 中可视化绑定。

**NT-GAME 吸收方案**: 类型安全的事件总线 + ECS 事件组件。

```rust
// event/mod.rs — 吸收 Godot Signals 的类型安全版本
pub struct EventBus {
    handlers: HashMap<TypeId, Vec<Box<dyn Fn(&dyn Any) -> ()>>>,
}

impl EventBus {
    pub fn subscribe<T: 'static>(&mut self, handler: impl Fn(&T) + 'static) {
        let type_id = TypeId::of::<T>();
        self.handlers
            .entry(type_id)
            .or_default()
            .push(Box::new(move |any| {
                if let Some(event) = any.downcast_ref::<T>() {
                    handler(event);
                }
            }));
    }

    pub fn emit<T: 'static>(&self, event: &T) {
        if let Some(handlers) = self.handlers.get(&TypeId::of::<T>()) {
            for handler in handlers {
                handler(event);
            }
        }
    }
}

// 预定义事件类型
pub struct DamageEvent {
    pub source: Entity,
    pub target: Entity,
    pub amount: f64,
    pub damage_type: DamageType,
}

pub struct DialogueEvent {
    pub npc: Entity,
    pub node_id: u32,
    pub response_index: usize,
}

pub struct ItemPickupEvent {
    pub entity: Entity,
    pub item: Item,
}

pub struct LevelUpEvent {
    pub entity: Entity,
    pub new_level: u32,
}
```

**与现有代码的集成**:
- `combat.rs` → 发射 `DamageEvent`
- `dialogue.rs` → 发射 `DialogueEvent`
- `quest.rs` → 发射 `QuestCompleteEvent`
- 意识系统 → 订阅所有事件，更新 GWT 权重

**验证**: 事件能正确发布/订阅，意识系统能感知游戏事件

---

### 4. 资源/资产管线 — 吸收 Godot Resource + Unity Addressables

**Godot 4 做法**: `Resource` 是基类，`load("res://icon.png")` 自动加载。资源在内存中缓存，引用计数管理生命周期。

**Unity 做法**: Addressables 按地址异步加载，支持远程资源。

**NT-GAME 吸收方案**: Handle + AssetServer + 异步加载。

```rust
// asset/mod.rs — 吸收 Godot Resource 系统
pub struct Handle<T> {
    id: AssetId,
    _marker: PhantomData<T>,
}

pub struct AssetServer {
    cache: HashMap<AssetId, Box<dyn Any>>,
    loading: HashMap<AssetId, JoinHandle<()>>,
    base_path: PathBuf,
}

impl AssetServer {
    /// 同步加载（热路径）
    pub fn load<T: 'static + Send>(&mut self, path: &str) -> Handle<T> {
        // 优先从缓存取
        // 否则从磁盘加载（RON/bincode/图片）
    }

    /// 异步加载（大资源）
    pub fn load_async<T: 'static + Send>(&mut self, path: &str) -> Handle<T> {
        // 后台线程加载
    }

    /// 热重载回调（吸收 Godot 热重载）
    pub fn watch(&mut self, path: &str) {
        // 用 notify crate 监听文件变化
    }
}
```

**资产类型注册表**:
```rust
pub enum AssetType {
    Texture,   // 图片
    TileSet,   // 瓦片集（RON）
    Prefab,    // 预制体（RON）
    Dialogue,  // 对话树（RON）
    Quest,     // 任务数据（RON）
    Audio,     // 音频
    Font,      // 字体
    Shader,    // 着色器
}
```

**验证**: 能从磁盘加载 RON 预制体并实例化

---

### 5. UI 系统 — 吸收 Godot Control + Unity UI Toolkit

**Godot 4 做法**: Control 节点（Margin/VBox/HBox/Grid），锚点布局，主题系统。

**Unity 做法**: UI Toolkit（XML + CSS），USS 样式表。

**NT-GAME 吸收方案**: ScreenStack + egui 集成 + RON UI 描述。

```rust
// ui/mod.rs — 吸收 Godot 的 ScreenStack 概念
pub enum Screen {
    MainMenu,
    GameWorld,
    Inventory,
    Dialogue { npc_entity: Entity },
    QuestLog,
    PauseMenu,
    Settings,
    DevConsole,
}

pub struct ScreenStack {
    screens: Vec<Screen>,
    transitions: HashMap<(Screen, Screen), Transition>,
}

impl ScreenStack {
    pub fn push(&mut self, screen: Screen) { ... }
    pub fn pop(&mut self) -> Option<Screen> { ... }
    pub fn replace(&mut self, screen: Screen) { ... }
    pub fn current(&self) -> Option<&Screen> { ... }
}

// ui/widgets.rs — 吸收 Godot Control 的组合式 UI
pub struct Widget {
    pub layout: Layout,
    pub style: Style,
    pub children: Vec<Widget>,
    pub on_click: Option<Box<dyn Fn()>>,
}

pub enum Layout {
    VBox { spacing: f32 },
    HBox { spacing: f32 },
    Grid { cols: usize, spacing: f32 },
    Anchor { top: f32, bottom: f32, left: f32, right: f32 },
    Margin { all: f32 },
}
```

**egui 集成（调试 UI）**:
```rust
// ui/debug.rs — 类似 Unity 的 Inspector
pub fn draw_debug_ui(ctx: &egui::Context, world: &mut GameWorld) {
    egui::SidePanel::left("inspector").show(ctx, |ui| {
        ui.heading("Entity Inspector");
        // 显示选中实体的所有组件
        // 实时编辑组件值
    });

    egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
        // 撤销/重做、播放/暂停、时间缩放
    });
}
```

**验证**: 能渲染主菜单 + 背包界面 + 调试面板

---

### 6. 状态机 — 吸收 Unity StatePattern + Godot StateMachine

**Unity 做法**: 状态模式（IState + StateMachine），Inspector 中可视化。

**Godot 4 做法**: StateMachine 节点 + Transition 系统。

**NT-GAME 吸收方案**: 泛型状态机 + RON 配置。

```rust
// state/mod.rs — 吸收 Unity StatePattern
pub trait GameState: Send + Sync {
    fn name(&self) -> &str;
    fn enter(&mut self, world: &mut GameWorld);
    fn update(&mut self, world: &mut GameWorld, dt: f64);
    fn exit(&mut self, world: &mut GameWorld);
}

pub struct StateMachine<S: GameState> {
    states: HashMap<String, S>,
    current: Option<String>,
    previous: Option<String>,
}

impl<S: GameState> StateMachine<S> {
    pub fn transition_to(&mut self, state: &str, world: &mut GameWorld) {
        if let Some(ref current) = self.current {
            self.states.get_mut(current).unwrap().exit(world);
        }
        self.previous = self.current.clone();
        self.current = Some(state.to_string());
        self.states.get_mut(state).unwrap().enter(world);
    }
}

// 预定义游戏状态
pub struct PlayingState;
pub struct PausedState;
pub struct DialogueState;
pub struct CombatState;
pub struct MenuState;
```

**与现有战斗状态机的集成**:
- 现有 `CombatState`（回合制战斗）→ 作为 `CombatState` 的内部实现
- 新增游戏级状态机管理 主菜单/游戏/对话/暂停 的切换

**验证**: 状态切换正确触发 enter/update/exit 生命周期

---

### 7. 动画系统 — 吸收 Godot AnimationPlayer + Unity Animator

**Godot 4 做法**: AnimationPlayer（时间轴关键帧）+ AnimationTree（混合树）。

**Unity 做法**: Animator（状态机驱动）+ Blend Tree（混合）。

**NT-GAME 吸收方案**: 关键帧动画 + 状态机驱动 + RON 数据。

```rust
// animation/mod.rs
pub struct AnimationPlayer {
    animations: HashMap<String, Animation>,
    current: Option<String>,
    time: f64,
    speed: f64,
    playing: bool,
}

pub struct Animation {
    pub keyframes: Vec<Keyframe>,
    pub looping: bool,
    pub duration: f64,
}

pub struct Keyframe {
    pub time: f64,
    pub sprite_region: Rect,    // 精灵表区域
    pub offset: Vec2,           // 位置偏移
    pub rotation: f32,          // 旋转
    pub scale: Vec2,            // 缩放
}

// animation/blend.rs — 吸收 Unity BlendTree
pub struct BlendTree {
    pub blends: Vec<BlendNode>,
    pub parameter: String,  // 混合参数（如速度）
}

pub struct BlendNode {
    pub animation: String,
    pub threshold: f32,     // 阈值
    pub weight: f32,        // 权重（运行时计算）
}
```

**RON 动画数据**:
```ron
// assets/animations/player_run.ron
Animation(
    keyframes: [
        Keyframe(time: 0.0, sprite_region: (0, 0, 32, 32)),
        Keyframe(time: 0.1, sprite_region: (32, 0, 32, 32)),
        Keyframe(time: 0.2, sprite_region: (64, 0, 32, 32)),
        Keyframe(time: 0.3, sprite_region: (96, 0, 32, 32)),
    ],
    looping: true,
    duration: 0.4,
)
```

**验证**: 角色能播放行走/攻击/闲置动画，动画状态机正确切换

---

### 8. 物理/碰撞 — 吸收 Godot Area2D + Unity 2D Physics

**Godot 4 做法**: Area2D（触发器）+ CharacterBody2D（物理移动）+ RigidBody2D（刚体）。

**NT-GAME 吸收方案**: 轻量级 2D 碰撞检测（不需要完整物理引擎）。

```rust
// physics/mod.rs — 吸收 Godot CollisionShape2D
pub enum ColliderShape {
    Rectangle { size: Vec2 },
    Circle { radius: f32 },
    Polygon { points: Vec<Vec2> },
}

pub struct Collider {
    pub shape: ColliderShape,
    pub offset: Vec2,
    pub is_sensor: bool,  // 触发器（不阻挡移动）
    pub layer: u32,       // 碰撞层
    pub mask: u32,        // 碰撞掩码
}

pub struct CollisionEvent {
    pub entity_a: Entity,
    pub entity_b: Entity,
    pub contact_point: Vec2,
    pub normal: Vec2,
}

// physics/spatial.rs — 吸收 Godot 的空间查询
pub struct SpatialHash {
    cell_size: f32,
    cells: HashMap<(i32, i32), Vec<Entity>>,
}

impl SpatialHash {
    pub fn query_rect(&self, rect: Rect) -> Vec<Entity> { ... }
    pub fn query_circle(&self, center: Vec2, radius: f32) -> Vec<Entity> { ... }
}
```

**与现有代码的集成**:
- `combat.rs` 的 `range` 字段 → 用碰撞检测替代距离计算
- `npc.rs` 的 `position` → 加入物理组件

**验证**: 能检测角色与 NPC/物品的碰撞，触发器正常工作

---

### 9. 游戏能力系统（GAS）— 吸收 UE5 GameplayAbilitySystem

**UE5 做法**: GameplayAbility（能力）+ GameplayEffect（效果）+ AttributeSet（属性集）+ GameplayTag（标签）。

**NT-GAME 吸收方案**: 标签驱动的能力系统 + 与现有 RPG 系统集成。

```rust
// ability/mod.rs — 吸收 UE5 GAS
pub struct GameplayTag {
    pub namespace: String,  // "ability.attack.fireball"
    pub tags: Vec<String>,
}

pub struct Ability {
    pub name: String,
    pub tags: Vec<GameplayTag>,
    pub cooldown: f64,
    pub cost: f64,
    pub effects: Vec<Effect>,
    pub conditions: Vec<Condition>,
}

pub struct Effect {
    pub attribute: String,   // "health", "mana", "speed"
    pub operation: Operation,
    pub value: f64,
    pub duration: Option<f64>,
    pub modifiers: Vec<Modifier>,
}

pub enum Operation {
    Add,
    Multiply,
    Set,
    Damage,
    Heal,
}

pub struct AbilitySystem {
    abilities: HashMap<String, Ability>,
    active_effects: Vec<ActiveEffect>,
    tags: Vec<GameplayTag>,
}
```

**与现有 RPG 的集成**:
- 现有 `CharacterStats` → 作为 `AttributeSet`
- 现有 `CombatAction` → 作为 `Ability` 的简化版
- 现有装备系统 → 装备附加 `Effect`

**验证**: 能用标签驱动的能力系统替代现有的硬编码战斗逻辑

---

### 10. 增强输入 — 吸收 UE5 Enhanced Input + Godot InputMap

**UE5 做法**: InputAction + InputMappingContext + InputModifier + InputTrigger。

**Godot 4 做法**: InputMap（预定义动作）+ Input.is_action_pressed()。

**NT-GAME 吸收方案**: 动作映射 + 组合键 + RON 配置。

```rust
// input/mod.rs — 吸收 UE5 EnhancedInput
pub struct InputAction {
    pub name: String,
    pub bindings: Vec<InputBinding>,
    pub modifiers: Vec<InputModifier>,
}

pub enum InputBinding {
    Key(KeyCode),
    MouseButton(MouseButton),
    GamepadButton(GamepadButton),
    GamepadAxis(GamepadAxis, f32),  // 轴 + 阈值
}

pub struct InputMappingContext {
    pub actions: HashMap<String, InputAction>,
    pub priority: i32,
}

pub struct InputState {
    pub pressed: HashSet<String>,
    pub just_pressed: HashSet<String>,
    pub just_released: HashSet<String>,
    pub axes: HashMap<String, f32>,  // 模拟轴值
}

// RON 配置
// assets/input.ron
InputMappingContext(
    actions: {
        "move_up": InputAction(
            name: "move_up",
            bindings: [Key(W), Key(Up), GamepadAxis(LeftY, -0.5)],
        ),
        "attack": InputAction(
            name: "attack",
            bindings: [Key(Space), MouseButton(Left), GamepadButton(A)],
        ),
    },
)
```

**验证**: 能通过 RON 配置输入映射，支持键盘/手柄

---

### 11. 存档系统 — 吸收 Godot ResourceSaver + Unity PlayerPrefs

**Godot 4 做法**: `ResourceSaver.save()` 序列化资源，`ResourceLoader.load()` 反序列化。

**Unity 做法**: `JsonUtility.ToJson()` + `PlayerPrefs`。

**NT-GAME 吸收方案**: bincode 二进制存档 + RON 可读存档 + 版本迁移。

```rust
// save/mod.rs
pub struct SaveGame {
    pub version: u32,
    pub timestamp: u64,
    pub player: PlayerData,
    pub world: WorldData,
    pub quests: Vec<QuestData>,
    pub inventory: Vec<ItemStack>,
    pub flags: HashMap<String, String>,
    pub consciousness: ConsciousnessData,  // 意识状态也保存
}

impl SaveGame {
    pub fn save(&self, path: &Path) -> Result<(), SaveError> {
        let bytes = bincode::serialize(self)?;
        std::fs::write(path, bytes)?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self, SaveError> {
        let bytes = std::fs::read(path)?;
        let save: SaveGame = bincode::deserialize(&bytes)?;
        save.migrate()  // 版本迁移
    }

    fn migrate(&mut self) -> Result<(), SaveError> {
        // 吸收 Godot 的版本迁移模式
        while self.version < CURRENT_SAVE_VERSION {
            self.version = match self.version {
                1 => self.migrate_v1_to_v2(),
                2 => self.migrate_v2_to_v3(),
                _ => break,
            };
        }
        Ok(())
    }
}
```

**自动存档（吸收 Godot Autosave）**:
```rust
pub struct AutosaveManager {
    interval: Duration,
    last_save: Instant,
    max_slots: usize,
}

impl AutosaveManager {
    pub fn tick(&mut self, world: &GameWorld) {
        if self.last_save.elapsed() >= self.interval {
            let slot = self.next_slot();
            SaveGame::from_world(world).save(&slot_path(slot))?;
            self.last_save = Instant::now();
        }
    }
}
```

**验证**: 能保存/加载游戏进度，版本迁移正确工作

---

### 12. 热重载 — 吸收 Godot GDScript 热重载

**Godot 4 做法**: 修改 GDScript → 重新编译 → 运行时替换，不丢失游戏状态。

**NT-GAME 吸收方案**: 数据文件热重载（RON/图片），无需重新编译。

```rust
// hot_reload/mod.rs
pub struct HotReloader {
    watcher: RecommendedWatcher,
    modified: HashSet<PathBuf>,
}

impl HotReloader {
    pub fn watch_dir(&mut self, path: &Path) {
        // 用 notify crate 监听目录
    }

    pub fn poll(&mut self) -> Vec<PathBuf> {
        // 返回修改过的文件列表
    }

    pub fn reload_asset(&mut self, path: &Path, assets: &mut AssetServer) {
        match path.extension().and_then(|e| e.to_str()) {
            Some("ron") => assets.reload_ron(path),
            Some("png" | "jpg") => assets.reload_texture(path),
            _ => {}
        }
    }
}
```

**验证**: 修改 RON 文件后，游戏中立即生效

---

### 13. 定时器系统 — 吸收 Unity Coroutine + Godot Timer

```rust
// timer/mod.rs
pub struct Timer {
    pub duration: f64,
    pub elapsed: f64,
    pub repeating: bool,
    pub callback: Option<Box<dyn FnOnce()>>,
    pub active: bool,
}

pub struct TimerManager {
    timers: Vec<Timer>,
}

impl TimerManager {
    pub fn add_timer(&mut self, duration: f64, repeating: bool, cb: impl FnOnce() + 'static) {
        self.timers.push(Timer {
            duration,
            elapsed: 0.0,
            repeating,
            callback: Some(Box::new(cb)),
            active: true,
        });
    }

    pub fn tick(&mut self, dt: f64) {
        for timer in &mut self.timers {
            if !timer.active { continue; }
            timer.elapsed += dt;
            if timer.elapsed >= timer.duration {
                if let Some(cb) = timer.callback.take() {
                    cb();
                }
                if timer.repeating {
                    timer.elapsed = 0.0;
                } else {
                    timer.active = false;
                }
            }
        }
    }
}
```

**验证**: 能创建重复/单次定时器，正确触发回调

---

### 14. 相机系统 — 吸收 Godot Camera2D + UE5 Camera

```rust
// camera/mod.rs — 吸收 Godot Camera2D
pub struct Camera2D {
    pub position: Vec2,
    pub zoom: f32,
    pub rotation: f32,
    pub target: Option<Entity>,
    pub smoothing: f32,
    pub limits: CameraLimits,
    pub shake: ScreenShake,
}

pub struct CameraLimits {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
}

pub struct ScreenShake {
    pub intensity: f32,
    pub duration: f32,
    pub elapsed: f32,
}
```

**验证**: 相机能跟随玩家、限制在地图范围内、支持屏幕震动

---

### 15. 本地化 — 吸收 Godot + Unity i18n

```rust
// localization/mod.rs
pub struct Localization {
    current_language: String,
    translations: HashMap<String, HashMap<String, String>>,
}

impl Localization {
    pub fn t(&self, key: &str) -> &str {
        self.translations
            .get(&self.current_language)
            .and_then(|m| m.get(key))
            .unwrap_or(key)
    }

    pub fn load(&mut self, path: &Path) {
        // 从 RON 加载翻译表
    }
}

// assets/i18n/zh_cn.ron
{
    "npc.merchant.greeting": "欢迎光临！",
    "quest.rat_kills.title": "消灭老鼠",
    "ui.inventory.title": "背包",
}
```

**验证**: 能切换语言，UI 文本正确显示

---

### 16. 光照系统 — 吸收 Godot 2D Lighting

```rust
// lighting/mod.rs — 吸收 Godot PointLight2D
pub struct Light2D {
    pub position: Vec2,
    pub color: Color,
    pub intensity: f32,
    pub radius: f32,
    pub shadow: bool,
}

pub struct LightOccluder2D {
    pub shape: ColliderShape,
    pub opacity: f32,
}

// 时间驱动的环境光（吸收 Godot 的 WorldEnvironment）
pub struct Environment {
    pub ambient_color: Color,
    pub ambient_intensity: f32,
    pub time_of_day: f32,  // 0.0-24.0
    pub sunrise_hour: f32,
    pub sunset_hour: f32,
}
```

**验证**: 能渲染动态光照，时间影响环境光

---

## 三、吸收优先级与实施顺序

### P0 — 最小可玩（Week 1-2）
| 特性 | 来源 | 改动量 |
|------|------|--------|
| 渲染系统 | macroquad + Godot Camera2D | 新增 |
| 场景树/预制体 | Godot SceneTree | 新增 |
| 碰撞检测 | Godot Area2D | 新增 |
| 基础 UI | egui + Godot ScreenStack | 新增 |

### P1 — 核心体验（Week 3-4）
| 特性 | 来源 | 改动量 |
|------|------|--------|
| 事件系统 | Godot Signals | 新增 |
| 状态机 | Unity StatePattern | 新增 |
| 资产管线 | Godot Resource | 新增 |
| 存档系统 | Godot ResourceSaver | 新增 |
| 动画系统 | Godot AnimationPlayer | 新增 |

### P2 — 完整体验（Week 5-6）
| 特性 | 来源 | 改动量 |
|------|------|--------|
| 增强输入 | UE5 EnhancedInput | 新增 |
| 定时器 | Unity Coroutine | 新增 |
| 热重载 | Godot 热重载 | 新增 |
| 本地化 | Godot i18n | 新增 |
| GAS 能力系统 | UE5 GAS | 新增 |

### P3 — 打磨（Week 7-8）
| 特性 | 来源 | 改动量 |
|------|------|--------|
| 光照系统 | Godot 2D Lighting | 新增 |
| 关卡流 | UE5 LevelStreaming | 新增 |
| 屏幕震动 | Godot ScreenShake | 新增 |

---

## 四、代码结构（吸收后）

```
neotrix-game/
├── src/
│   ├── core/
│   │   ├── ecs/          # 增强版 ECS（吸收 Godot Node）
│   │   ├── event.rs      # 事件总线（吸收 Godot Signals）
│   │   ├── state.rs      # 状态机（吸收 Unity StatePattern）
│   │   ├── timer.rs      # 定时器（吸收 Unity Coroutine）
│   │   └── scene.rs      # 场景管理器（吸收 Godot SceneTree）
│   ├── render/
│   │   ├── camera.rs     # 相机（吸收 Godot Camera2D）
│   │   ├── sprite.rs     # 精灵渲染
│   │   ├── tilemap.rs    # 瓦片渲染
│   │   ├── particle.rs   # 粒子渲染
│   │   ├── light.rs      # 光照（吸收 Godot 2D Lighting）
│   │   └── ui.rs         # UI 渲染（egui）
│   ├── asset/
│   │   ├── server.rs     # 资产服务器（吸收 Godot Resource）
│   │   ├── handle.rs     # 类型安全句柄
│   │   └── hot_reload.rs # 热重载（吸收 Godot 热重载）
│   ├── input/
│   │   ├── action.rs     # 输入动作（吸收 UE5 EnhancedInput）
│   │   ├── mapping.rs    # 输入映射
│   │   └── state.rs      # 输入状态
│   ├── save/
│   │   ├── mod.rs        # 存档系统（吸收 Godot ResourceSaver）
│   │   ├── autosave.rs   # 自动存档
│   │   └── migration.rs  # 版本迁移
│   ├── ui/
│   │   ├── screen.rs     # 屏幕栈（吸收 Godot ScreenStack）
│   │   ├── widgets.rs    # 组件（吸收 Godot Control）
│   │   └── debug.rs      # 调试 UI（吸收 Unity Inspector）
│   ├── ability/
│   │   ├── mod.rs        # 能力系统（吸收 UE5 GAS）
│   │   ├── effect.rs     # 效果
│   │   └── tag.rs        # 标签
│   ├── i18n/
│   │   └── mod.rs        # 本地化（吸收 Godot i18n）
│   ├── game/             # NT-GAME 现有模块（保留）
│   │   ├── ecs/
│   │   ├── rpg/
│   │   ├── world/
│   │   ├── ai/
│   │   ├── consciousness/
│   │   ├── evolution.rs
│   │   ├── framework.rs
│   │   └── mcp.rs
│   └── main.rs
├── assets/
│   ├── prefabs/          # 预制体（RON）
│   ├── animations/       # 动画数据（RON）
│   ├── input.ron         # 输入映射
│   ├── i18n/             # 翻译文件
│   └── textures/         # 图片资源
└── Cargo.toml
```

---

## 五、一句话总结

> **逐条吸收 Godot 的场景树/信号/资源/热重载、Unity 的状态机/动画/COROUTINE、UE5 的 GAS/增强输入，用 Rust 重新实现为轻量级模块，与 NT-GAME 现有的 200+ 模块无缝集成。**
