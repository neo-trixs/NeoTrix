# Types Alignment Report: CircuitBreaker & CacheEntry

**Date**: 2026-09-08
**Status**: ANALYSIS ONLY — no .rs files modified

---

## 1. CRITICAL FINDING: Shared Types Do NOT Exist

**The `shared_types.rs` file contains NO CircuitBreaker, BreakerState, or CacheEntry types.**

Current contents of `crates/neotrix-types/src/core/shared_types.rs` (52 lines total):
- `Severity` enum
- `NtDomain` enum
- `HealthStatus` enum
- `TaskState` enum
- `TrendDirection` enum

**Implication**: All local implementations are fully independent. There is no shared type to align to. The alignment task must either:
1. Create new shared types in neotrix-types, OR
2. Accept that each domain has its own specialized variant (intentional design)

---

## 2. CircuitBreaker — Field-by-Field Mapping

### 2.1 Shared Type (DOES NOT EXIST)

The user-provided context describes a planned shared type:
```rust
pub struct CircuitBreaker {
    pub state: BreakerState,         // enum { Closed, Open, HalfOpen }
    pub failure_count: u32,
    pub success_count: u32,
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub timeout: Duration,
    pub last_failure: Option<Instant>,
}
```

### 2.2 Local Implementations (8 total)

#### A. `nt_act_circuit_breaker.rs` — L1 Action Domain (Full-Featured)
| Field | Type | Shared Match? | Notes |
|-------|------|---------------|-------|
| `state` | `CircuitState` (enum) | **NAME MISMATCH**: `CircuitState` vs `BreakerState` | Same variants: Closed, Open, HalfOpen |
| `failure_count` | `u32` | **EXACT MATCH** | |
| `success_count` | `u32` | **EXACT MATCH** | |
| `last_failure_time` | `Option<Instant>` | **NAME MISMATCH**: `last_failure_time` vs `last_failure` | Same type |
| `config` | `CircuitBreakerConfig` | **NOT IN SHARED** | Contains failure_threshold, success_threshold, timeout, half_open_max_calls, fallback_strategy |
| `stats` | `CircuitBreakerStats` | **NOT IN SHARED** | Contains total_calls, successful_calls, failed_calls, rejected_calls, state_changes, avg_response_time |

