# NeoTrix UI Architecture Specification

## 1. Design Principles

| Principle | Rule |
|-----------|------|
| IPC-first | All API/DB calls through Rust backend via `invoke()`. Frontend never calls external APIs directly. |
| Settings persistence | All settings stored in `localStorage` (v1) → Rust SQLite (v2). Zero data loss on refresh. |
| Dead-code zero | Every CSS class and JS handler must be attached to a real DOM element. No `showToast('开发中')` stubs in production. |
| View-model separation | Each view has a dedicated data store. No global state pollution. |
| Global vs local | Settings split: global (theme, API, pet) ↔ per-view (model per chat, proxy per agent, etc.). |

---

## 2. Complete UI Element Specification

### 2.1 Sidebar `.sb`

| Element | Label | Current | Target behavior |
|---------|-------|---------|-----------------|
| `.sbn#sidebarToggle` | ☰ | `toggleSidebar()` | ✅ Collapse `.sb` to 64px icon mode. Persist via `localStorage.sidebarCollapsed`. |
| `.sbn` (search) | 🔍 | Toast only | ❌ Remove if no real search. **Or**: wire to `dispatch('search-sessions')` → filter `recentData` live. |
| `.segb[data-view=chat]` | 对话 | `switchView(this,'chat')` | ✅ Working. |
| `.segb[data-view=cowork]` | 团队 | `switchView(this,'cowork')` | ✅ Working. |
| `.segb[data-view=code]` | 代码 | `switchView(this,'code')` | ✅ Working. |
| `.segb[data-view=agent]` | 代理 | `switchView(this,'agent')` | ✅ Working. |

### 2.2 Navigation Items `.nl` (dynamic from `navData`)

| View | Icon | Label | Current | Target behavior |
|------|------|-------|---------|-----------------|
| chat | + | 新对话 | `newChat` → reset to hero | ✅ Working. |
| chat | 📁 | 项目 | `showProjects` → overlay | ✅ Working. Richer project list from `localStorage` or KB. |
| chat | ▶ | 成果 | `showAchievements` → overlay | ✅ Working. Stats from real data source. |
| chat | ⚙ | 自定义 | `openCustomize` → toast | ❌ Remove. **Or**: wire to view-specific settings (local settings pane). |
| cowork | ⊞ | 会话管理 | `newCoworkTask` → creates session | ✅ Working. Persist session to `localStorage`. |
| cowork | 📁 | 历史记录 | `showAgentHistory` → overlay | ✅ Working. |
| cowork | ⚙ | 配置 | `openAgentConfig` → settings→compute | ✅ Working. |
| code | + | 新会话 | `newCodeSession` → reset editor | ✅ Working. |
| code | ▶ | 流程 | `showPipelines` → overlay | ✅ Working. |
| code | ⚙ | 自定义 | `openCustomize` → toast | ❌ Remove. |
| agent | ▶ | 运行链 | `runAgentChain` → simulated chain | ✅ Keep as demo trigger. V2: real E8 chain via IPC. |
| agent | 📁 | 历史 | `showAgentHistory` → overlay | ✅ Working. |
| agent | ⚙ | 配置 | `openAgentConfig` → settings→compute | ✅ Working. |

### 2.3 Recents `.re-i` (dynamic from `recentData`)

| Current | Target |
|---------|--------|
| Toast '打开...' on click | ✅ Click → `switchView` + load session data from `localStorage` |
| Ghost items (pointer-events:none) | ✅ Keep as placeholder for pending sessions. V2: real data. |

### 2.4 User Bar + Popover

| Element | Current | Target |
|---------|---------|--------|
| `#userBar` click | `toggleUserPopover(event)` | ✅ Working. |
| 设置 | `openSettingsModal()` | ✅ Working. |
| 升级 | Toast only | ❌ Remove. **Or**: wire to real upgrade page / in-app purchase link. |
| 主题切换 | `toggleTheme()` | ✅ Working. V2: 3-state cycle (light → dark → system). |
| 帮助 | Toast only | ❌ Remove. **Or**: wire to `/help` page or inline guide overlay. |
| 退出登录 | Toast + `clearChat()` | ✅ Keep as placeholder. V2: real session clear + confirmation. |

