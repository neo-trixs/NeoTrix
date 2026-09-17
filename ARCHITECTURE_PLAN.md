# NeoTrix Frontend Architecture Improvement Plan

**Date**: 2026-09-17
**Status**: Draft → Ready for Review
**Version**: v0.2.0-plan

---

## 0. Executive Summary

This document is the output of a reverse-engineering exercise: starting from what NeoTrix *should* be (an AI-native developer toolkit), working backwards to what the frontend architecture *must* look like, then comparing against what actually exists. The gap analysis produces prioritized, actionable improvements with effort estimates.

**Key Finding**: NeoTrix has 14+ features scattered across 85+ components with no architectural boundaries. The codebase is functional but fragile — every change risks cascading regressions because there are no dependency rules. The fix is not a rewrite but a **structured migration** to Feature-Sliced Design (FSD), the industry-standard methodology for frontend architecture at scale.

---

## 1. Reverse-Engineering: From Capabilities to Architecture

### 1.1 What NeoTrix Must Do (Product Capabilities)

| Capability | Frontend Responsibility | Architectural Implication |
|---|---|---|
| AI Chat | Message I/O, streaming, tool results, approval flow | Chat is the primary surface; must be decomposable into independent features |
| Knowledge Graph | Node/edge visualization, search, filtering, zoom | Canvas module; heavy rendering; must not block chat thread |
| Domain Plugins | Per-domain UI injection (file, world, sense, etc.) | Plugin boundary: each domain owns its UI slice, no cross-imports |
| Agent Orchestrator | Task dispatch, progress tracking, approval | State management must track async agent lifecycle |
| Self-Model | Attention allocation, fatigue tracking, emotion | Background store; read-heavy, write-light; must not leak into UI layer |
| Physical World | Sensor visualization, motor commands, safety | High-frequency updates; must use signals/reactivity, not re-render loops |
| File Abilities | PDF/XLSX/DOCX preview, super-resolution, extraction | Heavy processing; offload to backend, show progress in UI |
| Cognitive Map | Causal chains, timeline, decision nodes | Canvas node kind; must share data model with chat |
| Memory | Session history, cross-session knowledge graph | Persistent store; must survive hot reload; indexed for search |
| Shield/Safety | Permission gates, risk scoring, audit log | Cross-cutting concern; must be injectable into any feature |

### 1.2 What the Architecture Must Provide

From the capabilities above, the frontend architecture must satisfy these **invariants**:

1. **Feature Isolation**: Each capability (chat, knowledge graph, domain plugin) must be modifiable without affecting others.
2. **Shared State with Clear Ownership**: Multiple features read the same data (e.g., chat messages, agent state), but only one feature owns each piece of state.
3. **Backend-Heavy, Frontend-Thin**: Business logic lives in Rust. Frontend is presentation + user interaction only. No computation in components.
4. **Plugin Boundary**: Domain plugins must be addable/removable without touching core code.
5. **Streaming Reactivity**: Chat, tool results, and agent progress must stream in without blocking the UI thread.
6. **Performance Isolation**: Canvas (knowledge graph, cognitive map) must not degrade chat responsiveness.

---

## 2. Current State Analysis

### 2.1 What Exists Today

```
neocodex-frontend/src/
├── api/           # 2 modules: neocodex.ts, domain.ts (incompatible types)
├── App.tsx        # 14 lazy routes, 3-panel layout wiring
├── canvas/        # SmartCanvas, bridge, 11 canvas node kinds, 12 route kinds
├── components/    # 85+ components, no grouping by feature
├── layout/        # Orphaned independent LLM chat UI (unused by main app)
├── lib/           # Direct Tauri API import (bypasses api/ layer)
├── routes/        # 14 page-level components, some monolithic
├── stores/        # 8 stores, no ownership boundaries
├── styles/        # Single index.css (all styles, 3000+ lines)
└── test/          # Minimal test coverage
```

### 2.2 Critical Issues (Severity: Blocker)

