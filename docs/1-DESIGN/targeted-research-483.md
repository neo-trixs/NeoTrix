# Targeted Research #483 — Internal Pain Points

## Pain Point 1: Visual Style Harmonizer Hardcoded Stubs

**Location:** `neotrix-core/src/l5_cognition/nt_core/visual/style_harmonizer.rs:113-165`

**Issue:** Three core methods (`_analyze_style`, `_harmonize`, `_match_colors`) return hardcoded values instead of performing real analysis. `_analyze_style` returns fixed `StyleAnalysis` with synthetic color distributions and feature vectors. `_harmonize` and `_match_colors` return fabricated similarity scores (0.88, 0.92) and processing times without executing any computation.

**Severity:** P1 — The visual style system is wired into the dynamic manga production pipeline but produces fake data, misleading downstream consumers into thinking style consistency is being enforced.

**Fix sketch:**
```rust
// style_harmonizer.rs — replace hardcoded _analyze_style
pub(crate) fn _analyze_style(&self, image_path: &str) -> Result<_StyleAnalysis, HarmonizerError> {
    let img = image::open(image_path).map_err(|e| HarmonizerError::Io(e.to_string()))?;
    let rgb = img.to_rgb8();
    let pixels: Vec<[u8; 3]> = rgb.pixels().map(|p| [p[0], p[1], p[2]]).collect();
    
    let total = pixels.len() as f32;
    let avg_r = pixels.iter().map(|p| p[0] as f32).sum::<f32>() / total;
    let avg_g = pixels.iter().map(|p| p[1] as f32).sum::<f32>() / total;
    let avg_b = pixels.iter().map(|p| p[2] as f32).sum::<f32>() / total;
    
    Ok(_StyleAnalysis {
        features: _Style特征 {
            color_distribution: vec![avg_r / 255.0, avg_g / 255.0, avg_b / 255.0],
            contrast: compute_contrast(&pixels),
            saturation: compute_saturation(&pixels),
            color_temperature: compute_temperature(avg_r, avg_b),
            texture_features: extract_texture(&rgb)?,
            style_tags: infer_tags_from_histogram(&rgb),
        },
        dominant_colors: extract_dominant_colors(&rgb, 3)?,
        style_tags: vec![],
        quality_score: compute_quality_score(&rgb),
    })
}
```

---

## Pain Point 2: Reference Generation Fake Output Paths

**Location:** `neotrix-core/src/l1_action/nt_io/reference_generation.rs:224-272`

**Issue:** `video_to_video`, `image_to_video`, and `style_transfer` all return fabricated `GenerationResult` structs with fake `success: true`, fake output file paths (`{input}_generated.mp4`), and fabricated quality scores (0.83–0.90). No actual model inference occurs. The caller receives a success response pointing to a non-existent file.

**Severity:** P0 — Consumers calling these functions will attempt to read output files that don't exist, causing silent failures or crashes in downstream pipelines.

**Fix sketch:**
```rust
// reference_generation.rs — return error for unimplemented paths
pub fn video_to_video(&mut self, input_path: &str) -> GenerationResult {
    let result = GenerationResult {
        success: false,
        output_paths: vec![],
        generation_time_ms: 0,
        model_used: self.config.model_name.clone(),
        reference_similarity: 0.0,
        quality_score: 0.0,
        error: Some(format!(
            "video_to_video not implemented. Route through \
             nt_io_provider::gateway::unified_inference with \
             VideoToVideo task_type. Input: {}", input_path
        )),
    };
    self.history.push(result.clone());
    result
}
```

---

## Pain Point 3: Storyboard Extractor Naive Paragraph Split

**Location:** `neotrix-core/src/l5_cognition/nt_core/visual/storyboard_extractor.rs:216-239`

**Issue:** `parse_script_to_shots` splits scripts by empty lines and treats each paragraph as a separate shot, assigning identical `default_shot_duration` to every shot. Characters, scene context, shot size, and camera movement are all left empty/default. The `TODO: 实际调用 LLM 解析` at line 217 confirms this is a stub.

**Severity:** P1 — The storyboard extractor is the entry point for the entire video production pipeline. Garbage-in here propagates incorrect shot timing, missing character assignments, and wrong scene breakdowns through the full production chain.

**Fix sketch:**
```rust
// storyboard_extractor.rs — add LLM-backed parsing
fn parse_script_to_shots(&self, script_text: &str) -> Result<Vec<Storyboard>, ExtractorError> {
    let prompt = format!(
        "Parse this script into structured shots. For each shot return JSON: \
         {{shot_number, description, characters[], scene, shot_size, \
         camera_movement, duration_secs, dialogue, emotion}}\n\n{}",
        script_text
    );
    
    let response = self.llm_provider.complete(&prompt).await
        .map_err(|e| ExtractorError::LlmError(e))?;
    
    let shots: Vec<Storyboard> = serde_json::from_str(&response)
        .map_err(|e| ExtractorError::ParseError(e))?;
    
    Ok(shots)
}
```
