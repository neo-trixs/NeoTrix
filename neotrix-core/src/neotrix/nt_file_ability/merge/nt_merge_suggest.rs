//! 联想/校验层: SchemaSuggestion + suggest_schema 扫描。
//!
//! 确定性表头匹配 + 可选 LLM 增强, 产出可固化草稿。生产 schema 保持纯净。
//! 由 `merge.rs` 门面重导出, 外部路径不变。

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::super::excel::tables::{read_csv, read_xlsx_sheets_all};
use super::nt_merge_schema_store::MergeSchemaJson;
use super::nt_merge_types::PRICE_TABLE_SCHEMA;
use crate::neotrix::nt_file_ability::types::{Result, TableData};

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
            if let Some(existing) = j.column_variants.iter_mut().find(|(c, _)| *c == *std_col) {
                for v in variants {
                    if !existing.1.contains(v) {
                        existing.1.push(v.clone());
                    }
                }
            } else {
                j.column_variants.push((std_col.clone(), variants.clone()));
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
            let tables: Vec<TableData> = if ext == "xlsx" {
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
            matched.entry(normalized).or_default().push(h.clone());
        } else {
            unmatched.push(h.clone());
        }
    }

    let mut suggested_variants: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    let mut llm_enhanced = false;
    if let Some(llm_fn) = llm {
        let mut prompt =
            String::from("你是表头映射专家。以下表头未命中价格表标准列, 请归类到标准列之一: \n");
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
