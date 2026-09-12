# Targeted Research #519 — Internal Pain Points (Iteration Loop)

**Date**: 2026-09-13
**Scope**: neotrix-core/src/ — TODO/FIXME stubs, hardcoded returns, dead code paths
**Method**: grep + manual review of 30+ TODO sites, 25+ dead_code annotations

---

## Pain Point 1: PTC Stubs & Exec Return Empty Results

**File**: `src/cli/commands/agent_cmds.rs:353-359` and `:467-477`
**Severity**: P1 — Core capability wired but hollow

**What's wrong**: `agent mcp stubs` and `agent mcp exec` both depend on `McpRegistry.gateway()` which is marked `FIXME: not yet implemented`. Both paths silently return empty `Vec<serde_json::Value>`, so the PTC (Programmatic Tool Calling) subsystem compiles but produces zero actual results. Users calling `/mcp stubs` get "0 typed signatures" with no error — a silent failure.

**Code sketch**:
```rust
// agent_cmds.rs:353-359 — currently:
// FIXME: McpRegistry.gateway() not yet implemented
let stubs: Vec<serde_json::Value> = Vec::new();

// Fix: route through registry's registered servers
let stubs: Vec<serde_json::Value> = registry
    .servers()
    .iter()
    .flat_map(|s| s.tool_schemas.iter().map(|t| {
        serde_json::json!({
            "server": s.name,
            "tool": t.name,
            "signature": t.python_stub(),
        })
    }))
    .collect();
```

---

## Pain Point 2: TemporalContinuity Scene/Element Checks Are No-Ops

**File**: `src/l1_action/nt_act/temporal_continuity.rs:218-255`
**Severity**: P1 — Silent validation bypass, production quality gate disabled

**What's wrong**: `check_scene_transition()` and `check_element_position()` always return `passed: true, issues: vec![]` regardless of input. Callers believe continuity checks pass, but no validation actually occurs. The `check_first_last_frame()` at line 134 does real work (character-level diff), proving the infrastructure exists — these two just were never implemented. A downstream `ProductionOrchestrator` relying on these would ship broken video content.

**Code sketch**:
```rust
// temporal_continuity.rs:223 — currently:
// TODO: 实际调用场景转场检查逻辑
ContinuityCheckResult { passed: true, .. }

// Fix: apply frame-diff heuristic to detect hard cuts vs smooth transitions
fn check_scene_transition(&self, frames: &[String], transition_type: &str) -> ContinuityCheckResult {
    let threshold = match transition_type {
        "fade" | "dissolve" => 0.15,
        "cut" => 0.6,
        _ => 0.3,
    };
    let diffs: Vec<f32> = frames.windows(2)
        .map(|w| self.calculate_frame_diff(&w[0], &w[1]))
        .collect();
    let issues = diffs.iter().enumerate()
        .filter(|(_, d)| **d > threshold)
        .map(|(i, d)| ContinuityIssue { /* ... */ })
        .collect();
    // ...build result
}
```

---

## Pain Point 3: L2 Disk Cache Never Reads — Bloom Filter Missing

**File**: `src/l1_action/nt_act/actions/core/nt_act_cache.rs:194-201`
**Severity**: P2 — Cache penetration path dead, performance degraded under L1 pressure

**What's wrong**: When an L1 (in-memory) cache miss occurs, the code checks `if let Some(ref mut _disk_cache)` but the body is empty (`// TODO: 实际从磁盘读取`). The L2 disk cache field exists but is never populated or read from. Additionally, the penetration protection path (`// TODO: 实现布隆过滤器`) is a no-op — repeated L1 misses for the same hot key always fall through to the backend. Under load, this causes redundant backend calls that the cache was designed to prevent.

**Code sketch**:
```rust
// nt_act_cache.rs:194-201 — currently:
if let Some(ref mut _disk_cache) = self.l2_cache {
    // TODO: 实际从磁盘读取
}
if self.config.penetration_protection {
    // TODO: 实现布隆过滤器
}

// Fix: implement bincode-based disk read + bloom probe
if let Some(ref mut disk_cache) = self.l2_cache {
    if let Ok(Some(entry)) = disk_cache.get::<CacheEntry>(key) {
        if !entry.is_expired() {
            self.l1_cache.insert(key.to_string(), entry.clone());
            self.stats.hits += 1;
            return CacheResult::Hit(entry);
        }
    }
}
if self.config.penetration_protection {
    if self.bloom.check(key) {
        return CacheResult::Miss; // known-miss, skip backend
    }
    self.bloom.insert(key);
}
```

---

## Summary

| # | File:Line | Severity | Issue |
|---|-----------|----------|-------|
| 1 | `agent_cmds.rs:353,467` | P1 | PTC stubs/exec always return empty — gateway() unimplemented |
| 2 | `temporal_continuity.rs:223,243` | P1 | Scene/element checks always pass — no validation occurs |
| 3 | `nt_act_cache.rs:194,200` | P2 | L2 disk cache + bloom filter both stubbed — cache miss path leaks |

All three are "compiled but hollow" — the type system is satisfied but runtime behavior is a no-op or returns hardcoded defaults. This pattern (structural completeness masking behavioral absence) is the #1 recurring pain point in the codebase.
