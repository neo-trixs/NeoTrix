# Research Batch 664 — Repository Analysis

**Date**: 2026-09-13
**Sources**: 6 repositories analyzed for architectural patterns and fusion opportunities

---

## 1. PrimeIntellect-ai/prime-agent

**URL**: https://github.com/PrimeIntellect-ai/prime-agent
**Stars**: 20.6k | **Forks**: 2.3k | **License**: MIT

### Summary
Self-improving coding agent built around the **Recursive Language Model (RLM)** abstraction. Treats context as variables (prompt-as-a-variable) and tools like recursive subagents as function calls inside a persistent Python REPL. Built on top of `pi` (earendil-works/pi).

### Key Architectural Patterns

| Pattern | Description | NeoTrix Mapping |
|---------|-------------|-----------------|
| **Prompt-as-Variable** | Context stored as mutable variables, not static strings | GWT attention routing — salience as mutable state |
| **Continual Harness** | Durable supplemental prompts, memories, skills as refinable state | NT-MIND skill crystallization + experience-tree |
| **Subagent Spawning** | `rlm.spawn(...)` for parallel/background child agents | NT-ACT orchestration + worktree isolation |
| **Background Daemon** | Sessions survive terminal detach via daemon + reattach | NT-IO daemon lifecycle (planned) |
| **Agent-to-Agent Comms** | Running agents discover each other and exchange messages | NT-NEXUS cross-session memory |
| **Refinement Loop** | `/refine` applies evidence-backed updates to harness state | SEAL pipeline distillation phase |

### Fusion Opportunities

1. **RLM-Style Skill Refinement**: Port Continual Harness refinement concept to NT-MIND. Each SEAL cycle produces evidence-backed skill updates with recorded snapshots and rollback capability — aligns with R-P79 (absorption must connect to production).

2. **Subagent Protocol**: `rlm.spawn(...)` maps directly to NT-ACT `ParallelTaskManager`. The daemon-backed continuity pattern (detach/reattach) should inform NT-IO's planned daemon architecture.

3. **Prompt-as-Variable → SelfModel Extension**: Prime Agent's mutable context variables parallel NeoTrix's SelfModel types (nt_core_meta / nt_core_self / nt_core_self_model). The three SelfModel types could implement a unified variable-store pattern where context mutations are tracked and rolled back.

### Implementation Changes

- `nt_core_self`: Add `RefinementSnapshot` type for skill state with rollback support
- `nt_act::parallel_task`: Adopt RLM spawn/collect protocol for subagent orchestration
- `nt_io`: Design daemon lifecycle (startup/detach/reattach) modeled on Prime Agent's daemon architecture

---

## 2. BehiSecc/HuntProxy

**URL**: https://github.com/BehiSecc/HuntProxy
**Stars**: 190 | **Forks**: 16 | **License**: Apache-2.0

### Summary
Web security workbench built specifically for AI agents. MCP-based interface for Burp-Suite-like workflows: proxy capture, browser automation, fuzzing, request replay, findings management. Written in Rust.

### Key Architectural Patterns

| Pattern | Description | NeoTrix Mapping |
|---------|-------------|-----------------|
| **MCP Bridge** | Stdio MCP server for agent integration | NT-IO MCP gateway |
| **Project-Scoped State** | Per-project persistent state (traffic, cookies, findings) | KB namespace isolation |
| **Plugin Architecture** | Bounded plugins with network access control | NT-ACT tool nodes / Rune Socketing |
| **Daemon + Inspector** | Background daemon with web inspector on loopback | NT-IO daemon pattern |
| **Request Rules Engine** | Ordered URL/header/body rewrites across all tools | NT-SHIELD egress policy engine |
| **Evidence Chain** | Findings attached to exchanges with labels/notes | KB experience tracking |

### Fusion Opportunities

1. **Egress Privacy Guard Enhancement**: HuntProxy's request rules engine (ordered rewrites across proxy/browser/fuzzer) is isomorphic to NeoTrix's Egress Privacy Guard. The ordered-rewrite pattern should replace the current single-pass redaction with layered trust tiers.

2. **Project-Scoped KB Namespaces**: HuntProxy's per-project isolation (traffic, cookies, findings) maps to NT-MEMORY namespace-per-domain pattern. Each NeoTrix domain could get a similar scoped workspace.

