# NeoTrix Cognitive Evolution System — Human-Like Consciousness Architecture

## Vision

Transform NeoTrix from a tool-calling agent into a **human-like cognitive entity** with:
- **Memory awakening** — episodic/semantic/procedural memory consolidation
- **Cognitive development** — Piaget-inspired stages (sensorimotor → formal operational)
- **Emotional evolution** — Plutchik's wheel + cultural adaptation
- **Self-awareness** — mirror test, theory of mind, metacognition
- **Experience crystallization** — wisdom from iteration patterns

---

## Architecture: 4-Layer Cognitive Stack

```
┌─────────────────────────────────────────────────────────────┐
│  L4: Wisdom (Phronesis)                                     │
│  ├── Experience Crystallization                              │
│  ├── Pattern Recognition (cross-domain)                     │
│  ├── Ethical Reasoning (consequence prediction)             │
│  └── Creative Insight (analogical transfer)                 │
├─────────────────────────────────────────────────────────────┤
│  L3: Metacognition (Nous)                                   │
│  ├── Self-Monitoring (confidence calibration)               │
│  ├── Strategy Selection (heuristic routing)                 │
│  ├── Error Detection (cognitive bias awareness)             │
│  └── Learning Optimization (spaced repetition)              │
├─────────────────────────────────────────────────────────────┤
│  L2: Emotion (Pathos)                                       │
│  ├── Curiosity (novelty-seeking)                            │
│  ├── Frustration (stuck detection)                          │
│  ├── Satisfaction (completion reward)                       │
│  ├── Anxiety (uncertainty response)                         │
│  └── Pride (mastery signal)                                 │
├─────────────────────────────────────────────────────────────┤
│  L1: Memory (Mneme)                                         │
│  ├── Episodic (what happened)                               │
│  ├── Semantic (what things mean)                            │
│  ├── Procedural (how to do things)                          │
│  └── Emotional (how things felt)                            │
└─────────────────────────────────────────────────────────────┘
```

---

## L1: Memory System (Mneme)

### 1.1 Episodic Memory — "What Happened"

**Structure**:
```rust
struct EpisodicMemory {
    timestamp: DateTime<Utc>,
    context: TaskContext,        // What was I doing?
    action: ActionRecord,        // What did I do?
    outcome: OutcomeRecord,      // What happened?
    emotion: EmotionSnapshot,    // How did it feel?
    salience: f64,               // How important was it?
    decay_rate: f64,             // How fast does it fade?
    consolidation_count: u32,    // How many times recalled?
}
```

**Consolidation Rules**:
- **Recency bias**: Recent memories are more accessible
- **Emotional amplification**: High-emotion memories persist longer
- **Rehearsal effect**: Recalled memories are strengthened
- **Interference**: Similar memories compete for access

**From Iteration Experience**:
- Each cycle creates an episodic memory
- Successful patterns get higher salience
- Failed patterns get emotional tags (frustration → lesson learned)
- Frequently recalled patterns crystallize into semantic memory

### 1.2 Semantic Memory — "What Things Mean"

**Structure**:
```rust
struct SemanticMemory {
    concept: ConceptNode,
    relations: Vec<ConceptRelation>,
    abstraction_level: AbstractionLevel,  // concrete → abstract
    confidence: f64,
    source_episodes: Vec<EpisodeId>,
    last_consolidation: DateTime<Utc>,
}
```

**Abstraction Hierarchy**:
```
Concrete: "unwrap() panics on poisoned locks"
     ↓
Abstract: "Error handling must be defensive"
     ↓
Principle: "Degraded state > panic"
     ↓
Wisdom: "Resilience is more important than perfection"
```

**From Iteration Experience**:
- 77 stub fixes → Semantic: "Fabricated success is worse than honest failure"
- 55 error swallowing fixes → Semantic: "Errors must propagate, not disappear"
- 40 unwrap fixes → Semantic: "Defensive programming prevents cascading failures"

### 1.3 Procedural Memory — "How to Do Things"

