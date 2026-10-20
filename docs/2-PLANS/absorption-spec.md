# NeoTrix UI — Absorption Specification

> 吸收 osaurus (Swift macOS AI Harness) + NovaChat 设计规格 → 统一 NeoTrix 桌面应用架构

---

## 1. 吸收来源总览

| 来源 | 核心价值 | 吸收到 NeoTrix 的模块 |
|------|---------|---------------------|
| **osaurus** | 原生 macOS AI Harness: 代理循环、三层记忆、隐私过滤、加密身份、MCP Server/Client、沙箱 VM、插件系统、语音、本地模型 | 代理 → Cowork/Agent 视图；记忆 → Chat 上下文；隐私 → 设置面板；身份 → 用户系统；沙箱 → Code 视图；插件 → MCP 工具注册表；语音 → 输入方式 |
| **NovaChat** | 三栏式聊天客户端: 流式渲染、会话管理、Markdown、附件、主题系统、错误处理、快捷键 | 直接对应 Chat 视图的完整实现规格；Sidebar 会话列表；Artifact Panel；SettingsModal |

---

## 2. 架构吸收映射

### 2.1 从 osaurus 吸收

| osaurus 特性 | NeoTrix 吸收位置 | 实现方式 |
|-------------|-----------------|---------|
| Agent Loop (working dir → tools → sandbox → todo → execute → summary) | 团队(Cowork) 视图 + 代理(Agent) 视图 | Cowork 负责工作流编排，Agent 负责执行监控 |
| 三层记忆 (identity → facts → episodes) + 背景合并器 | 对话(Chat) 视图上下文管理 + 设置记忆面板 | `localStorage` 会话级记忆 → v2 Rust 记忆管道 |
| 隐私过滤 (MLX 分类器 + 正则 + fail-closed + 审查面板) | 设置 → 隐私标签 + 对话发送前审查 UI | Rust keyring + 本地正则检测 + 审查对话框 |
| 加密身份 (secp256k1 + master key + 可撤销访问密钥) | 用户弹出层 + 设置 → 账户标签 | 先 mock → v2 Rust keyring crate 实现 |
| 安全信道 (X25519 + ChaCha20-Poly1305 E2EE) | 代理 → 中继(Relay) 子标签 | P3 功能，先留占位 |
| MCP Server + Client (stdio bridge + OAuth 2.1 远程提供者) | 代理 → 订阅 子标签 + 设置 → 计算资源 | MCP 服务器列表 + 自动发现 (已有 `mcp_discovery`) |
| 沙箱 VM (Apple Containerization + vsock) | 代码(Code) 视图 → 运行按钮 | `invoke('execute_code')` → Rust sandbox (已有 `nt_shield_sandbox`) |
| 技能与方案 (RAG 搜索选择) | 侧边栏 → 技能列表 + 设置 → 知识库 | 从 `~/.neotrix/skills/` 发现 (已有 `SkillsEngine`) |
| 语音 (ANE 离线转录 FluidAudio) | 输入框 → 语音按钮 | v2 Web Speech API → v3 Rust ANE bridge |
| 插件系统 (v3 宿主 API + 热重载) | 代理 → 节点 子标签 + MCP 工具注册表 | P3 功能 |

### 2.2 从 NovaChat 吸收

