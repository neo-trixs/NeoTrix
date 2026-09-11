# External Absorption — Agent Harness Architecture Fusion Plans

**Date**: 2026-09-11
**Scope**: Deep research on 5 external projects/patterns, reverse-engineered for NeoTrix fusion
**Sources**: NVlabs SoL-Pi, Grok Build (xAI), DeepSeek Harness/Cordis, Frontier Model Patterns (Claude/GPT/Gemini), Cross-Domain Universal Patterns

---

## 1. SoL-Pi: Token-Efficient Auto-Research Loops

### Source
- **URL**: https://nvlabs.github.io/SoL-Pi
- **Paper**: SoL-Pi: Scaling Auto-Research Loops for Efficient Agent Harnesses (NVlabs, Sep 2026)
- **Base harness**: Pi (418-line minimalist agent loop, 17.5K stars)
- **Benchmark**: EdgeBench (134 tasks, 12-72hr horizons, 38K+ hours of agent interaction)

### Algorithm / Data Structure

**4 surviving mechanisms from 152 proposals (2.6% survival rate)**:

1. **Action Fusion** — Merge edit + follow-up command into one tool call, eliminating an intermediate model round-trip. Oracle Analysis found 12.3% of cross-turn transitions were edit->command (85.1% Bash). Final trigger rate: 100%, -10.8% turns, -11.5% tokens.

2. **Online Context Compact** — Decompose tasks into subtasks; compact at semantic completion points (not late in run). Different compaction clock than KV-cache reuse. Only fires when expected future savings repay the rewrite cost.

3. **ObservationPack** — Replace repeated full tool output with stable handle + short excerpt. Archive payload locally, recall exact pages on demand. 2,048-byte head + 1,536-byte tail + 2 full sends before projection. Result: -23.58% provider bill, +22.92% normalized score.

4. **Evidence-Preserving Reducer** — Delegate log reading to cheaper agent, bind receipt to archived log, verify every quoted line before frontier agent sees it. Delegation without trust.

**Key insight**: Log-sigmoid learning curve fits environment learning (R-squared=0.998). Learning speed doubles every ~3 months across frontier models.

### NeoTrix Domain
**NT-MIND** (auto-research loops) + **NT-CORE** (context management) + **NT-MEMORY** (observation compression)

### Existing Skeleton
- `nt_mind::seal` — SEAL pipeline (evolution loop)
- `nt_core::attention` — GWT attention routing
- `nt_memory::kv_cache` — KV cache management

### Fusion Approach

| SoL-Pi Mechanism | NeoTrix Integration | Target Module |
|------------------|---------------------|---------------|
| **Action Fusion** | Extend nt_act tool registry with fused-action schema: `EditThenRun { edit, command }` — harness detects edit->command pattern, executes locally, returns combined observation | `nt_act::tool_registry` |
| **Online Context Compact** | Add `compact_clock` to NT-MEMORY: semantic completion detection triggers compaction opportunity evaluation, not just buffer-full | `nt_memory::context_manager` |
| **ObservationPack** | Implement observation lifecycle in NT-MEMORY: archive -> handle -> paged recall. Hook into GWT attention to control which observations stay resident | `nt_memory::observation_lifecycle` |
| **Evidence-Preserving Reducer** | Extend NT-MIND delegation model: subagent returns verified receipt (quoted lines + source archive reference), not summary. Parent agent verifies before consuming | `nt_mind::delegation` |

**Auto-Research Loop**: Port SoL-Pi's disposable skill loop pattern into SEAL pipeline. Each SEAL phase becomes a template instantiation, not a fixed graph.

### Universal Applicability
HIGH — All 4 mechanisms are model-agnostic. Action Fusion works with any model that generates tool calls. ObservationPack targets context window waste, universal across providers. Evidence-Preserving Reducer is delegation-pattern-agnostic.

---

## 2. Grok Build: Hook System & ACP Protocol

### Source
- **URL**: https://github.com/xai-org/grok-build (26.5K stars, Apache-2.0)
- **Docs**: https://docs.x.ai/build/overview
- **Language**: Rust (pinned toolchain)
- **Open source**: Jul 15, 2026

### Algorithm / Data Structure

