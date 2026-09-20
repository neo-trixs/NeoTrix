# NeoTrix 游戏构建 — 技术全栈完成清单

**日期**: 2026-09-20
**目标**: 从 `cargo new` 到可发布游戏的完整技术路径
**原则**: G1 数据即游戏 / G2 安全是默认态 / G3 模块即能力

---

## 总览：16 层技术栈

```
Layer 16  发布分发    ──── 平台打包、商店上架、自动更新
Layer 15  本地化      ──── 多语言、RTL、文化适配
Layer 14  无障碍      ──── 色盲模式、键位重映射、屏幕阅读器
Layer 13  手感打磨    ──── 粒子、屏幕震动、慢动作、音效反馈
Layer 12  AI 系统     ──── NPC 行为树、自适应难度、程序化叙事
Layer 11  模组系统    ──── 模组加载、沙箱隔离、版本兼容
Layer 10  网络层      ──── 状态同步、延迟补偿、断线重连
Layer 9   存档系统    ──── 自动存档、多槽位、回滚、云同步
Layer 8   UI 系统     ──── 菜单、HUD、对话框、背包界面、tooltip
Layer 7   音频系统    ──── BGM、SFX、环境音、空间音频、动态混音
Layer 6   物理碰撞    ──── AABB、圆形碰撞、射线检测、触发器
Layer 5   游戏能力    ──── 战斗/种植/对话/制作/任务/叙事（独立 crate）
Layer 4   输入系统    ──── 键盘、鼠标、手柄、触屏、输入映射
Layer 3   渲染系统    ──── 精灵、动画、图层、相机、粒子、UI 绘制
Layer 2   数据管线    ──── TOML/RON 加载、类型安全、热重载、缓存
Layer 1   核心引擎    ──── 游戏循环、状态管理、事件总线、时间系统
Layer 0   项目骨架    ──── workspace、crate 结构、依赖、CI
```

---

## Layer 0: 项目骨架

### 0.1 Workspace 初始化

- [ ] `cargo new --name neotrix-game` 创建顶层项目
- [ ] 配置 `Cargo.toml` workspace members
- [ ] 设置 `#![forbid(unsafe_code)]` 在每个 crate 根
- [ ] 配置 `rustfmt.toml`（tab 实际宽度 4、max_width 100）
- [ ] 配置 `clippy.toml`（cognitive_complexity_threshold = 30）
- [ ] 设置 `.cargo/config.toml`（target-specific runner）

### 0.2 Crate 结构

```
crates/
├── nt_game/                    # 顶层 re-export crate
├── nt_game_core/               # L1 核心引擎
├── nt_game_data/               # L2 数据管线
├── nt_game_render/             # L3 渲染系统
├── nt_game_input/              # L4 输入系统
├── nt_game_ability_combat/     # L5 战斗能力
├── nt_game_ability_dialogue/   # L5 对话能力
├── nt_game_ability_farming/    # L5 种植能力
├── nt_game_ability_crafting/   # L5 制作能力
├── nt_game_ability_inventory/  # L5 背包能力
├── nt_game_ability_quest/      # L5 任务能力
├── nt_game_ability_narrative/  # L5 叙事能力
├── nt_game_physics/            # L6 物理碰撞
├── nt_game_audio/              # L7 音频系统
├── nt_game_ui/                 # L8 UI 系统
├── nt_game_save/               # L9 存档系统
├── nt_game_network/            # L10 网络层（可选）
├── nt_game_mod/                # L11 模组系统
└── nt_game_ai/                 # L12 AI 系统
```

### 0.3 核心依赖

```toml
[workspace.dependencies]
# 序列化
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
ron = "0.8"

# 异步
tokio = { version = "1", features = ["full"] }

# 错误处理
thiserror = "2"
anyhow = "1"

# 日志
tracing = "0.1"
tracing-subscriber = "0.3"

# 时间
instant = "0.1"  # 跨平台时间（含 Wasm）

# 哈希
xxhash-rust = { version = "0.8", features = ["xxh3"] }

# ID 系统
ulid = "1"

# 资源管理
assets-manager = { version = "0.7", features = ["toml", "json", "ron"] }

# 渲染（按需选择）
macroquad = "0.4"        # 轻量 2D
# wgpu = "24"            # 现代 GPU（需要时启用）
# bevy_render = "0.15"   # Bevy 生态（可选）

# 音频
quad-snd = "0.1"         # macroquad 音频
# rodio = "0.19"         # 通用音频（需要时启用）

# 物理
parry2d = "0.17"         # 2D 碰撞检测

# ECS（可选，小游戏不需要）
# bevy_ecs = "0.15"
# hecs = "0.10"

# UI
egui = "0.30"            # 即时模式 UI
# egui-macroquad = "0.18"

# 网络（可选）
# laminar = "0.5"        # UDP
# reqwest = "0.12"       # HTTP

# Wasm
wasm-bindgen = "0.2"
web-sys = "0.3"
```

### 0.4 CI 配置

- [ ] GitHub Actions: `cargo check --all-targets`
- [ ] GitHub Actions: `cargo test --lib`
- [ ] GitHub Actions: `cargo clippy -- -D warnings`
- [ ] GitHub Actions: `cargo fmt --check`
- [ ] GitHub Actions: Wasm build 验证（`wasm-pack build`）
- [ ] GitHub Actions: 发布构建（release profile）

---

## Layer 1: 核心引擎

### 1.1 游戏循环

