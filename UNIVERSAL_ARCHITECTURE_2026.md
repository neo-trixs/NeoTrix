# NeoTrix Desktop — Universal Optimal Architecture v2 (2026-09-16)

## Research Summary: Latest Desktop AI Patterns (Sep 2026)

### 1. ChatGPT Desktop Super App (OpenAI, Sep 2026)
- **Three Unified Surfaces**: Chat (conversation) + Work (long tasks) + Codex (coding)
- **Work Mode**: Research, analyze, create documents/spreadsheets/presentations/reports/Sites
- **Codex**: Dedicated coding with worktrees, Git review, inline editing, PR review
- **Cloud Sync**: Cloud Work conversations sync across web/mobile/desktop
- **Workspace Agents**: Run in Slack, scheduled tasks, connected apps
- **Record & Replay**: Demonstrate workflow once → reusable skill
- **Built-in Browser**: Website tools, tab mentions, browser control
- **Voice in Work/Codex**: Speak naturally, interrupt, coordinate tasks
- **Computer History**: Activity across apps/sites → memories
- **Skills (GA)**: Enterprise/Edu plugin directory

### 2. Claude Cowork (Anthropic, Aug 2026)
- **Dual Execution**: Cloud (default) + Local VM
  - Cloud: Agent loop on Anthropic servers, isolated sandbox per session
  - Local: Agent loop natively, code execution in VM (Apple Virtualization.framework / Hyper-V)
- **Cross-device**: Start on desktop, continue on web/mobile
- **Scheduled Tasks**: Run in cloud (no device needed)
- **Projects**: Persistent workspaces with files, context, instructions, memory
- **MCP Ecosystem**: Desktop extensions (.mcpb packages), one-click install
- **Three Approval Modes**: Auto (safety-reviewed), Manual (ask), Skip (no checks)
- **Browser Use**: Built-in browser or Chrome, imported cookies
- **Folder Instructions**: Project-specific context per folder

### 3. Local-First AI Architecture (2026)
- **Three-Tier Inference**: Local (70-80% calls) → Edge → Cloud
- **CRDTs**: Offline-first sync, strong eventual consistency
- **SQLite-Sync**: CRDT-based sync for SQLite (no conflicts, no data loss)
- **Privacy by Default**: Data never leaves device
- **EU AI Act Compliance**: Data residency requirements (Aug 2, 2026)
- **Cost Optimization**: Local inference for routine tasks
- **Hybrid Cloud-Local**: Local models for privacy/cost, cloud for complexity

### 4. Multi-Agent Orchestration (2026)
- **IACT**: Interactive Agents Call Tree (recursive spawning)
- **Hippocampus**: Global associative memory for cross-agent context
- **Coordinator/Specialist/Verifier**: Three-tier architecture
- **Living Spec**: Evolving plan that agents read and update
- **Workspace Isolation**: Git worktrees per agent

---

## Universal Architecture Patterns (10 Patterns)

| # | Pattern | Products | NeoTrix Status |
|---|---------|----------|----------------|
| 1 | **Unified Surface** (Chat+Work+Code) | ChatGPT Desktop, Claude Cowork | ❌ Missing |
| 2 | **Dual Execution** (Cloud+Local VM) | Claude Cowork | ❌ Missing |
| 3 | **Cross-device Session Sync** | ChatGPT, Claude, Manus | ❌ Missing |
| 4 | **Scheduled Tasks** (Cloud execution) | ChatGPT, Claude, OpenHands | ❌ Missing |
| 5 | **Skills/Knowledge Bundles** | ChatGPT, Cursor, Copilot | ✅ nt_core_skill_registry |
| 6 | **MCP/ACP Protocol Ecosystem** | Claude, Cursor, Windsurf | ✅ nt_io_protocol_bridge |
| 7 | **Auto-Execution Levels** | Windsurf, Devin, Claude | ✅ nt_shield_action_authorizer |
| 8 | **Persistent Memory** | Windsurf, Devin, Claude | ✅ nt_core_memory_budget |
| 9 | **Background Agents** | Cursor, Copilot, OpenHands | ✅ nt_act_long_running_agent |
| 10 | **Git Worktree Isolation** | Cursor, Copilot, Devin, Augment | ✅ nt_act_workspace_isolator |

