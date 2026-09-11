# nt_file_ability — 统一文件能力

## 能力概览

| 模块 | 层级 | 功能 | SelfTest |
|------|------|------|----------|
| tables | L1 | 基础表格读写 (calamine) | ✅ |
| xlsx_fast | L1 | 快速 XLSX 解析 (zip+quick_xml) | ✅ |
| xlsx_parser | L2 | 统一 XLSX 解析入口 | ✅ |
| merge | L2 | 多文件合并引擎 | ✅ |
| template_engine | L2 | 模板检测 | ✅ |
| config_parser | L2 | 配置字段解析 | ✅ |
| path_metadata | L2 | 路径元数据提取 | ✅ |
| table_presenter | L3 | LLM 表示层 (Markdown/JSON/CSV) | ✅ |
| chunk_planner | L2 | 大表格分块 | ✅ |
| excel_tool_schema | L3 | Function Calling Schema | ✅ |
| excel_capability | L2 | 能力树注册 | ✅ |
| selftest_excel | - | SelfTest T1-T3 | - |
| event_types | - | EventBus 事件 | - |

## 使用示例

```rust
use neotrix::neotrix::nt_file_ability::*;

// 快速解析
let tables = parse_xlsx("data.xlsx")?;

// 自动解析 (读取+检测+提取)
let result = ExcelCapability::auto_parse("data.xlsx")?;

// 配置解析
let config = ConfigFields::parse("执行标准:美标,压力:150LB");

// 模板检测
let (template, map) = ColumnMap::detect(&header_row);

// 路径元数据
let meta = PathMetadata::from_path(std::path::Path::new("path/to/file.xlsx"));

// LLM 表示
let md = TablePresenter::new(&table).to_markdown();
let json = TablePresenter::new(&table).to_json();
```

## 路由策略

详见 `xlsx_parser.rs` 模块文档。
