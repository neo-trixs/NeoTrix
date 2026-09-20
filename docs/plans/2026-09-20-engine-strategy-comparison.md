# NT-GAME 引擎战略 — Godot / Unity / UE5 / 自建对比与决策

**日期**: 2026-09-20
**目标**: 确定 NeoTrix 游戏引擎的构建策略

---

## 一、三大商业引擎对比

### 架构对比

| 维度 | Godot 4 | Unity 6 | Unreal Engine 5 |
|------|---------|---------|-----------------|
| **架构** | 场景树 + Server 抽象 | GameObject + ECS/DOTS 渐进 | UObject + Actor-Component |
| **渲染** | Forward+/Mobile/Compatibility | URP/HDRP + Render Graph | Nanite + Lumen（光追） |
| **2D 能力** | 原生 2D（目的构建） | 原生 2D Renderer | Paper2D（次要） |
| **脚本** | GDScript / C# / GDExtension | C# 14 / .NET 10 | C++ / Blueprints |
| **编辑器** | 用自身引擎构建 | 独立进程 | 集成于引擎 |
| **ECS** | 无（场景树） | DOTS（渐进集成） | 无（Actor 模型） |
| **UI** | Control 节点（Flexbox） | UI Toolkit / uGUI | UMG（Widget） |

### 商业模型

| 维度 | Godot 4 | Unity 6 | Unreal Engine 5 |
|------|---------|---------|-----------------|
| **费用** | $0 永久 | Personal: 免费（<$200K）/ Pro: ~$2,310/年 | 免费（<$1M 收入） |
| **版税** | 无 | 无（Runtime Fee 已取消） | 5%（>$1M） |
| **源码** | 完全开放 MIT | 付费 Enterprise | 免费 GitHub |
| **信任风险** | 零（非营利基金会） | 中（多次定价变更） | 低（协议稳定） |

### 生态系统

| 维度 | Godot 4 | Unity 6 | Unreal Engine 5 |
|------|---------|---------|-----------------|
| **GitHub Stars** | 108K | N/A（私有） | N/A（私有） |
| **活跃开发者** | 2.5M+ | 7M+ | 未公开 |
| **资产商店** | ~3K 社区资产 | 70K-100K+ | Marketplace + Fab |
| **Game Jam 份额** | **47%**（#1） | 34%（#2） | <5% |
| **Steam 发布量** | 35K+ | 数十万 | 数万 |
| **招聘市场** | ~5K 职位 | ~150K 职位 | 显著但较少 |
| **Rust 集成** | gdext（成熟） | 无官方 | 无官方 |

### 2026 年趋势

```
Godot:  ↑↑↑ 快速增长（Game Jam #1、Slay the Spire 2 验证）
Unity:  → 稳定但信任受损（CoreCLR 迁移中）
Unreal: → 稳定（Nanite/Lumen 领先但门槛高）
```

---

## 二、引擎选择决策矩阵

### 选 Godot 4 当

- 独立开发者 / 小团队（1-3 人）
- 2D 或风格化 3D 游戏
- 零成本、零授权风险
- 快速迭代（164MB 安装、即时打开）
- 原型制作 / 学习
- 偏好开源工具

### 选 Unity 当

- 移动端优先（48-70% 市场份额）
- 需要最大资产市场
- 团队偏好 C#
- 需要广泛平台覆盖（含 WebGL）
- 职业流动性重要

### 选 Unreal 当

- 追求照片级真实 3D（PC/主机）
- 开放世界 / 环境密集型游戏
- 需要内置高级系统（GAS、MetaSounds）
- 团队熟悉 C++ / Blueprints
- 需要完整源码访问

### 自建引擎当

- 独特机制、长期项目
- 性能关键型模拟
- 需要完全控制
- 多游戏工作室（引擎即产品）
- 学习目的

---

## 三、NT-GAME 的正确策略

### 核心洞察

> **NeoTrix 不需要一个通用引擎。它需要一个"游戏能力平台"——用 Rust 构建游戏逻辑层，用成熟工具处理渲染/编辑器。**

### 推荐策略：分层混合架构

