#![forbid(unsafe_code)]

//! 蒸馏管线 — 提取 kb-analyze.sh Distiller + Meta-Cognition + migrate_*.py 本质模式
//!
//! 核心抽象:
//! - `ContentDistiller`: 仓库分析/概念分析/交叉引用 (kb-analyze.sh distiller_* 本质)
//! - `MetaCognitiveAnalyzer`: 错误模式检测 + 模式记录 (kb-analyze.sh meta_error_analysis 本质)
//! - `DataMigrator`: 通用 JSON/CSV → KB 迁移管线 (migrate_*.py 本质)
//! - `DistillationStage`: L0 原始 → L1 清洗 → L2 结构化 → L3 蒸馏层级
//!
//! 设计: 纯数据结构 + 变换逻辑; DB 访问通过 `rusqlite::Connection` 参数注入。

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════
// Content Distiller — kb-analyze.sh distiller_* 本质
// ═══════════════════════════════════════════════════════════════════

/// 仓库分析报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoAnalysis {
    pub total_repos: usize,
    pub total_stars: usize,
    pub avg_stars: usize,
    pub language_distribution: Vec<(String, usize)>,
    pub topic_distribution: Vec<(String, usize)>,
    pub dominant_language: Option<(String, usize)>,
}

/// 概念分析报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptAnalysis {
    pub total_concepts: usize,
    pub total_content_length: usize,
    pub avg_content_length: usize,
    pub domain_distribution: Vec<(String, usize)>,
}

/// 交叉引用分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossReferenceAnalysis {
    /// 出现 3+ 次的主题 (跨仓库引用)
    pub cross_topics: Vec<(String, usize)>,
    pub total_cross_topics: usize,
}

/// 内容蒸馏器: 从 KB 节点聚合分析
pub struct ContentDistiller;

impl ContentDistiller {
    /// 分析 Repository 类型节点 (语言/主题/星数分布)
    pub fn analyze_repos(conn: &Connection) -> Result<RepoAnalysis, String> {
        let mut langs: HashMap<String, usize> = HashMap::new();
        let mut topics: HashMap<String, usize> = HashMap::new();
        let mut total_stars = 0usize;
        let mut total_repos = 0usize;

        let mut stmt = conn
            .prepare(
                "SELECT COALESCE(metadata, '{}') FROM nodes WHERE node_type = 'Repository'",
            )
            .map_err(|e| format!("Prepare: {}", e))?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| format!("Query: {}", e))?;

