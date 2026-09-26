# NeoTrix 全域待解决任务清单（2026-09-26，只列清单不动）

> 本次动作为只读梳理 + 落盘清单。零构建、零改码、零提交，等 owner 逐项点名再下刀。
> 硬规则：`#![forbid(unsafe_code)]`；生产禁 unwrap/expect/panic；新模块 nt_ 前缀；改前读改后重读（R-P16）；
> 他人 worktree 不碰；单 cargo 串行；禁 `cargo clean`；禁 `--all-targets`；验证前 `sleep 2; touch <改过文件>`。

## 0. 世界基线（只读实测）

- 分支 `feat/capability-absorb-20260828`，HEAD `8a11227a refactor(godfile): shield mcp_security.rs 1895 行拆 8 模块`
  - 链：`8a11227a` / `ad59526e fix(tests): keyword 路由兜底` / `ad736137 feat(engine): 九源吸收` / `87479b4a` / `bec4019d` / `58a8ce41` / `aac16ad3` / `2c6e4a4e`
  - 注：交接条件曾写 HEAD=ad736137 则停；实际已前进 2 笔（拆分战役），非回退，故继续只读梳理。
- 脏树：`git status --short | wc -l` = **1087**（M 649 / D 272 / ?? 166）；`git diff HEAD --stat` = **921 files, +56756/-58992**
  - 顶层：neotrix-core 573 / src-tauri 248 / docs 81 / skills 78 / games 38 / crates 30 / scripts 25
  - 大类：neobot 30 / crystal 24（neotrix-core 内）/ desktop 248+1 / game 39（games 38D 全为 games/neotrix-guixu + crates/neotrix-game particles.rs 1M）
  - sessions 0 clean；target 0（gitignored）。
- worktree：本仓 `.worktrees/` 9 个（cap-absorb-fix / drift-3fix-20260926 / fix-3-drifts-20260828 / model-gw-router-fix / nt-act-cleanup-fix / resilience-fix / split-mcp-sec-god / split-mcp-security / typed-memory-fix，均为 detached，他人所建，一律不动）+ 归档仓 12 个（只 list 可见）。
- soul：`neobot core status` + `neotrix dialog core status` 双端 `soul online (1 models, neotrix-crystal at 127.0.0.1:3000/v1, 5~7ms, crystal_version=0.2.0 tools=9)` EXIT:0；晶体 PID 7459 跑 2h07m+，`:3000` LISTEN。
- 盘：`/` 6%；`~/.local/share/opencode/opencode.db` 4.0K（§38 删库已执行，新生 stub）。
- 密钥：`launchctl getenv CRYSTAL_TOKEN/NVAPI_KEY` 均在位 + `~/.config/neotrix/env.sh` 在位（本清单不贴明文）。

## 1. EXECUTION-QUEUE（docs/architecture/EXECUTION-QUEUE.md，全 58 行实读）