```rust
// nt_game_core/src/loop.rs
pub struct GameLoop {
    target_fps: u32,
    accumulator: f64,      // 固定时间步累加器
    fixed_dt: f64,         // 固定时间步（1/60）
    running: bool,
}
```

- [ ] 实现固定时间步游戏循环（accumulator 模式）
- [ ] 支持可配置目标 FPS（默认 60）
- [ ] 实现 `update(fixed_dt)` + `render(alpha)` 分离
- [ ] 处理窗口焦点丢失（暂停或降帧）
- [ ] 实现 `RequestExit` 事件优雅退出

### 1.2 状态管理

```rust
// nt_game_core/src/state.rs
pub trait GameState: Serialize + DeserializeOwned + 'static {
    fn init(&mut self, ctx: &mut GameContext) -> Result<()>;
    fn update(&mut self, ctx: &mut GameContext) -> Vec<GameEvent>;
    fn render(&self, ctx: &RenderContext, alpha: f64);
}
```

- [ ] 定义 `GameContext`（时间、输入、资源、事件队列）
- [ ] 定义 `RenderContext`（相机、渲染器引用）
- [ ] 实现状态栈（push/pop/replace）
- [ ] 实现 `StateTransition` 枚举（Push/Pop/Replace/None）
- [ ] 支持状态间数据传递（via `on_enter`/`on_exit` payload）

### 1.3 事件系统

```rust
// nt_game_core/src/events.rs
pub struct EventBus {
    listeners: HashMap<TypeId, Vec<Box<dyn Fn(&dyn Any)>>>,
    queue: VecDeque<Box<dyn Any>>,
    deferred: Vec<Box<dyn Any>>,
}
```

- [ ] 实现类型安全的事件发布/订阅
- [ ] 支持立即事件和延迟事件（deferred queue）
- [ ] 实现事件优先级（Critical > Normal > Low）
- [ ] 支持一次性订阅（`once`）和持续订阅（`on`）
- [ ] 实现事件过滤（只接收特定来源的事件）

### 1.4 时间系统

```rust
// nt_game_core/src/time.rs
pub struct TimeSystem {
    elapsed: Duration,       // 游戏内经过时间
    real_elapsed: Duration,  // 真实经过时间
    time_scale: f32,         // 时间缩放（慢动作/快进）
    paused: bool,
    day_night_cycle: Option<DayNightCycle>,
}
```

- [ ] 实现游戏时间 vs 真实时间分离
- [ ] 支持时间缩放（slow-motion, fast-forward）
- [ ] 实现暂停/恢复
- [ ] 可选：昼夜循环（基于真实时间或游戏时间）
- [ ] 可选：季节系统（春/夏/秋/冬）

### 1.5 实体标识

```rust
// nt_game_core/src/entity.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(Ulid);

pub struct Entity {
    pub id: EntityId,
    pub name: String,
    pub tags: HashSet<String>,
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
    pub visible: bool,
    pub components: HashMap<TypeId, Box<dyn erased_serde::Serialize>>,
}
```

- [ ] 实现 ULID 基础的实体 ID（全局唯一、可排序）
- [ ] 实现组件系统（HashMap<TypeId, Box<dyn Any>>）
- [ ] 支持标签系统（tag-based 查询）
- [ ] 实现实体生命周期（create/destroy/activate/deactivate）

---

## Layer 2: 数据管线

### 2.1 数据加载器

```rust
// nt_game_data/src/loader.rs
pub struct DataPipeline {
    base_path: PathBuf,
    cache: LruCache<u64, Box<dyn Any>>,  // xxh3 hash → data
    hot_reload: Option<HotReloadWatcher>,
}
```

- [ ] 实现 TOML/JSON/RON 三种格式加载
- [ ] 实现类型安全反序列化（`serde` derive）
- [ ] 实现 LRU 缓存（默认 256 条目）
- [ ] 实现数据验证（`validate` 方法，加载时检查）
- [ ] 实现默认值填充（missing field → schema default）

### 2.2 热重载

```rust
// nt_game_data/src/hot_reload.rs
pub struct HotReloadWatcher {
    watcher: RecommendedWatcher,
    changed_files: Arc<Mutex<Vec<PathBuf>>>,
}
```

- [ ] 使用 `notify` crate 监听文件变化
- [ ] 实现 debounce（200ms 内多次变化只触发一次）
- [ ] 实现增量重加载（只重载变化的文件）
- [ ] 实现重加载事件通知（通过 EventBus）
- [ ] 支持启用/禁用热重载（debug only）

### 2.3 数据 Schema

```rust
// nt_game_data/src/schema.rs
pub struct SchemaRegistry {
    schemas: HashMap<TypeId, Vec<SchemaEntry>>,
}

pub struct SchemaEntry {
    pub field: &'static str,
    pub r#type: FieldType,
    pub required: bool,
    pub default: Option<Value>,
    pub min: Option<f64>,
    pub max: Option<f64>,
}
```

- [ ] 实现声明式 Schema 注册
- [ ] 支持字段类型检查（Int/Float/String/Bool/Array/Table/Ref）
- [ ] 支持数值范围验证（min/max）
- [ ] 支持引用验证（Referenced entity exists?）
- [ ] 实现 Schema → 文档生成（自动生成数据格式文档）

### 2.4 数据格式约定

```toml
# assets/schemas/crop.toml — Schema 定义
[[fields]]
name = "name"
type = "String"
required = true

[[fields]]
name = "grow_time_days"
type = "Int"
required = true
min = 1
max = 30

[[fields]]
name = "sell_price"
type = "Int"
required = true
min = 0

[[fields]]
name = "seasons"
type = "Array<String>"
required = true
allowed_values = ["spring", "summer", "fall", "winter"]
```

