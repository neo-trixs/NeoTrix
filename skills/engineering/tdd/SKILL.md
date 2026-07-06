---
name: "tdd"
description: "Red-Green-Refactor test-driven development loop — one vertical slice at a time"
version: "1.0.0"
author: "NeoTrix (adapted from mattpocock/skills)"
triggers: tdd, test, red-green-refactor
---

# TDD — Test-Driven Development

Follow the Red-Green-Refactor loop for every unit of work. Work in one vertical slice at a time.

## Process

### 🔴 Red — Write a failing test
- Write the smallest possible test that expresses the desired behaviour
- Test behaviour, not implementation
- Ensure the test fails for the right reason (compilation error ≠ test failure)

### 🟢 Green — Make it pass
- Write the simplest code that makes the test pass
- Duplication is OK. Hardcoded values are OK. This is not the final code.
- If it takes more than 2 minutes to get green, the slice is too big.

### 🔵 Refactor — Improve without changing behaviour
- Remove duplication, extract functions, rename variables
- Improve design while tests stay green
- Run the full test suite after each refactor step

## Good Test Heuristics

- **One concern per test** — name describes the scenario
- **Arrange-Act-Assert** — clear separation
- **FIRST principles**: Fast, Isolated, Repeatable, Self-validating, Timely
- **Test the public API** — not private internals
- **Prefer realistic inputs** — avoid mocks when real instances work
- **Cover edge cases**: empty, null, error, boundary, max

## One Vertical Slice At A Time

Build a single end-to-end slice before moving to the next:
1. Write a test for one scenario of one function
2. Implement just enough to pass
3. Refactor
4. Repeat with the next scenario

Do NOT write all tests upfront. Do NOT implement unrelated functionality.

## When To Use

- Any new feature or function
- Bug fixes (write a test that reproduces the bug first)
- Refactoring (tests are your safety net)
