//! NT-MEMORY 设定一致性检查器 — 对标网文"每卷设定一致性检查"
//!
//! 网文作者写到 300 章不崩设定，靠版本化设定库 + 每卷检查前后矛盾。
//! 本模块对 KB 执行同样的"设定检查"：检测知识漂移/冲突/重复，
//! 防止知识库"写到 300 章崩设定"（概念被重复定义、定义互相矛盾、版本链断裂）。
//!
//! 检查维度（对标网文设定集检查）：
//!   1. 重复定义 (duplicate)  — 同一 title 多个节点（同概念多版本并存）→ warning
//!   2. 冲突定义 (conflict)   — 同 title 节点 summary 差异大（前后矛盾）→ error
//!   3. 版本漂移 (drift)      — supersedes 悬空引用 / 无来源节点 → warning/info
//!
//! 用法：`check(&conn)` 返回报告；`check_and_report(&conn, path)` 打印人类可读报告。

use rusqlite::Connection;

/// 问题严重度（对标网文设定检查的"硬伤/软伤/提示"）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
        }
    }
}

/// 单个设定问题
#[derive(Debug, Clone)]
pub struct SettingIssue {
    pub severity: Severity,
    pub dimension: &'static str,
    pub node_id: String,
    pub title: String,
    pub description: String,
}

/// 设定一致性检查报告
#[derive(Debug, Clone, Default)]
pub struct SettingConsistencyReport {
    pub checked_at: u64,
    pub issues: Vec<SettingIssue>,
    pub scanned_nodes: usize,
}

impl SettingConsistencyReport {
    pub fn count(&self, severity: Severity) -> usize {
        self.issues.iter().filter(|i| i.severity == severity).count()
    }

    pub fn is_clean(&self) -> bool {
        self.count(Severity::Error) == 0 && self.count(Severity::Warning) == 0
    }
}

/// 执行设定一致性检查（对标网文"每卷设定检查"）
pub fn check(conn: &Connection) -> rusqlite::Result<SettingConsistencyReport> {
    let mut report = SettingConsistencyReport {
        checked_at: now_ts(),
        ..Default::default()
    };

    // 扫描节点数
    report.scanned_nodes = conn.query_row("SELECT COUNT(*) FROM nodes", [], |r| r.get(0))?;

    check_duplicates(conn, &mut report)?;
    check_conflicts(conn, &mut report)?;
    check_drift(conn, &mut report)?;

    Ok(report)
}

/// 维度 1: 重复定义 — 同一 title 多个节点（同概念多 id 并存）
fn check_duplicates(conn: &Connection, report: &mut SettingConsistencyReport) -> rusqlite::Result<()> {
    let mut stmt = conn.prepare(
        "SELECT title, COUNT(*) AS c FROM nodes \
         WHERE title IS NOT NULL AND title != '' \
         GROUP BY title HAVING c > 1 ORDER BY c DESC LIMIT 50",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
    })?;
    for row in rows {
        let (title, count) = row?;
        // 找出这些重复节点的 id
        let mut ids = conn.prepare(
            "SELECT id FROM nodes WHERE title = ?1 ORDER BY created_at LIMIT 10",
        )?;
        let id_list: Vec<String> = ids
            .query_map([&title], |r| r.get::<_, String>(0))?
            .collect::<Result<_, _>>()?;
        report.issues.push(SettingIssue {
            severity: Severity::Warning,
            dimension: "duplicate",
            node_id: id_list.join(","),
            title: title.clone(),
            description: format!("同一标题 {} 个节点 (可能重复定义)", count),
        });
    }
    Ok(())
}

/// 维度 2: 冲突定义 — 同 title 节点 summary 差异大（前后矛盾）
fn check_conflicts(conn: &Connection, report: &mut SettingConsistencyReport) -> rusqlite::Result<()> {
    let mut stmt = conn.prepare(
        "SELECT title, COUNT(*) AS c FROM nodes
         WHERE title IS NOT NULL AND title != '' AND summary IS NOT NULL AND summary != ''
         GROUP BY title HAVING c > 1 LIMIT 50",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
    })?;
    for row in rows {
        let (title, count) = row?;
        // 取该 title 下所有 summary，比较长度差异
        let mut stmt2 = conn.prepare(
            "SELECT id, summary FROM nodes WHERE title = ?1 AND summary IS NOT NULL ORDER BY created_at LIMIT 10",
        )?;
        let summaries: Vec<(String, String)> = stmt2
            .query_map([&title], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
            .collect::<Result<_, _>>()?;
        if summaries.len() < 2 {
            continue;
        }
        let lens: Vec<usize> = summaries.iter().map(|(_, s)| s.len()).collect();
        let min_len = *lens.iter().min().unwrap_or(&0);
        let max_len = *lens.iter().max().unwrap_or(&0);
        // 长度差异 > 3 倍 → 疑似冲突定义
        if max_len > 0 && min_len > 0 && max_len > min_len * 3 {
            report.issues.push(SettingIssue {
                severity: Severity::Error,
                dimension: "conflict",
                node_id: summaries.iter().map(|(id, _)| id.clone()).collect::<Vec<_>>().join(","),
                title: title.clone(),
                description: format!(
                    "同一概念 {} 个节点 summary 长度差异 {}x ({} vs {}) — 疑似前后矛盾",
                    count, max_len / min_len, min_len, max_len
                ),
            });
        }
    }
    Ok(())
}