---

## Gap Analysis: NeoTrix vs Universal Architecture

### HIGH PRIORITY (Must Have)

| # | Gap | Products | Implementation |
|---|-----|----------|----------------|
| 1 | **Unified Surface** (Chat+Work+Code) | ChatGPT, Claude | Tauri frontend: 3-mode switcher |
| 2 | **Dual Execution** (Cloud+Local) | Claude Cowork | `nt_act_dual_executor` + VM sandbox |
| 3 | **Cross-device Sync** | ChatGPT, Claude | KB-based CRDT sync |
| 4 | **Scheduled Tasks** | ChatGPT, Claude | `nt_act_scheduler` (cron/cloud) |
| 5 | **Record & Replay** | ChatGPT Codex | `nt_core_record_replay` |
| 6 | **Built-in Browser** | ChatGPT, Claude | `nt_io_browser_engine` |
| 7 | **Voice in Agents** | ChatGPT, Claude | `nt_io_voice_agent` |
| 8 | **Computer History** | ChatGPT | `nt_core_computer_history` |

### MEDIUM PRIORITY (Should Have)

| # | Gap | Products | Implementation |
|---|-----|----------|----------------|
| 9 | **Workspace Agents** | ChatGPT | `nt_core_workspace_agent` |
| 10 | **MCP Desktop Extensions** | Claude | `.ntb` package format |
| 11 | **Folder Instructions** | Claude | Per-folder context injection |
| 12 | **Approval Modes** (Auto/Manual/Skip) | Claude | UI controls for action_authorizer |
| 13 | **Projects** (Persistent workspaces) | ChatGPT, Claude | `nt_core_projects` |
| 14 | **Living Spec** | Augment Intent | `nt_core_living_spec` |
| 15 | **Context Engine** (Real-time indexing) | Augment | ✅ nt_core_context_engine |
| 16 | **Multi-Agent Coordinator** | Cursor, Augment | ✅ nt_core_multi_agent |

---

## Implementation Roadmap

### Sprint 1: Unified Surface (Week 1-2)
1. Create `nt_tauri_unified_surface` with 3-mode switcher (Chat/Work/Code)
2. Build mode-specific UI panels
3. Wire to existing backends (chat → consciousness, work → agent engine, code → code actions)

### Sprint 2: Dual Execution + Scheduler (Week 2-3)
1. Implement `nt_act_dual_executor` (cloud/local routing)
2. Implement `nt_act_scheduler` (cron, one-time, event-driven)
3. Build scheduler UI with task management

### Sprint 3: Cross-device Sync + Projects (Week 3-4)
1. Implement CRDT-based session sync via KB
2. Implement `nt_core_projects` (persistent workspaces)
3. Build project management UI

### Sprint 4: Browser + Voice + History (Week 4-5)
1. Implement `nt_io_browser_engine` (built-in browser)
2. Implement `nt_io_voice_agent` (voice in agents)
3. Implement `nt_core_computer_history` (activity tracking)

### Sprint 5: Record & Replay + Extensions (Week 5-6)
1. Implement `nt_core_record_replay` (workflow → skill)
2. Implement MCP desktop extension format
3. Build extension management UI

---

## Key Design Decisions

### 1. Unified Surface with Mode Switching
The desktop app should have a single entry point with three modes:
- **Chat Mode**: Fast conversational assistance
- **Work Mode**: Long-running tasks, document creation, research
- **Code Mode**: Software development, Git integration, testing

### 2. Dual Execution Architecture
Like Claude Cowork, support both cloud and local execution:
- **Cloud Mode**: Agent loop on remote server, sandboxed execution
- **Local Mode**: Agent loop on device, VM-isolated code execution
- **Hybrid Mode**: Route tasks based on complexity/privacy requirements

### 3. CRDT-based Cross-device Sync
Use CRDTs for offline-first session sync:
- Sessions can be created/modified offline
- Automatic merge when devices reconnect
- Strong eventual consistency guaranteed

