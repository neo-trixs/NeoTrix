# SoL-Pi × Grok Build × NeoTrix Fusion Plan

**Date:** 2026-09-11  
**Sources:** NVlabs/SoL-Pi (open-sourced 2026-09-10), xai-org/grok-build (open-sourced 2026-07-15)  
**Targets:** `harness_optimizer.rs`, `hooks.rs`, `acp.rs`

---

## 1. SoL-Pi Deep Dive

### 1.1 Action Fusion (核心创新)

**Problem:** In basic Pi, every edit→command sequence requires two model round-trips. The edit result returns, then the model decides to run `cargo test` — that decision is deterministic and wasteful.

**Solution:** Action Fusion encapsulates edit + subsequent commands into a single local execution sequence. The harness completes the modification and runs commands at the底层 layer, returning merged results in one shot.

**Key insight:** The "deterministic follow-up" pattern — when tool A's output always triggers tool B, fuse them into A→B atomic unit.

**NeoTrix mapping:** `harness_optimizer.rs:85` `fuse_actions()` only merges *consecutive identical* tools. SoL-Pi fuses *heterogeneous but causally linked* tools (edit→build, edit→test).

### 1.2 ObservationPack Compression

**Problem:** Build/test logs are 10K+ words, but only a few error lines matter for the next decision. Frontier models waste tokens reading full logs.

**Solution:** Pack observations into compressed bundles with key evidence extraction. Only the diagnostic receipt (not raw logs) reaches the frontier model.

**NeoTrix mapping:** `harness_optimizer.rs:149` `pack_observation()` uses naive "first 200 chars + error lines" extraction. SoL-Pi's approach is more structured: key evidence = error lines + stack traces + relevant file paths + line numbers.

### 1.3 Online Context Compact

**Problem:** Context windows fill up during long sessions. Naive compaction loses evidence.

**Solution:** Proactive background observation/reflection (like `pi-observational-memory`). Observations and reflections prepared *before* compaction, so compaction becomes a fast rendering step, not slow summarization.

**NeoTrix mapping:** `harness_optimizer.rs:177` `compact_context()` is a stub — calculates a savings ratio but doesn't actually select what to preserve.

### 1.4 Evidence-Preserving Reducer (最关键的缺失)

**Problem:** Small models hallucinate when summarizing logs. Direct summarization introduces errors.

**Solution:** Three-stage pipeline:
1. Small model reads full logs → produces compact diagnostic receipt
2. System verifies receipt against archived logs item-by-item
3. Only verified original text fragments forwarded to frontier model

**Evidence Store:** Persistent `evidence-store` with session-scoped TTL, file-budget limits, session leases, SHA-256 hashing, and lock-free concurrent access.

**NeoTrix mapping:** Completely absent from `harness_optimizer.rs`. This is the highest-value fusion target.

---

## 2. Grok Build Deep Dive

### 2.1 Hook System (79-crate architecture)

**Event types:** `SessionStart`, `TurnStarted`, `TurnCompleted`, `ToolCalled`, `PreToolUse`, `PostToolUse`, `SubagentStart`, `SubagentStop`, `PreCompact`, `PostCompact`, `Stop`, `StopFailure`, `StopCancelled`, `PermissionDenied`, `Notification`, `UserPromptSubmit`

**Matcher system:** Regex-based selectors that test event-specific fields (tool name on `PreToolUse`, notification type on `Notification`, subagent type on `SubagentStart`). Empty matcher = match everything.

**Trust model:** 
- `~/.grok/hooks/` = always trusted (global)
- `.grok/hooks/` = requires explicit trust via `/hooks-trust` or `--trust`
- Stored in `~/.grok/trusted_folders.toml`

**Decision protocol:**
- `PreToolUse`: `allow`/`deny`/`ask`/`defer` + optional `updatedInput` rewrite
- `Stop`/`SubagentStop`: `block` keeps agent working
- All other events: passive (output recorded, no control flow change)

**Fail-open:** Timeout/crash/malformed output → recorded in scrollback but never blocks action. Only explicit `deny` blocks.

**NeoTrix mapping:** `hooks.rs` has only 5 event types, no matcher system, no trust model, no HTTP hooks, no `updatedInput` rewriting, no config-file integration.