| NovaChat 特性 | NeoTrix 吸收位置 | 当前状态 | 差距 |
|--------------|-----------------|---------|------|
| 三栏布局 (sidebar + chat + artifact) | 三者均已有 | ✅ 布局一致 | Artifact Panel 默认隐藏而不是自动滑出 |
| 流式 token-by-token 渲染 | Chat 视图 | ❌ 无 | 需要 Rust IPC + SSE 事件 |
| Markdown + 代码高亮 | Chat 消息渲染 | ❌ 纯文本 | 需要 `marked` + `highlight.js` 集成 |
| 会话管理 (按时间分组/搜索/置顶/重命名/删除) | Sidebar `recentData` | ❌ mock 数据 | 全部需要 `localStorage` → SQLite |
| 会话搜索 | Sidebar 搜索按钮 | ❌ 死按钮 | 需要实时过滤 |
| 附件 (拖拽/选择上传) | 输入框 + 按钮 | ❌ 空壳 | 需要隐藏 input + 拖拽事件 |
| 主题三态 (light/dark/system) | 设置 → 外观 | ❌ 只有 light/dark | 需要 `prefers-color-scheme` 监听 |
| 错误处理 (401/429/529) | Chat 消息状态 | ❌ 无 | 需要消息状态枚举 + 内联重试 |
| 键盘快捷键 | 全局 | ❌ 仅 Cmd+, | 需要完整 7 快捷键 |
| 消息操作条 (hover → 复制/编辑/重新生成/赞/踩) | Chat 消息悬停 | ❌ 无 | 需要 hover UI |
| EmptyState (问候语 + 建议卡片) | Chat hero | ✅ 已有 | 需要调整以匹配 NovaChat 规格 |
| 设置 Tab (通用/API/模型/快捷键/关于) | SettingsModal | ✅ 结构接近 | 需要 API Key tab + 快捷键 tab |
| API Key 安全 (keyring crate, 永不在前端) | 设置 → API | ❌ 所有设置不持久 | 需要 Rust keyring 集成 |
| InputBox (多行/自动增高/Enter vs Shift+Enter/附件 chip/模型选择/stop 按钮) | Chat 输入框 | ❌ 部分 | 需要完整重写 |

---

## 3. 每个 UI 元素的完整功能规格

### 3.1 侧边栏 Sidebar

```
┌─────────────────────┐
│  NeoTrix       [+]  │  ← 点击图标折叠/展开侧栏 (toggleSidebar)
│─────────────────────│      [+] = 新建会话 (Cmd+N)
│  🔍 搜索会话...      │  ← 输入实时过滤会话列表 (FTS5 / LIKE)
│─────────────────────│
│  <今天>              │
│  · 引擎调试          │  ← 单击切换会话, 右键: 重命名/置顶/删除
│  · 架构讨论          │     当前会话高亮: --accent-bg
│─────────────────────│
│  对话 团队 代码 代理   │  ← 视图切换 tabs (switchView)
│─────────────────────│
│  设置               │  ← 打开 SettingsModal (Cmd+,)
│  👤 Neo             │  ← 点击打开 UserPopover (toggleUserPopover)
└─────────────────────┘
```

#### Sidebar 元素规范

| 元素 | 动作 | 参数 | 实现 |
|------|------|------|------|
| Logo/图标 | 点击展开/折叠 `.sb` | `sidebarCollapsed: bool` | `toggleSidebar()` → 存储 `localStorage` |
| ➕ 按钮 | 新建会话 | `title: '新对话'` | `newChat()` → 生成 UUID → 追加到会话列表 → 切换到空 chat |
| 🔍 搜索框 | 输入过滤 | `query: string` | `onInput` → 过滤 `chatSessions` → 只显示匹配项 |
| 会话项 | 点击切换 | `sessionId: string` | `switchSession(id)` → 保存当前消息 → 加载目标会话消息 |
| 会话项 | 右键菜单 | `sessionId, action: 'pin'/'rename'/'delete'` | 显示上下文菜单 → 执行操作 |
| 视图 Tabs | 切换视图 | `view: 'chat'/'cowork'/'code'/'agent'` | `switchView(el, view)` |

### 3.2 对话视图 Chat