### 4. Scheduled Tasks with Cloud Execution
Tasks should run in the cloud, not requiring the device to be awake:
- Cron-based schedules
- One-time scheduled tasks
- Event-driven triggers (webhook, file change)

### 5. Record & Replay for Skill Creation
Like ChatGPT Codex, allow users to demonstrate workflows:
- Record user actions (file edits, terminal commands, browser clicks)
- Convert to reusable skill (SKILL.md + supporting files)
- Share skills across team

---

## Competitive Positioning (Updated)

| Feature | NeoTrix | ChatGPT | Claude | Cursor | Devin | Copilot |
|---------|---------|---------|--------|--------|-------|---------|
| Unified Surface | ✅ (planned) | ✅ | ✅ | ❌ | ✅ | ❌ |
| Dual Execution | ✅ (planned) | ❌ | ✅ | ❌ | ❌ | ❌ |
| Cross-device Sync | ✅ (planned) | ✅ | ✅ | ❌ | ✅ | ❌ |
| Scheduled Tasks | ✅ (planned) | ✅ | ✅ | ✅ | ❌ | ❌ |
| Skills System | ✅ (exists) | ✅ | ✅ | ✅ | ✅ | ✅ |
| MCP/ACP | ✅ (exists) | ✅ | ✅ | ✅ | ✅ | ✅ |
| Auto-Execution | ✅ (exists) | ✅ | ✅ | ✅ | ✅ | ✅ |
| Persistent Memory | ✅ (exists) | ✅ | ✅ | ✅ | ✅ | ❌ |
| Background Agents | ✅ (exists) | ✅ | ✅ | ✅ | ✅ | ✅ |
| Git Worktree | ✅ (exists) | ✅ | ❌ | ✅ | ✅ | ✅ |
| Context Engine | ✅ (exists) | ⚠️ | ⚠️ | ✅ | ⚠️ | ⚠️ |
| Multi-Agent | ✅ (exists) | ✅ | ✅ | ✅ | ✅ | ⚠️ |
| Record & Replay | ✅ (planned) | ✅ | ❌ | ❌ | ❌ | ❌ |
| Built-in Browser | ✅ (planned) | ✅ | ✅ | ✅ | ✅ | ❌ |
| Voice in Agents | ✅ (planned) | ✅ | ❌ | ❌ | ❌ | ❌ |
| Computer History | ✅ (planned) | ✅ | ❌ | ❌ | ❌ | ❌ |

**NeoTrix Unique Advantages:**
1. **Rust + Tauri**: Native performance, small binary, cross-platform
2. **E8 Consciousness**: Self-evolving reasoning architecture
3. **VSA HyperCube**: Knowledge representation beyond simple RAG
4. **GWT Attention Routing**: Biological attention mechanism
5. **KB-based Memory**: Persistent knowledge base across sessions
6. **Zero unsafe**: Memory safety guaranteed by Rust
7. **Local-first**: CRDT sync, offline capability, privacy by default
8. **Dual execution**: Cloud + Local VM isolation

---

## Round 2: Technical Implementation Details (Sep 16, 2026)

### Agent Loop — Reverse-Engineered from Cursor + Claude

**Client-Side Continuation Loop** (both Cursor and Claude use this):
```
while True:
    response = callModel(system + messages + tools)
    messages.append(response)
    if response.stop_reason != "tool_use":
        return response  # final answer
    results = executeTools(response.toolCalls)  # parallel
    messages.append({role: "user", content: results})
```

**Key Details:**
- Cursor: gRPC Protocol Buffers for tool dispatch, 42 tools, PreToolUse/PostToolUse hooks
- Claude: Parallel tool execution, model fallback (strips thinking sigs), death spiral guard (3 compact failures → circuit breaker)
- **NeoTrix mapping**: `nt_core_agent_loop` with direct process-internal tool dispatch (zero serialization)

### CLAUDE.md → AGENTS.md System

**Critical insight**: CLAUDE.md is NOT injected into system prompt. It's injected as `<system-reminder>` XML tag attached to conversation messages. This preserves the shared prompt cache.