| ID | file:line | 状态 | 完成定义 | 去重结论 |
|---|---|---|---|---|
| EQ-05 | EXECUTION-QUEUE.md:24 | 🟨（代码完，待门禁） | check-layer-deps L1×L5 下降＋全量门绿 | 待解决（待 CI/可构建窗口） |
| EQ-07 | EXECUTION-QUEUE.md:26 | 🟨（机械5处完，递延有据） | 同上三组计数下降 | 待解决 |
| EQ-08 | EXECUTION-QUEUE.md:27 | 🟨（SIM-42 allowlist 至 2026-10-31；SIM-43 D-1~D-6 已定；待 Phase 2 窗口） | L1×L2/L5 计数下降 | 待解决 |
| EQ-11 | EXECUTION-QUEUE.md:40 | 🟨（签字单已备，待签） | L5 书面签字 | 待解决（待签） |
| EQ-12 | EXECUTION-QUEUE.md:41 | 🟨（doc-drift 已接 bash -n 过；confidence 待全量门） | 本地可跑 | 待解决 |
| EQ-13 | EXECUTION-QUEUE.md:42 | 🟨（SIM-50 CI 侦察 main 全红；本分支末跑 09-17；待点火） | check --tests 全绿＋新基线 | 待解决 |
| EQ-14 | EXECUTION-QUEUE.md:48 | ⬜裸 | 覆盖率 80＋门绿 | 待解决（P2 未启动，保持 pending） |
| EQ-15 | EXECUTION-QUEUE.md:49 | ⬜裸 | bench 基线满周 | 待解决（P3 未启动） |
| EQ-16 | EXECUTION-QUEUE.md:50 | ⬜裸 | 烘焙逐项关 | 待解决（P3 未启动） |
| EQ-17 | EXECUTION-QUEUE.md:51 | ⬜＋R12 纪律注记 | 首 harness/toxic/对子 | 待解决（条件未到；纪律：eval-run 独立 binary，cargo test 永不花钱） |
| EQ-18 | EXECUTION-QUEUE.md:52 | 🟨（范围已定；checklist=IPC/最小权限/vault/外联/外泄；待主＋报告） | 审计报告＋问题清单 | 待解决（只读审计已备 53 命令清单，见§7；不改码不构建） |
| EQ-19 | EXECUTION-QUEUE.md:53 | 🟨（scope 已定；Architect 09-30 前三选一；待批） | 有主 | 待解决（待批） |
| EQ-20 | EXECUTION-QUEUE.md:54 | ⬜ | 会议纪要＋新版/归档 | 待解决（NT-STD-1.1 前置） |

## 2. 十一 handoff（一句话状态）

1. `sessions/handoff-generative-20260924.md`（597 行，止于§38）—— 生成式觉醒全量交接，§34 护栏/§35 devUrl/§36 App 包/§37 合体已落，§38 删库已执行，§39 缺失（见§4）。
2. `sessions/handoff-loop-20250925.md` —— 采矿 43 轮 + 抖音 62 视频收官，watcher5 持锁，勿双开。
3. `sessions/handoff-generative-awaken-kev-ab.md` —— A/B AgentJev>kev，verify3 54 passed，战略转向不拼参数。
4. `sessions/handoff-spire-restore-20260923.md` —— spire 归档恢复，动 members/搬目录先喊。
5. `sessions/handoff-fiveentity-20260923.md` —— 五实体蓝图 V3 + 8 commits + FINAL GATE 绿，禁他窗改状态格。
6. `sessions/handoff-cleanup-puremac-absorb.md` —— PureMac `nt_act_cleanup` 7 文件落盘未经编译（见§3）。
7. `sessions/handoff-ntbrowse-20260923.md` —— 自研浏览器内核 39 单测 + Lingee 收割，待 check/重编；browser 窗 3 缺参不代修。
8. `sessions/handoff-to-browser-window.md` —— 喊话 browser 窗补 1646/1949/2040 三缺参，不代修。
9. `sessions/handoff-ntcode-tauri-api.md:186-190` —— ntcode 后端 9 commands 完成；已知预存 4 项中 1 已修，残 3（见§6）。
10. `sessions/handoff-s001.md` —— 批量吸收→EQ→M1-M7 关窗，entry 4 行 hunk 待合，禁推远程。
11. `sessions/handoff-s000.md` —— main.rs 巨石拆分起手，AssetForge 双发 131 passed。

## 3. 清理窗 §5（sessions/handoff-cleanup-puremac-absorb.md:45-54）

- 口令：`CARGO_BUILD_JOBS=2 cargo check -p neotrix --lib` / `cargo test -p neotrix --lib -- nt_act_cleanup`（等可构建窗口）。
- 预期风险 4 类（非实测）：clippy pedantic / deny(warnings) / IsTerminal 版本 / 未用 import。
- 验证后：`target/debug/neotrix clean --dry-run --json` 再定提交（仅自有路径）。
- 状态：待解决（未跑，无 cargo 槽争用下排队）。

## 4. 生成式窗 §34–38＋§39（sessions/handoff-generative-20260924.md:550-597）

