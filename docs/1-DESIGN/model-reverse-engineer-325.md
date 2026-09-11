# Model Reverse Engineering — Cycle 325

**Date**: 2026-09-11
**Focus**: Efficient attention routing, world models for agents, memory lifecycle management, multi-agent coordination, self-evolving agent architectures

---

## 5 Models/Papers Analyzed

### 1. Qwen-AgentWorld — Native Language World Model
- **Source**: arXiv:2606.24597, GitHub: QwenLM/Qwen-AgentWorld
- **Scale**: MoE 35B total / 3B active parameters, 256K context
- **Training**: 3-stage pipeline (CPT→SFT→RL) on 10M+ real interaction trajectories
- **Performance**: 58.71 overall on AgentWorldBench (beats GPT-5.4 at 58.25)

#### Key Technical Contributions
- **Native world modeling**: Environment modeling is the training objective from CPT stage onward, not a post-hoc add-on
- **7-domain unification**: MCP, Search, Terminal, SWE, Android, Web, OS — first model to cover all 7 in a single architecture
- **Fictional-world training**: Agents trained in invented, self-consistent worlds generalize to real environments
- **Controllable perturbations**: Inject targeted perturbations to expose agent weaknesses

#### NeoTrix Domain Mapping

| Domain | Integration | Priority |
|--------|-------------|----------|
| **NT-WORLD** | Core: environment simulation for perception layer. Fictional-world training validates SEAL exploration stages. 7-domain model as template for cross-domain simulation. | P0 |
| **NT-MIND** | SEAL pipeline enhancement: world-model-based self-testing before real execution. Controllable perturbation for robustness testing. | P1 |
| **NT-CORE** | GWT attention: world-model predictions could inform salience scoring — what's important depends on what would happen next. | P2 |
| **NT-MEMORY** | Experience quality: world-model can validate whether stored experiences are consistent with environmental dynamics. | P2 |
| **NT-ACT** | Action planning: world-model simulation enables look-ahead planning without real execution cost. | P1 |

#### Absorption Pattern
The fictional-world training result is the most transferable insight: agents trained in synthetic but self-consistent environments generalize better than those trained only on real data. This validates our SEAL pipeline's exploration stages — we should generate synthetic training scenarios, not just absorb real experiences.

---

### 2. Declarative Attention (DA) — Model-Controlled Sparse Attention
- **Source**: arXiv:2609.02737
- **Scale**: Zero-shot on Gemma-4-31B, Qwen-3.6-27B
- **Performance**: 52% token reduction (Gemma-4-31B), 31% (Qwen-3.6-27B) with 1.27pp/2.75pp accuracy drops

#### Key Technical Contributions
- **Intrinsic attention declaration**: Model declares its own attention needs during chain-of-thought, partitioning into `<global>`, `<focus>`, `<local>` modes
- **No training required**: Works on off-the-shelf models, accuracy drops shrink with model scale
- **Parseable declarations**: Inference engine parses declarations like tool calls, skipping KV cache reads for unneeded regions
- **15 long-context tasks validated**: Zero-shot evaluation across diverse benchmarks

#### NeoTrix Domain Mapping

| Domain | Integration | Priority |
|--------|-------------|----------|
| **NT-CORE** | GWT attention routing: use model-declared attention zones instead of external salience scoring. Three modes (global/focus/local) map to GWT salience tiers. | P0 |
| **NT-MEMORY** | Memory retrieval: attention declarations indicate which memory regions are relevant for current reasoning. Could guide KB query scoping. | P1 |
| **NT-IO** | Context management: attention mode declarations enable smarter context window allocation across sessions. | P1 |
| **NT-MIND** | Self-test: model's ability to declare attention zones could be a diagnostic for reasoning quality. | P2 |

#### Absorption Pattern
DA proves that frozen models can self-regulate attention at inference time. The key architectural insight is treating attention mode as a parseable output (like tool calls), not an internal state. This maps directly to our GWT — instead of external salience computation, let the model declare its attention needs and route accordingly.

