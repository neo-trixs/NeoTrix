# Targeted Research #479 — Internal Pain Points (Post-27-Fix Iteration)

**Date**: 2026-09-12
**Scope**: neotrix-core/src/ — TODO/FIXME stubs, empty returns, dead code
**Method**: grep for `FIXME|TODO|unimplemented!|todo!` + `vec![]`/`Vec::new()` return patterns

---

## Pain Point 1: PTC Stubs & Exec Return Empty Vecs (FIXME)

**File**: `cli/commands/agent_cmds.rs:353-354` (stubs), `:467-468` (exec)
**Severity**: **P0** — Programmatic Tool Calling is a core P1 capability, both entry points are no-ops

**What's wrong**: The `stubs` subcommand renders `Vec::new()` and `exec` returns `Vec::new()` for results, both gated behind `FIXME: McpRegistry.gateway() not yet implemented`. The PTC pipeline (plan → validate → execute) validates but never executes — users get `0 typed signatures` and `0 call(s)` every time.

**Fix sketch**:
```rust
// agent_cmds.rs:353 — wire stubs to actual registry
// FIXME: McpRegistry.gateway() not yet implemented
let registry = get_mcp_registry();
let registry = registry.blocking_read();
let stubs: Vec<serde_json::Value> = registry.tools().iter().map(|t| {
    serde_json::json!({
        "tool": t.name, "server": t.server_name,
        "signature": format!("{}({})", t.name, t.input_schema.params.join(", ")),
    })
}).collect();

// agent_cmds.rs:467 — wire exec to governed execution path
let executor = ProgrammaticExecutor::new(&registry);
let results: Vec<serde_json::Value> = plan.stages_iter()
    .map(|stage| executor.execute_stage(stage))
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| CommandOutput::err(&format!("[exec] 执行失败: {}", e)))?;
```

---

## Pain Point 2: L2 Disk Cache Never Reads (Dead Code)

**File**: `l1_action/nt_act/actions/nt_act_cache.rs:194-196`
**Severity**: **P1** — L2 disk cache is declared, insert may write, but lookup always falls through to miss

**What's wrong**: The `get()` method checks `self.l2_cache` but the body is `// TODO: 实际从磁盘读取`. L2 cache is structurally present but functionally dead — all L2 lookups silently return `CacheResult::Miss`. The bloom filter for penetration protection (line 200) is also unimplemented, so the miss path doesn't benefit from dedup.

**Fix sketch**:
```rust
// nt_act_cache.rs:194 — implement L2 disk lookup
if let Some(ref mut disk_cache) = self.l2_cache {
    if let Some(entry) = disk_cache.get(key) {
        if !entry.is_expired() {
            // Promote to L1
            self.l1_cache.insert(key.to_string(), entry.clone());
            self.stats.hits += 1;
            self.update_hit_rate();
            return CacheResult::Hit(entry);
        }
    }
}
```

---

## Pain Point 3: Verifier Agent Uses Fake Heuristic Instead of VLM

**File**: `l6_meta/coordination/verifier_agent.rs:195-244`
**Severity**: **P1** — Quality control pipeline produces fabricated verification scores

**What's wrong**: `_verify_shot()` calls `simulate_verification()` which is a keyword-matching heuristic (checks if description contains "character"/"face" etc. and returns hardcoded 6-9 scores). The real VLM call is a TODO. This means the entire quality gate (pass/fail, regeneration decisions) is driven by string matching on the spec description, not actual visual inspection. Downstream `QualityControlPipeline` trusts these scores as real.

**Fix sketch**:
```rust
// verifier_agent.rs:195 — replace simulate with real VLM call
async fn verify_with_vlm(&self, video_path: &str, spec: &str) -> Vec<VerificationScore> {
    let prompt = format!(
        "Compare this video frame to the spec: {}. Score entity_consistency, \
         environment_consistency, narrative_progress, technical_quality each 0-10.",
        spec
    );
    let response = self.llm_provider
        .analyze_video(video_path, &prompt)
        .await
        .unwrap_or_else(|_| self.simulate_verification(spec, None)); // fallback
    parse_vlm_scores(&response)
}
```

---

## Summary

| # | File:Line | Issue | Severity | Category |
|---|-----------|-------|----------|----------|
| 1 | `agent_cmds.rs:353,467` | PTC stubs/exec return empty — FIXME gateway | P0 | Stub |
| 2 | `nt_act_cache.rs:194` | L2 disk cache lookup is dead code | P1 | Dead path |
| 3 | `verifier_agent.rs:195` | VLM verification faked via keyword heuristic | P1 | Hardcoded |

All three are production code paths that silently produce empty/fake results without erroring.
