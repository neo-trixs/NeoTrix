---
name: design-core
description: Use when building UI, designing components, or generating design tokens. Consolidates architect/build/theme/motion modes from ui-skill with 3-pass build validation from agentic-design-system. Single source for component implementation.
---

# Design Core

## Purpose

Unified component implementation skill — architect layout, build components, generate tokens, apply motion. Replaces the fragmented ui-skill + agentic-design-system/build overlap with a single coherent workflow.

## When to Use

- Building any UI component, page, or layout
- Generating design tokens, color palettes, or typography scales
- Applying animations or micro-interactions
- Before writing any UI code — always run Pre-Flight Check

## Pre-Flight Check (MANDATORY)

Before writing any UI code, verify:

- [ ] Design system loaded (tokens, colors, typography defined)
- [ ] No hardcoded hex values in existing components
- [ ] Responsive strategy decided (375px, 768px, 1024px, 1440px)
- [ ] Accessibility baseline set (WCAG AA)

## Workflow

### Step 1: Architect — Layout Planning

1. Understand requirements → component, page, or layout
2. Choose layout primitive → sidebar-main, dashboard-grid, centered-content, full-bleed
3. Define information hierarchy → primary → secondary → tertiary
4. Specify responsive collapse strategy per breakpoint
5. Output annotated layout plan → get approval before JSX

### Step 2: Build — Component Implementation

1. Run Pre-Flight Check (above)
2. Use only design tokens — no hardcoded values
3. Define all 4 interactive states: hover, active, focus-visible, disabled
4. Accept className prop for external overrides (shadcn pattern)
5. Include skeleton loaders and empty states

### Step 3: Theme — Token System

1. Generate full token set: 3 backgrounds, 3 text colors, secondary + hover/active
2. Add semantic colors: success, warning, error, info
3. Create CSS custom properties in `:root` and `.dark`
4. Map to Tailwind config extension
5. Provide dark mode variants for all tokens

### Step 4: Motion — Animations

1. Choose motion type: Framer Motion (React) or CSS transitions
2. Implement patterns: fade-in, stagger, spring modal, drawer, skeleton, counter
3. Ensure `@media (prefers-reduced-motion: reduce)` support
4. Keep motion subtle and structural — no random decoration

### Step 5: Build Validation (3-pass)

After implementation, run:

**Pass 1: design-review**
- Anti-patterns detected and removed
- Visual hierarchy established (primary → muted → faint)
- Spacing tightness checked (8pt grid)
- Product-fit validated

**Pass 2: ux-baseline-check**
- Loading states for all async components
- Empty states defined
- Error states defined
- Edge cases covered

**Pass 3: ui-polish-pass**
- Spacing tightened, alignment checked
- Visual finish improved (shadows, borders, radius)
- WCAG contrast check (4.5:1 minimum)
- Responsive behavior verified

## Output Contract

- Layout plan (annotated)
- Components with tokens, 4 states, className prop
- Token files: design-tokens.css, tailwind.config.js, theme.md
- Validation report: Pass/Fail per pass, final score X/10

## Anti-Patterns (NEVER)

- Purple/blue AI gradients
- Floating cards without clear information boundaries
- Mixed visual languages in same view
- Hardcoded hex values
- Motion without reduced-motion alternative
- Stacked same-surface sections without separation
