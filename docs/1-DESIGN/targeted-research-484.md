# Targeted Research #484 — Internal Pain Points (Iteration Loop)

**Date:** 2026-09-12
**Method:** Grep TODO/FIXME/HACK + hardcoded-return stubs + dead code paths in `neotrix-core/src/`
**Scope:** Production code only (excluding `#[cfg(test)]` and test modules)

---

## Pain Point 1: ModelAdapter — LoRA/IP-Adapter/ControlNet Are All Hardcoded Stubs

**Location:** `neotrix-core/src/l1_action/nt_io/model_adapter.rs:157-223`

**What's wrong:** `apply_lora()`, `_apply_ip_adapter()`, and `_apply_controlnet()` all return fabricated `AdapterResult` structs with hardcoded `success: true`, fake `application_time_ms`, and invented `similarity_score`. The actual model invocation logic is entirely absent — a TODO comment sits where the real work should be. Any downstream consumer trusting these results will believe a LoRA was applied when nothing happened.

**Severity:** P0

**Impact:** Users configuring visual consistency adapters (FaceConsistencyManager, ReferenceBasedGeneration) get fake results. The production pipeline silently produces unmodified images claiming they were transformed.

**Fix sketch:**

```rust
pub fn apply_lora(
    &mut self,
    lora_id: &str,
    input_path: &str,
    strength: f32,
) -> AdapterResult {
    let adapter = self.adapters.get(lora_id)
        .ok_or_else(|| format!("LoRA adapter '{}' not found", lora_id));

    let start = Instant::now();
    let result = match adapter {
        Ok(a) => {
            // Delegate to registered backend (ComfyUI/SD WebUI/etc.)
            let output = self.platform_gateway.execute_lora(
                &a.backend, lora_id, input_path, strength
            )?;
            AdapterResult {
                success: true,
                output_path: output.path,
                adapter_name: a.name.clone(),
                application_time_ms: start.elapsed().as_millis() as u64,
                similarity_score: output.similarity,
                error: None,
            }
        }
        Err(e) => AdapterResult {
            success: false,
            output_path: String::new(),
            adapter_name: String::new(),
            application_time_ms: 0,
            similarity_score: 0.0,
            error: Some(e),
        },
    };
    self.history.push(result.clone());
    result
}
```

---

## Pain Point 2: VideoPostProcessor — denoise()/sharpen() Return Fabricated Metrics

**Location:** `neotrix-core/src/l3_embodiment/nt_physical/video_post_processor.rs:341-369`

**What's wrong:** Both `denoise()` and `sharpen()` return `_PostProcessResult` with hardcoded scores (0.89, 0.87, 0.85, etc.) and fabricated `processed_frames: 150`. No actual frame processing occurs. The functions claim the output file is `video_denoised.mp4` / `video_sharpened.mp4` but never produce it. The `grade()` method in `lut_color_grading.rs:264-307` has the same problem — it generates a LUT in memory but never applies it to the video file.

**Severity:** P0

**Impact:** VideoPostProcessor is consumed by VideoPostProcessor (通用). Users calling denoise/sharpen/color-grade on video files get fabricated quality scores while the output file is either missing or unchanged. Any quality-gate relying on these scores will pass unprocessed content.

**Fix sketch:**

```rust
pub fn denoise(&self, video_path: &str) -> _PostProcessResult {
    let start = Instant::now();
    let output_path = format!("{}_denoised.mp4", video_path);
    // Delegate to ffmpeg or internal denoise pipeline
    let status = std::process::Command::new("ffmpeg")
        .args(["-i", video_path, "-vf", "nlmeans=s=3:p=7:r=3", &output_path])
        .status();

    match status {
        Ok(s) if s.success() => _PostProcessResult {
            success: true,
            processed_video_path: Some(output_path),
            processed_frames: count_frames(&output_path).unwrap_or(0),
            color_consistency_score: 0.0,
            temporal_stability_score: 0.0,
            quality_improvement_score: measure_psnr(video_path, &output_path),
            processing_time_ms: start.elapsed().as_millis() as u64,
            error: None,
        },
        Ok(e) => _PostProcessResult {
            success: false, processed_video_path: None,
            processed_frames: 0, color_consistency_score: 0.0,
            temporal_stability_score: 0.0, quality_improvement_score: 0.0,
            processing_time_ms: 0,
            error: Some(format!("ffmpeg denoise failed: {e}")),
        },
        Err(e) => _PostProcessResult {
            success: false, processed_video_path: None,
            processed_frames: 0, color_consistency_score: 0.0,
            temporal_stability_score: 0.0, quality_improvement_score: 0.0,
            processing_time_ms: 0, error: Some(format!("ffmpeg not found: {e}")),
        },
    }
}
```

---

## Pain Point 3: AgentCmds "stubs" — PTC Stubs Always Return Empty Vec

**Location:** `neotrix-core/src/cli/commands/agent_cmds.rs:350-359`

**What's wrong:** The `"stubs"` subcommand in agent bridge is marked `FIXME: McpRegistry.gateway() not yet implemented` and hardcodes `let stubs: Vec<serde_json::Value> = Vec::new()`. The `agent bridge stubs` command always reports 0 typed signatures, making the Programmatic Tool Calling (PTC) feature appear non-functional to any user or automation that queries it. This is the only user-facing entry point for PTC stubs.

**Severity:** P1

**Impact:** PTC (typed-stub tool calling) is advertised as a core capability in AGENTS.md. The CLI reports "0 typed signatures" regardless of configured MCP servers. Users attempting to use PTC from the CLI get an empty result, degrading trust in the MCP integration layer.

**Fix sketch:**

```rust
"stubs" => {
    use crate::neotrix::nt_io_mcp_registry::McpRegistry;
    let registry = McpRegistry::global();
    let stubs: Vec<serde_json::Value> = registry
        .gateway()
        .map(|gw| gw.tool_schemas().iter().map(|s| {
            serde_json::json!({
                "name": s.name,
                "signature": s.python_signature(),
                "description": s.description,
            })
        }).collect())
        .unwrap_or_default();
    let s = format!("PTC stubs: {} typed signatures\n", stubs.len());
    if want_json {
        return CommandOutput::ok(&s)
            .with_json(serde_json::json!({ "stubs": stubs, "count": stubs.len() }));
    }
    CommandOutput::ok(&s)
}
```

---

## Summary

| # | Pain Point | File:Line | Severity | Category |
|---|-----------|-----------|----------|----------|
| 1 | ModelAdapter LoRA/IP-Adapter/ControlNet stubs return fabricated results | `model_adapter.rs:157-223` | P0 | Hardcoded fake output |
| 2 | VideoPostProcessor denoise/sharpen hardcoded scores, no frame processing | `video_post_processor.rs:341-369` | P0 | Hardcoded fake output |
| 3 | AgentCmds PTC stubs always empty — McpRegistry.gateway() unimplemented | `agent_cmds.rs:350-359` | P1 | Dead code path |

**Pattern observed:** 53+ TODO stubs remain in production code across `nt_physical` (video processing), `nt_io` (model adapters), `nt_feel` (TTS/STT), and `cli/commands` (MCP bridge). The most dangerous are the ones returning fabricated `success: true` results — they silently deceive downstream consumers.
