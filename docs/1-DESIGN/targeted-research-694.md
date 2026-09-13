# Targeted Research #694: Documentation Pass for Priority Modules

**Date**: 2026-09-13
**Scope**: L1 IO Provider, L5 Mind Modules, L6 Meta Coordination

## Summary

Added `/// Note:` doc comments to ~40 undocumented or under-documented functions across
3 priority directories. Each comment explains what a real implementation needs beyond
the current stub/simplified version.

## Files Modified

### L1 Action / NT-IO Provider

| File | Functions Documented | Key Notes |
|------|---------------------|-----------|
| `gateway/routing/selection.rs` | `states_write`, `default_name_write`, `register_provider`, `register_provider_with_category`, `select_best`, `provider_status` | RwLock poison recovery, composite scoring, provider registration |
| `gateway/routing/learned_router.rs` | `cosine_sim`, `features_to_vec` (x2), `fast_tier_pick`, `decision_from`, `_create_router` | SIMD optimization needs, ANN indexes, feature normalization |
| `gateway/routing/intelligence.rs` | `record`, `predict_latency`, `predict_success_rate`, `route`, `update_weight` | Percentile estimation, EMA smoothing, multi-armed bandit |
| `gateway/execution.rs` | `call_provider`, `complete_with_account_pool`, `keyless_candidates`, `route_keyless`, `is_maintenance_window`, `exponential_backoff`, `is_model_unavailable` | Middleware stack, account rotation, error classification |
| `gateway/resilience.rs` | `should_allow`, `record_failure`, `record_success`, `record_metric`, `detect`, `heal` | Circuit breaker state machine, z-score anomaly, JSON healing |

### L5 Cognition / NT-Mind Mind Modules

| File | Functions Documented | Key Notes |
|------|---------------------|-----------|
| `seal/seal_enhanced.rs` | `adapt_learning_rate` | Learning rate scheduling, adjustment history |

### L6 Meta / Coordination

| File | Functions Documented | Key Notes |
|------|---------------------|-----------|
| `governance.rs` | `register_default_rules`, `check`, `check_rule` | Rule loading, AST parsing, build system integration |
| `cross_module_audit.rs` | `check_dynamic_emotion_consistency`, `check_rhythm_segment_consistency`, `check_parameter_bounds`, `calculate_consistency_score` | Fuzzy matching, configurable thresholds, weighted scoring |
| `quality_gate.rs` | `calculate_total_score`, `check_passed`, `get_history`, `statistics` | Per-level thresholds, KB persistence, telemetry |

## Recurring Patterns in Documentation Notes

1. **Hardcoded thresholds** — Many functions use magic numbers (0.7, 40%, 2.5 z-score)
   that should be configurable or adaptive.

2. **String-based error classification** — `is_maintenance_window`, `is_model_unavailable`
   parse error messages with `contains()`. Structured error codes from provider APIs
   would be more reliable.

3. **In-memory-only state** — Circuit breakers, anomaly detectors, and review history
   are lost on restart. Consider KB-backed persistence for production.

4. **Simple heuristics** — Scoring functions use linear combinations. Multi-armed bandit
   approaches (Thompson sampling, UCB) would provide better exploration/exploitation.

5. **Missing telemetry integration** — Many functions log locally but don't feed into
   the global EventBus or telemetry system for cross-module observability.

## STUB Functions Identified

| File | Function | Status |
|------|----------|--------|
| `learned_router.rs` | `_create_router` | Not wired into production |
| `seal_enhanced.rs` | `explore`, `distill`, `absorb` | Return `Err("not wired")` |
| `quality_gate.rs` | `_ai_initial_review` | Scores externally provided, no real VLM |
| `bpco.rs` | `critique` | C0 stub, returns score=0.0 |
