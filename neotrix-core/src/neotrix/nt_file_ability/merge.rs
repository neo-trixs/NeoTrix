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

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::tables::{read_csv, read_xlsx_sheets_all, write_csv, write_xlsx_table};
use super::types::{FileAbilityError, Result, TableData};

/// 单重/尺寸等列的补单位规则
#[derive(Debug, Clone, Copy)]
pub struct UnitRule {
    /// 目标标准列名 (如 "单重(Kg)")
    pub column: &'static str,
    /// 值缺失该后缀时追加 (如 "kg")
    pub suffix: &'static str,
    /// 值中已含这些标记则跳过 (如 ["kg", "千克"])
    pub skip_if_contains: &'static [&'static str],
}

/// 标准列数据类型 (用于输出校验)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnType {
    /// 数值列 (可含单位后缀/千分位/货币符; 解析失败记 warning)
    Numeric,
    /// 文本列 (默认)
    Text,
}

/// 多 sheet 选择策略 — 运行时参数 (替代编译期 preferred_sheets 变体函数)。
///
/// 之前每新增一个 sheet 策略就要新写编译期变体函数 (如
/// `consolidate_tables_first_sheet`) 并重新编译 — 变体爆炸。本枚举将策略
/// 数据化, 一个引擎 + 运行时参数覆盖全部场景 (R-2026-08-20)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SheetMode {
    /// 只取第一个 sheet (每个文件首个 sheet 即目标数据)
    FirstSheet,
    /// 按 preferred_sheets 精确匹配 (trim), 命中则只用该 sheet; 否则取第一个
    Preferred(&'static [&'static str]),
    /// 保留全部 sheet (旧行为)
    AllSheets,
}

/// 合并 Schema — 领域知识数据 (纯 const, 编译期校验)
#[derive(Debug, Clone, Copy)]
pub struct MergeSchema {
    /// Schema 名 (如 "价格表")
    pub name: &'static str,
    /// 标准列序 (合并输出的目标列)
    pub standard_columns: &'static [&'static str],
    /// 列名变体 → 标准列 (每项 (标准列, [变体...]))
    pub column_variants: &'static [(&'static str, &'static [&'static str])],
    /// 供应商/来源名推导: 文件名剥离的后缀标记 (如 "价格表"/"报价模板")
    pub filename_suffixes: &'static [&'static str],
    /// 需要补单位的列规则
    pub unit_rules: &'static [UnitRule],
    /// 多 sheet 文件优先选用的 sheet 名 (trim 精确匹配; 命中则只用该 sheet, 否则取第一个 sheet)。
    /// 为空 = 保留旧行为 (全 sheet 遍历合并)。
    pub preferred_sheets: &'static [&'static str],
    /// 空值标记 → 留空 (列, [标记列表]) — 如 单重(Kg) 列 "/" 表示无数据。
    /// 该列命中标记时输出空串 (不补单位)。
    pub empty_markers: &'static [(&'static str, &'static [&'static str])],
    /// 用于统计的价值列 (如 USD 报价列), 必须 ∈ standard_columns
    pub value_columns: &'static [&'static str],
    /// 附加透出列 (如 "备注"/"_source_file")
    pub extra_columns: &'static [&'static str],
    /// 输入目录扫描时跳过的文件名前缀 (如已合并输出自身)
    pub skip_prefixes: &'static [&'static str],
    /// 源表无供应商列时, 回退用文件名推导并写入该列
    pub supplier_column: Option<&'static str>,
    /// 同文件内跨 sheet 去重键 (E14: 用 (口径,单价,产品小类) 而非型号 —
    /// 型号常是文件名回退/垃圾值; 为空则不去重)。
    /// 注意: 去重仅限同一文件内, 不同文件=不同供应商, 不去重。
    pub dedup_columns: &'static [&'static str],
    /// 标准列数据类型 (输出校验: 数字列非数值 → validation_warning)
    pub column_types: &'static [(&'static str, ColumnType)],
}

