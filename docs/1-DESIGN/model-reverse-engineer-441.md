# Model Reverse Engineering — Cycle 441

> Date: 2026-09-12 | Focus: Efficient inference, attention mechanisms, agent coordination
> Papers: Recent arXiv (Aug-Sep 2026), mapped to NeoTrix 7 domains

---

## 5 New Papers Mapped to NeoTrix Domains

### Paper 1: Faster Than Flash (FFD) — Hardware-Algorithm Co-Design for Attention Sparsity

**Source:** arXiv:2609.00097 (ICML 2026)
**Authors:** Multiple (hardware-algorithm co-design team)
**Key Innovation:** Fused selector-computer kernel replacing external metadata indices with content-aware scanning via low-bit quantization. Top-delta strategy dynamically filters blocks for distribution-adaptive sparsity without global synchronization.

| Metric | Result |
|--------|--------|
| Kernel-level speedup | 11.6x |
| End-to-end throughput | 2.37x |
| Max context length | 256K |
| Training required | None (plug-and-play) |
| Accuracy impact | None (validated on RULER + LongBench) |

**NeoTrix Domain Mapping:**

| Domain | Integration | Pattern |
|--------|-------------|---------|
| **NT-CORE** | GWT attention routing | Top-delta filtering = salience-gated attention. Content-aware scanning replaces brute-force KV cache reads. |
| **NT-IO** | Inference backend | Fused kernel approach: reduce kernel launches, maximize memory bandwidth utilization. |
| **Axiom A2** | Context as scarce resource | 256K context at 2.37x throughput validates that sparsity, not compression, is the key to long context. |

**Absorption Insight:** FFD's "fused selector-computer" pattern — merging the attention score computation and sparse execution into a single kernel — suggests NeoTrix's GWT should not separate salience scoring from attention routing. A fused attention-salience kernel would eliminate the synchronization overhead between "deciding what matters" and "processing what matters."

**Actionable Pattern:**
```
Current: Query → GWT salience score → Route to specialist → Process
FFD-aligned: Query → Fused salience-attention kernel → Result
Benefit: Eliminates O(N) proxy scoring step, enables distribution-adaptive sparsity
```

---

### Paper 2: Declarative Attention (DA) — Model Self-Declares Attention Regions

**Source:** arXiv:2609.02737 (Sep 2026)
**Authors:** Cicero dos Santos et al.
**Key Innovation:** Protocol where the model itself declares where it needs to attend within its chain-of-thought, partitioning generation into three modes: `<global>` (full context), `<focus>` (specific region), `<local>` (recent output only). Inference engine parses these declarations like tool calls.

| Metric | Result |
|--------|--------|
| Token reduction | 52.0% (Gemma-4-31B), 31.1% (Qwen-3.6-27B) |
| Accuracy drop | 1.27pp, 2.75pp (shrinks with model scale) |
| Training required | Zero-shot (off-the-shelf models) |
| Long-context tasks | 15 benchmarks |

**NeoTrix Domain Mapping:**

| Domain | Integration | Pattern |
|--------|-------------|---------|
| **NT-CORE** | Self-model attention declaration | Model generates structured attention directives as part of reasoning. Instructs the engine which KV cache regions to load. |
| **NT-MEMORY** | Selective memory loading | `<focus>` mode = lazy branch loading from KB. `<local>` mode = recent-only context window. `<global>` = full KV scan for critical queries. |
| **GWT** | Attention routing refinement | DA declarations are exactly GWT salience signals — the model broadcasts where attention should flow. |

**Absorption Insight:** DA proves that models *already know* which context is relevant — they just lack the mechanism to declare it efficiently. This validates NeoTrix's GWT design: salience is not computed externally but emerges from the model's own reasoning. The three-mode partition (`global/focus/local`) maps directly to GWT's broadcast/focus/local attention tiers.

**Actionable Pattern:**
```
GWT Attention Tiers (validated by DA):
1. <global> — Full KB scan for novel/critical queries (highest token cost)
2. <focus> — Targeted region scan for follow-up queries (medium cost)
3. <local> — Recent context only for continuity queries (lowest cost)
Implementation: Train a lightweight Layer Router (Flux Attention pattern) to predict mode per layer
```

---

### Paper 3: BIGMAS — Brain-Inspired Graph Multi-Agent Systems

