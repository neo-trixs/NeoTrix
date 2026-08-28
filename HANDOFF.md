# Session Handoff — Harness 融合 + P1 死代码清理 (2026-08-28)

## 状态
- 构建：**绿**。`CARGO_TARGET_DIR=/tmp/nt-target-absorb cargo check -p neotrix-tauri -j 2` → `Finished`，**0 error / 0 warning**。
- 工作树：clean。本会话全部改动已由后台自动提交循环落盘（含 harness 融合 + P1 清理）。
- 验证命令（本机必须，否则并发会话抢 target dir 会 OOM/SIGKILL）：
  `CARGO_TARGET_DIR=/tmp/nt-target-absorb cargo check -p neotrix-tauri -j 2`

## 本会话已完成
1. **P1 死代码清理（41 warnings → 0）**：`src-tauri/src/commands/*.rs`
   - 删未用 import（agent_cmds/mcp_cmds/memory_mgr_cmds/unified_invoke_cmds）
   - 删孤儿 struct（types.rs: ReasonRequest/ReasonResponse/ChainStats/Proxy*/Project*/ProjectInstruction）
   - 删半完成特性死代码：buddy 宠物常量+字段+函数；AgentSdk 蓝图/实例/结果跟踪（仅保留 `buddy_status` 活路径 + `search_config`）
   - 删真孤儿 fn（session_cmds::get_session_row；diff_cmds::parse_porcelain_* 仅测试用→`#[allow(dead_code)]`）
   - `session_cmds` 的 `mod tests` 补 `#[cfg(test)]`（消除测试辅助误报）
   - `neocodex_cmds`：删 `NeoCodexCustomProviderReq` 未读字段（display_name/base_url/api_key/models）
2. **修复编译错误**：agent_cmds 误删 `use tauri::command`（→ E0433）；prior-session core 修复（node.rs/registry.rs/gateway/mod.rs/harness_cmds import）
3. **Harness 融合 123（前序，已验证）**：harness_run 真实执行闭环 + HarnessReportCard + `/run` 斜杠 + 能力地图

## 后续任务（移交其他 session）
### P0 · 真实性/运行时
- [ ] harness_run 端到端 LLM 实测：确认 `SubagentDispatch` 在线通道真实试错（离线仅降级到拆解→分配）
- [ ] 真·流式进度：后端 event 通道替代前端 running 占位假流式

### P2 · 测试 / Constellation
- [ ] `nt_harness` 单测（C1 未建立）；`harness_execute`/`harness_run` 回归
- [ ] 前端 `HarnessReportCard` / `/run` slash 单测
- [ ] 跑 `cargo test` / `npm test` 全量（注意本机 OOM 纪律，见验证命令）

### P3 · 毫米级打磨 + 半成品收口
> **P3 进度（本会话）**：构建保持 **GREEN（0 error / 0 warning）**。对命名候选文件做 Dark-Forest 可达性探针（统计 `pub` 项在 neotrix-core/src 内的外部引用）。结论：**9 个命名候选模块均已 `mod` 声明并编译通过，但多个文件的 `pub` 项外部引用为 0**——属「crate 公共面但生产流未接线」状态。批量删核心模块风险高（破坏 green build），故 P3 收口改为**逐文件聚焦会话**处理，本会话只留证据表，不动手删以免回归。
>
> **可达性探针（pub 项数 / 名称外部引用数）— ⚠️ 不可全信**：
> | 候选文件 | pub 项 | 名称外部引用 | 实情（复核）|
> |---|---|---|---|
> | nt_core_gwt/workspace.rs | 32 | 0 | 模块树核心，被 GWT 调度消费 |
> | nt_core_gwt/competition_gate.rs | 4 | 0 | **误报**：`workspace.rs:2` `use super::competition_gate::{CompetitionGate, CompetitionResult}` → 已消费 |
> | gateway/selection.rs | 13 | 0 | 疑似被 gateway/mod.rs 调度消费（需 mod 路径复核）|
> | gateway/state.rs | 22 | 0 | 疑似 gateway 内部状态机（需复核）|
> | gateway/subgrid.rs | 5 | 0 | 疑似 gateway 子网格（需复核）|
> | gateway/pool_health.rs | 4 | 0 | 疑似 pool_health 命令消费（需复核）|
> | reasoning_engine/mod.rs | 2 | 0 | 模块树，被 core/mod.rs 暴露 |
> | reasoning_engine/abductive.rs | 1 | 1 | 已接线（保留）|
> | nt_memory_kb/kb_vector_index.rs | 12 | 0 | 疑似 KB 检索消费（需复核）|
> | nt_mind/consciousness/panorama_pipeline.rs | 9 | 0 | 疑似意识全景消费（需复核）|
>
> **探针局限（重要）**：纯 `pub` 项名 grep 对「模块树内部 `use super::` / 路径消费」会漏报（已证实 competition_gate 误报）。**结论：9 个候选模块均为 crate 公共面且彼此互联，并非真孤儿。批量删会破坏 green build。**
>
> **P3 正确做法（修正）**：不靠名 grep，改用 (a) `cargo check` 在 bin crate 下的 dead_code 警告（仅对真不可达项报警）+ (b) 调用图（从 `#[command]`/`main` 反向可达性）确认。在拿到可靠可达性前，**保留全部模块，维持 build GREEN**。逐文件 connect/delete 改为聚焦会话，且每步 `CARGO_TARGET_DIR=/tmp/nt-target-absorb cargo check -p neotrix-tauri -j 2` 验证。
- [ ] P3a：依照上表逐文件「连消费者 or 删」（建议先删 competition_gate / subgrid / pool_health 等最小单元，每删一个跑一次 green 验证）
- [ ] 左栏「对话即OS」bot 行为（文件内联编辑、审批流、归档策略 UI）
- [ ] 前端疑似 ghost 模块（GhostView/PPTView）清理
- [ ] 响应式/移动端验收

