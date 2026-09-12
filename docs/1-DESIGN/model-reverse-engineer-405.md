# Model Reverse Engineering — Cycle 405 (2026-09-12)

## 5 New AI Models/Papers

---

### 1. Explicit Trait Inference for Multi-Agent Coordination (ACL 2026)
**Paper**: "Explicit Trait Inference for Multi-Agent Coordination"
**Authors**: Suhaib Abdurahman, Etsuko Ishii, Katerina Margatina, Divya Bhargavi, Monica Sunkara, Yi Zhang
**Source**: https://aclanthology.org/2026.acl-long.77

#### Core Innovation
Psychologically grounded multi-agent coordination via explicit trait tracking. Agents infer and track partner characteristics along warmth (trust) and competence (skill) dimensions from interaction histories.

#### Architecture
```
Interaction History
    ↓
[Trait Inference Module] → warmth + competence scores per partner
    ↓
[Coordination Policy] → adjusts collaboration strategy based on inferred traits
    ↓
[Action Selection] → partners with high competence get harder tasks; high warmth gets more trust
```

#### Key Mechanisms
- **Warmth-Competence Dual Axis**: Two psychological dimensions (derived from stereotyping research) that predict agent behavior. Warmth = trust/reliability. Competence = skill/effectiveness.
- **History-Based Inference**: Traits inferred from interaction histories, not declared. First systematic evidence that LLM agents can reliably infer others' traits from behavior.
- **Lightweight Mechanism**: No heavy training required — inference from interaction logs guides coordination decisions.

#### Results
- Reduces payoff loss by 45–77% in economic games
- Improves performance by 3–29% on MultiAgentBench vs CoT baseline
- Trait profiles predict agent actions; informative profiles drive improvements

#### NeoTrix Domain Mapping
| Domain | Integration | Pattern |
|--------|-------------|---------|
| **NT-CORE** | GWT attention routing | Trait inference = salience modifier; high-competence agents get more broadcast bandwidth |
| **NT-ACT** | Agent orchestration | Task allocation based on inferred competence; trust-based delegation for high-stakes tasks |
| **NT-MEMORY** | Knowledge graph | Trait profiles as persistent agent metadata; warmth/competence as edge properties |
| **NT-MIND** | SEAL pipeline | Trait tracking across evolution cycles; coordination quality as fitness signal |
| **NT-FEEL** | Emotion engine | Warmth maps to EmotionLabel::Trust; competence maps to EmotionLabel::Anticipation |

#### Actionable Patterns
1. **Trait Profiles for NT-ACT**: Add warmth/competence metadata to agent registry. Route tasks based on inferred competence rather than raw capability declarations.
2. **Trust-Based Delegation**: High-warmth agents receive more sensitive tasks. Low-competence agents get simpler subtasks or more supervision.
3. **History-Inferred, Not Declared**: Don't trust self-reported capabilities — infer from interaction outcomes. Aligns with NeoTrix's Evidence-First review methodology.

---

### 2. Flux Attention: Context-Aware Hybrid Attention (Apr 2026)
**Paper**: "Flux Attention: Context-Aware Hybrid Attention for Efficient LLMs Inference"
**Authors**: Quantong Qiu, Zhiyi Hong, Yi Yang, Haitian Wang, Kebin Liu, Qingqing Dang, Juntao Li, Min Zhang
**Source**: https://arxiv.org/abs/2604.07394

#### Core Innovation
Layer-level dynamic routing between Full Attention (FA) and Sparse Attention (SA) based on input context. Lightweight Layer Router inserted into frozen pretrained LLMs adaptively routes each layer.

#### Architecture
```
Input Context
    ↓
[Layer Router] → per-layer decision: Full Attention or Sparse Attention
    ↓
┌─────────────────────────────────────────┐
│ Layer 1: Full Attention (high-retrieval)  │
│ Layer 2: Sparse Attention (local context) │
│ Layer 3: Full Attention (long-range)      │
│ ...                                       │
└─────────────────────────────────────────┘
    ↓
[Output] → preserves high-fidelity retrieval + contiguous memory access
```

