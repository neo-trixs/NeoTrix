//! 统一 XLSX 解析入口 (L2 表格语义层) — **唯一对外入口**
//!
//! 本模块是 XLSX 解析的唯一对外入口。所有外部调用者（CLI、Capability、merge 等）
//! 应通过本模块提供的函数访问 XLSX 数据，**禁止直接调用** `tables::read_xlsx_sheets_all`
//! 或 `xlsx_fast::read_xlsx_fast`（除模块内部路由外）。
//!
//! 对外接口:
//! - [`parse_xlsx`]: 通用解析 (自动选择轻/重型，默认)
//! - [`parse_xlsx_fast`]: 快速解析 (纯文本，单 sheet)
//! - [`parse_xlsx_full`]: 完整解析 (公式/日期/多 sheet)
//! - [`parse_xlsx_with_config`]: 带配置解析 (max_rows/max_cols 裁剪)
//! - [`extract_xlsx_text`]: 文本提取 (用于 FileParser 兼容)
//!
//! 设计原则:
//! - **单一入口**: `FileAbility` 的 `xlsx_sheet*` 方法已标记 `#[deprecated]`，禁止外部调用
//! - 按场景自动选择最优解析器
//! - 统一返回 `TableData` 类型
//!
//! # XLSX 解析路由策略
//!
//! ## 双引擎设计
//! NeoTrix 保留两套 XLSX 解析引擎，各有适用场景:
//!
//! ### xlsx_fast (ZIP + quick_xml)
//! - **性能**: 0.2-0.5s/file
//! - **功能**: 纯文本提取，无公式/日期转换
//! - **适用**: 批量解析、模板检测、表头扫描、大文件 (>10MB)
//! - **限制**: 单 sheet，无数值精度
//!
//! ### tables (calamine)
//! - **性能**: 1-2s/file
//! - **功能**: 完整解析，支持公式/日期/多 sheet/数值精度
//! - **适用**: 需要数值精度、公式求值、多 sheet 的场景
//! - **限制**: 较慢
//!
//! ### 路由策略 (xlsx_parser)
//! ```text
//! Auto 模式 (默认):
//!   文件 > 10MB → xlsx_fast
//!   文件 ≤ 10MB → tables
//!
//! Fast 模式:
//!   始终使用 xlsx_fast
//!
//! Full 模式:
//!   始终使用 tables
//! ```
//!
//! ## L3 表示层 (table_presenter)
//! 所有解析结果均可通过 TablePresenter 转换为:
//! - **Markdown**: 供 LLM 理解和推理
//! - **JSON**: 供 Function Calling / Structured Output
//! - **CSV**: 最省 token (48 tokens/行 vs JSON 111 tokens/行)
//!
//! ## 分块策略 (chunk_planner)
//! 当表格超过 LLM 上下文窗口时:
//! - 按行分块 (默认，每 chunk ≤ 16K chars ≈ 4096 tokens)
//! - 重叠 2 行 (防止边界丢失上下文)
//! - 每个 chunk 保留表头

use std::path::Path;

use super::tables;
use super::types::{FileAbilityError, Result, TableData};
use super::xlsx_fast;

/// 解析模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseMode {
    /// 快速模式: 只解析 sharedStrings + sheet1，纯文本
    Fast,
    /// 完整模式: calamine 驱动，支持公式/日期
    Full,
    /// 自动模式: 根据文件大小选择 (默认)
    Auto,
}

impl Default for ParseMode {
    fn default() -> Self {
        Self::Auto
    }
}

/// 解析配置
#[derive(Debug, Clone)]
pub struct ParseConfig {
    pub mode: ParseMode,
    pub max_rows: Option<usize>,
    pub max_cols: Option<usize>,
}

impl Default for ParseConfig {
    fn default() -> Self {
        Self {
            mode: ParseMode::Auto,
            max_rows: None,
            max_cols: None,
        }
    }
}

/// 统一 XLSX 解析入口 (自动选择解析器)
pub fn parse_xlsx(path: impl AsRef<Path>) -> Result<Vec<TableData>> {
    parse_xlsx_with_config(path, &ParseConfig::default())
}