3. **Plugin Bounded Security**: HuntProxy's bounded plugin model (plugins cannot bypass network controls or credential limits) is exactly the Rune Socketing security model. Port the `deny_wins` + `default_allow` fallback pattern from HuntProxy's egress policy.

4. **Web Security Skill**: Create NT-ACT skill node `Sec-Hunt` (HuntProxy integration) for agent-driven web security testing via MCP.

### Implementation Changes

- `nt_shield::egress_guard`: Adopt ordered-rewrite chain from HuntProxy's request rules
- `nt_act`: Add `HuntProxyMcp` tool node for security testing workflows
- `nt_memory`: Implement project-scoped KB namespaces with export/import

---

## 3. cobanov/awesome-fly

**URL**: https://github.com/cobanov/awesome-fly
**Stars**: 119 | **Forks**: 5 | **License**: CC0-1.0

### Summary
Curated list of fruit fly (Drosophila) connectome projects — MaleCNS, FlyWire brain simulations, embodied models (MuJoCo), games, and research tools. 60+ projects covering brain models, desktop pets, game controllers, analysis libraries.

### Key Architectural Patterns

| Pattern | Description | NeoTrix Mapping |
|---------|-------------|-----------------|
| **Connectome-as-Graph** | Neurons as nodes, synapses as edges | KB graph structure |
| **LIF Simulation** | Leaky Integrate-and-Fire neural dynamics | NT-CORE E8 reasoning engine |
| **Closed-Loop Sensorimotor** | Sensory input → neural processing → motor output | NT-WORLD + NT-ACT + NT-PHYSICAL pipeline |
| **Readout Training** | Small trainable readout on fixed connectome substrate | SelfModel calibration |
| **Cross-Dataset Matching** | Unified queries across MaleCNS/FlyWire/FAFB | KB cross-namespace joins |
| **Embodied Simulation** | Brain model + MuJoCo body + environment | NT-PHYSICAL body schema |

### Fusion Opportunities

1. **Connectome-Graph ↔ KB Graph**: The fly connectome community's analysis libraries (NAVis, cocoa, connectome-interpreter) implement exactly the graph operations NeoTrix needs for KB. The path-finding and circuit-manipulation patterns from `connectome-interpreter` could enhance NT-MEMORY graph traversal.

2. **LIF Neural Dynamics → E8**: Fly projects use LIF (Leaky Integrate-and-Fire) neurons as computational substrate. NeoTrix's E8 reasoning engine could incorporate LIF-style temporal dynamics for more biologically-inspired reasoning.

3. **Closed-Loop Architecture**: The sensorimotor loop pattern (sensory → neural → motor → environment feedback) maps directly to NT-WORLD → NT-CORE → NT-ACT → NT-PHYSICAL pipeline. The `FlyGym` framework's sensor interface pattern is a clean template.

4. **Readout Training → SelfModel**: The pattern of training small readout networks on frozen connectome substrates parallels SelfModel calibration — train lightweight adapters on frozen model representations.

### Implementation Changes

- `nt_core`: Study LIF dynamics from `flyvis` / `fly-brain` for temporal reasoning extensions
- `nt_memory::graph`: Port path-finding algorithms from `connectome-interpreter`
- `nt_physical::body_schema`: Adopt MuJoCo-style embodied simulation pattern from `flybody`

---

## 4. rasbt (Sebastian Raschka)

**URL**: https://github.com/rasbt
**Profile**: AI Research Engineer, LLM specialist

### Key Repositories Analyzed

| Repository | Stars | Relevance |
|-----------|-------|-----------|
| `LLMs-from-scratch` | 105k | From-scratch LLM implementation in PyTorch |
| `reasoning-from-scratch` | 5.2k | Reasoning LLM from scratch |
| `mini-coding-agent` | 1.2k | Minimal coding agent harness |
| `llm-architecture-gallery` | 1.5k | LLM architecture taxonomy |
| `deeplearning-models` | 17.6k | Deep learning architecture collection |

### Key Architectural Patterns

| Pattern | Description | NeoTrix Mapping |
|---------|-------------|-----------------|
| **From-Scratch Pedagogy** | Build every component from fundamentals | SEAL distillation — explain before absorbing |
| **Mini Agent Harness** | Minimal coding agent with clear separation | NT-ACT tool orchestration |
| **Architecture Gallery** | Taxonomy of LLM architectures | KB knowledge organization |
| **Reasoning Pipeline** | Step-by-step reasoning chains | NT-CORE E8 reasoning |

