# Targeted Research 691 — Internal Dispatch Route Wiring

## Summary

Added 4 new internal dispatch routes to `consciousness_core/dispatch.rs` for knowledge management and SEAL evolution capabilities.

## Routes Added

| Keyword | Capability Tag | Domain | Specialist |
|---------|---------------|--------|------------|
| `second_brain` | `second_brain` | NT-MEMORY | KnowledgeIntegrator |
| `regression_test` | `regression_test` | NT-MIND | MetaCognitionAnalyst |
| `declarative_knowledge` | `declarative_knowledge` | NT-MIND | KnowledgeIntegrator |
| `procedural_recipes` | `procedural_recipes` | NT-MIND | KnowledgeIntegrator |

## Match Arm Behavior

### second_brain (organizational knowledge separation)
- Queries KB for layer counts (inbox/working/archive/public)
- Supports mode detection: organizational / personal / hybrid
- Domain: NT-MEMORY — knowledge layering and isolation

### regression_test (experience-tree regression testing)
- Reads `regression:last_run`, `pass_count`, `fail_count` from KB experience store
- Supports scope: full / delta / smoke
- Domain: NT-MIND — experience-tree self-test

### declarative_knowledge (SEAL declarative knowledge)
- Tracks declarative knowledge count via `seal:declarative:count`
- Supports categories: axiom / pattern / rule / fact
- Domain: NT-MIND — SEAL declarative knowledge accumulation

### procedural_recipes (SEAL procedural recipes)
- Reads `seal:procedural:count` and `seal:procedural:registry`
- Supports types: etl / build / deploy / general
- Domain: NT-MIND — SEAL procedural recipe registry

## Files Modified

- `neotrix-core/src/core/l5_consciousness/consciousness_core/dispatch.rs` — CAPABILITY_ROUTES + match arms
