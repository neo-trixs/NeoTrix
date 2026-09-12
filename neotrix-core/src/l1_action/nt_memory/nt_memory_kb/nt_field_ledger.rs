//! # G1 场化地基 — FieldLedger: 版本化场写入路径 (灵境协议 6 工程转译)
//!
//! 吸收来源: Agent Infra 钱学森灵境引擎参赛方案 协议 1/2/6 (2026-08-26, commit 68bee58c 同期)。
//! R-P42 接线: 本模块强化 nt_memory_kb 现有节点, 不建平行存储。
//!
//! 三条纪律的落点:
//! - **统一事实源**: 所有写者先 `field_stage` 暂存, 由任意一方 `field_tick` 统一求解下一版本;
//!   无点对点消息, 无锁仲裁 (SQLite BEGIN IMMEDIATE 提供全序)。
//! - **确定性共识**: 冲突消解是**可交换合并** (同键取字典序最大值), 最终状态只依赖 staged
//!   多重集, 与 tick 划分 / 提交顺序无关 —— 把协议 6 的"加法交换律"从断言变成构造保证。
//! - **审计回放**: `field_journal` 哈希链 sha256(prev_hash ‖ version ‖ canonical_entries),
//!   `field_verify_chain` 整链重算校验, 篡改必被发现。

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// 单次 tick 最多求解的暂存条目数 (防无界事务)。
const STAGING_DRAIN_CAP: i64 = 10_000;
const BUSY_TIMEOUT_MS: u64 = 5_000;
const GENESIS_HASH: &str = "GENESIS";

/// 一条已提交的场写入 (审计/重放单元)。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct FieldEntry {
    pub ns: String,
    pub key: String,
    pub value: String,
    pub writer: String,
}

/// 一次 field_tick 的回执。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldReceipt {
    pub version: u64,
    pub hash: String,
    /// 参与合并的条目数
    pub drained: usize,
    /// 实际改写 kv_store 的条目数 (其余被可交换合并淘汰)
    pub applied: usize,
}

fn now_nanos() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as i64
}

