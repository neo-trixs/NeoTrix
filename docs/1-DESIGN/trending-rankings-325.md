# Trending Rankings — Cycle 325

**Date**: 2026-09-11
**Focus**: Self-evolving agents, world models for agents, lightweight agent runtimes, MCP-native discovery, declarative attention routing, agent eval harnesses

---

## 10 New Projects (Not in Cycles 318-324)

### 1. OmniAgent — Full-Dimensional Self-Evolving Agent
- **GitHub**: https://github.com/YeQing17-2026/OmniAgent
- **Stars**: ~2,557 (growing)
- **What**: Open-source self-evolving agent framework with OmniEvolve: Proactive Memory (dual-path alignment), Skill Self-Evolution (auto-create/inspect/repair), Context Self-Evolution (real-time interaction feedback + LLM summarization), BrainModel Self-Evolution (online RL feedback loop GRPO+PRM). Hyper-Harness with dynamic multi-agent (Sentinel planner + Guardian safety), four-layer dynamic security scanning. Deep Reflexion dual-layer reflective architecture.
- **Key Pattern**: **Full-spectrum self-evolution** — skills, context, and brain model evolve simultaneously through interaction. The "Hyper-Harness" concept treats agent safety as a kernel concern, not a bolt-on.
- **NeoTrix Relevance**: Maps to NT-MIND (SEAL pipeline, skill crystallization), NT-SHIELD (four-layer security scanning), NT-FEEL (proactive memory as emotional context). The BrainModel self-evolution via online RL is a potential extension of our SelfModel dynamic performance model. Sentinel/Guardian agent separation parallels our NT-GOVERNANCE/NT-SHIELD split.

### 2. Qwen-AgentWorld — Language World Model for Agents
- **GitHub**: https://github.com/qwenlm/qwen-agentworld
- **Stars**: ~984 (new, Jun 2026)
- **What**: Native language world model (MoE 35B/3B active, 256K ctx) that simulates agentic environments via chain-of-thought reasoning across 7 domains: MCP, Search, Terminal, SWE, Android, Web, OS. Three-stage training (CPT→SFT→RL) on 10M+ real interaction trajectories. Achieves 58.71 overall score beating GPT-5.4 (58.25). Supports controllable perturbations and fictional-world construction.
- **Key Pattern**: **World model as environment simulator** — the model doesn't just predict next tokens, it simulates what would happen in an agent environment. Fictional-world training generalizes better than real-world training.
- **NeoTrix Relevance**: Directly maps to NT-WORLD (perception, environment modeling). Fictional-world construction validates our SEAL exploration stages. The 7-domain unified model is a template for our cross-domain agent simulation. Could enhance NT-MIND's SEAL pipeline by providing world-model-based self-testing before real execution.

### 3. Declarative Attention (DA) — Self-Sparse Attention Protocol
- **Paper**: arXiv:2609.02737
- **What**: Protocol that elicits models to declare where they need to attend within their chain-of-thought, partitioning generation into three modes: `<global>` (full context), `<focus>` (specific region), `<local>` (recent output only). Zero-shot, no training required. On Gemma-4-31B and Qwen-3.6-27B: 52% and 31% reduction in total attended tokens with 1.27pp and 2.75pp accuracy drops that shrink with model scale.
- **Key Pattern**: **Model-controlled attention** — instead of external heuristics deciding what to attend to, the model itself declares its attention needs during reasoning. Intrinsic vs extrinsic sparse attention.
- **NeoTrix Relevance**: Maps directly to NT-CORE GWT attention routing. The three attention modes (global/focus/local) parallel our GWT salience tiers. DA validates that models can self-regulate attention — our GWT could use model-declared attention zones rather than external salience scoring. Training-free approach aligns with our frozen-backbone philosophy.