| # | Issue | Evidence | Impact |
|---|---|---|---|
| C1 | **Chat.tsx is a 2311-line monolith** | `routes/Chat.tsx:1-2311` | Impossible to maintain; any change risks regressions |
| C2 | **Dual API layer** | `api/neocodex.ts` (attachment bug at line 91) + `api/domain.ts` (incompatible types) + `lib/api.ts` (bypasses both) | Type unsafety; inconsistent error handling |
| C3 | **No architectural boundaries** | Components import from each other freely; stores cross-reference | Changing one feature breaks unrelated features |
| C4 | **All styles in one file** | `styles/index.css` (3000+ lines) | No style isolation; cascade conflicts |

### 2.3 Medium Issues (Severity: Degradation)

| # | Issue | Evidence | Impact |
|---|---|---|---|
| M1 | **RightBar uses static demo data** | `RightBar.tsx` hardcodes `DEMO_CHAINS` | Not connected to real chatStore |
| M2 | **Attachment sending broken** | `api/neocodex.ts:91` — only forwards `content` | Users cannot send files |
| M3 | **layout/ module orphaned** | Complete independent chat UI not referenced by App.tsx | Dead code; confusion |
| M4 | **No testing strategy** | 85+ components, minimal tests | Regressions go undetected |
| M5 | **Unused dependencies** | `globe.gl`, `three`, `topojson-client` imported but not used in main flow | Bundle bloat |

---

## 3. Target Architecture: Feature-Sliced Design (FSD)

### 3.1 Why FSD

FSD is chosen over alternatives (Atomic Design, Clean Architecture, MVC) because:

- **Feature-first**: Organizes by business capability, not technical type. Matches NeoTrix's domain plugin model.
- **Layered dependency rules**: Explicit "what can import what" prevents C3 (no architectural boundaries).
- **Framework-agnostic**: Works with SolidJS, React, Vue — no lock-in.
- **Gradual adoption**: Can migrate slice-by-slice without a full rewrite.
- **Tooling**: `steiger` linter enforces architecture rules in CI.

### 3.2 Target Directory Structure

```
neocodex-frontend/src/
├── app/                          # App bootstrap, routing, providers
│   ├── index.tsx                 # Entry point
│   ├── router.tsx                # Route definitions (lazy-loaded)
│   ├── providers.tsx             # Context providers (theme, query)
│   └── styles/                   # Global styles, CSS variables
│
├── pages/                        # Route-level components (thin wrappers)
│   ├── chat/                     # Chat page
│   │   └── index.tsx             # Composes widgets, handles routing
│   ├── knowledge/                # Knowledge graph page
│   │   └── index.tsx
│   ├── canvas/                   # Canvas page
│   │   └── index.tsx
│   └── settings/                 # Settings page
│       └── index.tsx
│
├── widgets/                      # Composite UI blocks (page sections)
│   ├── chat-panel/               # Chat message list + input
│   │   ├── ui/                   # ChatPanel component
│   │   ├── model/                # Message list state
│   │   └── lib/                  # Message formatting utils
│   ├── right-sidebar/            # Right panel (CausalMap, tools)
│   │   ├── ui/                   # RightSidebar, CausalMap
│   │   ├── model/                # Causal chain state
│   │   └── api/                  # Causal chain API
│   ├── canvas-viewer/            # Canvas/graph visualization
│   │   ├── ui/                   # SmartCanvas wrapper
│   │   ├── model/                # Node/edge state
│   │   └── lib/                  # Layout algorithms
│   └── tool-panel/               # Tool results, permission gates
│       ├── ui/                   # ToolResult, ToolPermission
│       └── model/                # Tool execution state
│
├── features/                     # User-facing interactions + cross-page flows
│   ├── send-message/             # Send message action
│   │   ├── ui/                   # SendButton, InputBar
│   │   ├── model/                # Message creation logic
│   │   └── api/                  # sendMessage API call
│   ├── stream-response/          # Stream AI response (was processes/streaming)
│   │   ├── ui/                   # StreamingText
│   │   ├── model/                # Stream state machine
│   │   └── api/                  # Stream connection
│   ├── approval/                 # Human-in-the-loop approval (was processes/approval)
│   │   ├── model/                # Approval state machine
│   │   └── ui/                   # Approval buttons, modal
│   ├── edit-message/             # Edit sent message
│   │   ├── ui/                   # EditButton, EditModal
│   │   └── model/                # Edit state
│   └── search-knowledge/         # Search knowledge graph
│       ├── ui/                   # SearchBar, Results
│       └── api/                  # Search API
│
├── entities/                     # Business entities
│   ├── message/                  # Message entity
│   │   ├── model/                # Message type, state
│   │   ├── api/                  # Message CRUD API
│   │   └── ui/                   # MessageBubble, MessageContent
│   ├── tool/                     # Tool execution entity
│   │   ├── model/                # ToolCall type, state
│   │   └── ui/                   # ToolResult
│   ├── session/                  # Chat session entity
│   │   ├── model/                # Session type, state
│   │   └── api/                  # Session API
│   ├── agent/                    # Agent entity
│   │   ├── model/                # Agent state, capabilities
│   │   └── ui/                   # AgentIndicator
│   └── domain/                   # Domain plugin entity
│       ├── model/                # Domain type, registry
│       └── api/                  # Domain API
│
├── shared/                       # Reusable, business-agnostic
│   ├── ui/                       # Design system primitives
│   │   ├── button/
│   │   ├── input/
│   │   ├── icon/                 # Unified Lucide icon system
│   │   ├── layout/               # Panel, SplitView, etc.
│   │   └── index.ts              # Barrel export
│   ├── lib/                      # Utility functions
│   │   ├── tauri-bridge.ts       # Single Tauri API entry point
│   │   ├── format.ts             # Date/number formatting
│   │   └── cn.ts                 # clsx + tailwind-merge
│   ├── api/                      # Shared API utilities
│   │   ├── client.ts             # HTTP/SSE client
│   │   └── types.ts              # Shared API types
│   └── config/                   # App configuration
│       └── constants.ts
│
└── canvas/                       # Canvas system (keep as-is, refactor later)
    ├── nodes/                    # Canvas node kinds
    ├── bridge/                   # Frontend ↔ backend bridge
    └── ...
```

