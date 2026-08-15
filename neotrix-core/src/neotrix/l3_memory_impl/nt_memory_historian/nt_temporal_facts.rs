//! nt_temporal_facts — 时序上下文图 (opencontext absorb, G5)
//!
//! 事实以版本链存储, 具备:
//!   - valid_from / valid_until: 事实的时空有效期 (opencontext: 上下文有时效)
//!   - supersession 边: 更正时写入新版本并指向上版, 旧版 valid_until 自动截断
//!     (append-only 更正: 原始事实永不原地修改 — 更正性)
//!   - contradiction 边: 事实间矛盾互指 (opencontext: 矛盾保留, 供裁决)
//!   - point-in-time 查询: query_valid_at(ts) 返回该时刻有效的事实
//!
//! 存储: 独立 SQLite 表 temporal_facts (append-only), 默认落在 KB 主库。

use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TemporalFact {
    /// 事实 ID (版本唯一)
    pub id: String,
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub valid_from: i64,
    pub valid_until: Option<i64>,
    /// 被哪个新版本取代 (supersession 指针)
    pub superseded_by: Option<String>,
    /// 取代了哪个旧版本
    pub supersedes: Option<String>,
    /// 矛盾事实 ID 列表
    pub contradicted_by: Vec<String>,
    pub created_at: i64,
    /// 来源 (evidence/trace 溯源)
    pub source: String,
}

pub struct TemporalFactLedger {
    conn: Connection,
}

const CREATE_SQL: &str = "CREATE TABLE IF NOT EXISTS temporal_facts (
    id TEXT PRIMARY KEY,
    subject TEXT NOT NULL,
    predicate TEXT NOT NULL,
    object TEXT NOT NULL,
    valid_from INTEGER NOT NULL,
    valid_until INTEGER,
    superseded_by TEXT,
    supersedes TEXT,
    contradicted_by TEXT NOT NULL DEFAULT '[]',
    created_at INTEGER NOT NULL,
    source TEXT NOT NULL
)";

