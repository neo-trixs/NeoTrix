# Targeted Research #469 — Internal Pain Points (3)

**Date:** 2026-09-12 | **Source:** neotrix-core/src/ codebase scan | **Method:** TODO/FIXME + hardcoded returns + dead code grep

---

## Pain Point 1: VTuber Emotion Engine — All Sensing Stubs Return Neutral

**File:** `l4_emotion/nt_feel/nt_feel_vtuber.rs:207-308`
**Severity:** P1 (critical for VTuber product)

**What's wrong:** 5 public methods (`detect_from_voice`, `detect_from_visual`, `synthesize_speech`, `transcribe_speech`, `generate_response` voice output) all return hardcoded neutral/empty results. Any VTuber integration will silently produce emotionless output.

**Concrete fix:**
```rust
pub fn detect_from_voice(&self, audio: &[u8]) -> Result<_EmotionReading, String> {
    if audio.is_empty() {
        return Err("empty audio".into());
    }
    // Route to the registered voice emotion model via self.model_registry
    let model = self.model_registry.get("voice_emotion")
        .ok_or("voice_emotion model not registered")?;
    let embedding = model.encode(audio)?;
    let emotion = _EmotionType::from_embedding(&embedding);
    let intensity = embedding.confidence();
    Ok(_EmotionReading {
        emotion, intensity,
        source: _EmotionSource::Voice,
        raw_data: Some(audio.to_vec()),
        timestamp: chrono::Utc::now(),
    })
}
```

---

## Pain Point 2: Trade Orchestrator — Production/Logistics Engine Integration Dropped

**File:** `l1_action/nt_act/nt_act_trade/orchestrator.rs:695-889`
**Severity:** P0 (production trade workflow broken)

**What's wrong:** 7 TODOs where `ProductionEngine`, `LogisticsEngine`, `NegotiationEngine` calls were commented out during a refactor. Key consequences:
- `handle_objection()` (line 695): objection never resolved → stuck in negotiation
- `track_production()` (line 833): progress never triggers phase transition
- `_final_quality_check()` (line 870): always returns `true` regardless of actual check
- `apply_inspection_cert()` (line 889): cert generated without validation

**Concrete fix:**
```rust
pub fn handle_objection(
    &mut self, order_id: &str,
    objection: &ObjectionCategory, _concession: &Concession,
) -> Result<(), String> {
    let ctx = self.active_trades.get_mut(order_id)
        .ok_or_else(|| format!("Trade {} not found", order_id))?;
    let resolved = self.negotiation_engine.handle_objection(
        objection.clone(), _concession.clone()
    );
    ctx.conversations.push(format!("FT08_Objection:{:?}", objection));
    if resolved {
        ctx.current_phase = TradePhase26::Ft09ContractReviewSigning;
    }
    Ok(())
}
```

---

## Pain Point 3: Checkpoint Persistence — file_size Always Zero

**File:** `l1_action/nt_act/actions/checkpoint_persistence.rs:147`
**Severity:** P2 (data integrity gap)

**What's wrong:** `file_size` is hardcoded to `0` in `CheckpointMeta`. This means:
- Disk usage tracking is inaccurate
- Expiration pruning cannot prioritize large checkpoints
- Monitoring dashboards show zero for all checkpoint sizes

**Concrete fix:**
```rust
let json_str = serde_json::to_string_pretty(&data)
    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
let file_size = json_str.len() as u64;

let meta = CheckpointMeta {
    id: checkpoint_id.clone(),
    workflow_id: workflow_id.to_string(),
    stage_name: stage_name.to_string(),
    status: CheckpointStatus::Saved,
    created_at: current_timestamp(),
    expires_at: Some(current_timestamp() + self.config.expiration_secs),
    file_size, // now reflects actual serialized size
    description: None,
};
```

---

## Summary

| # | Pain Point | File:Line | Severity | Impact |
|---|-----------|-----------|----------|--------|
| 1 | VTuber emotion stubs | `nt_feel_vtuber.rs:207-308` | P1 | VTuber product non-functional |
| 2 | Trade engine integration dropped | `orchestrator.rs:695-889` | P0 | Production trade workflow broken |
| 3 | Checkpoint file_size=0 | `checkpoint_persistence.rs:147` | P2 | Disk monitoring blind spot |

**Scan stats:** 100+ TODO/FIXME markers found; 25 todo!/unimplemented! calls; 100+ hardcoded returns. Top 3 selected by blast radius.
