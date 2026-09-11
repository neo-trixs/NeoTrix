# Trending Rankings — Cycle 353

**Date**: 2026-09-12  
**Focus**: AI agents, LLM tools, reasoning frameworks, memory/attention/routing patterns  
**Sources**: GitHub Trending, ProductHunt, arXiv, FindARepo, Ossinsight

---

## Top 10 New Projects (Not in Cycles 318-352)

### 1. Reflexio — AI Agent Self-Improvement Harness
- **GitHub**: [ReflexioAI/reflexio](https://github.com/ReflexioAI/reflexio) — ★ 363, Apache-2.0
- **What**: Turns user corrections and successful execution paths into persisted behavioral improvements for agents. Captures feedback per interaction, extracts playbooks, aggregates across users.
- **Key Pattern**: Behavioral learning layer — agents improve without retraining. Playbook extraction via LLM clustering with change detection. Expert learning by comparing agent vs human responses.
- **NeoTrix Relevance**: Maps to NT-MIND (self-evolution), NT-MEMORY (experience persistence). The "trigger/instruction/pitfall SOP" playbook format is a reusable template for SEAL pipeline skill crystallization. The user-scoped → shared aggregation mirrors NeoTrix's per-session → KB hub experience flow.
- **Stars**: 363 | **License**: Apache-2.0 | **URL**: reflexio.ai

### 2. oMLX — Mac LLM Server with Tiered KV Cache
- **GitHub**: [jundot/omlx](https://github.com/jundot/omlx) — ★ 21,504, Apache-2.0
- **What**: macOS-native LLM inference server with SSD-persisted KV cache, continuous batching, multi-model serving (LLM/VLM/embedding/reranker). Drop-in for Claude Code, OpenClaw, Cursor.
- **Key Pattern**: Hot/cold tiered KV caching — cache blocks persist to SSD in safetensors format, restored from disk across server restarts. Continuous batching at 4.14× speedup. Multi-Mac distributed inference over Thunderbolt.
- **NeoTrix Relevance**: Directly validates KVMem paged KV architecture (Axiom A2). The tiered RAM→SSD approach matches NeoTrix's planned memory hierarchy. Multi-Mac cluster mode maps to NT-PHYSICAL distributed embodiment. The admin dashboard pattern (menu bar + web) is a UX template for NT-IO.
- **Stars**: 21,504 | **License**: Apache-2.0 | **URL**: omlx.ai

### 3. Mozaik — TypeScript Runtime for Concurrent AI Agents
- **GitHub**: [jigjoy-ai/mozaik](https://github.com/jigjoy-ai/mozaik) — ★ 127, MIT
- **What**: Event-driven TypeScript runtime where agents communicate through semantic events, coordinate without predefined workflows, and operate in parallel non-blocking mode.
- **Key Pattern**: Agent awareness via participant manifests + shared runtime state. Situation Handlers (decouple reaction logic from agent). Interception layer for human-in-the-loop governance. Fire-and-forget inference with event-based results.
- **NeoTrix Relevance**: The Situation Handler pattern mirrors GWT's attention routing (event → specialist). Participant Manifest maps to NT-ACT capability registry. Shared Runtime State is a reference implementation for NT-MEMORY cross-session state. The governance interception layer aligns with NT-GOVERNANCE.
- **Stars**: 127 | **License**: MIT | **URL**: mozaik.jigjoy.ai

### 4. screenpipe — AI Agent Memory from Screen Recording (YC S26)
- **GitHub**: [screenpipe/screenpipe](https://github.com/screenpipe/screenpipe) — ★ 21,410, Source-Available
- **What**: Continuously captures screen + audio locally, extracts text via accessibility APIs (OCR fallback), transcribes with Whisper, stores in local SQLite. MCP server for agent context.
- **Key Pattern**: Event-driven capture (only records on meaningful activity — app switches, clicks, typing pauses). On-device privacy model. Pipes system — scheduled AI agents defined as markdown files that query screen data and take actions.
- **NeoTrix Relevance**: Validates NT-WORLD perception architecture (event-driven sensory → local storage → AI). The "Pipes" pattern (markdown-defined scheduled agents) is a reusable template for NT-ACT automation. Local-first privacy model aligns with NT-SHIELD egress guard philosophy. The MCP server pattern maps to NT-IO interface design.
- **Stars**: 21,410 | **License**: Source-Available | **URL**: screenpipe.com

### 5. Kastra — Runtime Authorization for AI Agents
- **GitHub**: [kastra.ai](https://kastra.ai) — Product of the Day #3 (PH Jul 22, 2026)
- **What**: Runtime authorization layer that evaluates every tool call, prompt, shell command against deterministic policies BEFORE execution. Sub-1ms latency. Unified control plane across Claude Code, Cursor, Codex, OpenClaw.
- **Key Pattern**: Pre-execution policy evaluation (not post-hoc). "Recon" scan of local agent history to discover risky past actions. Policy pack library (open source) + enterprise control plane.
- **NeoTrix Relevance**: Directly maps to NT-SHIELD sandbox egress policy (P2 pattern from absorbed terminology). Sub-1ms authorization validates the need for lightweight local policy evaluation in NT-SHIELD. The "Recon scan" pattern could enhance SelfTest T3 (detect past policy violations). Policy pack library is a reusable governance template.
- **Stars**: 737 PH followers | **License**: Open source runtime + commercial control plane | **URL**: kastra.ai

### 6. IQ Routing — Trajectory-Aware LLM Routing
- **What**: Drop-in gateway that classifies requests, serves from cache, routes to cheapest model that clears quality bar. Trajectory-aware — considers multi-turn history, not just current query.
- **Key Pattern**: Trajectory-level routing (route per turn in multi-turn episodes, not per query). Cache-first serving. Quality-bar threshold with cost optimization.
- **NeoTrix Relevance**: Extends GWT attention routing with cost-aware multi-turn decisions (Axiom A1: Cost-Aware Routing). The MTRouter research (arXiv:2604.23530) validates history-model joint embeddings for routing decisions — maps to NT-CORE's GWT salience computation. The cache-first pattern aligns with NT-MEMORY embedding cache.
- **URL**: producthunt.com/products/iq-routing

### 7. Doop — Multiplayer Design Canvas with AI Agents
- **GitHub**: [kgoedecke/doop](https://github.com/kgoedecke/doop) — ★ 586, AGPL-3.0
- **What**: Open-source multiplayer canvas where humans and AI agents design together live. Agents connect via MCP, stream frames, self-review, share design memory.
- **Key Pattern**: Visible agent collaboration (cursors, presence, per-frame editing). Design memory (pin exemplars, distill style rules). Self-review via headless renderer screenshots. Comments → tasks feedback loop.
- **NeoTrix Relevance**: The "visible agent collaboration" pattern is a UX template for NT-IO agent transparency. Design memory (exemplar → style rule distillation) mirrors SEAL skill crystallization. The MCP-native agent connection validates NT-IO interface patterns. Self-review loop maps to NT-REPAIR self-healing.
- **Stars**: 586 | **License**: AGPL-3.0 | **URL**: doop.design

### 8. Almanac — Agent with a Second Brain (YCW26)
- **What**: Always-on AI agent with its own computer, signed into your tools. Compiles work into a self-updating wiki, then uses that memory to execute tasks from Slack/iMessage.
- **Key Pattern**: Self-updating wiki (continuously compiled from Slack/GitHub/Gmail/Granola). Always-on cloud machine with own browser/terminal. Human-in-the-loop at decision points. Proactive task suggestion from background worker.
- **NeoTrix Relevance**: The "self-updating wiki" pattern validates NT-MEMORY's continuous knowledge compilation. "Always-on computer" maps to NT-PHYSICAL persistent embodiment. The proactive background worker pattern aligns with ConsciousnessTree's background growth cycle. Decision-point interception maps to NT-GOVERNANCE policy gates.
- **URL**: usealmanac.com | **Pricing**: $30-$200/user/mo

### 9. EquiRouter — Decision-Aware LLM Routing
- **arXiv**: 2602.03478
- **What**: Addresses "routing collapse" where routers converge to always using the strongest model, wasting smaller models. Learns instance-wise model rankings to restore proper routing.
- **Key Pattern**: Decision-aware routing that directly supervises model rankings per instance. "Routing Collapse" metric — measures when routers degenerate to single-model usage. Reduces cost by ~17% at GPT-4-level performance.
- **NeoTrix Relevance**: Directly applicable to NT-IO provider routing. The "routing collapse" diagnosis tool could be added to SelfTest suite. Instance-wise ranking (not just query-level) extends GWT salience with task-difficulty awareness. The 17% cost reduction at equivalent quality validates cost-aware routing (A1).
- **URL**: arxiv.org/abs/2602.03478

### 10. AgensFlow — Coordination-Policy Substrate for Multi-Agent Systems
- **arXiv**: 2605.27466
- **What**: Treats multi-agent coordination as an online policy-learning problem under partial observability. Learns which skill protocol, model binding, and coordination topology to select per task class.
- **Key Pattern**: Partially observable sequential decision process for agent coordination. Learned auditable routing over skills, models, and topologies. Online policy update from repeated trajectories. Inspectable routing decisions.
- **NeoTrix Relevance**: The "learned coordination topology" pattern directly maps to GWT's resonance-based routing. The partial observability formulation matches NeoTrix's real-world deployment constraints. Auditable routing aligns with NT-GOVERNANCE compliance requirements. The skill/model/topology selection framework is a meta-pattern for NT-CORE architecture decisions.
- **URL**: arxiv.org/abs/2605.27466

---

## Pattern Summary

| Pattern | Frequency | NeoTrix Domain |
|---------|-----------|----------------|
| **Cost-aware routing** | 3/10 (IQ Routing, EquiRouter, MTRouter) | NT-IO + NT-CORE (GWT) |
| **Behavioral learning** | 2/10 (Reflexio, screenpipe Pipes) | NT-MIND + NT-MEMORY |
| **Tiered KV cache** | 2/10 (oMLX, screenpipe) | NT-MEMORY + NT-PHYSICAL |
| **Runtime authorization** | 1/10 (Kastra) | NT-SHIELD |
| **Agent coordination** | 2/10 (Mozaik, AgensFlow) | NT-ACT + NT-GOVERNANCE |
| **Always-on embodiment** | 2/10 (Almanac, oMLX) | NT-PHYSICAL + NT-IO |
| **Self-review/self-healing** | 2/10 (Doop, Reflexio) | NT-REPAIR + NT-MIND |

---

## Key Insight

The dominant pattern in this cycle is **"routing intelligence"** — three distinct approaches (trajectory-aware, decision-aware, coordination-policy) all converging on the same problem: how to route tasks to the cheapest capable model without degenerating to single-model usage. NeoTrix's GWT + cost-aware routing (Axiom A1) is well-positioned but should incorporate the "routing collapse" diagnostic to detect degenerate behavior.

A secondary trend is **"agent memory as a first-class product"** — screenpipe, Almanac, and Reflexio all treat agent memory not as a feature but as the core product. This validates NeoTrix's KB-first architecture.
