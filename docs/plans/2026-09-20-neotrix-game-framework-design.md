# NeoTrix Game Framework — 世界观吸收报告

**日期**: 2026-09-20
**吸收源**: corepunch/open-realm, bobeff/open-source-games, GitHub cozy-game topic, GitHub indie-game topic, ptrlrd/spire-codex
**吸收方法**: worldbuilding-absorption 五阶段模板
**一句话**: NeoTrix 游戏框架 = 数据驱动引擎 + 安全默认 + 模块化游戏逻辑，从 Rust 生态出发构建一个"写游戏而非写引擎"的开发体验。

---

## Phase 1: 资料收集（考据驱动，证据优先）

### 资料清单表

| Source | Pattern (机制) | 证据 (URL) |
|--------|---------------|-----------|
| corepunch/open-realm | Quake 风格 import/export 函数表、表驱动解析、服务端授权 UI、数据导向设计、严格引擎/游戏边界 | https://github.com/corepunch/open-realm |
| bobeff/open-source-games | 150+ 开源游戏目录、再实现/逆工程/源端口三大模式、SDL/OpenGL 跨平台标配、数据+引擎分离 | https://github.com/bobeff/open-source-games |
| GitHub cozy-game | 安全/丰裕/柔软三支柱、宽恕架构、无败局设计、桌面玩具、AI NPC 伴侣、Web 原生分发 | https://github.com/topics/cozy-game |
| GitHub indie-game | Rust/TypeScript 最快增长、Roguelite 融合主导、WebAssembly 浏览器分发、AI 辅助开发工具链爆发 | https://github.com/topics/indie-game |
| ptrlrd/spire-codex | 22 个正则解析器管线、FastAPI + Next.js、SmartFormat 模板解析、写入即刻保留变更日志、双数据通道 | https://github.com/ptrlrd/spire-codex |

---

## Phase 2: 世界观公理（先立公理，再推演）

### 三条底层公理

| # | 公理 | 内容 | 对应 NeoTrix 公理 | 违反后果 |
|---|------|------|-------------------|---------|
| **G1** | **数据即游戏** | 所有游戏逻辑、状态机、数值系统必须可从数据文件（TOML/JSON/RON）完全描述。代码是数据的执行器，不是数据的拥有者。 | **R-P1** `#![forbid(unsafe_code)]` — 数据与代码的边界不可跨越；**Dark Forest** — 无数据消费者的代码模块删除 | 违反 G1：游戏逻辑散落在代码各处，无法热更新、无法模组化、无法自动测试。等于网文"设定不服务剧情"。 |
| **G2** | **安全是默认态** | 框架默认输出的游戏必须是"安全的"——无永久惩罚、无不可逆状态、无隐性资源耗尽。惩罚必须是显式 opt-in。 | **指针守恒** — 资源（上下文/注意力）是稀缺的，框架必须保护而非消耗它 | 违反 G2：玩家感到焦虑而非放松，游戏偏离"cozy"核心，等于"世界观违背铁律"。 |
| **G3** | **模块即能力** | 每个游戏能力（战斗、种植、对话、制作）是自包含模块，通过声明式契约与框架交互，不直接依赖其他模块。 | **R-P42** — 禁止平行适配器模块；**R-P79** — 同 session 接线到生产 | 违反 G3：模块间隐式耦合，删除一个功能导致整个游戏崩溃。等于"金手指无戏份"。 |

### 公理推演验证

```
G1 (数据即游戏) ──推演──→ 所有游戏状态可序列化/反序列化
    │                          │
    ├──推演──→ 热重载 = 重新加载数据文件，无需重启
    ├──推演──→ 模组系统 = 第三方数据文件即模组
    └──推演──→ 自动测试 = 数据断言 + 快照比对

G2 (安全是默认态) ──推演──→ 框架提供"宽恕架构"原语
    │                          │
    ├──推演──→ 自动存档 = 任何时刻可回滚
    ├──推演──→ 无败局状态机 = 无 Game Over 分支（除非显式开启）
    └──推演──→ 资源保护 = 无隐性资源耗尽（体力/金币/时间）

G3 (模块即能力) ──推演──→ 能力 = 独立 crate + 声明式依赖
    │                          │
    ├──推演──→ 运行时组合 = 游戏 = 数据 + 能力选择，非代码拼接
    ├──推演──→ A/B 测试 = 替换同接口模块，无需修改其他代码
    └──推演──→ 社区生态 = 第三方能力 crate = npm for games
```

