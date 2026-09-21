# ADR-NNNN: [Short Decision Title]

> Copy this file to `NNNN-short-title.md` (next sequential number, see README index).
> Format: MADR 4.x + NeoTrix extensions (SIM-ID, quality-attributes, requirements).
> Lint bar: madr-lint recommended rules (required-sections, status-enum, date-iso8601,
> filename-format, no-broken-links, no-duplicate-numbering, supersedes-bidirectional).

---
status: proposed
date: YYYY-MM-DD
decision-makers: [name]
consulted: [name]
informed: [team]
sim-id: SIM-NN (required if a SIM trigger applied, else omit)
quality-attributes: [security, reliability] (ISO/IEC 25010:2023, max 3, primary drivers only)
requirements: [QR-001] (optional, TOGAF traceability)
supersedes: [] (ADR numbers this replaces, if any)
superseded-by: [] (filled by the replacing ADR, never edit history otherwise)
---

## Context and Problem Statement

[What is the issue motivating this decision? What forces are at play —
technical, security, schedule, team? Link the SIM record if one exists.]

## Decision Drivers

- [driver 1, e.g., "L1→L5越层必须消除，P1-02"]
- [driver 2, e.g., "SDB无Verifier路径 = BLOCKER"]

## Considered Options

- [Option 1]
- [Option 2]
- [Option 3]

## Decision Outcome

Chosen option: **[option]**, because [justification anchored in SIM evidence, not preference].

### Consequences

- Good: [positive consequence]
- Bad: [accepted downside — later engineers must not "fix" this without reading here]
- Neutral: [notes]

## Verification

- [ ] Fitness/script gate: [which gate proves this, e.g., `bash scripts/check-layer-deps.sh`]
- [ ] Test: [which test file covers it]
- [ ] Metric: [which dashboard cell moves, from→to]

## Links

- SIM: [SIM-NN](../architecture/SIM-PROTOCOL.md) (if any)
- Blueprint: [D-NN](../architecture/NEOTRIX-MASTER-BLUEPRINT.md) (if any)
- Code: [path/to/module](../../neotrix-core/src/...) (if any)
