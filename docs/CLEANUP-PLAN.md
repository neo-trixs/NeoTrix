# NeoTrix Project Cleanup Plan

> Generated: 2026-09-18 | Workspace root: `/Users/neo/Downloads/neotrix`

## Current State Metrics

| Category | Count | Notes |
|----------|-------|-------|
| Root-level `.md` files | 743 | 666 are `iteration_batch_*.md` logs |
| Workspace members | 10 | 2 use non-standard paths |
| `crates/` sub-crates | 3 | neotrix-types, neotrix-sysctl, nt-lang |
| `docs/` numbered dirs | 8 | 0-ARCHITECTURE → 7-REFERENCE (just normalized) |
| `docs/` loose `.md` files | 99 | Top-level docs, not in numbered dirs |
| `docs/` subdirectories | 23 | Includes `archive/`, `adr/`, `api/`, etc. |
| Non-standard root dirs | 10 | `l0_substrate/`, `snn/`, `models/`, `vendor/`, etc. |
| Build artifacts in root | ~30 | `.rlib`/`.rmeta` files from leaked builds |

## Recommended Directory Structure

```
neotrix/
├── Cargo.toml                    # Workspace root
├── crates/                       # All library crates (single source of truth)
│   ├── neotrix-core/             # Main crate (currently at root)
│   ├── neotrix-types/            # ✓ already in crates/
│   ├── neotrix-sysctl/           # ✓ already in crates/
│   ├── guard_core/               # ← MOVE from root
│   ├── nt-lang/                  # ✓ already in crates/
│   └── nt-core-capability-tree/  # ← EXTRACT from neotrix-core/src/
├── apps/                         # Application crates
│   ├── src-tauri/                # Desktop app
│   ├── neocodex-frontend/        # Frontend
│   ├── neotrix-sim/              # Simulation
│   └── nt-world-sim/             # World simulation (needs nested src-tauri/)
├── tools/                        # Build/CI tools (merge scripts/ here)
├── docs/                         # Documentation
│   ├── 0-ARCHITECTURE/
│   ├── 1-DESIGN/
│   ├── 2-PLANS/
│   ├── 3-API/
│   ├── 4-AUDITS/                 # ✓ just renamed
│   ├── 5-GUIDES/                 # ✓ just renamed
│   ├── 6-LEARNING/
│   └── 7-REFERENCE/
├── tests/                        # Integration tests (move from root e2e/)
├── archive/                      # Old/deprecated code (consolidate)
└── examples/                     # Usage examples
```

## Priority-Ordered Action Items

### P0 — Critical (do first)

| # | Action | Reason | Risk |
|---|--------|--------|------|
| 1 | **Archive 666 `iteration_batch_*.md`** to `archive/iteration-batches/` | Pollutes root, slows git | Low — logs are immutable |
| 2 | **Remove 30 `.rlib`/`.rmeta` build artifacts** from root | Build cache leaked into workspace | None — rebuild restores |
| 3 | **Clean stale `package-lock.json`** in root | Duplicate of `neocodex-frontend/` lock | Low — regenerate with `npm i` |

### P1 — High (workspace correctness)

| # | Action | Reason | Risk |
|---|--------|--------|------|
| 4 | **Move `guard_core/` → `crates/guard_core/`** | Workspace member should be in `crates/` | Medium — update `[workspace.dependencies]` paths |
| 5 | **Extract `nt_core_capability_tree` → `crates/nt-core-capability-tree/`** | Nested inside `neotrix-core/src/` — non-standard, violates crate independence | High — update internal `use` paths |
| 6 | **Move app crates into `apps/`**: `src-tauri/`, `neotrix-sim/`, `nt-world-sim/` | Separate library crates from application crates | Medium — update workspace members |

### P2 — Medium (docs hygiene)

| # | Action | Reason | Risk |
|---|--------|--------|------|
| 7 | **Consolidate `scripts/` → `tools/`** | `scripts/` has 80+ mixed-purpose files; `tools/` already exists | Low — update references |
| 8 | **Move `docs/` loose `.md` files** into numbered dirs | 99 files at `docs/` root; categorize by domain | Low — update `docs/.vitepress/` config |
| 9 | **Merge `docs/adr/` → `docs/0-ARCHITECTURE/adr/`** | ADRs belong with architecture decisions | Low — git tracks moves |
| 10 | **Merge `docs/api/` → `docs/3-API/`** | Duplicate API doc locations | Low |
| 11 | **Merge `docs/evolution/` → `docs/2-PLANS/`** | Evolution plans are plans | Low |

### P3 — Low (consistency)

| # | Action | Reason | Risk |
|---|--------|--------|------|
| 12 | **Standardize doc filenames to kebab-case** | 40+ files use `UPPER_SNAKE_CASE` or mixed case | Low — git rename |
| 13 | **Move non-standard root dirs** to proper locations | `snn/` → `crates/` or `archive/`, `l0_substrate/` → `archive/` | Low |
| 14 | **Consolidate `vendor/`** (30+ vendored crates) | Check if still needed; if build-time, move under `tools/vendor/` | Medium |
| 15 | **Clean `data/`, `models/`, `profile/`, `provenance/`** | Evaluate if still active; archive if stale | Low |

## What to Archive

| Path | Action | Destination |
|------|--------|-------------|
| `iteration_batch_*.md` (666 files) | Archive | `archive/iteration-batches/` |
| `*.rlib`, `*.rmeta` (build leaks) | Delete | N/A |
| `l0_substrate/` (empty dir) | Remove | — |
| `snn/` (1 Python file) | Archive or integrate | `archive/snn/` |
| `package-lock.json` (root) | Delete | — |

## What to Split

| Current | Split Into | Reason |
|---------|-----------|--------|
| `neotrix-core/src/nt_core_capability_tree/` | `crates/nt-core-capability-tree/` | Separate library crate from main crate source |
| `nt-world-sim/src-tauri/` (nested workspace member) | `apps/nt-world-sim-tauri/` | Avoid nested Cargo.toml in workspace member |
| `docs/` root loose files (99) | Into `0-7` numbered dirs | Categorize by domain |

## What to Reorganize

| Current | → New | Notes |
|---------|-------|-------|
| `guard_core/` | `crates/guard_core/` | Update workspace member path |
| `src-tauri/` | `apps/src-tauri/` | Application, not library |
| `neotrix-sim/` | `apps/neotrix-sim/` | Application crate |
| `nt-world-sim/` | `apps/nt-world-sim/` | Application crate |
| `scripts/` | `tools/scripts/` or `tools/` | Merge with existing `tools/` |
| `e2e/` | `tests/e2e/` | Standard Rust test location |
| `docs/adr/` | `docs/0-ARCHITECTURE/adr/` | ADRs are architecture decisions |
| `docs/api/` | `docs/3-API/` | Merge duplicate API docs |

## Post-Cleanup Verification

```bash
# 1. Verify workspace builds
cargo check --workspace --all-targets

# 2. Verify no broken internal references
cargo test --workspace --lib

# 3. Verify docs still build (if vitepress)
cd docs && npm run build

# 4. Check git status
git status --short | head -30
```

## Notes

- **Don't move `neotrix-core/`** — it's the main crate and moving it has cascading effects on all `use` paths. Keep at root.
- **`nt-world-sim/src-tauri/`** as a nested workspace member is unusual. Consider whether it can be a regular directory (non-member) with its own `Cargo.toml` only for `tauri build`.
- **`vendor/`** appears to be vendored crate sources (addr2line, aes, etc.). Verify if still needed or if `cargo vendor` output can be gitignored.