        for r in rows {
            let meta_str = r.unwrap_or_default();
            if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&meta_str) {
                total_repos += 1;
                if let Some(stars) = meta.get("stars").and_then(|v| v.as_f64()) {
                    total_stars += stars as usize;
                }
                let lang = meta
                    .get("language")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                *langs.entry(lang.to_string()).or_insert(0) += 1;
                if let Some(topics_arr) = meta.get("topics").and_then(|v| v.as_array()) {
                    for t in topics_arr {
                        if let Some(t_str) = t.as_str() {
                            *topics.entry(t_str.to_string()).or_insert(0) += 1;
                        }
                    }
                }
            }
        }

        let avg_stars = if total_repos > 0 {
            total_stars / total_repos
        } else {
            0
        };
        let mut lang_vec: Vec<(String, usize)> = langs.into_iter().collect();
        lang_vec.sort_by(|a, b| b.1.cmp(&a.1));
        let mut topic_vec: Vec<(String, usize)> = topics.into_iter().collect();
        topic_vec.sort_by(|a, b| b.1.cmp(&a.1));
        let dominant_language = lang_vec.first().cloned();

        Ok(RepoAnalysis {
            total_repos,
            total_stars,
            avg_stars,
            language_distribution: lang_vec,
            topic_distribution: topic_vec,
            dominant_language,
        })
    }

    /// 分析 Concept 类型节点 (域分布/内容量)
    pub fn analyze_concepts(conn: &Connection) -> Result<ConceptAnalysis, String> {
        let mut domains: HashMap<String, usize> = HashMap::new();
        let mut total_content = 0usize;
        let mut total = 0usize;

        let mut stmt = conn
            .prepare(
                "SELECT COALESCE(domain, 'unknown'), LENGTH(COALESCE(content, '')) \
                 FROM nodes WHERE node_type = 'Concept'",
            )
            .map_err(|e| format!("Prepare: {}", e))?;
        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0).unwrap_or_default(), row.get::<_, usize>(1).unwrap_or(0)))
            })
            .map_err(|e| format!("Query: {}", e))?;

        for r in rows {
            if let Ok((domain, clen)) = r {
                total += 1;
                total_content += clen;
                *domains.entry(domain).or_insert(0) += 1;
            }
        }

        let avg = if total > 0 { total_content / total } else { 0 };
        let mut domain_vec: Vec<(String, usize)> = domains.into_iter().collect();
        domain_vec.sort_by(|a, b| b.1.cmp(&a.1));

        Ok(ConceptAnalysis {
            total_concepts: total,
            total_content_length: total_content,
            avg_content_length: avg,
            domain_distribution: domain_vec,
        })
    }

    /// 交叉引用分析: 出现 3+ 次的主题
    pub fn cross_references(conn: &Connection) -> Result<CrossReferenceAnalysis, String> {
        let mut topic_repos: HashMap<String, usize> = HashMap::new();

        let mut stmt = conn
            .prepare("SELECT COALESCE(metadata, '{}') FROM nodes WHERE node_type = 'Repository'")
            .map_err(|e| format!("Prepare: {}", e))?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| format!("Query: {}", e))?;

        for r in rows {
            let meta_str = r.unwrap_or_default();
            if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&meta_str) {
                if let Some(topics_arr) = meta.get("topics").and_then(|v| v.as_array()) {
                    for t in topics_arr {
                        if let Some(t_str) = t.as_str() {
                            *topic_repos.entry(t_str.to_string()).or_insert(0) += 1;
                        }
                    }
                }
            }
        }

        let mut cross: Vec<(String, usize)> = topic_repos
            .into_iter()
            .filter(|(_, c)| *c >= 3)
            .collect();
        cross.sort_by(|a, b| b.1.cmp(&a.1));
        let total = cross.len();

        Ok(CrossReferenceAnalysis {
            cross_topics: cross,
            total_cross_topics: total,
        })
    }

    /// 生成完整蒸馏报告并持久化到 kv_store
    pub fn distill_and_persist(conn: &Connection) -> Result<DistillationReport, String> {
        let repos = Self::analyze_repos(conn)?;
        let concepts = Self::analyze_concepts(conn)?;
        let cross_refs = Self::cross_references(conn)?;

        let report = DistillationReport {
            repos,
            concepts,
            cross_refs,
            generated_at: now_secs(),
        };

        // 持久化
        let json =
            serde_json::to_string(&report).map_err(|e| format!("Serialize: {}", e))?;
        conn.execute(
            "INSERT OR REPLACE INTO kv_store (namespace, key, value, updated_at) \
             VALUES ('distiller', 'latest', ?1, ?2)",
            rusqlite::params![json, report.generated_at],
        )
        .map_err(|e| format!("Persist: {}", e))?;

        Ok(report)
    }
}

/// 完整蒸馏报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistillationReport {
    pub repos: RepoAnalysis,
    pub concepts: ConceptAnalysis,
    pub cross_refs: CrossReferenceAnalysis,
    pub generated_at: i64,
}

// ═══════════════════════════════════════════════════════════════════
// Meta-Cognitive Analyzer — kb-analyze.sh meta_error_analysis 本质
// ═══════════════════════════════════════════════════════════════════

/// 元认知错误分析报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaCognitiveReport {
    pub failed_crawl: usize,
    pub ingest_errors: usize,
    pub pending_crawl: usize,
    pub recurring_error_recorded: bool,
    pub recommendations: Vec<String>,
    pub analyzed_at: i64,
}

