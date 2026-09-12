# Trending Rankings — Cycle 363 (2026-09-12)

## Search Scope
GitHub Trending, ProductHunt, arXiv — AI agents, LLM tools, reasoning frameworks, memory/attention/routing patterns. Live web searches on 2026-09-12.

---

## 10 New Projects (Not in Cycles 318–362)

### 1. OpenClaw — Open-Source Personal AI Assistant
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/openclaw/openclaw |
| **Stars** | 389,000+ |
| **Language** | TypeScript/Swift/Go |
| **Category** | Autonomous Agent Framework |
| **What it does** | Cross-platform AI assistant that executes real-world tasks: email, smart home, scheduling, web browsing, voice interaction. Heartbeat Engine for proactive monitoring. Skills marketplace with community-contributed capabilities. Peekaboo for macOS screenshot/vision MCP. ACP (Agent Client Protocol) for stateful headless sessions. Lobster workflow shell for composable pipelines. Overtook React in GitHub stars (Aug 2026). |
| **NeoTrix mapping** | NT-IO (cross-platform interface) + NT-ACT (real-world action) + NT-SHIELD (local execution). OpenClaw = NT-IO's multi-platform agent with NT-ACT's tool orchestration. Heartbeat Engine = HeartbeatAggregator pattern (system health monitoring). Skills marketplace = NT-MIND's skill crystallization with community marketplace. ACP sessions = NT-NEXUS cross-session stateful context. |
| **Key pattern** | **Heartbeat Engine as proactive monitoring** — the agent doesn't just respond; it proactively monitors system state and triggers actions. This maps to NT-CORE's ConsciousnessTree: continuous background health monitoring that triggers attention shifts. The skills marketplace validates NT-MIND's skill crystallization — community contributions = decentralized SEAL pipeline. |

### 2. Solace Agent Mesh — Event-Driven Multi-Agent Orchestration
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/SolaceLabs/solace-agent-mesh |
| **Stars** | 5,000+ |
| **Language** | Go (runtime) / Python (tools) |
| **Category** | Event-Driven Agent Platform |
| **What it does** | Enterprise-grade multi-agent platform built on Solace Event Broker. Decomposes requests into tasks, dispatches to specialist agents via A2A protocol. Asynchronous execution with horizontal scaling. Data stays local — agents store data in in-memory databases, LLM only receives schemas for query generation. Entrypoints for auth/access control. MCP + A2A support. Go runtime for performance. |
| **NeoTrix mapping** | NT-ACT (orchestration) + NT-MEMORY (local data management) + NT-SHIELD (access control). Solace = NT-ACT's cross-domain routing via event mesh (EventBus pattern). Local data + schema-only LLM = Egress Privacy Guard pattern (never send raw data to external models). Entrypoints = NT-SHIELD's trust boundary enforcement. Go runtime = NT-PHYSICAL's hardware-aware execution. |
| **Key pattern** | **Data-local, schema-remote** — agents store data locally and only share schemas/metadata with LLMs. The LLM generates queries, not processes raw data. This is the Egress Privacy Guard pattern formalized: trust boundary between local data and external inference. Reduces token cost + preserves privacy. Validates NeoTrix's architecture: KB stays local, only embeddings cross trust boundaries. |

### 3. Julep — Durable Composable AI Agents
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/julep-ai/julep |
| **Stars** | 6,591+ |
| **Language** | Python |
| **Category** | Durable Agent Runtime |
| **What it does** | Agents as composable, durable dataflows — not ad-hoc loops. Flows crash and resume, retry safely, explain every step. @flow decorator defines graph steps at definition time; tools, reasoners, branches, fan-out compile to frozen wire-format IR. CLI: `julep ls/graph/run/lint/test/trace/deploy`. Cross-agent DAG discovery. Temporal layer optional. Vault integration for production secrets. MCP policy behavior. |
| **NeoTrix mapping** | NT-ACT (durable workflows) + NT-MEMORY (trace persistence) + NT-SHIELD (vault/secrets). Julep = NT-ACT's SEAL pipeline with durable execution semantics. @flow = declarative pipeline definition (like SEAL stage DAG). Trace tree = experience-tree's full session trace. CLI `julep graph` = CapabilityTree visualization. Vault = NT-SHIELD's secret management. |
| **Key pattern** | **Durable dataflow as agent primitive** — agents are not request-response loops but durable dataflows that survive crashes. The frozen wire-format IR is a compiled pipeline, not interpreted at runtime. Maps to NT-ACT's compiled skill nodes (C4 constellation): once compiled, the pipeline runs without re-interpretation. Crash-resume = NT-REPAIR's self-healing at the workflow level. |

