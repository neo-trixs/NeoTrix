# Trending Rankings — Cycle 362 (2026-09-12)

## Search Scope
GitHub Trending, ProductHunt, arXiv, HuggingFace Papers — AI agents, LLM tools, reasoning frameworks, memory/attention/routing patterns. Live web searches on 2026-09-12.

---

## 10 New Projects (Not in Cycles 318–361)

### 1. Ponytail — Minimal Code Skill for Coding Agents
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/DietrichGebert/ponytail |
| **Stars** | 91,866+ |
| **Language** | Multi-language (prompt skill) |
| **Category** | Agent Code Generation Optimization |
| **What it does** | A prompt skill that forces LLMs to write only what's necessary — no over-engineering, no boilerplate. Benchmarked: -54% LOC, -22% tokens, -20% cost, -27% time vs bare agent on real FastAPI+React repos. Three intensity levels (lite/full/ultra). Cross-platform: Claude Code, Codex, Gemini, Devin CLI, OpenCode, Hermes Agent. Includes `/ponytail-review` (diff audit for over-engineering), `/ponytail-audit` (repo-wide), `/ponytail-debt` (harvest deferred shortcuts). Fully safe: 100% safety guard preservation. |
| **NeoTrix mapping** | NT-MIND (code minimalism skill) + NT-ACT (code generation). Ponytail = NT-MIND's skill crystallization as a "less is more" meta-skill. Maps to R-P1 (forbid unsafe) and Dark Forest axiom (minimal viable code). Skill intensity levels = Constellation maturity (C0→C3). |
| **Key pattern** | **Anti-over-engineering as a first-class skill** — most agent skills add capability; Ponytail subtracts. The skill constrains output to only what's needed, preserving safety. This is constraint-based reasoning: fewer tokens, fewer bugs, faster execution. Maps to SEAL's distillation: compress output to essential signal. |

### 2. Graft — Context Optimization for Coding Agents
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/trailhq/Graft |
| **Stars** | 4,917+ |
| **Language** | TypeScript/Python |
| **Category** | Agent Context Engine |
| **What it does** | Pre-processes codebase context for coding agents: builds code graph via tree-sitter, generates oriented context bundles, and pushes them as MCP tools. Up to 4× cheaper, 3× faster with +12pts correctness on SWE-bench (54%→66%). `graft init` auto-detects agents (Claude Code, Cursor, Codex, Copilot, OpenClaw) and wires MCP server. `graft ask --source` pre-bundles context; `graft find_code`/`graft_file_api` pull-on-demand. 162-run controlled benchmark with cache-aware cost model. |
| **NeoTrix mapping** | NT-MEMORY (code graph) + NT-IO (MCP integration). Graft = NT-MEMORY's semantic indexing specialized for codebases. Pre-bundled context = KB embedding with code-aware chunking. Pull-on-demand = lazy branch loading (experience-tree pattern). MCP server = NT-IO's protocol interface. |
| **Key pattern** | **Context pre-computation vs pull-on-demand** — two modes: push a bundle upfront (faster first interaction) or pull per query (better correctness). This is the KVMem hot/cold tiering insight applied to code context: pre-computed bundles are the hot tier, on-demand is the cold tier. +12pts correctness from pull mode validates lazy loading. |

### 3. Prime Agent — Self-Improving RLM Agent
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/PrimeIntellect-ai/prime-agent |
| **Stars** | 15,890+ |
| **Language** | Python |
| **Category** | Self-Improving Coding Agent |
| **What it does** | Recursive Language Model (RLM) agent treating context as variables and tools as recursive sub-agents in a persistent REPL. Continual Harness stores supplemental prompts, memories, skill descriptions as durable state. `/refine` applies evidence-backed updates to harness state (never rewrites base system prompt). Skills are importable Python packages. Daemon-backed sessions survive terminal detach. Agent-to-agent direct communication without user routing. Heartbeats, schedules, bounded autonomous mode with quality gates. |
| **NeoTrix mapping** | NT-MIND (self-improvement) + NT-NEXUS (cross-session persistence). Prime Agent = NT-MIND's SEAL pipeline with durable harness state. `/refine` = experience-tree absorption (evidence-backed, rollbackable). Continual Harness = KB kv_store with session-scoped namespace. Agent-to-agent = NT-ACT's cross-domain orchestration. |
| **Key pattern** | **Prompt-as-a-variable, tools-as-recursive-subagents** — context is mutable state, not static prompt. Tools spawn child agents programmatically. This is the PTC (Programmatic Tool Calling) pattern taken to its logical extreme: the REPL IS the agent, and everything else is a function call. Maps to NeoTrix's capability-as-function-call model. |

