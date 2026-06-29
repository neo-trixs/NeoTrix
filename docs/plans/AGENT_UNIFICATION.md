# Agent Runtime Unification Plan

> DGM-H inspired: create a bridge layer, not a merge. Preserve all consumers.
> Date: 2026-06-18

## Executive Summary

There are two agent-related directories in `neotrix-core/src/`:

| Directory | Files | Lines | Role | Entry Point |
|-----------|-------|-------|------|-------------|
| `src/agent/` | 29 | ~13,643 | **Runtime execution layer** (tool dispatch, LLM provider, workflow engine, hooks, sub-agent pool) | `pub mod agent;` in `lib.rs` → `neotrix::agent::*` |
| `src/core/nt_core_agent/` | 37 | ~14,294 | **Data model & protocol layer** (message types, A2A, consensus, identity, sandbox rules, orchestrator data) | `pub use nt_core_agent::*` in `core/mod.rs` → `crate::core::nt_core_agent::*` |

**Key insight: These are complementary layers, not true duplicates.** The runtime layer (`agent/`) depends on the data-model layer (`core/nt_core_agent/`), but NOT vice versa — the architectural boundary is correct.

## Step 1: Overlap Analysis

### 1.1 Agent Bus

| Aspect | `agent::agent_bus` | `core::nt_core_agent::bus` |
|--------|-------------------|---------------------------|
| Pattern | **Supervisor-Worker** task queue | **Peer-to-Peer** mailbox |
| Core types | `AgentBus`, `BusTask`, `BusTopic`, `SupervisorAgent`, `WorkerAgent` | `AgentCommunicationBus`, `PeerMailbox`, `MailboxEntry`, `RoutedMessage` |
| Delivery | `publish(topic, msg)` pub/sub | `send(msg)` direct + `broadcast()` |
| Routing | EFC-based worker selection | Prioritized delivery + forward actions |
| Persistence | Age-based cleanup | Compaction-aware (via `Compactable` trait) |

**Verdict: Keep both.** They solve different problems. The supervisor-worker bus handles task orchestration (who does what), the peer-to-peer bus handles message passing (who talks to whom).

**Bridge action:** Let `agent::agent_bus` import `core::nt_core_agent::bus::AgentCommunicationBus` as a downstream transport for cross-agent messages that need routing beyond the local supervisor hierarchy.

### 1.2 SubAgent

| Aspect | `agent::sub_agent` | `core::nt_core_agent::sub_agent` |
|--------|-------------------|--------------------------------|
| Role | **Concurrent execution runtime** | **Data model + isolation strategy** |
| Core types | `SubAgentPool`, `SubAgentVariant`, `SubAgentHandle` | `SubAgentCapability`, `IsolationStrategy`, `AgentCommunicationBus` integration |
| Features | Timeout handling, event streaming, cancel/cancel_all | Capability enum, Cloud/Worktree isolation modes, MCP transport |
| Consumers | Internal to `agent/` | `ConsciousnessIntegration` fields + `modules_agent.rs` handlers |

**Verdict: Keep both.** The `agent/` version provides the execution engine; the `core/` version provides the type definitions used by the consciousness pipeline.

**Bridge action:** `agent::sub_agent` should use `core::nt_core_agent::sub_agent::SubAgentCapability` and `IsolationStrategy` as its configuration types, removing the duplicate definitions in `agent::sub_agent::types`.

### 1.3 AgentTeam / TeamOrchestrator

| Aspect | `agent::team` | `core::nt_core_agent::orchestrator` |
|--------|-------------|------------------------------------|
| Core type | `AgentTeam` (full implementation) | `TeamOrchestrator` (thin registry) |
| Process types | `Sequential`, `Hierarchical`, `Debate`, `Parallel` | None (just tracks team membership) |
| Consumers | `entry/mod.rs`, `goal_loop`, `nt_act_orchestrator` | Consciousness handlers |
| Lines | ~600 | ~400 |

**Verdict: Merge into `agent::team`.** `AgentTeam` has the real execution logic. `TeamOrchestrator` is a thin registry that should delegate to `AgentTeam`.

**Bridge action:** Add `AgentTeam` re-export from `core::nt_core_agent::*` via a thin bridge, so consciousness consumers can use the same type.

### 1.4 Unique to `agent/` (no overlap)

