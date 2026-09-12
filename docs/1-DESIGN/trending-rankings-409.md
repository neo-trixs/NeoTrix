# Trending Rankings — Cycle 409 (2026-09-12)

## Search Scope
- GitHub Trending, ProductHunt, AI Weekly, arXiv
- Focus: AI agents, LLM tools, reasoning frameworks, memory/attention/routing patterns
- New projects only (not in cycles 318-408)

---

## 1. HydraFusion (GitHub Copilot)
- **URL**: https://github.blog/ai-and-mll/
- **Category**: AI Coding / Selective Inference
- **What**: GitHub's research preview for selective coding workflows. Matches or exceeds Opus 5 baseline while reducing estimated workflow cost. Runs multiple agents in parallel within the Copilot app.
- **Stars**: N/A (proprietary research preview)
- **NeoTrix Mapping**: NT-ACT (parallel task orchestration), NT-CORE (cost-aware model routing). Validates Axiom A1 (Cost-Aware Routing) — selective coding = routing subtasks to cheapest capable model.
- **Pattern**: HydraFusion's selective coding = conditional compute allocation per code region, maps to GWT salience-weighted attention routing.

## 2. MagiCrew
- **URL**: https://github.com/MagiCrew
- **Category**: Multi-Agent Orchestration Platform
- **What**: Open-source AI Agent platform for deploying specialized digital workers. Multi-agent collaboration, enterprise controls, deliverable-ready outputs. Agents research, analyze, create reports, generate presentations.
- **Stars**: 269 (ProductHunt launch)
- **NeoTrix Mapping**: NT-ACT (production orchestrator), NT-MIND (skill crystallization). Maps to ProductionOrchestrator pattern — specialized agents with role-based memory and reasoning.
- **Pattern**: Role-specialized agent teams with centralized management platform. Validates P2 (Isolation-per-Task).

## 3. Agora (Auction-Based Task Allocation)
- **URL**: https://arxiv.org/abs/2607.09600
- **Category**: Agent Coordination / Reasoning Framework
- **What**: Uses confidence-calibrated auction mechanism to dynamically allocate reasoning steps to expert models/tools. Treats reasoning steps as tradeable items. Key innovation: calibrated competence vs raw confidence for allocation.
- **Stars**: N/A (arXiv paper)
- **NeoTrix Mapping**: NT-CORE (GWT salience routing), NT-ACT (capability registry). Direct mapping to NT-ACT total_calls ascending rotation + GWT salience. Auction = market-based routing replacing fixed priority.
- **Pattern**: Confidence-calibrated auctions for dynamic task routing. P1 (Model Routing) + P4 (Ordered Backend Fallback) with economic mechanism.

## 4. Flux Attention
- **URL**: https://arxiv.org/abs/2604.07394
- **Category**: Efficient Attention / LLM Inference
- **What**: Context-aware hybrid attention framework. Layer Router adaptively routes each layer to Full Attention or Sparse Attention based on input context. Only 12 hours training on 8xA800 GPUs. 2.8x prefill speedup, 2.0x decode speedup.
- **Stars**: N/A (arXiv paper)
- **NeoTrix Mapping**: NT-CORE (GWT attention), NT-MEMORY (KV cache). Layer-wise dynamic routing = consciousness-level attention modulation. Validates KVMem paged KV approach for >256K sessions.
- **Pattern**: Layer-level routing between dense/sparse attention. Maps to GWT's salience-based broadcast gating.

## 5. GLIDE (Guided Layerwise Hybrid Attention)
- **URL**: https://arxiv.org/abs/2607.24788
- **Category**: Efficient Attention / KV Cache Compression
- **What**: Integrates sliding-window softmax with linear recurrent aggregation. Exploits layer-wise heterogeneity: early layers sensitive to softmax removal, deeper layers tolerate aggressive replacement. Non-uniform compression across model.
- **Stars**: N/A (arXiv paper)
- **NeoTrix Mapping**: NT-CORE (attention mechanism), NT-MEMORY (KV cache optimization). Layer-heterogeneous compression = differentiated attention per consciousness layer.
- **Pattern**: Asymmetric attention allocation across layers. Maps to NeoTrix 6-layer architecture where L1-L3 have different attention requirements than L4-L6.