---

## Phase 3: 力量体系（能力阶梯 + 代价机制）

### 框架成熟度阶梯（对标 NeoTrix C0-C6）

| 阶段 | 名称 | 晋级条件 | 代价 | 能力表现 |
|------|------|---------|------|---------|
| **G0** | **脚手架** | `cargo new` 成功 | 无 | 空白项目、最小渲染窗口 |
| **G1** | **数据世界** | 能从 TOML/JSON 加载并渲染一个场景 | 学习数据格式规范 | 场景加载、精灵渲染、基础输入 |
| **G2** | **能力模块** | 至少一个游戏能力模块可独立运行 | 模块接口设计成本 | 战斗/种植/对话等能力即插即用 |
| **G3** | **状态机** | 游戏状态可序列化/反序列化 + 自动存档 | 状态设计复杂度 | 任意时刻回滚、热重载、模组加载 |
| **G4** | **网络化** | 多人游戏状态同步 | 网络延迟处理、冲突解决 | 客户端-服务端架构、快照同步 |
| **G5** | **生态化** | 模组市场 + 社区能力 crate | 模组安全沙箱、版本兼容 | 第三方模组即插即用、自动更新 |
| **G6** | **AI 原生** | NPC 自主行为 + 玩家行为学习 | AI 模型集成成本、隐私合规 | 程序化 NPC、自适应难度、动态叙事 |

### 核心能力（金手指）

| 能力名 | 作用 | 消费者 (R-P79) |
|--------|------|----------------|
| **数据管线** | 从数据文件到运行时状态的完整链路 | 所有游戏模块 |
| **宽恕引擎** | 自动存档 + 无败局 + 回滚 | 所有游戏逻辑 |
| **能力组合器** | 声明式模块依赖 + 运行时注入 | 模组系统、A/B 测试 |

---

## Phase 4: 一致性维护（设定库 + 检查）

### 架构总览

```
┌─────────────────────────────────────────────────────┐
│                  nt_game (顶层 crate)                │
│  ┌───────────┐  ┌───────────┐  ┌───────────┐       │
│  │ 游戏数据  │  │ 能力模块  │  │ 运行时    │       │
│  │ (TOML/RON)│  │ (crate)   │  │ (ECS)     │       │
│  └─────┬─────┘  └─────┬─────┘  └─────┬─────┘       │
│        │              │              │               │
│        ▼              ▼              ▼               │
│  ┌─────────────────────────────────────────────┐     │
│  │            nt_game_core (核心引擎)           │     │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐       │     │
│  │  │ 数据加载 │ │ 状态管理 │ │ 事件总线 │       │     │
│  │  └─────────┘ └─────────┘ └─────────┘       │     │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐       │     │
│  │  │ 宽恕引擎 │ │ 模组加载 │ │ AI 集成  │       │     │
│  │  └─────────┘ └─────────┘ └─────────┘       │     │
│  └─────────────────────────────────────────────┘     │
│        │                                             │
│        ▼                                             │
│  ┌─────────────────────────────────────────────┐     │
│  │         nt_game_render (渲染层)              │     │
│  │  (wgpu / macroquad / headless)              │     │
│  └─────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────┘
```

### 模块清单（从公理推演）

#### 1. `nt_game_core` — 核心引擎

**职责**: 游戏循环、状态管理、事件系统、数据加载

