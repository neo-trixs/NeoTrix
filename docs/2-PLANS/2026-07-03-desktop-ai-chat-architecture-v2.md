# Desktop AI Chat Application — Architecture V2

> **Synthesis**: NovaChat 设计规格文档 + osaurus (Swift 原生 AI 框架) + emilkowalski/skills (动效设计工程) + 桌面应用 UI/UX 最佳实践 + Tauri 2.x 生产模式

---

## 目录

1. [架构总览](#1-架构总览)
2. [目录结构（最终版）](#2-目录结构最终版)
3. [设计系统（Design Tokens v2）](#3-设计系统)
4. [组件树与层级](#4-组件树与层级)
5. [数据流与 IPC 边界](#5-数据流与-ipc-边界)
6. [状态管理](#6-状态管理)
7. [动画与动效体系](#7-动画与动效体系)
8. [主题系统](#8-主题系统)
9. [离线架构](#9-离线架构)
10. [开发路线图（细化分期）](#10-开发路线图细化分期)
11. [安全模型](#11-安全模型)
12. [关键技术选型论证](#12-关键技术选型论证)
13. [给实现者的落地检查清单](#13-落地检查清单)

---

## 1. 架构总览

```
┌──────────────────────────────────────────────────────────────────┐
│                         Tauri 2.x Shell                          │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                 React 18 + TypeScript                      │   │
│  │  ┌───────────┐  ┌──────────────────┐  ┌────────────────┐ │   │
│  │  │  Sidebar  │  │   Chat Area      │  │ Artifact Panel │ │   │
│  │  │  (260px)  │  │    (flex: 1)     │  │  (420px, 可选)  │ │   │
│  │  │  ┌──────┐ │  │  ┌────────────┐  │  │                │ │   │
│  │  │  │Login │ │  │  │  TopBar    │  │  │  代码/文档预览   │ │   │
│  │  │  │      │ │  │  ├────────────┤  │  │                │ │   │
│  │  │  │Search│ │  │  │MessageList │  │  │                │ │   │
│  │  │  │      │ │  │  ├────────────┤  │  │                │ │   │
│  │  │  │Conv  │ │  │  │ InputBox   │  │  │                │ │   │
│  │  │  │List  │ │  │  └────────────┘  │  │                │ │   │
│  │  │  │      │ │  │                  │  │                │ │   │
│  │  │  │ ⚙️   │ │  │  [PetBar]        │  │                │ │   │
│  │  │  └──────┘ │  └──────────────────┘  └────────────────┘ │   │
│  │  └───────────┘                                           │   │
│  └──────────────────────────────────────────────────────────┘   │
│                          ▲ invoke / ▼ events                     │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                    Rust Backend (Tauri)                    │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  │   │
│  │  │ commands │  │    db    │  │  anthropic│  │ keychain │  │   │
│  │  │  chat    │  │  sqlite  │  │  client   │  │  keyring │  │   │
│  │  │  convs   │  │          │  │  (SSE)    │  │          │  │   │
│  │  │  files   │  │          │  │           │  │          │  │   │
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘  │   │
│  └──────────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────────┘
```

### 核心设计原则

| 原则 | 说明 |
|------|------|
| **API 请求在 Rust 后端** | API Key 仅存在于 Rust 进程与系统密钥库之间，前端 JS 永不明文接触 Key |
| **离线优先持久化** | SQLite 本地存储，断网可查看历史对话全文 |
| **CSS 变量为设计系统基石** | 所有颜色/圆角/阴影仅通过 CSS 自定义属性引用，零硬编码 |
| **Zustand 高频写入无 Provider** | 流式更新直接 `set()`, 无需 Provider 嵌套 |
| **动画有据可依** | 每个动效有 raison d'être, 遵守频率/时长/缓动规范 |

---

## 2. 目录结构（最终版）

```
novachat/
├── src-tauri/                       # Rust 后端
│   ├── src/
│   │   ├── main.rs                  # Tauri 入口，注册 commands + plugins
│   │   ├── commands/
│   │   │   ├── mod.rs
│   │   │   ├── chat.rs              # send_message / stop_generation
│   │   │   ├── conversations.rs     # CRUD + search + pin + rename
│   │   │   ├── keychain.rs          # save / has / delete API Key
│   │   │   └── files.rs             # read_attachment / temp_path
│   │   ├── anthropic/
│   │   │   ├── mod.rs
│   │   │   ├── client.rs            # HTTP client: POST /v1/messages
│   │   │   ├── stream.rs            # SSE 解析，逐 token 回调
│   │   │   └── types.rs             # Request / Response 类型
│   │   ├── db/
│   │   │   ├── mod.rs
│   │   │   ├── schema.sql           # DDL
│   │   │   ├── conversations.rs     # SQL 操作
│   │   │   └── migrations.rs        # 未来 schema 升级
│   │   └── errors.rs                # 统一错误类型 → 前端序列化
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                             # 前端 React
│   ├── main.tsx
│   ├── App.tsx                      # 路由/布局壳
│   ├── components/
│   │   ├── Sidebar/
│   │   │   ├── Sidebar.tsx
│   │   │   ├── Sidebar.module.css
│   │   │   ├── ConversationList.tsx
│   │   │   ├── ConversationItem.tsx
│   │   │   ├── SearchBox.tsx
│   │   │   └── SidebarFooter.tsx
│   │   ├── Chat/
│   │   │   ├── ChatArea.tsx          # 消息列表 + 输入框组合
│   │   │   ├── TopBar.tsx
│   │   │   ├── MessageList.tsx
│   │   │   ├── MessageBubble.tsx
│   │   │   ├── MarkdownRenderer.tsx
│   │   │   ├── CodeBlock.tsx
│   │   │   ├── EmptyState.tsx
│   │   │   └── StreamingCursor.tsx
│   │   ├── InputBox/
│   │   │   ├── InputBox.tsx
│   │   │   ├── AttachmentChip.tsx
│   │   │   └── ModelSelector.tsx
│   │   ├── ArtifactPanel/
│   │   │   └── ArtifactPanel.tsx
│   │   ├── Settings/
│   │   │   ├── SettingsModal.tsx
│   │   │   ├── GeneralTab.tsx
│   │   │   ├── ApiKeyTab.tsx
│   │   │   └── ShortcutsTab.tsx
│   │   ├── PetBar/
│   │   │   ├── PetBar.tsx           # Canvas 桌面宠物
│   │   │   └── PetBar.module.css
│   │   └── ui/                      # 共享设计系统组件
│   │       ├── Button.tsx
│   │       ├── Dialog.tsx
│   │       ├── Dropdown.tsx
│   │       ├── Kbd.tsx              # 键盘快捷键标记
│   │       ├── Tooltip.tsx
│   │       └── ScrollArea.tsx       # 自定义滚动容器
│   ├── store/
│   │   ├── useConversationStore.ts  # 会话 + 消息状态
│   │   ├── useSettingsStore.ts      # 主题/模型/偏好
│   │   ├── useUIStore.ts            # 侧栏折叠/面板显隐/输入状态
│   │   └── useStreamingStore.ts     # 流式消息增量缓存
│   ├── hooks/
│   │   ├── useStreaming.ts          # listen stream-chunk + append
│   │   ├── useAutoScroll.ts         # 自动/暂停滚动逻辑
│   │   ├── useTheme.ts              # 三态主题管理
│   │   ├── useKeyboard.ts           # 全局快捷键注册
│   │   └── useReduceMotion.ts       # prefers-reduced-motion 检测
│   ├── styles/
│   │   ├── globals.css              # CSS 变量 + 基础重置
│   │   └── tailwind.css
│   ├── types/
│   │   └── index.ts                 # Conversation / Message / Attachment / Settings
│   └── lib/
│       ├── tauri.ts                 # invoke / listen 封装
│       ├── markdown.ts              # 解析/节流辅助
│       └── date.ts                  # 会话分组日期逻辑
├── public/
│   └── icon.svg
├── package.json
├── tsconfig.json
├── vite.config.ts
└── index.html
```

---

## 3. 设计系统

### 3.1 Design Tokens（CSS 自定义属性）

所有颜色/圆角/阴影定义在 `globals.css`，组件引用变量。

```css
/* globals.css — 仅通过 data-theme 切换，组件零硬编码 */

:root {
  /* 背景 */
  --bg-primary: #FAF9F6;
  --bg-sidebar: #F0EEE6;
  --bg-elevated: #FFFFFF;
  --bg-user-bubble: #EFEAE0;
  --bg-hover: rgba(0,0,0,0.04);

  /* 边框 */
  --border-subtle: #E5E2D9;
  --border-active: #D0CCC1;

  /* 文字 */
  --text-primary: #1F1E1C;
  --text-secondary: #6B6862;
  --text-tertiary: #9C9A8F;

  /* 强调色 — 陶土橙 palette */
  --accent-primary: #C96442;
  --accent-hover: #B35A3A;
  --accent-muted: rgba(201,100,66,0.10);
  --accent-text: #FFFFFF;

  /* 语义色 */
  --color-success: #4F7942;
  --color-danger: #C0392B;
  --color-warning: #D4A24C;

  /* 代码 */
  --code-bg: #F5F2EA;
  --code-text: #1F1E1C;

  /* 字体 */
  --font-ui: Inter, -apple-system, "Segoe UI", sans-serif;
  --font-mono: "JetBrains Mono", "Fira Code", ui-monospace, monospace;

  /* 字号 */
  --text-xs: 12px;
  --text-sm: 13px;
  --text-base: 15px;
  --text-lg: 17px;
  --text-xl: 20px;
  --text-2xl: 24px;
  --text-3xl: 30px;

  /* 行高 */
  --leading-tight: 1.3;
  --leading-normal: 1.5;
  --leading-relaxed: 1.65;

  /* 间距 */
  --space-1: 4px;
  --space-2: 8px;
  --space-3: 12px;
  --space-4: 16px;
  --space-5: 20px;
  --space-6: 24px;
  --space-8: 32px;
  --space-10: 40px;
  --space-16: 64px;

  /* 圆角 */
  --radius-sm: 6px;
  --radius-md: 10px;
  --radius-lg: 16px;
  --radius-full: 9999px;

  /* 阴影 */
  --shadow-sm: 0 1px 2px rgba(0,0,0,0.05);
  --shadow-md: 0 4px 12px rgba(0,0,0,0.08);
  --shadow-lg: 0 12px 28px rgba(0,0,0,0.10);
  --shadow-modal: 0 20px 40px rgba(0,0,0,0.16);

  /* 磨砂玻璃（用于浮层/面板） */
  --glass-bg: rgba(255,255,255,0.72);
  --glass-border: rgba(255,255,255,0.30);
  --glass-shadow: 0 8px 32px rgba(0,0,0,0.08);
  --glass-blur: 32px;   /* 性能推荐值 */
}

[data-theme="dark"] {
  --bg-primary: #211F1C;
  --bg-sidebar: #181713;
  --bg-elevated: #2A2925;
  --bg-user-bubble: #33312B;
  --bg-hover: rgba(255,255,255,0.06);

  --border-subtle: #3A382F;
  --border-active: #4F4C42;

  --text-primary: #EDEBE4;
  --text-secondary: #9C9A8F;
  --text-tertiary: #6B6862;

  --accent-primary: #D97757;
  --accent-hover: #E08A6C;
  --accent-muted: rgba(217,119,87,0.12);
  --accent-text: #FFFFFF;

  --code-bg: #1C1B17;
  --code-text: #EDEBE4;

  --shadow-sm: 0 1px 2px rgba(0,0,0,0.20);
  --shadow-md: 0 4px 12px rgba(0,0,0,0.30);
  --shadow-lg: 0 12px 28px rgba(0,0,0,0.40);
  --shadow-modal: 0 20px 40px rgba(0,0,0,0.50);

  --glass-bg: rgba(42,41,37,0.80);
  --glass-border: rgba(255,255,255,0.08);
  --glass-shadow: 0 8px 32px rgba(0,0,0,0.25);
}
```

### 3.2 磨砂玻璃（Glassmorphism）规范

| 使用场景 | 规范 |
|---------|------|
| 浮层面板 / 弹出菜单 | `--glass-bg` + `--glass-blur` + `--glass-border` |
| Artifact Panel（浮动模式） | 同上，带 `--glass-shadow` |
| 性能考量 | `backdrop-filter: blur(32px)` ≤ 32px 性能最优；慎用 `saturate()` |

### 3.3 字体与排版

| 层级 | 字号 | 字重 | 行高 | 用途 |
|------|------|------|------|------|
| body | 15px | 400 | 1.65 | 消息正文 |
| body-small | 13px | 400 | 1.5 | 次要文字/时间戳 |
| heading-h1 | 24px | 600 | 1.3 | 欢迎页标题 |
| heading-h2 | 20px | 600 | 1.3 | 弹窗标题 |
| code | 13px | 400 | 1.5 | 行内代码/代码块 |
| label | 12px | 500 | 1.3 | 按钮/标签 |
| caption | 12px | 400 | 1.4 | 辅助说明 |

---

## 4. 组件树与层级

```
App
├── ThemeProvider           # 监听 prefers-color-scheme, 设置 data-theme
├── KeyboardProvider        # 全局快捷键注册
├── Sidebar
│   ├── SidebarHeader       # Logo + NewChat 按钮
│   ├── SearchBox
│   ├── ConversationList
│   │   └── ConversationItem[]  # title + 时间 + hover 操作
│   └── SidebarFooter       # Settings 入口
├── ChatArea                # flex:1, 三态路由（empty / conversation / loading）
│   ├── TopBar
│   │   ├── ModelSelector
│   │   └── OverflowMenu
│   ├── MessageList
│   │   ├── EmptyState      # 欢迎页（非会话状态）
│   │   ├── MessageBubble[]
│   │   │   ├── MarkdownRenderer
│   │   │   │   ├── CodeBlock
│   │   │   │   └── StreamingCursor (尾部闪烁块)
│   │   │   └── MessageActions (hover 浮现)
│   │   └── ScrollToBottom  # 浮动按钮
│   ├── InputBox
│   │   ├── TextArea        # 自动增高
│   │   ├── AttachmentChip[]
│   │   └── SendButton / StopButton
│   └── PetBar              # Canvas 桌面宠物
└── [ArtifactPanel]         # 可选浮层
```

### 4.1 组件状态矩阵

| 组件 | 状态 | 覆盖 |
|------|------|------|
| Sidebar | expanded / collapsed(64px icon mode) / hidden(<900px) | hover 展开覆盖层 |
| ConversationList | loading / empty / populated / search-results / search-empty | |
| MessageList | empty(welcome) / loading(thinking dots) / streaming / complete / error | |
| InputBox | idle / has-text / has-attachments / disabled(AI generating) / error | |
| StreamingCursor | hidden / waiting(dots) / active(blink) / stopped | |
| ArtifactPanel | hidden / docked(420px) / floating(<1280px) / collapsed | |
| PetBar | visible / collapsed(scaleY 0.37) / day / night | |

---

## 5. 数据流与 IPC 边界

### 5.1 核心消息流（流式对话）

```
User types + Enter
  │
  ▼
InputBox
  │ dispatch('send')
  ▼
useConversationStore.sendMessage(content, attachments)
  │ invoke('send_message', { conversationId, messages, model })
  │ listen('stream-chunk', handler)
  ▼  ──────────────────────────────────────────────────────
Rust Backend                                              │
  │                                                        │
  ▼                                                        │
commands/chat.rs                                           │
  │ 1. keyring.get_password("anthropic_api_key")           │
  │ 2. build request body (messages + system)              │
  │ 3. POST https://api.anthropic.com/v1/messages          │
  │                                                         │
  ▼ ← SSE stream                                            │
stream.rs:                                                 │
  │ for each content_block_delta:                          │
  │   app_handle.emit("stream-chunk", {messageId, delta})  │
  │ on final message_stop:                                 │
  │   app_handle.emit("stream-done", {messageId})          │
  │                                                         │
  └───────────────────────────────────────────────────────────
                                                           │
  ▼  ← 前端事件循环                                          │
useStreaming hook                                          │
  │ on 'stream-chunk': appendToMessage(id, delta)          │
  │ on 'stream-done': markComplete(id)                     │
  │ throttled to 16ms frames                                │
  ▼                                                        │
MessageList re-renders with new content                     │
```

### 5.2 IPC 命令清单

| invoke 命令 | 参数 | 返回 | 说明 |
|-------------|------|------|------|
| `send_message` | conversationId, content, attachments?, model | messageId (streaming) | 发起流式请求 |
| `stop_generation` | conversationId | () | 中止当前请求 |
| `list_conversations` | search? | Conversation[] | 列表（可分页） |
| `create_conversation` | title?, model | Conversation | |
| `get_conversation` | id | Conversation + Message[] | 完整加载 |
| `rename_conversation` | id, title | () | |
| `delete_conversation` | id | () | 级联删除消息 |
| `pin_conversation` | id, pinned | () | |
| `save_api_key` | key | () | 写入系统密钥库 |
| `has_api_key` | — | boolean | |
| `delete_api_key` | — | () | |
| `read_attachment_file` | path | base64 + mime | 读取附件二进制 |
| `load_model_list` | — | string[] | 模型列表建议 |

### 5.3 事件清单（后端 → 前端）

| 事件 | Payload | 说明 |
|------|---------|------|
| `stream-chunk` | { messageId, delta } | 增量 token |
| `stream-done` | { messageId, stopReason } | 流完成 |
| `stream-error` | { messageId, errorCode, message } | 流错误 |

### 5.4 限流与合并策略

- **16ms 帧合并**：`stream-chunk` 事件高频到达时，前端以 `requestAnimationFrame` 同步合并增量再 `setState`
- 避免逐字符 React 重渲染
- Rust 端不做节流（SSE 原生已是按行推送）

---

## 6. 状态管理

### 6.1 Zustand Store 拆分

| Store | 职责 | 高频更新？ | 持久化？ |
|-------|------|-----------|---------|
| `useConversationStore` | 当前会话列表 + 激活会话 + 消息 map | 流式时高 | 否（通过 IPC 同步 Rust） |
| `useSettingsStore` | theme / fontSize / defaultModel / temperature | 低 | tauri-plugin-store |
| `useUIStore` | sidebarOpen / artifactOpen / searchQuery / inputValue | 中 | 否 |
| `useStreamingStore` | 流式消息增量缓存（尚未写入 store） | 极高（帧级） | 否 |

### 6.2 流式更新策略

```
useStreamingStore (per-message delta buffer)
  │ 16ms rAF sync
  ▼
useConversationStore (immutable message content)
  │
  ▼
MessageList re-render
```

- 流式过程中，增量 token 首先写入 `useStreamingStore.deltaBuffer[messageId]`
- 每帧（`requestAnimationFrame`）合并到 `useConversationStore.messages[last].content`
- 流式完成后，一次性调用 `invoke('append_message', ...)` 持久化到 SQLite

### 6.3 Tauri 2.x 状态管理（Rust 侧）

```rust
// main.rs — 使用 Mutex 包裹可变状态，Tauri 2 自动提供 Arc
use std::sync::Mutex;

struct ApiClient {
    client: reqwest::Client,
}

struct ActiveStream {
    abort: tokio_util::sync::CancellationToken,
}

fn main() {
    tauri::Builder::default()
        .manage(Mutex::new(ApiClient::new()))
        .invoke_handler(tauri::generate_handler![...])
        .run(tauri::generate_context!())
}
```

> Tauri 2 的 `State<T>` 内部使用 `Arc<T>`, 因此外层不需要包 `Arc`。对于 `TcpStream` 等非 `Send` 类型使用 `Mutex`。

---

## 7. 动画与动效体系

### 7.1 动画设计哲学（吸收自 emilkowalski/skills）

**每帧动画必须通过以下自检**：

1. **此动画解决了什么问题？** — 没理由的动画不要加
2. **频率是否合适？** — 高频操作（hover → 即时应答）用快速过渡（80-120ms）；低帧率（面板展开 → 200-300ms）
3. **缓动是否正确？** — 进入用 `ease-out`（快入慢出），退出用 `ease-in`（慢入快出）
4. **是否有对应反行动画？** — 展开 ↔ 收起、出现 ↔ 消失 应对称
5. **是否尊重 reduce-motion？** — 命中时直接跳变（transition: none）

### 7.2 动画参数表

| 场景 | 属性 | 时长 | 缓动 | 位移 |
|------|------|------|------|------|
| 消息淡入 | opacity + translateY | 120ms | ease-out | 8px |
| 侧栏折叠 | width | 200ms | ease-out | — |
| 面板滑出 | opacity + translateX | 250ms | ease-out | 16px |
| 弹窗出现 | opacity + scale | 200ms | ease-out | scale(0.95→1) |
| 按钮悬停 | background-color | 80ms | ease-out | — |
| 输入框聚焦 | box-shadow | 120ms | ease-out | — |
| 流光光标闪烁 | opacity | 800ms | steps(2) | — |
| 思考三点跳动 | opacity | 400ms | ease-in-out | — |

### 7.3 CSS 过渡实用类

```css
/* 预定义 transition 类，组件直接引用 */
.transition-fast {
  transition-duration: 80ms;
  transition-timing-function: cubic-bezier(0.25, 0.1, 0.25, 1);
}
.transition-normal {
  transition-duration: 200ms;
  transition-timing-function: cubic-bezier(0.25, 0.1, 0.25, 1);
}
.transition-slow {
  transition-duration: 300ms;
  transition-timing-function: cubic-bezier(0.25, 0.1, 0.25, 1);
}
```

---

## 8. 主题系统

### 8.1 三态切换

```
useSettingsStore.theme: 'light' | 'dark' | 'system'
  │
  ▼
useTheme hook:
  │ 'light' → html.dataset.theme = 'light'
  │ 'dark'  → html.dataset.theme = 'dark'
  │ 'system' → matchMedia('prefers-color-scheme: dark')
  │            实时监听 change 事件切换
  │            初次调用先读取当前值
```

### 8.2 实现要点

- 初始渲染时，读取 `useSettingsStore.theme`（从 `tauri-plugin-store` 加载）
- 如果 `theme='system'`, 在 `<head>` 注入阻塞脚本或通过 `prefers-color-scheme` CSS 媒体查询直接设定初始值，避免闪白
- Tailwind 中引用 CSS 变量：`bg-[var(--bg-primary)]` 而非 Tailwind 原生色板

---

## 9. 离线架构

### 9.1 本地持久化层级

| 层 | 技术 | 存储内容 | 访问路径 |
|----|------|---------|---------|
| 系统密钥库 | `keyring` crate | API Key | Rust 专属 |
| SQLite | `tauri-plugin-sql` | conversations, messages, attachments metadata  | Rust 专属 |
| 文件系统 | tauri-plugin-fs | 附件二进制文件 | Rust 读取后传 base64 |
| 轻量偏好 | tauri-plugin-store | theme, defaultModel, fontSize | Rust/Frontend |

### 9.2 离线行为

| 场景 | 表现 |
|------|------|
| 发送消息时无网络 | MessageBubble 状态设为 `error`, 内联「重新发送」按钮 |
| 查看历史对话 | 完全本地 SQLite 读取，零网络依赖 |
| 搜索结果 | SQLite FTS5 全文搜索，零延迟 |
| 主题切换/设置变更 | 本地偏好立即生效 |

---

## 10. 开发路线图（细化分期）

### Phase 0 — 骨架搭建（P0 MVP）

**目标**：跑通 Tauri + React 通信 + SQLite + API Key 存储 + 单会话流式对话

| 任务 | 预期工时 | 产出 |
|------|---------|------|
| 0.1 Tauri 2.x 脚手架 + React + TypeScript + Tailwind | 1h | `src-tauri/` + `src/` 基础目录 |
| 0.2 `invoke('ping')` 打通 RPC 通信 | 0.5h | 前端按钮 → Rust 返回 → 前端显示 |
| 0.3 CSS Design Tokens 落地 `globals.css` | 1h | 全部 CSS 变量定义（含 dark mode） |
| 0.4 SQLite schema + `tauri-plugin-sql` | 1h | conversation + messages 表 + FTS5 |
| 0.5 `keyring` API Key 存储命令 | 1h | save / has / delete 三个 command |
| 0.6 Anthropic SSE 客户端（Rust） | 3h | client.rs + stream.rs, 单消息发送 |
| 0.7 前端 MessageList + InputBox 骨架 | 2h | 发送→流式渲染→完成→持久化 |
| 0.8 Zustand store（conversation + settings） | 1.5h | 基础状态结构 |
| 0.9 主题切换 + ThemeProvider | 1h | light / dark / system |

**验收标准**：
- `Cmd/Ctrl+N` 新建会话 → 输入文字 → Enter → AI 流式返回 → 刷新后历史存在
- API Key 设置页：输入 → 保存 → 验证
- 浅色/深色切换完整

### Phase 1 — 交互完善（P1）

**目标**：多会话管理、Markdown 渲染、附件的 UI 层

| 任务 | 预期工时 | 产出 |
|------|---------|------|
| 1.1 Sidebar 完整实现（列表/分组/搜索） | 3h | 时间分组、搜索高亮、hover 操作 |
| 1.2 会话 CRUD（new/switch/rename/delete/pin） | 2h | IPC + UI 完整闭环 |
| 1.3 Markdown 渲染器（react-markdown + rehype） | 2h | 表格/列表/引用/行内代码 完整 GFM |
| 1.4 代码块：语法高亮 + 复制按钮 | 1.5h | 语言标签 + 一键复制 |
| 1.5 EmptyState（欢迎页 + 建议卡片） | 1h | 问候语 + 2×2 建议卡片 |
| 1.6 TopBar 完整功能 | 1h | 标题编辑、模型选择、更多菜单 |
| 1.7 附件上传 UI（拖拽 + 按钮 + Chip） | 2h | 前端选择/拖拽 → 显示 Chip |
| 1.8 PetBar 集成 | 1h | 从现有组件接入正式布线 |

### Phase 2 — 体验打磨（P2）

**目标**：搜索、快捷键、错误处理、性能优化

| 任务 | 预期工时 | 产出 |
|------|---------|------|
| 2.1 会话搜索（SQLite FTS5 + 前端高亮） | 2h | 实时搜索、结果列表 |
| 2.2 全局快捷键系统 | 1.5h | useKeyboard hook + shortcuts 列表页 |
| 2.3 错误处理 UI（401/429/529/网络） | 2h | 内联错误提示 + 重试机制 |
| 2.4 Message actions（复制/编辑/重新生成/点赞） | 1.5h | hover 浮现操作条 |
| 2.5 自动滚动 + ScrollToBottom | 1h | 暂停自动滚动逻辑 |
| 2.6 流式渲染性能优化（16ms 帧合并） | 1h | throttled streaming store |
| 2.7 附件 Rust 命令（read_attachment） | 1h | 读取文件传 base64 |

### Phase 3 — 扩展功能（P3）

**目标**：Artifact 面板、导出、窗口状态

| 任务 | 预期工时 | 产出 |
|------|---------|------|
| 3.1 Artifact Panel（代码/文档预览） | 3h | 滑动面板 + 语法高亮 |
| 3.2 对话导出（Markdown / JSON） | 1.5h | 文件保存对话框 |
| 3.3 tauri-plugin-window-state | 0.5h | 窗口尺寸/位置恢复 |
| 3.4 窗口响应式行为 | 1h | <900 sidebar 自动收起, <1280 artifact 浮层 |
| 3.5 自动更新（tauri-updater） | 2h | 发布 + 更新提示 |

---

## 11. 安全模型

### 11.1 API Key 生命周期

```
用户输入 Key
  │
  ▼
ApiKeyTab.tsx: input (type="password", 掩码显示)
  │ invoke('save_api_key', { key })
  ▼
Rust: keyring::Entry.set_password(&key)
  │
  ▼
macOS: Keychain.app 可见 "novachat / anthropic_api_key"
Windows: Windows Credential Manager
Linux: libsecret

此后所有 API 请求：
  Rust: keyring::Entry.get_password() → HTTP header x-api-key
  (前端 JS 永不可见)
```

### 11.2 安全策略

| 威胁 | 缓解 |
|------|------|
| XSS → 窃取 Key | Key 仅存在于 Rust 进程内存 + 系统密钥库；前端接触不到 |
| 依赖投毒 | Rust 编译时验证，`cargo deny` 检查依赖 |
| 明文日志 | Rust 端禁止 `println!` API Key |
| SQL 注入 | 参数化查询（tauri-plugin-sql 使用 prepared statement） |
| 文件遍历 | Rust 端验证附件路径在应用沙箱目录内 |

---

## 12. 关键技术选型论证

### 12.1 React + Tailwind 而非 Vue/Svelte

- 现有 NeoTrix 前端已是 React（`src-tauri/frontend/`），代码可复用
- Zustand 无缝对接现有架构
- `react-markdown` 生态成熟，覆盖 GFM + 代码高亮

### 12.2 图标系统

- `lucide-react`：细线条风格，与暖色调设计系统契合
- 树摇（tree-shaking）友好，按需引入

### 12.3 Design Token 策略（Hiyoko 模式灵感）

- 40 行 CSS 变量即可覆盖 90% 视觉一致性
- Tailwind `theme.extend.colors` 指向 CSS 变量：
  ```js
  // tailwind.config.js
  extend: {
    colors: {
      primary: 'var(--bg-primary)',
      sidebar: 'var(--bg-sidebar)',
      elevated: 'var(--bg-elevated)',
      accent: 'var(--accent-primary)',
      // ...
    }
  }
  ```

### 12.4 粘贴支持

- 监听 `paste` 事件，检测 `clipboardData.files`（截图/图片）
- 自动走附件流程上传

---

## 13. 落地检查清单

### 初始化

- [ ] Tauri 2.x 项目初始化（`npm create tauri-app`）
- [ ] 安装 plugins: sql, store, dialog, fs, window-state
- [ ] Rust dependencies: `keyring`, `reqwest` (with `stream`), `tokio`, `serde`, `serde_json`
- [ ] globals.css: 全部 design tokens + 基础重置 + transition 类
- [ ] Tailwind 指向 CSS 变量

### 组件开发顺序（推荐）

1. `globals.css` + `tailwind.css` → 设计系统奠基
2. `ui/` 原子组件（Button, Dialog, Tooltip, ScrollArea）
3. `App.tsx` 三栏布局 + ThemeProvider
4. `Sidebar` 骨架 + `ChatArea` 骨架
5. `InputBox` + 发送 → 显示 MessageList（mock）
6. Rust `commands/chat.rs` + `anthropic/` 流式实现
7. 前端 `useStreaming` hook 对接真实 IPC
8. `useConversationStore` + 会话 CRUD + `Sidebar`
9. `MarkdownRenderer` + `CodeBlock`
10. `EmptyState` + `SettingsModal` + `ApiKeyTab`
11. `PetBar` 集成
12. 搜索 / 快捷键 / 错误处理 / 附件 / 导出

### 测试验收点

- [ ] `npm run build` 无错误
- [ ] `npm test` 全部通过（含 PetBar 153 tests）
- [ ] 流式对话：发送 → 逐 token 渲染 → 完成持久化 → 刷新可见
- [ ] 多会话：新建 / 切换 / 重命名 / 删除 / 置顶
- [ ] 主题切换：浅色 ↔ 深色 ↔ 跟随系统，无闪烁
- [ ] 离线可用：历史对话读取、搜索
- [ ] API Key 安全：前端 console 不可见，SQLite 不可能存在

---

## 附录 A — 从 osaurus 吸收的设计模式

| 模式 | osaurus 做法 | 本项目的适配 |
|------|-------------|-------------|
| 离线优先 | 本地 SQLite + 缓存优先 | 相同：本地 SQLite 读写优先，API 仅写操作需网络 |
| 持久化记忆 | Core Data + 实体关系映射 | tauri-plugin-sql + schema 设计 |
| 加密身份 | 本地 keychain + 加密密钥对 | keyring crate 存 API Key |
| 无后台服务 | 纯前端应用，无 daemon | 同：Tauri 前端直接调用 Anthropic API |
| 最小权限 | 按需请求系统权限 | 仅请求文件系统读 + 网络；无后台/位置权限 |

## 附录 B — 从 emilkowalski/skills 吸收的动画标准

10 项动画审核标准：

1. ✅ 动画解决什么问题 — 每个动效有意图
2. ✅ 动画频率合适 — hover 80ms, 面板 200ms
3. ✅ 缓动正确 — ease-out 进入, ease-in 退出
4. ✅ 反行动画对称 — 展开=收起, 出现=消失
5. ✅ 尊重 reduce-motion — `prefers-reduced-motion` 时 transition: none
6. ✅ 不阻塞交互 — 非阻塞动画, 内容可操作
7. ✅ 使用 transforms + opacity — GPU 合成, 不触发 layout
8. ✅ 动画参数集中 — transition-duration / timing-function 统一变量
9. ✅ 注意入场/出场时机 — stagger 出场不堆积
10. ✅ 可测试 — 动画不隐藏功能, 测试可 `waitFor`

---

> **文档版本**: 2026-07-03  
> **来源**: NovaChat 设计规格文档 + [osaurus-ai/osaurus](https://github.com/osaurus-ai/osaurus) + [emilkowalski/skills](https://github.com/emilkowalski/skills) + Tauri 2.x 文档 + 桌面应用 UI/UX 最佳实践  
> **状态**: 设计草案，等待实现者签收