- [ ] 定义所有游戏实体的 Schema（crops, items, npcs, quests, etc.）
- [ ] 实现 Schema → Rust struct 自动生成（build.rs）
- [ ] 实现 Schema → 数据文件模板生成
- [ ] 实现 Schema → 前端/API 文档生成

---

## Layer 3: 渲染系统

### 3.1 渲染器抽象

```rust
// nt_game_render/src/renderer.rs
pub trait Renderer: Send + Sync {
    fn init(&mut self, config: &RenderConfig) -> Result<()>;
    fn begin_frame(&mut self, clear_color: Color);
    fn end_frame(&mut self);

    // 精灵渲染
    fn draw_sprite(&mut self, sprite: &Sprite, transform: &Transform, tint: Color);
    fn draw_sprite_region(&mut self, sprite: &Sprite, region: &Rect, transform: &Transform, tint: Color);

    // 几何渲染
    fn draw_rect(&mut self, rect: &Rect, color: Color);
    fn draw_circle(&mut self, center: Vec2, radius: f32, color: Color);
    fn draw_line(&mut self, start: Vec2, end: Vec2, thickness: f32, color: Color);

    // 文字渲染
    fn draw_text(&mut self, text: &str, pos: Vec2, font: &Font, size: f32, color: Color);
    fn measure_text(&self, text: &str, font: &Font, size: f32) -> Vec2;

    // UI 渲染
    fn draw_ui(&mut self, layout: &UILayout);
}
```

- [ ] 实现 `macroquad` 后端（默认）
- [ ] 实现 `headless` 后端（测试用）
- [ ] 实现 `wgpu` 后端（高级 2D/3D）
- [ ] 实现 `WebCanvas` 后端（Wasm）

### 3.2 精灵系统

```rust
// nt_game_render/src/sprite.rs
pub struct Sprite {
    pub texture: TextureId,
    pub source_rect: Option<Rect>,  // sprite sheet 子区域
    pub pivot: Vec2,                // 旋转中心
    pub flip_x: bool,
    pub flip_y: bool,
}

pub struct SpriteSheet {
    pub texture: TextureId,
    pub tile_size: Vec2,
    pub columns: u32,
    pub rows: u32,
}
```

- [ ] 实现纹理加载（PNG/WEBP/BMP）
- [ ] 实现 Sprite Sheet 切片
- [ ] 实现 Sprite Animation（帧序列 + 循环/一次性）
- [ ] 实现 AnimationPlayer（支持混合、事件回调）
- [ ] 实现图层排序（z-index / y-sort）

### 3.3 相机系统

```rust
// nt_game_render/src/camera.rs
pub struct Camera {
    pub position: Vec2,
    pub zoom: f32,
    pub rotation: f32,
    pub shake: ScreenShake,
    pub bounds: Option<Rect>,  // 世界边界限制
}
```

- [ ] 实现 2D 相机（平移、缩放、旋转）
- [ ] 实现屏幕震动（decay + frequency）
- [ ] 实现相机跟随（平滑插值 lerp）
- [ ] 实现相机边界限制（不超出地图）
- [ ] 实现相机 → 屏幕坐标转换（`world_to_screen` / `screen_to_world`）

### 3.4 粒子系统

```rust
// nt_game_render/src/particle.rs
pub struct ParticleEmitter {
    pub position: Vec2,
    pub rate: f32,             // 每秒发射数
    pub lifetime: Range<f32>,  // 粒子生命周期
    pub velocity: Range<Vec2>,
    pub color: Gradient<Color>,
    pub size: Range<f32>,
    pub gravity: Vec2,
    pub shape: EmitterShape,   // Point/Circle/Cone
}
```

- [ ] 实现基础粒子发射器
- [ ] 实现粒子生命周期管理（spawn/update/kill）
- [ ] 实现颜色/大小渐变
- [ ] 实现预设粒子效果（sparkle, dust, leaves, rain）
- [ ] 实现粒子池（对象复用，避免分配）

---

## Layer 4: 输入系统

### 4.1 输入管理器

```rust
// nt_game_input/src/manager.rs
pub struct InputManager {
    keyboard: KeyboardState,
    mouse: MouseState,
    gamepad: GamepadState,
    touch: TouchState,
    action_map: ActionMap,
}
```

- [ ] 实现键盘状态追踪（pressed/held/released）
- [ ] 实现鼠标状态追踪（position/buttons/wheel）
- [ ] 实现手柄状态追踪（axes/buttons/rumble）
- [ ] 实现触屏支持（tap/swipe/pinch）
- [ ] 实现输入事件缓冲（帧内一致）

### 4.2 动作映射

```rust
// nt_game_input/src/action.rs
pub struct ActionMap {
    bindings: HashMap<ActionId, Vec<InputBinding>>,
}

pub enum InputBinding {
    Key(KeyCode),
    MouseAxis(MouseAxis),
    GamepadButton(GamepadButton),
    GamepadAxis(GamepadAxis, f32),  // axis + threshold
    TouchGesture(TouchGesture),
}

pub enum ActionId {
    MoveUp, MoveDown, MoveLeft, MoveRight,
    Confirm, Cancel, Menu, Inventory,
    Attack, Ability1, Ability2, Ability3,
    // ... 游戏自定义
}
```