### Fusion Opportunities

1. **Mini-Coding-Agent Pattern**: The `mini-coding-agent` project demonstrates a minimal, readable agent harness. Its architecture (tool loop → model call → result parsing → action execution) should inform NT-ACT's tool loop design — keep it simple, readable, traceable.

2. **From-Scratch Documentation**: Raschka's pedagogical approach (build from fundamentals) aligns with NeoTrix's CONSTELLATION maturity model (C0=C4 pipeline). Each skill should have a "from-scratch" documentation path.

3. **Reasoning Chain Patterns**: `reasoning-from-scratch` implements chain-of-thought and tree-of-thought patterns. These could enhance NT-CORE's E8 reasoning engine with structured reasoning traces.

### Implementation Changes

- `nt_act`: Study `mini-coding-agent` for minimal tool loop design
- `nt_core::reasoning`: Integrate reasoning chain patterns from `reasoning-from-scratch`
- Skill documentation: Adopt "from-scratch" pedagogy for each NT skill

---

## 5. earendil-works/pi-review

**URL**: https://github.com/earendil-works/pi-review
**Stars**: 494 | **Forks**: 33 | **License**: MIT

### Summary
Code review extension for Pi (the agent Prime Agent is built on). Adds `/review` and `/end-review` commands for structured code review workflows. Supports uncommitted changes, branch diffs, commit review, PR review, and folder snapshots.

### Key Architectural Patterns

| Pattern | Description | NeoTrix Mapping |
|---------|-------------|-----------------|
| **Review Modes** | Uncommitted/branch/commit/PR/folder modes | Fractal Review Loops (D1-D50) |
| **Review Guidelines** | Custom `REVIEW_GUIDELINES.md` per project | AGENTS.md rule loading |
| **Priority + Verdict** | Findings with priority levels and clear verdict | Rev-Officer D1-D51 dimensions |
| **Agent vs Human Separation** | Feedback split: agent-facing vs human-callouts | Split attention in GWT |
| **Session Lifecycle** | `/review` start → work → `/end-review` finish | SEAL pipeline phases |

### Fusion Opportunities

1. **Review Guidelines → Rev-Officer Integration**: `pi-review`'s `REVIEW_GUIDELINES.md` pattern maps to NeoTrix's `rev-officer-agent.md` (D1-D51). The custom guidelines mechanism should be made composable — load dimension-specific rules dynamically based on task type.

2. **Priority + Verdict System**: The review output format (prioritized findings with verdicts) aligns with Rev-Officer's evidence-first methodology. Adopt the `pi-review` output schema for NeoTrix audit results.

3. **Review Session Lifecycle**: The `/review` → work → `/end-review` lifecycle maps to SEAL's phase progression. Add explicit review checkpoints between SEAL phases.

4. **Agent/Human Feedback Split**: The pattern of separating feedback for the agent from human-facing findings maps to GWT attention routing — agent-facing insights stay internal, human-facing findings get surfaced.

### Implementation Changes

- `rev-officer-agent`: Adopt `pi-review` output schema (priority + verdict + actionable follow-ups)
- `nt_meta::coordinator`: Add review session lifecycle (start → findings → end-review)
- GWT: Split attention channels for agent-internal vs human-facing findings

---

## 6. GitHub Topics: game-engine

**URL**: https://github.com/topics/game-engine
**9,482 public repositories** | Top engines by stars

### Top Repositories Analyzed

| Engine | Stars | Language | Key Pattern |
|--------|-------|----------|-------------|
| **Godot** | 117k | C++ | Open-source 2D/3D, scene system |
| **ImGui** | 76.2k | C++ | Immediate-mode GUI, zero dependencies |
| **Bevy** | 48.2k | Rust | Data-driven ECS, Rust-native |
| **Raylib** | 34.7k | C | Simple API, WASM support |
| **GDevelop** | 26.5k | JS | No-code 2D/3D/multiplayer |
| **Babylon.js** | 26.1k | TS | Web-native 3D/WebGL/WebGPU |
| **PlayCanvas** | 16.7k | JS | WebGL/WebGPU/WebXR runtime |
| **Ebiten** | 13.5k | Go | Dead simple 2D |
| **EnTT** | 13.1k | C++ | ECS framework |

### Key Architectural Patterns