- §34:550-560 空白屏＋启动护栏 —— 已完成（护栏）；回传待用户（被§35 替代）。
- §35:562-570 devUrl 魔咒 —— 已完成（两件套结论）。
- §36:572-577 NeoBot.app 独立包 —— 构建完成；**待用户目视确认**（App 未运行，不擅自 open）。
- §37:579-589 合体 —— 已完成（dialog 七子命令 + soul online + ARCHITECTURE.md §13）。
- §38:591-597 删库计划 —— **已执行**（opencode.db 4.0K，盘 6%；禁删项 `~/.neobot/neobot.db` / `~/.config/neotrix` / `~/Downloads/Neo` / NeoBot.app 均在）。
- §39 —— **缺失**（597 行止于§38；§39 仅被预告一句“删后新会话用§39提示词开场，token 从 launchctl/env.sh 取不贴明文”）。→ 待补（sessions 只写，不涉代码）。

## 5. 脏树归属（只读划分，不动他人）

- neotrix-core 573（含 crystal 24：`crystal_cmds/auto_crystallizer/skill_crystal/nt_crystal_core` + 11 新建 `nt_archive_train/awaken_loop/db_awakening/eval_loop/hf_bridge/...`）。
- src-tauri 248 + apps/neobot-desktop 1（`apps/` 下仅 neobot-desktop）。
- docs 81 / skills 78（多为 `D skills/assets/icons/*`）/ scripts 25 / crates 30。
- games 38D（games/neotrix-guixu 全删）＋ crates/neotrix-game particles.rs 1M（Emitter 233 行＋5 单测，未提交；命名门禁非 nt_ 前缀；CJK 他窗同文件共存 → 动则 hunk 级过滤；拍板保持未提交等钩子放行）。
- 本清单不认领：他人在途 649M 中的未知部分；合并回主区时必 `git diff` 逐 hunk 确认归属，冲突停下汇报。

## 6. 账本（预存 / parked / flake / blocked，去重）

- T46 stale：`handoff-generative-20260924.md:345` 判 stale（`Result<_,String>` 已扩 100+ 事实标准）；`handoff-fiveentity-20260923.md:37` 存量 4 处排期。→ **关闭不修**。
- tool_loop flake：`handoff-generative-20260924.md:511` `tool_loop_runs_bash_then_done` 偶发（单跑挂/组跑过/重跑过，与 diff 无关）。→ **按 flake 协议重跑 3 次，不修逻辑**。
- DDG 墙：`handoff-generative-20260924.md:441,466-468,480,487,501-502,530` DDG 本机 202 空墙 → 主路 Bing RSS → Wiki 垫底，`parse_ddg` 保留；与 `nt_web.rs:6-8` 一致。→ **已落地，无需修**。
- ntcode 残 3（`handoff-ntcode-tauri-api.md:186-190`）：`skill_evolution.rs:157` HashMap 未导入；`mapper.rs:181` / `knowledge_miner.rs:169` / `self_evolver.rs:152` RewardSource 类型不匹配（其中 skill_loader:660 已修）。→ **待修（点名后下刀）**。
- sidecar :8149 Down（curl 000＋无 LISTEN；史见 67 处引用，09-23 HEALTHY 后今 Down；`report-sidecar.md:1 SUCCESS`）。→ **blocked：须按 gate/watch 门拉起而非直起；wt-train 禁碰训练/采矿/sidecar；等 owner 点名**。
- :1422 无 LISTEN —— 正常（App 包内嵌 dist，不再需要 npm run dev；dev 调试才两件套）。
- guixu 38D —— 他人领地，未定意图。→ **park，不动**。
- 6 parked 嫁接（God-file 战役账本：mock_adapters / orchestrator_v2 / path_metadata-router(act_trade) / shield sandbox mod / stateful_bench，修复备在他窗 worktree 等落地）。→ **park，等他人落地后 patch 嫁接，冲突则停**。
- 长尾约 150 + game 失败簇 ~10 + nt_act 残余（冲突文件内）—— 他人领地/未 triage。→ **park**。
- C 批 3 文件合并 / DeepSeek key（无）/ Lingee 退役 —— 待 owner 决议（Lingee 已退役；DeepSeek 无 key 关闭）。
- T46/D5-next 不适用 / 无代理活测 / 搜索走 Bing·热点直连 —— 已知约束，不列修。

## 7. 四端在位＋审计（只读）