impl MergeSchema {
    /// 校验 schema 一致性 (编译期无法全查, 运行时断言):
    /// 标准列唯一 / value_columns ∈ standard_columns / extra 不与标准列冲突。
    pub fn validate(&self) -> std::result::Result<(), String> {
        let mut seen = std::collections::HashSet::new();
        for (i, c) in self.standard_columns.iter().enumerate() {
            if !seen.insert(*c) {
                return Err(format!("standard_columns[{}] 重复: '{}'", i, c));
            }
        }
        for v in self.value_columns {
            if !self.standard_columns.contains(v) {
                return Err(format!(
                    "value_column '{}' 不在 standard_columns 中 (schema '{}')",
                    v, self.name
                ));
            }
        }
        for e in self.extra_columns {
            if self.standard_columns.contains(e) {
                return Err(format!(
                    "extra_column '{}' 与 standard_columns 冲突 (schema '{}')",
                    e, self.name
                ));
            }
        }
        if let Some(sup) = self.supplier_column {
            if !self.standard_columns.contains(&sup) {
                return Err(format!(
                    "supplier_column '{}' 不在 standard_columns 中 (schema '{}')",
                    sup, self.name
                ));
            }
        }
        for d in self.dedup_columns {
            if !self.standard_columns.contains(d) {
                return Err(format!(
                    "dedup_column '{}' 不在 standard_columns 中 (schema '{}')",
                    d, self.name
                ));
            }
        }
        for (col, _t) in self.column_types {
            if !self.standard_columns.contains(col) {
                return Err(format!(
                    "column_types 引用的列 '{}' 不在 standard_columns 中 (schema '{}')",
                    col, self.name
                ));
            }
        }
        Ok(())
    }

    /// 变体 → 标准列 (未命中返回原样, 与旧 normalize_column_name 行为一致)
    pub fn normalize_column(&self, name: &str) -> String {
        let t = name.trim();
        for (std, variants) in self.column_variants {
            if *std == t || variants.contains(&t) {
                return (*std).to_string();
            }
        }
        t.to_string()
    }

    /// 标准列 → 索引
    pub fn column_index(&self, name: &str) -> Option<usize> {
        self.standard_columns.iter().position(|c| *c == name)
    }
}

/// 价格表标准列 (兼容导出 — 由 PRICE_TABLE_SCHEMA 派生)
pub const PRICE_STANDARD_COLUMNS: &[&str] = &[
    "产品大类",
    "产品小类",
    "产品型号",
    "阀体材质",
    "阀板材质",
    "阀杆材质",
    "阀座材质",
    "驱动方式",
    "连接方式",
    "标准",
    "压力",
    "口径",
    "含税单价(元)",
    "美元报价(USD)",
    "青岛港FOB报价(USD)",
    "天津港FOB报价(USD)",
    "单重(Kg)",
    "供应商名称",
    "档次",
];

/// 价格表领域 Schema (L3 差异化知识 — 数据化, 引擎零内置)。
/// 迁移自: 原 PRICE_STANDARD_COLUMNS + normalize_column_name 变体表 + 供应商后缀。
pub const PRICE_TABLE_SCHEMA: MergeSchema = MergeSchema {
    name: "价格表",
    standard_columns: PRICE_STANDARD_COLUMNS,
    column_variants: &[
        ("产品大类", &["大类"]),
        ("产品小类", &["小类"]),
        ("产品型号", &["型号", "规格型号", "产品规格"]),
        (
            "阀体材质",
            &["阀体", "body材质", "BODY阀体", "BODY体", "壳材质"],
        ),
        ("阀板材质", &["阀板", "disc材质", "碟板材质", "蝶板材质"]),
        (
            "阀杆材质",
            &["阀杆", "stem材质", "MAIN SHAFT主软", "阀轴材质"],
        ),
        (
            "阀座材质",
            &["阀座", "seat材质", "SEAT RING座座环", "密封圈材质"],
        ),
        ("驱动方式", &["驱动", "操作方式"]),
        ("连接方式", &["连接", "连接形式"]),
        ("标准", &["执行标准", "设计标准"]),
        ("压力", &["公称压力", "压力等级", "PN"]),
        ("口径", &["公称通径", "DN", "尺寸"]),
        (
            "含税单价(元)",
            &["含税单价", "单价(元)", "单价", "价格(元)", "单价(含税)", "含税价(元)"],
        ),
        (
            "美元报价(USD)",
            &["美元价(USD)", "美元价", "美元报价", "USD报价", "单价(美元)", "美元单价"],
        ),
        (
            "青岛港FOB报价(USD)",
            &["青岛港FOB单价(元)", "青岛港FOB单价", "青岛港FOB", "青岛FOB"],
        ),
        (
            "天津港FOB报价(USD)",
            &["天津港FOB单价(元)", "天津港FOB单价", "天津港FOB", "天津FOB"],
        ),
        (
            "单重(Kg)",
            &["单重", "重量(Kg)", "重量", "单重(kg)", "预估单重(Kg)"],
        ),
        ("供应商名称", &["供应商", "厂家", "品牌"]),
        ("档次", &["等级", "级别"]),
        ("备注", &["说明", "备注信息"]),
    ],
    filename_suffixes: &[
        "价格_报价", "价格表", "报价模板", "_报价", "-报价", "价格表_报价", "已完善",
        "-已更新", "-修改版", "-中高档", "-中低档", "-第一版", "-第二版", "-第五版本", "-含税",
    ],
    // 表头已含单位 (单重(Kg)), 值保持纯数字, 不再追加 "kg" (R-2026-08 优化)
    unit_rules: &[],
    // 多 sheet 文件优先用 "修改版" sheet, 无则取第一个 sheet
    preferred_sheets: &["修改版"],
    // 单重(Kg) 空值标记 "/" → 留空 (不产出 "/kg" 垃圾行)
    empty_markers: &[("单重(Kg)", &["/"])],
    value_columns: &["美元报价(USD)", "青岛港FOB报价(USD)", "天津港FOB报价(USD)"],
    extra_columns: &["备注", "_source_file"],
    skip_prefixes: &["consolidated", "native_consolidated"],
    supplier_column: Some("供应商名称"),
    dedup_columns: &["口径", "含税单价(元)", "产品小类"],
    column_types: &[
        ("口径", ColumnType::Numeric),
        ("含税单价(元)", ColumnType::Numeric),
        ("美元报价(USD)", ColumnType::Numeric),
        ("青岛港FOB报价(USD)", ColumnType::Numeric),
        ("天津港FOB报价(USD)", ColumnType::Numeric),
        ("单重(Kg)", ColumnType::Numeric),
    ],
};

