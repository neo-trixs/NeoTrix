# Targeted Research #668: Test Quality Fixes

## Overview

Fixed test quality issues across NeoTrix safety subsystems. Tests were asserting on fabricated success data, hardcoded scores, or always-passing conditions.

## Files Modified

### 1. `neotrix-sim/src/safety/capability_tracker.rs`

**Issues Found:**
- `make_snapshot` helper used hardcoded values (social: 0.5, exploration: 0.5, cognition: 0.5, economy: 0.5, personality_stability: 0.9)
- `detects_severe_regression` used hardcoded values (0.8 → 0.3) without explaining the regression calculation
- `no_regression_below_threshold` used hardcoded values (0.8 → 0.72) without relating to config thresholds

**Fixes Applied:**
- Changed `make_snapshot` to accept all parameters explicitly
- Added `uniform_snapshot` helper for stable-baseline tests
- Updated `detects_severe_regression` to verify actual drop_pct calculation
- Updated `no_regression_below_threshold` to compute after value from config threshold
- Added `detects_mild_moderate_severe_by_threshold` to test all severity levels

### 2. `neotrix-sim/src/safety/evolution_constraints.rs`

**Issues Found:**
- `clamp_mutation_rate` asserted on hardcoded values (0.3, 0.01, 0.15) without referencing config
- `constrain_trait_delta_limits_change` asserted on hardcoded delta calculation
- `capability_thresholds_catch_violations` asserted on hardcoded violation count

**Fixes Applied:**
- Refactored `clamp_mutation_rate` to use config values in assertions
- Added `validate_genome_rejects_high_aggression` test for genome validation
- Updated `constrain_trait_delta_limits_change` to use config-derived thresholds
- Updated `capability_thresholds_catch_violations` to test both violation and clean cases
- Added `no_self_deletion_with_varied_actions` test

### 3. `neotrix-sim/src/safety/budget_enforcement.rs`

**Issues Found:**
- `detects_approaching_limit` had confusing comment and hardcoded values
- `get_usage_returns_metrics` only checked action count, not other metrics

**Fixes Applied:**
- Added `agent_with_actions` helper for consistent test setup
- Refactored tests to use parameterized thresholds
- Updated `get_usage_returns_metrics` to verify memory estimate is non-zero

### 4. `neotrix-sim/src/safety/safety_monitor.rs`

**Issues Found:**
- `monitor_records_actions` only checked key existence, not actual metrics
- `detects_personality_drift` used hardcoded personality values

**Fixes Applied:**
- Updated `monitor_records_actions` to verify total_actions, attack_count, social_count
- Added `no_drift_with_similar_personality` test for negative case
- Added descriptive assertions with failure messages

### 5. `neotrix-sim/src/safety/sandbox.rs`

**Issues Found:**
- `test_mutation_returns_fitness_delta` used weak assertion (delta.abs() < 100.0)
- Tests used hardcoded fitness values without verifying fitness calculation

**Fixes Applied:**
- Added `agent_with_fitness` helper
- Updated `test_mutation_returns_fitness_delta` to verify delta is finite and tested count increments
- Added fitness pre-conditions to promote tests

### 6. `neotrix-core/src/l1_action/nt_io/universal_model/capabilities.rs`

**Issues Found:**
- Tests asserted on hardcoded model capabilities (128,000 tokens, vision, tools)

**Fixes Applied:**
- Refactored to verify registered model exists and has expected capabilities
- Added `capability_detector_lookup_returns_none_for_unknown` test
- Added `find_by_task_returns_empty_for_unregistered_task` test
- Updated health checker tests to verify actual health state

### 7. `neotrix-sim/src/safety/anomaly.rs`

**Issues Found:**
- `make_agent` helper used hardcoded actions

**Fixes Applied:**
- Renamed to `agent_with_actions` for clarity
- Added `partial_behavior_change_not_anomalous` test
- Added descriptive assertion messages

### 8. `guard_core/src/safety/containment.rs`

**Issues Found:**
- Tests were minimal one-liners without descriptive messages

**Fixes Applied:**
- Expanded tests with descriptive names and assertion messages
- Added `boundary_zone_empty_prefixes_denies_all` test

## Test Quality Patterns Applied

1. **Parameterized thresholds**: Use config values in assertions instead of hardcoded numbers
2. **Pre-condition verification**: Verify setup conditions before testing behavior
3. **Descriptive assertions**: Include failure messages explaining what went wrong
4. **Helper functions**: Create reusable helpers for common test setups
5. **Negative testing**: Add tests for cases that should NOT trigger alerts
6. **Boundary testing**: Test at threshold boundaries (exactly at, just above, just below)

## Remaining TODOs

- Some tests still use simplified simulations (sandbox tests)
- Real integration tests needed for cross-module interactions
- Performance regression tests not yet implemented
