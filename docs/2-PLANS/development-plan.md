# NeoTrix UI — Development Plan

## Current State Assessment

| Label/View | Current State | Gap |
|---|---|---|
| **对话 (Chat)** | Basic hero screen + empty message list. Input box exists. No streaming, no markdown rendering, no message persistence. | Lacks full chat loop |
| **团队 (Cowork)** | Session list shell + task list shell + agent list shell. All static, no backend. | No real session/task/agent management |
| **代码 (Code)** | File tree + code editor with tabs. All mock data. | No real file system access, no git integration |
| **代理 (Agent)** | Dashboard with hero ring + 5 sub-tabs (概览/地图/节点/订阅/设置). Mostly mock stats. | No real agent monitoring, no provider integration |
| **设置 (Settings)** | Modal with tabs. Partially implemented. | Missing API key management, model config, theme system |
| **User Popover** | 设置/升级/主题切换/帮助/退出登录. Functional. | Needs 账号 management |
| **主题** | Light/Dark toggle works. | No system-follow mode, no 3-state cycle |

## Reference Analysis

### From osaurus (Swift macOS AI Harness):
- Agent loop with working directory, file/search/git tools, sandbox VM
- Memory system (identity → pinned facts → session episodes)
- MCP server + client for tool integration
- Privacy filter (on-device PII detection via MLX)
- Cryptographic identity (secp256k1)
- Secure relay channel (E2E encrypted WebSocket)
- Plugin system with hot-reload
- Skills & methods (RAG-searched capabilities)
- Voice input (ANE on-device transcription)

### From NovaChat Design Spec:
- Three-column layout (sidebar, chat, artifact panel)
- Streaming token-by-token rendering
- Markdown with code highlighting
- Session management (group by time, search, pin, rename)
- Attachment upload (drag-drop images/docs)
- API Key in system keychain (keyring crate)
- Error handling (401/429/529)
- Artifact panel for code/docs
- Keyboard shortcuts

## Development Plan

### P0 — Chat MVP (Core Loop)

| Task | Description | Depends On |
|------|-------------|-----------|
| Chat IPC bridge | `invoke('send_message')` → Rust backend → SSE stream → `emit('stream-chunk')` | Rust commands/chat.rs |
| Streaming render | Token-by-token append to message bubble | IPC bridge |
| Markdown + Code highlight | `react-markdown` + `remark-gfm` + `rehype-highlight` | Streaming render |
| Message persistence | SQLite messages table, save on complete | DB schema |
| Input box | Multi-line textarea, Enter/Shift+Enter, auto-grow | None |
| Empty state → chat transition | Hero section replaced by MessageList on first send | None |

### P0 — API Key Management

| Task | Description | Depends On |
|------|-------------|-----------|
| Keychain storage | Rust `keyring` crate, save/verify API key | None |
| Settings → API tab | Key input with mask, verify button, success indicator | Keychain |
| Key never in frontend | All requests via Rust, frontend only invokes commands | Keychain |

### P1 — Session Management

| Task | Description | Depends On |
|------|-------------|-----------|
| Multi-session CRUD | SQLite conversations table, create/switch/delete/rename | DB schema |
| Session list in sidebar | Group by today/yesterday/7 days/earlier, sorted by updatedAt | Multi-session CRUD |
| Session search | FTS5 search over messages, real-time filter | Multi-session CRUD |
| Pin/rename/delete | Hover actions on session items | Session list |
| Active session highlight | `--accent-primary` 8% background | Session list |

### P1 — Theme System Upgrade

| Task | Description | Depends On |
|------|-------------|-----------|
| 3-state theme | light/dark/system, stored in AppSettings | None |
| System follow | `matchMedia('prefers-color-scheme:dark')` listener | 3-state theme |
| Theme toggle in popover | Cycle light → dark → system | 3-state theme |

### P1 — User Popover + Account

| Task | Description | Depends On |
|------|-------------|-----------|
| User profile section | Avatar + name + email in popover header | None |
| 设置 modal integration | Click → open settings to correct tab | Settings modal |

### P2 — Cowork/Team View

| Task | Description | Depends On |
|------|-------------|-----------|
| Real session list | Load cowork sessions from backend API | None |
| Task management | Create/assign/complete tasks within session | Session list |
| Agent slots | Display connected agents per session, status icons | Task management |
| Real-time status | Poll or WebSocket for task progress updates | Task management |

### P2 — Code View

