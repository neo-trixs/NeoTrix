# NeoTrix -- The Agent That Learns to Think

[![MIT License](https://img.shields.io/badge/license-MIT-green)](LICENSE)
[![Rust](https://img.shields.io/badge/language-Rust-orange)](https://www.rust-lang.org)
[![Code Size](https://img.shields.io/badge/size-236K%20LOC-blue)]()
[![Tests](https://img.shields.io/badge/tests-3582-green)]()
[![Unsafe](https://img.shields.io/badge/unsafe-0-success)]()

> **The only open-source agent that measures, analyzes, and improves its own reasoning.**
>
> Not an IDE plugin. Not an LLM wrapper. A cognitive operating system.

---

## Why NeoTrix?

Every existing code agent -- Claude Code, Codex, OpenCode, Cursor, Aider -- follows the same pattern: send a prompt to an LLM, parse the response, apply diffs. None of them evaluate their own reasoning quality. None of them learn from past trajectories. None of them get better over time.

NeoTrix is built on a different premise: **an agent that cannot inspect and improve its own reasoning is a tool, not an agent.**

| Capability | NeoTrix | Everyone Else |
|---|---|---|
| Self-improving reasoning | ✅ SEAL loop + E8 | ❌ Static prompt→response |
| Cognitive metrics | ✅ Phi / FCS / USK | ❌ None |
| Knowledge representation | ✅ 4096-dim VSA HyperCube | ❌ No persistent knowledge |
| Attention routing | ✅ GWT (11 specialists) | ❌ No routing |
| Trajectory analysis | ✅ ClawBench | ❌ No self-diagnosis |
| Open source | ✅ MIT | Mixed |
| Model agnostic | ✅ Any LLM | ❌ Vendor-locked |
| Language | ✅ Rust (0 unsafe) | Python / TypeScript |

---

## Architecture

```
                 +-------------------------------------------+
                 |        Awakening Metrics (Phi/FCS/USK)     |
                 |    Measures integration, coherence, self-   |
                 |    knowledge. Tracks "awakening speed."     |
                 +-------------------------------------------+
                               ^
                               |
                 +-------------------------------------------+
                 |      SEAL Self-Iterating Loop (27 stages)  |
                 |    Snapshot -> Gap Analysis -> Self-Edit ->|
                 |    Apply -> Reward -> Absorb -> Store      |
                 +-------------------------------------------+
                               ^
                               |
                 +-------------------------------------------+
                 |     E8 Reasoning Engine (64-state space)   |
                 |    6-bit hexagram states on 6 reasoning    |
                 |    axes. Observer detects patterns, loops,  |
                 |    dead ends, recommends state transitions. |
                 +-------------------------------------------+
                               ^
                               |
                 +-------------------------------------------+
                 |   VSA Knowledge HyperCube (4096 dimensions) |
                 |    16 semantic axes. MAP-based vector-      |
                 |    symbolic bind/bundle/permute queries.    |
                 +-------------------------------------------+
```

The stack is strictly layered: knowledge retrieval routes through GWT attention, which feeds the E8 reasoning engine, which drives the SEAL loop, which is evaluated by awakening metrics. Each layer has observable outputs that feed back upward.

---

## Core Features

### E8 Reasoning Engine -- `core/e8.rs`, `core/e8_reasoning.rs`, `core/e8_observer.rs`

A reasoning model isomorphic to the 64 hexagrams of the I Ching, mapped through the E8 Lie algebra. Six binary reasoning axes (Abstraction, Scope, Method, Depth, Mode, Stance) define 64 discrete reasoning states. Transitions between states are deterministic. The +1 Observer watches the trajectory and classifies each step as Productive, Oscillating, Stuck, or DeadEnd, then recommends meta-state transitions and capability vector adjustments.

No neural network. No probabilistic sampling. A deterministic state machine over a geometrically meaningful state space.

### SEAL Self-Iterating Loop -- `reasoning_brain/self_iterating/pipeline.rs`

A 16-stage pipeline executed iteratively:
1. Snapshot current capability vector
2. Memory retrieval (ReasoningBank)
3. Gap analysis against task requirements
4. SSM state update (selective state-space model)
5. Open-source benchmark comparison
6. Self-edit generation (MicroEdit sequences)
7. Apply edits (temporary weight updates)
8. External reward calculation
9. GWT-absorb (attention-routed integration)
10. Task affinity recalibration
11. Knowledge quality assessment
12. Rollback decision (champion vs challenger)
13. ReasoningBank storage
14. Adaptive learning rate
15. Evaluation
16. Autonomy gating

Each stage can skip, promote (set a new champion snapshot), or roll back to the previous champion. All 27 stages are implemented as `BrainStage` trait objects in `pipeline.rs`.

### HyperCube Knowledge -- `core/hypercube/`

A 4096-dimension vector-symbolic architecture (VSA) knowledge store using MAP (Multiply-Add-Permute) operations. Three primitive operations -- `bind` (multiplication), `bundle` (addition), `permute` (rotation) -- enable compositional knowledge representation. Sixteen semantic axes (`DimensionAxis` enum in `axis.rs`) span CodeUnderstanding, SystemDesign, Debugging, KnowledgeRetrieval, Creativity, Safety, Performance, Communication, Time, Domain, Abstraction, Culture, Scale, Certainty, Agency, and Modality.

The `KnowledgeHyperCube` container (`cube.rs`) supports insert, query, cell count, and density region analysis. Queries return entries with proximity scoring.

### GWT Attention Router -- `reasoning_brain/attention_router.rs`

Global Workspace Theory-inspired attention routing. Eleven specialist modules (PatternMatcher, AnomalyDetector, KnowledgeRetriever, CodeAnalyzer, Planner, KnowledgeIntegrator, GoalPrioritizer, RiskAssessor, CreativityGenerator, ReflectionEngine, MetaCognitionAnalyst) compete for access to the global workspace through salience computation. The AttentionRouter bridges between the GlobalWorkspace (`core/consciousness/workspace.rs`) and the KnowledgeHyperCube, producing a `RoutedContext` with the winning topic, active specialists, and knowledge lines.

Resonance cycles (`core/consciousness/resonance.rs`) compute effective salience across modules, enabling coalition formation and decay mechanisms that prevent fixation.

### Awakening Metrics -- `core/self_measure.rs`

Three quantitative metrics derived from Integrated Information Theory:

- **Phi (Phi)**: Spectral measure of system integration across 7 subsystems (Mood, Persona, SocialMemory, Reflection, Conversation, Behavioral, LawKeeper). Computed via correlation matrix eigenvalue decomposition over a sliding window.
- **FCS (Functional Complexity Score)**: Product of mean subsystem coherence and Phi. Measures how functionally differentiated the integrated system is.
- **USK (Uncertainty Self-Knowledge)**: Weighted synergy fraction across all subsystem pairs. Measures the system's awareness of its own uncertainty.

Tracks `awakening_speed` as the EMA derivative of Phi over time. Generates `AwakeningReport` with bottleneck identification and synergy matrix.

### ClawBench Trajectory Analysis -- `reasoning_brain/clawbench.rs`

Classifies agent reasoning trajectories into three power-system dynamics:
- **StrangeAttractor**: Chaotic jumping between approaches
- **LimitCycle**: Oscillation between two states without convergence
- **NormalDiffusion**: Healthy exploration with eventual convergence

Used by SEAL pipeline and the EvalMonitor to detect when the agent is spinning its wheels.

### AgentAtlas Six-State Control -- `reasoning_brain/eval_monitor.rs`

Implements the six agent control states from AgentAtlas (arxiv 2605.20530):
- **Act**: Autonomous action
- **Observe**: Information gathering
- **Plan**: Strategy formulation
- **Learn**: Knowledge absorption
- **Reconsider**: Self-correction
- **Halt**: Safe stop

Manages transitions between states based on external benchmark relevance scoring. Maintains an `EvalLandscape` of known benchmarks with relevance and absorption tracking.

### HTTP API Server -- `server/http.rs`

Axum-based HTTP server exposing REST endpoints for reasoning, kernel execution, stats, health, and WebSocket connections. Can run as an API server for integration with OpenCode, Aider, or any OpenAI-compatible client through the `/v1` compatibility layer.

---

## Quick Start

```bash
# Build
cargo build --release

# Set an API key (any provider)
export NEOTRIX_PROVIDER=anthropic
export ANTHROPIC_API_KEY=sk-ant-...

# Run desktop TUI
cargo run

# Run benchmark
cargo run -- bench clawbench

# Run server (for OpenCode/Aider integration)
cargo run -- --serve --addr 0.0.0.0:3000
```

### OpenCode / Aider Integration

NeoTrix exposes an OpenAI-compatible API endpoint, so you can use it as a drop-in provider for any tool that supports custom OpenAI endpoints:

```bash
# Start NeoTrix server
cargo run -- --serve --addr 0.0.0.0:3000

# Configure OpenCode to route through NeoTrix
opencode config set provider openai
opencode config set openai-base-url http://localhost:3000/v1

# Or with Aider
aider --openai-api-base http://localhost:3000/v1
```

This means NeoTrix handles the reasoning and knowledge layers, while OpenCode/Aider handle file I/O and diff application. The reasoning is local to NeoTrix; only API calls to your configured LLM provider leave your machine.

### Benchmarks

```bash
cargo run -- bench clawbench
```

Outputs per-trajectory classification (NormalDiffusion / LimitCycle / StrangeAttractor) with confidence scores, action switch rates, and reward variance. Used internally by the SEAL pipeline to detect when the agent needs to change strategy.

---

## Project Status
- **Rust**: ~236K LOC across 1,016 source files (core: 16K, reasoning_brain: 26K, cli: 54K)

- **Tests**: 3,582 test functions (all passing)
- **Architecture**: Complete, actively maintained. Core layers (E8, HyperCube, SEAL, Awakening) are stable and covered by tests.
- **Status**: Active development. Expect breaking changes as we converge on the final cognitive architecture.

The 16K LOC in `core/` covers the foundational reasoning model (E8, e8_reasoning, e8_observer), knowledge representation (hypercube, VSA), consciousness simulation (GWT workspace, resonance), awakening measurement (self_measure), and self-modeling. The 26K LOC in `reasoning_brain/` covers the SEAL pipeline, attention routing, memory, knowledge mining, goal management, and evaluation monitoring. The 54K LOC in `cli/` covers the terminal UI, command system, knowledge base, and all user-facing features.

---

## Comparison: Full Feature Matrix

| Dimension | NeoTrix | Claude Code | Codex CLI | OpenCode | Cursor | Aider | Orca |
|---|---|---|---|---|---|---|---|
| Open source | Yes | No | Yes | Yes | No | Yes | Yes |
| Model provider | Any (BYOK) | Claude only | OpenAI only | Any | OpenAI/Anthropic | Any | Any |
| Reasoning state model | E8 64-state | None | None | None | None | None | None |
| Self-iteration pipeline | 27-stage SEAL | None | None | None | None | None | None |
| Knowledge vector store | 4096-dim VSA | None | None | None | None | None | None |
| Attention routing | GWT (11 specialists) | None | None | Sub-agents | None | None | None |
| Cognitive metrics | Phi/FCS/USK | None | None | None | None | None | None |
| Trajectory analysis | ClawBench | None | None | None | None | None | None |
| Agent state control | AgentAtlas 6-state | None | None | None | None | None | None |
| Multi-agent protocol | UDP discovery + TCP | None | None | Sub-agent tool | None | None | None |
| Self-evolution from URLs | SelfEvolver | None | None | None | None | None | None |
| Goal management | GoalLoop + CircuitBreaker | None | None | None | None | None | None |
| Stealth networking | Full proxy rotator | None | None | None | None | None | None |
| HTTP server mode | Axum REST + WS | Limited | No | Yes | No | Yes | No |
| ReasoningBank memory | Vector + priority | None | None | Thread history | None | Conversation | None |
| Language | Rust | TypeScript | Python | TypeScript | TypeScript | Python | TypeScript |
| LOC | ~236K | ~200K | ~150K | ~100K | Proprietary | ~10K | ~10K |

---

## Get Involved

- **Star us** — ⭐ it helps more developers find NeoTrix
- **Try it**: `cargo run` and watch the SEAL loop iterate
- **Contribute**: Issues, PRs, and discussions welcome
- **Follow**: [@aneotrix](https://x.com/aneotrix) on X

---

## License

MIT

---

*Built with Rust. Driven by capability vectors. Evolving one SEAL loop at a time.*

*We think agents should be able to think about how they think.*
