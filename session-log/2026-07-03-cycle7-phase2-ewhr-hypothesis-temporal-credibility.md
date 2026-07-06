# Session Log — 2026-07-03 Cycle 7 Phase 2: EWHR Hypothesis/Temporal/Credibility Subsystems

## Goal
Extend the Evidence-Weighted Hypothesis Repository (EWHR) with three new subsystems:
1. **Formal Hypothesis Management** — Bayesian updating, subjective logic, audit trail
2. **Source Credibility Analysis** — tier-based scoring, trust propagation, multi-source aggregation
3. **Temporal Reasoning** — Allen interval algebra, anachronism detection, timeline reconstruction, evidence trend tracking

## Done
- **`nt_evidence_hypothesis.rs`** (NEW, 318 lines, 15 tests ✅):
  - `Hypothesis` struct with Bayesian update (`bayesian_update()`, `update_with_evidence()`), lifecycle status (`Proposed→Evaluating→Supported/Refuted/Inconclusive/Superseded`)
  - `SubjectiveOpinion` with cumulative fusion (`belief*uncertainty` product rule), averaging fusion, projected probability
  - `weight_of_evidence()` — log-odds ratio for evidence strength
  - `dempster_shafer_combine()` — mass function combination with conflict normalization
  - `AuditTrail` — versioned audit log with `record()`, `history()`, `recent()` for all evidence/hypothesis mutations
  - `HypothesisNetwork` — container with `propose_hypothesis()`, `find_strongest_supported()`, `find_strongest_refuted()`

- **`nt_evidence_credibility.rs`** (NEW, 248 lines, 10 tests ✅):
  - `SourceTier` (Primary/Secondary/Tertiary/Hearsay/Anonymous) with weight scores
  - `ReviewStatus` (PeerReviewed/Preprint/Conference/SelfPublished/Unreviewed) with weight scores
  - `CustodyChain` — chain-of-custody integrity scoring with depth/gap penalties
  - `SourceCredibility` — 9-factor composite score (tier, review, reputation, institutional, citations, custody, temporal proximity, independence, cross-validation)
  - `CredibilityAggregator` — weighted arithmetic, geometric mean, diversity score, trust propagation (PageRank-style)
  - `Bayesian Truth Serum (BTS)` signal — `bts_signal()` for empirical vs predicted frequency comparison

- **`nt_evidence_temporal.rs`** (NEW, 274 lines, 20 tests ✅):
  - Full Allen's interval algebra — `allen_relation()` with all 13 relations (Before/After/During/Contains/Overlaps/Meets/MetBy/Starts/StartedBy/Finishes/FinishedBy/Equal)
  - `AnachronismDetector` — entity timeline registration, anachronism check (pre-birth/post-death), event pair consistency
  - `TemporalEvidenceTracker` — value-over-time recording, linear regression slope trend analysis, recent trend (last-3 vs earlier), evidence consistency (coefficient of variation)
  - `TimelineReconstructor` — sorted event list, gap detection (>24h gaps), time span

- **`nt_evidence_api.rs`** (MOD, expanded 30→36 endpoints):
  - Hypothesis: `POST /api/ewhr/hypothesis/propose`, `GET /api/ewhr/hypothesis/{id}`, `POST /api/ewhr/hypothesis/{id}/update`, `GET /api/ewhr/hypothesis/list`, `GET /api/ewhr/hypothesis/strongest`
  - Credibility: `POST /api/ewhr/credibility/add`, `GET /api/ewhr/credibility/aggregate`, `GET /api/ewhr/credibility/geometric`, `GET /api/ewhr/credibility/diversity`, `POST /api/ewhr/credibility/trust_propagation`
  - Temporal: `POST /api/ewhr/temporal/record`, `GET /api/ewhr/temporal/trend/{id}`, `GET /api/ewhr/temporal/consistency/{id}`, `POST /api/ewhr/temporal/timeline/add`, `GET /api/ewhr/temporal/timeline/sorted`, `GET /api/ewhr/temporal/timeline/gaps`, `POST /api/ewhr/temporal/anachronism/check`, `POST /api/ewhr/temporal/anachronism/register`, `GET /api/ewhr/temporal/allen/{a_start}/{a_end}/{b_start}/{b_end}`
  - Opinion: `POST /api/ewhr/opinion/fuse`, `POST /api/ewhr/opinion/average`
  - Audit: `GET /api/ewhr/audit/{id}`, `GET /api/ewhr/audit/recent/{n}`

- **`mod.rs`** (MOD) — registered all 3 new modules + public re-exports

## Key Design Decisions
- **Bayesian updates**: Log-odds form with Bayes factor, clamped to [0.001, 0.999]. Prior/posterior tracked separately.
- **Status thresholds**: posterior > 0.85 → Supported, < 0.15 → Refuted, total weight > 5.0 → Inconclusive, else Evaluating.
- **Subjective Logic fusion**: Cumulative fusion uses `b*d_u + u*b` product rule (Josang 2016). Averaging fusion is simple mean.
- **Credibility weights**: 35% tier, 15% review, 15% author, 10% institutional, 8% citations, 10% custody, 5%×3 for temporal/independence/cross-validation.
- **Custody integrity**: `base - gap_penalty` × `1/depth` factor. 1-hop fully documented = 0.95.
- **Allen's algebra**: 12-clause match on start/end comparisons — covers all 13 classical interval relations.
- **Trend analysis**: Simple linear regression of value vs timestamp, with 3-point trailing window for recent trend detection.

## Build Status
- `cargo check --lib`: **0 errors**
- Hypotheses tests: **15/15 passed**
- Credibility tests: **10/10 passed**
- Temporal tests: **20/20 passed**
- Combined new tests: **45/45 passed**

## Files Changed
| File | Change |
|------|--------|
| `neotrix-core/src/neotrix/l3_memory_impl/nt_memory_historian/nt_evidence_hypothesis.rs` | **NEW** — Hypothesis management + 15 tests |
| `neotrix-core/src/neotrix/l3_memory_impl/nt_memory_historian/nt_evidence_credibility.rs` | **NEW** — Source credibility + 10 tests |
| `neotrix-core/src/neotrix/l3_memory_impl/nt_memory_historian/nt_evidence_temporal.rs` | **NEW** — Temporal reasoning + 20 tests |
| `neotrix-core/src/neotrix/l3_memory_impl/nt_memory_historian/nt_evidence_api.rs` | **MOD** — 30→36 REST endpoints |
| `neotrix-core/src/neotrix/l3_memory_impl/nt_memory_historian/mod.rs` | **MOD** — 3 new module registrations + re-exports |
| `session-log/2026-07-03-cycle7-phase2-ewhr-hypothesis-temporal-credibility.md` | **NEW** — This log |
