# Trending Rankings — Cycle 381

**Date:** 2026-09-12
**Sources:** GitHub Trending (Sep 2026), Product Hunt, GitTrend, StartupCorners, AIToolly

---

## Top 10 New Projects (Not in Cycles 318–380)

| # | Project | Stars | Category | Key Pattern | NeoTrix Domain |
|---|---------|-------|----------|-------------|----------------|
| 1 | **loopx** (huangruiteng) | 624+ | Loop-engineering state kernel for durable agent teams | Lightweight state kernel — quota-aware, long-running agent loop management with durable session state. Production-grade loop engineering for multi-agent teams | NT-CORE + NT-ACT |
| 2 | **screenpipe** (YC S26) | 5K+ | AI that records your computer work to power agents | Continuous screen recording → agent memory. OS-level activity capture feeds agent context — "agent that sees what you do" without explicit input | NT-WORLD + NT-MEMORY |
| 3 | **TencentDB-Agent-Memory** (TencentCloud) | 1367+ | Team-level memory hub for shared agent use | Converts conversations + docs + code into 4 reusable memory assets. Team-shared agent memory — context as collaborative, not individual | NT-MEMORY |
| 4 | **lean-ctx** (yvgude) | — | Local Rust binary for context intelligence layer | Controls what agents read, remember, and touch — 76 MCP tools. Context as governed resource: selective disclosure, not full dump | NT-CORE (GWT) + NT-SHIELD |
| 5 | **clutch** (ellmos-ai) | — | Provider-neutral LLM router with auto-learning budget zones | Auto-learning router + budget zones. Model routing adapts based on cost/performance — zero-config optimization across providers | NT-IO |
| 6 | **Flare** (ProductHunt) | — | Graph-first IDE and interactive map for agentic coding | Codebase-as-graph visualization for agent navigation — structural understanding, not file-level. Agent-native IDE design | NT-WORLD + NT-CORE |
| 7 | **GitNexus** (Akon Labs) | — | Open-source kernel for coding agents | Agent runtime kernel — provides primitives for coding agents (session, tools, context, review) as reusable infrastructure | NT-ACT + NT-IO |
| 8 | **Revolt** | — | AI for Software Engineering with interactive sessions | Workflow automation for SE tasks — interactive sessions (not just CLI). Structured agent-human collaboration patterns | NT-ACT |
| 9 | **Murmell** | — | Cloud canvas where team and AI agents work together | Real-time collaborative canvas for human-agent co-working. Shared workspace model — agents and humans in same visual context | NT-IO + NT-ACT |
| 10 | **hara** (hara-cli) | — | Coding agent CLI that runs like an engineering org | Role routing, plan DAGs, approvals, memory, 10-platform chat gateway. Engineering-org metaphor for agent coordination — structured decision flows | NT-ACT + NT-MIND |

---

## Pattern Analysis

### 1. Durable Loop Engineering (loopx, hara)
- **loopx**: Lightweight state kernel for durable, quota-aware agent loops. Long-running agents need state management across interruptions — loop engineering as first-class infrastructure.
- **hara**: Coding agent CLI with plan DAGs, approvals, and role routing. Engineering-org metaphor — agents don't just execute, they follow organizational decision flows.
- **NeoTrix mapping**: Validates SEAL pipeline's need for durable state. loopx's quota-aware loops map to SEAL's cycle budgeting. hara's plan DAGs = structured task decomposition that NT-ACT could adopt for complex multi-step tool chains.

### 2. Agent Memory as Shared Resource (TencentDB-Agent-Memory, lean-ctx)
- **TencentDB-Agent-Memory**: Team-level memory hub — 4 reusable memory assets (conversations, docs, code, context). Memory is collaborative, not per-agent. Context as shared infrastructure.
- **lean-ctx**: Local Rust binary acting as context intelligence layer — controls what agents read, remember, touch. 76 MCP tools for selective context disclosure.
- **NeoTrix mapping**: TencentDB's 4 memory assets align with NT-MEMORY's KB + experience tree + domain knowledge + session context. lean-ctx's selective disclosure validates GWT's attention filtering — not everything should be broadcast. Both confirm Axiom A2 (context as scarce resource).

### 3. OS-Level Agent Perception (screenpipe, Flare)
- **screenpipe**: Continuous screen recording → agent memory. OS-level activity capture feeds agents without explicit input. "Agent that sees what you do" — ambient context.
- **Flare**: Graph-first IDE for agentic coding. Codebase-as-graph visualization gives agents structural understanding of code, not just file contents.
- **NeoTrix mapping**: screenpipe validates NT-WORLD's ambient perception model — agents should passively observe, not just actively query. Flare's graph view aligns with NT-CORE's HyperCube representation — structure as first-class data.

### 4. Provider-Neutral Routing (clutch, GitNexus)
- **clutch**: Auto-learning LLM router with budget zones. Routes tasks to cheapest capable model, learns over time. Zero-config optimization.
- **GitNexus**: Open-source kernel for coding agents — provides runtime primitives (session, tools, context) as reusable infrastructure.
- **NeoTrix mapping**: clutch validates NT-IO's ordered backend router (P4) — routing as first-class concern. GitNexus's kernel-as-primitives model aligns with NT-ACT's tool-as-action philosophy — provide infrastructure, let agents compose.

### 5. Human-Agent Co-Working (Murmell, Revolt)
- **Murmell**: Cloud canvas for team + AI agents. Real-time collaborative workspace — agents and humans share visual context.
- **Revolt**: Interactive sessions for SE tasks. Not just CLI — structured agent-human collaboration with feedback loops.
- **NeoTrix mapping**: Both validate the "agent as coworker, not tool" pattern. Murmell's shared canvas = NT-IO's web server + real-time collaboration. Revolt's interactive sessions map to NT-ACT's orchestration layer — agents need structured collaboration modes.

---

## Signal Strength

| Pattern | Signal | Cycle Δ |
|---------|--------|---------|
| Durable loop state | New | +1 |
| Shared agent memory | Strong | +2 |
| OS-level perception | Growing | +1 |
| Provider-neutral routing | Strong | +2 |
| Human-agent co-working | New | +1 |

---

## Cross-Reference with Axioms

| Axiom | Confirmation | Project |
|-------|-------------|---------|
| A2: Context as Scarce Resource | lean-ctx selective disclosure, TencentDB memory compression | lean-ctx, TencentDB-Agent-Memory |
| P1: Model Routing | clutch auto-learning router | clutch |
| P4: Ordered Backend Fallback | clutch provider-neutral routing | clutch |
| P3: Profile-Driven Adaptation | TencentDB team-level memory profiles | TencentDB-Agent-Memory |
| P5: Skill as Reusable Template | GitNexus agent kernel primitives | GitNexus |

---

## Recommendation for NeoTrix

**Immediate absorption candidates:**
1. **lean-ctx** — Context intelligence layer. GWT could use its selective disclosure model for attention filtering. Rust-native, compatible.
2. **TencentDB-Agent-Memory** — Team memory hub. 4-asset model (conversations/docs/code/context) maps directly to NT-MEMORY's knowledge layers.
3. **loopx** — Durable loop state. SEAL pipeline needs quota-aware session management for long-running cycles.

**Monitor:**
- **screenpipe** — OS-level perception. Interesting but resource-heavy. Evaluate if NT-WORLD needs ambient capture.
- **clutch** — Provider-neutral routing. Validate against NT-IO's existing ordered backend router.
