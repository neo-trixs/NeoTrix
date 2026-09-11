# Model Reverse-Engineering Analysis & Universal Fusion Plan

**Date:** 2026-09-11
**Research Depth:** 4 model families + universal patterns
**Sources:** 12+ web searches, Anthropic/OpenAI/Google/Meta/Qwen/DeepSeek official docs, 5 academic papers

---

## Executive Summary

All frontier models converge on the same 4-stage agent loop: **Context Assembly → Reasoning → Tool Execution → Feedback Integration**. The divergence is in *how* each stage is optimized. This document extracts the architectural DNA from 6 model families and proposes universal fusion plans applicable to NeoTrix's multi-model backend.

---

## 1. Claude (Anthropic) — Architecture Deep Dive

### Core Reasoning Architecture
- **ReAct while-loop**: Single `queryLoop()` AsyncGenerator — no DAGs, no classifiers, no RAG
- **Extended Thinking**: Optional `...` blocks — model reasons internally before producing output
- **Adaptive Thinking** (Claude 4+): Model dynamically allocates thinking budget per request, steered by `budget` parameter rather than fixed token count
- **Interleaved Thinking**: Model thinks between tool calls, reasoning about results before deciding next step (auto-enabled on adaptive thinking models)

### Tool Use Protocol
- **Format**: `{type: "function", name, description, input_schema}` in `tools` array
- **Stop reasons**: `tool_use` (execute tools), `end_turn` (done), `max_tokens` (budget exhausted), `pause_turn` (server tool loop hit limit)
- **Three tool categories**:
  - **User-defined client tools**: JSON schema → model emits `tool_use` block → your code executes → send `tool_result`
  - **Anthropic-schema client tools**: `bash`, `text_editor`, `memory` — trained-in schemas, same execution model
  - **Server-executed tools**: `web_search`, `code_execution`, `web_fetch` — Anthropic runs internally, only `server_tool_use` blocks visible
- **Programmatic Tool Calling**: Model writes Python code in code_execution sandbox that calls tools as functions — intermediate results don't enter context, only final stdout does. ~37% token reduction on multi-tool workflows.
- **Tool Search Tool** (`tool_search`): Deferred tool discovery — tools marked `defer_loading: true` aren't in initial context; model searches and loads on-demand. Preserves prompt cache.
- **Mid-conversation tool changes** (beta): Add/remove tools between turns while cached prefix survives.
- **Tool Choice**: `auto` (default), `any`, `tool` (forced), `none`. Fable/Mythos 5.1 reject forced tool use.

### Context Management Strategy
- **5-layer compaction pipeline** (runs before every model call):
  1. **Budget reduction**: Trim individual tool outputs exceeding size limits
  2. **Snip**: Trim redundant historical messages
  3. **Microcompact**: Clean old tool results no longer needed
  4. **Context collapse**: Non-destructive projection-based folding (read-time, no original modification)
  5. **Auto-compact**: Full model summary as last resort
- **Prefix caching**: Two breakpoints (system + last message) — only new messages processed per turn
- **Context awareness**: Sonnet 4.5+ tracks remaining token budget via injected XML tags
- **Subagent isolation**: Sidechain transcripts — only summaries return to parent context
- **Memory tool**: File-based persistent memory across conversations (client-side)

### Safety/Alignment
- **Constitutional AI (RLAIF)**: AI feedback guided by written constitution
- **7 permission modes**: `plan` → `default` → `acceptEdits` → `auto` (ML classifier) → `dontAsk` → `bypassPermissions` (+ internal `bubble`)
- **Deny-first rule evaluation**: Broad deny always overrides narrow allow
- **YOLO classifier**: ML-based auto-mode classifier with CoT evaluation of tool safety
- **27 hook events**: 5 safety-related, 22 lifecycle/orchestration
- **Shell sandbox**: Independent filesystem/network restriction layer

### Memory/Learning
- **CLAUDE.md hierarchy**: Managed settings → directory-specific files → auto-memory entries
- **autoDream**: Background process that consolidates memory (orient → gather → consolidate → prune)
- **KAIROS**: Persistent proactive assistant (append-only daily logs, autonomous action)

---

## 2. GPT/GPT-5 (OpenAI) — Architecture Deep Dive

### Core Reasoning Architecture
- **ReAct loop** with structured output enforcement
- **Reasoning models** (o4-mini, GPT-5): `reasoning` items in responses, must be passed back with tool call outputs
- **Programmatic Tool Calling**: Model writes and runs JavaScript (not Python like Claude) in isolated V8 runtime — supports `top-level await`, no Node.js/filesystem/network. Same intermediate-result-hiding benefit.
- **Constrained decoding**: FSM (finite state machine) restricts model output to valid JSON matching function schemas — 92% reduction in invalid responses vs unconstrained

### Tool Use Protocol
- **Function tools**: `{type: "function", function: {name, description, parameters, strict}}` in `tools` array
- **Custom tools**: Free text input/output with optional CFG (context-free grammar) constraints via `lark` or `regex`
- **Strict mode** (`strict: true`): Model enforces schema compliance at inference time — refuses to generate non-conforming calls. 89% reduction in invalid schema errors.
- **Tool search** (`tool_search`): Deferred loading, only `gpt-5.4`+ supported
- **Parallel function calling**: GPT-5+ can call multiple functions in one turn; built-in tools cannot be in parallel batch
- **Response format**: Chat Completions (`tool_calls` array) and Responses API (`output` array with `function_call` items)
- **`allowed_callers`**: Controls `["direct"]` vs `["programmatic"]` invocation context
- **Custom CFG grammars**: `grammar` parameter with `lark` or `regex` syntax for constrained generation
- **Namespaces**: Group related tools by domain (`crm`, `billing`, `shipping`)

