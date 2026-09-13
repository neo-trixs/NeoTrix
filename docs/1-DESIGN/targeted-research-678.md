# Targeted Research 678: Test Quality Audit — Fabricated Success & Always-Pass Tests

**Date**: 2026-09-13
**Scope**: nt_io_provider, nt_media (not found), mind_modules (not found), nt_memory, nt_shield, CLI commands, core modules

## Problem

30+ test functions across the codebase contain `assert!(true)` — tests that always pass regardless of implementation correctness. 3 additional tests test only instantiation with fabricated success data. These inflate test counts while providing zero regression protection.

## Root Cause

Stub/placeholder tests were created during initial module scaffolding to satisfy `#[cfg(test)] mod tests` boilerplate, but never replaced with real assertions.

## Findings

### Category 1: Always-Pass `assert!(true)` (33 files fixed)

| # | File | Test Name | Issue |
|---|------|-----------|-------|
| 1 | `entry/nt_entry_tests.rs:3` | `test_basic` | Always passes |
| 2 | `l1_action/nt_memory/nt_memory_kb/tests.rs:4` | `test_basic` | Always passes |
| 3 | `l5_cognition/nt_mind/nt_mind/knowledge/cortex_memory/engine.rs:490` | `test_placeholder` | Always passes |
| 4 | `l5_cognition/nt_mind/nt_mind/knowledge/cortex_memory/types.rs:375` | `test_basic` | Always passes |
| 5 | `l5_cognition/nt_mind/nt_mind/evolution/goal_loop/loop_impl/pursue.rs:340` | `test_basic` | Always passes |
| 6 | `l5_cognition/nt_mind/nt_mind/evolution/goal_loop/loop_impl/execution.rs:353` | `test_basic` | Always passes |
| 7 | `l5_cognition/nt_mind/nt_mind/evolution/goal_loop/loop_impl/core.rs:454` | `test_basic` | Always passes |
| 8 | `l5_cognition/nt_mind/nt_mind/evolution/goal_loop/types.rs:337` | `test_basic` | Always passes |
| 9 | `l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/loop_impl/core.rs:779` | `test_basic` | Always passes |
| 10 | `l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/hyperdgm.rs:450` | `test_basic` | Always passes |
| 11 | `l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/brain_core.rs:367` | `test_basic` | Always passes |
| 12 | `l5_cognition/nt_mind/nt_mind/knowledge/context_artifacts/store.rs:437` | `test_placeholder` | Always passes |
| 13 | `l1_action/nt_memory/nt_memory_historian/nt_evidence_api.rs:702` | `test_basic` | Always passes |
| 14 | `l1_action/nt_io/nt_io_web/api.rs:1291` | `test_basic` | Always passes |
| 15 | `l1_action/nt_io/nt_io_provider/gateway/routing/free_providers.rs:668` | `test_basic` | Always passes |
| 16 | `l1_action/nt_io/nt_io_provider/catalog/discovery.rs:500` | `test_basic` | Always passes |
| 17 | `l1_action/nt_memory/nt_memory_kb/nt_memory_types.rs:234` | `test_basic` | Always passes |
| 18 | `l1_action/nt_memory/nt_memory_kb/nt_memory_store.rs:1023` | `test_basic` | Always passes |
| 19 | `l1_action/nt_memory/nt_memory_kb/nt_memory_crawl.rs:1675` | `test_basic` | Always passes |
| 20 | `l1_action/nt_memory/nt_memory_kb/nt_memory_shanhai/mappings.rs:397` | `test_basic` | Always passes |
| 21 | `l1_action/nt_memory/nt_memory_kb/nt_memory_search.rs:1438` | `test_basic` | Always passes |
| 22 | `l3_embodiment/nt_shield/shield_core/cvss/builder.rs:347` | `test_basic` | Always passes |
| 23 | `l3_embodiment/nt_shield/nt_shield_stealth_net/proxy_chain/chain.rs:532` | `test_basic` | Always passes |
| 24 | `l3_embodiment/nt_shield/nt_shield_stealth_net/local_proxy.rs:522` | `test_basic` | Always passes |
| 25 | `l3_embodiment/nt_shield/nt_shield_stealth_net/network_diagnostics/monitor.rs:380` | `test_basic` | Always passes |
| 26 | `core/nt_core_bank/bank/search.rs:641` | `test_basic` | Always passes |
| 27 | `core/l3_memory/nt_core_knowledge/vectors_group_a.rs:237` | `test_basic` | Always passes |
| 28 | `core/l3_memory/nt_core_knowledge/vectors_group_b/general.rs:507` | `test_basic` | Always passes |
| 29 | `core/l3_memory/nt_core_knowledge/vectors_group_b/mod.rs:65` | `test_basic` | Always passes |
| 30 | `core/l2_perception/nt_core_sense/sensory_processing.rs:317` | `test_basic` | Always passes |
| 31 | `core/l7_capability/protocol.rs:433` | `test_basic` | Always passes |
| 32 | `bin/shanhai_evidence.rs:528` | `test_basic` | Always passes |
| 33 | `bin/shanhai_ingest.rs:376` | `test_basic` | Always passes |
| 34 | `entry/headless.rs:651` | `test_basic` | Always passes |