/// 列名变体 → 标准列 (兼容导出 — 委托 PRICE_TABLE_SCHEMA)
pub fn normalize_column_name(name: &str) -> String {
    PRICE_TABLE_SCHEMA.normalize_column(name)
}

/// 合并报告
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConsolidationReport {
    /// 处理的文件数
    pub files_processed: usize,
    /// 读取失败的文件 (路径, 原因)
    pub files_failed: Vec<(String, String)>,
    /// 总数据行数
    pub total_rows: usize,
    /// 含美元报价的行数
    pub usd_rows: usize,
    /// 输出路径
    pub output: String,
    /// 同文件内跨 sheet 去重跳过的行 (E14: 来源标注 文件名::sheet名)
    pub dedup_rows: Option<Vec<String>>,
    /// 输出数据校验告警 (数字列非数值)
    pub validation_warnings: Vec<String>,
}

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
        headers: schema.standard_columns.iter().map(|s| s.to_string()).collect(),
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
                        std_row[idx] =
                            if v.trim().is_empty() { String::new() } else { v.trim().to_string() };
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
                        let sheet_suffix =
                            if !src.name.is_empty() { format!("::{}", src.name) } else { String::new() };
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
    table.headers.extend(schema.extra_columns.iter().map(|s| s.to_string()));
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
    let t = v
        .replace([',', '￥', '¥', '$'], "");
    // 单位后缀 (如 "2.5kg" / "300元") — 剥离尾部非数值字符
    let trimmed: String = t
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-' || *c == '+' || *c == 'e' || *c == 'E')
        .collect();
    trimmed.parse::<f64>().ok()
}

/// 价格表合并 (D4) — 薄封装: 委托通用引擎 + 价格表 schema。
/// 保持向后兼容签名; 领域知识已外置为 PRICE_TABLE_SCHEMA。
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

/// 首个 sheet 专用入口 — 每个文件只取第一个 sheet 合并 (不优先"修改版")。
///
/// 与 [`consolidate_tables`] 的差异仅在 sheet 选择策略;
/// 委托运行时 `SheetMode::FirstSheet`, 无需编译期哨兵变体 (R-2026-08-20 收敛)。
///
/// 适用: 目录文件已人工核对待确认 (如 "已核对待确认-0810"), 首个 sheet 即目标数据。
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

/// 建议 schema 草稿 (P3 选项 B: LLM 生成初稿 → 人工确认固化, 不直入生产)。
/// 确定性部分: 扫描目录收集所有表头 → 与 PRICE_TABLE_SCHEMA 变体表匹配, 标注命中/未命中。
/// 增强部分: 未命中列由 LLM 建议归类 (可选; LLM 不可用时纯确定性降级)。
/// 产出 JSON 草稿, 必须经人工确认后才固化为 MergeSchema const (Validator gate 不 PASS 不呈现)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaSuggestion {
    /// 扫描到的全部原始表头 (去重, 排序)
    pub observed_headers: Vec<String>,
    /// 命中标准列的变体 (标准列 → 原始表头列表)
    pub matched: std::collections::BTreeMap<String, Vec<String>>,
    /// 未命中任何标准列/变体的表头 (需人工或 LLM 归类)
    pub unmatched: Vec<String>,
    /// 建议的列名变体新增项 (标准列 → 新增变体), 来自 LLM 增强 (可为空)
    pub suggested_variants: std::collections::BTreeMap<String, Vec<String>>,
    /// LLM 增强是否可用 (false = 纯确定性降级)
    pub llm_enhanced: bool,
}

