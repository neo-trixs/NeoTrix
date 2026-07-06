---
name: decantr-design
description: Use when building UI, creating design systems, or setting up design context. Provides 3-layer context (DECANTR.md, scafford.md, section-*.md) and Guard Rules (DNA guards, Blueprint guards) from decantr-ai.
---

# Decantr Design System#

## Purpose#

Provide structured 3-layer design context and guard rules for UI work. This skill distills decantr-ai's design intelligence into a framework for NeoTrix ReasoningBrain.#

## When to Use#

- Building any UI component, page, or design system#
- Setting up design context before implementation#
- Need to enforce theme consistency, layout coherence, visual personality#
- Want to prevent design drift with guard rules#

## Core Layers#

### 1. DECANTR.md (Design Rules)#
**Rule**: Theme consistency — tokens, colors, typography locked to your palette.#
**Check**:#
- [ ] Design rules defined: theme-mode compatibility, color roles, typography mood#  
- [ ] CSS atoms: spacing, borders, radius, shadows defined#  
- [ ] Motion Philosophy: subtle and structural, no random decoration#  
- [ ] Interactivity Philosophy: drag/drop, pan/zoom built by default#  
- [ ] Voice & copy: consistent tone, CTA verbs, error messages#

### 2. scafford.md (App Topology)#
**Rule**: Layout coherence — shell implementation specs, patterns, topology define the structure.#
**Check**:#
- [ ] App topology: route map, zone transitions defined#  
- [ ] Voice & copy: consistent across all pages#  
- [ ] Shared components: identified and reused#  
- [ ] Zone transitions: smooth, no jarring jumps#  
- [ ] Development mode: clear workflow defined#

### 3. section-*.md (Pattern Specs)#
**Rule**: Visual personality — each section has quick start summary, shell implementation, decorator table, token palette.#
**Check**:#
- [ ] Section dimensions: width, height, regions defined#  
- [ ] Anti-patterns: listed and avoided#  
- [ ] Spacing guide: 8pt grid enforced#  
- [ ] Decorator table: shadows, borders, radius specified#  
- [ ] Token palette: colors, typography, motion defined#  
- [ ] Visual direction: hero, data cards, content cards, form/CTA#  
- [ ] Pattern specs: composition algebra, motion, responsive, accessibility#

## Guard Rules#

### DNA Guards (Errors)#
**Rule**: Style, density, accessibility, theme-mode compatibility must be enforced.#
**Check**:#
- [ ] Style: no mixed visual languages across modules#  
- [ ] Density: appropriate for audience (institutional vs playful)#  
- [ ] Accessibility: WCAG AA mandatory everywhere#  
- [ ] Theme-mode compatibility: light/dark modes both supported#

### Blueprint Guards (Warnings)#
**Rule**: Structure, layout, pattern existence must be verified.#
**Check**:#
- [ ] Structure: shell implementation matches scafford.md#  
- [ ] Layout: responsive collapse strategy defined#  
- [ ] Pattern existence: all required patterns (hero, cards, forms) present#

## Workflow#

1. **Load Context** → Read DECANTR.md, scafford.md, section-*.md (3-layer context)#  
2. **Apply Guards** → Enforce DNA guards (errors) and Blueprint guards (warnings)#  
3. **Build Components** → Use tokens, follow pattern specs, respect spacing guide#  
4. **Check Drift** → Run `decantr_check_drift` to detect inconsistencies#  
5. **Accept/Resolve** → Accept drift by updating DECANTR.md, or resolve by scoping changes#

## Output Contract#

- Design context files: DECANTR.md, scafford.md, section-*.md#  
- Guard rules applied: DNA guards (errors) + Blueprint guards (warnings)#  
- Drift check results: `decantr_check_drift` output#  
- Updated components with consistent theme, layout, visual personality#

## Integration with ReasoningBrain#

This skill maps to NeoTrix ReasoningBrain capabilities:#

- **KnowledgeSource::DecantrDesign** → 3-layer context + Guard Rules#  
- **CapabilityVector** extensions:#  
  - `domain_specificity: f64` → Layer 1 (DECANTR.md)#  
  - `quality_gates: f64` → Layer 2 (scafford.md)#  
  - `verification: f64` → Layer 3 (section-*.md) + Guard Rules#  