#### Key Mechanisms
- **Layer-Router**: Lightweight module (12 hours training on 8×A800) that routes each layer to FA or SA based on input context. Parameter-efficient — frozen pretrained LLM, only router trained.
- **Context-Aware Routing**: Dynamic allocation based on actual retrieval demands, not static ratios. Different tasks get different FA/SA layer distributions.
- **Hardware-Aware**: Preserves contiguous memory access for practical speedups (not just theoretical FLOP reduction).

#### Results
- 2.8× speedup in prefill, 2.0× in decode stages
- Superior performance-speed tradeoff vs static hybrid attention baselines
- Works on long-context and mathematical reasoning benchmarks

#### NeoTrix Domain Mapping
| Domain | Integration | Pattern |
|--------|-------------|---------|
| **NT-CORE** | GWT attention routing | Layer-Router = micro-GWT; per-layer attention allocation based on context salience |
| **NT-MEMORY** | Context management | FA layers for long-range retrieval; SA layers for local compression — mirrors KVMem hot/cold tiering |
| **NT-IO** | Inference optimization | 2.8× prefill speedup directly applicable to NeoTrix's LLM inference pipeline |

#### Actionable Patterns
1. **Per-Layer Attention Routing for GWT**: Extend GWT salience to operate at transformer layer level, not just module level. Each layer gets attention budget based on content type.
2. **FA/SA Dynamic Allocation**: For long-context sessions, dynamically allocate layers between full retrieval (FA) and local compression (SA) based on query complexity.
3. **Hardware-Aware Optimization**: Any attention optimization must consider actual memory access patterns, not just theoretical FLOP count.

---

### 3. TIPEX: Inference-Time Parallelism in Multi-Agent LLM Systems (ICML 2026)
**Paper**: "A Two-Tier Perspective on Inference-Time Parallelism in Multi-Agent LLM Systems"
**Authors**: Zihan Xu, Haolin Tian, Hai Jiang
**Source**: https://arxiv.org/abs/2608.05791

#### Core Innovation
Two-tier parallelism framework: Replica Parallelism (multiple complete solution paths) + Structural Parallelism (concurrent subtasks within one path). TIPEX unifies both under controllable execution semantics.

#### Architecture
```
Task Input
    ↓
[TIPEX Controller]
    ├── Replica Parallelism → N complete solution paths in parallel
    │   └── Each path: independent agent solving full task
    └── Structural Parallelism → decompose one path into concurrent subtasks
        └── Subtask A ←→ Subtask B (parallel)
    ↓
[Convergence] → select best result or merge
```

#### Key Mechanisms
- **Replica Parallelism**: Explore multiple complete solution paths simultaneously. Task-level diversity.
- **Structural Parallelism**: Decompose single path into concurrent subtasks. Step-level efficiency.
- **Complementary Effects**: Intermediate-complexity tasks benefit most from coordination. Overly aggressive parallelism doesn't always help.
- **Controllable Execution**: Systematic combinations of parallel strategies with parameter tuning.

#### Results
- Significant accuracy improvement + reduced end-to-end latency on GAIA benchmark
- Replica + Structural parallelism complementary across task complexities
- Intermediate tasks benefit most; extreme tasks see diminishing returns

#### NeoTrix Domain Mapping
| Domain | Integration | Pattern |
|--------|-------------|---------|
| **NT-ACT** | Agent orchestration | Replica parallelism = multiple workers on same task; Structural = DAG decomposition |
| **NT-CORE** | GWT attention routing | Parallel paths compete for salience; convergence = attention collapse to winner |
| **NT-MIND** | SEAL pipeline | Multiple evolution paths explored in parallel; best results propagated |
| **NT-SHIELD** | Safety isolation | Each replica needs isolated workspace to prevent cross-contamination |

#### Actionable Patterns
1. **Dual-Mode Execution for NT-ACT**: Implement both replica (N workers, same task) and structural (task decomposition) parallelism with controllable parameters.
2. **Complexity-Aware Parallelism**: Auto-detect task complexity and tune parallel strategy. Simple tasks → single path; complex tasks → replica parallelism; medium tasks → structural parallelism.
3. **Convergence Selection**: After parallel execution, use GWT salience to select best result or merge complementary partial results.

