---
name: ui-skill
description: Use when building UI, designing components, or generating design tokens. Provides 5 modes (architect, build, theme, motion, audit) with Pre-Flight Check before writing any UI code.
---

# UI Skill#

## Purpose#

Provide 5 specialized modes for UI work, each with structured workflow and Pre-Flight Check. This skill distills best practices from ui-skill project into a concise framework for NeoTrix ReasoningBrain.#

## When to Use#

- Building any UI component, page, or design system#
- Need to generate design tokens, color palettes, or typography scales#
- Auditing existing UI for anti-patterns or inconsistencies#
- Before writing any UI code — always run Pre-Flight Check first#

## 5 Modes#

### 1. architect — Layout & UX planning#

**Rule**: Plan layout before building. Define information hierarchy and responsive collapse strategy.#

**Workflow**:
1. **Understand requirements** → What to build (component, page, layout)#
2. **Choose layout primitive** → sidebar-main, dashboard-grid, centered-content, full-bleed#
3. **Define information hierarchy** → What is most important on screen#
4. **Specify responsive collapse strategy** → What happens at each breakpoint#
5. **Output short annotated layout plan** → Ask for approval before writing JSX#

**Check**:
- [ ] Layout primitive chosen (sidebar-main, dashboard-grid, centered-content, full-bleed)#
- [ ] Information hierarchy defined (primary → secondary → tertiary)#
- [ ] Responsive collapse strategy specified for 375px, 768px, 1024px, 1440px#
- [ ] Approval obtained before writing code#

### 2. build — Component implementation#

**Rule**: Build atomic, token-driven components with all 4 interactive states. Pre-Flight Check must pass first.#

**Workflow**:
1. **Run Pre-Flight Check** (see below)#
2. **Use only design tokens** — no hardcoded hex values#
3. **Define all 4 interactive states** — hover, active, focus-visible, disabled#
4. **Accept className prop** for external overrides (shadcn pattern)#
5. **Include skeleton loaders and empty states** — always included unless pure primitive#

**Check**:
- [ ] Pre-Flight Check passed#
- [ ] No hardcoded hex values in component#
- [ ] All 4 states defined: hover, active, focus-visible, disabled#
- [ ] className prop accepted for overrides#
- [ ] Skeleton loaders and empty states included#

### 3. theme — Design token systems#

**Rule**: Generate complete token set with dark mode variants and Tailwind config mapping.#

**Workflow**:
1. **Generate full token set** — 3 backgrounds, 3 text colors, secondary colors with hover/active#
2. **Add semantic colours** — success, warning, error, info#
3. **Create CSS custom properties** in `:root` and `.dark`#
4. **Map to Tailwind config extension** — ensure all tokens available in Tailwind#
5. **Provide dark mode variants** — all tokens must work in both modes#

**Output**:
- `design-tokens.css` — canonical token source#
- `tailwind.config.js` — Tailwind token mapping#
- `theme.md` — documentation#

### 4. motion — Animations & micro-interactions#

**Rule**: Use Framer Motion for React, CSS transitions for others. Always include reduced-motion alternative.#

**Workflow**:
1. **Choose motion type** — Framer Motion (React) or CSS transitions#
2. **Implement motion patterns** — fade-in, stagger, spring modal, drawer, skeleton, number counter#
3. **Ensure reduced-motion** — `@media (prefers-reduced-motion: reduce)` supported#
4. **Make motion subtle and structural** — avoid random animated decoration#

**Check**:
- [ ] Reduced-motion alternative provided#
- [ ] Motion is subtle and structural, not decorative#
- [ ] No random animated decoration#

### 5. audit — Visual quality review#

**Rule**: Score UI across 7 categories and return ranked, actionable fixes.#

**Workflow**:
1. **Score across 7 categories** — typography, color, spacing, hierarchy, accessibility, responsiveness, motion, overall#
2. **Rank fixes by priority** — critical, high, medium, low#
3. **Return actionable fixes** — each with specific code change#

**Output**:
- Score: X/10 (overall)#
- Critical fixes: list#
- High priority fixes: list#
- Medium priority fixes: list#

## Pre-Flight Check#

**Run BEFORE writing any code**:

1. **Design system check** — Looks for `DESIGN.md`, `design-tokens.css`, or `tailwind.config.js`. If found, reads it and honors every token decision. If not found, proposes a minimal token set (4 brand colours, 1 type scale, 4 spacing steps) before building anything.#

2. **Accessibility check** — contrast meets WCAG AA (4.5:1 for text, 3:1 for large text). All interactive elements have visible focus indicators. Semantic HTML structure used. Touch targets min 44×44px.#

3. **Responsive strategy check** — Defines responsive collapse strategy at each breakpoint before implementation. Breakpoints: 375px, 768px, 1024px, 1440px. Mobile layout avoids horizontal overflow.#

## Output Contract#

For each mode, deliver:

- **architect** → annotated layout plan with approval#
- **build** → token-driven component with 4 states, skeleton, empty state#
- **theme** → `design-tokens.css`, `tailwind.config.js`, `theme.md`#
- **motion** → motion implementation with reduced-motion support#
- **audit** → score + ranked fixes#

## Integration with ReasoningBrain#

This skill maps to NeoTrix ReasoningBrain capabilities:

- **KnowledgeSource::UISkill** → 5 modes + Pre-Flight Check#
- **CapabilityVector** extensions:#
  - `experimental: f64` → Mode selection (architect, build, theme, motion, audit)#
  - `whitespace: f64` → Pre-Flight Check (design system, accessibility, responsive)#
  - `tailwind_proficiency: f64` → Token-driven components, Tailwind config mapping#
  - `accessibility: f64` → WCAG AA, focus indicators, semantic HTML#
