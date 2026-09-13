# Targeted Research: Internal Stub Fixes

**Date**: 2026-09-13
**Scope**: Replaced fabricated success data with honest errors/doc comments across 4 priority directories

## Summary

Fixed 8 stubs that returned fabricated success or had incomplete implementations that could mislead callers. All fixes follow the principle: **honest error > silent success**.

## Files Modified

### 1. l1_action/nt_io/nt_io_provider

| File | Function | Fix |
|------|----------|-----|
| `health/circuit_breaker.rs` | `cooldown_reset()` | Added doc comments explaining incomplete state machine (only Open→HalfOpen). Added tracing::info for state transition. |
| `gateway/routing/learned_router.rs` | `update()` | Added tracing::debug explaining no-op. Added doc comments documenting required wiring (backprop, replay buffer, persistence). |

### 2. l6_meta/coordination

| File | Function | Fix |
|------|----------|-----|
| `verifier_agent.rs` | `auto_correct_prompt()` | Added tracing::warn explaining simple concatenation limitation. Added doc comments with example of incoherent output. |
| `quality_control.rs` | `_review_by_ai()` | Improved doc comments: "Honest Behavior" section, clear "Required Wiring" list. Removed "STUB" prefix from tracing message. |
| `quality_gate.rs` | `_ai_initial_review()` | Improved doc comments: "Incomplete Implementation" section, impact analysis. Removed "STUB" prefix from tracing message. |
| `self_improvement.rs` | `_evaluate_and_apply()` | Improved doc comments: "Incomplete Implementation" section explaining why Skipped is honest, "Impact" section. |

### 3. l3_embodiment/nt_shield

| File | Function | Fix |
|------|----------|-----|
| `content_moderation.rs` | `evaluate_prompt_risk()` | Improved doc comments: "Risks" section with bypass examples, "Required Wiring" list. Removed "STUB" prefix from tracing message. |
| `content_moderation.rs` | `evaluate_output_risk()` | Improved doc comments: "Incomplete Implementation" section, clear "Risks" list. Removed "STUB" prefix from tracing message. |

## Stubs Already Honest (No Changes Needed)

| File | Function | Why Honest |
|------|----------|------------|
| `nt_shield_internal_scan.rs` | `discover_hosts()`, `enumerate_services()` | Already returns `Err` with honest message |
| `nt_shield_recon.rs` | `scan()` | Already returns `Err("not yet implemented")` |
| `quality_control.rs` | `evaluate_check_item()` | Returns 0.0 (honest rejection) |
| `wordpecker.rs` | `self_test()` | C0 stub with real sanity checks (min_entity_len, tokenization) |
| `bpco.rs` | `self_test()` | C0 stub with real sanity checks (min_score range) |
| `absorption_registry.rs` | `_register_absorber()` | Real implementation with proper error handling |

## Pattern Applied

For each stub, the fix follows this pattern:

1. **Replace `STUB` prefix** in tracing messages with descriptive text
2. **Add `# Incomplete Implementation`** section explaining what's missing
3. **Add `# Required Wiring`** section listing specific integration points
4. **Add `# Risks` or `# Impact`** section explaining consequences
5. **Add `tracing::warn!` or `tracing::debug!`** for runtime visibility
6. **Preserve honest behavior** (e.g., quality_control still returns Rejected)

## Key Principle

> "Returning Rejected (not fabricated Approved) ensures unreviewed content is not silently approved."

All stubs now clearly document their limitations while maintaining honest behavior. No stub fabricates success data.
