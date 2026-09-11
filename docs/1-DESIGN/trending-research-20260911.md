# Trending AI/Agent Projects Research — 2026-09-11

## Source: Trendshift.io (Monthly #1 — August 2026)

### 1. DeepSeek Harness (dsh)
- **URL**: https://github.com/deepseek-ai/deepseek-harness
- **Stars**: 191K+ (215K as of research date)
- **Pattern**: "Everything is a Plugin" architecture powered by Cordis meta-framework. Micro-kernel where every component (model, tools, sessions, sandbox, UI) is a replaceable plugin with typed events and reversible effects. Profiles/bundles compose plugin trees at boot. Agent loop is swappable. Session state is durable JSONL.
- **NeoTrix Mapping**: NT-CORE (agent loop architecture), NT-MIND (skill/plugin composition), NT-IO (model provider abstraction)
- **Priority**: P0 — The dominant agent harness pattern of 2026. Cordis-style composability directly informs NeoTrix's plugin architecture.

---

### 2. PrimeAgent
- **URL**: https://github.com/PrimeIntellect-ai/prime-agent
- **Stars**: 15.9K+ (19.3K monthly)
- **Pattern**: Recursive Language Model (RLM) — treats context as variables and subagents as recursive function calls inside a persistent Python REPL (IPython kernel). Continual Harness stores skills/memories as durable state, refined through `/refine` pipeline. Agent-to-agent communication across sessions. Multi-process daemon architecture with worker isolation.
- **NeoTrix Mapping**: NT-CORE (RLM recursive delegation), NT-MIND (continual harness self-improvement), NT-ACT (subagent orchestration)
- **Priority**: P0 — RLM paradigm + continual self-improvement aligns directly with NeoTrix's SEAL pipeline and self-evolution.

---

### 3. archify (tt-a1i/archify)
- **URL**: https://github.com/tt-a1i/archify
- **Stars**: 51.6K
- **Pattern**: Agent skill for architecture diagram generation — self-contained HTML with motion and crisp export. Verifiable architecture diagrams (no Mermaid slop). Skill-based composable output format.
- **NeoTrix Mapping**: NT-IO (skill output format), NT-CORE (architecture visualization)
- **Priority**: P1 — Useful as skill output format pattern for NT-CORE architecture visualization.

---

### 4. diagram-design (cathrynlavery/diagram-design)
- **URL**: https://github.com/cathrynlavery/diagram-design
- **Stars**: 26.1K
- **Pattern**: 38 editorial diagram types for Claude Code, Codex, and Pi. Self-contained HTML + SVG. No shadows. No Mermaid slop. Standardized visual language for agent-produced diagrams.
- **NeoTrix Mapping**: NT-IO (visualization skills), NT-CORE (architecture diagram generation)
- **Priority**: P2 — Complementary to archify, standardized diagram skill pattern.

---

### 5. mattpocock/skills
- **URL**: https://github.com/mattpocock/skills
- **Stars**: 45.1K
- **Pattern**: "Skills for Real Engineers. Straight from my .agents directory." Curated, production-ready agent skills with progressive disclosure. Skills as first-class composable units.
- **NeoTrix Mapping**: NT-MIND (skill crystallization), NT-ACT (skill execution)
- **Priority**: P1 — Validates NeoTrix's skill node architecture (Small Passive → Notable Passive → Keystone).

---

## Source: GitHub Trending (August-September 2026)

### 6. TencentDB Agent Memory
- **URL**: https://github.com/TencentCloud/TencentDB-Agent-Memory
- **Stars**: 21.7K (25.9K at research time)
- **Pattern**: Team-level memory hub with "Trinity Memory Model" — vector (pgvector) + graph (Apache AGE) + relational table, all in one PostgreSQL instance. Four reusable memory assets: Chat Memory, Skill, LLM-Wiki, Code-Graph. L0→L1→L2→L3 layered distillation (conversation→atom→scenario→persona). Multi-tenant RLS isolation. Memory Hub as control panel for human governance.
- **NeoTrix Mapping**: NT-MEMORY (memory architecture, layered distillation), NT-GOVERNANCE (multi-tenant, human governance panel)
- **Priority**: P0 — The memory hub pattern directly maps to NeoTrix's KB namespace architecture. L0-L3 distillation is a strong pattern for experience-tree absorption.

---

