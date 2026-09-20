# NT-GAME 重构方案 — 从文本训练到视觉游戏

**日期**: 2026-09-20
**现状**: NT-GAME 是纯文本 LLM 训练框架（text-native observations, self-play, consciousness feedback）
**目标**: 在保留训练能力的基础上，增加视觉渲染层，使其能构建真正的可玩游戏

---

## 现状诊断

### 已有的（保留）

| 模块 | 行数 | 状态 | 保留策略 |
|------|------|------|---------|
| `framework.rs` | 613 | 完整 | **保留** — Actor/Episode/Rubric/Arena 是核心抽象 |
| `env.rs` | 280 | 完整 | **保留** — NtGameEnv trait 是游戏接口 |
| `ecs/` | ~220 | 基础可用 | **增强** — 添加 Position/Velocity/Sprite 组件 |
| `rpg/` | 621 | 完整 | **保留** — 角色/装备/技能树/修炼体系完整 |
| `world/combat/` | 完整 | 完整 | **保留** — 战斗状态/伤害/装备 |
| `world/npc/` | 完整 | 完整 | **保留** — NPC/对话树/关系 |
| `world/quest/` | 完整 | 完整 | **保留** — 任务/战利品 |
| `world/tilemap/` | 完整 | 完整 | **保留** — 瓦片地图/图层 |
| `world/generation/` | 完整 | 完整 | **保留** — 程序化世界生成 |
| `world/weather.rs` | 完整 | 完整 | **保留** — 天气系统 |
| `world/season.rs` | 完整 | 完整 | **保留** — 季节系统 |
| `world/time_system.rs` | 完整 | 完整 | **保留** — 游戏时钟 |
| `ai/` | 完整 | 完整 | **保留** — 行为树/策略 |
| `play/` | 完整 | 完整 | **保留** — 自适应难度/GRPO |
| `consciousness/` | 完整 | 完整 | **保留** — 意识反馈 |
| `evolution.rs` | 893 | 完整 | **保留** — 自主训练 |
| `mcp.rs` | 781 | 完整 | **保留** — MCP 工具接口 |

### 缺失的（需要新增）

| 缺失层 | 说明 | 优先级 |
|--------|------|--------|
| **渲染层** | 无任何视觉渲染代码 | P0 |
| **资产管线** | 无 sprite/texture/audio 加载 | P0 |
| **游戏循环（视觉版）** | 现有循环是文本 step-based，需要帧循环 | P0 |
| **UI 系统** | 无菜单/HUD/对话框 | P1 |
| **输入抽象** | world/input/ 存在但无跨平台抽象 | P1 |
| **存档系统** | persistence/ 有 replay 但无视觉游戏存档 | P1 |
| **数据驱动加载** | 无 TOML → 运行时数据管线 | P2 |

---

## 重构架构

### 核心设计：双模式架构

```
┌─────────────────────────────────────────────────┐
│                 nt_game (顶层)                    │
│                                                  │
│  ┌──────────────┐      ┌──────────────┐         │
│  │  Text Mode   │      │ Visual Mode  │         │
│  │  (训练用)    │      │ (游戏用)     │         │
│  │              │      │              │         │
│  │ NtGameEnv    │◄────►│ GameRuntime  │         │
│  │ text obs     │ bridge│ visual render│         │
│  │ self-play    │      │ asset load   │         │
│  └──────────────┘      └──────────────┘         │
│         │                      │                 │
│         └──────────┬───────────┘                 │
│                    │                             │
│  ┌─────────────────▼───────────────────────┐     │
│  │           共享层（已有）                   │     │
│  │  ECS / RPG / World / AI / Consciousness  │     │
│  └─────────────────────────────────────────┘     │
└─────────────────────────────────────────────────┘
```

### 新增模块