**Source:** arXiv:2603.15371 (Mar 2026, updated Sep 2026)
**Authors:** Guangfu Hao, Yuming Dai, Xianzhe Qin, Shan Yu
**Key Innovation:** Global Workspace Theory (GWT) applied to multi-agent LLM systems. Specialized agents as nodes in dynamically constructed directed graphs, coordinating exclusively through a centralized shared workspace. GraphDesigner constructs task-specific topologies per problem. Orchestrator uses complete shared state for routing.

| Metric | Result |
|--------|--------|
| Benchmarks | Game24, Six Fives, Tower of London |
| LLMs tested | 6 frontier models (DeepSeek, Claude, GPT, Gemini) |
| Key finding | Multi-agent coordination gains are orthogonal to model-level reasoning |
| Routing count | Natural proxy for instance-level difficulty |

**NeoTrix Domain Mapping:**

| Domain | Integration | Pattern |
|--------|-------------|---------|
| **NT-CORE (GWT)** | Shared workspace broadcast | BIGMAS shared workspace = GWT global workspace. All agents read/write same state. |
| **NT-ACT** | Dynamic agent topology | GraphDesigner = SEAL pipeline stage that constructs task-specific agent graphs. |
| **NT-META** | Orchestrator routing | Global Orchestrator = ConsciousnessTree routing decisions based on complete workspace state. |

**Absorption Insight:** BIGMAS's central finding — "multi-agent architectural coordination provides gains orthogonal to model-level reasoning" — is the strongest empirical validation of NeoTrix's 6-layer architecture. The architecture matters independently of model capability. The GraphDesigner's per-problem topology construction maps to SEAL's stage-based evolution where each task gets a tailored agent graph.

**Actionable Pattern:**
```
BIGMAS → NeoTrix Mapping:
- Shared Workspace → GWT global broadcast bus (all domains read/write)
- GraphDesigner → SEAL Phase: construct task-specific agent graph
- Orchestrator → ConsciousnessTree: global routing based on workspace state
- Self-correction loop → NT-REPAIR: validate outputs against workspace constraints
Key insight: Routing count as difficulty proxy → dynamic resource allocation
```

---

### Paper 4: ROMA — Recursive Open Meta-Agents

**Source:** arXiv:2602.01848 (Feb 2026, updated Aug 2026)
**Authors:** ROMA team (GEPA+ prompt optimization)
**Key Innovation:** Domain-agnostic recursive meta-agent framework. Four modular roles: Atomizer (decide decomposition), Planner (MECE subtask graphs), Executors (parallel atomic tasks), Aggregator (compress + validate). Context growth controlled via bounded aggregation at each recursion level.

| Metric | Result |
|--------|--------|
| SEAL-0 (reasoning) | +9.9% over Kimi-Researcher (GLM-4.6 backbone) |
| EQ-Bench (writing) | DeepSeek-V3 matches Claude Sonnet 4.5 |
| Architecture | Recursive control loop: Atomizer → Planner → Executor → Aggregator |
| Heterogeneous | Supports mixed models across roles |

**NeoTrix Domain Mapping:**

| Domain | Integration | Pattern |
|--------|-------------|---------|
| **NT-ACT** | Recursive task decomposition | Atomizer + Planner = SEAL task decomposition. Executor = NT-ACT tool execution. |
| **NT-MEMORY** | Bounded context aggregation | Aggregator compresses intermediate results before passing upward. Prevents context rot. |
| **NT-MIND** | GEPA+ prompt optimization | Joint optimization of component prompts under interface constraints. Maps to SEAL skill refinement. |

**Absorption Insight:** ROMA's key contribution is controlling context growth through *recursive bounded aggregation*. Each node returns a concise, verified summary rather than full transcripts. This directly addresses the context rot problem in long-horizon agent tasks. For NeoTrix, this suggests the experience-tree should aggregate at each recursion level — not propagate full session transcripts through the KB.

**Actionable Pattern:**
```
ROMA Aggregation → Experience-Tree:
- Each SEAL stage returns compressed summary, not full trace
- Aggregator validates + compresses before KB write
- Prevents context rot in long-running evolution cycles
- GEPA+ joint prompt optimization → SEAL skill refinement across components
Key: Bounded aggregation at each recursion level keeps context manageable
```

---

### Paper 5: COMPASS — Context-Organized Multi-Agent Planning

