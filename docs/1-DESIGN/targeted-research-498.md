# Targeted Research #498 — Internal Pain Points

## Pain Point 1: McpRegistry.gateway() Dead Path (P0)

**File:** `neotrix-core/src/cli/commands/agent_cmds.rs:353,467`

**What's wrong:** Two FIXME markers indicate `McpRegistry.gateway()` was never implemented. Both the `stubs` subcommand (PTC stub rendering) and the `exec` subcommand (PTC execution) silently return empty `Vec`s — the entire Programmatic Tool Calling pipeline is a no-op. A user calling `/mcp stubs` or `/mcp exec` gets a success response with 0 results, masking a complete integration failure.

**Severity:** P0 — PTC is a documented feature (CONTEXT.md: absorbed term "PTC"). Empty stubs/exec means the typed-stub tool-calling architecture is non-functional. Every agent relying on PTC gets silently degraded behavior.

**Fix sketch:**

```rust
// agent_cmds.rs — stubs subcommand (line 353)
// Replace: let stubs: Vec<serde_json::Value> = Vec::new();
// With:
let stubs: Vec<serde_json::Value> = registry
    .tools()
    .iter()
    .map(|t| serde_json::json!({
        "name": t.name,
        "signature": t.python_signature(),
        "input_schema": t.input_schema,
    }))
    .collect();
```

```rust
// agent_cmds.rs — exec subcommand (line 467)
// Replace: let results: Vec<serde_json::Value> = Vec::new();
// With:
let results: Vec<serde_json::Value> = plan
    .staged_calls()
    .iter()
    .map(|call| {
        registry.execute(call)
            .map(|r| serde_json::json!({"tool": call.name, "ok": true, "result": r}))
            .unwrap_or_else(|e| serde_json::json!({"tool": call.name, "ok": false, "error": e.to_string()}))
    })
    .collect();
```

---

## Pain Point 2: Checkpoint Persistence Metadata Lying (P1)

**File:** `neotrix-core/src/l1_action/nt_act/actions/checkpoint_persistence.rs:147`

**What's wrong:** `file_size` is hardcoded to `0` in every checkpoint saved. This metadata is persisted and later used for storage accounting, expiration decisions, and cleanup. A zero file size means: (a) the cleanup subsystem thinks checkpoints are empty and may aggressively purge them, (b) any monitoring/dashboard shows wrong disk usage, (c) the `CheckpointMeta` struct's `file_size` field is semantically a lie — it claims the data is saved but has no idea how big it is.

**Severity:** P1 — Silent data corruption in metadata. Checkpoints appear to work but all downstream consumers (monitoring, cleanup, budget tracking) operate on garbage values.

**Fix sketch:**

```rust
// checkpoint_persistence.rs — around line 147
// Before: file_size: 0,
// After:
let serialized = serde_json::to_vec(&state).unwrap_or_default();
let file_size = serialized.len() as u64;

let meta = CheckpointMeta {
    id: checkpoint_id.clone(),
    workflow_id: workflow_id.to_string(),
    stage_name: stage_name.to_string(),
    status: CheckpointStatus::Saved,
    created_at: current_timestamp(),
    expires_at: Some(current_timestamp() + self.config.expiration_secs),
    file_size,  // now accurate
    description: None,
};

// Write the serialized bytes to disk (replaces the later manual serialization)
let file_path = std::path::Path::new(&self.config.storage_path)
    .join(format!("{}.json", checkpoint_id));
std::fs::write(&file_path, &serialized).map_err(|e| e.to_string())?;
```

---

## Pain Point 3: Cache Penetration Protection is Cosmetic (P1)

**File:** `neotrix-core/src/l1_action/nt_act/actions/nt_act_cache.rs:194-201`

**What's wrong:** Two critical cache subsystems are stubbed out:
1. **L2 disk cache** (line 194-196): The `l2_cache` field exists but reads are never performed — every L2 lookup is a guaranteed miss. This defeats the purpose of having a two-level cache (L1 in-memory + L2 on-disk).
2. **Bloom filter penetration protection** (line 200-201): The `penetration_protection` config flag exists and is checked, but the bloom filter is never populated or queried. With penetration protection "enabled", a cache miss still goes straight to the origin — the protection is a no-op.

**Severity:** P1 — Users who configure `penetration_protection: true` and `l2_cache` get false confidence. The cache layer advertises features it doesn't implement, leading to unexpected origin load in production.

**Fix sketch:**

```rust
// nt_act_cache.rs — L2 disk cache lookup (line 194-196)
// Replace empty block with:
if let Some(ref mut disk_cache) = self.l2_cache {
    if let Some(entry) = disk_cache.get(&key) {
        self.stats.hits += 1;
        self.update_hit_rate();
        // Promote to L1
        self.l1_cache.insert(key, entry.clone());
        return CacheResult::Hit(entry);
    }
}

// nt_act_cache.rs — bloom filter (line 200-201)
// Replace empty block with:
if self.config.penetration_protection {
    if let Some(ref bloom) = self.bloom_filter {
        if !bloom.check(&key) {
            // Definitively not in cache — skip origin, return Miss immediately
            self.stats.misses += 1;
            self.update_hit_rate();
            return CacheResult::Miss;
        }
        // Might be in cache — proceed to origin (false positive allowed)
    }
}
```

---

## Summary

| # | Pain Point | File:Line | Severity | Impact |
|---|-----------|-----------|----------|--------|
| 1 | McpRegistry.gateway() dead path | `agent_cmds.rs:353,467` | P0 | PTC feature completely non-functional |
| 2 | Checkpoint file_size hardcoded to 0 | `checkpoint_persistence.rs:147` | P1 | Metadata corruption breaks monitoring/cleanup |
| 3 | Cache L2 + bloom filter are stubs | `nt_act_cache.rs:194-201` | P1 | Penetration protection is cosmetic, false confidence |
