# Module: nt_[name]

> One primary responsibility (NTS-B09). Public entry points use real signatures —
> CI imports every documented symbol (R-P232 anti-drift).

**Layer**: L[0-6]
**Primary Responsibility**: [one sentence]

## Entry Points

| Function | Signature | Input → Output |
|----------|-----------|----------------|
| `function_name` | `fn name(arg: Type) -> Result<Output>` | `InputType → OutputType` |

## Dependencies

**Imports** (what this module uses):

- `[layer]::[module]` — [why]

**Dependents** (what uses this module):

- `[layer]::[module]` — [why]

## Quality Attributes

| Attribute (ISO 25010) | This Module's Role |
|-----------------------|-------------------|
| [Security/Performance/…] | [validation/logging/caching/…] |

## Related ADRs

- [ADR-NNNN: title](../adr/NNNN-short-title.md)

## Threat Vectors (if external input is handled, else delete)

| Vector | Mitigation (code ref) | Test (file ref) | Residual Risk |
|--------|----------------------|----------------|---------------|
| [e.g., prompt injection] | [sanitize fn] | [test file] | [low/accepted] |
