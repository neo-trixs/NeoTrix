# NeoTrix Game 进化路线图（2026-09-24 版）

> Explanation 象限：为什么这么排、每条路线现状、缺什么、补齐标准。配合 ARCHITECTURE-MAP 读。

---

## 总览：四阶段演进

| 阶段 | 代号 | 目标 | 预计里程碑 | 关键交付 |
|------|------|------|------------|----------|
| **M1** | **Foundation** | 三件套架构落地、全绿、零警告、字体/图标/审计闭环 | **v0.3.0 ✅ 已达成** | 架构文档、经验账本、SHOT 基线 |
| **M2** | **Intelligence** | 敌人 AI（视野/寻路/决策）、章节叙事、战术 HUD | v0.4.0 | `nt_fov/nt_flow/nt_utility` 接入、8 派轮转、HUD 完备 |
| **M3** | **Expansion** | 爬塔/地牢/网络/商店/遗物/药水 —— 复用 abilities 全谱 | v0.5.0 | `towergen/dungeongen/store/nt_net` 落地游戏层 |
| **M4** | **Polish & Ship** | WASM 发布、Steam/itch 集成、模组/编辑器、性能收敛 | v1.0.0 | wasm32 实测、分发管线、模组 API |

---

## M2 详细路线（当前冲刺）

### M2-A：敌人 AI 三件套接入  ← **最高优先级**

| 子任务 | 现状 | 缺失 | 补齐标准 | 验收 |
|--------|------|------|----------|------|
| **FOV 视野门** | `nt_fov` 7 测试绿、对称阴影投射就绪 | 未挂到 Enemy 实体 | `Enemy::fov_radius`、`update_fov(dt)`、只攻击视野内目标 | 单测：墙后不追、视野内即锁 |
| **流场寻路** | `nt_flow` 11 测试绿、Dijkstra+A* 就绪 | 未对接平台物理/单向台 | `FlowField::recompute(grid, target)`、每帧 `vel = flow.grad(pos)`、`nt_move` 滑动 | 单测：绕障碍、单向台不卡、动态重算 < 2ms |
| **效用决策** | `nt_utility` 5 测试绿、打分/门禁/迟滞就绪 | 未定义 Enemy 行为树 | `UtilityBrain{actions:[Chase, Attack, Flee, Patrol], hysteresis:0.15}`、fail-open 回巡逻 | 单测：血量<30%→Flee、CD 内不抖、失目标→Patrol |

**数据需求**：
- `enemy_archetypes.ron`：`{id, hp, atk, fov_r, speed, brain_weights, combo_set}`
- `wave_ai_params.ron`：每波全局权重（凶险度/群体/精英比）

**落地文件**：
- `games/neotrix-guixu/src/ai.rs` (新建) —— `EnemyBrain`, `decide()`, `update_fov()`, `flow_steer()`
- `games/neotrix-guixu/src/main.rs` 引用 `ai::think_all()` 在 `update_play` 固定步长内

---

### M2-B：章节叙事扩展（八大门派轮转）

| 波次 | 门派 | 触发内容 | 数据缺口 |
|------|------|----------|----------|
| 10 | 少林 | 横幅「少林寺·达摩院」，切口对答，波次加成：敵人+护甲 | `SECTS[0].banner`, `SECTS[0].koan` |
| 13 | 武当 | 横幅「武当山·紫霄宫」，太极拳意，波次加成：敵人+氣上限 | `SECTS[1]...` |
| 16 | 峨眉 | ... | ... |
| 19 | 丐帮 | ... | ... |
| 22 | 唐门 | ... | ... |
| 25 | 青城 | ... | ... |
| 28 | 华山 | ... | ... |
| 31 | 逍遥 | ... | ... |

**现状**：`lore.rs::SECTS` 已有 8 条骨架（名/口诀/兵器/特色），缺 `banner`/`koan`/`wave_bonus` 字段。

**补齐**：
1. 扩展 `SectLore` 结构体加三字段
2. 8 条完整数据（真实武侠考据）
3. `waves.rs` 在 `wave_spawn` 注入 `sect_bonus(wave)`
4. `main.rs` 波次切换时 `story::run_sect_intro(sect)`

**验收**：波次 10/13/16... 出现横幅→切口选择→通过得奖励/失败得惩罚、敌人获得该派加成。

---

### M2-C：战术 HUD 补全

| HUD 元素 | 现状 | 缺失 | 实现位置 |
|----------|------|------|----------|
| 护甲条（敌/我） | 无 | `Health` 组件无 armor 字段 | `components.rs` 加 `armor: f32`，`damage.rs` 读写，HUD 绘制 |
| 氣槽（玩家） | `qi.rs` 有逻辑 | 无可视化 | `main.rs draw_hud()` 绘制：底条+充填+数值 |
| 武器等级/招式解锁提示 | `mastery.rs` 有等级 | 无浮层 | 暂停页已接 `weapon_lore`，战斗中招式解锁 toast |
| 切口对答浮层 | `lore.rs::slang_reply` 有逻辑 | 无 UI | 叙事 Choice 时渲染对联式两行 |

**技术**：复用 `nt_juice` 的 `TierTable` 做 HUD 动画（闪/抖/停），复用 `tween`（库存）做进度条缓动。

---

### M2-D：WASM 实测部署

