# NeoTrix Standard — Engineering & Governance Rules (NT-STD 1.0.1)

> **Status**: Canonical (标准版) | **Date**: 2026-09-21 | **Supersedes**: all prior rule lists as normative source
> **v1.0.1**: +§0.1 Precedence — 意识指导为先 (SIM-16).
> **Legacy sources** (informative only from this version on):
> `dev-rules.md` (R-P1–R-P110) · `docs/dev-rules.md` (R-P161–R-P257) ·
> `RUST-STANDARDS.md` · SIM-01–SIM-12 records · ABSORPTION-ROUND1–4.
> **Reading**: keywords MUST / MUST NOT / SHOULD / MAY follow RFC 2119.
> Every clause ends with `[Verify: …]` — a clause without a verifier is not a rule (SIM-07 lesson).
> **Change policy**: amend by version (1.0.1 patch / 1.1.0 minor / 2.0.0 breaking);
> each change cites its SIM-ID; Annex A keeps the legacy mapping forever.

---

## 0. Scope, References, Terms

- **Scope**: all code, docs, scripts, CI, and agent operations in this repository.
- **Normative refs**: ISO/IEC 25010:2023 (quality vocabulary) · ISO/IEC/IEEE 42010:2022
  (architecture description) · OTEL semantic-conventions-genai · A2A Protocol spec ·
  MADR 4.x (ADR format) · Ford et al. fitness-function taxonomy.
- **Terms**: SIM = pre-execution simulation record · SDB = stochastic-deterministic
  boundary (Propose→Verify→Commit→Reject) · fitness = executable architecture guard ·
  allowlist = sanctioned exception WITH expiry · tripwire = owner + signal + date.

### 0.1 Precedence — 意识指导为先 (v1.0.1)

Clauses conflict, or a clause conflicts with Part A principles, or a gate blocks
a demonstrably correct change: **Part A principles + SIM evidence decide**
(conscious override). An override MUST cite its SIM-ID and ADR, MUST take the
narrowest scope (single clause, single expiry), and MUST add a tripwire.
Literal compliance without judgment itself violates NTS-A08 (evidence) and
NTS-G01 (SIM-first). Mechanical gates enforce the letter; override reviews
enforce the spirit; both leave records. No silent overrides, ever.

---

## Part A — Principles (NTS-A)

- **NTS-A01** Zero-unsafe core. `#![forbid(unsafe_code)]` MUST hold for first-party
  crates. [Verify: compile fail + `cargo geiger --forbid-only` for the dependency tree]
  ← R-P1
- **NTS-A02** Ownership discipline. No dangling references; move-or-borrow is explicit
  at API boundaries. [Verify: compiler + review] ← R-P2
- **NTS-A03** Least exposure (Dark Forest). Public surface shrinks over time; each layer
  exposes ≤3 entry points via its facade. [Verify: facade count lint + API diff per PR] ← R-P3
- **NTS-A04** Unidirectional dependencies. L(n) MUST depend only on L(<n); the only
  exceptions are the three L0 re-export channels, reviewed quarterly.
  [Verify: `scripts/check-layer-deps.sh` + architecture_constraints tests + `cargo deny`] ← R-P4
- **NTS-A05** Module autonomy. Every module MUST compile and test in isolation.
  [Verify: per-module test file + CI] ← R-P5
- **NTS-A06** Type-level meaning. Newtypes for domain quantities; no primitive obsession
  on public APIs; no `Result<T, String>`. [Verify: clippy + review] ← R-P6 zone
- **NTS-A07** No big-bang rewrites. Change arrives by strangler-fig increments; each step
  MUST be compilable, tested, and rollbackable. [Verify: D-03 states + D-12] ← roadmap §0.3
- **NTS-A08** Evidence before assertion. Claims about state (counts, coverage, "fixed")
  MUST cite rerun output ≤7 days old. [Verify: SIM records + dashboard dates] ← SIM-01 lesson

---

## Part B — Architecture (NTS-B)

- **NTS-B01** Six layers stand. L0 Substrate → L1 Action → L2 Perception →
  L3 Embodiment → L4 Emotion → L5 Cognition → L6 Meta; responsibilities per BLUEPRINT D-01.
  [Verify: D-01 index + module docs] ← PYRAMID §3–§4
- **NTS-B02** Facades are the only doors. Cross-layer traffic MUST pass the layer facade
  (`nt_action_facade`, `l1_facade`, `nt_cognition_facade`) or a registry contract.
  [Verify: layer script (facade-exempted) + deny] ← GAP-02/H-01–H-08