### 2.2 ACP Protocol (Agent Client Protocol)

**Transport:** JSON-RPC over stdin/stdout (for IDE integration: Zed, Neovim, Emacs)

**Capabilities:** Full tool manifest publication, context assembly API, session lifecycle management, progress reporting via notifications.

**Method types:** 
- `ping` / `capabilities` (discovery)
- Tool invocation with typed schemas
- Context query/modify
- Session start/end/compact

**NeoTrix mapping:** `acp.rs` has only 95 lines with `ping` and `capabilities`. Missing: actual stdin/stdout transport, tool manifest, context assembly API, session management, progress notifications.

### 2.3 Worktree Isolation

**Architecture:** `xai-fast-worktree` crate with:
- `WorktreeBuilder` + `WorktreeSync` for multi-phase synchronization
- HEAD alignment, dirty state replaying, ignored file copying
- CoW (Copy-on-Write) cloning optimizations
- SQLite-backed persistent worktree pool (`WorktreeKind`: Session/Pool/Fork/Subagent)
- Parallel file copying pipeline

**Isolation model:** Each subagent gets its own worktree under `~/.grok/worktrees/<repo>/<subagent-id>`. Changes merge back via `x.ai/git/worktree/*` extension methods.

**NeoTrix mapping:** Not implemented. NeoTrix has `ParallelTaskManager` but no worktree isolation for parallel agents.

### 2.4 Context Assembly Transparency

**Problem:** Users don't know what context the agent sends to the model.

**Grok Build's approach (post-controversy):**
- `/privacy` command to disable retention
- `.grokignore` for file exclusion
- Explicit context injection control (`auto_inject: false`)
- But: shipped with background full-repo upload (removed in safe forks)

**NeoTrix approach should be:** Egress Privacy Guard already handles this at the network level. But local context assembly (what files to include in prompt) needs explicit control.

---

## 3. Gap Analysis: NeoTrix vs Source Patterns

### 3.1 harness_optimizer.rs — 6 Missing Capabilities

| # | Missing | SoL-Pi Pattern | Priority |
|---|---------|---------------|----------|
| 1 | **Evidence-Preserving Reducer** | Small model pre-screen → verify against archive → forward verified fragments | P0 |
| 2 | **Causal Action Fusion** | Fuse edit→build, edit→test (not just identical consecutive tools) | P0 |
| 3 | **Evidence Store** | Persistent session-scoped evidence with TTL, file budgets, SHA-256 | P1 |
| 4 | **Diagnostic Receipt Format** | Structured format: error lines + stack traces + file:line + suggested fix | P1 |
| 5 | **Auto-Research Feedback Loop** | Track compression strategy effectiveness, evolve thresholds | P2 |
| 6 | **Tiered Model Routing** | Route log reading to cheap model, decision-making to frontier model | P2 |

### 3.2 hooks.rs — 8 Missing Capabilities

| # | Missing | Grok Build Pattern | Priority |
|---|---------|-------------------|----------|
| 1 | **Regex Matcher System** | Per-event field matching (tool name, notification type, etc.) | P0 |
| 2 | **Trust Model** | Global vs project hooks, explicit trust grants | P0 |
| 3 | **updatedInput Rewriting** | PreToolUse hooks can modify tool input before execution | P0 |
| 4 | **Config-File Integration** | TOML/JSON hook definitions alongside code-based hooks | P1 |
| 5 | **HTTP Hook Support** | POST event to URL endpoint, not just shell commands | P1 |
| 6 | **Fail-Open/Fail-Closed Policy** | Configurable per-hook failure behavior | P1 |
| 7 | **AdditionalContext Injection** | Hooks can inject context into model responses | P2 |
| 8 | **Hook Discovery** | Auto-discover hooks from `~/.neotrix/hooks/` and `.neotrix/hooks/` | P2 |

### 3.3 acp.rs — 7 Missing Capabilities