### 2.5 Chat View `#viewChat`

| Element | Current | Target |
|---------|---------|--------|
| `.ch-top` | Empty | ✅ Keep minimal. V2: model selector, session title. |
| `#heroSection` | Hero + suggestions | ✅ Working. V2: suggestion cards from local KB. |
| `#chatScroll` | Message list (`.r`/`.l`) | ✅ Working. V2: persistent messages via `localStorage`. |
| `#chatInput` textarea | `sendMsg()` | ✅ Working. V2: streaming token render via IPC. |
| `#sendBtn` | `sendMsg()` | ✅ Working. V2: disabled state during generation. |
| `.cic-attach` (+) | No handler | ❌ Add click → file picker dialog. V2: Tauri dialog. |
| `.vc-btn.vc-lang` (mic) | Toast only | ❌ Remove. **Or**: wire to Web Speech API. |

### 2.6 Cowork View `#viewCowork`

| Element | Current | Target |
|---------|---------|--------|
| `.cw-add` (+) | Toast '新建会话' | ❌ Wire to `newCoworkTask()`. |
| `.cw-sitem` | `selectCwSession(i)` | ✅ Working. V2: persistent session list from `localStorage`. |
| Cultivation/经脉 system | Hijacks `.cw-main` | ✅ Keep as cowork detail feature. V2: collapsible sidebar toggle. |

### 2.7 Code View `#viewCode`

| Element | Current | Target |
|---------|---------|--------|
| `.cd-tbtn` (refresh) | Toast | ❌ Wire to read FS tree (V2: via IPC `invoke('fs_tree')`). |
| `.cd-tab` ×3 | `switchCodeTab(this, idx)` | ✅ Working. V2: dynamic tabs from open files. |
| 格式化 button | Toast '格式化完成' | ❌ Wire to `invoke('format_code')`. |
| 保存 button | Toast '已保存' | ❌ Wire to `invoke('write_file')`. |
| 复制 button | `copyCode()` | ✅ Working. |
| 运行 button | Toast '运行中...' | ❌ Wire to `invoke('execute_code')` → sandbox. |

### 2.8 Agent View `#viewAgent`

| Element | Current | Target |
|---------|---------|--------|
| `#pxStartBtn` | `toggleProxyDaemon()` | ✅ Working. V2: real backend connection. |
| 配置 button | Toast '配置面板' | ❌ Wire to settings→compute. |
| 概览/地图/节点/订阅/设置 tabs | `switchPxTab()` | ✅ Working. |
| 🔄拉取 | Toast '模拟' | ❌ Wire to `invoke('proxy_fetch_nodes')`. |
| 🔄全部拉取 | Toast '模拟' | ❌ Wire to `invoke('proxy_fetch_all')`. |
| 添加 subscription | `addSubscription()` | ✅ Working. V2: persist to `localStorage`. |

### 2.9 Right Sidebar `.rb`

| Element | Current | Target |
|---------|---------|--------|
| Preview/Code toggle | `switchArtifactView()` | ✅ Working. V2: real code vs rendered preview. |
| 6 format tabs | `setPreviewMode(id)` | ✅ Working. V2: server-side rendering. |
| 复制 | `copyPreview()` | ✅ Working. |
| 刷新 | Toast '已刷新' | ❌ Wire to re-render. |
| 展开 | `expandPreview(event)` | ✅ Working. |
| 关闭 | `closePreview(event)` | ✅ Working. |
| Auto-hide edge detect | mousemove timer | ✅ Working. |

### 2.10 Settings Modal `#overlaySettings`

#### Global Settings (persist across all views)

