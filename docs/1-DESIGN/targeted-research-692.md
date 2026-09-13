# Targeted Research 692 — Error Handling Improvements

## Summary

Fixed silent error swallowing patterns across priority directories in NeoTrix. Replaced `let _ =` discards and `unwrap()` calls with proper error logging/propagation.

## Fixes Applied

### 1. NaN-safe float comparison (critical)

| File | Line | Before | After |
|------|------|--------|-------|
| `l6_meta/memory/nt_memory_experience_tree.rs` | 200 | `.partial_cmp().unwrap()` | `.partial_cmp().unwrap_or(Ordering::Equal)` |

**Impact**: Prevents panic on NaN values in experience importance sorting.

### 2. Gateway plugin hooks — silent discard → warn log

| File | Line | Before | After |
|------|------|--------|-------|
| `l1_action/nt_io/nt_io_provider/gateway/execution.rs` | 347 | `let _ = plugins._run_post_response(...)` | `if let Err(e) = ... { log::warn!(...) }` |
| `l1_action/nt_io/nt_io_provider/gateway/execution.rs` | 358 | `let _ = plugins._run_on_error(...)` | `if let Err(e) = ... { log::warn!(...) }` |

**Impact**: Plugin hook failures now visible in logs instead of silently dropped.

### 3. Provider pool — silent discard → warn log

| File | Line | Before | After |
|------|------|--------|-------|
| `l1_action/nt_io/nt_io_provider/pool/provider_pool.rs` | 119 | `let _ = fs::set_permissions(...)` | `if let Err(e) = ... { log::warn!(...) }` |
| `l1_action/nt_io/nt_io_provider/pool/provider_pool.rs` | 140 | `let _ = self.save()` | `if let Err(e) = ... { log::warn!(...) }` |

**Impact**: Permission errors and save failures after `remove()` now logged.

### 4. Consciousness KB persistence — silent discard → warn log

| File | Line | Key |
|------|------|-----|
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs` | 130 | `kv_set("consciousness", "phi_report")` |
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs` | 146 | `kv_set("consciousness", "gold_standard")` |
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs` | 159 | `kv_set("consciousness", "trends")` |
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs` | 172 | `kv_set("consciousness", "conversation")` |
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs` | 189 | `kv_set("consciousness", "blind_spots")` |
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs` | 238 | `record_consciousness_snapshot()` |

**Impact**: All consciousness state persistence failures now logged. Previously these could silently fail, causing data loss in the consciousness system.

### 5. Maintenance handler KB operations — silent discard → warn log

| File | Line | Operation |
|------|------|-----------|
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_maintenance.rs` | 204 | `registry.add_dependency()` |
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_maintenance.rs` | 476 | `kb.field_tick()` |
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_maintenance.rs` | 570 | `store.mark_crawl_complete()` |
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_maintenance.rs` | 578 | `store.mark_crawl_complete()` (error path) |
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_maintenance.rs` | 770 | `update_cluster_stats()` |
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_maintenance.rs` | 940 | `kb.field_tick()` |

**Impact**: Capability graph dependencies, crawl completion status, and cluster stats updates now logged on failure.

### 6. Core handler KB operations — silent discard → warn log

| File | Line | Operation |
|------|------|-----------|
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_core.rs` | 265 | `kb.kv_set("state", "exploration_queue")` |
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_core.rs` | 373 | `kb.insert_or_get_node()` (session logging) |

**Impact**: Session event logging and queue state persistence failures now visible.

### 7. Background loop initialization — silent discard → warn log

| File | Line | Operation |
|------|------|-----------|
| `l5_cognition/nt_mind/nt_mind_background_loop/mod.rs` | 135 | `absorb_cad_experience()` |
| `l5_cognition/nt_mind/nt_mind_background_loop/run.rs` | 435 | `kb.rebuild_graph_cache()` |
| `l5_cognition/nt_mind/nt_mind_background_loop/run.rs` | 438 | `kb.insert_or_get_node()` |

**Impact**: CAD experience absorption and graph cache rebuild failures now logged at startup.

### 8. Other KB persistence — silent discard → warn log

| File | Line | Operation |
|------|------|-----------|
| `l5_cognition/nt_core/nt_core_parallel/contract.rs` | 270 | `kb.kv_set()` (contract persistence) |
| `l5_cognition/nt_mind/nt_mind/reason/reasoning_engine/engine_core.rs` | 1318 | `kb.store_conversation_record()` |
| `l5_cognition/nt_mind/nt_mind_automation.rs` | 139 | `nt_core_state::save("automation")` |
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_daily_intel.rs` | 54 | `kv_set("daily-intel")` |

**Impact**: Task contracts, conversation records, automation rules, and daily intel gap markers now logged on persistence failure.

## Patterns Not Fixed (intentional)

| Pattern | Reason |
|---------|--------|
| `let _ = tx.send(...)` in streaming | Channel send — receiver drop is expected behavior |
| `.ok()` on `reqwest::Client::builder().build()` | Probe/discovery — failure means "not available", returns `None` |
| `let _ = CONFIGURED.set(())` | `OnceLock::set` — `Err` means already set, which is fine |
| `let _ = register_cad_gwt(...)` | Returns `bool`, not `Result` |
| `let _ = child.kill()` / `child.wait()` | Process cleanup — best-effort |
| `.unwrap_or_default()` on env vars | Missing env var → empty string is intentional fallback |

## Total Fixes: 22 call sites across 12 files