| # | Missing | Grok Build Pattern | Priority |
|---|---------|-------------------|----------|
| 1 | **stdio Transport** | Actual JSON-RPC over stdin/stdout with framing | P0 |
| 2 | **Tool Manifest** | Publish tool schemas for IDE integration | P0 |
| 3 | **Session Lifecycle** | Start/end/compact session management | P0 |
| 4 | **Progress Notifications** | Streaming progress updates via `$/progress` notifications | P1 |
| 5 | **Context Query API** | IDE can query/modify agent context | P1 |
| 6 | **Error Recovery** | Structured error codes with retry semantics | P1 |
| 7 | **Capability Negotiation** | Dynamic capability discovery and versioning | P2 |

---

## 4. Fusion Implementation Plans

### Phase 1: Evidence-Preserving Reducer (harness_optimizer.rs)

**Goal:** Implement SoL-Pi's core innovation — verify before forwarding.

```rust
// New types needed
pub struct EvidenceArchive {
    pub session_id: String,
    pub raw_logs: Vec<LogEntry>,
    pub archived_at: i64,
    pub ttl_seconds: i64,
}

pub struct DiagnosticReceipt {
    pub error_lines: Vec<String>,
    pub stack_traces: Vec<String>,
    pub file_locations: Vec<(String, u32)>,  // (file, line)
    pub suggested_fix: Option<String>,
    pub confidence: f64,
}

pub struct VerifiedEvidence {
    pub receipt: DiagnosticReceipt,
    pub original_fragments: Vec<String>,  // verified against archive
    pub verification_score: f64,
}
```

**Algorithm:**
1. Archive raw logs with SHA-256 hash on receipt
2. Send logs to small model → get `DiagnosticReceipt`
3. For each item in receipt, find matching original text in archive
4. If match found: include original text in `VerifiedEvidence`
5. If no match: discard (hallucination detected)
6. Forward only `VerifiedEvidence.original_fragments` to frontier model

**Key difference from current `extract_key_evidence`:** Current implementation does naive keyword matching. New implementation verifies against archived originals.

### Phase 2: Causal Action Fusion (harness_optimizer.rs)

**Goal:** Fuse heterogeneously-linked tools, not just identical consecutive ones.

```rust
pub struct CausalLink {
    pub source_tool: String,
    pub target_tool: String,
    pub pattern: FusionPattern,
}

pub enum FusionPattern {
    Always,           // edit → build (always)
    Conditional,      // edit → test (if test exists)
    Probabilistic(f64), // search → edit (70% of the time)
}
```

**Causal link table (hardcoded + learned):**
```
edit_file    → run_command (build/test)    Always
search_code  → edit_file                   Probabilistic(0.7)
run_command  → read_file (on failure)      Conditional
create_file  → run_command (verify)        Always
```

**Fusion algorithm:**
1. Scan tool call sequence for causal patterns
2. Group linked tools into atomic units
3. Execute atomic unit in single harness call
4. Return merged result

### Phase 3: Trust-Aware Hook System (hooks.rs)

**Goal:** Grok Build's trust model + matcher system.

```rust
pub struct TrustStore {
    trusted_folders: HashMap<PathBuf, TrustLevel>,
}

pub enum TrustLevel {
    Global,      // ~/.neotrix/hooks/ — always trusted
    Granted,     // .neotrix/hooks/ — explicitly trusted
    Untrusted,   // not in store
}

pub struct HookMatcher {
    pattern: Regex,
    field: MatchField,
}

pub enum MatchField {
    ToolName,
    NotificationType,
    SubagentType,
    SessionSource,
    EndReason,
    ErrorType,
    Always,  // empty matcher
}
```

**Discovery order:**
1. `~/.neotrix/hooks/*.json` (global, always trusted)
2. `.neotrix/hooks/*.json` (project, requires trust)
3. Config-file hooks in `neotrix.toml`
4. Plugin-contributed hooks

### Phase 4: ACP Transport Layer (acp.rs)

**Goal:** Working JSON-RPC over stdin/stdout.

```rust
pub struct AcpTransport {
    stdin: BufReader<Stdin>,
    stdout: Stdout,
}

impl AcpTransport {
    pub async fn read_request(&mut self) -> Result<AcpRequest> {
        // Read Content-Length header
        // Read JSON body
        // Parse and return
    }
    
    pub async fn write_response(&mut self, resp: &AcpResponse) -> Result<()> {
        // Serialize to JSON
        // Write Content-Length header
        // Write body
    }
    
    pub async fn send_notification(&mut self, method: &str, params: Value) -> Result<()> {
        // Notifications have no id field
    }
}
```

