# Targeted Research #478 — Internal Pain Points (Round 28)

**Date**: 2026-09-12
**Scope**: Stub implementations returning fabricated results, misleading downstream consumers

---

## Pain Point 1: VerifierAgent fakes video verification with keyword heuristics

**File**: `neotrix-core/src/l6_meta/coordination/verifier_agent.rs:186-264`
**Severity**: **P0**

### What's wrong

`_verify_shot()` calls `simulate_verification()` which scores video quality by checking if description text contains words like "character", "lighting", "action". It never invokes a VLM or compares frames. Every "verified" video passes a fake quality gate — the verification result is fabricated from string matching.

This is a **production quality gate** that downstream consumers trust. A video with typos in its description gets lower scores than one with correct keywords, regardless of actual visual quality.

### Fix sketch

```rust
fn _verify_shot(&mut self, shot_id: &str, video_path: &str, spec: &str, ctx: Option<&str>) -> VerificationResult {
    let start = Instant::now();

    // Route through LLM capability网 instead of heuristic
    let scores = match self.llm_provider {
        Some(ref llm) => self.verify_with_vlm(llm, video_path, spec, ctx).await,
        None => {
            tracing::warn!("VerifierAgent: no VLM provider, falling back to heuristic");
            self.simulate_verification(spec, ctx) // explicit fallback, not default path
        }
    };

    let total_score = self.calculate_total_score(&scores);
    let passed = total_score >= self.config.pass_threshold;
    // ... rest unchanged
}
```

---

## Pain Point 2: ReferenceGeneration returns fabricated success for all generation modes

**File**: `neotrix-core/src/l1_action/nt_io/reference_generation.rs:224-271`
**Severity**: **P1**

### What's wrong

`video_to_video()`, `image_to_video()`, and `style_transfer()` all return `success: true` with hardcoded similarity scores (0.83–0.90) and fabricated output paths. No model is called. Any consumer checking `result.success` gets a lie — the output file doesn't exist.

This silently breaks downstream pipelines: `ProductionOrchestrator` marks tasks complete, `QualityControlPipeline` finds nothing to review, and users see "done" for work that never happened.

### Fix sketch

```rust
pub fn video_to_video(&mut self, input_path: &str) -> GenerationResult {
    let start = Instant::now();

    let output = match self.platform_gateway {
        Some(ref gw) => gw.submit_video_generation(input_path, &self.config.model_name)?,
        None => return GenerationResult {
            success: false,
            output_paths: vec![],
            generation_time_ms: 0,
            model_used: self.config.model_name.clone(),
            reference_similarity: 0.0,
            quality_score: 0.0,
            error: Some("No platform gateway configured — cannot generate".into()),
        },
    };

    let result = GenerationResult {
        success: output.status == "completed",
        output_paths: output.output_files,
        generation_time_ms: start.elapsed().as_millis() as u64,
        // ... real scores from output
    };
    self.history.push(result.clone());
    result
}
```

---

## Pain Point 3: StoryboardExtractor splits paragraphs instead of using LLM for scene understanding

**File**: `neotrix-core/src/l5_cognition/nt_core/visual/storyboard_extractor.rs:188-243`
**Severity**: **P1**

### What's wrong

`_extract_from_script()` calls `parse_script_to_shots()` which splits text by newlines and assigns every paragraph a `ShotSize::Medium` + `CameraMovement::Static`. No LLM call is made despite the TODO. The resulting storyboard is a flat line-by-line dump with no scene boundaries, no character detection, no camera direction.

For a 10-paragraph script, this produces 10 identical shots. The `StoryboardExtractor` capability advertised in CONTEXT.md ("LLM剧本→分镜自动拆解") is entirely faked.

### Fix sketch

```rust
fn parse_script_to_shots(&self, script_text: &str) -> Vec<Storyboard> {
    let llm = self.llm_provider.as_ref().ok_or("No LLM provider")?;

    let prompt = format!(
        "Break this script into cinematic shots. For each: shot_size, camera_movement, \
         characters present, scene location, emotion, duration_secs.\n\n{}",
        script_text
    );

    let response = llm.complete(&prompt, MaxTokens(2048))?;
    let parsed: Vec<ShotDef> = serde_json::from_str(&response)?;

    parsed.into_iter().enumerate().map(|(i, def)| Storyboard {
        id: format!("shot_{}", i + 1),
        shot_number: (i + 1) as u32,
        description: def.description,
        shot_size: def.shot_size,        // LLM-decided, not hardcoded
        camera_movement: def.camera,     // LLM-decided
        duration_secs: def.duration_secs,
        characters: def.characters,      // extracted, not empty
        // ...
    }).collect()
}
```

---

## Summary

| # | File | Issue | Severity | Category |
|---|------|-------|----------|----------|
| 1 | `verifier_agent.rs:186` | VLM verification replaced by string-matching heuristic | P0 | Stub returning fake data |
| 2 | `reference_generation.rs:224-271` | All 3 generation modes return hardcoded success | P1 | Stub returning fake data |
| 3 | `storyboard_extractor.rs:188-243` | LLM scene parsing replaced by newline splitting | P1 | Stub returning fake data |

All three follow the same anti-pattern: **function signature promises real work, body returns fabricated success**. Downstream consumers have no way to detect the deception without inspecting output file existence.
