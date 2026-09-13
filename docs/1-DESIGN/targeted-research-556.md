# Targeted Research 556: Error Handling Hardening

## Scope

Systematic elimination of silent error swallowing in NeoTrix non-test code, focused on three priority directories.

## Files Modified

### `neotrix-core/src/l6_meta/coordination/nt_governance/skill_validator/mod.rs`

**15 error-swallowing sites fixed:**

#### Pattern 1: `read_dir().into_iter().flatten().filter_map(|e| e.ok())` (6 locations)

Silently discarded directory read errors. Replaced with `match` that adds `ValidationFinding` on error.

| Function | Rule ID | Line (original) |
|----------|---------|-----------------|
| `validate_skills` | `SKILL-READDIR` | 214 |
| `validate_skill_references` | `SKILL-REFS-READDIR` | 323 |
| `validate_structure` (.claude-plugin) | `STRUCT-CLAUDE-READDIR` | 363 |
| `validate_commands` | `CMD-READDIR` | 383 |
| `validate_sidecars` (plugins) | `SIDECAR-PLUGINS-READDIR` | 485 |
| `find_uv_dirs` | N/A (returns empty on error) | 542 |

#### Pattern 2: `read_to_string().unwrap_or_default()` (7 locations)

Silently returned empty string on file read failure, causing downstream validators to produce misleading "missing frontmatter" errors instead of "can't read file" errors.

| Function | Rule ID | Line (original) |
|----------|---------|-----------------|
| `validate_plugin_json` | `PLUGIN-READ` | 144 |
| `validate_skill_frontmatter` | `SKILL-READ` | 239 |
| `validate_skill_length` | `SKILL-LEN-READ` | 303 |
| `validate_skill_references` | `SKILL-REFS-READ` | 327 |
| `validate_commands` | (continue on error) | 387 |
| `validate_paths` | (continue on error) | 410 |
| `validate_python_commands` | (continue on error) | 438 |

#### Pattern 3: `if let Ok(files) = self.find_files(...)` (2 locations)

Silently skipped file-finding errors. Replaced with `match` that adds warning finding.

| Function | Rule ID | Line (original) |
|----------|---------|-----------------|
| `validate_paths` | `PATH-FIND` | 408 |
| `validate_python_commands` | `PY-FIND` | 436 |

## Files Scanned (No Fixes Needed)

| File | Pattern Found | Verdict |
|------|---------------|---------|
| `l5_cognition/nt_core/other/nt_core_three_scope_map.rs` | `.unwrap_or(0)` for counts | **Acceptable** — count defaults are semantic |
| `l5_cognition/nt_core/other/nt_core_gencad.rs` | `.unwrap_or(Ordering::Equal)` for sort | **Acceptable** — NaN protection |
| `l5_cognition/nt_core/other/nt_core_gencad.rs` | `let _ = geo;` | **Test code only** |
| `l6_meta/coordination/layered_qa.rs` | `.unwrap_or("")` | **Acceptable** — JSON value extraction |
| `l6_meta/coordination/null_normalizer.rs` | `.unwrap_or_else` | **Acceptable** — intentional strategy fallback |
| `l6_meta/coordination/nt_meta_sentrux.rs` | `.unwrap_or(true)` | **Acceptable** — no-baseline = pass |
| `l6_meta/coordination/template_tag_registry.rs` | `.unwrap_or(Ordering::Equal)` | **Acceptable** — NaN protection |
| `l6_meta/coordination/nt_meta_async_safety.rs` | `.unwrap_or(default_gate_state)` | **Acceptable** — intentional default |
| `l1_action/nt_act/actions/*.rs` | `.unwrap()` | **Test code only** (lines 382, 349, 351, 380) |
| `l6_meta/coordination/nt_mind_repair/skill_improver/mod.rs` | `.unwrap()` | **Test code only** |

## Design Decisions

1. **Validator functions return `ValidationReport`, not `Result`** — errors are recorded as findings rather than propagated upward. This is correct for a validator that should report all issues, not fail-fast.

2. **`find_uv_dirs` uses silent fallback** — the function has no `report` parameter and its caller expects `Vec<PathBuf>`. On read failure, returning empty is correct (no discoverable dirs = no uv dirs).

3. **`read_to_string` in inner loops uses `continue`** — for `validate_commands`, `validate_paths`, `validate_python_commands`, individual file read failures skip that file rather than aborting the entire validation pass. This matches the "report all issues" philosophy.

4. **No `?` operator introduced** — the validate functions don't return `Result`, so `?` would require signature changes. The `match` + finding pattern is idiomatic for validator architecture.
