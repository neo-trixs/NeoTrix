# Vision Subsystem
> Version 0.5 — Status: 🟡 draft

## Purpose
The vision subsystem provides image understanding: file/base64 input → multimodal LLM → VSA encoding → consciousness sensory buffer. Lazily initialized on first use; degrades gracefully when no API key is configured.

## Exposed Operations

### Tools
- `analyze_image_file(path: String) -> Result<String, VisionError>` — Read image file, send to LLM, encode result as VSA
- `analyze_image_base64(b64: String, mime: String) -> Result<String, VisionError>` — Process base64-encoded image
- `analyze_image_raw(bytes: Vec<u8>) -> Result<String, VisionError>` — Process raw image bytes

### Resources
- `vision://status` — Pipeline initialization status
- `vision://last_analysis` — Most recent analysis result

## Configuration Schema

```rust
pub struct VisionConfig {
    pub openai_api_key: Option<String>,    // for gpt-4o
    pub anthropic_api_key: Option<String>, // for claude-sonnet-4-20250514
    pub default_model: String,             // fallback model name
    pub max_image_size_bytes: u64,         // default 20_000_000
}
```

## Dependencies
- **Consciousness** (nt_core_consciousness) — Sensory buffer for VSA-encoded results
- **VSA Core** (nt_core_hcube) — VSA encoding pipeline
- No external image processing crate (uses data URIs + std::fs)

## Error States

| State | Trigger | Recovery |
|-------|---------|----------|
| `NoApiKey` | Neither OPENAI nor ANTHROPIC key set | Return "vision unavailable", return None |
| `ImageTooLarge` | File size > max_bytes | Reject with size limit message |
| `LlmFailure` | Multimodal LLM returns error | Retry once, then log and return fallback |
| `EncodingFailure` | VSA encoding after LLM response fails | Skip VSA step, return raw LLM text |
