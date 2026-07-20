# neocodex — AI-Native Desktop Agent IDE

## Vision

neocodex is the desktop UI surface for **NeoTrix**: a visual agent IDE that combines ChatGPT Codex-style desktop capabilities with NeoTrix's 7-domain faction architecture, self-evolving reasoning, and VSA HyperCube knowledge representation.

**Core thesis**: The desktop is where agents meet the real world — files, apps, browser, terminal, plugins, and scheduled background work. neocodex makes NeoTrix's backend (McpGateway, KB, SEAL pipeline, EventBus, GWT consciousness) visible and controllable through a native macOS/Windows app.

---

## 1. Architecture

### 1.1 Stack

```
┌─────────────────────────────────────────────────────┐
│                    neocodex (Tauri)                   │
│  ┌──────────────────────────────────────────────┐   │
│  │  Frontend (SolidJS + Tailwind + D3.js)        │   │
│  │  ┌─────────┐ ┌──────────┐ ┌──────────────┐   │   │
│  │  │ Chat UI │ │ Projects │ │ Plugin Mktpl. │   │   │
│  │  └─────────┘ └──────────┘ └──────────────┘   │   │
│  │  ┌─────────┐ ┌──────────┐ ┌──────────────┐   │   │
│  │  │ Computer │ │ Settings │ │ Scheduled    │   │   │
│  │  │ Use View │ │          │ │ Task Mgr     │   │   │
│  │  └─────────┘ └──────────┘ └──────────────┘   │   │
│  └──────────────────────────────────────────────┘   │
│                         │ IPC (invoke)               │
│  ┌──────────────────────────────────────────────┐   │
│  │  Rust Core (Tauri commands)                   │   │
│  │  ┌──────────┐ ┌───────────┐ ┌────────────┐   │   │
│  │  │ Gateway  │ │ Sandbox   │ │ Scheduler  │   │   │
│  │  │ Bridge   │ │ Manager   │ │ Engine     │   │   │
│  │  └────┬─────┘ └─────┬─────┘ └─────┬──────┘   │   │
│  └──────┼──────────────┼─────────────┼──────────┘   │
└─────────┼──────────────┼─────────────┼──────────────┘
          │              │             │
          ▼              ▼             ▼
   ┌────────────┐ ┌───────────┐ ┌──────────────┐
   │ neotrix    │ │  OS APIs  │ │  File System │
   │ (crate)    │ │ (Acces.   │ │  + Git       │
   │            │ │  Screen)  │ │  Worktrees   │
   └────────────┘ └───────────┘ └──────────────┘
```

### 1.2 Key Architectural Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Framework | **Tauri 2.0** | Native performance, Rust backend, small bundle (<10MB), cross-platform |
| Frontend | **SolidJS** | Fine-grained reactivity, no virtual DOM overhead, small runtime |
| Styling | **Tailwind CSS** + custom design system | Consistent with NeoTrix faction visual identity |
| IPC | Tauri `invoke` + event system | Typed commands, async, secure |
| Backend dep | `neotrix-core` as library crate | All 7 domains accessible via Rust API, no HTTP overhead |
| Computer Use | macOS Accessibility + CGDisplay | Native screen capture, mouse/keyboard simulation |
| Plugin System | MCP protocol (reuse McpGateway) | Zero new protocol — NeoTrix already has McpGateway |
| Build | `cargo build -p neocodex` | Sibling Cargo workspace or standalone |

### 1.3 Crate Structure

