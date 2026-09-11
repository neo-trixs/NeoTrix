# Trending Rankings — Cycle 332

**Date**: 2026-09-11
**Focus**: AI agents, LLM tools, reasoning frameworks, memory/routing patterns
**Previous cycles**: 318–331 (excluded from this list)

---

## Top 10 New Trending Projects

### 1. claw-compactor — 14-Stage Fusion Pipeline for LLM Token Compression
- **URL**: https://github.com/open-compress/claw-compactor
- **Stars**: 2,033 | **Language**: Python | **License**: MIT
- **What**: Deterministic token compression engine with 14 specialized stages chained through immutable data flow. Each stage is content-type-aware (AST-aware code via tree-sitter, JSON schema sampling, simhash deduplication, log/diff folding). Zero LLM inference cost. 15–82% compression. ROUGE-L 0.653 at rate=0.3 (vs LLMLingua-2 0.346 = +88.2%). Reversible: hash-addressed LRU store lets LLM retrieve originals by marker ID.
- **Key Pattern**: **Content-type-aware deterministic compression** — not perplexity-based token dropping but stage-specific compressors that understand code ASTs, JSON schemas, and log structure. Zero inference cost + reversible = production-safe.
- **NeoTrix Relevance**: Maps to NT-MEMORY (experience compression) and NT-CORE GWT (context as scarce resource — Axiom A2). The 14-stage pipeline validates our SEAL pipeline multi-stage distillation. Content-aware routing could optimize how experience-tree compresses different content types (code vs narrative vs metrics).

### 2. Nex — Claude Cowork for High-Volume GTM Workflows (YC S26)
- **URL**: https://nex.ai | https://github.com/nex-crm
- **Stars**: YC Summer 2026 | **Language**: Multi-language
- **What**: Workflow execution and context layer for AI-native business operations. Builds reviewable AI GTM agents from natural language goals across 1000+ integrations. Knowledge graph per workspace as ground truth. Agent writes code for predictable parts (deterministic, fast, cheap) and reserves AI judgment for hard parts. Backed by HubSpot founder Dharmesh Shah. $49/mo Pro tier with $100 AI usage included.
- **Key Pattern**: **Deterministic + AI hybrid execution** — agent writes code for repetitive parts, uses LLM only for genuinely hard reasoning. Knowledge graph as ground truth, not ad-hoc memory. "Reviewable" = human-in-the-loop before execution.
- **NeoTrix Relevance**: Maps to NT-ACT orchestration and NT-MEMORY knowledge graph. The deterministic+AI hybrid pattern validates our Dark Forest axiom (every module must have clear I→T→O). Knowledge graph as ground truth parallels our KB namespace architecture. Reviewable agents align with NT-GOVERNANCE audit trails.

### 3. Dagu — Single-Binary Workflow Orchestrator with MCP
- **URL**: https://github.com/dagucloud/dagu
- **Stars**: Trending Sep 2026 | **Language**: Go
- **What**: Local-first workflow engine — single binary, no database, declarative YAML. Built-in MCP server for AI agents to author and run workflows. External CLI harness runs Claude Code, Codex, Gemini CLI as workflow steps. Scheduling, retries, approvals, audit history. Airflow alternative without the platform overhead.
- **Key Pattern**: **Workflow-as-configuration, not code** — YAML over scripts/SSH/containers. MCP-native: agents can read state, preview changes, start/retry/stop runs. Single binary = zero infrastructure.
- **NeoTrix Relevance**: Maps to NT-ACT production orchestration. Dagu's MCP-native approach validates our tool integration philosophy. Workflow-as-configuration aligns with our SEAL pipeline stage definitions. The "single binary" simplicity is aspirational for NeoTrix deployment.

### 4. agenticSeek — Fully Local Manus AI Alternative
- **URL**: https://github.com/Fosowl/agenticSeek
- **Stars**: 26,897 | **Forks**: 3,016 | **Language**: Python | **License**: GPL-3.0
- **What**: 100% local AI agent — no APIs, no cloud. Voice-enabled, autonomous web browsing, code generation, task planning. Runs on consumer hardware with Ollama/LM Studio. Smart agent selection: user asks, system routes to best specialist agent automatically. Multi-agent task decomposition for complex projects.
- **Key Pattern**: **Privacy-by-architecture** — not privacy as feature but as architectural constraint. Local-first with optional cloud fallback. Smart agent routing without user awareness of the routing layer.
- **NeoTrix Relevance**: Maps to NT-SHIELD (privacy) and NT-ACT agent routing. The "smart agent selection" pattern is directly relevant to GWT salience routing — lightweight routing that picks the right specialist without user overhead. Privacy-by-architecture validates our Egress Privacy Guard trust tiers.

