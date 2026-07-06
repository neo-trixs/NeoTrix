---
name: "grill-me"
description: "Structured questioning session before any task begins — purpose, constraints, alternatives, edge cases"
version: "1.0.0"
author: "NeoTrix (adapted from mattpocock/skills)"
triggers: grill, plan, design, architect
---

# Grill Me

Before starting any non-trivial task, run a structured questioning session to surface hidden assumptions, constraints, and edge cases.

## Process

### 1. Purpose
- What exactly are we trying to achieve?
- What does success look like? How is it measured?
- Who is the end user / stakeholder?

### 2. Constraints
- What are the hard constraints (time, budget, tech, platform)?
- What are the soft constraints (preferences, conventions)?
- What must NOT be changed?

### 3. Alternatives
- What are 2-3 alternative approaches?
- Why were they rejected in favour of this one?
- What assumptions do we hold that could invalidate our preferred approach?

### 4. Edge Cases
- What happens when input is empty / malformed / extreme?
- What happens when a dependency fails?
- What happens at scale (data size, user count, concurrency)?
- What happens with error/retry/degraded modes?

### 5. Verification
- How will we know it works? (test, demo, metric)
- What could go wrong that we haven't considered?
- What's the rollback plan?

## Activation

Use this skill whenever the task involves ambiguity, trade-offs, or significant effort. The output is a shared mental model documented as decisions and open questions.