```
nt_game/
├── ... (已有模块保持不变)
├── runtime/                    # 新增：视觉游戏运行时
│   ├── mod.rs                  # GameRuntime 主结构
│   ├── game_loop.rs            # 帧循环（固定时间步）
│   ├── renderer.rs             # 渲染器 trait + macroquad 实现
│   ├── camera.rs               # 相机系统
│   └── render_pipeline.rs      # 渲染管线（排序/批处理）
├── assets/                     # 新增：资产管线
│   ├── mod.rs                  # AssetManager
│   ├── loader.rs               # 资产加载（图片/音频/字体）
│   ├── cache.rs                # 资产缓存
│   ├── hot_reload.rs           # 热重载
│   └── sprite.rs               # Sprite/SpriteSheet
├── ui/                         # 新增：UI 系统
│   ├── mod.rs                  # UiSystem
│   ├── layout.rs               # UI 布局
│   ├── widgets/                # UI 组件
│   │   ├── mod.rs
│   │   ├── button.rs
│   │   ├── label.rs
│   │   ├── panel.rs
│   │   ├── dialog.rs
│   │   ├── inventory.rs
│   │   ├── hud.rs
│   │   └── tooltip.rs
│   └── style.rs                # UI 样式
├── input/                      # 新增：输入抽象（替代 world/input）
│   ├── mod.rs                  # InputManager
│   ├── keyboard.rs
│   ├── mouse.rs
│   ├── gamepad.rs
│   └── action_map.rs           # 动作映射
├── save/                       # 新增：存档系统
│   ├── mod.rs                  # SaveManager
│   ├── auto_save.rs
│   ├── slot.rs
│   └── serialization.rs
└── bridge/                     # 新增：文本↔视觉桥接
    ├── mod.rs                  # TextToVisualBridge
    ├── observation_renderer.rs # 文本观察 → 可视化
    └── state_sync.rs           # 训练状态 ↔ 游戏状态同步
```

---

## 实施步骤

### Step 1: 增强 ECS（添加视觉组件）

在现有 `ecs/component.rs` 上添加：

```rust
// 新增视觉相关组件
#[derive(Debug, Clone)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone)]
pub struct SpriteComponent {
    pub texture_id: String,
    pub source_rect: Option<(f32, f32, f32, f32)>,  // x, y, w, h
    pub flip_x: bool,
    pub flip_y: bool,
    pub tint: (f32, f32, f32, f32),  // RGBA
    pub z_order: i32,
}

#[derive(Debug, Clone)]
pub struct AnimationComponent {
    pub frames: Vec<String>,  // texture_ids
    pub current_frame: usize,
    pub frame_time: f32,
    pub elapsed: f32,
    pub looping: bool,
}

#[derive(Debug, Clone)]
pub struct ColliderComponent {
    pub width: f32,
    pub height: f32,
    pub is_sensor: bool,
}

#[derive(Debug, Clone)]
pub struct CameraFollow {
    pub smooth: f32,  // 插值系数
}
```

### Step 2: 创建渲染器 trait

```rust
// nt_game/runtime/renderer.rs
pub trait Renderer: Send + Sync {
    fn init(&mut self, width: u32, height: u32, title: &str) -> Result<()>;
    fn begin_frame(&mut self, clear_color: (f32, f32, f32, f32));
    fn end_frame(&mut self);

    // 精灵
    fn draw_sprite(&mut self, texture: &str, x: f32, y: f32, w: f32, h: f32,
                   source: Option<(f32, f32, f32, f32)>, flip_x: bool, tint: (f32, f32, f32, f32));

    // 几何
    fn draw_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: (f32, f32, f32, f32));
    fn draw_circle(&mut self, x: f32, y: f32, r: f32, color: (f32, f32, f32, f32));
    fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: (f32, f32, f32, f32));

    // 文字
    fn draw_text(&mut self, text: &str, x: f32, y: f32, size: f32, color: (f32, f32, f32, f32));
    fn measure_text(&self, text: &str, size: f32) -> (f32, f32);

    // 纹理管理
    fn load_texture(&mut self, id: &str, path: &str) -> Result<()>;
    fn texture_size(&self, id: &str) -> Option<(f32, f32)>;
}

// macroquad 实现
pub struct MacroquadRenderer {
    textures: HashMap<String, Texture2D>,
    camera: Camera2D,
}
```

### Step 3: 创建 GameRuntime

