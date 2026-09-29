# Handoff s000 — main.rs 战斗簇拆分进行中（3 个重复定义待删）

> **状态更新（同窗口 2026-09-21 继续执行后）：§5 的 1–5 已全部完成 — main 侧 3 个重复定义已删，6 处孤儿 import 已清（含 HashMap/combat 包/GameState），C2 save/load 已迁入 save.rs，`cargo check -p neotrix-game` 0 error，`cargo test` 115 passed，main.rs 2863→2213 行。下文是原始交接记录备查。**
>
> **第二轮（并行收网后）：C3 warnings 57→0（修 Trade 真 bug + calm/weather/memorial 接线 + 元素/rand 统一 + 删 legacy Destiny/xp_for_next/hp_mult/quest kill-talk）；三路子代理（纪年HUD/混沌归墟4关/驿道里程计费）+ Codex 分页 + 双结局（重铸/镇压）全部落地；`cargo check` 0 error 0 warning、`cargo test` 116 passed、wasm32 target 0 error；main.rs 2213→~2240 行。遗留：wasm-pack 本体 x86_64 无 Rosetta（改用 rustup cargo 直验 wasm32）；开天动画/教派大战/战后对话等提案级 L3 未动。**
>
> **第三轮（2026-09-21 又继续）：开天动画（标题三幕淡入，render `&mut self`）+ 鼠王降格门将（守墟鼠王 8 折/scale1.2）+ 教派大战（`sect_war`/`war_accord_hit` 纯函数 + 3 秒门控接线 + 120s 冷静期）+ 战后对话（结局回响）+ 修 `at_final` 写死 id2→表长；`cargo check` 0 error 0 warning、`cargo test` 119 passed。">注意：brew cargo 无 wasm sysroot，验 wasm 必须 `PATH="$HOME/.cargo/bin:$PATH"` 前缀。**
>
> **第四轮：PWA（manifest/sw.js/icon.svg+index.html 接线）+ WASM 包级 profile（opt-z）+ E2E3 修战斗 Pop 打架（`CombatModeState::on_tick` 不再 Pop，进出栈归 update_combat/end_combat）+ KB 桥（codex 新增种并入 species.list，生态位空=不自然定居）+ 美术音乐教程判为文档（大纲已给用户，未落文件）；`cargo check` 0 error 0 warning、`cargo test` 119 passed。**
>
> **第五轮（最优解结案）：①依赖特性维持不动——core 在用 serde_json::Value（别家地盘，preserve_order 语义不可动），macroquad 全特性不可拆（窗口/输入/字体链），opt-z 包级 profile 即确定性收益；②KB 桥维持保守——kb_codex.json 缺席，桥休眠零风险，若 KB 供种 later 再带 habitat 映射；③PWA 验完——node --check 通过，http.server 实测 200 + MIME 全对（manifest+json/javascript/svg+xml），index 接线 3 处；真机离线章需人工点 Chrome。**
>
> **第六轮（app 端）：`cargo build -p neotrix-game` 出包 target/debug/neotrix-game（8.5MB），后台冒烟 8 秒 banner 正常、窗口存活、无报错。双击即玩；另有 web/ PWA 与 wasm32 可选。release 包未打（LTO 耗时，需时再打）。**
>
> **第七轮（地图正式化）：worldmap.rs 正典配色 base_color + 格抖动 cell_jitter + 区域标注锚点 region_label_anchor（含测试）；ui.rs wrap_cjk + render_pill_centered（含测试）；render.rs 世界图重写（地形纹理/地界线/驿道路面/POI 形标/九州名牌/悬停面板换行）+ 小地图共用配色；`cargo check` 0 error 0 warning、`cargo test` 125 passed；debug 包已重打（需重启游戏看新图）。**
>
> **第八轮（真 3D + AssetForge）：用户明确 3D=三维视角 + 文字→素材→运转引擎。落地：① 3D 立体九州 diorama（M 开图后 V 切换）：9×9 地块按地形起伏立方体 + 锻造纹理贴图 + POI 石柱 + 驿道金线 + 当前关卡白柱，相机缓环绕；② AssetForge v0：data/assets.ron（9 资产）+ assetforge.rs（manifest/程序化像素/terrain_asset/预热缓存/file 直引通道预留 AI 生图）+ world.rs 接线；`cargo check` 0 error 0 warning、`cargo test` 128 passed；debug 包重打并重启。生文/生音乐/生视频走同 manifest 扩展，下轮。**
>
> **第九轮（外部像素素材）：学完 PixelSRPG-Forge（235★，版权自负声明）+ LiGameAcademy 美术导航。落地：vendor 9 文件 42KB（cave 洞穴场景 + 7 怪物 Idle + SOURCES.md 版权声明）→ AssetForge file 直引（asset_root 四候选：.app Resources/assets 打包 + target/debug/assets 软链）→ 实体精灵纹理化（模板 tint：火史莱姆红染/混沌金辉/鼠王金染）+ 名牌 pill + 洞穴关 cave 背景；`cargo check` 0 error 0 warning、`cargo test` 131 passed；桌面实包 + Web 真包（993KB，gl.js 正道）双发，浏览器已打开。注意：本轮发现 Edit 工具写 render.rs 间歇不落盘（md5 不动），后半全切 bash 直写+字节验；另有 5 窗口同跑锁竞争，构建慢属正常。**

## 1. 会话标识

- 窗口：s000
- 分支：`feat/capability-absorb-20260828`（`git branch --show-current` 实测）
- 交接时间：2026-09-21
- 工作区：`/Users/neo/Downloads/neotrix`；game crate 整目录未跟踪（`?? crates/neotrix-game/`），`git diff --stat` 看不到它，核对用 `git status --short` + 直接读文件

