# PrimeIntellect Prime Agent — Architecture Analysis Report

> **Source**: [PrimeIntellect-ai/prime-agent](https://github.com/PrimeIntellect-ai/prime-agent) (20.6k★, MIT License)
> **Paper**: [arXiv:2608.23552](https://arxiv.org/abs/2608.23552)
> **Purpose**: Self-improving RLM (Recursive Language Model) agent for coding workflows and long-running autonomous tasks
> **Date**: 2026-09-13

---

## 1. Architecture Overview

Prime Agent uses a **5-layer separation** between client presentation, process coordination, agent execution, model-facing Python, and persisted state:

```
┌─────────────────────────────────────────────────────────────────┐
│  Interactive TUI  │  Print/JSON/RPC clients                     │  ← Client Layer
├─────────────────────────────────────────────────────────────────┤
│  AgentConnection (client-side execution boundary)               │  ← Connection Layer
├─────────────────────────────────────────────────────────────────┤
│  Daemon Supervisor (routing · attachments · recovery)           │  ← Supervisor Layer
│  Catalog Process (saved-session scans)                          │
├─────────────────────────────────────────────────────────────────┤
│  Session Worker (one root session tree)                         │  ← Worker Layer
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  AgentSessionRuntime                                     │   │
│  │  ├── Root AgentSession (model streams, tools, compaction)│   │
│  │  ├── Scheduler (heartbeats, goals, autonomous)           │   │
│  │  ├── Root Python Kernel (persistent REPL)                │   │
│  │  └── RLM Child Runtimes (recursive subagents)            │   │
│  └──────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────┤
│  Model Providers (Anthropic, OpenAI, Google, Prime Inference)   │  ← Provider Layer
│  Session JSONL + Artifacts                                      │  ← Persistence Layer
└─────────────────────────────────────────────────────────────────┘
```

### Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| **Persistent Python REPL as single model tool** | One tool (`ipython`) replaces dozens of built-in tools; all capabilities compose through Python |
| **Daemon-backed workers** | Sessions survive terminal detach; workers are process-isolated for failure containment |
| **Separation of client and execution** | TUI owns rendering/input; worker owns execution; enables reattachment and headless modes |
| **Recursive subagents as native API** | `rlm.spawn()` creates real child agents with independent contexts, not just prompts |
| **Continual Harness** | Durable state (prompts, memories, skills, subagent specs) refines through small, evidence-backed updates |

---

## 2. Agent Components

### 2.1 Core Abstractions

| Component | Purpose | Lifetime |
|-----------|---------|----------|
| **RLM (Recursive Language Model)** | Treats context as variables, tools as recursive function calls | Session |
| **Continual Harness** | Stores supplemental prompts, memories, skill descriptions, subagent specs | Persistent across sessions |
| **AgentSession** | Owns provider calls, queues, tools, compaction, goals, child lifecycles, transcripts | Per-session |
| **Python Kernel** | Persistent REPL — the model-facing control environment | Per-session (survives compaction) |
| **Scheduler** | Manages heartbeats, goals, autonomous continuations | Per-session |
| **Daemon Supervisor** | Discovery, routing, attachments, worker health, cross-agent messages | System-wide |

### 2.2 Python Kernel Capabilities

The Python kernel is the single entry point for all model operations:

```python
# File operations
from pathlib import Path
config_files = list(Path(".").rglob("*.toml"))

# Shell commands
result = await bash("npm run check")

# Subagent spawning
handle = await rlm.spawn("Review auth flow", name="auth-reviewer")

# Skill invocation
report = await release_audit(repository=".", target_version="0.4.0")

# Agent communication
receipt = await agent_message.send("message", receiver_role="sibling")

# Heartbeats
await rlm_heartbeat.create("check test results", interval="5m")

# Goals
state = await goal.get()
await goal.complete()
```

### 2.3 Process Model

```
Supervisor Process
├── Catalog Process (session scanning)
└── Worker Process (per root session tree)
    ├── AgentSession (TypeScript)
    ├── Python Kernel (child process)
    ├── Scheduler
    └── Child AgentSessions (recursive)
        └── Python Kernel (per child)
```

Workers and kernels are **separate processes** for lifecycle/failure containment, not security sandboxes. They run with the same OS permissions as the client.

---

## 3. Task Execution Flow

### 3.1 Prompt Execution Sequence

```
User → AgentConnection → Supervisor → Worker → AgentSession
  → Model Provider (stream) → Python Kernel (if tool call)
    → Host Request (if typed operation) → Back to AgentSession
  → Session Storage (JSONL append)
  → Back through chain → User
```

**Key invariant**: From the session queue onward, the same path is used regardless of source (user, heartbeat, cron, goal, autonomous, or another agent).

### 3.2 Tool Call Flow

```
Model generates Python code → AgentSession sends to Kernel
  → Kernel executes
    → If typed host request: Kernel → AgentSession → back to Kernel
    → If ordinary execution: Kernel → result/stdout/error → AgentSession
  → AgentSession appends transcript
```

### 3.3 Background Command Handling

```python
# Non-blocking bash call
checks = bash("npm test")
checks.pid  # Returns immediately

# When process finishes, agent receives notice at next safe turn boundary
# Reads from any live cell withdraw the notice
await checks  # or checks.poll(), checks.output(), checks.tail()
```

### 3.4 Autonomous Mode Pipeline

```
Autonomous Mode Enabled
  → [Continue] injects follow-up message
  → Agent executes turns
  → [Quality Gates] run after each assistant response
    → All gates pass: can finish
    → Gate fails: output returned to agent for repair, retry up to retry limit
  → [Budget Checks] (order: continuations → turns → tokens → time)
    → Any limit reached: no more automatic continuations
  → Repeat until completion or budget exhaustion
```

---

## 4. Memory System

### 4.1 State Persistence Layers

| Layer | Content | Format | Lifetime |
|-------|---------|--------|----------|
| **JSONL Transcript** | Full conversation history | Flat JSONL files | Permanent |
| **Session Artifacts** | Feature-specific state per session | Filesystem | Permanent |
| **Python Kernel State** | Variables, imports, functions, task handles | In-memory (survives compaction) | Session |
| **Continual Harness** | Supplemental prompts, memories, skills, subagent specs | Durable state | Cross-session |
| **Retained Subagents** | Completed child sessions | Session files | Until parent closes |

### 4.2 Compaction

When context overflows or nears threshold:
1. **Summarize** older messages
2. **Retain** recent context and kernel state
3. **Continue** from compacted context

**Critical**: Python kernel persists through compaction — variables, imports, helper functions, and task state remain available.

```python
await compact.status()
await compact.run("Preserve failing tests and remaining migration steps")
```

### 4.3 Session Recovery

```
Terminal closes → Worker continues running
  → User: prime-agent agents (list running)
  → User: prime-agent attach <agent> (reattach)
  → Supervisor restores session state from JSONL + artifacts
```

### 4.4 Goal Persistence

```python
# Goals persist across turns until completed/paused/errored/cleared
/goal Ship the release and verify every artifact
/goal status
/goal pause
/goal resume
/goal clear

# From kernel
state = await goal.get()  # Includes token usage, elapsed time, continuation count
await goal.complete()
```

---

## 5. Tool Integration

### 5.1 Single Tool Philosophy

Prime Agent uses **one built-in model tool**: `ipython`. All capabilities compose through the Python kernel:

| Capability | How It's Accessed |
|------------|-------------------|
| File read/write | `pathlib`, `open()` in Python |
| Shell commands | `await bash("command")` |
| Subagents | `await rlm.spawn(...)` |
| Skills | `await skill_name(args)` |
| Web search | `await websearch("query")` |
| Goals | `await goal.get()` |
| Heartbeats | `await rlm_heartbeat.create(...)` |
| Agent messaging | `await agent_message.send(...)` |

### 5.2 Skills System

Skills follow the Agent Skills standard with Python-backed extensions:

```
my-skill/
├── SKILL.md              # Required: frontmatter + instructions
├── pyproject.toml        # Marks as Python-backed
├── src/
│   └── my_skill/
│       └── __init__.py   # Exposes run() or async callable
├── scripts/              # Helper scripts
└── references/           # On-demand documentation
```

**Discovery hierarchy**:
1. `~/.prime/agent/skills/` (global)
2. `.prime/agent/skills/` (project)
3. `.agents/skills/` (ancestors to git root)
4. `skills/` in packages
5. Built-in skills (lowest precedence)

**Loading**: Only skill metadata in startup prompt; full SKILL.md loaded on-demand when task matches.

### 5.3 Python-Backed Skill Contract

```python
# Skill exposes run() or direct async callable
async def run(query: str, limit: int = 5) -> str:
    """Search the web and return a concise summary."""
    ...

# Usage in kernel
result = await web_search("prime agent skills")
```

Skills are installed editable into the kernel venv. `pyproject.toml` changes trigger venv rebuild.

### 5.4 Extensions

Extensions load from:
- CLI: `-e <source>` (path, npm, or git)
- Settings: `extensions` array
- Discovery: `.prime/agent/extensions/`

---

## 6. Communication

### 6.1 Agent-to-Agent Messaging

Direct messages route through the daemon supervisor:

```python
# Discover agents
roster = await agent_observe.list_agents()

# Send to sibling
receipt = await agent_message.send(
    "Recheck the endpoint after the latest edit",
    receiver_role="sibling",
    receiver_name="api-reviewer",
    mode="auto",
)

# Send to child
children = await rlm.list_subagents()
await agent_message.send(
    "Continue with the updated diff",
    receiver_role="child",
    receiver_name=child.session_name,
)

# Broadcast within family roster
await agent_message.send("all", "New migration applied")
```

### 6.2 Delivery Modes

| Mode | Behavior |
|------|----------|
| `auto` | Steer busy target; deliver immediately to idle target |
| `steer` | Inject into active work regardless |
| `follow_up` | Wait until target's current work finishes |

### 6.3 Message Constraints

- Daemon derives sender identity
- Enforces message-size, rate, and pending-queue limits
- Receipt: `delivered` (reached idle context) or `queued` (accepted for later delivery)

### 6.4 Subagent Communication

```
Parent AgentSession
├── spawns child via rlm.spawn()
│   ├── Returns immediately with handle (name, session_dir, model)
│   └── Child runs independently
├── Child replies via agent_message.send(..., receiver_role="parent")
│   └── Parent receives as ordinary agent message
├── Parent follows up via agent_message.send(..., receiver_role="child")
│   └── Child receives as steering input
```

---

## 7. Error Handling

### 7.1 Process Isolation

- Workers and kernels are **separate processes**
- Kernel crash doesn't crash the worker
- Worker crash is detected by supervisor for recovery

### 7.2 Daemon Recovery

```
Supervisor
├── Monitors worker health
├── Routes attachments
├── Handles crash recovery
├── Manages lease-based lifecycle
└── Provides backpressure
```

### 7.3 Background Command Errors

```python
# Non-blocking commands return exit code
checks = bash("npm test")
result = await checks  # Includes output, exit code, error handling

# Unawaited handles send "Background command finished" notice
# with PID and foreground exit code
# Agent inspects with poll(), output(), or tail()
```

### 7.4 Gate Failures (Autonomous Mode)

```
Gate command runs → Pass: continue/finish
                   → Fail: bounded output returned to agent
                   → Agent repairs → Gate reruns
                   → Retry limit reached: gate exhausts
```

Prime Agent avoids rerunning unchanged failed gates — advances attempt count instead.

### 7.5 Schedule Resilience

- Due ticks claimed before delivery (crash doesn't replay uncertain prompts)
- Missed ticks coalesced (no unbounded backlog)

### 7.6 Compaction Safety

- Compaction is **not** a completion signal
- Does not stop goals, autonomous continuations, heartbeats, or child sessions
- Kernel state survives — variables and task handles persist

---

## 8. Performance Optimization

### 8.1 Context Management

| Technique | Mechanism |
|-----------|-----------|
| **Progressive disclosure** | Skill descriptions always in context; full instructions load on-demand |
| **Automatic compaction** | Summarizes older context while retaining recent messages and kernel state |
| **Prompt-as-variable** | Context treated as variables; Python holds working state |
| **Scoped models** | Ctrl+P cycling with `--models` patterns for cost optimization |

### 8.2 Parallelism

| Pattern | Implementation |
|---------|---------------|
| **Concurrent subagents** | `rlm.spawn()` returns immediately; children run in parallel |
| **Background commands** | Non-blocking `bash()` with live handles |
| **Independent children** | Spawn multiple, end turn, receive results via messages |

### 8.3 Resource Management

```bash
# Bounded autonomous mode
--autonomous-max-continuations 10
--autonomous-max-turns 40
--autonomous-max-tokens 500000
--autonomous-timeout-ms 1800000

# Gate-retry limits
--autonomous-gate-retries 2
--autonomous-gate-timeout-ms 300000
```

### 8.4 Daemon Efficiency

- **Lease-based lifecycle**: Workers hold leases; supervisor reclaims abandoned workers
- **Session JSONL**: Flat file append for transcripts (no database overhead)
- **Catalog process**: Separate process for saved-session scanning (non-blocking)
- **Backpressure**: Supervisor manages load across workers

---

## 9. Key Patterns

### 9.1 Pattern: Single Tool Surface

**Prime Agent**: One `ipython` tool replaces all built-in tools. All capabilities compose through Python.

**Applicability**: Reduces model cognitive load; enables arbitrary composition without tool proliferation.

### 9.2 Pattern: Programmatic Subagents

**Prime Agent**: `rlm.spawn()` creates real child agents with independent contexts, not just prompt wrappers.

**Applicability**: True parallelism, independent state, explicit message passing between agents.

### 9.3 Pattern: Daemon-Backed Continuity

**Prime Agent**: Sessions survive terminal detach. Workers own state; clients are transient.

**Applicability**: Long-running tasks, background processing, reattachment after disconnect.

### 9.4 Pattern: Continual Harness Refinement

**Prime Agent**: `/refine` applies small, evidence-backed updates to supplemental state. Never rewrites base system prompt. Snapshots support rollback.

**Applicability**: Self-improvement without catastrophic forgetting; auditable evolution.

### 9.5 Pattern: Typed Host Bridge

**Prime Agent**: Python kernel calls `rlm.host_request()` for operations whose authoritative state belongs outside the kernel (credentials, providers, transcripts, scheduling).

**Applicability**: Clean boundary between model-facing code and system state.

### 9.6 Pattern: Progressive Skill Loading

**Prime Agent**: Only skill metadata in startup prompt; full instructions load on-demand when task matches.

**Applicability**: Scales to hundreds of skills without context window pollution.

### 9.7 Pattern: Quality-Gated Autonomy

**Prime Agent**: Autonomous mode runs within budgets + user-defined quality gates. Gates must pass before run can finish.

**Applicability**: Bounded autonomy with verification; prevents premature completion.

---

## 10. Application to Game Development

### 10.1 NPC AI Agent Architecture

| Prime Agent Pattern | Game Dev Application |
|---------------------|----------------------|
| **RLM (persistent REPL)** | NPC with persistent state machine; Python-like scripting per NPC |
| **Daemon-backed workers** | Each NPC runs as background agent; survives scene transitions |
| **Recursive subagents** | NPCs spawn specialist sub-agents (navigation, combat, dialogue) |
| **Agent-to-agent messaging** | Direct NPC communication without global event bus |
| **Heartbeats/schedules** | NPC patrol schedules, periodic behavior triggers |
| **Persistent goals** | NPC long-term objectives (find item, reach location) |
| **Quality gates** | Behavior validation before committing action |

### 10.2 Game World Simulation

```
Game World Supervisor (like Daemon Supervisor)
├── NPC Worker A (persistent agent)
│   ├── AgentSession (behavior state)
│   ├── Python Kernel (scripting environment)
│   ├── Scheduler (patrol, combat, idle cycles)
│   └── Sub-agents (pathfinding, inventory, combat)
├── NPC Worker B
├── NPC Worker C
├── Event Worker (event processing)
└── World State Persistence (JSONL-like)
```

### 10.3 Skill System for Game Logic

```
combat-skill/
├── SKILL.md              # When to use combat behavior
├── pyproject.toml
├── src/
│   └── combat/
│       ├── __init__.py   # attack(), defend(), retreat()
│       └── tactics.py
├── scripts/
│   └── pathfinding.sh
└── references/
    └── enemy-patterns.md
```

### 10.4 Compilation to Rust/NT-*

| Prime Agent Concept | NT-* Equivalent |
|---------------------|-----------------|
| `AgentSession` | `AgentRuntime` in `nt_act` |
| `Python Kernel` | Skill execution context in `nt_mind` |
| `rlm.spawn()` | `CapabilityRouter::spawn_child()` in `nt_core` |
| `agent_message.send()` | `EventBus::emit()` + `SelectiveState` in `nt_core` |
| `Continual Harness` | `experience-tree` + KB `experience` namespace in `nt_memory` |
| `Scheduler` | `HeartbeatAggregator` in `nt_core` |
| `Daemon Supervisor` | `ConsciousnessTree` supervisor in `nt_meta` |
| Skills | Capability nodes in `CapabilityTree` |
| Compaction | `nt_memory` context management |
| Quality Gates | `SelfTest` + `converge_check` |

### 10.5 Direct Transferable Patterns

1. **Single Tool Surface** → Reduce NT-* tool count; compose through unified execution context
2. **Daemon-Backed Workers** → NT-* modules survive session detach; persistent background loops
3. **Progressive Disclosure** → Skill metadata always loaded; full instructions on-demand
4. **Typed Host Bridge** → Clean separation between agent-facing code and system state
5. **Quality-Gated Autonomy** → Bounded autonomous execution with verification gates

---

## Appendix: File Structure Reference

```
prime-agent/
├── packages/
│   └── coding-agent/
│       └── docs/
│           ├── architecture.md      # System architecture
│           ├── rlm.md              # RLM programming model
│           ├── long-running-agents.md  # Background/daemon agents
│           ├── skills.md           # Skills system
│           ├── usage.md            # CLI and interactive usage
│           ├── quickstart.md       # Getting started
│           ├── json.md             # JSON mode (headless)
│           ├── rpc.md              # RPC mode (integrations)
│           ├── providers.md        # Model providers
│           ├── sessions.md         # Session management
│           ├── compaction.md       # Context compaction
│           ├── keybindings.md      # Keyboard shortcuts
│           ├── settings.md         # Configuration
│           └── packages.md         # Package management
├── prime-agent-runtime/           # Python runtime package
├── scripts/                       # Build/install scripts
└── AGENTS.md                      # Repository instructions
```
