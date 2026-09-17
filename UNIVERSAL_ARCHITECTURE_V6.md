# NeoTrix Desktop App — Round 7 Architecture Fusion
## 2026-09-17 · 40+ Products Researched · 7 Iteration Cycles

---

## 1. Research Findings (Round 7 — Latest 2026)

### Cursor 3.7 (June 2026)
| Feature | Description | NeoTrix Mapping |
|---------|-------------|-----------------|
| **Design Mode** | Multi-element selection + voice queuing for UI editing | `panel-design` → enhance with multi-select |
| **Context Usage Report** | Canvas artifact showing token breakdown | `ContextDashboard` route ✅ built |
| **Shared Canvases** | Share interactive artifacts with teammates | `panel-canvas` → add share capability |
| **Auto-review** | Classifier subagent for tool approval | `nt_shield` auto-review mode |
| **/loop Skill** | Long-running agents on local schedule | `nt_mind_background_loop` already exists |
| **Agent Tabs** | Multi-chat side-by-side | SmartCanvas already supports |

### Claude Desktop (2026)
| Feature | Description | NeoTrix Mapping |
|---------|-------------|-----------------|
| **Three Tabs** | Chat / Cowork / Code | Sidebar already has view modes |
| **Connectors** | MCP servers with graphical setup | `panel-extensions` already exists |
| **Computer Use** | Screen interaction for any app | `nt_world_crawl` already exists |
| **Skills** | Reusable workflows | `route-skills` already exists |
| **Parallel Sessions** | Git-isolated concurrent sessions | `nt_workspace_isolator` already exists |
| **Side Chats** | Inline conversation branches | `handleForkSession` already exists |
| **Visual Diff Review** | Inline diff inspection | Canvas `diff` node kind exists |

### Windsurf/Devin Desktop (2026)
| Feature | Description | NeoTrix Mapping |
|---------|-------------|-----------------|
| **Agent Command Center** | Kanban view of all agents | `panel-kanban` already exists |
| **Spaces** | Shared context containers | `panel-spaces` already exists |
| **ACP Protocol** | Open agent-editor standard | `panel-acp` already exists |
| **Devin Local** | Rust rewrite, 30% more efficient | NeoTrix is already Rust-native |
| **Subagent Support** | Spawn parallel sub-sessions | `nt_multi_agent` already exists |
| **One-click Handoff** | Local → Cloud agent transfer | Need: `panel-handoff` |
| **Memories & Rules** | Customization persistence | `experience-tree` + KB already exists |
| **Turbo Mode** | Auto-execute commands | Need: auto-execution config |

### Token Optimization Research (August 2026)
| Pattern | Description | NeoTrix Mapping |
|---------|-------------|-----------------|
| **Context Stratification** | Separate system/conversation/tool contexts | `ContextCompactor` already exists |
| **Schema-Contracted Prompts** | Compact JSON contracts | Backend API already uses JSON |
| **Token-Aware Fallbacks** | Cached repeat loads | `prompt_cache` already exists |
| **Inter-Agent Compression** | Schema-contracted handoffs | Need: agent communication schema |
| **Relevance-Contrast** | Mix high/low relevance items | `ContextCompactor` relevance scoring |

### AI Agent Orchestration Patterns (2026)
| Pattern | Description | NeoTrix Mapping |
|---------|-------------|-----------------|
| **Sequential** | Linear pipeline | `nt_workflow` already exists |
| **Concurrent** | Parallel agents | `nt_dual_executor` already exists |
| **Handoff** | Dynamic delegation | Need: agent handoff UI |
| **Magentic** | Plan-build-execute with ledger | `plan` node kind exists |
| **Group Chat** | Shared conversation thread | SmartCanvas already supports |
| **HITL Gates** | Human approval checkpoints | `ActionAuthorizer` already exists |

---

## 2. Gap Analysis — What NeoTrix Already Has vs. What's Missing