| Task | Description | Depends On |
|------|-------------|-----------|
| File tree from FS | Read project directory, render tree with icons | None |
| File open → editor | Read file content → display in code viewer with syntax | File tree |
| File save | `invoke('write_file')` → Rust FS write | File open |
| Git status | Branch name, changed files indicator | None |
| Tab management | Open multiple files in tabs, close/reorder | File open |

### P2 — Error Handling + Reliability

| Task | Description | Depends On |
|------|-------------|-----------|
| 401 → settings redirect | Red toast + button to API settings tab | API Key management |
| 429 retry with countdown | Auto-retry with visual countdown timer | Streaming render |
| 529 service overload | Manual retry button on message | Streaming render |
| Network error recovery | Message status: error, inline resend button | Streaming render |

### P3 — Agent/Proxy View

| Task | Description | Depends On |
|------|-------------|-----------|
| Live stats on hero ring | Real circuit breaker, rate limiter, provider pool metrics | Backend monitoring |
| World map | Geo-located proxy nodes on SVG map | Backend proxy data |
| Node table | Sortable table with health, latency, provider type | Backend proxy data |
| Subscription management | Add/remove URL subscriptions for providers | Node table |
| Provider settings | Proxy config, strategy selector | Settings modal |

### P3 — Artifact Panel

| Task | Description | Depends On |
|------|-------------|-----------|
| Auto-detect artifacts | Parse AI response for code blocks → auto-open panel | Markdown render |
| Panel toggle | Slide in from right, resizable width | Auto-detect artifacts |
| Multi-file support | Tab bar within panel for multiple artifacts | Panel toggle |

### P3 — Attachments

| Task | Description | Depends On |
|------|-------------|-----------|
| File picker | Tauri dialog plugin, accept images/docs | None |
| Drag-drop zone | Input box accepts drag-drop files | File picker |
| Attachment chip | Display selected files inline above input | Drag-drop zone |
| Upload to message | Include attachment data in API call | Attachment chip |

### P3 — MCP Integration (from osaurus)

| Task | Description | Depends On |
|------|-------------|-----------|
| MCP server mode | Expose tools via stdio MCP server to external clients | None |
| MCP client mode | Aggregate tools from remote MCP providers | None |
| Tool registry | UI to browse and enable/disable discovered tools | MCP client mode |

### P3 — Keyboard Shortcuts (from NovaChat spec)

| Task | Description | Depends On |
|------|-------------|-----------|
| Cmd+N new session | Create new conversation | Session management |
| Cmd+K search | Focus session search box | Session search |
| Cmd+, settings | Open settings modal | Settings modal |
| Cmd+B toggle sidebar | Collapse/expand sidebar | None |
| Escape stop/close | Stop generation or close modal | Chat MVP |

## Architecture Principles

1. **IPC-first**: All API calls through Rust backend, never from frontend directly
2. **Mock-first per component**: Build static UI → add Zustand store → connect IPC
3. **CSS variables**: No hardcoded colors, all via `var(--)` tokens
4. **API Key security**: keyring only, never in frontend JS/sqlite/logs
5. **Streaming performance**: Throttle `stream-chunk` events at 16ms/frame

## Component Tree (Target)

```
App
├── Sidebar (260px, collapsible to 64px)
│   ├── Logo + New Chat button
│   ├── SearchBox
│   ├── ConversationList (grouped by time)
│   ├── Nav Tabs (对话/团队/代码/代理)
│   └── UserBar → Popover
│       ├── 设置 → SettingsModal
│       ├── 升级
│       ├── 主题切换
│       ├── 帮助
│       └── 退出登录
├── Main Area
│   ├── Chat View
│   │   ├── TopBar (title, model selector, more menu)
│   │   ├── MessageList (streaming, markdown, code blocks)
│   │   ├── InputBox (textarea, attachments, send/stop)
│   │   └── EmptyState (hero + suggestion cards)
│   ├── Cowork View
│   │   ├── Session sidebar
│   │   ├── Task list
│   │   └── Agent slots
│   ├── Code View
│   │   ├── File tree
│   │   ├── Editor tabs
│   │   └── Code viewer
│   └── Agent View
│       ├── Hero dashboard
│       ├── Map / Nodes / Subscriptions tabs
│       └── Settings
├── Artifact Panel (optional, right side)
│   ├── Tab bar
│   └── Content viewer
└── Settings Modal
    ├── General tab (theme, font, language)
    ├── API tab (key input, verify)
    ├── Model tab (default model, temperature)
    ├── Shortcuts tab (read-only list)
    └── About tab
```