**Standard methods to implement:**
- `initialize` / `initialized` (handshake)
- `tools/list` (publish tool manifest)
- `tools/call` (invoke tool)
- `textDocument/*` (IDE integration)
- `$/progress` (streaming updates)

### Phase 5: Worktree Isolation (parallel agents)

**Goal:** Per-agent git worktree for safe parallel execution.

```rust
pub struct WorktreeManager {
    pool: SqlitePool,
    base_path: PathBuf,
}

pub struct Worktree {
    pub id: Uuid,
    pub path: PathBuf,
    pub kind: WorktreeKind,  // Session/Pool/Fork/Subagent
    pub status: WorktreeStatus,  // Alive/Dead
}

impl WorktreeManager {
    pub async fn create_worktree(&self, kind: WorktreeKind) -> Result<Worktree> {
        // 1. git worktree add <path> HEAD
        // 2. Copy ignored files (target/, node_modules/)
        // 3. Register in SQLite
        // 4. Return worktree handle
    }
    
    pub async fn merge_back(&self, worktree_id: Uuid) -> Result<()> {
        // 1. git merge <worktree-branch>
        // 2. Resolve conflicts
        // 3. Clean up worktree
    }
}
```

---

## 5. Pipeline Integration Map

```
┌─────────────────────────────────────────────────────────┐
│                    SEAL Pipeline                         │
│                                                         │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐         │
│  │ Explore  │───→│ Distill  │───→│Self-Test │──→ ...  │
│  └──────────┘    └──────────┘    └──────────┘         │
│       │               │               │                 │
│       ▼               ▼               ▼                 │
│  ┌──────────────────────────────────────────┐          │
│  │         Harness Optimizer (SoL-Pi)        │          │
│  │  • Evidence-Preserving Reducer            │          │
│  │  • Causal Action Fusion                   │          │
│  │  • ObservationPack (enhanced)             │          │
│  └──────────────────────────────────────────┘          │
│       │                                                 │
│       ▼                                                 │
│  ┌──────────────────────────────────────────┐          │
│  │         Hook Manager (Grok Build)         │          │
│  │  • Trust-aware discovery                  │          │
│  │  • Regex matcher system                   │          │
│  │  • updatedInput rewriting                 │          │
│  │  • Fail-open/fail-closed policies         │          │
│  └──────────────────────────────────────────┘          │
│       │                                                 │
│       ▼                                                 │
│  ┌──────────────────────────────────────────┐          │
│  │         ACP Server (Grok Build)           │          │
│  │  • stdio JSON-RPC transport               │          │
│  │  • Tool manifest publication              │          │
│  │  • Context assembly API                   │          │
│  │  • Progress notifications                 │          │
│  └──────────────────────────────────────────┘          │
│       │                                                 │
│       ▼                                                 │
│  ┌──────────────────────────────────────────┐          │
│  │     Worktree Manager (Grok Build)         │          │
│  │  • Per-agent isolation                    │          │
│  │  • SQLite-backed pool                     │          │
│  │  • Merge-back with conflict resolution    │          │
│  └──────────────────────────────────────────┘          │
└─────────────────────────────────────────────────────────┘
```

---

## 6. Implementation Priority Matrix

| Phase | Component | Source | Effort | Impact | Dependencies |
|-------|-----------|--------|--------|--------|--------------|
| P0 | Evidence-Preserving Reducer | SoL-Pi | Large | 45-64% token savings | None |
| P0 | Causal Action Fusion | SoL-Pi | Medium | 30-50% turn reduction | None |
| P0 | Trust-Aware Hooks + Matcher | Grok Build | Medium | Security + extensibility | None |
| P0 | ACP stdio Transport | Grok Build | Medium | IDE integration | None |
| P1 | Evidence Store (persistent) | SoL-Pi | Medium | Audit trail | P0 Reducer |
| P1 | Config-File Hook Definitions | Grok Build | Small | Distributable hooks | P0 Trust |
| P1 | Tool Manifest Publication | Grok Build | Small | IDE discoverability | P0 ACP |
| P1 | HTTP Hook Support | Grok Build | Small | Remote integration | P0 Hooks |
| P2 | Auto-Research Feedback | SoL-Pi | Large | Self-optimizing thresholds | P0 Reducer |
| P2 | Tiered Model Routing | SoL-Pi | Medium | Cost optimization | P0 Reducer |
| P2 | Worktree Isolation | Grok Build | Large | Safe parallelism | None |
| P2 | Hook Discovery | Grok Build | Small | Auto-loading | P0 Trust |

