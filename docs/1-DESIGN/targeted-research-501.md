# Targeted Research 501 — Internal Pain Points (Iteration 3)

> **WISER**: 3 concrete pain points found after 27 fixes. Each is a hardcoded stub or dead code path.

---

## Pain Point 1: MCP PTC stubs/exec return empty Vec (P0)

**File**: `neotrix-core/src/cli/commands/agent_cmds.rs:354,467`

**What's wrong**: `/mcp stubs` and `/mcp exec` both build an empty `Vec` and return `0 count`. The `FIXME` says `McpRegistry.gateway()` is not yet implemented, so the entire programmatic tool calling (PTC) pipeline is a no-op. Users calling `/mcp exec git|{"cmd":"status"}` get `0 results` — silently succeeds with zero work done.

**Severity**: **P0** — core PTC feature is completely dead code; callers get misleading "success" with no output.

**Fix sketch**:
```rust
// agent_cmds.rs:353-354 — replace empty stubs with real gateway call
// BEFORE:
//   FIXME: McpRegistry.gateway() not yet implemented
//   let stubs: Vec<serde_json::Value> = Vec::new();
// AFTER:
let gateway = registry.gateway(); // or registry.as_native_tools()
let stubs: Vec<serde_json::Value> = gateway.iter().map(|tool| {
    serde_json::json!({
        "name": tool.name(),
        "signature": tool.type_signature(),
        "server": tool.server_name(),
    })
}).collect();

// agent_cmds.rs:467-468 — replace empty exec results
// BEFORE:
//   FIXME: McpRegistry.gateway() not yet implemented
//   let results: Vec<serde_json::Value> = Vec::new();
// AFTER:
let executor = registry.executor(); // gateway execution handle
let results: Vec<serde_json::Value> = plan.stages().iter().flat_map(|stage| {
    stage.calls.iter().filter_map(|call| {
        executor.execute(&call.tool, &call.args).ok().map(|r| r)
    }).collect::<Vec<_>>()
}).collect();
```

---

## Pain Point 2: Reference generation methods return fabricated results (P1)

**File**: `neotrix-core/src/l1_action/nt_io/reference_generation.rs:225-268`

**What's wrong**: `video_to_video()`, `image_to_video()`, and `style_transfer()` all return hardcoded `GenerationResult` with fabricated metrics (`reference_similarity: 0.89`, `quality_score: 0.86`, `success: true`) without calling any actual model. The file is referenced in production capability routing but silently produces fake outputs — no error, no warning.

**Severity**: **P1** — silently fabricates success metrics; downstream consumers trust these scores for quality gates.

**Fix sketch**:
```rust
// reference_generation.rs:224-237 — replace stub with real dispatch
pub fn video_to_video(&mut self, input_path: &str) -> GenerationResult {
    let start = std::time::Instant::now();
    let result = match self.dispatch_generation(GenerationMode::VideoToVideo, input_path, None) {
        Ok(output) => GenerationResult {
            success: true,
            output_paths: vec![output.path],
            generation_time_ms: start.elapsed().as_millis() as u64,
            model_used: self.config.model_name.clone(),
            reference_similarity: output.similarity,
            quality_score: output.quality,
            error: None,
        },
        Err(e) => GenerationResult {
            success: false,
            output_paths: vec![],
            generation_time_ms: start.elapsed().as_millis() as u64,
            model_used: self.config.model_name.clone(),
            reference_similarity: 0.0,
            quality_score: 0.0,
            error: Some(e.to_string()),
        },
    };
    self.history.push(result.clone());
    result
}
```

---

## Pain Point 3: Checkpoint persistence writes file_size as always 0 (P2)

**File**: `neotrix-core/src/l1_action/nt_act/actions/core/checkpoint_persistence.rs:147`

**What's wrong**: `CheckpointMeta.file_size` is hardcoded to `0` with a TODO comment. This metadata is persisted to disk and consumed by cleanup/pruning logic — a `file_size: 0` record means the pruning logic cannot accurately estimate disk usage, and the `total_size` metric reported by checkpoint status is always understated.

**Severity**: **P2** — incorrect metadata; pruning logic may under-delete or over-delete depending on strategy.

**Fix sketch**:
```rust
// checkpoint_persistence.rs:147 — compute actual serialized size
// BEFORE:
//   file_size: 0, // TODO: 计算实际大小
// AFTER:
let json_bytes = serde_json::to_vec(&data)
    .unwrap_or_default();
let file_size = json_bytes.len() as u64;

let meta = CheckpointMeta {
    id: checkpoint_id.clone(),
    workflow_id: workflow_id.to_string(),
    stage_name: stage_name.to_string(),
    status: CheckpointStatus::Saved,
    created_at: current_timestamp(),
    expires_at: Some(current_timestamp() + self.config.expiration_secs),
    file_size, // now accurate
    description: None,
};
```

---

## Summary

| # | File | Issue | Severity | Fix |
|---|------|-------|----------|-----|
| 1 | `agent_cmds.rs:354,467` | PTC stubs/exec return empty Vec, gateway not wired | P0 | Wire `McpRegistry` gateway into stubs/exec paths |
| 2 | `reference_generation.rs:225-268` | 3 generation methods return fabricated results | P1 | Add real `dispatch_generation()` call with error propagation |
| 3 | `checkpoint_persistence.rs:147` | `file_size` always 0, pruner has wrong disk estimate | P2 | Compute `json_bytes.len()` before persisting |

All three are **silent success with wrong data** — the most dangerous class of bug because callers never retry.
