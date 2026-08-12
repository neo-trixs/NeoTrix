# NT-IO 代码辅助 UI 体验增强 — 进度

## 目标
依据 2025 学术论文与主流 agentic 工具研究，系统性增强代码辅助（Code Assist）体验：
计划-执行分离、上下文纪律、diff 审阅、任务看板、live preview、@ 上下文应援。

## 研究综述（已完成的搜索）

### 1. 计划-执行分离（Plan & Execute）
- **Paper**: "Plan-and-Solve Prompting" (Wang et al., ACL 2023) — 两步分解优于直接 CoT。
- **Paper**: "Tree of Thoughts" (Yao et al.) — 计划阶段显式化，执行前有审查点。
- **Paper**: "On the Planning Abilities of LLMs" (Valmeekam et al., arXiv 2305.15771) — 单步计划生成弱，需执行期验证（计划 + 执行反馈闭环）。
- **工程对标**: Claude Code Plan Mode（`/plan` 只读阶段生成计划 + `View Plan` 明细）、Codex Agent Mode 双层；Nimbalyst/Nimbus 把 plan 渲染为可视化看板与 diff 预算。
- > 结论: 计划模式是「只读 + 计划明细可见 + 显式批准」三位一体，已有 plan 模式开关但缺计划明细 UI 展示与批准流。

### 2. 上下文工程（Context Engineering）
- **Paper**: "Context Length Alone Hurts LLM Performance Despite Perfect Retrieval" (Du et al., Findings of EMNLP 2025) — 即使完美检索，输入变长性能仍下降 13.9%–85%；无关 token 换成空白/全 masked 仍劣化。**最佳实践: "recite retrieved evidence before solving"**（长任务→短上下文重述法）。
- **Paper**: "Unable to Forget: Proactive Interference Reveals Working Memory Limits in LLMs Beyond Context Length" (Wang & Sun, arxiv 2506.08184) — 前摄干扰随更新累积呈对数线性衰退，加大 context window 无效。
- **Paper**: "Exploring Working Memory Capacity in LLMs" (Hong et al., IJCNLP 2025) — 单一输入内任务数/难度挤压工作记忆；cognitive marker 可减负。
- **Paper**: "Sculptor: Empowering LLMs with Cognitive Agency via Active Context Management" (arXiv 2508.04664) — 给模型 fragment/summary/hide/restore/search 工具主动雕塑工作记忆。
- > 结论: 上下文纪律的工程抓手 = (a) 展示 context_usage 仪表（已有只读轮询/compact 提示），(b) 提供 compact 手动/自动触发，(c) @-mention 显式注入精准上下文（避免用户先贴大段代码）。

### 3. Agentic UX（任务看板 + 可视化 + 内联交互）
- **Nimbalyst** (2026): 会话 kanban、内联 diff review、多会话并行、任务/跟踪板与 agent 同步。
- **0xc00010ff/proq**: 每个 task 一个 git worktree + 独立 agent + live preview；板即工作区。
- **Spedy**: ticket 内嵌 live preview 容器，元素标注（annotate）+ agent 见预览上下文直接改代码。
- **beyondworks/UI-Inspector**: MCP preview + DOM 元素点选，Claude 拿到精确 sourceLocation(file:line) 直接改。
- > 结论: agentic UX 的四大支柱 = 任务看板、live preview、内联 diff 审阅、画布/元素注释。前端已具备 diff 面板(GitPanel)与 sidechat，缺任务看板与 live preview 入口。

### 4. Diff 审阅增强（Diff Critique）
- **Nimbalyst 内联 diff**: 每文件内联 approve/reject，行内可视冲突。
- **Open Design Studio**: `apply_patch` 多文件原子化 + 智能上下文 elide（旧/大 tool result 自动省）。
- > 结论: GitPanel 已具备合理 accept/reject + hunk 明细 + 变更计数徽章，增强方向 = 审阅前置（提交前强制过 diff）+ 内联到 chat 流。

## 后端 API 缺口盘点（以现有命令面为准）

