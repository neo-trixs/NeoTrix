# Module Integration Map

## Dead Modules Found (2026-09-11)

| Module | File | Declared | Used | Integration Point | Priority |
|--------|------|----------|------|-------------------|----------|
| `addressable_store` | `l1_action/nt_memory/addressable_store.rs` | mod.rs | **NO** | Wire into `ContextSandbox` (citation_id generation) | P1 |
| `context_fs` | `l1_action/nt_memory/context_fs.rs` | mod.rs | **NO** | Wire into `nt_memory` (context organization) | P2 |
| `trinity` | `l1_action/nt_memory/trinity.rs` | mod.rs | **NO** | Already used by `shared_utils` consumers | P3 |
| `memory_types` | `l1_action/nt_memory/memory_types.rs` | mod.rs | **YES** (bins) | Already wired | - |
| `shared_utils` | `l1_action/nt_memory/shared_utils.rs` | mod.rs | **YES** | Already wired | - |
| `hooks` | `l1_action/nt_io/hooks.rs` | mod.rs | **NO** (separate agent hooks) | Wire into `nt_io_neocodex` lifecycle | P1 |
| `acp` | `l1_action/nt_io/acp.rs` | mod.rs | **NO** (separate neocodex acp) | Already has neocodex integration | P3 |
| `cache_compaction` | `l1_action/nt_io/cache_compaction.rs` | mod.rs | **NO** | Wire into `nt_io` context management | P2 |
| `mcp_server` | `l1_action/nt_io/mcp_server.rs` | mod.rs | **YES** (CLI) | Already wired | - |
| `context_sandbox` | `l1_action/nt_io/context_sandbox.rs` | mod.rs | **NO** | Wire into agent loop tool output | P1 |
| `procedural_graph` | `l5_cognition/nt_mind/seal/procedural_graph.rs` | mod.rs | **NO** | Wire into SEAL pipeline | P1 |
| `distillation` | `l5_cognition/nt_mind/seal/distillation.rs` | mod.rs | **NO** | Wire into SEAL pipeline | P2 |
| `crystallization` | `l5_cognition/nt_mind/seal/crystallization.rs` | mod.rs | **YES** (bg loop) | Already wired | - |
| `aegis` | `l5_cognition/nt_mind/seal/aegis.rs` | mod.rs | **NO** | Wire into SEAL pipeline | P1 |
| `harness_evolution` | `l5_cognition/nt_mind/seal/harness_evolution.rs` | mod.rs | **YES** (harness.rs) | Already wired | - |
| `harness_optimizer` | `l5_cognition/nt_mind/seal/harness_optimizer.rs` | mod.rs | **NO** | Wire into SEAL pipeline | P1 |
| `context_assembly` | `l5_cognition/nt_core/context_assembly.rs` | mod.rs | **NO** | Wire into GWT attention budget | P1 |
| `io_contract` | `core/nt_core_self/io_contract.rs` | mod.rs | **NO** | Wire into tool validation | P2 |
| `reference_view` | `l1_action/nt_act/reference_view.rs` | mod.rs | **NO** | Wire into tool output handling | P2 |
| `context_boundary` | `l3_embodiment/nt_shield/nt_shield/context_boundary.rs` | mod.rs | **NO** | Wire into SecurityManager | P1 |
| `refinement` | `l5_cognition/nt_mind/harness/refinement.rs` | mod.rs | **NO** | Wire into nt_mind background loop | P1 |

## Priority Wiring Summary

**P1 (Critical — 7 modules):**
1. `context_boundary` → SecurityManager (CPE attack prevention)
2. `harness_optimizer` → SEAL pipeline (token savings)
3. `refinement` → nt_mind background loop (harness evolution)
4. `context_assembly` → GWT attention budget (context optimization)
5. `addressable_store` → ContextSandbox (citation generation)
6. `procedural_graph` → SEAL pipeline (skill execution graph)
7. `aegis` → SEAL pipeline (4-stage evolution)

**P2 (Important — 5 modules):**
8. `context_fs` → nt_memory (context organization)
9. `cache_compaction` → nt_io (cache-aware compression)
10. `io_contract` → tool validation
11. `reference_view` → tool output handling
12. `distillation` → SEAL pipeline

**P3 (Already wired or low priority — 3 modules):**
13. `trinity` → already used via shared_utils
14. `acp` → already has neocodex integration
15. `memory_types` → already used by bins
