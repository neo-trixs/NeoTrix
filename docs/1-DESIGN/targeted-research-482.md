# Targeted Research #482 — Internal Pain Points (WISER Iteration)

**Date:** 2026-09-12
**Scope:** 3 hardcoded-stub / dead-code pain points in `neotrix-core/src/`
**Method:** Grep for TODO/FIXME/HACK/XXX, `#[allow(dead_code)]`, hardcoded return values

---

## Pain Point 1: `VisualConsistencyManager._fix_consistency` returns hardcoded success

**Location:** `l5_cognition/nt_core/visual/visual_consistency.rs:201-220`
**Severity:** P1

**What's wrong:** `_fix_consistency` always returns `success: true` with fabricated metrics (`consistency_score: 0.95`, `fix_time_ms: 2000`) and a path like `{input}_fixed.png`. No image is ever read, processed, or written. The `_element_id` and `_element_type` parameters are completely unused. Users calling this function believe consistency was fixed when nothing happened.

**Impact:** The entire visual consistency pipeline is a lie — downstream code trusts these fabricated scores, and quality metrics are inflated.

**Fix sketch:**

```rust
pub(crate) fn _fix_consistency(
    &mut self,
    image_path: &str,
    element_id: Option<&str>,
    element_type: _VisualElementType,
) -> _ConsistencyFixResult {
    let start = std::time::Instant::now();
    // Delegate to actual image processing (e.g. ADetailer / IP-Adapter via platform_gateway)
    let result = match self.invoke_consistency_engine(image_path, element_id, &element_type) {
        Ok(output) => _ConsistencyFixResult {
            success: true,
            fixed_image_path: Some(output.path),
            detected_elements: output.detected_count,
            fixed_elements: output.fixed_count,
            consistency_score: output.score,
            fix_time_ms: start.elapsed().as_millis() as u64,
            error: None,
        },
        Err(e) => _ConsistencyFixResult {
            success: false,
            fixed_image_path: None,
            detected_elements: 0,
            fixed_elements: 0,
            consistency_score: 0.0,
            fix_time_ms: start.elapsed().as_millis() as u64,
            error: Some(e.to_string()),
        },
    };
    self.fix_history.push(result.clone());
    result
}
```

---

## Pain Point 2: `ConsciousnessAgent.apply_patches` marks gaps as fixed without applying anything

**Location:** `l5_cognition/nt_core/nt_consciousness_core/agent.rs:314-328`
**Severity:** P1

**What's wrong:** `apply_patches` iterates over patches with `confidence >= 0.7`, calls `self.gap_registry.mark_fixed(&patch.gap_id)`, and increments a counter — but the actual `patch.apply()` call is commented out (`// patch.apply()`). This means the self-healing loop reports "N gaps fixed" each cycle while zero changes are made. The meta-learning path (`learn_meta_patterns`) then trains on fabricated fix counts.

**Impact:** The entire self-healing loop is non-functional. The system thinks it's healing itself. Gap counts never actually decrease.

**Fix sketch:**

```rust
fn apply_patches(&mut self, patches: &[Patch]) -> u32 {
    let mut fixed = 0;
    for patch in patches {
        if patch.confidence >= 0.7 {
            match patch.apply() {
                Ok(()) => {
                    self.gap_registry.mark_fixed(&patch.gap_id);
                    fixed += 1;
                }
                Err(e) => {
                    log::warn!("Patch apply failed for {}: {}", patch.gap_id, e);
                }
            }
        }
    }
    fixed
}
```

---

## Pain Point 3: `StyleHarmonizer._analyze_style` returns hardcoded analysis

**Location:** `l5_cognition/nt_core/visual/style_harmonizer.rs:113-128`
**Severity:** P1

**What's wrong:** `_analyze_style` returns identical hardcoded values regardless of input: `color_distribution: [0.3, 0.4, 0.3]`, `contrast: 0.7`, `saturation: 0.6`, `style_tags: ["cinematic"]`, `quality_score: 0.85`. The `_image_path` parameter is unused. This means style harmonization always produces the same "analysis" and downstream harmonization decisions are based on static data.

**Impact:** Style harmonization is deterministic and input-agnostic. Any pipeline relying on `_analyze_style` to compare or match styles gets identical results for every image.

**Fix sketch:**

```rust
pub(crate) fn _analyze_style(&self, image_path: &str) -> _StyleAnalysis {
    let img = match image::open(image_path) {
        Ok(img) => img,
        Err(e) => return _StyleAnalysis::default_with_error(&e.to_string()),
    };
    let rgb = img.to_rgb8();
    let color_distribution = compute_color_distribution(&rgb);
    let (contrast, saturation) = compute_contrast_saturation(&rgb);
    let dominant_colors = extract_dominant_colors(&rgb, 3);
    let style_tags = self.classify_style(&color_distribution, contrast, saturation);
    _StyleAnalysis {
        features: _Style特征 {
            color_distribution,
            contrast,
            saturation,
            color_temperature: estimate_temperature(&rgb),
            texture_features: compute_texture_features(&rgb),
            style_tags: style_tags.clone(),
        },
        dominant_colors,
        style_tags,
        quality_score: estimate_quality(&rgb),
    }
}
```

---

## Summary

| # | File:Line | Problem | Severity |
|---|-----------|---------|----------|
| 1 | `visual/visual_consistency.rs:201-220` | `_fix_consistency` always returns hardcoded success, never processes images | P1 |
| 2 | `nt_consciousness_core/agent.rs:314-328` | `apply_patches` marks gaps fixed without calling `patch.apply()` | P1 |
| 3 | `visual/style_harmonizer.rs:113-128` | `_analyze_style` returns static analysis, ignores input image | P1 |

**Pattern:** All three are "stub functions that lie about success" — they return fabricated positive results instead of either doing real work or returning explicit errors. This is worse than a `todo!()` panic because it silently corrupts downstream metrics and trust.

**Recommendation:** Either wire real implementations (invoke image SDK / platform gateway) or convert all stubs to `Err("not implemented: {function_name}")` so callers get honest failures.
