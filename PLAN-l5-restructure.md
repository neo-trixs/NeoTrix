# Plan: L5 Cognition Flat→Nested Restructure

## Current State

- **59 `pub mod` declarations** in `mod.rs` (83 lines including blanks/comments)
- **55 `.rs` files** + **13 directories** in `l5_cognition/`
- **38 re-exports** in `core/mod.rs` (lines 87-124) depend on `l5_cognition::nt_core_*` paths
- **39 internal references** to `layer_aliases` — must stay at l5_cognition level

## Module Inventory (59 total)

### Top-level staying (27 modules)
```
traits, nt_core, nt_mind, nt_cognition_facade, layer_aliases,
l1_facade, act_facade, io_facade, kb_facade, l2_facade, l3_facade, io_skills_facade,
nt_goal, nt_core_model_router, nt_core_skill_registry, nt_core_multi_agent,
nt_core_context_engine, nt_core_agents_md, nt_core_hybrid_search,
nt_core_hive, nt_core_byoa, nt_core_agent_circuit_breaker,
nt_core_model_gateway, nt_core_semantic_router
```
Note: `nt_core_model_gateway` is also assigned to `strategy/` — needs duplicate or re-export.

### reasoning/ (8 modules)
```
nt_core_reasoning.rs, nt_core_cot_generator.rs, nt_core_gate/ (DIR),
nt_core_prm/ (DIR), nt_core_meaning.rs, nt_core_paradigm.rs, nt_core_rule_memory.rs
```
Plus `nt_core_reasoning` also exists as `nt_core/nt_core_reasoning/` — no conflict since parent is different.

### math/ (4 modules)
```
nt_core_math.rs, nt_core_hex.rs, nt_core_walsh.rs, nt_core_kron.rs
```

### strategy/ (10 modules)
```
nt_core_plan/ (DIR), nt_core_policy.rs, nt_core_credit.rs, nt_core_dispatch.rs,
nt_core_orchestration_failure_taxonomy.rs, nt_core_arch_diagram.rs,
nt_core_arch_fitness.rs, nt_core_model_skills.rs, nt_core_coordination_principles.rs
```

### types/ (4 modules)
```
nt_core_shared_types.rs, nt_core_kernel_types.rs, nt_core_narrative_types.rs, nt_core_state.rs
```

### consciousness/ (11 modules)
```
nt_core_consciousness/ (DIR), nt_core_consciousness_core.rs,
nt_core_consciousness_tree/ (DIR), nt_core_consciousness_crystal/ (DIR),
consciousness_core/ (DIR), nt_core_gwt/ (DIR), nt_core_context/ (DIR),
nt_core_echo_terminal.rs, nt_core_aura/ (DIR), nt_core_cad_consciousness.rs,
nt_core_god_agent.rs
```

### evolution/ (9 modules)
```
nt_core_quantum_fusion.rs, nt_core_scoring_substrate.rs, nt_core_sae.rs,
nt_core_sae_bridge.rs, nt_core_td.rs, nt_core_trajectory_compress.rs,
nt_core_ttc.rs, nt_core_panic_recovery.rs, nt_core_second_brain.rs
```

### Summary
| Group | Count |
|-------|-------|
| Top-level (staying) | 25 (excluding mod.rs) |
| reasoning/ | 8 |
| math/ | 4 |
| strategy/ | 9 |
| types/ | 4 |
| consciousness/ | 11 |
| evolution/ | 9 |
| **Total** | **70** |

Note: 59 declarations but some items are directories with internal modules. The total of 70 includes sub-module directories.

## Execution Steps

### Step 1: Create 6 subdirectories + mod.rs
```bash
mkdir -p l5_cognition/{reasoning,math,strategy,types,consciousness,evolution}
```
Write mod.rs for each.

### Step 2: Move .rs files to subdirectories
Move 44 `.rs` files (the ones being grouped).

### Step 3: Move directory modules
Move 8 directories (`nt_core_gate/`, `nt_core_prm/`, `nt_core_plan/`, `nt_core_consciousness/`, `nt_core_consciousness_tree/`, `nt_core_consciousness_crystal/`, `consciousness_core/`, `nt_core_gwt/`, `nt_core_context/`, `nt_core_aura/`) to subdirectories.

### Step 4: Rewrite `l5_cognition/mod.rs`
- Keep 25 top-level `pub mod` declarations
- Add 6 sub-group `pub mod` declarations
- Add backward-compat re-exports: `pub use reasoning::*; pub use math::*;` etc.

### Step 5: Compile check
`cargo check -p neotrix --lib`

### Step 6: Fix any remaining path issues
Update `core/mod.rs` re-exports if needed (should work via re-exports).

## Backward Compatibility

All moved modules get re-exported at old paths via:
```rust
pub use reasoning::*;
pub use math::*;
pub use strategy::*;
pub use types::*;
pub use consciousness::*;
pub use evolution::*;
```

This means `crate::l5_cognition::nt_core_consciousness` still resolves even though the file moved to `consciousness/nt_core_consciousness/`.

## Risk: nt_core_model_gateway
This module appears in both "top-level staying" and "strategy/" in my categorization. Decision: keep at top level only (it's a new module from Cumora). Remove from strategy group.
