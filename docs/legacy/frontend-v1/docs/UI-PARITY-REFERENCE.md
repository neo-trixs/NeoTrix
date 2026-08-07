# UI Design Parity Reference — Codex Desktop × Claude Code Desktop

> 采集自: ManasesLovera/codex-desktop UI/UX 布局文档 (pixel-parity 参考)、Anthropic Claude Code Desktop 重构文档 (code.claude.com/docs/en/desktop)、2026-04-14 Desk桌面重构手记。
> 用途: NeoTrix NeoCodexPage 三栏布局的差异对照清单。source = absorption。

## 1. 窗口外壳 (Codex Desktop, 像素级参考)

```
┌───────────────────────────────────────────────────────────────────────────┐
│ ◐ titlebar (custom; workspace switcher · search · profile/usage)            │
├──────────┬─────────────────────────────────────────────┬───────────────────┤
│  LEFT    │              MAIN AREA                       │   RIGHT SIDE      │
│  SIDEBAR │  (active view: chat / agent / settings…)     │   PANEL (dock)    │
│         │                                             │                   │
│  nav     │   ┌───────────────────────────────────┐     │  tabs:            │
│  rail +  │   │ chat transcript (virtualized)      │     │  • Browser        │
│  list    │   │                                   │     │  • Chat           │
│          │   │                                   │     │  • Terminal       │
│          │   ├───────────────────────────────────┤     │  • Review changes │
│          │   │ composer (model · plan · attach)   │     │                   │
├──────────┴─────────────────────────────────────────────┴───────────────────┤
│ status bar (harness profile · branch/worktree · model · usage · sync)        │
└───────────────────────────────────────────────────────────────────────────┘
```

## 2. 左侧栏 Left sidebar (Codex)
- Workspace switcher (top) — 当前 workspace 下拉切换/新建
- Pinned（置顶，可拖拽排序）
- Chats — 可搜索，按日期分组
- Projects — 仓库，展开 worktrees/agents/sites
- Agents — 运行/近期 run，live status dots
- Automations / Sites / Plugins-MCP
- Footer: Profile / Usage / Settings

## 3. 主区域 (chat view)
- Transcript: 虚拟化消息列表, user/assistant 气泡, tool-call 卡片(可折叠), diff 卡片, plan 卡片, 附件 chips, 代码块复制+语法高亮, streaming 光标
- Composer: 多行输入; 左侧控件 = model 切换 / plan mode 切换 / attach 上传 / harness profile chip; send/stop 按钮; slash 命令 + @-mention
- 其它视图: agent run detail, settings, usage dashboard, automation 编辑器, MCP manager

## 4. 右侧面板 (dockable — 关键差异)
- 可 resizable + collapsible (可拖宽窄、持久化 per project)
- Tabbed surfaces: Browser(沙箱 web 预览) • Chat(side chat) • Terminal(xterm PTY) • Review changes(diff + per-file/hunk approve/stage)
- 面板状态 (open tab, width) 按 project/workspace 持久化

## 5. 状态栏
- 恒定可见: active harness profile (含 permission 图标)、当前 branch/worktree、model、usage 概览、后台 sync/agent 活动、错误/toast 锚点

## 6. Command palette
- Cmd/Ctrl-K 全局: 切换 chats/projects/workspaces、运行命令、启动 agents、改 model、开关面板; 模糊搜索跨实体

## 7. Theming / typography / motion
- Light+dark, tokens 驱动, 跟随系统偏好, 100/125/150% 缩放测试
- lucide 图标; system UI 字体 + monospace code/terminal
- 微妙 motion (token 淡入/面板滑动), honor prefers-reduced-motion
- 每个视图定义 empty/loading(inline skeleton 非 spinner)/error 状态

---

## GAP 对照 (NeoTrix 现有实现 vs 上述设计)

| 能力 | Codex/Claude 参考 | NeoTrix 现有 | 差距 |
|------|------------------|--------------|------|
| 三栏布局 | ✓ 侧栏/主区/右dock | ✓ 已实现 | 无 |
| 右面板 tabs | Browser/Chat/Terminal/Review | review/terminal/browser/file/tasks | 缺 **Chat** (side chat 已独立存在) |
| 右面板可调宽/持久化 | 可 resizable, per-project 持久化 | 固定宽度 (CSS) | **差: 需 resizable + persist** |
| Composer 控件 | model+plan+attach+permission chip | ✓ model/permission/plan/attach | 基本齐 |
| 状态栏 | branch/worktree/model/usage/permission | ✓ statusBtn+usagePopover | 缺 branch/worktree |
| Cmd/Ctrl-K palette | 全局模糊搜索跨实体 | ✓ CommandPalette 存在 | 无 |
| 会话搜索/侧栏 | 按日期分组+project 展开 | ✓ SessionSidebar search | 可加固 |
| tool-call 卡片可折叠/diff/plan | ✓ | ✓ (TaskPane/DiffPane/ChatView planToggle) | 无 |
| streaming 光标/代码块高亮复制 | ✓ decorateCodeBlocks 已有 | ✓ | 无 |
| empty/loading/inline-error | 明确三态 | 各面板自带 | 部分 |

