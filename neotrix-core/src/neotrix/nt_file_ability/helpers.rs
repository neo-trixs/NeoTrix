//! 便捷函数 — 薄封装 FileAbility 的常用操作。

use std::path::Path;

use office_oxide::{create, DocumentFormat};

use super::core::FileAbility;
use super::types::{FileAbilityError, Result};

/// 提取任何文件的纯文本
pub fn extract_text(path: impl AsRef<Path>) -> Result<String> {
    let mut ab = FileAbility::open(path)?;
    ab.register_consumer();
    Ok(ab.plain_text())
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