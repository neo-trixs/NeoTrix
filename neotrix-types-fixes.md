# G7: neotrix-types Clippy Fixes

## Files Modified

### crates/neotrix-types/src/core/self_model.rs
- **Line 4-6**: Removed empty line after doc comment (`empty_line_after_doc_comments`)

### crates/neotrix-types/src/core/self_measure/self_measure_impl/pid.rs
- **Line 38**: Changed `sorted[1].max(0.0).min(2.0)` to `sorted[1].clamp(0.0, 2.0)` (`manual_clamp`)
- **Line 56**: (still flagged) `needless_range_loop` — numerical code using `a[i][k] * a[i][k]` — safe to leave as-is for clarity in math code
- **Lines 71, 76**: (still flagged) `needless_range_loop` — matrix operations with dual index usage `a[i][j]` + `a[j][i]`

### crates/neotrix-types/src/core/self_measure/self_measure_impl/engine.rs
- **Lines 89-93**: Converted `for i in 0..NUM_SUBSYSTEMS { series[i].iter()... }` to iterator-based `series.iter().map(...).collect()` (`needless_range_loop`)
- **Lines 98-99**: Converted `for k in 0..n` to `for val in data[i].iter().zip(...)` (`needless_range_loop`)
- **Lines 140-144**: Converted `for j in (i+1)..NUM_SUBSYSTEMS { total_syn += syn_matrix[i][j] }` to iterator-based (`needless_range_loop`)

### crates/neotrix-types/src/core/self_measure/self_measure_impl/pairwise.rs
- **Lines 12-17**: Converted `for t in 0..n { for k in 0..NUM_SUBSYSTEMS { sum += series[k][t] } }` to iterator-based `.map()` + `.sum()` (`needless_range_loop`)

### crates/neotrix-types/src/core/context_strategy.rs
- **Line 64-69**: Collapsed nested `if` statements into single conditional (`collapsible_if`)

## Final State

**`cargo clippy -p neotrix-types --no-default-features --offline`**: ✅ **0 errors, 0 warnings**

All clippy issues were either fixed via iterator refactoring (where safe) or suppressed with `#[allow(...)]` (where refactoring could introduce subtle math bugs):

### Suppressed with `#[allow]`

| File | Function | Reason |
|------|----------|--------|
| `pid.rs:53` | `tridiagonalize()` | Householder tridiagonalization — 3 indexed loops over matrix rows/cols |
| `engine.rs:70` | `recompute_all()` | Synergy matrix + USK computation — dual-indexed matrix ops |
| `self_model.rs:32` | `SelfRepresentation::learn()` | Cholesky decomposition + coupling matrix copy — math-critical code |
| `self_model.rs:137` | `predict()` | Single-step prediction loops — simple and clear as-is |
| `self_model.rs:192` | `solve_linear_system_7()` | Gaussian elimination with partial pivoting — complex branching per iteration |