| 步骤 | 命令 | 验收 |
|------|------|------|
| 1. target 安装 | `rustup target add wasm32-unknown-unknown` | ✅ 已装 |
| 2. 编译 | `CARGO_TARGET_DIR=games/neotrix-guixu/target-wasm cargo build -p neotrix-guixu --target wasm32-unknown-unknown --release` | 无链接错 |
| 3. 打包 | `wasm-bindgen --out-dir games/neotrix-guixu/wasm_out --target web games/neotrix-guixu/target-wasm/.../neotrix-guixu.wasm` | `.js/.wasm` 产出 |
| 4. 本地服务 | `cd /tmp/wasm_out && python3 -m http.server 8080` | 浏览器跑通、音频/字体/截图正常 |
| 5. 部署脚本 | `games/neotrix-guixu/tools/build_wasm.sh` + `serve_wasm.sh` 入库 | 一键复现 |

**已知坑**：macroquad wasm 需 `miniquad` backend、音频需 `web_audio` feature、字体需预加载 base64。已在 `Cargo.toml` 配好。

---

## M3 预览（abilities 全谱落地）

| 能力模块 | 游戏层落地点 | 数据需求 |
|----------|--------------|----------|
| `towergen` | 无限塔模式：层/精英/商店/事件/首领 | `tower_profile.ron` |
| `dungeongen` | 探索地图：房间/走廊/锁门/宝箱 | `dungeon_theme.ron` |
| `store` | 波次间商店：刷新/购买/售价曲线 | `shop_inventory.ron` |
| `nt_net` | 联机对战/合作：房间/帧同步/重连 | `net_config.ron` |
| `relics/potions/keywords` | 遗物/药水/关键词系统 | 复用 `sts_*.ron` 直改名 |

---

## M4 收敛指标（发布门槛）

| 指标 | 目标 | 当前 | 备注 |
|------|------|------|------|
| 帧时间 (P99) | < 16.6ms (60fps) | ~8ms (debug) | release 更优 |
| 启动到可玩 | < 2s | ~1.2s | wasm 首屏 < 3s |
| 内存峰值 | < 300MB | ~180MB | 含 atlas |
| 二进制体积 | < 15MB | ~12MB | wasm gzip < 5MB |
| 零 `unwrap`/`expect` 生产码 | 0 | 0 | 已强制 |
| 单测覆盖核心逻辑 | > 90% | ~85% | AI/HUD 新增补齐 |

---

## 数据补齐清单（按优先级）

| 优先级 | 文件 | 字段/条数 | 来源 | 备注 |
|--------|------|-----------|------|------|
| P0 | `enemy_archetypes.ron` | 12 种×8 字段 | 武侠设定 + 平衡表 | M2-A 先决 |
| P0 | `wave_ai_params.ron` | 30 波×4 权重 | 曲线设计 | M2-A 先决 |
| P0 | `SectLore` 扩展 | 8 条×3 新字段 | 真实门派考据 | M2-B 先决 |
| P1 | `armor` 进 `Health` | 1 字段 | 伤害管线已有护甲穿透 | M2-C 先决 |
| P1 | `tower_profile.ron` | 50 层配置 | Spire 参考 | M3 |
| P1 | `dungeon_theme.ron` | 4 主题×房间集 | BrogueCE 参考 | M3 |
| P2 | 模组 manifest schema | `mod.toml` 规范 | 自定 | M4 |

---

## 并行公约（防冲突）

1. **单窗口串行重活**：cargo 全量/测试/运行三选一，target 隔离（各 crate 本地 `target/`，禁 /tmp）
2. **子代理只写新文件**：`ai.rs`/`enemy_archetypes.ron`/`wave_ai_params.ron`/`build_wasm.sh` 等新增交子代理；主线程做集成（main.rs 引用、系统注册）
3. **跨会话移动 crate 前喊一声**：`_archive/` 搬运需留 `handoff-*.md`
4. **Edit 后必重读（R-P16）**：防 `update_play` 类误替
5. **数字对账**：每收尾报 `tests: 53/83/27`、`charset: 2593/2593`、`warnings: 0/0/0`

---

## 里程碑检查单（Definition of Done）

### M2 DoD
- [ ] `ai.rs` 编译通过、单测 ≥ 10、Enemy 在墙后不追、绕障碍、低血逃离
- [ ] 8 派 `banner/koan/wave_bonus` 全入库、波次 10/13/16... 触发横幅+切口+加成
- [ ] HUD：护甲条/氣槽/招式解锁 toast/切口浮层全可见、动画流畅
- [ ] `build_wasm.sh`/`serve_wasm.sh` 入库、浏览器跑通、无控制台红字
- [ ] 三线测试全绿、零警告、SHOT 2/2 过

### M3 DoD
- [ ] 无限塔模式可玩（50 层）、地牢探索可玩、商店可买卖
- [ ] `nt_net` 局域网双人对战/合作跑通
- [ ] 模组加载器雏形（热更 `.ron`/脚本）

### M4 DoD
- [ ] wasm 部署 itch.io/自建站、Steam 构建管线
- [ ] 性能指标全达标、内存无泄漏（valgrind/heaptrack）
- [ ] 文档站（mdBook）上线、模组教程完备

---

## 回溯索引

- 架构全景：`docs/architecture/GAME-ARCHITECTURE-MAP-2026-09-24.md`
- 经验账本：`docs/architecture/LESSONS-2026-09-24.md`
- 正典标准：`docs/standards/NEOTRIX-STD-1.0.md`
- 文档地图：`DOCUMENTATION-MAP.md`