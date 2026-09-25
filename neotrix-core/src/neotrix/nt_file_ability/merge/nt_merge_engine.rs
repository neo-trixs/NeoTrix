//! 通用多表合并引擎 (L2 表格语义层 — 零领域知识)。
//!
//! 领域知识全部来自传入的 `MergeSchema` (L3), 本引擎对任何表格目录+对应 schema 通用。
//! 由 `merge.rs` 门面重导出, 外部路径不变。

use std::path::Path;

use super::super::excel::tables::{read_csv, read_xlsx_sheets_all, write_csv, write_xlsx_table};
use super::nt_merge_types::{
    ColumnType, ConsolidationReport, MergeSchema, SheetMode, PRICE_TABLE_SCHEMA,
};
use crate::neotrix::nt_file_ability::types::{FileAbilityError, Result, TableData};

/// 通用多表合并引擎 (L2 表格语义层 — 零领域知识)。
/// 领域知识全部来自传入的 `MergeSchema` (L3), 本引擎对任何表格目录+对应 schema 通用。
///
/// - 扫描 src_dir 下 xlsx/csv/tsv (跳过 skip_prefixes)
/// - 列名变体 → schema.standard_columns 归一化
/// - 单位规则 (schema.unit_rules) 补充缺失单位
/// - 供应商列缺失时从文件名推导 (schema.filename_suffixes)
/// - value_columns 非空计数统计
/// - 附加透出列 (schema.extra_columns, 含 _source_file)
/// - 输出 XLSX + 返回合并报告
pub fn merge_tables_with(
    schema: &MergeSchema,
    src_dir: impl AsRef<Path>,
    output: impl AsRef<Path>,
) -> Result<ConsolidationReport> {
    let mode = if schema.preferred_sheets.is_empty() {
        SheetMode::AllSheets
    } else {
        SheetMode::Preferred(schema.preferred_sheets)
    };
    merge_tables_with_mode(schema, src_dir, output, mode)
}