### 7. NVIDIA NeMo Switchyard
- **URL**: https://github.com/NVIDIA-NeMo/Switchyard
- **Stars**: 2.7K
- **Pattern**: Rust proxy/library for LLM traffic routing with protocol translation (OpenAI↔Anthropic). Typed, composable routing algorithms: LLM Classifier, Stage Router, Escalation Router. Signal-driven routing using conversation-level signals (tool results, errors). Embeddable as library (switchyard-libsy) or standalone server. 74% cost reduction demonstrated.
- **NeoTrix Mapping**: NT-IO (model routing, provider abstraction), NT-CORE (GWT cost-aware routing)
- **Priority**: P0 — Direct implementation of Axiom A1 (Cost-Aware Routing). Stage Router pattern aligns with GWT salience-based routing.

---

### 8. OpenViking (volcengine/OpenViking)
- **URL**: https://github.com/volcengine/OpenViking
- **Stars**: 23K+
- **Pattern**: "Context File System" — all context organized as virtual filesystem under `viking://` URI. Three context types: Resource (docs), Memory (cognition), Skill (callable). L0/L1/L2 tiered context loading. Directory recursive retrieval (lock on high-scoring dirs, then explore content). Unix-like API for agent context manipulation. Self-evolving session management.
- **NeoTrix Mapping**: NT-MEMORY (context filesystem, tiered loading), NT-WORLD (resource management), NT-MIND (self-iteration)
- **Priority**: P0 — The filesystem-as-context paradigm is a breakthrough pattern. Directly maps to NeoTrix's KB namespace + experience-tree architecture. Tiered loading (L0/L1/L2) validates NT-MEMORY lazy branch loading.

---

### 9. Semantica
- **URL**: https://github.com/semantica-agi/semantica
- **Stars**: 11.9K
- **Pattern**: Graph-native infrastructure for context and accountable AI. Deterministic (no LLM for graph construction). Context Graphs as structured queryable graph. Decision Intelligence — every AI decision is a first-class graph node with full lifecycle. Causal chains, W3C PROV-O provenance, temporal intelligence (Allen interval algebra). Polyglot graph storage (RDF + LPG). Policy engine with SHACL constraints.
- **NeoTrix Mapping**: NT-MEMORY (context graphs, decision provenance), NT-GOVERNANCE (audit trails, policy engine), NT-SHIELD (compliance)
- **Priority**: P1 — Decision Intelligence pattern is powerful for NT-GOVERNANCE audit trail. Causal chain reasoning could enhance ConsciousnessTree cross-domain health tracking.

---

### 10. LangChain Deep Agents
- **URL**: https://github.com/langchain-ai/deepagents
- **Stars**: 9.8K
- **Pattern**: Opinionated harness on LangGraph with planning, filesystem backend, subagent spawning, context management, and skills. Stateless subagents for context isolation. Supervisor pattern where main agent coordinates subagents as tools. Skills loaded on-demand via progressive disclosure.
- **NeoTrix Mapping**: NT-ACT (subagent orchestration), NT-MIND (skill management), NT-CORE (agent loop)
- **Priority**: P2 — Subagent-as-tool pattern is well-understood. Useful reference for NT-ACT delegation modes.

---

### 11. Grok Build (SpaceXAI)
- **URL**: https://github.com/x-ai/grok-build (open-sourced July 2026)
- **Stars**: ~10K+ (part of Pi ecosystem)
- **Pattern**: Six-plane harness architecture: Surfaces (TUI/IDE/CLI/WebSocket via ACP) → Leader daemon → Session actor (three nested loops: mailbox→turn→agentic) → Safety (Landlock/bubblewrap kernel sandbox × per-call permission engine) → Extensibility (5-layer config merge, 15 hook events, MCP, skills, plugins, FTS+vector memory with dream consolidation). Actor pattern everywhere with zero locks.
- **NeoTrix Mapping**: NT-CORE (harness architecture), NT-SHIELD (sandbox × permission), NT-IO (ACP protocol)
- **Priority**: P1 — The 5-layer config merge and actor-pattern-with-zero-locks are strong architectural patterns. Safety model (kernel sandbox × software permissions) is directly applicable to NT-SHIELD.

---

### 12. OpenMAIC (THU-MAIC)
- **URL**: https://github.com/THU-MAIC/OpenMAIC
- **Stars**: 27.6K
- **Pattern**: Open Multi-Agent Interactive Classroom — multi-agent learning environment. Agents with specialized roles interact in structured educational scenarios.
- **NeoTrix Mapping**: NT-IO (multi-agent interaction), NT-MIND (learning patterns)
- **Priority**: P2 — Educational multi-agent pattern, less directly relevant.