```
neocodex/
├── src-tauri/              # Tauri Rust backend
│   ├── src/
│   │   ├── main.rs         # Entry point
│   │   ├── lib.rs          # Plugin registration
│   │   ├── commands/       # IPC commands
│   │   │   ├── chat.rs     # LLM conversation
│   │   │   ├── projects.rs # Project CRUD
│   │   │   ├── plugins.rs  # Plugin marketplace
│   │   │   ├── scheduler.rs# Scheduled tasks
│   │   │   ├── computer.rs # Computer Use
│   │   │   ├── kb.rs       # Knowledge base queries
│   │   │   └── settings.rs # App configuration
│   │   ├── bridge/         # NeoTrix integration
│   │   │   └── mod.rs      # GatewayBridge, KBBridge
│   │   ├── sandbox/        # Permission enforcement
│   │   │   └── mod.rs
│   │   └── scheduler/      # RRULE-based task runner
│   │       └── mod.rs
│   ├── Cargo.toml          # Deps: neotrix-core, tauri, etc.
│   └── tauri.conf.json     # Tauri config
├── src/                    # Frontend (SolidJS)
│   ├── App.tsx
│   ├── routes/
│   │   ├── Chat.tsx
│   │   ├── Projects.tsx
│   │   ├── Plugins.tsx
│   │   ├── Scheduled.tsx
│   │   ├── Settings.tsx
│   │   └── ComputerUse.tsx
│   ├── components/
│   │   ├── Sidebar.tsx
│   │   ├── ChatMessage.tsx
│   │   ├── PluginCard.tsx
│   │   ├── FilePreview.tsx
│   │   └── AnnotationOverlay.tsx
│   ├── stores/             # SolidJS signals/stores
│   └── styles/
├── package.json
├── tsconfig.json
├── vite.config.ts
└── tailwind.config.js
```

---

## 2. Core Features

### 2.1 Projects & Chats (P0)

**ChatGPT Codex equivalent**: Projects view with multi-chat organization, pinned items, search, archive.

**neocodex implementation**:
- **Local projects**: Directory-linked — same as NeoTrix's worktree concept
- **ChatGPT-style projects**: Virtual — shared context (files, instructions, sources) across chats
- **Chat modes**: Quick Chat (one-off), Work Chat (multi-turn with tools), Agent Chat (autonomous with goal loop)
- **Project instructions**: Per-project AGENTS.md-equivalent, editable from UI
- **Sources panel**: Attached files, connected GitHub repos, KB query results
- **Rename / Pin / Archive / Search**: Full chat management

**NeoTrix backend integration**:
- Chat session state stored in KB (via `nt_memory_kb`)
- Project metadata in KB with `NodeType::Project` / `NodeType::Chat`
- Agent runs via `nt_act_autonomy` / `nt_core_self` SEAL pipeline

### 2.2 Chat / Agent Interface (P0)

**Features**:
- Streaming markdown rendering (code blocks, tables, math)
- File preview inline (documents, spreadsheets, presentations, PDF, images)
- Annotation-based refinement (select area in preview → request change)
- Tool call visualization (expandable tool call cards with input/output)
- Model selector (multi-provider: OpenAI, Anthropic, local)
- Reasoning effort slider (quick / balanced / deep)
- `/` slash commands (same as Codex: `/new`, `/resume`, `/plugins`, etc.)

**Annotation system**:
- Canvas overlay on file previews
- Selection → contextual edit request
- Scope: code blocks, rendered docs, spreadsheets, images

### 2.3 Plugin Marketplace (P1)

**Reuses NeoTrix McpGateway** (`nt_agent_mcp_gateway.rs`).

**Marketplace UI**:
- Browse / Search plugins
- Plugin detail page (description, tools, permissions, screenshots)
- One-click install + OAuth connector setup
- Installed plugins panel with enable/disable toggles

**Plugin = Bundle of**:
- **Skills**: Reusable prompt instructions (YAML/MD)
- **Connectors**: OAuth-linked external services (GitHub, Slack, Gmail, Drive)
- **MCP Servers**: Local or remote tool providers
- **Hooks**: Lifecycle scripts (on_install, on_uninstall, on_task_start)
- **Scheduled task templates**: Preconfigured recurring workflows

**Marketplace sources**:
- Personal (local filesystem: `~/.neocodex/plugins/`)
- Workspace (shared directory, git repo)
- OpenAI-style curated catalog (future)

### 2.4 Scheduled Tasks (P1)

**RRULE-based recurring task engine**:
- Create task from chat prompt or scheduled view
- Schedule: daily, weekly, custom RRULE
- Worktree isolation (git worktree per run) or local mode
- Runs in background with notification on completion
- Task history with per-run logs and outputs
- Skill integration: `$skill-name` invocation in task prompt

**Backend**:
- Scheduler engine in Rust (`nt_act_autonomy` schedule executor)
- Persistence in KB (`CrawlQueueItem`-style scheduled task entries)
- macOS: BGTaskScheduler / launchd integration
- Windows: Task Scheduler integration

### 2.5 Computer Use (P1)

