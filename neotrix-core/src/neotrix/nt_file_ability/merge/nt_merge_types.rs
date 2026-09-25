//! 合并类型层: UnitRule / ColumnType / SheetMode / MergeSchema + 价格表 Schema 常量 + 报告类型。
//!
//! 纯类型定义, 零引擎逻辑。由 `merge.rs` 门面重导出, 外部路径不变。

use serde::{Deserialize, Serialize};

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
            &[
                "含税单价",
                "单价(元)",
                "单价",
                "价格(元)",
                "单价(含税)",
                "含税价(元)",
            ],
        ),
        (
            "美元报价(USD)",
            &[
                "美元价(USD)",
                "美元价",
                "美元报价",
                "USD报价",
                "单价(美元)",
                "美元单价",
            ],
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
        "价格_报价",
        "价格表",
        "报价模板",
        "_报价",
        "-报价",
        "价格表_报价",
        "已完善",
        "-已更新",
        "-修改版",
        "-中高档",
        "-中低档",
        "-第一版",
        "-第二版",
        "-第五版本",
        "-含税",
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
