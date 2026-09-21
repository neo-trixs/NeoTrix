# NeoTrix Standard Templates (新规则模版套件)

> Effective 2026-09-21 (SIM-14). All new work MUST start from these templates.
> No hand-rolled formats for the four covered artifacts.

| Template | Use When | Rule |
|----------|----------|------|
| [SIM-RECORD-TEMPLATE.md](SIM-RECORD-TEMPLATE.md) | Any SIM-triggered task (R-P241 triggers) | R-P241–R-P245 |
| [MODULE-DOC-TEMPLATE.md](MODULE-DOC-TEMPLATE.md) | New module or module README rewrite | NTS-B09, R-P236 |
| [FITNESS-FN-TEMPLATE.rs](FITNESS-FN-TEMPLATE.rs) | New architecture fitness function (paste into `nt_core_arch_fitness.rs`, never compile standalone) | NTS-B05/B06, R-P246 |
| `docs/adr/0000-template.md` | Any architecturally significant decision (referenced, not duplicated) | NTS-B12, R-P249 |

Adding a fifth template requires an ADR (format proliferation is how standards die).
