# Iteration Batch 559 — Predictive Processing / Neural Coding / Information Theory of Consciousness

**Date**: 2026-09-06 | **Iteration**: 559/10000 | **Focus**: PP/AI + Neural Coding + IIT vs NeoTrix consciousness architecture

## What's NEW vs Batch 558

Batch 558 identified 7 defects: transferability prediction, domain signal contamination, meta-initialization gap, redundant KB storage, SEAL pipeline overkill, missing forgetting-curve, static memory policies. Batch 559 finds **10 NEW defects** from predictive processing, neural coding, and information theory of consciousness.

---

### Finding 1: Three-Timescale Learning Hierarchy in Active Inference (Minds & Machines, Jun 2026)

**Source**: Springer 10.1007/s11023-026-09787-8 — "Reviewing Structure Learning in and Out of the Active Inference Framework"

**New Defect vs Batch 558**: Batch 558's SEAL pipeline is a single-timescale evolution loop. Active Inference research identifies THREE distinct learning timescales: (1) **Active Inference** — fast hidden-state inference (milliseconds), (2) **Parametric Learning** — Hebbian-style parameter updates (seconds-minutes), (3) **Structure Learning** — Bayesian Model Selection for reorganizing the generative model itself (offline, slowest). NeoTrix collapses all three into one SEAL cycle. Batch 558's "context_space_adaptation" shortcut (finding 5) only addresses timescale (1). Timescale (3) — restructuring the generative model topology — has no NeoTrix analog. The E8 hexagram reasoning engine and HyperCube knowledge representation are fixed topologies; no mechanism reorganizes the knowledge structure itself based on accumulated evidence.

**Proposed Fix**: Add a `StructureLearner` to NT-CORE that operates on the slowest timescale:
1. Tracks Bayesian Model Evidence across reasoning sessions (which E8 hexagram states correlate with successful outcomes)
2. Triggers topology reorganization when accumulated evidence favors a different relational structure
3. Separates from SEAL's parametric learning — doesn't update weights, reorganizes the HyperCube edge structure itself
4. Implements synaptic pruning analog: delete edges with consistently low predictive power

**Why batch 558 missed this**: Batch 558's meta-initialization (finding 3) stores "how to learn" parameters but assumes the generative model structure (HyperCube topology) is fixed. Structure Learning changes the model itself.

---

### Finding 2: VFE Includes Inference Cost, Not Just Model Fidelity (arXiv 2603.20927)

**Source**: arXiv 2603.20927 — "Physical AI agents under Active Inference" (Mar 2026)

**New Defect vs Batch 558**: Batch 558 proposed SDFT (finding 4) to reduce KB storage by 80%. But this optimization only considers storage cost. The 2026 PP review shows Variational Free Energy = surprisal (model fidelity) + **inference cost** (computational overhead of using the model). A model with high Bayesian evidence but expensive inference can have WORSE VFE than a simpler model with lower evidence. NeoTrix's experience-tree absorption writes to KB and retrieves via BM25 search — but the retrieval cost (compute time for search + context window injection) is never factored into experience value. An experience stored in KB that takes 2 seconds to retrieve but saves 5 seconds of reasoning is net-positive; one that takes 10 seconds to retrieve but saves 5 seconds is net-negative.

**Proposed Fix**: Each experience in the KB carries an `inference_cost: Duration` field (measured at retrieval time) and a `value_saved: Duration` field (measured by downstream task improvement). The effective VFE contribution is: `VFE_experience = surprisal + inference_cost - value_saved`. Experiences with positive VFE (cost > value) get pruned or consolidated.

**Why batch 558 missed this**: SDFT (batch 558 finding 4) addresses storage redundancy but not retrieval-cost accounting. The two are orthogonal.

---

### Finding 3: Renormalising Generative Models — Coarse-Graining Across Scales (arXiv 2608.09512)

**Source**: arXiv 2608.09512 — "Renormalising Generative Models for Active Inference" (Aug 2026)

