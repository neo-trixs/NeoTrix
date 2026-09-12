# Model Reverse Engineering — Cycle 375

**Date**: 2026-09-12
**Focus**: Recent papers on efficient inference, attention, agent coordination
**Sources**: arXiv (2026), ACL 2026, ICLR 2026, GitHub trending repos

---

## Paper 1: Flux Attention — Context-Aware Hybrid Attention

**Paper**: [arXiv:2604.07394](https://arxiv.org/abs/2604.07394) (Apr 2026)
**Venue**: Preprint (under review)
**Authors**: Qiu et al.

### Core Idea
Standard attention has quadratic complexity — a severe bottleneck for long-context LLMs. Existing hybrid attention (Full Attention + Sparse Attention) uses static allocation ratios that fail to accommodate variable task demands. Flux Attention introduces a **layer-level dynamic router** that adaptively routes each layer to FA or SA based on input context.

### Key Mechanism
- **Layer Router**: Lightweight module inserted into frozen pretrained LLMs
- **Layer-wise routing**: Each layer independently chooses FA or SA based on input
- **Contiguous memory access**: Preserves hardware-friendly memory patterns
- **Parameter-efficient**: Only 12 hours training on 8×A800 GPUs

### Results
- **2.8× prefill speedup**, **2.0× decode speedup** vs baseline
- Superior performance-speed tradeoff on long-context and math reasoning benchmarks
- No accuracy degradation on standard benchmarks

### NeoTrix Mapping

| Component | NeoTrix Domain | Pattern |
|-----------|---------------|---------|
| Layer Router | NT-CORE (GWT) | Dynamic attention routing per layer based on task salience |
| FA/SA Selection | NT-IO (model routing) | Cost-aware model component selection |
| Memory Access | NT-MEMORY (KV cache) | Hardware-friendly memory access patterns |

**Key Insight for NeoTrix**: The GWT salience router could adopt layer-wise dynamic routing — not just which model to use, but which attention mechanism per layer based on input complexity. This is a finer-grained version of Cost-Aware Routing (Axiom A1).

**Integration Point**: Extend `nt_core_self::AttentionManager` to support per-layer attention mode selection based on input context salience. The Layer Router pattern maps directly to GWT's selective attention mechanism.

---

## Paper 2: Explicit Trait Inference (ETI) for Multi-Agent Coordination

**Paper**: [arXiv:2604.19278](https://arxiv.org/abs/2604.19278) (Apr 2026)
**Venue**: ACL 2026 Main Conference
**Authors**: Abdurahman et al.

### Core Idea
LLM-based multi-agent systems suffer from coordination failures: goal drift, error cascades, misaligned behaviors. ETI enables agents to **infer and track partner characteristics** along two psychological dimensions — warmth (trust) and competence (skill) — from interaction histories to guide coordination decisions.

### Key Mechanism
- **Trait Inference**: Agents build profiles of partner warmth and competence from interactions
- **Profile-Guided Decisions**: Coordination choices (who to delegate to, when to trust) based on inferred traits
- **Lightweight**: No additional training required — inference-time only
- **Two dimensions**: Warmth (trustworthiness) and Competence (skill level)

### Results
- **45-77% payoff loss reduction** in economic games
- **3-29% performance improvement** on MultiAgentBench
- Trait profiles predict agent actions — the inference is meaningful, not decorative

### NeoTrix Mapping

| Component | NeoTrix Domain | Pattern |
|-----------|---------------|---------|
| Trait Inference | NT-CORE (SelfModel) | Dynamic partner modeling |
| Warmth Dimension | NT-FEEL (social emotion) | Trust tracking across agents |
| Competence Dimension | NT-MIND (capability assessment) | Skill profiling for delegation |
| Profile-Guided Decisions | NT-ACT (orchestration) | Evidence-based task routing |

**Key Insight for NeoTrix**: The SelfModel could incorporate trait inference for multi-agent coordination. When NeoTrix delegates to sub-agents or external tools, tracking their "competence" (success rate) and "warmth" (reliability) enables evidence-based routing — not just random or round-robin assignment.

**Integration Point**: Extend `nt_core_self::SelfModel` with partner trait tracking. The AttentionManager's weapon set selection could incorporate inferred partner traits — "this tool has high competence for this task type" vs "this tool is unreliable for this pattern."

---

## Paper 3: Latent Action Reparameterization (LAR) for Efficient Agent Inference

**Paper**: [arXiv:2605.18597](https://arxiv.org/abs/2605.18597) (May 2026)
**Venue**: Preprint
**Authors**: Huang et al.

### Core Idea
LLM agents use long sequences of low-level textual actions, creating large decision horizons and high inference cost. LAR learns a **compact latent action space** where each latent action corresponds to a multi-step semantic behavior. Agents plan and execute over abstract action representations.

### Key Mechanism
- **Latent Action Space**: Learned from agent trajectories, not hand-crafted
- **Semantic Compression**: Each latent action = multi-step behavior macro
- **Shorter Horizon**: Decision making over compressed action space
- **Direct Integration**: Latent actions integrated into model — no external controller

### Results
- **Significant reduction in action tokens** (fewer tokens per task)
- **Reduced wall-clock inference time**
- **Maintained or improved task success rates**
- Complementary to model architecture and hardware advances

### NeoTrix Mapping

| Component | NeoTrix Domain | Pattern |
|-----------|---------------|---------|
| Latent Action Space | NT-ACT (tool abstraction) | Learned tool composition macros |
| Semantic Compression | NT-MIND (skill crystallization) | Behavior → skill compression |
| Shorter Horizon | NT-CORE (reasoning) | Efficient reasoning via abstraction |

**Key Insight for NeoTrix**: The SEAL pipeline's skill crystallization could incorporate LAR-style latent action learning. Instead of manually defining skill boundaries, the system learns "action atoms" — reusable behavioral primitives that compress multi-step operations into single semantic units.

**Integration Point**: Extend `nt_mind_skill_engine` with latent action discovery. The Disclosure Ladder (P4 pattern) could benefit from pre-learned action primitives that reduce context needed for skill invocation.

---

## Paper 4: Active Inference as Context Acquisition

**Paper**: [arXiv:2608.19202](https://arxiv.org/abs/2608.19202) (Jun 2026)
**Venue**: Preprint
**Authors**: Dutta et al.

### Core Idea
Interactive AI agents must acquire the right context efficiently. When a user omits constraints, agents can either proceed with defaults or spend tokens on clarification. This is formulated as **active inference for context acquisition** — an inner inference step updates beliefs over latent task state, an outer decision selects next action (context, task, or stop) to minimize expected free energy under cost.

### Key Mechanism
- **Active Inference Framework**: Epistemic action (gathering info) vs pragmatic action (doing task)
- **Expected Free Energy Minimization**: Balances information gain vs token cost
- **Dynamic Programming Oracle**: Optimal question-asking strategy
- **Model-Agnostic**: Framework, not model-specific

### Results
- Optimal Question Asking (OQA) outperforms baselines on binary and multiway tasks
- Framework scales from 25 to 300 candidate answers
- Token-budget-aware clarification before generation

### NeoTrix Mapping

| Component | NeoTrix Domain | Pattern |
|-----------|---------------|---------|
| Active Inference | NT-CORE (GWT) | Epistemic vs pragmatic action selection |
| Free Energy Minimization | NT-MIND (cost-aware evolution) | Information gain per token cost |
| Context Acquisition | NT-IO (input gathering) | Optimal clarification strategy |
| Token Budget | NT-ACT (resource management) | Bounded resource allocation |

**Key Insight for NeoTrix**: The GWT attention mechanism could incorporate active inference — deciding whether to act (pragmatic) or gather more context (epistemic) based on expected information gain per token cost. This is the formal version of "should I ask a clarifying question or just proceed?"

**Integration Point**: Extend GWT salience to include an epistemic value term — the expected information gain from asking a question vs proceeding with current context. This makes the Disclosure Ladder (P4) formally optimal rather than heuristic.

---

## Paper 5: REAL — Reasoning-Enhanced Graph for Long-Term Memory

**Paper**: [arXiv:2606.10694](https://arxiv.org/abs/2606.10694) (Jun 2026)
**Venue**: Preprint
**Authors**: (Not specified in search results)

### Core Idea
LLM memory systems face two challenges: (1) memory construction that captures causal/temporal/contrastive relationships, not just facts; (2) reasoning-enhanced retrieval that handles incomplete or ambiguous queries. REAL constructs a **multi-dimensional memory graph** with confidence scores and exploration intents, then retrieves via semantic evaluator-guided hybrid beam search with counterfactual inference.

### Key Mechanism
- **Confidence Stratification**: Each fact has confidence score from linguistic cues (hedging = low, declarative = high)
- **Exploration Intent Labels**: FACT, CAUSAL, TEMPORAL, CONTRAST, EVOLUTION per fact
- **Semantic Evaluator**: Guides beam search during retrieval
- **Counterfactual Inference**: Handles incomplete evidence
- **Memory Evidence Subgraph**: Globally pruned and compact

### Results
- Handles incomplete/ambiguous queries effectively
- Confidence scores prevent unconfirmed statement retrieval
- Exploration intents enable diverse retrieval strategies

### NeoTrix Mapping

| Component | NeoTrix Domain | Pattern |
|-----------|---------------|---------|
| Confidence Scoring | NT-MEMORY (KB reliability) | Trust-weighted knowledge retrieval |
| Exploration Intent | NT-CORE (reasoning modes) | Intent-aware retrieval strategy |
| Counterfactual Inference | NT-MIND (counterfactual reasoning) | Handling incomplete evidence |
| Evidence Subgraph | NT-MEMORY (KB structure) | Compact reasoning-relevant subgraph |

**Key Insight for NeoTrix**: The KB could implement confidence scoring and exploration intent labels. Not all facts are equal — hedged statements should have lower retrieval priority. The exploration intent labels map to different retrieval strategies (temporal queries use time-ordered retrieval, causal queries use dependency traversal).

**Integration Point**: Extend KB nodes with confidence scores (computed from linguistic cues during ingestion) and exploration intent labels. The KB query engine could use these to weight retrieval and select traversal strategy.

---

## Cross-Paper Synthesis: NeoTrix Integration Matrix

### Pattern 1: Dynamic Attention Routing (Flux Attention + Active Inference)
**Combined**: GWT salience routing that dynamically selects attention mechanism per layer AND decides when to gather more context vs act. Two levels of dynamic routing: spatial (which layer) and temporal (when to ask vs act).

### Pattern 2: Trait-Aware Agent Coordination (ETI + LAR)
**Combined**: Agents with learned trait profiles (competence/warmth) AND learned action primitives. Delegation decisions use both partner traits AND action complexity to select optimal agent-action pairs.

### Pattern 3: Reasoning-Enhanced Memory (REAL + Active Inference)
**Combined**: Memory with confidence scores and exploration intents that drives epistemic action selection. High-uncertainty memories trigger information gathering; high-confidence memories enable direct action.

### Priority Integration Path

| Phase | Pattern | Target Module | Complexity |
|-------|---------|---------------|------------|
| P1 | Confidence Scoring | KB node metadata | Low |
| P1 | Exploration Intent Labels | KB query strategy | Low |
| P2 | Layer-wise Attention Routing | GWT salience | Medium |
| P2 | Trait Inference | SelfModel | Medium |
| P3 | Latent Action Learning | SEAL skill engine | High |
| P3 | Active Inference Context | GWT decision layer | High |

---

## NeoTrix Domain Impact (Papers 1-5)

| Domain | Papers | New Capability |
|--------|--------|---------------|
| NT-CORE | Flux Attention, Active Inference, ETI | Dynamic attention routing, epistemic action selection, partner trait tracking |
| NT-MEMORY | REAL, Cognee (trending) | Confidence-scored KB, exploration intents, reasoning-enhanced retrieval |
| NT-MIND | LAR, ETI | Latent action learning, competence profiling for skill routing |
| NT-ACT | LAR, ETI | Learned action primitives, trait-aware delegation |
| NT-IO | Flux Attention | Layer-level cost-aware routing |
| NT-FEEL | ETI | Warmth (trust) tracking across agent interactions |
| NT-SHIELD | REAL | Confidence-gated knowledge provenance |
