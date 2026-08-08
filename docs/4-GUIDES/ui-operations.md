# NeoTrix Desktop — UI 功能操作手册

> 版本: 0.18.0 · 更新: 2026-08-08
> 覆盖范围: neocodex-frontend 全部界面组件 + 后端命令对接
> 本文档基于代码盘点（`neocodex-frontend/src/` + `src-tauri/src/commands/`），每个界面标注实际功能与操作步骤。

---

## 目录

1. [界面总览](#1-界面总览)
2. [对话视图（Chat）](#2-对话视图chat)
3. [侧边栏（Sidebar）](#3-侧边栏sidebar)
4. [右栏 Artifact Pane + 文件树（RightBar）](#4-右栏-artifact-pane--文件树rightbar)
5. [协同视图（CoworkView）](#5-协同视图coworkview)
6. [模型提供商选择（ProviderSelector）](#6-模型提供商选择providerselector)
7. [权限模式选择（PermissionModeSelector）](#7-权限模式选择permissionmodeselector)
8. [工具调用卡片（ToolCallCard）](#8-工具调用卡片toolcallcard)
9. [文件预览与标注（FilePreview / AnnotatedImage）](#9-文件预览与标注filepreview--annotatedimage)
10. [Markdown 渲染（Markdown）](#10-markdown-渲染markdown)
11. [Git 面板（GitPanel）](#11-git-面板gitpanel)
12. [项目视图（ProjectView）](#12-项目视图projectview)
13. [成本 / Token 看板（CostDashboard）](#13-成本--token-看板costdashboard)
14. [插件市场（PluginMarketplace）](#14-插件市场pluginmarketplace)
15. [定时任务（ScheduledTasks）](#15-定时任务scheduledtasks)
16. [Checkpoint 时间线（CheckpointTimeline）](#16-checkpoint-时间线checkpointtimeline)
17. [侧聊（SideChat）](#17-侧聊sidechat)
18. [电脑控制（ComputerUse）](#18-电脑控制computeruse)
19. [窗口控制（TrafficLights）](#19-窗口控制trafficlights)
20. [后端命令域总览](#20-后端命令域总览)
```

---

## 1. 界面总览

NeoTrix Desktop 是单页应用（SolidJS + Tauri v2），主路由 `/` 与 `/chat` 均渲染 `Chat` 组件。

```
┌─────────────┬──────────────────────────────┬──────────────┐
│  Sidebar    │  主区域（Chat / Cowork）      │  RightBar    │
│  (会话管理)  │  - 消息流 / 输入区 / 工具栏   │  (Artifact   │
│  - 视图切换  │  - 或协同看板                 │   + 文件树)  │
│  - 会话列表  │                              │              │
└─────────────┴──────────────────────────────┴──────────────┘
```

- **左栏 Sidebar**：视图切换（对话/协同）、会话管理（新建/切换/重命名/删除）、搜索入口、用户条
- **主区域**：对话视图（消息流 + 输入区）或协同视图（任务看板）
- **右栏 RightBar**：Artifact 预览面板 + 文件树（auto-hide 自动隐藏）

---

## 2. 对话视图（Chat）

**文件**: `src/routes/Chat.tsx`（667 行，核心界面）

### 实际功能
- 流式对话：调用 `neocodex_send_message_stream`，通过 5 个事件接收流式输出
- 消息渲染：用户消息（纯文本）/ 助手消息（Markdown）/ 工具消息（ToolCallCard）
- 会话自动创建：无会话时发送消息自动 `addSession()`
- 消息操作：重新生成、编辑并重发、复制
- 附件：文件预览 + 标注（annotation hint 随消息发送）
- 流式控制：停止生成、错误 Toast、标注提示条

### 操作步骤
1. **发送消息**：在底部输入框输入内容，按 `Enter` 发送（`Shift+Enter` 换行）
2. **停止生成**：生成期间发送按钮变为停止按钮（方块图标），点击即停止
3. **快速问答**：空状态时点击建议卡片（解释项目结构/修复编译错误/生成测试用例）
4. **重新生成**：悬停助手消息，点击 ↻ 图标
5. **编辑并重发**：悬停消息点击 ✎ 图标，修改后 `Ctrl/Cmd+Enter` 保存重发，`Esc` 取消
6. **复制**：悬停消息点击复制图标
7. **切换模型**：输入区左侧 ProviderSelector 图标（见 §6）
8. **权限模式**：输入区左侧 PermissionModeSelector（见 §7）
9. **附件标注**：附件预览中框选区域生成标注 hint，随下一条消息发送

### 快捷键
| 按键 | 功能 |
|------|------|
| `Enter` | 发送 |
| `Shift+Enter` | 换行 |
| `Ctrl/Cmd+Enter`（编辑态） | 保存并重发 |
| `Esc`（编辑态） | 取消编辑 |

### 后端对接
- `neocodex_send_message_stream`（发送，参数: content/attachments/regenerate/permission_mode/temperature/max_tokens）
- `neocodex_stop_stream`（停止）
- 事件: `neocodex_stream_start/token/end/done/tool`

---

## 3. 侧边栏（Sidebar）

**文件**: `src/components/Sidebar.tsx`（263 行）

### 实际功能
- 视图切换：对话 / 协同（segmented tabs）
- 会话管理：新建、切换、重命名、删除
- 会话按时间分组：今天 / 昨天 / 前7天 / 更早
- 搜索入口（开发中）、用户条（Free Plan）

### 操作步骤
1. **切换视图**：点击顶部"对话"或"协同"标签
2. **新建对话**：点击 + 按钮（或折叠态下的 + 图标）
3. **切换会话**：点击会话列表项（当前会话有红色左侧指示条）
4. **重命名**：悬停会话，点击 ✎ 图标，输入新名称
5. **删除**：悬停会话，点击 🗑 图标
6. **折叠/展开**：点击顶部 ChevronLeft 按钮（折叠后仅剩 + 按钮）

### 后端对接
- 会话数据来自 `chatStore`（前端 store），持久化经 `neocodex_list_sessions` / `neocodex_create_session` 等

---

## 4. 右栏 Artifact Pane + 文件树（RightBar）

**文件**: `src/components/RightBar.tsx`（352 行）

### 实际功能
- **Artifact Pane**：文件内容预览（Preview/Code 视图切换 + 6 种格式 tabs）
- **文件树**：目录展开/折叠、文件选中预览
- **auto-hide**：默认自动隐藏，鼠标悬停右侧边缘展开

### 操作步骤
1. **展开右栏**：鼠标移到窗口右边缘（auto-hide 模式）或点击浮动箭头按钮
2. **预览文件**：点击文件树中的文件，Artifact Pane 显示内容
3. **切换视图**：点击 Preview（眼睛）/ Code（代码）图标
4. **格式切换**：Raw / Rendered / WeChat / Zhihu / Juejin / Web 六种格式
5. **复制**：点击"复制"按钮复制当前文件内容
6. **展开/关闭**：点击"展开"放大面板，"关闭"关闭预览

> ✅ 已接线：文件树为**真实项目树**（`neocodex_project_tree`，跳过依赖/VCS 目录、深度/广度受限），点击文件经 `read_file` 懒加载真实内容。顶部显示项目根路径与文件数。

---

## 5. 协同视图（CoworkView）

**文件**: `src/components/CoworkView.tsx`（151 行）

### 实际功能
- 协同会话列表（左栏）+ 任务看板（右栏）
- 会话状态徽章：进行中（warn）/ 已完成（success）/ 失败（error）
- 任务进度：done/tasks + 百分比
- 智能体网格：每个智能体开关状态

### 操作步骤
1. **切换视图**：侧边栏点击"协同"标签
2. **新建会话**：点击会话列表顶部 + 按钮
3. **切换会话**：点击左侧会话项
4. **查看任务**：右侧显示当前会话的任务列表、进度、智能体状态

> ✅ 已接线：会话列表来自 `cowork_list`，新建经 `cowork_start`（工作区路径+描述），行动列表来自 `cowork_actions`，交付物来自 `cowork_list_deliverables`，支持暂停/恢复/停止（`cowork_pause/resume/stop`），顶部统计来自 `cowork_stats`。

---

## 6. 模型提供商选择（ProviderSelector）

**文件**: `src/components/ProviderSelector.tsx`（198 行）

### 实际功能
- 显示当前激活模型提供商
- 下拉列表展示所有可用提供商（含模型名、可用性标记）
- 切换提供商

### 操作步骤
1. **打开列表**：点击输入区左侧的提供商图标（或完整模式按钮）
2. **查看提供商**：列表中显示名称 + 模型名；"不可用"标记表示 API 不可解析
3. **切换**：点击目标提供商（当前激活项有 ✓ 标记）
4. **关闭**：点击列表外部区域

### 后端对接
| 命令 | 说明 |
|------|------|
| `neocodex_provider_config` | 获取提供商配置（provider_count/resolvable/active_model/providers） |
| `neocodex_set_provider` | 切换激活提供商（参数: name） |

---

## 7. 权限模式选择（PermissionModeSelector）

**文件**: `src/components/PermissionModeSelector.tsx`

### 实际功能
四种权限模式，控制 agent 执行动作的授权策略：

| 模式 | 说明 |
|------|------|
| `auto` | 自动模式（默认） |
| `manual` | 手动确认 |
| `accept_edits` | 接受编辑 |
| `plan` | 规划模式（只读，仅生成计划不执行） |

### 操作步骤
1. 点击输入区左侧的权限模式按钮
2. 从下拉选择模式（生成中禁用）

### 后端对接
- 模式值随 `neocodex_send_message_stream` 的 `permission_mode` 参数传递

---

## 8. 工具调用卡片（ToolCallCard）

**文件**: `src/components/ToolCallCard.tsx`（+ 测试 `ToolCallCard.test.tsx`）

### 实际功能
- 单行显示：Wrench 图标 + 工具名 + 成功✓/失败✗ + 耗时，默认折叠
- 展开显示 args / result 的 pre 块
- 失败状态红色边框

### 操作步骤
1. **查看工具调用**：消息流中工具消息显示为卡片
2. **展开详情**：点击卡片头部，展开显示 args 和 result
3. **复制**：展开后点击 args/result 旁的复制按钮

### 数据来源
- 流式事件 `neocodex_stream_tool`（name/args/result/duration_ms/success）
- 历史消息 `NeoCodexMessageItem.tool_call` 字段

---

## 9. 文件预览与标注（FilePreview / AnnotatedImage）

**文件**: `src/components/FilePreview.tsx` / `AnnotatedImage.tsx`

### 实际功能
- 附件文件预览（图片/表格/文本等类型图标区分）
- 图片标注：在图片上框选区域生成标注 hint

### 操作步骤
1. **查看附件**：消息中的附件显示为预览卡片
2. **标注图片**：点击图片进入标注模式，框选区域，生成标注文本
3. **发送标注**：标注 hint 显示在输入区上方提示条，随下一条消息发送（或点击 X 移除）

---

## 10. Markdown 渲染（Markdown）

**文件**: `src/components/Markdown.tsx`

### 实际功能
- 助手消息的 Markdown 渲染（代码块、列表、引用等）
- 代码块带 header + 复制按钮

### 操作步骤
- 无需操作，自动渲染；代码块右上角可复制

---

## 11. Git 面板（GitPanel）

**文件**: `src/components/GitPanel.tsx`（227 行）

### 实际功能
- 显示 Git 状态（分支 + dirty 标记）
- 显示 diff 文件列表（changed-file list，对标 Codex Desktop P0 痛点）
- 每个文件：add/del 计数 + 展开 diff hunks + accept/reject 操作

### 操作步骤
1. **打开面板**：点击 Git 入口（面板 open 状态）
2. **查看状态**：顶部显示分支名 + 干净/有改动徽章
3. **查看 diff**：文件列表自动展开第一个文件，显示 +add/-del 行
4. **展开/折叠**：点击文件头切换 diff 显示
5. **接受/拒绝**：点击 ✓（accept）/ ✗（reject）应用或丢弃改动
6. **刷新**：点击刷新按钮重新加载

### 后端对接
| 命令 | 说明 |
|------|------|
| `neocodex_git_status` | Git 状态（branch/dirty） |
| `neocodex_get_diff` | diff 文件列表（files/hunks/lines） |
| `neocodex_apply_diff` | 应用 diff（参数: path/action） |

---

## 12. 项目视图（ProjectView）

**文件**: `src/components/ProjectView.tsx`（185 行）

### 实际功能
- 项目目录树（真实读取，跳过依赖/VCS 目录，限制深度与广度）
- AGENTS.md 内容查看（项目宪法）
- 文件点击回调（可打开文件）

### 操作步骤
1. **打开面板**：点击项目视图入口
2. **浏览目录**：点击目录展开/折叠（自动展开一级目录）
3. **切换 tab**：tree（目录树）/ agents（AGENTS.md）
4. **打开文件**：点击文件节点（触发 onOpenFile 回调）

### 后端对接
| 命令 | 说明 |
|------|------|
| `neocodex_project_tree` | 项目树（root/tree/agents_md/file_count） |

---

## 13. 成本 / Token 看板（CostDashboard）

**文件**: `src/components/CostDashboard.tsx`

### 实际功能
- 已花费金额 + 预算
- Token 用量 + 轮数
- 上下文占用百分比（进度条）
- 进化迭代次数 + 运行时长
- 运行状态（运行中/空闲）

### 操作步骤
1. **打开面板**：点击成本看板入口
2. **查看指标**：四个统计卡片（已花费/Token/上下文/进化迭代）
3. **上下文进度条**：显示 context_usage 百分比

### 后端对接
| 命令 | 说明 |
|------|------|
| `neocodex_agent_status` | Agent 状态（running/current_task/uptime_secs/turn_count/tokens_used/context_usage/provider_model/evolution_iterations/cost_spent/cost_budget） |

---

## 14. 插件市场（PluginMarketplace）

**文件**: `src/components/PluginMarketplace.tsx`

### 实际功能
- 插件列表（id/name/version/enabled/loaded/load_time_ms/error）
- 安装插件（选择 manifest.json）
- 卸载 / 启用 / 禁用插件
- 事件日志（最近 30 条）

### 操作步骤
1. **打开面板**：点击插件市场入口
2. **安装**：点击"安装"按钮，文件对话框选择插件 manifest.json
3. **启用/禁用**：点击插件开关切换
4. **卸载**：点击卸载按钮
5. **查看日志**：底部显示插件事件日志
6. **刷新**：点击刷新按钮

### 后端对接
| 命令 | 说明 |
|------|------|
| `plugin_list` | 插件列表 |
| `plugin_install` | 安装（参数: path） |
| `plugin_uninstall` | 卸载（参数: id） |
| `plugin_enable` / `plugin_disable` | 启用/禁用（参数: id） |
| `plugin_event_log` | 事件日志（参数: count） |

---

## 15. 定时任务（ScheduledTasks）

**文件**: `src/components/ScheduledTasks.tsx`（290 行）

### 实际功能
- 创建定时任务（名称 + 提示词 + RRULE 调度）
- 任务列表（状态徽章：运行中/已暂停/错误/空闲）
- 任务操作：立即执行、暂停、恢复、删除
- 调度预设（每天/每周）

### 操作步骤
1. **打开面板**：点击定时任务入口
2. **创建任务**：填写任务名 + 提示词，选择调度预设（或输入 RRULE），点击创建
3. **立即执行**：点击任务行的"立即执行"
4. **暂停/恢复**：点击暂停/恢复按钮
5. **删除**：点击删除按钮

### 后端对接
| 命令 | 说明 |
|------|------|
| `list_background_tasks` | 任务列表 |
| `create_background_task` | 创建（参数: name/prompt/schedule） |
| `run_background_task_now` | 立即执行 |
| `pause_background_task` / `resume_background_task` | 暂停/恢复 |
| `delete_background_task` | 删除 |

---

## 16. Checkpoint 时间线（CheckpointTimeline）

**文件**: `src/components/CheckpointTimeline.tsx`（158 行）

### 实际功能
- 会话 checkpoint 列表（时间 + 消息数）
- 恢复到指定 checkpoint（Claude /rewind 对齐）

### 操作步骤
1. **打开面板**：点击 checkpoint 时间线入口
2. **查看列表**：显示各 checkpoint 的相对时间 + 消息数
3. **恢复**：点击恢复按钮 → 确认对话框 → 恢复后消息自动重载

### 后端对接
| 命令 | 说明 |
|------|------|
| `neocodex_checkpoint_list` | 列表（参数: session_id） |
| `neocodex_checkpoint_restore` | 恢复（参数: session_id/checkpoint_id） |

---

## 17. 侧聊（SideChat）

**文件**: `src/components/SideChat.tsx`（144 行）

### 实际功能
- 会话侧边聊天（独立于主对话的消息流）
- 发送/接收侧聊消息

### 操作步骤
1. **打开面板**：点击侧聊入口（需有激活会话）
2. **查看消息**：显示侧聊历史
3. **发送**：输入内容点击发送

### 后端对接
| 命令 | 说明 |
|------|------|
| `neocodex_get_side_chat` | 获取侧聊（参数: session_id） |
| `neocodex_send_side_chat` | 发送（参数: session_id/content） |

> ⚠️ 参数必须用 `session_id`（snake_case），后端 `rename_all = "snake_case"`。

---

## 18. 电脑控制（Computer Use）

**文件**: `src/components/ComputerUse.tsx`（350 行）

### 实际功能
- 屏幕截图 + 显示（data URL）
- 窗口列表 + 前台应用
- 鼠标：移动、点击、位置
- 键盘：输入文本、按键 + 修饰键
- 显示器列表

### 操作步骤
1. **打开面板**：点击 Computer Use 入口
2. **截图**：自动截图并显示；点击刷新重新截图
3. **移动鼠标**：输入 x/y 坐标点击移动
4. **点击**：点击"点击"按钮（当前鼠标位置）
5. **输入文本**：输入文本点击"输入"
6. **按键**：输入 key code + 勾选修饰键（Cmd/Ctrl/Alt/Shift）点击"按键"
7. **查看窗口**：面板显示前台应用 + 窗口列表

### 后端对接
| 命令 | 说明 |
|------|------|
| `computer_screen_list` | 显示器列表 |
| `computer_screenshot_and_save` | 截图保存（参数: path） |
| `computer_get_frontmost_app` | 前台应用（app_name/title） |
| `computer_get_window_list` | 窗口列表（title/pid/app_name） |
| `computer_mouse_move` | 移动鼠标（参数: x/y） |
| `computer_mouse_click` | 点击（参数: button） |
| `computer_mouse_position` | 鼠标位置 |
| `computer_keyboard_type` | 输入文本（参数: text） |
| `computer_keyboard_press` | 按键（参数: key/modifiers） |

> ⚠️ 需要 macOS 辅助功能权限（System Events）。

---

## 19. 窗口控制（TrafficLights）

**文件**: `src/components/TrafficLights.tsx`

### 实际功能
- 自绘 macOS 红绿灯（最小化/最大化/关闭）
- 窗口无系统装饰（decorations: false），红绿灯自绘

### 操作步骤
- 点击红/黄/绿按钮：关闭 / 最小化 / 最大化

### 后端对接
| 命令 | 说明 |
|------|------|
| `window_minimize` / `window_maximize` / `window_close` | 窗口控制 |

---

## 20. 后端命令域总览

后端共 **55 个命令文件、521 个注册命令**。前端消费 39 个命令（§2-§19 已列）。主要命令域：

| 命令域 | 文件 | 命令数 | 说明 |
|--------|------|--------|------|
| neocodex | `neocodex_cmds.rs` | 46 | 对话/会话/checkpoint/项目/搜索 |
| computer_interactive | `computer_interactive_cmds.rs` | 20 | 屏幕/鼠标/键盘/后台任务 |
| cowork | `cowork_cmds.rs` | 20 | 协同会话（前端未接线） |
| workflow | `workflow_cmds.rs` | 19 | 工作流 |
| preview | `preview_cmds.rs` | 18 | 预览 |
| teleport | `teleport_cmds.rs` | 18 | 传送 |
| annotation | `annotation_cmds.rs` | 17 | 标注 |
| memory_mgr | `memory_mgr_cmds.rs` | 17 | 记忆管理 |
| brain | `brain_cmds.rs` | 16 | 大脑 |
| term_tabs | `term_tabs_cmds.rs` | 16 | 终端标签 |
| voice | `voice_cmds.rs` | 15 | 语音 |
| enterprise | `enterprise_cmds.rs` | 15 | 企业 |
| marketplace | `marketplace_cmds.rs` | 15 | 市场 |
| channels | `channels_cmds.rs` | 14 | 频道 |
| loop | `loop_cmds.rs` | 14 | 循环 |
| routines | `routines_cmds.rs` | 14 | 例程 |
| security_scan | `security_scan_cmds.rs` | 14 | 安全扫描 |
| insights | `insights_cmds.rs` | 13 | 洞察 |
| unified_session | `unified_session_cmds.rs` | 13 | 统一会话 |
| profile | `profile_cmds.rs` | 12 | 画像 |
| context | `context_cmds.rs` | 11 | 上下文 |
| diff | `diff_cmds.rs` | 11 | diff |
| agent_view | `agent_view_cmds.rs` | 10 | agent 视图 |
| mcp_host | `mcp_host_cmds.rs` | 10 | MCP 宿主 |
| plugin | `plugin_cmds.rs` | 10 | 插件 |
| websearch | `websearch_cmds.rs` | 10 | 网页搜索 |
| summary | `summary_cmds.rs` | 10 | 摘要 |
| ... | ... | ... | ... |

---

## 附录 A：前端组件 ↔ 后端命令映射

| 组件 | 后端命令 |
|------|----------|
| Chat | neocodex_send_message_stream, neocodex_stop_stream, neocodex_provider_config |
| Sidebar | （经 chatStore → neocodex_list_sessions 等） |
| ProviderSelector | neocodex_provider_config, neocodex_set_provider |
| GitPanel | neocodex_git_status, neocodex_get_diff, neocodex_apply_diff |
| ProjectView | neocodex_project_tree |
| RightBar | neocodex_project_tree, read_file |
| CoworkView | cowork_list, cowork_start, cowork_actions, cowork_list_deliverables, cowork_pause/resume/stop, cowork_stats |
| CostDashboard | neocodex_agent_status |
| PluginMarketplace | plugin_list/install/uninstall/enable/disable/event_log |
| ScheduledTasks | list_background_tasks, create_background_task, run/pause/resume/delete_background_task |
| CheckpointTimeline | neocodex_checkpoint_list, neocodex_checkpoint_restore |
| SideChat | neocodex_get_side_chat, neocodex_send_side_chat |
| ComputerUse | computer_screen_list, computer_screenshot_and_save, computer_get_frontmost_app, computer_get_window_list, computer_mouse_*, computer_keyboard_* |
| TrafficLights | window_minimize, window_maximize, window_close |

## 附录 B：已知限制（未接线功能）

| 界面 | 状态 | 说明 |
|------|------|------|
| Sidebar 搜索 | 占位 | 搜索按钮无功能（开发中） |
| Sidebar 用户条 | 占位 | 设置按钮仅 console.log |
| 语音输入按钮 | 占位 | cic 输入区语音按钮无功能 |
| 附件按钮 | 占位 | cic 输入区附件按钮无功能 |