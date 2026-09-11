# Trending Rankings — Cycle 358 (2026-09-12)

## Search Scope
GitHub Trending, ProductHunt, arXiv, GitTrend — AI agents, LLM tools, reasoning frameworks, memory/attention/routing patterns.

---

## 10 New Projects (Not in Cycles 318–357)

### 1. OpenViking — Self-Evolving Context Database for AI Agents
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/volcengine/OpenViking |
| **Stars** | 30,700+ |
| **Language** | Python |
| **Category** | Agent Memory / Knowledge RAG |
| **What it does** | Self-evolving context database unifying agent memory, knowledge RAG, and skills into one substrate. Agent builds its own memory graph through interaction, not manual configuration. |
| **NeoTrix mapping** | NT-MEMORY (KB) + NT-NEXUS (cross-session). Self-evolving context = experience-tree auto-absorption where the system learns which memories matter without manual pruning. Validates NT-MEMORY's BM25+embedding hybrid approach. |
| **Key pattern** | **Self-evolving context** — memory substrate that restructures itself based on usage patterns, not static schemas. The agent's memory literally evolves like a biological system. |

### 2. Semantica — Graph-Native Infrastructure for Accountable AI
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/semantica-agi/semantica |
| **Stars** | 9,700+ |
| **Language** | Python |
| **Category** | Graph-Based AI Infrastructure |
| **What it does** | Graph-native infrastructure for context and accountable AI systems. Every agent action is traceable through a knowledge graph, enabling auditability and reasoning over causal chains. |
| **NeoTrix mapping** | NT-SHIELD (audit) + NT-GOVERNANCE (policy). Graph-native accountability = NT-GOVERNANCE's compliance verification with full action traceability. Causal graph = E8 hexagram reasoning with explicit edges. |
| **Key pattern** | **Graph-native accountability** — not bolt-on logging but structural accountability where the graph itself enforces causal consistency. Every decision has a traceable causal path. |

### 3. ZenBrain — Neuroscience-Inspired 7-Layer Memory
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/zensation-ai/zenbrain |
| **Stars** | New (2026, arXiv:2604.23878) |
| **Language** | TypeScript |
| **Category** | Agent Memory Architecture |
| **What it does** | 7-layer memory architecture inspired by neuroscience: FSRS spaced repetition, Hebbian learning, Ebbinghaus forgetting curves, sleep consolidation, emotional tagging. Wins 9/9 on LongMemEval-500. Zero dependencies. |
| **NeoTrix mapping** | NT-MEMORY (KB) + NT-FEEL (emotion). 7-layer memory = NT-MEMORY's namespace hierarchy (ephemeral→session→KB→cross-session). Emotional tagging = NT-FEEL's EmotionLabel modulating memory consolidation priority. |
| **Key pattern** | **Neuroscience-grade memory** — not just "vector DB + search" but actual biological mechanisms: forgetting is a feature (Ebbinghaus curves), emotion gates consolidation, sleep reorganizes. |

### 4. Switchyard — LLM Traffic Routing Across Models and Providers
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/NVIDIA-NeMo/Switchyard |
| **Stars** | 1,900+ |
| **Language** | Rust |
| **Category** | Model Routing / Inference Infrastructure |
| **What it does** | Routes LLM application traffic across models and providers while preserving native OpenAI and Anthropic API compatibility. Enables flexible model selection, benchmarking, and cost/performance optimization. |
| **NeoTrix mapping** | NT-IO (provider routing) + NT-ACT (orchestration). Switchyard = NT-IO's ordered backend router with production hardening. API-compatible routing = same interface, different backends. |
| **Key pattern** | **Provider-agnostic routing** — single API surface with backend swapping. Not model-specific, not provider-specific. The routing layer is invisible to the application. |

### 5. Scientific Agent Skills — AI Scientist Skill Library
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/K-Dense-AI/scientific-agent-skills |
| **Stars** | 41,200+ (6,248 stars/week) |
| **Language** | Python |
| **Category** | Domain-Specific Agent Skills |
| **What it does** | 165+ validated skills for AI scientists covering biology, chemistry, medicine, drug discovery. Compatible with Cursor, Claude Code, Codex. Open Agent Skills standard. |
| **NeoTrix mapping** | NT-MIND (skill crystallization) + NT-ACT (capability). Scientific skills = SKILL-SPEC.md contract for domain-specific capabilities. Validates NeoTrix's constellation maturity model (C0→C6) for skill validation. |
| **Key pattern** | **Domain skill as production template** — not generic tools but validated, domain-specific skill packages with tests and benchmarks. The "skill as production template" axiom (A3) validated at scale. |

### 6. MATSIR — Multi-Agent Monte Carlo Tree Search for Inductive Reasoning
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/SolarWindRider/MATSIR |
| **Stars** | New (ACL 2026) |
| **Language** | Python |
| **Category** | Multi-Agent Reasoning Framework |
| **What it does** | Plug-and-play test-time framework integrating multi-agent coordination with Monte Carlo Tree Search for inductive reasoning. Dual-reward mechanism with backtracking and pruning. +4.9% on QWQ-32B. |
| **NeoTrix mapping** | NT-CORE (E8 reasoning) + NT-ACT (multi-agent). MCTS = E8 hexagram tree search with pruning. Dual-reward = ConsciousnessTree's phi + coherence dual signal. Backtracking = SEAL pipeline error correction. |
| **Key pattern** | **MCTS for agent reasoning** — treat reasoning steps as a search tree with branching, evaluation, and pruning. Not linear chain-of-thought but tree exploration with rollback. |

