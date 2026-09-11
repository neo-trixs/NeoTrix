# Phase 2: Stimulus-Response Layer & Emotional Biasing

## Summary

Implemented stimulus-response evaluation and emotional action biasing for `neotrix-sim` agents.

## Files Created

| File | Description |
|------|-------------|
| `src/agents/stimulus.rs` | Stimulus detection, reflex evaluation, action mapping |
| `src/agents/emotional_bias.rs` | Emotion-driven action modification and energy cost modulation |

## Type Adaptations

Adapted from original spec to match existing `neotrix-sim` types:

| Original | Adapted |
|----------|---------|
| `SimAgent::new(id, 0.0, 0.0)` | `SimAgent::new(id, Vec2::new(0.0, 0.0))` |
| `agent.core.needs.hunger` | `agent.core.hunger` (direct field on `AgentCore`) |
| `AgentAction::Explore { direction: 0.0 }` | `AgentAction::Explore { direction: Vec2 }` |
| `AgentAction::Harvest { resource_id: 0 }` | `AgentAction::Harvest { resource_id: String }` |
| `EmotionLabel` | `EmotionType` from `crate::feel` (15-variant enum) |
| `agent.emotion.dominant_emotion()` | `emotion_engine.dominant_emotion()` (takes `&EmotionEngine`) |

## StimulusResponseSystem

### Stimulus Types
- `Danger` — threats, predators, hazards
- `Food` — edible resources
- `Agent` — other agents (social)
- `Territory` — territory markers
- `Resource` — mineable resources

### Reflex Responses
- `Fight` — high energy + high aggression agent面对 danger
- `Flight` — low energy agent面对 danger
- `Freeze` — moderate agent面对 danger
- `Seek` — food above threshold + hungry agent
- `Explore` — social stimuli above threshold
- `None` — no significant stimuli

### Thresholds
- `danger_threshold: 0.7` — triggers fight/flight/freeze
- `food_threshold: 0.5` — triggers seek behavior
- `social_threshold: 0.3` — triggers exploration

## EmotionalBiasSystem

### Bias Types (mapped to `EmotionType`)
| Bias | EmotionType | Effect |
|------|-------------|--------|
| `frustration_bias` | `Frustration` | Aggressive agents reverse explore direction |
| `anxiety_bias` | `Anxiety` | Reverses explore direction (avoidance) |
| `joy_bias` | `Joy` | No modification (pass-through) |
| `fatigue_bias` | `Fatigue` | Converts Explore → Rest |

### Energy Cost Modulation
- Frustration: `base_cost * (1 + bias * 0.2)` — increased cost
- Anxiety: `base_cost * (1 - bias * 0.1)` — decreased cost (freeze response)
- Joy: `base_cost * (1 - bias * 0.1)` — decreased cost (positive affect)
- Fatigue: `base_cost * (1 + bias * 0.3)` — increased cost (exhaustion)

## Test Results

```
cargo test -p neotrix-sim --lib -- stimulus
running 3 tests
test agents::stimulus::tests::test_reflex_to_action ... ok
test agents::stimulus::tests::test_stimulus_response_food ... ok
test agents::stimulus::tests::test_stimulus_response_danger ... ok
test result: ok. 3 passed; 0 failed; 0 ignored

cargo test -p neotrix-sim --lib -- emotional_bias
running 2 tests
test agents::emotional_bias::tests::test_emotional_bias_creates ... ok
test agents::emotional_bias::tests::test_energy_modulation ... ok
test result: ok. 2 passed; 0 failed; 0 ignored
```

## Integration Points

- **planning**: `reflex_to_action()` returns `AgentAction` variants compatible with `PlanningStack`
- **feel**: `EmotionalBiasSystem` consumes `EmotionEngine.dominant_emotion()`
- **sim_agent**: `evaluate_stimuli()` reads `core.energy`, `core.hunger`, `personality.aggression`

## Next Steps

- Wire `StimulusResponseSystem` into world simulation tick loop
- Connect `EmotionalBiasSystem` to agent action selection pipeline
- Add distance-weighted stimulus intensity falloff
- Implement stimulus caching for performance