```
┌─────────────────────────────────────────────────┐
│  Layer 4: NT-GAME 业务层（纯 Rust）              │
│  战斗/种植/对话/制作/任务/叙事/意识训练          │
│  ── 完全自控，无引擎依赖 ──                      │
├─────────────────────────────────────────────────┤
│  Layer 3: NT-GAME 框架层（Rust）                 │
│  ECS / 事件系统 / 数据管线 / 存档               │
│  ── 自建，可独立测试 ──                          │
├─────────────────────────────────────────────────┤
│  Layer 2: 渲染/输入/音频（Rust）                 │
│  macroquad（即时模式）                           │
│  ── 最快原型，WASM 支持 ──                       │
├─────────────────────────────────────────────────┤
│  Layer 1: 平台层（可选）                         │
│  Godot 编辑器 / Tauri 桌面 / Web 浏览器         │
│  ── 复用成熟工具 ──                              │
└─────────────────────────────────────────────────┘
```

### 为什么不用 Godot/Unity/UE5 做核心

| 原因 | 说明 |
|------|------|
| **Rust 是 NeoTrix 的语言** | 已有 200+ 模块是 Rust，引入 C#/C++ 引擎会割裂生态 |
| **意识系统是核心差异化** | GWT/IIT Phi/VSA 是 NeoTrix 独有，无法用现有引擎的脚本系统实现 |
| **训练循环是核心** | 自我对弈/GRPO/进化循环需要深度集成，引擎脚本不够 |
| **跨平台需要 Rust** | WASM/桌面/移动全用 Rust 统一 |
| **避免引擎锁定** | 不依赖任何引擎的 API/许可/版本 |

### 为什么用 macroquad 做渲染

| 原因 | 说明 |
|------|------|
| **最快时间到可玩** | ~16s 编译，即时模式 API 极简 |
| **WASM 一等公民** | 200KB-2MB 二进制，单命令部署 |
| **无全局状态依赖** | 可以按需启用/禁用，与现有代码无冲突 |
| **已有成功案例** | 多个商业游戏用 macroquad |
| **可替换** | 如果需要更复杂渲染，可换 wgpu/ggez |

---

## 四、与 Godot 的混合方案（可选）

### 方案 A：Godot 做编辑器，Rust 做运行时

```
Godot 编辑器（场景编辑/关卡设计/UI 布局）
    │
    ▼ 导出 .tscn / .tres 数据
    │
Rust 运行时（macroquad 渲染 + NT-GAME 逻辑）
    │
    ▼ 加载数据，运行游戏
```

**优点**: 用 Godot 编辑器做关卡/对话/UI 设计，Rust 做性能关键逻辑
**缺点**: 需要写数据转换器，两套工具链
**适合**: 如果需要复杂关卡编辑器

### 方案 B：Godot 做可视化层，Rust 做核心层

```
Rust 核心（ECS + 游戏逻辑 + 意识系统）
    │
    ▼ MCP/IPC 通信
    │
Godot 可视化（渲染 + UI + 编辑器）
```

**优点**: Godot 负责所有可视化，Rust 负责所有逻辑
**缺点**: 两进程通信延迟，调试复杂
**适合**: 如果需要 Godot 编辑器但逻辑必须在 Rust

### 方案 C：纯 Rust（推荐）

```
macroquad 渲染 ←→ NT-GAME ECS ←→ RON 数据
    │                  │
    ▼                  ▼
egui 调试 UI      意识系统/训练
```

**优点**: 单一语言、单一工具链、完全控制、最快迭代
**缺点**: 需要自建编辑器（或用数据文件代替）
**适合**: NeoTrix 的现状——已有大量 Rust 代码

---

## 五、引擎构建路线图

### Phase 1: 最小可渲染（Week 1-2）

```
目标: cargo run → 弹出窗口 → 渲染精灵 → 角色移动
技术: macroquad + 自建 ECS + RON 数据
产出: nt_game_core + nt_game_render + nt_game_data
验证: cargo check 通过 + 窗口可交互
```

### Phase 2: 游戏世界（Week 3-4）

```
目标: 瓦片地图 + NPC 对话 + 天气/季节
技术: 已有 world/ 子系统 + 渲染层接入
产出: 接入 combat/npc/quest/tilemap/weather/season
验证: 能在窗口中看到并交互
```

