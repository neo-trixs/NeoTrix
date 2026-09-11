# Cross-Layer Import Scan — Cycle 300

Scan time: 2026-09-11 18:02
Scope: `neotrix-core/src/` 6-layer directories (l1_action → l6_meta)
Exclusions: `*_test.rs`, `tests.rs`, `integration_tests`, `*facade*`, `mock`, `bench`, `example`

## Architecture Layers

| Layer | Directory | Role |
|-------|-----------|------|
| L1 Action | l1_action/ | nt_act + nt_io + nt_memory |
| L2 Perception | l2_perception/ | nt_world + nt_sense |
| L3 Embodiment | l3_embodiment/ | nt_physical + nt_shield + nt_feel |
| L4 Emotion | l4_emotion/ | nt_feel (core emotion engine) |
| L5 Cognition | l5_cognition/ | nt_core + nt_mind |
| L6 Meta | l6_meta/ | nt_meta + nt_repair + nt_nexus |

## Dependency Direction Rule

```
L6 Meta       -> L5 L4 L3 L2 L1  (downward, OK)
L5 Cognition  -> L4 L3 L2 L1     (downward, OK)
L4 Emotion    -> L3 L2 L1        (downward, OK)
L3 Embodiment -> L2 L1           (downward, OK)
L2 Perception -> L1              (downward, OK)
L1 Action     -> (none)          (leaf, no imports allowed)
```

**Upward imports (lower -> higher) are architecture violations.**

## Full Cross-Layer Matrix (non-facade, non-test)

| Source \ Target | l1_action | l2_perception | l3_embodiment | l4_emotion | l5_cognition | l6_meta |
|----------------|-----------|---------------|---------------|------------|--------------|---------|
| **l1_action** | -- | 0 | **3** ! | 0 | **2** ! | 0 |
| **l2_perception** | 62 OK | -- | **71** ! | 0 | 0 | 0 |
| **l3_embodiment** | 3 OK | 0 | -- | 0 | 0 | 0 |
| **l4_emotion** | 0 | 0 | 0 | -- | 0 | 0 |
| **l5_cognition** | 115 OK | 3 OK | 12 OK | 0 | -- | **22** ! |
| **l6_meta** | 1 OK | 0 | 0 | 0 | 15 OK | -- |

OK = downward (allowed), ! = upward (violation)

## Summary

| Metric | Count |
|--------|-------|
| **Downward imports** (expected) | 211 |
| **Upward violations** (architecture breach) | **98** |
| **Total cross-layer refs** | 309 |

## Upward Violations (98 total)

### 1. l2_perception -> l3_embodiment: 71 (UPWARD)

**Pattern**: Every world data source (nt_world_*.rs) defines its own `*_egress_rule()` and `*_egress_policy()` by directly importing `crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule`.

**Files** (21 sources x ~3-4 refs each):
- `nt_world_urlhaus.rs:267-277` -- urlhaus/cisa_kev egress rules
- `nt_world_usgs.rs:338-342` -- usgs egress rules
- `nt_world_bgpview.rs:151-155` -- bgpview egress rules
- `nt_world_gdelt.rs:209-216,328` -- gdelt egress rules
- `nt_world_ucdp.rs:274-278` -- ucdp egress rules
- `nt_world_aoi.rs:269-273` -- aoi egress rules
- `nt_world_gdacs.rs:194-198` -- gdacs egress rules
- `nt_world_ofac.rs:180-184` -- ofac egress rules
- `nt_world_polymarket.rs:158-162` -- polymarket egress rules
- `nt_world_opencorporates.rs:156-160` -- opencorporates egress rules
- `nt_world_adsb.rs:175-179` -- adsb egress rules
- `nt_world_edgar.rs:434-439,599` -- edgar egress rules
- `osint/fofa.rs:549-554` -- fofa egress rules
- `osint/shodan.rs:173-177` -- shodan egress rules
- `osint/zoomeye.rs:143-147` -- zoomeye egress rules
- `osint/censys.rs:141-145` -- censys egress rules

**Root cause**: No shared egress rule registry. Each data source hardcodes its own `EgressRule::allow(HOST, "443")`.

**Fix**: Extract a `nt_world_egress_registry` module that centralizes all source egress rules. l2_perception defines rules locally; l3_embodiment consumes them via trait, not direct type import.

### 2. l5_cognition -> l6_meta: 22 (UPWARD)

**Pattern**: Background loop handlers and SEAL pipeline directly instantiate l6_meta types.

