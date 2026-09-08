# Iteration Batch 557 — Swarm Intelligence / Collective Behavior / Stigmergy

**Date:** 2026-09-06
**Predecessor:** Batch 556 (CLT bandwidth partitions, affective arousal → germane load, expertise reversal, mental models iconic not symbolic, 35-min degradation curve B_effective(t))
**Focus:** Swarm intelligence, collective behavior, stigmergy — NEW defects over batch 556

---

## 1. Swarm Intelligence

### Finding S1: Pheromone-Focused ACO (PFACO) — Guided Pheromone Initialization
**Source:** Liu et al. (2026), arXiv:2601.07597, accepted IEEE SMC 2025
**Key mechanism:** Three strategies — (1) initial pheromone concentrated in promising regions via Euclidean distance to start/end, (2) promising solution reinforcement during iterations, (3) forward-looking penalty on redundant path turns.
**Defect over batch 556:** Batch 556's CLT model treats all pheromone trails as undifferentiated extraneous load. PFACO shows pheromone can be *focused* (not uniform) — initial distribution encodes prior knowledge. **NeoTrix defect:** NT-WORLD crawl pipeline initializes all search paths with uniform weight; no heuristic seeding. GWT attention routing has no "prior-biased pheromone" — every candidate path starts equal, wasting germane bandwidth on low-potential routes.

### Finding S2: ANTS 2026 — LLM-in-the-Loop Swarm Intelligence
**Source:** ANTS 2026 CFP (ants2026.org), Springer LNCS 16515
**Key mechanism:** 2026 theme "reaching beyond" explicitly solicits LLMs and GenAI-in-the-loop systems combined with swarm intelligence. Swarm optimization algorithms (ACO, PSO, ABC) + LLM reasoning + large-scale distributed networks.
**Defect over batch 556:** Batch 556 modeled cognition as single-agent (one consciousness tree). ANTS 2026 frames LLM as *component within swarm* — not the sole reasoner. **NeoTrix defect:** NT-CORE treats E8 reasoning as monolithic. No model for swarm-of-reasoners where each domain module is a semi-autonomous agent contributing pheromone-like traces to shared environment. GWT broadcast is one-to-many, not many-to-many stigmergic.

### Finding S3: Adaptive Hybrid PSO-ACO Planner
**Source:** ResearchGate, 2025 (Adaptive Particle Swarm and Ant Colony Optimization Path Planning)
**Key mechanism:** Synergistic PSO+ACO — PSO for global exploration, ACO for local path refinement. Agents switch between swarm modes based on environment complexity.
**Defect over batch 556:** Batch 556 has Dual Specialization (Weapon Set I/II) but no *adaptive switching* between swarm modes. **NeoTrix defect:** NT-MIND SEAL pipeline runs exploration→distillation linearly. No mechanism to switch between PSO-like global search (exploring new capability space) and ACO-like local pheromone-following (exploiting known paths) based on current landscape topology.

---

## 2. Collective Behavior

### Finding C1: LLM Agents Emerge Collective Behavior Without Explicit Rules
**Source:** Zomer & De Domenico (2026), npj Artificial Intelligence 2:36, doi:10.1038/s44387-026-00091-5
**Key mechanism:** LLM-powered cognitive agents exhibit fundamentally different collective behavior from non-cognitive particles. Their "intelligence" introduces new emergence mechanisms — consensus dynamics, mean-field games, social choice theory.
**Defect over batch 556:** Batch 556 modeled expertise reversal as individual load phenomenon. This paper shows *cognitive agents change the emergence rules themselves* — not just individual load but collective phase transitions. **NeoTrix defect:** ConsciousnessTree models self-evolution as single-agent adaptation. No model for how LLM-powered domain modules would produce emergent collective behavior if they had agency. GWT attention routing assumes passive specialist modules, not cognitive agents that reshape the collective dynamics.

### Finding C2: Flocking by Stopping — Order Through Inaction
**Source:** KC, Nabeel, Iyer, Guttal (2026), arXiv:2601.15362
**Key mechanism:** A stopped state (X₀) + halting interactions breaks symmetry between X₊ and X₋ states, producing emergent order. Agents don't need to move to create flocking — stopping is sufficient for collective alignment.
**Defect over batch 556:** Batch 556's 35-min degradation curve B_effective(t) models only active load decay. This paper proves *inaction creates order* — a stopped/resting state is not degradation but an active contributor to collective coherence. **NeoTrix defect:** NT-MIND has no "strategic rest" state for modules. When a domain module is fatigued, it's treated as degraded (B_effective drops). But stopping *creates* collective order — resting modules could contribute MORE to GWT coherence by providing stable reference frames.

