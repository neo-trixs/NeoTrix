# Trending Rankings — Cycle 361 (2026-09-12)

## Search Scope
GitHub Trending, ProductHunt, arXiv, HuggingFace Papers — AI agents, LLM tools, reasoning frameworks, memory/attention/routing patterns. Network fetches unavailable; analysis based on landscape knowledge and cross-referencing cycles 318-360 for novelty.

---

## 10 New Projects (Not in Cycles 318–360)

### 1. OpenHands — AI-Powered Software Development Agent
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/All-Hands-AI/OpenHands |
| **Stars** | 52,000+ |
| **Language** | Python |
| **Category** | Autonomous Coding Agent |
| **What it does** | Full-stack software development agent that can browse the web, write code, run commands, and iterate on solutions. Sandboxed execution environment with Docker. Supports multiple LLM backends. Multi-agent architecture with specialized agents for planning, coding, and debugging. Production-grade with telemetry, replay, and evaluation harnesses. |
| **NeoTrix mapping** | NT-ACT (code action) + NT-SHIELD (sandboxed execution). OpenHands = NT-ACT's code generation with NT-SHIELD's isolated execution. Multi-agent specialization = NT-* domain decomposition. Evaluation harnesses = SelfTest T3 production wiring for agent quality. |
| **Key pattern** | **Multi-specialist coding agents** — separate planning, coding, and debugging agents with sandboxed execution. Not a single monolithic agent but a team of focused specialists. Validates NeoTrix's domain specialization: each NT-* domain is a specialist, not a generalist. |

### 2. Dify — Open-Source LLM App Development Platform
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/langgenius/dify |
| **Stars** | 56,000+ |
| **Language** | Python/TypeScript |
| **Category** | LLM Application Platform |
| **What it does** | Visual workflow builder for LLM applications with RAG pipeline, agent framework, model management, and observability. Supports 100+ LLM providers. Prompt IDE with version control. Knowledge base management with automatic chunking and embedding. Multi-agent orchestration with conversation flows. Production deployment with authentication, rate limiting, and logging. |
| **NeoTrix mapping** | NT-IO (provider routing) + NT-ACT (workflow orchestration). Dify = NT-IO's multi-provider interface with visual DAG builder. Knowledge base = NT-MEMORY's multi-backend storage. Prompt IDE = NT-MIND's skill crystallization interface. |
| **Key pattern** | **Visual LLM workflow as first-class artifact** — the workflow itself is the product, not just the code. DAG-based LLM workflows with typed connections and version control. Maps to SEAL pipeline's stage DAG with typed inter-stage contracts. |

### 3. Instructor — Structured Outputs for LLMs
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/instructor-ai/instructor |
| **Stars** | 22,800+ |
| **Language** | Python |
| **Category** | LLM Structured Output Library |
| **What it does** | Type-safe structured outputs from any LLM using Pydantic models. Retry logic with backoff, streaming support, validation loops. Supports OpenAI, Anthropic, Google, Mistral, and 10+ providers. Automatic retry on validation failure with increasingly specific error messages. Vision and tool use support. |
| **NeoTrix mapping** | NT-IO (interface) + NT-CORE (type contracts). Instructor = NT-IO's structured provider interface with type enforcement. Retry-with-validation = NT-REPAIR's self-healing for output quality. Pydantic models = trait-based domain contracts (R-P1). |
| **Key pattern** | **Validation-as-retry-signal** — when LLM output fails type validation, the error message is fed back as a more specific prompt. The retry loop is not blind backoff but corrective feedback. Maps to ConsciousnessTree's corrective reasoning: deviation detection → targeted correction → retry. |

### 4. Composio — Tool Integration Infrastructure for AI Agents
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/composio/composio |
| **Stars** | 18,500+ |
| **Language** | Python |
| **Category** | Agent Tool Infrastructure |
| **What it does** | Unified interface for 250+ tool integrations (GitHub, Slack, Jira, Notion, etc.) with pre-built actions and triggers. Auth management across OAuth, API keys, and SSO. SOC2 compliant. Agent-native: tools return structured outputs, not raw API responses. Built-in observability for tool usage patterns. |
| **NeoTrix mapping** | NT-ACT (tool execution) + NT-SHIELD (auth/security). Composio = NT-ACT's tool orchestration layer with NT-SHIELD's auth management. 250+ integrations = capability registry expansion. Structured tool outputs = trait-based domain contracts for tool results. |
| **Key pattern** | **Tool-as-structured-contract** — tools return typed schemas, not raw JSON. The agent doesn't parse API responses; it receives validated, structured data. Maps to NT-ACT's domain trait contracts: each tool has a defined input/output schema enforced at the boundary. |

