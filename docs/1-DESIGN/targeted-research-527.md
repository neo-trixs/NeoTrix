# Targeted Research #527 — Internal Pain Points (WISER Loop)

**Date:** 2026-09-13
**Scope:** neotrix-core/src/ — incomplete implementations, hardcoded values, dead stubs
**Method:** grep TODO/FIXME/HACK + heuristic hardcoded-return scan

---

## Pain Point 1: VerifierAgent — Fake VLM Verification

**File:** `neotrix-core/src/l6_meta/coordination/verifier_agent.rs:195-282`

**What's wrong:** `_verify_shot` claims to perform "VLM verification + auto-regeneration" but `simulate_verification()` is a pure keyword-matching heuristic that returns hardcoded scores (7-9) based on whether the description contains words like "character" or "lighting". The "video path" argument is completely ignored. This module is registered in the production pipeline — callers believe they're getting real image verification.

**Severity:** P1 — Silent quality failure. Content passes verification that a real VLM would reject.

**Fix sketch:**

```rust
// verifier_agent.rs — replace simulate_verification body
fn simulate_verification(&self, description: &str, context: Option<&str>, video_path: Option<&str>) -> Vec<_VerificationScore> {
    // Delegate to VLM provider (e.g. MiniMax vision or local model)
    let vlm_result = self.config.vlm_provider
        .as_ref()
        .map(|p| p.verify(video_path.unwrap_or_default(), description, context))
        .transpose()?;

    let scores = vlm_result.unwrap_or_else(|| {
        // Fallback: keep existing heuristic but mark as degraded
        eprintln!("[verifier_agent] WARN: no VLM provider, using heuristic fallback");
        self.heuristic_fallback(description, context)
    });

    scores
}
```

**Impact:** Without this, every `VerificationResult` in the pipeline is a lie. The `needs_regeneration` flag never triggers real fixes.

---

## Pain Point 2: QualityControlPipeline — Hardcoded Pass for Human/Platform Reviews

**File:** `neotrix-core/src/l6_meta/coordination/quality_control.rs:283-312`

**What's wrong:** `_execute_review_flow` iterates through `[AI, Human, Platform]` levels. The `Human` and `Platform` arms return `ReviewStatus::Approved` with hardcoded `total_score: 0.90` / `0.88` immediately — no actual review occurs. The `_review_by_ai` method uses hardcoded base scores per check type (e.g. `Technical => 0.85`). The entire pipeline always approves everything.

**Severity:** P0 — Quality gate is decorative. Every artifact passes all three review levels with fake scores.

**Fix sketch:**

```rust
ReviewLevel::Human => {
    // Emit review request to EventBus, persist pending state
    let request_id = self.event_bus.emit(ReviewRequested {
        content_id: content_id.to_string(),
        level: ReviewLevel::Human,
        deadline: Utc::now() + chrono::Duration::hours(24),
    })?;

    ReviewResult {
        review_id: request_id,
        content_id: content_id.to_string(),
        level: ReviewLevel::Human,
        status: ReviewStatus::Pending, // NOT Approved
        total_score: 0.0,
        check_scores: HashMap::new(),
        issues: vec![],
        comments: Some("Awaiting human review".to_string()),
        review_time: 0,
        review_time_ms: 0,
    }
}
```

**Impact:** The 3-level quality gate (AI→Human→Platform) is the core safety mechanism for content production. Currently it's a no-op. Content with entity drift, compliance violations, or broken narrative flow will ship unchecked.

---

## Pain Point 3: NarrativeStructuring — Naive Paragraph Splitting Instead of LLM Structuring

**File:** `neotrix-core/src/l5_cognition/nt_core/other/narrative_structuring.rs:243-269`

**What's wrong:** `parse_text_to_shots` splits text by blank lines and maps each paragraph to a shot with: default duration (3s), `ShotSize::Medium`, `CameraMovement::Static`, empty elements, empty prompts, `None` for dialogue/narration/action/emotion. The module is named "narrative structuring" and advertised as "LLM-powered" but uses `text.lines().filter(|l| !l.trim().is_empty())`. The `TODO: 实际调用 LLM 解析` at line 244 confirms this is a stub.

**Severity:** P1 — The entire video production pipeline consumes `_NarrativeScript` output. All downstream systems (storyboard→SR→TTS) receive degraded, uniform, empty-shot data.

**Fix sketch:**

```rust
// narrative_structuring.rs — replace parse_text_to_shots body
fn parse_text_to_shots(&self, text: &str) -> Vec<_ShotUnit> {
    let prompt = format!(
        "Parse this script into structured shots. For each shot provide: \
         description, shot_size, camera_movement, duration_secs, dialogue, \
         narration, action, emotion, positive_prompt, negative_prompt.\n\n{text}"
    );

    let response = self.config.llm_provider
        .complete(&prompt)
        .unwrap_or_else(|e| {
            eprintln!("[narrative] LLM failed, falling back to paragraph split: {e}");
            return self.fallback_paragraph_split(text);
        });

    serde_json::from_str::<Vec<_ShotUnit>>(&response)
        .unwrap_or_else(|_| self.fallback_paragraph_split(text))
}
```

**Impact:** Every video produced through the `ReferenceBasedGeneration` pipeline gets uniform 3-second static shots with no prompts. This is the single biggest bottleneck for content quality.

---

## Summary

| # | Location | Issue | Severity | Type |
|---|----------|-------|----------|------|
| 1 | `verifier_agent.rs:195-282` | Fake VLM verification via keyword matching | P1 | Stub returning hardcoded values |
| 2 | `quality_control.rs:283-312` | Human/Platform reviews auto-approve with fake scores | P0 | Hardcoded pass |
| 3 | `narrative_structuring.rs:243-269` | "LLM structuring" is paragraph splitting | P1 | Stub returning empty data |

**Pattern:** All three are "production-decorated stubs" — real-looking structs and interfaces with placeholder logic that silently produces fake-but-plausible output. The danger is that tests pass (because they test the stubs) but production receives no real value.
