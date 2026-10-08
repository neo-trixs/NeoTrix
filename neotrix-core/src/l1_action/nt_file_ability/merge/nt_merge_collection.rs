//! CollectionMerge 统一入口 (范式归一, R-P42)。
//!
//! 设计见 docs/3-AUDITS/collection-merge-design-2026-08-20.md。
//! 一个入口按输入格式分派到既有闭环: pdf → merge_pdfs, xlsx/csv/tsv →
//! merge_tables_with_mode, docx → merge_docx, pptx → merge_pptx;
//! 混合格式 → L0 文本级聚合 (extract_text 拼接)。
//! 由 `merge.rs` 门面重导出, 外部路径不变。

use super::nt_merge_engine::merge_tables_with_mode;
use super::nt_merge_schema_store::MergeSchemaJson;
use super::nt_merge_types::{SheetMode, PRICE_TABLE_SCHEMA};
use crate::l1_action::nt_file_ability::types::{FileAbilityError, Result};

/// 合并策略 (SheetMode 三模式泛化, 语义对齐)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeStrategy {
    /// 只取首个文档 (透传)
    FirstOnly,
    /// 同名资源冲突时取修改版优先 (结构级合并的资源选择)
    Preferred,
    /// 全部文档拼接 (默认合并)
    All,
}

/// 统一合并请求。
#[derive(Debug, Clone)]
pub struct CollectionMergeRequest {
    /// 文档集合 (非空)
    pub inputs: Vec<std::path::PathBuf>,
    /// 合并策略
    pub strategy: MergeStrategy,
    /// 结构化提取规则 (xlsx 领域 schema; 其他格式忽略)
    pub schema: Option<MergeSchemaJson>,
    /// 输出路径
    pub output: std::path::PathBuf,
    /// 预览模式: true 时不写入文件, 仅返回预览信息 (输入排序/预估大小/策略说明)
    pub dry_run: bool,
}

impl Default for CollectionMergeRequest {
    fn default() -> Self {
        Self {
            inputs: Vec::new(),
            strategy: MergeStrategy::All,
            schema: None,
            output: std::path::PathBuf::new(),
            dry_run: false,
        }
    }
}

/// 合并结果 (按格式)。
#[derive(Debug, Clone)]
pub enum MergeOutcome {
    /// L0 文本级: (已合并文件数, 说明)
    Text { items: usize, note: String },
    /// DOCX 结构级: (已合并文件数, part 数)
    Docx { items: usize, parts: usize },
    /// PPTX 结构级: 幻灯片总数
    Pptx { slides: usize },
    /// PDF 结构级: 页数
    Pdf { pages: usize },
    /// XLSX 表格合并: (总行数, 说明)
    Xlsx { rows: usize, note: String },
}

/// 分发: 按输入格式路由到既有合并闭环。
///
/// - 全部 pdf → merge_pdfs (lopdf 对象图重建)
/// - 全部 xlsx/csv/tsv → merge_tables_with_mode (schema 数据化)
/// - 全部 docx → merge_docx (OPC part 级)
/// - 全部 pptx → merge_pptx (slide 追加)
/// - 混合格式 → L0 文本级聚合 (extract_text 拼接, 输出 txt/md)
pub fn collection_merge(req: &CollectionMergeRequest) -> Result<MergeOutcome> {
    if req.inputs.is_empty() {
        return Err(FileAbilityError::Other("合并输入为空".to_string()));
    }
    // 提取全部输入扩展名 (小写)
    let exts: Vec<String> = req
        .inputs
        .iter()
        .map(|p| {
            p.extension()
                .and_then(|x| x.to_str())
                .map(|x| x.to_lowercase())
                .unwrap_or_default()
        })
        .collect();

    // dry_run: 仅返回预览, 不写文件
    if req.dry_run {
        let sorted_inputs = if req.strategy == MergeStrategy::Preferred {
            let mut inputs = req.inputs.clone();
            sort_by_preferred(&mut inputs);
            inputs
        } else {
            req.inputs.clone()
        };
        let preview = format!(
            "CollectionMerge 预览 (dry_run):\n  策略: {:?}\n  输入数: {}\n  输入顺序:\n{}  输出: {}\n  Schema: {}\n  预估: 按输入顺序合并, 首个作为基座",
            req.strategy,
            sorted_inputs.len(),
            sorted_inputs.iter().enumerate().map(|(i,p)| format!("    {}. {}", i+1, p.display())).collect::<Vec<_>>().join("\n"),
            req.output.display(),
            req.schema.as_ref().map(|s| s.name.as_str()).unwrap_or("默认(价格表)")
        );
        return Ok(MergeOutcome::Text {
            items: sorted_inputs.len(),
            note: preview,
        });
    }

    match req.strategy {
        MergeStrategy::FirstOnly => {
            // 透传首个输入 (结构级语义: 只取首个文档)
            let src = &req.inputs[0];
            let data = std::fs::read(src).map_err(FileAbilityError::Io)?;
            std::fs::write(&req.output, &data).map_err(FileAbilityError::Io)?;
            Ok(MergeOutcome::Text {
                items: 1,
                note: format!("FirstOnly 透传: {}", src.display()),
            })
        }
        MergeStrategy::Preferred => {
            // Preferred: 先按文件名识别"修改版"排序 (修改版优先作为基座), 再分发
            let mut inputs = req.inputs.clone();
            sort_by_preferred(&mut inputs);
            let mut req_preferred = req.clone();
            req_preferred.inputs = inputs;
            dispatch_collection_merge(&req_preferred, &exts)
        }
        MergeStrategy::All => dispatch_collection_merge(req, &exts),
    }
}

