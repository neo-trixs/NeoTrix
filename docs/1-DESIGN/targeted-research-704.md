# Targeted Documentation Research #704

## Summary

Added `/// Note:` and `/// STUB:` doc comments to undocumented functions across 3 priority areas in NeoTrix. Focus on functions missing doc comments, having `// TODO` without explanation, or containing complex logic without comments.

## Files Modified

### L1 Action — nt_io_provider (5 files)

| File | Functions Documented | Key Findings |
|------|---------------------|--------------|
| `catalog/model_pool.rs` | `local_gguf()`, `cloud_free()` | Hardcoded base_url, no GGUF header parsing |
| `health/circuit_breaker.rs` | `new()`, `health_penalty()`, `force_open_secs()`, `on_success()`, `on_failure()`, `failure_rate()`, `cooldown_reset()` | Missing HalfOpen→Closed transition, no EventBus events |
| `health/compaction.rs` | `sanitize_history()` | O(n²) backward scanning, needs HashMap optimization |
| `gateway/routing/selection.rs` | `build_candidate_chain()`, `register_from_catalog()`, `register_from_unified_pool()`, `resolve_default_model_sync()` | Common registration logic duplicated, no lazy key resolution |
| `gateway/routing/learned_router.rs` | `KNNRouter::route()`, `KNNRouter::update()`, `MLPRouter::forward()`, `MLPRouter::route()`, `MLPRouter::update()`, `HybridRouter::route()`, `MultiTurnRouter::route()`, `MultiTurnRouter::route_multi_turn()` | Random-initialized MLP weights, no backpropagation, no exploration epsilon |

### L5 Cognition — nt_mind (2 files)

| File | Functions Documented | Key Findings |
|------|---------------------|--------------|
| `seal/seal_enhanced.rs` | `_execute_cycle()`, `self_test()`, `adapt_learning_rate()` | All 4 SEAL stages return `Err("not wired")`, no real backends |
| `knowledge/absorption_registry.rs` | `_register_absorber()`, `unregister()`, `_trigger_absorption()`, `get_stats()` | In-memory only, no KB persistence, no hot-reload |

### L6 Meta — coordination (3 files)

| File | Functions Documented | Key Findings |
|------|---------------------|--------------|
| `quality_gate.rs` | `new()`, `_ai_initial_review()`, `_manual_review()`, `_platform_final_review()` | AI review is STUB (caller-provided scores), hardcoded dimensions |
| `cross_module_audit.rs` | `check()`, `check_dynamic_emotion_consistency()`, `check_rhythm_segment_consistency()`, `check_parameter_bounds()`, `calculate_consistency_score()` | Sequential checks, hardcoded thresholds, no weighted scoring |
| `self_improvement.rs` | `run_cycle()`, `_generate_plans()`, `_evaluate_and_apply()`, `rollback()`, `verify()`, `actions_for_dimension()`, `severity_to_priority()`, `prune_history()`, `update_trends()` | Execution engine STUB (plans marked Skipped), no parameter snapshots |

## STUB Functions Identified

| Function | Location | What's Missing |
|----------|----------|----------------|
| `cooldown_reset()` | circuit_breaker.rs | Periodic background loop caller, HalfOpen→Closed transition |
| `forward()` | learned_router.rs | Pre-trained weights, GPU acceleration, softmax activation |
| `MLPRouter::update()` | learned_router.rs | Backpropagation, gradient computation, learning rate scheduling |
| `_execute_cycle()` | seal_enhanced.rs | LLM/research API for exploration, distillation, KB write for absorption |
| `_register_absorber()` | absorption_registry.rs | Plugin lifecycle hooks, capability-based routing, hot-reload |
| `_ai_initial_review()` | quality_gate.rs | VLM integration for real visual analysis |
| `_evaluate_and_apply()` | self_improvement.rs | ROI estimation, parameter adjustment logic |

## Common Patterns Found

1. **Duplicated Registration Logic**: `register_from_catalog()` and `register_from_unified_pool()` share identical API key check patterns
2. **Hardcoded Thresholds**: Circuit breaker window_size, quality gate 0.7 total threshold, rhythm consecutive count (2)
3. **In-Memory Only**: AbsorptionRegistry stats, self-improvement metrics history, quality gate review history
4. **Linear Mappings**: health_penalty (1.0/0.5/0.0), severity_to_priority (linear), trend detection (delta > 0.01)
5. **Missing Concurrency**: Sequential cross-module checks, lock-held-during-absorb, no drain timeout on unregister

## Recommendations

1. **Extract common registration helper** for catalog/unified pool providers
2. **Add EventBus integration** for circuit breaker state changes and absorption events
3. **Implement parameter snapshots** for self-improvement rollback
4. **Replace O(n²) compaction** with HashMap-based lookup
5. **Add exploration epsilon** to KNN router for discovering better model matches