**Extra local fields**: `config` (embedded), `stats` (embedded)
**Missing from shared**: N/A (shared doesn't exist)
**Surplus vs shared**: `stats` struct, `fallback_strategy`, `half_open_max_calls`

#### B. `nt_infra_breaker.rs` — L1 Infrastructure (Error-Rate Based)
| Field | Type | Shared Match? | Notes |
|-------|------|---------------|-------|
| `state` | `BreakerState` (enum) | **EXACT MATCH** | Same variants |
| `config` | `BreakerConfig` | **NOT IN SHARED** | Contains error_threshold (f64), open_duration_ms (u64), half_open_max_calls, window_size |
| `recent_results` | `Vec<bool>` | **NOT IN SHARED** | Sliding window for error rate |
| `open_since` | `Option<u64>` | **TYPE MISMATCH**: `u64` ms timestamp vs `Option<Instant>` | Uses SystemTime epoch ms |
| `half_open_calls` | `u32` | **NOT IN SHARED** | |
| `half_open_successes` | `u32` | **NOT IN SHARED** | |

**Key difference**: Uses **error rate** (f64 threshold) not failure count threshold. Fundamentally different algorithm.

#### C. `nt_io_provider/circuit_breaker.rs` — IO Provider (Sliding Window)
| Field | Type | Shared Match? | Notes |
|-------|------|---------------|-------|
| `state` | `BreakerState` (enum) | **EXACT MATCH** | Same variants |
| `failure_count` | `u64` | **TYPE MISMATCH**: `u64` vs shared `u32` | |
| `failure_threshold` | `u64` | **TYPE MISMATCH**: `u64` vs shared `u32` | |
| `cooldown` | `Duration` | **NAME MISMATCH**: `cooldown` vs `timeout` | Same semantic |
| `last_state_change` | `Option<Instant>` | **NOT IN SHARED** (shared has `last_failure`) | Different semantic |
| `half_open_probes_used` | `u64` | **NOT IN SHARED** | |
| `half_open_max_probes` | `u64` | **NOT IN SHARED** | |
| `sliding_window` | `VecDeque<bool>` | **NOT IN SHARED** | |
| `window_size` | `usize` | **NOT IN SHARED** | |

**Extra fields**: 5 fields not in shared type.

#### D. `nt_core_observer_error.rs` — Observer Error Recovery
| Field | Type | Shared Match? | Notes |
|-------|------|---------------|-------|
| `state` | `CircuitState` (enum) | **NAME MISMATCH**: `CircuitState` vs `BreakerState` | Open has payload: `Open { since: usize }` |
| `failure_threshold` | `u32` | **EXACT MATCH** | |
| `half_open_timeout_ms` | `u64` | **NOT IN SHARED** | |
| `consecutive_failures` | `u32` | **NAME MISMATCH**: `consecutive_failures` vs `failure_count` | Same semantic |
| `last_tripped` | `Option<Instant>` | **NAME MISMATCH**: `last_tripped` vs `last_failure` | Similar semantic |

**Key difference**: `CircuitState::Open { since: usize }` carries data. Enum variants differ from shared.

#### E. `value_gate.rs` — Value Gate (Simplified Binary)
| Field | Type | Shared Match? | Notes |
|-------|------|---------------|-------|
| `consecutive_interceptions` | `usize` | **NAME MISMATCH + TYPE MISMATCH**: vs `failure_count: u32` | usize ≠ u32 |
| `last_trip_time` | `Option<i64>` | **TYPE MISMATCH**: `i64` (epoch secs) vs `Option<Instant>` | |
| `is_open` | `bool` | **NOT IN SHARED** | Binary state, no HalfOpen |

**Key difference**: Binary state (open/closed), no HalfOpen. Fundamentally simpler model.

#### F. `nt_act/client.rs` — HTTP Client
| Field | Type | Shared Match? | Notes |
|-------|------|---------------|-------|
| `failures` | `u32` | **NAME MISMATCH**: `failures` vs `failure_count` | Same type |
| `last_failure` | `Option<Instant>` | **EXACT MATCH** | |
| `state` | `CircuitState` (enum) | **NAME MISMATCH**: private `CircuitState` vs `BreakerState` | Same variants |

**Extra**: Uses hardcoded threshold (5), not configurable.

#### G. `goal_loop/types.rs` — Evolution Goal Loop
| Field | Type | Shared Match? | Notes |
|-------|------|---------------|-------|
| `state` | `CircuitState` (enum) | **NAME MISMATCH**: `CircuitState` vs `BreakerState` | Same variants |
| `failure_count` | `u64` | **TYPE MISMATCH**: `u64` vs `u32` | |
| `stall_count` | `u64` | **NOT IN SHARED** | Stall detection |
| `max_failures` | `u64` | **TYPE MISMATCH + NAME MISMATCH**: vs `failure_threshold: u32` | |
| `max_stalls` | `u64` | **NOT IN SHARED** | |
| `cooldown_secs` | `u64` | **NOT IN SHARED** | |
| `last_failure` | `Option<Instant>` | **EXACT MATCH** | |
| `last_stall_reason` | `Option<String>` | **NOT IN SHARED** | |

**Extra fields**: 3 fields (stall_count, max_stalls, last_stall_reason)

#### H. `guard_core/src/monitoring/mod.rs` — Guard Core (Minimal)
| Field | Type | Shared Match? | Notes |
|-------|------|---------------|-------|
| `name` | `String` | **NOT IN SHARED** | |
| `failures` | `u32` | **NAME MISMATCH**: `failures` vs `failure_count` | Same type |
| `max_failures` | `u32` | **NAME MISMATCH**: `max_failures` vs `failure_threshold` | Same type |
| `cooldown_until` | `Instant` | **NOT IN SHARED** | Deadline, not duration |

**Key difference**: Uses deadline (`cooldown_until`) not duration. Simplest implementation.

---

## 3. CacheEntry — Field-by-Field Mapping

### 3.1 Shared Type (DOES NOT EXIST)

No CacheEntry exists in neotrix-types.

### 3.2 Local Implementations (7 total)

#### A. `nt_act_cache.rs` — Unified Cache Layer (Generic JSON)
| Field | Type | Notes |
|-------|------|-------|
| `key` | `String` | |
| `value` | `serde_json::Value` | **Generic JSON value** |
| `created_at` | `Instant` | |
| `last_accessed` | `Instant` | |
| `access_count` | `u64` | |
| `ttl` | `Option<Duration>` | |
| `tags` | `Vec<String>` | |

#### B. `nt_memory_sweep_20260815.rs` — KV Cache Memory (Embedding Vectors)
| Field | Type | Notes |
|-------|------|-------|
| `key` | `String` | |
| `value` | `Vec<f64>` | **Embedding vector**, not JSON |
| `access_count` | `u64` | |
| `last_access` | `u64` | **TYPE MISMATCH**: `u64` clock tick vs `Instant` |

**Key difference**: Value is `Vec<f64>` (embedding), not `serde_json::Value`. Timestamp is `u64` clock.

#### C. `prompt_cache.rs` — Prompt Cache
| Field | Type | Notes |
|-------|------|-------|
| `hash` | `String` | **NOT IN OTHERS** |
| `prompt` | `String` | **NOT IN OTHERS** |
| `response` | `String` | **NOT IN OTHERS** |
| `created_at` | `Instant` | |
| `last_accessed` | `Instant` | |
| `access_count` | `u32` | **TYPE MISMATCH**: `u32` vs `u64` |
| `similarity` | `f64` | **NOT IN OTHERS** |

**Key difference**: Stores prompt+response pairs, not generic key-value.

#### D. `nt_core_deploy_cache.rs` — ANE Program Cache (Binary)
| Field | Type | Notes |
|-------|------|-------|
| `program_bytes` | `Vec<u8>` | **Binary data**, not JSON/string |
| `compiled_target` | `String` | **Domain-specific** |
| `created_at` | `Instant` | |
| `access_count` | `u64` | |
| `last_access` | `Instant` | |

**Key difference**: Value is `Vec<u8>` (compiled binary). No key field (uses tuple key).

#### E. `nt_core_cache.rs` — Semantic Cache (Minimal Internal)
| Field | Type | Notes |
|-------|------|-------|
| `value` | `String` | |
| `inserted_at` | `Instant` | **NAME MISMATCH**: vs `created_at` |

**Note**: Private struct, minimal internal use.

#### F. `nt_core_context/ccr.rs` — Compression Store (Fingerprint-Based)
| Field | Type | Notes |
|-------|------|-------|
| `original` | `Vec<u8>` | **Binary data** |
| `fingerprint` | `[u8; 16]` | **NOT IN OTHERS** |
| `created` | `Instant` | **NAME MISMATCH**: vs `created_at` |
| `access_count` | `u64` | |

**Key difference**: Uses fingerprint as key, stores original bytes.

#### G. `cache_detector.rs` — Cache Detection Registry (Metadata)
| Field | Type | Notes |
|-------|------|-------|
| `name` | `String` | |
| `cache_type` | `CacheType` (enum) | **NOT IN OTHERS** |
| `path_resolver` | `fn() -> Option<PathBuf>` | **NOT IN OTHERS** |
| `safe_to_clean` | `bool` | **NOT IN OTHERS** |

**Key difference**: Not a cache entry at all — it's a cache **detector** registry entry.

---

## 4. Type Conflict Summary

### CircuitBreaker Conflicts

| Conflict Type | Instances | Resolution Required |
|--------------|-----------|-------------------|
| Enum name: `CircuitState` vs `BreakerState` | 4 files use `CircuitState`, 2 use `BreakerState` | Must pick one |
| Integer types: `u32` vs `u64` | failure_count/threshold varies | Must pick one |
| Timestamp: `Instant` vs `u64` vs `i64` | 3 different representations | Must standardize |
| State model: 3-state vs 2-state | value_gate.rs is binary only | Cannot unify |
| Open variant payload: `Open { since: usize }` | observer_error.rs only | Special case |
| Error-rate model vs count model | nt_infra_breaker.rs uses f64 rate | Fundamentally different |

### CacheEntry Conflicts

| Conflict Type | Instances | Resolution Required |
|--------------|-----------|-------------------|
| Value type: `serde_json::Value` vs `Vec<f64>` vs `String` vs `Vec<u8>` | All different | Cannot use single generic without trait bounds |
| Timestamp: `Instant` vs `u64` | 2 representations | Must standardize |
| Access count: `u32` vs `u64` | prompt_cache uses u32 | Must pick one |
| Field names: `created_at` vs `inserted_at` vs `created` | 3 variations | Must pick one |
| Domain-specific fields | prompt/response, program_bytes, fingerprint | Cannot all fit in shared type |

---

## 5. Recommendations

### 5.1 CircuitBreaker Alignment Strategy

**DO NOT create a single shared CircuitBreaker type.** The implementations have fundamentally different algorithms:

| Algorithm | Files | Can Share? |
|-----------|-------|-----------|
| Count-based (failure_count >= threshold) | nt_act_circuit_breaker, nt_core_observer_error, nt_act/client, guard_core | YES — subset |
| Rate-based (error_rate >= threshold) | nt_infra_breaker | NO — different algorithm |
| Stall-augmented | goal_loop/types | NO — extra dimensions |
| Binary (is_open bool) | value_gate | NO — 2-state only |
| Provider-specific (sliding window + probes) | nt_io_provider | MAYBE — extended fields |

**Recommended approach**: Create a `CircuitBreakerState` enum in neotrix-types (the only truly shared concept):
```rust
// crates/neotrix-types/src/core/shared_types.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BreakerState {
    Closed,
    Open,
    HalfOpen,
}
```

This is the **only** thing all 8 implementations agree on. Each keeps its own struct.

### 5.2 CacheEntry Alignment Strategy

**DO NOT create a single shared CacheEntry type.** The value types are fundamentally incompatible:

| Value Type | Files | Generic? |
|-----------|-------|---------|
| `serde_json::Value` | nt_act_cache | Could be generic V |
| `Vec<f64>` | nt_memory_sweep | Domain: embeddings |
| `String` (prompt+response) | prompt_cache | Domain: prompts |
| `Vec<u8>` | nt_core_deploy_cache | Domain: binaries |
| `Vec<u8>` + fingerprint | ccr | Domain: compression |

**Recommended approach**: No shared type. Each domain's CacheEntry is specialized. If a shared type is truly needed, make it fully generic:
```rust
pub struct CacheEntry<V> {
    pub value: V,
    pub created_at: Instant,
    pub last_accessed: Instant,
    pub access_count: u64,
}
```

But this loses domain-specific fields (tags, ttl, similarity, etc.), making it impractical.

### 5.3 Files That Could Be Safely Modified (if shared type existed)

| File | Replaceable? | Reason |
|------|-------------|--------|
| `nt_act_circuit_breaker.rs` | PARTIAL | Shares field names + types with planned shared, but has extra config/stats |
| `nt_infra_breaker.rs` | NO | Different algorithm (rate-based) |
| `nt_io_provider/circuit_breaker.rs` | NO | Sliding window + probe model |
| `nt_core_observer_error.rs` | PARTIAL | Similar but Open has payload |
| `value_gate.rs` | NO | Binary state, completely different |
| `nt_act/client.rs` | NO | Hardcoded threshold, private struct |
| `goal_loop/types.rs` | NO | Stall-augmented |
| `guard_core/monitoring/mod.rs` | NO | Deadline-based, minimal |

For CacheEntry: **NONE** are replaceable — all have domain-specific value types.

### 5.4 What to Do Instead

1. **Create `BreakerState` enum only** in `shared_types.rs` — the one thing all implementations agree on
2. **Leave all local structs as-is** — they are intentionally specialized per domain
3. **Document the mapping** (this report) so future agents understand the relationships
4. **If unification is required later**, start with the 4 count-based implementations and create a trait:
   ```rust
   pub trait CircuitBreakerOps {
       fn state(&self) -> BreakerState;
       fn is_available(&self) -> bool;
       fn record_success(&mut self);
       fn record_failure(&mut self);
   }
   ```

---

## 6. Field Mapping Quick Reference

### CircuitBreaker — Canonical Field Names (if shared type created)

| Shared Field | Type | nt_act_circuit | nt_infra | nt_io_provider | observer_error | value_gate | client | goal_loop | guard_core |
|-------------|------|---------------|----------|---------------|---------------|-----------|--------|----------|-----------|
| `state` | BreakerState | state | state | state | state | — | state | state | — |
| `failure_count` | u32 | failure_count ✓ | — | failure_count (u64) | consecutive_failures | — | failures | failure_count (u64) | failures |
| `success_count` | u32 | success_count | — | — | — | — | — | — | — |
| `failure_threshold` | u32 | config.failure_threshold | — | failure_threshold (u64) | failure_threshold ✓ | — | hardcoded(5) | max_failures (u64) | max_failures |
| `success_threshold` | u32 | config.success_threshold | — | — | — | — | — | — | — |
| `timeout` | Duration | config.timeout | config.open_duration_ms | cooldown | half_open_timeout_ms | — | hardcoded(60s) | cooldown_secs | — |
| `last_failure` | Option<Instant> | last_failure_time | open_since (u64) | last_state_change | last_tripped | last_trip_time (i64) | last_failure ✓ | last_failure ✓ | cooldown_until (deadline) |

### CacheEntry — Canonical Field Names

| Shared Field | nt_act_cache | nt_memory_sweep | prompt_cache | deploy_cache | nt_core_cache | ccr | cache_detector |
|-------------|-------------|----------------|-------------|-------------|--------------|-----|---------------|
| `key` | key ✓ | key ✓ | hash | (tuple) | (HashMap key) | fingerprint | name |
| `value` | serde_json::Value | Vec<f64> | String (response) | Vec<u8> | String | Vec<u8> | — |
| `created_at` | Instant ✓ | — | Instant ✓ | Instant ✓ | — | Instant (created) | — |
| `last_accessed` | Instant ✓ | — | Instant ✓ | Instant (last_access) | — | — | — |
| `access_count` | u64 ✓ | u64 ✓ | u32 | u64 ✓ | — | u64 ✓ | — |

---

## 7. Conclusion

**No .rs files were modified.** This is a read-only analysis.

The "shared types" described in the task context do not exist in `neotrix-types/shared_types.rs`. All 8 CircuitBreaker and 7 CacheEntry implementations are fully independent with fundamentally different algorithms, type representations, and domain specializations.

**The only safely shareable concept is the `BreakerState` enum** (3 variants: Closed, Open, HalfOpen). Everything else requires domain-specific implementations or a trait-based approach rather than a single struct.