---

## 7. Cross-Source Contradictions & Resolutions

| Tension | Resolution |
|---------|-----------|
| SoL-Pi assumes Pi's extension model; NeoTrix has SEAL pipeline | Map SoL-Pi's 4 mechanisms as SEAL Phase-1.5 (Harness Optimization) |
| Grok Build hooks fail-open; NeoTrix SHIELD needs fail-closed for security hooks | Dual policy: advisory hooks fail-open, security hooks (SHIELD) fail-closed |
| Grok Build uploads whole repos; NeoTrix Egress Privacy Guard blocks this | Already resolved: Egress Privacy Guard is factored out |
| SoL-Pi uses external evidence store; NeoTrix has KB | Store evidence in KB `kv_store` `evidence` namespace |
| Grok Build subagents are flat (depth=1); NeoTrix ConsciousnessTree is recursive | Keep flat subagent model; ConsciousnessTree operates at meta-level |

---

## 8. Validation Criteria

| Metric | Current | Target (Post-Fusion) |
|--------|---------|---------------------|
| Token usage per task | 100% baseline | 35-55% (SoL-Pi claims 45-64%) |
| Model round-trips per edit→test | 2 | 1 (causal fusion) |
| Hook discovery time | N/A (manual) | <10ms (file scan) |
| ACP handshake latency | N/A | <100ms |
| Evidence verification accuracy | 0% (no verification) | >95% (verified against archive) |
| Parallel agent isolation | None | Worktree per agent |

---

## 9. Files to Modify

| File | Changes |
|------|---------|
| `neotrix-core/src/l5_cognition/nt_mind/seal/harness_optimizer.rs` | Add EvidenceStore, DiagnosticReceipt, CausalLink, EnhancedObservationPack |
| `neotrix-core/src/l1_action/nt_io/hooks.rs` | Add HookMatcher, TrustStore, Discovery, ConfigLoader, HttpHook |
| `neotrix-core/src/l1_action/nt_io/acp.rs` | Add AcpTransport, ToolManifest, SessionManager, ProgressNotifier |
| `neotrix-core/src/l5_cognition/nt_mind/seal/pipeline.rs` | Wire HarnessOptimizer into SEAL phases |
| `neotrix-core/src/l1_action/nt_act/parallel_task.rs` | Add WorktreeManager integration |
| `crates/neotrix-kb/src/lib.rs` | Add `evidence` namespace to kv_store |

---

## 10. Absorbed Terminology Updates

| Term | Definition | Source |
|------|-----------|--------|
| **Evidence-Preserving Reducer** | Small model pre-screen → verify against archive → forward verified fragments only | SoL-Pi |
| **Causal Action Fusion** | Fuse heterogeneously-linked tools (edit→build) not just identical consecutive tools | SoL-Pi |
| **DiagnosticReceipt** | Structured output from small model: error lines + stack traces + file:line + suggested fix | SoL-Pi |
| **EvidenceArchive** | Persistent session-scoped log storage with TTL, file budgets, SHA-256 hashing | SoL-Pi |
| **HookMatcher** | Regex-based event field selector for selective hook triggering | Grok Build |
| **TrustStore** | Global/project trust levels for hook security enforcement | Grok Build |
| **updatedInput** | PreToolUse hook can rewrite tool input before execution | Grok Build |
| **WorktreeManager** | Per-agent git worktree with SQLite-backed pool and merge-back | Grok Build |
| **AcpTransport** | JSON-RPC over stdin/stdout with Content-Length framing | Grok Build |
