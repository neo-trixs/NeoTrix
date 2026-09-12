# Targeted Research #506 — Internal Pain Points (WISER Loop Iteration 5)

**Date:** 2026-09-13
**Scope:** neotrix-core/src/ — TODO/FIXME stubs, hardcoded returns, dead code paths
**Method:** Grep for `TODO|FIXME|HACK|XXX`, `return Vec::new()/String::new()`, `#[allow(dead_code)]`

---

## Pain Point 1 — L2 Disk Cache Never Reads (P0)

**File:** `neotrix-core/src/l1_action/nt_act/actions/core/nt_act_cache.rs:193-196`

**What's wrong:**
The `CacheLayer::get()` method has a fully typed `DiskCache` struct and a `penetration_protection` config flag, but the L2 lookup body is empty. When L1 misses, the cache **always returns `CacheResult::Miss`** regardless of L2 config. The `DiskCache` struct (line 70-74) stores path/max_size but has zero `impl` methods — it's a dead struct. The bloom filter for penetration protection (line 199-201) is also a no-op. Any production workload with `l2_enabled: true` silently gets L1-only performance, making the config a lie.

**Severity:** P0 — Config promises behavior that never executes; silent data loss under load.

**Fix sketch (10 lines):**
```rust
// In CacheLayer::get(), replace lines 193-201:
if let Some(ref mut disk_cache) = self.l2_cache {
    if let Some(entry) = disk_cache.read(&key) {
        if entry.is_expired() {
            disk_cache.evict(&key);
        } else {
            self.stats.hits += 1;
            self.update_hit_rate();
            return CacheResult::Hit(entry);
        }
    }
}

// Add impl DiskCache { fn read(&mut self, key: &str) -> Option<CacheEntry> { ... } }
```

---

## Pain Point 2 — Production Orchestrator Checkpoint = No-Op (P1)

**File:** `neotrix-core/src/l1_action/nt_act/actions/orchestration/production_orchestrator.rs:224-242`

**What's wrong:**
`save_checkpoint()` accepts a `workflow_id`, silently drops it (`let _ = workflow;`), and returns `Ok(())` — nothing is persisted. `restore_from_checkpoint()` just sets status to `Paused` without restoring any state. If a production workflow crashes mid-execution, the user sees a "checkpoint saved" success message but recovery is impossible. The sibling file `checkpoint_persistence.rs` already has working serialization logic (lines 158-174) but is never called from here.

**Severity:** P1 — Silent data loss on crash recovery; checkpoint system is a facade.

**Fix sketch (8 lines):**
```rust
// In save_checkpoint(), replace lines 226-228:
let json = serde_json::to_string(workflow)
    .map_err(|e| format!("Serialize failed: {e}"))?;
let path = format!("{}/checkpoints/{}.json", self.storage_dir, workflow_id);
std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap())?;
std::fs::write(&path, &json)?;
log::info!("Checkpoint saved: {path}");
Ok(())
```

---

## Pain Point 3 — Checkpoint File Size Always 0 (P2)

**File:** `neotrix-core/src/l1_action/nt_act/actions/core/checkpoint_persistence.rs:147`

**What's wrong:**
`file_size: 0` is hardcoded with a `// TODO: 计算实际大小` comment. The very next block (lines 170-173) correctly computes the actual file size after write, but it's only used for the return value — the `CheckpointMeta.file_size` in the saved JSON is always `0`. Downstream consumers (size-based eviction, quota enforcement, monitoring dashboards) see zero-byte checkpoints and either skip eviction or break.

**Severity:** P2 — Monitoring/eviction logic gets wrong data; low immediate impact but causes drift.

**Fix sketch (4 lines):**
```rust
// Replace line 147, compute size after write:
let actual_size = std::fs::metadata(&file_path)
    .map(|m| m.len())
    .unwrap_or(0);
let mut meta = meta.clone();
meta.file_size = actual_size;
```

---

## Summary

| # | Pain Point | File:Line | Severity | Impact |
|---|-----------|-----------|----------|--------|
| 1 | L2 disk cache empty body | `nt_act_cache.rs:193-201` | P0 | Silent config lie, L2 never used |
| 2 | Checkpoint save/restore no-op | `production_orchestrator.rs:224-242` | P1 | Crash recovery impossible |
| 3 | file_size hardcoded to 0 | `checkpoint_persistence.rs:147` | P2 | Monitoring/eviction get wrong data |

**Previous fixes (27):** Rounds 1-4 covered build poisoning, dead modules, type mismatches, structural violations.
**Next recommended:** Round 6 should audit `#[allow(dead_code)]` suppressions — 60+ instances found, many masking real unused code.
