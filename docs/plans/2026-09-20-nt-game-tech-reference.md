# NT-GAME 技术参考手册 — 开源生态调研汇总

**日期**: 2026-09-20
**来源**: 5 个并行研究任务（Rust 引擎生态 / ECS 架构 / 数据管线 / UI 系统 / 存档系统）
**用途**: 为 NT-GAME 重构提供技术选型和实现参考

---

## 1. Rust 游戏引擎生态（2026）

### 引擎对比

| 引擎 | Stars | 架构 | 许可证 | 最佳场景 |
|------|-------|------|--------|---------|
| **Bevy** | 48K | ECS 数据驱动 | MIT/Apache | 复杂游戏、长期项目 |
| **Macroquad** | 4.6K | 即时模式 | MIT | 原型、WASM、学习 |
| **ggez** | 4.7K | OOP 回调 | MIT | LÖVE/Pygame 迁移 |
| **Fyrox** | 9.5K | 场景图 | MIT | 需要编辑器的团队 |

### 关键决策：Macroquad vs Bevy

| 维度 | Macroquad | Bevy |
|------|-----------|------|
| **架构** | 即时模式、全局状态 | ECS、数据驱动 |
| **学习曲线** | 极低 | 陡峭（Rust + ECS） |
| **编译时间** | ~16s | 3-7 分钟首次 |
| **WASM** | 极佳（200KB-2MB） | 良好（较大） |
| **规模上限** | 小中型 | 大型复杂 |
| **社区** | 4.6K | 48K |
| **并行** | 单线程 | 自动多线程 |
| **资产管线** | 基础 async | Handle + 热重载 |
| **UI** | 即时模式 | ECS Flexbox |

**NT-GAME 选择**: Macroquad（快速原型 + WASM） + Bevy ECS（作为独立 ECS 库使用）

### Macroquad 核心模式

```rust
// 即时模式游戏循环
#[macroquad::main("GameName")]
async fn main() {
    loop {
        clear_background(DARKGREEN);
        // 输入处理
        if is_key_pressed(KeyCode::Space) { ... }
        // 游戏逻辑
        update();
        // 渲染
        draw_texture(texture, x, y, WHITE);
        draw_text("Score: 100", 10.0, 20.0, 30.0, WHITE);
        next_frame().await
    }
}
```

### Bevy ECS 核心模式

```rust
// ECS 游戏循环
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(PlayerScore(0))
        .add_systems(Startup, setup)
        .add_systems(Update, (move_player, check_collisions, update_ui))
        .run();
}

#[derive(Component)]
struct Player;

fn move_player(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    for mut transform in &mut query {
        if keyboard.pressed(KeyCode::ArrowRight) {
            transform.translation.x += 200.0 * time.delta_secs();
        }
    }
}
```

---

## 2. ECS 架构模式

### ECS 库对比

| 库 | 大小 | 存储方式 | 最佳场景 |
|---|------|---------|---------|
| **bevy_ecs** | 中等 | Archetype SoA | 功能完整、社区大 |
| **hecs** | ~1000 行 | Archetype | WASM、嵌入式、最小 |
| **legion** | 中等 | Archetype | 查询语法强大 |
| **自建** | 可控 | 可定制 | 学习、完全控制 |

### Archetype SoA 存储（推荐）

```
Archetype A (Player + Position + Velocity):
┌──────────┬──────────┬──────────┐
│ Entity   │ Position │ Velocity │
├──────────┼──────────┼──────────┤
│ 1        │ (0,0)    │ (1,0)    │
│ 2        │ (5,3)    │ (0,-1)   │
└──────────┴──────────┴──────────┘

Archetype B (Enemy + Position + Health):
┌──────────┬──────────┬──────────┐
│ Entity   │ Position │ Health   │
├──────────┼──────────┼──────────┤
│ 3        │ (10,5)   │ 50       │
│ 4        │ (8,2)    │ 30       │
└──────────┴──────────┴──────────┘
```

**优势**: 同类型组件连续存储，迭代缓存友好。游戏中实体很少改变 archetype，但系统每帧迭代数千实体。

### 推荐：自建轻量 ECS（300 行核心）