/// 通用多表合并引擎 + 运行时 sheet 策略 (R-2026-08-20)。
///
/// 与 [`merge_tables_with`] 同实现, 但 sheet 选择策略由 `SheetMode` 运行时决定,
/// 无需编译期变体函数。覆盖:
/// - `FirstSheet` — 每个文件取第一个 sheet (本次"已核对待确认"场景)
/// - `Preferred`  — 修改版优先 (默认 schema.preferred_sheets)
/// - `AllSheets`  — 旧行为 (全 sheet 遍历合并)
pub fn merge_tables_with_mode(
    schema: &MergeSchema,
    src_dir: impl AsRef<Path>,
    output: impl AsRef<Path>,
    mode: SheetMode,
) -> Result<ConsolidationReport> {
    schema.validate().map_err(FileAbilityError::Parse)?;
    let mut report = ConsolidationReport::default();
    let mut table = TableData {
        name: format!("统一{}", schema.name),
        headers: schema
            .standard_columns
            .iter()
            .map(|s| s.to_string())
            .collect(),
        rows: Vec::new(),
    };
    // 标准列名 → 索引
    let std_idx: std::collections::HashMap<String, usize> = schema
        .standard_columns
        .iter()
        .enumerate()
        .map(|(i, s)| (s.to_string(), i))
        .collect();
    // 附加透出列
    let mut extra_data: Vec<Vec<String>> = Vec::new();

    // 扫描输入目录
    let mut entries: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(read) = std::fs::read_dir(src_dir.as_ref()) {
        for e in read.flatten() {
            let p = e.path();
            let ext = p
                .extension()
                .and_then(|x| x.to_str())
                .map(|x| x.to_lowercase())
                .unwrap_or_default();
            if matches!(ext.as_str(), "xlsx" | "csv" | "tsv") {
                // 跳过已合并输出 (防止重复合并自身产物)
                let name = p
                    .file_name()
                    .map(|f| f.to_string_lossy().into_owned())
                    .unwrap_or_default()
                    .to_lowercase();
                if schema
                    .skip_prefixes
                    .iter()
                    .any(|pfx| name.starts_with(&pfx.to_lowercase()))
                {
                    continue;
                }
                entries.push(p);
            }
        }
    }
    entries.sort();

    for path in entries {
        let ext = path
            .extension()
            .and_then(|x| x.to_str())
            .unwrap_or("")
            .to_lowercase();
        let srcs: Vec<TableData> = match ext.as_str() {
            "xlsx" => match read_xlsx_sheets_all(&path) {
                Ok(tables) => select_preferred_sheets(tables, mode),
                Err(e) => {
                    report
                        .files_failed
                        .push((path.display().to_string(), e.to_string()));
                    continue;
                }
            },
            "csv" | "tsv" => match read_csv(&path) {
                Ok(t) => vec![t],
                Err(e) => {
                    report
                        .files_failed
                        .push((path.display().to_string(), e.to_string()));
                    continue;
                }
            },
            _ => continue,
        };
        if srcs.is_empty() {
            continue;
        }
        report.files_processed += 1;
        // 来源名 (从文件名剥离序号/后缀)
        let source_name = derive_source_name(&path, schema);
        // 文件名 (用于 _source_file)
        let file_name = path
            .file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_default();

        // 多 sheet 逐 sheet 合并 (E13: 每个 sheet 都可能含独立数据)
        // 同文件内跨 sheet 去重 (E14: key 用 schema.dedup_columns, 默认去重启用)
        let mut seen: std::collections::HashSet<Vec<String>> = std::collections::HashSet::new();
        let dedup_idx: Vec<Option<usize>> = schema
            .dedup_columns
            .iter()
            .map(|d| std_idx.get(*d).copied())
            .collect();
        for src in srcs {
            // 归一化源表头 → 标准列名
            let norm_headers: Vec<String> = src
                .headers
                .iter()
                .map(|h| schema.normalize_column(h))
                .collect();
            for row in &src.rows {
                // 标准列填充
                let mut std_row = vec![String::new(); schema.standard_columns.len()];
                for (c, h) in norm_headers.iter().enumerate() {
                    if let Some(&idx) = std_idx.get(h) {
                        let v = row.get(c).cloned().unwrap_or_default();
                        std_row[idx] = if v.trim().is_empty() {
                            String::new()
                        } else {
                            v.trim().to_string()
                        };
                    }
                }
                // 供应商列缺失 → 用文件名推导回填
                if let Some(sup) = schema.supplier_column {
                    if !norm_headers.iter().any(|h| h == sup) {
                        if let Some(&idx) = std_idx.get(sup) {
                            std_row[idx] = source_name.clone();
                        }
                    }
                }
                // 单位规则补充
                for rule in schema.unit_rules {
                    if let Some(&idx) = std_idx.get(rule.column) {
                        let v = std_row[idx].trim().to_string();
                        if !v.is_empty()
                            && !rule
                                .skip_if_contains
                                .iter()
                                .any(|mark| v.to_lowercase().contains(&mark.to_lowercase()))
                        {
                            std_row[idx] = format!("{v}{}", rule.suffix);
                        }
                    }
                }
                // 空值标记 → 留空 (不补单位, 不产出垃圾行)
                for (col, markers) in schema.empty_markers {
                    if let Some(&idx) = std_idx.get(*col) {
                        if markers.iter().any(|m| std_row[idx].trim() == *m) {
                            std_row[idx] = String::new();
                        }
                    }
                }
                // 同文件内跨 sheet 去重 (E14): key = dedup_columns 非空拼接
                if !dedup_idx.is_empty() {
                    let key: Vec<String> = dedup_idx
                        .iter()
                        .map(|oi| oi.map(|i| std_row[i].clone()).unwrap_or_default())
                        .collect();
                    // 至少一个 key 字段非空才有意义
                    if key.iter().any(|k| !k.is_empty()) && !seen.insert(key) {
                        report
                            .dedup_rows
                            .get_or_insert_with(Vec::new)
                            .push(format!("{}::{}", file_name, src.name));
                        continue;
                    }
                }
                table.rows.push(std_row.clone());
                // 附加列 (schema.extra_columns; 末位 _source_file 填 文件名[::sheet名])
                let mut extra = vec![String::new(); schema.extra_columns.len()];
                for (c, h) in norm_headers.iter().enumerate() {
                    for (ei, ecol) in schema.extra_columns.iter().enumerate() {
                        if h == *ecol {
                            extra[ei] = row.get(c).cloned().unwrap_or_default();
                        }
                    }
                }
                if let Some(last) = extra.last_mut() {
                    if schema.extra_columns.last() == Some(&"_source_file") {
                        let sheet_suffix = if !src.name.is_empty() {
                            format!("::{}", src.name)
                        } else {
                            String::new()
                        };
                        *last = format!("{file_name}{sheet_suffix}");
                    }
                }
                extra_data.push(extra);
                // value_columns 统计 (基于去重后 std_row, 非空计数)
                let has_value = schema.value_columns.iter().any(|vc| {
                    std_idx
                        .get(*vc)
                        .and_then(|&i| std_row.get(i))
                        .map(|v| !v.trim().is_empty())
                        .unwrap_or(false)
                });
                if has_value {
                    report.usd_rows += 1;
                }
            }
            // 清理: norm_headers 作用域结束
        }
    }

    // 拼接附加列到输出表格
    table
        .headers
        .extend(schema.extra_columns.iter().map(|s| s.to_string()));
    for (i, r) in table.rows.iter_mut().enumerate() {
        if let Some(e) = extra_data.get(i) {
            r.extend(e.iter().cloned());
        } else {
            r.extend(vec![String::new(); schema.extra_columns.len()]);
        }
    }
    report.total_rows = table.rows.len();
    report.output = output.as_ref().display().to_string();

    // 输出数据校验 (阶段2): 数字列非数值 → validation_warnings
    for (col, ctype) in schema.column_types.iter() {
        if *ctype != ColumnType::Numeric {
            continue;
        }
        let col_idx = std_idx.get(*col);
        let Some(&col_idx) = col_idx else { continue };
        for (ri, row) in table.rows.iter().enumerate() {
            let Some(v) = row.get(col_idx) else { continue };
            let v = v.trim();
            if v.is_empty() {
                continue; // 空值不算错误
            }
            if parse_numeric(v).is_none() {
                report.validation_warnings.push(format!(
                    "row{} 列'{}' 非数值: '{}'",
                    ri + 1,
                    col,
                    v
                ));
            }
        }
    }

    // 输出按扩展名分发: .csv/.tsv → 文本表格 (UTF-8 BOM + 逗号/制表符), 否则 xlsx
    let ext = output
        .as_ref()
        .extension()
        .and_then(|x| x.to_str())
        .map(|x| x.to_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "csv" => write_csv(output, &table, ',', true)?,
        "tsv" => write_csv(output, &table, '\t', true)?,
        _ => write_xlsx_table(output, &table)?,
    }
    Ok(report)
}

