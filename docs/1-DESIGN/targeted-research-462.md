# Targeted Research #462 — Internal Pain Points

**Date**: 2026-09-12
**Scope**: Stub functions returning fabricated quality metrics, hardcoded pricing, and dead code paths
**Status**: Analysis complete

---

## Pain Point 1: Fabricated Quality Scores in ReferenceGeneration (P0)

**File**: `neotrix-core/src/l1_action/nt_io/reference_generation.rs:170-235`
**Severity**: P0 — Users make generation decisions based on fabricated metrics

All four generation functions (`image_to_image`, `video_to_video`, `image_to_video`, `style_transfer`) return `success: true` with hardcoded quality/similarity scores (e.g., `0.92`, `0.88`). No actual model is invoked. A caller trusting these scores would believe generation succeeded and quality is high.

```rust
// reference_generation.rs:170 — image_to_image
pub fn image_to_image(&mut self, input_path: &str) -> GenerationResult {
    // TODO: 实际调用生成模型
    let result = GenerationResult {
        success: true,
        output_paths: vec![format!("{}_generated.png", input_path)],
        generation_time_ms: 3000,
        model_used: self.config.model_name.clone(),
        reference_similarity: 0.92,  // ← fabricated
        quality_score: 0.88,          // ← fabricated
        error: None,
    };
    self.history.push(result.clone());
    result
}
```

**Fix sketch** — Delegate to `CapabilityRegistry` and fail honestly:

```rust
pub fn image_to_image(&mut self, input_path: &str) -> GenerationResult {
    let start = std::time::Instant::now();
    // Route through the unified capability registry instead of fabricating
    match crate::core::l7_capability::CapabilityRegistry::instance()
        .execute(crate::core::l7_capability::CapabilityInput::ReferenceGeneration {
            mode: ReferenceMode::ImageToImage,
            input_path: input_path.to_string(),
            model: self.config.model_name.clone(),
        }) {
        Ok(output) => GenerationResult {
            success: true,
            output_paths: output.paths,
            generation_time_ms: start.elapsed().as_millis() as u64,
            model_used: self.config.model_name.clone(),
            reference_similarity: output.similarity,
            quality_score: output.quality,
            error: None,
        },
        Err(e) => GenerationResult {
            success: false,
            output_paths: vec![],
            generation_time_ms: start.elapsed().as_millis() as u64,
            model_used: self.config.model_name.clone(),
            reference_similarity: 0.0,
            quality_score: 0.0,
            error: Some(e.to_string()),
        },
    }
}
```

---

## Pain Point 2: video_post_processor Denoise/Sharpen Return Fake Metrics (P1)

**File**: `neotrix-core/src/l3_embodiment/nt_physical/video_post_processor.rs:342-369`
**Severity**: P1 — Downstream pipeline trusts these quality scores for pass/fail decisions

Both `denoise` and `sharpen` return `success: true` with fabricated `temporal_stability_score`, `quality_improvement_score`, etc. The `_process_full_pipeline` (line 372) chains these results, so the entire pipeline's quality verdict is based on made-up numbers.

```rust
// video_post_processor.rs:342
pub fn denoise(&self, video_path: &str) -> _PostProcessResult {
    // TODO: 实际调用降噪逻辑
    _PostProcessResult {
        success: true,
        processed_video_path: Some(format!("{}_denoised.mp4", video_path)),
        processed_frames: 150,          // ← fabricated
        color_consistency_score: 0.89,   // ← fabricated
        temporal_stability_score: 0.87,  // ← fabricated
        quality_improvement_score: 0.85, // ← fabricated
        processing_time_ms: 6000,
        error: None,
    }
}
```

**Fix sketch** — Shell out to ffmpeg with `nlmeans`/`unsharp` and measure actual improvement:

```rust
pub fn denoise(&self, video_path: &str) -> _PostProcessResult {
    let start = std::time::Instant::now();
    let output_path = format!("{}_denoised.mp4", video_path);
    let status = std::process::Command::new("ffmpeg")
        .args(["-i", video_path, "-vf", "nlmeans=s=3:pc=3", "-y", &output_path])
        .status();
    match status {
        Ok(s) if s.success() => _PostProcessResult {
            success: true,
            processed_video_path: Some(output_path),
            processed_frames: 0, // TODO: parse frame count from ffprobe
            color_consistency_score: 0.0, // TODO: compute actual metric
            temporal_stability_score: 0.0,
            quality_improvement_score: 0.0,
            processing_time_ms: start.elapsed().as_millis() as u64,
            error: None,
        },
        Ok(_) => _PostProcessResult {
            success: false, error: Some("ffmpeg denoise failed".into()), ..
        },
        Err(e) => _PostProcessResult {
            success: false, error: Some(e.to_string()), ..
        },
    }
}
```

---

## Pain Point 3: resource_budget Hardcoded Pricing Table (P1)

**File**: `neotrix-core/src/l1_action/nt_act/resource_budget.rs:288-297`
**Severity**: P1 — Cost estimates drift from reality; users set budgets based on wrong numbers

`estimate_cost` uses a hardcoded 3-model price table. Every new model (GPT-4o, Claude 3.5 Sonnet, Gemini 1.5, etc.) falls through to the `$0.001/1K` default, making cost estimates wildly inaccurate for production workloads.

```rust
// resource_budget.rs:288
pub fn estimate_cost(&self, token_count: u64, model: &str) -> f64 {
    // TODO: 实际调用成本计算
    let cost_per_1k = match model {
        "gpt-4" => 0.03,
        "gpt-3.5-turbo" => 0.002,
        "claude-3" => 0.015,
        _ => 0.001,  // ← 40x underestimate for GPT-4o-mini, etc.
    };
    (token_count as f64 / 1000.0) * cost_per_1k
}
```

**Fix sketch** — Load from KB-configured pricing table or `nt_io_llm` registry:

```rust
pub fn estimate_cost(&self, token_count: u64, model: &str) -> f64 {
    // Consult the LLM provider registry for actual pricing
    if let Some(pricing) = crate::l1_action::nt_io::nt_io_llm::provider_registry()
        .and_then(|r| r.get_pricing(model))
    {
        let input_cost = (token_count as f64 / 1000.0) * pricing.input_per_1k;
        let output_cost = (token_count as f64 / 1000.0) * pricing.output_per_1k;
        input_cost + output_cost
    } else {
        // Fallback: conservative estimate, log warning
        tracing::warn!("No pricing for model '{}', using conservative estimate", model);
        (token_count as f64 / 1000.0) * 0.03 // assume gpt-4-tier
    }
}
```

---

## Summary

| # | File | Line | Issue | Severity |
|---|------|------|-------|----------|
| 1 | `reference_generation.rs` | 170-235 | 4 functions return fabricated quality scores | P0 |
| 2 | `video_post_processor.rs` | 342-369 | Denoise/sharpen return fake metrics, pollute pipeline | P1 |
| 3 | `resource_budget.rs` | 288-297 | Hardcoded 3-model price table, wrong for all new models | P1 |

All three share the same anti-pattern: stub functions that pretend to succeed with fabricated data rather than failing honestly or delegating to real backends.