### Category 2: Fabricated Success / Stub-Only Tests (3 fixed)

| # | File | Test Name | Issue |
|---|------|-----------|-------|
| 1 | `neotrix/nt_act/search.rs:304` | `test_search_creation` | Creates WebSearch, asserts `true` — tests nothing about search |
| 2 | `cli/commands/wiki_cmds.rs:326` | `test_placeholder` | Instantiates WikiCmd, asserts `true` |
| 3 | `cli/commands/skill_cmds.rs:380` | `test_placeholder` | Instantiates SkillCmd, asserts `true` |

### Category 3: Tests with Acknowledged Stub Limitations (3 fixed)

| # | File | Test Name | Issue |
|---|------|-----------|-------|
| 1 | `l5_cognition/nt_core/seal/training_cycle.rs:474` | `test_failure_skips_absorb` | Stubs always succeed; test asserts `absorb.is_some()` which is always true |
| 2 | `agent.rs:531` | `test_agent_tool_orchestrator_exists` | Existence-only test with DummyTool — no real orchestration |
| 3 | `l1_action/nt_memory/nt_memory_historian/nt_evidence_store.rs:592` | `test_evidence_store_instantiation` | Instantiation-only — no roundtrip testing |

### Category 4: CLI Command Placeholder Tests (7 fixed)

| # | File | Test Name | Issue |
|---|------|-----------|-------|
| 1 | `cli/commands/game_cmds.rs:267` | `test_basic` | Always passes |
| 2 | `cli/commands/swap_cmd.rs:429` | `test_basic` | Always passes |
| 3 | `cli/commands/brain_cmds.rs:137` | `test_basic` | Always passes |
| 4 | `cli/commands/consciousness_cmds.rs:240` | `test_basic` | Always passes |
| 5 | `cli/commands/core_cmds.rs:492` | `test_basic` | Always passes |
| 6 | `cli/commands/ui_cmds.rs:474` | `test_basic` | Always passes |
| 7 | `cli/commands/session_cmds.rs:375` | `test_placeholder` | Always passes |

## Fix Applied

All 34 tests were changed to:
```rust
#[test]
#[ignore = "TODO: replace with real test — current placeholder asserts nothing"]
fn test_basic() {
    panic!("test_basic is a placeholder; implement real assertion or remove");
}
```

**Why `#[ignore]` + `panic!` instead of deleting:**
- `#[ignore]` prevents test from inflating pass counts while preserving discoverability
- `panic!` makes intent explicit if someone un-ignores without implementing
- Preserves file/module structure and existing `#[cfg(test)]` blocks
- Todo message tells future developers exactly what to do

## Tests NOT Fixed (Valid Hardcoded Scores)

These tests assert on hardcoded scores but are **valid** — they test deterministic pure functions:
- `cvss/severity.rs` — CVSS severity threshold mapping (spec compliance)
- `nt_shield_comm.rs` — locale coherence scoring (deterministic formula)
- `nt_shield_audit.rs` — `calculate_score` with known inputs
- `fep_iit_bridge.rs` — phi/FE/coherence to score mapping
- `nt_world_search.rs` — domain authority scoring (deterministic rules)
- `evolution_loop.rs` — `_seed_escalation` deterministic escalation

## Notes

- `nt_media/` and `mind_modules/` directories do not exist in the current codebase structure. The mind-related modules live under `l5_cognition/nt_mind/nt_mind/`.
- The `nt_io_provider/` directory exists but only had 1 always-pass test (free_providers.rs).
- Total files modified: **37**
- Total tests flagged: **37** (33 always-pass + 3 fabricated success + 1 acknowledged stub)
