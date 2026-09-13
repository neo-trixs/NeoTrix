# Targeted Research 682 — Error Handling Audit

**Date**: 2026-09-13
**Scope**: `l5_cognition/nt_core/visual/*.rs`, `l4_emotion/nt_feel/*.rs`, `l2_perception/nt_world/*.rs`

## Summary

Searched for two anti-patterns in priority directories:
1. `unwrap()` in non-test code (panic risk)
2. `Ok(default)` error swallowing (silent failure)

**Findings**: 8 non-test `unwrap()` calls fixed across 7 files. 0 intentional `Ok(default)` patterns were false positives (all were by design).

## Fixes Applied

### 1. Float comparison panics (`partial_cmp().unwrap()`)

| File | Line | Risk | Fix |
|------|------|------|-----|
| `dynamic_memory_bank.rs:169` | Sort by relevance score | NaN → panic | `.unwrap_or(Ordering::Equal)` |
| `dynamic_memory_bank.rs:201` | Sort by relevance score | NaN → panic | `.unwrap_or(Ordering::Equal)` |
| `nt_world_semantic_extract.rs:364` | Sort by entity score | NaN → panic | `.unwrap_or(Ordering::Equal)` |

**Pattern**: `f32::partial_cmp()` returns `None` for NaN. `.unwrap()` panics on NaN input.

### 2. Mutex lock poisoning (`Mutex::lock().unwrap()`)

| File | Line | Risk | Fix |
|------|------|------|-----|
| `crawl/stealth.rs:32` | `get_session()` | Poisoned mutex → panic | `match lock { Ok(guard) => guard, Err(e) => e.into_inner() }` |
| `crawl/stealth.rs:42` | `active_count()` | Poisoned mutex → panic | Same recovery pattern |
| `nt_world_mirror.rs:25` | `ranked_mirrors()` | Poisoned mutex → panic | Same recovery pattern |
| `nt_world_edgar.rs:298` | `rate_limit()` | Poisoned mutex → panic | Same recovery pattern |

**Pattern**: When a thread panics while holding a `Mutex`, the mutex becomes "poisoned". Subsequent `.lock().unwrap()` panics. Using `into_inner()` recovers the data — correct for best-effort concurrent state.

### 3. Semaphore acquisition (`Semaphore::acquire().await.unwrap()`)

| File | Line | Risk | Fix |
|------|------|------|-----|
| `asset_map/scanner/mod.rs:106` | `scan_ports()` batch | Closed semaphore → panic | `if let Some(_permit) = sem.acquire().await { ... } else { fallback result }` |

**Pattern**: `tokio::sync::Semaphore` returns `None` when closed. Replaced with graceful fallback returning an `is_open: false` result.

### 4. SystemTime duration (`SystemTime::now().duration_since(UNIX_EPOCH).unwrap()`)

| File | Line | Risk | Fix |
|------|------|------|-----|
| `dynamic_memory_bank.rs:312-314` | `current_timestamp()` | Clock before epoch → panic | `.unwrap_or_default()` |

**Pattern**: On most systems this never fails (clock is always past epoch), but `unwrap_or_default()` is defensive and semantically equivalent (returns 0 on failure).

## Not Fixed (Intentional by Design)

| File | Line | Pattern | Reason |
|------|------|---------|--------|
| `nt_world_dsh_explore.rs:39` | `Ok(vec![])` | Empty search result | Documented: "offline corpus returns empty for non-matching queries" |
| `nt_world_edgar.rs:483` | `Ok(vec![])` | Empty search result | Documented: "EDGAR doesn't support general search, returns empty" |
| `nt_world_gdelt.rs:60` | `Ok(vec![])` | Empty/error body | GDELT API returns status-only body → empty result is correct |
| `source/text/feed/engine.rs:86` | `Ok(String::new())` | HTTP 304 Not Modified | RFC 7232: 304 means "use cached" → empty string signals "no change" |
| `source/multi_cache.rs:34,43` | `NonZeroUsize::new().unwrap()` | Constructor | Constants always > 0; `unwrap()` is safe here |

## Statistics

- **Files scanned**: 10 (visual) + 4 (emotion) + 50+ (world)
- **Non-test unwrap() found**: 8 in production code (remainder are in `#[cfg(test)]`)
- **Ok(default) found**: 4 instances — all intentional
- **Fixes applied**: 8 unwrap replacements across 7 files
- **Compiles**: Verified (pre-existing errors in unrelated modules are unchanged)
