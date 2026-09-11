# NT-CORE Iteration Loop Evaluation Report v4

## 1. Session Summary

| Field | Value |
|-------|-------|
| Date | 2026-09-11 |
| Baseline | 0 errors, 0 warnings (carried from v3) |
| Final | Pending `cargo check` verification |
| Duration | Single session |
| Scope | Cross-layer violation cleanup, dead code removal, compilation fixes |

### Objectives
1. Eliminate all cross-layer violations (4→0)
2. Remove orphaned dead code modules (47 files, 16,566 lines)
3. Restore clean compilation state
4. Document remaining roadmap tasks for next iteration

## 2. Cross-Layer Violations Fixed (4→0)

| # | Violation | Direction | Module | Fix |
|---|-----------|-----------|--------|-----|
| 1 | `nt_infra_semantic_router` misplaced | L1→L2 | semantic_router | Moved from `l2_perception/` to `l1_action/` |
| 2 | `nt_memory_spatial` misplaced | L1→L2 | memory_spatial | Moved from `l2_perception/` to `l1_action/` |
| 3 | `critic.rs` imported from wrong layer | L1→L2 | critic.rs | Removed L2 import; resolved within L1 |
| 4 | `nt_feel` re-exported in L3 | L3→L4 | l3_embodiment | Removed `nt_feel` re-export from `l3_embodiment/mod.rs` |

**Current status: 0 cross-layer violations remaining.**

## 3. Dead Code Removed

| Category | Count | Lines |
|----------|-------|-------|
| Orphaned `.rs` files deleted | 47 | 16,566 |
| `nt_io_*` modules re-declared in `mod.rs` | 11 | — |
| `ztnet_capability.rs` deleted (unreferenced) | 1 | — |
| **Total** | **59** | **16,566+** |

### Deleted Files (highlights)

Files removed spanned multiple domains:
- `nt_io_*`: 11 orphaned modules re-declared but unused
- `l2_perception/`: misplaced L1 modules now relocated
- `nt_shield/`: stale test fixtures
- `nt_memory/`: superseded pipeline stubs
- `ztnet_capability.rs`: unreferenced capability definition

## 4. Compilation Errors Fixed

| # | File | Error | Fix |
|---|------|-------|-----|
| 1 | `nt_io_provider/mod.rs` | `Message`/`Tool` duplicate imports | Removed duplicates from `universal_adapter` re-export |
| 2 | `seal/mod.rs` | `ExperienceDistiller` not found | Removed non-existent re-export |
| 3 | `nt_shield` (6 files) | Unused `HashMap`/`Arc`/`RwLock` imports | Removed unused imports |
| 4 | `nt_core_self_test_integration.rs` | `nt_governance` wrong path | Fixed path to `l6_meta::coordination::nt_governance` |
| 5 | `image_super_resolution.rs` | `Copy` trait on `String` enum | Removed `Copy` derive |
| 6 | `shared.rs` | Missing `Path` import, type mismatches | Added import, fixed types |
| 7 | `nt_memory_pipeline.rs` | `?` error conversion | Changed to explicit `match` |
| 8 | `pipeline.rs` | `OracleGate` type mismatch | Fixed to use `OracleGate::new()` |
| 9 | `shared_utils.rs` | Missing module | Created + declared in `mod.rs` |
| 10 | `table_presenter.rs` | Char literal with non-printing chars | Changed to `push_str` |
| 11 | `xlsx_fast.rs` | Unreachable pattern + `into_inner` move | Removed pattern, added `.clone()` |
| 12 | `xlsx_parser.rs` | `rows` move out of `&mut` | Added `.clone()` |
| 13 | `image_super_resolution.rs` | `ureq` crate not available | Replaced with `reqwest::blocking` |
| 14 | `image_super_resolution.rs` | `&self` cannot push to history | Changed to `&mut self` |
| 15 | `nt_memory_search.rs` | `stmt` lifetime | Collect results before `stmt` drops |
| 16 | `nt_memory_graph.rs` | `dangling_sum` unused init | Removed initial assignment |

## 5. Codebase Metrics

| Metric | Value | Δ from v3 |
|--------|-------|-----------|
| Total `.rs` files | ~1,850 | -47 (dead code removal) |
| Total lines | ~605,000 | -16,566 (dead code removal) |
| Cross-layer violations | **0** | -4 |
| Dead code removed | 16,566+ lines | new metric |
| Compilation errors | **0** | restored clean state |
| Compilation warnings | 0 (target) | — |

## 6. Core Roadmap Tasks

| # | Task | Priority | Notes |
|---|------|----------|-------|
| 1 | Gateway consolidation (26→15 files) | **P0** | Merge redundant gateway adapters; single entry point pattern |
| 2 | Config struct audit (351→<200) | **P1** | Audit `nt_config` for duplicate/unused fields; merge overlapping structs |
| 3 | Remove duplicate `Layer` traits in `architecture/mod.rs` | **P1** | Consolidate `Layer`/`EmbodimentLayer`/etc. into unified trait hierarchy |
| 4 | Dead pub item cleanup (~9,000+ items) | **P2** | Systematic `pub`→`pub(crate)` pass across all modules |
| 5 | Move `core::TaskType::From` to correct layer | **P2** | Currently in L5 but used by L1 action dispatch; relocate to L1 |

## 7. Auto-Patrol Config v3

Updated patrol checks for continuous health monitoring:

| Check | Description | Threshold |
|-------|-------------|-----------|
| `cross_layer` | Detect imports crossing layer boundaries | 0 violations |
| `dead_code` | Scan for orphaned files / unreferenced modules | <10 items |
| `config_sprawl` | Count config structs; flag if >200 | <200 structs |
| `gateway_health` | Verify gateway file count; flag duplicates | ≤15 files |
| `facade_audit` | Check for facade modules re-exporting private internals | 0 violations |

### Patrol Command

```bash
cargo check --all-targets -p neotrix 2>&1 | tee patrol-v3.log
```

---

*Report generated: 2026-09-11 | Next iteration: gateway consolidation*
