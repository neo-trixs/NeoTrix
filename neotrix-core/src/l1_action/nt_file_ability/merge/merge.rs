//! 多文件合并 (D4): 通用引擎 + 领域 Schema 分离。
//!
//! 分层架构 (贯穿整个文件能力体系):
//!   L1 格式编解码 (通用): read_xlsx_table/write_csv/detect_encoding — 任何类型文件
//!   L2 表格语义 (通用):   merge_tables_with_mode(schema, mode) — 零领域知识的多表合并引擎
//!   L3 领域 schema (差异化): SchemaStore 数据化 (JSON: price_table/product_lib + 内置回退)
//!   L4 意图层 (通用):     意识核心 xlsx_consolidation → SchemaStore 选 schema → 调 merge_tables_with_mode
//!                          CLI /file consolidate --schema <name> --sheet-mode first|preferred|all
//!
//! 领域知识 (列名变体/标准列序/单位规则/供应商命名/跳过前缀) 全部数据化进 MergeSchema,
//! 不再编译进引擎函数。换行业/换策略 = 新增 JSON schema 或传 SheetMode, 不改引擎代码。
//! sheet 选择策略由 SheetMode 运行时参数化 (FirstSheet/Preferred/AllSheets), 消除变体爆炸。
//!
//! 本文件为门面: 实现已按职责拆分至 nt_ 子模块, 此处重导出保持外部路径不变。

pub use super::nt_merge_collection::*;
pub use super::nt_merge_engine::*;
pub use super::nt_merge_schema_store::*;
pub use super::nt_merge_suggest::*;
pub use super::nt_merge_types::*;

#[cfg(test)]
mod schema_tests {
    use super::super::nt_merge_collection::{
        CollectionMergeRequest, MergeOutcome, MergeStrategy, collection_merge,
    };
    use super::super::nt_merge_engine::{merge_tables_with_mode, select_preferred_sheets};
    use super::super::nt_merge_schema_store::{MergeSchemaJson, SchemaStore};
    use super::super::nt_merge_types::{SheetMode, PRICE_TABLE_SCHEMA};
    use crate::l1_action::nt_file_ability::excel::tables::{read_csv, write_csv, write_xlsx_table};
    use crate::l1_action::nt_file_ability::types::TableData;