/// 建表 (幂等)。kv_store 本体由 nt_memory_schema::initialize 负责。
pub fn ensure_tables(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS field_staging (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            ns TEXT NOT NULL, key TEXT NOT NULL, value TEXT NOT NULL,
            writer TEXT NOT NULL, ts INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS field_journal (
            version INTEGER PRIMARY KEY,
            entries_json TEXT NOT NULL, hash TEXT NOT NULL, ts INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS field_head (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            version INTEGER NOT NULL, hash TEXT NOT NULL
        );",
    )
    .map_err(|e| format!("field ledger ensure_tables: {}", e))
}

/// 规范序: (ns, key, value, writer) 字典序。同一多重集 ⇒ 同一序列。
fn canonical_sort(entries: &mut [FieldEntry]) {
    entries.sort_by(|a, b| {
        a.ns.cmp(&b.ns)
            .then(a.key.cmp(&b.key))
            .then(a.value.cmp(&b.value))
            .then(a.writer.cmp(&b.writer))
    });
}

fn hash_chain(prev_hash: &str, version: u64, entries: &[FieldEntry]) -> String {
    let mut h = Sha256::new();
    h.update(prev_hash.as_bytes());
    h.update(version.to_be_bytes());
    for e in entries {
        h.update(e.ns.as_bytes());
        h.update([0x1F]);
        h.update(e.key.as_bytes());
        h.update([0x1F]);
        h.update(e.value.as_bytes());
        h.update([0x1F]);
        h.update(e.writer.as_bytes());
        h.update([0x1E]);
    }
    format!("{:x}", h.finalize())
}

/// 写入者投递一条 ΔJ (不入正式状态, 等 tick 统一求解)。
pub fn field_stage(
    conn: &Connection,
    ns: &str,
    key: &str,
    value: &str,
    writer: &str,
) -> Result<i64, String> {
    let _ = conn.busy_timeout(std::time::Duration::from_millis(BUSY_TIMEOUT_MS));
    ensure_tables(conn)?;
    conn.execute(
        "INSERT INTO field_staging (ns, key, value, writer, ts) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![ns, key, value, writer, now_nanos()],
    )
    .map_err(|e| format!("field_stage: {}", e))?;
    Ok(conn.last_insert_rowid())
}

fn read_head(conn: &Connection) -> Result<(u64, String), String> {
    conn.query_row(
        "SELECT version, hash FROM field_head WHERE id = 1",
        [],
        |r| Ok((r.get::<_, u64>(0)?, r.get::<_, String>(1)?)),
    )
    .optional()
    .map_err(|e| format!("read_head: {}", e))
    .map(|opt| opt.unwrap_or((0, GENESIS_HASH.to_string())))
}

/// 统一求解: 收集全部暂存 → 规范序 → 可交换合并 → 版本+1 → 哈希链落账。
/// 任何写者都可调用; 并发调用由 BEGIN IMMEDIATE 全序化, 空集返回 None (幂等)。
pub fn field_tick(conn: &Connection) -> Result<Option<FieldReceipt>, String> {
    let _ = conn.busy_timeout(std::time::Duration::from_millis(BUSY_TIMEOUT_MS));
    ensure_tables(conn)?;

    conn.execute_batch("BEGIN IMMEDIATE")
        .map_err(|e| format!("field_tick begin: {}", e))?;
    let result = (|| -> Result<Option<FieldReceipt>, String> {
        let mut stmt = conn
            .prepare("SELECT id, ns, key, value, writer FROM field_staging ORDER BY id LIMIT ?1")
            .map_err(|e| e.to_string())?;
        let mut rows = stmt
            .query_map(params![STAGING_DRAIN_CAP], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    FieldEntry {
                        ns: r.get(1)?,
                        key: r.get(2)?,
                        value: r.get(3)?,
                        writer: r.get(4)?,
                    },
                ))
            })
            .map_err(|e| e.to_string())?;
        let mut staged: Vec<(i64, FieldEntry)> = Vec::new();
        for row in rows.by_ref() {
            staged.push(row.map_err(|e| e.to_string())?);
        }
        drop(rows);
        drop(stmt);
        if staged.is_empty() {
            return Ok(None);
        }
        let mut entries: Vec<FieldEntry> = staged.iter().map(|(_, e)| e.clone()).collect();
        canonical_sort(&mut entries);

        // 可交换合并: 仅当新值字典序更大才覆写 (与分区无关的确定性冲突消解)
        let mut applied = 0usize;
        for e in &entries {
            let changed = conn
                .execute(
                    "INSERT INTO kv_store (namespace, key, value, updated_at) \
                     VALUES (?1, ?2, ?3, ?4) \
                     ON CONFLICT(namespace, key) DO UPDATE \
                     SET value = excluded.value, updated_at = excluded.updated_at \
                     WHERE excluded.value > kv_store.value",
                    params![e.ns, e.key, e.value, now_nanos()],
                )
                .map_err(|err| format!("merge {}: {}", e.key, err))?;
            applied += changed;
        }

        let (prev_version, prev_hash) = read_head(conn)?;
        let version = prev_version + 1;
        let hash = hash_chain(&prev_hash, version, &entries);
        let entries_json =
            serde_json::to_string(&entries).map_err(|e| format!("serialize: {}", e))?;
        conn.execute(
            "INSERT INTO field_journal (version, entries_json, hash, ts) VALUES (?1, ?2, ?3, ?4)",
            params![version, entries_json, hash, now_nanos()],
        )
        .map_err(|e| format!("journal insert: {}", e))?;
        conn.execute(
            "INSERT INTO field_head (id, version, hash) VALUES (1, ?1, ?2) \
             ON CONFLICT(id) DO UPDATE SET version = excluded.version, hash = excluded.hash",
            params![version, hash],
        )
        .map_err(|e| format!("head upsert: {}", e))?;

        let ids: Vec<String> = staged.iter().map(|(id, _)| id.to_string()).collect();
        conn.execute(
            &format!(
                "DELETE FROM field_staging WHERE id IN ({})",
                ids.join(",")
            ),
            [],
        )
        .map_err(|e| format!("staging drain: {}", e))?;

        Ok(Some(FieldReceipt {
            version,
            hash,
            drained: entries.len(),
            applied,
        }))
    })();

    match result {
        Ok(r) => {
            conn.execute_batch("COMMIT").map_err(|e| format!("commit: {}", e))?;
            Ok(r)
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK");
            Err(e)
        }
    }
}