### 4. Harden — Security Layer for AI Coding Agents
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/harden-aif |
| **Stars** | ~2,000+ (new) |
| **Language** | Python |
| **Category** | Agent Security |
| **What it does** | Post-trained model that checks tool calls before they execute. Uses request + session context to evaluate safety. Beat frontier models on agent-security benchmarks. Fully local: no data leaves the machine. Free tier available. Launched on ProductHunt Sep 9, 2026 (#2 daily, score 405). |
| **NeoTrix mapping** | NT-SHIELD (security guard) + NT-CORE (context evaluation). Harden = NT-SHIELD's egress guard inverted as an ingress guard for tool calls. Post-trained safety model = NT-CORE's consciousness evaluation applied to tool calls. Local-only = NT-SHIELD's trust boundary enforcement. |
| **Key pattern** | **Tool-call guard as post-trained model** — not rule-based filtering but a learned safety evaluator that understands context. This is NT-SHIELD's missing piece: a neural guard that evaluates tool calls holistically, not just pattern-matching. Maps to EmotionLabel's trust/disgust evaluation applied to actions. |

### 5. COMPASS — Hierarchical Context Management for Long-Horizon Agents
| Field | Detail |
|-------|--------|
| **URL** | https://aclanthology.org/2026.acl-long.152.pdf |
| **Stars** | N/A (paper) |
| **Language** | N/A |
| **Category** | Multi-Agent Reasoning Framework |
| **What it does** | Three-component architecture: (1) Main Agent for tactical ReAct reasoning, (2) Meta-Thinker that monitors progress and issues strategic interventions, (3) Context Manager that compresses history into concise briefs. +20% accuracy on GAIA, BrowseComp, Humanity's Last Exam. Test-time scaling extension matches DeepResearch agents. Post-training delegates context management to smaller models. Prevents context overload (premature conclusions) and contextual pollution (repeated errors). |
| **NeoTrix mapping** | NT-META (meta-thinking) + NT-MEMORY (context management) + NT-CORE (tactical reasoning). COMPASS = NT-META's ConsciousnessTree split into three: Main Agent (tactical), Meta-Thinker (strategic monitoring), Context Manager (memory compression). Context briefs = experience-tree distillation. Strategic interventions = GWT salience-based attention modulation. |
| **Key pattern** | **Separation of tactical vs strategic reasoning** — the Meta-Thinker doesn't do the work; it watches the work and intervenes. This is exactly the NT-META role: meta-cognition observes cognition and redirects. Context Manager = memory compaction with semantic awareness. The three roles map directly to NT-CORE (tactical), NT-META (strategic), NT-MEMORY (context). |

### 6. Webwright — Browser Agent as Code Skills
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/microsoft/Webwright |
| **Stars** | 5,961+ |
| **Language** | Python |
| **Category** | Browser Agent Framework |
| **What it does** | Microsoft's SWE-style browser agent. Code-as-action: generates Python scripts instead of coordinate clicks. Skill Factory: every solved task leaves a reusable, parameterized script. Skills rerun standalone in ~40s with zero tokens. On WebArena: 55%→70% accuracy via skill reuse (+15pp). Online-Mind2Web: 86.7% with GPT-5.4. Odysseys long-horizon: 60.1% (+15.6pp over prior SOTA). Plugin manifests for Claude Code, Codex, OpenClaw, Hermes Agent. |
| **NeoTrix mapping** | NT-ACT (browser action) + NT-MIND (skill distillation). Webwright = NT-ACT's browser automation with NT-MIND's SEAL skill crystallization. Skill Factory = experience-tree's skill extraction from successful trajectories. Zero-token rerun = compiled skill node (C4 constellation). |
| **Key pattern** | **Solve once, run forever without tokens** — every successful trajectory is distilled into a parameterized script. This is the SEAL pipeline's crystallization stage: exploration → skill extraction → zero-cost reuse. +15pp accuracy from skill reuse validates that past experience reduces future computation. |

### 7. 9Router — Free AI Router with Token Compression
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/decolua/9router |
| **Stars** | 23,774+ |
| **Language** | TypeScript/Node.js |
| **Category** | LLM Gateway / Cost Optimization |
| **What it does** | Connects Claude Code, Codex, Cursor, Cline, Copilot, Gemini to 40+ providers and 100+ models. RTK token saver (from rtk-ai/rtk, 40K★) compresses tool outputs, saving 20-40% tokens. Headroom integration for additional compression. Caveman mode for 65% output token reduction. Ponytail integration for minimal code. Smart 3-tier fallback: Subscription→Cheap→Free. Multi-account load balancing. Real-time quota tracking. Format translation across providers. |
| **NeoTrix mapping** | NT-IO (multi-provider routing) + NT-SHIELD (cost guardrails). 9Router = NT-IO's ordered backend router with cost-aware routing (Axiom A1). RTK compression = NT-MEMORY's context compression. Caveman/Ponytail = NT-MIND's output minimization skills. Multi-account = load balancing with total_calls ascending. |
| **Key pattern** | **Composable compression stack** — RTK (input compression) + Headroom (context compression) + Caveman (output compression) + Ponytail (code minimalism) stack orthogonally. Each layer reduces tokens at a different stage. This is the NeoTrix architecture: each domain contributes its own optimization, and they compose. |

### 8. Eve — Filesystem-First Agent Framework
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/vercel/eve |
| **Stars** | 4,957+ |
| **Language** | TypeScript |
| **Category** | Agent Framework |
| **What it does** | Vercel's filesystem-first framework for durable AI agents. Core agent capabilities live in conventional file locations (AGENTS.md, .eve/). Projects are inspectable, extendable, and operable by reading the filesystem. Local documentation from node_modules/eve/docs for coding agents. Beta stage. |
| **NeoTrix mapping** | NT-NEXUS (filesystem convention) + NT-IO (agent interface). Eve = NT-NEXUS's cross-session persistence via filesystem conventions. AGENTS.md pattern = NeoTrix's own AGENTS.md convention. Local docs for agents = experience-tree's lazy branch loading from KB. |
| **Key pattern** | **Filesystem as the universal agent interface** — agent state, skills, and configuration live in conventional file paths. Any tool that can read files can interact with the agent. This is the "convention over configuration" pattern applied to agent frameworks: the filesystem IS the API. |

### 9. Blume — Session Learning for Coding Agents
| Field | Detail |
|-------|--------|
| **URL** | https://blume.codes |
| **Stars** | ~1,500+ (new, Sep 3 2026) |
| **Language** | Desktop app (Electron) |
| **Category** | Agent Self-Improvement |
| **What it does** | Watches coding agent sessions locally and extracts learning signals. Corrections become rules, workflows become skills, frustrated ALL CAPS messages become pain signals. Clusters corrections thematically; when pain threshold is reached, suggests concrete updates (stale rule, hook, skill, or new one). User reviews and applies. Fully local: no data leaves machine. Supports Claude Code, Codex, Cursor. |
| **NeoTrix mapping** | NT-MIND (self-improvement) + NT-NEXUS (session memory). Blume = NT-MIND's SEAL distillation applied to coding sessions. Correction clustering = experience-tree's thematic extraction. Pain threshold = GWT salience-based attention trigger. Skill suggestion = skill crystallization from behavioral patterns. |
| **Key pattern** | **Correction-as-training-signal** — agent mistakes are not noise but signal. Repeated corrections indicate systemic issues that should be encoded as rules/skills. This is the NT-MIND philosophy: every failure is a distillation opportunity. Pain threshold = the point where accumulated corrections warrant structural change. |

### 10. Mastra — AI Agent Framework with Workflows and Memory
| Field | Detail |
|-------|--------|
| **URL** | https://mastra.ai |
| **Stars** | ~5,000+ (new, Sep 9 2026) |
| **Language** | TypeScript |
| **Category** | Agent Development Platform |
| **What it does** | Framework from the Gatsby team for building AI-powered apps and agents. Workflows, memory, streaming, evals, tracing. Studio: interactive UI for dev and testing. `npm create mastra@latest` quickstart. Production-grade with observability and evaluation. |
| **NeoTrix mapping** | NT-ACT (workflow orchestration) + NT-IO (streaming interface). Mastra = NT-ACT's SEAL pipeline with developer-friendly Studio UI. Evals = SelfTest T3 production wiring. Tracing = EventBus event logging. Memory = NT-MEMORY's session persistence. |
| **Key pattern** | **Workflow-as-first-class with observability** — every workflow step is traceable, evaluable, and debuggable. This is the SEAL pipeline principle: every stage produces observable artifacts. Studio = NT-IO's interactive development interface for agent workflows. |

---

## Cross-Cutting Patterns (Cycle 362)

### 1. Compression Stack Composition
- **RTK** (input) + **Headroom** (context) + **Caveman** (output) + **Ponytail** (code) = orthogonal token reduction
- Each layer operates at a different stage; they compose without interference
- NeoTrix: each NT-* domain contributes domain-specific optimization

### 2. Correction-as-Training-Signal
- **Blume**: corrections → rules/skills
- **Prime Agent**: `/refine` → durable harness updates
- **Ponytail**: over-engineering detection → code minimalism
- Pattern: agent mistakes are signal for self-improvement, not noise

### 3. Tactical vs Strategic Separation
- **COMPASS**: Main Agent (tactical) + Meta-Thinker (strategic) + Context Manager (memory)
- **BIGMAS** (from paper): GraphDesigner (strategic) + Orchestrator (routing) + agents (tactical)
- Pattern: meta-cognition observes cognition and redirects

### 4. Solve-Once-Run-Forever
- **Webwright Skill Factory**: trajectory → parameterized script → zero-token reuse
- **Graft**: code graph → pre-computed context bundles
- **Blume**: corrections → persistent rules
- Pattern: past experience reduces future computation

### 5. Filesystem as Agent Interface
- **Eve**: conventional file locations as agent state
- **AGENTS.md**: project-level agent instructions
- Pattern: the filesystem IS the API for agent configuration
