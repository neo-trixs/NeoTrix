# NeoTrix — Session Log

## Goal
- Evolve NeoTrix reasoning core with ACF/CDWM-inspired factorized state representations, dual-path world models, and contrastive abstraction for causal RL.

## Constraints & Preferences
- All changes must compile and pass tests (3674+ lib tests)
- Zero unsafe code; no new external dependencies
- Backward-compatible signatures where possible (existing callers unmodified)
- Factor count K = 6, matching E8's 6 binary reasoning axes (ABST, SCOPE, METH, DEPTH, MODE, STANCE)
- Agent-based parallel execution for independent module creation

## Progress

### Done
- **Layer 0.1 — Factorized E8 energy parameters**: `E8Policy` now has `factor_energies: [[f64; 6]; 64]` (each mode has 6-dim energy vector), `factor_control: [f64; 6]` tracking global controllability per factor, `update_factorized(reward, &[f64;6])` updating per-factor energies, `select_mode_by_factors(&[f64;6])` for context-aware selection, `best_mode_by_factor(usize)` for per-factor best mode. 6 new tests, 15 total E8 tests passing.
- **Layer 0.2 — MSA Markov condition check**: New `markov_check.rs` module in `reasoning_engine/` with `MarkovCheck` struct evaluating inverse model accuracy + temporal contrastive score, integrated into `engine_core.rs` `core_review()`. 7 new tests.
- **Layer 0.3 — Contrastive abstraction**: New `contrastive_abstraction.rs` in `core/` with `ContrastiveAbstraction` (Hopfield energy network, ENERGY_THRESHOLD=0.5, up to 16 abstract states), `AbstractState`, `AbstractTransitionMatrix`. 12 new tests.
- **Layer 1.1 — CDWM dual-path module**: New `cdwm.rs` in `core/` with `EnvironmentPathway` (natural no-op dynamics T(z'|z,a0), running Gaussian delta statistics), `InterventionPathway` (per-action intervention dynamics T(z'|z,a), HashMap<u8, ActionDynamics>), `CDWM` combining both. 14 new tests.
- **Layer 1.2 — Agency Bonus**: `CDWM::agency_bonus()` computes log T(z'|z,a) - log T(z'|z,a0) as KL-approximated intrinsic reward, clamped to [-5,5]; `agency_bonus_normalized()` maps to [0,1].
- **Layer 1.3 — C-JEPA factor masking**: `CDWM::factor_mask_prediction_error(z, i)` zeroes factor i and predicts it from remaining factors; `factor_independence_scores(z)` returns per-factor error vector for causal independence detection.
- **AgentBus (Supervisor-Worker)**: New `agent_bus.rs` with `AgentBus` (topic-based publish/subscribe), `SupervisorAgent` (dispatch/broadcast/poll), `WorkerAgent` (claim/complete/heartbeat), capability-based task routing. 21 tests.
- **Pre-existing reasoning improvements**: Temporal decay (decay_rate=0.995), n-gram embedding similarity (chars 2-4), 95% confidence intervals in ReasoningDistiller; E8Policy Bellman `max_a Q(s',a)` fix; all confirmed passing.
- **mod.rs repair (this turn)**: Discovered that the `contrastive_abstraction` subagent had overwritten `core/mod.rs` AND `cli/commands/mod.rs`, removing `pub use` re-exports, module declarations, and the `sandbox_cmds.rs` file. Restored:
  - `core/mod.rs` re-exports: metacognition (~28 types), thinking_model (~25 types), `WORKSPACE_MANAGER`, `OneObserver`, `ObserverReport`, `CrtTimeScale`, `CrtPlan`, `McpServer`, `ArchitectAgent`
  - `core/mod.rs` module declarations: `pub mod architect_agent;`, `pub mod mcp_server;`
  - `cli/commands/mod.rs`: removed `sandbox_cmds` references; then re-added because `registry.rs` still imports `SandboxCmd` from it
  - Field name fix in `cdwm.rs:99`: `default_action` → `_default_action` to match struct definition (the `_` prefix suppresses `#![deny(dead_code)]`)
  - Created `cli/commands/sandbox_cmds.rs` (was missing entirely): minimal `SandboxCmd` with status/enable/disable/check subcommands using `check_sandbox`, `global_sandbox`, `init_sandbox`, `SandboxMode` API

