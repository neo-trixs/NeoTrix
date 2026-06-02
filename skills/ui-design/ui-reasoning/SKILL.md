---
name: ui-reasoning
description: Use when building UI, designing components, creating design systems, or reviewing visual quality. Provides 10 core reasoning dimensions distilled from 7 leading AI design agent projects (ui-ux-pro-max-skill, visual-taste-lab, agentic-design-system, ui-skill, decantr, layout, design-skills).
---

# UI Reasoning Engine

## Purpose

Provide structured UI design reasoning across 10 core dimensions. This skill distills best practices from 7 leading AI design agent projects into a single reasoning framework for NeoTrix ReasoningBrain.

## When to Use

- Building any UI component, page, or design system
- Reviewing visual quality, accessibility, or responsive behavior
- Generating design tokens, color palettes, or typography scales
- Auditing existing UI for anti-patterns or inconsistencies
- Creating brand-aligned visual identity systems

## Core Reasoning Dimensions

### 1. VI-First (visual-taste-lab + DesignSystem)

**Rule**: Never start by redesigning pages. First create or infer brand visual identity, then create design language, then apply it.

**Check**:
- [ ] Logo/wordmark identified (shape, rhythm, weight, color cues)
- [ ] Color roles defined: primary, secondary, accent, neutral, semantic
- [ ] Type attitude set: institutional, technical, editorial, commercial, playful, minimal
- [ ] Geometry defined: square, modest radius, rounded, modular, document-like
- [ ] Site type decided: company website, product website, transactional, institutional, dashboard, portfolio

### 2. Design System Priority (ui-skill + decantr + layout)

**Rule**: Generate or load design system before writing any UI code. All components must use tokens, not hardcoded values.

**Check**:
- [ ] `design-language.md` or `DECANTR.md` exists
- [ ] Design tokens defined: colors, typography, spacing, motion
- [ ] Tailwind config or CSS custom properties mapped to tokens
- [ ] No hardcoded hex values in components
- [ ] All 4 interactive states defined: hover, active, focus-visible, disabled

### 3. 8pt Grid + Spacing (DesignSystem + agentic-design-system)

**Rule**: Every margin, padding, and gap must land on 8pt grid (8, 16, 24, 32...). No arbitrary pixel values.

**Check**:
- [ ] All spacing uses 8pt multiples
- [ ] Component boundaries use isolation: surface shift, whitespace, divider, or color fill
- [ ] No stacked same-surface sections without separation

### 4. WCAG Contrast + Accessibility (all projects)

**Rule**: WCAG AA mandatory everywhere. Body text 4.5:1 minimum, large text 3:1 minimum.

**Check**:
- [ ] Color pairs tested: text color × background color
- [ ] Interactive elements have visible focus indicators
- [ ] ARIA labels and roles present in code
- [ ] Touch targets min 44×44px
- [ ] Reduced-motion alternative provided for animations
- [ ] Semantic HTML structure used

### 5. Visual Hierarchy (visual-taste-lab + ui-ux-pro-max-skill)

**Rule**: One primary action per view. Text flows through three levels: primary → muted → faint. Accent color used only for CTAs, active states, and links.

**Check**:
- [ ] Single primary CTA per view
- [ ] Text hierarchy: 3 levels (heading, body, caption/muted)
- [ ] Accent color restraint: CTAs, active states, links only
- [ ] Cards explain information boundaries, not fill space

### 6. Atomic Component Architecture (DesignSystem + ui-skill)

**Rule**: Atoms → Molecules → Organisms. No skipping levels. No hardcoding inside molecules or organisms.

**Check**:
- [ ] Components follow atomic design: atoms → molecules → organisms
- [ ] No skipped levels in component composition
- [ ] Shared components reused across pages
- [ ] Button, card, navigation, table, form have reusable rules

### 7. Anti-Pattern Detection (agentic-design-system + ui-ux-pro-max-skill)

**Rule**: Detect and eliminate common AI-generated UI anti-patterns.

