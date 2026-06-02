# NeoTrix Fusion Design: suna + context-mode + AutoScientists

> 2026-05-29 | Three-way absorption analysis → gap matrix → integration design → evolution roadmap

---

## 1. Panoramic Gap Matrix

### 1.1 NeoTrix vs suna (kortix-ai/suna v1.2.0)

| Capability | suna | NeoTrix | Gap | Priority |
|---|---|---|---|---|
| **Skills tier system** | 6 tiers: Foundation→MCP Function→Quality→Resource→Klipper→Multi-turn | No skill categorization. Flat `AgentPersona` with single capability vector | **Critical** — cannot prioritize skill context budget | P0 |
| **Agent tunnel (reverse)** | WebSocket mux with auth heartbeat, exec/pull/expand/tcp proxy, phone-voice | `agent_protocol/` has UDP discovery + TCP, no tunnel, no phone | **Significant** — no remote agent interaction | P0 |
| **Multi-platform channel** | Telegram/Discord/Slack/WhatsApp parallel channel adapters | `push_channel.rs` + `avatar_channel.rs` — Telegram only | **Significant** — locked to Telegram | P1 |
| **OpenCode/Dora wrapper** | suna wraps Claude Code CLI — 5x cost reduction, shared sessions | No external CLI wrapping. NeoTrix is standalone | **Medium** — missed cost optimization | P1 |
| **Encrypted secret store** | AES-256-GCM key derivation, key rotation, Supabase Vault | `security/keyvault.rs` + `vault.rs` (feature-gated, `identity_infrastructure`) | **Medium** — exists but gated and unintegrated | P1 |
| **Trigger system** | Cron + webhook triggers, configurable action/condition | `scheduler.rs` — cron-only, no webhook, no condition | **Medium** — no action abstraction | P2 |
| **Service manager** | Health probing, auto-heal, restart policy | `background_loop.rs` — no health probing | **Low** — basic orchestration exists | P2 |
| **Session ownership** | Per-user session filtering, tagging, query | No concept of users | **Medium** — important for multi-user REPL | P2 |
| **Phone/Voice integration** | Twilio SIP trunk, webhook routing, VAPI.ai | None | **Low** — niche | P3 |
| **PostgreSQL Drizzle persistence** | Type-safe SQL, migrations, full query | `surreal_rs` for ReasoningBank only | **Medium** — No strong-typed agent data store | P2 |

### 1.2 NeoTrix vs context-mode

| Capability | context-mode | NeoTrix | Gap | Priority |
|---|---|---|---|---|
| **Tool output sandboxing** | `ctx_execute()` runs in sandboxed process, captures stdout/stderr, returns structured result | All tool output goes raw into context. No sandbox | **Critical** — context pollution, token waste | P0 |
| **Session continuity** | SQLite FTS5 stores all tool calls. On reconnect, full context restored with resume snapshots | No resume — CLI/headless loses context on restart | **Critical** — fundamental UX gap | P0 |
| **Context truncation** | Multi-level hierarchical summarization: `truncate(maxTokens, keepInitial)` | No truncation. Context grows unbounded until OOM | **Critical** — linear context degradation | P0 |
| **Lifecycle hooks** | PreToolUse/PostToolUse/PreCompact/SessionStart — run before/after every tool call | `agent/hooks.rs` — HookRegistry exists but no standard hook points, not wired to tool execution | **Significant** — hooks exist but don't fire on tool context | P1 |
| **Multi-platform adapter** | 15 platform adapters (detect.ts + client-map.ts) auto-detecting CLI/IDE/api | Only its own CLI. No platform detection | **Medium** — scope for later if multi-platform needed | P2 |
| **File-level result caching** | Cache results by file hash, skip re-execution on unchanged files | No caching — re-reads files every time | **Medium** — perf optimization | P2 |
| **Codespace/remote exec** | Detects Codespace, SSH, remote contexts — adjusts tool behavior | No remote context awareness | **Low** — nice to have | P3 |
| **Smart exit classification** | success/failure/cancelled/timeout/applied — drives different recovery | No exit classification — all exits = task done | **Medium** — lost recovery info | P2 |