### ✅ Already Built (Solid Foundation)
1. **Agent Command Center** — `panel-kanban` with 4-column Kanban (排队/运行/审查/完成)
2. **Spaces** — `panel-spaces` for shared context containers
3. **Context Usage** — `panel-context` + `ContextDashboard` route
4. **Knowledge Graph** — `KnowledgeGraph` route with visual exploration
5. **ACP Protocol** — `panel-acp` for agent communication
6. **MCP Extensions** — `panel-extensions` for tool integration
7. **Design Mode** — `panel-design` for browser annotation
8. **Voice** — `panel-voice` for STT/TTS
9. **Workflow** — `route-workflows` for task automation
10. **Memory** — `route-memory` + KB system
11. **Skills** — `route-skills` + `experience-tree`
12. **Activity** — `route-activity` + `activity` canvas node
13. **Plan Mode** — `plan` canvas node for execution planning
14. **Flow Visualization** — `flow` canvas node for agent execution chains
15. **3-Panel Layout** — Sidebar + Chat + SmartCanvas

### 🔲 Missing Capabilities (To Build)

#### High Priority (Core UX)
1. **CostTracker Panel** — Real-time API cost breakdown (AI Token Monitor pattern)
2. **ModelSelector Panel** — Quick model switching with capability indicators
3. **GitStatus Panel** — Git integration view (branch, diff, PR status)
4. **Terminal Panel** — Embedded terminal output viewer
5. **Handoff Panel** — Local → Cloud agent transfer UI

#### Medium Priority (Enhancement)
6. **Notification Center** — Centralized alerts and status updates
7. **Session Replay** — Replay past agent sessions step-by-step
8. **Auto-Execution Config** — Turbo/Auto/Off modes for tool execution
9. **Connector Manager** — Graphical MCP server setup (Claude-style)
10. **PR Monitor** — Pull request status and review queue

#### Low Priority (Future)
11. **Shared Canvas Links** — Generate shareable canvas URLs
12. **Voice Queue** — Non-blocking voice input during agent runs
13. **Playbook Library** — Reusable workflow templates
14. **Leaderboard** — Usage comparison across sessions

---

## 3. Architecture Fusion — NeoTrix Optimal Design

### 3.1 The Three-Panel Architecture (Final)

```
┌─────────────┬──────────────────────┬─────────────────────┐
│   Sidebar   │       Chat           │    SmartCanvas      │
│   (280px)   │    (flex-1)          │    (380px)          │
│             │                      │                     │
│ ┌─────────┐ │ ┌──────────────────┐ │ ┌─────────────────┐ │
│ │ Search  │ │ │ Message Thread   │ │ │ Canvas Nodes    │ │
│ │ New Chat│ │ │                  │ │ │ (Drag & Drop)   │ │
│ │ Sessions│ │ │ • User Messages  │ │ │                 │ │
│ │         │ │ │ • Agent Replies  │ │ │ • Agent Kanban  │ │
│ │ Pinned  │ │ │ • Tool Results   │ │ │ • Context Usage │ │
│ │ Groups  │ │ │ • Code Blocks    │ │ │ • Knowledge Grp │ │
│ │         │ │ │ • Images         │ │ │ • Cost Tracker  │ │
│ │ Archive │ │ │                  │ │ │ • Model Select  │ │
│ │         │ │ ┌──────────────────┐ │ │ • Git Status    │ │
│ │         │ │ │ Input Bar        │ │ │ • Terminal      │ │
│ │         │ │ │ • Model Select   │ │ │ • Flow Visual   │ │
│ │         │ │ │ • Attachments    │ │ │ • Plan Mode     │ │
│ │         │ │ │ • Voice Input    │ │ │                 │ │
│ │         │ │ │ • Send Button    │ │ └─────────────────┘ │
│ │         │ │ └──────────────────┘ │                     │
│ │ Footer  │ └──────────────────────┘                     │
│ │ Settings│                                              │
│ └─────────┘                                              │
└─────────────┴──────────────────────┴─────────────────────┘
```

### 3.2 Capability Network (Final)

```
                    ┌─────────────────┐
                    │  NeoTrix Core   │
                    │  (Rust Backend) │
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
        ┌─────┴─────┐ ┌─────┴─────┐ ┌─────┴─────┐
        │  Domain   │ │  Agent    │ │  Memory   │
        │  Plugins  │ │  System   │ │  System   │
        └─────┬─────┘ └─────┬─────┘ └─────┬─────┘
              │              │              │
              └──────────────┼──────────────┘
                             │
                    ┌────────┴────────┐
                    │  SmartCanvas    │
                    │  (Capability    │
                    │   Registry)     │
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
        ┌─────┴─────┐ ┌─────┴─────┐ ┌─────┴─────┐
        │  Panel    │ │  Route    │ │  Data     │
        │  Nodes    │ │  Nodes    │ │  Nodes    │
        └───────────┘ └───────────┘ └───────────┘
```

