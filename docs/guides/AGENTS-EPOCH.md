# EarthEpoch Cognitive Framework — Agent Operations Manual

> The system evolves by switching between cognitive frameworks,
> not by optimizing within a single one.

---

## Architecture Overview

```
┌───────────────────────────────────────────────────────────┐
│                    PanoramicBrain                          │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐    │
│  │ E1 Myth  │ │ E2 Agri  │ │ E3 Axial │ │ ...E8    │    │
│  │ ontology │ │ ontology │ │ ontology │ │ emergent │    │
│  │ state[5] │ │ state[5] │ │ state[5] │ │ state[5] │    │
│  │ reward h │ │ reward h │ │ reward h │ │ reward h │    │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘    │
│                                                           │
│  route_task(task) → (primary: Epoch, weights: Vec)       │
│  absorb_reward(task, type, reward) → updates epoch state │
│  transfer_knowledge(from, to, rate) → cross-epoch blend  │
│  evaluate_task(task) → (score, all_scores)                │
│  legacy_capability: CapabilityVector (backward compat)   │
└───────────────────────────────────────────────────────────┘
```

### Files

| File | Role |
|------|------|
| `crates/neotrix-types/src/core/epoch/types.rs` | Data types: `EarthEpoch`, `CognitiveFramework`, `DimensionDef`, `FrameworkRoute`, `ActivationRecord` |
| `crates/neotrix-types/src/core/epoch/definitions.rs` | Ontologies, initial states, router biases, evaluator functions, 10 tests |
| `crates/neotrix-types/src/core/epoch/mod.rs` | `pub mod` + re-exports |
| `neotrix-core/src/core/epoch/mod.rs` | Bridge: re-exports from `neotrix_types::core::epoch` |
| `neotrix-core/src/neotrix/reasoning_brain/panoramic.rs` | `PanoramicBrain` — orchestrator holding all 8 frameworks, routing + absorption + transfer |

### Core Types

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EarthEpoch {
    E1Mythological,
    E2Agricultural,
    E3Axial,
    E4Scientific,
    E5Global,
    E6Planetary,
    E7Network,
    E8Emergent,
}

pub struct CognitiveFramework {
    pub epoch: EarthEpoch,
    pub state: Vec<f64>,                // dimension activation values [0,1]
    pub ontology: Vec<DimensionDef>,     // named dimensions
    pub activation_count: u64,
    pub accumulated_reward: f64,
    pub router_bias: f64,               // static preference weight
}

pub struct DimensionDef {
    pub name: String,
    pub description: String,
}

pub struct FrameworkRoute {
    pub primary: EarthEpoch,
    pub weights: Vec<(EarthEpoch, f64)>,
}
```

---

## How to Add a New Epoch

### 1. Add the enum variant

In `crates/neotrix-types/src/core/epoch/types.rs`, add to `EarthEpoch`:

```rust
#[derive(...)]
pub enum EarthEpoch {
    // ... existing variants ...
    E9NewEpoch,
}
```

Add to `EarthEpoch::all()` and `EarthEpoch::name()`:

```rust
EarthEpoch::E9NewEpoch => "New Epoch (E9)",
```

### 2. Define the ontology

In `crates/neotrix-types/src/core/epoch/definitions.rs`, add to `ontology_for()`:

```rust
EarthEpoch::E9NewEpoch => vec![
    DimensionDef { name: "dimension_a".into(), description: "描述...".into() },
    DimensionDef { name: "dimension_b".into(), description: "描述...".into() },
    // 3–6 dimensions recommended
],
```

### 3. Set initial state

Add to `initial_state_for()`:

```rust
EarthEpoch::E9NewEpoch => vec![0.3, 0.2, 0.1],  // match ontology length
```

### 4. Set router bias

Add to `default_router_bias()`:

```rust
EarthEpoch::E9NewEpoch => 0.40,  // 0.0–1.0
```

### 5. Implement the evaluator

Add to `evaluate_in_epoch()`:

```rust
EarthEpoch::E9NewEpoch => {
    let keyword_a = contains_any(&task_lower, &["word1", "word2", "phrase"]);
    let keyword_b = contains_any(&task_lower, &["word3", "word4"]);
    let base = if keyword_a { 0.6 } else { 0.3 }
        + if keyword_b { 0.5 } else { 0.2 };
    let avg_state = state.iter().sum::<f64>() / state.len() as f64;
    ((base / 1.1) * 0.3 + avg_state * 0.7).clamp(0.0, 1.0)
}
```

### 6. Update tests

In `definitions.rs`'s `#[cfg(test)] mod tests`:

