# Targeted Research 526 — Internal Pain Points (Round 28+)

> WISER iteration: 3 hardcoded-stub / dead-bridge pain points found in `neotrix-core/src/`.

---

## Pain Point 1: `e8_abduction_bridge.rs` — Hardcoded External Path

**File:** `neotrix-core/src/core/nt_core_e8/e8_abduction_bridge.rs:34`
**Severity:** P1

### What's Wrong

The bridge unconditionally tries to load a causal graph from a hardcoded external volume path:

```rust
let cortex_path = Path::new("/Volumes/NeoTrixBrain/working/causal_graph.json");
```

This path only exists on one developer's machine. On CI, other devs, or production, `cortex_path.exists()` silently returns `false` and the engine silently degrades to its built-in graph — no error, no config option, no fallback path. The caller has no way to know the cortex graph is missing or to override the path. Additionally, the `TODO(fusion-plan-215)` at line 9 signals this bridge should merge into `E8EwhrBridge` but remains a separate 241-line module duplicating state-prediction logic.

### Fix Sketch

```rust
// Replace hardcoded path with config-driven resolution
pub fn new(blend_weight: f64, cortex_path_override: Option<&Path>) -> Self {
    let mut abductive_engine = AbductiveReasoningEngine::new();
    let cortex_path = cortex_path_override
        .map(|p| p.to_path_buf())
        .or_else(|| dirs::home_dir().map(|h| h.join(".neotrix").join("causal_graph.json")))
        .unwrap_or_default();
    if cortex_path.exists() {
        if let Err(e) = abductive_engine.load_cortex_causal_graph(&cortex_path) {
            tracing::warn!(path = %cortex_path.display(), err = %e, "cortex graph load skipped");
        }
    } else {
        tracing::info!("no cortex graph found at {}, using built-in graph", cortex_path.display());
    }
    // ...
}
```

---

## Pain Point 2: `nt_feel_vtuber.rs` — All Sensory Inputs Return Static Neutral

**File:** `neotrix-core/src/l4_emotion/nt_feel/nt_feel_vtuber.rs:207-308`
**Severity:** P1

### What's Wrong

Three multimodal detection methods are stub implementations that always return the same static result regardless of input:

- `detect_from_voice(&self, _audio: &[u8])` — returns `Neutral, intensity: 0.5` always (line 207-216)
- `detect_from_visual(&self, _image: &[u8])` — returns `Neutral, intensity: 0.5` always (line 219-228)
- `synthesize_speech(&self, ...)` — returns empty `audio: vec![]` with estimated duration (line 294-301)
- `transcribe_speech(&self, _audio: &[u8])` — returns `Ok("".into())` (line 305-307)

Any downstream consumer calling `detect_from_voice` or `detect_from_visual` gets a guaranteed Neutral reading — the vtuber emotion pipeline is effectively text-only. The TTS/STT stubs return empty audio / empty string, making the voice pipeline non-functional.

### Fix Sketch

At minimum, return `Err` for unimplemented paths so callers know the capability is absent:

```rust
pub fn detect_from_voice(&self, _audio: &[u8]) -> Result<_EmotionReading, String> {
    // Option A: delegate to existing nt_feel emotion engine via trait object
    // Option B: explicit not-implemented so callers degrade gracefully
    Err("voice emotion detection not yet wired — requires ML backend".into())
}

pub fn synthesize_speech(&self, text: &str, emotion: &_EmotionType) -> Result<_VoiceOutput, String> {
    Err(format!("TTS not configured — cannot synthesize: {} chars", text.len()))
}
```

---

## Pain Point 3: `verifier_agent.rs` — `auto_correct_prompt` Ignores LLM

**File:** `neotrix-core/src/l6_meta/coordination/verifier_agent.rs:336-345`
**Severity:** P2

### What's Wrong

The `auto_correct_prompt` method claims to "automatically correct prompts" but does naive string concatenation instead of calling an LLM:

```rust
fn auto_correct_prompt(&self, prompt: &str, result: &VerificationResult) -> String {
    // TODO: 实际调用 LLM 修正提示词
    let mut corrected = prompt.to_string();
    for correction in &result.suggested_corrections {
        corrected.push_str(&format!(" {}", correction));
    }
    corrected
}
```

`suggested_corrections` is always `vec![]` (line 209), so this function is a no-op identity — it returns the original prompt unchanged. The caller at line 328-332 builds a `RegenerationRequest` with a `corrected_prompt` that is identical to the original, making the entire regeneration feedback loop inert.

### Fix Sketch

```rust
fn auto_correct_prompt(&self, prompt: &str, result: &VerificationResult) -> String {
    if result.suggested_corrections.is_empty() {
        return prompt.to_string();
    }
    // Build a structured correction instruction for the LLM
    let correction_block = result.suggested_corrections.iter()
        .map(|c| format!("- {}", c))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "Original prompt:\n{}\n\nApply these corrections:\n{}\n\nRevised prompt:",
        prompt, correction_block
    )
    // The caller should pass this to the LLM provider for actual revision
}
```

Combined with populating `suggested_corrections` in `simulate_verification`, this creates a functional feedback loop.

---

## Summary

| # | File:Line | Issue | Severity | Effort |
|---|-----------|-------|----------|--------|
| 1 | `e8_abduction_bridge.rs:34` | Hardcoded volume path, silent degradation | P1 | S |
| 2 | `nt_feel_vtuber.rs:207-308` | All sensory inputs return static Neutral/empty | P1 | M |
| 3 | `verifier_agent.rs:336-345` | Prompt correction is identity no-op | P2 | S |
