# nt_file_ability — 统一文件能力

## 能力概览

### 顶层模块

| 模块 | 层级 | 功能 | SelfTest |
|------|------|------|----------|
| core | L1 | 文件能力基础抽象 (FileAbility, FileKind) | ✅ |
| types | - | 公共类型定义 (FileAbilityError, TableData) | - |
| event_types | - | EventBus 事件定义 | - |
| capability | L1 | 能力树注册 (UnifiedCapability trait + PdfEnhanceCapability) | ✅ |
| image_super_resolution | L1 | 通用图像超分辨率 (12 模型, Tiled 推理) | ✅ |
| doc_parse | L2 | 文档解析管线 (magika 探测 → markitdown 归一 → FileModel) | ✅ |
| embedding | L2 | 文档向量嵌入 | ✅ |
| encoding | L2 | 编码检测与转换 | ✅ |
| structured | L2 | 结构化数据提取 | ✅ |
| format_route | L2 | 格式路由分发 | ✅ |
| helpers | - | 内部辅助函数 | - |
| e8 | - | E8 推理集成 | - |
| gwt | - | GWT 注意力集成 | - |

### 子目录

#### `excel/` — Excel/XLSX 能力

| 模块 | 层级 | 功能 | SelfTest |
|------|------|------|----------|
| tables | L1 | 基础表格读写 (calamine) | ✅ |
| xlsx_fast | L1 | 快速 XLSX 解析 (zip+quick_xml) | ✅ |
| xlsx_parser | L2 | 统一 XLSX 解析入口 | ✅ |
| excel_tool_schema | L3 | Function Calling Schema | ✅ |
| excel_capability | L2 | 能力树注册 | ✅ |
| selftest_excel | - | SelfTest T1-T3 | - |

#### `pdf/` — PDF 能力

| 模块 | 层级 | 功能 | SelfTest |
|------|------|------|----------|
| pdf_icon_enhance | L1 | PDF 图标清晰度提升 (提取→超分→嵌入) | ✅ |
| pdf_image_extract | L1 | PDF 图像提取 (XObject 树遍历) | ✅ |
| pdfedit | L2 | PDF 编辑操作 | ✅ |

#### `merge/` — 多文件合并能力

| 模块 | 层级 | 功能 | SelfTest |
|------|------|------|----------|
| merge | L2 | 通用多表合并引擎 (MergeSchema 数据化, SheetMode 运行时策略) | ✅ |
| merge_docx | L2 | DOCX 结构级合并 (OPC part 级) | ✅ |

#### `visual/` — 视觉理解能力

| 模块 | 层级 | 功能 | SelfTest |
|------|------|------|----------|
| visual | L2 | VLM 视觉提取管线 (doc7 吸收, Document/Slide prompt 路由) | ✅ |
| ocr | L1 | OCR 抽象 (OcrEngine trait + RuleBasedOcr 基线) | ✅ |
| grounding | L2 | Grounding 精确值校验 (数字/标识符缺失检测, reliability 评分) | ✅ |

## 目录结构

```
nt_file_ability/
├── mod.rs                 # 模块根
├── core.rs                # FileAbility 基础抽象
├── types.rs               # 公共类型 (FileAbilityError, TableData, FileKind)
├── capability.rs          # 能力树注册 (UnifiedCapability, PdfEnhanceCapability)
├── image_super_resolution.rs  # 图像超分辨率 (12 模型, Tiled 推理, HuggingFace 自动下载)
├── doc_parse.rs           # 文档解析管线
├── embedding.rs           # 文档向量嵌入
├── encoding.rs            # 编码检测
├── structured.rs          # 结构化提取
├── format_route.rs        # 格式路由
├── event_types.rs         # EventBus 事件
├── selftest.rs            # SelfTest 入口
├── helpers.rs             # 辅助函数
├── e8.rs / gwt.rs         # 意识核心集成
├── excel/                 # Excel 能力
│   ├── mod.rs
│   ├── tables.rs          # calamine 表格读写
│   ├── xlsx_fast.rs       # zip+quick_xml 快速解析
│   ├── xlsx_parser.rs     # 统一解析入口 + 路由策略
│   ├── excel_tool_schema.rs  # Function Calling Schema
│   ├── excel_capability.rs   # 能力树注册
│   └── selftest_excel.rs     # SelfTest T1-T3
├── pdf/                   # PDF 能力
│   ├── mod.rs
│   ├── pdf_icon_enhance.rs    # 图标清晰度提升管线
│   ├── pdf_image_extract.rs   # XObject 树遍历图像提取
│   └── pdfedit.rs             # PDF 编辑
├── merge/                 # 合并能力
│   ├── mod.rs
│   ├── merge.rs           # 通用合并引擎 + MergeSchema + SchemaStore
│   └── merge_docx.rs      # DOCX 结构级合并
└── visual/                # 视觉理解能力
    ├── mod.rs
    ├── visual.rs          # VLM 视觉提取 (doc7 prompt 路由)
    ├── ocr.rs             # OCR trait + RuleBasedOcr
    └── grounding.rs       # Grounding 校验 (reliability 评分)
```

