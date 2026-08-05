# Archived: L7 GreatFilterGate (`l7_capability_gate.rs`)

**Source**: `core/l7_capability/gate.rs` (133 lines + 4 tests, zero production callers)

**Claimed duty**: "调度必经 4 道大过滤器" (permission → budget → circuit → humility).

**Archival rationale (evidence-based)**:
- All input metrics lack production sources: `Capability.cost` (only constructed in tests),
  `BudgetState` (never constructed), `illusion_risk` (from dormant `TurkeyScientist`, zero callers),
  `p_value` (no significance-testing producer anywhere in the crate).
- Responsibilities already covered by wired production implementations:
  - Budget → L1 `rate_limiter` (`nt_io_provider/rate_limiter.rs`)
  - Humility/illusion → `nt_core_gate::GuardrailReport` (`GuardrailReport::evaluate`)
  - Promotion/maturity → `Constellation::derive` (`core/nt_core_consciousness_tree.rs`)
  - Overall adjudication → `nt_core_gate::GateDecision` (wired in `meta_panel/engine.rs`,
    `handlers_core.rs:31-49`, `run.rs:434`)
- Incomplete implementation: `GateCheck::Circuit` unused, `check_permission` is a stub,
  `max_exploration_bias` is a dead field.

**If resurrected**: fold the `verify_promotion` (illusion_risk + p_value) semantics into
`nt_core_gate::GateDecision` as a new Promotion dimension, once a production illusion-observability
chain and significance statistics exist. Do not revive this module standalone.
