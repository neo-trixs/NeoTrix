# Targeted Research 529 — Internal Pain Points (Iteration Loop Round 28)

**Date**: 2026-09-13
**Scope**: Stub functions returning fabricated results + dead PTC path

---

## Pain Point 1: PTC exec/stubs silently return empty results

**File**: `neotrix-core/src/cli/commands/agent_cmds.rs:353-359` (stubs), `:467-468` (exec)
**Severity**: P0
**What's wrong**: Both `mcp stubs` and `mcp exec` commands hit `FIXME: McpRegistry.gateway() not yet implemented`. They silently return `Vec::new()` — zero stubs, zero results — while printing success output. The user sees "0 typed signatures" or "0 call(s)" with an `ok` status, giving no indication the feature is broken. This is a **user-facing CLI command** with zero signal that it's non-functional.

**Fix sketch**:
```rust
// agent_cmds.rs:353 — stubs command
"stubs" => {
    // FIXME: McpRegistry.gateway() not yet implemented
    let stubs: Vec<serde_json::Value> = Vec::new();
    let s = format!("🐍 PTC stubs: {} typed signatures\n", stubs.len());
    if want_json {
        return CommandOutput::ok(&s).with_json(serde_json::json!({ "stubs": stubs, "count": stubs.len() }));
    }
    CommandOutput::ok(&s)
}

// FIX: Return an explicit NOT_IMPLEMENTED status
"stubs" => {
    CommandOutput::err("PTC stubs not yet implemented — McpRegistry.gateway() pending. Use `mcp list` for registered servers.")
}

// agent_cmds.rs:467 — exec command
// FIX: Same pattern — fail loudly instead of returning empty vec
"exec" => {
    // ... after plan validation ...
    CommandOutput::err("PTC exec not yet implemented — McpRegistry.gateway() pending. Stages validated but cannot execute without gateway.")
}
```

---

## Pain Point 2: Style harmonizer returns fabricated analysis scores

**File**: `neotrix-core/src/l5_cognition/nt_core/visual/style_harmonizer.rs:113-165`
**Severity**: P1
**What's wrong**: Three methods — `_analyze_style`, `_harmonize`, `_match_colors` — all return hardcoded constants instead of processing any image data. `_analyze_style` returns the same color_distribution `[0.3, 0.4, 0.3]` and `quality_score: 0.85` for *every* image. `_match_colors` returns `style_similarity: 0.92` and a fake 1000ms timing. Any caller using these scores for downstream decisions (e.g., threshold routing, quality gates) gets fabricated data. The functions accept `image_path` but never read the file.

**Fix sketch**:
```rust
// style_harmonizer.rs:113 — add an early-return stub marker
pub(crate) fn _analyze_style(&self, image_path: &str) -> _StyleAnalysis {
    // Return a sentinel indicating "not yet analyzed" instead of fake data
    _StyleAnalysis {
        features: _Style特征 {
            color_distribution: vec![],   // empty = not computed
            contrast: -1.0,               // sentinel: uninitialized
            saturation: -1.0,
            color_temperature: 0.0,
            texture_features: vec![],
            style_tags: vec!["STUB_NOT_IMPLEMENTED".to_string()],
        },
        dominant_colors: vec![],
        style_tags: vec!["STUB_NOT_IMPLEMENTED".to_string()],
        quality_score: -1.0,  // negative = not computed
    }
}
// Same pattern for _harmonize and _match_colors: set success=false, error=Some("not implemented")
```

---

## Pain Point 3: Visual consistency fixer returns fabricated fix results

**File**: `neotrix-core/src/l5_cognition/nt_core/visual/visual_consistency.rs:207-219`
**Severity**: P1
**What's wrong**: `_fix_consistency` accepts an image path and element ID, then returns `success: true` with `consistency_score: 0.95` and `fixed_elements: 1` — without reading the image, detecting any elements, or performing any fix. `_batch_fix` calls this in a loop, multiplying the fabrication. Any consumer trusting `success: true` will assume the image was actually fixed and pass it downstream (e.g., to rendering or publishing), producing inconsistent output with no error.

**Fix sketch**:
```rust
// visual_consistency.rs:207
pub(crate) fn _fix_consistency(
    &mut self,
    image_path: &str,
    _element_id: Option<&str>,
    _element_type: _VisualElementType,
) -> _ConsistencyFixResult {
    let result = _ConsistencyFixResult {
        success: false,  // FIX: was always true
        fixed_image_path: None,  // FIX: was Some(fake_path)
        detected_elements: 0,
        fixed_elements: 0,
        consistency_score: 0.0,
        fix_time_ms: 0,
        error: Some(format!(
            "Visual consistency fix not implemented — {} is a stub. \
             Use ImageSuperResolver for real image processing.",
            image_path
        )),
    };
    self.fix_history.push(result.clone());
    result
}
```

---

## Summary

| # | Location | Issue | Severity | Category |
|---|----------|-------|----------|----------|
| 1 | `agent_cmds.rs:353,467` | PTC stubs/exec return empty Vec on success | P0 | Dead code path / silent failure |
| 2 | `style_harmonizer.rs:113-165` | All 3 methods return fabricated scores | P1 | Hardcoded return / stub-as-production |
| 3 | `visual_consistency.rs:207-219` | Fix function returns fake success + score | P1 | Hardcoded return / stub-as-production |

**Pattern**: These are all **"stub-as-production"** functions — they have real signatures, are wired into production paths, but return fabricated constants. The danger is not that they exist (stubs are fine during development) but that they return `success: true` / plausible-looking numbers, making downstream code behave as if real work happened. The fix is to either: (a) return explicit failure with descriptive error, or (b) gate the function behind a `cfg(feature = "visual-engine")` so it can't be called in production builds.
