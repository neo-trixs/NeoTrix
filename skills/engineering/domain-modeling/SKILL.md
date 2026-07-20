---
name: "domain-modeling"
description: "Actively build and sharpen NeoTrix's shared language — challenge terms against CONTEXT.md, stress-test with edge-case scenarios, and update CONTEXT.md and ADRs inline."
version: "1.0.0"
author: "NeoTrix (adapted from mattpocock/skills)"
triggers: model, language, term, context, sharpen, ubiquitous
---

# Domain Modeling

Build and sharpen the project's shared language (ubiquitous language). Every session starts by loading `CONTEXT.md` as a prefix. This skill ensures the shared language stays accurate as the codebase evolves.

## When to Use

- A new concept emerges that doesn't have a name in CONTEXT.md
- An existing term feels imprecise or is used inconsistently
- Code contradicts what CONTEXT.md claims
- An external absorption introduces new patterns that need naming

## Process

### 1. Scan for Term Conflicts

- Read CONTEXT.md — know the current language
- Scan recent code, AGENTS.md, and session transcripts for terms used differently than CONTEXT.md defines them
- Look for:
  - Same concept, different names ("crawl pipeline" vs "data acquisition pipeline")
  - Same name, different concepts ("embedding" meaning KB vector vs VSA)
  - Missing terms — concepts that have emerged but aren't named

### 2. Challenge Each Term

For each candidate term, ask:
- **Precision**: Does this term distinguish the concept from similar ones?
- **Essential**: Does the concept need a name at all? (not everything does)
- **Stability**: Has the meaning settled, or is it still evolving?
- **Bounded**: Does the term describe a single concept, not a cluster?

### 3. Stress-Test with Edge Cases

Construct concrete scenarios that push against the term's boundaries:
- "If this is called X, what would be called Y?"
- "When we add feature Z, does X still fit?"
- "What's the opposite of X? Is there one?"

### 4. Cross-Reference Against Code

Find the code that embodies each term and verify:
- Function/type names use the term consistently
- Module hierarchy respects the term's scope
- Comments and docs use the term, not synonyms

If code uses a different term → update the code or update CONTEXT.md (document the ambiguity in Flagged Ambiguities).

### 5. Update CONTEXT.md

Propose changes:
- **New term**: Add to the appropriate section with Definition + Avoid columns
- **Refined term**: Update Definition, add the old usage to Flagged Ambiguities
- **Deprecated term**: Move to Flagged Ambiguities with resolution

### 6. Update ADRs (if needed)

If the language change reflects a genuine architecture decision (hard to reverse, surprising, trade-off), write or update an ADR in `docs/adr/`.

## Output

- Updated CONTEXT.md with precise terms and flagged ambiguities
- Optional: ADR documenting consequential language decisions
- Optional: code renames if term drift was found