- [ ] 实现声明式动作绑定（TOML 配置）
- [ ] 实现运行时重绑定（玩家自定义键位）
- [ ] 实现输入缓冲（200ms 内的输入合并）
- [ ] 实现连击检测（double-tap, long-press）
- [ ] 实现手势识别（swipe direction, pinch zoom）

### 4.3 平台适配

- [ ] 桌面：键盘 + 鼠标 + 手柄
- [ ] 移动：触屏 + 虚拟摇杆
- [ ] 浏览器：键盘 + 鼠标 + 手柄（Gamepad API）
- [ ] 实现输入设备自动检测
- [ ] 实现输入设备热切换

---

## Layer 5: 游戏能力模块

### 5.1 能力接口

```rust
// nt_game_core/src/ability.rs
pub trait GameAbility: 'static + Send + Sync {
    /// 唯一标识
    fn id(&self) -> AbilityId;

    /// 声明依赖（其他能力）
    fn dependencies(&self) -> Vec<AbilityId> { vec![] }

    /// 初始化（加载数据、注册事件）
    fn init(&mut self, ctx: &mut GameContext) -> Result<()>;

    /// 每帧更新
    fn update(&mut self, ctx: &mut GameContext) -> Vec<GameEvent>;

    /// 处理事件
    fn on_event(&mut self, event: &GameEvent, ctx: &mut GameContext) -> Vec<GameEvent>;

    /// 序列化当前状态
    fn state(&self) -> Box<dyn erased_serde::Serialize>;

    /// 反序列化状态
    fn restore(&mut self, state: &dyn erased_serde::Deserializer) -> Result<()>;
}
```

### 5.2 背包能力 (`nt_game_ability_inventory`)

```toml
# assets/items.toml
[[items]]
id = "turnip_seed"
name = "Turnip Seeds"
description = "Plant these in spring for a quick harvest."
type = "seed"
stack_size = 99
buy_price = 20
sell_price = 5
sprite = "sprites/items.png"
sprite_region = [0, 0, 16, 16]

[[items]]
id = "turnip"
name = "Turnip"
description = "A humble root vegetable."
type = "crop"
stack_size = 99
buy_price = 60
sell_price = 30
sprite = "sprites/items.png"
sprite_region = [16, 0, 16, 16]
```

- [ ] 实现物品数据加载（TOML → ItemDef）
- [ ] 实现背包存储（slot-based, 可扩展）
- [ ] 实现物品堆叠（stack_size 限制）
- [ ] 实现物品使用（consume/use/drop）
- [ ] 实现物品交换（拖拽、快捷键）
- [ ] 实现装备槽位（weapon, armor, accessory）
- [ ] 实现背包 UI（网格布局、tooltip、排序）

### 5.3 战斗能力 (`nt_game_ability_combat`)

```toml
# assets/skills.toml
[[skills]]
id = "slash"
name = "Slash"
description = "A quick sword slash."
type = "attack"
damage = 15
cooldown = 0.5
range = 1.5
target = "single"
animation = "slash"
effects = [{ type = "damage", value = 15 }]

[[skills]]
id = "heal"
name = "Heal"
description = "Restore 20 HP."
type = "heal"
heal_amount = 20
cooldown = 3.0
target = "self"
animation = "heal"
effects = [{ type = "heal", value = 20 }]
```

- [ ] 实现技能数据加载（TOML → SkillDef）
- [ ] 实现技能冷却系统
- [ ] 实现伤害计算公式（攻击力 - 防御力 + 随机浮动）
- [ ] 实现目标选择（single/aoe/self/all_enemies/all_allies）
- [ ] 实现状态效果（buff/debuff/poison/bleed/stun）
- [ ] 实现战斗状态机（idle → casting → animating → cooldown）
- [ ] 实现回合制战斗（可选）vs 即时战斗
- [ ] 实现战斗 UI（HP 条、技能栏、伤害数字）

### 5.4 对话能力 (`nt_game_ability_dialogue`)

```toml
# assets/dialogues/maru.toml
[node.start]
speaker = "Maru"
portrait = "sprites/portraits/maru.png"
text = "Hey! Working on something cool?"
choices = [
    { text = "Tell me about your project", next = "project" },
    { text = "Just passing through", next = "bye" },
]

[node.project]
speaker = "Maru"
portrait = "sprites/portraits/maru.png"
text = "I'm building a telescope! Want to see the stars with me?"
choices = [
    { text = "That sounds amazing!", next = "telescope", conditions = [{ type = "friendship", npc = "maru", min = 3 }] },
    { text = "Maybe another time", next = "bye" },
]

[node.telescope]
speaker = "Maru"
portrait = "sprites/portraits/maru.png"
text = "Great! Meet me tonight at the hill."
effects = [
    { type = "add_quest", quest = "stargazing" },
    { type = "change_friendship", npc = "maru", amount = 1 },
]

[node.bye]
speaker = "Maru"
portrait = "sprites/portraits/maru.png"
text = "See you around!"
```

- [ ] 实现对话树数据加载（TOML → DialogueTree）
- [ ] 实现节点类型（text/choice/condition/effect/branch）
- [ ] 实现条件系统（检查物品/任务/好感度/时间）
- [ ] 实现效果系统（给予物品/改变好感度/触发任务）
- [ ] 实现打字机效果（逐字显示）
- [ ] 实现对话 UI（头像、名字、文本框、选项按钮）
- [ ] 实现对话历史记录

### 5.5 种植能力 (`nt_game_ability_farming`)

