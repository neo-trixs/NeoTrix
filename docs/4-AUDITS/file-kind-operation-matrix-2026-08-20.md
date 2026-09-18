# FileKind × Operation 能力矩阵审计 (2026-08-20)

审计目标: 盘点 `nt_file_ability` 在各 FileKind 上的 `读/写/编辑/汇总/分析` 覆盖度,
为"每个文件类型都有基础需求"的统一能力体系提供落地依据。
审计基于源码盘点 (R-P16 实证), 非设计蓝图。

## 1. FileKind 枚举与后端

| FileKind | 后端 | 代表格式 |
|----------|------|---------|
| `Office(DocumentFormat)` | office_oxide | docx/xlsx/pptx/doc/xls/ppt |
| `Text` | neotrix-types FileParser | txt/md/json/xml/csv/rs/py |
| `Pdf` | lopdf + neotrix-types | pdf |
| `Image` | image crate | png/jpg/webp/gif/bmp/ico |
| `Audio` | 头部解析 | mp3/wav/ogg/flac/m4a |
| `Video` | 头部解析 | mp4/avi/mkv/mov/webm |
| `Binary` | MIME 猜测 | 其他 |

## 2. 能力矩阵 (实际代码支撑)

| FileKind | 读 | 写 | 编辑 | 汇总 | 分析 |
|----------|----|----|----|------|------|
| Office: xlsx | ✅ `read_xlsx_table` / `read_xlsx_sheets_all` / `xlsx_sheet` | ✅ `write_xlsx_table` | ✅ `edit_xlsx_table` | ✅ `merge_tables_with_mode` + SheetMode + SchemaStore (C4) | ✅ 数字列校验 / validation_warnings |
| Office: docx | ✅ `plain_text` / `to_markdown` / `to_html` | ✅ `save_as` | ✅ `replace_placeholder` (EditableDocx) | ⭕ 无多文档合并 | ⭕ 仅文本级 |
| Office: pptx | ✅ `plain_text` / `to_markdown` | ✅ `save_as` | ✅ `replace_placeholder` (EditablePptx) | ⭕ 无 | ⭕ |
| Office: doc/xls/ppt (legacy) | ✅ `plain_text` (FileParser) | ⭕ | ⭕ | ⭕ | ⭕ |
| Text: csv/tsv | ✅ `read_csv` (GBK/BOM 自动) | ✅ `write_csv` (BOM) | ✅ (TableData 变换) | ✅ 经 merge 引擎 (csv 输入/输出) | ✅ 数字校验 |
| Text: 其他 (md/json/rs...) | ✅ `plain_text` | ✅ `write` | ⭕ | ⭕ | ⭕ |
| Pdf | ✅ `plain_text` / `extract_pdf_tables` | ✅ `save_as` | ✅ `edit_pdf` (span redact + 原位替换) | ⭕ 无多 PDF 合并 | ⭕ 表格网格提取 (tables) |
| Image | ✅ `image_metadata` (尺寸/位深/alpha) | ✅ `convert_image` (png/jpeg 编码) | ⭕ | ⭕ | ⭕ OCR 占位 (`OcrEngine`) |
| Audio | ✅ `audio_duration_ms` | ⭕ | ⭕ | ⭕ | ⭕ |
| Video | ✅ `media` 元数据 | ⭕ | ⭕ | ⭕ | ⭕ |

图例: ✅ = 生产实现 + 测试; ⭕ = 缺口; ⭕+OCR = 有接口无生产接线。

## 3. 已闭合闭环 (2026-08-20 本会话)

Excel/CSV 纵向闭环 (C4 级) — **可复用范式**:
- `SheetMode` (FirstSheet/Preferred/AllSheets) 运行时参数化 → 零重编译换策略
- `SchemaStore` (JSON schema 数据化 + `~/.neotrix/schemas/`) → 零重编译换行业
- `suggest --save` → 扫描→建议→固化→复用 全闭环
- 输出按扩展名分发: `.csv` (UTF-8 BOM) / `.tsv` / `.xlsx`
- 能力树接线: CLI `/file consolidate` + 意识核心 `xlsx_consolidation` 同参数化入口

验证: 26 文件真实目录 → 6402 行 / 31 供应商 / 去重 320; 68+2 单测通过。

## 4. 缺口与横向推广优先级

| 优先级 | 目标 | 现状 | 复用范式 |
|--------|------|------|---------|
| ✅ 已完成 | Image 写 (png/jpeg 转换) | ✅ `convert_image` + `/file convert` + 意识核心 `image_convert` 路由 | image crate encoder |
| ✅ 已完成 | 目录级统一提取 (FileKind×读 统一入口) | ✅ `extract_dir` + `/file extract` + 意识核心 `dir_extract` 路由 | helpers `extract_dir` + TableData 行数 |
| ✅ 已完成 | Pdf 多文档合并 (结构级) | ✅ `merge_pdfs` (对象图重建+单一 Catalog/Pages) + `/file mergepdf` + 意识核心 `pdf_merge` 路由 | lopdf 官方 merge 流程 (renumber_objects_with) |
| P0 | docx 多文档合并 | ⭕ | 抽象 merge 为"文档集合→统一文档", 复用 SheetMode 三策略 |
| P0 | pptx 多文档合并 | ⭕ | 同上 (幻灯片级, 需分页粒度) |
| P2 | OCR 生产接线 | 接口有 | OcrEngine trait → 视觉闭环 (研究: oar-ocr/PaddleOCR-rs 零 unsafe 优先) |
| P2 | legacy doc/xls/ppt 写 | ⭕ | office_oxide 覆盖度扩展 |

> 注: PDF 合并 (P1) 已闭环 (2026-08-20, 见上表)。docx/pptx 结构级合并仍受
> office_oxide 无合并 API 限制, 可借鉴 DoclingDocument 统一文档模型做
> "文档集合→统一文档" 抽象 (研究见 2026-08-20 文献调研)。

## 5. 建议 (架构一致性)

1. **范式归一**: 把 `SheetMode` + `SchemaStore` 抽象为 `CollectionMerge` 通用模式
   (输入 = 文档集合 + 选择策略 + 输出格式), docx/pptx/pdf 合并统一走它 — R-P42 (强化现有节点, 非平行适配器)。
2. **schema 领域化**: 每文件类型维护 `schema.json` (结构化提取规则), 非仅 xlsx 列映射。
3. **能力网登记**: 每个新合并能力登记 `CapabilityRegistry` + 意识核心路由, 保持 Dark Forest (有消费者)。
4. **测试地基**: 每个闭环按 `SheetMode 三模式 + schema 加载 + 失败路径` 模式补单测 (参照 schema_tests 10 例)。