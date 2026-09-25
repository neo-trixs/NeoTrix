//! Schema 数据化层 (R-2026-08-20): MergeSchemaJson + SchemaStore。
//!
//! MergeSchema 是编译期 const (`&'static str`), 引擎零领域知识。
//! 领域 schema 数据化为 JSON 文件, 运行时经 SchemaStore 加载 → `Box::leak` 转
//! static → MergeSchema。新增领域 (如"中央产品信息库") 只需写 JSON, 零重编译。
//! 由 `merge.rs` 门面重导出, 外部路径不变。

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::nt_merge_types::{ColumnType, MergeSchema, UnitRule, PRICE_TABLE_SCHEMA};
use crate::neotrix::nt_file_ability::types::{FileAbilityError, Result};

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
            value_columns: schema.value_columns.iter().map(|s| s.to_string()).collect(),
            extra_columns: schema.extra_columns.iter().map(|s| s.to_string()).collect(),
            skip_prefixes: schema.skip_prefixes.iter().map(|s| s.to_string()).collect(),
            supplier_column: schema.supplier_column.map(|s| s.to_string()),
            dedup_columns: schema.dedup_columns.iter().map(|s| s.to_string()).collect(),
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
            Box::leak(
                v.iter()
                    .map(|s| leak(s))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            )
        }
        fn leak_variants(
            v: &[(String, Vec<String>)],
        ) -> &'static [(&'static str, &'static [&'static str])] {
            Box::leak(
                v.iter()
                    .map(|(std, vs)| (leak(std), leak_list(vs)))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            )
        }
        fn leak_markers(
            v: &[(String, Vec<String>)],
        ) -> &'static [(&'static str, &'static [&'static str])] {
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