### 5. Sheaf-ADMM — Multi-Agent Coordination via Cellular Sheaves
- **URL**: https://arxiv.org/abs/2605.31005 | https://github.com/SakanaAI/sheaf
- **Published**: ICML 2026 | **Authors**: Seely, Cupiał, Jones (Sakana AI)
- **What**: Differentiable optimization framework for multi-agent coordination. Input decomposed into overlapping local views, each processed by an agent solving a convex subproblem. Agents coordinate through ADMM with inter-agent constraints specified by a cellular sheaf — the sheaf defines which aspects of neighboring solutions must agree while leaving the rest private. 92.6% solve rate on multi-agent Sudoku vs 34.7% for MPNN baseline. Exposed primal/consensus/dual variables enable direct analysis of coordination dynamics.
- **Key Pattern**: **Sheaf-constrained consensus** — agents agree only on boundary interfaces, keep internals private. Exposed coordination variables (primal/consensus/dual) make dynamics analyzable and intervenable.
- **NeoTrix Relevance**: Maps to NT-CORE GWT attention routing and NT-ACT cross-domain coordination. Sheaf-constrained consensus is directly applicable to our 7-domain architecture — domains agree on interfaces (edges in KB) while keeping internal state private. Exposed variables align with ConsciousnessTree observability.

### 6. Token Sparse Attention — Interleaved Token Selection (ICML 2026)
- **URL**: https://arxiv.org/abs/2602.03216 | https://github.com/dongwonjo/Token-Sparse-Attention
- **Published**: ICML 2026 | **Authors**: Jo, Kang, Song, Kim
- **What**: Lightweight dynamic token-level sparsification for long-context LLM inference. Compresses per-head Q, K, V to reduced token set, then decompresses output back to full sequence — enabling token relevance to be reconsidered in subsequent layers. "Compress and then Decompress" design. Compatible with Flash Attention and existing sparse kernels. Up to 3.23x attention speedup at 128K context with <1% accuracy degradation.
- **Key Pattern**: **Reversible sparsity** — unlike permanent token eviction, interleaved selection preserves future selection space. Each head independently selects tokens per layer. Dynamic token coverage adjusts sparsity budget at inference time.
- **NeoTrix Relevance**: Maps to NT-CORE attention mechanisms. The reversible sparsity pattern is relevant to GWT — instead of permanently pruning attention, maintain full potential but compute only what's salient. Head-wise independence parallels our Dual Specialization (Weapon Set I/II). Composable with existing kernels validates our compositional architecture.

### 7. Flux Attention — Context-Aware Hybrid Attention for Efficient Inference
- **URL**: https://arxiv.org/abs/2604.07394
- **Published**: Apr 2026 | **Authors**: Qiu, Hong, Yang et al.
- **What**: Context-aware framework that dynamically routes each layer to Full Attention (FA) or Sparse Attention (SA) based on input context via a lightweight Layer Router. Layer-wise routing preserves high-fidelity information retrieval while ensuring contiguous memory access. Only 12 hours training on 8xA800 GPUs. Up to 2.8x prefill and 2.0x decode speedup. Addresses the key limitation of hybrid attention: static allocation ratios that can't adapt to variable task demands.
- **Key Pattern**: **Layer-level dynamic routing** — not static FA/SA ratio but input-dependent per-layer routing. Lightweight router (<1% overhead) selects FA or SA based on actual retrieval demands. Contiguous memory access for hardware efficiency.
- **NeoTrix Relevance**: Maps to NT-CORE GWT attention routing. Layer-level routing validates our attention routing philosophy — not one-size-fits-all but context-dependent allocation. The lightweight router pattern is directly applicable to cost-aware model routing (Axiom A1). Contiguous memory access informs our KV cache optimization.

### 8. MIRIX — Multi-Agent Personal Assistant with Visual Memory
- **URL**: https://github.com/Mirix-AI/MIRIX
- **Stars**: 3,400+ | **Language**: Python
- **What**: Multi-agent personal assistant that tracks on-screen activities and consolidates real-time visual data into structured memories. Captures screen context, identifies apps/websites/content, extracts structured information, builds knowledge base that adapts to digital experiences. Not just screen recording — active understanding and memory consolidation.
- **Key Pattern**: **Visual memory consolidation** — raw screen data → structured knowledge base. Active understanding (not passive recording) with adaptive memory that evolves with usage patterns.
- **NeoTrix Relevance**: Maps to NT-WORLD perception and NT-MEMORY experience storage. MIRIX's visual memory consolidation parallels our experience-tree's snapshot→distill→classify→persist flow. Adaptive knowledge base validates our KB versioning approach.

