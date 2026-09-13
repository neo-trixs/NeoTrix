# Targeted Research 719 — Undocumented Function Documentation

**Date**: 2026-09-13
**Scope**: Priority files across l1_action, l5_cognition, l6_meta

## Summary

Added `/// Note:` and `/// STUB:` doc comments to **50+ functions** across 11 files in the priority directories. Functions were categorized as:

- **STUB**: Returns `Err("not wired")` or hardcoded placeholder — needs real backend
- **Note**: Has logic but missing documentation explaining behavior, limitations, or production needs

## Files Modified

### L1 Action — nt_io_provider/gateway

| File | Functions Documented | Key Findings |
|------|---------------------|--------------|
| `execution.rs` | 8 | `find_provider_by_category` (linear scan), `coordinate` (routing), `complete_single` (wrapper), `complete_with_selection` (primary entry), `describe_image` (vision), `fire_event` (telemetry), `call_provider_backoff` (retry), `parse_retry_after` (JSON parse) |
| `resilience.rs` | 20+ | `LlmPoolHealth::evaluate/summarize`, cache enable/disable/getters, `tag_generation` (analytics), `heal_and_cache_response` (JSON repair + cache), `prompt_text/prompt_cache_key` (key generation) |
| `routing/market_router.rs` | 3 | `with_interval` (configurable), `re_evaluate` (periodic), `route` (weight-based selection) |

### L5 Cognition — nt_mind/mind_modules

| File | Functions Documented | Key Findings |
|------|---------------------|--------------|
| `seal/seal_enhanced.rs` | 2 | `new` (hardcoded stages), `stats` (in-memory only) |
| `agent/research.rs` | 5 | `_generate_hypothesis` (**STUB**), `_design_experiment` (**STUB**), `analyze_results` (**STUB**), `_generate_paper` (**STUB**), `stats` |
| `agent/pilot_failure.rs` | 6 | `new`, `detect` (pattern matching), `calculate_pattern_confidence` (indicator ratio), `_integrate_seal` (**STUB**), `detectors/patterns/stats` |
| `knowledge/absorption_registry.rs` | 2 | `_list_absorbers` (snapshot), `_absorber_count` (O(1) count) |

### L6 Meta — coordination

| File | Functions Documented | Key Findings |
|------|---------------------|--------------|
| `verifier_agent.rs` | 2 | `_generate_regeneration_request` (mode selection), `calculate_total_score` (weighted) |
| `layered_qa.rs` | 6 | `new/with_config` (hardcoded checks), `execute` (pipeline), `execute_stage/execute_check` (sequential), `calculate_total_score` (equal weight) |
| `self_improvement.rs` | 12 | `new/with_capacity`, `collect_metrics`, `diagnose` (threshold-based), `_generate_plans` (**STUB**), `_evaluate_and_apply` (**STUB**), `rollback`, `verify`, `run_cycle`, `actions_for_dimension` (templates), `severity_to_priority`, `prune_history`, `update_trends` |
| `nt_meta_sentrux.rs` | 2 | `_save_baseline` (in-memory), `_compare_with_baseline` (score delta) |

## Critical STUBs Requiring Real Backends

| STUB | Location | What's Needed |
|------|----------|---------------|
| `_generate_hypothesis` | `research.rs` | LLM-based hypothesis generation from topic/context |
| `_design_experiment` | `research.rs` | LLM-based experimental design with sample size calculation |
| `analyze_results` | `research.rs` | Actual statistical tests (t-test, ANOVA) |
| `_generate_paper` | `research.rs` | LLM-based section generation with citation management |
| `_integrate_seal` | `pilot_failure.rs` | Actual SEAL pipeline hook registration |
| `_generate_plans` | `self_improvement.rs` | LLM-based plan generation for novel issues |
| `_evaluate_and_apply` | `self_improvement.rs` | ROI estimation + parameter adjustment logic |
| `scan` | `nt_meta_sentrux.rs` | AST parsing, dependency graph analysis |
| `_check_rules` | `nt_meta_sentrux.rs` | Actual dependency graph analysis and cycle detection |
| `_verify_shot` | `verifier_agent.rs` | VLM endpoint (GPT-4V / Gemini Pro Vision) |

## Patterns Observed

1. **In-memory only**: Most state (metrics, history, baselines) is not persisted to KB
2. **Hardcoded configs**: Check items, dimension groups, and stage definitions are hardcoded
3. **Linear scans**: Provider lookups use O(n) scans instead of indexed lookups
4. **Simple thresholds**: Diagnosis uses fixed thresholds instead of adaptive/SPC methods
5. **No EventBus integration**: Telemetry and metrics not connected to event system

## Recommendations

1. **Priority 1**: Wire SEAL backends (explore/distill/absorb) to real LLM + KB paths
2. **Priority 2**: Connect self-improvement metrics to EventBus for real observation
3. **Priority 3**: Add KB persistence for metrics_history, verification history, and baselines
4. **Priority 4**: Replace hardcoded check items with configurable KB-loaded definitions
5. **Priority 5**: Add provider category index for O(1) lookup in capability routing
