//! 多文件合并 (D4): 通用引擎 + 领域 Schema 分离。
//!
//! 分层架构 (贯穿整个文件能力体系):
//!   L1 格式编解码 (通用): read_xlsx_table/write_csv/detect_encoding — 任何类型文件
//!   L2 表格语义 (通用):   merge_tables_with(schema) — 零领域知识的多表合并引擎
//!   L3 领域 schema (差异化): PRICE_TABLE_SCHEMA + skills md 镜像 — 唯一个性化层
//!   L4 意图层 (通用):     意识核心 xlsx_consolidation → 选 schema → 调 merge_tables_with
//!
//! 领域知识 (列名变体/标准列序/单位规则/供应商命名/跳过前缀) 全部数据化进 MergeSchema,
//! 不再编译进引擎函数。换行业 = 新增一个 schema const, 不改引擎代码。

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::tables::{read_csv, read_xlsx_sheets_all, write_xlsx_table};
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
                Ok(tables) => select_preferred_sheets(tables, schema.preferred_sheets),
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

    write_xlsx_table(output, &table)?;
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
    merge_tables_with(&PRICE_TABLE_SCHEMA, src_dir, output)
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
    /// 生成可固化为 MergeSchema 的草稿 — 仅输出确定性可验证部分,
    /// 未确认的变体一律不进 (防止幻觉污染生产 schema)。
    pub fn draft(&self) -> MergeSchema {
        PRICE_TABLE_SCHEMA
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

/// 多 sheet 表格按 preferred_sheets 选择: 命中任一 (trim 精确匹配) 只保留该 sheet;
/// 未命中则取第一个 sheet。preferred_sheets 为空 = 保留全部 (旧行为)。
fn select_preferred_sheets(tables: Vec<TableData>, preferred: &[&str]) -> Vec<TableData> {
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