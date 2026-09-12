# Targeted Research #473 — Internal Pain Points (3)

**Date:** 2026-09-12
**Source:** `neotrix-core/src/` TODO/FIXME/HACK scan + hardcoded-value detection
**Priority:** P0-P2

---

## P1: L2 Disk Cache — Completely Hollow (nt_act_cache.rs:194)

**File:** `neotrix-core/src/l1_action/nt_act/actions/nt_act_cache.rs:194`
**What:** The L2 (disk) cache lookup is a no-op. The `l2_cache` field exists but the `get()` method skips it entirely — `TODO: 实际从磁盘读取` followed by empty block. This means the two-tier cache architecture (memory L1 + disk L2) is a lie: L1 misses always result in a `CacheResult::Miss` regardless of whether the key exists on disk. Additionally, the bloom filter for penetration protection (line 200) is also a stub, so repeated cache misses for the same key will repeatedly hit the backing store.

**Severity:** P1 — Data is silently evicted from L1 without L2 fallback; hot restarts lose all cache state.

**Fix sketch:**

```rust
// nt_act_cache.rs:194 — replace empty L2 block
if let Some(ref mut disk_cache) = self.l2_cache {
    if let Some(entry) = disk_cache.get(&key) {
        // Promote back to L1
        self.l1_cache.insert(key.clone(), entry.clone(), self.config.l1_ttl);
        self.stats.hits += 1;
        self.update_hit_rate();
        return CacheResult::Hit(entry);
    }
}
// Penetration protection — skip backing-store call if bloom filter says "never seen"
if self.config.penetration_protection {
    if let Some(ref bloom) = self.bloom_filter {
        if !bloom.check(&key) {
            self.stats.misses += 1;
            self.update_hit_rate();
            return CacheResult::Miss;
        }
    }
}
```

---

## P2: ResourceBudget Cost Estimate — Hardcoded Model Prices (resource_budget.rs:288)

**File:** `neotrix-core/src/l1_action/nt_act/resource_budget.rs:288`
**What:** `estimate_cost()` uses a hardcoded `match` with 3 model names and a fixed fallback. Any new model (GPT-4o, Claude 3.5, Gemini, Ollama local) returns `$0.001/1k` regardless of actual cost. This defeats the cost-aware routing axiom (A1) — the budget tracker cannot distinguish a $0.03/1k call from a free local call, so budget enforcement is meaningless for any model not in the hardcoded list.

**Severity:** P2 — Cost tracking silently wrong for all models except 3 legacy ones; budget limits unenforceable.

**Fix sketch:**

```rust
// resource_budget.rs:288 — replace hardcoded match with registry lookup
pub fn estimate_cost(&self, token_count: u64, model: &str) -> f64 {
    let cost_per_1k = self.model_prices
        .get(model)
        .copied()
        .unwrap_or_else(|| {
            tracing::warn!("unknown model '{}', using default cost", model);
            0.001 // safe fallback, but logged
        });
    (token_count as f64 / 1000.0) * cost_per_1k
}
// Add to ResourceBudget struct:
// model_prices: HashMap<String, f64>,  // loaded from config or KB on init
```

---

## P0: ProductionOrchestrator Checkpoint — No Persistence (production_orchestrator.rs:226)

**File:** `neotrix-core/src/l1_action/nt_act/actions/production_orchestrator.rs:226`
**What:** `save_checkpoint()` accepts a workflow_id, does nothing (`let _ = workflow; Ok(())`), and `restore_from_checkpoint()` (line 236) sets status to `Paused` but never reads saved state. This means: (1) process crash loses all in-flight workflow state, (2) multi-step orchestration with checkpoint/restart is impossible, (3) the `CheckpointCreated` event fires (line 228 implicitly) with no actual data behind it. This is the core of the production orchestration capability — it's a complete no-op.

**Severity:** P0 — Orchestration checkpointing is advertised in the event system but has zero persistence; crash = total state loss.

**Fix sketch:**

```rust
// production_orchestrator.rs:226
pub fn save_checkpoint(&self, workflow_id: &str) -> Result<(), String> {
    let workflow = self.workflows.get(workflow_id)
        .ok_or("工作流不存在")?;
    let snapshot = serde_json::to_vec(workflow)
        .map_err(|e| format!("序列化失败: {e}"))?;
    let checkpoint_dir = self.checkpoint_dir.as_ref()
        .ok_or("checkpoint_dir not configured")?;
    std::fs::create_dir_all(checkpoint_dir)
        .map_err(|e| format!("创建目录失败: {e}"))?;
    let path = checkpoint_dir.join(format!("{workflow_id}.checkpoint.json"));
    std::fs::write(&path, &snapshot)
        .map_err(|e| format!("写入检查点失败: {e}"))?;
    Ok(())
}
// restore_from_checkpoint: deserialize from same path, replace workflow state
```

---

## Summary

| # | File:Line | Issue | Severity |
|---|-----------|-------|----------|
| 1 | `nt_act_cache.rs:194` | L2 disk cache is a no-op; bloom filter stub | P1 |
| 2 | `resource_budget.rs:288` | Hardcoded 3-model price table; breaks cost-aware routing | P2 |
| 3 | `production_orchestrator.rs:226` | Checkpoint save/restore is a no-op; crash = state loss | P0 |

**Total new pain points:** 3