```toml
# assets/crops.toml
[[crops]]
id = "turnip"
name = "Turnip"
grow_time_days = 4
seasons = ["spring"]
buy_price = 20
sell_price = 60
harvest_items = [{ item = "turnip", count = 1 }]
growth_stages = [
    { day = 0, sprite = "seed" },
    { day = 1, sprite = "sprout" },
    { day = 2, sprite = "growing" },
    { day = 3, sprite = "mature" },
    { day = 4, sprite = "harvestable" },
]
```

- [ ] 实现农田格子系统（grid-based）
- [ ] 实现种植流程（till → plant → water → wait → harvest）
- [ ] 实现生长阶段（基于游戏天数）
- [ ] 实现季节影响（不同季节适合不同作物）
- [ ] 实现浇水系统（土壤湿度）
- [ ] 实现收获逻辑（掉落物品、经验值）
- [ ] 实现农田 UI（网格显示、生长进度条）

### 5.6 制作能力 (`nt_game_ability_crafting`)

```toml
# assets/recipes.toml
[[recipes]]
id = "wooden_sword"
name = "Wooden Sword"
category = "weapons"
ingredients = [
    { item = "wood", count = 5 },
    { item = "stone", count = 2 },
]
result = { item = "wooden_sword", count = 1 }
craft_time = 2.0
skill_required = 0

[[recipes]]
id = "healing_potion"
name = "Healing Potion"
category = "consumables"
ingredients = [
    { item = "herb", count = 3 },
    { item = "honey", count = 1 },
]
result = { item = "healing_potion", count = 1 }
craft_time = 1.0
skill_required = 0
```

- [ ] 实现配方数据加载
- [ ] 实现制作流程（选择配方 → 检查材料 → 等待 → 获得物品）
- [ ] 实现材料消耗
- [ ] 实现制作队列（同时制作多个）
- [ ] 实现制作经验/技能等级
- [ ] 实现制作 UI（配方列表、材料需求、制作按钮）

### 5.7 任务能力 (`nt_game_ability_quest`)

```toml
# assets/quests.toml
[[quests]]
id = "first_harvest"
name = "First Harvest"
description = "Grow and harvest your first crop."
type = "tutorial"
objectives = [
    { id = "plant", type = "plant_crop", target = "any", count = 1, description = "Plant a crop" },
    { id = "harvest", type = "harvest_crop", target = "any", count = 1, description = "Harvest the crop", depends_on = "plant" },
]
rewards = [
    { type = "gold", amount = 100 },
    { type = "item", item = "turnip_seed", count = 5 },
]
prerequisites = []
```

- [ ] 实现任务数据加载
- [ ] 实现目标追踪（kill/gather/craft/reach/talk）
- [ ] 实现目标依赖（depends_on）
- [ ] 实现前置条件检查（prerequisites）
- [ ] 实现奖励发放
- [ ] 实现任务状态机（inactive/active/completed/failed）
- [ ] 实现任务 UI（任务日志、目标追踪、地图标记）

### 5.8 叙事能力 (`nt_game_ability_narrative`)

- [ ] 实现变量系统（全局变量 + 局部变量）
- [ ] 实现条件分支（if/else/switch 基于变量）
- [ ] 实现事件触发器（到达区域/击败敌人/获得物品）
- [ ] 实现时间触发器（第 N 天 / 特定时间）
- [ ] 实现多结局路径追踪
- [ ] 实现叙事日志（记录玩家选择）

---

## Layer 6: 物理碰撞

### 6.1 碰撞检测

```rust
// nt_game_physics/src/collision.rs
pub struct Collider {
    pub shape: CollisionShape,
    pub offset: Vec2,
    pub is_sensor: bool,  // 触发器（无物理碰撞）
}

pub enum CollisionShape {
    AABB { size: Vec2 },
    Circle { radius: f32 },
    Polygon { vertices: Vec<Vec2> },
}
```

- [ ] 实现 AABB 碰撞检测
- [ ] 实现圆形碰撞检测
- [ ] 实现 AABB vs Circle 碰撞
- [ ] 实现射线检测（raycast）
- [ ] 实现触发器（sensor，无物理响应）

### 6.2 碰撞响应

- [ ] 实现碰撞事件（`on_collision_enter` / `on_collision_exit`）
- [ ] 实现物理推开（push-back）
- [ ] 实现单向碰撞平台（从下方穿过，从上方落下）
- [ ] 实现碰撞层过滤（哪些层可以碰撞）
- [ ] 实现空间分区（grid/quadtree 优化大量实体）

### 6.3 简单移动

```rust
// nt_game_physics/src/movement.rs
pub struct Velocity {
    pub linear: Vec2,
    pub angular: f32,
}

pub struct Movement {
    pub velocity: Velocity,
    pub max_speed: f32,
    pub friction: f32,
    pub acceleration: f32,
}
```

- [ ] 实现基于速度的移动
- [ ] 实现摩擦力/阻力
- [ ] 实现加速度/减速度
- [ ] 实现朝向自动更新（根据移动方向）

---

## Layer 7: 音频系统

### 7.1 音频管理器

```rust
// nt_game_audio/src/manager.rs
pub struct AudioManager {
    music_track: Option<MusicHandle>,
    sfx_pool: Vec<SfxHandle>,
    ambient: Vec<AmbientHandle>,
    master_volume: f32,
    music_volume: f32,
    sfx_volume: f32,
}
```

