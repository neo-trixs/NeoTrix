# Targeted Research #463 — Internal Pain Points (WISER Iteration)

**Date:** 2026-09-12
**Scope:** neotrix-core/src/ — TODO/FIXME stubs, dead code, hardcoded returns
**Method:** Grep for TODO/FIXME/HACK/XXX, unimplemented, dead_code, hardcoded empty results

---

## Pain Point 1: `/kb consistency` command returns empty string (user-facing no-op)

**File:** `neotrix-core/src/cli/commands/kb_cmds.rs:157`
**Severity:** P1

### What's wrong

The `/kb consistency` CLI command claims to perform "设定一致性检查 (对标网文每卷设定检查)" but the entire implementation is stubbed out. The function opens the KB connection, does nothing with it, and returns `CommandOutput::ok("")` — an empty success string. Users invoking this command get zero output, silently.

```rust
fn cmd_consistency(_args: &[String]) -> CommandOutput {
    let _conn = match open_raw_conn() {
        Some(c) => c,
        None => return CommandOutput::err("无法打开知识库 ~/.neotrix/knowledge.db"),
    };
    let out = String::new();
    // TODO: setting_consistency module not found; stub
    // let _ = crate::l1_action::nt_memory::nt_memory_kb::setting_consistency::check_and_report_to_string(&conn, &mut out);
    CommandOutput::ok(&out)
}
```

The `setting_consistency` module was removed (or never created), and no replacement was wired. The function silently succeeds with empty output — worst UX for a consistency check.

### Fix sketch

```rust
fn cmd_consistency(_args: &[String]) -> CommandOutput {
    let kb = match open_kb() {
        Some(kb) => kb,
        None => return CommandOutput::err("无法打开知识库 ~/.neotrix/knowledge.db"),
    };
    // Scan all domain namespaces for orphan nodes and broken edges
    let domains = ["nt_core", "nt_mind", "nt_memory", "nt_world", "nt_act", "nt_io", "nt_shield"];
    let mut issues = Vec::new();
    for domain in &domains {
        if let Ok(nodes) = kb.kv_get_all(&format!("domain_{}", domain)) {
            // Check each node has at least one edge connecting it
            for (key, _val) in &nodes {
                if let Ok(edges) = kb.kv_get_all(&format!("edges_{}_{}", domain, key)) {
                    if edges.is_empty() {
                        issues.push(format!("Orphan node: {}/{}", domain, key));
                    }
                }
            }
        }
    }
    if issues.is_empty() {
        CommandOutput::ok("✅ 设定一致性检查通过: 无孤立节点")
    } else {
        CommandOutput::ok(&format!("⚠️ 发现 {} 个一致性问题:\n{}", issues.len(), issues.join("\n")))
    }
}
```

---

## Pain Point 2: PTC (Programmatic Tool Calling) stubs/exec always return empty vec

**File:** `neotrix-core/src/cli/commands/agent_cmds.rs:353,467`
**Severity:** P1

### What's wrong

Both the "stubs" and "exec" subcommands of `/mcp` are completely non-functional. They contain FIXME comments acknowledging `McpRegistry.gateway() not yet implemented`, and both return hardcoded `Vec::new()` — meaning PTC typed-stub tool calling never actually calls any tools.

**Line 353 (stubs):**
```rust
// FIXME: McpRegistry.gateway() not yet implemented
let stubs: Vec<serde_json::Value> = Vec::new();
let s = format!("🐍 PTC stubs: {} typed signatures\n", stubs.len());
```

**Line 467 (exec):**
```rust
// FIXME: McpRegistry.gateway() not yet implemented
let results: Vec<serde_json::Value> = Vec::new();
let s = format!("⚡ PTC exec: {} stage(s), {} call(s)\n", plan.stages(), results.len());
```

The `ProgrammaticPlanner::plan()` at line 462 validates the calls, but execution is a no-op. The entire PTC subsystem (typed-stub tool invocation described in CONTEXT.md as a core absorbed term) is dead.

### Fix sketch

