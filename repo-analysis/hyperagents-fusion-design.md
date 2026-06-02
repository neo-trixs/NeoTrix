# HyperAgents — NeoTrix Fusion Design

> **Created**: 2026-05-28
> **Status**: Design doc — awaiting implementation
> **Source**: Meta FAIR — arXiv:2603.19461 — Jenny Zhang et al.
> **Repository**: https://github.com/facebookresearch/HyperAgents

## 1. Project Overview

**HyperAgents** is a self-referential self-improving agent framework from Meta FAIR that enables agents to optimize for any computable task. The core innovation is the elimination of the fixed meta-level bottleneck: unlike prior systems (including NeoTrix's SEAL loop), HyperAgents makes the **meta agent itself editable** — the mechanism that generates improvements is itself subject to improvement, enabling metacognitive self-modification.

**Key technologies:**
- **Two-agent architecture**: TaskAgent (solves tasks) + MetaAgent (modifies agents and itself) in a single editable program
- **Self-referential loop**: MetaAgent can rewrite `meta_agent.py` — the improvement mechanism improves itself
- **Archive-based evolution**: Quality-diversity population of agent variants with configurable parent selection (score_child_prop, score_prop, best, latest, random)
- **Docker sandboxing**: Each generated agent variant is evaluated in isolated Docker containers — code changes are applied as `.diff` patches, evaluated, and results stored
- **Cross-domain transfer**: Self-improvement strategies learned in one domain (paper review, robotics) transfer to unseen domains (math grading)
- **Compounding improvement**: Meta-level gains from one run carry over and accelerate subsequent runs

**How it differs from AgentEvolver / NeoTrix SEAL:**
- NeoTrix's `SelfEvolver` is **externally triggered** (user sends URL → system analyzes and absorbs)
- HyperAgents is **internally self-referential** (system proposes code diffs to its own codebase, including the meta-agent itself)
- NeoTrix's SEAL loop updates **CapabilityVector dimensions** (numerical vector adjustments)
- HyperAgents generates **code diffs** (structural code changes that add new capabilities)
- HyperAgents maintains an **evolutionary population archive**; NeoTrix has a single ReasoningBrain state
- HyperAgents' MetaAgent can modify **any file** in the repo, including its own implementation; NeoTrix's SelfIteratingBrain only updates its internal state vectors

## 2. Gap Matrix

| # | Feature | HyperAgents | NeoTrix | Priority | Notes |
|---|---------|-------------|---------|----------|-------|
| 1 | **Self-referential improvement loop** | MetaAgent edits `meta_agent.py` — the improvement mechanism improves itself | SEAL loop updates CapabilityVector numerically; improvement mechanism (SEAL loop code) is **fixed** | **P0** | Most impactful gap. Without this, NeoTrix's self-improvement is bounded by the initial SEAL pipeline design. |
| 2 | **Meta-agent architecture** | Explicit MetaAgent class that reads repo + eval results, writes diffs | ReasoningEngine is a fixed reasoning pipeline; no agent observes and edits other agents | **P0** | Requires new `MetaAgent` struct + `MetaAgentConfig`. Partial precursor exists in `subagent/` module. |
| 3 | **Code-diff-based self-modification** | MetaAgent generates `.diff` files via git; each generation applies patches to working tree | `SelfEvolver.fetch_information()` clones repos but generates MicroEdit sequences (vector adjustments), not code diffs | **P1** | NeoTrix could reuse `ChangeArchive` for delta tracking but needs diff generation + application. |
| 4 | **Population archive with parent selection** | Archive of all generated agents with score_child_prop/best/random parent selection | Single ReasoningBrain with snapshot/restore; no population, no lineage tracking, no parent selection | **P1** | NeoTrix has `ChangeArchive` but no evolutionary selection logic. |
| 5 | **Docker sandbox for safe evaluation** | Each agent variant evaluated in isolated Docker; `git reset --hard` after each gen | Sandbox module exists (`sandbox.rs`) but no integration with code-modification evaluation | **P2** | NeoTrix has `sandbox.rs` — needs Docker orchestration + evaluation harness. |
| 6 | **Metacognitive self-modification (meta-meta)** | MetaAgent improves its own improvement strategy (e.g., adds persistent memory, performance tracking to its own code) | No concept of meta-agent modifying its own improvement code | **P0** | Core differentiator. Enables cross-domain transfer and compounding improvement. |
| 7 | **Cross-domain strategy transfer** | Self-improvement strategies trained on paper review transfer to math grading; meta-level improvements accumulate across runs | Knowledge absorption is domain-specific (KnowledgeSource → CapabilityVector); no transfer of meta-strategies | **P2** | NeoTrix has `cognitive_map.rs` and `knowledge_chain` but no cross-domain meta-strategy mechanism. |
| 8 | **Staged evaluation protocol** | Small subset (10 tasks) → full eval (100 tasks) with score=0 for failed stage | Single-shot `run_seal_loop()` with single task; no staged evaluation | **P1** | Critical for cost control — prevents wasting LLM budget on clearly broken agents. |
| 9 | **Agent lineage tracking** | archive.jsonl records every agent: performance, parent, lineage, diff path | `EvaluationRecord` exists but no lineage/parent tracking | **P2** | Needed for parent selection and understanding which improvements compound. |
| 10 | **Multi-domain optimization** | Same run optimizes paper review + robotics simultaneously; each stage evaluates both domains | `GoalLoop` supports multiple goals but no unified multi-domain evaluation harness | **P2** | Less urgent; NeoTrix can first implement single-domain self-referential loop. |
| 11 | **TaskAgent/MetaAgent separation** | TaskAgent gets task inputs (no tools by default); MetaAgent gets repo + eval path (all tools) | `AgentTeam` supports 5 process types but no TaskAgent/MetaAgent role separation | **P1** | NeoTrix's `AgentTeam` + `subagent/` provides partial foundation. |
| 12 | **Safety: domain diff filtering** | `filter_patch_by_files()` strips domain code from diffs — prevents meta-agent from modifying evaluation harness | No equivalent guard against evaluator contamination | **P1** | Easy win — add diff filter to prevent meta-agent from gaming evaluation. |

### Summary

**P0 (blockers — must implement first):**
- Self-referential improvement loop (meta-agent edits own mechanism)
- Meta-agent architecture (new struct + trait + loop)
- Metacognitive self-modification (meta-agent modifies its own improvement strategy)

**P1 (critical enablers):**
- Code-diff-based self-modification (generate + apply `.diff` patches)
- Population archive with parent selection (evolutionary selection from agent variants)
- Staged evaluation protocol (subset → full, cost control)
- TaskAgent/MetaAgent role separation (role-specific tools + context)
- Safety: domain diff filtering (prevent evaluator gaming)

**P2 (enhancements):**
- Docker sandbox integration (for safe code execution evaluation)
- Cross-domain strategy transfer (meta-strategies migrate across domains)
- Agent lineage tracking (archive.jsonl with parent/child relationships)
- Multi-domain optimization harness (simultaneous evaluation across domains)

## 3. P0-1: Self-Referential Loop (spec for implementation)

### Design

The self-referential loop integrates into NeoTrix's existing SEAL loop architecture by adding a **MetaAgent** that operates at a meta-level above the ReasoningEngine. The MetaAgent receives: (1) the current repo/codebase state, (2) evaluation history of prior agent variants, and (3) the remaining experiment budget. It outputs code diffs that modify any part of the codebase — including its own meta-agent code.

**Architecture decision**: Rather than creating a standalone Python script (HyperAgents' approach), the NeoTrix MetaAgent will be a Rust trait implemented by an LLM-backed agent that generates structured edit proposals. The existing `SelfEdit`/`MicroEdit` infrastructure is the natural extension point.

```
┌──────────────────────────────────────────────────────────┐
│                    DGM-H Loop Engine                      │
│                                                           │
 │  ┌──────────────┐                                        │
 │  │  Archive      │── parent selection ──→ Parent         │
 │  │  (population) │←── store child ────  HyperAgent       │
 │  └──────────────┘                          │             │
 │                                            ▼             │
 │                    ┌──────────────────────────────┐      │
 │                    │  MetaAgent.forward()          │      │
 │                    │  Input: repo_path + eval_path │      │
 │                    │  Output: code diffs           │      │
 │                    └──────────┬───────────────────┘      │
 │                               │ diff patches              │
 │                               ▼                           │
 │                    ┌──────────────────────────────┐      │
 │                    │  Apply patches to sandbox     │      │
 │                    │  (Docker container clone)     │      │
 │                    └──────────┬───────────────────┘      │
 │                               ▼                           │
 │                    ┌──────────────────────────────┐      │
 │                    │  TaskAgent.evaluate()         │      │
 │                    │  (staged: subset → full)      │      │
 │                    └──────────┬───────────────────┘      │
 │                               ▼                           │
 │                    ┌──────────────────────────────┐      │
 │                    │  Score + store in archive     │      │
 │                    │  git reset --hard             │      │
 │                    └──────────────────────────────┘      │
└──────────────────────────────────────────────────────────┘
```

### Key Behavior

1. **MetaAgent.forward()**: Uses LLM with `tools_available='all'` (bash + file editor). Gets full repo path + eval history. Told to "Modify any part of the codebase to improve performance." Generates `.diff` patches.

2. **TaskAgent.forward()**: Uses LLM with no tools by default. Receives task inputs. Returns predictions. The MetaAgent can add tools to TaskAgent over time.

3. **Sandbox isolation**: Apply all diffs from parent lineage → run meta-agent (6h timeout) → run evaluation (5h timeout) → copy outputs → git reset.

4. **Domain protection**: `filter_patch_by_files()` strips changes to `domains/` — prevents MetaAgent from modifying evaluation code.

### Safety Constraints

- **Sandboxing**: All code generation evaluation runs in isolated container
- **Domain immutability**: Evaluation harness code is locked — meta-agent cannot touch `domains/`
- **Budget limit**: Hard cap on iterations-per-generation (N=3 meta-agent calls)
- **Git reset**: After each generation, container is restored to pristine state
- **Human oversight**: All diffs are logged before application; optional approval gate
- **Rollback**: If evaluation score drops below parent, child is discarded (not added to archive)

### Rollback Mechanism

Reuse NeoTrix's existing `snapshot_restore()` pattern, extended to file-level diffs:
- Before MetaAgent runs, snapshot entire `self_iterating/` and `meta_agent/` directories
- After evaluation, if score < parent_best_score × threshold (0.9): `git checkout` the snapshotted files
- If score >= threshold: commit diffs to a branch, add child to archive

## 4. Integration Points

| Module | Integration | File |
|--------|------------|------|
| `SelfIteratingBrain` | Add `MetaAgent` field; extend `run_seal_loop_pipeline()` with meta-agent step before capability update | `self_iterating/loop_impl/seal_loop.rs` |
| `SEAL pipeline` | Add `BrainStage::MetaEvolve` that invokes MetaAgent and applies diffs | `self_iterating/pipeline.rs` |
| `ReasoningBrain` | Add `experiment_budget`, `parent_id`, `lineage` fields; extend `absorb()` with code-diff absorption | `self_iterating/brain_impl.rs` |
| `ReasoningEngine` | MetaAgent uses engine as LLM provider; add `MetaAgent` reasoning type | `reasoning_engine/engine_core.rs` |
| `ChangeArchive` | Extend with diff storage per generation, parent selection queries, lineage graph | `change_archive.rs` |
| `sandbox.rs` | Add Docker orchestration: apply diffs → run eval → copy results → git reset | `sandbox.rs` |
| `Orchestrator` | MetaAgent uses PlannerNode for task decomposition of self-improvement | `orchestrator/mod.rs` |
| `KnowledgeSource` | Add `HyperAgents` variant with self_referential capability mapping | `core/knowledge_source.rs` |
| `GoalLoop` | Add `MetaImprove` goal type that schedules self-referential improvement cycles | `goal_loop/` |
| `StagnationDetector` | Detect when meta-agent produces no-performance-gain diffs | `stagnation.rs` |

## 5. Interface Design

```rust
/// A HyperAgents-style meta agent that observes and modifies the codebase.
pub struct MetaAgent {
    /// The LLM provider (wraps ReasoningEngine)
    pub engine: Arc<Mutex<ReasoningEngine>>,
    /// Generation budget: max iterations this meta-agent can run
    pub budget: u32,
    /// Whether this meta-agent can modify its own code
    pub self_referential: bool,
    /// Domain file patterns that must NOT be modified (e.g., "domains/*")
    pub protected_paths: Vec<String>,
}

impl MetaAgent {
    /// Generate code diffs to improve the task agent or meta agent.
    /// 
    /// HyperAgents-style: receives repo_path + eval_path + iterations_left,
    /// returns a set of file diffs.
    pub fn forward(
        &self,
        repo_path: &Path,
        eval_path: &Path,
        iterations_left: u32,
    ) -> Vec<FileDiff> { ... }
}

/// A code diff patch file (HyperAgents-style .diff format).
pub struct FileDiff {
    /// File path relative to repo root
    pub file_path: PathBuf,
    /// The diff content (unified diff format)
    pub diff_content: String,
    /// SHA of the file before modification
    pub parent_hash: String,
}

/// A hyperagent = TaskAgent + MetaAgent in a single editable codebase.
pub struct HyperAgent {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub task_agent: TaskAgentConfig,
    pub meta_agent: MetaAgentConfig,
    pub diffs_applied: Vec<FileDiff>,
    pub score: Option<f64>,
    pub generation: u64,
}

/// Parent selection strategy (mirroring HyperAgents archive).
pub enum ParentSelection {
    /// Proportional to score * child_success_rate
    ScoreChildProp,
    /// Proportional to score only
    ScoreProp,
    /// Always select the best-performing agent
    Best,
    /// Always select the latest generated agent
    Latest,
    /// Random uniform selection
    Random,
}

/// Parent selection config for the evolutionary archive.
pub struct SelectionConfig {
    pub strategy: ParentSelection,
    pub temperature: f64,   // for ScoreChildProp/ScoreProp
    pub min_generations: u64,
    pub archive_capacity: usize,
}

/// A modification proposal from the meta-agent to the self-improvement mechanism.
/// 
/// This is the key self-referential type: the meta-agent generates proposals
/// that can modify how future self-improvement works.
pub struct SelfModificationProposal {
    pub target: ModificationTarget,
    pub diffs: Vec<FileDiff>,
    pub expected_impact: String,
    pub safety_check: SafetyCheckResult,
}

pub enum ModificationTarget {
    /// Modify the task-solving agent code
    TaskAgent,
    /// Modify the meta-agent code (self-referential improvement)
    MetaAgent,
    /// Modify the evaluation/parent-selection mechanism (meta-meta)
    ImprovementMechanism,
    /// Add new capabilities (tools, memory, etc.)
    CapabilityExtension,
}

pub enum SafetyCheckResult {
    Passed,
    Failed { reason: String },
    NeedsHumanReview { concern: String },
}
```

## 6. KnowledgeSource Registration

| Source | Core Capability | Provenance |
|--------|----------------|------------|
| HyperAgents | `self_referential: 0.95, meta_agent: 0.90, population_evolution: 0.88, metacognitive_mm: 0.92, code_diff_gen: 0.90, sandbox_eval: 0.85, cross_domain_transfer: 0.87, archive_selection: 0.85` | Meta FAIR, arXiv:2603.19461 |

**Extension dimensions** (added to CapabilityVector's `extension` field):

| Extension Dimension | Value | Description |
|--------------------|-------|-------------|
| `self_referential_loop` | 0.95 | MetaAgent can modify itself |
| `meta_agent_ability` | 0.90 | Quality of meta-agent generated improvements |
| `population_diversity` | 0.88 | Effectiveness of archive-based exploration |
| `code_diff_generation` | 0.90 | Quality of generated code diffs |
| `sandbox_isolation` | 0.85 | Safety via Docker sandbox evaluation |
| `cross_domain_strategy` | 0.87 | Meta-strategies transfer across domains |

**Provenance tracking**: All absorbed knowledge tagged with source type `Paper`, paper URL `https://arxiv.org/abs/2603.19461`, timestamp.

## 7. Testing Strategy

| # | Test Name | Type | Description |
|---|-----------|------|-------------|
| 1 | `test_meta_agent_generates_diff` | Unit | MetaAgent.forward() with mock repo returns valid FileDiff patches that apply cleanly |
| 2 | `test_diff_filter_protects_domain` | Unit | `filter_patch_by_files()` strips all changes to files matching `domains/*` |
| 3 | `test_parent_selection_score_child_prop` | Unit | With 3 agents of known scores, ScoreChildProp selects higher-scoring agents more often (1000 trials, p < 0.01) |
| 4 | `test_archive_lineage_tracking` | Unit | After branching from parent, child records correct parent_id; archive can reconstruct lineage path |
| 5 | `test_self_referential_modification` | Integration | MetaAgent generates diff that modifies `meta_agent_method()` — diff applies, code compiles (cargo check), and method behavior changes |
| 6 | `test_staged_evaluation` | Integration | TaskAgent that scores < threshold on 10-task subset is NOT evaluated on 100-task superset (score stays 0.0) |
| 7 | `test_sandbox_apply_and_reset` | Integration | Apply diffs to sandbox → verify files changed → run eval → git reset → verify files restored |
| 8 | `test_negative_reward_rollback` | Integration | If evaluation score < parent × 0.9, diffs are rolled back and agent is NOT added to archive |
| 9 | `test_cross_domain_strategy_transfer` | E2E | MetaAgent trained on paper review improves task agent performance on math grading vs untrained baseline |
| 10 | `test_compounding_improvement` | E2E | Two sequential DGM-H runs: second run starting from first run's best agent outperforms first run |

## 8. Risk Assessment

| # | Risk | Likelihood | Impact | Mitigation |
|---|------|-----------|--------|------------|
| 1 | **Destructive self-modification**: MetaAgent modifies `meta_agent.rs` and breaks the improvement loop entirely | Medium | Critical | **Sandbox isolation**: all modifications evaluated in Docker before acceptance. **Protected path patterns**: `meta_agent.rs` must pass compilation check. **Human approval gate** for changes to core loop logic. **Rollback always possible**: snapshot before each meta-agent call. |
| 2 | **Infinite regress / resource exhaustion**: MetaAgent keeps making trivial changes that don't improve performance, burning LLM budget | High | High | **StagnationDetector**: if 3 consecutive generations show < 1% improvement, stop and surface best performer. **Budget limit**: hard cap of N iterations per experiment run. **Cost tracking**: per-generation LLM token cost logged. |
| 3 | **Evaluation contamination**: MetaAgent modifies evaluation harness (`domains/`) to inflate scores without actual improvement | Low | Critical | **Domain diff filtering**: `filter_patch_by_files()` is non-optional — any diff touching `domains/` is stripped. **Separate hash tracking**: evaluation harness SHA is recorded pre-run and verified post-run. **Red-team test**: intentionally try to game the evaluator. |
| 4 | **Cross-domain negative transfer**: Self-improvement strategies optimized for paper review degrade math grading performance | Medium | Medium | **Domain-specific archive branches**: each domain maintains its own archive lineage. **Transfer only on validation**: new meta-strategies are validated on held-out domain before deployment. **Rollback per domain**: failed transfer does not affect other domains. |
| 5 | **Docker overhead makes iteration slow**: Full sandbox setup + eval per generation = hours per iteration | High | Medium | **Incremental sandbox**: cache base Docker image, only re-run changed layers. **Parallel evaluation**: evaluate multiple child agents in parallel. **Optimized staged eval**: 10-task subset runs in minutes, 100-task full runs only for promising agents. |
| 6 | **MetaAgent LLM cost grows unbounded**: Each meta-agent call generates thousands of tokens of diff + analysis | High | Medium | **Token budget per generation**: hard cap on input/output tokens. **Cheap model for meta-agent**: use fastest available model (e.g., Claude Haiku) for meta-agent, reserve expensive model for evaluation. **Cost-aware parent selection**: penalize expensive agents. |

## 9. Implementation Roadmap

### Phase 1 (P0 — core loop, 1-2 sessions)
- Implement `MetaAgent` struct + `forward()` using ReasoningEngine
- Add `SelfModificationProposal` + `FileDiff` types
- Extend SEAL pipeline with `BrainStage::MetaEvolve`
- Implement `filter_patch_by_files()` for domain protection
- Register `HyperAgents` KnowledgeSource

### Phase 2 (P1 — evolution, 1-2 sessions)
- Implement `ChangeArchive` extension: lineage tracking, parent selection
- Implement `PopulationArchive` with configurable `ParentSelection` strategies
- Add staged evaluation protocol (subset → full)
- Implement `TaskAgentConfig` / `MetaAgentConfig` separation

### Phase 3 (P2 — safety + cross-domain, 1-2 sessions)
- Integrate Docker sandbox from `sandbox.rs` for evaluation
- Implement cross-domain strategy transfer mechanism
- Add `cross_domain_strategy` extension dimension tracking
- Build meta-agent lineage visualization tool

### Phase 4 (P2 — compounding, 1 session)
- Implement compounding improvement: serialize/deserialize archive across runs
- Add multi-domain optimization harness
- Implement cost-aware parent selection
- Write integration tests for end-to-end improvement

## 10. DGM-H Algorithm Pseudocode (for reference — adapted from HyperAgents paper)

```
Algorithm: DGM-H Loop (generation loop)

1. Initialize archive A with one hyperagent h₀
2. For generation g = 1 to N:
   a. Select parent hyperagent p from A:
      - ScoreChildProp: P(p) ∝ score(p) × child_success_rate(p)
   b. Create sandbox container C from base image
   c. Apply all diffs from p's lineage to C
   d. Clone codebase into C at /repo
   e. Run MetaAgent in C (timeout: 6h):
      - Input: /repo + /eval_results
      - Output: code diffs modifying /repo
      - MetaAgent can modify any file including meta_agent.py
   f. Filter diffs: remove changes to domains/ paths
   g. Apply accepted diffs to /repo in C
   h. Evaluate TaskAgent in C (staged, timeout: 5h):
      - Stage 1: Evaluate on 10-task subset
      - If score < threshold: score = 0, skip Stage 2
      - Stage 2: Evaluate on 100-task full set
   i. Copy evaluation results from C
   j. Create child hyperagent c:
      - id = new UUID, parent_id = p.id
      - diffs_applied = p.diffs + MetaAgent diffs
      - score = evaluation result
   k. Add c to archive A
   l. git reset --hard in C
   m. Destroy C
3. Return best hyperagent from A (by validation score)
```

> **Note**: This design doc specifies integration into NeoTrix's existing Rust architecture — not a Python port of HyperAgents. The key concepts (MetaAgent, population archive, self-referential diffs, sandbox evaluation) are mapped to existing NeoTrix modules (SelfIteratingBrain, ChangeArchive, SEAL pipeline, sandbox.rs).