```
┌────────────────────────────────────────────────┐
│  [引擎调试]  [Claude Sonnet ▾]   [⋯ 更多 ▾]    │  ← TopBar
├────────────────────────────────────────────────┤
│                                                │
│  ┌───────────────────────────┐                 │
│  │ 你会如何优化 E8 引擎？     │  ← 用户消息 (右对齐, 气泡)
│  │                    10:32  │                 用户气泡: --bg-user-bubble
│  └───────────────────────────┘                  圆角 --radius-lg, 最大 70%
│                                                │
│  ┌───────────────────────────────────────────┐ │
│  │ 我来分析 E8 引擎的优化方向:                 │ │  ← AI 消息 (左对齐, 无气泡)
│  │                                          │ │     最大 780px 居中列
│  │ ## 1. 状态空间裁剪                        │ │     流式: token-by-token 渲染
│  │ 当前 64 态中仅有 12 态活跃...              │ │     最后 token 后闪烁光标块
│  │                                          │ │
│  │ ```rust                                   │ │     代码块: 语言标签 + 复制按钮
│  │ fn optimize() { ... }                     │ │    背景 --code-bg, 圆角 --r-md
│  │ ```                                       │ │
│  │                                     10:35 │ │
│  │   [📋 复制] [🔄 重新生成] [👍] [👎]       │ │  ← hover 浮现操作条
│  └───────────────────────────────────────────┘ │
│                                                │
├────────────────────────────────────────────────┤
│ [📎 file.pdf] [📄 doc.md]                     │  ← 附件 Chip (可移除)
│ ┌──────────────────────────────────────────┐  │
│ │ 输入消息...                        [🎤] ▸│  │  ← InputBox
│ │                                          │  │     自动增高, 1-10 行
│ │                                          │  │     Enter 发送 / Shift+Enter 换行
│ │                              [Claude ▾] │  │     左侧 📎 附件按钮
│ │                                          │  │     右侧 语音 + 发送/停止
│ └──────────────────────────────────────────┘  │
│ [42/2000 tokens]                              │  ← token 计数 (超限警示色)
└────────────────────────────────────────────────┘
```

#### Chat 元素规范

| 元素 | 动作 | 参数 | 实现 |
|------|------|------|------|
| 会话标题 | 点击内联编辑 | `title: string` | `contentEditable` → `onBlur` 保存 |
| 模型选择 | 下拉选择 | `model: string` | 从 `viewSettings.chat.model` 读取 |
| ⋯ 更多 | 下拉菜单 | `导出 Markdown/JSON, 清空上下文, 删除会话` | 打开上下文菜单 |
| 用户消息 | 渲染 | `content, timestamp` | 右对齐, `--bg-user-bubble`, 最大 70% |
| AI 消息 | 渲染 (流式) | `content, timestamp, status` | 左对齐, 无气泡, 最大 780px, 流式追加 |
| 消息 hover | 浮现操作条 | `📋复制/🔄重新生成/👍/👎` | `onMouseEnter` 显示, `onMouseLeave` 隐藏 |
| 代码块 | 语言标签 + 复制 | `language, code` | `highlight.js` 渲染 + clipboard API |
| 输入框 | 多行输入 | `value: string` | `autoResize`, Enter=send, Shift+Enter=newline |
| 📎 附件 | 文件选择 | `files: File[]` | 隐藏 `<input type=file multiple>` → 显示 chip |
| 🎤 语音 | 语音输入 | — | Web Speech API → 填入输入框 |
| ▸ 发送 | 发送消息 | — | `sendMsg()` → 流式 IPC |
| ■ 停止 | 停止生成 | — | `stopGeneration()` → IPC |
| Token 计数 | 显示用量 | `used, max` | 从消息计算, 超限警示色 |

### 3.3 团队视图 Cowork

从 osaurus Agent Loop 吸收:

```
┌────────────────────┬─────────────────────────────┐
│ 会话列表            │  任务看板                    │
│                     │                             │
│  ├ 架构讨论 (3任务)  │  ┌──────────────────────┐   │
│  ├ 代码审查 Sprint  │  │ 📋 优化 E8 状态空间    │   │
│  ├ 引擎调试 (2任务)  │  │  状态: 进行中           │   │
│                     │  │  负责人: agent-alpha    │   │
│  [+] 新建会话       │  │  ┌──────────────────┐  │   │
│                     │  │  │ ✓ 分析 64 态活跃度  │  │   │
│                     │  │  │ □ 裁剪死态          │  │   │
│                     │  │  │ □ 更新 E8 策略表    │  │   │
│                     │  │  └──────────────────┘  │   │
│                     │  └──────────────────────┘   │
│                     │                             │
│                     │  Agent 状态:                 │
│                     │  🟢 alpha (E8 引擎)          │
│                     │  🟡 beta (前端 UI)           │
│                     │  🔴 gamma (沙箱)             │
│                     │                             │
│                     │  [📎 附件] [📝 备注]         │
└────────────────────┴─────────────────────────────┘
```

#### Cowork 元素规范

