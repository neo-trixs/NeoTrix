---
name: "diagnose"
description: "Disciplined debugging loop — reproduce, minimise, hypothesise, instrument, fix, regression-test"
version: "1.0.0"
author: "NeoTrix (adapted from mattpocock/skills)"
triggers: debug, diagnose, bug, error, fix
---

# Diagnose — Debugging Protocol

A disciplined 6-step loop for finding and fixing bugs. Resist the urge to jump straight to fixing.

## Process

### 1. Reproduce
- Get a reliable, repeatable reproduction
- Document the exact steps, inputs, and environment
- If you cannot reproduce it, you cannot fix it

### 2. Minimise
- Strip away everything not related to the bug
- Reduce input data to the minimum that still triggers it
- Binary search: comment out half the code, see if bug persists, repeat

### 3. Hypothesise
- Form a specific hypothesis: "The bug is caused by X"
- State the mechanism: "Because when X happens, Y is null instead of a string"
- A good hypothesis predicts a testable observation

### 4. Instrument
- Add logging, assertions, or a debugger at the suspected point
- Print the actual values, not just "got here"
- Test your hypothesis: does the evidence support or refute it?
- If refuted, return to step 3 with a new hypothesis

### 5. Fix
- Write the minimal change that resolves the root cause
- Do not fix unrelated issues in the same change
- Run the reproduction to confirm it no longer occurs

### 6. Regression-Test
- Add a test that would have caught this bug
- Run the full test suite — did the fix break anything?
- Consider: could this bug exist elsewhere in the codebase?

## Rules

- **No fix before diagnosis** — if you don't know the root cause, you're guessing
- **One hypothesis at a time** — test them individually
- **Blame the code, not yourself** — the bug is in the program, not in you
- **If stuck, explain it to someone (or something)** — rubber duck debugging works

## When To Use

- Any unexpected behaviour, crash, or test failure
- Performance regressions
- Intermittent/flaky failures (hardest — need extra rigour on reproduction)
