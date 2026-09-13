# Targeted Research #666 — Consciousness Core Dispatch Route Expansion

**Date**: 2026-09-13
**File**: `neotrix-core/src/core/l5_consciousness/consciousness_core/dispatch.rs`

## Summary

Added 4 new internal dispatch routes to `CAPABILITY_ROUTES` and their corresponding match arms in `dispatch_internal_capability`. These routes bridge the consciousness core task loop to newly integrated subsystems.

## New Routes

| # | Keyword(s) | `capability_tag` | Domain | Specialist |
|---|------------|------------------|--------|------------|
| 1 | `universal_model`, `统一模型`, `模型接口`, `llm接口` | `universal_model` | NT-IO | CreativityGenerator |
| 2 | `file_enhance`, `文件增强`, `增强文件`, `pdf增强`, `pdf清晰度` | `file_enhance` | NT-ACT | CodeAnalyzer |
| 3 | `kb_governance`, `知识库治理`, `kb治理`, `kb清理`, `kb整理` | `kb_governance` | NT-MEMORY | KnowledgeRetriever |
| 4 | `seal_process`, `seal流水线`, `seal处理`, `流水线处理`, `执行流水线` | `seal_process` | NT-MIND | KnowledgeIntegrator |

## Dispatch Arm Behavior

### `universal_model` (NT-IO)
Calls `crate::neotrix::list_llm_providers()` to enumerate available LLM providers. Filters by keyword match; falls back to full list if no keyword match. Reports provider count and names.

### `file_enhance` (NT-ACT)
Extracts file path from task summary, calls `crate::neotrix::enhance_file_icon()`. This wires into the `PdfIconEnhance` pipeline (extract→SR→embed). Validates path existence before invocation.

### `kb_governance` (NT-MEMORY)
Opens `KnowledgeBase`, reads stats (nodes/edges/kv_entries), and reports governance-level metrics. Serves as a diagnostic read path for KB health.

### `seal_process` (NT-MIND)
Dispatches to one of three SEAL operations based on summary content:
- `distill` keyword → `seal_distill()`
- `absorb` keyword → `seal_absorb()`
- default → `seal_iterate()`

## Design Notes

- All routes follow the existing `CAPABILITY_ROUTES` convention: `(keyword, capability_tag, domain, specialist)`
- Match arms delegate to `crate::neotrix::*` public API surface — no new logic in dispatch.rs itself
- Routes are registered in keyword→tag order, enabling the `decompose_instruction` function to match natural-language task descriptions
