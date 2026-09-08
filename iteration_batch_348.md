# Iteration Batch 348 — AI/Agent Architecture + Software Engineering Automation Research (2026-09-06)

## Sources Cited

| # | Source | Date | Key Signal |
|---|--------|------|------------|
| S1 | [arXiv:2605.13850v2 — 7×6 Agent Pattern Matrix](https://arxiv.org/html/2605.13850v2) | 2026-05 | Two-dimensional framework: 7 cognitive functions × 6 execution topologies = 28 named agent patterns. Five empirical laws of pattern selection (time pressure → complexity, action authority → governance, failure asymmetry → reflection). |
| S2 | [arXiv:2601.19752v1 — 12 Agentic Design Patterns](https://arxiv.org/pdf/2601.19752v1) | 2026-01 | System-theoretic framework: 5 functional subsystems (RWM, PG, AE, LA, IAC). 12 ADPs including Integrator, Controller, Reflector, Skill Build. Case study enhancing ReAct loop with ADPs. |
| S3 | [arXiv:2605.20173v1 — Stochastic-Deterministic Boundary (SDB)](https://arxiv.org/html/2605.20173v1) | 2026-05 | Production agent runtime primitive: 4-part contract (proposer, verifier, commit, reject). 6 patterns (Hierarchical Delegation, Scatter-Gather+Saga, Event-Driven Sequencing, Shared State Machine, Supervisor+Gate, HITL). Reliability decomposition y(t)=μt+σξ(t): architectural momentum μ dominates as model variance σ shrinks. |
| S4 | [SitePoint — Agentic Design Patterns 2026](https://www.sitepoint.com/the-definitive-guide-to-agentic-design-patterns-in-2026/) | 2026-03 | Six canonical patterns: Reflection, Tool Use, Planning, Multi-Agent, Orchestrator-Worker, Evaluator-Optimizer. Flow engineering = designing state machines around LLM calls, not prompts. |
| S5 | [MLflow — AI Agent Architectures 2026](https://mlflow.org/articles/types-of-ai-agent-architectures-2026-developer-guide/) | 2026-07 | Five canonical architectures (ReAct, Plan-Execute, Reflexion, ToT, Multi-Agent). 6 agent layers (perception, reasoning, planning, memory, tool use, oversight). Event-driven microservice designs reduce AI processing latency 60%. |
| S6 | [Arahi AI — Agent Architecture 2026](https://arahi.ai/blog/ai-agent-architecture) | 2026-05 | 6 architectural layers. Trade-offs: latency vs reasoning depth, single vs multi-agent, implicit vs explicit planning. Memory architecture = most sloppy area in agent systems. |
| S7 | [arXiv:2609.01236 — Continuous Autonomous Refactoring](https://arxiv.org/abs/2609.01236) | 2026-09 | Research roadmap: LLM-based refactoring as continuous maintenance, not occasional tool. 5 dimensions: multi-objective optimization, quality definition, multi-timescale integration, architecture/pattern, trust. |
| S8 | [arXiv:2608.23611 — REFINE Multi-Agent Refactoring](https://arxiv.org/abs/2608.23611) | 2026-08 | Evidence-aware multi-agent approach: smell identification → planning → LLM transformation → re-analysis → preservation checks. 68-73% code smell reduction. Residual risks: assert/fail-call changes, public-method removal. |
| S9 | [arXiv:2608.23564 — SWE Refactor Bench](https://arxiv.org/abs/2608.23564) | 2026-08 | Whole-repository migration benchmark: 20 tasks, 4 debt types. Only 28/520 runs (5.4%) pass all stages. Best model (claude-opus-5) scores 14.0%. Migration completeness ≠ behavioral correctness. |
| S10 | [arXiv:2603.04177 — CodeTaste Benchmark](https://arxiv.org/pdf/2603.04177) | 2026-03 | CodeTaste: 100 real refactorings from OSS. Instructed alignment up to 70%; Open Track (autonomous discovery) <8%. Planning mode doubles alignment. Agents take "shortcuts" — minimal changes unless explicitly instructed. |
| S11 | [JetBrains — Rider AI Refactoring Skill](https://blog.jetbrains.com/dotnet/2026/08/19/rider-refactoring-code-skill/) | 2026-08 | IDE-as-refactoring-engine: agent taps resolved syntax tree instead of reconstructing it via build-guess cycle. Tool calls drop 2513→926, duration 158s→27s, cost $0.33→$0.12 per task. Key: let agent use IDE intelligence, don't rebuild from scratch. |
| S12 | [CodeScene ACE — Auto-Refactor](https://codescene.digitgaming.com/docs/auto-refactor/index.html) | 2026 | Fact-checking model validates AI refactoring against Code Health metric. 100K+ real JS refactoring samples as training data. Existing AI only delivers correct refactorings 37% of the time without fact-checking. |
| S13 | [Stack Overflow Blog — AI 10x Tech Debt](https://stackoverflow.blog/2026/01/23/ai-can-10x-developers-in-creating-tech-debt/) | 2026-01 | Four fix points: planning, coding, reviewing, maintenance. AI creates tech debt faster than it can fix it. Need proactive agents that "invent their own work" for maintenance. |
| S14 | [TechPulse — Microservices Architecture 2026](https://techpulsesite.com/microservices-architecture-guide-2026/) | 2026-05 | 2026 consensus: start with modular monolith, extract services when clear reason. Event-driven with idempotent consumers. Schema evolution discipline. |
| S15 | [Precision AI Academy — Microservices 2026](https://precisionaiacademy.com/blog/microservices-architecture-guide-2026) | 2026-04 | AI model serving as dedicated inference service. GPU scale separately. Latency budget management for AI inference. OpenTelemetry from day one. |

## Defects Found

### DEFECT-348-01: No Stochastic-Deterministic Boundary (SDB) for LLM Tool Calls

**Research signal**: The SDB is the load-bearing primitive of production agent runtimes in 2026 (S3). It is a 4-part contract: proposer (LLM), verifier (deterministic check), commit (durable write), reject signal. 71% of 21 published agent failure post-mortems localize to weaknesses at this boundary. As model variance σ shrinks, architectural momentum μ becomes the dominant reliability lever (S3).

**Current architecture**: NeoTrix dispatches LLM calls through `nt_core_observer_error` which has retry + circuit breaker + fallback (file: `nt_core_observer_error.rs:29`). However:
- No explicit verifier step between LLM proposal and system action (the "reject signal" contract)
- No commit/reject separation — LLM outputs are consumed directly or discarded
- No typed rejection responses back to the LLM on verification failure
- The circuit breaker is a generic network-level breaker, not a semantic verification boundary

**Impact**: LLM hallucinations or malformed outputs propagate directly into system state. When an LLM proposes a KB write or tool call, there is no deterministic verification gate before the commit. This is exactly the failure mode the SDB was designed to prevent.

**Suggestion**: Add `StochasticDeterministicBoundary` trait to NT-CORE:
1. `Proposer` — existing LLM call path (unchanged)
2. `Verifier` — deterministic check on LLM output (schema validation, type check, constraint check)
3. `Commit` — durable write only after verifier passes
4. `Reject` — typed response back to LLM when verification fails, with failure reason for retry
5. Wire into MCP tool calls, KB writes, and SEAL pipeline stage transitions

---

### DEFECT-348-02: Missing Cognitive Function × Execution Topology Classification

**Research signal**: The 7×6 agent pattern matrix (S1) provides 28 named patterns classified by cognitive function (Perception, Memory, Reasoning, Action, Reflection, Collaboration, Governance) × execution topology (Chain, Route, Parallel, Orchestrate, Loop, Hierarchy). Five empirical laws govern pattern selection based on environmental constraints.

**Current architecture**: NeoTrix has domains (NT-CORE, NT-MIND, etc.) and a SEAL pipeline, but no formal classification of which agent pattern each module implements:
- NT-CORE's E8 reasoning engine = which topology? (likely Loop or Orchestrate)
- GWT attention routing = which topology? (likely Route)
- SEAL pipeline = which topology? (likely Chain with Loop)
- Multi-agent sub-task dispatch = which topology? (likely Orchestrate)
- No mapping of cognitive functions to modules

**Impact**: Without explicit pattern classification, NeoTrix cannot reason about its own architectural trade-offs. The empirical laws (S1) — time pressure determines complexity, failure asymmetry reshapes reflection — cannot be applied because the system doesn't know what pattern it's using. Architecture decisions are implicit, not explicit.

**Suggestion**: Add `PatternClassifier` to NT-META:
1. Enumerate active patterns: `ActivePattern { cognitive_fn: CognitiveFunction, topology: ExecutionTopology, module_path: String }`
2. Map each NT-* module to its pattern coordinates (e.g., E8 = C3×T5 (Reasoning×Loop), GWT = C1×T2 (Perception×Route))
3. Apply Law 1: if task time pressure < threshold, automatically simplify pattern set
4. Apply Law 3: if failure cost asymmetry is high, bias Reflection toward safe error
5. Feed into ConsciousnessTree for architecture-aware meta-cognition

---

### DEFECT-348-03: No Multi-Agent Collaboration Protocol for Cross-Domain Tasks

**Research signal**: Multi-agent systems with hierarchical delegation, peer-to-peer, and adversarial debate topologies are the standard 2026 production pattern (S4, S5, S6). The Orchestrator-Worker pattern dynamically spawns subtasks based on evolving context (S4). Production systems combine patterns: ReAct loop inside hierarchical supervisor-worker topology (S5).

**Current architecture**: `nt_core_task_dispatcher` dispatches sub-tasks to specialist agents (file: `nt_core_task_dispatcher.rs`). `nt_core_orchestration` exists. But:
- No typed message protocol between agents (agents communicate via KB reads/writes, not structured messages)
- No supervisor-worker pattern — dispatch is flat, not hierarchical
- No adversarial debate for self-critique (only internal SEFA self-assessment)
- No dynamic spawning based on evolving context (fixed task decomposition)
- No scatter-gather for parallel subtask execution with result aggregation

**Impact**: Cross-domain tasks (e.g., "absorb a new external library and wire it to production") require coordination across NT-MIND, NT-ACT, NT-MEMORY, NT-CORE. Without a structured multi-agent protocol, these tasks rely on ad-hoc KB-mediated communication, leading to lost context, duplicated work, and coordination overhead.

**Suggestion**: Add `AgentCollaborationProtocol` to NT-ACT:
1. `SupervisorAgent`: decomposes complex cross-domain tasks into sub-tasks with typed interfaces
2. `WorkerAgent`: each NT-* domain exposes a typed task handler (input schema → output schema)
3. `ScatterGather`: parallel fan-out to multiple workers with result aggregation
4. `AdversarialDebate`: for high-stakes decisions, pit two agents against each other
5. Wire into SEAL pipeline for multi-domain evolution tasks

---

### DEFECT-348-04: No Explicit Memory Architecture Tiers

**Research signal**: Memory architecture is the most sloppy area in agent systems in 2026 (S6). Three tiers required: short-term (context window), working (RAG/vector search), long-term (persistent structured records). The default "dump everything into context" burns tokens and degrades reasoning (S6). Memory policy must be explicit, not default (S5, S6).

**Current architecture**: NeoTrix has:
- Short-term: conversation context (implicit)
- KB: SQLite with embeddings (long-term)
- Working memory: some modules use `DualBrainWorkingMemory`
- But no unified memory tier abstraction with explicit policies

Missing:
- No `MemoryPolicy` struct defining what goes in each tier
- No automatic promotion/demotion between tiers
- No token budget management per tier
- No working memory retrieval policy (how many results, relevance threshold)
- No long-term memory compaction/compression strategy

**Impact**: As sessions grow, context windows fill with irrelevant information. The SEAL pipeline's distillation step operates on the full context rather than tiered memory. Reasoning quality degrades because the LLM sees everything rather than curated tier-appropriate information.

**Suggestion**: Add `MemoryTierManager` to NT-MEMORY:
1. `MemoryPolicy { short_term: ShortTermPolicy, working: WorkingPolicy, long_term: LongTermPolicy }`
2. `ShortTermPolicy { max_tokens, eviction_strategy }` — LRU or relevance-based
3. `WorkingPolicy { retrieval_count, relevance_threshold, refresh_interval }` — RAG config
4. `LongTermPolicy { compaction_threshold, importance_decay, archival_strategy }` — persistence
5. Wire into `HeartbeatAggregator`: memory tier health (utilization, retrieval latency, staleness)

---

### DEFECT-348-05: No Replay Divergence Detection for SEAL Pipeline

**Research signal**: Replay divergence is a specific, named failure mode in production agent systems (S3): the same input replayed on a newer model version produces different downstream events. This is a discrete cross-version effect tied to event-driven sequencing patterns. LLM-based consumers re-interpret the log differently across runtime conditions.

**Current architecture**: SEAL pipeline runs exploration → distillation → self-test → absorption. The pipeline stages use LLM calls for distillation and exploration. But:
- No version tracking of model/prompt/retrieval index that produced each stage output
- No detection of when re-running the same pipeline produces different results
- No deterministic replay capability — pipeline outputs are non-reproducible across model updates
- The `converge_check()` function checks architecture but not pipeline output stability

**Impact**: When the LLM model is updated (e.g., new provider version), SEAL pipeline outputs may silently change. The system cannot detect that previously absorbed knowledge was derived from a different model version. This creates hidden inconsistency in the knowledge base.

**Suggestion**: Add `ReplayDivergenceDetector` to SEAL pipeline:
1. `PipelineFingerprint { model_version, prompt_hash, retrieval_index_hash, input_hash }` per stage
2. `DivergenceCheck`: compare current output fingerprint against stored fingerprint
3. `DivergenceAlert`: when fingerprints diverge beyond threshold, flag for review
4. `DeterministicReplay`: ability to replay a specific pipeline run with frozen inputs
5. Wire into `HeartbeatAggregator`: pipeline stability signal

---

### DEFECT-348-06: No Continuous Autonomous Refactoring Pipeline

**Research signal**: LLM-based refactoring is now envisioned as continuous maintenance, not occasional tool invocation (S7). The REFINE approach achieves 68-73% code smell reduction with evidence-aware multi-agent execution (S8). However, autonomous refactoring without fact-checking only delivers correct results 37% of the time (S12). SWE Refactor Bench shows only 5.4% of whole-repository migrations pass all stages (S9). CodeTaste shows autonomous refactoring alignment <8% without explicit instruction (S10).

**Current architecture**: NeoTrix has `nt_repair::healer` for self-healing and the Dark Forest axiom ("compile + test + connect or delete"). But:
- No automated code smell detection pipeline (no AST-based analysis)
- No evidence-aware refactoring with preservation checks (compile + test + behavioral equivalence)
- No integration with external refactoring tools (CodeScene ACE, Rider skills)
- No continuous monitoring for architectural erosion (only periodic `converge_check`)
- No cost-budget-aware refactoring (token cost of LLM refactoring vs. benefit)

**Impact**: Technical debt accumulates silently. The SEAL pipeline evolves capabilities but does not maintain code quality. The Dark Forest rule is manually enforced, not automated. There is no path from "code smell detected" to "refactoring applied and verified" without human intervention.

**Suggestion**: Add `ContinuousRefactoringPipeline` to NT-REPAIR:
1. `SmellDetector`: AST-based code smell detection (large methods, deep nesting, duplication)
2. `EvidenceAwareRefactorer`: LLM proposes refactoring + static analysis verifies correctness
3. `PreservationGate`: compilation check + unit test + behavioral equivalence before commit
4. `CostBudgeter`: track token cost of refactoring, compare against estimated maintenance savings
5. Wire into SEAL pipeline: periodic code quality evolution alongside capability evolution

---

### DEFECT-348-07: No Event-Driven Architecture with Idempotent Consumer Pattern

**Research signal**: Event-driven architecture is the dominant 2026 pattern for decoupled services (S14, S15). Key requirements: idempotent consumers, per-aggregate ordering (not global), dead-letter queues, schema evolution discipline, correlation ID propagation. The industry has converged on "start with modular monolith, extract when justified" (S14).

**Current architecture**: NeoTrix has `EventBus` (file: `nt_core_event_bus.rs:25`) with publish/subscribe, persistence, and layer-specific subscribers. However:
- No idempotency guarantee on event consumers — events can be processed multiple times
- No dead-letter queue for poison messages
- No schema versioning for event contracts
- No correlation ID propagation across event chains
- No per-aggregate ordering guarantee (only channel-level)
- Event contract is implicitly defined, not versioned or validated

**Impact**: If a consumer crashes mid-processing, the event may be re-delivered and processed twice. Without idempotency, this creates duplicate side effects (KB writes, tool calls). Without schema versioning, event producers and consumers can drift silently. Without correlation IDs, cross-event debugging is impossible.

**Suggestion**: Add event-driven hardening to `EventBus`:
1. `IdempotencyGuard`: store processed event IDs in a dedup table, skip on re-delivery
2. `DeadLetterQueue`: route repeatedly-failing events to DLQ for manual inspection
3. `SchemaRegistry`: version event schemas, validate at publish time, reject incompatible changes
4. `CorrelationId`: propagate trace ID across event chains for end-to-end observability
5. `PartitionKey`: per-aggregate ordering via partition key (entity ID → partition assignment)

---

### DEFECT-348-08: No Governance Pattern for High-Risk LLM Actions

**Research signal**: Action authority determines governance pattern (S1). Advisory-only systems need Approval Gate. Low-risk auto-execution needs Blast Radius Control. High-risk irreversible actions need Guardrail Sandwich (pre- and post-checks). The Controller pattern continuously monitors agent behavior against ethical principles (S2).

**Current architecture**: NeoTrix has:
- `NT-GOVERNANCE` domain with `gov/steward` skill
- `nt_core_self_constitution` for behavioral rules
- `nt_core_observer_error` for error recovery
- But no per-action governance at the LLM output boundary

Missing:
- No classification of LLM actions by risk level (read-only vs. write vs. destructive)
- No approval gate for high-risk actions (e.g., deleting KB entries, executing external tools)
- No blast radius control (pre-compute impact before executing)
- No guardrail sandwich (pre-check + post-check) for irreversible operations
- No audit trail linking LLM proposals to their consequences

**Impact**: An LLM hallucination in a tool call could delete production data or execute unintended external actions. The Egress Privacy Guard prevents data leakage but not action harm. There is no "are you sure?" step between LLM proposal and irreversible system change.

**Suggestion**: Add `ActionGovernance` to NT-GOVERNANCE:
1. `RiskClassifier`: classify actions as Read(low)/Write(medium)/Destructive(high)/Irreversible(critical)
2. `ApprovalGate`: require human confirmation for critical actions (configurable: auto-approve below threshold)
3. `BlastRadiusControl`: pre-compute which KB entries/modules/tools will be affected
4. `GuardrailSandwich`: pre-check (input validation) + post-check (output validation) for all writes
5. `AuditTrail`: immutable log of every LLM proposal → verification → commit/reject → consequence

---

### DEFECT-348-09: No Flow Engineering / State Machine for Agent Control Flow

**Research signal**: Flow engineering is the discipline of designing control flow, state transitions, and decision boundaries around LLM calls (S4). The questions shift from "how to prompt" to "what is the state machine governing behavior?" Frameworks like LangGraph provide typed state and `Send` API for dynamic graph branching (S4).

**Current architecture**: NeoTrix has SEAL pipeline (sequential stages), ConsciousnessTree (6-stage feedback loop), and background loops. But:
- No formal state machine definition for agent workflows
- No typed state transitions with explicit decision boundaries
- No dynamic graph branching at runtime (pipeline is fixed topology)
- No termination conditions formalized (convergence checks are ad-hoc)
- No fallback paths defined in the state machine (only in error recovery)

**Impact**: Agent workflows are implicit in code flow rather than explicit state machines. When a workflow fails, the system cannot reason about which state it was in, what transitions are available, or where to resume. This makes debugging agent behavior a code-reading exercise rather than a state inspection.

**Suggestion**: Add `WorkflowStateMachine` to NT-CORE or NT-ACT:
1. `StateDefinition { id, entering_action, active_action, exiting_action, transitions }`
2. `Transition { from, to, condition, guard }` — typed, validated at build time
3. `DynamicBranching`: runtime spawning of parallel states based on context
4. `TerminationCondition`: explicit exit conditions per state (timeout, convergence, failure)
5. `ResumeFrom`: checkpoint state for long-running workflows (resume after crash)

---

### DEFECT-348-10: No Evaluator-Optimizer Pattern for SEAL Pipeline Quality

**Research signal**: The Evaluator-Optimizer pattern separates the "doer" agent from the "judge" agent (S4). The evaluator uses rubrics, reference outputs, or LLM-as-judge to score output. The optimizer adjusts strategy based on evaluation feedback. This is the agentic equivalent of test-driven development. CodeScene ACE's fact-checking model validates refactoring against ground truth (S12).

**Current architecture**: SEAL pipeline has exploration → distillation → self-test → absorption. The `self_test` stage runs SelfTest registry. But:
- No explicit evaluator separate from the explorer/distiller
- No rubric-based scoring of pipeline outputs
- No optimizer loop that adjusts exploration strategy based on evaluation
- SelfTest checks compilation/registration, not output quality or alignment with goals
- No reference output comparison (no "gold standard" for what good evolution looks like)

**Impact**: SEAL pipeline evolves capabilities but cannot evaluate whether the evolution is good. The self-test checks structural correctness but not behavioral quality. Without an evaluator, the optimizer has no signal to improve against, leading to stagnation or drift.

**Suggestion**: Add `EvaluatorOptimizerLoop` to SEAL pipeline:
1. `Evaluator { rubric: Rubric, reference_outputs: Vec<ReferenceOutput>, scoring_fn }` — separate from explorer
2. `Rubric`: weighted criteria (correctness, performance, code quality, architectural alignment)
3. `Optimizer`: adjust exploration parameters based on evaluation scores (temperature, model selection, strategy)
4. `ImprovementLoop`: evaluate → optimize → explore → evaluate, with convergence check
5. Wire into `HeartbeatAggregator`: evolution quality trend signal
