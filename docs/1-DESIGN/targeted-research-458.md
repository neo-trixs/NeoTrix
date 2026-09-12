# Targeted Research #458 — Internal Pain Points (3 Fixes)

**Date**: 2026-09-12
**Method**: Internal-first grep for TODO/FIXME/HACK stubs + hardcoded returns + dead code

---

## Pain Point 1: `nt_act_cache.rs` — L2 Disk Cache & Bloom Filter Are No-Ops

**File**: `neotrix-core/src/l1_action/nt_act/actions/nt_act_cache.rs:194-201`
**Severity**: **P1** (production cache silently degrades to L1-only)

**What's wrong**: The `CacheLayer` struct declares `l2_cache: Option<DiskCache>` with a full `DiskCache` struct (path, max_size_mb, current_size_mb), and `CacheConfig` has `l2_enabled: bool` + `l2_path`. But the actual `get()` method at line 194 does:
```rust
if let Some(ref mut _disk_cache) = self.l2_cache {
    // TODO: 实际从磁盘读取
}
```
Similarly, penetration protection at line 200:
```rust
if self.config.penetration_protection {
    // TODO: 实现布隆过滤器
}
```
The L2 cache is configured but never reads/writes. Any L1 miss is a true miss even when L2 is enabled. Bloom filter protection (to prevent cache stampede on missing keys) is also a no-op.

**Fix sketch**:
```rust
// L2 查找 (replace lines 194-196)
if let Some(ref mut disk_cache) = self.l2_cache {
    if let Some(entry) = disk_cache.get(&key) {
        // Promote to L1
        self.l1_cache.insert(key.to_string(), entry.clone());
        self.stats.hits += 1;
        self.update_hit_rate();
        return CacheResult::Hit(entry);
    }
}

// 穿透保护 (replace lines 199-201)
if self.config.penetration_protection {
    if self.bloom_filter.might_contain(key.as_bytes()) {
        // Key was seen before but missing — real miss, pass through
    } else {
        // First miss — record in bloom filter to prevent stampede
        self.bloom_filter.insert(key.as_bytes());
        // Short-circuit: return Miss without probing downstream
    }
}
```

---

## Pain Point 2: `model_routing.rs` — `route()` Never Calls Any Model API

**File**: `neotrix-core/src/l1_action/nt_io/model_routing.rs:297-309`
**Severity**: **P1** (routing response is fabricated — `output: None`, hardcoded `latency_ms: 5000`, `cost: +5.0`)

**What's wrong**: The `route()` method selects a model via strategy logic, but the actual inference call is stubbed:
```rust
// TODO: 实际调用模型 API
let response = RoutingResponse {
    success: true,
    selected_model: model_id.clone(),
    output: None,           // <-- always None
    cost: cost + 5.0,       // <-- hardcoded 5.0
    latency_ms: 5000,       // <-- hardcoded 5 seconds
    retries,
    error: None,
};
```
Every route returns the same fabricated response. The `output` field is always `None` — callers receive no actual inference result. The `latency_ms` is hardcoded to 5000ms regardless of actual model. This makes the entire multi-model routing layer a facade.

**Fix sketch**:
```rust
// Replace lines 297-309
let start = std::time::Instant::now();
let model = self.models.get(&model_id).ok_or("model not found")?;
let result = self.provider_registry
    .complete(&model.provider, &model.id, &request.messages)
    .await;
let latency = start.elapsed().as_millis() as u64;

let response = match result {
    Ok(output) => RoutingResponse {
        success: true,
        selected_model: model_id.clone(),
        output: Some(output),
        cost: model.price_per_second * (latency as f64 / 1000.0),
        latency_ms: latency,
        retries,
        error: None,
    },
    Err(e) => {
        retries += 1;
        last_error = Some(e.to_string());
        continue; // retry loop
    }
};
```

---

## Pain Point 3: `production_orchestrator.rs` — Checkpoint Save/Restore Are No-Ops

**File**: `neotrix-core/src/l1_action/nt_act/actions/production_orchestrator.rs:224-243`
**Severity**: **P1** (orchestrator claims "断点续传" but never persists or restores)

**What's wrong**: `OrchestratorConfig.enable_checkpoint: true` is the default (line 144), and `checkpoint_interval_secs: 60` is configured. But both `save_checkpoint` and `restore_from_checkpoint` are stubs:
```rust
pub fn save_checkpoint(&self, workflow_id: &str) -> Result<(), String> {
    if let Some(workflow) = self.workflows.get(workflow_id) {
        // TODO: 实际保存检查点到持久化存储
        let _ = workflow;
        Ok(())
    } else { Err("工作流不存在".into()) }
}

pub fn restore_from_checkpoint(&mut self, workflow_id: &str) -> Result<(), String> {
    // TODO: 实际从持久化存储恢复
    if let Some(workflow) = self.workflows.get_mut(workflow_id) {
        workflow.status = WorkflowStatus::Paused;
        Ok(())
    } else { Err("工作流不存在".into()) }
}
```
`save_checkpoint` discards the workflow reference. `restore_from_checkpoint` just sets status to Paused without loading any saved state. A process crash mid-workflow loses all progress — the checkpoint feature is decorative.

**Fix sketch**:
```rust
pub fn save_checkpoint(&self, workflow_id: &str) -> Result<(), String> {
    let workflow = self.workflows.get(workflow_id)
        .ok_or("工作流不存在")?;
    let checkpoint_dir = std::path::Path::new(&workflow.output_dir).join(".checkpoints");
    std::fs::create_dir_all(&checkpoint_dir).map_err(|e| e.to_string())?;
    let path = checkpoint_dir.join(format!("{}.json", workflow_id));
    let data = serde_json::to_string_pretty(workflow).map_err(|e| e.to_string())?;
    std::fs::write(&path, data).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn restore_from_checkpoint(&mut self, workflow_id: &str) -> Result<(), String> {
    // Find workflow to get output_dir
    let output_dir = self.workflows.get(workflow_id)
        .map(|w| w.output_dir.clone())
        .ok_or("工作流不存在")?;
    let path = std::path::Path::new(&output_dir)
        .join(".checkpoints").join(format!("{}.json", workflow_id));
    let data = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let restored: Workflow = serde_json::from_str(&data).map_err(|e| e.to_string())?;
    self.workflows.insert(workflow_id.to_string(), restored);
    Ok(())
}
```

---

## Summary

| # | File | Issue | Severity | Effort |
|---|------|-------|----------|--------|
| 1 | `nt_act_cache.rs:194-201` | L2 disk cache + bloom filter are no-ops | P1 | Small |
| 2 | `model_routing.rs:297-309` | `route()` fabricates responses, never calls API | P1 | Medium |
| 3 | `production_orchestrator.rs:224-243` | Checkpoint save/restore are no-ops | P1 | Small |