```rust
use std::any::{Any, TypeId};
use std::collections::HashMap;

pub type EntityId = u64;

// 组件存储：HashMap<TypeId, HashMap<EntityId, Box<dyn Any>>>
pub struct World {
    entities: HashMap<EntityId, Entity>,
    components: HashMap<TypeId, HashMap<EntityId, Box<dyn Any + Send + Sync>>>,
    next_id: EntityId,
}

impl World {
    pub fn spawn(&mut self) -> EntityId {
        let id = self.next_id;
        self.next_id += 1;
        self.entities.insert(id, Entity { id, alive: true });
        id
    }

    pub fn insert<T: 'static + Send + Sync>(&mut self, entity: EntityId, component: T) {
        self.components
            .entry(TypeId::of::<T>())
            .or_default()
            .insert(entity, Box::new(component));
    }

    pub fn get<T: 'static>(&self, entity: EntityId) -> Option<&T> {
        self.components
            .get(&TypeId::of::<T>())
            .and_then(|m| m.get(&entity))
            .and_then(|b| b.downcast_ref::<T>())
    }

    pub fn query<T: 'static>(&self) -> Vec<(EntityId, &T)> {
        self.components
            .get(&TypeId::of::<T>())
            .map(|m| {
                m.iter()
                    .filter_map(|(id, b)| b.downcast_ref::<T>().map(|t| (*id, t)))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn query2<A: 'static, B: 'static>(&self) -> Vec<(EntityId, &A, &B)> {
        let a_map = self.components.get(&TypeId::of::<A>());
        let b_map = self.components.get(&TypeId::of::<B>());
        match (a_map, b_map) {
            (Some(a), Some(b)) => a
                .iter()
                .filter_map(|(id, a_box)| {
                    let a = a_box.downcast_ref::<A>()?;
                    let b = b.get(id)?.downcast_ref::<B>()?;
                    Some((*id, a, b))
                })
                .collect(),
            _ => vec![],
        }
    }
}
```

---

## 3. 数据管线

### 数据格式选择

| 格式 | 最佳用途 | Rust 支持 | 特点 |
|------|---------|----------|------|
| **RON** | 游戏数据定义 | 原生 derive | Rust 语法、注释、枚举支持 |
| **TOML** | 简单扁平配置 | `toml` crate | 适合 Cargo.toml 风格 |
| **JSON** | API 交换 | `serde_json` | 通用、工具支持好 |
| **Bincode** | 生产存档 | `bincode` | 最快、最紧凑 |

**推荐**: RON 用于游戏数据，Bincode 用于存档

### RON 游戏数据示例

```ron
// assets/entities/ogre.ron
Entity(
    name: "Ogre",
    glyph: '@',
    stats: Stats(hp: 50, attack: 8, defense: 3),
    ai: Hostile,
    loot: [
        (item: "gold_pouch", chance: 0.7),
        (item: "health_potion", chance: 0.2),
    ],
)

// assets/config.ron
GameConfig(
    window_size: (800, 600),
    window_title: "Cozy Farm",
    key_bindings: {
        "up": Up,
        "down": Down,
    },
)
```

### 资产管线阶段

```
Load → Parse → Validate → Transform → Cache → Hot-Reload
  │        │         │           │         │          │
  ▼        ▼         ▼           ▼         ▼          ▼
std::fs  serde    serde_valid  ECS     HashMap    notify
         deserialize  验证    World 注入  Handle   文件监听
```

### assets_manager 推荐用法

```rust
use assets_manager::{Asset, AssetCache};
use serde::Deserialize;

#[derive(Deserialize, Asset)]
#[asset_format = "ron"]
struct Monster {
    name: String,
    hp: u32,
    damage: f32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cache = AssetCache::new("assets")?;
    let handle = cache.load::<Monster>("common.monster")?;
    let monster = handle.read(); // RwLock 读锁
    println!("{}: {} HP", monster.name, monster.hp);
    // 热重载自动发生 — handle.read() 始终返回最新数据
    Ok(())
}
```

### 热重载模式

```rust
use notify::{Config, Event, RecommendedWatcher, Watcher};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn setup_hot_reload(path: &str) -> Arc<AtomicBool> {
    let should_reload = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&should_reload);

    let mut watcher = RecommendedWatcher::new(
        move |res: notify::Result<Event>| {
            if let Ok(event) = res {
                if event.kind.is_modify() {
                    flag.store(true, Ordering::SeqCst);
                }
            }
        },
        Config::default(),
    ).unwrap();

    watcher.watch(path.into(), notify::RecursiveMode::Recursive).unwrap();
    should_reload
}

// 游戏循环中检查
if should_reload.load(Ordering::SeqCst) {
    should_reload.store(false, Ordering::SeqCst);
    reload_assets();
}
```

---

## 4. UI 系统

### 即时模式 vs 保留模式

| 模式 | 工作方式 | 最佳场景 | 代表 |
|------|---------|---------|------|
| **即时模式** | 每帧重建 UI | 调试工具、快速原型 | egui, Dear ImGui |
| **保留模式** | 构建一次，事件修改 | 发布的玩家 UI | Bevy UI, Godot |

**生产模式**: 保留模式用于玩家 UI + 即时模式用于调试工具

### UI 状态栈模式

