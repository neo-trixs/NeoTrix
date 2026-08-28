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
> **P3a 结论（可靠探针，已修正）**：用**正确**的 rg 探针（去掉 zsh `!` glob 历史展开陷阱，改 `grep -v` 排除自身文件）重测 9 个候选模块——**全部有外部消费者，非孤儿**：
> | 候选文件 | pub 项 | 外部引用 | 判定 |
> |---|---|---|---|
> | workspace.rs | 32 | 22 | 已连（GWT 调度）|
> | competition_gate.rs | 4 | 4 | 已连（workspace `use super::`）|
> | gateway/selection.rs | 13 | 13 | 已连（网关调度）|
> | gateway/state.rs | 22 | 20 | 已连 |
> | gateway/subgrid.rs | 5 | 4 | 已连 |
> | gateway/pool_health.rs | 4 | 4 | 已连 |
> | reasoning_engine/mod.rs | 2 | 0 | 仅 2 个 pub 项未用（模块树已声明，低风险）|
> | reasoning_engine/abductive.rs | 1 | 1 | 已连 |
> | kb_vector_index.rs | 12 | 8 | 已连（KB 检索）|
> | panorama_pipeline.rs | 9 | 8 | 已连（意识全景）|
>
> **关键教训**：上一轮探针的「0 引用」是 **zsh 把 `--glob '!...'` 当历史展开**导致 rg 搜空——纯工具 bug，不是真孤儿。Dark-Forest 判定前必须排除 shell 元字符陷阱。**结论：18 个 prior-session 半完成重构均「已连消费者」，无需删除，green build 保护成立。** 仅 `reasoning_engine/mod.rs` 2 个 pub 项可后续清理（C5 微调，非阻塞）。
> **P3c 结论**：`GhostView`/`PPTView` **根本不存在**为组件（仅 `design-tokens.css` 的 `.btn-ghost` 合法按钮样式 + `index.css` ghost icon 用语）；宽泛「未用组件扫描」因 import 路径正则误报（如 Sidebar/SlashMenu 被判未用）而不可信，**无安全可删项**。
- [x] P3a：18 核心半完成重构可靠可达性 → 全部已连消费者，无删除（green 保护）
- [x] P3c：前端 ghost 模块（GhostView/PPTView 不存在；扫描不可信，无删）
- [x] P3b（部分）：左栏 bot「审批流」可见性面板 `ApprovalPanel` 已接 `harness_approval_list`（只读，挂载即拉取，双方视图均挂载）；`npm run typecheck` + `ApprovalPanel.test.tsx`(2) 通过
- [ ] P3b（待后端）：文件内联编辑（`read_file`/`write_file` 命令已存在，需前端编辑器组件）、审批交互（需 `harness_approval_approve/reject` 端点，当前无）、归档策略 UI（需 `session_archive` 端点，当前无）
- [ ] 响应式/移动端验收

### 已验证（本会话续）
- ✅ **P3a 可靠可达性**：修正 zsh `!` glob 陷阱后探针显示 9 候选模块全部有外部消费者（22/32、4/4、13/13、20/22、4/5、4/4、1/1、8/12、8/9）→ 非孤儿，无需删除，green build 保护。
- ✅ **P3c**：GhostView/PPTView 组件不存在；宽泛未用组件扫描因 import 正则误报不可用 → 无安全删除项。
- ✅ **P3b 审批流面板**：新增 `ApprovalPanel.tsx`（只读展示 `harness_approval_list` 待审批队列 + 待处理计数），`Chat.tsx` 双方视图（chat/agent）挂载，`onMount` 拉取。`HarnessApproval` 类型接入 `api/harness.ts`。`npm run typecheck` EXIT 0；`ApprovalPanel.test.tsx` **2 passed**。

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
6. **shell 元字符陷阱（Dark-Forest 探针）**：zsh 会把 `--glob '!pattern'` 当历史展开，导致 `rg` 搜空、可达性计数全 0（假孤儿）。排除自身文件用 `rg ... | grep -v "$file"` 而非 `!` glob；探针结论须人工抽样复核（已证实 competition_gate 被 workspace `use super::` 消费却报 0）。
