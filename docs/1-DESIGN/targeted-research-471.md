# Targeted Research 471 — Internal Pain Point Scan

**Date**: 2026-09-12
**Scope**: neotrix-core/src/ — stub implementations, hardcoded returns, dead code
**Method**: Grep for TODO/FIXME/HACK, `Ok(vec![])`, `#[allow(dead_code)]`, stubs

---

## Pain Point 1 — P0: `AgentTeam::execute()` is a pure stub

**Location**: `neotrix-core/src/agent.rs:235-237`

**What's wrong**: The core multi-agent orchestration function `AgentTeam::execute()` ignores its `_task` parameter entirely and returns a hardcoded `AgentResult` with `agent_name: "stub"` and empty output. Any caller relying on this to actually dispatch work across agents gets a no-op.

**Severity**: P0 — This is the agent coordination backbone. Multi-agent workflows are dead.

**Code sketch**:
```rust
pub fn execute(&self, _task: &str) -> Vec<AgentResult> {
    // TODO: Route task to agents based on role, collect results
    vec![AgentResult { agent_name: "stub".into(), success: true, output: String::new() }]
}
```

**Fix**:
```rust
pub fn execute(&self, task: &str) -> Vec<AgentResult> {
    self.agents
        .iter()
        .map(|role| {
            let output = role.execute(task); // delegate to AgentRole
            AgentResult {
                agent_name: role.name.clone(),
                success: !output.is_empty(),
                output,
            }
        })
        .collect()
}
```

---

## Pain Point 2 — P0: VTuber voice/visual emotion detection always returns Neutral

**Location**: `neotrix-core/src/l4_emotion/nt_feel/nt_feel_vtuber.rs:207-228`

**What's wrong**: `detect_from_voice()` and `detect_from_visual()` ignore their input (`_audio`, `_image`) and always return `EmotionType::Neutral` with `intensity: 0.5`. The NT-FEEL domain's multimodal emotion pipeline is completely non-functional for non-text inputs. Every VTuber interaction through voice or camera gets the same neutral reading.

**Severity**: P0 — NT-FEEL's core promise (multimodal emotion) is hollow.

**Code sketch**:
```rust
pub fn detect_from_voice(&self, _audio: &[u8]) -> Result<_EmotionReading, String> {
    // TODO: 集成语音情绪识别模型
    Ok(_EmotionReading {
        emotion: _EmotionType::Neutral,
        intensity: 0.5,
        source: _EmotionSource::Voice,
        raw_data: None,
        timestamp: chrono::Utc::now(),
    })
}
```

**Fix**:
```rust
pub fn detect_from_voice(&self, audio: &[u8]) -> Result<_EmotionReading, String> {
    if audio.is_empty() {
        return Err("empty audio buffer".into());
    }
    // Delegate to registered voice emotion model (registry pattern)
    let model = self.voice_model.as_ref()
        .ok_or("voice emotion model not registered")?;
    let (emotion, intensity) = model.predict(audio)?;
    Ok(_EmotionReading {
        emotion,
        intensity,
        source: _EmotionSource::Voice,
        raw_data: Some(audio.to_vec()),
        timestamp: chrono::Utc::now(),
    })
}
```

---

## Pain Point 3 — P1: BPCO critic always returns score 0.5 with stub critique

**Location**: `neotrix-core/src/l5_cognition/nt_mind/mind_modules/knowledge/bpco.rs:42-48`

**What's wrong**: `_BpcoCritic::critique()` ignores `_generated` text and returns `score: self.min_score` (0.5) with a hardcoded string `"[stub] best-practice critic not yet wired"`. The `passes()` check always returns true for this stub score. The entire BPCO training optimization pipeline is dead — it never rejects bad outputs, never improves generation quality.

**Severity**: P1 — Training pipeline quality gate is a no-op. Bad outputs pass through unchecked.

**Code sketch**:
```rust
fn critique(&self, _generated: &str) -> _CriticFeedback {
    // C0 stub: 占位中性反馈, 真实 critic 模型接线待 C1-C4 迭代。
    _CriticFeedback {
        critiques: vec!["[stub] best-practice critic not yet wired".into()],
        score: self.min_score,
    }
}
```

**Fix**:
```rust
fn critique(&self, generated: &str) -> _CriticFeedback {
    let mut critiques = Vec::new();
    let mut score = 1.0;

    // Rule-based best-practice checks (upgradeable to LLM critic)
    if generated.len() < 10 {
        critiques.push("output too short".into());
        score -= 0.3;
    }
    if generated.chars().filter(|c| c.is_uppercase()).count() > generated.len() / 4 {
        critiques.push("excessive capitalization".into());
        score -= 0.2;
    }
    if generated.contains("[stub]") || generated.contains("TODO") {
        critiques.push("contains placeholder text".into());
        score -= 0.4;
    }

    _CriticFeedback {
        critiques,
        score: score.clamp(0.0, 1.0),
    }
}
```

---

## Summary

| # | File:Line | Issue | Severity | Fix Complexity |
|---|-----------|-------|----------|----------------|
| 1 | `agent.rs:235` | `AgentTeam::execute()` returns hardcoded stub | P0 | Low — wire to `AgentRole::execute` |
| 2 | `nt_feel_vtuber.rs:207` | Voice/visual emotion always Neutral/0.5 | P0 | Medium — needs model registry |
| 3 | `bpco.rs:42` | BPCO critic always score 0.5, passes everything | P1 | Low — rule-based heuristics |

All 3 are production-critical stubs masquerading as real implementations. The stubs compile and pass tests (which assert the stub behavior), creating a false sense of completeness.