### 4. MagiCrew — Open-Source AI Workforce Platform
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/MagiCrew-ai/magicrew |
| **Stars** | ~2,500+ (new, Sep 3 2026) |
| **Language** | TypeScript |
| **Category** | Multi-Agent Workforce |
| **What it does** | Deploy specialized digital workers that research, analyze, create reports, generate presentations, and complete real business tasks. Multi-agent collaboration with enterprise controls. Deliverable-ready outputs. Turns AI from a tool into a workforce you manage. ProductHunt #1 daily (Sep 3 2026, score 269). |
| **NeoTrix mapping** | NT-ACT (multi-agent orchestration) + NT-MEMORY (deliverable persistence). MagiCrew = NT-ACT's cross-domain specialization with workforce management. Digital workers = NT-* domain specialists. Deliverable-ready outputs = experience-tree's structured output contracts. Enterprise controls = NT-SHIELD's governance layer. |
| **Key pattern** | **Workforce-as-a-service** — agents are not tools but managed employees with roles, responsibilities, and deliverables. This maps to NT-* domain architecture: each domain is a specialist worker, not a feature. The management layer (enterprise controls) is the NT-GOVERNANCE function. |

### 5. Construct Computer — AI Coworker with Its Own Computer
| Field | Detail |
|-------|--------|
| **URL** | https://www.construct.computer |
| **Stars** | ~1,200+ (new, Aug 23 2026) |
| **Language** | Cloud platform |
| **Category** | Cloud Agent Runtime |
| **What it does** | Your AI coworker gets a dedicated cloud computer. Runs autonomously with full OS access. Persistent state across sessions. Handles file operations, browser automation, code execution. Your day back by delegating repetitive work. |
| **NeoTrix mapping** | NT-PHYSICAL (embodied agent) + NT-ACT (autonomous action). Construct = NT-PHYSICAL's body schema realized as cloud VM. Persistent state = NT-NEXUS cross-session memory. Full OS access = NT-SHIELD's capability boundary (trust delegation to cloud). |
| **Key pattern** | **Embodied agent as cloud VM** — the agent has a physical body (cloud computer) with persistent state. This is NT-PHYSICAL's body schema principle: an agent without embodiment is a disembodied reasoner. Construct gives agents a persistent computational body, enabling long-running autonomous workflows. |

### 6. Dropstone — AI Runtime That Remembers, Learns, Acts Everywhere
| Field | Detail |
|-------|--------|
| **URL** | https://dropstone.ai |
| **Stars** | ~800+ (new, Sep 2026) |
| **Language** | Platform |
| **Category** | Persistent Agent Runtime |
| **What it does** | AI runtime with persistent memory, learning, and action across IDE integrations. Remembers context across sessions, learns from interactions, acts on learned patterns. 5/5 rating on ProductHunt. Integrates across development environments. |
| **NeoTrix mapping** | NT-NEXUS (cross-session persistence) + NT-MIND (self-improvement). Dropstone = NT-NEXUS's cross-session memory with NT-MIND's learning loop. Persistent memory = KB kv_store with session continuity. Learning from interactions = experience-tree absorption. |
| **Key pattern** | **Remember-learn-act loop** — memory informs learning, learning enables better action, action produces new memory. This is the SEAL pipeline's feedback loop: exploration → experience → distillation → better exploration. The triple guarantee (memory + learning + action) prevents the common failure mode of agents that remember but don't learn, or learn but don't act. |