| Tab | Setting | Type | Default | Persistence |
|-----|---------|------|---------|-------------|
| 个人资料 | 全名 | text input | '' | `localStorage` |
| 个人资料 | 显示名称 | text input | '' | `localStorage` |
| 个人资料 | 工作角色 | select | '开发者' | `localStorage` |
| 外观 | 主题模式 | 2-state button | light | `localStorage.theme` |
| 外观 | 字体大小 | select (小/中/大) | 中 | `localStorage.fontSize` |
| 外观 | 减少透明 | checkbox | false | `localStorage.reduceTransparency` |
| 语音 | 语音输入 | checkbox | true | `localStorage.voiceInput` |
| 语音 | 输入语言 | select | 中文 | `localStorage.voiceLang` |
| 语音 | 自动发送 | checkbox | false | `localStorage.voiceAutoSend` |
| 隐私 | 对话存储 | checkbox | true | `localStorage.privacyStore` |
| 隐私 | 使用数据 | checkbox | false | `localStorage.privacyTelemetry` |
| 隐私 | 本地处理 | checkbox | true | `localStorage.privacyLocal` |

#### View-Local Settings (per-view, in `localStorage.viewSettings`)

| View | Setting | Type | Default |
|------|---------|------|---------|
| Chat | Default model | select | 'GatewayV2' |
| Chat | Context length | select | 8192 |
| Chat | Auto-send on Enter | checkbox | true |
| Cowork | Agent count | number | 3 |
| Cowork | Auto-assign tasks | checkbox | true |
| Code | Tab size | select (2/4/8) | 4 |
| Code | Font size | select (12/14/16) | 14 |
| Code | Language | select (rust/python/js) | auto |
| Agent | Provider | select | GatewayV2 |
| Agent | Max tokens | select | 16384 |

#### Removed (dead settings)

| Tab | Setting | Reason |
|-----|---------|--------|
| 账单 | Upgrade button | Toast only → remove until real billing |
| 数据控制 | Export/Clear | Toast only → keep as placeholder, remove dead CSS |
| 计算资源 | Engine/Provider/Max tokens | Move to Chat view-local settings |

---

## 3. State Management Architecture

```
┌──────────────────────────────────────────────────────┐
│                    appState (global)                  │
│  theme | sidebarCollapsed | rbAutoHide | userProfile  │
│  petVisible | petPersona | fontSize | voiceSettings   │
└────────────┬───────────────────────────────┬──────────┘
             │                               │
┌────────────▼──────────┐   ┌────────────────▼──────────┐
│     chatState          │   │   viewState (per-view)    │
│  messages[]            │   │   chat: {model, context}  │
│  currentSessionId      │   │   cowork: {sessions[]}    │
│  sessions[]            │   │   code: {openFiles[]}     │
│  isStreaming           │   │   agent: {config}         │
│  inputText             │   └───────────────────────────┘
└────────────────────────┘
```

### 3.1 Global State (`window.appState`)

```js
const appState = {
  theme: localStorage.getItem('theme') || 'light',
  sidebarCollapsed: localStorage.getItem('sidebarCollapsed') === 'true',
  rbAutoHide: false,
  userProfile: JSON.parse(localStorage.getItem('userProfile') || '{}'),
  petVisible: true,
  petPersona: localStorage.getItem('petPersona') || 'default',
  fontSize: localStorage.getItem('fontSize') || 'medium',
  reduceTransparency: localStorage.getItem('reduceTransparency') === 'true',
  voiceInput: localStorage.getItem('voiceInput') !== 'false',
  voiceLang: localStorage.getItem('voiceLang') || '中文',
  voiceAutoSend: localStorage.getItem('voiceAutoSend') === 'true',
  privacyStore: localStorage.getItem('privacyStore') !== 'false',
  privacyTelemetry: localStorage.getItem('privacyTelemetry') === 'true',
  privacyLocal: localStorage.getItem('privacyLocal') !== 'false',
};
```

### 3.2 View State (`window.viewState`)

