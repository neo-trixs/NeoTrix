//! 便捷函数 — 薄封装 FileAbility 的常用操作。

use std::path::Path;

use office_oxide::{create, DocumentFormat};

use super::core::FileAbility;
use super::tables::{read_csv, read_xlsx_table};
use super::types::{DirExtractEntry, DirExtractReport, FileAbilityError, FileKind, Result};

/// 提取任何文件的纯文本
pub fn extract_text(path: impl AsRef<Path>) -> Result<String> {
    let mut ab = FileAbility::open(path)?;
    ab.register_consumer();
    Ok(ab.plain_text())
}

/// 合并多个 PDF 为单个文档。
///
/// 结构级合并 (对象图重建 + 单一 Catalog/Pages), 文本/字体/资源对象随对象图
/// 整体迁移。首个输入作为基座, 页面按输入顺序拼接。
pub fn merge_pdfs(inputs: &[std::path::PathBuf]) -> Result<Vec<u8>> {
    use neotrix_types::core::file_parser::pdf::PdfEditError;

    let data: Result<Vec<Vec<u8>>> = inputs
        .iter()
        .map(|p| std::fs::read(p).map_err(FileAbilityError::Io))
        .collect();
    let data = data?;
    let out = neotrix_types::core::file_parser::pdf::merge_pdfs(&data).map_err(|e| {
        match e {
            PdfEditError::Parse(m) => FileAbilityError::Parse(m),
            _ => FileAbilityError::Other(e.to_string()),
        }
    })?;
    Ok(out)
}

/// 目录级统一提取 (横向推广: FileKind×读 统一入口)。
///
/// 扫描目录下所有文件, 逐文件按大类提取:
/// - office/文本/PDF → 纯文本
/// - xlsx/csv → 文本 + 表格行数
/// - 图像/音视频 → 仅元数据 (text 为空)
/// 不递归子目录; 单文件失败不中断整体, 记入 entry.error。
pub fn extract_dir(dir: impl AsRef<Path>) -> Result<DirExtractReport> {
    let dir = dir.as_ref();
    let mut entries = Vec::new();
    let mut succeeded = 0usize;
    let mut failed = 0usize;
    let mut total_chars = 0usize;
    let rd = std::fs::read_dir(dir).map_err(FileAbilityError::Io)?;
    for e in rd.flatten() {
        let path = e.path();
        if !path.is_file() {
            continue;
        }
        let rel = path
            .strip_prefix(dir)
            .unwrap_or(&path)
            .display()
            .to_string();
        let mut entry = DirExtractEntry {
            path: rel,
            kind: "unknown".to_string(),
            text: String::new(),
            table_rows: None,
            error: None,
        };
        match FileAbility::open(&path) {
            Ok(fa) => {
                entry.kind = format!("{:?}", fa.kind());
                match extract_dir_one(&path, fa.kind()) {
                    Ok((text, table_rows)) => {
                        total_chars += text.chars().count();
                        entry.text = text;
                        entry.table_rows = table_rows;
                        succeeded += 1;
                    }
                    Err(err) => {
                        entry.error = Some(err.to_string());
                        failed += 1;
                    }
                }
            }
            Err(err) => {
                entry.error = Some(err.to_string());
                failed += 1;
            }
        }
        entries.push(entry);
    }
    Ok(DirExtractReport {
        entries,
        succeeded,
        failed,
        total_chars,
    })
}

/// 单文件按大类提取 (内部)
fn extract_dir_one(
    path: &Path,
    kind: FileKind,
) -> Result<(String, Option<usize>)> {
    // xlsx/csv → 表格读 (含行数)
    let is_xlsx = matches!(kind, FileKind::Office(office_oxide::DocumentFormat::Xlsx));
    let is_csv = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase() == "csv")
        .unwrap_or(false);
    if is_xlsx || is_csv {
        let table = if is_csv {
            read_csv(path)
        } else {
            read_xlsx_table(path)
        };
        if let Ok(table) = table {
            let text = table
                .rows
                .iter()
                .map(|r| r.join(", "))
                .collect::<Vec<_>>()
                .join("\n");
            return Ok((text, Some(table.row_count())));
        }
    }
    // 其余 → 纯文本 (图像/音视频 text 为空, 仅元数据)
    let mut fa = FileAbility::open(path)?;
    fa.register_consumer();
    Ok((fa.plain_text(), None))
}

/// 转换任何 Office 文件为 Markdown
pub fn to_markdown(path: impl AsRef<Path>) -> Result<String> {
    let mut ab = FileAbility::open(path)?;
    ab.register_consumer();
    Ok(ab.to_markdown())
}

/// 占位符替换 (返回替换次数)
pub fn replace_placeholder(path: impl AsRef<Path>, find: &str, replace: &str) -> Result<usize> {
    let ab = FileAbility::open(path)?;
    ab.replace_placeholder(find, replace)
}

/// 保存/导出能力句柄到目标路径
pub fn save_edited(ability: &FileAbility, target: impl AsRef<Path>) -> Result<()> {
    ability.save_as(target)
}

/// 健康检查 (Dark Forest 生存 + 内容快照)
pub fn check_health(path: impl AsRef<Path>) -> String {
    match FileAbility::open(&path) {
        Ok(mut ab) => {
            ab.register_consumer();
            format!(
                "FileHealth {{ path: {}, kind: {:?}, mime: {}, size: {}, maturity: {:?}, consumers: {} }}",
                path.as_ref().display(),
                ab.kind(),
                ab.mime_type(),
                ab.size_bytes(),
                ab.maturity(),
                ab.has_consumers(),
            )
        }
        Err(e) => format!("FileHealth ERROR: {e}"),
    }
}

/// 用 Markdown 创建 Office 文档 (office_oxide `create_from_markdown`)
pub fn create_from_markdown(
    markdown: &str,
    format: DocumentFormat,
    target: impl AsRef<Path>,
) -> Result<()> {
    create::create_from_markdown(markdown, format, target).map_err(FileAbilityError::Office)
}