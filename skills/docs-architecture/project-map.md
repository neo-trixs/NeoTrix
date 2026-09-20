# NeoTrix Project Structure Map

> Updated: 2026-09-20 | Version: v0.21.0 | Post-Restructuring

---

## Workspace Overview

| Metric | Value |
|--------|-------|
| Workspace members | 10 |
| Total crates (in `/crates/`) | 7 |
| Total `.rs` files | 2,618 |
| Total lines of code | 834,467 |
| Resolver | v2 |
| Edition | 2021 |
| MSRV | 1.81 |

---

## Workspace Members

| # | Crate | Layer | Files | Lines | Description |
|---|-------|-------|------:|------:|-------------|
| 1 | `neotrix-core` | Core | 2,401 | 794,128 | Monolith — all layers L0-L6 |
| 2 | `src-tauri` | UI | — | — | Desktop app (Tauri) |
| 3 | `crates/neotrix-types` | L0 | 128 | 27,824 | Foundation types & traits |
| 4 | `crates/neotrix-sysctl` | L1 | 1 | 145 | System control |
| 5 | `crates/neotrix-consciousness` | L5 | 21 | 2,720 | Consciousness subsystem |
| 6 | `crates/neotrix-reasoning` | L5 | 27 | 700 | Reasoning engine |
| 7 | `crates/neotrix-gateway` | L5 | 17 | 5,314 | Gateway / dispatch |
| 8 | `crates/neotrix-multi-agent` | L5 | 16 | 3,180 | Multi-agent orchestration |
| 9 | `nt_core_capability_tree` | L4 | 11 | 4,475 | Capability tree (nested in core) |
| 10 | `crates/nt-lang` | L7 | 5 | 271 | DSL / language support |

---

## Layer Architecture (L0–L7)

```
┌─────────────────────────────────────────────────────┐
│ L7  nt-lang              │ DSL, codegen             │ 5 files    271 lines
├─────────────────────────────────────────────────────┤
│ L5  Cognition            │ consciousness, reasoning │ 81 files  11,914 lines
│     (4 crates)           │ gateway, multi-agent     │
├─────────────────────────────────────────────────────┤
│ L4  nt_core_capability   │ capability tree          │ 11 files   4,475 lines
├─────────────────────────────────────────────────────┤
│ L1  neotrix-sysctl       │ system control           │ 1 file      145 lines
├─────────────────────────────────────────────────────┤
│ L0  neotrix-types        │ foundation types         │ 128 files 27,824 lines
├─────────────────────────────────────────────────────┤
│ Core  neotrix-core       │ monolith (all layers)    │ 2,401 files 794,128 lines
│   ├─ ffi/                │ FFI bindings             │ 12 files   2,907 lines
│   ├─ nt_crystal_core/    │ Crystal core logic       │ 44 files  12,714 lines
│   ├─ nt_file_ability/    │ File processing          │ 37 files  11,856 lines
│   └─ (top-level)         │ lib.rs, main.rs, etc.    │ 6 files
└─────────────────────────────────────────────────────┘
```

**Note:** `neotrix-core` is the legacy monolith containing 96% of the codebase. The 7 standalone crates under `/crates/` represent the extracted/modularized layers.

---

## New Crate Structures (Post-Restructuring)

### `crates/neotrix-types` (L0 — Foundation)
```
src/
├── core/
│   ├── nt_core_hcube/          # Hypercube types
│   ├── nt_core_bank/           # Bank system (bank_impl/)
│   ├── nt_core_knowledge/      # Knowledge vectors (vectors_group_b/)
│   ├── nt_core_self/           # Self-model types
│   ├── nt_core_self_org/       # Self-organization
│   ├── nt_core_meta/           # Meta types
│   ├── nt_core_gwt/            # GWT types
│   ├── self_measure/           # Self-measurement
│   ├── context/                # Context types
│   ├── file_parser/            # File parsing types
│   ├── tools/                  # Tool types
│   └── skills/                 # Skill types
├── write_guard_types.rs
└── lib.rs
```

### `crates/neotrix-consciousness` (L5)
```
src/
└── 21 files — consciousness state machine, awareness loops
```

### `crates/neotrix-reasoning` (L5)
```
src/
└── 27 files — inference engine, reasoning chains
```

### `crates/neotrix-gateway` (L5)
```
src/
└── 17 files — API gateway, request dispatch
```

### `crates/neotrix-multi-agent` (L5)
```
src/
└── 16 files — agent orchestration, task routing
```

### `crates/nt-lang` (L7)
```
src/
├── codegen/    # Code generation
└── lib.rs
```

### `crates/neotrix-sysctl` (L1)
```
src/
└── lib.rs      # Single file — system control interface
```

---

## Recent Restructuring Commits

| Commit | Description |
|--------|-------------|
| `098888d6` | refactor: melt old architecture into crystal core — import path migration + directory standardization |
| `d82ad31b` | refactor: Phase 1 — eliminate core/ re-exports, migrate import paths to direct layer paths |
| `13dfd9a8` | fix: build clean — 0 errors, resolve phantom module references and type mismatches |
| `ff672104` | refactor(core): delete migrated module old files |
| `b46a7561` | refactor: core/ → layer directory migration (L0-L6) |
| `74ad4869` | feat(crystal): CrystalState unified state space — MVP |
| `7f4976bc` | refactor(crystal): Phase 3 — clean old layer dirs + batch update imports |
| `00f4069b` | refactor(crystal): Phase 2 — core/mod.rs reorg by layer |
| `7f8f07f8` | refactor(crystal): Phase 1 — clean dead code modules |

---

## Key Directories (Non-Crate)

| Path | Purpose |
|------|---------|
| `config/` | Runtime configuration |
| `docs/` | Documentation |
| `skills/` | Skills, tools, architecture docs |
| `src-tauri/` | Tauri desktop app |
| `.neotrix/` | Agent state / capabilities |
| `.neotrix-absorb/` | Knowledge absorption cache |
| `repo-analyses/` | Project analysis reports |

---

*Generated by project-map.sh (adapted for macOS) + manual verification*
