# Targeted Research #522 — Internal Pain Points (Batch 5)

**Date**: 2026-09-13
**Scope**: neotrix-core/src/ — stub functions, hardcoded returns, dead code paths

---

## Pain Point 1: VTuber Emotion Engine — All Sensory Inputs Are Hardcoded Stubs

**File**: `neotrix-core/src/l4_emotion/nt_feel/nt_feel_vtuber.rs:207-308`

**What's wrong**: Four critical sensory functions return fabricated data:
- `detect_from_voice()` (line 207): Always returns `Neutral, intensity=0.5` regardless of audio input
- `detect_from_visual()` (line 219): Always returns `Neutral, intensity=0.5` regardless of image input
- `synthesize_speech()` (line 290): Returns empty `audio: vec![]` with a duration estimate
- `transcribe_speech()` (line 305): Returns empty string `""`

Any downstream consumer (VTuber rendering, emotion-driven responses) silently gets fake data. The `voice` field in `generate_response()` is always `None` (line 284).

**Severity**: P1 — VTuber domain is user-facing; fake emotion readings produce visibly wrong behavior.

**Fix sketch**:
```rust
pub fn detect_from_voice(&self, audio: &[u8]) -> Result<_EmotionReading, String> {
    if audio.len() < 100 {
        return Err("Audio too short for analysis".into());
    }
    // Delegate to LLM provider via nt_core_llm::provider::transcribe
    // For now: fail loudly instead of returning fake Neutral
    Err("Voice emotion detection not yet wired — use detect_from_text() or implement ML pipeline".into())
}

pub fn detect_from_visual(&self, image: &[u8]) -> Result<_EmotionReading, String> {
    if image.len() < 200 {
        return Err("Image too small for analysis".into());
    }
    // Wire to vision model: send image, parse emotion response
    Err("Visual emotion detection not yet wired — use detect_from_text() or implement CV pipeline".into())
}
```

**Principle**: Fail-loud (`Err`) beats silent-lie (`Ok(fake_data)`). Callers that receive `Err` can degrade gracefully; callers that receive fake `Ok` propagate wrong state.

---

## Pain Point 2: VerifierAgent Simulates VLM Verification With Keyword Matching

**File**: `neotrix-core/src/l6_meta/coordination/verifier_agent.rs:222-264`

**What's wrong**: `simulate_verification()` computes verification scores using `str::contains()` on description text instead of calling an actual vision-language model. Key behaviors:
- Entity consistency score is 7 if the description contains "character"/"face", otherwise 9 (line 227-232)
- Instruction following is scored purely by description length (line 252-258)
- No actual image/video comparison occurs

The `_verify_shot()` method (line 186) calls this and gates regeneration decisions on the result. A video that perfectly matches its spec but has a short description gets a 5/10 instruction score, triggering unnecessary regeneration. Conversely, a verbose but wrong description gets a high score.

**Severity**: P1 — Gates production regeneration decisions; incorrect scores waste compute or miss defects.

**Fix sketch**:
```rust
fn simulate_verification(&self, description: &str, context: Option<&str>) -> Vec<_VerificationScore> {
    // Keep as fallback, but add real VLM path:
    // if let Some(vlm) = &self.vlm_client {
    //     return vlm.verify_video(description, &self.video_path, context);
    // }
    // Fallback: log warning that we're in simulation mode
    tracing::warn!("VerifierAgent running in SIMULATION mode — no VLM wired");
    // ... existing keyword logic as fallback ...
}
```

**Principle**: Simulation is acceptable as a fallback, but must be logged so operators know results are approximate.

---

## Pain Point 3: Network Scanner Returns Hardcoded Fake Host Inventory

**File**: `neotrix-core/src/l3_embodiment/nt_shield/nt_shield_impl/nt_shield_internal_scan.rs:173-206`

**What's wrong**: `discover_hosts()` ignores the actual network and returns a hardcoded list of 3 hosts (192.168.1.1/10/25) with fabricated hostnames, OS versions, MAC addresses, and open ports. `enumerate_services()` (line 210) similarly returns hardcoded service data matched on IP string.

This is in NT-SHIELD (security domain). A user running internal network discovery gets a completely fabricated topology. Any downstream security analysis (vulnerability scanning, audit) operates on fictional data.

**Severity**: P0 — Security-critical function returning fabricated data; false sense of safety.

**Fix sketch**:
```rust
pub async fn discover_hosts(&mut self) -> Result<Vec<_HostInfo>, String> {
    // Check if network tools are available
    if !self.has_network_tools().await {
        return Err(
            "No network discovery tools available (fscan/nmap). \
             Install fscan or nmap, or run in --simulation mode explicitly.".into()
        );
    }
    let output = self.run_fscan(&self.config.target_range).await?;
    let hosts = self.parse_fscan_output(&output)?;
    self.internal_hosts = hosts.clone();
    Ok(hosts)
}
```

**Principle**: Security tooling must never fabricate data. If tools are missing, fail with actionable error. Add a `--simulation` flag for testing that explicitly returns mock data with a banner warning.

---

## Summary

| # | File | Issue | Severity |
|---|------|-------|----------|
| 1 | `nt_feel_vtuber.rs:207-308` | Voice/visual/TTS/STT always return fake data | P1 |
| 2 | `verifier_agent.rs:222-264` | VLM verification simulated via keyword matching | P1 |
| 3 | `nt_shield_internal_scan.rs:173-206` | Network scanner returns hardcoded fake hosts | P0 |

**Cross-cutting theme**: Silent fabrication. All three pain points return `Ok(result)` with fabricated data instead of `Err(unsupported)`. This violates the principle of fail-loud and makes downstream code believe it's operating on real data.