impl SchemaSuggestion {
    /// 生成可固化为 JSON schema 的草稿 (owned) — 基于价格表 schema 克隆,
    /// 不含 LLM 未确认变体 (deterministic matched 已含在 base 变体),
    /// 保持生产 schema 纯净。
    pub fn draft(&self) -> MergeSchemaJson {
        MergeSchemaJson::from_price_table()
    }

    /// 显式纳入 LLM 建议变体的固化草稿 (suggest --save 用) —
    /// 仅在调用方明确确认后使用, 防止幻觉污染生产 schema。
    pub fn draft_with_variants(&self) -> MergeSchemaJson {
        let mut j = self.draft();
        for (std_col, variants) in &self.suggested_variants {
            // 追加到既有变体列表 (若变体已存在则跳过)
            if let Some(existing) = j
                .column_variants
                .iter_mut()
                .find(|(c, _)| *c == *std_col)
            {
                for v in variants {
                    if !existing.1.contains(v) {
                        existing.1.push(v.clone());
                    }
                }
            } else {
                j.column_variants
                    .push((std_col.clone(), variants.clone()));
            }
        }
        j
    }
}

/// 扫描目录收集建议 schema 初稿 (P3)。
///
/// 确定性阶段 (无 LLM 依赖, 可离线):
///   - 扫描 xlsx/csv/tsv 文件, 收集全部表头
///   - 与 PRICE_TABLE_SCHEMA.column_variants 匹配 → matched / unmatched
///
/// 增强阶段 (可选):
///   - 若提供 `llm` 回调, 对 unmatched 表头调用, 建议归类到标准列
///   - LLM 建议仅进 `suggested_variants` (草稿), 不自动落进生产 schema
///
/// 返回 `SchemaSuggestion`。调用方 (CLI / 意识核心) 负责展示 + 人工确认。
pub fn suggest_schema(
    src_dir: impl AsRef<Path>,
    llm: Option<&dyn Fn(&str) -> Option<String>>,
) -> Result<SchemaSuggestion> {
    let schema = &PRICE_TABLE_SCHEMA;
    let mut headers: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    if let Ok(read) = std::fs::read_dir(src_dir.as_ref()) {
        for e in read.flatten() {
            let p = e.path();
            let ext = p
                .extension()
                .and_then(|x| x.to_str())
                .map(|x| x.to_lowercase())
                .unwrap_or_default();
            if !matches!(ext.as_str(), "xlsx" | "csv" | "tsv") {
                continue;
            }
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
            // 取第一个 sheet / 首行表头
            let tables = if ext == "xlsx" {
                read_xlsx_sheets_all(&p).unwrap_or_default()
            } else if ext == "csv" {
                read_csv(&p).map(|t| vec![t]).unwrap_or_default()
            } else {
                continue;
            };
            if let Some(first) = tables.into_iter().next() {
                for h in first.headers {
                    headers.insert(h.trim().to_string());
                }
            }
        }
    }

    let mut matched: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    let mut unmatched: Vec<String> = Vec::new();
    for h in &headers {
        let normalized = schema.normalize_column(h);
        // normalize 未命中时返回原样 — 若原样在标准列集合中视为命中
        if schema.standard_columns.contains(&normalized.as_str()) {
            matched
                .entry(normalized)
                .or_default()
                .push(h.clone());
        } else {
            unmatched.push(h.clone());
        }
    }

    let mut suggested_variants: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    let mut llm_enhanced = false;
    if let Some(llm_fn) = llm {
        let mut prompt = String::from(
            "你是表头映射专家。以下表头未命中价格表标准列, 请归类到标准列之一: \n",
        );
        for (i, h) in unmatched.iter().enumerate() {
            prompt.push_str(&format!("{}. {}\n", i + 1, h));
        }
        prompt.push_str("标准列: ");
        prompt.push_str(&schema.standard_columns.join(" / "));
        prompt.push_str("\n仅输出 '原始表头 → 标准列' 一行一条, 无法归类则 '原始表头 → NULL'");
        if let Some(resp) = llm_fn(&prompt) {
            llm_enhanced = true;
            for line in resp.lines() {
                let Some((raw, target)) = line.split_once("→") else {
                    continue;
                };
                let raw = raw.trim();
                let target = target.trim();
                if target == "NULL" || target.is_empty() {
                    continue;
                }
                if !schema.standard_columns.contains(&target) {
                    continue; // LLM 建议的目标不是合法标准列 → 丢弃 (防幻觉)
                }
                suggested_variants
                    .entry(target.to_string())
                    .or_default()
                    .push(raw.to_string());
            }
        }
    }

    Ok(SchemaSuggestion {
        observed_headers: headers.into_iter().collect(),
        matched,
        unmatched,
        suggested_variants,
        llm_enhanced,
    })
}