---

### 3. ROAM — Relation-Guided Atomic Memory Management
- **Source**: arXiv:2609.09778
- **Performance**: Up to 29.8pp accuracy improvement, 15.6pp higher answer-critical source recall

#### Key Technical Contributions
- **Atom pair classification**: Independent, equivalent, directionally subsuming, conflicting — structured relation types between memory atoms
- **Primary/Evidence role organization**: Primary memories are the answer source; Evidence memories support but don't compete independently
- **Fusion**: Combines complementary details and temporal changes into compact, potentially non-atomic views
- **Robust across manager scales**: Performance holds regardless of the LLM used for memory management

#### NeoTrix Domain Mapping

| Domain | Integration | Priority |
|--------|-------------|----------|
| **NT-MEMORY** | Core: experience-tree should classify experience pairs by relation type (equivalent, subsuming, conflicting). Primary/Evidence role distinction for experience hub organization. | P0 |
| **NT-MIND** | SEAL distillation: fusion is the distillation mechanism — combine atomic experiences into compact knowledge views. | P0 |
| **NT-CORE** | KB edge types: relation classification (independent/equivalent/subsuming/conflicting) as a new edge taxonomy for our knowledge graph. | P1 |
| **NT-GOVERNANCE** | Conflict resolution: conflicting memory detection is a form of governance — identify and resolve contradictions. | P1 |

#### Absorption Pattern
ROAM's key insight is that memory management is not just storage but relationship management. The Primary/Evidence role distinction solves a real problem: redundant memories competing for retrieval. For NeoTrix, this means our experience-tree should organize experiences by role (primary answer vs supporting evidence) and classify relations between them.

---

### 4. AgensFlow — Coordination-Policy Substrate
- **Source**: arXiv:2605.27466, GitHub: Nicolepcx/AgensFlow
- **Scale**: 6-layer framework, evaluated on distributed-systems and security-advisory corpora

#### Key Technical Contributions
- **Coordination as learnable policy**: Routing decisions are learned from trajectories, not hand-coded pipelines
- **skip:X topology learning**: Policy can learn to EXCLUDE skills from trajectories — a novel topology action
- **Folded policy graph**: Repeated handoff configurations collapse to same orchestration node, making search reusable
- **RelativeJudge**: Cross-judge averaging, per-axis decomposition, disagreement-derived confidence weighting
- **Warm-start transfer**: Policy graphs learned on one domain transfer to structurally similar domains with ~21% fewer tokens

#### NeoTrix Domain Mapping

| Domain | Integration | Priority |
|--------|-------------|----------|
| **NT-ACT** | Core: capability routing should learn from task trajectories. skip:X as a routing action — learn when NOT to use a capability. | P0 |
| **NT-MIND** | SEAL evolution: coordination topology should evolve based on task regime, not remain fixed. Warm-start transfer validates cross-domain learning. | P0 |
| **NT-GOVERNANCE** | Policy enforcement: folded policy graph as an auditable governance artifact — see exactly which actions were selected for each regime. | P1 |
| **NT-CORE** | GWT: coordination policy learning could inform attention salience — what's important depends on the coordination regime. | P2 |

#### Absorption Pattern
AgensFlow's skip:X is the most transferable concept: the ability to learn NOT to do something is as valuable as learning what to do. For NeoTrix, this means our capability routing should include "skip" as a first-class action. The folded policy graph is also directly applicable — repeated coordination patterns should collapse to reusable nodes.

---

### 5. Flux Attention — Context-Aware Hybrid Attention
- **Source**: arXiv:2604.07394
- **Scale**: Lightweight Layer Router, frozen backbone, 12 hours training on 8×A800
- **Performance**: 2.8× prefill speedup, 2.0× decode speedup at 256K context