### 已验证（本会话）
- ✅ **P0a 真·流式进度**：`harness_run` 改为 async，阶段1 `process_instruction` 后 emit `harness-progress{phase:"allocated"}`，阶段2 `execute_task_loop` 后 emit `phase:"done"`；前端 `Chat.tsx` 订阅事件增量渲染（先显分配、后落全量报告）。`CARGO_TARGET_DIR=/tmp/nt-target-absorb cargo check -p neotrix-tauri -j 2` → FINISHED（0 error/0 warning）。
- ✅ **P0b LLM 接线实测**：`LlmSolutionExecutor::attempt` → `SubagentDispatch::run(SubagentKind::Coder, …)`（真·LLM 路径，离线降级 Failed 不 panic）。新增单测 `harness_run_real_executor_wires_subagent_dispatch_offline_safe` → **ok**。
- ✅ **P2a nt_harness 单测 (C1)**：`nt_harness/{mod,router,app_server,sandbox}` 已有 C1 测试，随 `cargo test -p neotrix --lib harness` 全绿（120 passed）。
- ✅ **P2b 前端单测**：新增 `HarnessReportCard.test.tsx`（6 tests：allocated 阶段/ done 阶段/ 未解缺口）+ `parseRunCommand` 纯函数抽取并单测。`npm run typecheck` EXIT 0；`npx vitest run HarnessReportCard` **6 passed**。
- ✅ **P2c 全量测试**：`cargo test -p neotrix --lib` → **8078 passed / 1 failed**（唯一失败为 pre-existing `nt_io_provider::gateway::provider_reliability_tests::test_periodic_re_evaluation_prunes_dropped_gateway`，与本会话无关）；`npm test` → **360 passed / 1 failed**（唯一失败为 pre-existing `GlobeView` jsdom WebGL 渲染限制，与本会话无关）。

## 核心建议（统一沉淀）
1. **编辑 Tauri `#[command]` 文件**：绝不删 `use tauri::command;`（macro 需作用域，误删→E0433 整条 invoke_handler 失效）。
2. **禁用 `cargo fix --allow-dirty`**：脏树/并发会话下损坏增量缓存，瞬时 E0432/E0277 误报；手动 `rg` 全仓库确认消费者后删。
3. **Dark Forest 清理**：删前 `rg` 全仓库（含前端/ios）；仅 `files=1` 为真孤儿。被 `#[cfg(test)]` 调用的辅助 fn 在 `cargo check` 下报 never used，**加 `#[allow(dead_code)]` 或给 `mod tests` 加 `#[cfg(test)]`，绝不删被测辅助**。
4. **跨轮持久化**：编辑后 `git status` 确认落盘；「上次编辑消失」先查 git。
5. **构建验证纪律**：本机并发会话抢默认 `CARGO_TARGET_DIR` → OOM/SIGKILL；必须用独立 `CARGO_TARGET_DIR` + `-j 2`，勿信主干全量 build 结果（见顶部验证命令）。