impl MetaCognitiveReport {
    /// 分析 KB 错误模式
    pub fn analyze(conn: &Connection) -> Self {
        let now = now_secs();
        let failed_crawl: usize = conn
            .query_row(
                "SELECT COUNT(*) FROM crawl_queue WHERE status = 'failed'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let ingest_errors: usize = conn
            .query_row(
                "SELECT COUNT(*) FROM ingest_log WHERE status = 'error' OR status = 'empty'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let pending_crawl: usize = conn
            .query_row(
                "SELECT COUNT(*) FROM crawl_queue WHERE status = 'pending'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let mut recommendations = Vec::new();
        if failed_crawl > 0 {
            recommendations.push(format!("Review {} failed crawl entries", failed_crawl));
        }
        if pending_crawl > 0 {
            recommendations.push(format!("{} entries still pending in crawl queue", pending_crawl));
        }
        if ingest_errors > 0 {
            recommendations.push(format!("Investigate {} ingestion errors", ingest_errors));
        }

        // 高失败率时记录 RecurringError 模式
        let recurring_error_recorded = if failed_crawl > 100 || ingest_errors > 10 {
            let _desc = format!(
                "KB absorption: {} failed crawl, {} ingestion errors",
                failed_crawl, ingest_errors
            );
            let _ = conn.execute(
                "INSERT OR IGNORE INTO kv_store (namespace, key, value, updated_at) \
                 VALUES ('meta_cognition', 'recurring_error', ?1, ?2)",
                rusqlite::params![
                    serde_json::json!({
                        "failed_crawl": failed_crawl,
                        "ingest_errors": ingest_errors,
                        "ts": now,
                    })
                    .to_string(),
                    now
                ],
            );
            true
        } else {
            false
        };

        // 持久化
        let report = Self {
            failed_crawl,
            ingest_errors,
            pending_crawl,
            recurring_error_recorded,
            recommendations,
            analyzed_at: now,
        };
        if let Ok(json) = serde_json::to_string(&report) {
            let _ = conn.execute(
                "INSERT OR REPLACE INTO kv_store (namespace, key, value, updated_at) \
                 VALUES ('meta_cognition', 'error_analysis', ?1, ?2)",
                rusqlite::params![json, now],
            );
        }

        report
    }
}

// ═══════════════════════════════════════════════════════════════════
// Data Migrator — migrate_*.py 本质
// ═══════════════════════════════════════════════════════════════════

/// 迁移记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationReport {
    pub source: String,
    pub records_migrated: usize,
    pub records_skipped: usize,
    pub errors: Vec<String>,
    pub duration_ms: u64,
    pub migrated_at: i64,
}

/// 通用数据迁移器: JSON → KB nodes
pub struct DataMigrator;

impl DataMigrator {
    /// 从 JSON 文件迁移到 KB nodes
    ///
    /// `records`: JSON 对象数组, 每个对象至少包含 `title` 和 `node_type` 字段。
    /// `type_field`: 指定 node_type 的 JSON 字段名 (默认 "node_type")
    /// `title_field`: 指定 title 的 JSON 字段名 (默认 "title")
    pub fn migrate_json(
        conn: &Connection,
        records: &[serde_json::Value],
        source: &str,
        type_field: &str,
        title_field: &str,
    ) -> MigrationReport {
        let t0 = std::time::Instant::now();
        let mut migrated = 0usize;
        let mut skipped = 0usize;
        let mut errors = Vec::new();

        for record in records {
            let title = match record.get(title_field).and_then(|v| v.as_str()) {
                Some(t) => t.to_string(),
                None => {
                    skipped += 1;
                    continue;
                }
            };
            let node_type = match record.get(type_field).and_then(|v| v.as_str()) {
                Some(t) => t.to_string(),
                None => "Concept".to_string(),
            };
            let content = record
                .get("content")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let url = record
                .get("url")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let domain = record
                .get("domain")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            // 插入或复用节点
            let node_id = match super::nt_memory_store::insert_or_get_node(
                conn,
                &title,
                neotrix_types::knowledge_access::NodeType::from_str(&node_type),
                content.as_deref(),
                url.as_deref(),
                domain.as_deref(),
            ) {
                Ok(id) => id,
                Err(e) => {
                    errors.push(format!("{}: {}", title, e));
                    continue;
                }
            };

            // 写入 metadata (整个 JSON 对象)
            if let Ok(node) = super::nt_memory_store::get_node(conn, &node_id) {
                if let Some(mut node) = node {
                    node.metadata = Some(record.clone());
                    if let Err(e) = super::nt_memory_store::update_node(conn, &node) {
                        errors.push(format!("metadata {}: {}", title, e));
                    }
                }
            }

            migrated += 1;
        }

        let duration_ms = t0.elapsed().as_millis() as u64;
        let migrated_at = now_secs();

        // 持久化迁移记录
        let report = MigrationReport {
            source: source.to_string(),
            records_migrated: migrated,
            records_skipped: skipped,
            errors,
            duration_ms,
            migrated_at,
        };

        if let Ok(json) = serde_json::to_string(&report) {
            let _ = conn.execute(
                "INSERT OR REPLACE INTO kv_store (namespace, key, value, updated_at) \
                 VALUES ('migration', ?1, ?2, ?3)",
                rusqlite::params![source, json, migrated_at],
            );
        }

        report
    }

    /// 从 JSONL (每行一个 JSON) 文件迁移
    pub fn migrate_jsonl(
        conn: &Connection,
        jsonl_content: &str,
        source: &str,
    ) -> MigrationReport {
        let records: Vec<serde_json::Value> = jsonl_content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect();
        Self::migrate_json(conn, &records, source, "node_type", "title")
    }
}

// ═══════════════════════════════════════════════════════════════════
// Distillation Stage — 层级蒸馏
// ═══════════════════════════════════════════════════════════════════

/// 蒸馏层级 (L0→L3)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DistillationLevel {
    /// L0: 原始数据 (未处理)
    Raw,
    /// L1: 清洗后 (去噪/归一化)
    Cleaned,
    /// L2: 结构化 (提取实体/关系)
    Structured,
    /// L3: 蒸馏后 (洞察/模式)
    Distilled,
}

impl DistillationLevel {
    /// 从字符串解析
    pub fn from_str(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "raw" => Self::Raw,
            "cleaned" | "clean" => Self::Cleaned,
            "structured" | "struct" => Self::Structured,
            "distilled" | "distill" => Self::Distilled,
            _ => Self::Raw,
        }
    }
}