---

### 4. MemFactory: Unified Inference & Training Framework for Agent Memory (Apr 2026)
**Paper**: "MemFactory: Unified Inference & Training Framework for Agent Memory"
**Authors**: Ziliang Guo, Ziheng Li, Bo Tang, Feiyu Xiong, Zhiyu Li
**Source**: https://arxiv.org/abs/2603.29493

#### Core Innovation
First unified training and inference framework for memory-augmented agents. Abstracts memory lifecycle into atomic plug-and-play modules (Lego-like). GRPO training for memory management policies driven by multi-dimensional rewards.

#### Architecture
```
┌─────────────────────────────────────────────────┐
│                  MemFactory                       │
│                                                   │
│  Module Layer (atomic operations)                │
│  ├── Extractor → memory extraction               │
│  ├── Updater → state update decisions             │
│  ├── Retriever → context retrieval                │
│  └── Agent Module → composite capabilities        │
│                                                   │
│  Environment Layer                                │
│  ├── Dataset standardization                      │
│  └── Multi-dimensional reward signals             │
│                                                   │
│  Trainer Layer                                    │
│  └── GRPO → optimize memory management policies   │
└─────────────────────────────────────────────────┘
```

#### Key Mechanisms
- **Atomic Memory Operations**: Extractor, Updater, Retriever, Agent Module — each with standardized interface (generate, rollout, inference). Mix-and-match across research paradigms.
- **GRPO Training**: Group Relative Policy Optimization fine-tunes memory management policies via environmental feedback. Multi-dimensional rewards (not just accuracy).
- **Paradigm Support**: Out-of-box support for Memory-R1, RMM, MemAgent. Reproducible baseline for memory research.
- **Modular Assembly**: "Lego-like" composition — researchers swap extractors, updaters, retrievers independently.

#### Results
- 14.8% relative improvement on Qwen3-1.7B, 7.3% on Qwen3-4B-Instruct
- Consistent gains across in-domain and out-of-distribution evaluation
- First unified framework lowering barrier for memory-RL research

#### NeoTrix Domain Mapping
| Domain | Integration | Pattern |
|--------|-------------|---------|
| **NT-MEMORY** | KB operations | Atomic memory operations = KB write/read/update lifecycle; GRPO for memory policy optimization |
| **NT-MIND** | SEAL pipeline | Memory-RL training = SEAL distillation phase; multi-dimensional rewards = cross-domain health signals |
| **NT-CORE** | HyperCube | VSA embedding ↔ memory extraction; associative recall ↔ retrieval module |
| **NT-META** | Meta-cognition | Memory policy evaluation = meta-cognitive monitoring; reward signals = self-audit feedback |

#### Actionable Patterns
1. **Atomic Memory Interface for NT-MEMORY**: Define standardized Extractor/Updater/Retriever traits. Any memory operation composes from these atoms. Enables plug-and-play memory strategies.
2. **GRPO for Memory Policy**: Train memory management decisions (what to keep, what to forget, when to retrieve) via RL with environmental feedback. Not hand-coded heuristics.
3. **Multi-Dimensional Rewards**: Memory quality measured across accuracy, latency, storage cost, and retrieval precision — not just hit rate.

---

### 5. RAGEN-2: Reasoning Collapse in Agentic RL (Apr 2026)
**Paper**: "RAGEN-2: Reasoning Collapse in Agentic RL"
**Authors**: Zihan Wang et al. (Northwestern, UIUC, Imperial, Oxford, Stanford, Microsoft)
**Source**: https://arxiv.org/abs/2604.06268

#### Core Innovation
Diagnoses and fixes "template collapse" in multi-turn agent RL — reasoning drifts toward fluent but input-agnostic boilerplate while conditional entropy stays stable. SNR-Adaptive Filtering selects high-signal prompts each iteration.