impl TemporalFactLedger {
    pub fn open(db_path: Option<&std::path::Path>) -> Result<Self, String> {
        let path = db_path.map(|p| p.to_path_buf()).unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            std::path::PathBuf::from(home).join(".neotrix").join("knowledge.db")
        });
        let is_memory = path.to_string_lossy() == ":memory:";
        let conn = if is_memory {
            Connection::open_in_memory().map_err(|e| format!("open in-memory db: {e}"))?
        } else {
            Connection::open(&path).map_err(|e| format!("open db: {e}"))?
        };
        conn.execute_batch(CREATE_SQL).map_err(|e| format!("init schema: {e}"))?;
        Ok(Self { conn })
    }

    /// 新增事实 (append-only)。若与现有同 subject+predicate 冲突 → 返回 Err,
    /// 调用方应使用 supersede() 显式更正 (保证只新增不修改)。
    pub fn add_fact(
        &self,
        subject: &str,
        predicate: &str,
        object: &str,
        valid_from: Option<i64>,
        valid_until: Option<i64>,
        source: &str,
    ) -> Result<TemporalFact, String> {
        let id = format!(
            "tf_{}_{}",
            now_ts(),
            uuid6()
        );
        // 幂等防重: 同 subject+predicate+object 且仍有效 → 拒绝 (纯重复)。
        // 不同 object (真矛盾) 允许并存, 由 record_contradiction 显式建矛盾边。
        let conflict: Option<i64> = self
            .conn
            .query_row(
                "SELECT 1 FROM temporal_facts WHERE subject=?1 AND predicate=?2 AND object=?3
                 AND valid_until IS NULL LIMIT 1",
                params![subject, predicate, object],
                |_| Ok(1),
            )
            .ok();
        if conflict.is_some() {
            return Err(format!(
                "duplicate active fact: {subject} {predicate} {object} already exists — \
                 use supersede() to correct or record_contradiction() for conflicts"
            ));
        }
        let fact = TemporalFact {
            id: id.clone(),
            subject: subject.to_string(),
            predicate: predicate.to_string(),
            object: object.to_string(),
            valid_from: valid_from.unwrap_or_else(now_ts),
            valid_until,
            superseded_by: None,
            supersedes: None,
            contradicted_by: Vec::new(),
            created_at: now_ts(),
            source: source.to_string(),
        };
        self.insert_fact(&fact)?;
        Ok(fact)
    }

    /// 更正 (append-only): 新版本取代旧版本 — 旧版 valid_until 截断 + superseded_by,
    /// 新版 supersedes 指向旧版。原始事实不原地修改。
    pub fn supersede(
        &self,
        old_id: &str,
        new_object: &str,
        valid_from: Option<i64>,
        source: &str,
    ) -> Result<TemporalFact, String> {
        let old: TemporalFact = self.get_fact(old_id)?.ok_or_else(|| format!("fact {old_id} not found"))?;
        if old.superseded_by.is_some() {
            return Err(format!("fact {old_id} already superseded"));
        }
        // 截断旧版有效期 (append-only 更正)
        let cut = valid_from.unwrap_or_else(now_ts);
        self.conn
            .execute(
                "UPDATE temporal_facts SET valid_until=?2, superseded_by=?3 WHERE id=?1",
                params![old_id, cut, format!("sup-{old_id}")],
            )
            .map_err(|e| format!("close old fact: {e}"))?;
        // 写新版
        let new_id = format!("{old_id}-v{}", version_seq(old_id, &self.conn));
        let fact = TemporalFact {
            id: new_id,
            subject: old.subject.clone(),
            predicate: old.predicate.clone(),
            object: new_object.to_string(),
            valid_from: cut,
            valid_until: None,
            superseded_by: None,
            supersedes: Some(old_id.to_string()),
            contradicted_by: Vec::new(),
            created_at: now_ts(),
            source: source.to_string(),
        };
        self.insert_fact(&fact)?;
        Ok(fact)
    }

    /// 记录矛盾边: 两事实互指 contradicted_by (矛盾保留, 供裁决, 不删除)
    pub fn record_contradiction(&self, a_id: &str, b_id: &str) -> Result<(), String> {
        let a = self.get_fact(a_id)?.ok_or_else(|| format!("fact {a_id} not found"))?;
        let b = self.get_fact(b_id)?.ok_or_else(|| format!("fact {b_id} not found"))?;
        self.append_contradiction(a_id, b_id, &a)?;
        self.append_contradiction(b_id, a_id, &b)?;
        Ok(())
    }

    fn append_contradiction(&self, id: &str, other: &str, _fact: &TemporalFact) -> Result<(), String> {
        let cur: String = self
            .conn
            .query_row(
                "SELECT contradicted_by FROM temporal_facts WHERE id=?1",
                params![id],
                |r| r.get(0),
            )
            .map_err(|e| format!("read contradiction: {e}"))?;
        let mut list: Vec<String> =
            serde_json::from_str(&cur).unwrap_or_default();
        if !list.contains(&other.to_string()) {
            list.push(other.to_string());
            self.conn
                .execute(
                    "UPDATE temporal_facts SET contradicted_by=?2 WHERE id=?1",
                    params![id, serde_json::to_string(&list).unwrap_or("[]".into())],
                )
                .map_err(|e| format!("write contradiction: {e}"))?;
        }
        Ok(())
    }

    pub fn get_fact(&self, id: &str) -> Result<Option<TemporalFact>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, subject, predicate, object, valid_from, valid_until,
                        superseded_by, supersedes, contradicted_by, created_at, source
                 FROM temporal_facts WHERE id=?1",
            )
            .map_err(|e| format!("prepare: {e}"))?;
        let mut rows = stmt
            .query_map(params![id], row_to_fact)
            .map_err(|e| format!("query: {e}"))?;
        rows.next().transpose().map_err(|e| format!("read: {e}"))
    }

    /// 点时刻查询: 返回 ts 时刻有效 (valid_from<=ts 且 (valid_until IS NULL 或 ts<valid_until))
    /// 的事实版本 (含 supersession 链上各版本, 主版本优先)。
    pub fn query_valid_at(&self, ts: i64) -> Result<Vec<TemporalFact>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, subject, predicate, object, valid_from, valid_until,
                        superseded_by, supersedes, contradicted_by, created_at, source
                 FROM temporal_facts
                 WHERE valid_from<=?1 AND (valid_until IS NULL OR valid_until>?1)
                 ORDER BY valid_from ASC",
            )
            .map_err(|e| format!("prepare: {e}"))?;
        let rows = stmt
            .query_map(params![ts], row_to_fact)
            .map_err(|e| format!("query: {e}"))?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| format!("read: {e}"))
    }

    /// 事实版本链: 沿 supersedes 回溯
    pub fn history_chain(&self, leaf_id: &str) -> Result<Vec<TemporalFact>, String> {
        let mut chain = Vec::new();
        let mut cur = leaf_id.to_string();
        let mut guard = 0;
        while let Some(f) = self.get_fact(&cur)? {
            chain.push(f.clone());
            match f.supersedes {
                Some(prev) if guard < 64 => cur = prev,
                _ => break,
            }
            guard += 1;
        }
        Ok(chain)
    }

    pub fn count(&self) -> Result<usize, String> {
        self.conn
            .query_row("SELECT COUNT(*) FROM temporal_facts", [], |r| r.get(0))
            .map_err(|e| format!("count: {e}"))
    }

    /// 生产接线 (KB ingest): 记录一条节点事实 (append-only)。事实 id 由节点 id
    /// 确定性派生 (node → fact 1:1), 更正时 supersede_node_fact 可直接定位旧版本。
    pub fn add_node_fact(
        &self,
        node_id: &str,
        subject: &str,
        predicate: &str,
        object: &str,
        valid_from: Option<i64>,
        valid_until: Option<i64>,
        source: &str,
    ) -> Result<TemporalFact, String> {
        let id = format!("tf_n_{node_id}");
        // 幂等防重: 同 subject+predicate+object 且仍有效 → 拒绝 (纯重复, 如节点重摄)。
        let conflict: Option<i64> = self
            .conn
            .query_row(
                "SELECT 1 FROM temporal_facts WHERE subject=?1 AND predicate=?2 AND object=?3
                 AND valid_until IS NULL LIMIT 1",
                params![subject, predicate, object],
                |_| Ok(1),
            )
            .ok();
        if conflict.is_some() {
            return Err(format!(
                "duplicate active fact: {subject} {predicate} {object} already exists — \
                 use supersede() to correct or record_contradiction() for conflicts"
            ));
        }
        let fact = TemporalFact {
            id: id.clone(),
            subject: subject.to_string(),
            predicate: predicate.to_string(),
            object: object.to_string(),
            valid_from: valid_from.unwrap_or_else(now_ts),
            valid_until,
            superseded_by: None,
            supersedes: None,
            contradicted_by: Vec::new(),
            created_at: now_ts(),
            source: source.to_string(),
        };
        self.insert_fact(&fact)?;
        Ok(fact)
    }

    /// 生产接线 (KB 更正): 旧节点事实沿版本链 supersede (append-only) — 旧版
    /// valid_until 截断 + superseded_by, 新对象为更正后的正文。
    pub fn supersede_node_fact(
        &self,
        old_node_id: &str,
        new_object: &str,
        valid_from: Option<i64>,
        source: &str,
    ) -> Result<TemporalFact, String> {
        let old_fact_id = format!("tf_n_{old_node_id}");
        self.supersede(&old_fact_id, new_object, valid_from, source)
    }

    /// 生产接线 (KB 点时刻查询): 按 subject (节点标题) 过滤的有效事实版本。
    pub fn query_valid_at_subject(
        &self,
        subject: &str,
        ts: i64,
    ) -> Result<Vec<TemporalFact>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, subject, predicate, object, valid_from, valid_until,
                        superseded_by, supersedes, contradicted_by, created_at, source
                 FROM temporal_facts
                 WHERE subject=?1 AND valid_from<=?2 AND (valid_until IS NULL OR valid_until>?2)
                 ORDER BY valid_from ASC",
            )
            .map_err(|e| format!("prepare: {e}"))?;
        let rows = stmt
            .query_map(params![subject, ts], row_to_fact)
            .map_err(|e| format!("query: {e}"))?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| format!("read: {e}"))
    }

    fn insert_fact(&self, fact: &TemporalFact) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT INTO temporal_facts
                 (id, subject, predicate, object, valid_from, valid_until,
                  superseded_by, supersedes, contradicted_by, created_at, source)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
                params![
                    fact.id, fact.subject, fact.predicate, fact.object, fact.valid_from,
                    fact.valid_until, fact.superseded_by, fact.supersedes,
                    serde_json::to_string(&fact.contradicted_by).unwrap_or("[]".into()),
                    fact.created_at, fact.source,
                ],
            )
            .map_err(|e| format!("insert fact: {e}"))?;
        Ok(())
    }
}