/// 节点蒸馏状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDistillationState {
    pub node_id: String,
    pub current_level: DistillationLevel,
    pub last_distilled_at: Option<i64>,
    pub distillation_errors: Vec<String>,
}

// ═══════════════════════════════════════════════════════════════════
// 辅助
// ═══════════════════════════════════════════════════════════════════

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distillation_level_parse() {
        assert_eq!(DistillationLevel::from_str("raw"), DistillationLevel::Raw);
        assert_eq!(DistillationLevel::from_str("CLEANED"), DistillationLevel::Cleaned);
        assert_eq!(DistillationLevel::from_str("structured"), DistillationLevel::Structured);
        assert_eq!(DistillationLevel::from_str("DISTILLED"), DistillationLevel::Distilled);
        assert_eq!(DistillationLevel::from_str("unknown"), DistillationLevel::Raw);
    }

    #[test]
    fn test_meta_cognitive_report() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS crawl_queue (id TEXT PRIMARY KEY, status TEXT); \
             CREATE TABLE IF NOT EXISTS ingest_log (id INTEGER PRIMARY KEY, status TEXT); \
             CREATE TABLE IF NOT EXISTS kv_store (namespace TEXT, key TEXT, value TEXT, updated_at INTEGER);",
        )
        .unwrap();
        let report = MetaCognitiveReport::analyze(&conn);
        assert_eq!(report.failed_crawl, 0);
        assert_eq!(report.ingest_errors, 0);
        assert!(!report.recurring_error_recorded);
    }
}