```rust
fn test_e9_new_epoch_dimensions() {
    let frameworks = all_frameworks();
    let e9 = frameworks.iter().find(|fw| fw.epoch == EarthEpoch::E9NewEpoch);
    assert!(e9.is_some());
    assert_eq!(e9.unwrap().dim(), 3);
}
```

### 7. Verify

```
cargo check --lib -p neotrix-types
cargo test --lib -p neotrix-types "epoch"
cargo check --lib   # full project (may have unrelated errors)
```

---

## How to Modify an Existing Epoch's Ontology

### Change a dimension name/description

In `ontology_for()` in `definitions.rs`:

```rust
// Before:
DimensionDef { name: "old_name".into(), description: "old desc".into() }
// After:
DimensionDef { name: "new_name".into(), description: "new desc".into() }
```

Then update any tests that reference the old name:

```rust
// In tests:
assert!(fw.dimension_index("new_name").is_some());  // was "old_name"
```

### Add a new dimension

Append to the epoch's `vec![]` in `ontology_for()`:

```rust
EarthEpoch::E4Scientific => vec![
    // ... existing 6 ...
    DimensionDef { name: "reproducibility".into(), description: "实验结果必须可被独立重复验证".into() },
],
```

Then update `initial_state_for()` to add the corresponding initial value:

```rust
EarthEpoch::E4Scientific => vec![0.3, 0.3, 0.3, 0.2, 0.3, 0.4, 0.1],  // +reproducibility
```

Update the evaluator to handle the new dimension:

```rust
EarthEpoch::E4Scientific => {
    // ... existing keyword checks ...
    let reproduce = contains_any(&task_lower, &["reproduc", "replicat", "repeat"]);
    let base = /* ... */ + if reproduce { 0.3 } else { 0.1 };
    // ...
}
```

Update the test for expected dimension count:

```rust
(EarthEpoch::E4Scientific, 7),  // was 6
```

### Remove a dimension

Remove from `ontology_for()`, `initial_state_for()`, and the evaluator. Update any tests that reference the removed dimension's index or name.

---

## How the Routing System Works

### Routing Formula

`PanoramicBrain::route_task()` computes weight for each epoch:

```
weight = 0.40 * eval_score(epoch, task)
       + 0.30 * history_bonus(task_type, epoch)
       + 0.30 * effective_weight(epoch)
```

Where:
- **eval_score** = `evaluate_in_epoch(epoch, &fw.state, task)` — keyword-match by epoch (30%) + state vector strength (70%)
- **history_bonus** = EMA of rewards for this `(TaskType, EarthEpoch)` pair (exponential moving average: `new = old * 0.9 + reward * 0.1`)
- **effective_weight** = `0.7 * router_bias + 0.3 * average_reward` — combines static bias with dynamic reward history

Weights are sorted descending. The top epoch becomes the `primary` in `FrameworkRoute`.

### Task Flow

```
Input: task_description, task_type (optional)
  │
  ▼
route_task() → FrameworkRoute { primary: Epoch, weights: Vec<(Epoch, f64)> }
  │
  ├── evaluate_task() → (primary_score, all_scores)
  │     (read-only assessment, no side effects)
  │
  └── absorb_reward(task, task_type, reward)
        │
        ├── Record activation on primary epoch
        ├── Update epoch state: state += reward * 0.05 for all dims
        ├── Update epoch_success_by_task[t][e] = EMA
        ├── Push to activation_log
        └── Every 10th activation: sync_to_legacy()
```

### Selecting the Active Epoch

- **Default**: `E7Network` (current dominant paradigm)
- **Manual**: `brain.switch_to(EarthEpoch::E4Scientific)` — sets active_epoch + syncs legacy
- **Automatic**: `route_task()` always returns the best epoch as `primary`
- **Best by task_type**: `brain.best_epoch_for(TaskType::CodeAnalysis)` — uses learned history

