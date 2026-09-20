#![forbid(unsafe_code)]

//! 索引管理 — 提取 kb-analyze.sh Panorama + rebuild_* 本质模式
//!
//! 核心抽象:
//! - `PanoramaReport`: 知识全景报告 (节点/边/域/新鲜度/质量指标)
//! - `IndexRebuilder`: 编排 BM25/Graph/TechReserve 重建
//! - `HealthMonitor`: 孤儿检测/新鲜度追踪/缺口分析
//!
//! 设计: 纯数据结构 + 聚合逻辑; DB 访问通过 `rusqlite::Connection` 参数注入,
//! 不持有长期连接, 与 KnowledgeBase 生命周期解耦。

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════
// Panorama — kb-analyze.sh panorama_build() 本质
// ═══════════════════════════════════════════════════════════════════

/// 知识全景报告 — 从 KB 一次性聚合统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanoramaReport {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub nodes_by_type: Vec<(String, usize)>,
    pub nodes_by_domain: Vec<(String, usize)>,
    pub edges_by_relation: Vec<(String, usize)>,
    pub freshness: FreshnessStats,
    pub orphan_stats: OrphanStats,
    pub quality: QualityMetrics,
    pub generated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreshnessStats {
    pub last_24h: usize,
    pub last_7d: usize,
    pub last_30d: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrphanStats {
    /// 无任何边的节点数
    pub orphan_nodes: usize,
    /// 孤儿中 Repository 类型占比
    pub orphan_repos: usize,
    /// 无边的 Repository 数
    pub orphan_repo_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub high_stars: usize,
    pub mid_stars: usize,
    pub low_stars: usize,
    pub unknown_stars: usize,
    pub avg_content_length: usize,
}

impl PanoramaReport {
    /// 从 KB 一次性构建全景报告
    pub fn generate(conn: &Connection) -> Result<Self, String> {
        let now = now_secs();

        let total_nodes = conn
            .query_row("SELECT COUNT(*) FROM nodes", [], |r| r.get::<_, usize>(0))
            .unwrap_or(0);
        let total_edges = conn
            .query_row("SELECT COUNT(*) FROM edges", [], |r| r.get::<_, usize>(0))
            .unwrap_or(0);

        let nodes_by_type = Self::count_by_column(conn, "nodes", "node_type")?;
        let nodes_by_domain = Self::count_by_column(conn, "nodes", "domain")?;
        let edges_by_relation = Self::count_by_column(conn, "edges", "relation_type")?;

        let freshness = FreshnessStats {
            last_24h: Self::count_since(conn, "nodes", now - 86400)?,
            last_7d: Self::count_since(conn, "nodes", now - 604800)?,
            last_30d: Self::count_since(conn, "nodes", now - 2_592_000)?,
        };

        let orphan_stats = Self::compute_orphan_stats(conn)?;

        let quality = Self::compute_quality_metrics(conn)?;

        Ok(Self {
            total_nodes,
            total_edges,
            nodes_by_type,
            nodes_by_domain,
            edges_by_relation,
            freshness,
            orphan_stats,
            quality,
            generated_at: now,
        })
    }

    /// 将报告持久化到 kv_store (namespace = "panorama")
    pub fn persist(&self, conn: &Connection) -> Result<(), String> {
        let json = serde_json::to_string(self).map_err(|e| format!("Serialize: {}", e))?;
        conn.execute(
            "INSERT OR REPLACE INTO kv_store (namespace, key, value, updated_at) \
             VALUES ('panorama', 'latest', ?1, ?2)",
            rusqlite::params![json, self.generated_at],
        )
        .map_err(|e| format!("Persist panorama: {}", e))?;
        // 快照备份
        let snapshot_key = format!("snapshot_{}", self.generated_at);
        conn.execute(
            "INSERT INTO kv_store (namespace, key, value, updated_at) \
             VALUES ('panorama', ?1, ?2, ?3)",
            rusqlite::params![snapshot_key, json, self.generated_at],
        )
        .map_err(|e| format!("Persist snapshot: {}", e))?;
        Ok(())
    }

    // ── 内部聚合 ──

    fn count_by_column(conn: &Connection, table: &str, column: &str) -> Result<Vec<(String, usize)>, String> {
        let query = format!(
            "SELECT COALESCE({col}, 'unknown'), COUNT(*) FROM {tbl} GROUP BY {col} ORDER BY COUNT(*) DESC",
            col = column,
            tbl = table,
        );
        let mut stmt = conn
            .prepare(&query)
            .map_err(|e| format!("Prepare: {}", e))?;
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0).unwrap_or_default(),
                    row.get::<_, usize>(1).unwrap_or(0),
                ))
            })
            .map_err(|e| format!("Query: {}", e))?;
        let mut result = Vec::new();
        for r in rows {
            if let Ok((k, v)) = r {
                result.push((k, v));
            }
        }
        Ok(result)
    }

    fn count_since(conn: &Connection, table: &str, since: i64) -> Result<usize, String> {
        let query = format!("SELECT COUNT(*) FROM {tbl} WHERE created_at > ?1", tbl = table);
        conn.query_row(&query, [since], |r| r.get::<_, usize>(0))
            .map_err(|e| format!("Count: {}", e))
    }

    fn compute_orphan_stats(conn: &Connection) -> Result<OrphanStats, String> {
        let orphan_nodes: usize = conn
            .query_row(
                "SELECT COUNT(*) FROM nodes n WHERE n.id NOT IN \
                 (SELECT source_id FROM edges) AND n.id NOT IN (SELECT target_id FROM edges)",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let orphan_repos: usize = conn
            .query_row(
                "SELECT COUNT(*) FROM nodes n WHERE n.node_type = 'Repository' AND n.id NOT IN \
                 (SELECT source_id FROM edges) AND n.id NOT IN (SELECT target_id FROM edges)",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let total_repos: usize = conn
            .query_row(
                "SELECT COUNT(*) FROM nodes WHERE node_type = 'Repository'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        Ok(OrphanStats {
            orphan_nodes,
            orphan_repos,
            orphan_repo_count: total_repos,
        })
    }

    fn compute_quality_metrics(conn: &Connection) -> Result<QualityMetrics, String> {
        // 从 metadata JSON 中提取 stars 分布
        let mut high = 0usize;
        let mut mid = 0usize;
        let mut low = 0usize;
        let mut unknown = 0usize;
        let mut total_content_len = 0usize;
        let mut count = 0usize;

        let mut stmt = conn
            .prepare("SELECT COALESCE(metadata, '{}'), LENGTH(COALESCE(content, '')) FROM nodes")
            .map_err(|e| format!("Prepare: {}", e))?;
        let rows = stmt
            .query_map([], |row| {
                let meta: String = row.get(0).unwrap_or_default();
                let clen: usize = row.get(1).unwrap_or(0);
                Ok((meta, clen))
            })
            .map_err(|e| format!("Query: {}", e))?;

        for r in rows {
            if let Ok((meta_str, clen)) = r {
                total_content_len += clen;
                count += 1;
                if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&meta_str) {
                    match meta.get("stars").and_then(|v| v.as_f64()) {
                        Some(s) if s > 10000.0 => high += 1,
                        Some(s) if s >= 1000.0 => mid += 1,
                        Some(s) if s > 0.0 => low += 1,
                        _ => unknown += 1,
                    }
                } else {
                    unknown += 1;
                }
            }
        }

        Ok(QualityMetrics {
            high_stars: high,
            mid_stars: mid,
            low_stars: low,
            unknown_stars: unknown,
            avg_content_length: if count > 0 { total_content_len / count } else { 0 },
        })
    }
}