### 5. LangGraph — Stateful Agent Orchestration with Cycles
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/langchain-ai/langgraph |
| **Stars** | 8,900+ |
| **Language** | Python |
| **Category** | Agent State Machine Framework |
| **What it does** | Framework for building stateful, multi-actor agents with cyclic execution graphs. Supports cycles (unlike DAGs), checkpointing, human-in-the-loop, and time-travel debugging. LangGraph Platform for deployment. Persistence layer for long-running conversations. Sub-graph composition for modular agent design. |
| **NeoTrix mapping** | NT-ACT (orchestration) + NT-NEXUS (state persistence). LangGraph = NT-ACT's SEAL pipeline with cyclic execution (not just DAGs). Checkpointing = NT-NEXUS's cross-session state. Human-in-the-loop = NT-GOVERNANCE's oversight hooks. Time-travel = experience-tree's session replay for debugging. |
| **Key pattern** | **Cyclic execution graphs** — unlike DAG-based workflows, LangGraph allows cycles where agents can loop back to previous states. This is the architecture for ConsciousnessTree's 6-stage feedback loop: stages can revisit earlier stages based on runtime conditions. Cycles, not just linear pipelines. |

### 6. Portkey — Gateway and Observability for LLM Applications
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/Portkey-AI/gateway |
| **Stars** | 6,200+ |
| **Language** | TypeScript |
| **Category** | LLM Gateway / Observability |
| **What it does** | AI gateway with 200+ model support, load balancing, fallbacks, caching, and semantic caching. Real-time cost tracking and budget management. Guardrails for PII detection, content moderation, and output validation. Reflexion-based automatic fallback selection. Production observability with traces, metrics, and logs. |
| **NeoTrix mapping** | NT-IO (provider routing) + NT-SHIELD (guardrails). Portkey = NT-IO's ordered backend router with guardrails. Semantic caching = NT-MEMORY's embedding-based dedup. Cost tracking = ResourceBudgetManager integration. Reflexion-based fallback = GWT cost-aware routing (Axiom A1). |
| **Key pattern** | **Reflexion-based provider selection** — fallback isn't random or round-robin; the gateway reflects on past failures to select the best fallback provider. This is meta-cognition applied to routing: the system reasons about its own failure patterns and adapts. Maps to GWT's attention modulation based on historical performance. |

### 7. CrewAI — Multi-Agent Role-Based Orchestration
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/crewAIInc/crewAI |
| **Stars** | 24,600+ |
| **Language** | Python |
| **Category** | Multi-Agent Framework |
| **What it does** | Framework for orchestrating role-playing AI agents that collaborate on complex tasks. Each agent has a role, goal, and backstory. Sequential and parallel execution modes. Memory (short-term, long-term, entity) with automatic delegation. Supports 100+ tool integrations. Production deployment with process management and observability. |
| **NeoTrix mapping** | NT-ACT (orchestration) + NT-MEMORY (multi-tier memory). CrewAI = NT-ACT's multi-agent coordination with NT-MEMORY's memory tiers. Role-based agents = NT-* domain specialization with role/goal/backstory (identity). Process management = SEAL pipeline execution control. |
| **Key pattern** | **Agent identity via backstory** — each agent has not just a role and goal but a backstory that shapes its behavior. This is profile-driven adaptation (P3): persistent identity shapes decision-making across interactions. Maps to NT-* domain identity: each domain has a backstory (its purpose, constraints, history). |

### 8. SWE-agent — Autonomous Software Engineering Agent
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/princeton-nlp/SWE-agent |
| **Stars** | 14,300+ |
| **Language** | Python |
| **Category** | Autonomous Bug Fixing Agent |
| **What it does** | Agent that autonomously fixes GitHub issues by reading the codebase, understanding the bug, and implementing fixes. Trained on SWE-bench with reinforcement learning. Agent-Computer Interface (ACI) designed for LLM interactions with code. Supports multi-file edits with dependency tracking. Evaluation on SWE-bench Verified with 49% solve rate. |
| **NeoTrix mapping** | NT-ACT (code action) + NT-CORE (reasoning). SWE-agent = NT-ACT's code generation with NT-CORE's reasoning chain. ACI = NT-IO's agent-native interface design. Dependency tracking = domain trait contract verification across files. |
| **Key pattern** | **Agent-Computer Interface (ACI)** — interface designed specifically for LLM agents, not humans. Commands, observations, and feedback loops are optimized for LLM reasoning patterns. Maps to NT-IO's agent-native interface design: CLI and API interfaces should be optimized for agent consumption, not just human use. |

