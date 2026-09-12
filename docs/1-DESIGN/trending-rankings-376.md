# Trending Rankings — Cycle 376

**Date**: 2026-09-12
**Focus**: AI agents, LLM tools, reasoning frameworks, novel patterns for memory/attention/routing
**Sources**: GitHub Trending, ProductHunt, ossinsight.io, arXiv, awesome-llm-agents

---

## Top 10 New Projects (Not in Cycles 318-375)

### 1. SAGE — Self-Evolving Agents for Generalized Reasoning Evolution
- **URL**: https://arxiv.org/abs/2603.15255
- **Stars**: New (arXiv code release)
- **Language**: Python
- **What**: Closed-loop multi-agent framework where 4 specialized agents (Challenger, Planner, Solver, Critic) co-evolve from a shared LLM backbone using only a small seed set. No human-labeled data required.
- **Key Patterns**:
  - 4-agent closed-loop co-evolution (Challenger→Planner→Solver→Critic)
  - Quality filtering + format validation for stability
  - Dual-role Critic ensures both task quality and solution verification
  - Few-example seed → scalable self-play evolution
- **NeoTrix Mapping**: NT-MIND (self-evolution), NT-CORE (reasoning), NT-ACT (multi-agent orchestration)
- **Novel Pattern**: "Co-evolving agent quartet" — four agents with distinct roles (questioner, planner, solver, judge) evolve together. The Critic dual-roles (task quality + solution verification) prevent mode collapse. Minimal seed data enables infinite self-play.

### 2. MPAC — Multi-Principal Agent Coordination Protocol
- **URL**: https://arxiv.org/abs/2604.09744
- **Stars**: New (open-source, 223 tests)
- **Language**: Python + TypeScript
- **What**: Application-layer protocol for when independent principals' agents must coordinate over shared state. Fills the gap between MCP (tool invocation) and A2A (single-principal delegation).
- **Key Patterns**:
  - 5-layer coordination: Session, Intent, Operation, Conflict, Governance
  - Intent declaration as precondition for action
  - Conflicts as first-class structured objects
  - Lamport-clock causal watermarking
  - Pluggable governance layer (human-in-the-loop)
  - 95% coordination overhead reduction, 4.8× speedup
- **NeoTrix Mapping**: NT-ACT (multi-agent coordination), NT-GOVERNANCE (conflict resolution), NT-SHIELD (security profiles)
- **Novel Pattern**: "Intent-first coordination" — agents must declare intent before acting. Conflicts are first-class, not side effects. The governance layer enables policy-as-code for agent collectives. Extends beyond single-principal A2A to cross-organization agent collaboration.

### 3. Orchard — Open-Source Agentic Modeling Framework (Microsoft)
- **URL**: https://www.microsoft.com/en-us/research/publication/orchard-an-open-source-agentic-modeling-framework
- **Stars**: New (Microsoft Research, May 2026)
- **Language**: Python
- **What**: End-to-end framework for building agentic models. Distills 107K trajectories from frontier models, introduces credit-assignment SFT for learning from productive segments of unresolved trajectories. Orchard-SWE achieves 67.5% on SWE-bench Verified (SOTA among open-source).
- **Key Patterns**:
  - Trajectory distillation from frontier models (MiniMax-M2.5, Qwen3.5-397B)
  - Credit-assignment SFT: learns from productive segments of failed trajectories
  - Balanced Adaptive Rollout for RL
  - 4 success metrics: 74.1% (GUI), 67.0% (SWE), 64.0% (Claw)
  - Trained with only 0.2-0.4K synthetic tasks
- **NeoTrix Mapping**: NT-MIND (trajectory learning), NT-ACT (agentic execution), NT-CORE (reasoning)
- **Novel Pattern**: "Credit-assignment SFT" — not all trajectory segments are equal. Learning from productive segments of FAILED trajectories (where things went right before going wrong) is more efficient than learning from full successes. Minimal synthetic data (0.2K tasks) enables SOTA results.

### 4. screenpipe — AI Screen Recording for Agent Memory
- **URL**: https://github.com/mediar-ai/screenpipe (YC S26)
- **Stars**: ProductHunt featured (5.0 rating, Sep 2026)
- **Language**: Rust + TypeScript
- **What**: AI that records your computer work to power agents. Captures screen + audio, creates structured memory from user behavior. Powers downstream agents with persistent visual/conversational context.
- **Key Patterns**:
  - Continuous screen + audio capture → structured memory
  - Visual + conversational context fusion
  - Powers downstream agents with behavioral history
  - YC-backed, production-grade
  - Privacy-first local processing
