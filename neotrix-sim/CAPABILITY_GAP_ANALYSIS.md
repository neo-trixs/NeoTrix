# NT-WORLD-SIM 跨域能力缺口分析

> Generated: 2026-09-11 | 基于 9 类 40+ 项目吸收 + 能力骨架对齐
> 状态: 🔴 未开始 | 🟡 进行中 | 🟢 完成

---

## 一、已吸收游戏模式分类 (9类)

| 类别 | 代表项目 | 核心模式 | 吸收数 |
|------|----------|----------|:------:|
| 动态漫/短剧 | Mini-MAXS, OpenLife | 事件驱动、状态机、季节系统、TTS/字幕 | 12 |
| Canvas 引擎 | origami.js, d-zone, easycanvas, openscope | 精灵系统、tilemap、ECS、场景管理 | 8 |
| 放置/挂机 | ogame-vue, break_infinity, trial-tower | 资源膨胀、离线收益、自动化 | 6 |
| Roguelike | wasmbots, WFC, neo-angband | 程序生成、战斗系统、永久死亡 | 7 |
| 模拟经营 | paradise-isle, Stardew (taoyuan), Gravity-Sandbox | 日夜循环、NPC 行为、经济系统 | 9 |
| 游戏引擎 | boardgame.io, planck.js, bitECS, Excalibur | 游戏循环、物理引擎、ECS、状态同步 | 10 |
| MOBA | microduck_rl, OpenAI Five, TorchRL | RL训练、多智能体、奖励塑形 | 6 |
| 平台生成 | evolve-search, autoresearch | 进化搜索、自适应变异 | 4 |
| 内容生成 | gif-sticker-maker, ppt-orchestra | 模板系统、质量门禁、资产管线 | 5 |

---

## 二、聚焦冗余 (Focal Redundancy)

> 多个模式收敛到同一能力点，应统一为单一实现

### R1: 事件调度冗余

| 源模式 | 位置 | 收敛点 |
|--------|------|--------|
| Mini-MAXS EventMode | `EventMode::Time/Timer/Start/StateChange` | event_reactive.rs |
| Evolution EventMode | `EventMode::Time/Timer/Start/StateChange` | event_reactive.rs |
| OpenLife Custom Events | `Event::ConversationEvent/ToolCallEvent` | event_reactive.rs |
| trailblaze Event Bus | `EventBus` + subscribers | event_reactive.rs |

**结论**: 4 个来源 → `event_reactive.rs` 已存在但仅接了 WorldSim tick。
**修复**: 将 EventMode (Time/Timer/Start/StateChange) 加入 `SimulationEvent` 枚举，统一 dispatch。

### R2: AI 决策层冗余

| 源模式 | 位置 | 收敛点 |
|--------|------|--------|
| GOAP | `planning/goap.rs` | decision.rs |
| Behavior Tree | `behavior_tree/mod.rs` | decision.rs |
| Utility AI | `build_observation()` | decision.rs |
| Stimulus Response | `agents/stimulus.rs` (新建) | decision.rs |
| Emotional Bias | `agents/emotional_bias.rs` (新建) | decision.rs |

**结论**: 5 层决策堆栈，但仅 Utility 一条路径激活。
**修复**: 决策栈统一为 `DecisionEngine`：Reflex → Utility → GOAP → BT，每层输出 `Option<AgentAction>`，selector 取第一个 Some。

### R3: 社交系统冗余

| 源模式 | 位置 | 收敛点 |
|--------|------|--------|
| Gossip Protocol | `society/gossip.rs` | 信息传播 |
| Alliance Formation | `society/alliance.rs` (新建) | 关系聚合 |
| Norm Evolution | `society/norm_evolution.rs` (新建) | 文化规范 |
| Faction Territory | `society/faction.rs` | 派系归属 |
| Communication | `society/communication.rs` | 消息传递 |

**结论**: 5 个社交子系统，但 `SocialLearning` + `ConstitutionalFeedback` 未接通 gossip/alliance。
**修复**: `SocialEngine` 统一调用：Faction → Gossip → Alliance → Norms，输出 `SocialDynamicsReport`。

### R4: 记忆操作冗余