| 元素 | 动作 | 参数 | 实现 |
|------|------|------|------|
| 会话列表 | 点击切换 | `sessionId` | `switchCwSession(id)` |
| ➕ 新建 | 创建任务组 | `name: string` | `newCoworkTask()` → 持久化 |
| 任务卡片 | 查看/编辑 | `taskId` | 展开显示任务详情 + 子任务列表 |
| 子任务 | □ 勾选完成 | `subtaskId` | `toggleSubtask(id)` |
| Agent 状态 | 显示在线/离线 | `agentId, status` | 颜色指示器 🟢🟡🔴 |
| 附件 | 上传文件 | `files` | 同 Chat 附件逻辑 |
| 备注 | 文本编辑 | `note: string` | `contentEditable` |

### 3.4 代码视图 Code

从 osaurus Sandbox 吸收:

```
┌─────────────────────┬────────────────────────────┐
│ 📁 文件树            │  main.rs × lib.rs ×       │  ← 标签页
│                     │ ┌────────────────────────┐ │
│  src/               │ │ fn optimize_e8() {      │ │
│  ├ main.rs          │ │     // ...              │ │
│  ├ lib.rs           │ │ }                       │ │
│  ├ engine/          │ │                         │ │
│  │  ├ e8.rs         │ │  [格式化] [保存] [复制] [▶]│ │
│  │  └ sae.rs        │ └────────────────────────┘ │
│  └ config.rs        │                            │
│                     │ 输出:                       │
│  Cargo.toml         │ ┌────────────────────────┐ │
│                     │ │ $ cargo check           │ │
│ Git: main ↑1        │ │ Checking neotrix-core... │ │
│                     │ │ error[E0308]: ...        │ │
│                     │ └────────────────────────┘ │
└─────────────────────┴────────────────────────────┘
```

#### Code 元素规范

| 元素 | 动作 | 参数 | 实现 |
|------|------|------|------|
| 文件树 | 点击打开文件 | `path: string` | `invoke('fs_tree')` → 渲染树 |
| 文件标签页 | 切换/关闭 | `tabId` | 多标签管理 (同 NovaChat code tabs) |
| 代码编辑区 | 只读/编辑 | `content, language` | 语法高亮 + 行号 |
| 🔄 刷新 | 重读文件树 | — | `invoke('fs_tree')` |
| 格式化 | Rust fmt | — | `invoke('format_code', {path})` |
| 保存 | 写入文件 | `content` | `invoke('write_file', {path, content})` |
| 复制 | 到剪贴板 | — | `navigator.clipboard.writeText()` |
| ▶ 运行 | 沙箱执行 | — | `invoke('execute_code', {code})` |
| Git 状态 | 显示分支/diff | `branch, ahead` | `invoke('git_status')` |
| 输出面板 | 编译/运行输出 | `output: string` | 下方控制台风格面板 |

### 3.5 代理视图 Agent

从 osaurus MCP + 身份系统吸收:

```
┌────────────────────────────────────────────────┐
│  🚀 运行链       配置                          │
│                                                │
│  ┌──────────────────────────────────────┐      │
│  │  ⏺ uptime: 2h                       │      │
│  │  🔵 12 nodes  🟢 8 healthy  ⚠ 2 high-lat │   │  ← Hero 环
│  │  ⚡ 42 req/min  📊 98% success      │      │
│  └──────────────────────────────────────┘      │
│                                                │
│  E8 → SAE → GWT → L7 → L1                     │  ← 谐振链可视化
│  ● → ● → ● → ● → ● (全部完成, φ=0.91)         │
│                                                │
│  ┌──────────┬──────────┬──────────┬──────────┐ │
│  │ 📊概览   │ 🗺地图   │ 🔌节点    │ 📋订阅   │ │
│ ├──────────┴──────────┴──────────┴──────────┤ │
│ │  MCP 服务器列表                            │ │
│ │  🟢 osaurus.browser    v1.2.3  local      │ │
│ │  🟢 osaurus.filesystem v2.0.1  local      │ │
│ │  🟡 github-mcp-server   v0.5.0  remote    │ │
│ │  🔴 linear-mcp-server   unreachable        │ │
│ │  [添加 MCP 服务器...]                      │ │
│ └────────────────────────────────────────────┘ │
└────────────────────────────────────────────────┘
```

#### Agent 元素规范