| 能力 | 现有后端命令 | 前端使用 | 缺口 |
|------|------|------|------|
| 计划-执行 | permission_mode (含 plan)、exec_plan 逻辑在 core 侧 | PermissionModeSelector 已接 | ①计划明细 UI（批准/拒绝按钮、diff 预算） |
| 上下文 | agent_status → context_usage；compactSession | 已接 /compact + 自动提示 | ②上下文占用在消息列表实时显示（已有轮询，未渲染到每条消息） |
| 内联审阅 | git diff → getDiff；applyDiff(accept/reject) | GitPanel 已接 | ③审阅前置流（提交前强制过 GitPanel） |
| 任务看板 | 会话/行动(cowork sessions) | CoworkView 行动列表 | ④chat 流内 TODO 追踪 + 看板视图 |
| live preview | — | — | ⑤preview 容器（web-server ACP/浏览器源） |
| @-mention 应援 | searchFiles | Chat.tsx 已接 @mention | 完整；增强选项：@ 结果带 context 预算提示 |

## 批次计划（每批独立验证）

- [x] **批次 1**: Plan Mode 批准流 — plan 模式下流完成自动进入批准待定态（onDone 检测），输入区上方出现批准条：批准并执行（切 accept_edits + 同轮携带计划原文继续）/ 拒绝 / 取消；plan 激活时显示只读横幅。验证: tsc + vitest 42 全过。
- [x] **批次 2**: Context Bar — ch-top 下方常驻上下文占用 gauge 条（color 分级，>80% 红色 + 一键 /compact），复用 contextPct 轮询。验证: tsc + vitest 通过。
- [x] **批次 3**: 提交前 diff 审阅关卡 — GitPanel commit 前统计未暂存（未审阅）文件数，>0 时弹确认模态（知情关卡）；操作区顶部加未审阅徽章。验证: tsc + vitest 通过。
- [x] **批次 4**: 消息内 TODO 追踪 — 新增 TaskList 组件，解析 assistant markdown `- [ ]` checklist 为可勾选任务组（localStorage 按消息 id 持久化，进度计数）；chat 消息流接入。验证: tsc + vitest 47 全过（含 5 个新用例）。
- [x] **批次 5**: Live Preview 面板 — 新增 `LivePreview.tsx` 模态面板（URL 编辑 + 端口快选 + 自动探测 HEAD localhost + iframe 内嵌 + 刷新 + 外部打开）；接入 PanelId `'preview'`/侧栏「预览」入口；tauri CSP 放宽 frame-src/connect-src 至 localhost:*。验证: JSON 合法、tsc + vitest 通过（Sidebar 用例更新 6 面板）。
- [x] **批次 6**: @-mention 上下文预算 — @ 选择后读取内容估算行数/token，输入区上方渲染引用 chip + 显式总预算 ≈N tok；发送或 /clear 清空。验证: tsc + vitest 47 全过。

## 总结
6 个批次全部落地（先研究后设计再实现），后端零改动（纯前端 UI 层），前端 `tsc --noEmit` + 47 测试全绿。设计依据 2025 论文（Context Length Alone / PI-LLM / Sculptor / Plan-and-Solve / Valmeekam）与主流 agentic 工具（Claude Code plan、Nimbalyst、proq、UI-Inspector）。

## 批次 7：多维对标盘点 + 逐标签实测 → 缺陷修复（当前会话）

### 对标矩阵（Claude Code / Codex / Osaurus / Cursor / Windsurf / Gemini CLI）
- **快捷键**: Osaurus ⌘K 清聊天、Claude Code ⌘N 新建 / /clear、Windsurf 极简；本应用此前 ⌘1-5 面板 + ⌘6 电脑，preview 无快捷键、顺序与侧栏不一致。
- **panel 门禁**: Claude Code / Cursor 面板均按上下文可用（不可用时禁能，不假装可点）；本应用面板区被 `activeView==='chat'` 门禁但侧栏按钮全视图可点 → 死按钮。
- **工具调用折行**: Claude Code / Codex 工具失败即显失败原因摘要；本应用 result 已含 `TOOL_ERROR: …` 但卡片只显示 ✗。
- **命令面**: Claude Code 斜杠菜单 hover 同步选中态；Windsurf 聚焦当前输入；本应用 SlashMenu hover 为 no-op 注释。
- **API 契约**: Cursor 前端 IPC 参数键与后端严格对齐；本应用 `cowork.ts` 整文件 camelCase，违反 `client.ts` snake_case 约定。