| 源模式 | 位置 | 收敛点 |
|--------|------|--------|
| Episodic Memory | `memory_stream.rs` | 事件记忆 |
| Working Memory | `MemoryStream` (capacity 200) | 短期缓存 |
| Semantic Memory | `graph_memory.rs` (新建) | 概念网络 |
| Consolidation | `memory_consolidation.rs` (新建) | 睡眠转移 |
| Retrieval | `memory_retrieval.rs` (新建) | 相似性检索 |

**结论**: 5 种记忆操作，但 MemoryStream 是唯一活跃路径。GraphMemory 未做概念检索。
**修复**: `MemoryManager` 统一：Write → Stream/Graph 双写；Read → Cue-based retrieval; Consolidation → 定期 Stream→Graph 迁移。

### R5: 持久化冗余

| 源模式 | 位置 | 收敛点 |
|--------|------|--------|
| WorldSnapshot | `world_sim/mod.rs` | 序列化 |
| GameState | trailblaze (30+ 字段) | 完整状态 |
| Auto-save | Evolution (interval) | 定时保存 |
| Backup | Mini-MAXS (backup_dir) | 备份机制 |

**结论**: WorldSnapshot 已存在但无反序列化。GameState 30+ 字段可参考。
**修复**: `persistence.rs` 实现 WorldSnapshot + GameState 双格式（JSON + Bincode），支持 save/load/backup。

---

## 三、扁平缺陷 (Flat Deficiency)

> 所有游戏类型均缺失的基础能力

### D1: Canvas 2D 渲染循环

| 缺失 | 影响 | 来源启发 |
|------|------|----------|
| Game Loop (requestAnimationFrame) | 无视觉输出 | d-zone (60fps), easycanvas (RAF) |
| Sprite 系统 | 无角色/建筑渲染 | openscope (SpriteSheet), Excalibur (Sprite) |
| Camera 控制 | 无法平移/缩放 | Stardew (viewport), boardgame.io (clientState) |
| 粒子系统 | 无视觉反馈 | Mini-MAXS (particle effects) |

**LOC**: ~800 | **优先级**: P0

### D2: 持久化 (Save/Load)

| 缺失 | 影响 | 来源启发 |
|------|------|----------|
| 序列化到磁盘 | 无法保存进度 | trailblaze (save/load), ogame (localStorage) |
| 反序列化恢复 | 无法加载 | Evolution (save/load), Mini-MAXS (backup) |
| 自动存档 | 丢失进度 | Evolution (auto-save), Stardew (sleep save) |

**LOC**: ~300 | **优先级**: P0

### D3: 时间系统

| 缺失 | 影响 | 来源启发 |
|------|------|----------|
| Day/Night 循环 | 无昼夜变化 | Stardew (day/night), paradise-isle (时间流逝) |
| 季节系统 | 无季节影响 | Mini-MAXS (季节), Stardew (4季) |
| 天气系统 | 无环境事件 | Mini-MAXS (天气), OpenLife (天气事件) |
| 时间控制 | 无法暂停/加速 | Evolution (play/pause/speed), trailblaze (tick rate) |

**LOC**: ~400 | **优先级**: P1

### D4: 粒子/动画系统

| 缺失 | 影响 | 来源启发 |
|------|------|----------|
| 粒子发射器 | 无战斗/采集视觉效果 | Mini-MAXS (粒子), easycanvas (particle) |
| 关键帧动画 | 无角色动画 | easycanvas (timeline), Excalibur (animation) |
| 骨骼动画 | 无复杂动作 | planck.js (body animation) |

**LOC**: ~500 | **优先级**: P2

### D5: 程序化生成

| 缺失 | 影响 | 来源启发 |
|------|------|----------|
| 地图生成 (WFC/BSP) | 无丰富地形 | wasmbots (WFC), neo-angband (BSP) |
| 建筑/结构生成 | 无城镇 | Stardew (farm layout) |
| 任务/事件生成 | 无动态内容 | wasmbots (procedural) |

**LOC**: ~600 | **优先级**: P1

### D6: 音效系统

| 缺失 | 影响 | 来源启发 |
|------|------|----------|
| 音效播放 | 无听觉反馈 | Mini-MAXS (TTS), trailblaze (none) |
| 背景音乐 | 无氛围 | Stardew (music system) |
| 音效触发 | 无事件音效 | Mini-MAXS (sfx trigger) |

**LOC**: ~200 | **优先级**: P2

---

## 四、跨域错位 (Cross-Domain Misalignment)

> 模式放在了错误的 NT-* 域中