### 3.3 Panel Node Registry (Final)

| Kind | Label | Category | Source |
|------|-------|----------|--------|
| `panel-kanban` | Agent 看板 | agent | Devin Command Center |
| `panel-spaces` | 工作空间 | agent | Devin Spaces |
| `panel-context` | 上下文用量 | agent | Cursor Context Report |
| `panel-cost` | 成本追踪 | system | AI Token Monitor |
| `panel-model` | 模型选择 | system | Claude/Cursor model picker |
| `panel-git` | Git 状态 | tool | IDE git integration |
| `panel-terminal` | 终端 | tool | IDE terminal panel |
| `panel-handoff` | 代理移交 | agent | Devin one-click handoff |
| `panel-voice` | 语音 | tool | Claude voice input |
| `panel-extensions` | MCP 扩展 | tool | Claude Connectors |
| `panel-instructions` | 项目指令 | tool | AGENTS.md reader |
| `panel-sparse` | 上下文守护 | tool | Context compaction |
| `panel-design` | Design Mode | tool | Cursor Design Mode |
| `panel-acp` | ACP 协议 | agent | Devin ACP |
| `panel-files` | 文件预览 | data | IDE file viewer |
| `panel-filetree` | 文件树 | data | IDE file tree |
| `panel-map` | 3D 地球 | data | NT-WORLD |
| `panel-project` | 项目 | data | Project config |
| `panel-canvas` | 交互构件 | data | Canvas artifacts |
| `flow` | 执行流程 | agent | Agent execution chain |
| `plan` | 执行计划 | agent | Plan mode |
| `activity` | 活动流 | agent | Activity feed |
| `chart` | 图表 | data | Data visualization |
| `table` | 表格 | data | Tabular data |
| `code` | 代码 | data | Code blocks |
| `diff` | 差异 | data | Git diff |
| `json` | JSON | data | JSON viewer |
| `markdown` | 文档 | data | Markdown renderer |
| `mermaid` | 流程图 | data | Mermaid diagrams |
| `image` | 图像 | data | Image viewer |
| `kpi` | 指标 | data | KPI dashboard |

---

## 4. Implementation Plan

### Phase 1: Canvas Panel Renderers (Immediate)
- [x] `KnowledgeGraph` route — Visual knowledge exploration
- [x] `ContextDashboard` route — Full context analytics
- [ ] `panel-cost` renderer — Cost tracker with breakdown
- [ ] `panel-model` renderer — Model selector with capabilities
- [ ] `panel-git` renderer — Git status view
- [ ] `panel-terminal` renderer — Terminal output viewer
- [ ] `panel-handoff` renderer — Agent handoff UI

### Phase 2: Capability Integration (Next)
- [ ] Update `capabilities.ts` with new entries
- [ ] Update `App.tsx` with new routes
- [ ] Update `panelRenderers.tsx` with new renderers
- [ ] Add CSS for new panel types

### Phase 3: Backend Wiring (Future)
- [ ] Wire CostTracker to actual token usage data
- [ ] Wire ModelSelector to provider config API
- [ ] Wire GitStatus to git operations API
- [ ] Wire Terminal to shell execution API
- [ ] Wire Handoff to agent handoff API

---

## 5. Key Design Principles (Final)

1. **Capability = Canvas Node** — Every feature is a node on the SmartCanvas
2. **Route = Full-screen Node** — Routes are just nodes opened full-screen
3. **Panel = Embedded Node** — Panels are nodes embedded in the canvas
4. **Minimal Sidebar** — Only search, new chat, session list, archive, settings
5. **Context-Aware** — Canvas intelligently shows key nodes from conversation
6. **Drag & Drop** — Nodes can be rearranged freely on the canvas
7. **Rust-Native** — Backend in Rust for performance and safety
8. **MCP-Compatible** — Standard tool integration protocol
9. **ACP-Compatible** — Agent communication protocol support
10. **Token-Aware** — Context management and cost optimization built-in
