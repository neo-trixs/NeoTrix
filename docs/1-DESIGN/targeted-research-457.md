# Targeted Research #457 — Internal Pain Points

**Date**: 2026-09-12
**Approach**: WISER (internal-first pain point identification)
**Prior fixes**: 21 issues (FepIitBridge, AgentTeamSelfTest, HeartbeatAggregator, KB search, value_function, content_moderation, checkpoint_persistence, rate_limiter cleanup, temporal_continuity, verifier_agent, quality_control, unified_api, checkpoint cleanup, /kb embed, layered_qa execute_check, embedding search, HighestQuality routing, C2PA watermark, video_stitcher, video_post_processor)

---

## Pain Point 1 — P0: PublishGateway returns fake publish URLs

**File**: `neotrix-core/src/l1_action/nt_act/actions/publish_gateway.rs:176-206`

**What's wrong**: `publish()` returns `success: true` with `publish_url: "https://example.com/video/{task_id}"` and immediately marks the task as `Published` — no platform API is called. Any downstream code that trusts this result will believe the video was published when nothing happened.

**Severity**: **P0** — Data corruption; callers cannot distinguish real publishes from fakes.

**Fix sketch**:

```rust
// publish_gateway.rs:176 — replace stub with real dispatch
pub fn publish(&mut self, task_id: &str) -> PublishResult {
    if let Some(task) = self.tasks.iter_mut().find(|t| t.task_id == task_id) {
        task.status = PublishStatus::Uploading;
        task.updated_at = current_timestamp();
        
        let platform_api = self.get_platform_api(&task.config.platform);
        match platform_api.upload(&task.video_path, &task.metadata) {
            Ok(resp) => {
                task.status = PublishStatus::Published;
                task.updated_at = current_timestamp();
                let result = PublishResult {
                    success: true,
                    publish_url: Some(resp.url),
                    video_id: Some(resp.video_id),
                    status: PublishStatus::Published,
                    published_at: Some(current_timestamp()),
                    error: None,
                };
                self.history.push(result.clone());
                result
            }
            Err(e) => {
                task.status = PublishStatus::Failed;
                task.updated_at = current_timestamp();
                PublishResult {
                    success: false, publish_url: None, video_id: None,
                    status: PublishStatus::Failed, published_at: None,
                    error: Some(e.to_string()),
                }
            }
        }
    } else {
        PublishResult { success: false, /* ... */ }
    }
}
```

---

## Pain Point 2 — P0: ModelRouting returns fake success without calling any model

**File**: `neotrix-core/src/l1_action/nt_io/model_routing.rs:297-318`

**What's wrong**: `route()` returns `success: true`, `output: None`, `latency_ms: 5000` hardcoded — the model is never actually invoked. The caller gets back a "successful" routing response with no output and fake cost. This is the core LLM routing function; every agent turn depends on it.

**Severity**: **P0** — Core infrastructure non-functional; all model calls silently produce nothing.

**Fix sketch**:

```rust
// model_routing.rs:297 — replace stub with real LLM call
let response_text = self.providers
    .get(&model_id)
    .ok_or("provider not found")?
    .complete(&request.messages, &request.params)
    .await?;

let actual_latency = start.elapsed().as_millis() as u64;
let response = RoutingResponse {
    success: true,
    selected_model: model_id.clone(),
    output: Some(response_text),
    cost: self.estimate_cost(model_id, &request.messages),
    latency_ms: actual_latency,
    retries,
    error: None,
};
if let Some(state) = self.states.get_mut(&model_id) {
    state.total_requests += 1;
    state.total_cost += response.cost;
}
self.history.push(response.clone());
return response;
```

---

## Pain Point 3 — P1: StoryboardExtractor parses by line-split, not LLM

**File**: `neotrix-core/src/l5_cognition/nt_core/visual/storyboard_extractor.rs:216-243`

**What's wrong**: `parse_script_to_shots()` splits on newlines and produces one shot per line with `ShotSize::Medium`, `CameraMovement::Static`, no characters/scenes extracted. The entire module description says "LLM script→storyboard decomposition" but the LLM path is never invoked. Every shot gets identical defaults.

**Severity**: **P1** — Module produces garbage output; callers get nonsensical storyboards.

**Fix sketch**:

```rust
// storyboard_extractor.rs:216 — replace naive split with LLM call
fn parse_script_to_shots(&self, script_text: &str) -> Vec<Storyboard> {
    let prompt = format!(
        "Decompose this script into shots. For each shot return JSON: \
         {{shot_number, description, characters, scene, shot_size, \
          camera_movement, duration_secs, dialogue, emotion}}\n\n{}",
        script_text
    );
    match self.llm_client.complete(&prompt) {
        Ok(json) => serde_json::from_str::<Vec<Storyboard>>(&json)
            .unwrap_or_default(),
        Err(_) => {
            // fallback: line-split with at least meaningful defaults
            script_text.lines()
                .filter(|l| !l.trim().is_empty())
                .enumerate()
                .map(|(i, p)| Storyboard {
                    id: format!("shot_{}", i + 1),
                    shot_number: (i + 1) as u32,
                    description: p.to_string(),
                    shot_size: ShotSize::Medium,
                    camera_movement: CameraMovement::Static,
                    ..Default::default()
                })
                .collect()
        }
    }
}
```

---

## Summary

| # | Severity | File | Issue |
|---|----------|------|-------|
| 1 | **P0** | `publish_gateway.rs:176` | Returns fake `example.com` URLs, no API call |
| 2 | **P0** | `model_routing.rs:297` | Returns fake success with `output: None`, no LLM call |
| 3 | **P1** | `storyboard_extractor.rs:216` | Line-split stub instead of LLM decomposition |

All three are production-path functions that silently return fabricated results. P0 issues should be fixed before any release; P1 before beta.