### 逐标签实测结论与证据（失败行号）
| 标签/区域 | 实测结果 | 证据 |
|---|---|---|
| 协同（CoworkView） | **整体 broken**：`cowork_*` 命令 invoke 键全不匹配 → 永不加载 | `cowork.ts` camelCase vs `cowork_cmds.rs` snake_case（worker_path/session_id） |
| Chat 工具调用 | 0 时长完成事件被永久误判「执行中…」 | `nt_io_agent_loop.rs:347` 审批拒绝 `duration_ms:0,success:false`；`nt_io_neocodex.rs:1691` exit_code=0 `duration_ms:0`；前端 `ToolCallCard.tsx` 原 heuristic |
| 侧栏功能面板 | 非 chat 视图按钮点了无任何面板 → 死按钮 | `Chat.tsx:951` 面板区 `activeView==='chat'` 门禁 vs `Sidebar.tsx` 按钮无 gating |
| 快捷键 | ⌘1-5 与侧栏 6 面板顺序错位，preview 无快捷键 | `Chat.tsx` 原 handler `['git','tasks','cost',...]` vs 侧栏 `[git,cost,tasks,...]` |
| SlashMenu | hover 注释 `i()/* no-op */`，选中态不跟随鼠标 | `SlashMenu.tsx:46` |
| 右栏 Artifact（RightBar） | **非缺陷**：renderMd 先转义再加标签 + 围栏先提取，无 XSS 注入面（核实撤销告警） | `RightBar.tsx:60-81` |
| project_tree 导出 | **非缺陷**：仅一处 export（撤销重复导出的误报） | `neocodex.ts:156` |

### 修复落盘（tsc + vitest 49 全绿验证）
1. `cowork.ts` 全部 IPC 键转 snake_case → 协同标签恢复可加载。`coworkStart` 改传 `workspace_path`，其余 `session_id`。
2. `ToolCallCard`: `isRunning` 仅 `duration_ms == null`（全事件均为完成事件），失败态折行透出 result 首行失败原因；新增 0 时长成功/失败两个测试用例。
3. `Sidebar` 面板按钮：非 chat 视图 `aria-disabled + opacity-40 + cursor-not-allowed`，标题提示「仅在对话视图可用」。
4. 快捷键对齐：⌘1-6 按侧栏顺序 `[git,cost,tasks,timeline,sidechat,preview]`，⌘7 切电脑视图，⌘N 新建对话；`/help` 文案同步。
5. `SlashMenu` 新增 `onHover` 上抛选中索引，Chat 两处渲染接线 → 悬停高亮跟随鼠标。

## 批次 8：建议执行（⌘K / 截图内存化 / 格式去熵）

### 对标依据
- **⌘K 命令面板**: Osaurus ⌘K 清聊天、Claude Code 全局命令菜单、Cursor 命令面板 → 全套均以全局 ⌘K 浓缩高频动作（新建/清除/压缩/切换视图/切权限模式/设置）到一处，不依赖输入焦点。
- **截图内存化**: 既实现为「前端 readFile 二次往返 + /tmp 堆积」，对标 Claude Code Computer Use 内联截图（数据直接传回，无临时文件参与前端生命周期）。
- **格式 tabs**: 6 个格式仅 Raw/Rendered 有真实差异，其余 4 个（WeChat/Zhihu/Juejin/Web）为装饰标签 → 对标 Cursor 预览只给真实可切换格式，去除假选项。

### 落盘
1. **⌘K 命令面板**：新组件 `CommandPalette.tsx`（模糊过滤 + 上下导航 + Enter/悬停同步选中 + backdrop 关闭）；全局 keydown 接管 `⌘K`（不依赖输入焦点），Esc 层级提前 panel/slash/settings；9 条命令全部复用既有 handler（单一事实源）；`/help` 文案含 ⌘K。新增 4 条单测。
2. **截图内存化**：`computer_screenshot_and_save(path: Option<String>)` — 无 path 时后端捕获→base64→自删临时文件返回 `data_base64`（`ScreenCapture` 增 `Option` 字段，serde skip None，纯路径语义不破坏）；前端 `ComputerUse.capture()` 单次 invoke + 直解 dataURL，删除 `@tauri-apps/plugin-fs` readFile/remove 往返与 /tmp 泄漏。
3. **格式 tabs 去熵**：`PREVIEW_FORMATS` 收敛为 Raw/Rendered 两态。

### 验证
- 前端 `tsc --noEmit` 通过；vitest **53/53** 全绿（新增 CommandPalette 4 用例）。
- 后端 `cargo check -p neotrix-tauri` 通过（首次 build 报 nt_core_mcp.rs 未闭合分隔符为缓存瞬态，二次 build 干净 —— R-P9 防缓存毒化实证）。

## 约束
- R-P1 零 unsafe；前后端 TS/TSX 无 lint 增量过阈值。
- 每批改完运行对应前端测试 + `cargo check`/`cargo test`(后端未动则跳过)。
- 进度真实回写本文件，验证通过才勾选批次。