### 9. TCA-Attention — Training-Free Context-Adaptive Attention
- **URL**: https://pubmed.ncbi.nlm.nih.gov/42616635
- **Published**: 2026
- **What**: Training-free sparse attention mechanism that selectively attends to only informative tokens. Two lightweight phases: (1) offline calibration determines head-specific sparsity budgets via single forward pass, (2) online token selection adaptively retains core context tokens using lightweight redundancy metric. Unified solution for both prefilling and decoding. No parameter updates or architectural changes required.
- **Key Pattern**: **Training-free calibration** — offline pass determines head-specific budgets, online pass selects tokens adaptively. Zero training cost + zero architectural change = drop-in acceleration.
- **NeoTrix Relevance**: Maps to NT-CORE inference optimization. Training-free approach aligns with R-P1 (minimal complexity). Head-specific budgets validate that different attention heads specialize for different content types — relevant to our domain-specialized agents.

### 10. Microsoft Agent Framework Go — Production Agent Orchestration
- **URL**: https://github.com/microsoft/agent-framework-go
- **Stars**: 585 | **Language**: Go | **License**: MIT
- **What**: Microsoft's open-source Go SDK for production agent orchestration. Graph-based workflows: sequential, concurrent, group collaboration, conditional routing, subworkflows, checkpointing, streaming, human-in-the-loop, time-travel patterns. OpenTelemetry for observability. MCP + A2A + AG-UI interoperability. Agent Skills as domain-specific knowledge bases. Production-grade: not a prototype but a deployment framework.
- **Key Pattern**: **Enterprise-grade agent orchestration** — not framework-as-library but framework-as-infrastructure. Time-travel debugging, checkpointing, streaming. Full observability via OpenTelemetry. Interoperability-first (MCP, A2A, AG-UI).
- **NeoTrix Relevance**: Maps to NT-ACT orchestration and NT-IO interface. The enterprise orchestration patterns (checkpointing, time-travel) validate our SEAL pipeline robustness requirements. OpenTelemetry observability aligns with ConsciousnessTree health monitoring. Interoperability standards (MCP, A2A) validate our UTCP/tool protocol strategy.

---

## Key Trends (Cycle 332)

1. **Deterministic + AI Hybrid Execution**: Nex, claw-compactor, and Dagu all demonstrate that the winning pattern is not "AI for everything" but "deterministic for predictable, AI for hard." Code generation for repetitive parts, LLM for genuine reasoning. This is the production maturity signal.

2. **Reversible Sparsity Everywhere**: Token Sparse Attention (interleaved selection), TCA-Attention (training-free calibration), and Flux Attention (layer-level routing) all solve the same problem: how to be sparse without losing information. The key insight: sparsity must be reversible, not permanent.

3. **Sheaf Theory for Agent Coordination**: Sheaf-ADMM brings algebraic topology to multi-agent coordination. The sheaf defines which aspects agents must agree on (boundaries) while keeping internals private. This is the mathematical foundation for cross-domain coordination that NeoTrix needs.

4. **MCP as Universal Interface**: Dagu, Microsoft Agent Framework, and UTCP all converge on MCP as the standard for agent-tool interaction. The ecosystem is standardizing faster than expected.

5. **Training-Free Everything**: TCA-Attention, Token Sparse Attention, and claw-compactor all achieve significant improvements with zero training cost. The production constraint is not model quality but deployment friction.

---

## Absorption Targets

| Priority | Project | Pattern | Domain | Action |
|----------|---------|---------|--------|--------|
| P0 | claw-compactor | Content-type-aware compression | NT-MEMORY | Investigate 14-stage pipeline for experience-tree distillation |
| P0 | Sheaf-ADMM | Sheaf-constrained consensus | NT-CORE | Study cellular sheaf for 7-domain interface contracts |
| P1 | Token Sparse Attention | Reversible interleaved sparsity | NT-CORE | Evaluate for GWT attention optimization |
| P1 | Flux Attention | Layer-level dynamic routing | NT-CORE | Map Layer Router to cost-aware model routing |
| P1 | Nex | Deterministic+AI hybrid | NT-ACT | Study knowledge graph as ground truth pattern |
| P2 | Dagu | MCP-native workflow orchestration | NT-ACT | Evaluate for SEAL pipeline orchestration |
| P2 | TCA-Attention | Training-free calibration | NT-CORE | Consider for drop-in inference acceleration |
| P2 | MIRIX | Visual memory consolidation | NT-WORLD | Study adaptive knowledge base pattern |
| P3 | agenticSeek | Privacy-by-architecture | NT-SHIELD | Validate local-first approach against Egress Guard |
| P3 | MS Agent Framework Go | Enterprise orchestration | NT-ACT | Review checkpointing/time-travel patterns |