/// 宽松数值解析 (兼容 千分位/货币符/单位后缀/科学计数法)。
/// 返回 Some(数值) 若可解析, None 否则。
fn parse_numeric(v: &str) -> Option<f64> {
    let t = v.replace([',', '￥', '¥', '$'], "");
    // 单位后缀 (如 "2.5kg" / "300元") — 剥离尾部非数值字符
    let trimmed: String = t
        .chars()
        .take_while(|c| {
            c.is_ascii_digit() || *c == '.' || *c == '-' || *c == '+' || *c == 'e' || *c == 'E'
        })
        .collect();
    trimmed.parse::<f64>().ok()
}

/// ⚠️ 废弃: 请使用 `merge_tables_with_mode` 代替
#[deprecated(since = "0.2.0", note = "使用 merge_tables_with_mode 代替")]
pub fn consolidate_tables(
    src_dir: impl AsRef<Path>,
    output: impl AsRef<Path>,
) -> Result<ConsolidationReport> {
    let mode = if PRICE_TABLE_SCHEMA.preferred_sheets.is_empty() {
        SheetMode::AllSheets
    } else {
        SheetMode::Preferred(PRICE_TABLE_SCHEMA.preferred_sheets)
    };
    merge_tables_with_mode(&PRICE_TABLE_SCHEMA, src_dir, output, mode)
}