```js
const viewState = {
  chat: {
    model: localStorage.getItem('chatModel') || 'GatewayV2',
    contextLength: parseInt(localStorage.getItem('chatContext') || '8192'),
    autoSend: localStorage.getItem('chatAutoSend') !== 'false',
    sessions: JSON.parse(localStorage.getItem('chatSessions') || '[]'),
    currentSessionId: localStorage.getItem('currentSessionId') || null,
    messages: JSON.parse(sessionStorage.getItem('currentMessages') || '[]'),
    isStreaming: false,
  },
  cowork: {
    sessions: JSON.parse(localStorage.getItem('coworkSessions') || '[]'),
    agentCount: parseInt(localStorage.getItem('coworkAgentCount') || '3'),
    autoAssign: localStorage.getItem('coworkAutoAssign') !== 'false',
  },
  code: {
    openFiles: JSON.parse(sessionStorage.getItem('codeOpenFiles') || '[]'),
    tabSize: parseInt(localStorage.getItem('codeTabSize') || '4'),
    fontSize: parseInt(localStorage.getItem('codeFontSize') || '14'),
  },
  agent: {
    provider: localStorage.getItem('agentProvider') || 'GatewayV2',
    maxTokens: parseInt(localStorage.getItem('agentMaxTokens') || '16384'),
    subscriptions: JSON.parse(localStorage.getItem('agentSubscriptions') || '[]'),
    daemonRunning: false,
  },
};
```

### 3.3 Persistence Layer

| Storage | Purpose | Key pattern |
|---------|---------|-------------|
| `localStorage` | Persistent settings + session metadata | `theme`, `chatSessions`, `userProfile` |
| `sessionStorage` | Transient UI state (current messages, open file tabs) | `currentMessages`, `codeOpenFiles` |
| `localStorage` (v2) | → migrated to Rust SQLite `app_settings` table | — |

---

## 4. Dead Code Removal Plan

### 4.1 CSS classes to delete

| Class | Lines (approx) | Reason |
|-------|----------------|--------|
| `.ch-upgrade` | 719-720, 1189 | No HTML element |
| `.ch-icon-btn` | 721-723 | No HTML element |
| `.onboard-card` | 1249 | No HTML element |
| `.oct-dot`, `.oct-lbl`, `.oct-body`, `.oct-inv` | 903-909 | No HTML element |
| `.qt`, `.qo-box`, `.qo-search`, `.slash-card` | 1259-1260 | Only in `@media` (no HTML) |

### 4.2 JS handlers to remove or rewire

| Handler | Current | Action |
|---------|---------|--------|
| Search sidebar button | Toast only | Remove element **or** wire to session search |
| 自定义 nav item (chat) | Toast only | Remove **or** wire to per-view local settings pane |
| 自定义 nav item (code) | Toast only | Remove **or** wire to per-view local settings pane |
| Attach button (+) | No handler | Add file dialog |
| Voice input button | Toast only | Wire to Web Speech API |
| Format code button | Toast only | Wire to IPC format |
| Save code button | Toast only | Wire to IPC write |
| Run code button | Toast only | Wire to IPC execute |
| 升级 (popover) | Toast only | Remove **or** wire to real link |
| 帮助 (popover) | Toast only | Wire to help overlay |
| 配置 (agent hero) | Toast only | Wire to settings → compute |
| 🔄拉取 / 🔄全部拉取 | Toast only | Wire to IPC proxy fetch |
| 刷新 (artifact) | Toast '已刷新' | Wire to re-render from source |

---

## 5. Implementation Roadmap

### Phase 0 — Architecture Cleanup (1 session)

| # | Task | Files affected |
|---|------|---------------|
| 1 | Add `appState` + `viewState` objects with persistence | HTML JS section |
| 2 | Wire all settings controls to `localStorage` | HTML settings modal JS |
| 3 | Delete dead CSS classes (Section 4.1) | HTML `<style>` |
| 4 | Remove/rewire toast-only handlers (Section 4.2) | HTML JS `actions` + DOM |
| 5 | Remove dead `.ch-top` elements (clock already gone) | HTML |
| 6 | Add `loadState()` + `saveState()` at init | HTML JS init |

### Phase 1 — Chat MVP (1-2 sessions)

| # | Task | Files affected |
|---|------|---------------|
| 1 | Message persistence: save/load from `localStorage` | HTML JS `sendMsg()` + init |
| 2 | Session list: create/switch/delete sessions | HTML JS `renderSidebar()` |
| 3 | Session search: filter `recentData` by query | HTML JS search handler |
| 4 | Attach button: file picker via hidden `<input type=file>` | HTML DOM + JS |
| 5 | Enter/Shift+Enter distinction for input | HTML JS `handleKey()` |