### 3.3 Dependency Rules (FSD Layer Hierarchy)

```
app → pages → widgets → features → entities → shared
```

**Hard rules**:
- A module can ONLY import from layers **below** it
- A module can NOT import from the **same** layer (no sibling imports within a layer)
- `shared/` has NO dependencies on any other layer
- Each module exposes a **public API** (`index.ts`) — no deep imports

> **Note**: FSD v2.1 deprecated the `processes` layer. Cross-page flows (streaming, approval) are now placed in `features/`.

### 3.4 State Ownership Model

| Store | Owner Layer | Readers | Notes |
|---|---|---|---|
| `chat` | `entities/message/model/` | All chat features | Single source of truth for messages |
| `session` | `entities/session/model/` | Chat, memory | Session lifecycle |
| `agent` | `entities/agent/model/` | Chat (progress), approval | Agent state machine |
| `knowledge` | `entities/domain/model/` (knowledge slice) | Knowledge graph page | Graph data |
| `canvas` | `widgets/canvas-viewer/model/` | Canvas page | Node/edge rendering |
| `ui` | `shared/ui/` | Everything | Theme, sidebar state |
| `self-model` | `entities/agent/model/` (self slice) | Background only | Attention, fatigue |
| `streaming` | `processes/streaming/model/` | Chat features | Stream buffer |

---

## 4. Migration Plan

### Phase 1: Foundation (Week 1-2) — Est. 16h

**Goal**: Establish the skeleton without breaking anything.

| Task | Effort | Risk | Details |
|---|---|---|---|
| 1.1 Create FSD directory skeleton | 2h | Low | Create empty `app/`, `processes/`, `pages/`, `widgets/`, `features/`, `entities/`, `shared/` with `index.ts` barrel files |
| 1.2 Extract `shared/ui/` | 4h | Low | Move `Button`, `Icon`, `Input`, `Panel`, `SplitView` to `shared/ui/`. Update imports. Verify build. |
| 1.3 Extract `shared/lib/` | 2h | Low | Move `cn.ts`, `format.ts`, Tauri bridge to `shared/lib/`. Single entry point for Tauri API. |
| 1.4 Extract `shared/api/` | 2h | Medium | Unify `api/neocodex.ts` + `api/domain.ts` + `lib/api.ts` into single `shared/api/client.ts`. Fix attachment bug (C2). |
| 1.5 Create `entities/message/` | 3h | Medium | Extract message types, `MessageBubble`, `MessageContent` from `components/`. Wire to `chat` store. |
| 1.6 Create `entities/tool/` | 2h | Low | Extract `ToolResult` from `components/`. |
| 1.7 Add `steiger` linter | 1h | Low | Install `steiger`, configure FSD rules, add to `package.json` scripts. |