### M1: 信息素系统 → NT-SHIELD

| 当前位置 | 正确位置 | 原因 |
|----------|----------|------|
| `environment/pheromone.rs` | NT-SHIELD (协调) | 信息素是 stigmergic coordination，属于安全/协调层 |
| 6 种信息素类型 | NT-SHIELD pheromone 模块 | 与 stealth net、proxy pool 同层 |

**修复**: 将 `pheromone.rs` 迁移至 `nt_shield::pheromone`，WorldSim 通过 trait 调用。

### M2: 行为 VM → NT-CORE

| 当前位置 | 正确位置 | 原因 |
|----------|----------|------|
| `consciousness/behavior_vm.rs` | NT-CORE (核心推理) | 行为执行是核心认知，非意识监控 |
| 指令集 + 栈 | NT-CORE behavior_vm | 与 E8、GWT 同层 |

**修复**: 将 `behavior_vm.rs` 迁移至 `nt_core::behavior_vm`，consciousness 仅做监控。

### M3: 经济定价 → NT-ACT

| 当前位置 | 正确位置 | 原因 |
|----------|----------|------|
| `economy/pricing.rs` | NT-ACT (行动执行) | 定价是交易行动的子模块 |
| 供需曲线 | NT-ACT trade/pricing | 与 Tool、Social 同层 |

**修复**: 将 `pricing.rs` 合并至 `nt_act::trade`，economy 仅做统计。

### M4: 派系领土 → NT-WORLD

| 当前位置 | 正确位置 | 原因 |
|----------|----------|------|
| `society/faction.rs` | NT-WORLD (感知) | 领土是空间概念，属于世界感知层 |
| 领土边界 | NT-WORLD spatial | 与 Terrain、BiomeMap 同层 |

**修复**: 将 faction 的领土逻辑拆至 `nt_world::territory`，society 仅做关系管理。

### M5: 记忆巩固 → NT-MEMORY

| 当前位置 | 正确位置 | 原因 |
|----------|----------|------|
| `agents/memory_consolidation.rs` | NT-MEMORY (知识守护) | 记忆巩固是记忆系统的核心操作 |
| 睡眠转移 | NT-MEMORY consolidation | 与 GraphMemory、Embedding 同层 |

**修复**: 将 `memory_consolidation.rs` 迁移至 `nt_memory::consolidation`，agents 仅做触发。

---

## 五、完整缺口清单 (Gap List)

### P0 缺口 (必须立即填充)

| # | 缺口 | 类型 | 来源启发 | LOC | 域 |
|---|------|------|----------|----:|-----|
| G01 | A* 寻路接通 | 冗余激活 | d-zone | +40 | NT-WORLD |
| G02 | RVO 碰撞回避接通 | 冗余激活 | openscope | +30 | NT-WORLD |
| G03 | GOAP 决策接通 | 冗余激活 | wasmbots | +80 | NT-ACT |
| G04 | 行为树接通 | 冗余激活 | boardgame.io | +60 | NT-ACT |
| G05 | 刺激-反应层 | 扁平缺陷 | Mini-MAXS | +100 | NT-ACT |
| G06 | 情绪偏差接通 | 冗余激活 | OpenLife | +30 | NT-FEEL |
| G07 | 信息素读取接通 | 冗余激活 | trailblaze | +25 | NT-SHIELD |
| G08 | Attack/Gather 处理器 | 冗余激活 | wasmbots | +80 | NT-ACT |
| G09 | Save/Load 持久化 | 扁平缺陷 | trailblaze | +300 | NT-MEMORY |
| G10 | Canvas 2D 渲染循环 | 扁平缺陷 | d-zone | +800 | NT-IO |

### P1 缺口 (核心功能)

