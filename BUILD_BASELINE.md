# NeoTrix Build Baseline

**Measured**: 2026-09-16

## Source Metrics

| Metric | Value |
|--------|-------|
| Total Rust files | 2,005 |
| Total lines of code | 685,768 |
| Files with `#[cfg(test)]` | 1,324 (66.0%) |
| Total `#[test]` functions | 10,969 |
| Public API items (`pub fn/struct/enum/trait`) | 22,631 |

## Build Health (`cargo check -p neotrix --lib`)

| Metric | Value |
|--------|-------|
| Compilation result | **FAIL — 109 errors** |

### Error Breakdown (by category)

| Category | Count | Description |
|----------|-------|-------------|
| E0433 — failed to resolve | 47 | Missing modules (`l2_perception`, `l1_body`, `l0_substrate`, `nt_core_vector_store` in `crate::core`) |
| E0432 — unresolved import | 55 | Types not found in re-exported paths (`CapabilityVector`, `LlmProvider`, `SearchResult`, `SaeFeature`, etc.) |
| Unused imports | 7 | Dead imports (not blocking) |

### Top Offending Patterns

1. **`crate::core::*` imports broken** — 30 + 12 + 8 = 50 errors reference items expected in `crate::core` that don't exist there (e.g., `l2_perception`, `nt_core_sense`, `CapabilityVector`, `SparseAutoencoder`, `ReasoningHexagram`)
2. **`nt_io_provider` re-export chain broken** — ~15 errors from `common::types::*` / `super::types::*` not resolving
3. **Duplicate `SearchResult` definition** — 1 explicit naming collision in `l1_facade.rs:118`

### Healthy

- Zero `unsafe` in core (R-P1 compliant)
- Test coverage: 1,324/2,005 modules have tests (66%)
- ~5.5 tests per file average (10,969 tests / 2,005 files)

## Summary

| Dimension | Status |
|-----------|--------|
| Codebase size | Large (685K LOC, 2K files) |
| Test infrastructure | Strong (66% module coverage, 11K tests) |
| Public API surface | Large (22K+ public items) |
| Build status | **Broken** — 109 compile errors, mostly from module path mismatches during layer migration |
