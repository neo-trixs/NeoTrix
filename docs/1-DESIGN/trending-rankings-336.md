# Trending Rankings — Cycle 336 (2026-09-11)

## Summary

10 new AI/developer tool projects discovered across GitHub trending, ProductHunt, and research repos. Focus areas: agent security firewalls, MCP governance runtime, self-evolving agent frameworks, multi-agent topology optimization, LLM self-declared attention.

---

## 1. Mastra (mastra-ai/mastra)

- **Stars**: ~491 upvotes on ProductHunt (Sept 9, 2026)
- **Language**: TypeScript
- **URL**: https://github.com/mastra-ai/mastra
- **Description**: TypeScript AI framework for agents and workflows. From the Gatsby team. Graph-based workflow engine, memory, MCP server authoring, observability. npm create mastra@latest.
- **Key Pattern**: **Agent-as-a-Server** — agents, workflows, tools deployed as unified server endpoints. Graph-based workflow engine with `.then()`, `.branch()`, `.parallel()` control flow. Human-in-the-loop suspensions with persistent state.
- **NeoTrix Mapping**: NT-IO + NT-ACT — their server-deployed agents map to our capability network's service endpoints. Graph workflows parallel our SEAL pipeline stage orchestration. MCP server authoring validates our tool abstraction.
- **Relevance**: HIGH — TypeScript-native agent framework with production deployment model. Their workflow engine is a potential reference for SEAL pipeline orchestration. Gatsby team pedigree ensures production hardening.

---

## 2. Harden AIF (hardenrun/aif)

- **Stars**: 405 upvotes on ProductHunt, #2 Product of the Day (Sept 9, 2026)
- **Language**: Rust + Python
- **URL**: https://github.com/hardenrun/aif
- **Description**: AI Firewall — local-first security layer for coding agents. Post-trained 8B model checks every tool call before execution. Beats frontier models on agent-security benchmarks. Works with Claude Code, Codex, Cursor, Gemini CLI.
- **Key Pattern**: **Pre-execution intent-aware firewall** — every tool call evaluated against developer intent + session context before running. Graduated responses: Allow/Block/Ask/Make-Safe/Record. On-device inference keeps code local.
- **NeoTrix Mapping**: NT-SHIELD — direct analog to our egress privacy guard, but for tool calls (not just outbound requests). Their "make safe" pattern (rewrite dangerous actions) extends our RiskAssessor's block/allow with active remediation. Session-context-aware evaluation validates our NT-SHIELD's audit dimension approach.
- **Relevance**: VERY HIGH — this is the missing security layer NeoTrix needs for NT-ACT tool execution. Their 329M-token training corpus on agent trajectories is a blueprint for our guard training. Pre-execution checking is strictly superior to post-execution auditing.

---

## 3. Noodle Seed (noodleseed.com)

- **Stars**: 254 upvotes on ProductHunt (Sept 9, 2026)
- **Language**: TypeScript
- **URL**: https://noodleseed.com/
- **Description**: MCP server governance runtime. Author one `server.ts`, get a multi-tenant governed endpoint. Thin language, fat runtime. Credential broker, policy engine, audit trail, OAuth DCR/PKCE.
- **Key Pattern**: **Governed MCP runtime** — separation of capability declaration (thin) from production infrastructure (fat runtime). Credential broker never forwards inbound tokens to backends. Policy enforcement before connector side effects.
- **NeoTrix Mapping**: NT-IO + NT-GOVERNANCE — their credential broker pattern is exactly our egress privacy guard's secret scrubbing. Policy-before-side-effects validates our NT-SHIELD's pre-execution checking philosophy. Multi-tenant runtime maps to our capability network's service isolation.
- **Relevance**: VERY HIGH — Noodle Seed solves the MCP governance gap. Their "thin language, fat runtime" principle is exactly our capability network philosophy. Credential broker + policy gate is the production-grade version of our egress guard.

---

## 4. OmniAgent (YeQing17-2026/OmniAgent)

- **Stars**: 2,557 ⭐
- **Language**: Python
- **URL**: https://github.com/YeQing17-2026/OmniAgent
- **Description**: Self-evolving agent framework with full-dimensional evolution (Skill, Context, BrainModel). Hyper-Harness for dynamic multi-agent + concurrent tool execution. Deep Reflexion dual-layer reflective architecture.
- **Key Pattern**: **Real-time skill evolution** — skills evolve during execution (not post-hoc). Progressive Context Loading (L0/L1/L2) inspired by Anthropic's progressive disclosure. Four-layer dynamic security scanning (LLM review → Policy → Interactive approval → Sandbox).
- **NeoTrix Mapping**: NT-MIND + NT-SHIELD — real-time skill evolution maps to our SEAL pipeline's in-flight adaptation. Progressive context loading is exactly our experience-tree's lazy branch loading. Their four-layer security validates our NT-SHIELD's layered defense model.
- **Relevance**: HIGH — their OmniEvolve's three-axis evolution (Skill/Context/BrainModel) is a more granular version of our SEAL pipeline's exploration→distillation→absorption cycle. The four-layer security model is a production blueprint.