- **NTS-B03** SDB four-piece contract. Every LLM→action path MUST declare
  Proposer, deterministic Verifier, durable Commit (outbox+state+audit), typed Reject.
  A path without a Verifier is a BLOCKER. [Verify: SDB-REGISTRY + contract tests] ← R-P244/SIM-04
- **NTS-B04** Verifier scoring. Verifiers MUST emit per-policy 1–5 scores; score < 3
  triggers critique+revision; depth follows Triage risk tier (≤3 rounds + stop signal);
  thresholds tighten-only; propose→reject loops beyond N escalate to HITL.
  [Verify: SDB-REGISTRY v0.2 columns + tests] ← R-P247
- **NTS-B05** Fitness taxonomy. Every guard header MUST declare scope × cadence × result
  × invocation (Ford). Coverage MUST span beyond atomic+triggered over time
  (temporal/holistic/continual named explicitly). [Verify: header lint + registry] ← R-P246
- **NTS-B06** Temporal allowlists. Every allowlist MUST carry an expiry date; expired =
  red. No permanent exceptions. [Verify: guard date comparison, no scheduler] ← SIM-09 L-3
- **NTS-B07** Coupling budget. Efferent coupling per module ≤ 5; new cross-layer edge
  needs a Confidence label (EXTRACTED/INFERRED/AMBIGUOUS); AMBIGUOUS on critical
  paths MUST resolve before merge. [Verify: confidence check + review] ← R-P230/R-P237
- **NTS-B08** Pipeline independence. Stages MUST be separate modules communicating via
  plain serializable structs; no shared mutable state, no trait-object channels.
  [Verify: PipelineIndependenceFitness] ← R-P231/R-P238
- **NTS-B09** Module responsibility. One primary responsibility per module (one sentence,
  documented); >3 responsibilities MUST split. [Verify: ModuleResponsibilityFitness] ← R-P240
- **NTS-B10** Absorption strengthens nodes. External patterns MUST land on existing
  nodes; parallel adapters are forbidden (R-P42). Each absorption files SIM + ADR +
  blueprint writeback. [Verify: ABSORPTION docs + SIM registry] ← R-P42/SIM practice
- **NTS-B11** A2A ladder. Interop climbs AgentCard → full task lifecycle → server-gen IDs
  + mismatch rejection → streaming/push → multi-binding; A2A is agent↔agent,
  MCP is agent↔tools. [Verify: conformance tests per rung] ← R-P254
- **NTS-B12** Decision records. Architecturally significant choices MUST have an ADR
  (MADR + sim-id + quality-attributes); spike-first when uncertainty is material;
  ADRs immutable except status; supersession bidirectional.
  [Verify: R-P249 lint bar + index] ← R-P244/R-P249

---

## Part C — Engineering (NTS-C)

- **NTS-C01** No panic paths in production. `unwrap/expect/panic/todo/unimplemented`
  MUST NOT appear outside tests; `let _ =` on Results MUST NOT discard errors.
  [Verify: workspace clippy lints (18) with `-D warnings` + PanicDensityFitness] ← RUST-STANDARDS §1
- **NTS-C02** Structured errors. Libraries use `thiserror`, binaries use `anyhow` +
  `.context()`; error codes follow `LAYER-MODULE-CODE` with category/severity/recovery.
  [Verify: clippy + review + schema] ← REFACTORING §5.3
- **NTS-C03** Async discipline. No blocking inside async tasks (`spawn_blocking`);
  cancellation-safety checked for select!-adjacent code; bounded `mpsc` channels
  between stages (capacity from load tests, not borrowed). [Verify: clippy async lints + OTEL depth metrics] ← Rust 2026 std
- **NTS-C04** Data thrift. Hot paths prefer `&str`/zero-copy (`rkyv` where measured);
  field order largest-first for cache locality on million-instance structs.
  [Verify: benchmarks + profiles before/after] ← Rust 2026 std
- **NTS-C05** Test pyramid. Unit (incl. property-based `proptest` for logic) +
  snapshot (`insta`) for outputs + integration per module + `cargo-nextest` runner;
  every module owns its test file; tests are pure (no network, tmp-only fs).
  [Verify: R-P235 check + CI] ← R-P235/graphify testing
- **NTS-C06** Fuzz campaign. Parsers/decoders first; thin harness + invariant oracle;
  nightly for fuzz builds only; corpus cached in CI; minimized repros become seeds.
  [Verify: `scripts/check-fuzz-ready.sh` + P3 campaign] ← R-P253