## 2. 目标（一句话）

> 把 `crates/neotrix-game/src/main.rs` 巨石文件按 combat/render/world/save 簇拆成小模块，消掉 combat.rs 迁移后 main 侧残留的重复定义，恢复 `cargo check -p neotrix-game` 零错误。

## 3. 已完成

- [x] `try_attack` / `start_combat_with_ecs` / `geo_mult` / `player_strike_mult` / `enemy_strike_mult` / `fortune_of` 共 6 个函数已从 main.rs 删除，combat.rs 侧 `pub` 版是唯一定义（grep 已核：main.rs 无 `fn start_combat_with_ecs|geo_mult|fortune_of|player_strike_mult|enemy_strike_mult|try_attack`，combat.rs:303/323/372/402/721/740 有 `pub fn` 版）
- [x] render.rs 拆分闭环：main.rs 里 `fn render_` 计数已为 0，15 个渲染函数只在 render.rs
- [x] GameWorld 跨模块字段已改为 `pub(crate)`（combat.rs 的 `impl GameWorld` 能访问字段的前提）
- [x] 分支/状态已核对：`git branch --show-current` = `feat/capability-absorb-20260828`，`git status --short` 已跑（本窗口只动了 game crate；其他 M/D 改动是别的窗口/历史遗留，勿碰）

## 4. 正在改的文件（关键！逐个列）

| 文件完整路径 | 改到什么程度 | 是否可独立提交 |
|---|---|---|
| `/Users/neo/Downloads/neotrix/crates/neotrix-game/src/main.rs` | 已删 6 个战斗函数；**还剩 3 个重复定义未删**：`fn update_combat`（约1898行）、`fn enemy_turn`（约1947行）、`fn end_combat`（约1996行）；另有 3 个未使用 import 待清（`InputState`:72行、`StateContext`:76行、`NpcData`:151行）| 否，删完 3 个重复定义并通过 check 前不可提交 |
| `/Users/neo/Downloads/neotrix/crates/neotrix-game/src/combat.rs` | 已收齐 9 个 `pub fn`（try_attack:303、start_combat_with_ecs:323、player_strike_mult:372、enemy_strike_mult:402、update_combat:428、enemy_turn:477、end_combat:526、geo_mult:721、fortune_of:740）；combat.rs 侧**不再改动**，只等 main 侧删重复 | 否，与 main.rs 是同一原子改动 |
| `/Users/neo/Downloads/neotrix/crates/neotrix-game/src/render.rs` | 15 个渲染函数已迁入，main 侧已清零；本窗口内视为完成，接手时只需随 check 连带验证 | 是（但 game crate 整体 untracked，随整包提交即可） |
| `/Users/neo/Downloads/neotrix/crates/neotrix-game/src/world.rs` | GameWorld struct + new() 已迁入；本窗口内视为完成，随 check 连带验证 | 同上 |

## 5. 下一步（按优先级排序）

1. 在 `/Users/neo/Downloads/neotrix/crates/neotrix-game/src/main.rs` 删除最后 3 个重复定义（`update_combat` 约1898行起、`enemy_turn` 约1947行起、`end_combat` 约1996行起，删到其各自闭合 `}` 为止，保留后随的非战斗函数如 `close_dialogue` 等），注意 combat.rs:428/477/526 的 `pub` 版是保留对象、main.rs 的私有版是删除对象，别删反
2. 跑 `cargo check -p neotrix-game`：预期 E0592/E0034 全部消失；若还有错，优先看是否删多/删少了花括号
3. 清 3 个 unused import（main.rs:72 `InputState`、:76 `StateContext`、:151 `NpcData`），确认 combat.rs 内确无引用后再删（跨模块 `use ui::` 函数内路径不受影响）
4. 跑 `cargo test -p neotrix-game --bin neotrix-game`，预期 113 passed（此前基线）
5. C2：把 main.rs:1315 `save_game` / :1401 `load_game` 迁入 `/Users/neo/Downloads/neotrix/crates/neotrix-game/src/save.rs`（`save.rs` 本窗口尚未动过，上述 check/test 全绿后再动手）

## 6. 阻塞点

- 无新增阻塞。提醒：此前有用户终端后台 `cargo check` 占住 `target/debug/.cargo-lock` 导致构建卡死的前科，若 cargo 无响应先 `pkill -9 -f cargo; pkill -9 -f rustc; sleep 2; rm -f target/debug/.cargo-lock`。本窗口最后一次 `cargo check` 报的是 E0592×8（8 个重复定义）+ E0034×9（调用歧义，删完重复即消）+ 3 个 unused import warning；此后又删了 5 个重复，未再跑 cargo 验证。

## 7. 给接手会话的话

- 恢复命令：先读本文件，再跑 `git status --short`（确认 game crate 仍是 `??` 未跟踪态）+ `grep -n "fn update_combat\|fn enemy_turn\|fn end_combat" crates/neotrix-game/src/main.rs`（若还有输出，说明 §5.1 没做完）
- 禁止事项：不要 `git add/commit`（game crate 未跟踪 + 工作区有别的窗口的 M/D 改动，提交会误收）；不要跑 `cargo check --all-targets`，只用 `cargo check -p neotrix-game`；不要动 `render.rs`/`world.rs`（已闭环）
- 风险提示：`main.rs` 与 `combat.rs` 现在是"一删即好"的中间态——combat.rs 一个字符都别改，只删 main.rs 侧；`end_combat` 体量大（含模板结算/loot/XP/poison buff），删除时用行号锚点逐段确认，别整段懵删；工作区其他 M/D 文件（neotrix-core CLI 删除迁移等）与本任务无关，勿碰
