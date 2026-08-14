//! PROV-O 决策溯源 — 吸收自 semantica-agi/semantica (W3C PROV-O provenance:
//! 决策是一等对象, 每个断言携带 provenance, 冲突标记而非静默覆盖)。
//!
//! 为 NT-MEMORY 提供**决策溯源链**: 每条记录捕获 (who/agent, did/activity,
//! what/entity, why/evidence, when/timestamp), 落 KB kv_store `provenance`
//! 命名空间, 供审计回查 (D14/D20)。决策对象化让"为什么这个结论存在"
//! 可独立检索, 而非埋在节点 metadata 里不可查询。

use rusqlite::Connection;
use serde_json::json;

use super::nt_memory_unify::{kv_get, kv_set};

/// 溯源活动类型 — 对应 W3C PROV-Activity 的子类。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProvActivity {
    Absorb,
    Curate,
    Supersede,
    Recommend,
    VisibilityGate,
    AbsorbSpecMerge,
}

impl ProvActivity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Absorb => "absorb",
            Self::Curate => "curate",
            Self::Supersede => "supersede",
            Self::Recommend => "recommend",
            Self::VisibilityGate => "visibility-gate",
            Self::AbsorbSpecMerge => "absorb-spec-merge",
        }
    }
}

/// 一条决策溯源记录 (W3C PROV-O 语义)。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProvenanceRecord {
    /// 记录 id (uuid, 全局唯一)。
    pub id: String,
    /// 做出决策的 agent (如 "nt_memory_curation", "nt_mind_evolution_daemon")。
    pub agent: String,
    /// 活动类型 (决策动作)。
    pub activity: String,
    /// 受影响/产出的实体 (node id / spec id / query)。
    pub entity: String,
    /// 依据 (证据/被引用实体/理由)。
    pub evidence: Vec<String>,
    /// 决策结果摘要。
    pub outcome: String,
    /// unix 秒时间戳。
    pub created_at: i64,
}

impl ProvenanceRecord {
    pub fn new(
        agent: impl Into<String>,
        activity: ProvActivity,
        entity: impl Into<String>,
        outcome: impl Into<String>,
    ) -> Self {
        Self {
            id: format!("prov-{}", uuid::Uuid::new_v4()),
            agent: agent.into(),
            activity: activity.as_str().to_string(),
            entity: entity.into(),
            evidence: Vec::new(),
            outcome: outcome.into(),
            created_at: now_unix(),
        }
    }

    /// 追加证据 (被引用的实体/理由)。
    pub fn with_evidence(mut self, evidence: Vec<String>) -> Self {
        self.evidence = evidence;
        self
    }
}

/// 写入一条决策溯源记录 (kv_store `provenance` 命名空间)。
pub fn record_provenance(
    conn: &Connection,
    record: &ProvenanceRecord,
) -> Result<(), String> {
    let value = serde_json::to_string(record).map_err(|e| format!("serialize provenance: {}", e))?;
    kv_set(conn, "provenance", &record.id, &value)
}

/// 按 agent/activity/entity 过滤查询决策溯源。
/// 返回匹配记录 (按时间倒序)。
pub fn query_provenance(
    conn: &Connection,
    agent: Option<&str>,
    activity: Option<&str>,
    entity: Option<&str>,
) -> Result<Vec<ProvenanceRecord>, String> {
    let mut results = Vec::new();
    // 遍历 kv_store provenance 命名空间 (记录数有限, 全表扫可接受;
    // 若增长超阈值应迁移到独立表)。
    let all = kv_get(conn, "provenance", "__index__")?;
    let keys: Vec<String> = match all {
        Some(idx) => serde_json::from_str(&idx).unwrap_or_default(),
        None => Vec::new(),
    };
    for (pos, key) in keys.iter().enumerate() {
        if let Some(val) = kv_get(conn, "provenance", key)? {
            if let Ok(rec) = serde_json::from_str::<ProvenanceRecord>(&val) {
                if agent.map(|a| rec.agent == a).unwrap_or(true)
                    && activity.map(|a| rec.activity == a).unwrap_or(true)
                    && entity.map(|e| rec.entity == e).unwrap_or(true)
                {
                    results.push((pos, rec));
                }
            }
        }
    }
    // 按时间倒序; 同时间戳 (同秒写入) 以索引插入位置倒序作次键 (后写入在前)。
    results.sort_by(|(pa, a), (pb, b)| {
        b.created_at
            .cmp(&a.created_at)
            .then_with(|| pb.cmp(pa))
    });
    Ok(results.into_iter().map(|(_, r)| r).collect())
}

