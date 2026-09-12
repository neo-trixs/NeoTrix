# Targeted Research #524 — Internal Pain Points (Iteration 5)

**Date:** 2026-09-13
**Scope:** neotrix-core/src/ — hardcoded returns, dead stubs, silent lies
**Prior iteration:** 27 critical issues already fixed

---

## Pain Point 1: Style Harmonizer — Silent Hardcoded Lies

**File:** `neotrix-core/src/l5_cognition/nt_core/visual/style_harmonizer.rs:113-165`
**Severity:** P1

**What's wrong:** Three core methods (`_analyze_style`, `_harmonize`, `_match_colors`) return fabricated results without processing any input image. A caller passing `image_path="photo.jpg"` gets back `style_similarity: 0.92` and `quality_score: 0.85` regardless of actual content. This silently corrupts the visual consistency pipeline — downstream components trust these scores.

**Impact:** VisualConsistencyManager, FaceConsistencyManager, and any style-aware generation pipeline receive false confidence signals. No error, no warning — just wrong data flowing through.

**Fix sketch:**

```rust
pub(crate) fn _analyze_style(&self, image_path: &str) -> _StyleAnalysis {
    // Real impl: load image, compute color histogram, run lightweight feature extractor
    let img = image::open(image_path).expect("failed to open image");
    let rgb = img.to_rgb8();
    let (hist_r, hist_g, hist_b) = compute_color_histogram(&rgb);
    let contrast = compute_contrast(&rgb);
    let saturation = compute_saturation(&rgb);
    let dominant_colors = extract_dominant_colors(&rgb, 3);

    _StyleAnalysis {
        features: _Style特征 {
            color_distribution: normalize_histogram(&hist_r, &hist_g, &hist_b),
            contrast,
            saturation,
            color_temperature: estimate_color_temperature(&hist_r, &hist_b),
            texture_features: extract_lbp_features(&rgb),
            style_tags: classify_style_tags(contrast, saturation, &dominant_colors),
        },
        dominant_colors,
        style_tags: classify_style_tags(contrast, saturation, &dominant_colors),
        quality_score: compute_niqe(&rgb) as f32, // no-reference quality
    }
}
```

---

## Pain Point 2: Speculative Decoding — Fabricated Metrics, Zero Speedup

**File:** `neotrix-core/src/l1_action/nt_io/nt_io_inference/speculative_decoding.rs:161-185`
**Severity:** P0

**What's wrong:** `SpeculativeDecoder::generate()` returns fabricated throughput multipliers (`2.0 + acceptance_rate * 2.0`) and empty token lists (`output_tokens: vec![]`) without running any draft model or verification. The `acceptance_rate` is computed from the previous (also fabricated) `acceptance_stats`. This means the speculative decoding feature — advertised as 2-3x throughput improvement — is a no-op that returns fake numbers.

**Impact:** ModelSelector may route requests through SpeculativeDecoder expecting speedup, getting identical latency with fabricated metrics. Debugging this is extremely difficult because the numbers *look* plausible.

**Fix sketch:**

```rust
pub async fn generate(&self, prompt: &str, max_tokens: usize) -> SpeculativeResult {
    let draft = load_draft_model(&self.draft_model.model_path)?;
    let target = load_target_model(&self.target_model.model_path)?;
    let mut accepted = 0usize;
    let mut all_tokens = Vec::new();

    while all_tokens.len() < max_tokens {
        let n = self.draft_model.num_draft_tokens.min(max_tokens - all_tokens.len());
        let draft_tokens = draft.generate_draft(prompt, &all_tokens, n).await?;
        let verdicts = target.verify_batch(prompt, &all_tokens, &draft_tokens).await?;

        let first_reject = verdicts.iter().position(|v| !v).unwrap_or(n);
        all_tokens.extend(draft_tokens[..first_reject].iter().cloned());
        accepted += first_reject;

        if first_reject < n {
            let resampled = target.generate_one(prompt, &all_tokens).await?;
            all_tokens.push(resampled);
        }
    }

    SpeculativeResult {
        output_tokens: all_tokens,
        total_draft_tokens: self.draft_model.num_draft_tokens,
        accepted_count: accepted,
        rejection_count: self.draft_model.num_draft_tokens - accepted.min(self.draft_model.num_draft_tokens),
        throughput_multiplier: accepted as f64 / self.draft_model.num_draft_tokens as f64,
        latency_savings_ms: start.elapsed().as_millis() as u64,
    }
}
```

---

## Pain Point 3: PTC Stubs — `mcp stubs` Always Returns Empty

**File:** `neotrix-core/src/cli/commands/agent_cmds.rs:350-359`
**Severity:** P1

**What's wrong:** The `mcp stubs` command (Programmatic Tool Calling) always returns an empty `Vec` because `McpRegistry.gateway()` is not implemented. The code creates `let stubs: Vec<serde_json::Value> = Vec::new();` — a hardcoded empty vector — then formats it with count 0. The `FIXME` comment at line 353 acknowledges this. PTC was absorbed as a core axiom (typed-stub tool invocation), but the entry point is dead.

**Impact:** Any agent relying on `mcp stubs` to discover available tool signatures for typed-stub invocation gets zero tools. The entire PTC workflow is silently broken.

**Fix sketch:**

```rust
"stubs" => {
    let gw = match McpRegistry::global().gateway() {
        Some(gw) => gw,
        None => {
            return CommandOutput::err("MCP gateway not initialized. Run `mcp connect` first.");
        }
    };
    let stubs: Vec<serde_json::Value> = gw.tool_schemas().iter().map(|schema| {
        let sig = schema_to_python_signature(schema);
        serde_json::json!({
            "name": schema.name,
            "signature": sig,
            "parameters": schema.parameters,
        })
    }).collect();
    let s = format!("🐍 PTC stubs: {} typed signatures\n", stubs.len());
    if want_json {
        return CommandOutput::ok(&s)
            .with_json(serde_json::json!({ "stubs": stubs, "count": stubs.len() }));
    }
    CommandOutput::ok(&s)
}
```

---

## Summary

| # | File | Severity | Pattern | Fix Effort |
|---|------|----------|---------|------------|
| 1 | `style_harmonizer.rs:113-165` | P1 | Hardcoded fake analysis results | Medium — needs image crate integration |
| 2 | `speculative_decoding.rs:161-185` | P0 | Fabricated throughput metrics, zero real decoding | High — needs draft+target model loading |
| 3 | `agent_cmds.rs:350-359` | P1 | PTC stubs always empty (`FIXME` known) | Low — gateway already partially exists |

**Common thread:** These are "silent lies" — code that returns plausible-looking results without doing real work, making debugging extremely difficult and eroding trust in the system's output signals.