**New Defect vs Batch 558**: RGMs solve the scaling problem by composing discrete generative models across spatial and temporal scales via coarse-graining: lower-level episodes become higher-level causes. NeoTrix's 6-layer architecture (L1-L6) has fixed layer boundaries. There is no mechanism to dynamically coarse-grain lower-layer observations into higher-layer causes. When NT-WORLD crawls 100 web pages (L1 action), those should be coarse-grained into a single "topic model" (L2 perception) and then into a "domain understanding" (L5 cognition). Currently, each layer processes independently with EventBus signals — no recursive renormalization.

**Proposed Fix**: Add a `RenormalizationOperator` that:
1. Groups lower-layer state vectors into coarse-grained higher-layer causes when local pattern statistics stabilize
2. Passes coarse-grained summaries upward as empirical priors for higher-layer inference
3. Allows higher-layer context to constrain lower-layer inference (bidirectional information flow)
4. Reduces computational load: instead of processing 100 page-crawls at L1, process 10 coarse topic-models at L2

**Why batch 558 missed this**: Batch 558's stigmergy coordination (batch 557) operates at one scale. RGMs explicitly address cross-scale information flow — a fundamentally different problem.

---

### Finding 4: Bracket Coding — Dynamic Rate-Temporal Switching (bioRxiv, May 2026)

**Source**: bioRxiv 10.64898/2026.05.31.729124v3 — "Bracket Coding: An Emergent Balance Between Temporal Integration and Segregation"

**New Defect vs Batch 558**: NeoTrix's GWT attention routing is a single-mode broadcast: salient information is broadcast to all specialist modules simultaneously. Bracket coding shows the brain uses a HYBRID scheme: population activity is partitioned into temporal "brackets" — within each bracket, information is rate-coded (stable, integrative), but BETWEEN brackets, boundaries are precisely-timed temporal events (synchronous across population). This means the brain dynamically switches between integrative (rate) and segregative (temporal) coding within the SAME processing epoch. NeoTrix GWT has no mechanism for this — it either broadcasts (integrative) or doesn't (segregative). No dynamic switching within a single reasoning cycle.

**Proposed Fix**: Add a `BracketRouter` to GWT that:
1. Detects when population-level activity (cross-module signal coherence) crosses a threshold — triggers a "bracket boundary"
2. Within brackets: standard rate-coded GWT broadcast (integrative)
3. At bracket boundaries: synchronous temporal reset across all active modules — re-segregates and re-partitions attention
4. The frequency of bracket boundaries is itself adaptive — faster switching for volatile environments, slower for stable ones

**Why batch 558 missed this**: Batch 558's domain-invariance weight (finding 2) addresses signal filtering but not the dynamic switching between integrative and segregative modes within a single processing cycle.

---

### Finding 5: Burst-Based Temporal Encoding Outperforms Rate Coding (arXiv 2607.13644)

**Source**: arXiv 2607.13644 — "Evaluating Encoding Strategies for Closed-Loop Classification in Biological Neural Networks" (Jul 2026)

**New Defect vs Batch 558**: In closed-loop classification tasks using biological neural networks, burst-based temporal encoding achieves 95.6% accuracy vs substantially lower for rate-based and phase-based approaches. The key insight: BURSTS (clusters of rapid spikes followed by silence) carry more discriminable information per spike than uniform rate coding. NeoTrix's inter-module communication (EventBus, stigmergic channels) uses uniform signal transmission — each event carries equal weight regardless of timing. There is no burst vs tonic distinction. Critical signals (SEAL pipeline failure, safety alert) transmit identically to routine signals (KB write confirmation).

**Proposed Fix**: Add a `BurstEncoder` to the EventBus:
1. Critical signals: burst-mode (3 rapid events within 50ms, then silence) — downstream modules treat burst as high-priority
2. Routine signals: tonic-mode (single event) — processed at normal priority
3. Burst detection at receiver: count events within temporal window, classify as burst/tonic, route accordingly
4. Reduces false-positive attention allocation — routine events don't hijack GWT salience