**Files**:
- `l6_facade.rs:11-17` -- re-exports of `ConsciousnessGoldStandard`, `ConsciousnessMonitor`, `EvalHarness`, `EvolutionHarness`, `LoopConfig` (15 refs)
- `handlers_consciousness.rs:266-276,962,1591-1592,1764,1791,2030` -- direct instantiation of l6_meta types (5 refs)
- `handlers_maintenance.rs:940` -- `use crate::l6_meta::coordination::self_improvement::SystemMetrics`
- `pipeline.rs:3079,3102` -- `ConsciousnessMonitor::new()`, `ConsciousnessGoldStandard::new()`

**Root cause**: l5_cognition needs l6_meta types for self-test registration and consciousness monitoring, but no abstraction layer exists.

**Fix**: Define traits in l5_cognition (`ConsciousnessMonitorApi`, `GoldStandardApi`, etc.) that l6_meta implements. l5_cognition depends on traits, not concrete types.

### 3. l1_action -> l3_embodiment: 3 (UPWARD)

**Files**:
- `l1_action/nt_io/nt_io_provider/factory.rs:530-535` -- `PolicyDecision` match arms

```rust
crate::l3_embodiment::nt_shield::nt_shield::policy::PolicyDecision::Allow => true,
crate::l3_embodiment::nt_shield::nt_shield::policy::PolicyDecision::RequireConfirmation => { ... },
crate::l3_embodiment::nt_shield::nt_shield::policy::PolicyDecision::Deny => { ... },
```

**Root cause**: LLM provider factory directly checks shield policy decisions.

**Fix**: Define `PolicyDecision` enum in l1_action (shared types) or use a callback/closure pattern so l1_action doesn't import l3 types.

### 4. l1_action -> l5_cognition: 2 (UPWARD)

**Files**:
- `l1_action/nt_io/nt_io_neocodex/agent.rs:110,233` -- `SelfIteratingBrain` type reference

```rust
Arc<tokio::sync::RwLock<crate::l5_cognition::nt_mind::nt_mind::self_iterating::SelfIteratingBrain>>,
```

**Root cause**: Neocodex agent directly couples to cognition brain type.

**Fix**: Define a trait `BrainApi` in l1_action that l5_cognition implements.

## Circular Dependency: l5_cognition <-> l6_meta

While l6_meta -> l5_cognition is downward (allowed), it creates a **circular dependency** with the upward l5_cognition -> l6_meta (22 refs):

- l5_cognition imports l6_meta concrete types: `ConsciousnessMonitor`, `GoldStandard`, `EvalHarness`, `EvolutionHarness`, `LoopConfig`
- l6_meta imports l5_cognition trait definitions: `EvolutionHarnessApi`, `GoldStandardApi`, `EvalHarnessApi`, `ConsciousnessMonitorApi`, `RegistryNodeInfo`

**Fix**: Extract shared traits to a `nt_types` crate or `l1_action::nt_l1_shared_types` module, breaking the cycle.

## Downward Imports (211 total, expected)

| Route | Count | Primary Types |
|-------|-------|---------------|
| l5_cognition -> l1_action | 115 | `KnowledgeBase`, `NodeType`, `fetch_safe_http`, `OracleGate`, `SemanticEntropyGate`, `ActionSandbox`, `SessionRecoveryManager`, `ProjectSnapshot` |
| l2_perception -> l1_action | 62 | `KnowledgeBase`, `NodeType`, `CrawlCycleReport`, `DiscoveryPipelineConfig`, `DownloadOptions`, `http` utils |
| l5_cognition -> l3_embodiment | 12 | `create_reasoning_trace_auditor`, `CohGuard`, `browser_security`, `check_registry`, `tool_inspection_stack` |
| l6_meta -> l5_cognition | 15 | `EvolutionHarnessApi`, `RegistryNodeInfo`, `GoldStandardApi`, `EvalHarnessApi`, `ConsciousnessMonitorApi` |
| l5_cognition -> l2_perception | 3 | `WorldModelV2`, `UnifiedSearch`, `ExplorationEngine` |
| l3_embodiment -> l1_action | 3 | `LlmResponse`, `Usage`, `GatewayV2`, `KnowledgeBase` |
| l6_meta -> l1_action | 1 | `KnowledgeBase` |

## l4_emotion: Isolated

l4_emotion has **zero** cross-layer imports (excluding one commented-out line). This layer is fully isolated.

## Prioritized Fix Plan

| Priority | Violation | Count | Fix Strategy |
|----------|-----------|-------|-------------|
| **P0** | l2->l3 egress rules | 71 | Centralize egress rule registry in l2_perception; l3 consumes via trait |
| **P1** | l5->l6 direct types | 22 | Define monitoring traits in l5_cognition; l6_meta implements them |
| **P1** | l5<->l6 trait circular | 15 | Extract shared traits to a types crate or l1_action shared module |
| **P2** | l1->l3 policy | 3 | Move PolicyDecision to l1_action shared types or use callback pattern |
| **P2** | l1->l5 brain | 2 | Define BrainApi trait in l1_action; l5_cognition implements it |
