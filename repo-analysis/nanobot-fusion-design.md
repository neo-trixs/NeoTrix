# Nanobot × NeoTrix GoalLoop — Fusion Design Document

> **Date**: 2026-05-28
> **Sources**: HKUDS/nanobot v0.2.0 (43.3K⭐), NeoTrix `goal_loop/` (4 files, ~1700 lines)
> **Analyst**: NeoTrix Self-Improvement Engine

---

## 1. Project Overview

**Nanobot** is an ultra-lightweight (~4000 lines Python) open-source AI agent framework from HKU Data Science Lab. Its v0.2.0 (2026-05-15) introduced a `/goal` system for sustained objectives, built as a thin metadata layer over the existing agent loop.

**Key design philosophy**: nanobot's goal system is intentionally minimal — no dedicated struct, no sub-agent orchestrator, no circuit breakers. Goals live as JSON in session metadata. The LLM manages the objective via two tools (`long_task` / `complete_goal`), and the goal is mirrored into Runtime Context every turn to survive compaction.

**NeoTrix GoalLoop** is a full-featured goal tracking system in Rust with state machines, budgets, hierarchical plans, circuit breakers, rate limiters, E8 hexagram priority, and Orchestrator/AgentTeam integration.

---

## 2. Core Abstraction Comparison

| Dimension | Nanobot `/goal` | NeoTrix GoalLoop |
|-----------|----------------|-------------------|
| **Language** | Python 3.11+ | Rust |
| **Total lines** | ~400 (tools + command + goal_state.py) | ~1700 (4 files) |
| **Goal storage** | Session metadata JSON blob (`goal_state` key) | `GoalTracker` struct → `goals.json` persistence |
| **State machine** | Implicit: `active` ↔ `completed` (via tool calls) | Explicit: `Pursuing/Paused/Achieved/Unmet/BudgetLimited` |
| **LLM integration** | Goal mirrored into Runtime Context each turn via `goal_state_runtime_lines()` | `continuation_prompt()` / `budget_prompt()` injected as context |
| **Timeout handling** | Wall clock timeout **disabled** while goal active; streaming uses idle timeout only | `max_duration_secs` in `GoalConfig`; hard cut when exceeded |
| **Execution model** | Normal agent loop (same runner, compaction as configured) | `SelfIteratingBrain.run_seal_loop()` + `pursue_iteration()` |
| **Multi-goal queue** | No — single goal per session | Yes — priority-sorted queue with dedup, max_queue=5 |
| **Budget tracking** | None (no cost/token/iteration limits) | 4 dimensions: iterations, cost, duration, tokens |
| **Circuit breaker** | None | Configurable stall/failure trip + cooldown |
| **Rate limiter** | None | Sliding-window (default 100 calls/hour) |
| **Hierarchical plans** | None | Macro→Meso→Micro with skip conditions + reflection triggers |
| **E8 priority** | None | Hexagram hamming distance priority adjustment |
| **CRT time scale** | None | Multi-scale temporal planning (Huntian/etc.) |
| **Motivation rebalance** | None | `MotivationState` → priority adjustment |
| **Auto goal generation** | None | Capability-aware auto goal + diverse candidates |
| **WebUI surfacing** | Goal shown in chat header via WebSocket blob | TBD |
| **Subagent support** | Subagents honor longer budget | Via AgentTeam integration |
| **Distillation** | None | SessionDistiller integration for pattern extraction |
| **Long-goal skill** | Dedicated skill teaching LLM how to write goals | None |
| **Goal continuation** | `SUSTAINED_GOAL_CONTINUE_PROMPT` injected mid-turn | Manual `continuation_prompt()` |
| **Orchestrator** | None | `Orchestrator.run_recursive_loop()` |
| **Memory storage** | Implicit via session history | Explicit `ReasoningBank.store()` on iteration |

---

## 3. Gap Matrix

### What Nanobot Has That NeoTrix Doesn't (P1 Candidates)