---

## 5. Prime Agent (PrimeIntellect-ai/prime-agent)

- **Stars**: 1,456 ⭐
- **Language**: Python
- **URL**: https://github.com/PrimeIntellect-ai/prime-agent
- **Description**: Self-improving RLM (Recursive Language Model) agent. Prompt-as-a-variable, tools as recursive subagent function calls. Continual Harness stores durable state. `/refine` applies evidence-backed updates.
- **Key Pattern**: **Continual Harness** — supplemental prompts, memories, skill descriptions as durable state refined through small, evidence-backed updates. Never rewrites immutable base system prompt. Subagents communicate directly without routing through user.
- **NeoTrix Mapping**: NT-MIND + NT-NEXUS — their Continual Harness is our experience-tree's persistent hub + lazy branch loading. `/refine` is our SEAL pipeline's absorption step. Direct agent-to-agent communication validates our EventBus architecture.
- **Relevance**: HIGH — Prime Agent's "prompt-as-a-variable" + Continual Harness pattern is the inference-level analog of our experience-tree. Their evidence-backed refinement (never rewrites base prompt) validates our AGENTS.md pointer conservation rule.

---

## 6. ReActNet (arXiv:2609.05774)

- **Stars**: Research paper (arXiv, Sept 4, 2026)
- **Language**: N/A
- **URL**: https://arxiv.org/abs/2609.05774
- **Description**: Inference-Time Graph Engineering for Multi-Agent LLM Workflows. Compiles query + agents into temporal communication graphs. Training-free — no RL or gradient-based topology optimization needed.
- **Key Pattern**: **Temporal graph compilation** — separate graph compilation from graph execution. Each graph snapshot = one reasoning stage; each edge = natural-language instruction for agent-to-agent messages. Task-conditioned topology.
- **NeoTrix Mapping**: NT-CORE + NT-ACT — temporal graph compilation maps to our GWT's attention routing with stage-aware broadcast. Their "separate compilation from execution" is our SEAL pipeline's compile→execute separation. Edge-as-instruction is our EventBus message semantics.
- **Relevance**: VERY HIGH — ReActNet's temporal graph is the formalized version of our GWT attention routing. Their key insight ("effective orchestration depends not only on which agents communicate, but on engineering executable workflow graphs that encode when, why, and how information should flow") is exactly our GWT design philosophy.

---

## 7. Codebook Agent (arXiv:2609.02264)

- **Stars**: Research paper (arXiv, Sept 2, 2026)
- **Language**: N/A
- **URL**: https://arxiv.org/abs/2609.02264
- **Description**: Amortized topology design for multi-agent systems. Vector-quantized autoencoder compresses successful topologies into 16-entry codebook. Topology generated in 2.4ms, 21-33% fewer LLM tokens.
- **Key Pattern**: **Topology codebook** — successful agent communication patterns compressed into query-independent codebook. Reward-weighted MLP maps query → topology code. No iterative search at test time.
- **NeoTrix Mapping**: NT-CORE — topology codebook is a compressed version of our GWT routing patterns. Their finding that topologies collapse to ~6 distinct graphs even with 64-entry capacity validates our Constellation maturity model (limited stable patterns). Negative correlation between edge count and token cost validates sparse routing.
- **Relevance**: HIGH — Codebook Agent proves that multi-agent topologies have low effective dimensionality. This validates our skill tree's 3-tier model (Small Passive → Notable Passive → Keystone) as the right granularity for routing patterns.

---

## 8. CondenseFlow (ACL 2026 Findings)

- **Stars**: Research paper (aclanthology.org, 2026)
- **Language**: N/A
- **URL**: https://aclanthology.org/2026.findings-acl.669.pdf
- **Description**: Scalable latent space collaboration via semantic compression. Latent Thought Condenser (LTC) compresses KV caches into fixed-size representations — O(1) communication complexity regardless of context length. 99%+ memory reduction.
- **Key Pattern**: **Fixed-size semantic compression** — learnable semantic probes compress variable-length KV caches into fixed-size vectors. Cross-attention aggregation preserves information patterns most valuable for downstream reasoning. Error bounded by attention concentration.
- **NeoTrix Mapping**: NT-MEMORY + NT-CORE — LTC's fixed-size compression is our KB embedding compression for cross-session experience transfer. Their "semantic probes discover valuable information patterns through end-to-end learning" maps to our experience-tree's route table matching. O(1) complexity validates our lazy branch loading.
- **Relevance**: VERY HIGH — CondenseFlow solves the multi-agent memory scaling problem. Their 99% KV reduction with <2% accuracy loss is the production-grade version of our experience-tree's hub index + lazy loading. Theoretical error bounds provide formal guarantees our system lacks.