### 7. Monid — Universal API Gateway for AI Agents
| Field | Detail |
|-------|--------|
| **URL** | https://monid.ai |
| **Stars** | ~468 upvotes (ProductHunt #1, Sep 2 2026) |
| **Language** | Platform |
| **Category** | Agent Tool Infrastructure |
| **What it does** | Connect AI agents to 1,800+ APIs without subscriptions. SEO, lead gen, video/music generation, social media, stocks, market trends, on-chain data, competitor tracking, sentiment analysis. One API key for all. |
| **NeoTrix mapping** | NT-ACT (tool access) + NT-IO (unified interface). Monid = NT-ACT's tool registry with unified access layer. 1,800+ APIs = NT-ACT's capability registry at scale. One API key = single trust boundary for all external tools. |
| **Key pattern** | **Unified tool access with single trust boundary** — instead of per-API authentication, one key grants access to all. This is NT-ACT's CapabilityRegistry pattern: a single interface with ordered backend fallback (Pattern P4). The trust boundary is centralized, not distributed. |

### 8. Clockwork — Calendar Where AI Agents Show Up for Work
| Field | Detail |
|-------|--------|
| **URL** | https://clockwork.ai |
| **Stars** | ~500+ (new, Sep 2026) |
| **Language** | Platform |
| **Category** | Agent Scheduling |
| **What it does** | AI-native calendar that schedules agent tasks alongside human meetings. Agents have time blocks, priorities, and deadlines. Visual interface showing agent workloads. Integration with task management and project tracking. |
| **NeoTrix mapping** | NT-ACT (task scheduling) + NT-META (priority management). Clockwork = NT-ACT's task scheduling with NT-META's attention allocation. Time blocks = GWT's temporal attention windows. Agent workloads = resource allocation across NT-* domains. |
| **Key pattern** | **Temporal resource allocation for agents** — agents don't just execute tasks; they need scheduled time blocks with priorities. This maps to GWT's attention routing: the system must decide WHEN each domain gets attention, not just WHAT it does. Clockwork makes the temporal dimension of agent coordination explicit. |

### 9. Inferock Bench — Independent Receipt for LLM API Calls
| Field | Detail |
|-------|--------|
| **URL** | https://inferock.com |
| **Stars** | ~300+ (new, Aug 15 2026) |
| **Language** | Platform |
| **Category** | LLM Cost Transparency |
| **What it does** | Independent verification receipt for every LLM API call. Proves what model was used, what tokens were consumed, and what the actual cost was. Prevents vendor lock-in and enables cost comparison. Transparency layer between agents and providers. |
| **NeoTrix mapping** | NT-IO (provider transparency) + NT-SHIELD (audit trail). Inferock = NT-IO's provider interface with NT-SHIELD's audit logging. Independent receipts = EventBus event logging with tamper-proof verification. Cost transparency = Axiom A1 (Cost-Aware Routing) with proof. |
| **Key pattern** | **Tamper-proof API receipts** — independent verification of what happened between agent and provider. This is NT-SHIELD's audit trail principle applied to LLM API calls. Every call produces a verifiable receipt, preventing vendor manipulation and enabling true cost-aware routing (Axiom A1). |

### 10. Konig — Graph-First IDE for Agentic Coding
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/flarelabs-net/konig |
| **Stars** | ~1,500+ (new, Aug 2026) |
| **Language** | TypeScript |
| **Category** | Code Intelligence IDE |
| **What it does** | Graph-first IDE that visualizes code relationships as an interactive map. Agents navigate the code graph to understand dependencies, call chains, and impact. Interactive exploration of codebase structure. Graph-based code intelligence for AI coding agents. |
| **NeoTrix mapping** | NT-MEMORY (code graph) + NT-CORE (graph reasoning). Konig = NT-MEMORY's semantic indexing with graph visualization. Interactive code graph = KB node-edge structure made visible. Agent navigation = GWT attention routing over code graph. |
| **Key pattern** | **Code as navigable graph** — code relationships are exposed as a graph that agents can traverse. This maps to NT-CORE's E8 hexagram reasoning: the codebase is a graph, and reasoning is graph traversal. Konig makes the implicit code graph explicit, enabling agents to make better routing decisions based on structural understanding. |

---

## Cross-Cutting Patterns (Cycle 363)

### 1. Durable Execution as First-Class Primitive
- **Julep**: durable dataflows with crash-resume
- **Construct Computer**: persistent cloud VM for agents
- **Dropstone**: persistent runtime with memory
- Pattern: agents must survive crashes and maintain state across sessions, not just respond to requests

### 2. Data-Local, Schema-Remote
- **Solace Agent Mesh**: data stays local, only schemas to LLM
- **OpenClaw**: local execution with Heartbeat Engine
- **Inferock**: independent receipts for verification
- Pattern: trust boundary between local data and external inference — the Egress Privacy Guard principle

### 3. Embodied Agents Need Physical Bodies
- **Construct Computer**: cloud VM as agent body
- **OpenClaw**: cross-platform with persistent state
- **Clockwork**: temporal scheduling for agent workloads
- Pattern: agents without persistent computational bodies are limited to request-response; embodiment enables autonomous long-running work

### 4. Workforce-as-Managed-Specialists
- **MagiCrew**: managed digital workers with deliverables
- **Monid**: unified API access for 1,800+ tools
- **Konig**: graph-based code intelligence
- Pattern: agents are not generalists but managed specialists with clear roles and tool access

### 5. Transparency and Audit as Trust Foundations
- **Inferock**: tamper-proof API receipts
- **Solace**: event-driven observability
- **Julep**: trace trees for every execution
- Pattern: trust requires verifiable transparency — every action produces a receipt, trace, or audit trail