```rust
// nt_game/runtime/game_loop.rs
pub struct GameRuntime {
    // 已有
    world: World,                    // ECS World
    scheduler: SystemScheduler,      // 系统调度器
    npc_states: HashMap<EntityId, NpcState>,
    combat_state: Option<CombatState>,
    player: Character,
    tilemap: LayeredTileMap,
    weather: WeatherState,
    season: SeasonState,
    clock: GameClock,

    // 新增
    renderer: Box<dyn Renderer>,
    asset_manager: AssetManager,
    input_manager: InputManager,
    ui_system: UiSystem,
    save_manager: SaveManager,
    camera: Camera,

    // 游戏循环
    target_fps: u32,
    accumulator: f64,
    fixed_dt: f64,
    running: bool,
}

impl GameRuntime {
    pub fn new(config: GameConfig) -> Result<Self> { ... }

    pub fn run(&mut self) -> Result<()> {
        loop {
            // 1. 处理输入
            self.input_manager.update();

            // 2. 固定时间步更新
            let dt = self.get_frame_time();
            self.accumulator += dt;
            while self.accumulator >= self.fixed_dt {
                self.update(self.fixed_dt);
                self.accumulator -= self.fixed_dt;
            }

            // 3. 渲染（alpha 插值）
            let alpha = self.accumulator / self.fixed_dt;
            self.render(alpha);

            // 4. 检查退出
            if self.input_manager.quit_requested() || !self.running {
                break;
            }
        }
        Ok(())
    }

    fn update(&mut self, dt: f64) {
        // 输入处理
        self.handle_input();

        // ECS 系统更新
        let entities = self.world.entities.alive_ids();
        self.scheduler.run(&entities, &mut self.world.components, dt);

        // 游戏逻辑
        self.update_npcs(dt);
        self.update_weather(dt);
        self.clock.advance(dt);
        self.update_season();

        // UI 更新
        self.ui_system.update(dt);
    }

    fn render(&mut self, alpha: f64) {
        self.renderer.begin_frame((0.1, 0.1, 0.1, 1.0));

        // 1. 渲染瓦片地图
        self.render_tilemap();

        // 2. 渲染实体（按 z_order 排序）
        self.render_entities();

        // 3. 渲染粒子
        self.render_particles();

        // 4. 渲染 UI
        self.ui_system.render(&mut *self.renderer);

        self.renderer.end_frame();
    }
}
```

### Step 4: 资产管线

```rust
// nt_game/assets/loader.rs
pub struct AssetManager {
    base_path: PathBuf,
    textures: HashMap<String, TextureData>,
    audio: HashMap<String, AudioData>,
    fonts: HashMap<String, FontData>,
    cache: LruCache<String, AssetId>,
}

impl AssetManager {
    pub fn load_texture(&mut self, id: &str, path: &str) -> Result<()> { ... }
    pub fn load_audio(&mut self, id: &str, path: &str) -> Result<()> { ... }
    pub fn load_font(&mut self, id: &str, path: &str) -> Result<()> { ... }

    // 从 TOML 批量加载
    pub fn load_from_manifest(&mut self, manifest_path: &str) -> Result<()> { ... }
}

// 资产清单
// assets/manifest.toml
[textures]
"player" = "sprites/player.png"
"tileset" = "sprites/tileset.png"
"items" = "sprites/items.png"
"ui" = "sprites/ui.png"

[audio]
"bgm_explore" = "audio/explore.ogg"
"sfx_hit" = "audio/hit.wav"

[fonts]
"default" = "fonts/main.ttf"
"dialogue" = "fonts/dialogue.ttf"
```

### Step 5: UI 系统

```rust
// nt_game/ui/mod.rs
pub struct UiSystem {
    root: UiNode,
    focused: Option<UiNodeId>,
    style: UiStyle,
    visible: bool,
}

// UI 节点类型
pub enum UiNode {
    Panel { background: Color, border: Option<f32> },
    Label { text: String, size: f32, color: Color },
    Button { label: String, on_click: Box<dyn Fn()>, hover: bool },
    Image { texture: String, tint: Color },
    Grid { columns: u32, spacing: f32, children: Vec<UiNodeId> },
    List { direction: Direction, children: Vec<UiNodeId> },
    Dialog { title: String, content: Box<UiNode>, buttons: Vec<(String, Box<dyn Fn()>)> },
}

// 预制 UI
impl UiSystem {
    pub fn main_menu(on_new: impl Fn(), on_continue: impl Fn(), on_settings: impl Fn(), on_quit: impl Fn()) -> Self { ... }
    pub fn hud(hp: u32, max_hp: u32, mp: u32, max_mp: u32, gold: u32, level: u32) -> Self { ... }
    pub fn dialogue_box(speaker: &str, portrait: &str, text: &str, choices: &[(String, Box<dyn Fn()>)]) -> Self { ... }
    pub fn inventory_grid(items: &[Option<Item>], on_select: impl Fn(usize)) -> Self { ... }
}
```