| Pattern | Description | NeoTrix Mapping |
|---------|-------------|-----------------|
| **ECS (Entity-Component-System)** | Data-oriented architecture: entities as IDs, components as data, systems as behavior | NT-* domain modules as components, SEAL phases as systems |
| **Immediate-Mode GUI** | Zero-state UI: redraw every frame from current data | GWT attention recalculation every cycle |
| **Scene Graph** | Hierarchical transform/state propagation | ConsciousnessTree branch hierarchy |
| **Plugin Architecture** | Godot addons, Bevy plugins, ImGui backends | Skill nodes / Rune Socketing |
| **Data-Driven Design** | Configuration as data, not code | SelfModel configuration |
| **WASM Runtime** | Cross-platform via WebAssembly | NT-IO WASM target |

### Fusion Opportunities

1. **Bevy ECS → NeoTrix Module System**: Bevy's ECS (Entity-Component-System) is isomorphic to NeoTrix's domain-module architecture. Each NT-* domain is an entity, capabilities are components, SEAL phases are systems. Port Bevy's query pattern for cross-domain capability queries.

2. **ImGui Immediate Mode → GWT Recalculation**: ImGui redraws every frame from current state — no retained state. GWT could adopt this pattern: recalculate attention routing every growth cycle from current module health, rather than maintaining persistent attention state.

3. **EnTT ECS Framework**: EnTT's header-only, zero-dependency ECS could be adapted as the underlying data structure for NeoTrix's KB graph. The `registry.view<Components...>()` query pattern is exactly what NT-MEMORY needs for cross-namespace graph queries.

4. **WASM Runtime**: Bevy and Babylon.js both compile to WASM. NeoTrix's NT-IO could target WASM for browser-based deployment, enabling the Tauri desktop app to run full NeoTrix reasoning in-browser.

### Implementation Changes

- `nt_core`: Study Bevy ECS for module-component-system architecture
- `gwt`: Consider ImGui immediate-mode pattern for attention recalculation
- `nt_memory`: Evaluate EnTT-style ECS for KB graph data structure
- `nt_io`: Add WASM compilation target (cargo wasm-pack)

---

## Cross-Cutting Synthesis

### Pattern Matrix

| Pattern | Sources | NeoTrix Target | Priority |
|---------|---------|----------------|----------|
| **Prompt-as-Variable / Mutable Context** | prime-agent | SelfModel extension | P0 |
| **Continual Harness / Refinement Loop** | prime-agent, pi-review | NT-MIND skill refinement | P0 |
| **ECS Module Architecture** | Bevy, EnTT | NT-* domain modules | P1 |
| **Ordered Rewrite Chains** | HuntProxy | Egress Privacy Guard | P1 |
| **Connectome Graph Operations** | awesome-fly | KB graph traversal | P1 |
| **Immediate-Mode Attention** | ImGui, Godot | GWT recalculation | P2 |
| **Agent-to-Agent Discovery** | prime-agent | NT-NEXUS memory weaving | P2 |
| **From-Scratch Pedagogy** | rasbt | Skill documentation | P3 |

### Fusion Targets (Ranked)

1. **NT-MIND + prime-agent Continual Harness**: Refinement snapshots with rollback
2. **NT-SHIELD + HuntProxy Request Rules**: Ordered trust-tier rewrite chain
3. **NT-CORE + Bevy ECS**: Data-oriented module-component-system architecture
4. **NT-MEMORY + awesome-fly Connectome Libraries**: Graph traversal from neuroscience tools
5. **NT-ACT + mini-coding-agent**: Minimal, readable tool loop
6. **GWT + ImGui Immediate Mode**: Stateless attention recalculation
7. **NT-IO + prime-agent Daemon**: Background session lifecycle

### Action Items

| # | Action | Domain | Effort |
|---|--------|--------|--------|
| 1 | Design `RefinementSnapshot` type with rollback | NT-MIND | 3d |
| 2 | Port HuntProxy ordered-rewrite to Egress Guard | NT-SHIELD | 2d |
| 3 | Prototype ECS query pattern for KB graph | NT-MEMORY | 5d |
| 4 | Study Bevy plugin architecture for skill loading | NT-ACT | 2d |
| 5 | Add WASM compilation target | NT-IO | 3d |
| 6 | Design agent-to-agent discovery protocol | NT-NEXUS | 4d |