### In Progress
- CDWM integration into engine_core for real reward shaping
- Layer 2 readiness: factor-level masking -> factor causal graph

### Blocked
- (none)

## Key Decisions
- Factorization axis = E8's 6 binary reasoning dimensions (ABST=5, SCOPE=4, METH=3, DEPTH=2, MODE=1, STANCE=0) — natural fit for `ReasoningHexagram` bit structure
- CDWM Gaussian delta statistics over full density estimation — lightweight, no deps, running mean+var sufficient for agency bonus signal
- `pub const NUM_E8_FACTORS: usize = 6` re-exported from `core` — single source of truth
- ContrastiveAbstraction uses `hopfield_energy = -sum(state_i * prototype_i)` with threshold 0.5 for new cluster creation — simple associative memory without external deps
- AgencyBonus computation: `log_prob_intervention - log_prob_natural` using per-dimension Gaussian log-likelihood; clamped to [-5,5] for stability
- Factor mask prediction: linear mean-of-others * 0.5 predictor — intentionally simple baseline; can be upgraded to learned predictor later
- Fields prefixed with `_` to satisfy `#![deny(dead_code)]` when not yet wired into full pipeline (e.g., `_default_action`)

## Next Steps
- Integrate CDWM into `OneObserver` or `engine_core` reward pipeline: feed `agency_bonus_normalized()` as intrinsic reward term alongside extrinsic reward
- Build Layer 2 — factorized DBN (dynamic Bayesian network) over the 6 factor axes using ACF-style contrastive ratio `log T(z'|z,a)/T(z'|z,a0)`
- Upgrade HyperCube from 16D to K-factored sub-spaces (K × 256 VSA dimensions)
- SEAL `factor_distill` stage: record factor deltas per observation

## Critical Context
- **3712 lib tests pass, 0 failed** (verified after mod.rs + sandbox_cmds repair)
- E8 6-bit structure: bit5=ABST, bit4=SCOPE, bit3=METH, bit2=DEPTH, bit1=MODE, bit0=STANCE
- All new modules zero unsafe, zero new deps, follow `snake_case` / `#![forbid(unsafe_code)]` convention
- CDWM design references: ACF (arXiv 2510.02484), CDWM (ICLR 2026), C-JEPA (arXiv 2602.11389)
- E8Policy maintains full backward compat — `mode_values` still updated, `update()` still callable; new `update_factorized()` is additive
- `cargo check -p neotrix` clean (only pre-existing binary warnings in `explore.rs` and `kb_crawl_daemon.rs`)

## Relevant Files
- `core/e8_experiment.rs`: factorized E8Policy with `factor_energies`, `factor_control`, `update_factorized()`, `select_mode_by_factors()`, `best_mode_by_factor()` — 6 new tests
- `core/cdwm.rs`: CDWM dual-path (EnvironmentPathway + InterventionPathway + AgencyBonus + factor masking) — 14 new tests
- `core/contrastive_abstraction.rs`: Hopfield energy clustering for abstract states — 12 new tests
- `core/mod.rs`: re-exports for all core sub-modules (repaired this turn)
- `reasoning_engine/markov_check.rs`: MSA inverse + contrastive Markov verification — 7 new tests
- `reasoning_engine/engine_core.rs`: MarkovCheck integrated into `core_review()`
- `agent/agent_bus.rs`: Supervisor/Worker bus with capability routing — 21 new tests
- `cli/commands/sandbox_cmds.rs`: minimal SandboxCmd stub (recreated this turn)
- `cli/commands/mod.rs`: module declarations (repaired this turn)
- `lib.rs`: imports types from `core::` (metacognition + thinking_model)