- [ ] 实现 BGM 播放（循环、淡入淡出、交叉渐变）
- [ ] 实现 SFX 播放（一次性、可重叠）
- [ ] 实现环境音（雨声、风声、鸟鸣）
- [ ] 实现音量控制（master/music/sfx/ambient 独立）
- [ ] 实现空间音频（距离衰减 + 左右声道）

### 7.2 音频触发

- [ ] 实现事件 → 音效映射（TOML 配置）
- [ ] 实现冷却（同一音效 N 秒内不重复）
- [ ] 实现随机音效（多个变体随机选择）
- [ ] 实现动态混音（战斗时降低 BGM 音量）
- [ ] 实现音频设置 UI（音量滑块、静音开关）

### 7.3 音频数据

```toml
# assets/audio.toml
[music]
exploration = { file = "music/exploration.ogg", volume = 0.7, fade_in = 2.0 }
battle = { file = "music/battle.ogg", volume = 0.8, fade_in = 0.5 }
festival = { file = "music/festival.ogg", volume = 0.6, fade_in = 3.0 }

[sfx]
pick_up = { file = "sfx/pick_up.wav", volume = 0.8 }
water_plant = { file = "sfx/water.wav", volume = 0.6, variants = ["sfx/water_1.wav", "sfx/water_2.wav"] }
hit = { file = "sfx/hit.wav", volume = 0.9, cooldown = 0.1 }
```

---

## Layer 8: UI 系统

### 8.1 UI 框架

```rust
// nt_game_ui/src/layout.rs
pub struct UILayout {
    pub root: UINode,
    pub focus: Option<UINodeId>,
    pub navigation: NavigationMode, // Keyboard/Controller/Touch
}

pub enum UINode {
    Panel { background: Color, border: Option<Border> },
    Label { text: String, font: FontId, size: f32, color: Color },
    Button { label: String, on_click: Callback },
    Image { sprite: Sprite, tint: Color },
    Grid { columns: u32, spacing: Vec2, children: Vec<UINode> },
    List { direction: Direction, children: Vec<UINode> },
    Scroll { content: Box<UINode>, scroll_pos: Vec2 },
}
```

- [ ] 实现声明式 UI 布局（TOML 或 Rust builder）
- [ ] 实现 UI 节点树
- [ ] 实现布局计算（flexbox-like 或 grid）
- [ ] 实现 UI 事件（hover/click/scroll）
- [ ] 实现焦点管理（键盘/手柄导航）

### 8.2 常见 UI 组件

- [ ] **主菜单**（新游戏/继续/设置/退出）
- [ ] **设置菜单**（音量/键位/画面/语言）
- [ ] **暂停菜单**（继续/设置/保存/退出）
- [ ] **HUD**（HP 条/金币/小地图/任务追踪）
- [ ] **对话框**（头像/名字/文本/选项）
- [ ] **背包**（网格/拖拽/排序/筛选）
- [ ] **制作面板**（配方列表/材料需求/制作按钮）
- [ ] **任务日志**（活跃/已完成/奖励）
- [ ] **地图**（全屏/小地图/标记）
- [ ] **商店**（购买/出售/价格显示）
- [ ] **Tooltip**（物品信息/技能描述/状态效果）
- [ ] **通知系统**（获得物品/完成任务/成就解锁）

### 8.3 UI 动画

- [ ] 实现淡入淡出（fade in/out）
- [ ] 实现滑入滑出（slide in/out）
- [ ] 实现弹跳效果（bounce）
- [ ] 实现打字机效果（逐字显示文本）
- [ ] 实现 HP 条平滑变化（lerp）

---

## Layer 9: 存档系统

### 9.1 自动存档

```rust
// nt_game_save/src/auto_save.rs
pub struct AutoSave {
    interval: Duration,        // 默认 60 秒
    max_auto_saves: usize,     // 默认 10
    save_dir: PathBuf,
}
```

- [ ] 实现定时自动存档（可配置间隔）
- [ ] 实现存档槽位（auto_1, auto_2, ..., auto_N 循环覆盖）
- [ ] 实现存档元数据（时间、游戏时长、缩略图、版本）
- [ ] 实现存档列表 UI

### 9.2 手动存档

- [ ] 实现手动保存/加载（玩家主动操作）
- [ ] 实现存档命名（自定义名称）
- [ ] 实现存档删除确认
- [ ] 实现存档覆盖保护

### 9.3 快照回滚（宽恕引擎）

```rust
// nt_game_save/src/rollback.rs
pub struct RollbackSystem {
    snapshots: VecDeque<GameStateSnapshot>,
    max_snapshots: usize,  // 默认 100
    current_index: usize,
}
```

- [ ] 实现状态快照（每 N 帧自动创建）
- [ ] 实现回滚到任意快照（undo/redo）
- [ ] 实现快照压缩（差异存储）
- [ ] 实现回滚 UI（时间线可视化）

### 9.4 云同步（可选）

- [ ] 实现存档序列化为字节流
- [ ] 实现上传/下载 API
- [ ] 实现冲突解决（last-write-wins）
- [ ] 实现存档版本兼容（旧版本存档迁移）

---

## Layer 10: 网络层（可选）

### 10.1 连接管理

- [ ] 实现客户端-服务端架构
- [ ] 实现连接握手
- [ ] 实现心跳检测
- [ ] 实现断线重连

### 10.2 状态同步

- [ ] 实现快照同步（全量状态发送）
- [ ] 实现增量同步（只发送变化）
- [ ] 实现客户端预测（减少延迟感）
- [ ] 实现服务端验证（防作弊）

