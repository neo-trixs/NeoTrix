# Fix: Deduplicate cosine_sim in neotrix-sim

## Summary

Removed 3 duplicate `cosine_sim` implementations from neotrix-sim, consolidating to the canonical version in `math_bridge.rs`.

## Changes

### 1. `src/agents/graph_memory.rs`
- Added `use crate::foundation::math_bridge::cosine_sim;`
- Removed local `fn cosine_sim(a: &[f32; 16], b: &[f32; 16]) -> f32` (lines 943-951)
- Removed `// Shared utilities` section header (orphaned after removal)
- **Callers unchanged**: `&[f32; 16]` auto-coerces to `&[f32]` (canonical signature)

### 2. `src/agents/memory_stream.rs`
- Added `use crate::foundation::math_bridge::cosine_sim;`
- Removed local `fn cosine_sim(a: &[f32; 16], b: &[f32; 16]) -> f32` (lines 289-298)
- Removed 3 redundant test functions: `cosine_sim_identical`, `cosine_sim_zero_vector`, `cosine_sim_orthogonal` (already covered by `math_bridge.rs` tests)
- **Callers unchanged**: `&[f32; 16]` auto-coerces to `&[f32]`

### 3. `src/consciousness/dual_representation.rs`
- Added `use crate::foundation::math_bridge::cosine_sim;`
- Removed local `fn cosine_sim(a: &[f32], b: &[f32]) -> f32` (lines 129-135)
- **Callers unchanged**: already passing `&[f32]` via `&target.values` / `&v.values`

## Canonical Source

`src/foundation/math_bridge.rs:318` — two public functions:
- `cosine_sim(a: &[f32], b: &[f32]) -> f32` — generic slice version
- `cosine_sim_16(a: &[f32; 16], b: &[f32; 16]) -> f32` — convenience wrapper

## Verification

| Check | Result |
|-------|--------|
| `cargo check -p neotrix-sim` | 0 errors |
| `cargo test -p neotrix-sim --lib` | 322 passed, 0 failed |

**Note**: Test count decreased from 325 → 322 because 3 redundant `cosine_sim` unit tests were removed (identical behavior already tested in `math_bridge.rs`).
