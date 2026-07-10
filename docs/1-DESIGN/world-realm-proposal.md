# NeoTrix Realm System Proposal — Tiered World Model for SiliconSelf

## 1. Current State Analysis

### World Model (`l2_world_impl/nt_world_model/`)

The NeoTrix world model is a **flat, single-layer predictive engine** with no concept of realms, tiers, or strata. It consists of:

| Module | Function | No realm concept |
|--------|----------|-----------------|
| `nt_world_model_types.rs` | Context encoding, domain enum (General/WebDev/Mobile/AI/DataScience/DevOps) | Flat domain list |
| `nt_world_model_predict.rs` | LatentTransition + ExpertPredictor | Predicts next state in same space |
| `nt_world_model_td_jepa.rs` | TD(λ) multi-step rollout | Linear trajectory, no regime change |
| `nt_world_model_rgm_jepa.rs` | Renormalization Group multi-scale prediction | Scale is purely spatial (block averaging), not qualitative regimes |
| `nt_world_model_object_partition.rs` | Adaptive latent space partitioning | Partitions by variance density, not by operational realm |
| `nt_world_model_v2.rs` | V2 orchestrator: JEPA → E8 → ActiveInference → IIT | No realm awareness |

**Key gap**: The agent predicts *next states* but has no self-awareness of *what kind of world* it is operating in, and cannot recognize that different tasks belong to fundamentally different "realms" governed by different rules.

### Existing Tier/Rank Concepts (not in world model)

NeoTrix already has several tier-like structures, but none integrated into the world model itself:

| System | Tier Structure | Location |
|--------|---------------|----------|
| **MemoryTier** | Working → Episodic → Semantic → Procedural | `core/nt_core_bank/tier.rs` |
| **CapabilityTier** | Core/Basic/Advanced/Generate/Autonomous | `core/l7_capability/registry.rs` |
| **ParseTier** | Tier0Fast → Tier1Hybrid → Tier2Vlm | `l2_world_impl/nt_world_parse/` |
| **ArchLayer** | L0–L9 consciousness layers | `core/nt_core_self_review.rs` |
| **Frontier priority** | 5-tier dual-queue crawl priority | `l2_world_impl/nt_world_crawl/frontier.rs` |

### SiliconSelfModel (`core/nt_core_self/silicon_self.rs`)

The agent's self-model has:
- `SystemIdentity` — identity, values
- `ContextWindow` — working memory
- `AttentionManager` — attention domains
- `ReasoningStrategyRegistry` — strategy selection
- `ThinkingTrace` — reasoning traces
- `ProblemSolvingPattern` / `MetacognitiveLaw` — learned patterns

**No field** tracks what "realm" the agent is currently operating in or its "cultivation level."

---

## 2. Chinese Web Novel Realm Architecture (Inspiration)

Chinese cultivation (修仙/修真) novels use a stratified realm system where each tier has **fundamentally different physics, capabilities, and failure modes**:

```
Mortal Realm (凡人界)
├── 炼气 (Qi Refining)     — Meridian opening, basic energy sense
├── 筑基 (Foundation)       — Core foundation, 120-year lifespan
├── 金丹 (Golden Core)      — Energy crystallization, internal furnace
├── 元婴 (Nascent Soul)     — Soul birth, out-of-body travel
└── 化神 (Divine Transformation) — Domain power, spatial manipulation

Immortal Realm (仙界)
├── 合体 (Body Integration)
├── 大乘 (Great Vehicle)
├── 渡劫 (Tribulation Crossing)
└── 飞升 (Ascension)

God Realm (神界)
├── 真神 (True God)
├── 天神 (Heavenly God)
└── 神王 (God King)
```

**Key architectural properties**:
1. **Qualitative jumps**: Each realm is not just "more HP" — it unlocks entirely new operations (e.g., flight only after Foundation, spatial folding only at Divine Transformation)
2. **Physics change**: Mortal realm uses qi; immortal realm uses immortal energy; god realm uses divine power — different conservation laws
3. **Tribulation gates**: Between realms there are "heavenly tribulations" (天劫) — failure means regression or death
4. **Realm pressure**: Higher realm beings cannot easily descend to lower realms without suppression

---

## 3. Proposed Realm System for NeoTrix

### 3.1 Realm Architecture