### 4. AgensFlow — Coordination-Policy Substrate
- **GitHub**: https://github.com/Nicolepcx/AgensFlow
- **Paper**: arXiv:2605.27466
- **Stars**: ~10 (alpha, but significant research)
- **What**: Framework treating multi-agent coordination as online policy learning under partial observability. Folded policy graph keyed by task signatures with UCB1-selected actions. skip:X topology learning — policy can exclude skills, not just reorder. RelativeJudge scoring with cross-judge averaging and confidence weighting. Warm-started policy graphs reduce exploration cost by ~21% while preserving plateau quality.
- **Key Pattern**: **Coordination as learnable object** — routing decisions are not fixed pipelines but learned policies that improve over trajectories. skip:X is a novel topology action: the system learns when NOT to use a skill.
- **NeoTrix Relevance**: Maps to NT-ACT orchestration, NT-MIND evolution. The folded policy graph is a natural extension of our SEAL pipeline — coordination topology should evolve based on task regime. skip:X is directly applicable to our capability routing: learn when to skip a capability node. RelativeJudge is relevant to our experience quality assessment.

### 5. nanobot — Ultra-Lightweight Agent Framework
- **GitHub**: https://github.com/HKUDS/nanobot
- **Stars**: ~47,663 (fast-growing)
- **What**: Ultra-lightweight, self-hosted personal AI agent framework. Python, WebUI, terminal, chat apps (Telegram, Discord, Slack, WeChat, Email, Mattermost). Tools: files, shell, web search, MCP, cron, image generation, subagents. Dream memory system. Model routing with OpenAI-compatible APIs. v0.3.0 "Agency Release" adds inline subagents, model switching per session.
-- **Key Pattern**: **Minimal-core agent runtime** — small readable internals with MCP, memory, deployment, and automation built in. Dream memory as persistent long-term context.
- **NeoTrix Relevance**: Validates our lightweight-first philosophy. The Dream memory system is relevant to NT-MEMORY — persistent memory as a named concept. Model switching per session maps to our Dual Specialization (Weapon Set I/II). The agent gateway deployment pattern aligns with NT-IO.

### 6. ConvMem — Convolutional Memory for Long-Context
- **Paper**: arXiv:2609.10441
- **What**: Training-free framework reformulating long-context reasoning as hierarchical convolution. LLM as convolutional kernel summarizes text segments hierarchically — reasoning path becomes logarithmic tree instead of linear chain. Configurable Strides + Skip Connections + Multi-Kernel Convolution. Outperforms training-free baselines and avoids RL overfitting on out-of-distribution tasks.
-- **Key Pattern**: **Hierarchical context compression** — instead of sequential reading, compress context logarithmically through convolutional layers. Multi-kernel decomposition separates semantic channels.
- **NeoTrix Relevance**: Maps to NT-MEMORY context management and GWT attention. The logarithmic tree compression is a model for how our KB should organize experience — hierarchical summaries rather than flat storage. Multi-kernel decomposition parallels our multi-domain architecture. Training-free aligns with frozen-backbone philosophy.

### 7. ROAM — Relation-Guided Atomic Memory Management
- **Paper**: arXiv:2609.09778
- **What**: Framework using atomicity for memory management with richer answer-time representations. Classifies atom pairs as independent/equivalent/subsuming/conflicting. Organizes into Primary and Evidence roles. Fusion combines complementary details into non-atomic views. Improves answer accuracy by up to 29.8 percentage points. 15.6pp higher answer-critical source recall.
- **Key Pattern**: **Memory lifecycle management** — not just storing memories but managing their relationships (equivalence, subsumption, conflict) and roles (primary vs evidence). Fusion creates compact views from atomic parts.
- **NeoTrix Relevance**: Directly maps to NT-MEMORY experience management. The Primary/Evidence role distinction is a model for our experience hub — some experiences are primary answers, others are supporting evidence. Fusion is relevant to our SEAL distillation stage. The relation classification (independent/equivalent/subsuming/conflicting) could enhance our KB edge types.

