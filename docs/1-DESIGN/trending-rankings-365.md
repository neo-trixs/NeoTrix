# Trending Rankings — Cycle 365 (2026-09-12)

## 10 New Projects (Not in Cycles 318–364)

### 1. Graft — Context Engine for Coding Agents
- **URL**: https://github.com/trailhq/Graft
- **Stars**: 4,917
- **Language**: TypeScript/Python
- **What**: Turbocharges Claude Code, Cursor, Codex, Gemini by injecting codebase-specific context bundles upfront. Ships as MCP server, CLI tool, and library. Up to 4× cheaper, 3× faster, +12 pts SWE-bench correctness vs cold-start baseline. Code-as-pull vs code-as-injection modes.
- **Key Insight**: "Push context up front, not on demand" — `graft ask --source` bundles a repo's structure into a context prelude. The pull variant (graft_find_code/graft_file_api) charges only when asked, trading some speed for +5 pts correctness. Tree-sitter-powered code graph extraction.
- **NeoTrix Mapping**: NT-CORE → GWT attention routing could adopt push-first context injection. NT-MEMORY → context prelude = KB embedding preload for active workspace. NT-IO → MCP server integration aligns with NeoTrix's MCP protocol support.

### 2. 9Router — Free AI Router & Token Saver
- **URL**: https://github.com/decolua/9router
- **Stars**: 23,774
- **Language**: TypeScript/Go
- **What**: Open-source AI gateway connecting all coding agents to 40+ providers (100+ models). RTK token compression saves 20-40% tokens per request. Smart 3-tier fallback: Subscription → Cheap → Free. Multi-account load balancing. OAuth auto-refresh. Cloud sync.
- **Key Insight**: RTK (40K stars standalone) compresses tool outputs (git diff, grep, ls) before sending to LLM. Caveman mode injects terse-speak prompt for 65% output token savings. Ponytail integration for YAGNI-first code generation.
- **NeoTrix Mapping**: NT-IO → Ordered Backend Router pattern (P4) implemented at production scale. NT-ACT → token-aware task routing. NT-SHIELD → multi-account redundancy as defense-in-depth.

### 3. Webwright — Microsoft Browser Agent Framework
- **URL**: https://github.com/microsoft/Webwright
- **Stars**: 5,961
- **Language**: Python
- **What**: SOTA browser agent achieving 86.7% on Online-Mind2Web and 60.1% on Odysseys (long-horizon tasks). Code-as-action beats coordinate prediction. Skill Factory distills solved tasks into reusable parameterized CLI tools that run standalone in ~40s with zero tokens. Supports Claude Code, Codex, OpenClaw, Hermes plugins.
- **Key Insight**: "Skills are programs, not context the model reads." Skill Factory produces parameterized tools from solved tasks — reuse lifts held-out accuracy 55% → 70% (+15 pp). Skills are composable, versionable, run without model invocation.
- **NeoTrix Mapping**: NT-ACT → Skill crystallization pattern (SKILL.md contract). NT-WORLD → browser automation perception. NT-MIND → distillation pipeline: task → script → parameterized skill.

### 4. Hypha — Harness-Oriented Agent System Framework
- **URL**: https://github.com/CodeSoul-co/Hypha
- **Stars**: 165
- **Language**: TypeScript
- **What**: Production agent runtime with Agent Core (reasoning/ReAct) + Production Harness (FSM control, policies, checkpoints, recovery, replay, audit). DomainPack compiles product-specific behavior into shared runtime. Cache & Reuse Plane accelerates without becoming authority.
- **Key Insight**: Cache is disposable projection, never source of truth. Event/Artifact/checkpoint evidence is the authority. DomainPack decouples product from runtime. FSM-driven execution with bounded continuation and recovery workers.
- **NeoTrix Mapping**: NT-CORE → FSM execution model maps to ConsciousnessTree cycle stages. NT-MEMORY → Cache & Reuse Plane = experience-tree lazy branch loading. NT-REPAIR → recovery workers + checkpoint replay.

### 5. Eve — Vercel Filesystem-First Agent Framework
- **URL**: https://github.com/vercel/eve
- **Stars**: 4,957
- **Language**: TypeScript
- **What**: Core agent capabilities live in conventional filesystem locations. Agents read their own config from node_modules/eve/docs. Markdown-first skill definitions. Filesystem as interface contract — projects are inspectable, extensible, operable by reading directory structure.
- **Key Insight**: "Agent lives in the filesystem" — configuration, skills, memory all manifest as files. No hidden state. Coding agents can read their own framework docs locally. Filesystem is the API surface.
- **NeoTrix Mapping**: NT-CORE → VSA HyperCube could use filesystem-as-memory metaphor. NT-MEMORY → file-based experience nodes. NT-ACT → skill files as executable contracts.

