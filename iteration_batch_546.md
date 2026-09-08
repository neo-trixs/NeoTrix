# Iteration 546 — Agent Framework Landscape, Tool Use Patterns, Evaluation Critique

## Sources Consulted

### Agent Frameworks
1. wirecraft.ai — "CrewAI vs LangGraph vs AutoGen: 2026 Framework Guide" (2026-07-20)
2. devtoollab.com — "LangGraph vs CrewAI vs AutoGen 2026" (2026-06-17)
3. beri.net — "The D*AI*LY BRIEF: One of Them Is Retired" (2026-08-01)
4. shahvatsal.com — "Production Comparison 2026" (2026-06-18)
5. langchain.com — "Best AI Agent Frameworks in 2026" (2026-06-06)
6. trendix.tech — "LangGraph vs CrewAI vs OpenAI Agents SDK vs AutoGen" (2026-05-12)
7. groovyweb.co — "Framework Comparison 2026" (2026-04-12)

### Tool Use
8. developers.openai.com — "Function Calling Guide"
9. agent-engineering.ch — "Tool Use & Function Calling" (2026-02-18)
10. arxiv 2609.01736 — "HEART: Agent-Native Reusable Tool Primitives" (2026-09-01)
11. jatinbansal.com — "LLM Tool Use as Typed RPC" (2026-05-18)
12. airbyte.com — "Agent Tool Calling Implementation Guide" (2026-03-06)
13. dev.to/paxrel — "AI Agent Tool Use: Function Calling 2026" (2026-03-31)

### Agent Evaluation
14. swebench.com — SWE-bench Leaderboards
15. arxiv — "DeepSWE: Measuring Frontier Coding Agents" (2607.07946)
16. arxiv — "Claw-SWE-Bench: Agent Harnesses as First-Class Variables" (2606.12344v1)
17. ICML 2026 — "SWE-Bench Pro" (poster 61047)
18. zylos.ai — "AI Agent Evaluation Stack 2026" (2026-03-25)
19. arxiv — "SWE-Together: Multi-turn Coding Sessions" (2606.29957v1)

---

## Defects & Improvements Found (vs Batch 545)

Batch 545 established: execution-trace accumulator missing, calibration blind spot under distribution shift, RND finite-width collapse, AdamW running variance as free preconditioner, interacting parallel filters outperforming independent ones. Batch 546 adds:

### DEFECT 6: Checkpointing Granularity Blind Spot

**Source**: beri.net (source 3), trendix.tech (source 6)

LangGraph checkpoints state at **super-step boundaries only** — meaning state inside a long-running node is NOT saved. If step 7 of 12 makes three API calls and dies on the third, all three are replayed. CrewAI defaults to local SQLite via `@persist` decorator — fine for laptop, liability at 50K runs/month across replicas. Microsoft Agent Framework has "workflow checkpointing" but granularity unspecified.

**Implication for NeoTrix**: The SEAL pipeline's evolution phases are long-running multi-step processes. If an evolution node runs internally (distillation → absorption → test) and crashes mid-phase, all internal progress is lost. NeoTrix needs **sub-node checkpointing** — the SEAL stages themselves should be checkpoint boundaries, not just the outer evolution loop.

### DEFECT 7: Retry Without Idempotency = Duplicate Side Effects

**Source**: beri.net issue #5802 (source 3), jatinbansal.com (source 11)

CrewAI's `max_retry_limit` re-executes `@tool` decorated functions that already ran. If an agent touches a payment rail, outbound email, or trade execution, duplicates occur. The fix is **idempotency keys at the harness runtime layer**, not in individual tool code. Jatin Bansal frames this precisely: "Treat the LLM as an at-least-once caller, not an exactly-once one."

**Implication for NeoTrix**: NT-ACT's tool execution layer currently assumes single-shot tool calls. If a SEAL evolution cycle crashes mid-tool-execution and retries, the tool may fire again. NeoTrix needs a **harness-level idempotency registry** — tool call IDs with TTL-based deduplication, executed at the `nt_act` orchestration layer before any tool function body runs.