### 7. Needle — 14MB Foundation Model for Tiny Devices
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/cactus-compute/needle |
| **Stars** | 9,700+ (3,838 stars/week) |
| **Language** | Python |
| **Category** | Edge AI / Tiny Models |
| **What it does** | 14MB foundation model for phones, wearables, smart home, and robots. Proves that useful AI can run on constrained devices with minimal memory footprint. |
| **NeoTrix mapping** | NT-PHYSICAL (embodiment) + NT-IO (edge deployment). 14MB model = NT-PHYSICAL's body schema for resource-constrained physical agents. Validates A1 (cost-aware routing) at hardware level. |
| **Key pattern** | **Embodied minimal AI** — the agent doesn't need to be big to be useful. A 14MB model on a phone can do meaningful work. Challenges the "bigger is better" assumption. |

### 8. HarnessRouter — Unified Interface for Agent Harnesses
| Field | Detail |
|-------|--------|
| **URL** | ProductHunt (Aug 16, 2026) |
| **Stars** | New |
| **Language** | N/A |
| **Category** | Agent Orchestration |
| **What it does** | Open-source unified interface for agent harnesses. Single control plane for Claude Code, Codex, Cursor, and other coding agents. Route tasks to optimal harness. |
| **NeoTrix mapping** | NT-IO (interface) + NT-ACT (orchestration). HarnessRouter = NT-IO's multi-provider interface with NT-ACT's task routing. Validates Pattern P4 (Ordered Backend Fallback). |
| **Key pattern** | **Harness as infrastructure** — coding agents are interchangeable backends behind a unified routing interface. The routing intelligence > any single agent. |

### 9. Dropstone — AI Runtime That Remembers and Learns
| Field | Detail |
|-------|--------|
| **URL** | ProductHunt (Aug 26, 2026) |
| **Stars** | New (5.0 rating, 4 reviews) |
| **Language** | N/A |
| **Category** | Agent Runtime / Memory |
| **What it does** | AI runtime that remembers, learns, and acts everywhere. Persistent memory across sessions with adaptive learning from user interactions. IDE integration. |
| **NeoTrix mapping** | NT-NEXUS (cross-session memory) + NT-MIND (self-evolution). Persistent memory = NT-NEXUS's session bridging. Adaptive learning = NT-MIND's SEAL pipeline with continuous distillation. |
| **Key pattern** | **Persistent agent runtime** — not stateless API calls but a runtime that accumulates knowledge across interactions. The agent gets smarter over time by design. |

### 10. Flare — Graph-First IDE for Agentic Coding
| Field | Detail |
|-------|--------|
| **URL** | ProductHunt (Aug 26, 2026) |
| **Stars** | New (open source) |
| **Language** | N/A |
| **Category** | Agent-Native Development Environment |
| **What it does** | Graph-first IDE and interactive map for agentic coding. Visualizes code relationships as a graph, enabling agents to navigate codebases structurally rather than sequentially. |
| **NeoTrix mapping** | NT-WORLD (perception) + NT-CORE (reasoning). Graph-first IDE = NT-WORLD's code perception as structured graph, not flat file trees. Agent navigation = E8 hexagram reasoning over code topology. |
| **Key pattern** | **Graph-native code perception** — agents should see code as a graph (functions→calls→types→deps) not a tree (files→lines). Structural navigation > sequential scanning. |

---

## Meta-Patterns (Cycle 358)

| Pattern | Count | NeoTrix Implication |
|---------|-------|---------------------|
| **Self-Evolving Memory** | 3 | KB should auto-restructure based on usage, not require manual schema |
| **Graph-Native Reasoning** | 2 | E8 hexagram reasoning should use explicit graph edges, not implicit heuristics |
| **Neuroscience-Grade Memory** | 1 | Consider biological mechanisms (forgetting, emotion-gated consolidation) |
| **MCTS for Agent Reasoning** | 1 | Replace linear CoT with tree search + backtracking for complex reasoning |
| **Provider-Agnostic Routing** | 2 | NT-IO should be invisible infrastructure, not application-visible |
| **Domain Skill Packages** | 1 | SKILL-SPEC.md validated at scale with scientific rigor |
| **Embodied Minimal AI** | 1 | 14MB models challenge A1 — cost-aware routing goes to hardware level |

## Source Density

| Source | Projects Found | Quality |
|--------|---------------|---------|
| GitHub Trending (weekly) | 4 | High — real codebases with star velocity |
| ProductHunt (Aug-Sep 2026) | 3 | Medium — launch-stage, signal > quality |
| arXiv + GitHub (ACL 2026) | 2 | High — peer-reviewed with code |
| GitHub Explore (featured) | 1 | High — GitHub-curated |

## Novel vs Incremental

- **Truly Novel**: ZenBrain (neuroscience memory), MATSIR (MCTS reasoning), Needle (14MB foundation model)
- **Incremental but Valuable**: OpenViking (self-evolving context), Switchyard (provider routing), Semantica (graph accountability)
- **Signal-Only**: HarnessRouter, Dropstone, Flare (validate agent infrastructure direction)