    #[test]
    fn test_sheet_mode_first_takes_one() {
        // FirstSheet: 只取第一个 sheet
        let tables = vec![
            TableData {
                name: "修改版".into(),
                headers: vec![],
                rows: vec![],
            },
            TableData {
                name: "工作表1".into(),
                headers: vec![],
                rows: vec![],
            },
        ];
        let got = select_preferred_sheets(tables, SheetMode::FirstSheet);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].name, "修改版");
    }

    #[test]
    fn test_sheet_mode_preferred_hits_else_first() {
        let tables = vec![
            TableData {
                name: "工作表1".into(),
                headers: vec![],
                rows: vec![],
            },
            TableData {
                name: "修改版".into(),
                headers: vec![],
                rows: vec![],
            },
        ];
        // 命中 preferred → 只用修改版
        let got = select_preferred_sheets(tables.clone(), SheetMode::Preferred(&["修改版"]));
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].name, "修改版");
        // 未命中 → 取第一个
        let got = select_preferred_sheets(tables, SheetMode::Preferred(&["不存在"]));
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].name, "工作表1");
    }

    #[test]
    fn test_sheet_mode_all_keeps_all() {
        let tables = vec![
            TableData {
                name: "a".into(),
                headers: vec![],
                rows: vec![],
            },
            TableData {
                name: "b".into(),
                headers: vec![],
                rows: vec![],
            },
        ];
        let got = select_preferred_sheets(tables, SheetMode::AllSheets);
        assert_eq!(got.len(), 2);
    }

    #[test]
    fn test_schema_json_roundtrip_from_price_table() {
        let json = MergeSchemaJson::from_price_table();
        json.validate_json().unwrap();
        // 往返: JSON → MergeSchema → 校验通过
        let schema = json.to_schema();
        schema.validate().unwrap();
        assert_eq!(schema.name, "价格表");
        assert_eq!(schema.standard_columns.len(), 19);
        assert!(schema.value_columns.contains(&"美元报价(USD)"));
        assert!(schema.dedup_columns.contains(&"口径"));
    }

    #[test]
    fn test_schema_store_load_price_table_builtin() {
        let mut store = SchemaStore::with_dir(std::env::temp_dir());
        let schema = store.load("price_table").unwrap();
        assert_eq!(schema.name, "价格表");
        assert!(store.list().contains(&"price_table".to_string()));
    }

    #[test]
    fn test_schema_store_load_failure_paths() {
        // 空目录: 非内置 schema 不存在 → 清晰报错 (不 panic, 不回退到内置)
        let dir = std::env::temp_dir().join(format!("nt_schema_store_fail_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let mut store = SchemaStore::with_dir(&dir);
        let err = store.load("not_exist_xyz").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("not_exist_xyz"), "报错应含 schema 名: {msg}");
        assert!(msg.contains("加载失败"), "报错应标明加载失败: {msg}");

        // 坏 JSON 文件 → 解析报错 (不 panic)
        std::fs::write(dir.join("broken.json"), b"{ not valid json ").unwrap();
        let err = store.load("broken").unwrap_err();
        assert!(!err.to_string().is_empty(), "坏 JSON 应报错");

        // 合法 JSON 但校验失败 (value_column 不在标准列) → 校验报错
        let mut bad = MergeSchemaJson::from_price_table();
        bad.name = "bad_schema".to_string();
        bad.value_columns.push("幽灵列".to_string());
        std::fs::write(
            dir.join("bad_schema.json"),
            serde_json::to_string_pretty(&bad).unwrap(),
        )
        .unwrap();
        let err = store.load("bad_schema").unwrap_err();
        assert!(
            err.to_string().contains("幽灵列"),
            "校验失败应报列名: {}",
            err
        );

        // 缓存命中: 二次加载同 schema 不重新读文件 (删除文件后仍可加载)
        let mut store2 = SchemaStore::with_dir(&dir);
        let _ = store2.load("bad_schema").is_err(); // 首次失败不缓存
                                                    // 写一个有效 schema, 加载后删文件, 再加载 → 缓存命中
        let good = MergeSchemaJson::from_price_table();
        std::fs::write(
            dir.join("good_schema.json"),
            serde_json::to_string_pretty(&good).unwrap(),
        )
        .unwrap();
        let _ = store2.load("good_schema").unwrap();
        std::fs::remove_file(dir.join("good_schema.json")).ok();
        let cached = store2.load("good_schema").unwrap();
        assert_eq!(cached.name, "价格表", "缓存命中应返回原 schema");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_schema_store_load_json_file() {
        // 写一个临时 JSON schema, 验证文件加载路径
        let dir = std::env::temp_dir().join("nt_schema_store_test");
        std::fs::create_dir_all(&dir).unwrap();
        let json = MergeSchemaJson {
            name: "临时目录".to_string(),
            standard_columns: vec!["品名".to_string(), "价格".to_string()],
            column_variants: vec![(
                "品名".to_string(),
                vec!["型号".to_string(), "产品型号".to_string()],
            )],
            filename_suffixes: vec!["目录".to_string()],
            unit_rules: vec![],
            preferred_sheets: vec![],
            empty_markers: vec![],
            value_columns: vec!["价格".to_string()],
            extra_columns: vec!["_source_file".to_string()],
            skip_prefixes: vec!["consolidated".to_string()],
            supplier_column: None,
            dedup_columns: vec!["品名".to_string()],
            numeric_columns: vec!["价格".to_string()],
        };
        json.validate_json().unwrap();
        let path = dir.join("tmp_dir.json");
        std::fs::write(&path, serde_json::to_string_pretty(&json).unwrap()).unwrap();
        let mut store = SchemaStore::with_dir(&dir);
        let schema = store.load("tmp_dir").unwrap();
        assert_eq!(schema.name, "临时目录");
        schema.validate().unwrap();
    }

    #[test]
    fn test_schema_json_bad_validation_rejected() {
        let mut bad = MergeSchemaJson::from_price_table();
        // value_column 不在 standard_columns → 校验必须失败
        bad.value_columns.push("不存在列".to_string());
        assert!(bad.validate_json().is_err());
    }

    #[test]
    fn test_merge_tables_output_csv_with_bom() {
        // 输出扩展名分发: .csv → UTF-8 BOM + 逗号; 数据与 xlsx 输出一致
        let dir = std::env::temp_dir().join("nt_merge_csv_out");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let t = TableData {
            name: "s1".into(),
            headers: vec!["产品型号".to_string(), "含税单价(元)".to_string()],
            rows: vec![vec!["闸阀\"双引号\"".to_string(), "100".to_string()]],
        };
        write_xlsx_table(dir.join("1_华东_价格.xlsx"), &t).unwrap();
        let out = dir.join("native_consolidated.csv");
        let rep =
            merge_tables_with_mode(&PRICE_TABLE_SCHEMA, &dir, &out, SheetMode::FirstSheet).unwrap();
        assert_eq!(rep.total_rows, 1);
        let raw = std::fs::read(&out).unwrap();
        assert!(raw.starts_with(&[0xEF, 0xBB, 0xBF]), "CSV 应带 UTF-8 BOM");
        let text = String::from_utf8_lossy(&raw);
        assert!(
            text.contains("产品大类") || text.contains("产品型号"),
            "输出表头应包含标准列: {text}"
        );
        // 含引号的单元格应双引号包裹 + 内部引号双写
        assert!(
            text.contains("闸阀\"\"双引号\"\""),
            "引号单元格应正确转义: {text}"
        );
        // 读回一致性
        let back = read_csv(&out).unwrap();
        assert_eq!(back.row_count(), 1);
        let found = back
            .headers
            .iter()
            .position(|h| h == "产品型号")
            .map(|i| back.rows[0][i].as_str())
            .unwrap_or("");
        assert_eq!(found, "闸阀\"双引号\"");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_merge_tables_mixed_csv_and_xlsx_input() {
        // CSV 混合目录: 引擎同时消费 xlsx + csv 输入, 去重与单一格式行为一致
        let dir = std::env::temp_dir().join("nt_merge_mixed_in");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        // xlsx 源 (1 行)
        let t1 = TableData {
            name: "s1".into(),
            headers: vec!["产品型号".to_string(), "含税单价(元)".to_string()],
            rows: vec![vec!["闸阀X".to_string(), "100".to_string()]],
        };
        write_xlsx_table(dir.join("1_华东_价格.xlsx"), &t1).unwrap();
        // csv 源 (1 行, 同 schema 列)
        let t2 = TableData {
            name: "s2".into(),
            headers: vec!["产品型号".to_string(), "含税单价(元)".to_string()],
            rows: vec![vec!["蝶阀Y".to_string(), "200".to_string()]],
        };
        write_csv(dir.join("2_华南_价格.csv"), &t2, ',', true).unwrap();
        // 输出 csv
        let out = dir.join("native_consolidated.csv");
        let rep =
            merge_tables_with_mode(&PRICE_TABLE_SCHEMA, &dir, &out, SheetMode::FirstSheet).unwrap();
        assert_eq!(rep.files_processed, 2, "xlsx + csv 都应处理");
        assert_eq!(rep.total_rows, 2, "各 1 行 → 2 行");
        let back = read_csv(&out).unwrap();
        assert_eq!(back.row_count(), 2);
        let model = back.headers.iter().position(|h| h == "产品型号").unwrap();
        let mut models: Vec<&str> = back.rows.iter().map(|r| r[model].as_str()).collect();
        models.sort();
        assert_eq!(models, vec!["蝶阀Y", "闸阀X"], "两个源的数据都应进入");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_collection_merge_dispatch_routes() {
        // 分发: 全部 pdf → merge_pdfs; 全部 docx → merge_docx; 混合 → L0 文本
        fn pdf_bytes() -> Vec<u8> {
            // 最小单页 PDF (lopdf 构造)
            use lopdf::content::{Content, Operation};
            use lopdf::dictionary;
            let mut doc = lopdf::Document::with_version("1.5");
            let pages_id = doc.new_object_id();
            let font_id = doc.add_object(lopdf::dictionary! {
                "Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Courier",
            });
            let res_id = doc.add_object(lopdf::dictionary! {
                "Font" => lopdf::dictionary! { "F1" => font_id },
            });
            let content = Content {
                operations: vec![
                    Operation::new("BT", vec![]),
                    Operation::new("Tf", vec!["F1".into(), 24.into()]),
                    Operation::new("Td", vec![60.into(), 700.into()]),
                    Operation::new("Tj", vec![lopdf::Object::string_literal("MergePDFTest")]),
                    Operation::new("ET", vec![]),
                ],
            };
            let cid = doc.add_object(lopdf::Stream::new(
                lopdf::Dictionary::new(),
                content.encode().expect("encode"),
            ));
            let page_id = doc.add_object(lopdf::dictionary! {
                "Type" => "Page", "Parent" => pages_id, "Contents" => cid,
                "Resources" => res_id,
                "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
            });
            let pages = lopdf::dictionary! {
                "Type" => "Pages", "Kids" => vec![page_id.into()], "Count" => 1,
            };
            doc.objects
                .insert(pages_id, lopdf::Object::Dictionary(pages));
            let catalog = doc.add_object(lopdf::dictionary! {
                "Type" => "Catalog", "Pages" => pages_id,
            });
            doc.trailer.set("Root", catalog);
            doc.compress();
            let mut buf = Vec::new();
            doc.save_to(&mut buf).expect("save");
            buf
        }

        let tmp = std::env::temp_dir().join(format!("nt_colmerge_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        // 1. 全部 pdf → Pdf outcome (2 页)
        let p1 = tmp.join("a.pdf");
        let p2 = tmp.join("b.pdf");
        std::fs::write(&p1, pdf_bytes()).unwrap();
        std::fs::write(&p2, pdf_bytes()).unwrap();
        let req = CollectionMergeRequest {
            inputs: vec![p1.clone(), p2.clone()],
            schema: None,
            strategy: MergeStrategy::All,
            output: tmp.join("merged.pdf"),
            dry_run: false,
        };
        match collection_merge(&req).expect("pdf 合并") {
            MergeOutcome::Pdf { pages } => assert_eq!(pages, 2, "两 PDF → 2 页"),
            other => panic!("应为 Pdf outcome: {other:?}"),
        }

        // 2. 全部 docx → Docx outcome
        let d1 = tmp.join("a.docx");
        let d2 = tmp.join("b.docx");
        std::fs::write(&d1, crate::l1_action::nt_file_ability::make_min_docx("A")).unwrap();
        std::fs::write(&d2, crate::l1_action::nt_file_ability::make_min_docx("B")).unwrap();
        let req = CollectionMergeRequest {
            inputs: vec![d1, d2],
            schema: None,
            strategy: MergeStrategy::All,
            output: tmp.join("merged.docx"),
            dry_run: false,
        };
        match collection_merge(&req).expect("docx 合并") {
            MergeOutcome::Docx { items, .. } => assert_eq!(items, 2),
            other => panic!("应为 Docx outcome: {other:?}"),
        }

        // 3. 混合格式 → L0 文本级 (pdf + docx)
        let p1_c = tmp.join("a.pdf");
        std::fs::write(&p1_c, pdf_bytes()).unwrap();
        let req = CollectionMergeRequest {
            inputs: vec![p1_c, tmp.join("c.docx")],
            schema: None,
            strategy: MergeStrategy::All,
            output: tmp.join("merged.txt"),
            dry_run: false,
        };
        // 混合输入中 docx 用 make_min_docx 写入
        std::fs::write(
            &req.inputs[1],
            crate::l1_action::nt_file_ability::make_min_docx("C"),
        )
        .unwrap();
        match collection_merge(&req).expect("混合 L0 合并") {
            MergeOutcome::Text { items, .. } => {
                assert_eq!(items, 2, "混合 2 文件 → L0 聚合");
                let text = std::fs::read_to_string(&req.output).unwrap();
                assert!(
                    text.contains("MergePDFTest") && text.contains("C"),
                    "L0 应含全部源文本: {text}"
                );
            }
            other => panic!("混合应回退 Text: {other:?}"),
        }

        // 4. 空输入 → 报错
        let req = CollectionMergeRequest {
            inputs: vec![],
            schema: None,
            strategy: MergeStrategy::All,
            output: tmp.join("empty.docx"),
            dry_run: false,
        };
        assert!(collection_merge(&req).is_err(), "空输入应报错");

        // 5. Preferred 策略: 修改版优先作为基座
        let doc_modified = tmp.join("报价_修改版.docx");
        let doc_normal = tmp.join("报价_标准版.docx");
        std::fs::write(
            &doc_modified,
            crate::l1_action::nt_file_ability::make_min_docx("Modified"),
        )
        .unwrap();
        std::fs::write(
            &doc_normal,
            crate::l1_action::nt_file_ability::make_min_docx("Normal"),
        )
        .unwrap();
        let req = CollectionMergeRequest {
            inputs: vec![doc_normal.clone(), doc_modified.clone()], // 故意把普通版放前面
            schema: None,
            strategy: MergeStrategy::Preferred,
            output: tmp.join("preferred.docx"),
            dry_run: false,
        };
        match collection_merge(&req).expect("Preferred 合并") {
            MergeOutcome::Docx { items, .. } => assert_eq!(items, 2),
            other => panic!("应为 Docx outcome: {other:?}"),
        };
        // 验证修改版作为基座: 其内容应在最前
        let merged_bytes = std::fs::read(&tmp.join("preferred.docx")).unwrap();
        let merged_result = neotrix_types::core::file_parser::FileParser::extract_text(
            "preferred.docx",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            &merged_bytes,
        );
        let merged_text = merged_result.text;
        assert!(
            merged_text.find("Modified").unwrap() < merged_text.find("Normal").unwrap(),
            "Preferred 策略应把修改版作为基座 (内容在前)"
        );

        // 6. dry_run 预览模式: 不写文件, 仅返回预览信息
        let req = CollectionMergeRequest {
            inputs: vec![doc_normal.clone(), doc_modified.clone()],
            schema: None,
            strategy: MergeStrategy::Preferred,
            output: tmp.join("dry_run.txt"),
            dry_run: true,
        };
        match collection_merge(&req).expect("dry_run 预览") {
            MergeOutcome::Text { items, note } => {
                assert_eq!(items, 2);
                assert!(note.contains("dry_run"), "预览应标注 dry_run");
                assert!(note.contains("修改版"), "预览应包含排序信息");
            }
            other => panic!("dry_run 应返回 Text: {other:?}"),
        };
        assert!(
            !tmp.join("dry_run.txt").exists(),
            "dry_run 不应生成输出文件"
        );

        // 7. FirstOnly 策略: 只透传首个
        let req = CollectionMergeRequest {
            inputs: vec![p1.clone(), p2.clone()],
            schema: None,
            strategy: MergeStrategy::FirstOnly,
            output: tmp.join("firstonly.pdf"),
            dry_run: false,
        };
        match collection_merge(&req).expect("FirstOnly") {
            MergeOutcome::Text { items, note } => {
                assert_eq!(items, 1);
                assert!(note.contains("FirstOnly"));
            }
            other => panic!("FirstOnly 应返回 Text: {other:?}"),
        };

        // 8. 单文档透传 (所有策略)
        let req = CollectionMergeRequest {
            inputs: vec![doc_normal.clone()],
            schema: None,
            strategy: MergeStrategy::All,
            output: tmp.join("single.docx"),
            dry_run: false,
        };
        match collection_merge(&req).expect("单文档") {
            MergeOutcome::Docx { items, .. } => assert_eq!(items, 1),
            other => panic!("单文档应 Docx outcome: {other:?}"),
        };

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