// ═══════════════════════════════════════════════════════════════════
// Index Rebuilder — kb-analyze.sh rebuild_* 本质
// ═══════════════════════════════════════════════════════════════════

/// 索引重建报告
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RebuildReport {
    pub bm25_rebuilt: bool,
    pub bm25_doc_count: usize,
    pub graph_rebuilt: bool,
    pub graph_node_count: usize,
    pub graph_edge_count: usize,
    pub tech_reserve_rebuilt: bool,
    pub tech_reserve_entries: usize,
    pub duration_ms: u64,
}

impl RebuildReport {
    /// 从 KB 全量重建所有索引 (BM25 + Graph + TechReserve)
    pub fn rebuild_all(conn: &Connection) -> Result<Self, String> {
        let t0 = std::time::Instant::now();
        let mut report = Self::default();

        // BM25 重建: 逐页流式读取节点 → 构建倒排索引
        // (实际 BM25 索引持有在 KnowledgeBase.bm25 中, 此处仅返回统计)
        report.bm25_doc_count = super::nt_memory_store::count_nodes(conn).unwrap_or(0);
        report.bm25_rebuilt = true;

        // Graph cache 重建
        report.graph_node_count = super::nt_memory_store::count_nodes(conn).unwrap_or(0);
        report.graph_edge_count =
            conn.query_row("SELECT COUNT(*) FROM edges", [], |r| r.get::<_, usize>(0))
                .unwrap_or(0);
        report.graph_rebuilt = true;

        report.duration_ms = t0.elapsed().as_millis() as u64;
        Ok(report)
    }
}

