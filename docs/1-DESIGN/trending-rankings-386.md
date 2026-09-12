# Trending Rankings — Cycle 386

**Date**: 2026-09-12
**Sources**: GitHub Trending, Product Hunt, ODSC, ByteByteGo, DataAIHub

## 10 New Projects (Not in Cycles 318–385)

### 1. OmniAgent
- **URL**: https://github.com/YeQing17-2026/OmniAgent
- **Stars**: 2,557 | **Lang**: Python
- **What**: Self-evolving agent framework with full-dimensional self-evolution (OmniEvolve): skill evolution, context evolution, brainmodel evolution via online RL. Dual-layer Deep Reflexion loop. Hyper-Harness with dynamic multi-agent + 4-layer security scanning.
- **NeoTrix Mapping**: NT-MIND (self-evolution pipeline), NT-CORE (GWT attention routing via progressive context loading), NT-SHIELD (4-layer security scanning)

### 2. Qwen-AgentWorld
- **URL**: https://github.com/qwenlm/qwen-agentworld
- **Stars**: 984 | **Lang**: Python
- **What**: Language world model (MoE 35B/3B active) simulating agentic environments across 7 domains (MCP, Search, Terminal, SWE, Android, Web, OS). Native world model from CPT stage. AgentWorldBench evaluation. Beats GPT-5.4 overall score (58.71 vs 58.25).
- **NeoTrix Mapping**: NT-WORLD (world simulation), NT-CORE (E8 reasoning over environment states), NT-MEMORY (KB environment modeling)

### 3. Nanobot
- **URL**: https://github.com/HKUDS/nanobot
- **Stars**: 47,663 | **Lang**: Python
- **What**: Ultra-lightweight self-hosted personal AI agent framework. WebUI, tools, long-term memory (Dream), MCP, multi-agent workflows, scheduled automation. v0.3.0 "Agency Release" adds subagent coordination, model switching per session.
- **NeoTrix Mapping**: NT-IO (interface gateway), NT-MEMORY (Dream long-term memory), NT-ACT (tool orchestration)

### 4. Harden AIF
- **URL**: https://harden.run
- **Stars**: Product Hunt #2 (Sep 9, 2026, 405 pts)
- **What**: Free, local security tool for AI coding agents. Post-trained model checks tool calls before execution using request+session context. Beats frontier models on agent-security benchmarks. Keeps repo and tool output on-device.
- **NeoTrix Mapping**: NT-SHIELD (tool call validation, sandbox security), NT-CORE (pre-execution reasoning)

### 5. GitNexus (Akon Labs)
- **URL**: https://www.producthunt.com/products/gitnexus-akon-labs
- **Stars**: 45,000+ GitHub | **Lang**: N/A
- **What**: Knowledge Graph Kernel unifying every codebase into one source of truth for coding agents. Deterministic graph (callers, imports, impact) vs embedding guesses. 51% cheaper agent runs over MCP.
- **NeoTrix Mapping**: NT-MEMORY (KB graph representation), NT-CORE (VSA HyperCube analog), NT-WORLD (code graph traversal)

### 6. HyperProbe
- **URL**: https://www.producthunt.com/products/hyperprobe
- **Stars**: Product Hunt #4 (Sep 5, 2026, 208 pts)
- **What**: Lets AI agents debug production without redeploying. Read-only probes dropped into running services capture variable state. Agents debug with local-repro fidelity.
- **NeoTrix Mapping**: NT-REPAIR (production debugging), NT-ACT (read-only probe execution), NT-SHIELD (safe sandboxed probes)

### 7. GNAP (Git-Native Agent Protocol)
- **URL**: https://github.com/farol-team/gnap
- **Stars**: 1,747 (awesome-ai-agents-2026 list)
- **What**: Coordinate AI agent teams with 4 JSON files in a git repo. No server, no database. Any agent that can git push can participate. Git-native agent coordination.
- **NeoTrix Mapping**: NT-ACT (agent orchestration), NT-MEMORY (git-versioned state), NT-NEXUS (cross-session coordination)

### 8. Mastra Factory
- **URL**: https://mastra.ai
- **Stars**: Product Hunt #1 (Sep 9, 2026, 491 pts)
- **What**: Framework for building AI-powered apps and agents with workflows, memory, streaming, evals, tracing, and Studio (interactive UI for dev/testing). "From issue to production, run by agents."
- **NeoTrix Mapping**: NT-IO (dev interface), NT-MEMORY (agent memory), NT-ACT (workflow orchestration)

### 9. Noodle Seed
- **URL**: https://noodleseed.com
- **Stars**: Product Hunt (Sep 9, 2026, 254 pts)
- **What**: Makes products ready for AI agents. Expose workflows through branded assistant inside product, same capabilities available to external agents. Governed runtime for identity, permissions, secrets, audit, operations.
- **NeoTrix Mapping**: NT-SHIELD (governed runtime, permissions), NT-IO (branded interface), NT-ACT (agent integration)

### 10. DeerFlow v2.0
- **URL**: https://github.com/bytedance/deer-flow
- **Stars**: 25,000+ | **Lang**: Python
- **What**: ByteDance open-source super agent harness. Ground-up rewrite v2.0. Orchestrates sub-agents, memory, sandboxes, extensible skills. #1 GitHub Trending Feb 2026.
- **NeoTrix Mapping**: NT-ACT (sub-agent orchestration), NT-MEMORY (agent memory), NT-SHIELD (sandbox execution)

## Trend Analysis

| Pattern | Count | NeoTrix Domain |
|---------|-------|----------------|
| Self-evolution / adaptive agents | 3 | NT-MIND, NT-CORE |
| Security / governed execution | 3 | NT-SHIELD |
| Knowledge graphs / memory | 3 | NT-MEMORY |
| Multi-agent orchestration | 3 | NT-ACT |
| World simulation | 1 | NT-WORLD |
| Production debugging | 1 | NT-REPAIR |

## Key Signals

- **Self-evolution is mainstream**: OmniAgent's full-dimensional self-evolution (skill+context+brainmodel) validates NT-MIND SEAL pipeline direction
- **Git-native coordination**: GNAP + GitNexus show agents-as-repos pattern gaining traction — aligns with NeoTrix KB hub pattern
- **Security as first-class concern**: Harden AIF (405 pts) proves agent security is a $0 premium product opportunity
- **World models beat frontier**: Qwen-AgentWorld-397B beating GPT-5.4 validates environment simulation for agent reasoning
- **Debugging without redeploying**: HyperProbe validates NT-REPAIR production debugging approach