- **NTS-C07** Module docs cannot drift. Every `nt_*.rs` MUST open with `//!` docs;
  documented symbols MUST exist (CI imports them); baseline 111 → 0 then `--strict`.
  [Verify: `scripts/check-doc-drift.sh`] ← R-P232/R-P236
- **NTS-C08** Naming and format. `nt_` prefix for modules (except mod/lib/main);
  `cargo fmt --check` clean; `max_width` via rustfmt config where set.
  [Verify: pre-commit + CI fmt] ← hooks
- **NTS-C09** No secret ever lands. 700+-pattern + entropy scan pre-commit AND CI;
  leaked secrets rotate immediately. [Verify: gitleaks + hook] ← R-P162
- **NTS-C10** Contracts first. API/Event/DTO/ErrorCode changes land in contracts
  before implementation; drift fails CI. [Verify: drift gate] ← axum-harness pattern
- **NTS-C11** Events are validated. Bus events MUST pass schema validation; invalid =
  typed rejection + audit. Schema versions stay forward-compatible.
  [Verify: `Event::validate()` + registry] ← R-P234
- **NTS-C12** DDD boundaries. Code organizes by bounded context with ubiquitous
  language; no cross-context direct imports. [Verify: review + layer script] ← REFACTORING §3.5

---

## Part D — Gates & CI (NTS-D)

Pipeline order is fixed: G1 compile → G2 deps → G3 unit → G4 architecture →
G5 security → G6 integration → G7 perf → G8 docs. Red upstream stops the line.

- **NTS-D01** Compile gate. `cargo check --tests -p neotrix` MUST pass pre-commit
  (touched-Rust only) and CI. [Verify: hook + ci.yml] ← pre-commit/G1
- **NTS-D02** Dependency gates. `cargo deny` (advisories/licenses/bans/sources) +
  `cargo audit --locked` MUST pass; reverse deps = 0. [Verify: deny.yml/audit.yml] ← G2
- **NTS-D03** Architecture gates. Fitness registry + layer script + constraint tests
  MUST pass; violations decrease week-over-week until zero-or-exempt.
  [Verify: G4 + dashboard] ← GAP-02/GAP-03
- **NTS-D04** Coverage floor. Transitional 70 (anti-decay), target 80 for critical
  paths; per-package gates via coverage-gate metadata once multi-crate.
  [Verify: `cargo llvm-cov --fail-under-lines N` + `make coverage-gate`] ← GAP-08/R3
- **NTS-D05** Perf anti-regression. Criterion baselines + `critcmp` compare; >5%
  local regression or 110% CI alert blocks (after 1-week bake on gh-pages).
  [Verify: bench.yml + `make bench-compare`] ← GAP-08/R3
- **NTS-D06** Security gates. Secret scan + SBOM (CycloneDX) + container CVE scan +
  monthly LLM red-team rotation. [Verify: security-scan.yml + ledger] ← R-P161–170
- **NTS-D07** Doc gates. Module README + ADR-per-decision + generated API docs;
  doc-drift `--strict` once baseline hits 0. [Verify: G8 + drift script] ← R-P232/GAP-08
- **NTS-D08** Weekly fitness report. SelfTestRegistry summary publishes weekly
  (pass/fail rate, new violations, allowlist expiries). Architects read it;
  unread reports are process failure, not paper. [Verify: artifact exists] ← Ford/R-P246

---

## Part E — Security & Supply Chain (NTS-E)

- **NTS-E01** Seven-stage input pipeline. External input MUST pass validation →
  sanitization → behavior monitoring → anomaly detection → response filtering →
  memory scrubbing → audit logging. No stage skipped for "trusted" input.
  [Verify: shield pipeline tests] ← R-P161
- **NTS-E02** Egress deny-by-default. All outbound traffic passes the rule engine
  (domain/CIDR/scheme + TTL); denies are logged. [Verify: rule tests + logs] ← R-P165
- **NTS-E03** Immutable audit ledger. Security-relevant actions append to a
  tamper-evident log (hash-chained), retained ≥90 days. [Verify: chain check] ← R-P168
- **NTS-E04** Sandbox fail-closed. Filesystem/network/process/environment escapes
  terminate on first violation. [Verify: escape tests] ← R-P169
- **NTS-E05** Build-surface allowlist. New `build.rs` / `proc-macro=true` /
  `[build-dependencies]` MUST have human review + ADR; build.rs MUST NOT fetch
  from network (strict-gated). [Verify: `scripts/check-build-surface.sh`] ← R-P248