**macOS implementation** (Phase 1):
- Screen capture via `CGDisplayStream` / `SCContentFilter`
- Mouse/keyboard simulation via `CGEvent`
- Accessibility API for UI element inspection
- Plugin toggles: "Computer Use" skill + MCP server
- App whitelist: always-allowed apps list in settings
- Locked use: authorization plug-in for post-lock automation

**Windows implementation** (Phase 2):
- Win32 API for window management
- UI Automation for element inspection
- Foreground-only operation

**Safety**:
- Per-app approval prompts (always-allow list in config)
- Screen recording permission gate
- Sensitive action confirmation (credentials, payments)
- Visual indicator when Computer Use is active

### 2.6 File Viewer & Annotations (P1)

**Supported formats**:
- Documents: Markdown, HTML, PDF (native render)
- Spreadsheets: CSV, XLSX (tabular view with charts)
- Presentations: PPTX (slide viewer)
- Images: PNG, JPEG, GIF, SVG
- Code: Syntax-highlighted with line numbers

**Annotation overlay**:
- Canvas layer on top of rendered file
- Selection rectangle → contextual edit prompt
- Multiple annotation types: comment, change request, question
- Annotation history per file version

### 2.7 Knowledge Base Integration (P2)

- KB search panel (semantic + FTS5 + BM25)
- Graph view (D3.js knowledge graph — reuse `wiki graph` HTML)
- Tech reserve 4D explorer
- Crawl queue monitor (pending/completed/failed counts)
- Embedding status dashboard

### 2.8 Settings & Configuration (P2)

**Tabs**:
- **General**: Theme (light/dark/system), font size, language
- **Models**: Provider config (API keys, model selection, endpoint URLs)
- **Sandbox**: Permission mode selector (read-only / workspace-write / full access)
- **Plugins**: Installed plugins, connector status
- **Computer Use**: App whitelist, locked use toggle
- **Scheduled**: Global toggle, resource limits
- **Data**: Archive/restore chats, export, clear cache
- **About**: Version, build info, NeoTrix backend status

### 2.9 Long-Running Work & Notifications (P2)

- Background agent runs with progress indicator
- Notification center: bell icon in sidebar
- macOS: native Notification Center integration
- Run history with per-step logs
- Pause / resume / cancel controls

---

## 3. Integration Points with NeoTrix

| neocodex Feature | NeoTrix Backend | Status |
|-----------------|-----------------|--------|
| Chat session | `nt_memory_kb` KB nodes (Chat/Message types) | ✅ Existing |
| Agent execution | `nt_act_autonomy` goal loop | ✅ Existing |
| LLM calls | `nt_io_provider` gateway | ✅ Existing |
| Plugin system | `nt_agent_mcp_gateway` McpGateway | ✅ Already built (Cycle 81) |
| Knowledge search | `nt_memory_kb` FTS5/BM25/semantic | ⚠️ Embedding API key needed |
| Scheduled tasks | `nt_act_autonomy` schedule executor | ✅ Existing |
| File operations | `nt_world_crawl` fetcher + `nt_world_scrape` parser | ✅ Existing |
| Computer Use | NEW: `nt_act_computer_use` module | ❌ Not built |
| Skill system | `nt_core_skill_crystallizer` + `nt_io_skill_review` | ✅ Existing |
| Event bus | `nt_core_event` EventBus | ✅ Existing |

---

## 4. Implementation Roadmap

### Phase 0 — Scaffold (Week 1)
- [ ] `cargo init neocodex --lib` at `/Users/neo/Downloads/neocodex/`
- [ ] `npm create tauri-app` with SolidJS + TypeScript
- [ ] Wire `tauri.conf.json` (window size, title, permissions)
- [ ] Add `neotrix-core` as dependency in `Cargo.toml`
- [ ] Create basic IPC command: `greet`
- [ ] Build Sidebar + Chat shell (static mock)
- [ ] Verify `cargo tauri dev` opens blank window with sidebar

### Phase 1 — Chat MVP (Week 2)
- [ ] GatewayBridge: connect to NeoTrix McpGateway
- [ ] Streaming markdown renderer (SolidJS)
- [ ] Model selector + provider config
- [ ] Chat CRUD: create, list, switch, archive
- [ ] File attachment via drag-drop
- [ ] `/` slash command autocomplete
- [ ] Tool call visualization