| Module | Role | Consumers |
|--------|------|-----------|
| `executor` | Agent execution loop wrapping ReasoningBrain | `entry/mod.rs` |
| `workflow` | `WorkflowEngine` (route/parallel/loop/repeat) | `entry/headless.rs` |
| `hooks` | `HookRegistry` lifecycle hooks | `entry/`, `cli/` |
| `tool` / `tools` | Tool lifecycle, registry, MCP tools | `nt_act_mcp.rs`, `cli/` |
| `skills` | Skills engine | Internal |
| `playbook` | Playbook execution | Internal |
| `persona` | Agent persona registry | Internal |
| `decoder` | Output decoding | Internal |
| `proxy` | Agent proxy | Internal |
| `blackboard` | Shared state | Internal |
| `cognitive_memory` | Agent cognitive memory | Internal |

### 1.5 Unique to `core/nt_core_agent/` (no overlap)

| Module | Role | Consumers |
|--------|------|-----------|
| `a2a_v12` | A2A protocol v1.2 gRPC interop | Consciousness pipeline |
| `consensus` | Byzantine consensus layer | `consensus_verifier`, `modules_kb` |
| `cdp_session` | Chrome DevTools Protocol session | `modules_agent` |
| `browser_mcp` | Browser MCP bridge | `modules_agent` |
| `identity` | Agent identity management | Internal |
| `permission` | Permission decisions | `modules_core` |
| `sandbox_rules` | Sandbox policy rules | Internal |
| `preview` | Preview engine | `modules_core` |
| `quant_data` | Quantitative data ingestion | `modules_agent` |
| `remote_host` | Remote agent host | `modules_agent` |
| `mcp_intelligence` | MCP intelligence server | `modules_agent` |
| `hyperagent` | HyperAgent arena | `types.rs` (CI fields) |
| `proving_window` | Epoch-level proving periods | Internal |
| `three_role` | Provider/Requester/Validator separation | Internal |
| `dispatch_pipeline` | Handler dispatch pipeline | Internal |

## Step 2: Dependency Graph (Current)

```
entry/mod.rs, cli/, nt_act_orchestrator, goal_loop
    │
    └──► agent/  (runtime layer)
    │       │
    │       └──► core/ (via crate::core::*)  ✓ correct dependency
    │
consciousness pipeline, builder, consensus_verifier
    │
    └──► core/nt_core_agent/  (data model layer)
            │
            └──► core/ (other core modules)  ✓ correct dependency
```

The runtime layer correctly depends on the data model layer. No reverse dependency exists. This is good architecture — we just need a clean bridge.

## Step 3: Bridge Architecture

### 3.1 Create bridge file

**`src/agent/bridge.rs`** — A single file that:
1. Re-exports selected types from `core::nt_core_agent` for use by `agent/` internal modules (replacing direct `crate::core::` imports)
2. Provides conversion functions between `agent::` types and `core::nt_core_agent::` types where they overlap
3. Is the **only** file in `agent/` that imports from `crate::core::nt_core_agent`

```rust
// src/agent/bridge.rs
//! Bridge between agent/ (runtime) and core/nt_core_agent/ (data model).
//! This is the ONLY file in agent/ that should import from core/nt_core_agent.

use crate::core::nt_core_agent;

// ── Re-exports: agent/ modules use these instead of importing core directly ──

// SubAgent data model types used by agent::sub_agent execution engine
pub use nt_core_agent::sub_agent::SubAgentCapability;
pub use nt_core_agent::sub_agent::IsolationStrategy;

// Bus types for cross-layer communication
pub use nt_core_agent::bus::AgentCommunicationBus;
pub use nt_core_agent::bus::BusStats;

// Team orchestration types
pub use nt_core_agent::orchestrator::TeamOrchestrator;

// Message types
pub use nt_core_agent::message::{AgentId, AgentMessage, MessageContent, MessagePriority};

// ── Conversion functions ──

/// Convert an agent::team::AgentTeam into a core::orchestrator::TeamOrchestrator entry
pub fn team_to_orchestrator_entry(/* ... */) -> /* ... */ {
    // ...
}
```

### 3.2 Type unification map