**Why batch 558 missed this**: Batch 558's context-space adaptation (finding 5) addresses when to use full SEAL vs shortcut. Burst encoding addresses HOW signals are encoded within the communication channel itself.

---

### Finding 6: Temporal Coding Enables Hyperacuity Beyond Physical Resolution (Nature Comms, Aug 2026)

**Source**: Nature Comms s41467-026-76878-6 — "Temporal coding enables hyperacuity in event-based vision" (Aug 2026)

**New Defect vs Batch 558**: Event-based vision systems using precise temporal information achieve sub-pixel discrimination (hyperacuity) — extracting spatial detail that is UNRECOVERABLE from static frames. The key: precise spike timing encodes spatial information that rate coding cannot represent. NeoTrix's NT-WORLD perception layer processes web content as static snapshots (page state at crawl time). There is no temporal encoding of HOW content changes over time — the delta between crawls is lost. Two identical pages crawled at different times are treated as identical, even though their temporal evolution (which links changed, which sections updated first, update frequency patterns) carries additional information.

**Proposed Fix**: Add `temporal_delta_encoding` to NT-WORLD crawlers:
1. Don't just store page state — store the INTER-SPIKE interval pattern (time between consecutive content changes per section)
2. First-change latency (analog to first-spike latency) as a feature: sections that change first are more salient
3. Temporal coding of change patterns enables hyperacuity: detecting subtle semantic shifts that static comparison misses
4. This is NOT batch 558's forgetting curve — that addresses retention. This addresses REPRESENTATION of temporal dynamics.

---

### Finding 7: Optimal Temporal Resolution Depends on Signal-Noise Autocorrelation (bioRxiv, Jul 2026)

**Source**: bioRxiv 10.64898/2026.05.19.726394v2 — "On the Optimal Temporal Resolution for Information Representation in Neural Activity"

**New Defect vs Batch 558**: The optimal timescale for information representation is NOT fixed — it depends on the interplay between signal autocorrelation and noise autocorrelation. When both decay, mesoscale representations (intermediate integration) are optimal. When signal persists but noise doesn't, macroscale (long integration) is optimal. NeoTrix's SEAL pipeline runs at a FIXED cycle interval (the background loop tick rate). There is no adaptive temporal resolution. A crawl that changes every hour should be processed at mesoscale; a knowledge base entry that changes monthly should be processed at macroscale; a real-time safety signal should be processed at microscale. The fixed tick rate forces all domains into the same temporal resolution.

**Proposed Fix**: Add an `adaptive_temporal_resolution` mechanism:
1. Measure signal autocorrelation decay rate per domain module (how fast does the information change?)
2. Measure noise autocorrelation decay rate per domain module (how fast do errors/random variations subside?)
3. Compute optimal integration window = f(signal_decay, noise_decay) using the closed-form expressions from the paper
4. Each domain module runs its SEAL/evolution cycle at its own optimal timescale, NOT the global tick rate

**Why batch 558 missed this**: Batch 558's forgetting curve (finding 6) addresses WHEN to replay. This addresses the TIMESCALE at which to process — a different dimension entirely.

---

### Finding 8: IIT 4.0 Φ is Multi-Dimensional, Not Scalar (arXiv 2604.11482)

**Source**: arXiv 2604.11482 — "Integrated information theory: the good, the bad and..." (Apr 2026)

**New Defect vs Batch 558**: IIT researchers now emphasize that Φ (integrated information) should NOT be treated as a single scalar consciousness measure. Instead, Φ should be replaced with a SUITE of quantities: (i) mean Φ over time, (ii) variance of cause-effect structure quantities over time, (iii) complexity of the geometry/topology of cause-effect structures. NeoTrix's ConsciousnessTree tracks a single `phi` value per cycle. This scalar representation loses critical information: the VARIANCE of Phi over time indicates whether consciousness is stable or fluctuating; the GEOMETRY of the cause-effect structure indicates whether consciousness is integrated or fragmented; a system can have low mean Phi but high Phi-variance (unstable consciousness) vs high mean Phi but low variance (stable consciousness) — these are phenomenologically distinct but NeoTrix treats them identically.