### 1.3 NeoTrix vs AutoScientists (Harvard)

| Capability | AutoScientists | NeoTrix | Gap | Priority |
|---|---|---|---|---|
| **Self-organized team formation** | Monitor→Analysts→GPU teams form around hypotheses. Teams self-recruit based on TASK.md | AgentTeam uses Coordinator.route_task() — top-down assignment, no self-organization | **Significant** — no emergent teaming | P1 |
| **Discussion→Proposal→Critique→Execute** | Propose hypothesis → peer critique → form team → execute → validate | No proposal forum. Tasks go straight to execution with no peer review | **Significant** — no quality gating before execution | P1 |
| **Shared state model** | champion/log/forum/dead-end registry — persistent shared state for entire experiment | ReasoningBank (individual), GoalLoop (goals only). No shared experimentation state | **Significant** — agents can't share experiment learnings | P1 |
| **Heartbeat protocol** | 8-step loop with health check, stall detection, periodic review | No orchestration loop. Tasks dispatched once, no ongoing health monitoring | **Significant** — no long-running experiment supervision | P1 |
| **Role separation** | Monitor NEVER runs code. Analysts NEVER claim champion. GPU Agents NEVER modify train.py | AgentTeam roles are flexible — same agent can propose and execute and validate | **Medium** — conflated roles cause bias | P1 |
| **Noise-aware champion validation** | Run champion candidate 3+ seeds, only accept if outperforms on avg | Single-shot capability vector update. No multi-validation | **Medium** — false positives in self-improvement | P2 |
| **Dead-end registry** | Track failed hypotheses → prevent re-exploration | No dead-end tracking. `ReasoningBank` grows with no negative weighting | **Medium** — wasted re-exploration | P2 |
| **Loop self-evolution** | Update role templates from experience (efficiency.txt, error_patterns.md) | No template evolution. AGENTS.md updated manually | **Low** — structural but important | P2 |
| **KEEP-post-inductive reasoning** | After champion found: "what made this work → what else shares that property" | No generalization after success | **Medium** — missed cross-domain transfer | P2 |

---

## 2. Priority Matrix

```
                    Impact
                    ┃
         Critical   ┃  P0: Tool sandbox (cm)
                    ┃  P0: Session continuity (cm)
                    ┃  P0: Context truncation (cm)
                    ┃  P0: Skills tier system (suna)
                    ┃  P0: Agent reverse tunnel (suna)
                    ┃
         High       ┃  P1: Lifecycle hooks (cm)
                    ┃  P1: Self-org team (AutoS)
                    ┃  P1: Proposal→Critique→Exec (AutoS)
                    ┃  P1: Shared state model (AutoS)
                    ┃  P1: Heartbeat protocol (AutoS)
                    ┃  P1: Multi-channel (suna)
                    ┃  P1: Role separation (AutoS)
                    ┃  P1: OpenCode wrapper (suna)
                    ┃
         Medium     ┃  P2: File caching (cm)
                    ┃  P2: Dead-end registry (AutoS)
                    ┃  P2: Secret store (suna)
                    ┃  P2: Trigger system (suna)
                    ┃  P2: Exit classification (cm)
                    ┃  P2: Noise-aware champion (AutoS)
                    ┃  P2: KEEP post-inductive (AutoS)
                    ┃  P2: Session ownership (suna)
                    ┃  P2: PostgreSQL persistence (suna)
                    ┃
         Low        ┃  P3: Phone/voice (suna)
                    ┃  P3: Codespace (cm)
                    ┃  P3: Template evolution (AutoS)
                    ┗━━━━━━━━━━━━━━━━━━━━━━━ Urgency →
                     Low          Med        High
```