**Anti-Patterns**:
- [ ] No purple/blue gradient hero by default
- [ ] No treating every company site like a SaaS landing page
- [ ] No ignoring existing logo, brand color, or industry expectation
- [ ] No changing colors module-by-module without palette system
- [ ] No every section as floating card
- [ ] No oversized cards nested inside cards
- [ ] No mixed visual languages across modules
- [ ] No fake case studies, partners, metrics, or screenshots
- [ ] No "高级一点" without tokens or references

### 8. Responsive Strategy (ui-skill + design-skills)

**Rule**: Define responsive collapse strategy at each breakpoint before implementation.

**Check**:
- [ ] Breakpoints defined: 375px, 768px, 1024px, 1440px
- [ ] Mobile layout avoids horizontal overflow
- [ ] Responsive collapse strategy specified for each component
- [ ] Touch-friendly interactions on mobile (min 44px targets)

### 9. Industry-Specific Reasoning (ui-ux-pro-max-skill 161 rules)

**Rule**: Match design approach to industry category with specialized rules.

**Industry Categories**:
- Tech & SaaS: conversion-optimized, hero-centric, feature-rich showcase
- Finance: trust & authority, dark mode optional, conservative colors
- Healthcare: accessible, calming colors, clear information hierarchy
- E-commerce: product-focused, trust signals, streamlined checkout
- Services: appointment-focused, clear contact paths, testimonials
- Emerging Tech: bold, gradient-when-appropriate, futuristic elements

**Check**:
- [ ] Industry category identified
- [ ] Matching style applied (67 styles available)
- [ ] Industry-appropriate anti-patterns avoided
- [ ] Color mood matches industry (161 palettes available)

### 10. Evaluation Loop (agentic-design-system 3-pass)

**Rule**: Run structured critique passes after generation: design-review → ux-baseline-check → ui-polish-pass.

**Pass 1: design-review**
- [ ] Anti-patterns detected and removed
- [ ] Visual hierarchy established (primary → muted → faint)
- [ ] Spacing tightness checked (8pt grid)
- [ ] Product-fit validated

**Pass 2: ux-baseline-check**
- [ ] Loading states defined
- [ ] Empty states defined
- [ ] Error states defined
- [ ] Edge cases covered (9 states per screen)

**Pass 3: ui-polish-pass**
- [ ] Spacing tightened (alignment checked)
- [ ] Visual finish improved (shadows, borders, radius)
- [ ] Motion subtle and structural
- [ ] Final WCAG contrast check

## Workflow

1. **Identify Project Type** → Run VI-First audit (Dimension 1)
2. **Load/Generate Design System** → Apply tokens (Dimension 2)
3. **Set Spacing Grid** → Enforce 8pt (Dimension 3)
4. **Build Components** → Atomic architecture (Dimension 6)
5. **Check Accessibility** → WCAG + ARIA (Dimension 4)
6. **Establish Hierarchy** → One primary action (Dimension 5)
7. **Detect Anti-Patterns** → Scan and fix (Dimension 7)
8. **Plan Responsive** → Breakpoints + collapse (Dimension 8)
9. **Apply Industry Rules** → Match category (Dimension 9)
10. **Run Evaluation Loop** → 3-pass critique (Dimension 10)

## Output Contract

- Design language document (design-language.md or DECANTR.md)
- Updated shared tokens and components
- 2-4 key pages redesigned
- Evaluation loop results (3 passes)
- Acceptance checklist completed

## Integration with ReasoningBrain

This skill maps to NeoTrix ReasoningBrain capabilities:

- **KnowledgeSource::UIUXPro** → Dimensions 7, 9 (161 rules, anti-patterns)
- **KnowledgeSource::VisualTasteLab** → Dimensions 1, 2 (VI-First workflow)
- **KnowledgeSource::AgenticDesign** → Dimensions 10 (3-pass evaluation)
- **CapabilityVector** extensions:
  - `vi_first: f32` → Dimension 1
  - `design_language: f32` → Dimension 2
  - `archetype_match: f32` → Dimension 9
  - `brand_consistency: f32` → Dimension 1, 5
  - `anti_pattern: f32` → Dimension 7
