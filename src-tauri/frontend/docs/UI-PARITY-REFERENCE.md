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

## 优先级建议
1. **P0 右面板可 resizable + 宽度持久化**（最直接的对 Codex pixel-parity 差距）
2. **P1 command palette Cmd/K 跨实体模糊**（已有 palette，强化 scope）
3. **P1 状态栏补 branch/worktree**（呼应 R-P47/状态感知）
4. **P2 empty-state 三态统一**（each pane）