### Phase 3: UI + 存档（Week 5-6）

```
目标: 菜单 + HUD + 背包 + 存档/读档
技术: ScreenStack + egui + bincode
产出: 完整 UI 系统 + 存档系统
验证: 能保存/加载游戏进度
```

### Phase 4: 打磨 + 发布（Week 7-8）

```
目标: 粒子 + 音效 + 本地化 + 打包
技术: macroquad 音效 + notify 热重载
产出: 可发布的桌面/WASM 游戏
验证: cargo build --release + 平台测试
```

### 可选 Phase 5: Godot 编辑器集成

```
目标: 用 Godot 编辑器设计关卡
技术: Godot 导出 .tscn → Rust 解析器加载
产出: 关卡编辑器 + 对话编辑器
验证: Godot 编辑的关卡能在 Rust 运行时中渲染
```

---

## 六、关键决策点

| 决策 | 选择 | 理由 |
|------|------|------|
| **渲染框架** | macroquad | 最快原型、WASM、无全局状态 |
| **ECS** | 自建（~300 行） | 控制权、无依赖、已有基础 |
| **数据格式** | RON | Rust 原生、注释支持、社区标准 |
| **存档格式** | Bincode 2 | 最快、最紧凑 |
| **UI** | ScreenStack + egui | 玩家 UI + 调试工具 |
| **编辑器** | 暂不构建 | 先用数据文件，需要时加 Godot |
| **音频** | quad-snd | macroquad 原生 |
| **物理** | parry2d | 2D 碰撞检测 |
| **脚本** | 暂不需要 | 纯 Rust 足够，需要时加 mlua |

---

## 七、与 NeoTrix 已有能力的映射

| NeoTrix 已有能力 | 引擎层映射 | 改动量 |
|-----------------|-----------|--------|
| `nt_game/ecs/` | Layer 3 ECS | 增强（添加 Position/Sprite 组件） |
| `nt_game/rpg/` | Layer 4 业务 | 保留（角色/装备/技能树） |
| `nt_game/world/` | Layer 4 业务 | 保留（14 个子系统） |
| `nt_game/ai/` | Layer 4 业务 | 保留（行为树/策略） |
| `nt_game/consciousness/` | Layer 4 业务 | 保留（意识-游戏桥） |
| `nt_game/evolution.rs` | Layer 4 业务 | 保留（自主训练） |
| `nt_game/mcp.rs` | Layer 3 接口 | 保留（MCP 工具） |
| `nt_game/framework.rs` | Layer 3 框架 | 保留（Actor/Episode/Rubric） |
| 无 | Layer 2 渲染 | **新增** macroquad |
| 无 | Layer 3 UI | **新增** ScreenStack |
| 无 | Layer 3 存档 | **新增** bincode |
| 无 | Layer 3 数据管线 | **新增** RON + assets_manager |

---

## 八、结论

### 不做什么

| 不做 | 理由 |
|------|------|
| **不引入 Godot/Unity/UE5** | 割裂 Rust 生态，无法集成意识系统 |
| **不自建通用引擎** | 200+ 模块已够，不需要再建渲染器/物理引擎 |
| **不建编辑器** | 先用数据文件，需要时加 Godot 桥接 |
| **不建脚本语言** | Rust 足够，需要时加 Lua（mlua） |

### 做什么

| 做 | 方法 |
|---|------|
| **用 macroquad 做渲染** | 最快时间到可玩 |
| **自建轻量 ECS** | ~300 行，完全控制 |
| **用 RON 做数据** | 类型安全、热重载 |
| **用 bincode 做存档** | 最快、最紧凑 |
| **自建 ScreenStack** | 游戏 UI 状态管理 |
| **用 egui 做调试** | 开发工具 |
| **保留所有已有模块** | RPG/World/AI/Consciousness |
| **保持纯 Rust** | 统一工具链、零割裂 |

### 一句话总结

> **用 macroquad 渲染 + 自建 ECS + 已有 NT-GAME 逻辑 = 最小成本、最大控制、最快可玩。**