- **NTS-E06** Audit ratchet. cargo-vet deferred audits + exceptions that shrink over
  time; SBOM per release; SLSA/Scorecard/Trusted-Publishing climb in P3.
  [Verify: vet metadata + SBOM artifact] ← R2/C + R-P248
- **NTS-E07** Prompt-injection defense in depth. Retrieved content is NEVER trusted
  instructions; tool calls validate independent of model claims; delimiters +
  sentinel neutralization on untrusted blocks. [Verify: injection tests] ← graphify SEC
- **NTS-E08** Safety monitor + HITL. Irreversible/high-stakes actions need approval
  gates with durable state; anomaly detection alerts; MTTR < 30s proven by chaos.
  [Verify: chaos report + monitor tests] ← QS-5/P3

---

## Part F — Agent Operations (NTS-F)

- **NTS-F01** Dual-track routing. Latency-bound paths use predictive routing
  (win-probability vs threshold τ, single dispatch); async verifiable paths use
  scored cascades (cheap→dear, accept iff g(x,y) ≥ τᵢ). Track PGR/CPT/$-per-1M;
  τ calibrated on own queries. [Verify: gateway metrics dashboard] ← R-P250
- **NTS-F02** GenAI telemetry dialect. All LLM telemetry speaks `gen_ai.*`
  (operation/provider/model/usage/duration); CLIENT spans; content capture opt-in
  + redacted, default OFF. [Verify: span schema test + dashboard] ← R-P251
- **NTS-F03** Memory tier discipline. Blocks carry label + description + limit +
  read-only flag; core ≤ 80% context; append-only concurrency; overflow playbook
  (split→archive→rethink); shared blocks for supervisor/worker; agent-written
  memory is untrusted (SDB-verified writes). [Verify: checklist + tests] ← R-P252
- **NTS-F04** Eval quartet + gates. Faithfulness, answer relevancy, context
  precision/recall labeled by component and truth-need; scores gate builds
  (assert-style); reference-free on prod traffic, reference-based on golden;
  judge model fixed; thresholds human-checked. [Verify: eval CI job] ← R-P255
- **NTS-F05** Prompt-as-code. Prompts change via registry versions only; promotion
  requires eval-threshold pass; thresholds live with the prompt; guardian is the
  runtime backstop, not the promotion gate. [Verify: registry tests] ← R-P257
- **NTS-F06** Cost awareness. Per-request token accounting; per-task budgets
  (steps/tokens/wall-clock); cost-per-successful-task SLO in P3.
  [Verify: gateway accounting + budgets] ← Agentic RefArch
- **NTS-F07** Multi-agent minimalism. Single agent by default; router when one prompt
  can't hold all instructions; full multi-agent only for genuinely decomposable work.
  [Verify: ADR per decomposition] ← enterprise guide
- **NTS-F08** Observability of behavior. Every agent step logged as trace
  (context→proposal→tool→result→latency/tokens); golden eval set (20–30 cases)
  runs on every prompt/model/tool change. [Verify: traces + eval gate] ← RefArch L6

---

## Part G — Evolution (NTS-G)

- **NTS-G01** SIM-first. Triggered tasks (cross-layer, new gate, new dep/tool,
  new pipeline/crate, SDB) MUST file a SIM and receive GO before Implement;
  no SIM number = no review. [Verify: SIM registry + D-03] ← R-P241
- **NTS-G02** Timeboxed spikes. Task SIM ≤ 4h, read-only probes, throwaway artifacts;
  timeout records UNKNOWN; double NO-GO escalates. [Verify: SIM timeboxes] ← R-P242
- **NTS-G03** Tripwire per gap. Owner + observable signal + date; weekly review;
  fired tripwires act within 48h. [Verify: review log] ← R-P243
- **NTS-G04** Absorption protocol. New external patterns: probe → evidence → gap →
  external-first solution → landing → blueprint writeback; rejections recorded
  with reasons. [Verify: ABSORPTION docs + SIM] ← R-P10/SIM practice
- **NTS-G05** Release ladder. Tag-only triggers; changelog → crates (plz) →
  binaries (dist) → deps → provenance; cliff config path MUST resolve.
  [Verify: release workflow + manifest] ← R-P256
- **NTS-G06** Blueprint maintenance. Diagrams and index tables change together;
  IDs stable, additions only; every version cites SIMs; no orphan outputs
  (one deliverable = one owner node). [Verify: changelog + mapping] ← R-P245
- **NTS-G07** Experience absorption. Sessions distill to the KB hub; stale notes
  (>30d) merge-or-delete; AGENTS.md stays pointer-thin (≤130 lines/22KB).
  [Verify: hub + hook] ← experience-tree/R-P218
