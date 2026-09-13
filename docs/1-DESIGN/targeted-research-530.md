# Targeted Research 530 — Internal Pain Points (Iteration 28)

> Found: 3 user-facing features that silently return empty/hardcoded results.

---

## Pain Point 1: MCP PTC `stubs` + `exec` are dead stubs

**File:** `neotrix-core/src/cli/commands/agent_cmds.rs:353-354, 467-468`
**Severity:** P1

**What's wrong:** The `/mcp stubs` and `/mcp exec` subcommands compile and accept input, but silently return empty results. Both contain `FIXME: McpRegistry.gateway() not yet implemented`. A user invoking `/mcp stubs` sees "PTC stubs: 0 typed signatures" with no explanation that the feature is unimplemented. `/mcp exec` plans the call, validates it, then discards it — returning 0 results.

**Impact:** PTC (programmatic tool calling) is advertised in the CLI but completely non-functional. Users get silent empty responses instead of clear errors.

**Fix sketch:**

```rust
// agent_cmds.rs:353 — replace empty vec with explicit unsupported message
"stubs" => {
    // FIXME: McpRegistry.gateway() not yet implemented
    return CommandOutput::err("PTC stubs not yet implemented — McpRegistry.gateway() is pending. Use /mcp list to inspect available tools.");
}

// agent_cmds.rs:467 — same treatment for exec
"exec" => {
    // ... existing plan() validation ...
    // FIXME: McpRegistry.gateway() not yet implemented
    return CommandOutput::err(&format!(
        "PTC exec not yet implemented — McpRegistry.gateway() is pending. \
         Planned {} call(s) across {} stage(s) but execution is unavailable.",
        plan.total_calls(), plan.stages()
    ));
}
```

---

## Pain Point 2: VTuber emotion module returns hardcoded Neutral for all modalities

**File:** `neotrix-core/src/l4_emotion/nt_feel/nt_feel_vtuber.rs:207-308`
**Severity:** P1

**What's wrong:** Four public methods are placeholder stubs that always return the same hardcoded value regardless of input:
- `detect_from_voice()` (line 207): always returns `Neutral, intensity=0.5` — ignores audio bytes entirely
- `detect_from_visual()` (line 219): always returns `Neutral, intensity=0.5` — ignores image bytes entirely
- `synthesize_speech()` (line 290): returns empty `audio: vec![]` with fabricated `duration_ms`
- `transcribe_speech()` (line 305): returns `Ok("")` — silently discards audio input

**Impact:** Any code path calling these methods (VTuber pipeline, emotion-driven response) gets degenerate Neutral readings. The `generate_response()` method at line 257 chains `detect_from_text` (which works) with these stubs, so multi-modal emotion is broken.

**Fix sketch:**

```rust
// nt_feel_vtuber.rs:207 — return Err instead of silent Neutral
pub fn detect_from_voice(&self, _audio: &[u8]) -> Result<_EmotionReading, String> {
    Err("voice emotion detection not implemented — no ML model loaded".into())
}

// nt_feel_vtuber.rs:219 — same for visual
pub fn detect_from_visual(&self, _image: &[u8]) -> Result<_EmotionReading, String> {
    Err("visual emotion detection not implemented — no CV model loaded".into())
}

// nt_feel_vtuber.rs:290 — return Err instead of silent empty audio
pub fn synthesize_speech(&self, text: &str, emotion: &_EmotionType) -> Result<_VoiceOutput, String> {
    Err(format!("TTS not implemented for emotion {:?}", emotion))
}

// nt_feel_vtuber.rs:305 — return Err instead of silent empty string
pub fn transcribe_speech(&self, _audio: &[u8]) -> Result<String, String> {
    Err("STT not implemented — no speech recognition engine loaded".into())
}
```

---

## Pain Point 3: `/kb consistency` is a silent no-op

**File:** `neotrix-core/src/cli/commands/kb_cmds.rs:150-160`
**Severity:** P1

**What's wrong:** The `/kb consistency` command exists in the CLI help, accepts user input, opens the KB connection, but returns an empty string with `CommandOutput::ok`. The actual `setting_consistency` module is commented out (line 158: `// TODO: setting_consistency module not found; stub`). Users get a success response with zero information.

**Impact:** Users think consistency checking passed. The command is a lie — it reports success for an operation it never performed.

**Fix sketch:**

```rust
// kb_cmds.rs:150 — fail loudly when the module is missing
fn cmd_consistency(_args: &[String]) -> CommandOutput {
    let _conn = match open_raw_conn() {
        Some(c) => c,
        None => return CommandOutput::err("无法打开知识库 ~/.neotrix/knowledge.db"),
    };
    // setting_consistency module was removed; command is non-functional
    CommandOutput::err(
        "/kb consistency is not implemented — the setting_consistency module was removed. \
         Use `nt memory consistency` or remove this subcommand from help."
    )
}
```

---

## Summary

| # | Location | Issue | Severity | Fix |
|---|----------|-------|----------|-----|
| 1 | `agent_cmds.rs:353,467` | PTC stubs/exec silently return empty results | P1 | Return explicit `CommandOutput::err` |
| 2 | `nt_feel_vtuber.rs:207-308` | 4 multimodal methods return hardcoded Neutral/empty | P1 | Return `Err` instead of silent degenerate values |
| 3 | `kb_cmds.rs:157` | `/kb consistency` is a no-op returning ok("") | P1 | Return `CommandOutput::err` explaining unimplemented |
