# Pre-Cleanup Archive (2026-09-17)

## What Changed

This archive preserves the state of `core/mod.rs` before the Phase 1 cleanup that
removed all backward-compat `pub use` re-exports.

### core/mod.rs Before (201 lines)

The original `core/mod.rs` contained:
- **120+ `pub use` re-export lines** mapping `crate::core::*` → layer paths
- **13 `pub mod` declarations** for infrastructure modules that physically live in core/
- **35+ type re-exports** (NeoTrixError, CapabilityVector, ReasoningHexagram, etc.)

### What Was Removed

All `pub use` re-exports were removed. Consumers now import directly from layer paths:
- `crate::core::nt_core_hcube` → `crate::l2_perception::nt_core_hcube`
- `crate::core::nt_core_self` → `crate::l6_meta::nt_core_self`
- `crate::core::nt_core_consciousness` → `crate::l5_cognition::nt_core_consciousness`
- etc.

### What Remains in core/

Only infrastructure modules that physically live in `core/`:
- `nt_core_di` — Dependency injection
- `nt_core_axiom_tree` — Architecture axioms
- `nt_core_cap` — Capability primitives
- `nt_core_event` — Event system
- `nt_core_traits` — Core trait definitions
- `nt_core_span` — Distributed tracing
- `nt_core_platform` — Platform initialization
- `nt_core_telemetry` — Telemetry
- `nt_core_schema_watchdog` — Schema drift detection

### Files in This Archive

| File | Description |
|------|-------------|
| `core_mod.rs.original` | Original core/mod.rs with all re-exports (201 lines) |