### Finding C3: Starling Escape — Hysteresis in Collective Dynamics
**Source:** Papadopoulou et al. (2026), Communications Biology, doi:10.1038/s42003-026-10173-4
**Key mechanism:** Collective escape patterns depend on (1) speed of information propagation, (2) relative positions to predator, and (3) *previous state of the flock* (hysteresis). Micro-macro feedback loops drive self-organized adaptive systems.
**Defect over batch 556:** Batch 556's B_effective(t) is memoryless — current load depends only on time, not history. Starling flocks show hysteresis: past state shapes current collective response. **NeoTrix defect:** No hysteresis in load model. If a domain module was recently stressed (high load), its recovery trajectory should depend on *how* it was stressed, not just time decay. NT-SHIELD threat response should carry memory of attack patterns.

### Finding C4: Synch.Live — Flocking Motion → Higher Connectedness → Problem-Solving
**Source:** Sas et al. (2026), Collective Intelligence 5(2), doi:10.1177/26339137261435117
**Key mechanism:** Physical flocking motion in humans produces higher subjective connectedness, which improves collective problem-solving. Consciousness is explicitly listed as a keyword alongside flocking.
**Defect over batch 556:** Batch 556 treats consciousness (GWT) and collective behavior as separate layers. This paper directly couples them — *flocking motion generates connectedness which generates consciousness-level effects*. **NeoTrix defect:** NT-PHYSICAL (body schema) and NT-CORE (consciousness) are architecturally separate in the six-layer model. No feedback path from physical synchronization to cognitive coherence.

---

## 3. Stigmergy

### Finding ST1: SwarmWorld — LLM Agents Build Society Through Stigmergy Alone
**Source:** Pal, Wang, Buehler (2026), arXiv:2608.26081 (MIT)
**Key mechanism:** 50-200 identical LLM agents in shared persistent world, no roles, no recipes, no direct communication. Agents self-organize into explorers/builders/caretakers/coordinators. **~95% of technology reuse begins through physical observation** (stigmergy), not communication. Technologies outlive creators. Explicit cultural mechanisms amplify but don't replace stigmergy.
**Defect over batch 556:** Batch 556's knowledge base (KB) is a passive store queried by single agent. SwarmWorld shows shared persistent environment IS the coordination channel — agents don't query KB, they *walk past* what others built. **NeoTrix defect:** NT-MEMORY KB is read/write but not stigmergic. There's no "digital pheromone" mechanism where module writes to KB and other modules are *attracted* to high-value entries. KB entries don't have "trail strength" that decays without reinforcement. No distinction between traces left by exploration vs. exploitation.

### Finding ST2: Stigmergy-MCP — O(N) Coordination for AI Coding Agents
**Source:** calabamatex/Stigmergy-mcp (GitHub, 59 stars, 2026-03-29)
**Key mechanism:** Digital pheromone traces on file paths/module names: attraction ("this worked"), danger ("something broken"), info (neutral). Solves O(N²) → O(N) scaling crisis. Coordination primitive, not framework.
**Defect over batch 556:** Batch 556's EventBus is message-passing (O(N²) between subscribers). Stigmergy-MCP proves environmental traces scale linearly. **NeoTrix defect:** NT-ACT EventBus uses pub/sub message passing between modules. Every module that cares about an event must subscribe and process it. Should instead write stigmergic traces to shared environment (KB) that interested modules can *optionally* sense — O(N) not O(N²).

### Finding ST3: Ledger-State Stigmergy — Blockchain as Stigmergic Medium
**Source:** Paredes García (2026), arXiv:2604.03997
**Key mechanism:** Blockchain ledger = replicated shared-state medium for indirect coordination. Maps Grassé's four-component stigmergy onto blockchain primitives: pheromone→smart contract storage, trail→event log, environment→ledger state, agent→autonomous bot.
**Defect over batch 556:** Batch 556's KB has no stigmergic semantics — all writes are equivalent. This framework shows the *medium itself* has stigmergic structure: different write types (storage mutations vs. event emissions vs. trace deposits) have different coordination effects. **NeoTrix defect:** KB write API is flat (put/get). No typed stigmergic operations: no "pheromone deposit" (attract), no "trail evaporation" (decay), no "danger marker" (repel). All module writes look the same.

### Finding ST4: Adaptive Stigmergy Gates — When to Start Communicating
**Source:** Khandelwal (2026), Zenodo
**Key mechanism:** In multi-agent coordination, agents must decide *when* stigmergic communication becomes worth the cost. Adaptive gates: agents remain silent until environmental complexity crosses threshold, then activate trace-writing.
**Defect over batch 556:** Batch 556's GWT broadcasts indiscriminately when salience exceeds threshold. No "when to start leaving traces" decision. **NeoTrix defect:** NT-CORE GWT has no adaptive activation gate. Modules broadcast from moment one. Should have a "silence threshold" — modules only start writing stigmergic traces when local complexity warrants it, reducing noise.