fn sort_by_preferred(inputs: &mut [std::path::PathBuf]) {
    // 将包含"修改版"/"已更新"/"已完善"等标记的文件排到前面 (作为基座)
    inputs.sort_by(|a, b| {
        let a_pref = is_preferred(a);
        let b_pref = is_preferred(b);
        match (a_pref, b_pref) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => std::cmp::Ordering::Equal,
        }
    });
}

fn is_preferred(path: &std::path::Path) -> bool {
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    [
        "修改版",
        "已更新",
        "已完善",
        "更新版",
        "最新版",
        "final",
        "modified",
        "updated",
    ]
    .iter()
    .any(|kw| name.contains(kw))
}

fn dispatch_collection_merge(
    req: &CollectionMergeRequest,
    exts: &[String],
) -> Result<MergeOutcome> {
    let all = |e: &str| exts.iter().all(|x| x == e);
    if all("pdf") {
        let bytes = crate::l1_action::nt_file_ability::helpers::merge_pdfs(&req.inputs)?;
        std::fs::write(&req.output, &bytes).map_err(FileAbilityError::Io)?;
        let pages = neotrix_types::core::file_parser::FileParser::extract_pdf_pages(&bytes).len();
        return Ok(MergeOutcome::Pdf { pages });
    }
    if all("docx") {
        let report = super::merge_docx::merge_docx(&req.inputs, &req.output)?;
        return Ok(MergeOutcome::Docx {
            items: report.items,
            parts: report.parts,
        });
    }
    if all("pptx") {
        let report = super::merge_docx::merge_pptx(&req.inputs, &req.output)?;
        return Ok(MergeOutcome::Pptx {
            slides: report.items + report.parts, // items=文档数, parts=slide part 数
        });
    }
    if exts
        .iter()
        .all(|x| matches!(x.as_str(), "xlsx" | "csv" | "tsv"))
    {
        // 转发表格合并引擎 (schema 数据化)
        let schema = match req.schema.as_ref() {
            Some(json) => json.to_schema(),
            None => PRICE_TABLE_SCHEMA,
        };
        let mode = match req.strategy {
            MergeStrategy::FirstOnly => SheetMode::FirstSheet,
            MergeStrategy::Preferred => SheetMode::Preferred(schema.preferred_sheets),
            MergeStrategy::All => SheetMode::AllSheets,
        };
        let report = merge_tables_with_mode(
            &schema,
            req.inputs[0].parent().unwrap_or(std::path::Path::new(".")),
            &req.output,
            mode,
        )
        .map_err(|e| FileAbilityError::Other(format!("表格合并失败: {e}")))?;
        return Ok(MergeOutcome::Xlsx {
            rows: report.total_rows,
            note: format!(
                "处理 {} 文件, 失败 {}",
                report.files_processed,
                report.files_failed.len()
            ),
        });
    }
    // 混合格式 → L0 文本级聚合
    let mut chunks: Vec<String> = Vec::new();
    let mut failed = 0usize;
    for p in &req.inputs {
        match crate::l1_action::nt_file_ability::helpers::extract_text(p) {
            Ok(text) => {
                let name = p
                    .file_name()
                    .map(|f| f.to_string_lossy().into_owned())
                    .unwrap_or_default();
                chunks.push(format!("=== {name} ===\n{text}"));
            }
            Err(e) => {
                failed += 1;
                chunks.push(format!("=== {} ===\n<提取失败: {e}>", p.display()));
            }
        }
    }
    let merged = chunks.join("\n\n");
    // 输出扩展名决定格式 (无则 .txt)
    let ext = req
        .output
        .extension()
        .and_then(|x| x.to_str())
        .map(|x| x.to_lowercase())
        .unwrap_or_default();
    let write = if ext == "md" {
        std::fs::write(&req.output, format!("# 合并文本\n\n{merged}"))
    } else {
        std::fs::write(&req.output, &merged)
    };
    write.map_err(FileAbilityError::Io)?;
    Ok(MergeOutcome::Text {
        items: req.inputs.len() - failed,
        note: format!(
            "L0 文本级聚合 (混合格式), 成功 {} / 失败 {}",
            req.inputs.len() - failed,
            failed
        ),
    })
}