```rust
// 公理 G1 的直接体现：所有状态可序列化
pub trait GameState: Serialize + DeserializeOwned {
    fn tick(&mut self, ctx: &GameContext) -> Vec<GameEvent>;
    fn snapshot(&self) -> GameStateSnapshot;
    fn restore(&mut self, snapshot: &GameStateSnapshot) -> Result<()>;
}

// 公理 G2 的直接体现：宽恕架构
pub struct ForgivenessEngine {
    auto_save_interval: Duration,
    max_snapshots: usize,        // 默认 1000
    undo_stack: VecDeque<Snapshot>,
    resource_guard: ResourceGuard, // 防止隐性资源耗尽
}

// 公理 G3 的直接体现：能力即模块
pub trait GameAbility: 'static + Send + Sync {
    fn name(&self) -> &str;
    fn dependencies(&self) -> Vec<AbilityId>; // 声明式依赖
    fn init(&mut self, ctx: &mut GameContext) -> Result<()>;
    fn on_event(&mut self, event: &GameEvent, ctx: &mut GameContext) -> Vec<GameEvent>;
    fn state(&self) -> Box<dyn erased_serde::Serialize>;
}
```

#### 2. `nt_game_data` — 数据管线

**职责**: TOML/RON/JSON → 运行时数据，类型安全反序列化

```rust
// 从 spire-codex 吸收的管线模式
pub struct DataPipeline {
    parsers: HashMap<DataType, Box<dyn DataParser>>,
    cache: LruCache<PathBuf, DynamicCompound>,
    hot_reload: HotReloadWatcher,
}

// 表驱动解析（从 open-realm 吸收）
#[derive(Debug, Deserialize)]
pub struct SchemaEntry {
    pub field: &'static str,
    pub r#type: FieldType,
    pub default: Option<Value>,
    pub validate: Option<fn(&Value) -> bool>,
}

pub fn parse_from_schema<T: DeserializeOwned>(
    data: &Value,
    schema: &[SchemaEntry],
) -> Result<T> { ... }
```

#### 3. `nt_game_render` — 渲染抽象层

**职责**: 跨平台渲染，支持 wgpu/macroquad/headless

```rust
// 从 open-realm 吸收的渲染层分离
pub trait Renderer: Send + Sync {
    fn init(&mut self, config: &RenderConfig) -> Result<()>;
    fn begin_frame(&mut self);
    fn draw_sprite(&mut self, sprite: &Sprite, transform: &Transform);
    fn draw_ui(&mut self, layout: &UILayout); // 服务端授权 UI
    fn end_frame(&mut self);
}

// 多后端支持（从 open-realm 的多 GL 后端吸收）
pub enum RenderBackend {
    Wgpu,           // 跨平台现代 GPU
    Macroquad,      // 轻量 2D
    Headless,       // 服务端/测试
    WebCanvas,      // 浏览器 (Wasm)
}
```

#### 4. `nt_game_ability_*` — 能力模块（每个独立 crate）

从 bobeff/open-source-games 的 150+ 游戏中提炼的高频能力：

| 能力 crate | 功能 | 来源模式 |
|-----------|------|---------|
| `nt_game_ability_combat` | 回合制/即时战斗 | Spire Codex 状态机、open-realm 技能模块 |
| `nt_game_ability_dialogue` | 对话树 + 分支 | RPG 对话系统 |
| `nt_game_ability_crafting` | 制作/合成 | Cozy game 核心循环 |
| `nt_game_ability_farming` | 种植/收获/季节 | Cozy game 第一机制 |
| `nt_game_ability_inventory` | 背包/资源管理 | 通用游戏基础设施 |
| `nt_game_ability_quest` | 任务/事件/成就 | Spire Codex 事件决策树 |
| `nt_game_ability_narrative` | 动态叙事/程序化故事 | AI NPC + 分支叙事 |

#### 5. `nt_game_mod` — 模组系统

```rust
// 从 spire-codex 的双数据通道吸收
pub struct ModLoader {
    stable_path: PathBuf,
    beta_path: PathBuf,
    active_mods: Vec<Mod>,
   沙箱: Sandbox, // 安全隔离
}

// 模组 = 数据文件 + 可选能力扩展
pub struct Mod {
    manifest: ModManifest,      // TOML 描述
    data_files: Vec<DataFile>,  // 游戏数据
    abilities: Vec<AbilityId>,  // 可选：新能力
    dependencies: Vec<ModRef>,  // 依赖其他模组
}
```