| 元素 | 动作 | 参数 | 实现 |
|------|------|------|------|
| 🚀 启动/停止 | 切换守护进程 | `daemonRunning: bool` | `toggleProxyDaemon()` |
| 配置 | 打开设置 → 计算资源 | — | `openAgentConfig()` |
| Hero 环 | 实时状态 | `uptime, nodes, health, req/min` | `tickProxy()` 900ms 轮询 |
| 谐振链 | 步骤可视化 | `steps[], activeStep` | 动画显示 E8→SAE→GWT→... |
| 概览/地图/节点/订阅 tabs | 切换 | `tab: string` | `switchPxTab()` |
| MCP 服务器列表 | 显示/管理 | `servers[]` | 从 `mcp_discovery` 读取 |
| 添加 MCP 服务器 | 输入 URL/命令 | `url/command` | `addSubscription()` |
| 节点健康 | 颜色指示 | `status: 'healthy'/'high-lat'/'down'` | 🟢🟡🔴 |

### 3.6 用户弹出层 UserPopover

```
┌─────────────────────┐
│  ┌──┐               │
│  │N │  Neo           │  ← 头像 + 显示名称
│  └──┘               │
│─────────────────────│
│  ⚙️ 设置             │  → SettingsModal
│  🔄 主题切换          │  → light/dark/system 循环
│  📖 帮助             │  → 帮助弹窗/跳转
│─────────────────────│
│  🚪 退出登录          │  → 确认 → 清空会话
└─────────────────────┘
```

#### Popover 元素规范

| 元素 | 动作 | 参数 | 实现 |
|------|------|------|------|
| 头像 + 名称 | 显示用户信息 | `avatar, displayName` | 从 `appState.userProfile` 读取 |
| 设置 | 打开设置 | — | `openSettingsModal()` |
| 主题 | 循环切换 | `theme: 'light'→'dark'→'system'` | `cycleTheme()` |
| 帮助 | 打开帮助 | — | 帮助覆盖层 / 链接 |
| 退出 | 确认后退出 | — | `confirm('确定退出?')` → 清空 |

### 3.7 设置模态框 SettingsModal

从 NovaChat 5-tab 结构 + osaurus 隐私吸收:

```
┌────────────────────────────────────────────────┐
│  🟠 🟡 🟢                                       │
│  ┌─────────┐  ┌──────────────────────────────┐ │
│  │ 通用     │  │  API Key                     │ │
│  │ API      │  │  ┌────────────────────────┐  │ │
│  │ 模型     │  │  │ ●●●●●●●●●●        [显示]│  │ │
│  │ 隐私     │  │  └────────────────────────┘  │ │
│  │ 快捷键   │  │  [✅ 验证]                   │  │ │
│  │ 关于     │  │  ✓ Key 有效 · Anthropic     │  │ │
│  │         │  └──────────────────────────────┘ │
│  └─────────┘                                    │
└────────────────────────────────────────────────┘
```

#### 设置 Tab 规范

| Tab | 设置项 | 类型 | 默认值 | 持久化 |
|-----|--------|------|--------|--------|
| **通用** | 主题模式 | select (浅色/深色/跟随系统) | light | `localStorage` |
| | 字体大小 | select (小/中/大) | 中 | `localStorage` |
| | 语言 | select (中文/English) | 中文 | `localStorage` |
| **API** | API Key | password input + show toggle | '' | Rust `keyring` crate |
| | 验证 | button → 绿色对勾/红色错误 | — | — |
| **模型** | 默认模型 | text input + 下拉建议 | GatewayV2 | `localStorage` |
| | Temperature | slider (0-2, 步进 0.1) | 0.7 | `localStorage` |
| | Max Tokens | select (4096/8192/16384/32768) | 8192 | `localStorage` |
| **隐私** | 对话存储 | checkbox | true | `localStorage` |
| | 使用数据收集 | checkbox | false | `localStorage` |
| | 本地处理优先 | checkbox | true | `localStorage` |
| | 发送前审查 | checkbox | true | `localStorage` (从 osaurus) |
| **快捷键** | 全部快捷键 | 只读列表 | — | — |
| **关于** | 版本/许可/鸣谢 | 只读 | — | — |

---

## 4. 完整快捷键表

