# Targeted Research #508 — WISER Internal Pain Points

**Date:** 2026-09-13  
**Scope:** neotrix-core/src/ — TODO stubs, hardcoded returns, dead code  
**Previous fixes:** 27 critical issues (issues 1–27 in prior sessions)

---

## Pain Point 1: VTuber Emotion Engine — Hardcoded Neutral Returns

**File:** `neotrix-core/src/l4_emotion/nt_feel/nt_feel_vtuber.rs:207-228`

**What's wrong:** `detect_from_voice()` and `detect_from_visual()` always return `Neutral` with `intensity: 0.5`, ignoring the input entirely. The `synthesize_speech()` method at line 295 returns an empty audio vec (`vec![]`), and `transcribe_speech()` at line 306 returns `Ok("".into())`. These are 4 functions in a public API that silently lie about their capabilities.

**Severity:** P1 — Users calling these get fake results with no error, corrupting downstream emotion state.

**Fix sketch:**
```rust
pub fn detect_from_voice(&self, _audio: &[u8]) -> Result<_EmotionReading, String> {
    // Return explicit "not implemented" instead of fake Neutral
    if self.voice_model.is_none() {
        return Err("Voice emotion model not loaded. \
            Load a model via VtuberEngine::with_voice_model() first.".into());
    }
    // ... actual inference ...
}

pub fn synthesize_speech(&self, text: &str, emotion: &_EmotionType) -> Result<_VoiceOutput, String> {
    let engine = self.tts_engine.as_ref()
        .ok_or("TTS engine not initialized. Call set_tts_engine() first.")?;
    let audio = engine.synthesize(text, emotion)?;
    Ok(_VoiceOutput { audio, text: text.to_string(), emotion: emotion.clone(), duration_ms: audio.len() as u64 / 16 })
}
```

---

## Pain Point 2: Production Orchestrator — No-Op Checkpoint Persistence

**File:** `neotrix-core/src/l1_action/nt_act/actions/orchestration/production_orchestrator.rs:224-242`

**What's wrong:** `save_checkpoint()` discards the workflow data (`let _ = workflow;`) and returns `Ok(())`. `restore_from_checkpoint()` just sets status to `Paused` without restoring any state. A crash during workflow execution loses all progress — the checkpoint system is purely decorative.

**Severity:** P0 — Long-running multi-step workflows (batch production, video pipelines) lose all progress on crash. This defeats the purpose of having a production orchestrator.

**Fix sketch:**
```rust
pub fn save_checkpoint(&self, workflow_id: &str) -> Result<(), String> {
    let workflow = self.workflows.get(workflow_id)
        .ok_or("Workflow not found")?;
    let path = Path::new(&self.config.checkpoint_dir)
        .join(format!("{}.ckpt.json", workflow_id));
    let json = serde_json::to_vec_pretty(workflow)
        .map_err(|e| e.to_string())?;
    std::fs::create_dir_all(path.parent().unwrap())
        .and_then(|_| std::fs::write(&path, json))
        .map_err(|e| e.to_string())
}

pub fn restore_from_checkpoint(&mut self, workflow_id: &str) -> Result<(), String> {
    let path = Path::new(&self.config.checkpoint_dir)
        .join(format!("{}.ckpt.json", workflow_id));
    let json = std::fs::read_to_string(&path)
        .map_err(|_| format!("No checkpoint found for {}", workflow_id))?;
    let restored: Workflow = serde_json::from_str(&json)
        .map_err(|e| e.to_string())?;
    self.workflows.insert(workflow_id.to_string(), restored);
    Ok(())
}
```

---

## Pain Point 3: Model Adapter — Fake Similarity Scores and Timing

**File:** `neotrix-core/src/l1_action/nt_io/model_adapter.rs:157-228`

**What's wrong:** `apply_lora()`, `_apply_ip_adapter()`, and `_apply_controlnet()` all return `success: true` with fabricated `application_time_ms` (1000/2000/1500) and `similarity_score` (0.95/0.88/0.85). They never actually call any model. The output paths are format strings with no verification the file exists. Consumers using these metrics for quality decisions get meaningless data.

**Severity:** P1 — Quality control pipelines consume `similarity_score` to decide whether to accept/reject generated content. Fake scores bypass the entire quality gate.

**Fix sketch:**
```rust
pub fn apply_lora(&mut self, lora_id: &str, input_path: &str, strength: f32) -> AdapterResult {
    let start = Instant::now();
    let adapter = match self.adapters.get(lora_id) {
        Some(a) => a,
        None => return AdapterResult {
            success: false, output_path: String::new(),
            adapter_name: String::new(), application_time_ms: 0,
            similarity_score: 0.0,
            error: Some(format!("Adapter '{}' not found", lora_id)),
        },
    };
    let output_path = match self.run_lora_inference(adapter, input_path, strength) {
        Ok(p) => p,
        Err(e) => return AdapterResult {
            success: false, output_path: String::new(),
            adapter_name: adapter.name.clone(), application_time_ms: start.elapsed().as_millis() as u64,
            similarity_score: 0.0, error: Some(e),
        },
    };
    AdapterResult {
        success: true, output_path, adapter_name: adapter.name.clone(),
        application_time_ms: start.elapsed().as_millis() as u64,
        similarity_score: self.compute_similarity(input_path, &output_path),
        error: None,
    }
}
```

---

## Summary

| # | Location | Issue | Severity | Impact |
|---|----------|-------|----------|--------|
| 1 | `nt_feel_vtuber.rs:207-306` | 4 functions return hardcoded Neutral/empty | P1 | Emotion state corruption |
| 2 | `production_orchestrator.rs:224-242` | Checkpoint save/restore are no-ops | P0 | Workflow progress loss on crash |
| 3 | `model_adapter.rs:157-228` | Fake similarity scores and timing | P1 | Quality gate bypass |

**Priority order:** P0 (fix checkpoint first — permanent data loss on crash), then P1s (stubs that lie about capabilities).