/// 当前场版本号。
pub fn field_version(conn: &Connection) -> Result<u64, String> {
    let _ = conn.busy_timeout(std::time::Duration::from_millis(BUSY_TIMEOUT_MS));
    ensure_tables(conn)?;
    Ok(read_head(conn)?.0)
}

/// 当前头哈希。
pub(crate) fn field_head_hash(conn: &Connection) -> Result<String, String> {
    let _ = conn.busy_timeout(std::time::Duration::from_millis(BUSY_TIMEOUT_MS));
    ensure_tables(conn)?;
    Ok(read_head(conn)?.1)
}

/// G3 感知侧 (协议4-for-field): 读取版本链上 after_version 之后的事实增量。
/// 其他写者经 tick 落下的源项条目由此进入任何 agent 的感知回路 ——
/// 写回即事实, 共识靠读同一版本链, 无需消息传递。
pub fn field_journal_since(
    conn: &Connection,
    after_version: u64,
    cap: usize,
) -> Result<Vec<(u64, Vec<FieldEntry>)>, String> {
    let _ = conn.busy_timeout(std::time::Duration::from_millis(BUSY_TIMEOUT_MS));
    ensure_tables(conn)?;
    let mut stmt = conn
        .prepare("SELECT version, entries_json FROM field_journal WHERE version > ?1 ORDER BY version LIMIT ?2")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![after_version, cap as i64], |r| {
            Ok((r.get::<_, u64>(0)?, r.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for row in rows {
        let (version, entries_json) = row.map_err(|e| e.to_string())?;
        let entries: Vec<FieldEntry> =
            serde_json::from_str(&entries_json).map_err(|e| format!("parse v{}: {}", version, e))?;
        out.push((version, entries));
    }
    Ok(out)
}

/// 整链回放校验: 从创世逐版本重算哈希, 任何篡改返回 false。
pub fn field_verify_chain(conn: &Connection) -> Result<bool, String> {
    let _ = conn.busy_timeout(std::time::Duration::from_millis(BUSY_TIMEOUT_MS));
    ensure_tables(conn)?;
    let mut stmt = conn
        .prepare("SELECT version, entries_json, hash FROM field_journal ORDER BY version")
        .map_err(|e| e.to_string())?;
    let mut rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, u64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    let mut prev_hash = GENESIS_HASH.to_string();
    let mut expect_version: u64 = 1;
    while let Some(row) = rows.next() {
        let (version, entries_json, hash) = row.map_err(|e| e.to_string())?;
        if version != expect_version {
            return Ok(false); // 版本断裂
        }
        let entries: Vec<FieldEntry> =
            serde_json::from_str(&entries_json).map_err(|e| e.to_string())?;
        let recomputed = hash_chain(&prev_hash, version, &entries);
        if recomputed != hash {
            return Ok(false); // 哈希不连续 = 篡改
        }
        prev_hash = hash;
        expect_version += 1;
    }
    let (head_version, head_hash) = read_head(conn)?;
    Ok(head_version == expect_version.saturating_sub(1) && (head_version == 0 || head_hash == prev_hash))
}

/// G4 多锚点共识 (灵境协议6 收尾): 聚合每个 writer 已参与求解的最高版本游标。
/// 按版本序升序扫描链上条目, 后见覆盖先见 ⇒ 终值为 writer 出现的最高版本;
/// 从未入链的写者不出现。返回按 writer 字典序稳定排列。
pub fn field_writer_cursors(conn: &Connection) -> Result<Vec<(String, u64)>, String> {
    let _ = conn.busy_timeout(std::time::Duration::from_millis(BUSY_TIMEOUT_MS));
    ensure_tables(conn)?;
    let mut stmt = conn
        .prepare("SELECT version, entries_json FROM field_journal ORDER BY version")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| Ok((r.get::<_, u64>(0)?, r.get::<_, String>(1)?)))
        .map_err(|e| e.to_string())?;
    let mut cursors: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
    for row in rows {
        let (version, entries_json) = row.map_err(|e| e.to_string())?;
        let entries: Vec<FieldEntry> = serde_json::from_str(&entries_json)
            .map_err(|e| format!("parse v{}: {}", version, e))?;
        for e in entries {
            cursors.insert(e.writer, version);
        }
    }
    Ok(cursors.into_iter().collect())
}

/// 多锚点共识帧: 头版本 + 全体锚点的最小安全对齐点 (quorum) + 各写者滞后量。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusFrame {
    /// 当前场头版本号
    pub head: u64,
    /// 所有锚点 cursor 的最小值 (无锚点时 = head); 追赶至此即全员对齐
    pub quorum: u64,
    /// (writer, cursor, lag = head - cursor), 按 writer 字典序
    pub anchors: Vec<(String, u64, i64)>,
}

/// 求解当前共识帧: head 来自 field_head, 锚点游标来自 journal 聚合。
pub fn consensus_frame(conn: &Connection) -> Result<ConsensusFrame, String> {
    let head = field_version(conn)?;
    let mut quorum: Option<u64> = None;
    let mut anchors = Vec::new();
    for (writer, cursor) in field_writer_cursors(conn)? {
        let lag = head as i64 - cursor as i64;
        quorum = Some(quorum.unwrap_or(cursor).min(cursor));
        anchors.push((writer, cursor, lag));
    }
    Ok(ConsensusFrame {
        head,
        quorum: quorum.unwrap_or(head),
        anchors,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase;
    use std::sync::Arc;

    fn temp_kb(tag: &str) -> KnowledgeBase {
        let path = std::env::temp_dir().join(format!(
            "g1_field_{}_{}_{}.db",
            tag,
            std::process::id(),
            now_nanos()
        ));
        KnowledgeBase::open(Some(path)).expect("open temp kb")
    }

    #[test]
    fn test_stage_tick_basic() {
        let kb = temp_kb("basic");
        assert_eq!(kb.field_version().unwrap(), 0);
        kb.field_stage("g1", "door", "closed", "w1").unwrap();
        let r = kb.field_tick().unwrap().expect("receipt");
        assert_eq!(r.version, 1);
        assert_eq!(r.drained, 1);
        assert_eq!(kb.field_version().unwrap(), 1);
        assert_eq!(
            kb.kv_get("g1", "door").unwrap().as_deref(),
            Some("closed")
        );
        // 空集 tick 幂等
        assert!(kb.field_tick().unwrap().is_none());
        assert!(kb.field_verify_chain().unwrap());
    }

    #[test]
    fn test_commutative_merge_partition_independent() {
        // 同一多重集 {k: low, high} 的两种 tick 划分 → 最终状态必须一致 (协议6 顺序无关)
        let values = ["aaa_low", "zzz_high"];

        let kb_a = temp_kb("partA");
        for v in values {
            kb_a.field_stage("g1", "state", v, "w").unwrap();
        }
        kb_a.field_tick().unwrap();

        let kb_b = temp_kb("partB");
        for v in values.iter().rev() {
            kb_b.field_stage("g1", "state", v, "w").unwrap();
            kb_b.field_tick().unwrap(); // 每条单独一 tick, 且顺序反转
        }

        let va = kb_a.kv_get("g1", "state").unwrap();
        let vb = kb_b.kv_get("g1", "state").unwrap();
        assert_eq!(va, vb, "最终状态与 tick 划分无关");
        assert_eq!(va.as_deref(), Some("zzz_high"), "可交换合并取最大值");
    }

    #[test]
    fn test_tamper_detection() {
        let kb = temp_kb("tamper");
        kb.field_stage("g1", "a", "1", "w").unwrap();
        kb.field_tick().unwrap();
        kb.field_stage("g1", "b", "2", "w").unwrap();
        kb.field_tick().unwrap();
        assert!(kb.field_verify_chain().unwrap());

        {
            let conn = kb.raw_conn().unwrap();
            conn.execute(
                "UPDATE field_journal SET hash = 'deadbeef' WHERE version = 1",
                [],
            )
            .unwrap();
        }
        assert!(!kb.field_verify_chain().unwrap(), "篡改必须被发现");
    }

    #[test]
    fn test_three_writers_concurrent_consistency() {
        let kb0 = temp_kb("conc");
        let db_path = kb0.db_path.clone();
        drop(kb0);

        const KEYS: usize = 5;
        const PER_WRITER: usize = 25;

        let handles: Vec<_> = (0..3)
            .map(|wid| {
                let path = db_path.clone();
                std::thread::spawn(move || {
                    let kb = KnowledgeBase::open(Some(path)).expect("writer open");
                    let writer = format!("w{}", wid);
                    for i in 0..PER_WRITER {
                        let key = format!("k{}", i % KEYS);
                        let value = format!("{:0>4}_{}", i, writer);
                        kb.field_stage("g1", &key, &value, &writer).unwrap();
                        if i % 8 == 7 {
                            kb.field_tick().unwrap(); // 中途机会性求解
                        }
                        std::thread::sleep(std::time::Duration::from_micros(
                            (wid * 37 + i * 11) as u64 % 500,
                        ));
                    }
                    kb.field_tick().unwrap();
                })
            })
            .collect();
        for h in handles {
            h.join().expect("writer thread");
        }

        let verifier = KnowledgeBase::open(Some(db_path)).expect("verifier open");
        verifier.field_tick().unwrap(); // 兜底清空暂存

        // 期望态: 每键取全部 staged 值的字典序最大 (可交换合并的定义)
        for k in 0..KEYS {
            let key = format!("k{}", k);
            let mut candidates: Vec<String> = Vec::new();
            for wid in 0..3 {
                for i in 0..PER_WRITER {
                    if i % KEYS == k {
                        candidates.push(format!("{:0>4}_w{}", i, wid));
                    }
                }
            }
            candidates.sort();
            let expected = candidates.last().unwrap().clone();
            assert_eq!(
                verifier.kv_get("g1", &key).unwrap().as_deref(),
                Some(expected.as_str()),
                "key {} 最终值必须等于多重集最大值",
                key
            );
        }
        assert!(
            verifier.field_verify_chain().unwrap(),
            "并发写后哈希链必须完整"
        );
        assert!(verifier.field_version().unwrap() >= 1);
    }

    #[test]
    fn test_shared_arc_kb_multithread_staging() {
        let kb = Arc::new(temp_kb("arc"));
        let handles: Vec<_> = (0..4)
            .map(|wid| {
                let kb = Arc::clone(&kb);
                std::thread::spawn(move || {
                    for i in 0..10 {
                        kb.field_stage("g1", &format!("key{}", i), &format!("v{}_{}", i, wid), &format!("w{}", wid))
                            .unwrap();
                    }
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
        let r = kb.field_tick().unwrap().unwrap();
        assert_eq!(r.drained, 40);
        assert!(kb.field_verify_chain().unwrap());
    }

    #[test]
    fn test_journal_since_cursor_semantics() {
        let kb = temp_kb("since");
        for i in 0..3 {
            kb.field_stage("g1", &format!("k{}", i), &i.to_string(), "w").unwrap();
            kb.field_tick().unwrap();
        }
        {
            let conn = kb.raw_conn().unwrap();
            let all = field_journal_since(&conn, 0, 10).unwrap();
            assert_eq!(all.len(), 3, "从创世读全量");
            let tail = field_journal_since(&conn, 1, 10).unwrap();
            assert_eq!(tail.len(), 2);
            assert_eq!(tail[0].0, 2, "游标后第一条是 v2");
            let capped = field_journal_since(&conn, 0, 2).unwrap();
            assert_eq!(capped.len(), 2, "cap 生效");
        }
    }

    #[test]
    fn test_writer_cursors_and_consensus_frame() {
        let kb = temp_kb("consensus");
        // 空库: 无版本无锚点 → quorum 兜底为 head (0)
        {
            let conn = kb.raw_conn().unwrap();
            assert!(field_writer_cursors(&conn).unwrap().is_empty());
            let f = consensus_frame(&conn).unwrap();
            assert_eq!(f.head, 0);
            assert_eq!(f.quorum, 0);
            assert!(f.anchors.is_empty());
        }
        // 3 写者不同节奏入场: v1=w1, v2=w2, v3=w1+w3
        kb.field_stage("g4", "k1", "v", "w1").unwrap();
        kb.field_tick().unwrap();
        kb.field_stage("g4", "k2", "v", "w2").unwrap();
        kb.field_tick().unwrap();
        kb.field_stage("g4", "k3", "va", "w1").unwrap();
        kb.field_stage("g4", "k4", "vb", "w3").unwrap();
        kb.field_tick().unwrap();

        {
            let conn = kb.raw_conn().unwrap();
            let cursors = field_writer_cursors(&conn).unwrap();
            assert_eq!(
                cursors,
                vec![
                    ("w1".to_string(), 3u64),
                    ("w2".to_string(), 2),
                    ("w3".to_string(), 3)
                ],
                "cursor = writer 参与求解的最高版本 (w1 复出于 v3)"
            );
            let f = consensus_frame(&conn).unwrap();
            assert_eq!(f.head, 3);
            assert_eq!(f.quorum, 2, "quorum = 全体锚点 cursor 最小值 (停在 v2 的 w2)");
            assert_eq!(f.anchors.len(), 3);
            assert_eq!(f.anchors[0], ("w1".to_string(), 3, 0));
            assert_eq!(f.anchors[1], ("w2".to_string(), 2, 1));
            assert_eq!(f.anchors[2], ("w3".to_string(), 3, 0));
        }
    }

    #[test]
    fn test_consensus_frame_after_writer_catch_up() {
        // 落后写者追平后 quorum 必须单调抬升 (安全对齐点只进不退)
        let kb = temp_kb("catchup");
        kb.field_stage("g4", "a", "1", "slow").unwrap();
        kb.field_tick().unwrap(); // slow cursor = 1
        for i in 0..2 {
            kb.field_stage("g4", &format!("b{}", i), "x", "fast").unwrap();
            kb.field_tick().unwrap(); // fast cursor = 3
        }
        {
            let conn = kb.raw_conn().unwrap();
            let f = consensus_frame(&conn).unwrap();
            assert_eq!(f.head, 3);
            assert_eq!(f.quorum, 1);
        }
        kb.field_stage("g4", "a2", "2", "slow").unwrap();
        kb.field_tick().unwrap(); // slow 追至 v4
        let f = consensus_frame(&kb.raw_conn().unwrap()).unwrap();
        assert_eq!(f.head, 4);
        assert_eq!(f.quorum, 3, "fast 未动, quorum 随 slow 抬升");
    }
}