### DEFECT 8: Tool Selection Collapse Past ~30 Tools

**Source**: jatinbansal.com (source 11), arxiv 2609.01736 (source 10)

Tool description tokens serialize into a hidden system prompt on every API call. At 30+ tools, the "token tax" reaches 8-12K tokens before the user message. Worse, **tool selection accuracy collapses** — the model struggles to disambiguate similar tools, and descriptions interfere with each other. The arxiv paper proposes Tool Primitives (natural language interface) + HEART (Planner→Router→Verifier) to handle 25,519 tools. OpenAI's `tool_search` (gpt-5.4+) loads deferred tools only when needed.

**Implication for NeoTrix**: NeoTrix's capability network exposes dozens of tools per domain. With 7 domains × N tools each, the tool space is well past 30. The current approach of serializing all tool schemas into context is a **token budget bomb**. NeoTrix needs:
- A two-stage retrieval layer: embed tool descriptions, retrieve top-k relevant to current task
- Per-domain tool clustering so only the relevant domain's tools are injected
- HEART-style verification to catch hallucinated tool selections

### DEFECT 9: Benchmarks Measure Harness, Not Model

**Source**: Claw-SWE-Bench (source 16), Zylos Research (source 18)

Claw-SWE-Bench demonstrates that **harness choice is a first-class variable** — under a fixed Qwen 3.6-flash model, changing only the agent loop produces a 27.4pp spread (38.6% to 66.0% Pass@1). SWE-bench scores conflate three causally distinct factors: the LLM, the harness, and the task instances. SWE-Bench Verified has contamination evidence (OpenAI's Feb 2026 analysis found 59.4% of audited tasks have material test/design problems).

**Implication for NeoTrix**: When evaluating NeoTrix's self-evolution (SEAL pipeline's success rates), the harness (SEAL stages, tool ordering, retry logic) may dominate the measurement. If we report "SEAL achieves X% evolution success," we must decompose: how much is the LLM's reasoning vs. the SEAL scaffolding? NeoTrix's self-test should **isolate harness contribution** from model contribution, similar to Claw-SWE-Bench's design.

### DEFECT 10: Static Benchmarks Miss Multi-Turn Tool Orchestration

**Source**: SWE-Together (source 19), Zylos Research (source 18)

SWE-Together evaluates coding agents across multi-turn user sessions with evolving instructions, not just single-shot task descriptions. It reports **User Correction** (corrective steering required) as a diagnostic — stronger agents require fewer interventions. No existing benchmark evaluates **tool-calling loops under distribution shift** (tools that change behavior between calls, tools that fail intermittently, tools whose outputs degrade over time).