| 快捷键 | 行为 | 来源 |
|--------|------|------|
| `Cmd/Ctrl + N` | 新建会话 | NovaChat |
| `Cmd/Ctrl + K` | 打开会话搜索 | NovaChat |
| `Cmd/Ctrl + ,` | 打开设置 | NovaChat |
| `Cmd/Ctrl + B` | 折叠/展开侧栏 | NovaChat |
| `Cmd/Ctrl + Shift + M` | 打开管理窗口 | osaurus |
| `Esc` | 停止 AI 生成 / 关闭弹窗 | NovaChat |
| `↑` (输入框为空) | 载入上一条用户消息 | NovaChat |
| `Cmd/Ctrl + Enter` | 换行 (替代 Shift+Enter) | NovaChat 互补 |
| `Cmd/Ctrl + Shift + C` | 复制最后代码块 | 新增 |

---

## 5. 数据模型 (完整合并)

```typescript
// 全局设置 (AppSettings)
interface AppSettings {
  theme: 'light' | 'dark' | 'system';
  fontSize: 'sm' | 'md' | 'lg';
  language: 'zh' | 'en';
  defaultModel: string;
  temperature: number;
  maxTokens: number;
  privacyStoreMessages: boolean;
  privacyTelemetry: boolean;
  privacyLocalFirst: boolean;
  privacyPreflightCheck: boolean;   // 从 osaurus: 发送前隐私审查
}

// 用户信息
interface UserProfile {
  displayName: string;
  avatarInitial: string;
  // apiKey 永不在前端存在
}

// 会话
interface Conversation {
  id: string;
  title: string;
  model: string;
  pinned: boolean;
  createdAt: number;
  updatedAt: number;
  messageCount: number;
}

// 消息
interface Message {
  id: string;
  conversationId: string;
  role: 'user' | 'assistant';
  content: string;
  attachments?: Attachment[];
  status: 'pending' | 'streaming' | 'complete' | 'error';
  errorType?: '401' | '429' | '529' | 'network';
  errorMessage?: string;
  createdAt: number;
}

// 附件
interface Attachment {
  id: string;
  fileName: string;
  mimeType: string;
  sizeBytes: number;
  localPath?: string;
  base64Data?: string;
}

// 团队会话 (从 osaurus Agent Loop)
interface CoworkSession {
  id: string;
  name: string;
  status: 'active' | 'paused' | 'completed';
  tasks: CoworkTask[];
  agents: AgentSlot[];
  createdAt: number;
}

interface CoworkTask {
  id: string;
  title: string;
  status: 'todo' | 'in_progress' | 'done';
  assignee: string;        // agent id
  subtasks: Subtask[];
}

interface AgentSlot {
  id: string;
  name: string;
  status: 'online' | 'busy' | 'offline';
  lastActive: number;
}

// MCP 服务器 (从 osaurus)
interface McpServer {
  id: string;
  name: string;
  version: string;
  transport: 'stdio' | 'http' | 'sse';
  status: 'healthy' | 'unstable' | 'down';
  tools: string[];
}
```

---

## 6. 持久化方案

| 数据类型 | 存储 | 位置 |
|---------|------|------|
| AppSettings | `localStorage.setItem('appSettings', JSON.stringify(...))` | 前端 |
| UserProfile | `localStorage.setItem('userProfile', JSON.stringify(...))` | 前端 |
| API Key | Rust `keyring` crate → macOS Keychain | Rust 后端 |
| Conversations | `localStorage.setItem('conversations', JSON.stringify(...))` | v1 前端 → v2 Rust SQLite |
| Messages | `localStorage.setItem('messages_{convId}', JSON.stringify(...))` | v1 前端 → v2 Rust SQLite |
| CoworkSessions | `localStorage.setItem('coworkSessions', JSON.stringify(...))` | v1 前端 |
| McpServers | `localStorage.setItem('mcpServers', JSON.stringify(...))` | v1 前端 |

---

## 7. Communication Architecture

```
┌─────────────────────────────────────────────────┐
│                  Frontend (JS)                    │
│                                                   │
│  invoke('send_message', {convId, content, model}) │
│       ↓ (Tauri IPC)                               │
│  Rust Backend                                      │
│  ├─ keyring::get_password("anthropic_api_key")    │
│  ├─ assemble request (model, messages, stream)    │
│  ├─ POST https://api.anthropic.com/v1/messages    │
│  ├─ Parse SSE stream                              │
│  └─ emit('stream-chunk', {messageId, delta})      │
│       ↑ (Tauri event)                              │
│  Frontend listens → appendToMessage()              │
└─────────────────────────────────────────────────┘
```

