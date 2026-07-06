# Deep Research: FARS · Agents-A1 · Hermes-Agent · ORCA Cross-Reference

Date: 2026-07-02

## Source Matrix

| Source | Type | Key Alias | Core Thesis |
|--------|------|-----------|-------------|
| **FARS** (2606.31651) | arXiv paper | Fully Automated Research System | 4-stage closed-loop research pipeline (Ideation → Planning → Experiment → Writing) with stage-specific agents, shared workspace, and checkpointed fault tolerance |
| **Agents-A1** (2606.30616) | arXiv paper | Scaling the Horizon | 35B MoE agent matches trillion-param performance via Knowledge-Action Graph (KAG) process supervision + multi-teacher distillation + domain-routed training |
| **Hermes-Agent** | GitHub (207k★) | NousResearch/hermes-agent | Self-improving agent: soul-skill creation during use, FTS5 session search, Honcho dialectic user modeling, subagent spawning, batch trajectory generation |
| **ORCA** | CHI 2026, CVPR 2026 | Orchestrating Causal Agent | Causal mental models for agents: react-plan 3-step strategy, theory-of-mind for multi-agent coordination, human-in-the-loop interfaces |

## Cross-Reference: Found Architecture Patterns

### 1. Trajectory as First-Class Citizen

All four sources independently converged on **structured trajectory records** as the critical missing primitive in agent architectures:

| Source | Trajectory Primitive | Purpose |
|--------|---------------------|---------|
| Hermes-Agent | soul-skill | Self-created skills from trajectory patterns → procedural memory |
| Hermes-Agent | batch_trajectories() | Generate training data for next-gen models |
| FARS | shared-workspace | Persistent project memory across stages |
| Agents-A1 | KAG nodes | Process-level supervision via evidence→action→observation→verifier |
| NeoTrix | E8 state_trajectory | Only raw hexagram sequence, no step-level structure |

**NeoTrix Gap**: `state_trajectory: Vec<FullReasoningState>` is a flat hexagram sequence. No `AgentStep` abstraction with input/output/specialist/reward. No compression. No KB absorption.

**Implementation**: `core/nt_core_trajectory_compress.rs` — `AgentTrajectory` type + 3-level `TrajectoryCompressor` (Light/Medium/Aggressive) + `TrajectoryCompressionReport`. Integrated into `ReasoningEngine.reason()`.

### 2. Process-Level Supervision (KAG)

Agents-A1 introduces the **Knowledge-Action Graph** as the core oversight mechanism:

```
Evidence → Action → Observation → Verifier
```

Each `reason()` call should produce a structured KAG chain:
- **Evidence**: retrieved knowledge, conversation context
- **Action**: LLM call, tool call, subagent dispatch
- **Observation**: response, tool output, error
- **Verifier**: confidence check, reward assignment, PRM score

**NeoTrix Gap**: `reason()` returns `NeoTrixResult<String>` — opaque string, no KAG chain, no verifier gate.

**Implementation**: `engine_core.reason_multi_agent()` — delegates to L7 `Orchestrator` pattern (Supervisor/Swarm/Pipeline), recording agent outputs with confidence/verification.

### 3. Subagent Delegation & Parallel Execution

| Source | Mechanism | Key Feature |
|--------|-----------|-------------|
| Hermes-Agent | `agent spawn` | Spawn isolated subprocess with communication |
| ORCA | react-plan delegation | Decompose task → assign subagents → monitor |
| FARS | stage-specific agents | Each pipeline stage has dedicated agent |

**NeoTrix Gap**: `SubagentExecutor::execute()` was synchronous single-task; no `execute_parallel()`.

**Implementation**: `executor.rs` — added `execute_parallel()` using `futures::future::join_all`, plus `Arc<Mutex<ReasoningEngine>>` for shared engine access.

### 4. Multi-Teacher Distillation

Agents-A1's key training insight: no single model excels at all domains. Domain-specific teachers → multi-teacher distillation with **Salient Vocabulary Alignment** (SVA).

**NeoTrix Relevance**: `ConversationDistillStage` already writes `EvolutionRecord` patterns (RecurringError, CommunicationOptimization, etc.) — but these patterns are not domain-routed and not distilled into a unified student model.

**Not Yet Implemented**: `nt_core_multi_teacher_distill` would:
1. Maintain per-domain teacher experts (Code, Reasoning, Memory, Safety, ...)
2. Collect SVA vectors for salient token identification
3. Align KL-divergence losses from multiple teachers
4. Output domain-routed student logits

### 5. Causal Mental Models (ORCA)

ORCA's core contribution: agents maintain **online causal models** of other agents' behavior (theory of mind).

**NeoTrix Gap**: No module models agent interaction causality. Engine has no concept of "agent A's action caused agent B's response".

**Not Yet Implemented**: `nt_core_mental_model` would:
1. Track agent→action→effect triples
2. Build causal graph over agent interactions
3. Predict next action from inferred mental state
4. Detect coordination failures from causal chain breaks

## Compilation Status

- `cargo check --lib -p neotrix`: **0 errors** ✅
- `cargo test --no-run -p neotrix --lib`: **0 errors** ✅
- `cargo test -p neotrix --lib -- nt_core_trajectory_compress`: **6/6 passed** ✅
- `cargo test -p neotrix --lib -- nt_act_orch_patterns`: **18/18 passed** ✅
- `cargo test -p neotrix --lib -- subagent`: **19/19 passed** ✅

## Remaining P0 Blind Spots (Future Work)

| Blind Spot | Papers | Module | Est. Effort |
|-----------|--------|--------|------------|
| Multi-teacher distillation pipeline | Agents-A1 | `nt_core_multi_teacher_distill` | 3-5 days |
| Causal mental model (theory of mind) | ORCA | `nt_core_mental_model` | 2-3 days |
| Subprocess isolation for subagents | Hermes-Agent | `nt_agent_subagent::remote` | 2-3 days |
| Soul-skill procedural memory | Hermes-Agent | `nt_memory_procedural` absorption | 1-2 days |
| E2E research pipeline (FARS-style) | FARS | L8 pipeline stage | 2-3 days |
