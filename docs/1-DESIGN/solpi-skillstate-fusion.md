# SoL-Pi + SKILL.state + Livabble Fusion — Final Absorption

## Date: 2026-09-13
## Total Absorbed Sources: 67+ (this batch: 3)

---

## SoL-Pi (1.6k★, NVIDIA) — Agent Harness Efficiency

**Repo**: https://github.com/NVlabs/SoL-Pi
**Core Insight**: "Before scaling agent loops, can agents first make the harness itself more efficient?"

### 4 Efficiency Mechanisms

| Mechanism | What It Does | NeoTrix Mapping |
|-----------|-------------|-----------------|
| **Action Fusion** | Edit/write runs follow-up validation in same tool call | nt_task_orchestrator: batch execution |
| **ObservationPack** | Repeated large text → stable handles with paged recall | KB entry deduplication + reference |
| **Evidence-Preserving Reducer** | Long logs → compact receipts (every quote matches source) | Experience-tree: distill phase |
| **Online Context Compact** | Completed plan steps → compaction candidates | ConsciousnessTree: phase completion |

### 4 Rules (Universal)

1. **No patches** — use public APIs only
2. **Explicit opt-in** — missing config = disabled
3. **Preserve evidence** — originals remain available
4. **Use runtime choices** — auth/URL/model under host control

### NeoTrix Integration

```rust
/// Action Fusion — batch execution with validation
pub struct ActionFusion {
    /// Primary action (edit/write)
    primary: Action,
    /// Follow-up validation (compile/test)
    validation: Option<Action>,
}

/// ObservationPack — deduplicated observation handles
pub struct ObservationPack {
    /// Original observation (stored locally)
    original: Observation,
    /// Stable handle for reference
    handle: ObservationHandle,
    /// Paged recall for large content
    pages: Vec<ObservationPage>,
}

/// Evidence-Preserving Reducer — compact receipts
pub struct EvidencePreservingReducer {
    /// Original evidence (kept locally)
    evidence: Vec<Evidence>,
    /// Compact receipt (sent to model)
    receipt: CompactReceipt,
    /// Verification: every quote matches source
    verified: bool,
}

/// Online Context Compact — phase completion compaction
pub struct OnlineContextCompact {
    /// Completed phases
    completed: Vec<Phase>,
    /// Compaction candidates
    candidates: Vec<CompactCandidate>,
    /// Window pressure check
    pressure: WindowPressure,
}
```

---

## SKILL.state (EMNLP 2026) — State Transition Architecture

**Paper**: arXiv:2608.26263
**Core Insight**: Replace append-only conversation with explicit mutable execution state

### Key Pattern

```
Input:  At = (P, Σt, Ot)  — spec, state, observation
Output: (Rt, ΔΣt, at)     — reasoning[DISCARD], state_patch, action
State:  Σt+1 = Σt ⊕ ΔΣt   — dict merge with null-deletion
```

### Results
- O(1) prompt growth per step
- O(T) cumulative cost (vs O(T²))
- 16.2x token reduction + improved accuracy
- 90%+ token cost reduction

---

## Combined Pattern: SKILL.state + SoL-Pi

The two papers are COMPLEMENTARY:

| Aspect | SKILL.state | SoL-Pi |
|--------|-------------|--------|
| **Problem** | Append-only history growth | Harness inefficiency |
| **Solution** | Explicit mutable state | 4 efficiency mechanisms |
| **Scope** | Agent runtime | Agent harness |
| **Focus** | State transitions | Token/action optimization |

**Combined Architecture for NeoTrix**:

```
┌─────────────────────────────────────────────────────────┐
│                    SKILL.state Runtime                    │
│  Input: (P, Σt, Ot) → Output: (Rt, ΔΣt, at) → Σt+1    │
├─────────────────────────────────────────────────────────┤
│                    SoL-Pi Efficiency                      │
│  Action Fusion | ObservationPack | Reducer | Compact     │
├─────────────────────────────────────────────────────────┤
│                    NeoTrix Capability Network             │
│  SEAL Pipeline | Consciousness Core | KB Memory          │
└─────────────────────────────────────────────────────────┘
```

---

## Livabble (livabble.com/zh) — Data-Driven City Evaluation

**Platform**: Digital nomad city ranking
**Pattern**: Multi-dimensional city scoring (cost, climate, food, transport, etc.)
**NeoTrix Mapping**: Multi-dimensional capability scoring

```rust
/// Capability scoring (inspired by Livabble city ranking)
pub struct CapabilityScore {
    pub name: String,
    pub dimensions: Vec<ScoreDimension>,
    pub total: f64,
    pub rank: u32,
}

pub struct ScoreDimension {
    pub name: String,
    pub weight: f64,
    pub score: f64,
}
```

---

## Universal Patterns Extracted (All 67 Sources)

### Pattern 1: Bounded State (SKILL.state)
- Replace unbounded history with explicit mutable state
- O(1) per-step cost
- Discard reasoning after state update

### Pattern 2: Harness Efficiency (SoL-Pi)
- Action Fusion: batch execution
- ObservationPack: deduplication
- Evidence Reducer: compact receipts
- Context Compact: phase completion

### Pattern 3: Multi-Dimensional Scoring (Livabble)
- Score across multiple dimensions
- Weighted aggregation
- Rank by total score

### Pattern 4: Auto-Research Loops (SoL-Pi)
- Before scaling, optimize the harness
- Measure efficiency gains
- Keep what works, discard what doesn't

### Pattern 5: Evidence Preservation (SoL-Pi)
- Originals always available
- Compact for model, keep for audit
- Verify every quote matches source

---

## NeoTrix Integration Roadmap

### Phase 1: SKILL.state Integration (Cycles 750-800)
1. Design ExecutionState schema for SEAL pipeline
2. Implement state transition in consciousness core
3. Replace append-only history with bounded state
4. Verify O(1) prompt growth

### Phase 2: SoL-Pi Integration (Cycles 800-850)
1. Implement Action Fusion in nt_task_orchestrator
2. Add ObservationPack for KB entries
3. Build Evidence-Preserving Reducer for experience-tree
4. Add Online Context Compact for phase completion

### Phase 3: Universal Model Interface (Cycles 850-900)
1. Complete adapter pattern (OpenAI/Anthropic/Gemini/Ollama)
2. Add model capability detection
3. Implement fallback chains
4. Add cost estimation

### Phase 4: Production Hardening (Cycles 900-1000+)
1. Error recovery (graceful degradation)
2. Security hardening (egress guard)
3. Performance optimization (hot/cold tiering)
4. Documentation completion
