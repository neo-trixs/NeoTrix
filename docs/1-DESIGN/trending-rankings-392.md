# Trending Rankings — Cycle 392 (2026-09-12)

## 10 New Projects (Not in Cycles 318–391)

### 1. screenpipe — AI Screen Recording for Agent Memory
- **URL**: https://github.com/mediar-ai/screenpipe
- **Stars**: 52K+ | **Forks**: 3,100+ | **Language**: TypeScript
- **What**: YC S26. Continuous screen + audio recording that feeds structured context to AI agents. Captures app usage, browser activity, and audio transcripts in real-time. Agents query the recording history to understand user behavior patterns without explicit instruction. Privacy-first: all processing local, no cloud upload.
- **Novel Pattern**: Ambient context capture as passive memory substrate. Instead of explicit agent memory writes, screenpipe passively builds a temporal knowledge graph from user activity. Agents gain situational awareness without asking.
- **NeoTrix Mapping**: NT-MEMORY (passive episodic capture = continuous experience logging), NT-WORLD (ambient perception layer), NT-FEEL (user behavior modeling for emotional context), ConsciousnessTree (passive observation feeds self-model growth cycles).

### 2. Reflxio — Behavioral Learning for Agent Self-Improvement
- **URL**: https://www.producthunt.com/products/reflxio
- **Stars**: #2 ProductHunt of the Day (Sep 5, 2026)
- **What**: Behavioral learning engine that makes AI agents improve over time. Tracks which agent behaviors produce successful outcomes, builds behavioral reinforcement models, and adapts agent strategies based on accumulated experience. Unlike prompt optimization, operates on action-level patterns.
- **Novel Pattern**: Action-behavior reinforcement loop: observe outcome → update behavioral model → adjust future action selection. Agents self-improve through accumulated behavioral trace analysis rather than explicit fine-tuning.
- **NeoTrix Mapping**: NT-MIND (self-evolution via behavioral feedback), NT-CORE (E8 hexagram: action-outcome mapping = state transition learning), SEAL (behavioral distillation into skill nodes), NT-REPAIR (failed behavior detection and correction).

### 3. Cortex by SKYNETLAB — Selective Memory Layer for Agents
- **URL**: https://www.producthunt.com/products/cortex-by-skynetlab
- **Stars**: New launch (Aug 2026)
- **What**: "The memory layer that decides what's worth remembering." AI infrastructure that evaluates incoming information for memorability based on novelty, relevance, and reusability. Filters the firehose of agent experience into a curated memory store. MCP-native integration.
- **Novel Pattern**: Memorability scoring — not all experience is worth storing. Cortical filtering selects for novelty + reusability, preventing memory bloat while preserving high-value patterns.
- **NeoTrix Mapping**: NT-MEMORY (selective memory = experience-tree absorption filtering), NT-CORE (GWT salience for memory attention), NT-MIND (distillation before storage = quality gate), Axiom A2 (context scarcity demands selective memory).

### 4. IQ Routing — Trajectory-Aware LLM Cost Optimization
- **URL**: https://www.producthunt.com/products/iq-routing
- **Stars**: New launch (Aug 2026)
- **What**: Unified API with trajectory-aware LLM routing. Analyzes the full reasoning trajectory (not just individual requests) to route sub-tasks to the cheapest capable model. Cuts agent cost 40-70% by decomposing trajectories and routing easy reasoning steps to cheap models, hard steps to expensive ones.
- **Novel Pattern**: Trajectory-level routing (not request-level). Analyzes the full chain of thought to identify which reasoning steps need expensive models vs. which can use cheap ones. Cost savings compound across multi-step agent workflows.
- **NeoTrix Mapping**: NT-IO (multi-provider routing with trajectory awareness), NT-CORE (GWT salience + cost weight per reasoning step), Axiom A1 (Cost-Aware Routing at sub-request granularity), NT-ACT (step-level model delegation).

### 5. GitNexus — Zero-Server Code Intelligence Engine
- **URL**: https://github.com/nicholasgasior/gitnexus
- **Stars**: 58K (+202 stars/day) | **Language**: TypeScript
- **What**: Client-side knowledge graph creator that runs entirely in the browser. Drop in a git repo (GitHub, GitLab, Azure, local ZIP) and get an interactive knowledge graph with built-in Graph RAG Agent. No server needed — all processing client-side. Perfect for code exploration, architecture understanding, and onboarding.
- **Novel Pattern**: Zero-server code intelligence with Graph RAG. Client-side knowledge graph construction from repository structure. Agent queries the graph locally without external API calls.
- **NeoTrix Mapping**: NT-MEMORY (code knowledge graph as structural memory), NT-WORLD (code perception + relationship extraction), NT-IO (zero-server = edge-capable inference), NT-SHIELD (client-side = data never leaves device).