---

### 13. reverse-skill (zhaoxuya520/reverse-skill)
- **URL**: https://github.com/zhaoxuya520/reverse-skill
- **Stars**: 22.4K (33.3K at research time)
- **Pattern**: AI-powered routing + on-demand toolchain bootstrapping + self-evolving knowledge base for reverse engineering/pentesting. Skill router pack that auto-routes to appropriate tools based on task classification.
- **NeoTrix Mapping**: NT-ACT (skill routing), NT-SHIELD (security research)
- **Priority**: P1 — Skill router pattern with self-evolving knowledge base directly maps to NT-MIND skill crystallization + NT-ACT tool routing.

---

### 14. Pi (earendil-works/pi)
- **URL**: https://github.com/earendil-works/pi (also pi.dev)
- **Stars**: 62.4K+
- **Pattern**: Minimal agent harness — "adapt Pi to your workflows, not the other way around." Extensions, skills, prompt templates, themes as composable packages. Ships with NO subagents, NO plan mode, NO MCP built-in. ACP protocol for IDE integration. Tree-saved sessions. Token-efficient context management.
- **NeoTrix Mapping**: NT-IO (minimal harness pattern), NT-CORE (ACP protocol)
- **Priority**: P1 — Validates "minimal core + composable skills" philosophy. NeoTrix's architecture should maintain lean core with extension points.

---

### 15. pi-agent-harness (baryonlabs)
- **URL**: https://github.com/baryonlabs/pi-agent-harness
- **Stars**: 6
- **Pattern**: Team-architecture factory — converts domain description into multi-agent teams with 6 patterns (Pipeline, Fan-out/Fan-in, Expert Pool, Producer-Reviewer, Supervisor, Hierarchical Delegation). Maps to pi's delegation modes (single/parallel/chain). Auto-generates skills with progressive disclosure.
- **NeoTrix Mapping**: NT-ACT (team orchestration patterns), NT-MIND (skill generation)
- **Priority**: P2 — The 6 architecture patterns are a useful reference catalog for NT-ACT delegation modes.

---

## Source: arxiv Papers (Recent)

### 16. Infini Memory (arXiv:2606.10677)
- **URL**: https://arxiv.org/abs/2606.10677
- **Stars**: N/A (research paper)
- **Pattern**: Maintainable topic-structured documents for long-term LLM agent memory. Each topic document is a semantic unit for evidence collection, metadata preservation, and fact revision. Buffer staging + periodic consolidation. Agentic retrieval via iterative tool calls (not single retrieval). 64.7% on MemoryAgentBench.
- **NeoTrix Mapping**: NT-MEMORY (topic-structured memory, iterative retrieval), NT-MIND (evidence consolidation)
- **Priority**: P1 — Topic-document memory architecture is a strong evolution of flat KB storage. Iterative retrieval aligns with NT-MEMORY lazy loading.

---

### 17. Agent Memory Distillation (AMD) (arXiv:2608.07169)
- **URL**: https://arxiv.org/abs/2608.07169
- **Stars**: N/A (research paper)
- **Pattern**: Three complementary memory types from teacher trajectories: Workflow (task-level strategies), Subtask (intermediate behavioral examples), Function (per-function calling conventions). Proactive injection (Workflow+Subtask at task start) + reactive injection (Function on error). Training-free knowledge transfer from large to small agents.
- **NeoTrix Mapping**: NT-MEMORY (hierarchical memory types), NT-MIND (distillation pipeline), NT-ACT (error-reactive knowledge injection)
- **Priority**: P1 — Three-type memory taxonomy (Workflow/Subtask/Function) maps well to NeoTrix's experience hierarchy. Error-reactive injection is a useful pattern for NT-REPAIR.

---

### 18. AdaCoM (arXiv:2605.30785)
- **URL**: https://arxiv.org/abs/2605.30785
- **Stars**: N/A (research paper)
- **Pattern**: Adaptive Context Management — trains external LLM to manage context of frozen agent via RL. Fidelity-Reliability Trade-off: high-performing agents need higher-fidelity preservation; lower-performing agents need aggressive compression. Generalizes across similar-capability agents.
- **NeoTrix Mapping**: NT-MEMORY (adaptive context compression), NT-CORE (agent self-optimization)
- **Priority**: P2 — Interesting trade-off insight but requires training infrastructure not yet in NeoTrix.