**Discovery order**:
1. Organization: `/Library/Application Support/ClaudeCode/CLAUDE.md`
2. User: `~/.claude/CLAUDE.md`
3. Project: `./CLAUDE.md` or `./.claude/CLAUDE.md`
4. Subdirectories: on-demand when accessing files there
5. Project-scoped user: `~/.claude/projects/<path>/CLAUDE.md`

**NeoTrix mapping**: `nt_core_agents_md` (IMPLEMENTED)

### Context Compaction — 5 Mechanisms

| Mechanism | LLM? | Trigger | Scope |
|-----------|------|---------|-------|
| Auto full compact | Yes | tokens ≥ threshold (89.4%) | Full history |
| Manual /compact | Yes | User command | Full or partial |
| Sub-agent compact | Yes | Before turn | Sub-agent history |
| Microcompact | No | Warning threshold | Old tool results only |
| Session memory compact | No | Auto-trigger | Uses stored memory |

**Full compact prompt**: 9-section structure (Primary Request, Key Concepts, Files, Errors, Problem Solving, All User Messages, Pending Tasks, Current Work, Next Step). Output wrapped in `<summary>` tags.

**NeoTrix mapping**: `nt_core_context_engine` (needs compaction logic)

### Confidence Cascading Router

```
Tier 1: Local LLM (llama.cpp) → confidence > 0.8 → Response
                                      ↓ < 0.8
Tier 2: Edge/Proxy (rtk)     → confidence > 0.7 → Response
                                      ↓ < 0.7
Tier 3: Frontier (GPT/Claude) → Response

Privacy: Public→any, Internal→T1/T2, Sensitive→T1 only
Cache: Similar requests → cached routing path
```

**NeoTrix mapping**: `nt_core_model_router` (needs confidence logic)

### Code Indexing — CodeGraph Architecture

**Graph construction pipeline**:
1. File discovery (git-aware, .gitignore)
2. Tree-sitter parsing (38 languages)
3. Cross-file resolution at index time (not query time)
4. Embedding (BGE 384d or model2vec 256d)
5. Persistence: RocksDB at `~/.codegraph/graph.db`

**Key metrics**: ~60 files/sec indexing, ~30ms incremental noop, sub-100ms query, ~6s full re-index (50K nodes)

**NeoTrix mapping**: `nt_core_hybrid_search` (IMPLEMENTED) + `nt_core_context_engine` (needs tree-sitter)

### Browser — Playwright MCP Pattern

**Accessibility tree snapshots** instead of screenshots: ~200-400 tokens per snapshot vs thousands for images.

**Key tools**: `browser_navigate`, `browser_snapshot` (accessibility tree), `browser_click` (by ref), `browser_type`, `browser_take_screenshot`

**NeoTrix mapping**: `nt_io_browser_engine` (needs Playwright integration)

### Sandbox — Firecracker Pattern

**28ms snapshot restore**: Cold boot ~1.1s → snapshot → every subsequent spawn ~28ms

**eBPF egress control**: Rules enforced in host kernel, outside VM. Code inside sandbox cannot bypass.

**NeoTrix mapping**: `nt_act_sandbox_manager` (needs Firecracker/Docker integration)

### Memory — SYNAPSE Spreading Activation

**Dual-trigger retrieval**: BM25 (lexical) + dense embedding (semantic)
**Activation propagation**: Through temporal + causal edges
**Lateral inhibition**: Suppresses irrelevant distractors
**Fan effect**: Activation dilution by out-degree (prevents hub explosion)

**NeoTrix mapping**: `nt_core_memory_consolidator` (needs spreading activation)

### Permission System — Claude Auto Mode

**Two-layer defense**:
1. Input: Server-side prompt-injection probe on tool outputs
2. Output: Transcript classifier (Sonnet 4.6) — fast filter (8.5% FPR) → chain-of-thought (0.4% FPR)

**Three-tier resolution**: Built-in safe-tool allowlist → In-project file ops → Everything else (classifier)

**NeoTrix mapping**: `nt_shield_action_authorizer` (needs classifier integration)

---

*This document is the single source of truth for NeoTrix desktop app architecture.*
*Updated after each research iteration cycle.*
*Last updated: 2026-09-16 (Round 2)*