### 8. GitTrends AI v5.0 — MCP-Native Discovery Engine
- **GitHub**: https://github.com/jastfan/github-trending
- **What**: Open-source real-time GitHub velocity tracker with native MCP server. 4 editorial leaderboards: Agent Skills, MCP Servers, Ecosystem Marketplaces, Star Velocity Radar. 1-click coding agent discovery via `claude mcp add gittrends`. Off-peak scheduled updates, self-healing API fallback, atomic rebase deployment. Machine-readable JSON + RSS feed.
- **Key Pattern**: **Agent-native discovery** — discovery tools built as MCP servers so agents can query trends mid-task without leaving terminal. Velocity-based ranking (stars/day) beats absolute count.
- **NeoTrix Relevance**: Maps to NT-WORLD (information acquisition) and NT-IO (tool discoverability). The MCP server pattern validates our tool routing approach. Velocity-based ranking is relevant to our Constellation maturity tracking — growth rate matters more than absolute size. The self-healing update pattern aligns with our self-healing architecture.

### 9. Oqoqo — Agent Eval Harness
- **ProductHunt**: https://www.producthunt.com/products/oqoqo
- **What**: Build evals and custom benchmarks for real-world agent tasks. Spins up isolated sandboxes, executes tasks against agents (Codex, Claude Code, OpenClaw, Hermes, Pi, Opencode, Cursor, GitHub Copilot), catalogs every step including tool calls, retries, discovery loops. Token consumption, cost, success/failure evaluation. Regression test MCP, CLI, skills, SDK interfaces.
- **Key Pattern**: **Agent benchmarking as infrastructure** — standardized eval harness that spins up isolated environments and measures agent performance across dimensions (success, cost, token usage, step count).
- **NeoTrix Relevance**: Maps to NT-MIND SelfTest infrastructure. Oqoqo's step-level tracing is a model for our T3 Production Wiring — not just "does the detection function exist" but "does it actually improve behavior". The sandbox-based isolation validates our worktree isolation pattern. The multi-agent comparison framework is relevant to our Dual Specialization evaluation.

### 10. Inference-Time Graph Engineering (ReActNet)
- **Paper**: arXiv:2609.05774
- **What**: Training-free framework compiling queries and role-specialized agents into temporal workflow graphs. Each graph snapshot is a reasoning stage; edges carry natural-language instructions specifying messages. Separates graph compilation from execution — explicit, inspectable, task-conditioned without RL. Consistently improves over fixed-topology and learned-topology baselines.
- **Key Pattern**: **Temporal graph compilation** — multi-agent coordination as a sequence of directed graphs, each snapshot encoding a reasoning stage. Compilation separated from execution enables inspection and task-conditioning.
- **NeoTrix Relevance**: Maps to NT-ACT orchestration and NT-CORE GWT. The temporal graph model is a natural fit for our SEAL pipeline stages — each stage compiles a different graph topology. Separation of compilation and execution validates our architecture: plan topology, then execute. The natural-language edge instructions are relevant to our EventBus message semantics.

---

## Cross-Cutting Patterns

| Pattern | Projects | NeoTrix Mapping |
|---------|----------|-----------------|
| **Self-evolution as first-class** | OmniAgent, AgensFlow, ConvMem | NT-MIND SEAL pipeline |
| **Model-declared attention** | Declarative Attention, ConvMem | NT-CORE GWT salience |
| **Memory lifecycle management** | ROAM, ConvMem, MemoryLACE | NT-MEMORY experience-tree |
| **World models for agents** | Qwen-AgentWorld | NT-WORLD environment simulation |
| **Lightweight-first runtimes** | nanobot, Oqoqo | NT-IO minimal core |
| **Agent-native discovery** | GitTrends AI | NT-WORLD/NT-IO tool routing |
| **Graph-based orchestration** | ReActNet, AgensFlow | NT-ACT coordination topology |

---

## Market Signals

- **Self-evolution is the new frontier**: OmniAgent, AgensFlow, and ConvMem all treat agent improvement as a continuous process, not a one-time training step.
- **Memory is the differentiator**: ROAM, ConvMem, and MemoryLACE show memory management as the key to long-horizon agent reliability.
- **Sparse attention is production-ready**: Declarative Attention achieves 52% token reduction with zero training — attention efficiency is now an inference-time optimization, not a training requirement.
- **MCP ecosystem maturing**: GitTrends AI v5.0 shows the MCP ecosystem is large enough to need its own discovery engine.