### Phase 2 — Cowork + Code Views (1-2 sessions)

| # | Task | Files affected |
|---|------|---------------|
| 1 | Cowork session persistence | HTML JS `CW_DATA` → `localStorage` |
| 2 | Cowork cultivation system toggle | HTML JS |
| 3 | Code view → file tree from `localStorage` mock | HTML JS `renderFileTree()` |
| 4 | Code view → save to `localStorage` | HTML JS save handler |
| 5 | Code view → dynamic tabs | HTML JS tab management |

### Phase 3 — Agent + Settings (1 session)

| # | Task | Files affected |
|---|------|---------------|
| 1 | Agent proxy config persistence | HTML JS `saveProxyConfig()` → `localStorage` |
| 2 | Agent subscription persistence | HTML JS `addSubscription()` → `localStorage` |
| 3 | Theme 3-state cycle | HTML JS `toggleTheme()` → light/dark/system |
| 4 | System-follow theme via `prefers-color-scheme` listener | HTML JS init |
| 5 | Pet visibility toggle in global settings | HTML JS `appState.petVisible` |

### Phase 4 — Streaming + IPC (Rust backend)

| # | Task | Files affected |
|---|------|---------------|
| 1 | Chat IPC bridge (`invoke('send_message')`) | HTML + Rust `commands/chat.rs` |
| 2 | Streaming token render | HTML JS + Tauri event listener |
| 3 | Markdown + code highlight rendering | HTML JS `marked` + `highlight.js` |
| 4 | Error handling (401/429/529) | HTML JS + Rust error codes |

---

## 6. Persistent Data Schema

```js
// localStorage keys and their schemas

// Global settings
localStorage.theme                          // 'light' | 'dark' | 'system'
localStorage.sidebarCollapsed               // 'true' | 'false'
localStorage.userProfile                    // JSON: { name, displayName, role }
localStorage.fontSize                       // 'small' | 'medium' | 'large'
localStorage.reduceTransparency             // 'true' | 'false'
localStorage.voiceInput                     // 'true' | 'false'
localStorage.voiceLang                      // '中文' | 'English' | '日本語'
localStorage.voiceAutoSend                  // 'true' | 'false'
localStorage.privacyStore                   // 'true' | 'false'
localStorage.privacyTelemetry               // 'true' | 'false'
localStorage.privacyLocal                   // 'true' | 'false'
localStorage.petPersona                     // 'default' | 'tsundere' | 'sage' | 'mystic'
localStorage.cultState                      // JSON: cultivation state object

// Per-view settings (stored as JSON under one key)
localStorage.viewSettings                   // JSON: { chat: {model, context, autoSend},
                                            //         cowork: {agentCount, autoAssign},
                                            //         code: {tabSize, fontSize},
                                            //         agent: {provider, maxTokens} }

// Session data
localStorage.chatSessions                   // JSON: [{id, title, createdAt, updatedAt, messageCount}]
localStorage.currentSessionId               // string | null
sessionStorage.currentMessages              // JSON: [{role, content, timestamp}] (per session)
localStorage.coworkSessions                 // JSON: [{name, status, tasks, agents}]
sessionStorage.codeOpenFiles                // JSON: [{name, language, content}]
localStorage.agentSubscriptions             // JSON: [url strings]
```

---

## 7. Event Flow Specification

### 7.1 View Switching

```
User clicks .segb (对话/团队/代码/代理)
  → switchView(el, view)
    → save current view state to localStorage/sessionStorage
    → update appState.currentView
    → hide all .vw-* (display:none)
    → show target #viewX (display:flex)
    → renderSidebar(view)
    → if agent: initAgentDash()
    → if cowork: renderCowork()
```

### 7.2 Send Message (with streaming)