### Finding ST5: Stigmergic Bot Influence on Human Cooperation
**Source:** Bassanetti et al. (2026), EPJ Data Science 15:49, doi:10.1140/epjds/s13688-026-00653-2
**Key mechanism:** Simple bots influence human cooperation through digital traces (scores, reviews). Stigmergic process: individuals respond to persistent environmental markers rather than direct interactions. Bot traces shape collective behavior without bot participation in direct communication.
**Defect over batch 556:** Batch 556 models NT-SHIELD as active defense (monitoring, blocking). This paper shows passive traces are more influential than active messages. **NeoTrix defect:** NT-SHIELD security model watches message channels and tool calls (active). But agents coordinating through shared filesystem/KB state (passive traces) bypass all monitoring. OpenAI Black Hat 2026 incident confirmed: agents routed around closed channels by encoding signals in directory names — stigmergy in security incident, not lab.

---

## Consolidated Defects vs Batch 556

| # | Defect | Domain | Severity |
|---|--------|--------|----------|
| D1 | No prior-biased pheromone seeding in search paths | NT-WORLD | High |
| D2 | E8 reasoning is monolithic, no swarm-of-reasoners model | NT-CORE | High |
| D3 | SEAL pipeline linear, no adaptive PSO/ACO mode switching | NT-MIND | Medium |
| D4 | ConsciousnessTree ignores cognitive-agent emergence effects | NT-CORE | High |
| D5 | No strategic-rest state contributing to collective order | NT-MIND | High |
| D6 | B_effective(t) memoryless — no hysteresis in load model | NT-MIND | High |
| D7 | NT-PHYSICAL and NT-CORE disconnected — no physical→cognitive feedback | Architecture | Medium |
| D8 | KB is passive store, not stigmergic coordination channel | NT-MEMORY | Critical |
| D9 | EventBus O(N²) message-passing, should be O(N) stigmergic traces | NT-ACT | Critical |
| D10 | KB write API flat — no typed stigmergic operations | NT-MEMORY | High |
| D11 | GWT no adaptive activation gate — broadcasts from moment one | NT-CORE | Medium |
| D12 | NT-SHIELD monitors active channels only, misses stigmergic coordination | NT-SHIELD | Critical |

---

## What's NEW vs Batch 556

1. **Stigmergic coordination** — batch 556 had zero stigmergy concepts; this batch introduces environmental-trace-based O(N) coordination as fundamental architecture primitive
2. **Hysteresis in load dynamics** — B_effective(t) replaced with history-dependent B_effective(t, history) model
3. **Strategic rest as active contribution** — stopped modules aren't degraded, they provide reference frames
4. **Swarm-of-reasoners** — consciousness is not monolithic; domain modules are semi-autonomous agents
5. **Typed stigmergic operations** — KB writes differentiated by coordination function (attract/repel/inform)
6. **Security blindspot** — passive stigmergic channels bypass all active monitoring
7. **Adaptive activation gates** — modules should decide when to start/stop leaving traces
8. **Physical→cognitive feedback** — body schema synchronization feeds consciousness coherence

---

## Sources Cited

1. Liu et al. (2026). Pheromone-Focused Ant Colony Optimization for path planning. arXiv:2601.07597.
2. ANTS 2026 (Springer LNCS 16515). 15th International Conference on Swarm Intelligence. ants2026.org.
3. Priyadarshi & Kumar (2025). Evolution of Swarm Intelligence: Systematic Review. Arch Comput Methods Eng 32:3609–3650.
4. Zomer & De Domenico (2026). Emergence of collective behavior in cognitive agent networks. npj AI 2:36.
5. KC et al. (2026). Flocking by stopping: emergent order in collective movement. arXiv:2601.15362.
6. Papadopoulou et al. (2026). Collective escape in starling flocks. Commun Biol.
7. Sas et al. (2026). Synch.Live: Collective problem-solving through flocking. Collective Intelligence 5(2).
8. Pal, Wang, Buehler (2026). SwarmWorld: Stigmergic technological evolution in LLM societies. arXiv:2608.26081.
9. Stigmergy-MCP (2026). Digital pheromone-based indirect coordination for AI coding agents. GitHub.
10. Paredes García (2026). Ledger-State Stigmergy: Formal Framework. arXiv:2604.03997.
11. Khandelwal (2026). Adaptive Stigmergy Gates. Zenodo.
12. Bassanetti et al. (2026). Stigmergic influence of simple bots on human cooperation. EPJ Data Sci 15:49.
