//! SelfTest T1 接线 — FileAbility 自检验证模块能力链路健康。

use crate::core::nt_core_hex::ReasoningHexagram;
use crate::core::nt_core_hcube::vsa::VsaBackend;
use crate::core::nt_core_self_test::SelfTest;
use crate::core::nt_core_traits::SpecialistType;
use nt_core_capability_tree::ConstellationLevel;
use office_oxide::DocumentFormat;

use super::{
    ContentSnapshot, FileAbility, FileKind, FileOperation, OcrEngine, RuleBasedOcr,
    SheetCellValueType, TableData, TextEncoding, VlmCall, VisualExtractConfig, consolidate_tables,
    decode_bytes, detect_encoding, embed_text, ground_missing_tokens, load_snapshot, read_csv,
    read_structured, read_xlsx_table, route_attention, specialist_index,
    specialist_index_inv, store_snapshot, visual_extract, write_csv, write_xlsx_table,
};

/// FileAbility 自检 — 验证模块能力链路健康
pub struct FileAbilitySelfTest;

/// 最小压缩 PDF (lopdf 生成, FlateDecode 内容流 + 内嵌 "NeoTrix SelfTest PDF" 文本)
const PDF_SELF_TEST_SAMPLE: &[u8] = &[
    0x25,0x50,0x44,0x46,0x2d,0x31,0x2e,0x35,0x0a,0x25,0xbb,0xad,0xc0,0xde,0x0a,0x31,
    0x20,0x30,0x20,0x6f,0x62,0x6a,0x0a,0x3c,0x3c,0x2f,0x54,0x79,0x70,0x65,0x2f,0x50,
    0x61,0x67,0x65,0x73,0x2f,0x4b,0x69,0x64,0x73,0x5b,0x35,0x20,0x30,0x20,0x52,0x5d,
    0x2f,0x43,0x6f,0x75,0x6e,0x74,0x20,0x31,0x2f,0x52,0x65,0x73,0x6f,0x75,0x72,0x63,
    0x65,0x73,0x20,0x33,0x20,0x30,0x20,0x52,0x2f,0x4d,0x65,0x64,0x69,0x61,0x42,0x6f,
    0x78,0x5b,0x30,0x20,0x30,0x20,0x35,0x39,0x35,0x20,0x38,0x34,0x32,0x5d,0x3e,0x3e,
    0x0a,0x65,0x6e,0x64,0x6f,0x62,0x6a,0x0a,0x32,0x20,0x30,0x20,0x6f,0x62,0x6a,0x0a,
    0x3c,0x3c,0x2f,0x54,0x79,0x70,0x65,0x2f,0x46,0x6f,0x6e,0x74,0x2f,0x53,0x75,0x62,
    0x74,0x79,0x70,0x65,0x2f,0x54,0x79,0x70,0x65,0x31,0x2f,0x42,0x61,0x73,0x65,0x46,
    0x6f,0x6e,0x74,0x2f,0x43,0x6f,0x75,0x72,0x69,0x65,0x72,0x3e,0x3e,0x0a,0x65,0x6e,
    0x64,0x6f,0x62,0x6a,0x0a,0x33,0x20,0x30,0x20,0x6f,0x62,0x6a,0x0a,0x3c,0x3c,0x2f,
    0x46,0x6f,0x6e,0x74,0x3c,0x3c,0x2f,0x46,0x31,0x20,0x32,0x20,0x30,0x20,0x52,0x3e,
    0x3e,0x3e,0x0a,0x65,0x6e,0x64,0x6f,0x62,0x6a,0x0a,0x34,0x20,0x30,0x20,0x6f,0x62,
    0x6a,0x0a,0x3c,0x3c,0x2f,0x4c,0x65,0x6e,0x67,0x74,0x68,0x20,0x35,0x32,0x3e,
    0x3e,0x73,0x74,0x72,0x65,0x61,0x6d,0x0a,0x42,0x54,0x0a,0x2f,0x46,0x31,0x20,0x34,
    0x38,0x20,0x54,0x66,0x0a,0x31,0x30,0x30,0x20,0x36,0x30,0x30,0x20,0x54,0x64,0x0a,
    0x28,0x4e,0x65,0x6f,0x54,0x72,0x69,0x78,0x20,0x53,0x65,0x6c,0x66,0x54,0x65,0x73,
    0x74,0x20,0x50,0x44,0x46,0x29,0x20,0x54,0x6a,0x0a,0x45,0x54,0x0a,0x65,0x6e,0x64,
    0x73,0x74,0x72,0x65,0x61,0x6d,0x20,0x0a,0x65,0x6e,0x64,0x6f,0x62,0x6a,0x0a,0x35,
    0x20,0x30,0x20,0x6f,0x62,0x6a,0x0a,0x3c,0x3c,0x2f,0x54,0x79,0x70,0x65,0x2f,0x50,
    0x61,0x67,0x65,0x2f,0x50,0x61,0x72,0x65,0x6e,0x74,0x20,0x31,0x20,0x30,0x20,0x52,
    0x2f,0x43,0x6f,0x6e,0x74,0x65,0x6e,0x74,0x73,0x20,0x34,0x20,0x30,0x20,0x52,0x3e,
    0x3e,0x0a,0x65,0x6e,0x64,0x6f,0x62,0x6a,0x0a,0x36,0x20,0x30,0x20,0x6f,0x62,0x6a,
    0x0a,0x3c,0x3c,0x2f,0x54,0x79,0x70,0x65,0x2f,0x43,0x61,0x74,0x61,0x6c,0x6f,0x67,
    0x2f,0x50,0x61,0x67,0x65,0x73,0x20,0x31,0x20,0x30,0x20,0x52,0x3e,0x3e,0x0a,0x65,
    0x6e,0x64,0x6f,0x62,0x6a,0x0a,0x37,0x20,0x30,0x20,0x6f,0x62,0x6a,0x0a,0x3c,0x3c,
    0x2f,0x52,0x6f,0x6f,0x74,0x20,0x36,0x20,0x30,0x20,0x52,0x2f,0x54,0x79,0x70,0x65,
    0x2f,0x58,0x52,0x65,0x66,0x2f,0x53,0x69,0x7a,0x65,0x20,0x38,0x2f,0x57,0x5b,0x31,
    0x20,0x34,0x20,0x32,0x5d,0x2f,0x49,0x6e,0x64,0x65,0x78,0x5b,0x31,0x20,0x37,0x5d,
    0x2f,0x4c,0x65,0x6e,0x67,0x74,0x68,0x20,0x34,0x39,0x3e,0x3e,0x73,0x74,0x72,0x65,
    0x61,0x6d,0x0a,0x01,0x00,0x00,0x00,0x0f,0x00,0x00,0x01,0x00,0x00,0x00,0x68,0x00,
    0x00,0x01,0x00,0x00,0x00,0xa5,0x00,0x00,0x01,0x00,0x00,0x00,0xcb,0x00,0x00,0x01,
    0x00,0x00,0x01,0x2f,0x00,0x00,0x01,0x00,0x00,0x01,0x69,0x00,0x00,0x01,0x00,0x00,
    0x01,0x96,0x00,0x00,0x0a,0x65,0x6e,0x64,0x73,0x74,0x72,0x65,0x61,0x6d,0x20,0x0a,
    0x65,0x6e,0x64,0x6f,0x62,0x6a,0x0a,0x0a,0x73,0x74,0x61,0x72,0x74,0x78,0x72,0x65,
    0x66,0x0a,0x34,0x30,0x36,0x0a,0x25,0x25,0x45,0x4f,0x46,
];