#### 6. `nt_game_ai` — AI 集成

从 cozy-game topic 的 AI NPC 趋势吸收：

```rust
// NPC 自主行为
pub trait NpcBehavior: 'static + Send + Sync {
    fn think(&mut self, world: &WorldState) -> NpcAction;
    fn remember(&mut self, event: &GameEvent);     // ACT-R 记忆模型
    fn feel(&mut self, situation: &Situation) -> Emotion; // PAD/OCC 情感模型
    fn personality(&self) -> &Personality;          // Big Five 人格
}

// 自适应难度
pub struct AdaptiveDifficulty {
    player_skill_estimate: f32,
    frustration_threshold: f32,
    engagement_target: f32,
}
```

---

## Phase 5: 工业化生产（生产计划 + 接线点）

### crate 结构（Rust workspace）

```
neotrix/
├── crates/
│   ├── nt_game/                    # 顶层 crate，重新导出所有子 crate
│   ├── nt_game_core/               # 核心引擎：游戏循环、状态管理、事件系统
│   ├── nt_game_data/               # 数据管线：TOML/RON 加载、热重载
│   ├── nt_game_render/             # 渲染抽象：wgpu/macroquad/headless
│   ├── nt_game_forgiveness/        # 宽恕引擎：自动存档、回滚、资源保护
│   ├── nt_game_mod/                # 模组系统：加载、沙箱、版本兼容
│   ├── nt_game_ai/                 # AI 集成：NPC 行为、自适应难度
│   ├── nt_game_ability_combat/     # 战斗能力模块
│   ├── nt_game_ability_dialogue/   # 对话能力模块
│   ├── nt_game_ability_crafting/   # 制作能力模块
│   ├── nt_game_ability_farming/    # 种植能力模块
│   ├── nt_game_ability_inventory/  # 背包能力模块
│   ├── nt_game_ability_quest/      # 任务能力模块
│   └── nt_game_ability_narrative/  # 叙事能力模块
├── examples/
│   ├── cozy_farm/                  # 示例：cozy 种植游戏
│   ├── roguelite_deck/             # 示例：roguelite 卡牌游戏
│   └── idle_pet/                   # 示例：桌面宠物
└── docs/
    └── game-framework/             # 框架文档
```

### 示例：最小 Cozy 游戏

```rust
// examples/cozy_farm/src/main.rs
use nt_game::prelude::*;

fn main() {
    Game::builder()
        // G1: 数据即游戏 — 所有内容从数据加载
        .data_path("assets/")
        // G2: 安全是默认态 — 自动启用宽恕架构
        .forgiveness(ForgivenessConfig::default())
        // G3: 模块即能力 — 声明式选择能力
        .ability(NtFarmingAbility::new())
        .ability(NtInventoryAbility::new())
        .ability(NtDialogueAbility::new())
        // 运行
        .run();
}
```

### 示例：数据文件

```toml
# assets/game.toml
[game]
title = "My Cozy Farm"
default_season = "spring"

[[crops]]
name = "turnip"
grow_time_days = 4
sell_price = 60
buy_price = 20
seasons = ["spring"]

[[crops]]
name = "tomato"
grow_time_days = 7
sell_price = 80
buy_price = 30
seasons = ["summer"]

[npc.maru]
likes = ["mining", "engineering"]
birthday = "summer_10"
dialogue = "data/maru_dialogue.toml"
```

### 与 NeoTrix 现有模块的接线点

| NeoTrix 模块 | 接线方式 | 用途 |
|-------------|---------|------|
| `neotrix-types` | 依赖基础类型 | 共享错误类型、ID 系统 |
| `neotrix-consciousness` | 可选集成 | AI NPC 的"意识"层 |
| `neotrix-reasoning` | 可选集成 | NPC 决策推理 |
| `nt-lang` | 可选集成 | 游戏内脚本语言 |
| `neotrix-gateway` | 可选集成 | 多人游戏网络层 |

### 实施优先级