#### Key Technical Contributions
- **Layer-level routing**: Instead of per-head sparsity (hardware-unfriendly), route entire layers to full or sparse attention based on input context
- **Gumbel-Softmax routing**: Differentiable soft routing during training, discretized to hard routing at inference
- **Parameter-efficient**: Only router trained, backbone frozen — 12 hours to converge
- **Context-aware**: Router infers task demands from semantic context and assigns layers accordingly
- **KV cache reduction**: Sparse layers bypass full historical KV access and storage

#### NeoTrix Domain Mapping

| Domain | Integration | Priority |
|--------|-------------|----------|
| **NT-CORE** | GWT: layer-level routing as a model for attention-tier assignment. Context-aware routing validates task-dependent attention allocation. | P0 |
| **NT-IO** | Context management: layer-level KV cache reduction strategy — sparse layers need minimal cache, retrieval layers need full cache. | P1 |
| **NT-MIND** | Self-test: router quality could be a diagnostic — does the router correctly identify which layers need full attention? | P2 |
| **NT-PHYSICAL** | Hardware efficiency: layer-level routing preserves contiguous memory access, translating theoretical sparsity into wall-clock speedup. | P1 |

#### Absorption Pattern
Flux Attention's key insight is that layer-level routing is more hardware-friendly than head-level routing. The Gumbel-Softmax training approach is a general pattern: train a lightweight router to make discrete decisions, using differentiable relaxation during training. For NeoTrix, this validates our approach of training lightweight routing components while keeping backbones frozen.

---

## Cross-Model Synthesis

### Pattern: Attention as a First-Class Routing Problem
All five models address the same fundamental question: **where should computation be allocated?**
- Qwen-AgentWorld: allocate simulation resources across 7 domains
- Declarative Attention: let the model declare where to attend
- ROAM: allocate retrieval resources based on memory roles
- AgensFlow: allocate coordination resources based on task regime
- Flux Attention: allocate attention resources based on layer importance

**NeoTrix implication**: Our GWT attention routing should be unified across all these dimensions — attention to context (DA), attention to memory (ROAM), attention to capabilities (AgensFlow), attention to layers (Flux), attention to domains (Qwen-AgentWorld).

### Pattern: Frozen Backbones + Lightweight Routers
Three of five models use frozen backbones with lightweight trained components:
- Declarative Attention: zero training, model declares attention
- Flux Attention: 12-hour router training, backbone frozen
- ConvMem (from trending): training-free hierarchical compression

**NeoTrix implication**: Our architecture is well-positioned — frozen backbones with lightweight routers is exactly our approach. The training budget should go to routing/orchestration components, not backbone modification.

### Pattern: Memory as Relationship Graph
ROAM and MemoryLACE both treat memory as a graph of relationships, not a flat store:
- ROAM: atom pairs with relation types (equivalent, subsuming, conflicting)
- MemoryLACE: lifecycle relations (merge, supersession, contradiction)

**NeoTrix implication**: Our KB edges should be typed with these relation types. Experience-tree should manage experience lifecycle, not just store experiences.

### Pattern: Coordination Topology is Learnable
AgensFlow and ReActNet both show that coordination topology should be learned, not designed:
- AgensFlow: folded policy graph with UCB1 selection
- ReActNet: temporal graph compilation with natural-language edge instructions

**NeoTrix implication**: Our SEAL pipeline stages should have learnable topology — the system should discover which coordination patterns work for which task regimes.

---

## Priority Integration Matrix

| Model | NT-CORE | NT-MIND | NT-MEMORY | NT-WORLD | NT-ACT | NT-IO | NT-SHIELD |
|-------|---------|---------|-----------|----------|--------|-------|-----------|
| Qwen-AgentWorld | ★ | ★★ | ★ | ★★★ | ★★ | - | - |
| Declarative Attention | ★★★ | ★ | ★★ | - | - | ★★ | - |
| ROAM | ★★ | ★★★ | ★★★ | - | - | - | ★ |
| AgensFlow | ★ | ★★★ | - | - | ★★★ | - | ★★ |
| Flux Attention | ★★★ | ★ | - | - | - | ★★ | - |

★★★ = Core integration, ★★ = Strong relevance, ★ = Minor relevance