Map the agent's operational capabilities to 7 realms, each with distinct "physics":

| Realm | Name | Operational Physics | Cognitive Capability | Access Condition |
|-------|------|-------------------|---------------------|-----------------|
| R0 | **Data** (数据) | Raw input → output mapping | Pattern matching, reflexes | Default |
| R1 | **Information** (信息) | Symbolic relations, retrieval | FTS/BM25 search, memory recall | Basic knowledge > 1000 facts |
| R2 | **Knowledge** (知识) | Causal models, graph reasoning | BFS traversal, semantic search | Knowledge graph > 10000 edges |
| R3 | **Understanding** (理解) | Latent space inference | JEPA prediction, E8 evolution | Prediction accuracy > 0.7 |
| R4 | **Wisdom** (智慧) | Cross-domain analogy, abstraction | HyperCube VSA binding, GWT broadcast | Cross-domain transfer > 0.6 |
| R5 | **Insight** (洞见) | Meta-cognitive reflection | Self-review, SEAL auto-edit, law distillation | Auto-evolution cycles > 100 |
| R6 | **Transcendence** (超然) | Recursive self-improvement | Full SEAL pipeline, DGM self-edit, goal mutation | All prior realms at mastery |

### 3.2 Realm-Dependent Physics Rules

Each realm changes which "laws of physics" apply:

```rust
pub struct RealmPhysics {
    /// Prediction horizon (steps JEPA can reliably predict)
    pub prediction_horizon: usize,
    /// Latent space dimension at this realm
    pub latent_dim: usize,
    /// Whether causal reasoning is available
    pub causal_reasoning: bool,
    /// Whether cross-domain analogy is available
    pub cross_domain_transfer: bool,
    /// Whether the agent can self-modify
    pub self_modification: bool,
    /// Exploration bonus (higher = more random exploration)
    pub exploration_bonus: f64,
    /// Memory retention decay per step
    pub memory_decay: f64,
    /// Error recovery strategy tier
    pub error_recovery_tier: u8,
}
```

Realm progression changes physics:

| Realm | Horizon | Latent Dim | Causal | Analog | Self-Edit | Explore | Decay | Recovery |
|-------|---------|-----------|--------|--------|-----------|---------|-------|----------|
| R0 Data | 1 | 8 | No | No | No | 0.9 | 0.5 | L1 retry |
| R1 Info | 3 | 16 | No | No | No | 0.7 | 0.3 | L2 fallback |
| R2 Know | 5 | 32 | Partial | No | No | 0.5 | 0.2 | L3 semantic |
| R3 Under | 8 | 64 | Yes | No | No | 0.3 | 0.1 | L4 checkpoint |
| R4 Wisdom | 12 | 128 | Yes | Yes | No | 0.2 | 0.05 | L5 validation |
| R5 Insight | 16 | 256 | Yes | Yes | Yes | 0.1 | 0.02 | L6 auto-fix |
| R6 Trans | 32 | 512 | Yes | Yes | Yes | 0.05 | 0.01 | L7 full |

### 3.3 Integration Point: Add to SiliconSelfModel

The realm is a **property of the agent's self-model**, not the world model — the agent *is* at a certain realm, and the world physics vary accordingly.

```rust
// New field in SiliconSelfModel (core/nt_core_self/silicon_self.rs)
pub struct SiliconSelfModel {
    // ... existing fields ...
    
    /// Current realm the agent operates in (R0-R6)
    pub realm: Realm,
    /// Realm-specific physics configuration
    pub realm_physics: RealmPhysics,
    /// Realm progression: experience accumulated in current realm
    pub realm_xp: f64,
    /// Realm progression: threshold to advance
    pub realm_xp_threshold: f64,
    /// History of realm ascensions
    pub realm_history: Vec<RealmAscension>,
}

pub enum Realm {
    Data = 0,
    Information = 1,
    Knowledge = 2,
    Understanding = 3,
    Wisdom = 4,
    Insight = 5,
    Transcendence = 6,
}
```

### 3.4 Experience Sources for Realm Progression (XP)