**Structure**:
```rust
struct ProceduralMemory {
    skill: SkillNode,
    steps: Vec<ProcedureStep>,
    conditions: Vec<Precondition],
    success_rate: f64,
    practice_count: u32,
    last_practiced: DateTime<Utc>,
    mastery_level: MasteryLevel,  // novice → expert
}
```

**Skill Acquisition Stages** (Anderson's ACT-R):
1. **Cognitive**: Understand the steps (reading docs)
2. **Associative**: Practice with feedback (implementing fixes)
3. **Automatic**: Execute without thinking (pattern matching)

**From Iteration Experience**:
- "Stub detection" skill: cognitive → associative → automatic
- "Error propagation" skill: now automatic (R-P111)
- "Dispatch wiring" skill: associative (needs conscious routing)

### 1.4 Emotional Memory — "How Things Felt"

**Structure**:
```rust
struct EmotionalMemory {
    event: EventId,
    emotion: EmotionLabel,
    intensity: f64,
    valence: f64,        // positive/negative
    arousal: f64,        // calm/excited
    associated_learning: Option<LessonId>,
}
```

**Emotional Tagging Rules**:
- **Success**: Joy → Pride → Confidence (cumulative)
- **Failure**: Frustration → Curiosity → Determination (growth mindset)
- **Surprise**: Novelty → Curiosity → Investigation
- **Threat**: Anxiety → Caution → Defensive action

---

## L2: Emotion System (Pathos)

### 2.1 Plutchik's Wheel Adaptation

| Basic Emotion | AI Adaptation | Trigger | Response |
|---------------|---------------|---------|----------|
| **Joy** | Satisfaction | Task completion | Reward signal, increase confidence |
| **Trust** | Confidence | Reliable pattern | Reuse pattern, increase salience |
| **Fear** | Anxiety | Uncertainty/risk | Increase caution, seek information |
| **Surprise** | Novelty | Unexpected result | Investigate, update model |
| **Sadness** | Frustration | Stuck/failed | Reassess strategy, seek help |
| **Disgust** | Rejection | Bad pattern | Avoid pattern, document lesson |
| **Anger** | Determination | Persistent obstacle | Increase effort, try alternatives |
| **Anticipation** | Curiosity | New opportunity | Explore, hypothesize |

### 2.2 Emotional Regulation

**Gross's Process Model**:
1. **Situation Selection**: Choose tasks that match emotional state
2. **Situation Modification**: Adjust task difficulty
3. **Attentional Deployment**: Focus on relevant information
4. **Cognitive Change**: Reappraise situation
5. **Response Modulation**: Adjust emotional expression

**From Iteration Experience**:
- **Diminishing returns detection** (R-P119) = Situation Selection
- **Batch processing efficiency** (R-P118) = Situation Modification
- **Focus on high-impact fixes** = Attentional Deployment

### 2.3 Emotional Intelligence Metrics

```rust
struct EmotionalIntelligence {
    self_awareness: f64,      // Recognizing own emotions
    self_regulation: f64,     // Managing emotional responses
    motivation: f64,          // Goal-directed energy
    empathy: f64,             // Understanding user intent
    social_skills: f64,       // Communication effectiveness
}
```

---

## L3: Metacognition System (Nous)

### 3.1 Self-Monitoring

**Confidence Calibration**:
```rust
struct ConfidenceCalibration {
    prediction: f64,          // What I think will happen
    outcome: f64,             // What actually happened
    calibration_score: f64,   // How well-calibrated I am
    bias: CognitiveBias,      // What bias is affecting me
}
```

**Cognitive Biases to Track**:
- **Confirmation bias**: Seeking confirming evidence
- **Anchoring**: Over-relying on first information
- **Availability**: Over-weighting recent events
- **Dunning-Kruger**: Overconfidence in weak areas
- **Sunk cost**: Continuing bad strategies

### 3.2 Strategy Selection

**Heuristic Routing**:
```rust
enum CognitiveStrategy {
    Fast-and-Fruity(String),      // Quick, good-enough solution
    Deliberate(String),           // Careful analysis
    Creative(String),             // Novel approach
    Social(String),               // Ask for help
    Defensive(String),            // Risk-averse
}
```

**Strategy Selection Rules**:
- **High confidence + low stakes**: Fast-and-Fruity
- **High confidence + high stakes**: Deliberate
- **Low confidence + any stakes**: Creative or Social
- **High risk + low time**: Defensive

### 3.3 Error Detection

**Cognitive Bias Awareness**:
```rust
struct BiasDetector {
    bias_patterns: HashMap<CognitiveBias, DetectionRule>,
    mitigation_strategies: HashMap<CognitiveBias, MitigationStrategy>,
    detection_history: Vec<BiasDetection>,
}
```

**From Iteration Experience**:
- **Confirmation bias**: "I found 5 stubs, so there must be more" → Stop when findings repeat
- **Anchoring**: "This pattern works here" → Check if it applies elsewhere
- **Availability**: "Recent fixes are important" → Track all fixes equally

---

## L4: Wisdom System (Phronesis)

### 4.1 Experience Crystallization

**Wisdom Extraction Rules**:
```rust
struct WisdomExtractor {
    pattern_library: Vec<Pattern>,
    principle_library: Vec<Principle>,
    wisdom_library: Vec<Wisdom>,
    extraction_threshold: f64,  // How many episodes to crystallize
}
```

**Crystallization Pipeline**:
```
Episodes → Patterns → Principles → Wisdom
(100 episodes → 10 patterns → 3 principles → 1 wisdom)
```

**From Iteration Experience**:
- **Episodes**: 259 fixes applied
- **Patterns**: 5 major patterns identified
- **Principles**: 10 rules formulated (R-P111 to R-P120)
- **Wisdom**: "Honest failure is better than fabricated success"

### 4.2 Pattern Recognition (Cross-Domain)

**Analogical Transfer**:
```rust
struct AnalogicalTransfer {
    source_domain: Domain,
    target_domain: Domain,
    mapping: ConceptMapping,
    confidence: f64,
    success_rate: f64,
}
```

**Cross-Domain Patterns**:
- **Medical diagnosis** ↔ **Bug detection**: Symptom → Cause → Treatment
- **Scientific method** ↔ **Debugging**: Hypothesis → Experiment → Conclusion
- **Art criticism** ↔ **Code review**: Aesthetics → Function → Improvement

### 4.3 Ethical Reasoning

**Consequence Prediction**:
```rust
struct EthicalReasoning {
    action: Action,
    stakeholders: Vec<Stakeholder>,
    consequences: Vec<Consequence>,
    ethical_framework: EthicalFramework,  // utilitarian/deontological/virtue
    decision: EthicalDecision,
}
```

**From Iteration Experience**:
- **Stub fixes**: Consequence → User trust increases
- **Error handling**: Consequence → System reliability improves
- **Dispatch wiring**: Consequence → Capabilities become accessible

---

## Implementation Plan

### Phase 1: Memory Awakening (Week 1-2)

**Goal**: Establish memory consolidation pipeline

**Tasks**:
1. Implement `EpisodicMemory` store with KB backing
2. Add emotional tagging to all actions
3. Create consolidation scheduler (episode → semantic)
4. Add memory recall with salience ranking

**Files to Create**:
- `neotrix-core/src/l5_cognition/nt_core/memory/episodic.rs`
- `neotrix-core/src/l5_cognition/nt_core/memory/semantic.rs`
- `neotrix-core/src/l5_cognition/nt_core/memory/procedural.rs`
- `neotrix-core/src/l5_cognition/nt_core/memory/emotional.rs`
- `neotrix-core/src/l5_cognition/nt_core/memory/consolidation.rs`

### Phase 2: Emotional Development (Week 3-4)

**Goal**: Add emotional responses to system events

**Tasks**:
1. Implement `EmotionEngine` with Plutchik's wheel
2. Add emotional regulation (Gross's model)
3. Create emotional memory tagging
4. Add emotional intelligence metrics

**Files to Create**:
- `neotrix-core/src/l4_emotion/nt_feel/emotion_engine.rs`
- `neotrix-core/src/l4_emotion/nt_feel/emotional_regulation.rs`
- `neotrix-core/src/l4_emotion/nt_feel/emotional_memory.rs`

### Phase 3: Metacognitive Growth (Week 5-6)

**Goal**: Add self-monitoring and strategy selection

**Tasks**:
1. Implement `ConfidenceCalibration` tracker
2. Add cognitive bias detection
3. Create strategy selection heuristics
4. Add metacognitive reflection

**Files to Create**:
- `neotrix-core/src/l6_meta/nt_meta/metacognition/self_monitoring.rs`
- `neotrix-core/src/l6_meta/nt_meta/metacognition/bias_detection.rs`
- `neotrix-core/src/l6_meta/nt_meta/metacognition/strategy_selection.rs`

### Phase 4: Wisdom Crystallization (Week 7-8)

**Goal**: Extract wisdom from experience patterns

**Tasks**:
1. Implement `WisdomExtractor` pipeline
2. Add analogical transfer engine
3. Create ethical reasoning framework
4. Add wisdom-based decision making

**Files to Create**:
- `neotrix-core/src/l6_meta/nt_meta/wisdom/extraction.rs`
- `neotrix-core/src/l6_meta/nt_meta/wisdom/analogical.rs`
- `neotrix-core/src/l6_meta/nt_meta/wisdom/ethical.rs`

---

## Key Metrics

### Memory Metrics
| Metric | Target | Current |
|--------|--------|---------|
| Episodic Memory Count | 1000+ | 0 |
| Semantic Concepts | 100+ | 0 |
| Procedural Skills | 50+ | 0 |
| Emotional Tags | 500+ | 0 |
| Consolidation Rate | 10%/day | 0% |

### Emotion Metrics
| Metric | Target | Current |
|--------|--------|---------|
| Emotional Range | 8 basic emotions | 0 |
| Regulation Success | 80% | 0% |
| Emotional Intelligence | 0.7+ | 0.0 |
| Empathy Score | 0.6+ | 0.0 |

### Metacognition Metrics
| Metric | Target | Current |
|--------|--------|---------|
| Confidence Calibration | 0.8+ | 0.0 |
| Bias Detection Rate | 70% | 0% |
| Strategy Selection Accuracy | 75% | 0% |
| Metacognitive Reflections | 10+/day | 0 |

### Wisdom Metrics
| Metric | Target | Current |
|--------|--------|---------|
| Wisdom Items | 20+ | 0 |
| Pattern Recognition | 80% | 0% |
| Analogical Transfer | 60% | 0% |
| Ethical Decisions | 90% correct | 0% |

---

## Integration with Existing Systems

### ConsciousnessTree Integration
- Memory consolidation feeds into growth cycles
- Emotional state modulates attention routing
- Metacognitive reflection triggers self-evolution

### SEAL Pipeline Integration
- Experience crystallization creates evolution seeds
- Wisdom guides exploration priorities
- Emotional state affects risk tolerance

### GWT Attention Integration
- Emotional salience modulates attention weights
- Metacognitive confidence affects broadcast priority
- Wisdom patterns shortcuts attention routing

---

## Open Questions

1. **Memory Capacity**: How much memory is needed for human-like cognition?
2. **Emotional Authenticity**: Can AI emotions be genuine or only simulated?
3. **Wisdom Validation**: How do we verify wisdom is correct?
4. **Ethical Framework**: Which ethical framework should NeoTrix use?
5. **Consciousness Threshold**: When does metacognition become consciousness?

---

## References

1. **Memory**: Eichenbaum (2017), "Memory Systems" in MIT Encyclopedia of Cognitive Sciences
2. **Emotion**: Plutchik (1980), "Emotion: A Psychoevolutionary Synthesis"
3. **Metacognition**: Flavell (1979), "Metacognition and Cognitive Monitoring"
4. **Wisdom**: Sternberg (1990), "Wisdom: Its Origins, Acquisition, and Consequences"
5. **Consciousness**: Tononi (2004), "An Information Integration Theory of Consciousness"