---

## 3. Integration Point Design

### Phase 1: Context Window & Tool Sandboxing (context-mode, 3-4 sessions)

#### P1.1: Tool Sandbox (`core/context/sandbox.rs`)

**NeoTrix insertion point**: New module `core/context/` parallel to `core/thinking_model/`

```
core/context/
├── mod.rs
├── sandbox.rs        # Tool execution sandbox
├── session.rs        # Session continuity store
├── truncator.rs      # Context truncation engine
└── hooks.rs          # Lifecycle hook registry (wraps existing HookRegistry)
```

**Sandbox architecture**:
```
Input: SandboxPayload { tool_id, args, metadata }
    → SandboxConfig { timeout, max_output_bytes, allowed_commands }
    → ProcessSandbox::execute(payload)
        → fork() or thread + isolated tokio runtime
        → capture stdout/stderr
        → enforce timeout (default 60s, configurable)
        → produce SandboxResult { exit_code, stdout, stderr, truncated, duration_ms }
    → SessionStore.record(call)
```

**Integration points**:
- `agent/hooks.rs:HookRegistry` → wire PreToolUse: sandbox.enforce_policy() before tool exec
- `MCP tools (mcp_tools.rs)` → wrap tool handler in sandbox::run()
- ReasoningBank → store sandboxed results tagged with `result_id` for traceability

#### P1.2: Session Continuity (`core/context/session.rs`)

**SQLite-based** (use existing `rusqlite` if available, or add dep):
```
SessionStore {
    create_session(config) -> SessionId
    record_tool_call(session_id, tool_id, input, output) -> CallId
    get_session_history(session_id, max_tokens) -> TruncatedHistory
    get_resume_snapshot(session_id) -> ResumeSnapshot
    list_sessions(user?) -> Vec<SessionSummary>
}
```

**Integration points**:
- CLI `--resume <session_id>` flag → `CliConfig.session_id`
- `headless.rs:run()` → check for resume on startup → restore reasoning state
- `reasoning_brain/background_loop.rs` → periodic session checkpoint

#### P1.3: Context Truncation (`core/context/truncator.rs`)

Three-level engine (inspired by context-mode):
```
Truncator {
    level_0: discard_low_value_tool_calls(history)  # remove failed/non-essential calls
    level_1: hierarchical_summary(history, target_tokens)  # summarization
    level_2: prune_oldest_with_importance(history, keep_ratio)  # eviction
}
```

**Integration points**:
- `reasoning_brain/context_window.rs:ContextWindow.manage()` → call truncator before token limit
- Trigger points: PreCompact lifecycle hook (Phase 1 built-in), memory pressure detection

#### P1.4: Lifecycle Hooks (`core/context/hooks.rs`)

Extend existing `HookRegistry` from `agent/hooks.rs` to support:
```
enum LifecycleEvent {
    PreToolUse(SandboxPayload),
    PostToolUse(SandboxResult),
    PreCompact(ContextState),
    SessionStart(SessionConfig),
    SessionEnd(SessionSummary),
    PreReason(QueryContext),
}

enum HookAction {
    Continue,
    Modify(SandboxPayload),
    Abort(String),          // reject tool call with message
    Defer(Box<dyn Future>), // async continuation
}
```

**Integration points**:
- Wire to existing `HookRegistry` → HookRegistry.lifecycle_hooks: Vec<Box<dyn Fn(LifecycleEvent) -> HookAction>>
- `mcp_tools.rs` tool dispatch → fire PreToolUse before exec → PostToolUse after

### Phase 2: Skills Tier System & Agent Tunnel (suna, 4-5 sessions)

#### P2.1: Skills Tier System (`reasoning_brain/skills/`)

```
reasoning_brain/skills/
├── mod.rs
├── tier.rs            # Tier enum + SkillDefinition
├── registry.rs        # SkillRegistry — load/save/manage
├── matcher.rs         # SkillMatcher — match task to tier
└── provider.rs        # SkillProvider trait for external skill repos
```