/// 多 sheet 表格按 sheet_mode 选择 (运行时策略):
/// - FirstSheet: 只取第一个 sheet
/// - Preferred(list): 命中任一 (trim 精确匹配) 只保留该 sheet; 未命中取第一个
/// - AllSheets: 保留全部
fn select_preferred_sheets(tables: Vec<TableData>, mode: SheetMode) -> Vec<TableData> {
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
pub(super) fn derive_source_name(path: &Path, schema: &MergeSchema) -> String {
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
    s.trim_matches(|c: char| c == '_' || c == '-' || c == '、' || c == '.' || c == ' ').to_string()
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

// ─── Schema 数据化 (R-2026-08-20) ────────────────────────────────────────────
// MergeSchema 是编译期 const (`&'static str`), 引擎零领域知识。
// 领域 schema 数据化为 JSON 文件, 运行时经 SchemaStore 加载 → `Box::leak` 转
// static → MergeSchema。新增领域 (如"中央产品信息库") 只需写 JSON, 零重编译。

/// Schema 的 JSON 表示 (owned, 可 serde) — 与 MergeSchema 字段一一对应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeSchemaJson {
    pub name: String,
    pub standard_columns: Vec<String>,
    pub column_variants: Vec<(String, Vec<String>)>,
    pub filename_suffixes: Vec<String>,
    /// 补单位规则: (列, 后缀, [skip_if_contains])
    #[serde(default)]
    pub unit_rules: Vec<(String, String, Vec<String>)>,
    #[serde(default)]
    pub preferred_sheets: Vec<String>,
    /// 空值标记: (列, [标记])
    #[serde(default)]
    pub empty_markers: Vec<(String, Vec<String>)>,
    pub value_columns: Vec<String>,
    #[serde(default)]
    pub extra_columns: Vec<String>,
    #[serde(default)]
    pub skip_prefixes: Vec<String>,
    #[serde(default)]
    pub supplier_column: Option<String>,
    #[serde(default)]
    pub dedup_columns: Vec<String>,
    /// 数字列: ["列名"...] (仅 Numeric 需列出, Text 为默认)
    #[serde(default)]
    pub numeric_columns: Vec<String>,
}

impl MergeSchemaJson {
    /// 从 PRICE_TABLE_SCHEMA 导出 (作为默认 schema 文件的事实源)
    pub fn from_price_table() -> Self {
        Self {
            name: PRICE_TABLE_SCHEMA.name.to_string(),
            standard_columns: PRICE_TABLE_SCHEMA
                .standard_columns
                .iter()
                .map(|s| s.to_string())
                .collect(),
            column_variants: PRICE_TABLE_SCHEMA
                .column_variants
                .iter()
                .map(|(std, vs)| (std.to_string(), vs.iter().map(|v| v.to_string()).collect()))
                .collect(),
            filename_suffixes: PRICE_TABLE_SCHEMA
                .filename_suffixes
                .iter()
                .map(|s| s.to_string())
                .collect(),
            unit_rules: PRICE_TABLE_SCHEMA
                .unit_rules
                .iter()
                .map(|r| {
                    (
                        r.column.to_string(),
                        r.suffix.to_string(),
                        r.skip_if_contains.iter().map(|s| s.to_string()).collect(),
                    )
                })
                .collect(),
            preferred_sheets: PRICE_TABLE_SCHEMA
                .preferred_sheets
                .iter()
                .map(|s| s.to_string())
                .collect(),
            empty_markers: PRICE_TABLE_SCHEMA
                .empty_markers
                .iter()
                .map(|(c, ms)| (c.to_string(), ms.iter().map(|m| m.to_string()).collect()))
                .collect(),
            value_columns: PRICE_TABLE_SCHEMA
                .value_columns
                .iter()
                .map(|s| s.to_string())
                .collect(),
            extra_columns: PRICE_TABLE_SCHEMA
                .extra_columns
                .iter()
                .map(|s| s.to_string())
                .collect(),
            skip_prefixes: PRICE_TABLE_SCHEMA
                .skip_prefixes
                .iter()
                .map(|s| s.to_string())
                .collect(),
            supplier_column: PRICE_TABLE_SCHEMA.supplier_column.map(|s| s.to_string()),
            dedup_columns: PRICE_TABLE_SCHEMA
                .dedup_columns
                .iter()
                .map(|s| s.to_string())
                .collect(),
            numeric_columns: PRICE_TABLE_SCHEMA
                .column_types
                .iter()
                .filter(|(_, t)| *t == ColumnType::Numeric)
                .map(|(c, _)| c.to_string())
                .collect(),
        }
    }

    /// 从任意 MergeSchema 序列化为 JSON (可重命名 name) — suggest → 固化闭环。
    pub fn from_schema(schema: &MergeSchema, name: &str) -> Self {
        Self {
            name: name.to_string(),
            standard_columns: schema
                .standard_columns
                .iter()
                .map(|s| s.to_string())
                .collect(),
            column_variants: schema
                .column_variants
                .iter()
                .map(|(std, vs)| (std.to_string(), vs.iter().map(|v| v.to_string()).collect()))
                .collect(),
            filename_suffixes: schema
                .filename_suffixes
                .iter()
                .map(|s| s.to_string())
                .collect(),
            unit_rules: schema
                .unit_rules
                .iter()
                .map(|r| {
                    (
                        r.column.to_string(),
                        r.suffix.to_string(),
                        r.skip_if_contains.iter().map(|s| s.to_string()).collect(),
                    )
                })
                .collect(),
            preferred_sheets: schema
                .preferred_sheets
                .iter()
                .map(|s| s.to_string())
                .collect(),
            empty_markers: schema
                .empty_markers
                .iter()
                .map(|(c, ms)| (c.to_string(), ms.iter().map(|m| m.to_string()).collect()))
                .collect(),
            value_columns: schema
                .value_columns
                .iter()
                .map(|s| s.to_string())
                .collect(),
            extra_columns: schema
                .extra_columns
                .iter()
                .map(|s| s.to_string())
                .collect(),
            skip_prefixes: schema
                .skip_prefixes
                .iter()
                .map(|s| s.to_string())
                .collect(),
            supplier_column: schema.supplier_column.map(|s| s.to_string()),
            dedup_columns: schema
                .dedup_columns
                .iter()
                .map(|s| s.to_string())
                .collect(),
            numeric_columns: schema
                .column_types
                .iter()
                .filter(|(_, t)| *t == ColumnType::Numeric)
                .map(|(c, _)| c.to_string())
                .collect(),
        }
    }

    /// 转换为编译期 MergeSchema (Box::leak 保 static; 进程级一次性加载可接受)。
    pub fn to_schema(&self) -> MergeSchema {
        fn leak(s: &str) -> &'static str {
            Box::leak(s.to_string().into_boxed_str())
        }
        fn leak_list(v: &[String]) -> &'static [&'static str] {
            Box::leak(v.iter().map(|s| leak(s)).collect::<Vec<_>>().into_boxed_slice())
        }
        fn leak_variants(v: &[(String, Vec<String>)]) -> &'static [(&'static str, &'static [&'static str])] {
            Box::leak(
                v.iter()
                    .map(|(std, vs)| (leak(std), leak_list(vs)))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            )
        }
        fn leak_markers(v: &[(String, Vec<String>)]) -> &'static [(&'static str, &'static [&'static str])] {
            leak_variants(v)
        }
        fn leak_units(v: &[(String, String, Vec<String>)]) -> &'static [UnitRule] {
            Box::leak(
                v.iter()
                    .map(|(c, s, skip)| UnitRule {
                        column: leak(c),
                        suffix: leak(s),
                        skip_if_contains: leak_list(skip),
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            )
        }
        MergeSchema {
            name: leak(&self.name),
            standard_columns: leak_list(&self.standard_columns),
            column_variants: leak_variants(&self.column_variants),
            filename_suffixes: leak_list(&self.filename_suffixes),
            unit_rules: leak_units(&self.unit_rules),
            preferred_sheets: leak_list(&self.preferred_sheets),
            empty_markers: leak_markers(&self.empty_markers),
            value_columns: leak_list(&self.value_columns),
            extra_columns: leak_list(&self.extra_columns),
            skip_prefixes: leak_list(&self.skip_prefixes),
            supplier_column: self.supplier_column.as_deref().map(leak),
            dedup_columns: leak_list(&self.dedup_columns),
            column_types: Box::leak(
                self.numeric_columns
                    .iter()
                    .map(|c| (leak(c), ColumnType::Numeric))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
        }
    }

    /// schema.validate() 等价校验 (JSON 加载后执行, 防坏 schema 进入引擎)
    pub fn validate_json(&self) -> std::result::Result<(), String> {
        let mut seen = std::collections::HashSet::new();
        for c in &self.standard_columns {
            if !seen.insert(c.clone()) {
                return Err(format!("standard_columns 重复: '{c}'"));
            }
        }
        for v in &self.value_columns {
            if !self.standard_columns.contains(v) {
                return Err(format!("value_column '{v}' 不在 standard_columns"));
            }
        }
        for e in &self.extra_columns {
            if self.standard_columns.contains(e) {
                return Err(format!("extra_column '{e}' 与 standard_columns 冲突"));
            }
        }
        if let Some(s) = &self.supplier_column {
            if !self.standard_columns.contains(s) {
                return Err(format!("supplier_column '{s}' 不在 standard_columns"));
            }
        }
        for d in &self.dedup_columns {
            if !self.standard_columns.contains(d) {
                return Err(format!("dedup_column '{d}' 不在 standard_columns"));
            }
        }
        for c in &self.numeric_columns {
            if !self.standard_columns.contains(c) {
                return Err(format!("numeric_column '{c}' 不在 standard_columns"));
            }
        }
        Ok(())
    }
}

