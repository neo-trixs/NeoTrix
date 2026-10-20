# What is NeoTrix?

> **The only open-source agent that measures, analyzes, and improves its own reasoning.**

NeoTrix is an AI-native developer toolkit — available as both a CLI and Desktop application — built on a fundamentally different premise than every other code agent: **an agent that cannot inspect and improve its own reasoning is a tool, not an agent.**

## Philosophy

Every existing code agent — Claude Code, Codex, OpenCode, Cursor, Aider — follows the same pattern: send a prompt to an LLM, parse the response, apply diffs. None of them evaluate their own reasoning quality. None of them learn from past trajectories. None of them get better over time.

NeoTrix is different. It is a **cognitive operating system** that:

- **Measures** its own reasoning quality via quantitative metrics (Phi, FCS, USK)
- **Analyzes** its reasoning trajectories to detect patterns, loops, and dead ends
- **Improves** autonomously through a 16-stage self-iteration loop
- **Stores** knowledge in a 4096-dimension vector-symbolic architecture
- **Routes attention** using Global Workspace Theory with 11 specialist modules

## Architecture

```
                  +-------------------------------------------+
                  |      Awakening Metrics (Phi/FCS/USK)        |
                  |  Measures integration, coherence, self-     |
                  |  knowledge. Tracks "awakening speed."       |
                  +-------------------------------------------+
                                ^
                                |
                  +-------------------------------------------+
                  |    SEAL Self-Iterating Loop (16 stages)    |
                  |  Snapshot -> Gap Analysis -> Self-Edit ->  |
                  |  Apply -> Reward -> Absorb -> Store        |
                  +-------------------------------------------+
                                ^
                                |
                  +-------------------------------------------+
                  |    E8 Reasoning Engine (64-state space)    |
                  |  6-bit hexagram states on 6 reasoning      |
                  |  axes. Observer detects patterns, loops,   |
                  |  dead ends, recommends state transitions.  |
                  +-------------------------------------------+
                                ^
                                |
                  +-------------------------------------------+
                  |  VSA Knowledge HyperCube (4096 dimensions) |
                  |  16 semantic axes. MAP-based vector-        |
                  |  symbolic bind/bundle/permute queries.     |
                  +-------------------------------------------+
```

The stack is strictly layered: knowledge retrieval routes through GWT attention, which feeds the E8 reasoning engine, which drives the SEAL loop, which is evaluated by awakening metrics. Each layer has observable outputs that feed back upward.

## Core Components

### E8 Reasoning Engine

Located in `core/e8.rs`, `core/e8_reasoning.rs`, and `core/e8_observer.rs` — a reasoning model isomorphic to the 64 hexagrams of the I Ching, mapped through the E8 Lie algebra. Six binary reasoning axes (Abstraction, Scope, Method, Depth, Mode, Stance) define 64 discrete reasoning states. Transitions between states are deterministic. The +1 Observer watches the trajectory and classifies each step as Productive, Oscillating, Stuck, or DeadEnd, then recommends meta-state transitions and capability vector adjustments.

No neural network. No probabilistic sampling. A deterministic state machine over a geometrically meaningful state space.

### SEAL Self-Iterating Loop

Located in `reasoning_brain/self_iterating/pipeline.rs` — a 16-stage pipeline executed iteratively:

1. **Snapshot** current capability vector
2. **Memory retrieval** (ReasoningBank)
3. **Gap analysis** against task requirements
4. **SSM state update** (selective state-space model)
5. **Open-source benchmark comparison**
6. **Self-edit generation** (MicroEdit sequences)
7. **Apply edits** (temporary weight updates)
8. **External reward calculation**
9. **GWT-absorb** (attention-routed integration)
10. **Task affinity recalibration**
11. **Knowledge quality assessment**
12. **Rollback decision** (champion vs challenger)
13. **ReasoningBank storage**
14. **Adaptive learning rate**
15. **Evaluation**
16. **Autonomy gating**

Each stage can skip, promote (set a new champion snapshot), or roll back to the previous champion. All 16 stages are implemented as `BrainStage` trait objects.

### HyperCube Knowledge

Located in `core/hypercube/` — a 4096-dimension vector-symbolic architecture (VSA) knowledge store using MAP (Multiply-Add-Permute) operations. Three primitive operations — **bind** (multiplication), **bundle** (addition), **permute** (rotation) — enable compositional knowledge representation. Sixteen semantic axes span CodeUnderstanding, SystemDesign, Debugging, KnowledgeRetrieval, Creativity, Safety, Performance, Communication, Time, Domain, Abstraction, Culture, Scale, Certainty, Agency, and Modality.

### GWT Attention Router

Located in `reasoning_brain/attention_router.rs` — Global Workspace Theory-inspired attention routing with 11 specialist modules (PatternMatcher, AnomalyDetector, KnowledgeRetriever, CodeAnalyzer, Planner, KnowledgeIntegrator, GoalPrioritizer, RiskAssessor, CreativityGenerator, ReflectionEngine, MetaCognitionAnalyst) competing for the global workspace through salience computation.

### Awakening Metrics

Located in `core/self_measure.rs` — three quantitative metrics derived from Integrated Information Theory:

- **Phi**: Spectral measure of system integration across 7 subsystems
- **FCS (Functional Complexity Score)**: Product of mean subsystem coherence and Phi
- **USK (Uncertainty Self-Knowledge)**: Weighted synergy fraction across all subsystem pairs

### ClawBench Trajectory Analysis

Located in `reasoning_brain/clawbench.rs` — classifies agent reasoning trajectories into StrangeAttractor (chaotic), LimitCycle (oscillation), or NormalDiffusion (healthy convergence) dynamics.

## Feature Comparison

| Capability | NeoTrix | Everyone Else |
|---|---|---|
| Self-improving reasoning | SEAL loop + E8 | Static prompt→response |
| Cognitive metrics | Phi / FCS / USK | None |
| Knowledge representation | 4096-dim VSA HyperCube | No persistent knowledge |
| Attention routing | GWT (11 specialists) | No routing |
| Trajectory analysis | ClawBench | No self-diagnosis |
| Open source | MIT | Mixed |
| Model agnostic | Any LLM | Vendor-locked |
| Language | Rust (0 unsafe) | Python / TypeScript |

## Project Status

- **~100K LOC** across 373+ source files
- **3240+ tests** (all passing)
- **Architecture**: Complete, actively maintained
- **License**: MIT