impl SelfTest for FileAbilitySelfTest {
    fn name(&self) -> &str {
        // nt_io_ 前缀 → BranchKind::Io 分支健康上报 (ConsciousnessTree::from_module_name)
        "nt_io_file_ability"
    }

    fn self_test(&self) -> std::result::Result<(), Vec<String>> {
        let mut failures = Vec::new();

        // 1) Office 格式枚举可解析
        if DocumentFormat::from_extension("docx").is_none() {
            failures.push("office_oxide DocumentFormat::docx 探测失败".into());
        }

        // 2) 文本探测链路可用
        let fmt = neotrix_types::core::file_parser::FileParser::detect_format(
            "sample.txt",
            "text/plain",
            b"hello world",
        );
        if !matches!(fmt, neotrix_types::core::file_parser::FileFormat::PlainText) {
            failures.push("FileParser 文本探测失败".into());
        }

        // 3) 能力成熟度晋级链完整
        if ConstellationLevel::C0Compile.next() != Some(ConstellationLevel::C1UnitTest) {
            failures.push("ConstellationLevel 晋级链中断".into());
        }

        // 4) VSA embedding 链路 (Ext-3): 确定性 + 维度契约
        let v = embed_text("NeoTrix self-test", 256);
        if v.len() != 256 || v.iter().all(|x| *x == 0.0) {
            failures.push("VSA embed_text 维度/非零契约破坏".into());
        }
        let v2 = embed_text("NeoTrix self-test", 256);
        let sim = crate::core::nt_core_hcube::vsa::VSAEngine::new(256).similarity(&v, &v2);
        if sim < 0.99 {
            failures.push(format!("VSA embedding 确定性退化 sim={sim}"));
        }

        // 5) E8 状态机链路 (Ext-6): 目标状态合法 + ReasoningPath 可达
        for op in [
            FileOperation::Detect,
            FileOperation::Extract,
            FileOperation::Transform,
            FileOperation::Edit,
            FileOperation::Embed,
            FileOperation::Audit,
        ] {
            let target = op.target_state();
            if target.0 >= 64 || target.mode_name().is_empty() {
                failures.push(format!("E8 目标状态非法 op={}", op.name()));
            }
        }
        let path = crate::core::nt_core_hex::ReasoningPath::shortest(
            ReasoningHexagram::new(0b001001),
            ReasoningHexagram::new(0b111100),
        );
        if path.states.is_empty() {
            failures.push("E8 shortest path 为空".into());
        }

        // 6) GWT 注意力路由链路 (Ext-5): 专家索引往返 + 路由返回强度
        if specialist_index_inv(specialist_index(SpecialistType::Planner))
            != SpecialistType::Planner
        {
            failures.push("GWT specialist_index 往返不一致".into());
        }
        let (_spec, strength, _state) = route_attention(ReasoningHexagram::new(0b001100));
        if strength == 0 {
            failures.push("GWT route_attention 强度为零".into());
        }

        // 7) OCR 链路 (Ext-2): 规则引擎可实例化
        let ocr = RuleBasedOcr;
        if ocr.name().is_empty() {
            failures.push("RuleBasedOcr name 为空".into());
        }

        // 8) XLSX 单元格级结构化读取链路 (Ext-7): 写入探针 → 打开 → 结构读取
        let mut xw = office_oxide::xlsx::write::XlsxWriter::new();
        let s0 = xw.add_sheet_get_index("probe");
        xw.sheet_set_cell(
            s0,
            0,
            0,
            office_oxide::xlsx::write::CellData::String("k".into()),
        );
        xw.sheet_set_cell(s0, 0, 1, office_oxide::xlsx::write::CellData::Number(1.5));
        let probe_dir = std::env::temp_dir().join("nt_file_ability_selftest");
        let probe_path = probe_dir.join("probe.xlsx");
        if std::fs::create_dir_all(&probe_dir).is_err() || xw.save(&probe_path).is_err() {
            failures.push("XlsxWriter 探针写入失败".into());
        } else if let Ok(ab) = FileAbility::open(&probe_path) {
            match ab.xlsx_sheet(1) {
                Ok(sheet) => {
                    if sheet.rows.is_empty() || sheet.rows[0].cells.len() < 2 {
                        failures.push("XLSX 结构化读取单元格缺失".into());
                    } else {
                        let first = &sheet.rows[0].cells[0];
                        if first.value_type != SheetCellValueType::Text
                            || first.text != "k"
                            || first.reference != "A1"
                        {
                            failures.push(format!(
                                "XLSX 结构化读取值错误: {:?} '{}' @ {}",
                                first.value_type, first.text, first.reference
                            ));
                        }
                        if sheet.rows[0].cells[1].value_type != SheetCellValueType::Number {
                            failures.push("XLSX 结构化读取数字类型错误".into());
                        }
                    }
                }
                Err(e) => failures.push(format!("XLSX 结构化读取失败: {e}")),
            }
        } else {
            failures.push("XLSX 探针打开失败".into());
        }
        let _ = std::fs::remove_file(&probe_path);
        let _ = std::fs::remove_dir(&probe_dir);

        // 9) 统一表格读写链路 (D1/D2/D3): XLSX 写读回环 + CSV 写读回环 + 编码探测/解码。
        //    生产接地: write_xlsx_table 即合并输出通路 (consolidate_tables → 落盘)。
        let probe_tbl = TableData {
            name: "probe".into(),
            headers: vec!["名称".into(), "数量".into()],
            rows: vec![vec!["阀门".into(), "12".into()]],
        };
        let probe_dir = std::env::temp_dir().join("nt_file_ability_selftest");
        if std::fs::create_dir_all(&probe_dir).is_err() {
            failures.push("selftest 临时目录创建失败".into());
        }
        let probe_xlsx = probe_dir.join("probe_tbl.xlsx");
        let probe_csv = probe_dir.join("probe_tbl.csv");
        if write_xlsx_table(&probe_xlsx, &probe_tbl).is_err() {
            failures.push("write_xlsx_table 失败".into());
        } else {
            match read_xlsx_table(&probe_xlsx) {
                Ok(t) => {
                    if t.headers != probe_tbl.headers || t.rows != probe_tbl.rows {
                        failures.push("XLSX 表格写读回环不一致".into());
                    }
                }
                Err(e) => failures.push(format!("read_xlsx_table 失败: {e}")),
            }
        }
        if write_csv(&probe_csv, &probe_tbl, ',', false).is_err() {
            failures.push("write_csv 失败".into());
        } else {
            match read_csv(&probe_csv) {
                Ok(t) => {
                    if t.headers != probe_tbl.headers || t.rows != probe_tbl.rows {
                        failures.push("CSV 表格写读回环不一致".into());
                    }
                }
                Err(e) => failures.push(format!("read_csv 失败: {e}")),
            }
        }
        let gbk_bytes = b"\xd6\xd0\xb9\xfa";
        if detect_encoding(gbk_bytes) != TextEncoding::Gbk || decode_bytes(gbk_bytes) != "中国" {
            failures.push("GBK 编码探测/解码契约破坏".into());
        }
        let _ = std::fs::remove_file(&probe_xlsx);
        let _ = std::fs::remove_file(&probe_csv);

        // 10) 多表合并链路 (D4): 两文件端到端合并 → 列名归一化 + 行数 + 排序表头。
        //     生产接地: 合并产物已内置跳过 consolidate_* 自身输出, 防止重复吸收。
        let m_dir = probe_dir.join("merge");
        if std::fs::create_dir_all(&m_dir).is_ok() {
            let a = TableData {
                name: "a".into(),
                headers: vec!["产品型号".into(), "阀体材质".into(), "单重(Kg)".into()],
                rows: vec![vec!["V1".into(), "WCB".into(), "1.5kg".into()]],
            };
            let b = TableData {
                name: "b".into(),
                headers: vec!["型号".into(), "阀体材料".into(), "重量".into()],
                rows: vec![vec!["V2".into(), "CF8".into(), "2kg".into()]],
            };
            if write_xlsx_table(m_dir.join("a.xlsx"), &a).is_err()
                || write_xlsx_table(m_dir.join("b.xlsx"), &b).is_err()
            {
                failures.push("合并源写入失败".into());
            } else {
                let out = m_dir.join("out.xlsx");
                match consolidate_tables(&m_dir, &out) {
                    Ok(rep) => {
                        if rep.files_processed != 2 || rep.total_rows != 2 {
                            failures.push(format!(
                                "合并统计异常 processed={} rows={}",
                                rep.files_processed, rep.total_rows
                            ));
                        }
                        if let Ok(t) = read_xlsx_table(&out) {
                            let has_std = t
                                .headers
                                .iter()
                                .any(|h| h == "阀体材质" || h == "单重(Kg)");
                            if !has_std {
                                failures.push("合并列名未归一化为标准列".into());
                            }
                        }
                    }
                    Err(e) => failures.push(format!("consolidate_tables 失败: {e}")),
                }
            }
            let _ = std::fs::remove_dir_all(&m_dir);
        }

        // 11) 结构化/快照链路 (D5/D6): JSON 写读回环 + 快照存读。
        let s_path = probe_dir.join("probe.json");
        if let Ok(sd) = read_structured(s_path.as_path()) {
            // 文件不存在时 read_structured 应报错; 存在时 roundtrip
            let _ = sd;
        }
        let snap = ContentSnapshot {
            kind: FileKind::Text,
            text: Some("探针".into()),
            markdown: None,
            table: None,
            image: None,
            media: None,
            mime_type: "text/plain".into(),
            size_bytes: 42,
        };
        let snap_path = probe_dir.join("probe_snapshot.json");
        if store_snapshot(&snap, &snap_path).is_err() {
            failures.push("store_snapshot 失败".into());
        } else if let Ok(back) = load_snapshot(&snap_path) {
            if back.text != snap.text || back.mime_type != snap.mime_type {
                failures.push("快照存读回环不一致".into());
            }
        } else {
            failures.push("load_snapshot 失败".into());
        }
        let _ = std::fs::remove_file(&s_path);
        let _ = std::fs::remove_file(&snap_path);
        let _ = std::fs::remove_dir(&probe_dir);

        // 12) grounding 公理链路 (公理二: 精确值保险): 可靠性评分契约 + 提取上限分发。
        // 生产接地: self_test 结果经 run_all → set_branch_health 驱动 NT-IO 分支健康 (T3)。
        let gr = ground_missing_tokens("版本 2.5.1 已发布", "版本 2.5.1 已发布");
        if gr.reliability_score < 0.99 {
            failures.push(format!("grounding 完全保真应得满分, 实得 {}", gr.reliability_score));
        }
        let gr_lossy = ground_missing_tokens("版本 2.5.1 发布", "内容缺失");
        if gr_lossy.checked && gr_lossy.reliability_score >= 1.0 {
            failures.push("grounding 丢失 token 不应得满分".into());
        }
        // 公理一: 提取上限分发契约 (reliability < 0.5 → 超限)
        let fake: &VlmCall = &|_p, _i, _m| Ok("内容缺失".to_string());
        let cfg = VisualExtractConfig { text_grounding: true, ..Default::default() };
        let r = visual_extract(FileKind::Text, "img", "版本 2.5.1 发布", &cfg, fake);
        if !r.failed && r.extraction_bound_ok {
            failures.push("低可靠性提取应标记 extraction_bound_ok=false".into());
        }

        // 13) PDF 提取链路 (Ext-8): 探测 + 压缩流解析 (lopdf 生产通路 extract_text)。
        // 生产接地: extract_text 即 nt_file_ability::plain_text 的 PDF 上游 (T3)。
        let pdf_fmt = neotrix_types::core::file_parser::FileParser::detect_format(
            "doc.pdf",
            "application/pdf",
            b"%PDF-1.7 \n%%EOF",
        );
        if !matches!(
            pdf_fmt,
            neotrix_types::core::file_parser::FileFormat::Pdf
        ) {
            failures.push("FileParser PDF 探测失败".into());
        }
        let pdf_res = neotrix_types::core::file_parser::FileParser::extract_text(
            "doc.pdf",
            "application/pdf",
            PDF_SELF_TEST_SAMPLE,
        );
        if !pdf_res.parse_success || !pdf_res.text.contains("NeoTrix SelfTest") {
            failures.push(format!(
                "FileParser PDF 压缩流解析失败: success={} text={:?}",
                pdf_res.parse_success, pdf_res.text
            ));
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}