**Hook System (Claude-compatible JSON format)**:
- Events fire at 3 cadences: per-session (SessionStart/End), per-turn (UserPromptSubmit, Stop), per-tool (PreToolUse, PostToolUse)
- 13 event types: SessionStart, UserPromptSubmit, PreToolUse, PostToolUse, PostToolUseFailure, PermissionDenied, Stop, StopFailure, Notification, SubagentStart, SubagentStop, PreCompact, PostCompact, SessionEnd
- `matcher`: regex on tool name (Claude names auto-mapped to Grok names)
- `type`: "command" (shell) or "http" (POST to URL)
- **Blocking events**: PreToolUse (deny), Stop/SubagentStop (block — feeds reason back to model, up to 8 continuations)
- **Fail-open**: timeouts/crashes/malformed output -> proceed, log failure
- Environment variables: GROK_HOOK_EVENT, GROK_HOOK_NAME, GROK_SESSION_ID, GROK_WORKSPACE_ROOT

**ACP (Agent Client Protocol)**:
- Embeddable in editors via stdio
- Headless mode: `-p "prompt"`, `--output-format streaming-json`
- Session persistence: `~/.grok/sessions/` (same layout for TUI, headless, ACP)

**Repository Layout**:
```
crates/codegen/xai-grok-shell    # Agent runtime + leader/stdio/headless
crates/codegen/xai-grok-tools    # Tool implementations
crates/codegen/xai-grok-hooks    # Hook discovery + dispatch
crates/codegen/xai-grok-workspace # Host filesystem, VCS, checkpoints
```

### NeoTrix Domain
**NT-SHIELD** (hook safety, sandbox) + **NT-ACT** (tool dispatch) + **NT-IO** (ACP protocol, headless)

### Existing Skeleton
- `nt_shield::egress_privacy_guard` — outbound request filtering
- `nt_act::tool_dispatch` — tool execution pipeline
- `nt_io::cli` — CLI interface

### Fusion Approach

| Grok Pattern | NeoTrix Integration | Target Module |
|--------------|---------------------|---------------|
| **Hook Event System** | Implement `NtHookRegistry` with same 13 event types. JSON config files in `~/.neotrix/hooks/`. Matcher regex on tool names. Blocking PreToolUse + Stop gates with continuation cap (8 max) | `nt_shield::hooks` |
| **Fail-Open Contract** | Default behavior: hook failure -> proceed + log. Only explicit deny blocks. Matches NeoTrix R-P82 risk assessment pattern | `nt_shield::hooks` |
| **ACP Protocol** | Implement `NtAcpServer` as stdio transport for agent runtime. Enable editor embedding (VSCode, Cursor, Zed). Session persistence via NT-MEMORY | `nt_io::acp` |
| **Sandbox Execution** | Extend `nt_shield::sandbox` with worktree-per-subagent isolation. Each subagent gets own filesystem view | `nt_shield::sandbox` |
| **Tool Name Aliasing** | Support Claude/Grok tool name mapping: Bash <-> run_terminal_cmd, Read <-> read_file, Edit <-> search_replace. Hooks written for one platform work on NeoTrix | `nt_act::tool_alias` |

**Hook Events to NT-HOOK Event Map**:
```
SessionStart     -> nt_mind::session::on_start
UserPromptSubmit -> nt_core::gwt::on_prompt
PreToolUse       -> nt_shield::gate::before_tool
PostToolUse      -> nt_act::tool_registry::after_tool
Stop             -> nt_core::gwt::on_turn_end
SubagentStart    -> nt_mind::delegation::on_spawn
PreCompact       -> nt_memory::context::before_compact
```

### Universal Applicability
HIGH — Hook events are model-agnostic. ACP is transport-level, not model-level. Sandbox isolation works with any provider.

---

## 3. DeepSeek Harness / Cordis: Everything is a Plugin

### Source
- **URL**: https://github.com/deepseek-ai/deepseek-harness (215K stars, MIT)
- **Framework**: Cordis (TypeScript plugin framework, 4 years in production via Koishi chatbot, 4000+ community plugins)
- **Paper**: "A Programming Paradigm for Spatiotemporal Composability" (Peking University + DeepSeek, Aug 2026)
- **Default deployment**: 159 plugins in default config

### Algorithm / Data Structure