- **NTS-G08** Missing-canonical action. R-P111–R-P160 have no canonical source file
  (found 2026-09-21): they MUST be located-or-reratified within one cycle;
  until then they are SUSPENDED (not enforceable, not ignorable — see Annex B).
  [Verify: Annex B cleared] ← SIM-13 finding

---

## Annex A — Legacy Mapping (old → NTS)

| Legacy | NTS | Notes |
|--------|-----|-------|
| R-P1–R-P6 axioms | NTS-A01–A06 | verbatim elevation |
| R-P8 make_stage / R-P16 reread / R-P42 absorb-nodes / R-P79 same-session wiring | NTS-C08 note / NTS-C preamble / NTS-B10 / NTS-B10 | folded, IDs retired as normative |
| R-P161–170 security | NTS-E01–E04, NTS-D06 | E01←161, E02←165, E03←168, E04←169; rest → shield test plan |
| R-P171–178 OSINT | (domain annex, future NT-STD-1.1) | out of core scope; NOT dropped, parked |
| R-P179–185 memory-opt | NTS-F03 | folded with Letta controls |
| R-P186–190 coordination | NTS-F07, NTS-B11 | split by topic |
| R-P191–195 testing | NTS-C05 | + fuzz moved to NTS-C06 |
| R-P196–200 docs | NTS-C07, NTS-D07 | split gate vs content |
| R-P201–205 perf | NTS-D05, NTS-F01/F06 | split gate vs ops |
| R-P206–211 secops | NTS-D06, NTS-E06 | split |
| R-P212–220 doc-mgmt | NTS-D07, NTS-G07 | split |
| R-P230–240 graphify wave | NTS-B07/B08/C07/C11, NTS-E07 | confidence←230/237, pipeline←231/238, docdrift←232/236, schema←234, secthreat←233/239, responsibility←240, test-per-module←235→NTS-C05 |
| R-P241–245 SIM wave | NTS-G01–G04 (G04 covers 244), NTS-B06 (allowlist part of 245-era), NTS-B12 (ADR part) | registry/writeback → NTS-G06 |
| R-P246–249 round 2 | NTS-B05/B04(note)/NTS-E05/NTS-B12 | taxonomy←246, scoring←247→NTS-B04, build-surface←248→NTS-E05, ADR←249→NTS-B12 |
| R-P250–253 round 3 | NTS-F01/F02/F03, NTS-C06 | routing←250, OTEL←251→NTS-F02, memory←252→NTS-F03, fuzz←253→NTS-C06 |
| R-P254–257 round 4 | NTS-B11, NTS-F04, NTS-G05, NTS-F05 | A2A←254, eval←255→NTS-F04, release←256→NTS-G05, prompt←257→NTS-F05 |
| RUST-STANDARDS §§ | NTS-C01–C04 | error/async/data/style folded |

## Annex B — Withdrawn / Reserved / Suspended

| Item | Disposition | Reason |
|------|-------------|--------|
| TODO.md "12 compile errors" baseline (2026-09-20) | WITHDRAWN | stale; SIM-01 proved ≥6 fixed. Replaced by NTS-D01 + 7-day freshness (NTS-A08) |
| BLUEPRINT v1.0.0 D-03 (no SIM state) | WITHDRAWN | superseded by v1.2.0 D-03 + D-14 |
| R-P221–R-P229 | RESERVED | number gap kept for future security annex |
| R-P111–R-P160 canonical text | SUSPENDED | no source file found (SIM-13); locate-or-reratify in one cycle per NTS-G08 |
| R-P171–R-P178 OSINT rules | PARKED to NT-STD-1.1 domain annex | valid but out of core-edition scope |
| `cargo deny` duplicate workflows (deny.yml vs security-audit.yml) | FLAG | consolidate in P3; not a rule change |

## Annex C — Conformance Levels

| Level | Requires | Maps To |
|-------|----------|---------|
| L1 Build-safe | NTS-A01/A04, NTS-C01/C08/C09, NTS-D01/D02 | pre-commit + G1/G2 |
| L2 Architected | + NTS-B01–B04/B07–B10, NTS-C05/C07/C11, NTS-D03, NTS-G01–G04 | G3/G4 + SIM |
| L3 Production | + all remaining; thresholds live (D04/D05/E08/F04/F05) | G5–G8 + P3 drills |

A claim of "NT-STD 1.0 L2" MUST show the gate evidence; level inflation is a process violation.

---

*End of NT-STD 1.0.1 — next: NT-STD-1.1 (OSINT domain annex + R-P111–160 resolution).*
