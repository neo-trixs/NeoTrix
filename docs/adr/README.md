# NeoTrix Architecture Decision Records

> Format: MADR 4.x + NeoTrix extensions (`sim-id`, `quality-attributes`, `requirements`,
> `supersedes`/`superseded-by`). Template: [0000-template.md](0000-template.md).
> Rules: R-P244 (Spike-first-ADR-second), R-P249 (ADR lint standard).
> ADRs are immutable — only `status` changes; supersession is bidirectional.

## Index

| ADR | Title | Status | Date | SIM |
|-----|-------|--------|------|-----|
| [0000](0000-template.md) | Template (do not use directly, copy it) | — | — | — |

## Conventions

1. Numbering: sequential `NNNN-short-title.md`. Check this index before creating (no tooling yet — manual, R-P249).
2. Status enum: `proposed | accepted | rejected | deprecated | superseded`.
3. Dates: ISO-8601 (`YYYY-MM-DD`).
4. Every ADR links its SIM-ID when a SIM trigger applied; every SIM-triggered ADR cites evidence, not preference.
5. Supersession must be bidirectional: A.supersedes=[B] ⟺ B.superseded-by=[A]. Stale pointers fail review.