**Cordis Core (5 ideas)**:
1. **Plugin** — Object contributing a capability. No categories, no hierarchy.
2. **Context** — Repository of capabilities. Stable `ctx.<key>` names (ctx.tools, ctx.llm, ctx.agents).
3. **Dependencies** — Declared via `inject`. Plugin waits until required services exist. Load order derived, not manual.
4. **Events** — Typed with declared dispatch modes: `emit` (observe), `waterfall` (wrap), `parallel` (fan out), `serial` (ordered), `bail` (stop at first).
5. **Teardown** — Every registration knows how to undo itself. Plugin unload -> all installed effects unwind.

**Bundle/Profile/Layer Architecture**:
- **Bundle**: Distribution format for Cordis config rows + code. Each declares in `package.json` under `dsh` field.
- **Profile**: Named composition (web, headless, sdk, acp). Lists bundles to stack.
- **Layer ordering**: base bundles -> profile patch -> home patch -> CLI overlay

**Session as Append-Only Event Log**:
- `Session` = append-only source of truth. LLM message history is *derived* from it.
- `deriveMessages()` incrementally projects surface entries.
- "Model-visible means logged" — anything reaching model must be reconstructable from log.

**Service Registry Pattern**:
```
core/session  -> ctx.sessions (append-only event log)
core/tools    -> ctx.tools (scoped registry + execution)
core/agent    -> ctx.agents (agent interface + lifecycle)
llm/llm       -> ctx.llm (message vocabulary + adapter seam)
core/system-prompt -> ctx.systemPrompt (prompt assembly)
```

**Event Dispatch Modes**:
| Mode | Awaited? | Dispatch | Return Value |
|------|----------|----------|--------------|
| emit | No | registration order | No |
| waterfall | No | registration order | Yes |
| parallel | Yes | all in parallel | No |
| serial | Yes | registration order | Yes |
| bail | No | until first bail | Yes |

### NeoTrix Domain
**NT-CORE** (context registry) + **NT-MEMORY** (event-sourced session) + **NT-MIND** (plugin lifecycle)

### Existing Skeleton
- `nt_core::capability_registry` — capability registration
- `nt_memory::session` — session management
- `nt_mind::skill_engine` — skill loading

### Fusion Approach

| Cordis Pattern | NeoTrix Integration | Target Module |
|----------------|---------------------|---------------|
| **No Privileged Core** | Restructure NeoTrix modules as Cordis-style plugins: each domain registers services on stable context keys. No module imports another directly — lookup by name | `nt_core::context_registry` |
| **Inject Dependencies** | Add `inject` field to NT-* module declarations. Module startup waits until required services exist. Eliminates manual boot ordering | `nt_core::lifecycle` |
| **Reversible Effects** | Every NT-* registration (tool, listener, prompt section) returns a disposer. Module unload -> clean unwind. Enables hot-reload | `nt_core::effect_registry` |
| **Event Dispatch Modes** | Implement 5 dispatch modes for NT EventBus: emit/waterfall/parallel/serial/bail. Currently EventBus only has emit. Add waterfall for prompt assembly, bail for gate checks | `nt_core::eventbus` |
| **Event-Sourced Session** | Restructure NT-MEMORY session as append-only log. LLM message history derived via `deriveMessages()`. "Model-visible means logged" invariant | `nt_memory::event_sourced_session` |
| **Bundle/Profile System** | Create NeoTrix profiles: `desktop` (Tauri), `cli` (terminal), `server` (headless), `acp` (editor embedding) | `nt_io::profiles` |

**Key Fusion**: The Cordis `inject` pattern directly addresses NeoTrix's current manual dependency wiring in `nt_mind::boot`. Instead of explicit init order, modules declare `inject: ['tools', 'llm', 'sessions']` and Cordis resolves order automatically.

### Universal Applicability
HIGH — Plugin architecture is model-agnostic. Event-sourced session works with any LLM provider. Bundle/profile system enables multi-surface deployment.

---

## 4. Frontier Model Patterns: Context Management & Agent Loops

### 4a. Claude Code Architecture

**Source**: Anthropic (Claude Code v2.1.88 analysis, MBZUAI VILA-Lab)
- **Key stat**: ~98.4% harness infrastructure, ~1.6% AI decision logic (512K lines across 1,884 files)

**6-Layer Architecture**:
```
Input Layer        -> session mgmt, permission gating, YAML trust tiers
Knowledge Layer    -> skill registry, context compressor (5-layer cascade), task graph, cross-session memory
Multi-Agent Layer  -> subagents (lightweight, parent-child) + agent teams (independent instances)
Observability Layer -> event bus + lifecycle hooks + daemon threads
Master Loop        -> single-threaded "dumb loop" — all intelligence in layers, not loop
```

