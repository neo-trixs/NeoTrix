---
name: "nt_multimodal"
description: "Multimodal file inspection via Qwen-MM-Plugins bridge — media_info first, dynamic-resolution read_image/read_video, visualize-anything, save_view chains, credential-gated web/image search, plus local PDF text grounding"
version: "1.1.0"
author: "NeoTrix (absorbed from QwenLM/Qwen-MM-Plugins, Apache-2.0)"
triggers: multimodal, image, video, visualize, media_info, read_image, read_video, reverse-image, NIfTI, PDF render, PDF grounding
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
   then locate what you need on them (see **Grounding** below).
4. **Confirm before commit**: anything you cannot confirm from the media
   alone (identity, fact) MUST go through `web_search` (needs a backend
   key) before answering. Never commit from appearance alone.
5. **Images land on disk**: `image` blocks are saved under the artifacts
   dir and only paths flow back as text. Feed paths to the vision path
   (`image_url`), don't pretend base64 flowed through.

## Grounding (坐标定位) — 两条路，别自己编框

| Input | Path | What you get |
|---|---|---|
| **PDF**（有文字层） | `pdf_ground_text {path, query}` | 页码 + `0-1000` 归一化框，y 已翻成图像坐标系。**本地 Rust（`lopdf` 读 content 流），零外部依赖、无 key、不 spawn** |
| **PDF**（扫描件/文字转轮廓） | 同上，会明确回「没找到文字层」 | 改用 `qwen_visualize` 渲染该页自己看；**不要拿估计的框交差** |
| **图片 / 渲染页** | 直接问 VLM 要框 | 让模型输出 `0-1000` 归一化框（同制式），并注明这是估计值 |

坐标制式统一为 `0-1000`、左上原点 —— 与 Qwen-MM-Plugins 的 `crop`/`draw_bbox`
及 Qwen2.5-VL 绝对坐标同一制式，换算像素时乘以图宽/图高再取整。

⚠️ **没挂载的上游工具**（2026-09-29 实测，写下来防止"以为能用"）：
`crop`、`draw_bbox`、`image_search`、通用 `OCR` **都不在** NeoTrix 的工具面上。
上游有 ≠ 这里能调（"导出 ≠ 调用"）。原因分别是：前两个要 grounding 且当前
只覆盖 PDF 文字；`image_search` 要 Serper key；通用 OCR 要外挂二进制或数百 MB
检测权重（装了 tesseract 也是**多一个外部依赖**，`nt_world/ocr` 那两个引擎至今
是占位实现）。**缺口就写在这里，不要假装它存在。**

## Tools

| Tool | Server | Gate |
|---|---|---|
| `media_info`, `read_image`, `read_video` | core | needs ffmpeg/ffprobe for video/audio |
| `visualize` | core | Office needs LibreOffice; 3D best with Blender (else fallback) |
| `save_view` | core | write files (Medium risk) |
| `crop`, `draw_bbox` | core | ⚠️ **未挂载**（见 Grounding 一节） |
| `web_search`, `web_extractor` | search | needs one backend key (SERPER/TAVILY/EXA/SERPLY) |
| `image_search` | search | ⚠️ **未挂载**（Serper Lens 要 key） |
| `pdf_ground_text` | **本地 Rust**（非 MCP） | 无外部依赖；只对有文字层的 PDF 有效 |

## Failure Modes

| Mode | Detection | Recovery |
|---|---|---|
| Server not installed | startup prints `<server> unavailable: …` + install hint | follow the hint (`uvx …@tag` or sparse checkout + `QWEN_MM_PLUGINS_CHECKOUT`); tools stay unlisted until then (fail-closed) |
| Search unregistered | report says "no search backend key" | export one `*_API_KEY` and restart |
| Call timeout | `timed out after Nms` + server stderr tail | shrink scope (fewer frames/pages, smaller budget), retry once |
| `[image block NOT saved]` in output | artifacts dir unwritable | point `artifacts_dir` elsewhere; text parts still usable |

## References

- Bridge: `neotrix-core/src/nt_mcp_stdio_session.rs`, `nt_qwen_mm_manifests.rs`
- Local grounding: `crates/neotrix-neobot/src/nt_pdf_ground.rs`（工具 `pdf_ground_text`）
- Upstream: `src/capabilities/{core,search}/skill/SKILL.md` (Qwen-MM-Plugins)
- Absorption record: `docs/architecture/ABSORPTION-QWEN-MM-2026-09-28.md`
