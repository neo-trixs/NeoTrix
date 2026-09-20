---
name: design-audit
description: Use when reviewing UI quality, auditing design systems, checking accessibility, or detecting anti-patterns. Consolidates ui-skill/audit + agentic-design-system/review + ui-reasoning anti-pattern dimensions. 10-dimension evaluation engine.
---

# Design Audit Engine

## Purpose

Unified design quality evaluation across 10 reasoning dimensions. Consolidates the fragmented audit/review capabilities from ui-skill, agentic-design-system, and ui-reasoning into a single structured audit workflow.

## When to Use

- Reviewing visual quality of existing UI
- Auditing design system consistency
- Checking accessibility compliance
- Detecting anti-patterns before release
- Validating responsive behavior

## 10-Dimension Audit

### D1: VI Consistency (from visual-taste-lab)
- Logo/wordmark usage consistent
- Color roles match defined palette
- Type attitude matches brand
- Geometry language consistent

### D2: Design System Adherence
- All components use tokens (no hardcoded values)
- CSS custom properties mapped to Tailwind
- All 4 interactive states defined per component

### D3: Spacing & Grid (8pt)
- All spacing uses 8pt multiples
- Component boundaries use isolation (surface shift, whitespace, divider, color fill)
- No stacked same-surface sections without separation

### D4: WCAG Accessibility
- Color pairs: 4.5:1 (body), 3:1 (large text)
- Visible focus indicators on all interactive elements
- ARIA labels and roles present
- Touch targets min 44x44px
- Reduced-motion alternative for animations
- Semantic HTML structure

### D5: Visual Hierarchy
- Single primary CTA per view
- Text hierarchy: 3 levels (heading, body, caption)
- Accent color restraint: CTAs, active states, links only

### D6: Anti-Pattern Detection
- No purple/blue AI gradients
- No floating cards without information boundaries
- No mixed visual languages
- No random animated decoration
- No stacked sections without separation

### D7: Responsive Behavior
- 375px (mobile): content reflows, no horizontal overflow
- 768px (tablet): layout adapts, sidebar collapses
- 1024px (desktop): full layout
- 1440px (wide): max-width enforced

### D8: State Coverage (9 states per screen)
- Loading, Empty, Error, Success, Inactive
- Partial, Stale, Offline, First-use

### D9: Industry Fit
- Style appropriate for domain (fintech ≠ gaming ≠ healthcare)
- Trust signals present for institutional contexts
- Conversion elements present for product contexts

### D10: Motion Quality
- Motion is subtle and structural
- No random decoration
- Reduced-motion alternative exists
- Consistent timing across components

## Audit Workflow

1. **Scan** → Run all 10 dimensions against target
2. **Score** → Each dimension: Pass/Warning/Fail + evidence (file:line)
3. **Rank** → Sort issues by severity (Fail > Warning) and impact
4. **Report** → Structured output with fix recommendations
5. **Re-audit** → After fixes, re-run failed dimensions

## Output Contract

```
## Design Audit Report

### Summary
- Total dimensions: 10
- Pass: X | Warning: Y | Fail: Z
- Score: X/10

### Issues (ranked by severity)
1. [FAIL] D4:WCAG - Contrast ratio 3.2:1 below 4.5:1
   - File: src/components/Button.tsx:42
   - Fix: Change text color from #666 to #555
2. [WARN] D3:Grid - Spacing 12px not on 8pt grid
   - File: src/components/Card.tsx:18
   - Fix: Change padding from 12px to 16px
...
```

## Integration with ReasoningBrain

Maps to capability dimensions:
- `verification` (audit pass/fail decisions)
- `quality_gates` (blocking vs advisory)
- `analysis` (multi-dimensional evaluation)
- `synthesis` (cross-dimension pattern detection)