**Three Anthropic Harness Design Patterns**:
1. **Lean on the model, not the harness** — use what Claude already knows
2. **Strip your harness down** — test what Claude can do without harness mediation
3. **Set boundaries carefully** — promote actions to dedicated tools for typed intercept/audit

**Prompt Caching Optimization**: Stable prompt prefix, append-only history, fixed tool catalog per session, state transitions as messages/mode flags (not prompt rewrites). Cache hits = 10% cost.

### 4b. GPT Agent Patterns

**Source**: OpenAI Agents SDK, Responses API

**Patterns**:
- **Handoffs**: Full context transfer to specialized agent (takes over conversation)
- **Agents as Tools**: Agent runs independently, returns result to parent (parallel execution)
- **LLM-as-a-Judge**: Generate -> second model critiques -> iterate
- **CodeAct**: Model writes Python program calling tools, runs in sandbox, returns consolidated result (collapses multi-turn into single turn)
- **Hosted Multi-Agent**: Server-coordinated GPT subagents via Responses API

### 4c. Gemini Context Management

**Source**: Google Gemini Enterprise Agent Platform, Gemini CLI

**Key Features**:
- **Agent Memory Bank**: Structured schemas for auto-extracting/maintaining conversation context across long-running tasks (7-day runtime)
- **Agent Identity**: Native IAM type binding access to agent runtime (least-privilege, non-repudiable audit)
- **Agent Gateway**: Central control point with Model Armor (prompt injection, tool poisoning, data leakage protection)
- **Context Caching**: Implicit (automatic, 90% discount on Gemini 2.5+) and explicit (manual cache objects, TTL control)
- **AdaCoM** (arXiv:2605.30785): Trained external context manager using RL. Flexible modification actions (not just summarization). 25% improvement across 28 cross-agent pairs.

### NeoTrix Domain
**NT-CORE** (GWT attention routing) + **NT-MEMORY** (context management) + **NT-MIND** (delegation patterns)

### Fusion Approach

| Frontier Pattern | NeoTrix Integration | Target Module |
|------------------|---------------------|---------------|
| **5-Layer Context Compressor** | Implement cascade in NT-MEMORY: (1) duplicate detection, (2) stale observation eviction, (3) Evidence-Preserving Reduction (from SoL-Pi), (4) semantic compression, (5) full summarization. Each layer has different evidence preservation guarantee | `nt_memory::context_compressor` |
| **CodeAct Pattern** | Add `CodeExecution` tool to NT-ACT: model writes Python/Rust script, sandbox executes, returns consolidated result. Collapses multi-tool-turn into single turn | `nt_act::code_execution` |
| **Prompt Cache Optimization** | Restructure NT-CORE system prompt: stable prefix (instructions + skills), append-only history, fixed tool catalog. Model state as mode flags, not prompt rewrites | `nt_core::prompt_engine` |
| **Agent Memory Bank** | Implement structured memory schemas in NT-MEMORY: auto-extract key facts, preferences, decisions from conversation. Persistent across sessions via KB | `nt_memory::agent_memory_bank` |
| **AdaCoM Context Manager** | Train lightweight context manager (Qwen3-4B scale) as NT-MIND module. Flexible edit actions: inject, delete, summarize, reorder — not just prune. RL-trained | `nt_mind::context_manager` |

### Universal Applicability
HIGH — Context compression is provider-agnostic. CodeAct works with any coding model. Prompt cache optimization applies to all providers with caching (Anthropic, Google, OpenAI).

---

## 5. Cross-Domain Universal Patterns

### 5a. Two-Dimensional Agent Pattern Framework

**Source**: arXiv:2605.13850 (Huang & Zhou, 2026)

**Cognitive Function x Execution Topology matrix**:
- 7 cognitive functions: Perception, Memory, Reasoning, Action, Reflection, Collaboration, Governance
- 6 execution topologies: Chain, Route, Parallel, Orchestrate, Loop, Hierarchy
- 28 named patterns (15 original)

**5 Laws of Pattern Selection** (empirical):
1. Time pressure -> favor Chain/Route over Orchestrate
2. Action authority -> parallel when independent, sequential when dependent
3. Failure cost asymmetry -> more verification when cost is high
4. Volume -> parallelize embarrassingly parallel stages
5. Environmental uncertainty -> favor Loop with reflection