/// 维度 3: 版本漂移 — supersedes 悬空引用 + 无来源节点
fn check_drift(conn: &Connection, report: &mut SettingConsistencyReport) -> rusqlite::Result<()> {
    // 3a: supersedes 指向不存在的节点
    let mut stmt = conn.prepare(
        "SELECT id, title, supersedes FROM nodes
         WHERE supersedes IS NOT NULL AND supersedes != ''
         AND supersedes NOT IN (SELECT id FROM nodes) LIMIT 50",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?))
    })?;
    for row in rows {
        let (id, title, supersedes) = row?;
        report.issues.push(SettingIssue {
            severity: Severity::Warning,
            dimension: "drift",
            node_id: id,
            title,
            description: format!("supersedes 悬空引用: 指向不存在的节点 {}", supersedes),
        });
    }

    // 3b: 无来源节点（source_episode 为空且 url 为空）→ info
    let mut stmt = conn.prepare(
        "SELECT id, title FROM nodes
         WHERE (source_episode IS NULL OR source_episode = '')
         AND (url IS NULL OR url = '') LIMIT 50",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
    })?;
    for row in rows {
        let (id, title) = row?;
        report.issues.push(SettingIssue {
            severity: Severity::Info,
            dimension: "drift",
            node_id: id,
            title,
            description: "无来源节点 (缺 source_episode 且缺 url) — 无法追溯设定出处".to_string(),
        });
    }
    Ok(())
}

/// 便捷函数: 执行检查并打印人类可读报告（对标"每卷设定检查"）
pub fn check_and_report(conn: &Connection, path: &str) -> rusqlite::Result<SettingConsistencyReport> {
    let report = check(conn)?;
    println!("=== KB 设定一致性检查 (每卷设定检查) ===");
    println!("  检查时间: {}", report.checked_at);
    println!("  扫描节点: {}", report.scanned_nodes);
    println!(
        "  发现: {} error / {} warning / {} info",
        report.count(Severity::Error),
        report.count(Severity::Warning),
        report.count(Severity::Info)
    );
    for issue in &report.issues {
        println!(
            "  [{}] {} {} — {} (node: {})",
            issue.severity.as_str(),
            issue.dimension,
            issue.title.chars().take(40).collect::<String>(),
            issue.description,
            issue.node_id.chars().take(40).collect::<String>(),
        );
    }
    if report.is_clean() {
        println!("  ✅ 设定一致，无硬伤");
    } else {
        println!("  ⚠️ 存在设定问题，建议修复后继续");
    }
    Ok(report)
}

fn now_ts() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_schema;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        nt_memory_schema::initialize(&conn).unwrap();
        conn
    }

    fn insert_node(conn: &Connection, id: &str, title: &str, summary: &str, supersedes: Option<&str>) {
        let now = now_ts();
        conn.execute(
            "INSERT INTO nodes (id,node_type,title,summary,content,url,domain,language,
             confidence,importance,created_at,updated_at,access_count,metadata,
             data_tier,temporal,supersedes,source_episode,tier)
             VALUES (?1,'concept',?2,?3,'','','test','zh',1.0,0.5,?4,?4,0,'{}','core',NULL,?5,'test','warm')",
            rusqlite::params![id, title, summary, now, supersedes],
        )
        .unwrap();
    }

    #[test]
    fn test_clean_kb_no_issues() {
        let conn = test_conn();
        insert_node(&conn, "n1", "唯一概念", "定义A", None);
        let report = check(&conn).unwrap();
        assert_eq!(report.count(Severity::Error), 0);
        assert_eq!(report.count(Severity::Warning), 0);
        assert!(report.is_clean());
    }

    #[test]
    fn test_duplicate_detection() {
        let conn = test_conn();
        insert_node(&conn, "n1", "重复概念", "定义A", None);
        insert_node(&conn, "n2", "重复概念", "定义B", None);
        let report = check(&conn).unwrap();
        let dup = report.issues.iter().find(|i| i.dimension == "duplicate");
        assert!(dup.is_some(), "应检测到重复定义");
        assert_eq!(dup.unwrap().severity, Severity::Warning);
    }

    #[test]
    fn test_conflict_detection() {
        let conn = test_conn();
        insert_node(&conn, "n1", "冲突概念", "短", None);
        insert_node(&conn, "n2", "冲突概念", &"长".repeat(100), None);
        let report = check(&conn).unwrap();
        let conflict = report.issues.iter().find(|i| i.dimension == "conflict");
        assert!(conflict.is_some(), "应检测到冲突定义");
        assert_eq!(conflict.unwrap().severity, Severity::Error);
    }

    #[test]
    fn test_drift_dangling_supersedes() {
        let conn = test_conn();
        insert_node(&conn, "n1", "漂移概念", "定义", Some("ghost-node"));
        let report = check(&conn).unwrap();
        let drift = report.issues.iter().find(|i| i.dimension == "drift" && i.severity == Severity::Warning);
        assert!(drift.is_some(), "应检测到悬空 supersedes");
    }
}