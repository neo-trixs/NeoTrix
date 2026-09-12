# Targeted Research #461 — Internal Pain Points

**Date:** 2026-09-12
**Scope:** neotrix-core/src/ — dead code, hardcoded returns, incomplete implementations
**Approach:** WISER iteration — find 3 high-signal pain points with concrete fixes

---

## Pain Point 1: SecurityCheckRegistry::run_all_checks always returns Ok (P0)

**File:** `neotrix-core/src/l3_embodiment/nt_shield/nt_shield/check_registry.rs:1039-1048`

**What's wrong:** The `SecurityCheckRegistry` trait impl for `CheckRegistry` has `run_all_checks()` that **always returns `Ok`**, regardless of actual check results. It maps every check name into `passed` without evaluating the check functions. Any L5 cognition code calling this trait method gets a false security compliance signal.

```rust
// CURRENT — broken: ignores all check results
fn run_all_checks(&self) -> Result<Vec<String>, Vec<String>> {
    let passed: Vec<String> = self.checks.iter()
        .map(|c| c.name.clone())
        .collect();
    if passed.is_empty() {
        Ok(passed)
    } else {
        Ok(passed)  // <-- always Ok!
    }
}
```

**Why it's P0:** This is the bridge between L3 shield security checks and L5 meta-cognition. If L5 queries "are all security checks passing?", the answer is always yes — even if SEC-001 (dangerous commands) or SEC-007 (secrets in args) are disabled or failing.

**Fix sketch (6 lines):**

```rust
fn run_all_checks(&self) -> Result<Vec<String>, Vec<String>> {
    let mut failed = Vec::new();
    for check in &self.checks {
        let ctx = ToolCallContext {
            tool_name: "__self_test__".into(),
            args: serde_json::json!({}),
            source: ToolSource::Consciousness,
        };
        match (check.check_fn)(&ctx) {
            CheckVerdict::Fail(reason) => failed.push(format!("{}: {}", check.name, reason)),
            CheckVerdict::Warn(reason) => failed.push(format!("{} (warn): {}", check.name, reason)),
            CheckVerdict::Pass => {}
        }
    }
    if failed.is_empty() {
        Ok(self.checks.iter().map(|c| c.name.clone()).collect())
    } else {
        Err(failed)
    }
}
```

---

## Pain Point 2: StyleHarmonizer — all 3 core methods are pure stubs (P1)

**File:** `neotrix-core/src/l5_cognition/nt_core/visual/style_harmonizer.rs:113-165`

**What's wrong:** `_analyze_style`, `_harmonize`, and `_match_colors` all return fabricated results. Every call returns the same hardcoded values (contrast=0.7, saturation=0.6, quality_score=0.85, similarity=0.88/0.92). This module claims to do "multi-model output style unification" but does zero image processing.

**Impact:** If any consumer calls `_harmonize()` expecting actual style transfer, it silently returns a fake success with a fabricated output path (`{input}_harmonized.png`) that doesn't exist on disk. Downstream code would then fail trying to read the non-existent file.

**Fix sketch (10 lines) — return `Err` instead of faking success:**

```rust
pub(crate) fn _analyze_style(&self, image_path: &str) -> Result<_StyleAnalysis, String> {
    // Validate file exists before claiming analysis
    if !std::path::Path::new(image_path).exists() {
        return Err(format!("image not found: {}", image_path));
    }
    // TODO(integration): wire to image::open + color histogram + LBP texture
    Err("style analysis not yet implemented — stub requires image processing backend".into())
}

pub(crate) fn _harmonize(&mut self, input_path: &str, _reference_path: Option<&str>) -> Result<_StyleHarmonizationResult, String> {
    if !std::path::Path::new(input_path).exists() {
        return Err(format!("input not found: {}", input_path));
    }
    // TODO(integration): wire to neural style transfer or color transfer algorithm
    Err("harmonization not yet implemented — stub requires style transfer backend".into())
}

pub(crate) fn _match_colors(&self, source_path: &str, _target_path: &str) -> Result<_StyleHarmonizationResult, String> {
    if !std::path::Path::new(source_path).exists() {
        return Err(format!("source not found: {}", source_path));
    }
    // TODO(integration): wire to Reinhard color transfer or histogram matching
    Err("color matching not yet implemented — stub requires color transfer backend".into())
}
```

---

## Pain Point 3: VerifierAgent VLM verification uses keyword heuristic (P1)

**File:** `neotrix-core/src/l6_meta/coordination/verifier_agent.rs:195-281`

**What's wrong:** `_verify_shot` is supposed to call a VLM (Vision-Language Model) to verify video shot quality against a spec. Instead, `simulate_verification` does string matching on the description text to produce scores. It checks if the description contains words like "character", "lighting", "action" and assigns scores based on keyword presence and description length. The video file (`_video_path`) is completely ignored — never read, never analyzed.

**Impact:** The quality gate for video production is a text-only heuristic. A shot described as "character walks through lighting scene with action and dialogue" gets EntityConsistency=7, EnvironmentConsistency=6, NarrativeProgression=8, InstructionFollowing=8 — all fabricated from text alone. Actual visual defects (blur, color drift, wrong character) are never detected.

**Fix sketch (10 lines) — gate on VLM availability:**

```rust
pub(crate) fn _verify_shot(
    &mut self,
    _shot_id: &str,
    video_path: &str,
    spec_description: &str,
    memory_context: Option<&str>,
) -> Result<VerificationResult, String> {
    if !std::path::Path::new(video_path).exists() {
        return Err(format!("video not found: {}", video_path));
    }
    let start = std::time::Instant::now();

    // TODO(integration): wire to VLM backend (e.g., GPT-4V / Qwen-VL)
    // Extract keyframes from video_path, send to VLM with spec_description prompt,
    // parse structured JSON response into Vec<_VerificationScore>.
    //
    // For now, fail-closed instead of fabricating scores:
    let result = VerificationResult {
        passed: false,
        total_score: 0.0,
        scores: vec![],
        error_types: vec!["VLM_NOT_CONNECTED".into()],
        suggested_corrections: vec!["Connect VLM backend for real verification".into()],
        needs_regeneration: false,
        verification_time_ms: start.elapsed().as_millis() as u64,
    };
    self.history.push(result.clone());
    Ok(result)
}
```

---

## Summary

| # | File | Line | Severity | Issue |
|---|------|------|----------|-------|
| 1 | `check_registry.rs` | :1039 | **P0** | `run_all_checks` always returns Ok — security checks silently bypassed |
| 2 | `style_harmonizer.rs` | :113-165 | **P1** | 3 core methods return hardcoded values, fake success paths |
| 3 | `verifier_agent.rs` | :195-281 | **P1** | VLM verification is keyword heuristic, video file never read |

**Pattern across all 3:** Interface exists, trait is wired, but the implementation body is a stub that fabricates results. The fix pattern is consistent: return `Err`/fail-closed instead of fake success, then integrate the actual backend.