**Tier system** (adapted from suna):
```
enum SkillTier {
    Foundation,  // git, fs, search — always available
    Core,        // process, system, web — budget 3
    Quality,     // lint, test, review — budget 2
    Resource,    // network, api, db — budget 2
    Advanced,    // supervised chains — budget 1
    Orchestration, // complex multi-step — budget 1
}

struct SkillDefinition {
    id: SkillId,
    name: String,
    tier: SkillTier,
    context_weight: f32,        // token budget multiplier
    requires: Vec<SkillId>,     // prerequisites
    capability_vector: CapabilityVector,
}
```

**Context budget allocation**:
- Foundation: 40% of context (always loaded)
- Core: 20%
- Quality: 15%
- Resource: 15%
- Advanced: 7%
- Orchestration: 3%

**Integration points**:
- `reasoning_brain/core.rs` → SkillDefinition links to CapabilityVector
- `SelfIteratingBrain.run_seal_loop()` → before task execution → SkillMatcher.select_skills(task)
- `background_loop.rs:skill_ticker` → periodic skill re-evaluation
- KnowledgeSource enum → new `HybridSkills(suna)` variant

#### P2.2: Agent Reverse Tunnel (`agent/tunnel.rs`)

```
agent/tunnel/
├── mod.rs
├── tunnel.rs         # WebSocket mux tunnel (extends agent_protocol)
├── auth.rs           # Session auth + heartbeat
├── exec.rs           # Remote execution (exec/pull/expand commands)
├── proxy.rs          # TCP proxy through tunnel
└── phone.rs          # Phone/voice channel (if/when needed)
```

**Tunnel protocol** (adapted from suna):
```
Client → Server:
  { type: "auth", session_id, token, capabilities }
  { type: "exec", cmd: "bash|tool|skill", args }
  { type: "pull", path: "/path/to/file" }
  { type: "ping", seq: 123 }

Server → Client:
  { type: "auth_ok", session_id }
  { type: "result", seq, data, error? }
  { type: "pong", seq }
  { type: "push", channel: "notification", data }
```

**Integration points**:
- Existing `agent_protocol/discovery.rs` → add tunnel capability advertisement
- Existing `agent_protocol/capabilities.rs` → extend with tunnel-related caps
- `agent_protocol/server.rs` → add WebSocket upgrade handler for tunnel
- `agent/coordinator.rs` → route remote tasks through tunnel

### Phase 3: Self-Organization Protocol (AutoScientists, 2-3 sessions)

#### P3.1: Self-Org Team Formation (`agent/self_org/`)

```
agent/self_org/
├── mod.rs
├── forum.rs          # Proposal forum — hypothesis → peer review
├── heartbeat.rs      # Orchestration loop (8-step protocol)
├── roles.rs          # Role definitions + separation constraints
├── state.rs          # Shared state (champion/log/forum/dead-end)
└── evolve.rs         # Template evolution from experience
```

**Heartbeat loop** (8 steps):
```
1. Determine state (init/running/review/completed/stalled)
2. Read TASK.md (shared goal document)
3. Dimension discussion (hypothesis space exploration)
4. Form teams + seed queue (self-recruit around hypotheses)
5. Execute loop:
   a. Pre-cycle setup
   b. Analysts propose experiments
   c. GPU Agents execute
   d. Wait for completion / collect results
   e. Champion evaluation (multi-seed validation)
   f. Health check (stall detection, dead-end detection)
   g. Periodic review (re-assess progress)
   h. Loop back or finalize
6. Final report generation
7. Template evolution (update role prompts from experience)
8. KEEP analysis (generalization of successful patterns)
```

