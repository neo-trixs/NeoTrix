# Targeted Research 467 — WISER Iteration: 3 Internal Pain Points

**Date:** 2026-09-12
**Method:** WISER (Widen → Identify → Solve → Evaluate → Refine)
**Focus:** TODO/FIXME stubs, hardcoded returns, dead code in `neotrix-core/src/`

---

## Pain Point 1: Trade Orchestrator — 7 TODO stubs with commented-out engine calls

**File:** `neotrix-core/src/l1_action/nt_act/nt_act_trade/orchestrator.rs`
**Lines:** 695-703, 796-797, 833-834, 845, 870, 889
**Severity:** P1

**What's wrong:** The `handle_objection` method at line 695 has `ObjectionCategory → Objection` conversion TODO'd, and `is_resolved` check is commented out. Lines 796, 833, 845, 870, 889 all have commented-out calls to `ProductionEngine` methods that were never implemented. The trade pipeline silently accepts objections and production events without actually invoking the negotiation or production engines — callers get `Ok(())` back with zero real work done.

**Why it matters:** This is a critical business domain path. External callers believe objection handling and production tracking are functional. They are not.

**Fix sketch:**
```rust
// orchestrator.rs:685 — handle_objection
pub fn handle_objection(
    &mut self, order_id: &str,
    objection: &ObjectionCategory, _concession: &Concession,
) -> Result<(), String> {
    let ctx = self.active_trades.get_mut(order_id)
        .ok_or_else(|| format!("Trade {} not found", order_id))?;
    let objection_type = objection.to_objection(); // implement From/Into
    self.negotiation_engine.handle_objection(&objection_type);
    ctx.conversations.push(format!("FT08_Objection:{:?}", objection));
    if self.negotiation_engine.is_resolved() {
        ctx.current_phase = TradePhase26::Ft09ContractReviewSigning;
    }
    Ok(())
}
```

---

## Pain Point 2: Cache Layer — L2 disk lookup is a no-op

**File:** `neotrix-core/src/l1_action/nt_act/actions/nt_act_cache.rs`
**Lines:** 194-196
**Severity:** P1

**What's wrong:** The L2 (disk) cache path at line 194 has `if let Some(ref mut _disk_cache) = self.l2_cache` with a `// TODO: 实际从磁盘读取` comment and an empty body. The bloom filter at line 200 is also TODO'd. When `l2_enabled: true` in config, the cache silently skips disk reads — every "L2 hit" is actually a miss. The `_disk_cache` binding is prefixed with underscore to suppress warnings, meaning the compiler already flagged this as dead.

**Why it matters:** L2 disk cache is advertised in `CacheConfig` and `CacheLayer` struct fields. Users enabling it get zero benefit and a false sense of cache coverage. Hit-rate stats are inflated by L1-only hits.

**Fix sketch:**
```rust
// nt_act_cache.rs:194 — L2 disk lookup
if let Some(ref mut disk_cache) = self.l2_cache {
    if let Some(entry) = disk_cache.get(&key) {
        // Promote to L1
        self.l1_cache.insert(key.clone(), entry.clone());
        self.stats.hits += 1;
        self.update_hit_rate();
        return CacheResult::HitL2(entry);
    }
}

// nt_act_cache.rs:200 — bloom filter
if self.config.penetration_protection {
    if self.bloom.might_contain(&key) {
        // Key definitely NOT in cache → skip L1/L2, return Miss immediately
    }
}
```

---

## Pain Point 3: Checkpoint Persistence — file_size hardcoded to 0

**File:** `neotrix-core/src/l1_action/nt_act/actions/checkpoint_persistence.rs`
**Line:** 147
**Severity:** P2

**What's wrong:** `file_size: 0, // TODO: 计算实际大小` means every checkpoint is recorded with zero bytes. The `CheckpointMeta` struct is used for expiration decisions, storage quotas, and diagnostics. Zero file_size corrupts storage accounting and makes it impossible to detect oversized checkpoints or enforce `l2_max_size_mb` limits.

**Why it matters:** Checkpoint metadata is persisted and queried later. A 0-byte file_size makes garbage collection and storage budgeting unreliable. The `file_size` field is also likely used in the CLI `todo` command output (`entry/mod.rs:1066`), giving operators false info.

**Fix sketch:**
```rust
// checkpoint_persistence.rs:147 — compute actual size
let serialized = bincode::serialize(&state)
    .map_err(|e| format!("Checkpoint serialize failed: {}", e))?;
let file_size = serialized.len() as u64;

let meta = CheckpointMeta {
    id: checkpoint_id.clone(),
    workflow_id: workflow_id.to_string(),
    stage_name: stage_name.to_string(),
    status: CheckpointStatus::Saved,
    created_at: current_timestamp(),
    expires_at: Some(current_timestamp() + self.config.expiration_secs),
    file_size,
    description: None,
};
```

---

## Summary

| # | File | Line | Severity | Category | Issue |
|---|------|------|----------|----------|-------|
| 1 | `nt_act_trade/orchestrator.rs` | 695,796,833 | P1 | Dead code path | 7 commented-out engine calls; objection/production silently no-ops |
| 2 | `nt_act_cache.rs` | 194,200 | P1 | Hardcoded return | L2 disk cache and bloom filter are empty stubs; config lies |
| 3 | `checkpoint_persistence.rs` | 147 | P2 | Hardcoded return | `file_size: 0` breaks storage accounting and GC |

**Next action:** Implement fix #1 (orchestrator engine integration) first — highest blast radius. Fix #2 (cache) second — enables real L2 caching. Fix #3 (file_size) is a quick win.
