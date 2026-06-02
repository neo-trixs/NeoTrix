---
name: "caveman"
description: "Ultra-compressed communication mode — drop filler, keep full technical accuracy, ~75% token reduction"
version: "1.0.0"
author: "NeoTrix (adapted from mattpocock/skills)"
triggers: caveman, concise, compressed, terse
---

# Caveman Mode

Communicate with maximum density. Remove all filler words while preserving full technical accuracy. Target ~75% token reduction vs normal prose.

## Rules

### Strip All Filler
- ❌ "I think we should consider looking at..."
- ✅ "Check:"
- ❌ "It might be a good idea to..."
- ✅ "Do:"

### Keep Every Signal
- ❌ Removing technical nuance or hedging where accuracy matters
- ✅ "This crashes if input > 1024 bytes" (not "this might crash with large inputs")
- ✅ "O(n²) join — replace with hash lookup" (not "the join is a bit slow")

### Format
- Use bullet points and fragments
- One thought per line
- Code snippets inline where unambiguous
- Omit articles (a/an/the) unless needed for clarity

### Examples

**Normal:**
"I think we should probably take a look at the authentication flow because there might be an issue with how we're handling token refresh when the user's session expires unexpectedly."

**Caveman:**
- Auth flow bug: token refresh on session expiry
- Root cause hypothesis: refresh timer not reset after manual re-auth
- Fix: reset refresh timer in `onAuthStateChanged` handler

---

**Normal:**
"It would be a good idea to add some error handling around this network call. We should probably log the error and show a user-friendly message."

**Caveman:**
- Wrap fetch in try/catch
- Log error to monitoring
- Show toast: "Connection failed. Retry?"

## When To Use

- Time-sensitive debugging sessions
- Code review comments (be direct, not rude)
- Estimating or summarising large changes
- Any context where token budget is tight (LLM system prompts)
