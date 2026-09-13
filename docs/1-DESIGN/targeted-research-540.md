# Targeted Research 540: Internal Capability Dispatch Wiring

**Date**: 2026-09-13  
**Rule**: R-P110 (Non-essential CLI command building forbidden, internal dispatch routing)

## Summary

Wired 6 unwired file-ability functions into `dispatch_internal_capability`, enabling the consciousness core to route tasks directly to internal Rust functions instead of falling through to the generic `_` fallback.

## Changes

### File: `neotrix-core/src/core/l5_consciousness/nt_core_consciousness_core.rs`

#### 1. New CAPABILITY_ROUTES entries (lines 842-854)

| Chinese Keyword | Capability Tag | Domain | Specialist |
|---|---|---|---|
| 智能合并 / 混合合并 | `collection_merge` | NT-ACT | CodeAnalyzer |
| 编辑表格 / 单元格 | `xlsx_edit` | NT-ACT | CodeAnalyzer |
| 读取结构 / 读取json / 读取yaml | `structured_read` | NT-WORLD | CodeAnalyzer |
| 写入json / 保存json | `json_write` | NT-ACT | CodeAnalyzer |
| pdf图片统计 / pdf图像信息 | `pdf_image_stats` | NT-WORLD | CodeAnalyzer |
| pdf提取图片 | `pdf_extract_images` | NT-ACT | CodeAnalyzer |

#### 2. New dispatch match arms (lines 1783-2057)

| Tag | Function Called | Input Parsing | Output |
|---|---|---|---|
| `collection_merge` \| `smart_merge` | `collection_merge(&CollectionMergeRequest)` | `<out> <in1> <in2> ...` | MergeOutcome variant description |
| `xlsx_edit` \| `table_edit` | `edit_xlsx_table(path, &[TableEdit])` | `<path> --set r,c=v --insert r --remove r` | Edit count + row count |
| `structured_read` \| `json_read` | `read_structured(path)` | `<path>` | Format + pretty-printed JSON |
| `json_write` \| `structured_write` | `write_json(path, &Value, true)` | `<path> <json_string>` | Confirmation |
| `pdf_image_stats` \| `pdf_images_info` | `pdf_image_stats(path)` | `<pdf_path>` | Image count, formats, avg size |
| `pdf_extract_images` | `extract_pdf_images(path, dir, &config)` | `<pdf_path> [out_dir]` | Image count + elapsed ms |

## Capability Coverage

### Before (8 wired + generic fallback)

```
xlsx_consolidation, data_merge, file_extract, content_extraction, file_parsing,
pdf_edit, image_convert, dir_extract, pdf_merge, doc_merge,
pdf_icon_enhance, pdf_enhance, _ (generic)
```

### After (14 wired + generic fallback)

```
xlsx_consolidation, data_merge, file_extract, content_extraction, file_parsing,
pdf_edit, image_convert, dir_extract, pdf_merge, doc_merge,
pdf_icon_enhance, pdf_enhance,
collection_merge, smart_merge,           ← NEW
xlsx_edit, table_edit,                   ← NEW
structured_read, json_read,              ← NEW
json_write, structured_write,            ← NEW
pdf_image_stats, pdf_images_info,        ← NEW
pdf_extract_images,                      ← NEW
_ (generic)
```

### Unwired CAPABILITY_ROUTES (agent-level, external gap closure)

These tags remain unwired because they require multi-step agent execution (LLM, crawling, SEAL pipeline), not single function calls:

| Tag | Domain | Reason |
|---|---|---|
| `hybrid_retrieval` | NT-MEMORY | KB search with embedding + BM25 |
| `skill_crystallize` | NT-MIND | SEAL pipeline distillation |
| `tdd` | NT-MIND | Multi-step TDD workflow |
| `code_refactor` | NT-ACT | Code analysis + refactor |
| `security_audit` | NT-SHIELD | Multi-check audit |
| `security_governance` | NT-SHIELD | Policy enforcement |
| `architecture_decision` | NT-CORE | Multi-criteria evaluation |
| `consciousness_tree` | NT-CORE | 6-stage feedback loop |
| `meta_cognition` | NT-META | Cross-session reflection |
| `root_cause_method` | NT-REPAIR | Multi-step diagnosis |
| `build_hygiene` | NT-REPAIR | Build + test cycle |
| `unified_crawler` | NT-WORLD | Web crawling pipeline |
| `frontend_ui` | NT-IO | UI generation |
| `experience_absorb` | NT-MEMORY | 5-stage absorption |

## Design Rationale

- **R-P110 compliance**: All new capabilities are wired via `dispatch_internal_capability`, not CLI subcommands
- **R-P79 alignment**: Each match arm calls the actual Rust function (not just marking "executed")
- **Backward compatibility**: Existing `_` fallback unchanged; new tags simply don't fall through
- **Error handling**: Each arm returns `(false, error_msg)` on failure, `(true, success_msg)` on success