| # | Feature | Nanobot Mechanism | NeoTrix Gap | Priority |
|---|---------|-------------------|-------------|----------|
| G1 | **Runtime Context goal pinning** | `goal_state_runtime_lines()` appends goal text to LLM context every turn; survives compaction | GoalLoop only injects prompt at `pursue_iteration()` — if the LLM generates many tool calls, the goal context may be compacted away | **P0** |
| G2 | **LLM wall timeout auto-widening** | `runner_wall_llm_timeout_s()` returns 0.0 (no timeout) when goal active; streaming falls back to idle timeout | `max_duration_secs` is a hard cut — kills long reasoning passes mid-thought | **P1** |
| G3 | **Streaming idle timeout distinction** | Streaming requests use idle timeout (no output = kill) vs wall timeout (total elapsed). Goal widens wall, idle stays | GoalLoop doesn't distinguish; hard timeout kills active streams | **P1** |
| G4 | **WebUI goal header** | `goal_state_ws_blob()` pushes goal to WebSocket; WebUI shows in chat header | No WebUI goal surfacing | **P2** |
| G5 | **Goal-aware mid-turn continuation** | `build_goal_continue_message()` injected when goal active and no tool calls pending — keeps agent on track | Manual continuation prompt only at start of `pursue_iteration()` | **P2** |
| G6 | **Subagent budget inheritance** | Subagents inherit longer timeout when parent has active goal | AgentTeam subagents don't inherit goal budget | **P2** |
| G7 | **Long-goal skill / prompt engineering** | Dedicated skill teaching LLM: idempotent, self-contained, bounded goals with explicit done-ness | No guidance for goal writing; relies on fixed templates | **P3** |
| G8 | **Metadata-only goal tracking** | Goal lives in session metadata — no dedicated persistence path | GoalTracker is a full struct with JSON serialization (heavier) | **P3** |

### What NeoTrix Has That Nanobot Doesn't

| # | Feature | NeoTrix Capability | Nanobot Gap |
|---|---------|-------------------|-------------|
| H1 | **Full state machine** | 5 states + is_terminal() | 2 states (active/completed) |
| H2 | **Multi-goal queue** | Priority-sorted with dedup | Single goal per session |
| H3 | **Budget enforcement** | 4 dimensions with exhaust detection | None |
| H4 | **Circuit breaker** | Stall/failure trip + cooldown | None |
| H5 | **Rate limiter** | Sliding-window (100 calls/h) | None |
| H6 | **Hierarchical plans** | Macro→Meso→Micro with skip/rfl triggers | None |
| H7 | **E8 hexagram priority** | Hamming distance adjustment | None |
| H8 | **CRT time scales** | Multi-scale temporal planning | None |
| H9 | **Motivation rebalance** | `MotivationState` → priority | None |
| H10 | **Auto goal generation** | Capability + memory + iteration aware | None |
| H11 | **Session distillation** | Pattern extraction from history | None |
| H12 | **Orchestrator integration** | `run_recursive_loop()` | None |
| H13 | **ReasoningBank memory** | `store()` on each iteration | Implicit via session history |

---

## 4. Priority Classification

| Priority | Features | Rationale |
|----------|----------|-----------|
| **P0** | G1 — Runtime Context goal pinning | Without this, LLM forgets the goal after context compaction, making the entire GoalLoop pointless |
| **P1** | G2 — LLM timeout widening | Long-running goals are killed mid-thought by hard wall-clock timeout |
| **P1** | G3 — Idle vs wall timeout distinction | Active streaming should not be killed by total elapsed time |
| **P2** | G4 — WebUI goal header | UX improvement for WebUI users |
| **P2** | G5 — Goal-aware mid-turn continuation | Keeps agent on track without re-triggering `pursue_iteration()` |
| **P2** | G6 — Subagent budget inheritance | Team execution consistency |
| **P3** | G7 — Long-goal skill / prompt guide | Documentation improvement |
| **P3** | G8 — Metadata-only lightweight goal | Performance optimization |

---

## 5. Integration Points

### G1: Runtime Context Goal Pinning

**Problem**: `GoalLoop.pursue_iteration()` sets a continuation prompt at the start of each iteration, but if the SelfIteratingBrain's `run_seal_loop()` generates many tool calls, context compaction may drop the goal context.