### Context Management Strategy
- **128K context window** (GPT-4o), up to 1M+ for GPT-5 series
- **Structured outputs**: `response_format: {type: "json_schema"}` for guaranteed valid JSON
- **Tool definitions in system message**: Injected via trained-in syntax, count against context limit
- **Conversation state preservation**: Full `tool_calls` array must be preserved exactly (id, type, function.name, function.arguments)

### Safety/Alignment
- **RLHF** (Reinforcement Learning from Human Feedback)
- **Strict mode**: Schema enforcement prevents invalid actions
- **`tool_choice` control**: Auto, required, forced function, allowed tools subset
- **System prompt injection resistance**: Training-based, not structural

### Memory/Learning
- **Stateless API**: Each request independent; conversation history managed client-side
- **Persistent threads** (Assistants API): Built-in memory, file search, code interpreter
- **Reasoning items preservation**: Must pass back reasoning content for continuity

---

## 3. Gemini (Google DeepMind) — Architecture Deep Dive

### Core Reasoning Architecture
- **Reasoning-enhanced models** (Gemini 3 Pro): Extended thinking with reasoning tokens
- **Serial algorithmic simulation**: Model executes step-by-step symbolic procedures in thinking traces — demonstrated on Blocksworld planning (100+ goal blocks, optimal plans)
- **Geometric navigation hypothesis**: Spatial view of problem space — navigating P* graph structures
- **Subgoal decomposition** (MiRA): Dynamic milestoning framework — decomposes high-level goals into structured subgoals with milestone-based reward shaping
- **Agent Factory**: Multi-agent orchestration with task specifications, monitoring, and feedback loops

### Tool Use Protocol
- **Function declarations**: OpenAI-compatible format (`tools` array with `function` objects)
- **Server-side tools**: Web search, code execution run on Google infrastructure
- **Project Mariner**: Vision-first web navigation using Gemini 2.5 — processes rendered visual output, not HTML/DOM
  - **Vision Encoder**: Fine-tuned visual understanding at 60 FPS
  - **Action Planner**: Task graph with backtracking support
  - **Execution Engine**: Low-level browser event translation

### Context Management Strategy
- **Context caching**: Server-side KV cache reuse for repeated prompts
- **Long context**: Gemini 3 Pro supports 1M+ tokens
- **Agentic scaffolding vs long context**: Complementary — "the amount of context we will need to solve the problem is going to be extremely high" (Prateek Jain, Google DeepMind)
- **MatFormer**: Elastic model that dials power up/down based on task complexity, server load, or token cost

### Safety/Alignment
- **AI Control Roadmap**: Defense-in-depth treating agents as "insider threats"
  - **Detection levels D1-D4**: Monitor reasoning chains → analyze behavior → inspect inner workings
  - **Prevention/Response R1-R3**: Async review → sync blocking → real-time prevention
- **Supervisor agents**: Trusted AI systems review working agent's reasoning, actions, plans
- **Zero-Retention Agent Processing**: Page content in ephemeral memory, purged on completion
- **MITRE ATT&CK-inspired threat taxonomy**: Track agent risks via tactics/techniques

### Memory/Learning
- **Milestone-based RL** (MiRA): Dense reward signals at intermediate checkpoints
- **Offline RL fine-tuning**: Dual-critic architecture (goal-conditioned + potential critic)
- **Long-horizon planning**: Subgoal decomposition for multi-step tasks

---

## 4. Open Source Models

### Llama 4 (Meta)

#### Architecture
- **MoE (Mixture of Experts)**: Llama 4 Scout (17B active, 109B total, 16 experts) and Maverick (17B active, 400B total, 128 experts)
- **Alternating dense + MoE layers**: Each token → shared expert + one of N routed experts
- **Native multimodality via early fusion**: Text + vision tokens unified in backbone
- **iRoPE architecture**: Interleaved attention layers without positional embeddings + inference-time temperature scaling → 10M token context for Scout

#### Agent Framework
- **Llama Conductor**: Python orchestration engine
  - Declarative YAML agent definitions
  - Dynamic task routing via 405B model
  - Hierarchical supervisor-worker patterns
  - Parallel execution with synchronization
- **Tool Integration Layer**: Compatible with both MCP and OpenAI function-calling format
- **Llama Memory**: Three-tier memory system
  - Working memory: In-process short-term
  - Episodic memory: FAISS vector database
  - Semantic memory: Knowledge graph
- **Agent models**: 8B (edge), 70B (production), 405B (orchestration)
- **AgentInstruct-2M**: 2M curated agent behavior examples

#### Safety
- **Llama Guard**: System-level safety filter on inputs/outputs
- **Prompt Guard**: Prompt injection resistance
- **Code Shield**: Code execution safety

### Qwen 3.5 (Alibaba)

#### Architecture
- **MoE variants**: Qwen3.5-35B-A3B, Qwen3.5-397B-A17B
- **Reasoning model**: `<think>...</think>` blocks with `reasoning_content` field
- **XML-based tool calls**: `<tool_call><function=name><parameter=param>value