---

## Evaluator Function Pattern

Each epoch evaluator follows a consistent structure in `evaluate_in_epoch()`:

```rust
EarthEpoch::EXName => {
    // 1. Define keyword sets that characterize this epoch's cognitive mode
    let keyword_a = contains_any(&task_lower, &["trigger1", "trigger2"]);
    let keyword_b = contains_any(&task_lower, &["trigger3"]);

    // 2. Base score from keyword matching (0.0–1.5 range typical)
    let base = if keyword_a { 0.6 } else { 0.2 }
        + if keyword_b { 0.4 } else { 0.1 };

    // 3. State dimension to use (choose most relevant dim, or average)
    let dim_score = state.first().copied().unwrap_or(0.0);
    // OR: let avg_state = state.iter().sum::<f64>() / state.len() as f64;

    // 4. Combine: keyword relevance (30%) + actual capability (70%)
    ((base / total_keyword_weight) * 0.3 + dim_score * 0.7).clamp(0.0, 1.0)
}
```

### Pattern Rules

1. **Keyword sets should be small** (2–5 words) and high-precision. Each set captures one dimension of the epoch's cognitive mode.
2. **Base keyword weight** is capped proportionally: sum of keyword contributions → divide by sum of max possible.
3. **State/structure ratio** is 70/30 (state-dominant). The evaluator is not just keyword matching — it reflects the system's learned strength in each dimension.
4. **Output is always clamped** to `[0.0, 1.0]` for compatibility with routing weights.
5. **Each epoch is different**: E4 uses all dimensions averaged; E1 uses only the first dimension; E3 uses the third dimension. Choose the dimension(s) that best represent the epoch's core cognitive mode.

### Example: E4 Scientific

```rust
EarthEpoch::E4Scientific => {
    // Keywords capture: analysis, precision, reduction
    let analysis = contains_any(&task_lower, &["analy", "measure", "calculate", "verify", "test", "experiment", "prove"]);
    let precision = contains_any(&task_lower, &["precise", "exact", "accurate", "quantif", "metric"]);
    let reduction = contains_any(&task_lower, &["decompose", "reduce", "break down", "component", "element"]);
    let base = if analysis { 0.6 } else { 0.3 }
        + if precision { 0.5 } else { 0.2 }
        + if reduction { 0.4 } else { 0.2 };
    let avg_state = state.iter().sum::<f64>() / state.len() as f64;
    ((base / 1.5) * 0.3 + avg_state * 0.7).clamp(0.0, 1.0)
}
```

### Example: E1 Mythological

```rust
EarthEpoch::E1Mythological => {
    // Keywords capture: narrative, cyclical time, animism
    let narrative = contains_any(&task_lower, &["story", "myth", "ritual", "symbol", "archetype", "ceremony", "sacred"]);
    let cyclical = contains_any(&task_lower, &["cycle", "season", "return", "rebirth", "eternal"]);
    let animism = contains_any(&task_lower, &["nature", "spirit", "soul", "alive", "consciousness of"]);
    let base = if narrative { 0.6 } else { 0.2 }
        + if cyclical { 0.3 } else { 0.1 }
        + if animism { 0.3 } else { 0.1 };
    let dim_score = state.first().copied().unwrap_or(0.0);  // Only first dimension
    ((base / 1.2) * 0.3 + dim_score * 0.7).clamp(0.0, 1.0)
}
```

---

## Cross-Epoch Knowledge Transfer

`PanoramicBrain::transfer_knowledge(from, to, rate)` blends state vector dimensions by index:

```rust
let min_len = source.len().min(target.len());
for i in 0..min_len {
    let delta = source[i] - target[i];
    target[i] += rate * delta;  // rate typically 0.05–0.20
}
target.normalize();  // clamp max ≤ 1.0
```

This is a **simple dimension-index mapping** (not semantic). When epochs have different dimension counts, only the shared dimensions (by index) are transferred. Future improvements could use semantic mapping through the ontology dimension names.

---

## Compile Gate

```
cargo check --lib -p neotrix-types        # epoch types + definitions
cargo test --lib -p neotrix-types "epoch"  # 10 epoch tests
cargo check --lib                          # full project (may have unrelated errors)
```