**Integration points**:
- `agent/team.rs:AgentTeam` → add `self_org_mode: Option<SelfOrgConfig>` toggle
- `agent/team.rs:Coordinator` → add `propose_and_critique()` before `route_task()`
- `reasoning_brain/reasoning_bank.rs` → add `dead_end_registry: HashSet<TaskHash>`
- `reasoning_brain/goal_loop.rs:GoalLoop` → wire heartbeat as goal evaluation cycle

---

## 4. Architecture Decision Records

### ADR-1: Tool Sandbox — process isolation vs WASM

**Decision**: Thread-based sandbox with tokio runtime isolation (not WASM)
**Rationale**: WASM sandbox (suna approach) provides stronger isolation but requires WASM compilation for each tool. Thread isolation is simpler to implement within existing Rust/tokio architecture and sufficient for CLI context. WASM layer can be added later as optional enhancer.
**Cost**: ~200 lines new code vs ~500 for WASM integration
**Tradeoff**: Less security isolation — acceptable for local CLI use

### ADR-2: Session Store — SQLite vs SurrealDB

**Decision**: SQLite via rusqlite (new dependency)
**Rationale**: SurrealDB (already a dep) is overengineered for a linear session log. SQLite FTS5 (context-mode approach) provides full-text search of tool calls, low latency, zero config, and is universally available. Session store is append-heavy with occasional reads — perfect SQLite workload.
**Cost**: Add `rusqlite` with `bundled` feature (+ ~500KB binary)
**Tradeoff**: Another persistence layer alongside SurrealDB — acceptable because data domains are disjoint (surreal = knowledge, sqlite = session log)

### ADR-3: Skills Tier — suna-aligned vs custom

**Decision**: Direct adaptation of suna's 6-tier system
**Rationale**: suna's tier system is battle-tested across 60+ skills in production. The 6 tiers correspond directly to context budget allocation problems NeoTrix faces. Existing `CapabilityVector` maps naturally to tier scoring.
**Cost**: ~400 lines new code + migration of existing tools to tier system
**Tradeoff**: Overhead of classifying each tool — acceptable as one-time cost

### ADR-4: Self-Organization — integrated vs overlay

**Decision**: Integrated into existing AgentTeam (not overlay)
**Rationale**: AgentTeam already has Coordinator + WorkerNode. Rather than building a parallel system, extend Coordinator with `propose_and_critique()` mode. Heartbeat protocol maps to GoalLoop's existing pursuit cycle. This preserves existing routing while adding self-org capability.
**Cost**: ~300 lines modifications to existing files
**Tradeoff**: Tight coupling — but simpler than maintaining two agent orchestration systems

### ADR-5: Reverse Tunnel — extend vs new

**Decision**: Extend existing `agent_protocol/` (not new module)
**Rationale**: `agent_protocol/` already has UDP discovery + TCP server. Adding WebSocket tunnel mux extends the protocol rather than rewriting it. The capability routing already supports adding `tunnel` capability.
**Cost**: ~250 lines new code
**Tradeoff**: Larger `agent_protocol/` module — acceptable, cohesion is high

---

## 5. Phased Evolution Roadmap

