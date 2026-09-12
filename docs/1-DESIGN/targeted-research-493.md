# Targeted Research 493 — Internal Pain Points (WISER Iteration 4)

**Date**: 2026-09-12
**Scope**: neotrix-core/src/ — 3 internal pain points found via TODO/FIXME/stub/hardcoded sweep

---

## Pain Point 1: `/kb consistency` returns empty string — dead command

**File**: `cli/commands/kb_cmds.rs:151-159`
**Severity**: P1 (user-facing command silently does nothing)

The `cmd_consistency` function opens the DB, builds `out = String::new()`, then returns it
with no content. The actual `setting_consistency` module import is commented out. A user running
`/kb consistency` gets a blank success response — misleading.

```rust
fn cmd_consistency(_args: &[String]) -> CommandOutput {
    let _conn = match open_raw_conn() {
        Some(c) => c,
        None => return CommandOutput::err("无法打开知识库 ~/.neotrix/knowledge.db"),
    };
    let out = String::new();
    // TODO: setting_consistency module not found; stub
    CommandOutput::ok(&out)  // ← always blank
}
```

**Fix sketch**:

```rust
fn cmd_consistency(_args: &[String]) -> CommandOutput {
    let conn = match open_raw_conn() {
        Some(c) => c,
        None => return CommandOutput::err("无法打开知识库 ~/.neotrix/knowledge.db"),
    };
    let mut out = String::new();
    if let Err(e) = crate::l1_action::nt_memory::nt_memory_kb::setting_consistency::check_and_report_to_string(&conn, &mut out) {
        return CommandOutput::err(&format!("设定一致性检查失败: {e}"));
    }
    if out.is_empty() {
        out = "✅ 设定一致性检查通过，无冲突".into();
    }
    CommandOutput::ok(&out)
}
```

If the module truly doesn't exist, the function should return `CommandOutput::err("setting_consistency 未实现")`
instead of silently succeeding with empty output.

---

## Pain Point 2: `check_region_health` always marks region Available — no real check

**File**: `l1_action/nt_act/actions/multi_region_scheduler.rs:190-199`
**Severity**: P0 (scheduler trusts fake health data, routes to dead regions)

`_check_region_health` unconditionally sets `RegionStatus::Available` regardless of actual
region state. The multi-region scheduler uses this to decide routing — a dead region will
never be marked unhealthy, causing requests to fail.

```rust
pub(crate) fn _check_region_health(&mut self, region_id: &str) -> bool {
    if let Some(region) = self.regions.get_mut(region_id) {
        // TODO: 实际的健康检查逻辑
        region.status = RegionStatus::Available;  // ← always optimistic
        true
    } else {
        false
    }
}
```

**Fix sketch**:

```rust
pub(crate) fn _check_region_health(&mut self, region_id: &str) -> bool {
    if let Some(region) = self.regions.get_mut(region_id) {
        let healthy = tokio::runtime::Handle::current().block_on(async {
            crate::l1_action::nt_io::health_probe::ping_region(
                &region.endpoint,
                std::time::Duration::from_secs(5),
            ).await
        });
        region.status = if healthy {
            RegionStatus::Available
        } else {
            RegionStatus::Degraded
        };
        region.last_health_check = Some(std::time::Instant::now());
        healthy
    } else {
        false
    }
}
```

Even without a full probe, the minimum viable fix is: if `health_probe` module doesn't exist,
at minimum check if the region endpoint is reachable via a TCP connect with timeout, rather
than unconditionally returning `Available`.

---

## Pain Point 3: `McpRegistry.gateway()` returns empty results — PTC stubs/exec are no-ops

**File**: `cli/commands/agent_cmds.rs:353-359` (stubs) and `agent_cmds.rs:467-477` (exec)
**Severity**: P0 (PTC pipeline silently returns 0 results, user gets fake success)

Both `agent mcp stubs` and `agent mcp exec` build an empty `Vec::new()` and format a
success message saying 0 calls were executed. The `FIXME` comment acknowledges
`McpRegistry.gateway()` is not yet implemented. This means the entire programmatic tool
calling pipeline is a dead end.

```rust
// stubs (line 353-354)
// FIXME: McpRegistry.gateway() not yet implemented
let stubs: Vec<serde_json::Value> = Vec::new();

// exec (line 467-468)
// FIXME: McpRegistry.gateway() not yet implemented
let results: Vec<serde_json::Value> = Vec::new();
```

**Fix sketch**:

```rust
// In stubs handler — return error instead of empty success
let stubs = match crate::l1_action::nt_io::nt_io_mcp_registry::McpRegistry::gateway_stubs() {
    Ok(s) => s,
    Err(e) => return CommandOutput::err(&format!("PTC stubs unavailable: {e}")),
};

// In exec handler — return error instead of empty success
let results = match crate::l1_action::nt_io::nt_io_mcp_registry::McpRegistry::gateway_execute(&calls).await {
    Ok(r) => r,
    Err(e) => return CommandOutput::err(&format!("PTC exec failed: {e}")),
};
```

If the gateway truly cannot be implemented yet, these commands should return
`CommandOutput::err("PTC gateway 未实现")` so callers know the pipeline is unavailable,
rather than returning a fabricated "0 calls executed" success message.

---

## Summary

| # | File:Line | Severity | Issue |
|---|-----------|----------|-------|
| 1 | `kb_cmds.rs:157` | P1 | `/kb consistency` returns empty string, silently succeeds |
| 2 | `multi_region_scheduler.rs:193` | P0 | Health check always returns Available, no real probe |
| 3 | `agent_cmds.rs:353,467` | P0 | PTC stubs/exec return empty Vec as success, gateway not wired |

### Recommendation

- **P0 items (2,3)**: Convert silent empty success to explicit error. Callers must know the capability is unavailable.
- **P1 item (1)**: Either wire the `setting_consistency` module or return a clear "not implemented" error.