### 6. HarnessRouter — Unified Agent Harness Interface
- **URL**: https://www.producthunt.com/products/harnessrouter
- **Stars**: #1 ProductHunt (Aug 17, 2026)
- **What**: Open-source unified interface for agent harnesses. Single interface to manage Claude Code, Codex, Gemini CLI, Cursor, and other coding agents. Routes tasks to the best harness based on task type, manages shared context across harnesses, and provides observability across the fleet.
- **Novel Pattern**: Harness abstraction layer — task routing across heterogeneous agent harnesses. Each harness has different strengths; HarnessRouter dynamically selects the best harness per task.
- **NeoTrix Mapping**: NT-ACT (harness orchestration = capability routing), NT-IO (multi-provider unification), NT-CORE (GWT routing to best harness), NT-MEMORY (shared context across harnesses).

### 7. Vibe-Trading — Personal Trading Agent
- **URL**: https://github.com/HKUDS/Vibe-Trading
- **Stars**: 13K+ | **Forks**: 1,800+ | **Language**: Python
- **What**: AI-powered personal trading agent from HKU Data Science. Multi-agent architecture with specialized sub-agents for market analysis, risk assessment, strategy generation, and execution. Integrates real-time market data, news sentiment, and technical analysis. Reinforcement learning for strategy adaptation.
- **Novel Pattern**: Multi-agent financial reasoning with RL-driven strategy evolution. Specialized agents for different trading aspects coordinate through a central planning agent. Strategy evolves through reward-weighted behavioral learning.
- **NeoTrix Mapping**: NT-ACT (multi-agent coordination), NT-CORE (reasoning under uncertainty), NT-MEMORY (market knowledge graph), NT-MIND (strategy self-evolution via RL), NT-FEEL (sentiment analysis for market emotion).

### 8. OpenMemory — Persistent Cross-Agent Memory Standard
- **URL**: https://github.com/openmemory-ai/openmemory
- **Stars**: 15K+ | **Forks**: 2,100+ | **Language**: TypeScript
- **What**: Open standard for persistent AI agent memory. Temporal graph-based memory that persists across sessions, agents, and tools. Works with Claude Code, Cursor, Copilot, Codex, and any MCP-compatible agent. Memory entries have timestamps, decay functions, and relevance scores.
- **Novel Pattern**: Temporal graph memory with built-in decay. Memory entries lose relevance over time unless reinforced by usage. Graph structure captures relationships between memories, enabling associative recall.
- **NeoTrix Mapping**: NT-MEMORY (temporal knowledge graph with decay = experience-tree branch relevance), NT-IO (cross-tool memory standard = multi-provider compatibility), NT-CORE (associative recall via graph traversal), NT-REPAIR (memory decay = natural pruning of stale patterns).

### 9. OfficeCLI — AI-Native Office Suite
- **URL**: https://github.com/iOfficeAI/OfficeCLI
- **Stars**: 16K+ | **Forks**: 816+ | **Language**: Rust
- **What**: First Office suite purpose-built for AI agents. Single binary, no Office installation required. Agents read, edit, and automate Word, Excel, and PowerPoint files programmatically. MCP-native with structured output for agent consumption.
- **Novel Pattern**: AI-first document manipulation — files as structured data for agent workflows. Instead of agents trying to use human-oriented GUIs, provides programmatic document APIs optimized for LLM consumption.
- **NeoTrix Mapping**: NT-ACT (document automation capability), NT-IO (structured document interface for agents), NT-WORLD (document perception + parsing), NT-MEMORY (document-as-knowledge representation).

### 10. Prefactor — Agent Error Catching Before Deployment
- **URL**: https://www.producthunt.com/products/prefactor
- **Stars**: #1 ProductHunt of the Day (Jul 28, 2026)
- **What**: Catches agent mistakes before they reach customers. Observability + evaluation layer that monitors agent outputs in real-time, detects hallucinations, logical errors, and behavioral drift. Provides explainable error reports with suggested corrections.
- **Novel Pattern**: Pre-deployment error interception — agent outputs are evaluated against expected behavior patterns before reaching users. Explainable corrections, not just error flags.
- **NeoTrix Mapping**: NT-SHIELD (pre-deployment quality gate), NT-REPAIR (error detection + correction), NT-CORE (meta-cognitive self-evaluation), ConsciousnessTree (health monitoring before action execution).

## Cross-Cutting Patterns (Cycle 392)

| Pattern | Projects | NeoTrix Domain |
|---------|----------|----------------|
| **Selective Memory** | Cortex, OpenMemory, screenpipe | NT-MEMORY |
| **Trajectory-Aware Routing** | IQ Routing, HarnessRouter | NT-IO, NT-CORE |
| **Behavioral Self-Improvement** | Reflxio, Vibe-Trading | NT-MIND |
| **Zero-Server/Edge Intelligence** | GitNexus, OfficeCLI | NT-SHIELD, NT-IO |
| **Pre-Deployment Quality Gates** | Prefactor, HarnessRouter | NT-SHIELD, NT-REPAIR |
