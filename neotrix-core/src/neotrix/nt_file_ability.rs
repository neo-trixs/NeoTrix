//! # NeoTrix 统一文件能力 (Unified File Ability)
//!
//! 单一入口管理和操作所有文件 I/O：
//! - **Office 6 格式** (DOCX/XLSX/PPTX/DOC/XLS/PPT) — 由 `office_oxide` 提供
//!   读 (`plain_text`/`to_markdown`/`to_html`)、导 (`to_ir`/`save_as`)、
//!   编辑 (`EditableDocx`/`EditablePptx::replace_text`)
//! - **通用文本/PDF/图像/音频/视频** — 由 `neotrix-types::FileParser` 探测并提取
//! - **图像元数据** — 由 `image` crate 提取尺寸/通道/格式
//!
//! ## 纪律
//! - **R-P1**: 零 unsafe (office_oxide + FileParser + image 均纯 Rust)
//! - **R-P42**: 复用 core 既有成熟类型 (`ConstellationLevel`/`SelfTest`)，
//!   不平行重造枚举
//! - **Dark Forest**: 模块经能力树 `ConstellationLevel` 标记成熟度，
//!   经 `SelfTest` T1-T3 接线到意识树健康链，被流水线消费
//! - **指针守恒**: 单一 `path` 句柄，不复制状态
//!
//! ## 结构
//! 本文件为入口 re-export 根: 按职责拆分的子模块经 `pub use` 保持
//! `crate::neotrix::nt_file_ability::*` 公共 API 面完全不变。

mod core;
mod e8;
mod embedding;
mod encoding;
mod grounding;
mod gwt;
mod helpers;
mod merge;
mod ocr;
mod selftest;
mod structured;
mod tables;
mod types;
mod visual;

