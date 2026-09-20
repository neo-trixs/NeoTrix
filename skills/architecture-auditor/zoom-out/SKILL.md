---
name: "zoom-out"
description: "Understand code in system context — ask the agent to explain code in the context of the whole system"
version: "1.0.0"
author: "NeoTrix (adapted from mattpocock/skills)"
triggers: zoom, context, architecture, overview
---

# Zoom Out — System Context Analysis

Before modifying code, understand how it fits into the larger system. This skill forces the agent to step back and reason about the whole before the part.

## Process

### 1. Identify the Subsystem
- What module / file / function are we looking at?
- What is its primary responsibility?
- What contracts does it expose (public API, types, events)?

### 2. Map Dependencies
- What does this code depend on? (upstream)
- What depends on this code? (downstream)
- Are there implicit dependencies (global state, config, env)?

### 3. Trace the Data Flow
- Where does the input come from?
- Where does the output go?
- What transformations happen along the way?
- What side effects occur (I/O, state mutation, network calls)?

### 4. Identify Architectural Role
- Which layer does this belong to? (presentation, domain, infrastructure)
- Does it follow the project's architectural conventions?
- Is this the right place for this logic?

### 5. Assess Change Impact
- If we change this code, what else must change?
- What tests would need updating?
- Are there deployment considerations (migrations, feature flags)?

## Output

A concise summary covering:
- **Responsibility**: what this code does in one sentence
- **Context**: where it sits in the system
- **Data flow**: input → transform → output
- **Impact radius**: what breaks if this changes

## When To Use

- Before refactoring any non-trivial function
- When encountering unfamiliar code
- When planning architectural changes
- During code review (verify reviewer understands context)
