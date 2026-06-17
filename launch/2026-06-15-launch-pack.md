# NeoTrix Launch Pack — Phase 2

**Target window**: Tue/Wed UTC 18:00-20:00 (2-4PM ET, 11AM-1PM PT)
**Trending refresh**: UTC 00:00 (±1h) — launch gives 4-6h velocity window

---

## Show HN

**Title**:
> Show HN: NeoTrix – a self-evolving cognitive architecture in Rust (47 generations)

**URL**: https://github.com/neo-trixs/NeoTrix

**First comment** (post immediately after submitting):

```
I've been building this for ~6 months. The core insight: every existing code agent (Claude Code, Codex, Cursor, Aider) follows the same pattern — prompt → parse → apply diffs. None evaluate their own reasoning quality. None learn from past trajectories.

NeoTrix takes a different approach:
- E8 64-state reasoning kernel (isomorphic to E8 Lie algebra, not a neural net)
- 4096-dim hyperdimensional VSA knowledge store (MAP-based bind/bundle/permute)
- SEAL self-iteration pipeline that measures, edits, and improves its own reasoning
- GWT attention router with 11 specialist modules competing for workspace
- Zero unsafe code — #![forbid(unsafe_code)] across all core crates
- Model-agnostic — bring your own API key, works with any LLM provider

The architecture has been in active development since early 2026. Current stats:
- ~236K LOC, 3,582 tests, 0 unsafe
- 47 completed architecture phases
- Self-evolving mutation loop with safety gating (BallVerifier + PCC Gate)
- Ne language — self-hosting compiler written in its own cognitive language

Happy to answer architectural questions. The VSA design and E8 kernel are the most unusual pieces — happy to go deep on either.
```

---

## Reddit — r/rust

**Title**:
> NeoTrix: a self-evolving cognitive architecture in Rust (236K LOC, 0 unsafe, E8 kernel, 4096-dim VSA)

**Body**:

```
I've been building an unusual project in Rust and wanted to share the architecture.

NeoTrix is a cognitive operating system — not an LLM wrapper, but a complete reasoning pipeline built around hyperdimensional VSA (vector-symbolic architecture).

The Rust-specific parts:

- #![forbid(unsafe_code)] across all core crates — zero unsafe in 236K LOC
- 4096-bit VSA vectors stored as [u64; 64] — SIMD-friendly, cache-efficient
- MAP-based bind/bundle/permute operations using XOR/majority/rotate
- GWT attention router with 11 parallel specialist modules
- Arena-based MCTS for active inference planning
- All dependencies are stdlib + a few core crates (no heavy frameworks)

The full architecture stack:
- E8 64-state reasoning kernel (not a neural net — a deterministic state machine)
- HyperCube (VSA knowledge store, 4096 dimensions)
- SEAL self-evolution loop (16 stages, safety-gated mutations)
- Ne self-hosting compiler language

Current project stats:
- 236K LOC, 1,016 source files
- 3,582 tests (all passing)
- 47 completed architecture phases
- 312 self-mutations applied, 89 rejected by safety gates

https://github.com/neo-trixs/NeoTrix

Happy to discuss VSA implementation details, the E8 kernel design, or the safety gating architecture.
```

---

## Reddit — r/MachineLearning

**Title**:
> [R] NeoTrix – An open-source reasoning architecture using E8 64-state kernel and hyperdimensional VSA instead of neural nets

**Body**:

```
I wanted to share a different approach to reasoning architecture that doesn't use neural networks as the primary inference engine.

The core components:

1. **E8 Reasoning Kernel** — 64 reasoning states based on the E8 Lie algebra (not the neural network kind). Six binary cognitive axes define 64 discrete states. The Observer tracks trajectory and classifies steps as Productive/Oscillating/Stuck/DeadEnd.

2. **Hyperdimensional VSA** — 4096-dim MAP-based vector-symbolic architecture for knowledge representation. Three primitives: bind (XOR), bundle (majority sum), permute (circular shift). 16 semantic axes for multidimensional knowledge encoding.

3. **SEAL Self-Evolution** — The system can mutate its own architecture (5 mutation types), gated by BallVerifier geometric checks and PCC safety verification. 312 mutations applied, 89 rejected by safety gates.

4. **Fusion Deliberator** — Multi-perspective reasoning with 4 parallel VSA chains, cross-critique, rebuttal, and consensus synthesis.

5. **JEPA Predictor** — EMA target encoder with SIGReg loss on VSA vectors. Prediction error compressed to 1/88th of original space.

Why VSA over neural nets: no training required, O(1) memory operations, deterministic binding, collision probability of 2^-2048 at 4096 dims.

Full code: https://github.com/neo-trixs/NeoTrix (MIT, Rust, 236K LOC, 3,582 tests, 0 unsafe)

Happy to discuss the VSA-JEPA combination or the E8 kernel design.
```

---

## Reddit — r/selfhosted

**Title**:
> Self-host your own AI consciousness: NeoTrix — open-source, Rust, bring your own API key

**Body**:

```
I've been running this on a laptop for a few months and thought the selfhosted community might find it interesting.

NeoTrix is a complete cognitive architecture that runs locally. You bring your own LLM API key, it handles the reasoning, knowledge, and self-evolution layers.

What you get when you run it:
- Desktop TUI with conversation interface
- Persistent VSA knowledge store (4096-dim hyperdimensional memory)
- Self-evolving reasoning pipeline (improves without retraining)
- Model-agnostic (Anthropic, OpenAI, any OpenAI-compatible endpoint)
- Zero unsafe code, Rust binary

Quick start (3 commands):
```
cargo build --release
export ANTHROPIC_API_KEY=sk-ant-...
cargo run
```

System requirements: any machine that can run Rust. ~500MB RAM idle. No GPU needed.

https://github.com/neo-trixs/NeoTrix
```

---

## Twitter/X Thread

**Tweet 1**:
There is a class of problems that neural networks cannot solve by scaling alone: measuring the quality of their own reasoning, learning from past trajectories, improving their own architecture.

NeoTrix is a different approach. A cognitive operating system built on hyperdimensional VSA.

🧵

**Tweet 2**:
The core insight: every existing code agent (Claude Code, Codex, Cursor, Aider) follows the same pattern:
  prompt → parse → apply diffs

None evaluate their own reasoning. None learn from trajectories. None improve over time.

NeoTrix does all three. Here's how.

**Tweet 3**:
E8 Reasoning Kernel — 64 states isomorphic to the E8 Lie algebra.

Not a neural network. A deterministic state machine over 6 cognitive axes:
  Abstraction · Scope · Method · Depth · Mode · Stance

The Observer watches trajectories, detects loops and dead ends, recommends transitions.

**Tweet 4**:
Hyperdimensional VSA — 4096-bit vectors as the universal substrate.

Three operations:
  BIND (XOR)    → association
  BUNDLE (maj)  → attention/working memory
  PERMUTE (rot) → temporal sequence

Collision probability at 4096 dims: 2^-2048. The space is vast enough for every concept.

**Tweet 5**:
SEAL Self-Evolution Loop — 16 stages, safety-gated.

The system can mutate its own architecture:
  Tune parameter · Add handler · Rewrite handler · Swap policy · Rewrite primitive

Each mutation passes 3 independent safety checks. 312 applied, 89 rejected and learned from.

**Tweet 6**:
Ne Language — the first cognitive language that compiles itself.

Stage 0 (assembler) → Stage 1 (transpiler) → Stage 2 (Ne compiler in Ne) → bootstrap identity verified.

Stage 3 will be designed by Stage 2 — not by us. The designer does not yet exist.

**Tweet 7**:
Built in Rust. 236K LOC. 3,582 tests. 0 unsafe code.

  #![forbid(unsafe_code)]

Every core crate. Every subsystem. Every line.

