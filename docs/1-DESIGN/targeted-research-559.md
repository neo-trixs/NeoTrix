# Targeted Research: Internal Stub Fixes (559)

## Summary

Replaced fabricated success data in 3 files with honest error returns and added documentation comments.

## Files Modified

### 1. `l4_emotion/nt_feel/nt_feel_vtuber.rs`

**Before**: Functions returned `Ok(Neutral, 0.5)` or `Ok("")` for unimplemented features.

**After**: Functions return `Err("not wired: ...")` with clear error messages.

| Function | Change |
|----------|--------|
| `detect_from_voice` | `Ok(Neutral, 0.5)` → `Err("not wired: speech emotion recognition...")` |
| `detect_from_visual` | `Ok(Neutral, 0.5)` → `Err("not wired: visual emotion recognition...")` |
| `synthesize_speech` | `Ok(empty audio)` → `Err("not wired: TTS synthesis...")` |
| `transcribe_speech` | `Ok("")` → `Err("not wired: STT transcription...")` |

### 2. `l1_action/nt_io/nt_io_messaging.rs`

**Before**: WhatsApp and Email providers returned fake message IDs and empty vectors.

**After**: All MessagingProvider trait methods return `Err(CapabilityError::NotAvailable(...))`.

| Provider | Methods Fixed |
|----------|---------------|
| WhatsAppProvider | `send`, `receive`, `get_status` |
| EmailProvider | `send`, `receive`, `get_status` |

### 3. `l1_action/nt_io/nt_io_plugin/wasm.rs`

**Before**: `call_export` returned fabricated `Ok("wasm:func(arg):ok")`.

**After**: Returns `Err("not wired: WASM result extraction not implemented")`.

## Impact

- **Callers**: Must now handle `Err` cases instead of receiving fabricated success
- **No regressions**: All error paths were previously dead code (fabricated success masked failures)
- **Observability**: Error messages clearly indicate what needs to be implemented

## Verification

```bash
cargo check --all-targets -p neotrix
cargo test -p neotrix --lib
```