**Solution**: Add a `goal_context_lines()` method to `GoalTracker` that produces compact goal reminder lines, injected into the Runtime Context (analogous to nanobot's `goal_state_runtime_lines()`). Modify `SelfIteratingBrain` to inject these lines before each LLM call within `run_seal_loop()`.

**Files to change**:
- `goal_loop/tracker.rs` — Add `GoalTracker::runtime_context_lines() → Vec<String>` method
- `goal_loop/loop_impl/pursue.rs` — In `pursue_iteration()`, after `rate_limiter.allow_call()` check, inject context lines into brain's current prompt state
- `self_iterating.rs` — Modify `run_seal_loop()` to accept an optional goal context injection callback
- `mod.rs` — Export new method

**Design**:
```rust
impl GoalTracker {
    pub fn runtime_context_lines(&self, max_objective_chars: usize) -> Vec<String> {
        let mut lines = vec![format!("Active goal: {}", self.state.label())];
        let objective = truncate(&self.description, max_objective_chars);
        lines.push(objective);
        lines.push(format!("Iteration: {}/{}", self.iterations_completed, self.config.max_iterations));
        if self.stalled_count > 0 {
            lines.push(format!("⚠ Stalled: {}x", self.stalled_count));
        }
        lines
    }
}
```

**Test strategy**:
- Unit test: `tracker.runtime_context_lines()` returns correct lines
- Integration test: after 10 iterations of `pursue_iteration()`, verify goal context is still accessible
- Stress test: with compaction simulation, verify goal lines survive

---

### G2: LLM Wall Timeout Widening

**Problem**: `GoalConfig.max_duration_secs` (default 3600) applies a hard wall-clock timeout. Long reasoning passes or deep tool chains may legitimately exceed this.

**Solution**: Add `timeout_widening_factor: f64` to `GoalConfig`. When a goal is actively being pursued, multiply the LLM request timeout by this factor. Add `goal_timeout_s()` method that returns `None` (no timeout) when goal is active, similar to nanobot's `runner_wall_llm_timeout_s()`.

**Files to change**:
- `goal_loop/types.rs` — Add `GoalConfig.timeout_widening_factor: f64` (default 5.0 = 5x widening)
- `goal_loop/loop_impl/core.rs` — Add `GoalLoop::goal_timeout_s(&self) -> Option<f64>` returning `None` when goal active
- `self_iterating.rs` — Use `goal_timeout_s()` instead of hard timeout when goal active

**Test strategy**:
- Unit test: `goal_timeout_s()` returns `None` when `active_goal.is_some()`
- Unit test: `goal_timeout_s()` returns original timeout when no active goal

---

### G3: Idle vs Wall Timeout Distinction

**Problem**: Streaming LLM responses should use idle timeout (no output for N seconds = kill), not wall timeout (total elapsed time). A deep-thinking model may pause between reasoning tokens.

**Solution**: Modify the LLM provider layer to accept two timeout values: `wall_timeout` and `idle_timeout`. When a goal is active, set `wall_timeout = None` (or very large) but keep `idle_timeout` at a reasonable value (e.g. 120s).

**Files to change**:
- `core/` (provider layer) — Add `LlmTimeoutConfig { wall: Option<Duration>, idle: Option<Duration> }`
- `goal_loop/types.rs` — Add `GoalConfig.idle_timeout_secs: u64` (default 120)
- `reasoning_brain/reasoning_engine.rs` — Thread timeout config through to provider

**Test strategy**:
- Integration test: streaming LLM call with active goal does not hard-timeout after `max_duration_secs`
- Integration test: idle timeout still fires after `idle_timeout_secs` of no output

---

### G4: WebUI Goal Header

**Problem**: The WebUI (if any) has no awareness of active goals. Users can't see current goal status in the UI.

**Solution**: Add `goal_state_ws_blob()` equivalent to the `GoalLoop` that serializes a minimal goal state snapshot. The WebSocket / HTTP API endpoints expose this via the goal loop's `status()` method already — the WebUI just needs to read it.

**Files to change**:
- `goal_loop/loop_impl/core.rs` — Add `GoalLoop::ui_blob(&self) -> serde_json::Value` returning `{"active": true, "description": "...", "summary": "...", "progress_pct": 42.0}`
- `mcp_tools.rs` — If WebSocket channel exists, push goal blob on iteration
- `background_loop.rs` — Push goal state on each goal_ticker cycle

**Test strategy**:
- Unit test: `ui_blob()` returns correct JSON structure for active goal
- Unit test: `ui_blob()` returns `{"active": false}` when no goal

---

### G5: Goal-Aware Mid-Turn Continuation

**Problem**: After a tool call returns, the LLM may switch to a different topic or stop. There's no mechanism to gently nudge it back to the goal.

**Solution**: Add `goal_continue_prompt()` to `GoalTracker` that returns a gentle continuation message. After tool results are processed in `run_seal_loop()`, inject this prompt if no explicit completion signal is detected.

**Files to change**:
- `goal_loop/tracker.rs` — Add `GoalTracker::goal_continue_prompt() -> Option<String>` returning the continuation prompt if state is `Pursuing` and not stalled/budgeted
- `self_iterating.rs` — In tool result processing loop, inject continuation prompt if goal is active and agent appears to stray

**Test strategy**:
- Unit test: `goal_continue_prompt()` returns non-empty for `Pursuing` goal
- Unit test: `goal_continue_prompt()` returns `None` for terminal states

---

### G6: Subagent Budget Inheritance

**Problem**: When `GoalLoop` delegates to `AgentTeam`, the subagents use default timeout/budget settings, not the active goal's config.

**Solution**: When starting subagents via `AgentTeam.execute()`, pass the active goal's timeout config as context.

**Files to change**:
- `goal_loop/loop_impl/execution.rs` — In `_run_single_goal()`, pass `goal_timeout_s()` to AgentTeam
- `agent/team.rs` — Accept optional timeout override per task

**Test strategy**:
- Integration test: subagent task inherits widened timeout when parent goal is active

---

### G7: Long-Goal Skill / Prompt Guide

**Problem**: GoalLoop has no guidance for what makes a good goal description.

**Solution**: Write an internal `LONG_GOAL_SKILL_PROMPT` constant in `tracker.rs` that guides goal writing: idempotent, self-contained, bounded, explicit done-ness. Use this in `start_goal()` and `auto_goal_generate()`.

**Files to change**:
- `goal_loop/tracker.rs` — Add `LONG_GOAL_SKILL_PROMPT` const (doc comment)
- `goal_loop/loop_impl/pursue.rs` — Reference in `auto_goal_generate()`

---

### G8: Metadata-Only Lightweight Goal Option

**Problem**: GoalTracker is a heavy struct with full JSON persistence. For simple goals, a lightweight metadata-only mode would be faster.

**Solution**: Add `GoalConfig.lightweight_mode: bool`. When set, skip history tracking, circuit breaker, rate limiter, and complex iteration. Use a simple `GoalState` enum with minimal fields for fast iteration.

**Files to change**:
- `goal_loop/types.rs` — Add `lightweight_mode: bool` to `GoalConfig`
- `goal_loop/loop_impl/pursue.rs` — In `pursue_iteration()`, skip complexity if `lightweight_mode`

---

## 6. KnowledgeSource Registration Scheme

Each gap feature maps to a new or existing `KnowledgeSource` variant for traceability:

| Gap | KnowledgeSource | Capability Vector Impact | Provenance |
|-----|----------------|--------------------------|------------|
| G1 | `RuntimeContextGoal` | `reasoning_coherence: +0.15`, `context_management: +0.20` | nanobot/goal_state.py |
| G2 | `TimeoutWidening` | `execution_robustness: +0.15`, `resource_management: +0.10` | nanobot/goal_state.py |
| G3 | `StreamingTimeout` | `execution_robustness: +0.10` | nanobot/runner.py |
| G4 | `WebUIGoal` | `ui_integration: +0.15` | nanobot/goal_state.py |
| G5 | `GoalContinuation` | `reasoning_coherence: +0.10` | nanobot/runtime.py |
| G6 | `SubagentTimeout` | `coordination: +0.10` | nanobot/runner.py |
| G7 | `GoalSkillPrompt` | `prompt_engineering: +0.15` | nanobot/long_task.py |
| G8 | `LightweightGoal` | `performance: +0.10` | nanobot/long_task.py |

**Registration**: The `absorb()` method in `self_iterating.rs` already handles `KnowledgeSource` variants. Add new variants and map them to `capability_vector()` in `reasoning_brain/core.rs`.

---

## 7. Test Strategy

### Unit Tests (19 new)

| Module | Test | Coverage |
|--------|------|----------|
| `tracker.rs` | `runtime_context_lines()` returns formatted goal lines | G1 |
| `tracker.rs` | `runtime_context_lines()` truncates long descriptions | G1 |
| `tracker.rs` | `runtime_context_lines()` shows stalled count | G1 |
| `tracker.rs` | `runtime_context_lines()` has correct labels per state | G1 |
| `core.rs` | `goal_timeout_s()` returns `None` when goal active | G2 |
| `core.rs` | `goal_timeout_s()` returns default when no goal | G2 |
| `core.rs` | `ui_blob()` returns `active: true` for Pursuing | G4 |
| `core.rs` | `ui_blob()` returns `active: false` when no goal | G4 |
| `core.rs` | `ui_blob()` includes progress_pct | G4 |
| `tracker.rs` | `goal_continue_prompt()` returns prompt for Pursuing | G5 |
| `tracker.rs` | `goal_continue_prompt()` returns None for terminal | G5 |
| `tracker.rs` | Config `lightweight_mode` defaults to false | G8 |
| `types.rs` | `timeout_widening_factor` defaults to 5.0 | G2 |
| `types.rs` | `idle_timeout_secs` defaults to 120 | G3 |

### Integration Tests (7 new)

| Test | Gap | Scenario |
|------|-----|----------|
| Goal context survives 50 tool calls | G1 | Simulate compaction, verify goal context lines remain in brain state |
| Goal active with timeout widening no hard cut | G2 | With `max_duration_secs=5`, simulate a 10s LLM call — should succeed |
| Goal continuation prompt after tool results | G5 | After tool result, verify continuation prompt was injected |
| Subagent inherits goal timeout | G6 | Via AgentTeam, verify subagent's timeout is widened |
| Lightweight goal skips history | G8 | Verify no history entries in lightweight mode |
| Idle timeout still fires during goal | G3 | No LLM output for 200s, verify it times out (not hard wall) |
| WebUI blob pushes on iteration | G4 | MCP tool receives goal state WebSocket event after `pursue_iteration()` |

### Runtime Context Injection Integration Test (G1)

```
1. Start a complex goal "design a login page and implement backend"
2. Run 5 iterations with 10 tool calls each (simulate heavy tool usage)
3. On each iteration, verify the goal context lines appear in the LLM prompt
4. Verify the goal was NOT compacted away
```

### Timeout Widening Stress Test (G2+G3)

```
1. Start goal with max_duration_secs=5
2. Set timeout_widening_factor=10.0
3. Issue a streaming LLM call that takes 30 seconds to complete
4. Verify: call succeeds (30s < 50s widened budget)
5. Issue another call with no output for 300s
6. Verify: call fails due to idle timeout (120s idle budget)
```

---

## 8. Implementation Order

```
Phase 1 (P0):
  ├── G1: Runtime Context goal pinning — GoalTracker.runtime_context_lines()
  └── G1: SelfIteratingBrain integration — inject before each LLM call

Phase 2 (P1):
  ├── G2: LLM timeout widening — timeout_widening_factor in GoalConfig
  └── G3: Idle vs wall timeout — LlmTimeoutConfig, idle_timeout_secs

Phase 3 (P2):
  ├── G4: WebUI goal blob — GoalLoop.ui_blob()
  ├── G5: Goal continuation prompt — GoalTracker.goal_continue_prompt()
  └── G6: Subagent budget inheritance — AgentTeam timeout override

Phase 4 (P3):
  ├── G7: Long-goal skill prompt — LONG_GOAL_SKILL_PROMPT const
  └── G8: Lightweight mode — GoalConfig.lightweight_mode
```

---

## 9. Key Design Decisions

1. **Nanobot's approach is fundamentally correct** for G1: mirror goal state into Runtime Context every turn. This is the single most impactful feature to port because it solves "agent forgets the goal after compaction" — which is the #1 failure mode of long-running goals.

2. **NeoTrix should NOT adopt nanobot's metadata-only approach** for goal storage. The `GoalTracker` struct is justified because NeoTrix has budgets, circuit breakers, and multi-goal queuing. The lightweight mode (G8) is optional for simple goals.

3. **Timeout model should be dual** — wall timeout for non-streaming (fast failures), idle timeout for streaming (respects thinking time). This is a more nuanced design than nanobot's "disable wall timeout entirely" approach.

4. **Goal-aware continuation** (G5) should be conservative — only inject when the agent appears to have stopped producing goal-relevant output, detected by stall count or topic drift analysis.

5. **KnowledgeSource registration** ensures each absorbed nanobot feature is traceable and can be rolled back independently.