/// 带配置的 XLSX 解析
pub fn parse_xlsx_with_config(
    path: impl AsRef<Path>,
    config: &ParseConfig,
) -> Result<Vec<TableData>> {
    let path = path.as_ref();

    if !path.exists() {
        return Err(FileAbilityError::Parse(format!(
            "文件不存在: {}",
            path.display()
        )));
    }

    let mode = if config.mode == ParseMode::Auto {
        let file_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        // 大文件 (>10MB) 用快速模式避免 calamine 内存峰值
        if file_size > 10 * 1024 * 1024 {
            ParseMode::Fast
        } else {
            ParseMode::Full
        }
    } else {
        config.mode
    };

    match mode {
        ParseMode::Fast => parse_xlsx_fast_mode(path, config),
        ParseMode::Full => parse_xlsx_full_mode(path, config),
        ParseMode::Auto => unreachable!(),
    }
}

/// 快速解析入口 (zip+quick_xml，纯文本)
pub fn parse_xlsx_fast(path: impl AsRef<Path>) -> Result<Vec<TableData>> {
    let config = ParseConfig {
        mode: ParseMode::Fast,
        ..Default::default()
    };
    parse_xlsx_with_config(path, &config)
}

/// 完整解析入口 (calamine 驱动，公式/日期)
pub fn parse_xlsx_full(path: impl AsRef<Path>) -> Result<Vec<TableData>> {
    let config = ParseConfig {
        mode: ParseMode::Full,
        ..Default::default()
    };
    parse_xlsx_with_config(path, &config)
}

/// 快速模式解析: xlsx_fast grid → TableData
fn parse_xlsx_fast_mode(path: &Path, config: &ParseConfig) -> Result<Vec<TableData>> {
    let grid = xlsx_fast::read_xlsx_fast(path)?;

    if grid.is_empty() {
        return Ok(Vec::new());
    }

    let header_idx = grid
        .iter()
        .position(|row| row.iter().any(|c| !c.trim().is_empty()))
        .unwrap_or(0);

    let headers: Vec<String> = grid.get(header_idx).cloned().unwrap_or_default();
    let rows: Vec<Vec<String>> = grid
        .iter()
        .enumerate()
        .filter(|(i, row)| *i > header_idx && row.iter().any(|c| !c.trim().is_empty()))
        .map(|(_, row)| row.clone())
        .collect();

    let max_cols = config.max_cols.unwrap_or(usize::MAX);
    let max_rows = config.max_rows.unwrap_or(usize::MAX);

    let truncated_headers: Vec<String> = headers.into_iter().take(max_cols).collect();
    let truncated_rows: Vec<Vec<String>> = rows
        .into_iter()
        .take(max_rows)
        .map(|r| r.into_iter().take(max_cols).collect())
        .collect();

    let name = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();

    Ok(vec![TableData {
        name,
        headers: truncated_headers,
        rows: truncated_rows,
    }])
}

/// 完整模式解析: calamine 多 sheet → TableData
fn parse_xlsx_full_mode(path: &Path, config: &ParseConfig) -> Result<Vec<TableData>> {
    let mut tables = tables::read_xlsx_sheets_all(path)?;

    let max_cols = config.max_cols.unwrap_or(usize::MAX);
    let max_rows = config.max_rows.unwrap_or(usize::MAX);

    for table in &mut tables {
        table.headers.truncate(max_cols);
        table.rows = table
            .rows
            .clone()
            .into_iter()
            .take(max_rows)
            .map(|r| r.into_iter().take(max_cols).collect())
            .collect();
    }

    Ok(tables)
}

/// 快速提取 XLSX 文本 (用于 FileParser 兼容)
pub fn extract_xlsx_text(path: impl AsRef<Path>) -> Result<String> {
    let tables = parse_xlsx(path)?;

    let mut text = String::new();
    for table in &tables {
        text.push_str(&format!("=== {} ===\n", table.name));
        text.push_str(&table.headers.join(" | "));
        text.push('\n');
        for row in &table.rows {
            text.push_str(&row.join(" | "));
            text.push('\n');
        }
        text.push('\n');
    }

    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mode_default() {
        assert_eq!(ParseMode::default(), ParseMode::Auto);
    }

    #[test]
    fn test_parse_config_default() {
        let cfg = ParseConfig::default();
        assert_eq!(cfg.mode, ParseMode::Auto);
        assert!(cfg.max_rows.is_none());
        assert!(cfg.max_cols.is_none());
    }

    #[test]
    fn test_parse_xlsx_nonexistent_file() {
        let err = parse_xlsx("/tmp/nonexistent_xlsx_test_file.xlsx").unwrap_err();
        assert!(
            err.to_string().contains("文件不存在"),
            "应报文件不存在: {err}"
        );
    }
}