```rust
pub enum Transition {
    Push(Box<dyn Screen>),
    Pop,
    Replace(Box<dyn Screen>),
    Quit,
}

trait Screen {
    fn on_start(&mut self, ctx: &mut GameContext);
    fn on_stop(&mut self, ctx: &mut GameContext);
    fn on_pause(&mut self, ctx: &mut GameContext);
    fn on_resume(&mut self, ctx: &mut GameContext);
    fn update(&mut self, ctx: &mut GameContext) -> Transition;
    fn draw(&self, ctx: &mut GameContext);
    fn handle_input(&mut self, input: &InputEvent) -> bool; // true = consumed
    fn blocks_update_below(&self) -> bool; // 暂停冻结游戏
    fn blocks_draw_below(&self) -> bool;   // 覆盖 vs 全屏
}

struct ScreenStack {
    screens: Vec<Box<dyn Screen>>,
}

impl ScreenStack {
    fn apply(&mut self, transition: Transition, ctx: &mut GameContext) {
        match transition {
            Transition::Push(screen) => {
                if let Some(top) = self.screens.last_mut() {
                    top.on_pause(ctx);
                }
                screen.on_start(ctx);
                self.screens.push(screen);
            }
            Transition::Pop => {
                if let Some(mut screen) = self.screens.pop() {
                    screen.on_stop(ctx);
                }
                if let Some(top) = self.screens.last_mut() {
                    top.on_resume(ctx);
                }
            }
            // ...
        }
    }
}
```

### HUD 架构原则

- **观察，不驱动**: HUD 订阅游戏状态事件，永不轮询或修改模拟
- **事件/观察者模式**: `health.changed.connect(|cur, max| { ui.health_bar.fill = cur / max; });`
- **信息层级**: 常驻一瞥 → 上下文瞬态 → 按需（专用屏幕）

### 对话系统数据模型

```rust
struct DialogueNode {
    id: String,
    speaker: String,
    text: Vec<String>,
    choices: Vec<DialogueChoice>,
    conditions: Vec<Condition>,  // 仅在条件满足时显示
    actions: Vec<Action>,        // 进入时执行
    next: Option<String>,        // 线性推进
}

struct DialogueChoice {
    text: String,
    next_id: String,
    conditions: Vec<Condition>,
    actions: Vec<Action>,
}
```

### 背包网格实现

```rust
struct InventoryGrid {
    cells: Vec<Vec<Option<Item>>>,
    width: usize,
    height: usize,
}

impl InventoryGrid {
    fn can_place(&self, item: &Item, x: usize, y: usize) -> bool {
        item.shape.iter().all(|(dx, dy)| {
            let (cx, cy) = (x + dx, y + dy);
            cx < self.width && cy < self.height
                && self.cells[cy][cx].is_none()
        })
    }

    fn place(&mut self, item: Item, x: usize, y: usize) -> bool {
        if !self.can_place(&item, x, y) { return false; }
        for (dx, dy) in &item.shape {
            self.cells[y + dy][x + dx] = Some(item.clone());
        }
        true
    }
}
```

### 渲染层级顺序

```
1. 游戏世界（2D/3D 场景）
2. 世界空间覆盖（伤害数字、名牌）
3. 屏幕空间覆盖（血条、小地图）
4. 菜单/背包（全屏，阻止下方输入）
5. 模态对话框（确认弹窗）
6. 过渡/加载屏幕
```

---

## 5. 存档系统

### 序列化格式性能

| 格式 | 大小 | 序列化 | 反序列化 | 人类可读 |
|------|------|--------|---------|---------|
| JSON | 100% | ~8ms | ~8ms | 是 |
| RON | ~88% | ~9ms | ~40ms | 是 |
| Bincode 2 | ~39% | ~2.4ms | ~2.9ms | 否 |
| Postcard | ~15% | ~1.1ms | ~1.7ms | 否 |

**推荐**: 开发用 JSON/RON，生产用 Bincode

### 存档文件结构

```rust
#[derive(Serialize, Deserialize)]
struct SaveFile {
    magic: [u8; 4],           // b"SAVE"
    version: u32,
    checksum: u32,            // CRC32 或 xxHash
    timestamp: u64,
    sections: HashMap<String, Vec<u8>>,  // 分区存储
}
```

### 版本迁移模式

```rust
#[derive(Serialize, Deserialize)]
enum SaveData {
    V1(SaveDataV1),
    V2(SaveDataV2),
    V3(SaveDataV3),  // 当前版本
}

impl SaveData {
    fn migrate(self) -> Self {
        match self {
            SaveData::V1(v1) => SaveData::V2(SaveDataV2 {
                position: v1.position,
                health: v1.hp,
                stamina: 100,
            }).migrate(),
            SaveData::V2(v2) => SaveData::V3(SaveDataV3 {
                position: Vec2::from(v2.position),
                health: v2.health,
                stamina: v2.stamina,
                perks: vec![],
            }),
            SaveData::V3(_) => self,
        }
    }
}
```

