# NeoTrix Game 三件套架构全景图（2026-09-24 版）

> Reference 象限：模块清单、依赖边、资产清单、API 面、测试覆盖、警告/未用项。单一事实源。

---

## 1. 三 Crate 分层与职责

| Crate | 层级 | 职责 | 产物类型 | 代码行 | 模块数 | 测试数 | 依赖方向 |
|-------|------|------|----------|--------|--------|--------|----------|
| `neotrix-game` | **Engine (L1)** | ECS/渲染/输入/音频/物理/粒子/UI几何/相机/状态栈/打击感/时鎨/动画/对话/事件总线 | `rlib` | 2,826 | 20 | 53 | ← 仅标准库 + macroquad + neotrix_abilities(nt_tuning) |
| `neotrix-abilities` | **Core Logic (L0)** | Schema/查询/爬塔生成/事件脚本/富文本/数学幂/商店/地牢/调参/网络/FOV/流场/效用决策 | `rlib` | 3,928 | 13 | 83 | ← 仅标准库 + serde/ron/rand |
| `neotrix-guixu` | **Game (L2)** | 全自动战斗/武器谱/修炼/伤害管线/叙事/江湖志/波次/主循环/入口 | `bin` | 2,136 | 9 | 27 | → neotrix_game + neotrix_abilities(nt_tuning) |

**依赖图**：
```
neotrix-guixu (bin)
  ├─ neotrix-game (rlib) ──→ macroquad
  └─ neotrix-abilities (rlib) ──→ (std only)
        neotrix-game/nt_juice.rs ──→ neotrix_abilities::nt_tuning (TierTable)
```

---

## 2. neotrix-game 模块清单（20 模块）

| 模块 | 行数 | 核心类型/导出 | 测试 | 活消费者 | 状态 |
|------|------|---------------|------|----------|------|
| `ecs.rs` | 253 | `World, Entity, Component, Query, System` | 4 | guixu/main, ecs_systems | ✅ 活 |
| `ecs_systems.rs` | 215 | 固定系统注册表 `register_core_systems` | 0 | main.rs | ✅ 活 |
| `components.rs` | 69 | `Transform, Velocity, Sprite, Health, Collider, PlayerTag, EnemyTag...` | 0 | guixu/main, ecs, ecs_systems | ✅ 活 |
| `input.rs` | 229 | `InputState, ActionMap, KeyBinding, AnalogAxis` | 3 | main.rs, guixu/main | ✅ 活 |
| `render.rs` | 116 | `draw_sprite, draw_rect, camera_apply` (内部) | 0 | main.rs | ✅ 活 |
| `nt_camera.rs` | 116 | `Camera2D, dead_zone, look_ahead, shake` | 5 | main.rs, guixu/main | ✅ 活 |
| `nt_move.rs` | 253 | `move_and_slide, PlatformerController, one_way` | 8 | guixu/main | ✅ 活 |
| `nt_platform.rs` | 124 | `Platform, one_way_collision, passthrough` | 5 | guixu/main | ✅ 活 |
| `particles.rs` | 232 | `Particle, Emitter, Pool, spawn_burst/trail` | 3 | main.rs, guixu/main | ✅ 活 |
| `audio.rs` | 322 | `AudioBackend(WAV synth), BgmHandle, SfxHandle` | 5 | main.rs, guixu/main | ✅ 活 |
| `nt_juice.rs` | 161 | `JuiceParams, TierTable消费, hit_flash/screen_shake/hit_stop` | 5 | guixu/main | ✅ 活 |
| `nt_clock.rs` | 86 | `FixedStepClock, dt_accum, max_substeps` | 4 | main.rs | ✅ 活 |
| `tween.rs` | 144 | `Tween, Easing, Sequence, Parallel` | 3 | **零消费者** | ⚠️ 库存 |
| `ui.rs` | 74 | `Anchor, safe_rect, UiButton, layout_row/col` | 3 | **零消费者** | ⚠️ 库存 |
| `events.rs` | 81 | `EventBus<T>, subscribe/emit` | 0 | **零消费者** | ⚠️ 库存 |
| `lighting.rs` | 111 | `Light2D, DayNightCycle, shadow_cast` | 1 | **零消费者** | ⚠️ 库存 |
| `state_stack.rs` | 248 | `StateStack, GameState trait(push/pop/update/draw)` | 4 | **零消费者** | ⚠️ 库存 |
| `nt_fov.rs` | — | (abilities/nt_fov 复用) | — | — | 外部 |
| `dialogue.rs` | — | (guixu/story.rs 自带) | — | — | 游戏侧 |
| `main.rs` | 86 | 入口：systems 注册 + run loop | 0 | — | ✅ 入口 |