| 优先级 | 阶段 | crate | 工作量估算 |
|--------|------|-------|-----------|
| P0 | G0→G1 | `nt_game_core` + `nt_game_data` + `nt_game_render` | 2-3 周 |
| P1 | G1→G2 | `nt_game_forgiveness` + `nt_game_ability_inventory` | 1-2 周 |
| P2 | G2→G3 | `nt_game_ability_combat` + `nt_game_ability_dialogue` | 2-3 周 |
| P3 | G3→G4 | `nt_game_mod` + `nt_game_ability_farming` | 2-3 周 |
| P4 | G4→G5 | `nt_game_ai` + `nt_game_ability_narrative` | 3-4 周 |
| P5 | G5→G6 | 社区生态 + 模组市场 | 持续 |

---

## 附录：吸收源模式映射

### 从 open-realm 吸收

| 模式 | NeoTrix 映射 |
|------|-------------|
| Quake import/export 函数表 | `GameAbility` trait — 模块通过 trait 与核心交互 |
| 表驱动解析 (DDX schema) | `DataPipeline` + `SchemaEntry` — 数据格式由 schema 定义 |
| 服务端授权 UI | 渲染层接收声明式 UI 布局，不包含游戏逻辑 |
| 严格引擎/游戏边界 | `nt_game_core` 零游戏知识，游戏逻辑全在能力 crate |
| 数据导向设计 | 扁平结构、零中间分配、`Serialize + Deserialize` |

### 从 open-source-games 吸收

| 模式 | NeoTrix 映射 |
|------|-------------|
| 数据+引擎分离 | `nt_game_data` (数据) + `nt_game_core` (引擎) 完全解耦 |
| 跨平台标配 SDL/wgpu | `RenderBackend` 多后端抽象 |
| 模块化插件系统 | 能力 crate = 插件，运行时组合 |
| 社区驱动的复刻生态 | 模组系统 = 社区生态的基础设施 |

### 从 cozy-game 吸收

| 模式 | NeoTrix 映射 |
|------|-------------|
| 安全/丰裕/柔软三支柱 | G2 公理 + `ForgivenessEngine` |
| 宽恕架构 | 自动存档、无败局、回滚 |
| 无败局设计 | `no_game_over: true` 为默认值 |
| 桌面玩具/环境存在 | Headless 渲染 + 系统托盘模式 |
| AI NPC 伴侣 | `nt_game_ai` + `NpcBehavior` trait |
| Web 原生分发 | Wasm 渲染后端 |
| 实时时间集成 | `RealTimeClock` 组件 |

### 从 indie-game 吸收

| 模式 | NeoTrix 映射 |
|------|-------------|
| Rust 最快增长 | NeoTrix 本身就是 Rust，天然优势 |
| Roguelite 融合 | 能力组合器支持任意能力混搭 |
| WebAssembly 分发 | Wasm 渲染后端 + `web-sys` |
| AI 辅助开发 | `nt_game_ai` 同时服务 NPC 和开发者工具 |

### 从 spire-codex 吸收

| 模式 | NeoTrix 映射 |
|------|-------------|
| 22 个解析器管线 | `DataPipeline` + 可插拔 `DataParser` |
| FastAPI + Next.js | 游戏前后端分离架构 |
| SmartFormat 模板 | 数据文件中的本地化模板系统 |
| 写入即刻保留变更日志 | 模组版本管理 + 变更审计 |
| 双数据通道 (stable/beta) | `ModLoader` 的 stable/beta 路径 |

---

## 质量门控

- [x] 每个结论都有 evidence（URL），零无证据断言
- [x] 三条公理覆盖"世界运转 / 核心稀缺 / 金手指"
- [x] 每条公理映射到 NeoTrix 架构公理
- [x] 每条公理有"违反后果"
- [x] 境界阶梯完整（G0-G6），每阶段有晋级条件+代价+能力
- [x] 核心能力有明确消费者（R-P79 接线门）
- [x] 吸收成果已接线到 crate 结构和实施计划
- [x] 无消费者模式已排除（Dark Forest）
