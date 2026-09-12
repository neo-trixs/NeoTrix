# Trending Rankings — Cycle 426 (2026-09-12)

## Methodology
- GitHub trending (AI agents, LLM tools, reasoning frameworks)
- ProductHunt September 2026 launches
- arXiv September 2026 papers
- Cross-referenced against cycles 318–425 to ensure novelty

---

## Top 10 New Projects

### 1. Colibri — Pure-C Inference Engine for MoE on Consumer Hardware
| Field | Value |
|-------|-------|
| **Platform** | GitHub trending Jul 2026 |
| **Stars** | ~14.7K |
| **Repo** | [JustVugg/colibri](https://github.com/JustVugg/colibri) |
| **Lang** | C |

**What it does:** A tiny, pure-C inference engine with zero dependencies that runs GLM-5.2 (744B MoE) on a consumer machine with ~25GB RAM by streaming experts from disk as needed. No GPU required — disk-streamed mixture-of-experts inference.

**NeoTrix mapping:**
- NT-PHYSICAL: Edge inference on consumer hardware → physical layer inference optimization
- NT-IO: Zero-dependency single-binary → minimal runtime surface
- NT-CORE (A2): Context as Scarce Resource → memory-mapped expert streaming as KV optimization analog
- NT-SHIELD: Local-first privacy → no data leaves device

**Novel Signal:** Disk-streamed MoE is the hardware analog of KVMem's paged KV — both decompose a model's working set into hot/cold tiers. Colibri proves you can run 744B parameters on 25GB RAM by treating expert weights as a pageable resource.

---

### 2. LoopX — Durable State Kernel for Long-Running Agent Teams
| Field | Value |
|-------|-------|
| **Platform** | GitHub trending Aug 8, 2026 |
| **Stars** | +624/day |
| **Repo** | [huangruiteng/loopx](https://github.com/huangruiteng/loopx) |
| **Lang** | — |

**What it does:** A lightweight loop-engineering state kernel for durable, quota-aware, long-running AI agent teams. Manages state persistence across agent loops with quota enforcement.

**NeoTrix mapping:**
- NT-ACT: Agent loop orchestration → SEAL pipeline stage management
- NT-MEMORY: Durable state persistence → KB experience write-ahead log
- NT-CORE (A2): Quota-aware execution → context budget management
- NT-MIND: Loop engineering → self-evolution cycle tracking

**Novel Signal:** "Loop engineering" as a first-class concept. Most agent frameworks treat loops as implicit; LoopX makes the state machine explicit with quota awareness — a missing primitive in long-running agent systems.

---

### 3. Sovereign Stack — MCP Server for AI Memory Across Session-Death
| Field | Value |
|-------|-------|
| **Platform** | GitHub autonomous-agents topic |
| **Repo** | [templetwo/sovereign-stack](https://github.com/templetwo/sovereign-stack) |
| **Category** | AI Memory + Governance |

**What it does:** MCP server for AI memory, governance, and continuity across session-death. A self-verifying chronicle: honest on write, on read, on reach, and about itself. Failed writes can't report success, partial answers can't pass as complete, capability surface generated from live registry, not typed. 100% local.

**NeoTrix mapping:**
- NT-MEMORY: Self-verifying KB → KB write/read integrity guarantees
- NT-SHIELD: Governance layer → audit trail + capability surface verification
- NT-CORE: ConsciousnessTree health chain → self-verification of system state
- NT-NEXUS: Cross-session continuity → experience persistence across sessions

**Novel Signal:** "Self-verifying chronicle" — a memory system that guarantees write-read consistency. The three invariants (no false writes, no partial completions, live-not-typed capabilities) map directly to KB integrity requirements.

---

### 4. Agora — Auction-Based Task Allocation for LLM Agent Reasoning
| Field | Value |
|-------|-------|
| **Platform** | arXiv 2607.09600 |
| **Date** | 2026-08-30 |
| **Category** | Multi-Agent Coordination |

**What it does:** Uses confidence-calibrated auctions to dynamically route reasoning steps to expert models/tools. Treats reasoning steps as tradeable items; allocation based on calibrated competence rather than raw confidence. Without reliable success probability measures, dynamic allocation risks assigning critical logic to overconfident but incompetent agents.

**NeoTrix mapping:**
- NT-CORE (GWT): Auction-based salience → confidence-calibrated attention routing
- NT-ACT: Task delegation → capability registry routing with competence scoring
- NT-MIND: Meta-reasoning → self-assessment of competence calibration
- NT-CORE (A1): Cost-Aware Routing → quality-cost tradeoff tunable via auction price

**Novel Signal:** Auction mechanisms for agent task allocation. The confidence-calibration insight is critical — raw confidence is unreliable, calibrated competence enables better routing decisions. This is GWT with market dynamics.

---

### 5. MKA — Memory-Keyed Attention for Hierarchical KV Routing
| Field | Value |
|-------|-------|
| **Platform** | arXiv 2603.20586 (ACM SIGMOD CF'26) |
| **Date** | 2026-05-19 |
| **Category** | Efficient Attention |

**What it does:** Hierarchical attention mechanism integrating multi-level KV caches (local L1, session L2, long-term L3) with learned routing gates that dynamically route each query across memory tiers. FastMKA variant fuses memory sources before attention computation for efficiency.

**NeoTrix mapping:**
- NT-CORE (GWT): Multi-tier attention routing → GWT salience across module hierarchy
- NT-MEMORY: 3-tier KB → hot/warm/cold knowledge tiers with routing
- NT-NEXUS: Session memory → cross-session experience routing
- NT-PHYSICAL: Hardware-friendly routing → memory-bandwidth-aware design

**Novel Signal:** Hierarchical KV routing across memory tiers is exactly the hot/cold KV tiering problem (KVMem) but solved at the attention mechanism level rather than the KV cache level. MKA routes attention itself, not just KV blocks.

---

### 6. Flux Attention — Context-Aware Hybrid Attention Router
| Field | Value |
|-------|-------|
| **Platform** | arXiv 2604.07394 |
| **Date** | 2026-04-08 |
| **Category** | Efficient Inference |

**What it does:** Layer-level context-aware routing between Full Attention and Sparse Attention. A lightweight Layer Router is inserted into frozen pretrained LLMs; each layer adaptively routes to FA or SA based on input context. Only 12 hours training on 8×A800 GPUs. Achieves 2.8× prefill and 2.0× decode speedup.

**NeoTrix mapping:**
- NT-CORE (GWT): Layer-wise attention routing → per-module attention mode selection
- NT-MIND: Parameter-efficient adaptation → SEAL pipeline lightweight tuning
- NT-PHYSICAL: Memory access pattern optimization → contiguous memory for hardware acceleration
- NT-CORE (A1): Cost-Aware Routing → adaptive FA/SA selection based on task complexity

**Novel Signal:** Routing attention mode per-layer based on context content. Not all layers need the same attention fidelity — Flux Attention proves you can freeze the model and only add a lightweight router. This is the "Ordered Backend Router" pattern applied to attention mechanisms.

---

### 7. GNAP — Git-Native Agent Protocol
| Field | Value |
|-------|-------|
| **Platform** | GitHub awesome-ai-agents-2026 |
| **Repo** | caramaschiHG/awesome-ai-agents-2026 |
| **Category** | Agent Coordination Protocol |

**What it does:** Coordinate AI agent teams with 4 JSON files in a git repo. No server, no database. Any agent that can `git push` can participate. MIT licensed. Git as the coordination substrate.

**NeoTrix mapping:**
- NT-ACT: Agent coordination → distributed agent team orchestration
- NT-MEMORY: Git as state store → KB as version-controlled knowledge
- NT-NEXUS: Cross-agent state sharing → experience graph persistence
- NT-SHIELD: No server = no central attack surface → decentralized security

**Novel Signal:** Git as the universal coordination primitive for agent teams. The insight is that git already solves distributed consensus, versioning, and conflict resolution — why build a new coordination layer? GNAP makes agent coordination a git-native operation.

---

### 8. Construct Computer — Your AI Coworker Gets a Computer
| Field | Value |
|-------|-------|
| **Platform** | ProductHunt Aug 23, 2026 |
| **Category** | AI Infrastructure |
| **Score** | Top 5 of day |

**What it does:** Gives AI agents a persistent, sandboxed computer environment. The agent gets a desktop, terminal, browser, and file system — a full computing context rather than isolated API calls. "Your AI coworker gets a computer. You get your day back."

**NeoTrix mapping:**
- NT-PHYSICAL: Embodied computing environment → physical layer agent body
- NT-WORLD: Persistent environment → world model with state persistence
- NT-SHIELD: Sandboxed execution → safety kernel for agent actions
- NT-IO: Desktop + terminal + browser → multi-modal interface domain

**Novel Signal:** "Agent computer" as a product category. The shift from "agent calls tools" to "agent inhabits an environment" represents the embodiment layer in NeoTrix's 3-layer architecture becoming a concrete product.

---

### 9. NewsMCP — Event-Deduplicated News MCP Server
| Field | Value |
|-------|-------|
| **Platform** | ProductHunt Sep 11, 2026 |
| **Category** | MCP Server |
| **Score** | 13 |

**What it does:** MCP server that deduplicates news by event identity. Multiple articles reporting the same event are merged into a single news event with structured metadata. Returns deduplicated event units rather than raw article dumps.

**NeoTrix mapping:**
- NT-WORLD: Content deduplication → crawl pipeline entity resolution
- NT-MEMORY: Event identity → KB node deduplication and entity linking
- NT-CORE: Semantic clustering → HyperCube concept binding
- NT-ACT: Agent-ready structured output → capability registry input format

**Novel Signal:** Event-level deduplication for agent-consumed news. The shift from "article feed" to "event stream" reduces agent context consumption while preserving information density. This is content compression for agent consumption.

---

### 10. HarnessRouter — Open-Source Unified Interface for Agent Harnesses
| Field | Value |
|-------|-------|
| **Platform** | ProductHunt Aug 16, 2026 |
| **Category** | Agent Infrastructure |
| **URL** | Community Edition |

**What it does:** Open-source unified interface for agent harnesses (Claude Code, Codex CLI, Cursor, Copilot, Gemini CLI). Single interface to switch between agent backends, manage sessions, and route tasks.

**NeoTrix mapping:**
- NT-IO: Unified agent interface → provider routing abstraction
- NT-ACT: Harness switching → capability registry backend selection
- NT-CORE (P4): Ordered Backend Fallback → harness selection with fallback chain
- NT-SHIELD: Session management → sandbox isolation per harness

**Novel Signal:** The "harness abstraction layer" — treating different coding agent CLIs as interchangeable backends behind a unified interface. This is the Ordered Backend Router pattern (P4) applied to agent harnesses rather than search backends.

---

## Cross-Cutting Patterns

### Pattern 1: Memory as Routing Primitive
MKA, Sovereign Stack, and LoopX all treat memory not as passive storage but as an active routing substrate. Memory tiers determine how queries are processed, not just what's stored. This maps to NeoTrix's KB with hot/warm/cold tiers and GWT salience routing.

### Pattern 2: Git as Coordination Substrate
GNAP and Sovereign Stack both leverage git as the coordination layer for distributed agent systems. Git solves consensus, versioning, and conflict resolution — agent coordination becomes a git operation rather than a custom protocol.

### Pattern 3: Environment Embodiment
Construct Computer and Colibri both push agent execution beyond API calls into persistent environments. The agent inhabits a computing context (desktop or disk-streamed MoE) rather than making stateless function calls.

### Pattern 4: Confidence Calibration Over Raw Confidence
Agora's core insight — calibrated competence beats raw confidence — applies across all routing decisions. GWT salience, task delegation, and attention routing all benefit from calibration rather than raw score comparison.

### Pattern 5: Event-Level Deduplication for Agents
NewsMCP and Sovereign Stack both reduce agent context consumption through semantic deduplication. Agents don't need every article — they need deduplicated events with structured metadata.

---

## Priority Absorption Candidates

| Priority | Project | Pattern | NeoTrix Domain | Impact |
|----------|---------|---------|---------------|--------|
| P0 | MKA 3-tier routing | Hierarchical KV routing | NT-CORE (GWT) + NT-MEMORY | Direct GWT enhancement |
| P0 | Agora auction routing | Confidence-calibrated allocation | NT-CORE (GWT + A1) | GWT salience improvement |
| P1 | Sovereign Stack verification | Self-verifying memory | NT-MEMORY + NT-SHIELD | KB integrity guarantees |
| P1 | GNAP git coordination | Git-native agent protocol | NT-ACT + NT-NEXUS | Distributed coordination |
| P1 | Flux Attention router | Context-aware attention mode | NT-CORE (GWT) | Attention cost optimization |
| P2 | LoopX loop engineering | Durable agent state | NT-ACT + NT-MIND | SEAL pipeline state |
| P2 | NewsMCP event dedup | Agent content compression | NT-WORLD + NT-MEMORY | Crawl pipeline efficiency |
| P2 | Colibri disk-streamed MoE | Edge inference on consumer HW | NT-PHYSICAL | Physical layer optimization |
