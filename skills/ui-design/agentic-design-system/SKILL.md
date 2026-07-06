---
name: agentic-design-system
description: Use when building UI, components, or design systems. Provides 3-pass evaluation loop (design-review, ux-baseline-check, ui-polish-pass) distilled from aa-on-ai/agentic-design-system.
---

# Agentic Design System

## Purpose

Provide structured 3-pass evaluation loop for UI work. This skill distills the agentic-design-system project into a concise evaluation framework for NeoTrix ReasoningBrain.

## When to Use:

- Building any UI component, page, or design system
- Reviewing visual quality, accessibility, or responsive behavior
- After generating UI code, before presenting to user
- When the agent tends to skip verification steps

## Core Rule:

**Build first, then run structured critique passes.** Never present unverified UI output.

The evaluation loop runs three passes:

| Pass | What it checks | When it runs |
| --- | --- | --- |
| **design-review** | Anti-patterns, hierarchy, spacing, product-fit (11 reference files) | Always, for any visual work |
| **ux-baseline-check** | Loading, empty, error, edge-case states | Always, for any visual work |
| **ui-polish-pass** | Spacing tightness, alignment, visual finish | Always, as the final step |

## Evaluation Loop Workflow:

### Pass 1: design-review

**Criteria**:
- [ ] Anti-patterns detected and removed (purple/blue gradients, floating cards, mixed visual languages)
- [ ] Visual hierarchy established (primary → muted → faint)
- [ ] Spacing tightness checked (8pt grid)
- [ ] Product-fit validated (industry-appropriate style)
- [ ] Company sites foreground organization, not just product
- [ ] Product sites foreground value, proof, conversion

**Action**: If fails, fix issues and re-run this pass.

### Pass 2: ux-baseline-check

**Criteria**:
- [ ] Loading states defined for all async components
- [ ] Empty states defined (no data, no results)
- [ ] Error states defined (network error, validation error)
- [ ] Edge cases covered (9 states per screen)
- [ ] State inventory complete (loading, empty, error, success, inactive, etc.)

**Action**: If fails, add missing states and re-run this pass.

### Pass 3: ui-polish-pass

**Criteria**:
- [ ] Spacing tightened (alignment checked)
- [ ] Visual finish improved (shadows, borders, radius)
- [ ] Motion subtle and structural (no random decoration)
- [ ] Final WCAG contrast check (4.5:1 minimum)
- [ ] Responsive behavior verified (375px, 768px, 1024px, 1440px)

**Action**: If fails, tighten alignment/spacing and re-run this pass.

## Output Contract:

After completing all 3 passes, report:

- Pass 1: design-review → Pass/Fail + issues fixed
- Pass 2: ux-baseline-check → Pass/Fail + states added
- Pass 3: ui-polish-pass → Pass/Fail + polish applied
- Final UI score: X/10
- Changed files: list

## Integration with ReasoningBrain:

This skill maps to NeoTrix ReasoningBrain capabilities:

- **KnowledgeSource::AgenticDesign** → 3-pass evaluation loop
- **CapabilityVector dimensions**:
  - `verification: f64` → Pass 1 + Pass 3
  - `quality_gates: f64` → Pass 2
  - `analysis: f64` → design-review criteria
  - `synthesis: f64` → applying fixes

## Anti-Patterns Detected:

- Skipping evaluation loop (presenting unverified UI)
- Running only 1 pass instead of 3
- Fixing issues without re-running the failed pass
- Claiming "looks good" without evidence
- Ignoring industry-appropriate styles (e.g., using SaaS style for banking)

## Example Usage:

**User**: "Build me a dashboard showing agent uptime"

**Agent workflow**:
1. Build first draft of dashboard
2. Run **design-review** → fix anti-patterns, establish hierarchy
3. Run **ux-baseline-check** → add loading/error states
4. Run **ui-polish-pass** → tighten spacing, improve finish
5. Report: "All 3 passes passed. Final UI score: 8/10. Changed files: Dashboard.tsx, useDashboard.ts"