---

## 8. 实现路线图 (优先级分期)

### P0 — Chat MVP (核心对话循环)

| # | 任务 | 描述 | 文件 |
|---|------|------|------|
| 1 | 消息持久化 | `sendMsg()` 保存到 `localStorage`，加载时恢复 | HTML JS `sendMsg()` + init |
| 2 | 会话管理 | 新建/切换/删除/重命名会话，列表渲染 | HTML JS `renderSidebar()` + actions |
| 3 | 会话搜索 | 搜索框实时过滤会话列表 | HTML JS search handler |
| 4 | 附件上传 | 隐藏 `<input type=file>` + 拖拽区域 + chip UI | HTML DOM + JS |
| 5 | 输入框完整 | Enter/Shift+Enter, 自动增高, 发送/停止切换 | HTML JS `handleKey()` |
| 6 | 设置持久化 | 所有设置读写 `localStorage` + 应用到 DOM | HTML JS settings + appState |

### P1 — 流式 + 渲染

| # | 任务 | 描述 | 文件 |
|---|------|------|------|
| 1 | 流式 IPC | `invoke('send_message')` → Rust → SSE → `emit('stream-chunk')` | HTML + Rust `commands/chat.rs` |
| 2 | 流式渲染 | Token 追加到消息气泡，节流 16ms | HTML JS `listen('stream-chunk')` |
| 3 | Markdown 渲染 | `marked` + `remark-gfm` 集成 | HTML JS 渲染函数 |
| 4 | 代码高亮 | `highlight.js` 集成，语言标签 + 复制按钮 | HTML JS 渲染函数 |
| 5 | 消息操作条 | hover 显示复制/重新生成/好评/差评 | HTML CSS + JS |
| 6 | 三态主题 | light/dark/system + `prefers-color-scheme` 监听 | HTML JS `toggleTheme()` |

### P1 — Key 管理

| # | 任务 | 描述 | 文件 |
|---|------|------|------|
| 1 | 设置 → API tab | Key 输入框 + 显示切换 + 验证按钮 | HTML settings |
| 2 | Rust keyring | `save_api_key` / `has_api_key` / `get_api_key` 命令 | Rust `commands/keychain.rs` |
| 3 | 401 错误处理 | API Key 无效 → toast + 设置跳转按钮 | HTML JS message status |

### P2 — 团队 + 代码视图

| # | 任务 | 描述 | 文件 |
|---|------|------|------|
| 1 | Cowork 持久化 | 会话/任务/子任务 CRUD → `localStorage` | HTML JS `CW_DATA` 重构 |
| 2 | Cowork Agent 状态 | 🟢🟡🔴 状态指示器 + 轮询 | HTML JS |
| 3 | Code 文件树 | 从 mock → `invoke('fs_tree')` → 可交互树 | HTML JS `renderFileTree()` |
| 4 | Code 保存/运行 | `invoke('write_file')` + `invoke('execute_code')` | HTML JS + Rust |
| 5 | Code 输出面板 | 编译/运行结果 → 下方控制台风格面板 | HTML DOM |

### P2 — 错误处理

| # | 任务 | 描述 | 文件 |
|---|------|------|------|
| 1 | 消息状态 enum | `pending/streaming/complete/error` | HTML JS message 模型 |
| 2 | 429 重试 | 倒计时 + 自动退避 | HTML JS stream handler |
| 3 | 529 手动重试 | 错误消息内联重试按钮 | HTML JS message render |
| 4 | 网络错误 | 断网检测 + 重发按钮 | HTML JS |

### P3 — 代理 + Artifact