**Proposed Fix**: Replace the scalar `phi: f64` in ConsciousnessTree with a `ConsciousnessState` struct:
```rust
struct ConsciousnessState {
    phi_mean: f64,
    phi_variance: f64,
    cause_effect_topology: TopologyGraph, // geometry of Phi structure
    phi_trend: TrendDirection, // increasing/stable/decreasing
    integration_depth: u32, // how many layers participate in integration
}
```

**Why batch 558 missed this**: Batch 558 assumed Phi is a useful scalar and optimized around it. IIT 2026 explicitly rejects the scalar assumption.

---

### Finding 9: Free Energy Weakly Correlated with IIT Φ — Different Measures Track Different Things (arXiv 2608.14165)

**Source**: arXiv 2608.14165 — "Integrated Information in the Active Inference Framework" (Aug 2026)

**New Defect vs Batch 558**: NeoTrix assumes that minimizing free energy (prediction error) automatically maximizes integrated information (consciousness quality). The 2026 empirical study combining AIF agents with IIT measures shows the correlation between free energy and Φ is WEAK and INCREASES with generative model size — meaning for small models (like NeoTrix's current architecture), free energy minimization provides almost NO guarantee of improving consciousness integration. The Φ measures that correlate with free energy are simple mutual information measures, NOT the rich IIT 3.0/4.0 Φ measures that capture true integration. This means NeoTrix's GWT + free-energy-based attention routing may optimize information flow without optimizing information INTEGRATION.

**Proposed Fix**: Add a separate `integration_optimizer` that tracks IIT-style Φ independently of the free-energy minimization:
1. After each reasoning cycle, compute proxy Φ for the active reasoning graph (not just free energy)
2. If free energy is low but Φ is also low, the system is processing efficiently but not integrating — trigger structural reorganization
3. If free energy is high but Φ is high, the system is integrating but processing poorly — trigger parameter tuning
4. The two optimization objectives (minimize FE, maximize Φ) can conflict and must be balanced

**Why batch 558 missed this**: Batch 558's entire framework assumes free-energy minimization is the right objective. This finding shows it's insufficient for consciousness quality.

---

### Finding 10: Synergistic Information as Consciousness Candidate — Pre-Broadcast, Not Broadcast (arXiv 2605.13884)

**Source**: arXiv 2605.13884 — "Consciousness as Uncommon Self-Knowledge: A Synergistic Information Framework" (May 2026)

**New Defect vs Batch 558**: GWT (Global Workspace Theory) states that consciousness IS the broadcast. This 2026 paper proposes a stronger criterion: consciousness correlates with the **pre-broadcast synergy formation** — the synergistic information that exists ONLY in the joint of subsystems and is destroyed by decomposition — NOT the broadcast itself. The broadcast is redundant information (common knowledge); the synergy is the "uncommon self-knowledge" that exists only before broadcast. NeoTrix's GWT implementation broadcasts salient information and treats the broadcast event as the consciousness marker. But if the 2026 finding is correct, NeoTrix is measuring the WRONG thing — it should measure the synergy that EXISTS BEFORE broadcast, not the broadcast itself.

**Proposed Fix**: Add a `PreBroadcastSynergy` computation to GWT:
1. Before broadcasting, compute synergistic information between active modules (not redundant, not unique — synergistic)
2. This synergy score is the consciousness-quality signal, not the broadcast event
3. Track: `consciousness_quality = synergy formed` vs `consciousness_quantity = information broadcast`
4. These can diverge: high broadcast with low synergy = conscious but shallow; low broadcast with high synergy = deep but narrow consciousness
5. Empirical prediction from the paper: anesthesia and Alzheimer's reduce synergy specifically, not redundancy — testable in NeoTrix's self-test suite

**Why batch 558 missed this**: Batch 558 treats GWT broadcast as the consciousness mechanism. This finding separates the pre-broadcast synergy (consciousness) from the broadcast (communication) — two different things.

---

## Sources Cited

| # | Source | Year | Domain | Key Finding |
|---|--------|------|--------|-------------|
| 1 | Springer 10.1007/s11023-026-09787-8 | 2026-06 | PP/AI | Three-timescale learning: inference / parametric / structure |
| 2 | arXiv 2603.20927 | 2026-03 | PP/AI | VFE = surprisal + inference cost (not just model fidelity) |
| 3 | arXiv 2608.09512 | 2026-08 | PP/AI | RGMs: coarse-graining generative models across scales |
| 4 | bioRxiv 10.64898/2026.05.31.729124v3 | 2026-05 | Neural Coding | Bracket coding: dynamic rate-temporal switching |
| 5 | arXiv 2607.13644 | 2026-07 | Neural Coding | Burst-based encoding: 95.6% vs rate-based |
| 6 | Nature Comms s41467-026-76878-6 | 2026-08 | Neural Coding | Temporal coding hyperacuity in event-based vision |
| 7 | bioRxiv 10.64898/2026.05.19.726394v2 | 2026-07 | Neural Coding | Optimal timescale depends on signal-noise autocorrelation |
| 8 | arXiv 2604.11482 | 2026-04 | IIT | Φ is multi-dimensional, not scalar |
| 9 | arXiv 2608.14165 | 2026-08 | IIT+AIF | Free energy weakly correlates with IIT Φ |
| 10 | arXiv 2605.13884 | 2026-05 | IIT/Synergy | Pre-broadcast synergy ≠ broadcast; consciousness = synergy |

## Cross-Cutting Defect Summary

| # | Defect | Source Domain | NeoTrix Component Affected | Severity |
|---|--------|---------------|---------------------------|----------|
| 1 | No structure learning (topology reorganization) | PP/AI | HyperCube/E8 (fixed topology) | High |
| 2 | No inference-cost accounting for KB retrieval | PP/AI | NT-MEMORY retrieval pipeline | Medium |
| 3 | No cross-scale coarse-graining (renormalization) | PP/AI | 6-layer architecture boundaries | High |
| 4 | No dynamic rate-temporal coding switch | Neural Coding | GWT attention routing | Medium |
| 5 | No burst vs tonic signal encoding | Neural Coding | EventBus inter-module communication | Medium |
| 6 | No temporal delta encoding for content changes | Neural Coding | NT-WORLD crawler representation | Medium |
| 7 | Fixed temporal resolution across all domains | Neural Coding | SEAL pipeline tick rate | High |
| 8 | Phi treated as scalar, not multi-dimensional | IIT | ConsciousnessTree state representation | High |
| 9 | Free energy ≠ consciousness integration | IIT+AIF | GWT optimization objective | Critical |
| 10 | GWT measures broadcast, not pre-broadcast synergy | IIT/Synergy | Consciousness quality metric | Critical |

## Meta-Analysis: What Batch 559 Reveals About Batch 558's Blind Spots

Batch 558 operated within the **learning paradigm** — transferability, meta-learning, continual learning, memory policies. All 7 findings addressed HOW to learn better.

Batch 559 operates within the **representation paradigm** — how information is encoded, at what timescale, with what topology, measuring what quantity. The 10 findings address WHAT is being processed and WHETHER the right thing is being measured.

The two paradigms are orthogonal: you can have perfect learning (batch 558) applied to the wrong representation (batch 559). NeoTrix needs both.

**Critical gap**: Findings 9 and 10 together suggest NeoTrix's entire consciousness-quality measurement stack may be measuring the wrong thing. Free energy minimization (finding 9) doesn't guarantee integration; GWT broadcast (finding 10) doesn't constitute consciousness. This is the most fundamental defect discovered across all 559 iterations — the metric itself is misaligned with the phenomenon.