### 原子写入（崩溃安全）

```rust
fn atomic_save(target: &Path, data: &[u8]) -> std::io::Result<()> {
    let tmp = target.with_extension("tmp");
    // 1. 写入临时文件
    std::fs::write(&tmp, data)?;
    // 2. 原子重命名
    std::fs::rename(&tmp, target)?;
    // 3. 同步父目录
    if let Some(parent) = target.parent() {
        std::fs::File::open(parent)?.sync_all()?;
    }
    Ok(())
}
```

### 自动存档策略

```rust
enum AutoSaveTrigger {
    EventDriven(Vec<String>),   // level_loaded, checkpoint_reached
    Timed { interval_ms: u64 }, // 定期安全网（15-30 秒）
    Lifecycle,                   // on_pause, on_quit
}
```

### Undo/Redo 系统

```rust
struct UndoRedoSystem {
    undo_stack: Vec<Memento>,
    redo_stack: Vec<Memento>,
    max_history: usize,
}

impl UndoRedoSystem {
    fn record(&mut self, state: &GameState) {
        self.undo_stack.push(Memento {
            snapshot: state.clone(),
            timestamp: now(),
        });
        self.redo_stack.clear();
        if self.undo_stack.len() > self.max_history {
            self.undo_stack.remove(0);
        }
    }

    fn undo(&mut self, current: &GameState) -> Option<GameState> {
        let memento = self.undo_stack.pop()?;
        self.redo_stack.push(Memento {
            snapshot: current.clone(),
            timestamp: now(),
        });
        Some(memento.snapshot)
    }
}
```

---

## 6. Cozy Game 设计模式

### UI 设计原则

- **极简边框**: UI 融入世界，而非附加
- **纸质/日记美学**: 菜单设计为游戏内物品（Stardew 的日志、AC 的 Nook 手机）
- **无败局反映**: 无红色警告、无紧迫指示器
- **柔和色板**: 土色调、温暖柔和色、像素艺术柔和对比
- **一致空间语言**: 背包始终在同一位置，可预测的标签导航

### 关键模式

- **叙事 UI 元素**: 游戏内物品兼作 UI（Stardew 的邮箱 = 通知）
- **渐进式披露**: 系统逐步揭示，不一次性倾倒
- **自动排序/自动装备最佳**: 好的默认值消除琐事
- **背包作为约束**: 有限槽位迫使有意义的选择

---

## 7. 推荐技术栈（NT-GAME）

```
┌─────────────────────────────────────────┐
│  渲染层: macroquad（即时模式）           │
│  UI 层: egui-macroquad（调试）          │
│         + 自建 ScreenStack（玩家 UI）    │
├─────────────────────────────────────────┤
│  ECS: 自建轻量 ECS（~300 行核心）       │
│  或 hecs（最小）                         │
├─────────────────────────────────────────┤
│  数据: RON（游戏数据）+ Bincode（存档）  │
│  资产: assets_manager + notify 热重载    │
├─────────────────────────────────────────┤
│  已有: nt_game RPG/World/AI/Consciousness│
└─────────────────────────────────────────┘
```

### 关键 Crate 依赖

```toml
[dependencies]
# 渲染
macroquad = "0.4"
# UI（调试）
egui = "0.30"
egui-macroquad = "0.18"
# 序列化
serde = { version = "1", features = ["derive"] }
ron = "0.8"
bincode = "2"
# 资产
assets-manager = { version = "0.7", features = ["ron"] }
notify = "7"
# 音频
quad-snd = "0.1"
# 碰撞
parry2d = "0.17"
# ID
ulid = "1"
# 哈希
xxhash-rust = { version = "0.8", features = ["xxh3"] }
```

---

## 8. 实现优先级（基于调研）

| 优先级 | 模块 | 技术选型 | 理由 |
|--------|------|---------|------|
| **P0** | 自建 ECS | ~300 行核心 | 已有 nt_game ECS 需增强，不引入新依赖 |
| **P0** | macroquad 渲染 | 即时模式 | 最快原型、WASM 支持好 |
| **P0** | RON 数据加载 | assets-manager | 类型安全、热重载、社区标准 |
| **P1** | ScreenStack UI | 自建 | 已有对话/菜单需要状态管理 |
| **P1** | Bincode 存档 | bincode 2 | 最快、最紧凑 |
| **P1** | notify 热重载 | notify crate | 开发体验关键 |
| **P2** | egui 调试 UI | egui-macroquad | 开发工具 |
| **P2** | parry2d 碰撞 | parry2d | 物理碰撞 |
| **P3** | Lua 脚本 | mlua | 模组系统 |
