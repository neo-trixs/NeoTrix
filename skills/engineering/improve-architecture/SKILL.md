---
name: "improve-architecture"
description: "Rescue ball-of-mud codebases — find deepening opportunities and inform from domain language"
version: "1.0.0"
author: "NeoTrix (adapted from mattpocock/skills)"
triggers: refactor, architecture, cleanup, tech-debt
---

# Improve Architecture

Systematic approach to rescuing a tangled codebase. Find deepening opportunities and let the domain language guide the structure.

## Process

### 1. Assess the Current State
- Identify the biggest pain points: slow tests, long files, circular dependencies, God objects
- Measure: file sizes, module coupling, test coverage, build times
- Catalog anti-patterns: copy-paste code, shotgun changes, inappropriate intimacy

### 2. Listen to the Domain Language
- Talk to domain experts or read the product spec
- What terms keep coming up? Those are candidate modules.
- The domain should suggest the structure — code organisation should mirror business concepts
- Extract domain terms that are currently implicit (stringly-typed, scattered in comments)

### 3. Find Deepening Opportunities
- **Extract Module**: group related functions/types into a new module when you notice a natural boundary
- **Stratify Layer**: separate concerns (e.g., pull data access out of business logic)
- **Unify Duplication**: three instances of similar code → extract once, not zero
- **Define Interface**: replace implicit contracts (convention/comments) with explicit types/traits
- **Isolate Side Effects**: push I/O, networking, and mutation to the edges

### 4. Pick the Highest-ROI Change
- What gives the most improvement for the least risk?
- Prefer mechanical refactors the compiler can verify (rename, extract, move)
- Defer changes that require deep domain rethinking

### 5. Execute Safely
- One structural change at a time
- Keep tests green between each change
- If a refactor requires changing tests, the architecture is working — tests should be specifying behaviour, not structure

## Principles

- **Strangler Fig pattern**: build the new structure alongside the old, route traffic gradually
- **Tell, don't ask**: push behaviour to where the data lives
- **Open/closed**: modules should be open for extension, closed for modification
- **Dependency inversion**: depend on abstractions, not concretions

## When To Use

- Starting work on an unfamiliar codebase
- When every change requires touching 5 files
- When tests are brittle and slow
- When the same concept is implemented differently in 3 places