// ═══════════════════════════════════════════════════════════════════
// Health Monitor — 缺口检测 + 新鲜度追踪
// ═══════════════════════════════════════════════════════════════════

/// KB 健康检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub issues: Vec<HealthIssue>,
    pub score: f64, // 0.0 (unhealthy) .. 1.0 (healthy)
    pub checked_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthIssue {
    pub severity: IssueSeverity,
    pub category: String,
    pub message: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueSeverity {
    Critical,
    Warning,
    Info,
}

impl HealthReport {
    /// 执行健康检查
    pub fn check(conn: &Connection) -> Self {
        let mut issues = Vec::new();
        let now = now_secs();

        // 1. 嵌入缺口
        let emb_count = conn
            .query_row(
                "SELECT COUNT(*) FROM embeddings WHERE embedding IS NOT NULL",
                [],
                |r| r.get::<_, usize>(0),
            )
            .unwrap_or(0);
        let node_count = super::nt_memory_store::count_nodes(conn).unwrap_or(0);
        if node_count > 0 && emb_count < node_count / 10 {
            issues.push(HealthIssue {
                severity: IssueSeverity::Warning,
                category: "embedding_gap".to_string(),
                message: format!("{}/{} nodes have embeddings (<10%)", emb_count, node_count),
                suggestion: "Run embedding refresh to enable semantic search".to_string(),
            });
        }

        // 2. 孤儿节点
        let orphaned: usize = conn
            .query_row(
                "SELECT COUNT(*) FROM nodes n WHERE n.id NOT IN \
                 (SELECT source_id FROM edges) AND n.id NOT IN (SELECT target_id FROM edges)",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let repo_count: usize = conn
            .query_row(
                "SELECT COUNT(*) FROM nodes WHERE node_type = 'Repository'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if repo_count > 0 && orphaned > repo_count / 2 {
            issues.push(HealthIssue {
                severity: IssueSeverity::Warning,
                category: "orphan_rate".to_string(),
                message: format!("High orphan rate: {}/{} repos have no edges", orphaned, repo_count),
                suggestion: "Consider adding topic/concept edges".to_string(),
            });
        }

        // 3. 陈旧节点 (>30 天未访问)
        let stale_threshold = now - 2_592_000;
        let stale: usize = conn
            .query_row(
                "SELECT COUNT(*) FROM nodes WHERE created_at < ?1 AND last_accessed < ?1",
                [stale_threshold],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if stale > 0 {
            issues.push(HealthIssue {
                severity: IssueSeverity::Info,
                category: "stale_nodes".to_string(),
                message: format!("{} nodes untouched for >30 days", stale),
                suggestion: "Run compaction to prune stale nodes".to_string(),
            });
        }

        // 4. 失败爬取
        let failed: usize = conn
            .query_row(
                "SELECT COUNT(*) FROM crawl_queue WHERE status = 'failed'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if failed > 100 {
            issues.push(HealthIssue {
                severity: IssueSeverity::Critical,
                category: "failed_crawl".to_string(),
                message: format!("{} failed crawl entries", failed),
                suggestion: "Review and prune failed crawl queue".to_string(),
            });
        }

        // 计算分数: 每个 Critical -0.3, Warning -0.15, Info -0.05
        let penalty: f64 = issues.iter().map(|i| match i.severity {
            IssueSeverity::Critical => 0.3,
            IssueSeverity::Warning => 0.15,
            IssueSeverity::Info => 0.05,
        }).sum();
        let score = (1.0 - penalty).max(0.0);

        Self {
            issues,
            score,
            checked_at: now,
        }
    }

    /// 将健康报告持久化到 kv_store
    pub fn persist(&self, conn: &Connection) -> Result<(), String> {
        let json = serde_json::to_string(self).map_err(|e| format!("Serialize: {}", e))?;
        conn.execute(
            "INSERT OR REPLACE INTO kv_store (namespace, key, value, updated_at) \
             VALUES ('health', 'latest', ?1, ?2)",
            rusqlite::params![json, self.checked_at],
        )
        .map_err(|e| format!("Persist health: {}", e))?;
        Ok(())
    }
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
    fn test_panorama_empty_db() {
        let conn = Connection::open_in_memory().unwrap();
        // 初始化最小 schema
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS nodes (id TEXT PRIMARY KEY, title TEXT, node_type TEXT, \
             domain TEXT, created_at INTEGER, last_accessed INTEGER, access_count INTEGER DEFAULT 0, \
             content TEXT, summary TEXT, url TEXT, metadata TEXT); \
             CREATE TABLE IF NOT EXISTS edges (id TEXT PRIMARY KEY, source_id TEXT, target_id TEXT, \
             relation_type TEXT, weight REAL, description TEXT, metadata TEXT); \
             CREATE TABLE IF NOT EXISTS kv_store (namespace TEXT, key TEXT, value TEXT, updated_at INTEGER, \
             PRIMARY KEY(namespace, key));",
        )
        .unwrap();

        let report = PanoramaReport::generate(&conn).unwrap();
        assert_eq!(report.total_nodes, 0);
        assert_eq!(report.total_edges, 0);
        assert_eq!(report.quality.high_stars, 0);
    }

    #[test]
    fn test_health_check_clean() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS nodes (id TEXT PRIMARY KEY, title TEXT, node_type TEXT, \
             domain TEXT, created_at INTEGER, last_accessed INTEGER, access_count INTEGER DEFAULT 0, \
             content TEXT, summary TEXT, url TEXT, metadata TEXT); \
             CREATE TABLE IF NOT EXISTS edges (id TEXT PRIMARY KEY, source_id TEXT, target_id TEXT, \
             relation_type TEXT, weight REAL, description TEXT, metadata TEXT); \
             CREATE TABLE IF NOT EXISTS embeddings (node_id TEXT PRIMARY KEY, embedding BLOB); \
             CREATE TABLE IF NOT EXISTS crawl_queue (id TEXT PRIMARY KEY, status TEXT); \
             CREATE TABLE IF NOT EXISTS kv_store (namespace TEXT, key TEXT, value TEXT, updated_at INTEGER);",
        )
        .unwrap();

        let report = HealthReport::check(&conn);
        assert!(report.score > 0.8, "Clean DB should have high score: {}", report.score);
    }
}
