//! PDF 文本编辑 + 表格结构化提取 (R-P79 生产接线)。
//!
//! - 编辑: span 级 redact + 原位替换, 包装 neotrix-types FileParser::edit_pdf_text。
//! - 表格: 从解码内容流文本 run 做行/列聚类重建网格, 包装 extract_pdf_tables。
//!
//! 纯 Rust 实现, 不依赖外部二进制 (R-P48)。替换文本支持任意 Unicode: 提供 TTF
//! 时嵌入 Type0/Identity-H 子集字体, 否则回退 Helvetica base14 (仅 Latin-1)。

use std::path::Path;

use super::super::types::{FileAbilityError, Result};

/// 单条 PDF 文本编辑 (镜像 neotrix-types PdfTextEdit)。
#[derive(Debug, Clone)]
pub struct PdfEdit {
    /// 目标页 (1-based)
    pub page: u32,
    /// 待定位的源文本
    pub find: String,
    /// 替换文本 (None = 仅删除)
    pub replace: Option<String>,
}

/// 编辑 PDF 文本并写出。
///
/// - `src` 源 PDF; `output` 目标路径。
/// - `edits` 编辑列表 (按序应用)。
/// - `ttf` 嵌入字体 (支持任意 Unicode); `None` 时替换文本仅限 Latin-1。
///
/// 返回应用成功的编辑条数。
pub fn edit_pdf(
    src: impl AsRef<Path>,
    output: impl AsRef<Path>,
    edits: &[PdfEdit],
    ttf: Option<&[u8]>,
) -> Result<usize> {
    use neotrix_types::core::file_parser::pdf::{PdfEditError, PdfTextEdit};

    let data = std::fs::read(src.as_ref()).map_err(FileAbilityError::Io)?;
    let edits: Vec<PdfTextEdit> = edits
        .iter()
        .map(|e| PdfTextEdit {
            page: e.page,
            find: e.find.clone(),
            replace: e.replace.clone(),
        })
        .collect();
    let out = neotrix_types::core::file_parser::FileParser::edit_pdf_text(&data, &edits, ttf)
        .map_err(|e| match e {
            PdfEditError::Parse(m) => FileAbilityError::Parse(m),
            PdfEditError::NotFound { find } => {
                FileAbilityError::Other(format!("未找到文本: {find}"))
            }
            PdfEditError::PageNotFound { page } => {
                FileAbilityError::Other(format!("页面 {page} 不存在"))
            }
            PdfEditError::Font(m) => FileAbilityError::Parse(m),
            PdfEditError::UnsupportedGlyph(c) => {
                FileAbilityError::Other(format!("字形不支持: {c}"))
            }
            PdfEditError::Write(m) => FileAbilityError::Other(format!("写 PDF 失败: {m}")),
            PdfEditError::Merge(m) => FileAbilityError::Other(format!("PDF 合并失败: {m}")),
        })?;
    std::fs::write(output.as_ref(), out).map_err(FileAbilityError::Io)?;
    Ok(edits.len())
}

/// 从 PDF 提取表格网格 (行/列聚类), 每张表格渲染 Markdown。
///
/// 纯坐标启发式, 对逐单元格绘制的表格型 PDF (设备清单/报价表) 有效;
/// 复杂合并单元格需外部分析器。返回 (页号, 列数, markdown)。
pub fn extract_pdf_tables(src: impl AsRef<Path>) -> Result<Vec<(u32, usize, String)>> {
    use neotrix_types::core::file_parser::pdf::PdfTable;

    let data = std::fs::read(src.as_ref()).map_err(FileAbilityError::Io)?;
    let tables = neotrix_types::core::file_parser::FileParser::extract_pdf_tables(&data);
    if tables.is_empty() {
        return Err(FileAbilityError::Other("未检测到表格形态的文本布局".to_string()));
    }
    Ok(tables
        .iter()
        .map(|t: &PdfTable| (t.page, t.columns.len(), t.to_markdown()))
        .collect())
}