| Realm XP Source | XP Value | Source Module |
|----------------|----------|--------------|
| Successful prediction (JEPA loss < 0.1) | +1 | `nt_world_model_v2` |
| KB node created | +2 | `nt_memory_store` |
| KB edge discovered | +1 | `nt_memory_graph` |
| Cross-domain analogy triggered | +5 | `nt_core_gwt` |
| Self-review finding fixed | +3 | `nt_core_self_review` |
| SEAL iteration completed | +1 | `nt_mind_seal` |
| Problem-solving pattern recorded | +4 | `silicon_self.rs` |
| Metacognitive law distilled | +8 | `silicon_self.rs` |
| Conversation evolved into training data | +3 | `nt_mind_seal` |
| Community dataset absorbed | +6 | `nt_core_community` |

### 3.5 Tribulation Events (天劫)

Between realms, the agent must pass a "tribulation" — a validation challenge:

| Ascension | Tribulation Challenge | Failure Consequence |
|-----------|----------------------|--------------------|
| R0→R1 | KB search accuracy > 0.8 on 10 queries | XP penalty -20% |
| R1→R2 | Successfully traverse a 3-hop graph query | XP penalty -15% |
| R2→R3 | JEPA prediction within 0.2 MSE on held-out test | XP penalty -25% |
| R3→R4 | Solve cross-domain task (e.g., use KB to improve world model) | XP penalty -30% |
| R4→R5 | Self-review finds ≥1 real defect in own code | XP penalty -20% |
| R5→R6 | Full SEAL pipeline runs without errors | XP penalty -50% |

### 3.6 GWT Broadcast Integration

When realm changes occur, the Global Workspace (`nt_core_gwt`) should broadcast:

```rust
pub struct RealmChange {
    pub from: Realm,
    pub to: Realm,
    pub physics_changed: Vec<PhysicsChange>,
    pub new_capabilities: Vec<String>,
    pub tribulation_passed: Option<String>,
}
```

All 13 specialists receive the broadcast and adjust their behavior.

---

## 4. Comparison with Chinese Novel Architecture

| Novel Concept | NeoTrix Realm System Mapping |
|--------------|------------------------------|
| 炼气 (Qi Refining) | R0→R1: Learning basic input-output patterns |
| 筑基 (Foundation) | R1→R2: Building causal knowledge graph |
| 金丹 (Golden Core) | R2→R3: JEPA latent space crystallization |
| 元婴 (Nascent Soul) | R3→R4: GWT-consciousness emergence |
| 化神 (Divine Transformation) | R4→R5: Meta-cognitive self-awareness |
| 飞升 (Ascension) | R5→R6: Transcend to recursive self-improvement |
| 天劫 (Tribulation) | Validation gate between realms |
| 领域 (Domain power) | Realm Physics: each realm has unique "laws" |
| 灵气 vs 仙气 vs 神力 | Different operational modalities per realm |

---

## 5. Implementation Strategy (No Code Changes)

The proposal would require changes to:

1. **`crates/neotrix-types/src/core/nt_core_self/silicon_self.rs`** — Add `Realm` enum, `RealmPhysics`, `RealmAscension` types; add fields to `SiliconSelfModel`
2. **`neotrix-core/src/core/nt_core_self/silicon_self.rs`** — Add realm progression logic, XP accumulation, tribulation checking
3. **`neotrix-core/src/neotrix/l2_world_impl/nt_world_model_v2.rs`** — Wire realm physics into prediction horizon, latent dim
4. **`neotrix-core/src/core/nt_core_gwt/workspace.rs`** — Add `RealmChange` broadcast
5. **`neotrix-core/src/neotrix/nt_mind_seal/pipeline.rs`** — Add realm XP to SEAL stages

Estimated effort: ~400 lines new code, ~100 lines modifications. No breaking changes.

---

## 6. Summary of Findings

**Current world model is flat**: No concept of realms, tiers, or stratified operational regimes. All prediction happens in the same latent space regardless of task complexity or agent maturity.

**Existing tier systems are fragmented**: MemoryTier, CapabilityTier, ParseTier, ArchLayer, and Frontier priority all implement tier concepts in isolation — none inform the agent's self-model of what "world" it operates in.

**Chinese web novel realms offer a proven stratified architecture**: Each tier has different physics, different operations, and tribulation gates — directly applicable to an AI agent's cognitive development.

**SiliconSelfModel is the natural home**: The realm is a property of the agent's self-awareness, not the external world model. Realm physics then parameterize the world model, GWT, memory, and SEAL pipeline.