## 使用示例

### Excel 解析

```rust
use neotrix::neotrix::nt_file_ability::excel::*;

// 快速解析
let tables = parse_xlsx("data.xlsx")?;

// 自动解析 (读取+检测+提取)
let result = ExcelCapability::auto_parse("data.xlsx")?;
```

### 多表合并

```rust
use neotrix::neotrix::nt_file_ability::merge::*;

// 价格表合并 (修改版 sheet 优先)
let report = merge_tables_with_mode(
    &PRICE_TABLE_SCHEMA,
    "suppliers/",
    "output/consolidated.xlsx",
    SheetMode::Preferred(&["修改版"]),
)?;

// 自定义 schema 合并
let schema = MergeSchema { name: "产品库", /* ... */ };
let report = merge_tables_with(&schema, "data/", "merged.xlsx")?;

// 统一合并入口 (按格式自动路由)
let outcome = collection_merge(&CollectionMergeRequest {
    inputs: vec!["a.pdf".into(), "b.pdf".into()],
    strategy: MergeStrategy::All,
    output: "merged.pdf".into(),
    ..Default::default()
})?;

// Schema 建议 (扫描目录 → LLM 增强 → 人工确认)
let suggestion = suggest_schema("suppliers/", Some(&llm_callback))?;
```

### PDF 增强

```rust
use neotrix::neotrix::nt_file_ability::pdf::*;

// PDF 图标清晰度提升 (提取→超分→嵌入)
let result = enhance_pdf_icons_with_config(
    std::path::Path::new("document.pdf"),
    PdfIconEnhanceConfig::default(),
)?;

// PDF 图像提取
let images = extract_pdf_images(std::path::Path::new("document.pdf"))?;
```

### 视觉理解

```rust
use neotrix::neotrix::nt_file_ability::visual::*;

// VLM 视觉提取 (doc7 管线)
let result = visual_extract(
    FileKind::Image,
    &image_base64,
    &embedded_text,
    &VisualExtractConfig::default(),
    &vlm_call_closure,
);

// Grounding 校验
let report = ground_missing_tokens(&source_text, &vlm_output);
// report.reliability_score >= 0.5 表示提取在保真上限内
```

### 图像超分辨率

```rust
use neotrix::neotrix::nt_file_ability::image_super_resolution::*;

// 超分辨率推理
let result = super_resolve(
    "input.png",
    "output.png",
    SuperResolutionModel::RealEsrganGeneral,
    2,  // scale
)?;
```

### OCR

```rust
use neotrix::neotrix::nt_file_ability::visual::*;

// 使用默认 RuleBasedOcr
let ability = FileAbility::new("image.png");
let ocr_result = ability.ocr(None);

// 注入自定义 OCR 引擎
let ocr_result = ability.ocr(Some(Box::new(MyCustomOcr)));
```

### 文档解析

```rust
use neotrix::neotrix::nt_file_ability::doc_parse::*;

// 解析任意格式文档为 FileModel
let model = parse_document("report.docx")?;
```

## 能力树注册

`nt_file_ability` 通过 `CapabilityRegistry` 向 NT-CORE 能力树注册:

| 能力 ID | 域 | 层级 | 说明 |
|---------|------|------|------|
| `nt-file-pdf-enhance` | NtFileAbility | L1Action | PDF 图标/图像超分增强 |

### 注册方式

```rust
// 本地注册
let mut registry = CapabilityRegistry::new();
register_pdf_enhance_capability(&mut registry);

// NT-CORE 能力工厂消费 (双 trait 实现)
let cap = create_pdf_enhance_capability(); // Arc<dyn UnifiedCapability>
```

### UnifiedCapability 接口

```rust
pub trait UnifiedCapability: Send + Sync {
    fn meta(&self) -> CapabilityMeta;       // 能力元数据
    fn health(&self) -> CapabilityHealth;   // 健康度
    fn execute(&self, input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError>;
    fn supports(&self, input: &CapabilityInput) -> bool;
}
```

## 架构分层

```
L1 格式编解码 (通用)     → tables, xlsx_fast, pdfedit, ocr
L2 表格语义 (通用)       → merge engine, xlsx_parser, grounding, visual
L3 领域 schema (差异化)  → MergeSchema (价格表/产品库), excel_tool_schema
L4 意识核心 (调度)       → capability.rs → NT-CORE CapabilityFactory
```

## 设计原则

- **R-P42**: 复用组合，不建平行模块。pdf_icon_enhance 组合 image_super_resolution
- **Schema 数据化**: 领域知识全进 MergeSchema const / JSON，引擎零内置
- **Grounding 公理**: 视觉理解是提取上限，grounding 是精确值保险
- **闭包注入**: VLM 模型调用经闭包注入，测试接假闭包（零网络依赖）