### 5b. Harness-Bench: Model-Harness Pair Evaluation

**Source**: arXiv:2605.27922 (Yao et al., May 2026)

**Key insight**: Agent capability should be reported at model-harness configuration level, not base model alone. Same model under different harness -> different rankings.

### 5c. Universal Agent Harness Architecture

**Source**: Gist: "Modern Agent Harness Blueprint 2026" + AgentSmith + Microsoft Agent Framework

**Consensus 5-Layer Architecture**:
```
Surface Layer     -> CLI / IDE / Web / API
Adapter Layer     -> ACP / REST / MCP
Orchestration     -> Agent loop, subagent manager, delegation
State Layer       -> Session log, memory, context, artifacts
Observability     -> Tracing, metrics, replay, eval
```

**Key Cross-Harness Patterns**:
1. **Session as unit of resumability** — every conversation/job is resumable
2. **Task as coordination object** — status, owner, dependencies, blockers, artifacts
3. **Artifact as durable output** — referenced by URI, not inline
4. **PTC (Programmatic Tool Calling)** — typed-stub tool invocation, parallel calls in single turn
5. **Egress Privacy Guard** — outbound request filtering (already in NeoTrix)

### NeoTrix Domain
**NT-CORE** (pattern framework) + **NT-MEMORY** (session/task/artifact model) + all domains (universal application)

### Fusion Approach

| Universal Pattern | NeoTrix Integration | Target Module |
|-------------------|---------------------|---------------|
| **2D Pattern Matrix** | Use as NT-CORE reasoning framework: map task properties to architectural choices via Cognitive Function x Execution Topology. GWT salience weighted by 5 Laws | `nt_core::pattern_selector` |
| **Model-Harness Pair Reporting** | Add harness-version to all NT performance metrics. Benchmark at (model, harness) pair level, not model alone. Lock-harness protocol for internal A/B tests | `nt_mind::metrics` |
| **5-Layer Consensus Architecture** | Align NeoTrix 6-layer architecture with consensus. Map: L1 Action -> Surface+Adapter, L2 Perception -> Observation, L3 Embodiment -> State, L4 Emotion -> (no consensus equivalent), L5 Cognition -> Orchestration, L6 Meta -> Observability | `nt_core::architecture` |
| **Session/Task/Artifact Model** | Formalize NT-MEMORY with session (resumable unit), task (coordination object), artifact (durable output with URI). Currently ad-hoc in nt_memory | `nt_memory::data_model` |
| **Cross-Harness Memory** | Implement Warp Oz-style cross-harness persistent memory: knowledge persists across Claude Code, Codex, NeoTrix sessions. Already partially in `kv_store` | `nt_memory::cross_session` |

### Universal Applicability
HIGH — These patterns are framework-agnostic by design. The 2D matrix and 5 Laws apply to any agent architecture. The 5-Layer consensus is emerging as industry standard.

---

## 6. Priority Ranking & Implementation Roadmap

### P0: Immediate (next sprint)

| Pattern | Source | Effort | Impact |
|---------|--------|--------|--------|
| **Hook Event System** | Grok Build | Medium | High — enables safety gates, logging, automation |
| **Action Fusion** | SoL-Pi | Low | High — 10-11% token savings with minimal code |
| **Event Dispatch Modes** | Cordis | Medium | High — unlocks waterfall/bail patterns for EventBus |

### P1: Short-term (1-2 months)

| Pattern | Source | Effort | Impact |
|---------|--------|--------|--------|
| **Inject Dependencies** | Cordis | Medium | High — eliminates manual boot ordering |
| **ObservationPack** | SoL-Pi | Medium | High — 23% cost reduction on tool outputs |
| **Event-Sourced Session** | Cordis | High | High — enables resume/fork/replay |
| **Prompt Cache Optimization** | Claude | Low | Medium — 10% cost on cached providers |

### P2: Medium-term (2-4 months)

| Pattern | Source | Effort | Impact |
|---------|--------|--------|--------|
| **5-Layer Context Compressor** | Claude + SoL-Pi | High | High — systematic context management |
| **CodeAct Pattern** | GPT/OpenAI | Medium | Medium — collapses multi-turn into single turn |
| **Bundle/Profile System** | Cordis | High | Medium — multi-surface deployment |
| **ACP Protocol** | Grok Build | Medium | Medium — editor embedding |

