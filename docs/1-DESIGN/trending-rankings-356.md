# Trending Rankings — Cycle 356 (2026-09-12)

## Search Scope
GitHub Trending, ProductHunt, arXiv, OSSInsight — AI agents, LLM tools, reasoning frameworks, memory/attention/routing patterns.

---

## 10 New Projects (Not in Cycles 318–355)

### 1. screenpipe (YC S26)
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/screenpipe/screenpipe |
| **Stars** | 21,430+ |
| **Language** | Rust |
| **Category** | Always-on Screen AI Memory |
| **What it does** | Continuously captures screen + audio locally, indexes everything, provides MCP server for agents (Claude, Codex, Hermes) to query "what did I do last Tuesday?" Local-first, privacy by default. |
| **NeoTrix mapping** | NT-WORLD (perception) + NT-MEMORY (episodic recall). The "always-on local context" pattern maps to a PerceptionBridge variant where sensory input is screen/audio rather than sensors. |
| **Key pattern** | **Event-driven capture** — only captures on app switches, clicks, typing pauses, not every frame. O(1) storage per event. |

### 2. OpenHands (formerly OpenDevin)
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/OpenHands/openhands |
| **Stars** | 85,800+ |
| **Language** | Python |
| **Category** | Autonomous Cloud Coding Agent |
| **What it does** | Production-grade autonomous coding agent with Docker sandbox, model-agnostic (Anthropic/OpenAI/Gemini), Agent SDK for custom workflows, Agent Canvas UI. 1.0 release with RBAC, audit trails, resource limits. |
| **NeoTrix mapping** | NT-ACT (action execution) + NT-SHIELD (sandbox isolation). The per-agent ephemeral container pattern aligns with NeoTrix's worktree isolation (P2: Isolation-per-Task). |
| **Key pattern** | **Context Condenser** — compresses token usage by condensing conversation history while preserving task-critical info. |

### 3. Agora — Auction-Based Agent Reasoning
| Field | Detail |
|-------|--------|
| **URL** | arXiv:2607.09600 |
| **Category** | Agent Task Allocation / Reasoning |
| **What it does** | Reformulates task allocation as a confidence-calibrated auction. Each reasoning step is "bid on" by specialized agents; the auction mechanism routes to the most capable agent per step. |
| **NeoTrix mapping** | NT-CORE (GWT salience routing) + NT-MIND (SEAL pipeline). Maps directly to GWT attention routing with cost-aware model selection (Axiom A1). |
| **Key pattern** | **Confidence-calibrated auction** — each agent bids with calibrated confidence, preventing overconfident-but-incompetent agents from hijacking critical reasoning nodes. |

### 4. BoundaryRouter — Agent/LLM Routing
| Field | Detail |
|-------|--------|
| **URL** | arXiv:2605.07180 |
| **Category** | Inference Routing / Cost Optimization |
| **What it does** | Training-free routing framework. Uses early behavioral experience + rubric-guided reasoning to decide: answer with lightweight LLM or escalate to full agent execution. Reduces inference time 60.6% vs agent, +28.6% over direct LLM. |
| **NeoTrix mapping** | NT-CORE (GWT) + NT-IO (provider routing). Direct implementation of Axiom A1 (Cost-Aware Routing). Builds "experience memory" of routing decisions — maps to NT-NEXUS cross-session memory. |
| **Key pattern** | **Cold-start experience memory** — executes both systems on a shared seed set, builds compact memory, retrieves similar cases at inference time. |

### 5. MRAgent — Graph Memory with Active Reconstruction
| Field | Detail |
|-------|--------|
| **URL** | arXiv:2606.06036 (ICML 2026) |
| **Category** | Agent Memory / Long-Horizon Reasoning |
| **What it does** | Combines associative memory graph (Cue-Tag-Content) with active reconstruction. Agent iteratively explores and prunes retrieval paths based on accumulated evidence — memory adapts to reasoning context. +23% over baselines. |
| **NeoTrix mapping** | NT-MEMORY (KB graph) + NT-CORE (reasoning). The Cue-Tag-Content graph maps to NeoTrix's KB node/edge/embedding structure. Active reconstruction = dynamic VSA HyperCube traversal. |
| **Key pattern** | **Reconstruction > Retrieval** — memory is dynamically reconstructed during reasoning, not retrieved as static snapshots. |

