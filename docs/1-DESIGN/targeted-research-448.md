# Targeted Research #448 — Internal Pain Points (Round 2)

**Date:** 2026-09-12
**Method:** Internal-first pain point grep (TODO/FIXME + hardcoded stubs + dead code)

## Pain Point 1: UnifiedApiImpl::handle is a non-functional stub (P0)

**File:** `neotrix-core/src/neotrix/nt_unified_api/mod.rs:366-394`

**What's wrong:** The entire unified API entry point — the single surface that routes requests across L1-L6 layers — returns a hardcoded string `"统一 API 待实现"` with zeroed-out metadata (phi=0.0, coherence=0.0, confidence=0.0). Every caller hitting this path gets a fake response. The 4-step comment (parse intent → GWT route → dispatch → aggregate) describes the correct design but none of it is implemented.

**Severity:** P0 — this is the public API surface for the whole system.

**Fix sketch:**

```rust
async fn handle(&self, request: UnifiedRequest) -> Result<UnifiedResponse, UnifiedError> {
    // 1. Intent parsing via ConsciousnessTree
    let intent = self.intent_parser.parse(&request.content).await?;
    let mode = intent.infer_response_mode();

    // 2. GWT attention routing
    let salient = self.gwt_router.route(&intent, &self.subsystems).await?;

    // 3. Dispatch to capable layer
    let result = salient.dispatch(request).await?;

    // 4. Aggregate with real metadata
    Ok(UnifiedResponse {
        response_id: uuid::Uuid::new_v4().to_string(),
        session_id: request.session_id.unwrap_or_default(),
        content: result.content,
        payload: result.payload,
        message_type: result.message_type,
        metadata: ResponseMetadata {
            duration_ms: start.elapsed().as_millis() as u64,
            layers_involved: salient.layers_touched(),
            capabilities_used: salient.capabilities_used(),
            consciousness_state: self.consciousness.snapshot(),
            confidence: salient.confidence(),
        },
        is_stream_chunk: false,
        stream_done: true,
    })
}
```

---

## Pain Point 2: VerifierAgent uses heuristic keyword matching instead of VLM (P1)

**File:** `neotrix-core/src/l6_meta/coordination/verifier_agent.rs:195-232, 336-344`

**What's wrong:** `_verify_shot()` calls `simulate_verification()` which scores video quality using `str::contains("character")` — literal keyword matching on the spec description. The `auto_correct_prompt()` appends correction strings verbatim instead of using an LLM to rewrite the prompt. The entire verification quality gate is smoke-and-mirrors: it always passes non-entity content with score 9/10 and "corrects" by string concatenation.

**Severity:** P1 — the quality control pipeline silently passes bad content.

**Fix sketch:**

```rust
async fn _verify_shot(&mut self, shot_id: &str, video_path: &str, spec: &str, ctx: Option<&str>) -> VerificationResult {
    let start = Instant::now();

    // Real VLM verification: send frame + spec to vision model
    let vlm_result = self.vlm_provider.verify_frame(video_path, spec).await;

    let scores = vec![
        _VerificationScore { dimension: "entity_consistency".into(), score: vlm_result.entity_score },
        _VerificationScore { dimension: "environment_consistency".into(), score: vlm_result.env_score },
        _VerificationScore { dimension: "audio_sync".into(), score: vlm_result.audio_score },
    ];

    let total = scores.iter().map(|s| s.score).sum::<f32>() / scores.len() as f32;
    let passed = total >= self.config.pass_threshold;

    VerificationResult { passed, total_score: total, scores, ..Default::default() }
}

async fn auto_correct_prompt(&self, prompt: &str, result: &VerificationResult) -> String {
    // Use LLM to rewrite prompt based on verification failures
    let correction_ctx = format!("Original prompt: {}\nFailures: {:?}", prompt, result.error_types);
    self.llm_provider.complete(&correction_ctx).await
}
```

---

## Pain Point 3: ConsciousnessAgent uses hardcoded snapshots, patches never applied (P1)

**File:** `neotrix-core/src/l5_cognition/nt_core/nt_consciousness_core/agent.rs:270-276, 318-323`

**What's wrong:** `take_snapshot()` computes phi and coherence using `(self.cycle * 0.001).min(0.5)` — deterministic pseudo-values that drift linearly and never reflect actual system state. `apply_patches()` marks gaps as fixed (`mark_fixed`) but the actual `patch.apply()` call is commented out, so the gap registry lies about what's been repaired.

**Severity:** P1 — the self-healing loop operates on fabricated data and claims false fixes.

**Fix sketch:**

```rust
fn take_snapshot(&mut self) {
    let system = &self.system_health; // injected dependency
    self.state_snapshot = StateSnapshot::new(
        self.cycle,
        system.compute_phi(),        // real IIT integration measure
        system.compute_coherence(),   // real cross-module coherence
    );
}

fn apply_patches(&mut self, patches: &[Patch]) -> u32 {
    let mut fixed = 0;
    for patch in patches {
        if patch.confidence >= 0.7 {
            match patch.apply() {       // actually execute the patch
                Ok(()) => {
                    self.gap_registry.mark_fixed(&patch.gap_id);
                    fixed += 1;
                }
                Err(e) => tracing::warn!("Patch {} failed: {}", patch.id, e),
            }
        }
    }
    fixed
}
```

---

## Summary

| # | Pain Point | File:Line | Severity | Fix Effort |
|---|-----------|-----------|----------|------------|
| 1 | UnifiedApi returns stub `"统一 API 待实现"` | `nt_unified_api/mod.rs:366` | **P0** | Medium (needs GWT dispatch wiring) |
| 2 | Verifier uses `str::contains` instead of VLM | `verifier_agent.rs:195` | **P1** | Low (swap heuristic for provider call) |
| 3 | Consciousness snapshot hardcoded, patches fake-applied | `agent.rs:271,320` | **P1** | Low (uncomment + inject real health source) |
