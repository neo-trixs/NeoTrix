---
name: "nt_multimodal"
description: "Multimodal file inspection via Qwen-MM-Plugins bridge — media_info first, dynamic-resolution read_image/read_video, visualize-anything, save_view chains, credential-gated web/image search"
version: "1.0.0"
author: "NeoTrix (absorbed from QwenLM/Qwen-MM-Plugins, Apache-2.0)"
triggers: multimodal, image, video, visualize, media_info, read_image, read_video, reverse-image, NIfTI, PDF render
---

# NT Multimodal — Qwen-MM-Plugins Bridge

Read and visualize local media through the `qwen-mm-plugins-core` /
`qwen-mm-plugins-search` MCP servers (session-bridged, see
`neotrix-core/src/nt_mcp_stdio_session.rs`). This skill **reinforces**
L2 perception and `nt_file_ability` (R-P42): Rust side keeps doing
text/structured parsing; the pixel side (render anything → image the
model can see) goes through this bridge. No parallel module was built.

## Discipline (from upstream SKILL.md, enforced here)

1. **Metadata first**: any video/audio → `media_info` BEFORE `read_video`
   or any clip/edit. Header-only, fast on huge files. Check VFR flag,
   rotation, fps/DAR mismatch, audio rate — never assume fixed fps.
2. **Budgets**: `small` preview, `normal` (~1024) default, `large` detail.
   Video skim: `fps=1` in 5-min chunks, then `fps=2, budget=large` on
   interesting segments; window with `start_time`/`end_time`.
3. **Chains**: document pages / video frames → `save_view` to files →
   then `crop` / `draw_bbox` / OCR / `image_search` on the saved paths.
4. **Confirm before commit**: anything you cannot confirm from the media
   alone (identity, fact) MUST go through `web_search`/`image_search`
   before answering. Never commit from appearance alone.
5. **Images land on disk**: `image` blocks are saved under the artifacts
   dir and only paths flow back as text. Feed paths to the vision path
   (`image_url`), don't pretend base64 flowed through.

## Tools

| Tool | Server | Gate |
|---|---|---|
| `media_info`, `read_image`, `read_video` | core | needs ffmpeg/ffprobe for video/audio |
| `visualize` | core | Office needs LibreOffice; 3D best with Blender (else fallback) |
| `save_view`, `crop`, `draw_bbox` | core | write files (Medium risk); 0–1000 normalized coords |
| `web_search`, `web_extractor` | search | needs one backend key (SERPER/TAVILY/EXA/SERPLY) |
| `image_search` | search | Serper Lens; private by default (`allow_public_upload=false`) |

## Failure Modes

| Mode | Detection | Recovery |
|---|---|---|
| Server not installed | startup prints `<server> unavailable: …` + install hint | follow the hint (`uvx …@tag` or sparse checkout + `QWEN_MM_PLUGINS_CHECKOUT`); tools stay unlisted until then (fail-closed) |
| Search unregistered | report says "no search backend key" | export one `*_API_KEY` and restart |
| Call timeout | `timed out after Nms` + server stderr tail | shrink scope (fewer frames/pages, smaller budget), retry once |
| `[image block NOT saved]` in output | artifacts dir unwritable | point `artifacts_dir` elsewhere; text parts still usable |

## References

- Bridge: `neotrix-core/src/nt_mcp_stdio_session.rs`, `nt_qwen_mm_manifests.rs`
- Upstream: `src/capabilities/{core,search}/skill/SKILL.md` (Qwen-MM-Plugins)
- Absorption record: `docs/architecture/ABSORPTION-QWEN-MM-2026-09-28.md`
