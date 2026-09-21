# NeoTrix Absorption Round 3: Routing / OTEL-GenAI / Memory / Fuzz

> **Absorption Date**: 2026-09-21 | **SIM**: SIM-11 | **Status**: Landed
> **Round 1**: graphify patterns (R-P230–R-P240) | **Round 2**: fitness/verifier/supply-chain/ADR (R-P246–R-P249)
> **This round**: cost-quality routing, telemetry standard completion, memory tier discipline, fuzz bake plan.

---

## 1. Source Map

| # | Source | Type | Used For |
|---|--------|------|----------|
| S-11 | RouteLLM (Ong et al., ICLR 2025; lm-sys/RouteLLM, MF router) | Paper + GitHub | Predictive routing: win-probability + cost threshold τ, PGR/APGR/CPT metrics |
| S-12 | FrugalGPT (Chen et al., arXiv 2305.05176) | Paper | Sequential cascade: scoring fn g(x,y), escalate rule, joint budget optimization |
| S-13 | llms.blog routing-vs-cascades production comparison (2026) | Industry guide | Router-type decision table (MF 2–8ms vs BERT 15–35ms vs cascade tail latency) |
| S-14 | OTEL semantic-conventions-genai (spans + metrics + MCP, Development status) | Standard | gen_ai.* span/metric names, CLIENT kind, token/duration histograms |
| S-15 | OTEL blog GenAI observability (2026-05) + OpenObserve practical guide | Guide | Content-capture opt-in + redaction, Copilot/Codex/Claude precedents |
| S-16 | Letta memory architecture (skills/memory-architecture.md, agent-development SKILL.md) | GitHub project | 3-tier blocks, limits/descriptions, append-vs-replace concurrency, overflow splits |
| S-17 | Agent-memory-atlas Letta page (code map + risks) | Analysis | Block recompilation, read-only flags, optimism locking, trust-layer gap |
| S-18 | Rust Fuzz Book (cargo-fuzz) + fuzze.rs end-to-end (2026) | Book + walkthrough | Thin harness + invariant oracle, tmin, corpus seeds, nightly-only fuzz build |
| S-19 | Trail of Bits / RedHat cargo-fuzz SKILL.md | Skill docs | sanitizer-none for safe Rust, Arbitrary structured inputs, crash triage |
| S-20 | depot.dev distributed Rust fuzzing in GHA (2026) | Industry guide | Corpus cache prefix-restore + timestamped save, matrix shards, -merge=1 job |

Rejected alternatives: causal-LLM router (expressive but high latency — llms.blog);
AFL++/LibAFL (heavier than cargo-fuzz for a first campaign); full SLSA already deferred R2.

---

## 2. Pattern → NeoTrix Mapping (with local baselines)

### 2.1 Routing: from FallbackChain to scored cascade + predictive router (S-11–S-13)

Local baseline (measured): `FallbackChain` struct exists in
`crates/neotrix-gateway/src/model_gateway.rs:166` (availability fallback only);
`model_router.rs` documents fallback chains; **zero** occurrences of cascade/score/judge
in gateway files — no scoring function, no predictive router, no cost metrics.