/// Schema 注册表 — 从 JSON 文件加载领域 schema (L3 知识外置)。
/// 加载后缓存, 同一 schema 进程内只 leak 一次。
pub struct SchemaStore {
    dir: std::path::PathBuf,
    cache: std::collections::HashMap<String, MergeSchema>,
}

impl Default for SchemaStore {
    fn default() -> Self {
        Self::new()
    }
}

impl SchemaStore {
    /// 默认目录: $NEOTRIX_SCHEMA_DIR 或 ~/.neotrix/schemas
    pub fn new() -> Self {
        let dir = std::env::var_os("NEOTRIX_SCHEMA_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .unwrap_or_else(|| std::path::PathBuf::from("."))
                    .join(".neotrix")
                    .join("schemas")
            });
        Self {
            dir,
            cache: std::collections::HashMap::new(),
        }
    }

    pub fn with_dir(dir: impl AsRef<Path>) -> Self {
        Self {
            dir: dir.as_ref().to_path_buf(),
            cache: std::collections::HashMap::new(),
        }
    }

    /// 当前 schema 目录 (供 CLI 展示/落盘定位)
    pub fn dir(&self) -> &std::path::Path {
        &self.dir
    }

    /// 加载 schema by name (name.json)。
    /// 内置 "price_table" 回退到编译期常量 (零文件依赖)。
    pub fn load(&mut self, name: &str) -> Result<MergeSchema> {
        if let Some(s) = self.cache.get(name) {
            return Ok(*s);
        }
        let schema = if name == "price_table" {
            PRICE_TABLE_SCHEMA
        } else {
            let path = self.dir.join(format!("{name}.json"));
            let text = std::fs::read_to_string(&path).map_err(|e| {
                FileAbilityError::Parse(format!(
                    "schema '{name}' 加载失败 ({}): {e}",
                    path.display()
                ))
            })?;
            let json: MergeSchemaJson =
                serde_json::from_str(&text).map_err(|e| FileAbilityError::Parse(e.to_string()))?;
            json.validate_json().map_err(FileAbilityError::Parse)?;
            json.to_schema()
        };
        self.cache.insert(name.to_string(), schema);
        Ok(schema)
    }

    /// 列出已注册的 schema 文件名 (不含 .json)
    pub fn list(&self) -> Vec<String> {
        let mut out = vec!["price_table".to_string()];
        if let Ok(rd) = std::fs::read_dir(&self.dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.extension().and_then(|x| x.to_str()) == Some("json") {
                    if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                        out.push(stem.to_string());
                    }
                }
            }
        }
        out.sort();
        out.dedup();
        out
    }
}