**Verification**: `npm run typecheck` passes, `npm run build` succeeds, all routes render.

### Phase 2: Chat Decomposition (Week 3-4) — Est. 24h

**Goal**: Break the 2311-line Chat.tsx monolith into composable features.

| Task | Effort | Risk | Details |
|---|---|---|---|
| 2.1 Extract `widgets/chat-panel/` | 6h | High | Extract message list, input area, scroll behavior from Chat.tsx into `ChatPanel`. |
| 2.2 Extract `widgets/right-sidebar/` | 4h | Medium | Extract RightBar content, CausalMap, ToolResult list into `RightSidebar`. Connect to real chatStore (fix M1). |
| 2.3 Extract `features/send-message/` | 3h | Medium | Extract send logic, keyboard shortcuts, permission mode into feature slice. Fix attachment bug (C2). |
| 2.4 Extract `features/stream-response/` | 4h | High | Extract streaming connection, chunk buffering, text assembly into process slice. Fix StreamingText flicker permanently. |
| 2.5 Extract `features/approve-tool/` | 3h | Low | Extract approval flow from ToolResult into feature slice. |
| 2.6 Extract `features/edit-message/` | 2h | Low | Extract edit logic from MessageBubble into feature slice. |
| 2.7 Rewrite `pages/chat/` as thin wrapper | 2h | Low | Compose `ChatPanel` + `RightSidebar` + tool panel. <200 lines. |

**Verification**: Chat works identically. Each feature can be modified independently. `steiger` passes.

### Phase 3: Domain Plugin Boundary (Week 5) — Est. 12h

**Goal**: Each domain plugin owns its UI slice.

| Task | Effort | Risk | Details |
|---|---|---|---|
| 3.1 Define domain plugin UI contract | 3h | Medium | TypeScript interface: `DomainPluginUI { render(): Component, routes: Route[] }` |
| 3.2 Migrate file-domain UI | 4h | Medium | Move file-related components (PDF preview, XLSX viewer, image resolver) to `entities/domain/file/` |
| 3.3 Migrate knowledge-domain UI | 3h | Low | Move knowledge graph components to `entities/domain/knowledge/` |
| 3.4 Remove `layout/` orphaned module | 1h | Low | Delete or archive (fix M3) |
| 3.5 Remove unused dependencies | 1h | Low | Remove `globe.gl`, `three`, `topojson-client` if unused (fix M5) |

**Verification**: Domain plugins load independently. Removing a plugin doesn't break core.

### Phase 4: Styling & Testing (Week 6) — Est. 16h

**Goal**: Style isolation and regression safety.

| Task | Effort | Risk | Details |
|---|---|---|---|
| 4.1 Split `styles/index.css` by feature | 4h | Medium | Each feature/entity gets co-located styles. Global CSS variables stay in `app/styles/`. |
| 4.2 Add CSS module or Tailwind per-feature | 4h | Medium | Prevent cascade conflicts between features. |
| 4.3 Add unit tests for entities | 4h | Low | Test message, tool, session models. |
| 4.4 Add integration tests for features | 4h | High | Test send-message, stream-response, approve-tool flows. |

**Verification**: CSS changes in one feature don't affect others. Test coverage >60% for entities.

---

## 5. Version Management & Development Standards

### 5.1 Versioning Strategy

| Component | Format | Increment Rule |
|---|---|---|
| Frontend app | `MAJOR.MINOR.PATCH` (SemVer) | MAJOR = breaking UI change, MINOR = new feature, PATCH = bugfix |
| Backend (neotrix-core) | `MAJOR.MINOR.PATCH` | Same as frontend |
| Workspace root | `MAJOR.MINOR.PATCH` | Bump when any sub-crate has breaking change |