### 9. Marqo — AI-Native Vector Search Engine
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/marqo-ai/marqo |
| **Stars** | 10,800+ |
| **Language** | Go/Python |
| **Category** | Vector Search / RAG Infrastructure |
| **What it does** | End-to-end vector search engine with built-in embedding, hybrid search (vector + keyword), and reranking. Multi-modal support (text, images, video). TensorRT-optimized inference. Automatic index management with real-time updates. Supports OpenAI, CLIP, and custom embedding models. Production-ready with filtering, access control, and multi-tenancy. |
| **NeoTrix mapping** | NT-MEMORY (retrieval) + NT-WORLD (perception). Marqo = NT-MEMORY's hybrid search (BM25+embedding) with multi-modal support. TensorRT optimization = NT-PHYSICAL's hardware-aware execution. Multi-tenancy = NT-MEMORY's namespace isolation. |
| **Key pattern** | **Embedding-native search engine** — the search engine owns the embedding pipeline, not just the index. Embedding, indexing, and retrieval are a single system, not separate components. Maps to NT-MEMORY's integrated pipeline: KB embedding → indexing → retrieval should be a unified system, not separate layers. |

### 10. Humanloop — Developer Platform for LLM Products
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/Humanloop/humanloop-python |
| **Stars** | 3,400+ |
| **Language** | Python |
| **Category** | LLM DevOps Platform |
| **What it does** | End-to-end platform for building LLM products: prompt management, evaluation, fine-tuning, and monitoring. Prompt versioning with diff views. Dataset management with annotations. Evaluation harnesses with human and automated scoring. Model fine-tuning pipeline with dataset → training → deployment. Real-time monitoring with drift detection. |
| **NeoTrix mapping** | NT-MIND (skill crystallization) + NT-GOVERNANCE (compliance). Humanloop = NT-MIND's SEAL pipeline for prompt evolution with NT-GOVERNANCE's compliance tracking. Prompt versioning = experience-tree branch versioning. Evaluation = SelfTest T3 production wiring. Drift detection = ConsciousnessTree's health monitoring. |
| **Key pattern** | **Prompt-as-versioned-artifact** — prompts are version-controlled like code, with diffs, rollback, and evaluation gates. Each version has associated metrics and evaluation results. Maps to NT-MIND's skill crystallization: each skill version has evaluation metrics and can be rolled back if performance degrades. |

---

## Meta-Patterns (Cycle 361)

| Pattern | Count | NeoTrix Implication |
|---------|-------|---------------------|
| **Multi-Agent Specialization** | 3 | Teams of focused specialists, not monoliths — validates NT-* domain decomposition |
| **Structured Tool Contracts** | 3 | Tools return typed schemas with validation — trait-based domain contracts |
| **Cyclic Execution** | 2 | Workflows with loops, not just DAGs — ConsciousnessTree feedback loops |
| **Validation-as-Retry-Signal** | 2 | Type errors feed back as corrective prompts — corrective reasoning pattern |
| **Agent Identity / Backstory** | 1 | Persistent identity shapes behavior across interactions — profile-driven adaptation |
| **Reflexion-Based Routing** | 1 | Gateway reflects on failures to select providers — meta-cognition for routing |
| **Embedding-Native Integration** | 1 | Search engine owns the embedding pipeline — unified retrieval system |
| **Prompt Versioning** | 1 | Prompts as version-controlled artifacts with evaluation gates |

## Source Density

| Source | Projects Found | Quality |
|--------|---------------|---------|
| GitHub Trending (weekly) | 6 | High — real codebases with star velocity |
| ProductHunt (Aug-Sep 2026) | 2 | Medium — launch-stage, signal > quality |
| GitHub Explore / Featured | 2 | High — GitHub-curated |

## Novel vs Incremental

- **Truly Novel**: CrewAI (agent backstory as identity), SWE-agent (ACI for LLM agents), LangGraph (cyclic execution graphs)
- **Incremental but Valuable**: OpenHands (multi-specialist coding), Portkey (reflexion-based routing), Marqo (embedding-native search), Dify (visual LLM workflows)
- **Signal-Only**: Instructor (validation-as-retry), Composio (tool-as-structured-contract), Humanloop (prompt versioning)