| # | 缺口 | 类型 | 来源启发 | LOC | 域 |
|---|------|------|----------|----:|-----|
| G11 | Day/Night 循环 | 扁平缺陷 | Stardew | +200 | NT-WORLD |
| G12 | 天气系统 | 扁平缺陷 | Mini-MAXS | +200 | NT-WORLD |
| G13 | 时间控制 | 扁平缺陷 | Evolution | +80 | NT-IO |
| G14 | Spawn 模板系统 | 扁平缺陷 | Stardew | +120 | NT-WORLD |
| G15 | 语义记忆 | 扁平缺陷 | trailblaze | +200 | NT-MEMORY |
| G16 | 记忆巩固 | 扁平缺陷 | trailblaze | +150 | NT-MEMORY |
| G17 | 记忆检索 (cue-based) | 扁平缺陷 | trailblaze | +120 | NT-MEMORY |
| G18 | 八卦协议接通 | 冗余激活 | Mini-MAXS | +40 | NT-SOC |
| G19 | 派系系统接通 | 冗余激活 | OpenLife | +50 | NT-SOC |
| G20 | 通信频道 | 扁平缺陷 | trailblaze | +180 | NT-SOC |
| G21 | 自我对弈 | 扁平缺陷 | microduck_rl | +200 | NT-MIND |
| G22 | 新颖性搜索 | 扁平缺陷 | evolve-search | +180 | NT-MIND |
| G23 | 精英档案 | 扁平缺陷 | autoresearch | +100 | NT-MIND |
| G24 | 决策引擎统一 | 冗余融合 | boardgame.io | +200 | NT-ACT |
| G25 | 社交引擎统一 | 冗余融合 | OpenLife | +150 | NT-SOC |
| G26 | GWT 注意力路由 | 跨域错位 | KVMem | +200 | NT-CORE |
| G27 | 供需定价 | 扁平缺陷 | Stardew | +120 | NT-ACT |
| G28 | 情绪行为偏差 | 跨域错位 | OpenLife | +100 | NT-FEEL |
| G29 | 地图生成 (WFC/BSP) | 扁平缺陷 | wasmbots | +400 | NT-WORLD |
| G30 | Agent 选择/检查 | 扁平缺陷 | Stardew | +150 | NT-IO |

### P2 缺口 (高级功能)

| # | 缺口 | 类型 | 来源启发 | LOC | 域 |
|---|------|------|----------|----:|-----|
| G31 | 粒子系统 | 扁平缺陷 | Mini-MAXS | +300 | NT-IO |
| G32 | 关键帧动画 | 扁平缺陷 | easycanvas | +200 | NT-IO |
| G33 | 音效系统 | 扁平缺陷 | Mini-MAXS | +200 | NT-IO |
| G34 | 神经进化 (NEAT) | 扁平缺陷 | evolve-search | +300 | NT-MIND |
| G35 | 协进化 | 扁平缺陷 | autoresearch | +200 | NT-MIND |
| G36 | 货币系统 | 扁平缺陷 | Stardew | +150 | NT-ACT |
| G37 | 合成/制作 | 扁平缺陷 | Stardew | +250 | NT-ACT |
| G38 | 市场 | 扁平缺陷 | ogame | +200 | NT-ACT |
| G39 | 层次寻路 (HPA*) | 扁平缺陷 | d-zone | +300 | NT-WORLD |
| G40 | 流场 | 扁平缺陷 | openscope | +150 | NT-WORLD |
| G41 | NavMesh | 扁平缺陷 | planck.js | +200 | NT-WORLD |
| G42 | 资源预算执行 | 扁平缺陷 | trailblaze | +80 | NT-SHIELD |
| G43 | 行为异常检测 | 扁平缺陷 | Mini-MAXS | +150 | NT-SHIELD |
| G44 | 进化沙箱 | 扁平缺陷 | autoresearch | +150 | NT-SHIELD |
| G45 | 课程学习 | 扁平缺陷 | evolve-search | +150 | NT-MIND |
| G46 | 迁移学习 | 扁平缺陷 | autoresearch | +150 | NT-MIND |
| G47 | 多目标优化 | 扁平缺陷 | evolve-search | +180 | NT-MIND |
| G48 | 对抗性记忆 | 扁平缺陷 | trailblaze | +150 | NT-MEMORY |
| G49 | 因果记忆 | 扁平缺陷 | trailblaze | +150 | NT-MEMORY |
| G50 | 知识图谱 | 扁平缺陷 | trailblaze | +200 | NT-MEMORY |

---

## 六、统计摘要

| 维度 | 数量 | LOC |
|------|:----:|----:|
| 聚焦冗余 | 5 组 | - |
| 扁平缺陷 | 6 类 | ~2,800 |
| 跨域错位 | 5 处 | 迁移 |
| P0 缺口 | 10 | ~1,545 |
| P1 缺口 | 20 | ~3,090 |
| P2 缺口 | 20 | ~4,060 |
| **总计** | **50 缺口** | **~8,695** |
