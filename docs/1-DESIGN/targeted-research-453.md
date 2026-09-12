# Targeted Research #453 — Internal Pain Point Scan (WISER Iteration)

**Date:** 2026-09-12
**Method:** Grep `TODO/FIXME/HACK/XXX/todo!()/unimplemented!()/#[allow(dead_code)]` in `neotrix-core/src/`, then read context to find concrete broken/stubbed implementations.
**Scope:** neotrix-core/src/ only. Previous batches (18 fixes) excluded.

---

## Pain Point 1: McpRegistry — Complete No-Op Stub (PTC Dead)

| Field | Value |
|-------|-------|
| **File** | `cli/commands/agent_cmds.rs:48-59` |
| **Severity** | **P0** |
| **What** | The entire `McpRegistry` struct is a stub: every method returns `None`, `Vec::new()`, or `0`. The real module import is commented out (line 10: `// use crate::agent::tool::mcp::{McpRegistry, McpDiscovery};`). This means: `/mcp stubs` always reports 0 signatures, `/mcp exec` always returns empty results, `/mcp gateway()` always returns `None`, `as_native_tools()` returns empty vec. The PTC (Programmatic Tool Calling) pipeline is completely non-functional — the planning gate works (`ProgrammaticPlanner`) but execution always yields nothing. |
| **Impact** | Users running `/mcp stubs` or `/mcp exec` get silently empty output. No error, no warning — just zero results. |

**Fix (5-10 lines):**

```rust
// Replace stub McpRegistry with delegation to the real registry.
// In agent_cmds.rs, replace lines 48-59:

pub struct McpRegistry;
impl McpRegistry {
    pub fn new() -> Self { Self }
    pub fn gateway(&self) -> Option<String> {
        // Delegate to the real MCP registry singleton
        get_mcp_registry().blocking_read().gateway_url.clone()
    }
    pub fn list_tools(&self) -> Vec<McpToolInfo> {
        get_mcp_registry().blocking_read().list_tools()
            .into_iter().map(|t| McpToolInfo { name: t.name, description: t.description, server_name: t.server_name }).collect()
    }
    // ... same pattern for search/publish/tool_count
}
```

---

## Pain Point 2: NT-GOVERNANCE Human Oversight SelfTest — Empty Registration

| Field | Value |
|-------|-------|
| **File** | `l6_meta/coordination/nt_governance/mod.rs:10-13` |
| **Severity** | **P1** |
| **What** | `register_human_oversight_self_tests()` is called from `nt_core_self_test_integration.rs:179` during SelfTest registry initialization, but the function body is empty (just a TODO comment). The entire NT-GOVERNANCE domain has **zero self-tests** registered. This means: the "human oversight governance affordance" absorbed from `trailofbits/skills` has no T1/T2/T3 coverage. The ConsciousnessTree governance branch is a dead leaf — it claims to exist but never reports health. |
| **Impact** | Governance violations (policy drift, oversight atrophy) go undetected. The SelfTest scan reports 0 governance tests, but nobody notices because the function silently succeeds. |

**Fix (5-10 lines):**

```rust
// nt_governance/mod.rs — replace empty function body:

pub fn register_human_oversight_self_tests(registry: &mut crate::core::nt_core_self_test::SelfTestRegistry) {
    registry.register(Box::new(HumanOversightSelfTest));
    registry.register(Box::new(PolicyDriftDetector));
}

struct HumanOversightSelfTest;
impl crate::core::nt_core_self_test::SelfTest for HumanOversightSelfTest {
    fn name(&self) -> &str { "GOV-HO-001" }
    fn evaluate(&self) -> crate::core::nt_core_self_test::SelfTestResult {
        // Check that governance policy files exist and are non-empty
        let policy_dir = std::path::Path::new("governance/policies");
        let has_policies = policy_dir.exists() && std::fs::read_dir(policy_dir).map(|d| d.count() > 0).unwrap_or(false);
        crate::core::nt_core_self_test::SelfTestResult {
            passed: has_policies,
            message: if has_policies { "Governance policies present".into() } else { "GOV: No governance policies found".into() },
        }
    }
}
```

---

## Pain Point 3: IterationAgent take_snapshot — Hardcoded Fake Phi/Coherence

| Field | Value |
|-------|-------|
| **File** | `l5_cognition/nt_core/nt_consciousness_core/agent.rs:270-276` |
| **Severity** | **P1** |
| **What** | `take_snapshot()` creates `StateSnapshot` with hardcoded deterministic values: `phi = 0.5 + (cycle * 0.001).min(0.5)` and `coherence = 0.5 + (cycle * 0.0005).min(0.5)`. These are **not real system metrics** — they're fake monotonic curves that always increase. The iteration agent's entire gap-detection loop operates on fabricated health data. Combined with `apply_patches` (line 320: `// TODO: 实际应用补丁`) which only marks gaps as fixed without actually applying code changes, the consciousness core's self-healing loop is a no-op that generates fake progress reports. |
| **Impact** | The "ConsciousnessTree" reports improving health every cycle regardless of actual system state. False confidence; real degradation invisible. |

**Fix (5-10 lines):**

```rust
// agent.rs:270-276 — replace hardcoded snapshot with real metrics:

fn take_snapshot(&mut self) {
    // Read actual system health from HeartbeatAggregator
    let heartbeat = crate::core::nt_core_heartbeat::HeartbeatAggregator::global();
    let health = heartbeat.snapshot();
    self.state_snapshot = StateSnapshot::new(
        self.cycle,
        health.phi_score,           // real IIT phi
        health.coherence_score,     // real GWT coherence
    );
}
```

---

## Summary

| # | Pain Point | Severity | File:Line | Fix Effort |
|---|-----------|----------|-----------|------------|
| 1 | McpRegistry stub — PTC pipeline dead | **P0** | `agent_cmds.rs:48-59` | Medium (wire to real registry) |
| 2 | NT-GOVERNANCE self-test empty | **P1** | `nt_governance/mod.rs:10-13` | Small (implement 1-2 SelfTests) |
| 3 | IterationAgent hardcoded phi/coherence | **P1** | `agent.rs:270-276` | Small (wire to HeartbeatAggregator) |

**Scan stats:** 12 `todo!()/unimplemented!()` macros found, 100+ TODO/FIXME comments, 96 `#[allow(dead_code)]` annotations. Most TODOs are intentional placeholders (shield tools, vtuber integrations). These 3 are the ones that cause **silent incorrect behavior** rather than just missing features.