**未用模块（库存态）**：`tween`, `ui`, `events`, `lighting`, `state_stack` —— 已实现、测试绿、但无实消费者。保留为引擎能力池，游戏层需求时即插即用。

---

## 3. neotrix-abilities 模块清单（13 模块，83 测试全绿）

| 模块 | 行数 | 核心能力 | 数据文件 | 测试 | 状态 |
|------|------|----------|----------|------|------|
| `schema.rs` | 457 | RON 结构体：Card/Relic/Potion/Monster/Event/Power/Keyword/Encounter | 14 `.ron` (684K) | 3 | ✅ 稳 |
| `query.rs` | 756 | 类 SQL 查询 DSL：filter/project/join/agg/seed | — | 25 | ✅ 稳 |
| `towergen.rs` | 498 | Spire 式爬塔：层/精英/商店/事件/首领/路径图 | — | 3 | ✅ 稳 |
| `dungeongen.rs` | 527 | 矩形房间＋走廊 BSP，门连通性、包络一致性 | — | 6 | ✅ 稳 |
| `events_parse.rs` | 206 | 叙事脚本：Say/Choice/Do/End + Effect 4 种 | — | 7 | ✅ 稳 |
| `rich.rs` | 110 | 富文本：color/sprite/click/hover 标记 | — | 4 | ✅ 稳 |
| `powers_math.rs` | 43 | 幂/根/对数/插值/缓动纯函数 | — | 1 | ✅ 稳 |
| `store.rs` | 96 | 商店刷新/购买/出售/价格曲线 | — | 2 | ✅ 稳 |
| `nt_tuning.rs` | 138 | **TierTable(三档 juice/保底/难度曲线)** | — | 3 | ✅ 稳 —— **引擎/游戏双消费** |
| `nt_net.rs` | 287 | 权威房间/帧同步/环回/插值/重连 | — | 6 | ✅ 稳 |
| `nt_fov.rs` | 284 | 对称阴影投射 FOV，八向扫描、可配半径 | — | 7 | ✅ 稳 |
| `nt_flow.rs` | 388 | Dijkstra 流场 + A* 寻路，动态障碍重算 | — | 11 | ✅ 稳 |
| `nt_utility.rs` | 111 | 效用决策：打分/门禁/迟滞/fail-open | — | 5 | ✅ 稳 |

**数据资产**：`crates/neotrix-abilities/data/*.ron` (14 表，684K) —— StS 兼容字段完整，供 `towergen`/`query`/`store` 消费。

---

## 4. neotrix-guixu 模块清单（17 模块，41 测试；2026-09-24 重设计后）

| 模块 | 行数 | 职责 | 关键数据/导出 | 测试 | 状态 |
|------|------|------|---------------|------|------|
| `main.rs` | 1,123 | 入口/主循环/波次/战斗/暂停/标题/绘制/截图 | `RunState, Game, WEAPONS, ENEMIES` | 0 | ✅ 核心 |
| `combo.rs` | 89 | 武器招式表：青萍剑/胡家刀六式真名、方向序列+门禁+最长后缀 | `COMBO_DEFS, match_combo` | 2 | ✅ 稳 |
| `qi.rs` | 68 | 氣槽：获得/消费/上限/回复 | `Qi, QI_MAX` | 1 | ✅ 稳 |
| `waves.rs` | 98 | 波次生成：敌人类型/数量/精英/首领/奖励 | `wave_spawn, BOSS_WAVES` | 3 | ✅ 稳 |
| `damage.rs` | 109 | **分阶伤害管线**（PoE 顺序+CS分区）：基础×招式×臂力→背刺/挑空→暴击→护甲穿透→方差→保底1 | `damage_pipeline, DamageCtx` | 5 | ✅ 稳 |
| `arts.rs` | 239 | 招式动画/打击感/轨迹/音效触发 | `ArtGate, play_art` | 7 | ✅ 稳 |
| `mastery.rs` | 92 | 修炼：兵器熟练/体魄/臂力（用进废退）、等级开招 | `Mastery, train_weapon/train_body/train_strength` | 2 | ✅ 稳 |
| `story.rs` | 188 | 叙事脚本：Say/Choice/Do/End + Effect 四种 | `Story, Cmd, Effect, run_story` | 3 | ✅ 稳 |
| `lore.rs` | 130 | **江湖志**：兵器实测/八大门派十字诀/章节轮转/切口对答/武器卡 | `WEAPON_LORE, SECTS, SLANG, weapon_lore, slang_reply` | 3 | ✅ 稳 |