| Current `agent/` type | Current `core/` type | Action |
|----------------------|---------------------|--------|
| `agent::agent_bus::AgentBus` | `core::bus::AgentCommunicationBus` | Keep both, bridge via `agent::bridge` |
| `agent::sub_agent::SubAgentCapability` | `core::sub_agent::SubAgentCapability` | **Deprecate agent version**, re-export core version |
| `agent::sub_agent::SubAgentPool` | (none) | Keep in agent/, use core types for config |
| `agent::team::AgentTeam` | `core::orchestrator::TeamOrchestrator` | **Merge**: AgentTeam becomes primary, TeamOrchestrator delegates |
| `agent::workflow::WorkflowEngine` | (none) | Keep in agent/ only |
| `agent::hooks::HookRegistry` | (none) | Keep in agent/ only |

### 3.3 Deprecation markers

Add `#[deprecated(note = "use crate::core::nt_core_agent::... via agent::bridge")]` to the types in `agent/` that have equivalents in `core/`. This lets existing consumers compile with warnings, not errors.

```rust
// In agent/sub_agent/types.rs
#[deprecated(note = "use agent::bridge::SubAgentCapability instead")]
pub use crate::core::nt_core_agent::sub_agent::SubAgentCapability;
```

## Step 4: Migration Plan (4 Phases)

### Phase 1: Bridge Creation (1 session)
- Create `src/agent/bridge.rs` with re-exports and conversion functions
- No consumer changes yet — pure additive
- **Verification:** `cargo check --lib` passes with 0 new errors

### Phase 2: Internal agent/ migration (1 session)
- Update `src/agent/sub_agent/` to use `bridge::SubAgentCapability` instead of its own definition
- Update `src/agent/agent_bus.rs` to optionally use `AgentCommunicationBus` as downstream transport
- No changes to any consumer outside `src/agent/`
- **Verification:** `cargo check --lib` passes, agent tests pass

### Phase 3: TeamOrchestrator → AgentTeam delegation (1 session)
- Add `AgentTeam` re-export to `core/nt_core_agent/mod.rs` via bridge import
- Make `TeamOrchestrator` store `AgentTeam` instances internally
- Consciousness consumers keep using `core::nt_core_agent::AgentTeam` via re-export
- **Verification:** All tests pass, entry tests pass

### Phase 4: Consumer cleanup (1 session)
- Add deprecation warnings to old type locations in `agent/` mod.rs
- Update `entry/`, `cli/`, `goal_loop/` imports to use `agent::bridge::*` where appropriate
- **Verification:** `cargo build` with 0 errors, `RUSTFLAGS="-D deprecated"` identifies remaining usages

## Step 5: Success Criteria

1. **Zero consumer breakage** — every existing `use neotrix::agent::*` still compiles
2. **Zero reverse dependency** — `core/nt_core_agent/` still does NOT import from `agent/`
3. **Single bridge file** — `agent/bridge.rs` is the ONLY cross-layer import point
4. **Type dedup** — `SubAgentCapability` and `IsolationStrategy` defined in exactly one place (core/)
5. **Team unification** — `AgentTeam` is the single team execution type, `TeamOrchestrator` delegates to it

## Appendix: File Counts and Sizes

```
agent/ (29 items, ~13,643 lines)
├── active modules (19): executor, absorb, adapters, team, workflow, skills,
│   tool, tools, hooks, sub_agent, persona, blackboard, cognitive_memory,
│   step_generator, worktree, playbook, agent_bus, agent_workflow, channel
├── supporting (4): credit_assignment, deps, experience_pool, proxy
├── configuration (1): mod.rs
├── feature-gated (2): decoder, tunnel
└── legacy/renamed (3): self_org, agent_interface, memory_optimizer

core/nt_core_agent/ (37 items, ~14,294 lines)
├── protocol layer (6): bus, message, a2a_v12, consensus, identity, permission
├── data models (10): sub_agent, agent_kind, agent_memory, error, task_list,
│   tool_result, transcript, compaction, proving_window, three_role
├── runtime integration (7): cdp_session, browser_mcp, mcp_intelligence,
│   remote_control, remote_host, lead_agent, orchestrator
├── infrastructure (8): daemon_mode, decent_mem, design_framework,
│   dispatch_pipeline, factor_miner, harness, hyperagent, preview
├── security (2): sandbox_rules, ua_rotation
└── specialized (4): qr_code, quant_data, verify_loop, mod.rs
```