#### Architecture
```
Multi-Turn Agent RL Training
    ↓
[Template Collapse Detection]
    ├── Within-input diversity: H(Z|X) — conditional entropy
    └── Cross-input distinguishability: I(X;Z) — mutual information
    ↓
[SNR-Adaptive Filtering] → uses reward variance as proxy for signal quality
    ↓
[High-Signal Prompt Selection] → only train on prompts with discriminative reasoning
    ↓
[Stable Training] → prevents reasoning degradation
```

#### Key Mechanisms
- **Template Collapse**: Reasoning appears diverse within one input (high H(Z|X)) but becomes input-agnostic across inputs (low I(X;Z)). Fluent but useless boilerplate.
- **SNR-Adaptive Filtering**: Uses reward variance as lightweight proxy to select high-signal prompts. Directly addresses root cause by filtering training data.
- **Dual Entropy Decomposition**: Separates within-input diversity from cross-input distinguishability. Template collapse is invisible to standard entropy monitoring.
- **Echo Trap Detection**: Reward variance cliffs and gradient spikes as early warning signals.

#### Results
- Detects and mitigates template collapse across planning, math reasoning, web navigation, code execution
- SNR filtering improves input dependence and performance across tasks, algorithms, model scales, modalities
- Lightweight intervention — no architectural changes needed

#### NeoTrix Domain Mapping
| Domain | Integration | Pattern |
|--------|-------------|---------|
| **NT-MIND** | SEAL pipeline | Template collapse = SEAL distillation stagnation; SNR filtering = quality gate for evolution cycles |
| **NT-CORE** | GWT attention | I(X;Z) = attention discriminability; template collapse = GWT broadcasting noise |
| **NT-MEMORY** | Knowledge quality | Template collapse in memory = storing boilerplate facts; SNR = fact novelty scoring |
| **NT-REPAIR** | Self-healing | Echo Trap = early warning for reasoning degradation; SNR = health metric |

#### Actionable Patterns
1. **SNR Filtering for SEAL**: Before each evolution cycle, compute reward variance across training prompts. Skip low-SNR prompts (template collapse signals). Train only on high-signal experiences.
2. **Dual Entropy Monitoring for GWT**: Track both H(Z|X) (within-query diversity) and I(X;Z) (cross-query discriminability) as GWT health metrics. Low I(X;Z) = attention broadcasting noise.
3. **Echo Trap Detection**: Monitor for reward variance cliffs and gradient spikes during evolution. If detected, trigger SNR filtering or cycle reset.
4. **Fact Novelty Scoring**: Apply I(X;Z) to KB writes — only commit facts with high cross-query discriminability. Prevents knowledge base boilerplate accumulation.

## Cross-Paper Synthesis

| Pattern | Source Papers | NeoTrix Integration |
|---------|---------------|---------------------|
| Trait-based agent coordination | ETI (ACL 2026) | NT-ACT: warmth/competence metadata in agent registry |
| Layer-level attention routing | Flux Attention | NT-CORE: per-layer GWT salience allocation |
| Dual-mode parallelism | TIPEX (ICML 2026) | NT-ACT: replica + structural parallel execution |
| Atomic memory operations | MemFactory | NT-MEMORY: standardized Extractor/Updater/Retriever traits |
| Template collapse detection | RAGEN-2 | NT-MIND: SNR filtering for SEAL quality gates |
| Verifiable self-improvement | Factory + RAGEN-2 | NT-MIND: falsifiable predictions + SNR-adaptive training |

## Action Items for NeoTrix

1. **Define Memory Trait Interface** (from ETI + MemFactory): warmth/competence tracking for agents + atomic memory operations trait system
2. **Implement SNR Filtering** (from RAGEN-2): reward variance-based quality gate for SEAL evolution cycles
3. **Layer-Level Attention Budget** (from Flux Attention): extend GWT to allocate attention per transformer layer
4. **Dual-Mode Parallel Execution** (from TIPEX): replica + structural parallelism in NT-ACT orchestration
5. **Template Collapse Monitoring** (from RAGEN-2): dual entropy tracking (H(Z|X) + I(X;Z)) for GWT and memory health