**Current**: Frontend v0.1.0, Core v0.21.0 → After architecture migration: Frontend v0.2.0

### 5.2 Development Standards

#### Branch Strategy
```
main (stable) ← develop (integration) ← feature/* (per-feature)
                                         fix/* (bugfix)
                                         refactor/* (architecture)
```

#### Commit Convention
```
<type>(<scope>): <description>

type: feat | fix | refactor | style | test | docs | chore
scope: chat | knowledge | canvas | domain | shared | api | store
description: imperative, <72 chars
```

Examples:
```
feat(chat): extract ChatPanel widget from Chat.tsx monolith
fix(api): forward attachments in sendMessageStream
refactor(shared): unify dual API layer into single client
style(chat): co-locate message styles with MessageBubble entity
test(tool): add unit tests for ToolResult args parsing
```

#### Code Review Checklist
- [ ] FSD layer hierarchy respected (no upward imports)
- [ ] Public API exported from `index.ts` (no deep imports)
- [ ] Component <300 lines (if larger, decompose)
- [ ] Store changes include type tests
- [ ] New features have at least 1 integration test
- [ ] `steiger` passes (architecture lint)
- [ ] `npm run typecheck` passes
- [ ] `npm run build` succeeds
- [ ] CSS changes don't affect other features (visual regression)

#### PR Template
```markdown
## What
Brief description of change.

## Why
Link to issue or rationale.

## How
Key implementation decisions.

## Verification
- [ ] `npm run typecheck` passes
- [ ] `npm run build` passes
- [ ] Manual testing: <describe what you tested>
- [ ] `steiger` passes (if architecture change)

## Rollback
How to revert if needed.
```

### 5.3 CHANGELOG Enforcement

Every PR that touches user-facing code MUST update `neocodex-frontend/CHANGELOG.md`:

```markdown
## [version] - YYYY-MM-DD

### Added
- New feature X

### Changed
- Modified behavior Y

### Fixed
- Bug Z

### Removed
- Deprecated feature W

### Architecture
- Structural changes (migration, refactoring)
```

---

## 6. Risk Assessment

| Risk | Probability | Impact | Mitigation |
|---|---|---|---|
| Chat.tsx decomposition breaks streaming | High | Critical | Phase 2.4 first; feature-flag new vs old path |
| FSD migration introduces import cycles | Medium | High | `steiger` linter in CI; manual review |
| Performance regression from added layers | Low | Medium | Benchmark before/after each phase |
| Domain plugin contract too rigid | Medium | Medium | Design contract as interface, not class |
| Team doesn't follow conventions | Medium | High | Automated enforcement (steiger, CI checks) |

---

## 7. Success Metrics

| Metric | Current | Target (Post-Migration) |
|---|---|---|
| Largest component | 2311 lines (Chat.tsx) | <300 lines |
| Components with tests | <10% | >60% |
| CSS file count | 1 (3000+ lines) | ~20 (feature-co-located) |
| API entry points | 3 (incompatible) | 1 (unified) |
| Architecture lint violations | Unknown | 0 (steiger enforced) |
| Build time (frontend) | ~8s | <10s (no regression) |
| Time to add new feature | High (touch many files) | Low (add slice in correct layer) |

---

## 8. References

- [Feature-Sliced Design](https://feature-sliced.design/) — Official documentation
- [Steiger](https://github.com/feature-sliced/steiger) — FSD architecture linter
- [SolidJS + Monorepo](https://moonrepo.dev/docs/guides/examples/solid) — Monorepo patterns
- [Tauri 2 Project Structure](https://v2.tauri.app/start/project-structure/) — Official Tauri structure
- [Tauri Architecture Guide](https://github.com/dannysmith/tauri-template/blob/main/docs/developer/architecture-guide.md) — State management patterns
- [UNIVERSAL_ARCHITECTURE_V6.md](./UNIVERSAL_ARCHITECTURE_V6.md) — NeoTrix architecture fusion document
