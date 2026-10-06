# NT-GAME 收敛路线 —— 补渲染层 + 删重复抽象

> 2026-10-06 · 分支 `feat/capability-absorb-20260828`
> ⭐ 本文只写**实测**结论。所有行号可复核，推测处明确标注。

---

## 0. 起点事实（实测）

`neotrix-core/src/l5_cognition/nt_mind/nt_game/`：**73 文件 / 20,844 行 / 449 个 `#[test]`**。
**零未挂载目录**（两条独立路径验证：仓库自有门 `nt_orphan_dir.py` + 自写脚本）。

⇒ 问题不是「没编译」，而是**编译了、测试了、但没人用**。

### 0.1 `render/` 的真相

模块头 `render/mod.rs:8` 写着「渲染管线（**macroquad 即时模式**）」，但：

| 实测 | |
|---|---|
| `macroquad` 在 `Cargo.toml` | **不存在** |
| `macroquad` 全仓出现位置 | **仅 3 处注释**（`render/mod.rs:8`、`input.rs:225`、`:288`） |
| `render/` 内真实 IO | 只有 `save.rs:3` 的 `std::fs` 与 `hot_reload.rs:76,126` 的 `read_dir`/`metadata` |
| 绘制调用 | **零** |

⇒ 那句注释是**无实现依据的文档债**，`components.rs` 的 `Transform2D/Sprite/Camera2D/Collider`
是纯数据结构。

### 0.2 ⭐ 主线：三个游戏被实现了**三遍**，而活的那一遍不在 `nt_game` 里

| 实现 | 位置 | 行数 | 状态 |
|---|---|---|---|
| **A** `NtGameEnv` 正式版 | `hex_crucible.rs:637` + `builtin/hex_tictactoe.rs:142` + `builtin/game_2048.rs:145` | 1,824 | 仅被自己的 `mcp.rs:17,18,21` 引用 |
| **B** `trait AutoGame` 内联版 | `evolution.rs:83` + `AutoTicTacToe:96` + `Auto2048:224` + `AutoHexCrucible:411` | ~530 | 零消费者 |
| **C** 自由函数版 | `l5_cognition/nt_mind/nt_mind_background_loop/handlers_game.rs:44,81,136` | ~150 | ⭐ **活的** |

活证据链：`run.rs:903` `spawn_handler!(GAME_TRAINING_INTERVAL_SECS, "game_training", …)`、
`run.rs:50` `= 300`（5 分钟一次）→ `handlers_game.rs:367 handle_game_training`。

**B 与 C 的注释写着同一个理由**：
- `handlers_game.rs:12`「Self-contained game types (avoids cross-module dependency issues)」
- `evolution.rs:79`「Inline Game Engines (self-contained, no cross-module imports)」

`GameTickReport` 也重复：`evolution.rs:62`（13 字段）vs `handlers_game.rs:15`（11 字段）。

⛔ 而 `mcp.rs` 的唯一调用者是它自己（`mcp.rs:133` → `mod.rs:85 default_registry()`）
⇒ **闭环自指，外部零入口**。全仓 `examples/`/`benches/`/`crates/` 对 nt_game 引用数 = **0**。

---

## 1. 渲染层选型（联网调研，附来源）

### 1.1 三个前提纠正

