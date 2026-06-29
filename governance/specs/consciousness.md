# Consciousness Subsystem
> Version 1.0 — Status: ✅ active

## Purpose
The consciousness subsystem is the central reasoning and awareness engine. It runs a 4-phase pipeline per cycle: Zero (low-level sensing), One (reflexive awareness), Two (deliberation), Three (metacognition). Operates on VSA 4096-bit vectors as the universal representation.

## Exposed Operations

### Tools
- `feed_consciousness_text(text: String) -> ()` — Ingest user text as VSA-encoded sensory input
- `process_user_request(text: String) -> String` — Synchronous intent detection + pipeline dispatch
- `handle_consciousness_batch() -> PipelineSummary` — Execute one full 4-phase cycle
- `init_image_pipeline(api_key: String) -> Result<(), ImageError>` — Lazily initialize vision pipeline

### Resources
- `consciousness://state` — Current cycle state (phase, load, arousal, coherence)
- `consciousness://pipeline/steps` — Full 130+ step pipeline descriptor
- `consciousness://metrics` — Meta-health KPI (ECE, pass_rate, cognitive_load)

## Configuration Schema

```rust
pub struct ConsciousnessConfig {
    pub phase_cycles: [u32; 4],       // cycles per phase, default [1, 3, 5, 2]
    pub cognitive_load_max: f64,       // 0.0–1.0, default 0.9
    pub arousal_decay: f64,            // per-cycle arousal decay, default 0.95
    pub coherence_threshold: f64,      // resonance trigger, default 0.7
}
```

## Dependencies
- **VSA Core** (nt_core_hcube) — All state is VSA-encoded
- **Global Workspace** (nt_core_gwt) — Attention routing
- **Neural Engine** (nt_core_e8) — E8 attractor dynamics
- **Meta Module** (nt_core_meta) — Metacognitive evaluation

## Error States

| State | Trigger | Recovery |
|-------|---------|----------|
| `PipelineStall` | Phase >15s without completion | Dead cycle detector → warn + reset |
| `CognitiveOverload` | load > 0.95 | Reduce phase frequency, skip non-critical handlers |
| `ArousalCollapse` | arousal < 0.1 | Force reset to baseline arousal |
| `CoherenceBreak` | coherence drops 50% in 3 cycles | Emit `CoherenceWarning`, trigger resonance search |