fn row_to_fact(row: &rusqlite::Row) -> rusqlite::Result<TemporalFact> {
    let contradicted_by: String = row.get(8)?;
    Ok(TemporalFact {
        id: row.get(0)?,
        subject: row.get(1)?,
        predicate: row.get(2)?,
        object: row.get(3)?,
        valid_from: row.get(4)?,
        valid_until: row.get(5)?,
        superseded_by: row.get(6)?,
        supersedes: row.get(7)?,
        contradicted_by: serde_json::from_str(&contradicted_by).unwrap_or_default(),
        created_at: row.get(9)?,
        source: row.get(10)?,
    })
}

fn version_seq(id: &str, conn: &Connection) -> i64 {
    conn.query_row(
        "SELECT COUNT(*) FROM temporal_facts WHERE id LIKE ?1",
        params![format!("{id}-v%")],
        |r| r.get::<_, i64>(0),
    )
    .unwrap_or(0)
}

fn uuid6() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    format!("{:x}{:x}", now_ts(), nanos)
}

#[cfg(test)]
mod tests {
    use super::*;

fn ledger() -> TemporalFactLedger {
        TemporalFactLedger::open(Some(std::path::Path::new(":memory:"))).unwrap()
    }

    #[test]
    fn test_add_fact_and_point_in_time_query() {
        let lg = ledger();
        let f = lg.add_fact("rust", "borrow_checker", "enforces_lifetimes", Some(100), None, "test").unwrap();
        assert!(f.valid_until.is_none());
        let at = lg.query_valid_at(500).unwrap();
        assert_eq!(at.len(), 1);
        assert_eq!(at[0].id, f.id);
        // 未来时刻应不含此事实
        let later = lg.add_fact("rust", "edition", "2021", Some(100), Some(300), "test").unwrap();
        assert!(later.valid_until.is_some());
        let at_late = lg.query_valid_at(400).unwrap();
        assert_eq!(at_late.len(), 1);
        assert_eq!(at_late[0].id, f.id);
    }