// ─── CollectionMerge 统一入口 (范式归一, R-P42) ────────────────────────────
// 设计见 docs/3-AUDITS/collection-merge-design-2026-08-20.md。
// 一个入口按输入格式分派到既有闭环: pdf → merge_pdfs, xlsx/csv/tsv →
// merge_tables_with_mode, docx → merge_docx, pptx → merge_pptx;
// 混合格式 → L0 文本级聚合 (extract_text 拼接)。

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
    ["修改版", "已更新", "已完善", "更新版", "最新版", "final", "modified", "updated"]
        .iter()
        .any(|kw| name.contains(kw))
}

fn dispatch_collection_merge(
    req: &CollectionMergeRequest,
    exts: &[String],
) -> Result<MergeOutcome> {
    let all = |e: &str| exts.iter().all(|x| x == e);
    if all("pdf") {
        let bytes = super::helpers::merge_pdfs(&req.inputs)?;
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
    if exts.iter().all(|x| matches!(x.as_str(), "xlsx" | "csv" | "tsv")) {
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
        let report =
            merge_tables_with_mode(&schema, &req.inputs[0].parent().unwrap_or(std::path::Path::new(".")), &req.output, mode)
                .map_err(|e| FileAbilityError::Other(format!("表格合并失败: {e}")))?;
        return Ok(MergeOutcome::Xlsx {
            rows: report.total_rows,
            note: format!("处理 {} 文件, 失败 {}", report.files_processed, report.files_failed.len()),
        });
    }
    // 混合格式 → L0 文本级聚合
    let mut chunks: Vec<String> = Vec::new();
    let mut failed = 0usize;
    for p in &req.inputs {
        match super::helpers::extract_text(p) {
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

#[cfg(test)]
mod schema_tests {
    use super::*;

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
        let dir = std::env::temp_dir().join(format!(
            "nt_schema_store_fail_{}",
            std::process::id()
        ));
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
            headers: vec![
                "产品型号".to_string(),
                "含税单价(元)".to_string(),
            ],
            rows: vec![vec![
                "闸阀\"双引号\"".to_string(),
                "100".to_string(),
            ]],
        };
        write_xlsx_table(dir.join("1_华东_价格.xlsx"), &t).unwrap();
        let out = dir.join("native_consolidated.csv");
        let rep = merge_tables_with_mode(
            &PRICE_TABLE_SCHEMA,
            &dir,
            &out,
            SheetMode::FirstSheet,
        )
        .unwrap();
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
        let rep = merge_tables_with_mode(
            &PRICE_TABLE_SCHEMA,
            &dir,
            &out,
            SheetMode::FirstSheet,
        )
        .unwrap();
        assert_eq!(rep.files_processed, 2, "xlsx + csv 都应处理");
        assert_eq!(rep.total_rows, 2, "各 1 行 → 2 行");
        let back = read_csv(&out).unwrap();
        assert_eq!(back.row_count(), 2);
        let model = back
            .headers
            .iter()
            .position(|h| h == "产品型号")
            .unwrap();
        let mut models: Vec<&str> = back.rows.iter().map(|r| r[model].as_str()).collect();
        models.sort();
        assert_eq!(models, vec!["蝶阀Y", "闸阀X"], "两个源的数据都应进入");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_collection_merge_dispatch_routes() {
        // 分发: 全部 pdf → merge_pdfs; 全部 docx → merge_docx; 混合 → L0 文本
        use std::io::Write;
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
            doc.objects.insert(pages_id, lopdf::Object::Dictionary(pages));
            let catalog = doc.add_object(lopdf::dictionary! {
                "Type" => "Catalog", "Pages" => pages_id,
            });
            doc.trailer.set("Root", catalog);
            doc.compress();
            let mut buf = Vec::new();
            doc.save_to(&mut buf).expect("save");
            buf
        }

        let tmp = std::env::temp_dir().join(format!(
            "nt_colmerge_{}",
            std::process::id()
        ));
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
        std::fs::write(&d1, crate::neotrix::nt_file_ability::make_min_docx("A")).unwrap();
        std::fs::write(&d2, crate::neotrix::nt_file_ability::make_min_docx("B")).unwrap();
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
        let req = CollectionMergeRequest {
            inputs: vec![p1, tmp.join("c.docx")],
            schema: None,
            strategy: MergeStrategy::All,
            output: tmp.join("merged.txt"),
            dry_run: false,
        };
        // 混合输入中 docx 用 make_min_docx 写入
        std::fs::write(&req.inputs[1], crate::neotrix::nt_file_ability::make_min_docx("C")).unwrap();
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
        std::fs::write(&doc_modified, crate::neotrix::nt_file_ability::make_min_docx("Modified")).unwrap();
        std::fs::write(&doc_normal, crate::neotrix::nt_file_ability::make_min_docx("Normal")).unwrap();
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
        let merged_text = std::fs::read_to_string(&tmp.join("preferred.docx")).unwrap();
        assert!(merged_text.find("Modified").unwrap() < merged_text.find("Normal").unwrap(),
            "Preferred 策略应把修改版作为基座 (内容在前)");

        // 6. dry_run 预览模式: 不写文件, 仅返回预览信息
        let req = CollectionMergeRequest {
            inputs: vec![doc_normal, doc_modified],
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
        assert!(!tmp.join("dry_run.txt").exists(), "dry_run 不应生成输出文件");

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
            inputs: vec![p1],
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