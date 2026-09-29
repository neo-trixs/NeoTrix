# MIGRATION-ENGINE-STRANGLE — neotrix-game 掏空路线图（2026-09-23）

> 决议：像素游戏掏空只留引擎，业务迁 `games/neotrix-cards`。绞杀策略：
> lib 认领 → bin `pub use` 垫片 → 双编全绿 → cards 接管 → game 删内容。
> 垫片原理：`pub use neotrix_game::X` 在 bin 根建同名项，全部 `super::X` 零改动通过。

## Slice-1 ✅（本会话）

17 引擎模块双编译：nt_juice/nt_clock/nt_camera/lighting/particles/audio/events/
state_stack/ecs/ui/input/states/components/dialogue/tween/ecs_systems/status。
附带修复：`components::NpcRole` 改 `crate::dialogue::`（双边通路）；
`nt_camera` 前视测试期望修正（死区盒内保持≠回中）+ 前视挂载（读速 3 行）。
验证：lib 35 + bin 123 = 158 全绿，SHOT explore/event 零回归。

## Slice-2 ✅（本会话）：Group-D 叶数据模块（12 文件零依赖，同模式复刻）

calendar/codex/cultivation/destiny/evolution/feedback/govern/inventory/
livelihood/quest/scholar/timeline。lib 29 模块；lib 80 + bin 78 = 158 守恒全绿；
SHOT explore 正常。bin 剩余 `mod` 即 Slice-3/4 范围。

## Slice-3 ✅（本会话）：成簇搬（包裹团/命格团/循环对/生态链，同迁保闭包）

crafting/equipment/shop + crystal/judgment/entity + data↔worldmap +
ecology/species/assetforge（weather 沾 combat 留 Slice-4）。
lib 40 模块；lib 134 + bin 24 = 158 守恒全绿；SHOT explore 正常。
bin 剩 14 `mod` = Slice-4 全集（见下），无遗漏。

## Slice-4a ✅（本会话）：克隆-对拍跑道（未删 game 一行）

15 文件（main/world/render/save/ai/mind/weather/manuals/combat/cards/relics/
powers/events_sts/spire/damage_pipeline）+ `data/`（136K）克隆进 cards；
cards 补 serde/serde_json/ron/log 依赖；cards/main.rs 相对 game 仅 3 行差
（标题 + 2 字体路径）。Slice-1/2/3 垫片使克隆零改动通过。
验证：check 零错 · cards 单测 24 过 · SHOT event 与 game 版像素级对拍通过
（仅时钟/FPS 文本差）。

## Slice-4b ✅（本会话）：对拍 + 删除（用户选 B：跳过抖动深究）

- 对拍：8 场景双跑全出图；字节差=摆拍抖动（时钟文本/未播种 RNG，自跑亦差同量级）；
  event 场景像素级确认。用户决断：不阻塞删除。
- 安全网变更：game/cards 全未被 git 跟踪（`??`）→ `checkout` 恢复链不存在，
  改走 `_archive/strangle-slice4b/` 文件备份 16 文件（14 mod + main + shot_audit.sh）。
- 已删：game 侧 14 `mod` + main 游戏体 + `tools/shot_audit.sh`（随游戏体走）；
  main.rs 重写为 86 行引擎 demo（ECS 往返体 + 点击粒子 + 死区跟随 + HUD，6s 存活冒烟过）。
- 未删（调整）：`data/` 留（lib data.rs 编译期 include，删即断 lib）；
  `assets/` 留（cards `../../../` 引用 + web）；wasm 未切（defer Slice-5，可验证时再动）。
- 验证：game lib 134 + bin 0 全绿零警告；cards 24 过 + SHOT event 正常（删后独立性实证）。


## Slice-5（收尾，未动）

- `data/` 归属：lib data.rs 改读 cards 自有 `data/`（或数据层上收 spire），后删 game `data/`。
- `assets/`（2.3M，字体/图标）搬入 cards 自有，改 `../` 路径；game 侧按需清理（含 web/ 引用）。
- wasm 构建目标切到 cards（需浏览器实测，本会话无验证条件）。
- cards 侧补 SHOT 审计脚本（`tools/shot_audit.sh` 随删，需重建）+ 全 8 场景基线。
- 字体缺字：标题"浴" tofu——事件库后于字体构建导入，`charset.txt` 重生 + `build_font_pixel.py` 重构。
- `plan` 关 M1–M5 对齐（cards PLAN.md 已立）。

## 红线

- 每 slice 独立编译+全测+SHOT（explore/event）三绿才合。
- 不改行为：垫片/搬运零语义差；行为变更（死区/前视/视野门）只发生在72504此前已验收项。
- 同工作区单 watcher；`stash` 兜底再删（2026-09-22 事故教训）。
