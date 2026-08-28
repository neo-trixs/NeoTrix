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
- [ ] 18 个 prior-session 核心半完成重构（selection.rs/provider_pool.rs/factory.rs/workspace.rs/reasoning_engine/nt_io_agent/nt_core_consciousness_tree/kb_vector_index/panorama_pipeline 等）：逐一确认「连消费者 or 删」（Dark Forest），行为验证
- [ ] 左栏「对话即OS」bot 行为（文件内联编辑、审批流、归档策略 UI）
- [ ] 前端疑似 ghost 模块（GhostView/PPTView）清理
- [ ] 响应式/移动端验收

## 核心建议（统一沉淀）
1. **编辑 Tauri `#[command]` 文件**：绝不删 `use tauri::command;`（macro 需作用域，误删→E0433 整条 invoke_handler 失效）。
2. **禁用 `cargo fix --allow-dirty`**：脏树/并发会话下损坏增量缓存，瞬时 E0432/E0277 误报；手动 `rg` 全仓库确认消费者后删。
3. **Dark Forest 清理**：删前 `rg` 全仓库（含前端/ios）；仅 `files=1` 为真孤儿。被 `#[cfg(test)]` 调用的辅助 fn 在 `cargo check` 下报 never used，**加 `#[allow(dead_code)]` 或给 `mod tests` 加 `#[cfg(test)]`，绝不删被测辅助**。
4. **跨轮持久化**：编辑后 `git status` 确认落盘；「上次编辑消失」先查 git。
5. **构建验证纪律**：本机并发会话抢默认 `CARGO_TARGET_DIR` → OOM/SIGKILL；必须用独立 `CARGO_TARGET_DIR` + `-j 2`，勿信主干全量 build 结果（见顶部验证命令）。