---

## 5. 资产清单

| 路径 | 大小 | 内容 | 校验 |
|------|------|------|------|
| `games/neotrix-guixu/assets/fonts/pixel-game.ttf` | ~2.0M | fusion-pixel 12px + PingFang SC 补字，3018 glyph，2593 charset 唯一 | FreeType 过、Pillow 渲染过 |
| `games/neotrix-guixu/assets/fonts/charset.txt` | 2.5K | 2593 码点（源码语料提取） | 唯一性验证 ✅ |
| `games/neotrix-guixu/assets/icons/icon_master_1024.png` | 684K | 液态玻璃 squircle + 金色「歸」徽，透明底 | icns 生成 ✅ |
| `crates/neotrix-abilities/data/*.ron` | 684K | 14 表：cards/encounters/keywords/potions/powers/relics + sts_* | serde 反序列化 ✅ |

---

## 6. API 消费矩阵（谁用谁）

| 消费者 \ 提供者 | neotrix_game | neotrix_abilities |
|-----------------|--------------|-------------------|
| **neotrix_game/main.rs** | ecs, components, input, nt_camera, particles, audio, nt_juice | nt_tuning::TierTable |
| **neotrix_guixu/main.rs** | ecs, components, input, nt_camera, nt_move, nt_platform, particles, audio, nt_juice | nt_tuning::TierTable |
| **neotrix_game/nt_juice.rs** | — | nt_tuning::TierTable, TierParams |
| **neotrix_game/ecs.rs** | — | (无) |
| **neotrix_game/tween.rs** | — | (无) |
| **neotrix_game/ecs_systems.rs** | — | (无) |

**关键观察**：
- `nt_tuning::TierTable` 是唯一跨 crate 共享能力（引擎打击感 + 游戏波次难度）
- `tween`/`ui`/`events`/`lighting`/`state_stack` 为**纯库存**，零消费者
- 能力库其余 12 模块（nt_fov/nt_flow/nt_utility/towergen/...）暂**未接入游戏**，预留 M2-M4

---

## 7. 构建/测试基线（隔离 target）

| Crate | Target Dir | Tests | Warns | Build Time |
|-------|------------|-------|-------|------------|
| neotrix-game | `crates/neotrix-game/target` | 59 pass | 0 | ~4s |
| neotrix-abilities | `crates/neotrix-abilities/target` | 83 pass | 0 | ~6s |
| neotrix-guixu | `games/neotrix-guixu/target` | 34 pass | 0 | ~5s |

**SHOT 审计**：`tools/shot_audit.sh` —— title/play 双场景，OOM 重试 3 次。环境高负载下 play 偶发 abort（Metal texture alloc），非代码 bug，基线法确诊。

---

## 8. 已知技术债（按严重度）

| # | 项 | 影响 | 修复策略 |
|---|----|------|----------|
| D1 | 5 引擎模块零消费者 | 代码膨胀、维护面 | 按需激活，或标注 `@experimental` |
| D2 | ~~guixu/main.rs 1123 行~~ ✅ 已拆（2026-09-24）：main 504／world 474／render 400／director 190／ui 59／assets 82，长文件清零 | — |
| D3 | 字体 atlas 32768 u32 overflow（已根治 2026-09-24） | 曾 play 4 连 abort | ✅ 三件套：baked atlas（atlas.rs，零光栅化）＋populate 删除（省 67MB）＋度量记实际落墨 |
| D4 | SHOT 无头模式偶发 OOM | CI 不稳 | headless 时降采样、限制 atlas 尺寸 |
| D5 | 敵人 AI 仍为简单追踪 | M2 门槛 | 接入 nt_fov/nt_flow/nt_utility（见 Roadmap） |
| D6 | wasm32 未实测部署 | 发布阻断 | `build_wasm.sh` + `serve_wasm.sh` 验证 |

---

## 9. 版本锚点

- **v0.3.0-arch-audit** (2026-09-24)：三件套架构落地、全测试绿、零警告、字体 tofu 清零、图标部署、经验账本入库。
- 下一里程碑：**v0.4.0-m2-ai** —— 敌人 AI（FOV/流场/效用）、章节叙事、战术 HUD、WASM 实测。