### 10.3 延迟补偿

- [ ] 实现输入延迟补偿（input delay）
- [ ] 实现状态插值（interpolation）
- [ ] 实现回滚网络（rollback netcode）

---

## Layer 11: 模组系统

### 11.1 模组加载

```rust
// nt_game_mod/src/loader.rs
pub struct ModLoader {
    mods_dir: PathBuf,
    active_mods: Vec<LoadedMod>,
    load_order: Vec<ModId>,
}

pub struct LoadedMod {
    manifest: ModManifest,
    data_overrides: HashMap<PathBuf, DataOverride>,
    ability_extensions: Vec<AbilityExtension>,
}
```

- [ ] 实现模组清单解析（`mod.toml`）
- [ ] 实现数据文件覆盖（模组数据 > 原版数据）
- [ ] 实现能力扩展（新能力或修改现有能力）
- [ ] 实现模组依赖解析
- [ ] 实现模组启用/禁用

### 11.2 模组沙箱

- [ ] 实现模组文件隔离（不能访问其他模组文件）
- [ ] 实现 API 权限控制（模组可用/禁用的 API 列表）
- [ ] 实现资源限制（内存、CPU 时间）
- [ ] 实现崩溃隔离（模组崩溃不影响主游戏）

### 11.3 模组生态

- [ ] 实现模组版本管理
- [ ] 实现模组兼容性检查
- [ ] 实现模组市场 API（上传/下载/评分）
- [ ] 实现模组更新通知

---

## Layer 12: AI 系统

### 12.1 NPC 行为

```rust
// nt_game_ai/src/behavior.rs
pub trait NpcBehavior: 'static + Send + Sync {
    fn think(&mut self, world: &WorldState, npc: &Npc) -> NpcAction;
    fn remember(&mut self, event: &GameEvent);
    fn feel(&mut self, situation: &Situation) -> Emotion;
    fn personality(&self) -> &Personality;
}
```

- [ ] 实现行为树（Behavior Tree）
- [ ] 实现状态机（FSM for simple NPCs）
- [ ] 实现效用系统（Utility AI for complex decisions）
- [ ] 实现记忆系统（最近事件、重要事件、遗忘曲线）
- [ ] 实现情感模型（PAD: Pleasure/Arousal/Dominance）
- [ ] 实现人格模型（Big Five: OCEAN）

### 12.2 自适应难度

- [ ] 实现玩家技能评估（基于表现）
- [ ] 实现挫败感检测（连续失败次数）
- [ ] 实现参与度目标（保持 "心流" 状态）
- [ ] 实现动态调整（敌人HP/伤害/掉落率）

### 12.3 程序化内容

- [ ] 实现对话生成（基于 NPC 性格 + 当前状态）
- [ ] 实现事件生成（随机事件、季节事件）
- [ ] 实现地图生成（房间、宝箱、敌人分布）

---

## Layer 13: 手感打磨（Juice）

### 13.1 视觉反馈

- [ ] 屏幕震动（hit, explosion, landing）
- [ ] 闪光效果（damage flash, heal glow）
- [ ] 慢动作（critical hit, last enemy）
- [ ] 拖影（motion blur for fast movement）
- [ ] 粒子效果（sparks, dust, leaves, rain）

### 13.2 动画反馈

- [ ] 角色弹跳（landing squash & stretch）
- [ ] UI 弹跳（button press, notification pop）
- [ ] 伤害数字弹出（floating combat text）
- [ ] 物品获得动画（fly to inventory）

### 13.3 音频反馈

- [ ] 动态音乐强度（战斗激烈 → 音乐加快）
- [ ] 环境音变化（进入洞穴 → 回声）
- [ ] UI 音效（button click, menu open, item pickup）
- [ ] 脚步声（不同地面材质）

### 13.4 手柄反馈

- [ ] 振动反馈（hit, explosion, heartbeat）
- [ ] 自适应扳机（拉弓阻力）
- [ ] HD 震动（细腻触感）

---

## Layer 14: 无障碍

### 14.1 视觉无障碍

- [ ] 色盲模式（protanopia/deuteranopia/tritanopia）
- [ ] 高对比度模式
- [ ] 可调节 UI 缩放
- [ ] 字体大小可调
- [ ] 屏幕闪烁警告（癫痫安全）

### 14.2 操作无障碍

- [ ] 完整键位重映射
- [ ] 单手操作模式
- [ ] 辅助瞄准（aim assist）
- [ ] 连击重复（hold to repeat）
- [ ] QTE 替代方案（跳过/自动完成）

### 14.3 听觉无障碍

- [ ] 字幕系统（对话 + 环境音描述）
- [ ] 视觉化音效指示（方向指示器）
- [ ] 节奏游戏替代方案（视觉提示）

### 14.4 认知无障碍

- [ ] 简化模式（减少 UI 元素）
- [ ] 任务标记（清晰的目标指引）
- [ ] 进度提示（"你上次在这里"）
- [ ] 提示系统（卡住时可查看提示）

---

## Layer 15: 本地化

### 15.1 文本本地化

```toml
# assets/locales/en.toml
[game.title]
en = "Cozy Farm"
zh = "温馨农场"
ja = "コージーファーム"

[crop.turnip.name]
en = "Turnip"
zh = "萝卜"
ja = "カブ"
```

- [ ] 实现本地化数据加载（key → 多语言映射）
- [ ] 实现运行时语言切换
- [ ] 实现复数规则（one/other/few/many）
- [ ] 实现变量插值（`"You earned {gold} gold"`）
- [ ] 实现文本方向（LTR/RTL）

