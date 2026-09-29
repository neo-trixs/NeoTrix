# ABSORPTION-GAME-SKILLS — 游戏 skill 八项吸收（2026-09-23）

> 方法（worldbuilding-absorption）：先立公理，再推模块，同会话接到生产（R-P79）。
> 源：`/tmp/absorb/gamedev-skills`（73 skills）· `higgsfield-ai/skills`
> （game-flow + game-design-system §0–13）· OpenAI `codex/use-cases/browser-games`。

## 一、八项公理

**A. higgsfield-websites** — 游戏=6纯函数（meta/setup/validateAction/applyAction/isGameOver/viewFor）；
逻辑无导入、无 wall-clock、JSON 可序列化（`check:logic`）；`validateAction` 是唯一防线；
`viewFor` 是信息隐藏（卡牌只给手牌+他人牌数）；PLAN 关门才许写码；唯一规划文件 `assets.csv`
（id/role/type/description/size/style line/source）；STYLE FORMULA 逐字节复用，无 FORMULA 无视觉。

**B. game-engine**（phaser-core/physics-tuning）— 物理定步长 50–60Hz、渲染只管呈现；
CCD+速度 cap 防穿透；力/速度/碰撞读数只在 fixed 步内。

**C. game-developer**（unity/unreal + performance-optimization）— 先 profile 再动手；
性能=开工前定的帧预算（超标=crash 级 bug）；诊断序：draw calls→GPU→CPU→ spikes；
对象池/合批/帧内零分配/资产预算；Gameplay 组件化 + 数据驱动（≈RON）。

**D. game-ui-design**（game-ui-ux）— 锚点+容器，禁绝对像素；参考分辨率缩放+宽高比策略+安全区；
每屏初始焦点+焦点导航；屏栈 push/pop 代 flag soup；HUD 事件驱动，禁每帧轮询。

**E. game-design-theory**（§§1–5/9–10 + card-game + roguelike）— 牌=数据+效果解释器；
zone 唯一性（move=remove-then-add）；出牌原子（先验后提交）；seeded shuffle；
能量调度；连通性 flood-fill 保证；run/profile 存档分离；成本曲线/非传递结构/难度 gap/
经济源汇/EV+方差+pity；**数值全进数据，一次只调一个**。

**F. game-feel** — 5–8 微反馈/100ms；瞬态夸张必回静；按重要性 small/medium/large 分级；
trauma² 震屏（相机偏移非本体，正弦噪声非逐帧随机）；顿帧=时间缩放+真实时间恢复；
squash 用 BACK/EASE_OUT（线性=机械）；juice 不碰模拟、不阻塞输入、配减震屏/减闪烁项。

**G. multiplayer-game**（godot-multiplayer + replication-and-rpc + higgsfield rooms）—
服务端权威，客户端只发 intent；输入走可靠通道，位置走不可靠最新胜出；只传纯数据；
per-node authority；visibility 过滤≈viewFor；房间 per-path，规则进单测（vitest 级）。

**H. develop-web-game**（OpenAI browser-games）— PLAN.md 八要素（目标/主循环/输入/胜负/
成长/视觉/技术栈/里程碑）；AGENTS.md（构建测试验证/PLAN 指引/.logs/截图迭代/.prompts 复用）；
先写概念再迭代；栈 Next+Phaser/Pixi、前后端 Fastify/WS/Postgres/Redis（我方对应 macroquad+wasm）。

## 二、生产接线（本会话落地）

| # | 产物 | 公理 | 测试 |
|---|---|---|---|
| 1 | `crates/neotrix-spire/src/nt_tuning.rs`：TierTable/pity/power_gap/soft_cost/on_cost_curve | E、F1 | 3 |
| 2 | `crates/neotrix-spire/src/nt_net.rs`：Room/validate/apply/viewFor/LoopbackBus/CounterRoom | G、A | 5 |
| 3 | `crates/neotrix-game/src/nt_juice.rs`：trauma/freeze/tick/shake/trigger（零 macroquad，纯 f32） | F | 5 |
| 4 | `crates/neotrix-game/src/nt_clock.rs`：FixedStepper（acc+螺旋 guard） | B、§12.1 | 4 |
| 5 | `src/ui.rs`：Anchor 九锚点 + safe_rect；login_buttons/hud/panel_centered 锚点化（数值逐位一致，单测锁定） | D | 3 |
| 6 | 挂载：`update` 首行 `juice.tick`（顿帧阀，输入先采样不吞键） | F | — |
| 7 | 挂载：`render` 世界层包偏移相机（HUD 零位移）；debug 加 `Frame ms + Trauma/FREEZE`（§6.3） | F、C | — |
| 8 | 挂载：`card_damage` 按伤害分级 trigger（small 无顿帧，击杀恒 large） | F1 | — |
| 9 | 挂载：SHOT 循环定步长（同场景同帧⇒同画面） | B、§12.1 | — |
| 10 | `games/neotrix-cards/PLAN.md` + `.logs/2026-09-23-absorption.md` | H | — |
| 11 | `crates/neotrix-game/design/assets.csv` + `STYLE-FORMULA.md` + `tools/shot_audit.sh`（8 场景审计闭环） | A、H | — |

验证：spire 57/57 · game 146/146（134+12）· cards check 过 · 新模块零警告（去重还消 4 旧警告）。

## 三、诚实审计（按症状来，不预判加码）

- **碰撞 cap/CCD**：运动积分分散（ai/ecs_systems），无集中积分点；已知穿透系 SHOT 传送
  伪影（正常游玩不触发）。§6.2 禁预判优化——不碰。下一步：先收敛运动积分点，再加 cap。
- **输入缓冲**：freeze 不吞键已由架构保证（`input.update()` 跑在 `juice.tick()` 之前）；
  6 处 attack 门语义冻结。不建无消费者模块。下一步：cards 迁移时加出牌宽容窗。
- **定步长主循环**：`input` 轮询在 `update()` 内，多步/帧会重复采样——先把 input 移出
  update 再迁主循环（已在 `nt_clock.rs` 文档记录，不碰）。
- **真联机 socket**：协议层已定（Action/Event/view 不变即可替换传输）；`LoopbackBus`
  即双人房间可用态。下一步：ws 传输 + 大厅骨架（godot-multiplayer lobby 对应物）。

## 四、架构重申

`neotrix-game` + `neotrix-spire` **不合并**（本会话已决，见 `.logs/2026-09-23-absorption.md`）：
分离 = 增量 3s、无头单测 0.1s、WASM/服务端复用、crates.io 可发布。本次新增模块分层亦然：
纯逻辑进 spire（nt_tuning/nt_net），引擎表现进 game（nt_juice/nt_clock/ui）。