```rust
// In agent_cmds.rs, replace the FIXME block at ~line 467:
"exec" => {
    let planner = ProgrammaticPlanner::new(&registry);
    let plan = match planner.plan(calls) {
        Ok(p) => p,
        Err(e) => return CommandOutput::err(&format!("[exec] 校验失败: {}", e)),
    };
    let gateway = match McpRegistry::gateway() {
        Some(g) => g,
        None => return CommandOutput::err("[exec] MCP gateway not available — no registered servers"),
    };
    let mut results = Vec::new();
    for stage in plan.stages_iter() {
        for call in stage.calls() {
            match gateway.execute(call).await {
                Ok(r) => results.push(r),
                Err(e) => results.push(serde_json::json!({"error": e.to_string()})),
            }
        }
    }
    let s = format!("⚡ PTC exec: {} stage(s), {} call(s)\n", plan.stages(), results.len());
    if want_json {
        return CommandOutput::ok(&s).with_json(serde_json::json!({
            "stages": plan.stages(),
            "count": results.len(),
            "results": results,
        }));
    }
    CommandOutput::ok(&s)
}
```

---

## Pain Point 3: CacheLayer entire module is dead code with L2/bloom filter stubs

**File:** `neotrix-core/src/l1_action/nt_act/actions/nt_act_cache.rs:10,194-201`
**Severity:** P2

### What's wrong

The entire `nt_act_cache` module has `#![allow(dead_code)]` at line 10 and is **never imported** anywhere in the codebase (confirmed via `use.*nt_act_cache` and `use.*CacheLayer` grep — zero matches). The module contains:

1. **L2 disk cache** (line 194-196): `if let Some(ref mut _disk_cache) = self.l2_cache { // TODO: 实际从磁盘读取 }` — the disk cache lookup is a dead code block that never reads from disk.

2. **Bloom filter penetration protection** (line 199-201): `if self.config.penetration_protection { // TODO: 实现布隆过滤器 }` — the bloom filter is never implemented.

3. The `_` prefix on `_disk_cache` confirms the author knew this was unused.

This is 276 lines of dead code with two critical feature gaps (L2 persistence, penetration protection) that will silently fail if anyone tries to use the cache layer.

### Fix sketch

Either wire it into the production path or delete it. Since `DiskCache` and `BloomFilter` are both stubs, the pragmatic fix is to add a `warn!` and remove the dead code flag:

```rust
// nt_act_cache.rs — remove #![allow(dead_code)] and add:
impl CacheLayer {
    pub fn get(&mut self, key: &str) -> CacheResult {
        // L1 memory lookup (working)
        if let Some(entry) = self.l1_cache.get(&key.to_string()) {
            self.stats.hits += 1;
            self.update_hit_rate();
            return CacheResult::Hit(entry);
        }

        // L2 disk lookup — currently disabled, log warning
        if self.config.l2_enabled {
            tracing::warn!("L2 disk cache lookup requested but not implemented — falling through to miss");
        }

        // Penetration protection — currently disabled
        if self.config.penetration_protection {
            tracing::warn!("Bloom filter penetration protection requested but not implemented");
        }

        self.stats.misses += 1;
        self.update_hit_rate();
        CacheResult::Miss
    }
}
```

---

## Summary

| # | File:Line | Issue | Severity | Category |
|---|-----------|-------|----------|----------|
| 1 | `kb_cmds.rs:157` | `/kb consistency` returns empty string — stubbed consistency check | **P1** | Stub / no-op |
| 2 | `agent_cmds.rs:353,467` | PTC stubs/exec always return `Vec::new()` — FIXME gateway not implemented | **P1** | Stub / no-op |
| 3 | `nt_act_cache.rs:10,194-201` | Entire CacheLayer dead code; L2 disk + bloom filter never implemented | **P2** | Dead code / incomplete |

**Already fixed in prior iterations:** 27 issues (FepIitBridge, AgentTeamSelfTest, HeartbeatAggregator, KB search, value_function, content_moderation, checkpoint_persistence, rate_limiter cleanup, temporal_continuity, verifier_agent, quality_control, unified_api, checkpoint cleanup, /kb embed, layered_qa execute_check, embedding search, HighestQuality routing, C2PA watermark, video_stitcher, video_post_processor _align_colors, video_post_processor stabilize, publish_gateway, model_routing, parallel_task backoff, check_registry, reference_generation).