## 键盘快捷键对照 (Claude Code Desktop 官方表, code.claude.com/docs/en/desktop)

| 官方快捷键 | 动作 | NeoTrix 现状 |
|------------|------|--------------|
| `⌘/` | 快捷键面板 | ✓ |
| `⌘N` | 新建会话 | ✓ |
| `⌘W` | 关闭会话 | ✓ |
| `Ctrl Tab` / `Ctrl Shift Tab` | 下/上一会话 | ✓ |
| `⌘Shift ]` / `⌘Shift [` | 下/上一会话 | ✓ (2026-08-04 补) |
| `Esc` | 停止生成 | ✓ |
| `⌘Shift D` | 切换 Diff 面板 | ✓ (修 Shift 大写 key 隐性 bug) |
| `⌘Shift B` | 切换 Browser 面板 | ✓ (2026-08-04 补, 原仅 ⌘Shift P) |
| `Ctrl ~` | 切换终端 | ✓ |
| `⌘\` | 关闭焦点面板 | ✓ (2026-08-04 补, 关闭右 dock) |
| `⌘;` | 侧聊 | ✓ (dock Chat tab) |
| `Ctrl O` | 循环视图模式 | ✓ |
| `⌘Shift M` | 审批模式菜单 | ✓ (2026-08-04 补, 循环 tri-state) |
| `⌘Shift I` | 模型菜单 | ✓ (2026-08-04 补, window event→ModelSelector) |
| `⌘Shift E` | effort 菜单 | 无 (后端无 effort 概念, 跳过) |
| `1–9` | 选中打开菜单中的项 | ⌘1-9 用于会话切换 (differentiator) |
| `⌘Shift S` | Browser 选元素 | 无 (依赖 Browser 交互后端) |

> 教训 (2026-08-04): 真实 keydown 事件 Shift+字母产出大写 `e.key` (如 ⌘Shift+F → "F"),
> 此前 `e.key === "f"` 的判断在生产环境永不触发。已统一归一化
> `key = e.key.length === 1 ? e.key.toLowerCase() : e.key`。同步修复 ⌘Shift+P 分支
> 顺序 (shift 变体必须先于 ⌘P file palette 判断, 否则被吞)。

## 其余官方交互补缺 (2026-08-04)

| 能力 | 官方行为 | 现状 |
|------|---------|------|
| 右面板 resizable + 持久化 | 拖边缘调宽, per-project 持久化 | ✓ 早已实现 (`rightPanelWidth` + localStorage), **doc 原 P0 已过时** |
| Diff stats indicator | 显示 `+12 -1`, 点击打开 diff viewer | ✓ 已有 (health.diff_stats), **新增点击切到 review 面板** |
| Diff 行内评论 | 点 diff 行 → 评论框, Enter 添加, ⌘Enter 提交全部 | ✓ 2026-08-04 新增 (DiffPane line comments → window event → 走现有 chat send 管道, 无新 Rust IPC) |
| Review code 按钮 | diff 工具栏顶部 | ✓ 已有 (`AI 审查` = cmd_diff_review) |
| diff 支持 staged/unstaged/base/file/neocodex scope | 文件列表左 + 变更右 | ✓ 已有 |
| 逐文件 accept/reject | per-file 按钮 | ✓ 已有 (含 untracked stage-add) |

## 优先级建议 (更新)
1. **P1 command palette Cmd/K 跨实体模糊**（已有 palette，强化 scope）
2. **P1 状态栏补 branch/worktree**（呼应 R-P47/状态感知; branch 已有, worktree 待补）
3. **P2 empty-state 三态统一**（each pane）
4. **P2 file pane 磁盘变更警告/Save**（依赖 file 读写的 Rust IPC, 深度三 blocked 时不动）

## Claude Desktop 前端重构 (2026-08-04)
按 Claude Code Desktop 前端结构重构 (commit dca03c3 之后的 Phase A-G 工作):
- **Phase A — 顶层三 tab**: `TopTabBar` (Chat/Cowork/Code), Code 激活其余占位 (`TabPlaceholder`), App.tsx 条件渲染, 5 tests。
- **Phase B — Composer 行对齐**: 权限三态 → 5 模式菜单 (manual/accept/plan/auto/bypassPermissions, `permission-option-*`); ⌘Shift+M 打开菜单; Transcript 视图下拉 (正常/详细/摘要); `UsageRing` 圆环 (绿/黄/红, 点击开 usage popover); ChatView `+` 按钮菜单 (附加文件 / Slash 命令 / 引用文件)。
- **Phase B2 — 控件收敛到 composer footer**: model/usage ring/模式/项目 pill/权限/视图下拉从聊天区顶部工具栏移入 ChatView 底部 `composerFooter` (`composerControls` 插槽), 顶部工具栏删除 — 对齐 Claude "控件都在底部 composer 行" 布局。
- **Phase C — 视觉对齐**: 消息气泡 → Claude 式扁平消息 (无背景/无边框, 全宽) + 角色标签 (你 / NeoCodex); composerFooter flex-wrap 防溢出。
- 验证: TSC clean, 214 tests, vite build ok。
- 遗留: 深度三 (Rust IPC) 仍被并发会话阻塞; 视觉 token 对齐近黑/ivory 待用户目检确认。
