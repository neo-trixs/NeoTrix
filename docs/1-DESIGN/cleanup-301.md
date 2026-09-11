# cleanup-301: Cross-Layer Import Scan

> 2026-09-11 | neotrix-core/src/ 6-layer scan | exclude test/facade

## Scope

Scanned `use crate::l{1-6}_*` across all `.rs` files in the 6 layers:
`l1_action`(387) → `l2_perception`(209) → `l3_embodiment`(177) → `l4_emotion`(6) → `l5_cognition`(345) → `l6_meta`(59)

## Layer Compliance Summary

| Layer | Direct cross-layer imports | Facade-mediated | Status |
|-------|---------------------------|-----------------|--------|
| l1_action | 0 | n/a | CLEAN |
| l2_perception | 0 | n/a | CLEAN |
| l3_embodiment | 0 | via l1_facade (6) | CLEAN |
| l4_emotion | 0 | n/a | CLEAN |
| l5_cognition | **5 violations** | 33 via facades | **VIOLATIONS** |
| l6_meta | 0 | via l1_facade (1) | CLEAN |

## Violations Detail

### V1: l5_cognition → l1_action (DOWN, 2 locations)

**File**: `l5_cognition/nt_mind/nt_mind_background_loop/handlers_maintenance.rs:679-680`
```rust
use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_community::{CommunityDetector, CommunityAwareSearch};
use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_store::{
    get_all_nodes, get_all_edges, ensure_domain_cluster, update_cluster_stats,
};
```
**Context**: `handle_clustering()` — community detection for GWT attention routing.
**Fix**: Route through `kb_facade` (already has 13 consumers). Add `CommunityDetector`/`CommunityAwareSearch`/`get_all_nodes`/`get_all_edges`/`ensure_domain_cluster`/`update_cluster_stats` to `kb_facade` re-exports.

---

**File**: `l5_cognition/nt_mind/evolution/self_diagnose.rs:10`
```rust
use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot;
```
**Context**: Self-diagnosis reads project snapshot for issue thresholding.
**Fix**: Add `ProjectSnapshot` to `act_facade` re-exports.

---

**File**: `l5_cognition/nt_mind/evolution/evolution_loop.rs:67`
```rust
pub use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot;
```
**Context**: Re-exports `ProjectSnapshot` to other l5 modules.
**Fix**: Remove `pub use` from here; consumers use `act_facade::ProjectSnapshot` instead.

### V2: l5_cognition → l6_meta (UP, 1 location)

**File**: `l5_cognition/nt_mind/nt_mind_background_loop/handlers_maintenance.rs:940`
```rust
use crate::l6_meta::coordination::self_improvement::SystemMetrics;
```
**Context**: `handle_self_improvement()` —采集系统指标 for meta-improvement loop.
**Fix**: Add `SystemMetrics` to `l6_facade` re-exports (already has 6 consumers).

## Facade Usage (Correct Pattern)

L5 cognition properly uses facades for 33 cross-layer accesses:

| Facade | Consumers | Purpose |
|--------|-----------|---------|
| `kb_facade` | 13 | KB knowledge ops |
| `l6_facade` | 6 | Consciousness monitoring |
| `l2_facade` | 6 | World model / search |
| `io_facade` | 4 | LLM provider / reasoning |
| `act_facade` | 3 | Crypto agent / recipes |
| `l3_facade` | 1 | Shield / safety |

## Recommended Fixes (Priority Order)

1. **V1-handles_maintenance**: Add community detection APIs to `kb_facade`
2. **V1-self_diagnose/evolution_loop**: Add `ProjectSnapshot` to `act_facade`
3. **V2-handles_maintenance**: Add `SystemMetrics` to `l6_facade`

All fixes are facade re-export additions — no logic changes required.
