# SKILL-SPEC.md — NT-* Skill Interface Contract

> Derived from Easel's SKILL.md pattern (2026-09-08, 8-Source Batch Absorption).
> All NT-* skill implementations MUST follow this contract.

## Contract: SKILL-SPEC.md

Every skill node in the NT-* domain system must provide:

```
skill-<name>/
├── SKILL-SPEC.md      # This file: execution contract (<200 lines)
├── references/         # Domain knowledge (loaded on-demand)
├── scripts/            # Executable code (called at runtime)
└── tests/              # Prompt + expected output pairs
```

### SKILL-SPEC.md Structure

```markdown
# Skill: <name>

## Identity
- **Domain**: NT-* (which faction)
- **Tier**: Small Passive | Notable Passive | Keystone
- **Constellation**: C0-C6 maturity level
- **Aliases**: Legacy names (if any)

## Purpose
<1-3 sentences: what this skill does>

## Input Contract
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| ... | ... | ... | ... |

## Output Contract
| Field | Type | Description |
|-------|------|-------------|
| ... | ... | ... |

## Execution Steps
1. <Step 1>
2. <Step 2>
...

## Dependencies
- **Requires**: Other skills/modules needed
- **Produces**: Artifacts written to outputs/
- **Consumes**: Artifacts read from inputs/

## Failure Modes
| Mode | Detection | Recovery |
|------|-----------|----------|
| ... | ... | ... |

## References
- <path to domain knowledge files>
```

### Rules

1. **SKILL-SPEC.md < 200 lines** — prevents bloat, forces clarity
2. **references/ loaded on-demand** — not preloaded into context
3. **scripts/ are executable** — called at runtime, not imported
4. **tests/ are prompt+expected pairs** — for validation, not unit tests
5. **Stateless atoms** — no skill depends on another skill's runtime state
6. **Profile injected as prefix** — not global state (avoids concurrency issues)
7. **Manifest (.ntx-manifest.json)** is thin index — summary + outputs[] paths only

### Mapping to NeoTrix Tiers

| Easel Layer | NeoTrix Tier | Token Budget |
|-------------|-------------|--------------|
| Foundation (6 skills) | Small Passive | < 500 tokens |
| Discover + Plan (25) | Notable Passive | 500-2000 tokens |
| Create + Publish + Attribute (81) | Keystone | 2000-5000 tokens |

### Anti-Patterns

- ❌ SKILL-SPEC.md > 200 lines (split into references/)
- ❌ Skill depends on another skill's runtime state (use manifest)
- ❌ Skill produces chat responses instead of artifacts (Agent-as-Executor)
- ❌ No tests/ directory (Dark Forest: compile + test + connect or delete)
- ❌ Profile as global state (use message prefix injection)