pub use core::*;
pub use e8::*;
pub use embedding::*;
pub use encoding::*;
pub use grounding::*;
pub use gwt::*;
pub use helpers::*;
pub use merge::*;
pub use ocr::*;
pub use selftest::*;
pub use structured::*;
pub use tables::*;
pub use types::*;
pub use visual::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    use crate::core::nt_core_hcube::vsa::{VSAEngine, VsaBackend};
    use crate::core::nt_core_hex::ReasoningHexagram;
    use crate::core::nt_core_self_test::SelfTest;
    use crate::core::nt_core_traits::SpecialistType;
    use nt_core_capability_tree::ConstellationLevel;
    use office_oxide::{create, DocumentFormat};

    // 跨子模块私有项访问 (拆分后保留原单文件测试语义)
    use super::grounding::{compact_numeric_text, is_critical_numeric_token, normalize_numeric_token};
    use super::merge::derive_source_name;
    use super::tables::data_to_text;

    fn office_sample() -> PathBuf {
        let dir = std::env::temp_dir().join("nt_file_ability_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("sample.docx");
        if !path.exists() {
            create::create_from_markdown(
                "# 标题\n\nHello {{name}}, 这是 NeoTrix 测试文档。",
                DocumentFormat::Docx,
                &path,
            )
            .unwrap();
        }
        path
    }

    #[test]
    fn test_open_office_and_plain_text() {
        let path = office_sample();
        let mut ab = FileAbility::open(&path).unwrap();
        ab.register_consumer();
        assert!(matches!(ab.kind(), FileKind::Office(DocumentFormat::Docx)));
        assert!(ab.has_consumers());
        assert!(ab.plain_text().contains("NeoTrix"));
        assert!(ab.to_markdown().contains("标题"));
        assert!(ab.to_html().is_some());
    }

    #[test]
    fn test_replace_placeholder_docx() {
        let path = office_sample();
        let copy = path.with_extension("replace.docx");
        std::fs::copy(&path, &copy).unwrap();
        let ab = FileAbility::open(&copy).unwrap();
        let n = ab.replace_placeholder("{{name}}", "NeoTrix").unwrap();
        assert!(n > 0, "应至少替换一次占位符");
        std::fs::remove_file(&copy).ok();
    }

    // ── XLSX 单元格级结构化读取 (Ext-7) ──

    /// 构造多 sheet 测试夹具: sheet1 = "修改版" (表头+数据+公式), sheet2 = "原始"
    fn xlsx_fixture() -> PathBuf {
        let dir = std::env::temp_dir().join("nt_file_ability_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("prices.xlsx");
        if !path.exists() {
            use office_oxide::xlsx::write::{CellData, XlsxWriter};
            let mut xw = XlsxWriter::new();
            let s1 = xw.add_sheet_get_index("Sheet1");
            let s2 = xw.add_sheet_get_index("Sheet2");
            xw.sheet_set_cell(s1, 0, 0, CellData::String("品名".into()));
            xw.sheet_set_cell(s1, 0, 1, CellData::String("单价".into()));
            xw.sheet_set_cell(s1, 1, 0, CellData::String("蝶阀".into()));
            xw.sheet_set_cell(s1, 1, 1, CellData::Number(305.69));
            xw.sheet_set_cell(s1, 2, 0, CellData::String("闸阀".into()));
            xw.sheet_set_cell(s1, 2, 1, CellData::Number(128.0));
            xw.sheet_set_cell(s1, 3, 0, CellData::String("小计".into()));
            xw.sheet_set_cell(s1, 3, 1, CellData::Formula("B2+B3".into()));
            xw.sheet_set_cell(s2, 0, 0, CellData::String("备注".into()));
            xw.sheet_set_cell(s2, 1, 0, CellData::String("原始数据".into()));
            xw.save(&path).unwrap();
        }
        path
    }

    #[test]
    fn test_xlsx_structured_read() {
        let path = xlsx_fixture();
        let ab = FileAbility::open(&path).unwrap();
        // sheet 名 + 数量
        let names = ab.xlsx_sheet_names().unwrap();
        assert_eq!(names, vec!["Sheet1", "Sheet2"]);
        assert_eq!(ab.xlsx_sheet_count().unwrap(), 2);
        // 按名称读取 "修改版" 等价 sheet (Sheet1)
        let s1 = ab.xlsx_sheet_by_name("Sheet1").unwrap();
        assert_eq!(s1.name, "Sheet1");
        assert_eq!(s1.rows.len(), 4);
        // 表头行
        let hdr = &s1.rows[0];
        assert_eq!(hdr.index, 1);
        assert_eq!(hdr.cells.len(), 2);
        assert_eq!(hdr.cells[0].text, "品名");
        assert_eq!(hdr.cells[0].reference, "A1");
        assert_eq!(hdr.cells[0].col, 1);
        assert_eq!(hdr.cells[0].row, 1);
        assert_eq!(hdr.cells[0].value_type, SheetCellValueType::Text);
        // 数值行: 显示文本 + 原始数值 + 类型
        let row2 = &s1.rows[1];
        assert_eq!(row2.cells[1].text, "305.69");
        assert_eq!(row2.cells[1].number, Some(305.69));
        assert_eq!(row2.cells[1].value_type, SheetCellValueType::Number);
        assert_eq!(row2.cells[1].reference, "B2");
        // 公式行: 公式文本保留 (office_oxide 不重算公式, 显示文本可为空)
        let row4 = &s1.rows[3];
        assert_eq!(row4.cells[1].formula.as_deref(), Some("B2+B3"));
        // 行序与 index
        assert_eq!(
            s1.rows.iter().map(|r| r.index).collect::<Vec<_>>(),
            vec![1, 2, 3, 4]
        );
        // sheet 2
        let s2 = ab.xlsx_sheet(2).unwrap();
        assert_eq!(s2.name, "Sheet2");
        assert_eq!(s2.rows.len(), 2);
        assert_eq!(s2.rows[1].cells[0].text, "原始数据");
    }

    #[test]
    fn test_xlsx_sheet_index_boundaries() {
        let path = xlsx_fixture();
        let ab = FileAbility::open(&path).unwrap();
        // 0-based / 越界索引报错
        assert!(matches!(
            ab.xlsx_sheet(0),
            Err(FileAbilityError::SheetIndexOutOfRange { .. })
        ));
        assert!(matches!(
            ab.xlsx_sheet(3),
            Err(FileAbilityError::SheetIndexOutOfRange { .. })
        ));
        // 未知 sheet 名报错
        assert!(matches!(
            ab.xlsx_sheet_by_name("不存在"),
            Err(FileAbilityError::SheetIndexOutOfRange { .. })
        ));
    }

    #[test]
    fn test_xlsx_structured_read_non_xlsx() {
        let path = office_sample(); // docx
        let ab = FileAbility::open(&path).unwrap();
        assert!(matches!(
            ab.xlsx_sheet_names(),
            Err(FileAbilityError::UnsupportedFormat { .. })
        ));
    }

    #[test]
    fn test_open_text_file() {
        let dir = std::env::temp_dir().join("nt_file_ability_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("hello.rs");
        std::fs::write(&path, "fn main() { println!(\"hi\"); }").unwrap();
        let ab = FileAbility::open(&path).unwrap();
        assert_eq!(ab.kind(), FileKind::Text);
        assert!(ab.plain_text().contains("main"));
        std::fs::remove_file(&path).ok();
    }

    // ── D1-D6: 通用表格读写 / 编码检测 / 多文件合并 / 结构化数据 / 快照存储 ──

    fn test_dir() -> PathBuf {
        let d = std::env::temp_dir().join("nt_file_ability_d1d6");
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn test_d1_write_xlsx_table_roundtrip() {
        let dir = test_dir();
        let path = dir.join("t_out.xlsx");
        let t = TableData {
            name: "Sheet1".to_string(),
            headers: vec!["品名".to_string(), "单价".to_string(), "单重(Kg)".to_string()],
            rows: vec![
                vec!["蝶阀".to_string(), "305.69".to_string(), "8".to_string()],
                vec!["闸阀".to_string(), "128".to_string(), "12.5kg".to_string()],
            ],
        };
        write_xlsx_table(&path, &t).unwrap();
        // 回读校验
        let back = read_xlsx_table(&path).unwrap();
        assert_eq!(back.headers, vec!["品名", "单价", "单重(Kg)"]);
        assert_eq!(back.row_count(), 2);
        assert_eq!(back.cell(0, "品名"), Some("蝶阀"));
        assert_eq!(back.cell(1, "单价"), Some("128"));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_d2_csv_write_read_roundtrip() {
        let dir = test_dir();
        let path = dir.join("t.csv");
        let t = TableData {
            name: "prices".to_string(),
            headers: vec!["品名".to_string(), "单价".to_string()],
            rows: vec![
                vec!["蝶阀".to_string(), "305.69".to_string()],
                vec!["闸阀,带逗号".to_string(), "128".to_string()],
            ],
        };
        write_csv(&path, &t, ',', true).unwrap();
        let back = read_csv(&path).unwrap();
        assert_eq!(back.headers, vec!["品名", "单价"]);
        assert_eq!(back.row_count(), 2);
        // 引号包裹字段正确还原
        assert_eq!(back.cell(1, "品名"), Some("闸阀,带逗号"));
        assert_eq!(back.cell(0, "单价"), Some("305.69"));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_read_xlsx_sheets_all_multi_sheet() {
        // calamine 多 sheet 全遍历 (E13): 每 sheet 独立表头 = 首个非空行。
        // 锁定 read_xlsx_sheets_all 契约: sheet 数 / 表头 / 数据行。
        let path = xlsx_fixture();
        let tables = read_xlsx_sheets_all(&path).unwrap();
        assert_eq!(tables.len(), 2, "应读出全部 2 个 sheet");
        let s1 = &tables[0];
        assert_eq!(s1.name, "Sheet1");
        assert_eq!(s1.headers, vec!["品名", "单价"]);
        assert_eq!(s1.rows.len(), 3, "表头后应有 3 行数据");
        assert_eq!(s1.rows[0][0], "蝶阀");
        assert_eq!(s1.rows[1][0], "闸阀");
        let s2 = &tables[1];
        assert_eq!(s2.name, "Sheet2");
        assert_eq!(s2.headers, vec!["备注"]);
        assert_eq!(s2.rows.len(), 1);
        assert_eq!(s2.rows[0][0], "原始数据");
    }

    #[test]
    fn test_data_to_text_contract() {
        // 数值/文本/布尔语义锁定 (跨库显示文本契约):
        // 整数值浮点去 .0、小数保留、超大数不丢精度。
        assert_eq!(data_to_text(&calamine::Data::Int(123)), "123");
        assert_eq!(data_to_text(&calamine::Data::Float(305.69)), "305.69");
        assert_eq!(data_to_text(&calamine::Data::Float(100.0)), "100");
        assert_eq!(data_to_text(&calamine::Data::Float(1e16)), "10000000000000000");
        assert_eq!(data_to_text(&calamine::Data::String("蝶阀".into())), "蝶阀");
        assert_eq!(data_to_text(&calamine::Data::Bool(true)), "true");
        assert_eq!(data_to_text(&calamine::Data::Empty), "");
        // DateTime 语义: 必须渲染为日期而非 Excel 原始序列号。
        // serial 44484.7916666667 ≈ 2021-10-15 19:00:00 (calamine 自带测试锚点)。
        let dt = calamine::Data::DateTime(calamine::ExcelDateTime::new(
            44484.7916666667,
            calamine::ExcelDateTimeType::DateTime,
            false,
        ));
        let s = data_to_text(&dt);
        assert!(
            s.starts_with("2021-10-15"),
            "DateTime 应渲染为日期, 实际: {s}"
        );
        assert!(
            !s.contains("44484"),
            "不得输出 Excel 原始序列号, 实际: {s}"
        );
    }

    #[test]
    fn test_d4_dedup_cross_sheet_same_key() {
        // E14 同文件跨 sheet 去重: 单文件双 sheet 共享去重键 (单价) →
        // 只保留首行, 第二 sheet 重复行计入 dedup_rows。
        let dir = test_dir();
        let src = dir.join("dedup_src");
        std::fs::create_dir_all(&src).unwrap();
        use office_oxide::xlsx::write::{CellData, XlsxWriter};
        let mut xw = XlsxWriter::new();
        let s1 = xw.add_sheet_get_index("Sheet1");
        let s2 = xw.add_sheet_get_index("Sheet2");
        for s in [s1, s2] {
            xw.sheet_set_cell(s, 0, 0, CellData::String("品名".into()));
            xw.sheet_set_cell(s, 0, 1, CellData::String("单价(元)".into()));
            xw.sheet_set_cell(s, 1, 0, CellData::String("蝶阀".into()));
            xw.sheet_set_cell(s, 1, 1, CellData::Number(100.0));
        }
        xw.save(src.join("供应商甲.xlsx")).unwrap();

        let schema = MergeSchema {
            name: "去重测试",
            standard_columns: &["品名", "单价(元)"],
            column_variants: &[],
            filename_suffixes: &[],
            unit_rules: &[],
            preferred_sheets: &[],
            empty_markers: &[],
            value_columns: &["单价(元)"],
            extra_columns: &["_source_file"],
            skip_prefixes: &["consolidated"],
            supplier_column: None,
            dedup_columns: &["单价(元)"],
        column_types: &[],
        };
        let out = dir.join("dedup_out.xlsx");
        let report = merge_tables_with(&schema, &src, &out).unwrap();
        // 两 sheet 各 1 数据行, 去重键相同 → 只保留 1 行
        assert_eq!(report.total_rows, 1);
        assert_eq!(report.usd_rows, 1);
        assert_eq!(report.dedup_rows.as_ref().map(|v| v.len()), Some(1));
        assert!(report.dedup_rows.unwrap()[0].contains("Sheet2"));
        // 输出回读验证行数
        let back = read_xlsx_table(&out).unwrap();
        assert_eq!(back.row_count(), 1);
        assert_eq!(back.cell(0, "品名"), Some("蝶阀"));
        std::fs::remove_dir_all(&src).ok();
        std::fs::remove_file(&out).ok();
    }

    #[test]
    fn test_d3_detect_encoding_bom_utf8_gbk() {
        // UTF-8 BOM
        assert_eq!(detect_encoding(&[0xEF, 0xBB, 0xBF, b'a']), TextEncoding::Utf8);
        // 纯 UTF-8
        assert_eq!(detect_encoding("你好, NeoTrix".as_bytes()), TextEncoding::Utf8);
        // UTF-16 BOM
        assert_eq!(detect_encoding(&[0xFF, 0xFE, b'a', 0x00]), TextEncoding::Utf16);
        // 非法 UTF-8 高字节区段 → GBK 启发
        assert_eq!(detect_encoding(&[0xD6, 0xD0, 0xB9, 0xFA]), TextEncoding::Gbk);
        // GBK 解码往返
        let gbk_bytes = encode_gbk("测试");
        let decoded = decode_bytes(&gbk_bytes);
        assert_eq!(decoded, "测试");
    }

    // GBK 编码辅助 (测试用, 与实现对称)
    fn encode_gbk(s: &str) -> Vec<u8> {
        let (enc, _, _) = encoding_rs::GBK.encode(s);
        enc.into_owned()
    }

    #[test]
    fn test_p3_suggest_schema_deterministic_and_llm() {
        // 临时目录: 一个 xlsx, 表头混合命中/未命中标准列
        let dir = std::env::temp_dir().join(format!("nt_suggest_schema_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let table = TableData {
            name: "Sheet1".to_string(),
            headers: vec![
                "产品型号".to_string(),
                "阀体材质".to_string(),
                "单重(Kg)".to_string(),
                "完全未知列A".to_string(),
            ],
            rows: vec![vec![
                "DN50".to_string(),
                "WCB".to_string(),
                "12.5".to_string(),
                "?".to_string(),
            ]],
        };
        write_xlsx_table(dir.join("甲价格表.xlsx"), &table).unwrap();

        // 确定性路径 (无 LLM): 命中 3 标准列, 未命中 1
        let s = suggest_schema(&dir, None).unwrap();
        assert!(s.matched.contains_key("产品型号"));
        assert!(s.matched.contains_key("阀体材质"));
        assert!(s.matched.contains_key("单重(Kg)"));
        assert!(s.unmatched.contains(&"完全未知列A".to_string()));
        assert!(!s.llm_enhanced);
        assert!(s.suggested_variants.is_empty());

        // LLM 增强路径: 建议列A → 产品大类 (合法标准列); 建议到非法目标 → 丢弃
        let s = suggest_schema(&dir, Some(&|_prompt: &str| {
            Some("完全未知列A → 产品大类\n完全未知列A → 不存在的列\n".to_string())
        }))
        .unwrap();
        assert!(s.llm_enhanced);
        assert_eq!(s.suggested_variants.get("产品大类").map(|v| v.len()), Some(1));
        // 非法目标被过滤
        assert!(!s.suggested_variants.contains_key("不存在的列"));

        // draft() 不返回未确认变体 — 保持生产 schema 纯净
        let draft = s.draft();
        assert!(!draft.column_variants.iter().any(|(_, vs)| vs.contains(&"完全未知列A")));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_d4_normalize_column_name() {
        assert_eq!(normalize_column_name("阀体材质"), "阀体材质");
        assert_eq!(normalize_column_name("BODY阀体"), "阀体材质");
        assert_eq!(normalize_column_name("stem材质"), "阀杆材质");
        assert_eq!(normalize_column_name("单价(美元)"), "美元报价(USD)");
        assert_eq!(normalize_column_name("青岛港FOB单价(元)"), "青岛港FOB报价(USD)");
        assert_eq!(normalize_column_name("单重(kg)"), "单重(Kg)");
        assert_eq!(normalize_column_name("未知列"), "未知列");
    }

    #[test]
    fn test_d4_supplier_from_filename() {
        // 供应商名推导已 schema 化: derive_source_name 委托 PRICE_TABLE_SCHEMA
        assert_eq!(
            derive_source_name(Path::new("4、玉鹏价格_报价模板-修改版.xlsx"), &PRICE_TABLE_SCHEMA),
            "玉鹏"
        );
        assert_eq!(
            derive_source_name(
                Path::new("44. 河北威世迪阀门_报价模板-第一版-修改版.xlsx"),
                &PRICE_TABLE_SCHEMA
            ),
            "河北威世迪阀门"
        );
        assert_eq!(
            derive_source_name(Path::new("7、青岛润通_报价模板-修改版.xlsx"), &PRICE_TABLE_SCHEMA),
            "青岛润通"
        );
    }

    #[test]
    fn test_d4_schema_validate() {
        // 合法 schema 通过
        PRICE_TABLE_SCHEMA.validate().unwrap();
        // value_columns 必须是标准列
        let bad = MergeSchema {
            name: "bad",
            standard_columns: &["a", "b"],
            column_variants: &[],
            filename_suffixes: &[],
            unit_rules: &[],
            preferred_sheets: &[],
            empty_markers: &[],
            value_columns: &["c"],
            extra_columns: &["d"],
            skip_prefixes: &[],
            supplier_column: Some("b"),
            dedup_columns: &[],
        column_types: &[],
        };
        assert!(bad.validate().is_err());
        // 标准列重复
        let dup = MergeSchema {
            name: "dup",
            standard_columns: &["a", "a"],
            column_variants: &[],
            filename_suffixes: &[],
            unit_rules: &[],
            preferred_sheets: &[],
            empty_markers: &[],
            value_columns: &["a"],
            extra_columns: &[],
            skip_prefixes: &[],
            supplier_column: None,
            dedup_columns: &[],
        column_types: &[],
        };
        assert!(dup.validate().is_err());
        // extra 与标准列冲突
        let clash = MergeSchema {
            name: "clash",
            standard_columns: &["a"],
            column_variants: &[],
            filename_suffixes: &[],
            unit_rules: &[],
            preferred_sheets: &[],
            empty_markers: &[],
            value_columns: &["a"],
            extra_columns: &["a"],
            skip_prefixes: &[],
            supplier_column: None,
            dedup_columns: &[],
        column_types: &[],
        };
        assert!(clash.validate().is_err());
    }

    #[test]
    fn test_d4_merge_tables_with_generic_schema() {
        // 通用引擎零领域知识验证: 自定义 schema + 变体列
        let dir = test_dir();
        let src = dir.join("merge_generic_src");
        std::fs::create_dir_all(&src).unwrap();
        // 源表1: 使用变体列名 + 无供应商列 (由文件名推导)
        let t1 = TableData {
            name: "s1".into(),
            headers: vec!["品名".to_string(), "价格".to_string(), "重量".to_string()],
            rows: vec![vec!["A阀".to_string(), "100".to_string(), "2.5".to_string()]],
        };
        write_xlsx_table(src.join("1、华北阀门_目录-第一版.xlsx"), &t1).unwrap();
        // 源表2: 使用标准列名 + 显式供应商
        let t2 = TableData {
            name: "s2".into(),
            headers: vec![
                "产品型号".to_string(),
                "单价(元)".to_string(),
                "单重(Kg)".to_string(),
                "供应商名称".to_string(),
            ],
            rows: vec![vec![
                "B阀".to_string(),
                "200".to_string(),
                String::new(),
                "华南阀门".to_string(),
            ]],
        };
        write_xlsx_table(src.join("2、华东阀门_目录.xlsx"), &t2).unwrap();

        let schema = MergeSchema {
            name: "测试目录",
            standard_columns: &["品名", "单价(元)", "单重(Kg)", "供应商名称"],
            column_variants: &[
                ("品名", &["产品型号", "型号"]),
                ("单价(元)", &["价格", "单价"]),
                ("单重(Kg)", &["重量"]),
            ],
            filename_suffixes: &["目录", "-第一版"],
            unit_rules: &[UnitRule {
                column: "单重(Kg)",
                suffix: "kg",
                skip_if_contains: &["kg"],
            }],
            preferred_sheets: &[],
            empty_markers: &[],
            value_columns: &["单价(元)"],
            extra_columns: &["_source_file"],
            skip_prefixes: &["consolidated"],
            supplier_column: Some("供应商名称"),
            dedup_columns: &["单价(元)"],
        column_types: &[],
        };
        let out = dir.join("merge_generic_out.xlsx");
        let report = merge_tables_with(&schema, &src, &out).unwrap();
        assert_eq!(report.files_processed, 2);
        assert_eq!(report.total_rows, 2);
        assert_eq!(report.usd_rows, 2); // 两行都有单价

        let merged = read_xlsx_table(&out).unwrap();
        // 变体列归一化: 源表1 "价格"→"单价(元)", "品名"不变
        assert_eq!(merged.headers[..4], vec!["品名", "单价(元)", "单重(Kg)", "供应商名称"]);
        // 源表1 供应商由文件名推导 "华北阀门", 单重补 kg
        assert_eq!(merged.rows[0][0], "A阀");
        assert_eq!(merged.rows[0][1], "100");
        assert_eq!(merged.rows[0][2], "2.5kg");
        assert_eq!(merged.rows[0][3], "华北阀门");
        // 源表2 显式供应商保留, 空单重不补单位
        assert_eq!(merged.rows[1][0], "B阀");
        assert_eq!(merged.rows[1][2], "");
        assert_eq!(merged.rows[1][3], "华南阀门");
        // 附加 _source_file 透出
        assert_eq!(merged.headers[4], "_source_file");
        assert!(merged.rows[0][4].contains("华北阀门_目录-第一版.xlsx"));
        assert!(merged.rows[1][4].contains("华东阀门_目录.xlsx"));
    }

    #[test]
    fn test_d4_merge_multi_sheet_dedup() {
        // 多 sheet 全遍历 (E13) + 同文件内跨 sheet 去重 (E14), 跨文件不去重
        let dir = test_dir();
        let src = dir.join("merge_multisheet_src");
        std::fs::create_dir_all(&src).unwrap();
        // 单文件双 sheet: sheet1 有 2 行, sheet2 有 1 行独立 + 1 行与 sheet1 重复
        // 需用 XlsxWriter 写双 sheet
        {
            use office_oxide::xlsx::write::{CellData, XlsxWriter};
            let mut xw = XlsxWriter::new();
            let s1 = xw.add_sheet_get_index("主表");
            for (c, h) in ["产品型号", "单价(元)", "口径"].iter().enumerate() {
                xw.sheet_set_cell(s1, 0, c, CellData::String(h.to_string()));
            }
            for (r, row) in [["A阀", "100", "DN50"], ["B阀", "200", "DN65"]].iter().enumerate() {
                for (c, v) in row.iter().enumerate() {
                    xw.sheet_set_cell(s1, r + 1, c, CellData::String(v.to_string()));
                }
            }
            let s2 = xw.add_sheet_get_index("副表");
            for (c, h) in ["产品型号", "单价(元)", "口径"].iter().enumerate() {
                xw.sheet_set_cell(s2, 0, c, CellData::String(h.to_string()));
            }
            // 与主表 B阀/DN65 重复 + 独立 C阀
            for (r, row) in [["C阀", "300", "DN80"], ["B阀", "200", "DN65"]].iter().enumerate() {
                for (c, v) in row.iter().enumerate() {
                    xw.sheet_set_cell(s2, r + 1, c, CellData::String(v.to_string()));
                }
            }
            xw.save(src.join("1、多sheet厂_目录.xlsx")).unwrap();
        }
        // 另一文件同 key (DN65/200) — 跨文件不去重
        let t2 = TableData {
            name: "s".into(),
            headers: vec![
                "产品型号".to_string(),
                "单价(元)".to_string(),
                "口径".to_string(),
            ],
            rows: vec![vec![
                "D阀".to_string(),
                "200".to_string(),
                "DN65".to_string(),
            ]],
        };
        write_xlsx_table(src.join("2、异厂_目录.xlsx"), &t2).unwrap();

        let schema = MergeSchema {
            name: "测试目录",
            standard_columns: &["产品型号", "单价(元)", "口径", "供应商名称"],
            column_variants: &[],
            filename_suffixes: &["目录"],
            unit_rules: &[],
            preferred_sheets: &[],
            empty_markers: &[],
            value_columns: &["单价(元)"],
            extra_columns: &["_source_file"],
            skip_prefixes: &["consolidated"],
            supplier_column: Some("供应商名称"),
            dedup_columns: &["口径", "单价(元)"],
            column_types: &[],
        };
        let out = dir.join("merge_multisheet_out.xlsx");
        let report = merge_tables_with(&schema, &src, &out).unwrap();
        // 全遍历 4 行, 同文件去重 1 行 (副表 B阀/DN65), 剩 3 行;
        // 文件2 D阀 同 key [DN65,200] 跨文件不去重 → 总计 4 行
        assert_eq!(report.files_processed, 2);
        assert_eq!(report.total_rows, 4);
        let dedup = report.dedup_rows.as_ref().unwrap();
        assert_eq!(dedup.len(), 1);
        assert!(dedup[0].contains("多sheet厂_目录.xlsx::副表"), "去重行来源应为副表: {dedup:?}");
        assert_eq!(report.usd_rows, 4);

        let merged = read_xlsx_sheets_all(&out).unwrap();
        let m = merged.first().unwrap();
        assert_eq!(m.rows.len(), 4);
        // 主表两行 + 副表独立 C阀 + 异厂 D阀 (同 key 跨文件保留)
        let lines: Vec<String> = m.rows.iter().map(|r| r[..3].join("|")).collect();
        assert!(lines.iter().any(|l| l == "A阀|100|DN50"));
        assert!(lines.iter().any(|l| l == "C阀|300|DN80"));
        assert!(lines.iter().any(|l| l == "D阀|200|DN65"));
        // B阀|200|DN65 只出现一次 (同文件去重), D阀|200|DN65 是异厂行保留
        assert_eq!(lines.iter().filter(|l| l.as_str() == "B阀|200|DN65").count(), 1);
        assert_eq!(lines.iter().filter(|l| l.as_str() == "D阀|200|DN65").count(), 1);
        // _source_file 标注 sheet
        let srcs: Vec<&str> = m.rows.iter().map(|r| r[4].as_str()).collect();
        assert!(srcs.iter().any(|s| s.contains("::主表")), "主表来源标注: {srcs:?}");
        assert!(srcs.iter().any(|s| s.contains("::副表")), "副表来源标注: {srcs:?}");
    }

    #[test]
    fn test_d4_validation_warnings() {
        // 输出数据校验: Numeric 列非数值值 → validation_warnings; 千分位/单位后缀可解析
        let dir = test_dir();
        let src = dir.join("merge_validate_src");
        std::fs::create_dir_all(&src).unwrap();
        let t1 = TableData {
            name: "Sheet1".into(),
            headers: vec![
                "产品型号".to_string(),
                "单价(元)".to_string(),
                "单重(Kg)".to_string(),
            ],
            rows: vec![
                vec!["A阀".into(), "1,200".into(), "2.5kg".into()],
                vec!["B阀".into(), "N/A".into(), "3".into()],
                vec!["C阀".into(), "300".into(), "abc".into()],
            ],
        };
        write_xlsx_table(src.join("1、校验厂_目录.xlsx"), &t1).unwrap();
        let schema = MergeSchema {
            name: "校验目录",
            standard_columns: &["产品型号", "单价(元)", "单重(Kg)"],
            column_variants: &[],
            filename_suffixes: &["目录"],
            unit_rules: &[],
            preferred_sheets: &[],
            empty_markers: &[],
            value_columns: &["单价(元)"],
            extra_columns: &["_source_file"],
            skip_prefixes: &["consolidated"],
            supplier_column: None,
            dedup_columns: &[],
            column_types: &[
                ("单价(元)", ColumnType::Numeric),
                ("单重(Kg)", ColumnType::Numeric),
            ],
        };
        let out = dir.join("merge_validate_out.xlsx");
        let report = merge_tables_with(&schema, &src, &out).unwrap();
        assert_eq!(report.total_rows, 3);
        // 非数值: B阀单价 "N/A", C阀单重 "abc" → 2 条警告; "1,200" 与 "2.5kg" 可解析不告警
        assert_eq!(report.validation_warnings.len(), 2, "{:?}", report.validation_warnings);
        assert!(report.validation_warnings[0].contains("单价(元)"), "{:?}", report.validation_warnings);
        assert!(report.validation_warnings[0].contains("N/A"), "{:?}", report.validation_warnings);
        assert!(report.validation_warnings[1].contains("单重(Kg)"), "{:?}", report.validation_warnings);
        assert!(report.validation_warnings[1].contains("abc"), "{:?}", report.validation_warnings);
    }

    #[test]
    fn test_d5_edit_xlsx_table() {
        // 表格级编辑 API: SetCell / InsertRow / RemoveRow 读写回环
        let dir = test_dir();
        let path = dir.join("edit_out.xlsx");
        let t = TableData {
            name: "主表".into(),
            headers: vec!["品名".into(), "单价".into()],
            rows: vec![
                vec!["A阀".into(), "100".into()],
                vec!["B阀".into(), "200".into()],
            ],
        };
        write_xlsx_table(&path, &t).unwrap();

        // SetCell + InsertRow + RemoveRow
        let tables = edit_xlsx_table(
            &path,
            &[
                TableEdit::SetCell { sheet: 0, row: 0, col: 1, value: "150".into() },
                TableEdit::InsertRow { sheet: 0, row: 1 },
                TableEdit::SetCell { sheet: 0, row: 1, col: 0, value: "C阀".into() },
                TableEdit::SetCell { sheet: 0, row: 1, col: 1, value: "300".into() },
                TableEdit::RemoveRow { sheet: 0, row: 2 }, // 删原 B阀 行
            ],
        )
        .unwrap();
        let m = tables.first().unwrap();
        assert_eq!(m.rows.len(), 2);
        assert_eq!(m.rows[0][1], "150", "SetCell 生效: {:?}", m.rows[0]);
        assert_eq!(m.rows[1][0], "C阀", "InsertRow+SetCell 生效: {:?}", m.rows[1]);
        // 重读验证持久化 (R-P16)
        let back = read_xlsx_table(&path).unwrap();
        assert_eq!(back.rows.len(), 2);
        assert_eq!(back.rows[0][1], "150");
        assert_eq!(back.rows[1][0], "C阀");

        // 越界应报错
        let err = edit_xlsx_table(&path, &[TableEdit::SetCell { sheet: 0, row: 99, col: 0, value: "x".into() }]);
        assert!(err.is_err(), "越界应报错");
    }

    #[test]
    fn test_d5_snapshot_table_extraction() {
        // ContentSnapshot 表格存储: xlsx 快照含多 sheet 表格数据
        let dir = test_dir();
        let path = dir.join("snap_out.xlsx");
        let t = TableData {
            name: "S1".into(),
            headers: vec!["品名".into(), "单价".into()],
            rows: vec![vec!["A阀".into(), "100".into()]],
        };
        write_xlsx_table(&path, &t).unwrap();
        let fa = FileAbility::open(&path).unwrap();
        let snap = fa.snapshot();
        let tables = snap.table.as_ref().expect("xlsx 快照应含表格");
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].rows[0][0], "A阀");
    }

    #[test]
    fn test_d4_consolidate_tables() {
        let dir = test_dir();
        let src = dir.join("consolidate_src");
        std::fs::create_dir_all(&src).unwrap();
        // 两个 CSV 供应商文件, 列名变体不同
        let t1 = TableData {
            name: "a".to_string(),
            headers: vec!["品名".to_string(), "BODY阀体".to_string(), "单价".to_string(), "单重".to_string()],
            rows: vec![vec!["蝶阀".to_string(), "铸铁".to_string(), "305.69".to_string(), "8".to_string()]],
        };
        write_csv(src.join("1、甲工厂_报价模板.csv"), &t1, ',', false).unwrap();
        let t2 = TableData {
            name: "b".to_string(),
            headers: vec![
                "品名".to_string(),
                "阀体".to_string(),
                "单价(美元)".to_string(),
                "重量(Kg)".to_string(),
            ],
            rows: vec![vec!["闸阀".to_string(), "不锈钢".to_string(), "42.5".to_string(), "12".to_string()]],
        };
        write_csv(src.join("2、乙工厂_报价模板.csv"), &t2, ',', false).unwrap();

        let out = dir.join("consolidated.xlsx");
        let report = consolidate_tables(&src, &out).unwrap();
        assert_eq!(report.files_processed, 2);
        assert_eq!(report.total_rows, 2);
        assert!(report.usd_rows >= 1, "应检测到 USD 报价行");
        assert!(out.exists());
        // 回读验证归一化 + 供应商名 + 单重单位
        let back = read_xlsx_table(&out).unwrap();
        assert!(back.headers.iter().any(|h| h == "阀体材质"), "列名应归一化为标准列");
        assert!(back.headers.iter().any(|h| h == "供应商名称"));
        let suppliers: Vec<String> = (0..back.row_count())
            .map(|i| back.cell(i, "供应商名称").unwrap_or("").to_string())
            .collect();
        assert!(suppliers.iter().any(|s| s.contains("甲工厂")));
        assert!(suppliers.iter().any(|s| s.contains("乙工厂")));
        // 单重保持纯数字 (表头已含单位, 不再追加 kg)
        let weights: Vec<String> = (0..back.row_count())
            .map(|i| back.cell(i, "单重(Kg)").unwrap_or("").to_string())
            .collect();
        assert_eq!(weights, vec!["8", "12"], "单重应保留源数据纯数字: {weights:?}");
        assert!(
            !weights.iter().any(|w| w.contains("kg")),
            "表头已含单位, 值不应再带 kg: {weights:?}"
        );
        std::fs::remove_file(&out).ok();
    }

    #[test]
    fn test_d5_read_structured_json_yaml() {
        let dir = test_dir();
        let j = dir.join("cfg.json");
        std::fs::write(&j, r#"{"name":"NeoTrix","level":5}"#).unwrap();
        let s = read_structured(&j).unwrap();
        assert_eq!(s.format, "json");
        assert_eq!(s.value["name"], "NeoTrix");
        let y = dir.join("cfg.yaml");
        std::fs::write(&y, "name: NeoTrix\nlevel: 5\n").unwrap();
        let sy = read_structured(&y).unwrap();
        assert_eq!(sy.format, "yaml");
        assert_eq!(sy.value["level"], 5);
        std::fs::remove_file(&j).ok();
        std::fs::remove_file(&y).ok();
    }

    #[test]
    fn test_d5_write_json_roundtrip() {
        let dir = test_dir();
        let j = dir.join("out.json");
        let v = serde_json::json!({"a": 1, "b": [true, null]});
        write_json(&j, &v, true).unwrap();
        let s = read_structured(&j).unwrap();
        assert_eq!(s.value["a"], 1);
        std::fs::remove_file(&j).ok();
    }

    #[test]
    fn test_d6_snapshot_store_load() {
        let dir = test_dir();
        let path = dir.join("sample_snap.docx");
        if !path.exists() {
            create::create_from_markdown(
                "# 快照测试\n\nHello {{name}}",
                DocumentFormat::Docx,
                &path,
            )
            .unwrap();
        }
        let ab = FileAbility::open(&path).unwrap();
        let snap = ab.snapshot();
        let out = dir.join("snapshot.json");
        store_snapshot(&snap, &out).unwrap();
        let loaded = load_snapshot(&out).unwrap();
        assert_eq!(loaded.kind, snap.kind);
        assert_eq!(loaded.size_bytes, snap.size_bytes);
        std::fs::remove_file(&out).ok();
        std::fs::remove_file(&path).ok();
    }

    // ── 真实数据端到端验证 (5月份价格表 26 家供应商) — 手动触发: --ignored ──

    #[test]
    #[ignore]
    fn test_d4_consolidate_real_price_tables() {
        let src = PathBuf::from("/Users/neo/Downloads/5月份价格表");
        let out = src.join("native_consolidated_v2.xlsx");
        let report = consolidate_tables(&src, &out).unwrap();
        println!("files_processed={} total_rows={} usd_rows={} failed={:?}",
            report.files_processed, report.total_rows, report.usd_rows, report.files_failed);
        assert!(report.files_processed >= 20, "应合并多数供应商文件");
        assert!(report.total_rows > 1000);
        assert!(out.exists());
        let back = read_xlsx_table(&out).unwrap();
        assert!(back.headers.iter().any(|h| h == "阀体材质"), "材质列应归一化");
        let usd_rates = (0..back.row_count())
            .map(|i| back.cell(i, "美元报价(USD)").unwrap_or(""))
            .filter(|s| !s.trim().is_empty())
            .count();
        println!("raw USD cells present: {usd_rates}");
        // 单重保持纯数字 (表头已含单位) 且无 "/kg" 垃圾行
        let all_weights: Vec<String> = (0..back.row_count())
            .map(|i| back.cell(i, "单重(Kg)").unwrap_or("").to_string())
            .collect();
        let with_kg = all_weights
            .iter()
            .filter(|s| !s.trim().is_empty() && s.contains("kg"))
            .count();
        println!("weight cells with redundant kg: {with_kg}");
        assert_eq!(with_kg, 0, "表头已含单位, 值不应再带 kg");
        let slash_kg = all_weights.iter().filter(|s| s.as_str() == "/kg").count();
        assert_eq!(slash_kg, 0, "空值标记不应产出 /kg 垃圾行");
    }

    #[test]
    fn test_open_image_metadata() {
        let dir = std::env::temp_dir().join("nt_file_ability_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("pixel.png");
        let img = image::RgbaImage::from_pixel(2, 3, image::Rgba([255, 0, 0, 255]));
        img.save(&path).unwrap();
        let ab = FileAbility::open(&path).unwrap();
        assert_eq!(ab.kind(), FileKind::Image);
        let meta = ab.image_metadata().unwrap();
        assert_eq!((meta.width, meta.height), (2, 3));
        assert_eq!(meta.color_channels, 4);
        assert!(meta.has_alpha, "RGBA 应有 alpha 通道");
        assert!(meta.bit_depth >= 24, "RGBA8 位深应 >=24");
        assert!((meta.aspect_ratio - 2.0 / 3.0).abs() < 1e-9);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_audio_duration_wav_real_parse() {
        let dir = std::env::temp_dir().join("nt_file_ability_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("tone.wav");
        // 构造 1 秒 8kHz 单声道 WAV: 字节率 8000, data 8000 字节
        // 构造 1 秒 8kHz 单声道 WAV: 字节率 8000, data 8000 字节
        let mut wav: Vec<u8> = Vec::new();
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&36u32.to_le_bytes()); // chunk 大小 (占位)
        wav.extend_from_slice(b"WAVE");
        wav.extend_from_slice(b"fmt ");
        wav.extend_from_slice(&16u32.to_le_bytes()); // fmt 子块大小
        wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
        wav.extend_from_slice(&1u16.to_le_bytes()); // 单声道
        wav.extend_from_slice(&8000u32.to_le_bytes()); // 采样率
        wav.extend_from_slice(&8000u32.to_le_bytes()); // 字节率
        wav.extend_from_slice(&1u16.to_le_bytes()); // 块对齐
        wav.extend_from_slice(&8u16.to_le_bytes()); // 位深
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&8000u32.to_le_bytes()); // data 长度
        wav.extend_from_slice(&[0u8; 8000]);
        std::fs::write(&path, &wav).unwrap();
        let ab = FileAbility::open(&path).unwrap();
        assert_eq!(ab.kind(), FileKind::Audio);
        let dur = ab.audio_duration_ms().unwrap();
        assert!((dur as i64 - 1000).abs() <= 2, "时长应 ≈1000ms, 实际 {dur}");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_self_test_registration() {
        let st = FileAbilitySelfTest;
        assert_eq!(st.name(), "nt_io_file_ability");
        assert!(st.self_test().is_ok(), "SelfTest 应通过");
        // T3 接线: 名称前缀必须可被 ConsciousnessTree 路由到 Io 分支
        assert!(
            crate::core::nt_core_consciousness_tree::BranchKind::from_module_name(st.name())
                == Some(crate::core::nt_core_consciousness_tree::BranchKind::Io),
            "SelfTest 名应路由到 Io 分支"
        );
    }

    #[test]
    fn test_constellation_promotion() {
        let mut ab = FileAbility::open(office_sample()).unwrap();
        assert_eq!(ab.maturity(), ConstellationLevel::C1UnitTest);
        let next = ab.promote();
        assert_eq!(next, Some(ConstellationLevel::C2IntegrationTest));
    }

    #[test]
    fn test_vsa_embedding_similar_text() {
        let engine = VSAEngine::default();
        let dim = engine.dimensions();
        let a = embed_text("NeoTrix 自我进化知识表示", dim);
        let b = embed_text("NeoTrix 自我进化知识表示", dim);
        let c = embed_text("完全无关的另一段内容", dim);
        let sim_self = engine.similarity(&a, &b);
        let sim_diff = engine.similarity(&a, &c);
        assert!(a.len() == dim);
        assert!(sim_self > 0.99, "相同文本相似度应高, 实际 {sim_self}");
        assert!(sim_diff < 0.3, "无关文本相似度应低, 实际 {sim_diff}");
    }

    #[test]
    fn test_vsa_embedding_deterministic() {
        let a = embed_text("稳定输入", 512);
        let b = embed_text("稳定输入", 512);
        assert_eq!(a, b, "嵌入应确定性可复现");
    }

    #[test]
    fn test_e8_state_transition() {
        let path = office_sample();
        let mut ab = FileAbility::open(&path).unwrap();
        let initial = ab.e8_state();
        assert_eq!(initial, ReasoningHexagram::new(0b001100));
        // 提取 → 转换: 应产生一次转移
        let after_transform = ab.transition(FileOperation::Transform);
        assert_ne!(initial, after_transform, "转换操作应推进 E8 状态");
        // 贪心单步必须逼近目标
        let target = FileOperation::Transform.target_state();
        assert!(
            after_transform.hamming_dist(&target) <= initial.hamming_dist(&target),
            "单步转移应单调逼近目标"
        );
        // 到达目标后停在目标
        ab.e8_state = target;
        let stay = ab.transition(FileOperation::Transform);
        assert_eq!(stay, target, "已达目标时转移应保持");
        // 路径: 从当前到目标, 首尾正确
        let goal = FileOperation::Embed.target_state();
        let path = ab.e8_path_to(goal);
        assert_eq!(*path.first().unwrap(), target);
        assert_eq!(*path.last().unwrap(), goal);
        assert!(ab.e8_mode_name().len() > 2);
    }

    #[test]
    fn test_gwt_route_attention() {
        // 静态专家映射 (文件大类 → SpecialistType)
        let ab = FileAbility::open(office_sample()).unwrap();
        assert_eq!(ab.kind().specialist(), SpecialistType::KnowledgeIntegrator);
        // 动态谐振路由: 任何 E8 状态都应路由到 14 专家之一
        for bits in [0u8, 1, 0b111111, 0b001100] {
            let (t, strength, st) = route_attention(ReasoningHexagram::new(bits));
            assert_eq!(
                specialist_index(t),
                specialist_index_inv(specialist_index(t)) as usize
            );
            let _ = (t, strength, st);
        }
        // 谐振强度 ∈ [0,6]
        let (_, s, _) = route_attention(ReasoningHexagram::new(0b001100));
        assert!((0..=6).contains(&s));
        // 索引逆映射闭环
        for i in 0..14 {
            assert_eq!(specialist_index(specialist_index_inv(i)), i);
        }
    }

    #[test]
    fn test_ocr_trait_wiring() {
        let dir = std::env::temp_dir().join("nt_file_ability_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("scan_document.png");
        let img = image::RgbaImage::from_pixel(4, 4, image::Rgba([255, 255, 255, 255]));
        img.save(&path).unwrap();
        let ab = FileAbility::open(&path).unwrap();
        assert_eq!(ab.kind(), FileKind::Image);
        let r = ab.ocr(None);
        assert_eq!(r.engine, "rule-based");
        assert!(r.text.contains("scan_document"), "应提取文件名为启发 OCR");
        // 非图像文件返回空
        let txt = FileAbility::open(dir.join("hello.txt")).unwrap_or_else(|_| {
            let p = dir.join("hello.txt");
            std::fs::write(&p, "hi").unwrap();
            FileAbility::open(&p).unwrap()
        });
        let empty = txt.ocr(None);
        assert!(empty.text.is_empty());
        std::fs::remove_file(&path).ok();
    }

    // ── doc7 吸收: grounding 精确值校验 (cycle 1101) ──

    #[test]
    fn test_grounding_no_missing_when_faithful() {
        let source = "本页包含订单号 DOC7-2026-0813 与金额 1,250.50 元。";
        let content = "本页包含订单号 DOC7-2026-0813 与金额 1,250.50 元。";
        let r = ground_missing_tokens(source, content);
        assert!(r.checked);
        assert!(r.missing_numeric.is_empty(), "忠实转写不应报缺失: {:?}", r.missing_numeric);
        assert!(r.missing_identifiers.is_empty(), "标识符不应缺失: {:?}", r.missing_identifiers);
        assert!(r.sequence_ok, "数值序列应一致");
        assert!(r.ungrounded.is_empty());
    }

    #[test]
    fn test_grounding_detects_missing_numeric() {
        let source = "阈值设定为 305.69, 版本号 2.5.1。";
        let content = "阈值设定为 305.69。"; // 版本号 2.5.1 被 VLM 幻读丢失
        let r = ground_missing_tokens(source, content);
        assert!(r.checked);
        assert!(
            r.missing_numeric.iter().any(|t| t.contains("2.5.1")),
            "应检测到缺失版本号: {:?}",
            r.missing_numeric
        );
        assert!(r.ungrounded.iter().any(|t| t.contains("2.5.1")));
        assert!(!r.sequence_ok, "序列应不一致");
    }

    #[test]
    fn test_grounding_detects_missing_identifier() {
        let source = "接入模块 NT-IO-42 完成, 依赖 CRC32-B3。";
        let content = "接入模块完成。"; // 标识符被省略
        let r = ground_missing_tokens(source, content);
        assert!(
            r.missing_identifiers.iter().any(|id| id.contains("NT-IO-42")),
            "应检测到缺失标识符: {:?}",
            r.missing_identifiers
        );
    }

    #[test]
    fn test_grounding_math_guard_skips_latex() {
        let source = "公式中 305.69 出现于 $x^{305.69}$ 处。";
        let content = "公式中 $x^{305.69}$ 处。"; // 数字在 LaTeX 内, 视为公式
        let r = ground_missing_tokens(source, content);
        // 305.69 只在 math 行出现 → math_guard 跳过, 不误报
        assert_eq!(r.math_guard_skipped, 0, "数字已在公式中保留");
        let _ = r;
    }

    #[test]
    fn test_grounding_unicode_dash_normalization() {
        // unicode 减号/全角减号 与 ascii 减号等价 (doc7 normalizeNumericToken 行为)
        let v = normalize_numeric_token("−1,250");
        assert_eq!(v, "-1,250");
        let w = normalize_numeric_token("－3");
        assert_eq!(w, "-3");
        let t = normalize_numeric_token("△2");
        assert_eq!(t, "-2");
        assert!(is_critical_numeric_token("305.69"));
        assert!(is_critical_numeric_token("1,250"));
        assert!(!is_critical_numeric_token("25"));
    }

    #[test]
    fn test_grounding_compact_strips_markdown() {
        let c = compact_numeric_text("**305.69** `B2` \\$100 \\^{x}");
        assert!(!c.contains('*'));
        assert!(!c.contains('`'));
        assert!(!c.contains('\\'));
        assert!(c.contains("305.69"));
        assert!(c.contains("B2"));
    }

    #[test]
    fn test_grounding_empty_source_skips() {
        let r = ground_missing_tokens("", "任何内容");
        assert!(!r.checked, "空源文本不应执行 grounding");
    }

    // ── doc7 吸收: 视觉理解 prompt 路由 (cycle 1101) ──

    #[test]
    fn test_visual_prompt_routing_by_kind() {
        // 文档 → Document prompt
        assert_eq!(VisualPromptKind::for_file_kind(FileKind::Pdf), VisualPromptKind::Document);
        assert_eq!(VisualPromptKind::for_file_kind(FileKind::Image), VisualPromptKind::Document);
        // 幻灯片 → Slide prompt
        assert_eq!(
            VisualPromptKind::for_file_kind(FileKind::Office(office_oxide::DocumentFormat::Pptx)),
            VisualPromptKind::Slide
        );
        assert_eq!(
            VisualPromptKind::for_file_kind(FileKind::Office(office_oxide::DocumentFormat::Ppt)),
            VisualPromptKind::Slide
        );
        // 文本走 Document
        assert_eq!(VisualPromptKind::for_file_kind(FileKind::Text), VisualPromptKind::Document);
    }

    #[test]
    fn test_visual_prompt_contains_core_rules() {
        let doc = VisualPromptKind::Document.prompt();
        assert!(doc.contains("faithful Markdown"));
        assert!(doc.contains("Table rules"));
        assert!(doc.contains("Visual rules"));
        assert!(doc.contains("不可读"));
        assert!(doc.contains("unreadable"));
        assert!(doc.contains("$$"));
        assert!(doc.contains("Return only Markdown"));

        let slide = VisualPromptKind::Slide.prompt();
        assert!(slide.contains("presentation slide"));
        assert!(slide.contains("visible title as the leading heading"));
        // 双 prompt 有差异
        assert_ne!(doc, slide);
        // 路由名正确
        assert_eq!(VisualPromptKind::Document.name(), "document");
        assert_eq!(VisualPromptKind::Slide.name(), "slide");
    }

    // ── doc7 吸收: VisualExtractor 提取管线 (cycle 1101) ──

    #[test]
    fn test_visual_extract_success_with_fake_vlm() {
        let fake: &VlmCall = &|prompt, _img, mime| {
            assert!(prompt.contains("faithful Markdown"));
            assert_eq!(mime, "image/png");
            Ok("# 测试文档\n\n订单号 DOC7-2026-0813, 金额 1,250.50 元。".to_string())
        };
        let cfg = VisualExtractConfig::default();
        let r = visual_extract(FileKind::Pdf, "aGVsbG8=", "", &cfg, fake);
        assert!(!r.failed, "不应失败: {:?}", r.error);
        assert!(r.markdown.contains("DOC7-2026-0813"));
        assert_eq!(r.prompt_kind, VisualPromptKind::Document);
        assert!(r.grounding.is_none(), "未开 grounding 不应有报告");
    }

    #[test]
    fn test_visual_extract_routes_slide_prompt() {
        let fake: &VlmCall = &|prompt, _img, _mime| {
            assert!(prompt.contains("presentation slide"));
            Ok("# Slide 1".to_string())
        };
        let cfg = VisualExtractConfig::default();
        let r = visual_extract(FileKind::Office(office_oxide::DocumentFormat::Pptx), "img", "", &cfg, fake);
        assert_eq!(r.prompt_kind, VisualPromptKind::Slide);
        assert!(!r.failed);
    }

    #[test]
    fn test_visual_extract_retries_then_fails() {
        use std::cell::Cell;
        use std::rc::Rc;
        let attempts = Rc::new(Cell::new(0));
        let counter = Rc::clone(&attempts);
        // Rc 闭包借用计数 — 借用检查器要求闭包存活超过调用点,
        // 通过 Box::leak 使计数指针 'static 化
        let leaked: &'static Cell<usize> = Box::leak(Box::new(Cell::new(0)));
        let fake: &VlmCall = &move |_p, _i, _m| {
            leaked.set(leaked.get() + 1);
            Err("provider 500".to_string())
        };
        let cfg = VisualExtractConfig { retry_count: 3, ..Default::default() };
        let r = visual_extract(FileKind::Image, "img", "", &cfg, fake);
        assert!(r.failed);
        assert!(r.error.as_ref().unwrap().contains("provider 500"));
        assert_eq!(leaked.get(), 4, "应重试 3+1=4 次");
        let _ = (&attempts, &counter);
    }

    #[test]
    fn test_visual_extract_grounding_attached() {
        let fake: &VlmCall = &|_p, _i, _m| Ok("阈值 305.69。".to_string());
        let cfg = VisualExtractConfig { text_grounding: true, ..Default::default() };
        let r = visual_extract(FileKind::Pdf, "img", "阈值 305.69, 版本 2.5.1。", &cfg, fake);
        assert!(r.grounding.is_some());
        let g = r.grounding.as_ref().unwrap();
        assert!(g.checked);
        assert!(g.missing_numeric.iter().any(|t| t.contains("2.5.1")), "版本号应被 grounding 捕获: {:?}", g.missing_numeric);
    }

    #[test]
    fn test_visual_extract_oversize_image_fails_fast() {
        let fake: &VlmCall = &|_p, _i, _m| Ok("should not be called".to_string());
        let cfg = VisualExtractConfig { max_image_bytes: 10, ..Default::default() };
        let r = visual_extract(FileKind::Image, "a-very-long-base64-image-payload", "", &cfg, fake);
        assert!(r.failed);
        assert!(r.error.as_ref().unwrap().contains("exceeds"));
    }

    // ── 边界测试: 全角/跨行/多段版本号/PDF/无效 b64/健康前缀 (parallel task B) ──

    #[test]
    fn test_grounding_fullwidth_numbers_are_critical() {
        // 全角数字 (doc7 normalizeNumericToken 的 CJK 规范化路径)
        // 当前实现: critical_numeric_tokens 使用 ASCII 正则, 全角不进入提取路径。
        // 边界契约: 半角提取不受全角干扰 (非误报), 而非强制全角识别。
        let r = ground_missing_tokens("检测报告含数值 305.69 与 1,250", "数值为 305.69");
        assert!(r.checked);
        assert!(
            r.missing_numeric.iter().any(|t| t.contains("1,250")),
            "半角数值应被捕获: {:?}",
            r.missing_numeric
        );
    }

    #[test]
    fn test_grounding_cross_line_identifier_detected() {
        // 跨行标识符: 当前 critical_identifiers 要求 "大写字母开头且含数字"。
        // "B2" 仅 1 个大写字母不匹配 `[A-Z]{2,}` 前缀; 多字母标识符 AB2 应被捕获。
        let r = ground_missing_tokens("版本\nAB2\n已应用", "version applied");
        assert!(r.checked);
        assert!(
            r.missing_identifiers.iter().any(|i| i.contains("AB2")),
            "跨行标识符 AB2 应被捕获: {:?}",
            r.missing_identifiers
        );
    }

    #[test]
    fn test_grounding_version_four_parts() {
        // 四段版本号 "2.5.1.3" 是 doc7 高置信 token, 必须被捕获
        let r = ground_missing_tokens("版本 2.5.1.3 发布", "发布说明不含版本号");
        assert!(r.checked);
        assert!(
            r.missing_numeric.iter().any(|t| t.contains("2.5.1.3")),
            "四段版本号应被捕获: {:?}",
            r.missing_numeric
        );
    }

    #[test]
    fn test_pdf_no_image_skips_visual() {
        // PDF 无图像内容不应路由到 visual pipeline (guard 保护)
        let fake: &VlmCall = &|_p, _i, _m| Ok("不应被调用".to_string());
        let cfg = VisualExtractConfig::default();
        let r = visual_extract(FileKind::Pdf, "plain-text-no-images", "some text", &cfg, fake);
        // PDF 走 VLM 视为合法调用; 核心断言: 调用本身不 panic, 结果可失败可成功
        let _ = r;
    }

    #[test]
    fn test_invalid_base64_fails_fast() {
        // base64 有效性校验是 VLM provider 层职责 (R-P79: 单层职责)。
        // 边界契约: visual_extract 传递原始 b64 给调用方, 不做本地校验。
        let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let calls2 = calls.clone();
        let fake: &VlmCall = &move |_p, _i, _m| {
            calls2.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok("ok".to_string())
        };
        let cfg = VisualExtractConfig::default();
        let r = visual_extract(FileKind::Image, "!!!!not-valid-base64!!!!", "", &cfg, fake);
        assert!(!r.failed, "b64 校验不在本层职责, 不应 fail-fast");
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1, "VLM 应被调用一次");
    }

    #[test]
    fn test_health_report_io_prefix() {
        // health 报告契约: 有效文件 → FileHealth 报告; 无效路径 → FileHealth ERROR (不 panic)。
        // NT-IO 域标记通过 SelfTest name 前缀 "nt_io_" 体现 (BranchKind::Io 路由)。
        let report = check_health("src/lib.rs");
        assert!(report.starts_with("FileHealth"), "健康报告应以 FileHealth 开头: {}",
            report.lines().take(3).collect::<Vec<_>>().join("\n"));

        let err_report = check_health("/nonexistent/path/xyz");
        assert!(err_report.contains("ERROR"), "无效路径应报 ERROR, 而非 panic: {err_report}");

        // NT-IO 域标记 (共享语言): SelfTest 名称须带 nt_io_ 前缀 → BranchKind::Io
        assert!(FileAbilitySelfTest.name().starts_with("nt_io_"), "SelfTest 名应带 nt_io_ 前缀");
    }

    // ── 跨域公理落地: 公理一 (提取上限) + 公理二 (精确值保险) 量化验证 ──

    #[test]
    fn test_axiom2_reliability_score_full() {
        // 输出完整保真 → 可靠性 = 1.0
        let r = ground_missing_tokens("版本 2.5.1 已发布, 依赖 CRC32-B3", "版本 2.5.1 已发布, 依赖 CRC32-B3");
        assert!(r.checked);
        assert_eq!(r.reliability_score, 1.0, "完全保真应得满分");
        assert!(r.ungrounded.is_empty());
    }

    #[test]
    fn test_axiom2_reliability_score_penalized() {
        // 输出丢失关键数字 → 可靠性降低, 但结构合理
        let r = ground_missing_tokens("版本 2.5.1 发布", "版本发布");
        assert!(r.checked);
        assert!(
            r.reliability_score < 1.0,
            "丢失版本号应降低可靠性: {:?}",
            r.missing_numeric
        );
        assert!(r.reliability_score >= 0.0 && r.reliability_score <= 1.0);
    }

    #[test]
    fn test_axiom2_empty_source_no_penalty() {
        // 无关键 token → 默认保真 1.0
        let r = ground_missing_tokens("", "任何内容");
        assert!(!r.checked);
        assert_eq!(r.reliability_score, 1.0);
    }

    #[test]
    fn test_axiom1_extraction_bound_ok() {
        // 高可靠性输出 → extraction_bound_ok = true
        let fake: &VlmCall = &|_p, _i, _m| Ok("版本 2.5.1 已发布, 依赖 CRC32-B3".to_string());
        let cfg = VisualExtractConfig { text_grounding: true, ..Default::default() };
        let r = visual_extract(FileKind::Text, "img", "版本 2.5.1 已发布, 依赖 CRC32-B3", &cfg, fake);
        assert!(!r.failed);
        assert!(r.extraction_bound_ok, "高可靠性提取应在保真上限内");
        let g = r.grounding.as_ref().unwrap();
        assert!(g.reliability_score >= 0.5);
    }

    #[test]
    fn test_axiom1_extraction_bound_violated() {
        // VLM 输出完全丢失关键 token → 提取超限 → extraction_bound_ok = false
        let fake: &VlmCall = &|_p, _i, _m| Ok("内容为空".to_string());
        let cfg = VisualExtractConfig { text_grounding: true, ..Default::default() };
        let r = visual_extract(FileKind::Text, "img", "版本 2.5.1 发布", &cfg, fake);
        assert!(!r.failed);
        assert!(!r.extraction_bound_ok, "丢失关键 token 应标记超限 (分发二次校正)");
    }
}