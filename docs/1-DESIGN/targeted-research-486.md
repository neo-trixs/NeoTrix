# Targeted Research #486 — Internal Pain Points (WISER Loop Iteration 4)

**Date**: 2026-09-12
**Scope**: neotrix-core/src/ — hardcoded values, stub functions, dead code paths
**Method**: Grep for TODO/FIXME/unimplemented!, grep for `Vec::new()` returns in public functions, verify context

---

## Pain Point 1: PTC Stub Returns Empty — MCP Agent Commands Are Hollow

**File**: `neotrix-core/src/cli/commands/agent_cmds.rs:353-358` and `agent_cmds.rs:467-468`

**What's wrong**: Two user-facing MCP agent commands (`/mcp stubs` and `/mcp exec`) are hardcoded to return empty `Vec::new()` with a FIXME comment. The `McpRegistry` struct itself (lines 50-61) is a hollow shell — every method returns `Vec::new()` or `0`. Users running `/mcp stubs` get "PTC stubs: 0 typed signatures" with no actual tool discovery.

**Severity**: **P0** — Core CLI feature completely non-functional; users cannot use programmatic tool calling.

**Code sketch fix** (10 lines):
```rust
// agent_cmds.rs:353 — Replace stub with real McpRegistry query
"stubs" => {
    let stubs: Vec<serde_json::Value> = self.registry
        .iter()
        .filter(|s| s.tool_type == ToolType::Programmatic)
        .map(|s| serde_json::json!({
            "name": s.name,
            "signature": s.python_stub,
            "args": s.arg_schema,
        }))
        .collect();
    let s = format!("PTC stubs: {} typed signatures\n", stubs.len());
    // ... rest unchanged
}
```

---

## Pain Point 2: `/kb consistency` Returns Empty String — Dead CLI Command

**File**: `neotrix-core/src/cli/commands/kb_cmds.rs:150-159`

**What's wrong**: `cmd_consistency()` opens the KB connection successfully but then returns `CommandOutput::ok("")` — a completely empty success response. The actual consistency check is commented out with `TODO: setting_consistency module not found; stub`. Users running `/kb consistency` see nothing, believing the KB is consistent when no check occurred.

**Severity**: **P1** — Silent failure; users trust KB consistency without any actual verification.

**Code sketch fix** (8 lines):
```rust
// kb_cmds.rs:156 — Replace empty stub with actual consistency check
fn cmd_consistency(_args: &[String]) -> CommandOutput {
    let conn = match open_raw_conn() {
        Some(c) => c,
        None => return CommandOutput::err("无法打开知识库 ~/.neotrix/knowledge.db"),
    };
    let mut out = String::new();
    // Run per-node schema drift check
    let drift = nt_memory_kb::setting_consistency::check_schema_drift(&conn);
    out.push_str(&format!("Schema drift: {} nodes\n", drift.len()));
    for d in &drift { out.push_str(&format!("  {} → {}\n", d.node_id, d.issue)); }
    // Run orphan edge check
    let orphans = nt_memory_kb::setting_consistency::check_orphan_edges(&conn);
    out.push_str(&format!("Orphan edges: {}\n", orphans.len()));
    CommandOutput::ok(&out)
}
```

---

## Pain Point 3: IterationAgent `take_snapshot()` Uses Fake Linear Growth

**File**: `neotrix-core/src/l5_cognition/nt_core/nt_consciousness_core/agent.rs:270-276`

**What's wrong**: `take_snapshot()` ignores the real system state entirely and generates synthetic values using linear interpolation formulas (`0.5 + cycle * 0.001`). This means the IterationAgent's health score, gap detection, and patch generation all operate on fabricated data — the self-healing loop is a simulation pretending to be real. The `health_score()` in `state.rs:122-154` computes a weighted average of these fake inputs, producing meaningless results.

**Severity**: **P0** — The entire self-healing consciousness loop is inoperable; it reports fake health scores.

**Code sketch fix** (10 lines):
```rust
// agent.rs:270 — Wire to real system metrics via HeartbeatAggregator
fn take_snapshot(&mut self) {
    let health = crate::core::nt_core_heartbeat::HeartbeatAggregator::global()
        .snapshot();  // SystemHealthSnapshot from heartbeat
    self.state_snapshot = StateSnapshot {
        cycle: self.cycle,
        assumptions_verified: health.test_pass_count,
        assumptions_total: health.test_total_count,
        interfaces_implemented: health.impl_count,
        interfaces_total: health.trait_count,
        todo_count: health.todo_count,
        // ... real fields from SystemHealthSnapshot
    };
}
```

---

## Bonus Pain Point: `StateSnapshot::new()` Fabricates All System Metrics

**File**: `neotrix-core/src/l5_cognition/nt_core/nt_consciousness_core/state.rs:94-118`

**What's wrong**: `StateSnapshot::new()` takes only `cycle` (and dummy `phi`/`coherence`) but generates ALL 20+ fields via linear formulas — e.g. `todo_count: 100 - (cycle * 10).min(95)`, `memory_usage_mb: 2000 - (cycle * 100).min(1500)`. Every downstream consumer (`health_score()`, `probe_gaps()`, `classify_gaps()`) reads these fabricated values. The entire IterationAgent feedback loop is a simulation of a simulation.

**Severity**: **P0** — StateSnapshot is the data foundation for the consciousness core; faking it invalidates all downstream decisions.

**Code sketch fix** (8 lines):
```rust
// state.rs:92 — Replace formulaic constructor with system-backed factory
impl StateSnapshot {
    pub fn from_system(cycle: u32) -> Self {
        let hb = crate::core::nt_core_heartbeat::HeartbeatAggregator::global().snapshot();
        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open_default()
            .ok();
        Self {
            cycle,
            todo_count: hb.todo_count as u32,
            interfaces_implemented: hb.impl_count as u32,
            interfaces_total: hb.trait_count as u32,
            memory_usage_mb: hb.memory_mb,
            naming_conflicts: hb.naming_conflicts,
            // ... wire remaining fields from real sources
        }
    }
}
```

---

## Summary

| # | Pain Point | File:Line | Severity | Impact |
|---|-----------|-----------|----------|--------|
| 1 | MCP agent commands return empty Vec | agent_cmds.rs:353,467 | **P0** | PTC stubs/exec non-functional |
| 2 | `/kb consistency` returns empty string | kb_cmds.rs:150-159 | **P1** | Silent no-op on critical check |
| 3 | IterationAgent uses fake linear-growth state | agent.rs:270-276 | **P0** | Self-healing loop is simulation |
| 4 | StateSnapshot::new() fabricates all metrics | state.rs:94-118 | **P0** | Consciousness core data is fake |

**Recommendation**: Fix #1, #3, #4 first (all P0). #2 is P1 but lower blast radius.