```
Phase 0: Foundation (current session)
├── Fix CLI commands ✓
├── Complete gap analysis + design doc ← YOU ARE HERE
└── TODO.md + Session log ✓

Phase 1: Context Window Revolution (3-4 sessions)
├── Session 1a: Tool Sandbox
│   ├── core/context/sandbox.rs — sandboxed tool execution
│   ├── Integration: wire into MCP tools dispatch
│   └── Test: sandbox enforces timeout, captures output
├── Session 1b: Session Continuity
│   ├── core/context/session.rs — SQLite FTS5 session store
│   ├── Integration: CLI --resume, headless auto-resume
│   └── Test: resume session with full context
├── Session 1c: Context Truncation
│   ├── core/context/truncator.rs — 3-level truncation
│   ├── Integration: ContextWindow, lifecycle hooks
│   └── Test: truncation preserves essential context
└── Session 1d: Lifecycle Hooks
    ├── core/context/hooks.rs — standardized lifecycle events
    ├── Integration: wire to HookRegistry, MCP dispatch
    └── Test: PreToolUse can abort/modify calls

Phase 2: Skills & Remote Access (4-5 sessions)
├── Session 2a: Skills Tier System
│   ├── reasoning_brain/skills/ — SkillRegistry + Tier system
│   ├── Integration: CapabilityVector mapping, SelfIteratingBrain
│   └── Test: tier-based context budget allocation
├── Session 2b: Tool Migration to Tiers
│   ├── Reclassify all ~40 existing tools into 6 tiers
│   ├── Integration: SkillMatcher in task dispatch
│   └── Test: skill selection by task type
├── Session 2c: Reverse Tunnel
│   ├── agent/tunnel/ — WebSocket mux tunnel
│   ├── Integration: extend agent_protocol, Coordinator
│   └── Test: bidirectional tunnel with auth
├── Session 2d: Multi-Platform Channel (if priority)
│   ├── channel/ — Telegram/Discord/Slack adapters
│   └── Integration: PushChannel trait
└── Session 2e: Secret Store + Triggers (if needed)
    ├── Un-gate keyvault/vault, integrate with session ownership
    └── Extend scheduler with webhook triggers

Phase 3: Self-Organizing Teams (2-3 sessions)
├── Session 3a: Heartbeat + Self-Org Protocol
│   ├── agent/self_org/ — forum, heartbeat, roles, state
│   ├── Integration: extend AgentTeam, Coordinator
│   └── Test: self-forming teams around hypotheses
├── Session 3b: Shared State + Dead-End Registry
│   ├── Shared champion/log/forum/dead-end state
│   └── Integration: ReasoningBank, GoalLoop
└── Session 3c: Template Evolution + KEEP
    ├── Role template evolution from experience
    ├── KEEP post-inductive generalization
    └── Test: generalization across tasks

Phase 4: Polish & Integration (2 sessions)
├── Noise-aware champion validation (multi-seed)
├── File-level result caching
├── Session ownership + user tagging
├── PostgreSQL Drizzle option for strong-typed data
└── Full integration test suite

Phase 5: Advanced (future)
├── Phone/Voice integration
├── Codespace/remote context detection
├── OpenCode/Dora wrapper for cost optimization
└── Template self-evolution
```

---

## 6. KnowledgeSource Registration

After each phase, register knowledge sources:

```
Phase 1 → KnowledgeSource::ContextSandboxing
  capability_vector: { tool_safety: 0.95, context_efficiency: 0.92, session_continuity: 0.90 }

Phase 2 → KnowledgeSource::SkillsTierSystem (from suna)
  capability_vector: { skill_prioritization: 0.90, context_budget: 0.88, tool_routing: 0.85 }
Phase 2 → KnowledgeSource::AgentTunnel (from suna)
  capability_vector: { remote_connectivity: 0.92, cross_machine: 0.88 }

Phase 3 → KnowledgeSource::SelfOrganizingTeams (from AutoScientists)
  capability_vector: { self_organization: 0.90, peer_review: 0.88, heartbeat: 0.85 }
```

---

## 7. Estimated Effort

| Phase | Sessions | New Lines | Modified Lines | New Files |
|---|---|---|---|---|
| P1: Context Window | 3-4 | ~800 | ~200 | 5 |
| P2: Skills+Tunnel | 4-5 | ~1000 | ~300 | 8 |
| P3: Self-Org | 2-3 | ~600 | ~200 | 5 |
| P4: Polish | 2 | ~300 | ~150 | 2 |
| P5: Advanced | future | ~400 | ~100 | 3 |
| **Total** | **11-14** | **~3100** | **~950** | **23** |

---

*Next step: User approval → Phase 1 implementation begins with Session 1a: Tool Sandbox*