## 6. screenpipe (YC S26)
- **URL**: https://github.com/mediar-ai/screenpipe
- **Category**: AI Memory / Screen Recording Agent
- **What**: Records computer work (screen + conversations) to power agents. Creates structured memories from raw inputs. YC S26 batch.
- **Stars**: Featured on ProductHunt
- **NeoTrix Mapping**: NT-MEMORY (experience recording), NT-WORLD (perception capture). Maps to experience-tree pattern — structured recording for cross-session recall.
- **Pattern**: Continuous screen+audio capture → structured memory → agent retrieval. Validates Context as Scarce Resource (Axiom A2).

## 7. Vectorize
- **URL**: https://vectorize.io
- **Category**: AI Infrastructure / Agent Memory
- **What**: "Agents that remember. Agents that learn." Vector database purpose-built for agent memory persistence. Learning from interactions.
- **Stars**: 5.0 rating (ProductHunt)
- **NeoTrix Mapping**: NT-MEMORY (KB embedding), NT-NEXUS (cross-session weaving). Validates KB pipeline for agent memory with learning loops.
- **Pattern**: Vector-native agent memory with learning. Maps to NT-MEMORY namespace hub pattern.

## 8. HarnessRouter Community Edition
- **URL**: https://github.com/strand-agents/harness-router
- **Category**: Agent Harness / Model Routing
- **What**: Open-source unified interface for agent harnesses. Multi-model, multi-cloud agent control plane.
- **Stars**: 365 (ProductHunt)
- **NeoTrix Mapping**: NT-ACT (orchestration), NT-IO (provider management). Direct mapping to Ordered Backend Router pattern.
- **Pattern**: Unified harness interface with model routing fallback. P4 (Ordered Backend Fallback).

## 9. SAR (Self-Distilled Agentic Reinforcement Learning)
- **URL**: https://github.com/ZJU-REAL/SDAR
- **Category**: Agent Training / Self-Evolution
- **What**: Self-distilled agentic RL for skill acquisition. Agents learn skills through self-play and distillation. 343 stars.
- **Stars**: 343
- **NeoTrix Mapping**: NT-MIND (SEAL pipeline), NT-CORE (self-evolution). Maps to SEAL pipeline's distillation stage — agents self-improve through experience.
- **Pattern**: Self-distillation for agent skill crystallization. Validates Constellation maturity ladder (C0-C6).

## 10. AdCo (Adaptive Coopetition)
- **URL**: https://aclanthology.org (ACL 2026)
- **Category**: Multi-Agent Reasoning / Coordination
- **What**: UCB-based coopetition mechanism for LLM agents. Agents choose to collaborate or compete based on coarse verifier signals. 20% relative improvement on challenging math benchmarks. No high-performance verifier required.
- **Stars**: N/A (ACL paper)
- **NeoTrix Mapping**: NT-CORE (GWT routing), NT-ACT (multi-agent coordination). Maps to GWT attention modulation — agents dynamically choose cooperation vs competition based on confidence.
- **Pattern**: Adaptive cooperation/competition switching. Validates A1 (Cost-Aware Routing) — cheap tasks compete, hard tasks cooperate.

---

## Synthesis: Top Patterns for NeoTrix

| Pattern | Source | NeoTrix Integration |
|---------|--------|---------------------|
| **Auction-Based Task Routing** | Agora | GWT salience + economic mechanism for provider selection |
| **Layer-Heterogeneous Attention** | GLIDE/Flux | Per-layer attention strategy in 6-layer architecture |
| **Confidence-Calibrated Allocation** | Agora | Replace raw confidence with calibrated competence in routing |
| **Selective Coding Workflows** | HydraFusion | Conditional compute allocation per code region |
| **Screen-to-Memory Pipeline** | screenpipe | Continuous capture → structured memory → agent recall |
| **UCB-Based Coopetition** | AdCo | Dynamic collaboration/competition switching in GWT |
| **Self-Distilled Agent Skills** | SAR | SEAL pipeline distillation stage automation |
| **Unified Harness Interface** | HarnessRouter | Ordered Backend Router with multi-model fallback |

## Cost-Aware Routing Validation (A1)

Agora's confidence-calibrated auction and AdCo's UCB-based coopetition both validate Axiom A1: routing tasks to cheapest capable model via economic/UCB mechanisms rather than fixed priority. This reinforces the need for NT-ACT to incorporate market-based provider selection alongside total_calls ascending rotation.