**Implication for NeoTrix**: NeoTrix's SEAL pipeline runs iterative tool-calling loops where each tool invocation changes the system state (KB mutations, evolution state updates). No benchmark currently tests whether an agent can maintain correct tool orchestration when:
- Tool outputs drift between calls (calibration blind spot from batch 545)
- Parallel tool calls create race conditions (batch 545's interacting filters)
- Tool schemas evolve mid-execution (dynamic tool catalogues)

### DEFECT 11: Token Cost Grows Quadratically with Step Count

**Source**: wirecraft.ai (source 1)

"Every loop re-sends the accumulated history, cost grows with the square of the step count." This is the **context accumulation problem** — not just token waste, but accuracy degradation as context lengthens. Models perform worse on early information when context grows. LangGraph has no built-in context compression; CrewAI adds context via Flow state; AutoGen has no persistent state by default.

**Implication for NeoTrix**: The SEAL pipeline's evolution loops re-send accumulated state every cycle. If an evolution runs 50+ cycles, the context grows quadratically and early evolution decisions get diluted. NeoTrix needs **context windowing with summary injection** — periodically distill accumulated state into a compressed summary, inject the summary into context, and drop the raw history.

### DEFECT 12: No Evaluation of Error Recovery Strategy Quality

**Source**: airbyte.com (source 12), dev.to/paxrel (source 13)

Agent tool calling guides emphasize "always return structured errors so the model can recover," but **no metric evaluates the quality of error recovery strategies**. The pattern is: tool fails → model gets error message → model tries alternative. But which alternative? A good recovery reduces system load; a bad recovery multiplies it (retry storm). No benchmark measures whether the agent's error recovery strategy is optimal, adequate, or catastrophic.

**Implication for NeoTrix**: NeoTrix's self-healing (NT-REPAIR) should not just "try alternatives" — it should rank alternatives by expected cost and pick the cheapest viable path. The evaluation should measure **recovery efficiency**: how many tool calls does it take to reach a working state after a failure? This is distinct from success rate — two systems can both "succeed" but one takes 3 calls and the other takes 15.

---

## What's NEW vs Batch 545

| Batch 545 Finding | Batch 546 Extension |
|---|---|
| Missing execution-trace accumulator | Now quantified: checkpointing granularity blind spot (DEFECT 6) — sub-node state lost, SEAL phases need own checkpoint boundaries |
| Calibration blind spot under distribution shift | Extended to tool domain: tool selection collapses past 30 tools (DEFECT 8), tool output drift between calls (DEFECT 10) |
| RND finite-width collapse | No direct extension this batch |
| AdamW running variance as free preconditioner | No direct extension this batch |
| Interacting parallel filters outperform independent ones | Extended: parallel tool calls assume independence — dangerous for state-touching tools (DEFECT 7's idempotency requirement) |
| **NEW** | Retry without idempotency = duplicate side effects (DEFECT 7) |
| **NEW** | Benchmarks measure harness, not model (DEFECT 9) |
| **NEW** | Token cost grows quadratically with step count (DEFECT 11) |
| **NEW** | No evaluation of error recovery strategy quality (DEFECT 12) |
| **NEW** | No benchmark tests tool orchestration under distribution shift (DEFECT 10) |

---

## Key Landscape Shifts (2026 Mid-Year)

1. **Framework consolidation**: AutoGen → maintenance mode; Microsoft Agent Framework 1.0 (April 2026) is the official successor. AG2 is a community fork, not Microsoft's path.
2. **LangGraph dominates production**: 50K+ GitHub stars, Klarna/Uber/JPMorgan deployments, explicit graph state with 3 durability modes.
3. **CrewAI = prototype speed**: Fastest time-to-working-agent, but checkpointing gap and no native HITL.
4. **Evaluation is broken**: SWE-bench Verified contamination confirmed, SWE-Bench Pro shows 32.4% verifier-judge disagreement, DeepSWE shows 1.4% (order of magnitude better).
5. **Tool calling is becoming RPC**: The "typed RPC" framing (source 11) is the correct mental model — schemas are IDL, agent loop is RPC client, all distributed systems problems (idempotency, timeouts, retries) reappear.
6. **HEART framework** (arxiv 2609.01736): Planner→Router→Verifier for 25K+ tool catalogues — addresses tool selection collapse via natural language interfaces.

---

## Actionable Next Steps for NeoTrix

1. **Implement sub-node checkpointing** in SEAL pipeline (DEFECT 6)
2. **Add harness-level idempotency registry** to NT-ACT tool execution (DEFECT 7)
3. **Build two-stage tool retrieval** layer for capability network (DEFECT 8)
4. **Decompose evolution success** into harness vs model contribution (DEFECT 9)
5. **Design tool-orchestration-under-shift benchmark** for SEAL evaluation (DEFECT 10)
6. **Implement context windowing** with periodic summary injection in evolution loops (DEFECT 11)
7. **Add recovery efficiency metric** to NT-REPAIR self-healing (DEFECT 12)