/// ⚠️ 废弃: 请使用 `merge_tables_with_mode` 代替
#[deprecated(since = "0.2.0", note = "使用 merge_tables_with_mode 代替")]
pub fn consolidate_tables_first_sheet(
    src_dir: impl AsRef<Path>,
    output: impl AsRef<Path>,
) -> Result<ConsolidationReport> {
    merge_tables_with_mode(&PRICE_TABLE_SCHEMA, src_dir, output, SheetMode::FirstSheet)
}

/// 价格表合并 + 运行时 sheet 策略 — 统一参数化入口 (推荐)。
/// 覆盖全部场景: 首 sheet / 修改版优先 / 全 sheet, 无需编译期变体。
pub fn consolidate_tables_with_mode(
    src_dir: impl AsRef<Path>,
    output: impl AsRef<Path>,
    mode: SheetMode,
) -> Result<ConsolidationReport> {
    merge_tables_with_mode(&PRICE_TABLE_SCHEMA, src_dir, output, mode)
}

/// 多 sheet 表格按 sheet_mode 选择 (运行时策略):
/// - FirstSheet: 只取第一个 sheet
/// - Preferred(list): 命中任一 (trim 精确匹配) 只保留该 sheet; 未命中取第一个
/// - AllSheets: 保留全部
pub(crate) fn select_preferred_sheets(
    tables: Vec<TableData>,
    mode: SheetMode,
) -> Vec<TableData> {
    match mode {
        SheetMode::FirstSheet => tables.into_iter().take(1).collect(),
        SheetMode::AllSheets => tables,
        SheetMode::Preferred(preferred) => {
            if preferred.is_empty() {
                return tables;
            }
            if let Some(t) = tables
                .iter()
                .find(|t| preferred.iter().any(|p| t.name.trim() == *p))
            {
                return vec![t.clone()];
            }
            tables.into_iter().take(1).collect()
        }
    }
}

/// 从文件名推导来源名 (通用: 剥离序号/后缀, 后缀来自 schema.filename_suffixes)。
/// 例: "4、玉鹏价格_报价模板-修改版" → "玉鹏"
pub(crate) fn derive_source_name(path: &Path, schema: &MergeSchema) -> String {
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut s = stem.as_str();
    // 剥离开头的数字序号 (如 "4、", "44. ", "7、")
    if let Some(stripped) = strip_leading_num(s) {
        s = stripped;
    }
    // 剥离后缀标记 (schema.filename_suffixes)
    for marker in schema.filename_suffixes {
        if let Some(pos) = s.find(marker) {
            s = &s[..pos];
        }
    }
    s.trim_matches(|c: char| c == '_' || c == '-' || c == '、' || c == '.' || c == ' ')
        .to_string()
}

/// 剥离开头的数字序号 (返回剩余部分)
fn strip_leading_num(s: &str) -> Option<&str> {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == 0 {
        return None;
    }
    // 跳过后续分隔符 (、. 空格)
    let rest = &s[i..];
    let rest = rest.trim_start_matches(['、', '.', ' ', '-', '_']);
    if rest.is_empty() {
        None
    } else {
        Some(rest)
    }
}