### 6. Omnigent — Meta-Harness for Agent Orchestration
- **URL**: https://github.com/temporalio/omnigent
- **Stars**: New (launched Aug 2026)
- **Language**: Python
- **What**: Open-source meta-harness orchestrating Claude Code, Codex, Cursor, OpenCode, Hermes, Pi across terminal/browser/phone/desktop. Swap harnesses without rewriting. Policy enforcement, sandboxing, real-time collaboration. Temporal.io-backed for durable execution.
- **Key Insight**: "Harness-agnostic orchestration" — one interface to rule all agents. Policy layer separates what agents can do from how they do it. Durable execution survives disconnects.
- **NeoTrix Mapping**: NT-ACT → harness-agnostic execution layer. NT-SHIELD → policy enforcement as orthogonal concern. NT-IO → multi-surface interface (terminal/web/mobile).

### 7. GitAgent — Git-Native AI Agent Framework
- **URL**: https://github.com/open-gitagent/gitagent
- **Stars**: 670
- **Language**: TypeScript
- **What**: Agent IS a git repository — identity, rules, memory, tools, skills are all version-controlled files. "Agents as repos" paradigm. MCP client/server support. Voice mode split into separate package for supply-chain compliance.
- **Key Insight**: Version-controlled agent state means every agent change is a git commit. Rollback, branch, merge agent configurations like code. Identity = committed files, not runtime state.
- **NeoTrix Mapping**: NT-MEMORY → version-controlled experience nodes. NT-CORE → agent identity as committed artifact (ConsciousnessTree state). NT-REPAIR → git-based rollback for self-healing.

### 8. KISS Sorcar — Multi-Model Agent with 641-Model Catalog
- **URL**: https://pypi.org/project/kiss-agent-framework/
- **Stars**: Growing (Berkeley research)
- **Language**: Python
- **What**: Open-source agent framework supporting 641 models across 9 provider categories. Multi-model workflows via prompts (mix OpenAI, Anthropic, Gemini in same task). Runs as VS Code extension + browser app + Python SDK. LLM as scheduler replacing complex agent frameworks with "a paragraph of prompt."
- **Key Insight**: "Complex AI systems can be replaced with a paragraph of prompt." Model catalog with prices, context lengths, capability flags (fc/gen/emb). Built-in Claude Code CLI and Codex CLI support.
- **NeoTrix Mapping**: NT-IO → model routing with cost/capability metadata. NT-CORE → prompt-as-scheduler maps to GWT attention routing. NT-ACT → subagent spawning via `rlm(...)`.

### 9. GoModel — Open-Source OpenRouter in Go
- **URL**: https://gomodel.enterpilot.io/
- **Stars**: New (ProductHunt Sep 9 2026)
- **Language**: Go
- **What**: Self-hosted AI gateway in Go. One OpenAI-compatible API for every provider. Budgets, caching, guardrails, load balancing, failover. Single binary ~20MB Docker image. MIT license. Alternative to OpenRouter and LiteLLM.
- **Key Insight**: Guardrails + load balancing + failover in a single lightweight binary. Semantic caching reduces repeated LLM calls. Budget management at the gateway level, not per-provider.
- **NeoTrix Mapping**: NT-IO → provider gateway with guardrails. NT-SHIELD → egress guard at gateway level. NT-ACT → budget-aware task routing.

### 10. Noodle Seed — Agent Governance Runtime for Products
- **URL**: https://noodleseed.com/
- **Stars**: New (ProductHunt Sep 9 2026, Score 254)
- **Language**: TypeScript
- **What**: Governed runtime for exposing workflows to AI agents. Identity, permissions, secrets, audit, operations — all built-in. Expose capabilities via branded assistant inside your product + external agents. Instead of stitching MCP SDKs, get a governed runtime.
- **Key Insight**: "Governed runtime for agent-product integration" — the missing layer between "build an agent" and "ship agent capabilities to users." Identity + permissions + audit as first-class concerns.
- **NeoTrix Mapping**: NT-SHIELD → governance layer for NT-ACT tool exposure. NT-IO → branded agent interface. NT-MEMORY → audit trail as experience nodes.

---

## Meta-Patterns Across Cycle 365

| Pattern | Projects | NeoTrix Implication |
|---------|----------|---------------------|
| **Token Compression** | Graft, 9Router (RTK) | NT-IO provider layer should compress tool outputs before LLM calls |
| **Skill-as-Program** | Webwright Skill Factory, Hypha DomainPack | SEAL pipeline should crystallize reusable executable skills |
| **Harness-Agnostic Orchestration** | Omnigent, KISS Sorcar | NT-ACT should support multi-harness execution |
| **Filesystem-as-Interface** | Eve, GitAgent | Experience nodes as version-controlled filesystem artifacts |
| **Governed Agent Runtime** | Noodle Seed, GoModel | NT-SHIELD governance layer for production agent exposure |
| **Context Push vs Pull** | Graft (push-first), 9Router (pull-on-demand) | GWT attention routing: pre-computed salience vs on-demand |