---

### 19. Role-Agent (arXiv:2606.10917)
- **URL**: https://arxiv.org/abs/2606.10917
- **Stars**: N/A (research paper)
- **Pattern**: Dual-role evolution — single LLM acts as both agent and environment simultaneously. World-In-Agent (predict future states, alignment as process reward) + Agent-In-World (analyze failure modes, retrieve similar failure patterns for targeted practice). Bootstrapped co-evolution.
- **NeoTrix Mapping**: NT-MIND (self-evolution via dual-role), NT-CORE (meta-cognition)
- **Priority**: P2 — Dual-role concept aligns with ConsciousnessTree's self-monitoring but requires significant architectural work.

---

### 20. NeuroTaint (arXiv:2604.23374)
- **URL**: https://arxiv.org/abs/2604.23374
- **Stars**: N/A (research paper)
- **Pattern**: Taint tracking framework for LLM agents — semantic transformation + causal influence + cross-session persistence tracking. Replaces exact string matching with semantic evidence and causal reasoning. 400-scenario TaintBench benchmark across 20 frameworks.
- **NeoTrix Mapping**: NT-SHIELD (taint tracking, information flow security), NT-SHIELD (egress privacy guard enhancement)
- **Priority**: P1 — Directly enhances NT-SHIELD's egress privacy guard with semantic taint tracking. Cross-session persistence tracking is novel.

---

### 21. Secure LLM Agents Survey (arXiv:2606.10749)
- **URL**: https://arxiv.org/abs/2606.10749
- **Stars**: N/A (research paper)
- **Pattern**: Lifecycle-based security framework — information flow × delegated authority × persistent state. 247 papers synthesized. Key finding: prompt injection dominates but persistent state corruption and multi-agent propagation are emerging. Defenses weakly compositional.
- **NeoTrix Mapping**: NT-SHIELD (security architecture), NT-GOVERNANCE (compliance framework)
- **Priority**: P1 — Essential reference for NT-SHIELD architecture. "Explicit trust boundaries + principled privilege control + provenance-aware state management" maps directly.

---

### 22. RePro (arXiv:2606.14302)
- **URL**: https://arxiv.org/abs/2606.14302
- **Stars**: N/A (research paper)
- **Pattern**: Retrospective Progress-Aware Training — forward-then-reflect paradigm. Agent executes actions online, then retrospectively reassesses step-wise progress given completed trajectory. Self-generated progress signals without external supervision. +12% absolute success rate.
- **NeoTrix Mapping**: NT-MIND (self-reflection), NT-CORE (progress awareness)
- **Priority**: P2 — Retrospective reflection pattern is valuable for SEAL pipeline's distillation phase.

---

## Source: Harness/Agent Pattern Research

### 23. Grok-Pi Bridge
- **URL**: https://github.com/Dwsy/grok-pi
- **Stars**: N/A (extension package)
- **Pattern**: Bridges Grok CLI session models into Pi via cli-chat-proxy. Reuses existing auth, ACP protocol translation. Same pattern as AI SDK harness adapters — unified interface across heterogeneous agent runtimes.
- **NeoTrix Mapping**: NT-IO (harness adapter pattern), NT-CORE (ACP protocol)
- **Priority**: P2 — Validates NeoTrix's capability bridge concept but not novel.

---

## Summary: Priority Rankings

### P0 — Immediate Absorption (5 projects)
| Project | Key Pattern | NT Domain |
|---------|------------|-----------|
| DeepSeek Harness | Everything-is-a-plugin (Cordis) | NT-CORE, NT-MIND, NT-IO |
| PrimeAgent | RLM + Continual Harness self-improvement | NT-CORE, NT-MIND |
| TencentDB Agent Memory | Trinity Memory + L0-L3 distillation | NT-MEMORY, NT-GOVERNANCE |
| NVIDIA Switchyard | Cost-aware LLM routing (Rust) | NT-IO, NT-CORE (A1 axiom) |
| OpenViking | Context File System + tiered loading | NT-MEMORY, NT-WORLD |

