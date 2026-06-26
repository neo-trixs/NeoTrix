# Agentic Code Reasoning

Based on Meta 2026 — semi-formal reasoning for code generation.

**Reference**: arXiv 2603.01896 — structured reasoning templates improve semantic verification by up to 11pp. Key insight: premises → execution traces → formal conclusions forces the agent to gather evidence before concluding, preventing premature judgments.

## Integration with NeoTrix Pipeline

1. Analyze request + read target file context
2. Generate structured plan (semi-formal template)
3. Write code using SelfCodeWriter patterns
4. Review code via semi-formal template (premises + traces + conclusions)
5. Refine from review feedback (iterate)
6. Verify with cargo check (compile gate)
7. Record success/failure to EditHistoryTracker

## Types

### Semi-formal reasoning template

- `SemiFormalPremise` — category, statement, confidence
- `PremiseCategory` — Structural, TypeFact, Dependency, CallGraph, TestCoverage, Requirement, Custom
- `ExecutionTrace` — step label, file, line range, observed behaviour
- `FormalConclusion` — statement, supported_by, contradicts, verdict
- `Verdict` — Pass, Fail, Uncertain, NeedsReview
- `SemiFormalTemplate` — premises + traces + conclusions + overall assessment + format()

### Reasoning step types

- `AnalyzeRequest` — description + file context
- `PlanImplementation` — steps + semi-formal template
- `WriteCode` — generated code + target file
- `ReviewCode` — reasoning template + issues
- `RefineCode` — changes + reason
- `VerifyCorrectness` — compile_ok + output

### AgenticCodeReasoner

- `max_steps`, `current_step`, `history`, `quality_score`
- `analyze_request()` → AnalyzeRequest
- `plan_implementation()` → PlanImplementation
- `write_code()` → WriteCode
- `review_code()` → ReviewCode
- `refine_code()` → RefineCode
- `verify_with_cargo()` → VerifyCorrectness
- `run_reasoning_cycle()` — full cycle (analyze → plan → write → review → refine → verify)
- `apply_changes()` — write via SafeCodeApplier with backup + cargo check gate
- `generate_steps()`, `execute_step()`

### Utility

- `run_cargo_check()` — runs `cargo check --lib`
- `analyze_file_context()` — extracts file metadata (line count, functions, unsafe count, unwrap count)