---

## 9. CEDAR (arXiv:2609.07237)

- **Stars**: Research paper (arXiv, Sept 7, 2026)
- **Language**: N/A
- **URL**: https://arxiv.org/abs/2609.07237
- **Description**: Error-Bounded Residual Routing for Efficient Long-Context Attention. Coarse-to-fine method: cheap KV summaries + residual attention path for high-error chunks. 3× speedup at 128K context.
- **Key Pattern**: **Residual attention routing** — cheap summaries for easy chunks, exact attention only for high-error chunks. Single softmax normalization (not additive). Variable refinement budget based on within-chunk key/value dispersion.
- **NeoTrix Mapping**: NT-CORE — CEDAR's coarse-to-fine attention maps to our GWT's salience-based attention allocation. Their "refinement replaces rather than duplicates coarse evidence" validates our single-fact-source KB design. Variable budget per chunk = our cost-aware routing (A1) at the attention level.
- **Relevance**: HIGH — CEDAR is the attention-level analog of our GWT routing. Their error-bounded approach provides formal guarantees for when to escalate attention (cheap → exact). This is the micro-level version of Switchyard's escalation routing (cycle 335).

---

## 10. Declarative Attention (arXiv:2609.02737)

- **Stars**: Research paper (arXiv, Sept 2, 2026)
- **Language**: N/A
- **URL**: https://arxiv.org/abs/2609.02737
- **Description**: Language models can control their own attention. DA protocol elicits model to declare attention regions in chain-of-thought: `<global>`, `<focus>`, `<local>`. Off-the-shelf models (Gemma-4-31B, Qwen-3.6-27B) reduce attended tokens by 31-52%.
- **Key Pattern**: **Intrinsic attention declaration** — model declares where it needs to attend (not external proxy scoring). Partitioning into global/focus/local modes. Works on off-the-shelf models without training.
- **NeoTrix Mapping**: NT-CORE — DA's attention declaration is the model-level analog of our GWT's salience broadcasting. Their three modes (global/focus/local) map to our GWT's broadcast/specialist/focused routing. Zero-shot applicability validates our runtime adaptation philosophy.
- **Relevance**: VERY HIGH — Declarative Attention proves that models already know which context matters. This is the strongest validation of our GWT design philosophy: attention routing should be intrinsic, not bolted on. The three modes are a natural fit for our GWT attention levels.

---

## Cross-Cutting Patterns

| Pattern | Projects | NeoTrix Integration |
|---------|----------|---------------------|
| **Pre-execution Security** | Harden AIF, OmniAgent | NT-SHIELD tool call firewall, graduated responses |
| **Temporal Agent Graphs** | ReActNet, Codebook Agent | GWT attention routing, SEAL pipeline stages |
| **Fixed-Size Semantic Compression** | CondenseFlow, CEDAR | KB embedding compression, experience-tree lazy loading |
| **Declarative Attention** | Declarative Attention, CEDAR | GWT salience broadcasting, cost-aware routing |
| **Governed MCP Runtime** | Noodle Seed | Credential broker, policy-before-side-effects |
| **Continual Harness** | Prime Agent, OmniAgent | Experience-tree persistent hub, evidence-backed refinement |
| **TypeScript Agent Frameworks** | Mastra | Agent-as-a-server deployment model |

## Velocity Indicators

| Metric | Value |
|--------|-------|
| Projects tracked | 10 (5 repos + 5 papers) |
| Fastest growing | Mastra (#1 ProductHunt, $35M raised) |
| Most impactful | Harden AIF (agent security gap-filler) |
| Language split | Python 2, TypeScript 1, Research 5, Rust+Python 1 |
| Dominant theme | Agent security + attention optimization |

## Priority Absorptions

1. **Harden AIF pre-execution firewall** — NT-SHIELD tool call checking (P0, immediate security gap)
2. **CondenseFlow fixed-size compression** — KB experience compression for cross-session transfer (P0)
3. **Declarative Attention intrinsic routing** — GWT attention mode design (P1)
4. **ReActNet temporal graph compilation** — GWT broadcast + stage-aware routing (P1)
5. **Noodle Seed governed MCP runtime** — credential broker + policy gate (P1)