- 二进制：`target/debug/nt-crystal-serve` 52M(09-24 09:38) / `neobot` 16M(21:20) / `neotrix` 148M(21:18) / `neobot-desktop` 48M(20:47) / `NeoBot.app` 同刻（dist 内嵌，identifier ai.neobot.desktop）。
- 版本：neobot/neotrix 0.21.0；`nt-crystal-serve --version` 回 upstream 5 models ok（hotaru/laoxi dead skipped）。
- 探活：`:3000/v1/models` 401（`{"error":"unauthorized"}` 形状，24B，未带 token）；`:8149/health` 000 Down。
- neobot：`crates/neotrix-neobot/src/` lib 20 pub mod，`#![forbid(unsafe_code)]`；`rg TODO|FIXME|todo!|unimplemented` 0 行；desktop 前端 0 行。干净。
- IPC（`apps/neobot-desktop/src/main.rs:26-80` 53 命令）：sys 17 + run 2 + tasks 11 + convo 14 + core 6；插件 dialog/fs/shell/single-instance；`AuditEvent.detail` 永不外发，DTO 脱敏。
- capability：`apps/neobot-desktop/capabilities/default.json` windows[main,settings]，fs 仅 `$HOME/.neobot/**,$TEMP/**`，无 allow-neotrix-commands（比主应用紧）。主 `src-tauri` 为另一包（0.22.0，窗无饰透明＋CSP 宽＋updater），审计区分，EQ-18 只认桌面包。

## 8. 统一待修清单（去重后，点名制）

### P0 服务（blocked/待目视，不擅动）

- [ ] G-01 sidecar :8149 门拉起 —— 前置：owner 点名＋可构建窗口外；完成定义：`/health` 200＋gate done；归属：guardian（`nt_watch.sh`/`nt_reporter.sh` 别杀）。
- [ ] G-02 NeoBot.app 目视 —— 前置：owner 在屏；完成定义：三栏＋雪域＋登录 filter＋模型键目视 OK；归属：owner。
- [ ] G-03 §39 补齐 —— sessions 只写；完成定义：§39 交接提示词落盘（token 只写来源不贴明文）。✅ 可首点名（零风险）。

### P1 编译·单测（点名后单 cargo 串行）

- [ ] G-04 nt_act_cleanup 编译验证 —— 口令见§3；完成定义：`check` 零错零警告＋过滤单测绿；归属：wt-cleanup。
- [ ] G-05 ntcode 残 3 —— `skill_evolution.rs:157` / `mapper.rs:181` / `knowledge_miner.rs:169` / `self_evolver.rs:152`；完成定义：对应模块过滤单测绿；归属：点名窗。
- [ ] G-06 Emitter —— 保持未提交（等钩子放行）；完成定义：钩子放行后 hunk 级过滤提交。归属：engine（park）。
- [ ] G-07 tool_loop flake —— 完成定义：同命令连跑 3 次绿；不修逻辑。
- [ ] G-08 T46 —— 关闭（stale，不修）。

### P2 队列·文档（只改状态格＋证据行）

- [ ] G-09 EQ-05/07/08/11/12/13/18/19 状态推进 —— 只改状态格＋证据行；条件任务 EQ-14/15/16/17/20 保持 pending；归属：wt-queue。
- [ ] G-10 C 批 3 文件合并 —— 待 owner 决议。
- [ ] G-11 DeepSeek/Lingee —— Lingee 退役已定；DeepSeek 无 key 保持关闭。

### PARK（本次不动）

- P-01 guixu 38D / P-02 6 parked 嫁接 / P-03 长尾 150＋game 10＋nt_act 残余 / P-04 desktop 高风险改动 / P-05 训练·采矿·浏览器（Chrome 一律 blocked 等用户）。

## 9. 本次动作记录

- 只读侦察 3 lane（队列/脏树/四端）＋主区复核 EXECUTION-QUEUE.md 全文；零 cargo、零改码、零提交。
- 落盘本文件 1 个（`?? sessions/handoff-global-todo-20260926.md`），待重读验证。
- 下一步：等 owner 逐项点名（建议首点 G-03 §39 → G-04 cleanup → G-05 ntcode → G-09 EQ 状态格）。