- **NeoTrix Mapping**: NT-WORLD (perception), NT-MEMORY (behavioral memory), NT-PHYSICAL (screen/audio sensing)
- **Novel Pattern**: "Behavioral memory substrate" — agents don't need to ask what the user did; the screen recording IS the memory. Structured extraction from raw visual/audio creates a behavioral knowledge base that agents query. Shifts from "user tells agent" to "agent observes user."

### 5. Mozaik — TypeScript Runtime for Interoperable AI Agents
- **URL**: https://github.com/jigjoy-ai/mozaik
- **Stars**: New (Sep 2026)
- **Language**: TypeScript
- **What**: TypeScript runtime for interoperable AI agents. Provides typed agent primitives, cross-agent communication, and a sandbox for testing concurrent AI coding agents.
- **Key Patterns**:
  - TypeScript-native agent primitives (typed, not stringly-typed)
  - Cross-agent interoperability protocol
  - Concurrent agent sandbox for testing
  - Runtime-level isolation between agents
  - CLI tool (Baro) for goal→PR automation
- **NeoTrix Mapping**: NT-IO (agent runtime), NT-ACT (concurrent execution), NT-SHIELD (agent isolation)
- **Novel Pattern**: "Typed agent runtime" — agents as first-class typed primitives in TypeScript. Not Python string passing, but compile-time checked agent interfaces. The runtime enforces interoperability contracts at the type level.

### 6. Flare — Graph-First IDE for Agentic Coding
- **URL**: ProductHunt (launched Sep 2026, 120 upvotes)
- **Stars**: New
- **Language**: Unknown (open-source)
- **What**: Graph-first IDE and interactive map for agentic coding. Visualizes code relationships as a navigable graph, enabling agents to reason about codebase structure rather than flat file lists.
- **Key Patterns**:
  - Code-as-graph (not file tree)
  - Interactive visualization of dependencies
  - Agent navigation via graph traversal
  - Open-source
  - Complements coding agents (Cursor, Claude Code)
- **NeoTrix Mapping**: NT-CORE (graph reasoning), NT-WORLD (code perception), NT-MEMORY (codebase knowledge)
- **Novel Pattern**: "Code as navigable graph" — agents navigate codebases via dependency graphs, not grep. The IDE visualizes the graph, enabling both human and agent to reason about code structure at the architectural level.

### 7. Orra — Plan Engine for Dynamic Agent Workflows
- **URL**: https://github.com/orra-dev/orra
- **Stars**: Featured on ossinsight trending
- **Language**: TypeScript
- **What**: A plan engine for dynamic planning and reliable execution of AI agent workflows. Handles the gap between static workflow definitions and dynamic agent behavior.
- **Key Patterns**:
  - Dynamic plan generation (not static DAGs)
  - Reliable execution with retry/recovery
  - Plan revision during execution
  - Observability built-in
  - Agent-agnostic (works with any LLM)
- **NeoTrix Mapping**: NT-ACT (workflow execution), NT-MIND (plan adaptation), NT-IO (observability)
- **Novel Pattern**: "Plans that revise themselves" — not static workflow graphs but dynamic plans that adapt during execution. The engine handles the gap between "what I planned" and "what actually happened" with automatic replanning.

### 8. ARISE-RL — Agentic Rubric-Grounded Iterative Self-Evolution
- **URL**: https://arxiv.org/abs/2609.01058
- **Stars**: New (Sep 2026)
- **Language**: Python
- **What**: Full-cycle self-evolution framework coupling task/rubric Generator and reasoning Solver through rubric-mediated co-evolution. Addresses the fundamental challenge of training agents on open-ended tasks without gold-standard answers.
- **Key Patterns**:
  - Rubric-mediated co-evolution (Generator ↔ Solver)
  - Scalable rubrics replace gold-standard answers
  - Near capability boundary: handles brittle/noisy rewards
  - Full-cycle: task generation → rubric creation → solving → evaluation → refinement
- **NeoTrix Mapping**: NT-MIND (self-evolution), NT-CORE (reasoning), NT-GOVERNANCE (quality rubrics)
- **Novel Pattern**: "Rubric-as-verification" — instead of gold answers, use rubrics (criteria lists) as verification signals. The Generator learns to create tasks with evaluable rubrics; the Solver learns to satisfy rubric criteria. Both evolve together.