    #[test]
    fn test_supersede_is_append_only() {
        let lg = ledger();
        let old = lg.add_fact("kb", "search_engine", "bm25", Some(100), None, "test").unwrap();
        let new = lg.supersede(&old.id, "hybrid", Some(200), "corrected").unwrap();
        assert_eq!(new.supersedes.as_deref(), Some(old.id.as_str()));
        // 旧版被截断: 200 时刻新版本有效, 旧版无效
        let at = lg.query_valid_at(150).unwrap();
        assert_eq!(at.len(), 1);
        assert_eq!(at[0].object, "bm25");
        let at2 = lg.query_valid_at(250).unwrap();
        assert_eq!(at2.len(), 1);
        assert_eq!(at2[0].object, "hybrid");
        // 链: 新 → 旧
        let chain = lg.history_chain(&new.id).unwrap();
        assert_eq!(chain.len(), 2);
        assert_eq!(chain[0].id, new.id);
        assert_eq!(chain[1].id, old.id);
    }

    #[test]
    fn test_conflict_rejects_duplicate_active_fact() {
        let lg = ledger();
        let _ = lg.add_fact("s", "p", "o1", Some(100), None, "test").unwrap();
        let duplicate = lg.add_fact("s", "p", "o1", Some(200), None, "test");
        assert!(duplicate.is_err(), "同 subject+predicate+object 活动事实应拒绝");
        // 不同 object (真矛盾) 允许并存
        let different = lg.add_fact("s", "p", "o2", Some(200), None, "test");
        assert!(different.is_ok(), "矛盾事实应允许显式记录");
    }

    #[test]
    fn test_contradiction_edges_bidirectional() {
        let lg = ledger();
        let a = lg.add_fact("x", "temperature", "hot", Some(100), None, "test").unwrap();
        let b = lg.add_fact("x", "temperature", "cold", Some(100), None, "test").unwrap();
        lg.record_contradiction(&a.id, &b.id).unwrap();
        let a2 = lg.get_fact(&a.id).unwrap().unwrap();
        let b2 = lg.get_fact(&b.id).unwrap().unwrap();
        assert!(a2.contradicted_by.contains(&b.id));
        assert!(b2.contradicted_by.contains(&a.id));
    }
}