### P3: Long-term (4-6 months)

| Pattern | Source | Effort | Impact |
|---------|--------|--------|--------|
| **AdaCoM Context Manager** | Gemini research | Very High | High — RL-trained context optimization |
| **Agent Memory Bank** | Gemini | High | Medium — structured cross-session memory |
| **2D Pattern Matrix** | Research | Medium | Medium — reasoning framework for architecture selection |
| **Disposable Skill Loops** | SoL-Pi | High | High — SEAL pipeline evolution |

---

## 7. Cross-Source Synthesis: 5 Universal Patterns

These patterns emerge consistently across ALL sources:

### Pattern 1: Session-as-Source-of-Truth
Every source converges on append-only event logs as the canonical state representation. LLM message history is *derived*, not stored.
- Grok: `~/.grok/sessions/` (same layout TUI/headless/ACP)
- Cordis: `Session` append-only, `deriveMessages()` projection
- Claude: event bus + lifecycle hooks as audit trail
- SoL-Pi: trajectory logs for auto-research
- Gemini: Agent Memory Bank structured schemas

### Pattern 2: Fail-Open Safety Gates
Safety hooks block on explicit deny only. Everything else fails open.
- Grok: PreToolUse deny, Stop block, timeout = fail-open
- Cordis: reversible effects, plugin unload = clean unwind
- Claude: permission classifiers + static analysis + conservative defaults
- NeoTrix: R-P82 risk assessment (score >= 60 requires human, >= 80 auto-reject)

### Pattern 3: Context as Scarce Resource
Context window management is the primary engineering challenge.
- SoL-Pi: ObservationPack, Online Context Compact, Evidence-Preserving Reducer
- Claude: 5-layer context compressor at 95% capacity
- Gemini: AdaCoM RL-trained context manager, implicit/explicit caching
- KVMem (from CONTEXT.md): paged KV virtualization for >256K sessions

### Pattern 4: Delegation Without Trust
Subagent results must be verifiable, not trusted.
- SoL-Pi: Evidence-Preserving Reducer (verified receipts)
- Cordis: inject + service replacement (provider swap propagates)
- Grok: worktree-per-subagent isolation
- Claude: subagents (parent-child) + agent teams (independent)

### Pattern 5: Model-Harness Co-Evolution
Harness design must evolve as models improve. What's in the harness today may move to the model tomorrow.
- Anthropic: "Strip your harness down — test what Claude can do without"
- Harness-Bench: agent capability = f(model, harness), not f(model) alone
- SoL-Pi: capability floors constrain efficiency gains
- NeoTrix Axiom A1: Cost-Aware Routing (cheap models for I/O, expensive for reasoning)

---

## 8. NeoTrix-Specific Integration Map

```
Current NeoTrix Module          Absorbed Pattern              Source
─────────────────────────────   ──────────────────────────    ──────────
nt_act::tool_dispatch           Action Fusion                 SoL-Pi
nt_act::tool_dispatch           Tool Name Aliasing            Grok Build
nt_core::eventbus               5 Dispatch Modes              Cordis
nt_core::capability_registry    No Privileged Core            Cordis
nt_core::lifecycle              Inject Dependencies           Cordis
nt_core::prompt_engine          Prompt Cache Optimization     Claude
nt_core::pattern_selector       2D Pattern Matrix             Research
nt_memory::session              Event-Sourced Session         Cordis
nt_memory::context_manager      Online Context Compact        SoL-Pi
nt_memory::observation_lifecycle ObservationPack              SoL-Pi
nt_memory::context_compressor   5-Layer Cascade               Claude+SoL-Pi
nt_memory::agent_memory_bank    Agent Memory Bank             Gemini
nt_memory::cross_session        Cross-Harness Memory          Warp Oz
nt_mind::seal                   Disposable Skill Loops        SoL-Pi
nt_mind::delegation             Evidence-Preserving Reducer   SoL-Pi
nt_mind::context_manager        AdaCoM Context Manager        Gemini
nt_shield::hooks                Hook Event System             Grok Build
nt_shield::sandbox              Worktree Isolation            Grok Build
nt_io::acp                      ACP Protocol                  Grok Build
nt_io::profiles                 Bundle/Profile System         Cordis
nt_act::code_execution          CodeAct Pattern               GPT/OpenAI
```
