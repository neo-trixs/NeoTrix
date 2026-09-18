# Archived Old Modules — 2026-09-17

## Purpose
Orphaned scaffold files and unused modules removed from the active codebase during final cleanup.

## Archived Files

| File | Original Path | Lines | Reason |
|------|---------------|-------|--------|
| `harness_scaffold.rs` | `l1_action/nt_act/nt_act_orchestrator/` | 462 | Scaffold file, not referenced |
| `mgm_scaffold.rs` | `l6_meta/coordination/nt_meta_cleanup/` | 68 | Scaffold file, not referenced |
| `nt_mind_consciousness_gold_standard.rs` | `l6_meta/healing/` | 558 | Gold standard reference, not integrated |
| `scaffold_self_modify.rs` | `l5_cognition/nt_mind/nt_mind/evolution/` | 78 | Scaffold file, not referenced |
| `stakeholder_comm.rs` | `l5_cognition/nt_mind/nt_mind/infrastructure/` | 237 | Unused stakeholder communication module |
| `nt_core_golden_ratio.rs` | `l5_cognition/nt_core/visual/` | 292 | Golden ratio visual module, not integrated |

**Total: 6 files, 1695 lines**

## Verification
All files were verified to be:
- Not referenced in any other `.rs` file via `mod` declarations or `use` statements
- Not part of the active build path
- Safe to archive without breaking compilation

## Restoration
To restore any file, copy it back to its original path and add the appropriate `mod` declaration to the parent module's `mod.rs`.