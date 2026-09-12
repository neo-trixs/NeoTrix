# Targeted Research #523 — Internal Pain Points (WISER Loop Batch 6)

**Date:** 2026-09-13
**Scope:** 3 internal pain points found via code archaeology (TODO/stub/hardcoded scan)

---

## Pain Point 1: Verifier Agent Uses Keyword Heuristic Instead of LLM Verification

**File:** `neotrix-core/src/l6_meta/coordination/verifier_agent.rs:222-258`
**Severity:** P1 — Verifier is the quality gate for regenerated content; fake scores mean broken feedback loops.

**What's wrong:** `simulate_verification()` returns hardcoded scores based on substring matching (`contains("character") → 7`, else `9`). This means the verifier never actually evaluates visual consistency — it just checks if keywords exist in the description text. The `auto_correct_prompt()` at line 336 also just appends corrections as suffix strings instead of calling an LLM.

**Impact:** Every verification pass is deterministic and meaningless. Bad content passes, good content may be flagged. The entire regeneration loop (`needs_regeneration` flag) is compromised.

**Fix sketch:**
```rust
// verifier_agent.rs — replace simulate_verification
fn simulate_verification(&self, description: &str, context: Option<&str>) -> Vec<_VerificationScore> {
    let prompt = format!(
        "Evaluate this generated content against the spec.\n\
         Description: {}\nContext: {}\n\
         Score each dimension 0-10: entity_consistency, environment, narrative, instruction_following, overall",
        description, context.unwrap_or("none")
    );
    match self.llm_provider.complete(&prompt, 256) {
        Ok(raw) => parse_scores_from_json(&raw).unwrap_or_else(|| self.fallback_heuristic(description, context)),
        Err(_) => self.fallback_heuristic(description, context),
    }
}
// Keep the old logic as fallback_heuristic for offline mode
```

---

## Pain Point 2: Consciousness Core Agent Never Actually Applies Patches

**File:** `neotrix-core/src/l5_cognition/nt_core/nt_consciousness_core/agent.rs:314-328`
**Severity:** P1 — The self-healing loop detects gaps and generates patches but skips the application step.

**What's wrong:** `apply_patches()` iterates over patches and checks `confidence >= 0.7`, but the actual application is commented out (`// patch.apply();`). It only calls `mark_fixed()` — recording the fix without doing it. The `take_snapshot()` at line 270 also generates fake phi/coherence values (`0.5 + cycle * 0.001`).

**Impact:** The consciousness tree's MAPE-K cycle (Monitor→Analyze→Plan→Execute) is broken at Execute. Gaps are detected and "fixed" on paper but never in code. The self-healing loop is an illusion.

**Fix sketch:**
```rust
// agent.rs — apply_patches should delegate to a patch executor
fn apply_patches(&mut self, patches: &[Patch]) -> u32 {
    let mut fixed = 0;
    for patch in patches {
        if patch.confidence >= 0.7 {
            match self.patch_executor.apply(patch) {  // New trait: PatchExecutor
                Ok(()) => {
                    self.gap_registry.mark_fixed(&patch.gap_id);
                    self.event_bus.emit(PatchApplied { gap_id: patch.gap_id.clone() });
                    fixed += 1;
                }
                Err(e) => log::warn!("Patch apply failed for {}: {}", patch.gap_id, e),
            }
        }
    }
    fixed
}
```

---

## Pain Point 3: Storyboard Extractor Uses Line-Split Heuristic Instead of LLM

**File:** `neotrix-core/src/l5_cognition/nt_core/visual/storyboard_extractor.rs:216-234`
**Severity:** P1 — Core feature of the 动态漫 pipeline; paragraph splitting produces no camera/emotion/character data.

**What's wrong:** `parse_script_to_shots()` splits text by newlines, then assigns each line a `ShotSize::Medium`, `CameraMovement::Static`, zero characters, and `"默认场景"`. Every shot looks identical. The `_extract_from_script()` at line 194 has a TODO to call LLM but uses this fallback unconditionally.

**Impact:** The entire storyboard pipeline produces uniform 3-second static medium shots with no differentiation. The downstream `BlankSpaceChecker`, `TransitionGradientAdvisor`, and `NarrativeStructuring` all receive degenerate input.

**Fix sketch:**
```rust
// storyboard_extractor.rs — replace parse_script_to_shots
fn parse_script_to_shots(&self, script_text: &str) -> Vec<Storyboard> {
    let prompt = format!(
        "Parse this script into shots. For each shot return JSON: \
         {{\"description\",\"characters\":[],\"scene\",\"shot_size\",\
         \"camera_movement\",\"duration_secs\",\"emotion\",\"action\"}}\n\n{}",
        script_text
    );
    match self.llm_provider.complete(&prompt, 2048) {
        Ok(raw) => parse_shots_from_json(&raw).unwrap_or_else(|| self.fallback_line_split(script_text)),
        Err(_) => self.fallback_line_split(script_text),
    }
}
// Move current line-split logic to fallback_line_split for offline mode
```

---

## Summary

| # | File | Issue | Severity | Fix Complexity |
|---|------|-------|----------|---------------|
| 1 | `verifier_agent.rs:222` | Keyword heuristic instead of LLM verification | P1 | Medium — need LLM provider injection |
| 2 | `agent.rs:318-322` | Patches detected but never applied | P1 | Medium — need PatchExecutor trait |
| 3 | `storyboard_extractor.rs:216` | Line-split instead of LLM storyboard parsing | P1 | Medium — need LLM provider injection |

All three are P1 because they break core feedback loops (verification, self-healing, content generation) that downstream features depend on. The common pattern: stub methods that return plausible-looking but meaningless results, making the system appear functional while doing nothing useful.