### 6. NS-Mem — Neuro-Symbolic Memory
| Field | Detail |
|-------|--------|
| **URL** | arXiv:2603.15280 |
| **Category** | Multimodal Agent Memory |
| **What it does** | Three-layer memory: episodic + semantic + logic rule layer. SK-Gen consolidates structured knowledge from multimodal experiences. Hybrid retrieval: similarity search + deterministic symbolic query. +4.35% overall, +12.5% on constrained reasoning. |
| **NeoTrix mapping** | NT-MEMORY (KB) + NT-CORE (E8 reasoning). Three-layer memory maps to NeoTrix's KB namespaces (episodic experience, semantic domain, logic rules). Symbolic rules = E8 hexagram patterns. |
| **Key pattern** | **Hybrid retrieval** — combines neural similarity (inductive) with symbolic query (deductive) in a single retrieval pass. |

### 7. RAGEN-2 — Reasoning Collapse in Agentic RL
| Field | Detail |
|-------|--------|
| **URL** | arXiv:2604.06268 (ICML 2026 Oral) |
| **Category** | Agent Training / RL |
| **What it does** | Identifies "template collapse" — agents produce fluent but input-agnostic reasoning. Diagnoses via mutual information (MI) proxy. Proposes SNR-Aware Filtering to select high-variance prompts per iteration. |
| **NeoTrix mapping** | NT-MIND (SEAL self-evolution) + NT-CORE (reasoning quality). Template collapse is exactly the "Echo Trap" — maps to SEAL pipeline monitoring where reasoning quality must be input-dependent, not just diverse. |
| **Key pattern** | **MI > Entropy** — cross-input distinguishability (MI) predicts performance better than within-input diversity (entropy). |

### 8. FreeLLM API — Smart Provider Routing
| Field | Detail |
|-------|--------|
| **URL** | github.com/tashfeenahmed/freellmapi |
| **Stars** | 114+ |
| **Language** | Python |
| **Category** | LLM API Proxy / Cost Optimization |
| **What it does** | OpenAI-compatible proxy aggregating free tiers from 28 LLM providers behind a single endpoint. Smart routing + failover. |
| **NeoTrix mapping** | NT-IO (provider management). Direct implementation of P4 (Ordered Backend Fallback) and Axiom A1 (Cost-Aware Routing). |
| **Key pattern** | **Provider aggregation** — unified interface with ordered fallback across 28 providers, zero external API cost. |

### 9. LoopX — Durable Agent State Kernel
| Field | Detail |
|-------|--------|
| **URL** | github.com/huangruiteng/loopx |
| **Stars** | 624+ |
| **Category** | Agent Infrastructure / State Management |
| **What it does** | Lightweight loop-engineering state kernel for durable, quota-aware, long-running AI agent teams. Manages agent lifecycle, quota budgets, and inter-agent state. |
| **NeoTrix mapping** | NT-ACT (orchestration) + NT-MIND (SEAL pipeline state). The "loop kernel" pattern maps to SEAL pipeline execution with quota awareness. |
| **Key pattern** | **Quota-aware loop** — agent execution bounded by resource quotas with automatic pause/resume across sessions. |

### 10. Flare — Graph-First IDE for Agentic Coding
| Field | Detail |
|-------|--------|
| **URL** | ProductHunt (Sep 2026) |
| **Category** | IDE / Developer Tools |
| **What it does** | Graph-first IDE and interactive map for agentic coding. Visualizes code relationships as a navigable graph, enabling agents and humans to understand codebase topology. |
| **NeoTrix mapping** | NT-CORE (HyperCube visualization) + NT-WORLD (codebase exploration). Graph-first visualization = HyperCube as navigable topology. |
| **Key pattern** | **Topology-as-UI** — codebase structure as interactive graph, not file tree. |

---

## Cross-Cutting Patterns

| Pattern | Projects | NeoTrix Mapping |
|---------|----------|-----------------|
| **Agent Memory as Infrastructure** | screenpipe, MRAgent, NS-Mem | NT-MEMORY KB namespaces |
| **Routing > Raw Capability** | Agora, BoundaryRouter, FreeLLM | GWT salience + cost weight (A1) |
| **Sandboxed Execution** | OpenHands, LoopX | NT-SHIELD + worktree isolation |
| **Collapse Detection** | RAGEN-2 | SEAL pipeline self-monitoring |
| **Always-On Local Context** | screenpipe | PerceptionBridge for ambient input |

---

## Trend Summary

The dominant signal in cycle 356: **agent memory is becoming its own infrastructure layer**. Projects are splitting "what agents remember" from "how agents act" — memory is now a standalone data management system with its own retrieval, update, and maintenance lifecycle. This aligns with NeoTrix's KB-as-shared-state architecture.

The second signal: **routing and cost awareness are table stakes**. BoundaryRouter and Agora show that the choice of which model to use for which query is now a first-class architectural concern, not an afterthought. This validates Axiom A1 (Cost-Aware Routing).