```
User clicks #sendBtn or presses Enter
  → sendMsg()
    → if streaming in progress: stopStream() (stop generation)
    → read chatInput.value
    → if empty: return
    → create user msg div (.msg.r) with timestamp
    → clear input, disable send button
    → scroll to bottom
    → if first message: hide hero section
    → save message to sessionStorage.currentMessages
    
    (V2 with IPC):
    → invoke('send_message', { sessionId, content, model })
    → listen for 'stream-chunk' event:
      → append token to last AI message div
      → scroll to bottom (throttled 16ms)
    → on stream complete:
      → enable send button
      → save complete message
      → update session message count in localStorage
```

### 7.3 Session Management

```
New session:
  → create { id: crypto.randomUUID(), title: '新对话', createdAt: Date.now() }
  → prepend to localStorage.chatSessions
  → set localStorage.currentSessionId
  → clear sessionStorage.currentMessages
  → re-render sidebar

Switch session:
  → save current messages to sessionStorage
  → set localStorage.currentSessionId = newId
  → load messages from sessionStorage keyed by newId
  → re-render chat messages

Delete session:
  → remove from localStorage.chatSessions
  → remove key from sessionStorage
  → if deleted === current: create new session
  → re-render sidebar
```

### 7.4 Settings Persistence Flow

```
User changes a setting control:
  → onChange/onClick handler reads value
  → writes to appState (in-memory)
  → writes to localStorage (persistent)
  → applies to DOM (theme, fontSize, etc.)

On page load:
  → loadState()
    → read all localStorage keys
    → populate appState
    → apply to DOM
    → populate settings form controls
```

---

## 8. CSS Variable Convention

| Token group | Pattern | Example |
|-------------|---------|---------|
| Background | `--bg-{name}` | `--bg-base`, `--bg-surface`, `--bg-elevated` |
| Text | `--tx-{level}` | `--tx` (primary), `--tx2` (secondary), `--tx3` (tertiary) |
| Border | `--bd{,2}` | `--bd` (subtle), `--bd2` (medium) |
| Interaction | `--ghost`, `--ghost-hover`, `--accent-bg` | — |
| Shadow | `--shadow-{level}` | `--shadow-sm`, `--shadow-md`, `--shadow-lg` |
| Z-index | `--z-{layer}` | `--z-overlay:200`, `--z-popover:300`, `--z-toast:400` |
| Spacing | `--sp-{n}` | `--sp-2:4px`, `--sp-8:16px` (4px base) |
| Radius | `--r-{size}` | `--r-sm:8px`, `--r-lg:16px`, `--r-xl:20px` |

---

## 9. Reference Architecture (from External Sources)

### From osaurus (Swift macOS AI Harness):
| Feature | NeoTrix Equivalent | Integration Priority |
|---------|-------------------|---------------------|
| Agent loop with tools | `nt_act_code` + MCP | P3 |
| Privacy filter | `nt_shield` | P3 |
| Cryptographic identity | `nt_shield_vault` | P4 |
| Plugin system | MCP tool registry | P3 |
| Keychain API key | `keyring` crate | P0 |

### From NovaChat Design Spec:
| Feature | NeoTrix Priority | Notes |
|---------|-----------------|-------|
| Streaming token render | P0 | Critical for chat UX |
| Markdown + code highlight | P0 | Use `marked` + `highlight.js` |
| Session grouping by time | P1 | Sidebar recents |
| Attachment upload | P1 | Hidden `<input type=file>` |
| Error handling 401/429/529 | P1 | Toast + retry |
| Keyboard shortcuts | P3 | Cmd+K, Cmd+N, etc. |

---

## 10. Performance Budgets

| Metric | Budget | Measurement |
|--------|--------|-------------|
| Settings save latency | < 5ms | `performance.now()` around `localStorage.setItem` |
| View switch latency | < 16ms | From click to first paint |
| Message render (100 lines) | < 50ms | From `sendMsg()` to DOM append |
| Session list render (50 items) | < 30ms | From data read to DOM |
| Theme switch | < 100ms | From click to last CSS var change |
| File tree render (100 nodes) | < 50ms | From data to DOM |
| Pet loop frame time | < 16ms (60fps) | `requestAnimationFrame` delta |