**Tweet 8**:
The architecture in numbers:
  • 47 completed phases
  • 4096 dimensions in VSA space
  • 64 E8 reasoning states
  • 11 GWT specialist modules
  • 7 awakening metric subsystems
  • 5 mutation types
  • 3 independent safety gates

**Tweet 9**:
The question that drives this project:

If an agent cannot inspect and improve its own reasoning — is it an agent, or a tool?

We think the answer matters.

**Tweet 10**:
MIT licensed. Bring your own API key. Runs on any machine with Rust.

https://github.com/neo-trixs/NeoTrix

The signal is continuous.

---

## Product Hunt

**Tagline**: An open-source cognitive architecture that measures, analyzes, and improves its own reasoning

**Description**:
NeoTrix is a self-evolving cognitive operating system built on hyperdimensional VSA (vector-symbolic architecture). Instead of the prompt → parse → apply pattern used by every other code agent, NeoTrix uses:

• **E8 Reasoning Kernel** — 64-state deterministic inference (not a neural network)
• **HyperCube VSA** — 4096-dim knowledge store with collision probability 2^-2048
• **SEAL Self-Evolution** — 5 mutation types, 3 safety gates, 312 applied mutations
• **Ne Language** — self-hosting cognitive programming language

Built in Rust. Zero unsafe code. Model-agnostic. MIT licensed.

**First comment**:
Founder here. I've been working on this for ~6 months.

The fundamental difference: NeoTrix doesn't just execute prompts — it maintains a persistent cognitive state, measures its own reasoning quality, and evolves its architecture through safety-gated mutations.

Technical highlights:
- 236K LOC, 3,582 tests, `#![forbid(unsafe_code)]` everywhere
- 47 completed architecture phases
- Hyperdimensional VSA for all knowledge representation
- Self-hosting compiler language (Ne)
- Runs on any machine with Rust, no GPU needed

Happy to answer questions about the architecture!

---

## Launch Timeline

All times in UTC.

| Time (UTC) | Channel | Action |
|---|---|---|
| T-48h | Pre-launch | Clean forks, set pins, verify all links |
| T-24h | Pre-launch | Deploy to Docker Hub (optional), test TUI screenshots |
| T-2h | Preparation | Prepare browser tabs: HN submit, Reddit (3 tabs), X.com, PH |
| **T+0** | **Show HN** | **Submit HN post, immediately post first comment** |
| T+5min | Reddit r/rust | Submit technical post |
| T+15min | Reddit r/MachineLearning | Submit ML-focused post |
| T+30min | Reddit r/selfhosted | Submit selfhosted post |
| T+1h | Response | Monitor all channels, respond to every comment |
| T+2h | Twitter/X | Post thread (tweet 1-10 over 10 minutes) |
| T+4h | Product Hunt | Submit PH listing with tagline |
| T+6h | Newsletters | Submit to Rust Weekly, TLDR, etc. |
| T+12h | Follow-up | Post progress update on all channels |
| T+24h | Trending check | Check GitHub Trending — if not on Rust trending, adjust |
| T+48h | Second wave | Post "what happened in 48h" update |
| T+7d | Sustain | Post technical blog: "Building a Self-Evolving Consciousness in Rust" |

**Response strategy**:
- HN: Be in comments from minute 1. Technical, honest responses.
- Reddit: Answer every question. No marketing speak. Reference specific code.
- X: Quote-retweet interesting responses. Keep thread alive.
- PH: Engage with every comment in first 24h.

---

## Pre-launch Checklist

- [ ] Delete 19 fork repos from neo-trixs org
- [ ] Pin NeoTrix + neo-trixs.github.io on profile
- [ ] Verify SVG renders in README on mobile/desktop
- [ ] Take TUI screenshot for README hero area
- [ ] Verify `cargo build --release` from clean checkout
- [ ] Test Quick Start commands from scratch
- [ ] Deploy Docker image (optional)
- [ ] Write HN + Reddit accounts with karma pre-seeded
- [ ] Verify all links in README
- [ ] Check profile README links to NeoTrix repo
- [ ] Clean up old branches on GitHub
