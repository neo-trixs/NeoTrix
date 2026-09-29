# ABSORPTION-ROUND17 — 引擎掏空完成 + 真音频后端（2026-09-23）

> 用户决断：删光旧游戏代码（只留引擎+能力）。本轮交付掏空 + 引擎能力。

## 一、掏空执行（26 文件删，备份 `_archive/purge-round17/`）

- 删除：calendar/codex/cultivation/destiny/evolution/feedback/govern/inventory/
  livelihood/quest/scholar/crystal/ecology/species/worldmap/data/crafting/
  equipment/shop/judgment/entity/assetforge/dialogue/status/states/timeline。
- 拆分：`components.rs` 只留 Position/Velocity/Physics/Sprite/Health/Name/
  PlayerMarker/PlatformMarker/Gravity/Static/Collider/常量；
  `events.rs` 只留 EventBus；`input.rs` 默认表收至 8 中性动作 + 删 `GameMode`；
  `ui.rs` 删 `login_buttons`（锚点保留）。
- game lib 现 15 模块：nt_juice/nt_clock/nt_camera/nt_move/lighting/particles/
  audio/events/state_stack/ecs/ui/input/components/tween/ecs_systems + demo main。
- 测试 141→38（删的是游戏测试，无失败）；新模块零警告。

## 二、真音频后端（引擎能力，F-skill 核心通道）

- 确诊：`audio.play()` 是日志桩 + macroquad `audio` feature 未开（dummy 路径）。
- 实现：`audio.rs` 全量合成 WAV（22050Hz 单声道 16-bit）——Hit 噪声爆点 /
  Hurt 方波下坠 / Select 880 正弦 / Confirm 双音上行 / LevelUp 三音琶音 /
  QuestComplete / Dialogue；`init().await` 预加载 7 音，失败回退静默不断链。
- 修测试 math 两处（i16 对称映射 -32767；相位先采样后推进零起点防 click）。
- 开 feature 后 lib 38 全绿；swords 启动调 `init`，SHOT 无警告（真路径激活，
  有喇叭即出声，CI 静默安全）。

## 三、崩溃翻案（Metal OOM，非代码）

- 同二进制崩一次、重跑即过；swords 同字体双屏全过；日志屡见
  `MTLCommandBufferErrorDomain Code=8 Insufficient Memory`。
- 结论：16G 机并行构建期 GPU 显存压力 abort；审计脚本已加 3 次重试；
  对拍不以字节比对为准（时钟/粒子本就逐跑不同）。

## 四、版图终态

- `neotrix-game`：lib 15 引擎模块 + demo bin（可跑可 wasm）。
- `neotrix-spire`：纯逻辑 76，不动。
- `games/neotrix-swords`：完整武侠游戏（combo/qi/waves + 主循环 + 自有字体，
  2529→2978 字），M1 可玩，SHOT 双屏通过。
- 旧游戏残留：零（`_archive` 有全量备份：strangle-slice4b + swords-rebuild +
  purge-round17）。

## 六、跨会话事故（spire 被搬）

- 并行会话将 `crates/neotrix-spire/` 移入 `crates/_archive/` 并删 members 行，
  未留字条；本会话构建全断。已恢复（目录归位＋成员加回，76 复绿），
  详见 `sessions/handoff-spire-restore-20260923.md`。
- 教训：`purge-round17` 备份曾因 `../../` 落错进 `crates/_archive/`（已归位
  `_archive/`）；未被 git 跟踪的目录，移动即失联——大搬移前先喊一声。

## 五、验证

game lib **40**（含 BGM 两测）· spire **76** · swords **7**，全绿；swords SHOT 审计 2/2；
wasm32 编译通过（Homebrew/rustup 工具链遮挡已定位，按 `build_wasm.sh` 方抓药）。