### Phase 2 — Projects + KB (Week 3-4)
- [ ] Project CRUD (create from folder, switch, pin)
- [ ] Project instructions editor
- [ ] Sources panel (files, GitHub, KB)
- [ ] KB search integration
- [ ] Knowledge graph viewer (D3.js)
- [ ] Chat search (Cmd+G)

### Phase 3 — Plugins + Scheduled Tasks (Week 5-6)
- [ ] Plugin browser UI
- [ ] Plugin install/uninstall flow
- [ ] Connector OAuth setup
- [ ] McpGateway tool registration from plugins
- [ ] Scheduled task creation + management view
- [ ] RRULE editor
- [ ] Task run history + log viewer
- [ ] Worktree isolation for scheduled tasks

### Phase 4 — Computer Use (Week 7-8)
- [ ] Screen capture module (macOS)
- [ ] Mouse/keyboard simulation
- [ ] App whitelist UI
- [ ] Accessibility element inspector
- [ ] Locked use (authorization plug-in)
- [ ] Permission gate UI (screen recording + accessibility)
- [ ] Computer Use skill + MCP server

### Phase 5 — Polish (Week 9-10)
- [ ] File viewer + annotation system
- [ ] Notification center
- [ ] Background agent runs UI
- [ ] Settings pages (all tabs)
- [ ] Theming (dark/light/system)
- [ ] macOS notarization + code signing
- [ ] Windows installer

---

## 5. Design System

### 5.1 Faction Visual Identity

Extend the 7-domain Warhammer 40k faction aesthetic from NeoTrix:

| Faction | Color | Role in neocodex |
|---------|-------|-----------------|
| NT-CORE | Gold (#FFD700) | Agent reasoning visualization |
| NT-MIND | Cyan (#00E5FF) | Evolution/self-improvement panels |
| NT-MEMORY | Indigo (#4B0082) | KB search, graph view |
| NT-WORLD | Green (#00FF88) | Crawler, file operations |
| NT-ACT | Red (#FF3333) | Tool calls, action execution |
| NT-IO | Blue (#3399FF) | Chat bubbles, settings |
| NT-SHIELD | Gray (#666666) | Sandbox, permissions, security |

### 5.2 Typography

- UI: Inter (system font stack fallback)
- Code: JetBrains Mono / Fira Code
- Scale: 12/14/16/20/24/32/48px

### 5.3 Layout

```
┌──────────────────────────────────────────────────┐
│  Title Bar (custom, with sidebar toggle)          │
├──────┬───────────────────────────────────────────┤
│      │                                            │
│ Side │           Main Content Area                │
│ bar  │                                            │
│      │   ┌──────────────────────────────────┐    │
│      │   │  Chat / Project / Plugin View   │    │
│      │   └──────────────────────────────────┘    │
│      │                                            │
│      │   ┌──────────────────────────────────┐    │
│      │   │  File Preview / Graph / Output   │    │
│      │   └──────────────────────────────────┘    │
│      │                                            │
├──────┴───────────────────────────────────────────┤
│  Status Bar (model, tools, connection, progress)  │
└──────────────────────────────────────────────────┘
```

---

## 6. Key Principles

1. **Zero-unsafe backend**: `#![forbid(unsafe_code)]` in Tauri commands (reuse neotrix rule)
2. **Local-first**: All data stored locally in KB SQLite; cloud sync is optional add-on
3. **Sandbox by default**: New projects start in read-only mode; escalate explicitly
4. **Plugin isolation**: Each plugin runs as its own MCP server process; crash isolation
5. **Self-documenting**: neocodex maintains its own AGENTS.md for agent onboarding
6. **Backward compatible**: All IPC commands versioned; UI can fall back gracefully

---

## 7. Open Questions

1. **Monorepo or separate repo?** neocodex as sibling vs within neotrix workspace
2. **Native vs web-first?** Tauri with full native capabilities vs Electron-style web app
3. **Maturity target?** C1 (unit tests) or C2 (integration tests) before first release?
4. **macOS-only or cross-platform?** Mac first then Windows, or both from day one?
5. **KB embedding API key?** Required for semantic search — env var or settings UI?
6. **Agent approval UI?** Sandbox permission prompts inline in chat vs modal dialogs?