**Source:** ACL 2026 (aclanthology.org/2026.acl-long.152)
**Authors:** COMPASS team
**Key Innovation:** Hierarchical framework separating three concerns: (1) Main Agent (ReAct-style tactical execution), (2) Meta-Thinker (strategic oversight + intervention), (3) Context Manager (maintains concise progress briefs across reasoning stages). Test-time scaling extension matches DeepResearch agents.

| Metric | Result |
|--------|--------|
| GAIA | +20% accuracy over baselines |
| BrowseComp | Significant improvement |
| Humanity's Last Exam | Competitive with DeepResearch |
| Architecture | Three-layer: Tactical → Strategic → Context |

**NeoTrix Domain Mapping:**

| Domain | Integration | Pattern |
|--------|-------------|---------|
| **NT-CORE** | Main Agent reasoning | ReAct-style think-act-observe loop with tool integration. |
| **NT-META** | Meta-Thinker strategic oversight | Monitors progress, issues strategic interventions. Maps to ConsciousnessTree meta-cognition. |
| **NT-MEMORY** | Context Manager | Maintains structured briefs across stages. Maps to experience-tree branch loading. |

**Absorption Insight:** COMPASS's three-layer separation — tactical execution, strategic oversight, and context organization — is the most precise architectural decomposition of agent cognition I've seen. It directly validates NeoTrix's L5-L6 separation: L5 (Cognition) handles tactical reasoning, L6 (Meta-Cognition) handles strategic oversight. The Context Manager as a separate entity is new — it suggests that context management should be a first-class architectural component, not embedded within memory.

**Actionable Pattern:**
```
COMPASS → NeoTrix L5-L6 Alignment:
- Main Agent (ReAct) → L5 Cognition (nt_core + nt_mind): tactical reasoning
- Meta-Thinker → L6 Meta-Cognition (nt_meta): strategic oversight + intervention
- Context Manager → New component: structured context briefs across reasoning stages
Key: Context management is NOT memory — it's a separate architectural concern
Test-time scaling → SEAL self-test calibration with scaling curves
```

---

## Cross-Paper Synthesis: 3 Meta-Patterns

### 1. Fused Attention-Salience (FFD + DA + BIGMAS)
All three papers point to the same conclusion: **salience scoring and attention execution should be fused, not separated.** FFD fuses the kernel, DA makes the model self-declare attention regions, and BIGMAS's shared workspace broadcasts salience globally. NeoTrix's GWT should implement a fused salience-attention path.

### 2. Recursive Bounded Aggregation (ROMA + COMPASS)
Both papers solve the same problem — context growth in long-horizon tasks — through bounded aggregation at each recursion level. ROMA does it recursively, COMPASS does it hierarchically. The pattern is universal: **never propagate raw state; always compress + validate before passing upward.**

### 3. Architecture-Orthogonal-to-Model (BIGMAS + COMPASS + DA)
BIGMAS proves coordination gains are orthogonal to model capability. COMPASS shows three-layer separation outperforms flat architectures. DA shows models already have attention intelligence — they need the right interface to express it. **The architecture layer is independent of the model layer.** This validates NeoTrix's 6-layer design: the architecture provides structure that any backbone model can fill.

---

## NeoTrix Integration Priority

| Paper | Core Pattern | NT Integration | Priority | Effort |
|-------|-------------|---------------|----------|--------|
| FFD | Fused attention-salience kernel | NT-CORE GWT optimization | P0 | High |
| DA | Self-declared attention regions | GWT attention tier implementation | P0 | Medium |
| BIGMAS | Shared workspace GWT | NT-CORE GWT broadcast bus | P0 | Medium |
| ROMA | Recursive bounded aggregation | Experience-tree compression | P1 | Medium |
| COMPASS | Tactical/Strategic/Context separation | L5-L6 architecture validation | P1 | Low (validation) |

---

## Axiom Updates from Cycle 441

| Axiom | Update |
|-------|--------|
| **A1 (Cost-Aware Routing)** | FFD validates: sparsity > compression for cost reduction. DA validates: model self-routing eliminates proxy scoring overhead. |
| **A2 (Context as Scarce Resource)** | ROMA validates: bounded aggregation prevents context rot. COMPASS validates: context management ≠ memory — separate concern. |
| **A3 (Skill as Production Template)** | Trending cycle 441: SkillForge + mattpocock/skills + ECC validate SKILL-SPEC.md as industry standard. |