### 9. WorldEvolver — Self-Evolving World Models for Agent Planning
- **URL**: https://arxiv.org/abs/2606.30639
- **Stars**: New (Jun 2026)
- **Language**: Python
- **What**: Self-evolving world model framework that revises its deployment-time context while keeping the downstream agent frozen. Three modules: Episodic Memory, Semantic Memory, Selective Foresight.
- **Key Patterns**:
  - Test-time memory revision (no retraining)
  - Episodic memory: retrieval-based simulation from real transitions
  - Semantic memory: persistent heuristic rules from prediction-observation mismatches
  - Selective foresight: filter low-confidence predictions
  - Agent parameters stay frozen
- **NeoTrix Mapping**: NT-MEMORY (episodic/semantic memory), NT-CORE (world model), NT-MIND (self-evolution)
- **Novel Pattern**: "Frozen agent, evolving world model" — the agent's policy doesn't change, but its understanding of the world does. Episodic memory replays real transitions; semantic memory extracts rules from failures. The world model gets smarter without retraining the agent.

### 10. Vectorize — Memory + Learning Platform for Agents
- **URL**: ProductHunt (Sep 2026, 5.0 rating)
- **Stars**: New
- **Language**: Unknown
- **What**: "Agents that remember. Agents that learn." Platform providing persistent memory and learning capabilities for AI agents. Focus on cross-session continuity and accumulated knowledge.
- **Key Patterns**:
  - Memory as infrastructure (not application feature)
  - Learning from accumulated interactions
  - Cross-agent knowledge sharing
  - Production-grade persistence
  - Developer-facing API (not end-user)
- **NeoTrix Mapping**: NT-MEMORY (persistent memory), NT-MIND (learning), NT-IO (memory API)
- **Novel Pattern**: "Memory as infrastructure service" — memory isn't built into each agent; it's a shared service that agents connect to. Multiple agents share learned knowledge. Memory is infrastructure like a database, not application logic like session state.

---

## Emerging Meta-Patterns (Cycle 376)

### Pattern 1: Self-Evolution Without Human Data
SAGE, ARISE-RL, and WorldEvolver all demonstrate that agents can evolve without human-labeled data. SAGE uses 4-agent self-play; ARISE-RL uses rubric-mediated verification; WorldEvolver revises world models at test time. The common thread: **verification signals replace supervision**.

### Pattern 2: Intent-First Coordination
MPAC's intent declaration as precondition for action is a shift from "act then resolve conflicts" to "declare then coordinate." This is protocol-level, not application-level. Conflicts become first-class structured objects, not error logs.

### Pattern 3: Plans That Revise Themselves
Orra and WorldEvolver both handle the gap between planned and actual execution. Static DAGs break when agents encounter unexpected states. Dynamic planning with revision loops enables robust long-horizon execution.

### Pattern 4: Memory as Infrastructure
Vectorize and screenpipe both treat memory as infrastructure service, not application feature. Memory is a shared substrate that multiple agents connect to, not a per-agent module. This enables cross-agent knowledge sharing.

### Pattern 5: Typed Agent Runtimes
Mozaik's TypeScript-native agent primitives represent a shift from stringly-typed (Python) to compile-time checked agent interfaces. This catches interoperability bugs at build time, not runtime.

---

## NeoTrix Domain Impact Summary

| Domain | New Patterns | Priority |
|--------|-------------|----------|
| NT-MIND | Self-evolution (SAGE/ARISE-RL/WorldEvolver), Credit-Assignment SFT (Orchard) | P0 |
| NT-MEMORY | Memory-as-Infrastructure, Behavioral Memory (screenpipe), Episodic/Semantic (WorldEvolver) | P0 |
| NT-ACT | Intent-First Coordination (MPAC), Dynamic Planning (Orra), Typed Agent Runtime (Mozaik) | P1 |
| NT-CORE | Code-as-Graph (Flare), World Model Evolution | P1 |
| NT-GOVERNANCE | Rubric-as-Verification (ARISE-RL), Conflict-First Protocols (MPAC) | P1 |
| NT-WORLD | Behavioral Perception (screenpipe), Graph-Native Code Navigation (Flare) | P2 |
| NT-IO | Agent Memory API (Vectorize), Observability (Orra) | P2 |
| NT-SHIELD | Agent Isolation (Mozaik), Security Profiles (MPAC) | P2 |