### 15.2 资源本地化

- [ ] 实现纹理本地化（不同语言的 UI 图片）
- [ ] 实现音频本地化（不同语言的配音）
- [ ] 实现字体本地化（CJK 字体支持）

### 15.3 文化适配

- [ ] 实现日期格式本地化（MM/DD/YYYY vs DD/MM/YYYY）
- [ ] 实现货币格式本地化（$100 vs 100¥）
- [ ] 实现颜色文化适配（红色在不同文化含义不同）

---

## Layer 16: 发布分发

### 16.1 构建配置

```toml
# Cargo.toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
panic = "abort"

[profile.release.package."*"]
opt-level = 3
```

- [ ] 配置 release profile（LTO、strip、codegen-units）
- [ ] 实现增量编译优化
- [ ] 实现构建时间优化（sccache）

### 16.2 平台打包

- [ ] **Windows**: NSIS 安装包 / MSIX
- [ ] **macOS**: DMG（codesign + notarize）
- [ ] **Linux**: AppImage / Flatpak / .deb
- [ ] **Web**: Wasm + HTML（`wasm-pack build --target web`）
- [ ] **Android**: APK / AAB
- [ ] **iOS**: IPA（需要 Apple 开发者账号）

### 16.3 商店上架

- [ ] Steam: Steamworks SDK 集成（成就、排行榜、云存档）
- [ ] Itch.io: DRM-free 打包
- [ ] Google Play: Play Billing Library
- [ ] App Store: StoreKit 集成
- [ ] Web: 自托管或 itch.io web

### 16.4 自动更新

- [ ] 实现版本检查（HTTP API）
- [ ] 实现增量更新（只下载变化的文件）
- [ ] 实现更新提示 UI
- [ ] 实现强制更新（关键版本）

### 16.5 遥测（可选）

- [ ] 实现匿名游玩统计（时长、完成率、放弃点）
- [ ] 实现崩溃报告（sentry 或自建）
- [ ] 实现性能监控（FPS、内存、加载时间）
- [ ] 实现 GDPR/隐私合规（opt-in）

---

## 完成判定标准

### 最小可玩（MVP）

| 层 | 必需 | 可选 |
|---|------|------|
| L0 项目骨架 | workspace + crate + CI | - |
| L1 核心引擎 | 游戏循环 + 状态管理 + 事件 | 时间系统 |
| L2 数据管线 | TOML 加载 + 类型安全 | 热重载 + Schema |
| L3 渲染系统 | macroquad 后端 + 精灵 | 粒子 + 相机特效 |
| L4 输入系统 | 键盘 + 鼠标 | 手柄 + 触屏 |
| L5 游戏能力 | 2-3 个能力模块 | 全部 7 个 |
| L6 物理碰撞 | AABB 碰撞 | 射线检测 |
| L7 音频系统 | BGM + SFX | 空间音频 |
| L8 UI 系统 | 主菜单 + HUD + 对话框 | 全部 UI 组件 |
| L9 存档系统 | 自动存档 + 手动存档 | 回滚 + 云同步 |
| L10 网络层 | - | 全部 |
| L11 模组系统 | - | 全部 |
| L12 AI 系统 | 简单状态机 | 行为树 + 效用系统 |
| L13 手感打磨 | 屏幕震动 + 粒子 | 全部 |
| L14 无障碍 | 键位重映射 | 全部 |
| L15 本地化 | 英文 + 中文 | 全部语言 |
| L16 发布分发 | 桌面打包 | 全平台 |

### 发布就绪（Release Ready）

- [ ] 所有 MVP 层完成
- [ ] 通过 `cargo clippy -- -D warnings`
- [ ] 测试覆盖率 > 80%（核心模块）
- [ ] 所有 UI 组件完成
- [ ] 至少 3 个平台打包验证
- [ ] 无障碍检查通过（WCAG 2.1 AA）
- [ ] 本地化完成（至少 2 语言）
- [ ] 性能基准测试（目标 60fps on mid-range hardware）
- [ ] 内存使用 < 512MB（桌面）/ < 256MB（移动）
- [ ] 冷启动 < 3 秒

---

## 实施路线图

### Phase 1: 骨干（Week 1-2）

```
L0 → L1 → L2
目标：cargo run 能弹出窗口、加载 TOML 数据、渲染一个精灵
产出：nt_game_core + nt_game_data + nt_game_render (macroquad)
```

### Phase 2: 交互（Week 3-4）

```
L4 → L3 → L6 → L7
目标：角色能移动、碰撞、播放音效
产出：nt_game_input + nt_game_physics + nt_game_audio
```

### Phase 3: 玩法（Week 5-8）

```
L5 (选择 2-3 个核心能力) → L8 → L9
目标：完整的游戏循环（探索 → 交互 → 成长 → 存档）
产出：2-3 个 ability crate + nt_game_ui + nt_game_save
```

### Phase 4: 打磨（Week 9-12）

```
L13 → L14 → L15
目标：手感流畅、视觉丰富、可本地化
产出：粒子系统 + UI 动画 + 无障碍 + 本地化
```

### Phase 5: 发布（Week 13-16）

```
L16 → 测试 → 修复 → 发布
目标：打包上架
产出：可发布的游戏
```

### 可选扩展（Phase 6+）

```
L10 网络层 → L11 模组系统 → L12 AI 系统
目标：多人游戏 + 模组支持 + 智能 NPC
```