### Step 6: 文本↔视觉桥接

```rust
// nt_game/bridge/mod.rs
/// 桥接 LLM 训练模式和视觉游戏模式
pub struct TextToVisualBridge {
    /// 将文本观察转换为可视化状态
    pub fn observation_to_visual(obs: &Observation) -> VisualState { ... }

    /// 将视觉游戏状态转换为文本观察（供 LLM 使用）
    pub fn state_to_observation(state: &GameState, player: &Character) -> Observation { ... }

    /// 同步训练模式和视觉模式的状态
    pub fn sync_states(
        text_env: &mut dyn NtGameEnv,
        visual_runtime: &mut GameRuntime,
    ) -> Result<()> { ... }
}
```

---

## 最小可玩 Demo 路径

### 目标：一个能跑的 cozy farm demo

```
Week 1: 增强 ECS + 创建 GameRuntime + macroquad 渲染
  → cargo run 能弹出窗口、渲染瓦片地图、角色能移动

Week 2: 资产管线 + UI 系统
  → 能加载 sprite、显示 HUD、打开背包

Week 3: 接入已有世界子系统
  → 对话系统能工作、天气/季节能渲染、种植能交互

Week 4: 存档 + 打磨
  → 能存档/读档、粒子效果、屏幕震动
```

### Demo 代码预览

```rust
// examples/cozy_farm/src/main.rs
use nt_game::prelude::*;

fn main() -> Result<()> {
    let mut runtime = GameRuntime::new(GameConfig {
        title: "Cozy Farm",
        width: 1280,
        height: 720,
        target_fps: 60,
        assets_path: "assets/".into(),
    })?;

    // 加载资产
    runtime.assets().load_from_manifest("assets/manifest.toml")?;

    // 创建世界
    let tilemap = LayeredTileMap::from_toml("assets/maps/farm.toml")?;
    runtime.set_tilemap(tilemap);

    // 创建玩家
    let mut player = Character::new("Farmer");
    player.stats.add_bonus(Stat::Str, 3);
    runtime.set_player(player);

    // 添加 NPC
    let maru = NpcState::new("Maru", NpcRole::Friend, vec!["mining".into()]);
    runtime.add_npc("maru", maru);

    // 启用能力
    runtime.enable_farming();
    runtime.enable_dialogue();
    runtime.enable_inventory();

    // 运行
    runtime.run()?;
    Ok(())
}
```

---

## 与 NeoTrix 其他模块的接线

| NeoTrix 模块 | 接线方式 | 用途 |
|-------------|---------|------|
| `neotrix-consciousness` | `consciousness/` 已有桥接 | NPC 行为受意识系统影响 |
| `neotrix-reasoning` | 可选集成 | NPC 决策使用推理引擎 |
| `nt-lang` | 可选集成 | 游戏内脚本语言 |
| `neotrix-gateway` | `mcp.rs` 已有接口 | MCP 工具：game_list/create/step |
| `neotrix-multi-agent` | 可选集成 | 多 NPC 协调 |
| `nt_file_ability` | 资产管线 | 文件解析能力 |

---

## 验证标准

### 编译验证
- [ ] `cargo check -p neotrix-core` 通过（已有模块不破坏）
- [ ] `cargo test -p neotrix-core --lib` 通过（已有测试不失败）
- [ ] `cargo build --example cozy_farm` 通过（新 demo 可构建）

### 功能验证
- [ ] `cargo run --example cozy_farm` 弹出窗口
- [ ] 窗口中渲染瓦片地图
- [ ] 角色可移动（WASD/方向键）
- [ ] NPC 可对话
- [ ] 天气/季节可视化
- [ ] HUD 显示 HP/金币/等级
- [ ] 背包可打开/关闭
- [ ] 存档/读档工作

### 性能验证
- [ ] 60fps on mid-range hardware
- [ ] 内存 < 256MB
- [ ] 冷启动 < 3 秒