/// 便捷: 记录 + 维护索引。返回记录 id。
pub fn record_with_index(
    conn: &Connection,
    record: ProvenanceRecord,
) -> Result<String, String> {
    record_provenance(conn, &record)?;
    // 维护 __index__ (幂等追加, 防重复)
    let all = kv_get(conn, "provenance", "__index__")?;
    let mut keys: Vec<String> = all
        .and_then(|idx| serde_json::from_str(&idx).ok())
        .unwrap_or_default();
    if !keys.contains(&record.id) {
        keys.push(record.id.clone());
        let idx = serde_json::to_string(&keys).map_err(|e| format!("serialize index: {}", e))?;
        kv_set(conn, "provenance", "__index__", &idx)?;
    }
    Ok(record.id)
}

fn now_unix() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// 将溯源记录序列化为 audit 友好的 JSON (供审计链消费)。
pub fn to_audit_json(record: &ProvenanceRecord) -> serde_json::Value {
    json!({
        "prov": "PROV-O",
        "id": record.id,
        "agent": record.agent,
        "activity": record.activity,
        "entity": record.entity,
        "evidence": record.evidence,
        "outcome": record.outcome,
        "at": record.created_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn mem_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS kv_store (
                namespace TEXT NOT NULL,
                key TEXT NOT NULL,
                value TEXT,
                updated_at INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (namespace, key)
            );",
        )
        .unwrap();
        conn
    }

    #[test]
    fn record_and_query_by_agent() {
        let conn = mem_conn();
        let rec = ProvenanceRecord::new(
            "nt_memory_curation",
            ProvActivity::Supersede,
            "node-a",
            "a superseded by b",
        )
        .with_evidence(vec!["node-b".into()]);
        let id = record_with_index(&conn, rec).unwrap();
        assert!(id.starts_with("prov-"));

        let hits = query_provenance(&conn, Some("nt_memory_curation"), None, None).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].entity, "node-a");
        assert_eq!(hits[0].evidence, vec!["node-b".to_string()]);
    }

    #[test]
    fn query_filters_by_activity_and_entity() {
        let conn = mem_conn();
        record_with_index(
            &conn,
            ProvenanceRecord::new("a1", ProvActivity::Absorb, "e1", "absorbed"),
        )
        .unwrap();
        record_with_index(
            &conn,
            ProvenanceRecord::new("a1", ProvActivity::Curate, "e2", "curated"),
        )
        .unwrap();
        record_with_index(
            &conn,
            ProvenanceRecord::new("a2", ProvActivity::Absorb, "e1", "absorbed"),
        )
        .unwrap();

        let by_absorb = query_provenance(&conn, None, Some("absorb"), None).unwrap();
        assert_eq!(by_absorb.len(), 2);
        let by_entity = query_provenance(&conn, None, None, Some("e1")).unwrap();
        assert_eq!(by_entity.len(), 2);
        let by_both = query_provenance(&conn, Some("a1"), Some("absorb"), None).unwrap();
        assert_eq!(by_both.len(), 1);
    }

    #[test]
    fn newest_first_order() {
        let conn = mem_conn();
        record_with_index(
            &conn,
            ProvenanceRecord::new("a1", ProvActivity::Recommend, "e-old", "first"),
        )
        .unwrap();
        record_with_index(
            &conn,
            ProvenanceRecord::new("a1", ProvActivity::Recommend, "e-new", "second"),
        )
        .unwrap();
        let hits = query_provenance(&conn, None, Some("recommend"), None).unwrap();
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].entity, "e-new", "newest first");
    }

    #[test]
    fn record_idempotent_index() {
        let conn = mem_conn();
        let rec = ProvenanceRecord::new("a", ProvActivity::Absorb, "e", "o");
        let id = record_with_index(&conn, rec.clone()).unwrap();
        // 重复写入同 id → 索引不重复
        record_with_index(&conn, rec).unwrap();
        let hits = query_provenance(&conn, Some("a"), None, None).unwrap();
        assert_eq!(hits.len(), 1);
        let _ = id;
    }

    #[test]
    fn audit_json_shape() {
        let rec = ProvenanceRecord::new("agent", ProvActivity::VisibilityGate, "q", "allowed");
        let v = to_audit_json(&rec);
        assert_eq!(v["prov"], "PROV-O");
        assert_eq!(v["activity"], "visibility-gate");
        assert!(v["at"].is_i64());
    }
}