Landing design (P-task for gateway owner, not this round's code):

1. **Keep FallbackChain for availability** (provider down → next). Add **scoring cascade
   for quality**: FrugalGPT rule — try cheap model, score g(x,y) in [0,1],
   accept iff ≥ τᵢ else escalate. Score fns in increasing cost order:
   deterministic schema parse (free) → quality classifier (cheap) → LLM judge (dear).
2. **Add predictive router for latency-bound paths** (RouteLLM MF style):
   P(strong-wins|q) vs threshold τ; single-model dispatch, 2–8ms overhead.
   Decision table (llms.blog): interactive p99-bound → predictive router;
   async verifiable (extraction, batch summary) → cascade.
3. **Metrics**: PGR (gap recovered), CPT(x%) (strong-call % for x% PGR), $/1M tokens.
   Threshold τ calibrated on own query sample, not borrowed (RouteLLM README warning).
4. Triage Gate stays as the risk-tier classifier (feeds V-2 depth + router choice).

### 2.2 OTEL-GenAI: complete the map, don't rebuild (S-14–S-15)

Local baseline (measured): `nt_core_span.rs` already carries 7 gen_ai.* fields
(operation, provider, stream, response id/model/finish_reasons, usage in/out tokens);
opentelemetry 0.27 stack optional behind `telemetry` feature.

Gap table (spec → status):

| Spec item | Status in NeoTrix |
|-----------|-------------------|
| `gen_ai.request.model` / `response.model` split | PARTIAL (response only) → add request |
| `temperature/top_p/max_tokens` | MISSING → add (quality-regression correlation) |
| `time_to_first_chunk` + duration histogram buckets | MISSING → add per spec buckets |
| `execute_tool` spans (`gen_ai.tool.name`) | MISSING → add for tool calls |
| `gen_ai.conversation.id` session tracing | MISSING → add |
| Content capture (messages/tool args) | MISSING → opt-in flag + Collector redaction, default OFF (PII) |
| Token/cost dashboards per (provider, model) | MISSING → P-task |

Rule: never ship provider SDK field names (`usage.prompt_tokens`) into telemetry;
map to `gen_ai.usage.input_tokens` at instrumentation (OpenObserve migration note).

### 2.3 Memory tier discipline: Letta controls on top of existing tiers (S-16–S-17)

Local baseline (measured): `tiered_memory/` already mirrors Letta 3-tier
(Tier1Core 2–5K chars / Tier2Archival vector / Tier3Recall keyword+time);
**missing** Letta controls: block descriptions, read-only flags, char-limit
enforcement (only a "No capacity limit" comment found), append-vs-replace
concurrency rules, overflow split playbook, 80%-of-context cap, shared-block pattern.

Landing checklist (P-task for memory owner; rule R-P252 enforces):

- [ ] Every block: label + description (when to read/write) + char limit + read-only flag
- [ ] Core total ≤ 80% context window (measured, not wished)
- [ ] Concurrent writes use append (`memory_insert` semantics); replace/rethink single-writer only
- [ ] Overflow playbook: split-by-topic → split-by-time → archive → rethink-summarize
- [ ] Shared blocks for supervisor/worker patterns (single source of truth)
- [ ] Trust note (atlas warning): agent-written memory can encode wrong beliefs —
  pair with SDB verifier on memory-write path (links R-P247)

### 2.4 Fuzz bake plan (S-18–S-20)

Local baseline: no `fuzz/` dir. Rules of the campaign:

1. Nightly ONLY for `cargo fuzz` builds; production stays stable (fuzze.rs).
2. Thin harness + **invariant oracle** (round-trip/idempotency/order), not just crash-seeking.
3. First targets = parsers/decoders (local candidates: `nt_io_web/api.rs`, `tiles.rs`,
   `keyframe_motion.rs`) — unsafe/FFI first once inventoried.
4. `--sanitizer none` for pure-safe-Rust targets (2x speed); ASan where unsafe/FFI.
5. CI: corpus cache prefix-restore + timestamped save, matrix shards, `-merge=1` job
   (depot pattern); crashes → artifact on failure only.
6. Seed corpus committed; `tmin` before filing; minimized input becomes regression seed.

---

## 3. Landings (this round)

| ID | Artifact | Verifies As |
|----|----------|-------------|
| L-R3-1 | scripts/check-fuzz-ready.sh (readiness 1/4: nightly ok; cargo-fuzz/fuzz/corpus missing) | script runs, reports honestly |
| L-R3-2 | Makefile `fuzz` target | `make -n fuzz` resolves |
| L-R3-3 | R-P250–R-P253 in dev-rules v1.6.0 | rules present with source+implementation |
| L-R3-4 | SIM-11 record in SIM-PROTOCOL.md | row + §9 present |
| L-R3-5 | BLUEPRINT v1.2.2 changelog | line present |

Deferred (recorded): gateway cascade/router code (P-task, needs calibration data);
telemetry gap-table completion (P-task); memory checklist enforcement (P2 with coverage gate);
vet/Scorecard still P3 (R2).

---

*End of Absorption Round 3*