| 常见说法 | 实测 |
|---|---|
| 「`macroquad` 是候选」 | 0.4.16（2026-07-30），**262 open issues**、**WebGL2-only**、**无 WebGPU 计划**、master 曾编译不过（[issue #1033](https://github.com/not-fl3/macroquad/issues/1033)）；社区实战复盘评「lightly maintained」（[Reddit 2025-12](https://www.reddit.com/r/rust_gamedev/comments/1pc9wyv/)） |
| 「Phaser 底层是 Pixi」 | ❌ **错**。仅 Phaser 1/2 用 Pixi；**Phaser 3（2018）起自研渲染器**，Phaser 4（2026-04）整个重建（[作者本人回答](https://www.html5gamedevs.com/topic/34261-does-phaser-3-use-pixijs/)） |
| 「Rust 侧从零引入传输层」 | ⛔ **`axum 0.8` + `features=["ws"]` 已在 `neotrix-core/Cargo.toml:178`**，且 `l1_action/nt_io/nt_io_web/server.rs:207` 已有能跑的 `ws_echo_handler` |

另：`tokio-tungstenite 0.24` 已声明（`:171`）但 `src/` **零引用** ⇒ **死依赖**，要么接上要么移出。

### 1.2 传输：WebSocket + JSON（`rev` 单调）

| 方案 | 2026 现状 | 结论 |
|---|---|---|
| **WebSocket + JSON** | axum `ws` 已就位 | ✅ **主力**。单客户端 1,000 msg/s 以内 `tokio::broadcast` 不是瓶颈（[axum-signal 实测](https://github.com/rdcm/axum-signal)） |
| WebTransport | 2026-03-24 起全浏览器可用，但需 HTTPS；[Widely Available 预计 2028-09](https://web-features-explorer.github.io/web-features-explorer/features/webtransport/) | ⛔ Baseline 未稳 |
| WebRTC data channel | 需信令 + 打洞 | ❌ 单机不值这个复杂度 |

⭐ **必须带单调 `rev`**：`broadcast::Sender::send` 在无接收者时**直接丢弃**
⇒ 丢帧后客户端无法自愈。且**必须先 `subscribe()` 再发快照**，顺序反了会漏帧
（模式参考 [agent-of-empires 的 `?since=<seq>`](https://github.com/agent-of-empires/blob/main/src/server/acp_ws.rs)）。

### 1.3 ⛔ 明确否掉的选项（附依据）

- **Bevy / lightyear**：Bevy **官方维护者自己说** `bevy_ecs` 不适合 server-side
  （[discussion #21820](https://github.com/bevyengine/bevy/discussions/21820)：「Warp/Tokio 更适合
  backend 而不是面向主线程」）。且 `NtGameEnv` 是 `Box<dyn>` + `&mut self` 的回合制模型，
  **引入 Bevy 等于重写游戏子系统**。
- **任何 Rust 原生渲染后端**（macroquad / ggez / raylib-rs / Piston）：客户端已有浏览器，
  引入等于与「Rust 只做权威逻辑、不要 Rust 侧主循环」**天然冲突**。
- **Phaser 用于卡牌/棋盘**：① Phaser 4 已 WebGL-only、Canvas 官方废弃
  （[官方公告](https://phaser.io/news/2026/04/phaser-4-renderer-faster-cleaner-and-built-for-modern-games)）
  ② 渲染单元是 Phaser 专属 `RenderNode`，**灌不进**我们的 `Transform2D` 场景图
  （[v4.0.0 notes](https://github.com/phaserjs/phaser/releases/tag/v4.0.0)）
  ③ 自带主循环 + 场景管理 + 物理，而这三样 Rust 侧**全有** ⇒ 两个循环打架
  ④ v4.0.0 才 2026-04-10，生态仍在震荡
  ⑤ 它的核心卖点（`SpriteGPULayer` 百万精灵）对我们**几十个实体**的客户端毫无价值。
- **JSON 字段级增量**：实测 120 实体全动时只省 **19%**（`[[1234,100.5]]` 的语法开销
  几乎等于省下的字段名）。⇒ **全量快照 + 客户端插值**，真要增量切二进制
  （postcard 实测压到 JSON 的 22.8%）。

### 1.4 前端三选一（实测依据）

| 类别 | 选型 | 依据 |
|---|---|---|
| **卡牌/棋盘/点击**（炉石、杀戮尖塔、Cookie Clicker） | **React + DOM**，需要遮盖排序/粒子时升级 **[react-konva 19.3.0](https://www.npmjs.com/package/react-konva)** | 卡牌有大量文本/可访问性/hover/拖拽 ⇒ DOM 最省事。react-konva 周下载 **2,371,207**、维护至 2026-09，是三者中最健康 |
| 2D 动作 | **Phaser 4.2** 或 **PixiJS 8.16** | [Phaser 4.2](https://phaser.io/news/2026/07/phaser-4-2-spine-renderer-mesh2d-stencil) 的 `SpriteGPULayer`/`TilemapGPULayer`/`Mesh2D`/`Stencil` 是大量动画的最优；PixiJS 走 WebGPU 一等公民 |
| 3D | **three.js + R3F 9.6.1** | v10 仍是 alpha、drei 支持 **pending**（[官方 migration](https://r3f.docs.pmnd.rs/next/migration/v10)）⇒ **不要等 v10** |

### 1.5 ⭐ 观测体积实测（决定 schema 形态）

按 `game_2048.rs::text_state` 的真实格式复现：

| 载荷 | 字节 | 对比 |
|---|---|---|
| **2048 文本观测（现状）** | **123 B** | 1.00× |
| 2048 场景图 verbose（直接映射 `components.rs`） | 1,563 B | 12.7× |
| **2048 场景图 compact**（扁平数组 + 资产 id 化） | **124 B** | **1.01×** |
| 卡牌式（20 卡 + 2 敌 + 意图）verbose | 10,771 B | 87.6× |
| **卡牌式 compact** | **502 B** | **4.1×** |

⭐ **卡牌的 21.5× 缩减全部来自「资产与实例分离」**（`texture_path`/`region`/`color`
在 20 张卡上逐字节重复），**不是来自增量**。

⚠️ **棋盘游戏应送网格坐标，不是 `Transform2D`** —— 布局是前端的职责，
Rust 只送逻辑位置。这是 1,563 → 124 B 的核心原因。

### 1.6 schema 三原则

1. **`rev` 必填且单调** —— 丢帧自愈的唯一依据
2. **资产与实例分离** —— 场景图里只放整数 id
3. ⭐ **保留 `text_state` 作为第三个字段** —— `observe()` 现在服务 LLM
   （GRPO self-play），**不要为了渲染把模型输入弄坏**

具体 schema（2048 / 卡牌各一例）见
[调研原始报告](https://github.com/muddassarsaleemiqbal/ludo-royale)（同架构项目）
与本窗口实测数字。

### 1.7 同架构可抄的开源项目

| 项目 | 可借鉴 |
|---|---|
| ⭐ **[ludo-royale](https://github.com/muddassarsaleemiqbal/ludo-royale)** | Rust 引擎 + axum WS + React 客户端，**与本仓架构逐条相同**。① 一份逻辑 wasm/server 两种部署 ② **30 秒回合超时 + 断线宽限 + AI 接管**防对局挂死 |
| **[channel-zero](https://github.com/JacobStephens2/channel-zero)** | 「**每个房间 = 一个独占 GameState 的 Tokio task，状态周围无锁，所有变更经一条 mpsc 命令队列串行化**」⇒ 直接解决 `NtGameEnv: !Send` 与多客户端锁竞争 |
| **[bevy_netahoy](https://github.com/wtfdemon/bevy_netahoy)** | 「**The same function runs everywhere**」—— 客户端预测 / 回放 / 服务端消费共用同一份 `step.rs`；POD 单结构体原则；8 条指令冗余发送抵消丢包 |
| **[elura](https://github.com/Arion-Dsh/elura)** | 「**The timeline comes from declaration, not arrival**」；`Delta/Keyframe` 双模式是标配 |
| **[vanillastone](https://github.com/amvid/vanillastone)** | 「**The client is a dumb renderer; it never decides game state**」 |

⛔ **没找到**「Rust 权威 + **纯浏览器**（非 wasm）渲染」的成熟项目 ⇒ 这条路线**是先驱，没有现成答案**。

---

## 2. 冗余审计（实测）

### 2.1 可安全删（按「删掉的行数 ÷ 风险」排序）

| 排名 | 目标 | 行数 | 测试 | 风险 |
|---|---|---|---|---|
| 1 | **`render/` 整目录** | **4,836** | 100 | **极低**：无后端依赖、零外部消费者、零绘制 |
| 2 | `world/` 除 combat | 2,764 | 12 | 极低：12 个子模块逐个实测零消费者 |
| 3 | `consciousness/` | 1,297 | 43 | 极低：唯一引用是重导出；外部同名类型全是**不同类型**（全仓 4 个 `PhiReport` / 3 个 `HealthReport` 并存） |
| 4 | `rpg/` | 1,257 | 13 | 极低：`cultivation ↔ mod` 自循环，外部零入口 |
| 5 | `evolution.rs` 的 3 个 `Auto*` | ~530 | — | 低：纯重复，但**须与 §3 同批**做 |
| — | `ai/` 413 · `persistence/` 145 · `ecs/` 332 · `nt_clock+nt_commands` 227 | 1,117 | 25 | 极低 |
| — | `world/combat/combat.rs` | 215 | 13 | 极低（仅重导出 + tests） |

**合计 ≈ 10,684 行 = nt_game 的 51.2%**。

### 2.2 已排除的「疑似重复」（实测不成立）

| 候选 | 结论 |
|---|---|
| `hex_crucible` ↔ `builtin/hex_tictactoe` | ❌ **不同游戏**：前者六边形棋盘/E8 六线/能量/共振 hamming≤2/Phi 计分；后者 3×3 三子棋 |
| `world/dungeongen` ↔ `world/generation` | ❌ 不同职责：BrogueCE 地牢房间图 ↔ Perlin 噪声+生物群系+POI。`generation.rs:290` 的 `Dungeon` 是 `PoiType::Dungeon` **枚举变体** |
| `play/{adaptive,advantage,buffer}` | ❌ **是流水线不是重复**：调难度 → 环形缓冲 → 信用分配 → GRPO。字段/方法零重叠 |
| `render/physics::SpatialHash` ↔ `l2 GridIndex` | ❌ 算法同构但**跨层不可合并**（L5 vs L2，`check-layer-deps.sh` 禁） |

### 2.3 真重复（需合并而非删）

| 重复 | 证据 |
|---|---|
| **`rpg::Equipment*` ↔ `world::combat::equipment::*`** | `EquipmentSlot`(8 变体同序) / `EquipSlot`；`display_name()` **字面量完全相同**；**9 个同名方法同义**；碰撞已被 `world/mod.rs:18` 用别名 `as CombatEquipmentLoadout` 掩盖，而 `nt_game/mod.rs:76` **同时**导出两份 `EquipmentLoadout` ⇒ **同 crate 内两份并存**。B 侧是超集且有唯一外部消费者 `error_conversions.rs:20,26` ⇒ **删 A 留 B** |
| `render/save.rs` ↔ `persistence/save_state.rs` | 都是 `std::fs::write` + serde JSON；⚠️ **两套互不知情的版本机制**（`CURRENT_SAVE_VERSION` vs `version` 字段）⇒ 两者都零消费者 ⇒ **全删**，别合并 |
| `render/input.rs` ↔ `world/input/mod.rs` | `InputAction` 在一侧是 **struct**、另一侧是 **enum(14 变体)**；`clear_frame()` 重复。两者都零消费者 ⇒ 全删 |
| `state_machine::GameState`(trait) ↔ `env.rs::GameState`(struct) | ⚠️ **同名不同物**，非重复，是命名撞车 |

### 2.4 必须留

`framework.rs`(613，10 消费者) · `env.rs`(280) · `hex_crucible.rs`(1,021，主训练游戏) ·
`builtin/*`(803) · `play/{advantage,buffer,grpo_adapter,scaling}`(1,355) ·
`world/combat/{equipment,inventory}`(886，唯一外部生产消费者) · `events/`(211)

⭐ `hex_crucible.rs:49` 已有**去重先例**：`RESONANCE_THRESHOLD` 改为
`pub use neotrix_types::…`（注释记「曾有全仓 4 份副本」）⇒ 本仓认可的收敛模式。

---

## 3. 执行顺序（**顺序错了会白删**）

```
阶段 1 ▸ 接线（最高价值，先做）
  handlers_game.rs 的 3 个手写游戏 → 改调 nt_game 的 GameEvolutionLoop
  删 evolution.rs:83-610（trait AutoGame + 3 个 Auto*）
  删 handlers_game.rs:44-190（3 个 run_* + rng_step）
  两份 GameTickReport 合并为一份
  ⇒ 净减 ~980 行重复，且 nt_game 从「零外部消费者」变成系统活路径
  ⚠️ 不做这一步，后面删完也还是没人用

阶段 2 ▸ 删零消费者块（可证伪）
  render/ · world-except-combat · consciousness/ · rpg/ · ai/
  persistence/ · ecs/ · nt_clock+nt_commands · world/combat/combat.rs
  ⇒ 预测测试数 449 → 152

阶段 3 ▸ 补渲染层（DTO 层，不引入任何渲染后端）
  新建 nt_wire.rs：rev / assets 分离 / 网格坐标 / 保留 text_state
  observe() 增返回场景图 JSON；先只加字段不破坏现有 String 返回
  server.rs:207 旁边加 /ws/game/{id}，加 rev + 先 subscribe 后快照两约束
  删 tokio-tungstenite 死依赖；改 render/mod.rs:8 的 macroquad 注释
```

### 阶段 2 的证伪工具（⭐ 必做）

删完测试数**不降** ⇒ 删错了（有未发现的消费者）；
**多降** ⇒ 连带删掉了本该保留的活资产测试；
**编译不过** ⇒ 消费者表有洞。

基线用本仓既有的 `scripts/check-test-baseline.sh`（只读模式）取，
⛔ **不要手改** `scripts/test-failures-baseline.txt`。

---

## 4. 风险清单

| # | 风险 | 缓解 |
|---|---|---|
| 1 | `components.rs` 直接当 wire schema ⇒ 卡牌 10.8 KB/步 | **DTO 与 ECS 结构解耦**：新增 `nt_wire.rs` 只放整数 id + 网格坐标 |
| 2 | JSON 字段级增量几乎白干（全动只省 19%） | **先全量快照 + 客户端插值**；真要增量切 `postcard` |
| 3 | `broadcast` 丢帧不可恢复 | 每个响应带单调 `rev`；跳号 → `?since=<rev>` 全量重同步；**先 subscribe 再发快照** |
| 4 | `NtGameEnv: !Send` + 多客户端锁竞争 | 照 channel-zero：**每房间一个独占 task + mpsc 命令队列，状态周围无锁**；改 `.rs` 后必跑 `nt_lock_audit.py` |
| 5 | 误引入 Bevy/macroquad 导致两个游戏循环打架 | **渲染层 100% 放浏览器**；删掉那句无依据的 macroquad 注释 |
| 6 | 阶段 2 删错 ⇒ 丢掉「编译了但没用」的唯一可运行实现 | 用 §3 的测试数预测**证伪**；分小批提交，每批单独验证 |
---

## 6. 阶段 1 执行裁决（2026-10-06，实测驱动）

> 本节由实际执行推翻并修正了本文件上半部分的部分判据。**保留原文不改写**，
> 因为「原判据错在哪」本身是证据。

### 6.1 裁决反转：审计判的「530 行冗余副本」不是冗余

原判据：`evolution.rs` 里 `AutoTicTacToe`/`Auto2048`/`AutoHexCrucible` 是
`builtin/` 与 `hex_crucible.rs` 的重复实现，删掉即可。

**实测反证**：被删的 `trait AutoGame` 比 `NtGameEnv` **多两个方法** ——
`reward(player) -> f64` 与 `board_hexagrams() -> Vec<u8>`。
两者都不是 `NtGameEnv` 的方法。

⇒ 这不是随手重写的副本，是**给缺失合约打的补丁**。
若只删副本不补合约，等于**把补丁连同它掩盖的能力一起删掉**。

### 6.2 六个实测缺陷

| # | 缺陷 | 实测证据 | 修法 |
|---|---|---|---|
| D1 | `NtGameEnv` 无终局出口 | `win_rate ≡ 0.000`、`scores={0:0.0,1:0.0}`；三局内部**早有** `winner` 却无出口 | 新增 `GameOutcome` + `outcome()` |
| D2 | 两局未定义 phi，吃默认 `0.0` | `phi_avg ≡ 0.0000`，且「真是 0」与「没定义」不可分辨 | `phi_contribution()` 改 `-> Option<f64>` + 补两局实现 |
| D3 | `health = win_rate*0.6 + phi*0.4` 结构退化 | 零和对称局胜率≈0.5；记分制无胜率概念 | **保留公式 + 事实注释**，不在 D5 之前换公式 |
| D4 | `hex_crucible.rs:457` 下溢 panic | `attempt to subtract with overflow`，`constellation≥2` 必踩 | 差值只算一次 `saturating_sub` |
| D6 | 回合预算双重权威 | 游戏预算 20/30/40/50/60/80 vs 外层硬帽 50 ⇒ c=4/5 `turns≡50.0`、`30/30` 全 draw | 新增 `turn_budget()`，有效预算 `= max(安全帽, 本局预算)` |
| **D5** | **假策略（本轮未修，最高优先）** | `idx = (ep_seed + steps) % actions.len()` —— 注释自称 `simulating a policy network` | 待接线 `play/self_play_loop.rs::SelfPlayLoop` |

修后实测（六星位全部产出真实胜负）：

| c | 游戏 | 修前 | 修后 |
|---|---|---|---|
| 0 | HexTicTacToe | `win_rate ≡ 0` | `W/L/D=27/0/3`，`0.900` |
| 1 | 2048 | `win_rate ≡ 0` | `0/0/30`（**正确**：记分制无胜者） |
| 2 | HexCrucible | **panic** | `10/14/6`，`0.333` |
| 4 | HexCrucible | `0/0/30`，`turns=50`（截断） | `12/17/1`，`turns=60`，`0.400` |
| 5 | HexCrucible | `0/0/30`，`turns=50`（截断） | `12/15/3`，`turns=80`，`0.400` |

### 6.3 对阶段 2（删冗余）的修正

本节 6.1 已证明：**「零消费者」不等于「可删」**。
因此阶段 2 的删除清单必须先经 `rg` 逐块复核，且判据从
「零消费者」升级为「**零消费者 且 其存在理由已被真正替代**」。
当前基线：游戏测试 **458** 条（原估 449→152 的预测作废，须重算）。

### 6.4 未闭合项

- 🔴 `check-feature-gates.sh` 本次改动了 trait 签名，**改动后**的验证被中止 ⇒ 提交未覆盖 6 个非默认 feature 的门检查。不得引用改动前那次 PASS。