| # | 任务 | 描述 | 文件 |
|---|------|------|------|
| 1 | MCP 服务器管理 | 列表 + 添加 + 移除 + 健康轮询 | HTML JS `renderNodeTable()` |
| 2 | Agent 实时统计 | Hero 环 + 节点地图 + 速率实时更新 | HTML JS `tickProxy()` |
| 3 | Artifact Panel | 代码块自动检测 → 面板滑出 → 预览/代码切换 | HTML JS + CSS |
| 4 | 快捷键完整 | Cmd+N, Cmd+K, Cmd+B, Esc, ↑, Cmd+Shift+C | HTML JS 全局 keydown |
| 5 | 键盘导航 | Tab/方向键/Enter 在会话列表导航 | HTML JS |
| 6 | 帮助弹窗 | 快捷键列表 + 功能说明 | HTML overlay |

### P4 — 高级 (从 osaurus)

| # | 任务 | 描述 |
|---|------|------|
| 1 | 隐私过滤器 | 发送前正则检测 PII + 审查 UI + fail-closed |
| 2 | 加密身份 | secp256k1 + keychain master key + 访问密钥 |
| 3 | 插件系统 | MCP 工具注册表 + 热重载 |
| 4 | 语音输入 | Web Speech API → Rust ANE bridge |
| 5 | 沙箱 VM | Apple Containerization 完整集成 |
| 6 | 安全信道 | X25519 + ChaCha20-Poly1305 E2EE |

---

## 9. 当前代码库差距分析

| 区域 | 当前状态 | P0 目标 | 工作量 |
|------|---------|---------|--------|
| 设置持久化 | 全部不持久，刷新丢失 | 全部 `localStorage` 读写 | 小 (0.5d) |
| 消息持久化 | 不保存，刷新丢失 | `localStorage` 保存/加载 | 小 (0.5d) |
| 会话管理 | mock 数据，不会话切换 | 新建/切换/删除/重命名 | 中 (1d) |
| 流式渲染 | 无，setTimeout mock | token-by-token + IPC | 大 (2d + Rust) |
| Markdown | 无 | `marked` + `highlight.js` | 中 (1d) |
| 附件 | 无 handler | 文件选择 + chip UI | 小 (0.5d) |
| 主题三态 | 仅 light/dark | light/dark/system | 小 (0.25d) |
| 快捷键 | 仅 Cmd+, | 7+ 快捷键 | 小 (0.5d) |
| 消息操作条 | 无 | hover 复制/重新生成/赞/踩 | 中 (1d) |
| 错误处理 | 无 | 401/429/529 状态 + UI | 中 (1d) |
| Cowork 持久化 | mock，刷新丢失 | `localStorage` CRUD | 小 (0.5d) |
| Code 运行 | toast 仅 | IPC 执行 + 输出面板 | 中 (1d + Rust) |
| MCP 管理 | mock 节点表 | 真实注册表 + 健康轮询 | 中 (1d) |

**总计 P0**: ~2 天 (纯前端, 无 Rust 依赖)
**总计 P1**: ~4 天 (需要 Rust IPC 集成)
**总计 P2**: ~3 天 (前端为主)
**总计 P3-P4**: 架构依赖, 需 Rust 后端就绪

---

## 10. 优先执行计划

### 第 1 步 (当前): 架构清理 + 持久化
- [x] 删除死 CSS (已完成)
- [x] 添加 `appState` + `viewState` 对象
- [ ] 所有设置 → `localStorage` 读写
- [ ] 消息保存 → `localStorage`

### 第 2 步: 会话管理 + 搜索
- [ ] `conversations` CRUD
- [ ] 会话列表按时间分组渲染
- [ ] 搜索框实时过滤

### 第 3 步: 输入/附件 + Send 流程
- [ ] 完整 InputBox (Enter/Shift+Enter/自动增高)
- [ ] 附件选择 + chip
- [ ] Send/Stop 切换

### 第 4 步: 主题 + 快捷键 + 消息操作条
- [ ] 三态主题
- [ ] 完整快捷键
- [ ] hover 操作条

### 第 5 步: 设置模态框完成
- [ ] API Key tab (前端 + Rust)
- [ ] 模型 tab
- [ ] 快捷键 tab

### 第 6 步: Rust IPC 集成
- [ ] Stream message
- [ ] Markdown 渲染
- [ ] 错误处理

### 第 7 步: Cowork + Code
- [ ] 持久化 + Agent 状态
- [ ] 文件树 + 保存/运行

### 第 8 步: Agent + Artifact
- [ ] MCP 管理
- [ ] 实时统计
- [ ] Artifact Panel
