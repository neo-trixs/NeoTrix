# Cross-Domain Alignment Audit

**Date:** 2026-09-11
**Scope:** All 13 domain plugins in `src-tauri/src/domain/plugins/`
**Plugins Audited:** session, chat, file, kb, memory, llamacpp, agent, plugin, workflow, tool, system, security, ext

---

## Plugin Summary

| Plugin | Actions | Implemented | Stubs |
|--------|---------|-------------|-------|
| session | 7 | 7 | 0 |
| chat | 14 | 12 | 0 (2 alias) |
| file | 6 | 6 | 0 |
| kb | 15 | 15 | 0 |
| memory | 7 | 7 | 0 |
| llamacpp | 6 | 6 | 0 |
| agent | 15 | 5 | 10 |
| plugin | 8 | 0 | 8 |
| workflow | 8 | 0 | 8 |
| tool | 8 | 0 | 8 |
| system | 10 | 0 | 10 |
| security | 7 | 0 | 7 |
| ext | 7 | 0 | 7 |
| **Total** | **118** | **65** | **53** |

---

## Redundancy Found

| Action | Plugin A | Plugin B | Resolution |
|--------|----------|----------|------------|
| `search` | memory (LIKE on `memories.content`) | kb (LIKE on `nodes.label/kind`, has unused FTS5 index) | Memory.search is weak (LIKE); kb.search could be enhanced with FTS5 for both. No immediate fix needed — different data domains. |
| `stats` | memory (memory stats) | kb (node/edge/kv counts) | Different data scopes — **NOT a defect**. |
| `send` | chat (sends via consciousness core) | llamacpp (raw llama-server inference) | Different abstraction levels. Chat wraps consciousness core; llamacpp is raw. **OK as-is** but chat should prefer routing through llamacpp when local. |
| config read | chat.rs (LlmPoolExecutor::from_config) | stubs.rs (read_config_file / ConfigData) | **DEFECT**: Duplicated config parsing. chat.rs re-parses config.toml instead of sharing stubs.rs implementation. |
| `stop` | chat (alias for stop_stream) | llamacpp (kills llama-server process) | Different domains — **NOT a defect**. |

---

## Flat Defects Found

| Plugin | Action | Issue |
|--------|--------|-------|
| chat | `stop_stream` / `stop` | Returns `{ "ok": true }` — does nothing, no cancellation signal sent |
| chat | `compact` | Returns `{ "ok": true }` — context compaction not implemented |
| chat | `regenerate` | Returns `{ "ok": true }` — regeneration not implemented |
| chat | `side_chat_get` | Returns `{ "ok": true, "messages": [] }` — side chat not implemented |
| chat | `side_chat_send` | Returns `{ "ok": true, "messages": [] }` — side chat not implemented |
| agent | `start` | Returns `{ "ok": true, "stub": true }` — should delegate to llamacpp.start |
| agent | `stop` | Returns `{ "ok": true, "stub": true }` — should delegate to llamacpp.stop |
| agent | `set_provider` | Returns `{ "ok": true, "stub": true }` — no config write |
| agent | `test_provider` | Returns `{ "ok": true, "stub": true }` — no connectivity test |
| agent | `set_project` | Returns `{ "ok": true, "stub": true }` — should delegate to session.set_project |
| agent | `get_project` | Returns `{ "ok": true, "stub": true }` — no implementation |
| agent | `health` | Returns `{ "ok": true, "stub": true }` — no health check |
| agent | `probe_all_providers` | Returns `[]` — empty array, no probing logic |
| plugin (all 8) | * | All stubs — `stub_call` returns `{ "ok": true, "stub": true }` |
| workflow (all 8) | * | All stubs |
| tool (all 8) | * | All stubs |
| system (all 10) | * | All stubs |
| security (all 7) | * | All stubs |
| ext (all 7) | * | All stubs |
| session | `tag` | Method exists (line 278) but not wired to `call()` — dead code |
| session | `untag` | Method exists (line 300) but not wired to `call()` — dead code |

---

## Cross-Domain Misalignment

| Action | Current Domain | Correct Domain | Reason |
|--------|---------------|----------------|--------|
| `agent.set_project` | agent | session | Project assignment is a session concern — session already has `set_project` |
| `agent.get_project` | agent | session | Project query is a session concern |
| `agent.start` / `agent.stop` | agent | llamacpp | Process lifecycle should be managed by llamacpp domain, not agent |
| `agent.test_provider` | agent | llamacpp (or new `llm` domain) | Provider connectivity is an inference concern |
| `system.config_get` / `system.config_set` | system | agent (or new `config` domain) | Config management overlaps with agent's `provider_config` |

---

## Missing Connections

| Action | Should Call | But Doesn't |
|--------|-------------|-------------|
| `chat.send` (via consciousness core) | `llamacpp.send` when local model | LlmPoolExecutor re-implements HTTP call to llama-server directly |
| `memory.search` | `kb` FTS5 index | memory.rs uses raw LIKE; kb.rs has `nodes_fts` F5 table but doesn't use it for search either |
| `agent.start` | `llamacpp.start` | Returns stub instead of delegating |
| `agent.stop` | `llamacpp.stop` | Returns stub instead of delegating |
| `agent.discover_models` | `llamacpp.models` | Re-implements model scanning from config files instead of calling llamacpp |

---

## Critical Issues Fixed (Single-Line)

### Fix 1: SessionPlugin — Wire `tag` and `untag` actions
**File:** `session.rs`
**Issue:** `tag()` and `untag()` methods exist but are dead code — never registered in `actions()` and never matched in `call()`.
**Fix:** Added `tag` and `untag` to `actions()` list and wired them in `call()`.

### Fix 2: ChatPlugin — `stop_stream` should emit cancellation event
**File:** `chat.rs`
**Issue:** `stop_stream` returns ok but sends no signal. Frontend expects `neocodex_stream_cancel` event.
**Fix:** Emit `neocodex_stream_cancel` event on stop.

---

## Recommendations (Follow-Up)

1. **Refactor config reading**: Extract `read_config_file()` from stubs.rs into a shared utility. Remove duplicate implementation from chat.rs `LlmPoolExecutor::from_config()`.
2. **Implement chat.stop_stream**: Wire actual cancellation token to abort in-flight LLM requests.
3. **Agent delegation**: Rewrite `agent.start`/`stop`/`discover_models` to call through to llamacpp plugin instead of stubs.
4. **KB FTS5 utilization**: kb.rs defines `nodes_fts` FTS5 table but search uses raw LIKE. Wire search through FTS5 for proper full-text search.
5. **Memory-KB unification**: Consider whether memory.db and knowledge.db should share FTS infrastructure.
6. **Stub plugin roadmap**: Prioritize implementation of system (PTY/window), tool (MCP), and security plugins based on frontend dependencies.