### P1 — High Value (8 projects)
| Project | Key Pattern | NT Domain |
|---------|------------|-----------|
| Semantica | Decision Intelligence + context graphs | NT-MEMORY, NT-GOVERNANCE |
| reverse-skill | Skill router + self-evolving KB | NT-ACT, NT-MIND |
| Pi | Minimal core + composable skills | NT-IO, NT-CORE |
| Infini Memory | Topic-structured memory documents | NT-MEMORY |
| AMD (arxiv) | Three-type hierarchical memory | NT-MEMORY, NT-MIND |
| NeuroTaint | Semantic taint tracking for agents | NT-SHIELD |
| Secure Agents Survey | Lifecycle security framework | NT-SHIELD |
| archify / mattpocock/skills | Skill-as-production-unit pattern | NT-MIND |

### P2 — Reference Value (5 projects)
| Project | Key Pattern | NT Domain |
|---------|------------|-----------|
| Deep Agents | Subagent-as-tool supervisor | NT-ACT |
| Grok Build | Actor pattern + 5-layer config | NT-CORE, NT-SHIELD |
| AdaCoM | Adaptive context compression | NT-MEMORY |
| Role-Agent | Dual-role co-evolution | NT-MIND |
| RePro | Retrospective progress reflection | NT-MIND |

---

## Key Architectural Patterns to Absorb

### 1. Cordis-Style Plugin Composability (from DeepSeek Harness)
- Every component is a plugin with typed events and reversible effects
- Profiles/bundles compose plugin trees at boot
- No privileged core — extend by mounting plugins
- **NeoTrix mapping**: Refactor NT-CORE to Cordis-like plugin architecture

### 2. Context-as-Filesystem (from OpenViking)
- `viking://` URI scheme for unified context addressing
- L0 (abstract) → L1 (overview) → L2 (full content) tiered loading
- Directory recursive retrieval (high-score dirs first, then drill down)
- **NeoTrix mapping**: Enhance KB namespace with filesystem-like URI + tiered lazy loading

### 3. Trinity Memory Model (from TencentDB Agent Memory)
- Vector (semantic) + Graph (relational) + Relational (factual) in one system
- L0→L1→L2→L3 distillation pipeline
- Human governance panel for memory lifecycle
- **NeoTrix mapping**: Extend KB with graph relations alongside vector+BM25

### 4. Cost-Aware Model Routing (from NVIDIA Switchyard)
- LLM Classifier: content-based routing to weak/strong models
- Stage Router: signal-driven routing from conversation context
- Escalation Router: weak-first, escalate on sustained difficulty
- **NeoTrix mapping**: Implement in GWT salience routing with token cost weights

### 5. Recursive Language Model (from PrimeAgent)
- Context as variable, subagents as recursive function calls
- Persistent REPL (IPython kernel) as model-facing control environment
- Continual Harness: skills/memories refined through evidence-based refinement
- **NeoTrix mapping**: Enhance SEAL pipeline with recursive subagent delegation + continual refinement

### 6. Decision Intelligence as First-Class Objects (from Semantica)
- Every AI decision is a graph node with full lifecycle
- Causal chains linking upstream causes and downstream effects
- W3C PROV-O provenance for audit compliance
- **NeoTrix mapping**: Add decision provenance to NT-GOVERNANCE audit trail

### 7. Semantic Taint Tracking (from NeuroTaint)
- Replace exact string matching with semantic evidence + causal reasoning
- Cross-session persistence tracking
- **NeoTrix mapping**: Enhance egress_privacy_guard with semantic-level information flow tracking

### 8. Three-Type Memory Taxonomy (from AMD)
- Workflow memory (task-level strategies)
- Subtask memory (behavioral examples)
- Function memory (per-tool calling conventions, reactive injection on error)
- **NeoTrix mapping**: Structure experience-tree nodes into three tiers

---

## Source: NVlabs/SoL-Pi (Targeted)

The specific project `https://github.com/NVlabs/SoL-Pi` was not found as a distinct public repository. Research suggests "SoL-Pi" may refer to:
1. **Pi Coding Agent** (earendil-works/pi) — the minimal agent harness (62.4K stars)
2. **NVIDIA AI-Q Blueprint** — multi-agent research architecture with intent classifier → shallow/deep researchers
3. A possible internal NVIDIA project not yet public

The closest match is Pi itself, which has been analyzed above (Entry #14). NVIDIA's AI-Q Blueprint (docs.nvidia.com/aiq-blueprint) shows a multi-agent intent-classification architecture that routes to shallow/deep researcher agents — relevant to NT-WORLD perception routing.

---

*Research conducted: 2026-09-11 | Sources: Trendshift.io, GitHub Trending, arxiv, project repositories*
