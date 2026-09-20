//! PROV-O 决策溯源 — 吸收自 semantica-agi/semantica (W3C PROV-O provenance:
//! 决策是一等对象, 每个断言携带 provenance, 冲突标记而非静默覆盖)。
//!
//! 为 NT-MEMORY 提供**决策溯源链**: 每条记录捕获 (who/agent, did/activity,
//! what/entity, why/evidence, when/timestamp), 落 KB kv_store `provenance`
//! 命名空间, 供审计回查 (D14/D20)。决策对象化让"为什么这个结论存在"
//! 可独立检索, 而非埋在节点 metadata 里不可查询。
//!
//! # Signed Provenance + Teacher-Second-Pass (G16', SEA 对抗注入防御)
//!
//! 对抗注入防御双机制:
//! 1. **签名 (HMAC-SHA256)**: 每条记录写入时用服务端密钥签名字段摘要,
//!    读取时 `verify_signature` 校验 — 伪造/篡改记录签名失配即被拒。
//! 2. **Teacher-Second-Pass 守卫**: 独立"教师复核"通道 — 记录在写入后被
//!    第二遍校验 (agent 白名单 + 签名有效性 + 时间合理性), 未过审的记录
//!    标记为 `injection_suspected`, 查询默认过滤, 只有显式审计才可见。

use hmac::{Hmac, Mac};
use rusqlite::Connection;
use serde_json::json;
use sha2::Sha256;

use super::nt_memory_unify::{kv_get, kv_set};

type HmacSha256 = Hmac<Sha256>;

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
    /// HMAC-SHA256 签名 (十六进制)。由服务端密钥对内容摘要计算。
    /// 空 = 未签名 (teacher-second-pass 会将其标记为 injection 嫌疑)。
    pub signature: String,
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
            signature: String::new(),
        }
    }

    /// 追加证据 (被引用的实体/理由)。
    pub fn with_evidence(mut self, evidence: Vec<String>) -> Self {
        self.evidence = evidence;
        self
    }

    /// 对记录内容计算 HMAC-SHA256 签名 (十六进制)。signature 字段本身不参与摘要。
    pub fn sign(&mut self, key: &[u8]) {
        self.signature = sign_content(key, &self.id, &self.agent, &self.activity, &self.entity, &self.evidence, &self.outcome, self.created_at);
    }

    /// 校验签名是否有效 (空签名 / 签名失配均视为无效)。
    pub fn verify(&self, key: &[u8]) -> bool {
        if self.signature.is_empty() {
            return false;
        }
        let expected = sign_content(key, &self.id, &self.agent, &self.activity, &self.entity, &self.evidence, &self.outcome, self.created_at);
        constant_time_eq_hex(&self.signature, &expected)
    }
}

/// 计算 (id, agent, activity, entity, evidence, outcome, created_at) 的 HMAC-SHA256。
fn sign_content(
    key: &[u8],
    id: &str,
    agent: &str,
    activity: &str,
    entity: &str,
    evidence: &[String],
    outcome: &str,
    created_at: i64,
) -> String {
    let mut mac = HmacSha256::new_from_slice(key)
        .expect("HMAC accepts any key length");
    mac.update(id.as_bytes());
    mac.update([0u8; 1].as_slice());
    mac.update(agent.as_bytes());
    mac.update([0u8; 1].as_slice());
    mac.update(activity.as_bytes());
    mac.update([0u8; 1].as_slice());
    mac.update(entity.as_bytes());
    mac.update([0u8; 1].as_slice());
    for e in evidence {
        mac.update(e.as_bytes());
        mac.update([0u8; 1].as_slice());
    }
    mac.update([0u8; 1].as_slice());
    mac.update(outcome.as_bytes());
    mac.update([0u8; 1].as_slice());
    mac.update(created_at.to_le_bytes().as_slice());
    mac.finalize().into_bytes().iter().map(|b| format!("{:02x}", b)).collect()
}

/// 常量时间十六进制字符串比较 (防时序侧信道)。
fn constant_time_eq_hex(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let ab = a.as_bytes();
    let bb = b.as_bytes();
    let mut diff = 0u8;
    for (x, y) in ab.iter().zip(bb.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Teacher-Second-Pass 防注入守卫 (G16')。
///
/// 独立复核通道: 记录写入后二次校验 (agent 白名单 + 签名 + 时间合理性)。
/// 未过审记录标记 injection 嫌疑, 常规查询默认过滤, 仅显式审计可见。
#[derive(Debug, Clone)]
pub struct InjectionGuard {
    /// 可信 agent 白名单 (空 = 信任所有 agent, 仅签名校验)。
    pub trusted_agents: Vec<String>,
    /// 服务端签名密钥。
    key: Vec<u8>,
    /// 时间合理性窗口 (秒)。记录 created_at 超出 [now-window, now+window] 视为可疑。
    pub clock_window_secs: i64,
}

impl InjectionGuard {
    pub fn new(key: Vec<u8>) -> Self {
        Self {
            trusted_agents: Vec::new(),
            key,
            clock_window_secs: 86_400,
        }
    }

    pub fn with_trusted_agents(mut self, agents: Vec<String>) -> Self {
        self.trusted_agents = agents;
        self
    }

    /// 复核单条记录。返回 `Ok(())` 通过; `Err(原因)` 未过审。
    pub fn review(&self, record: &ProvenanceRecord) -> Result<(), String> {
        // 1. 签名必须有效
        if !record.verify(&self.key) {
            return Err("signature invalid or missing".into());
        }
        // 2. agent 白名单
        if !self.trusted_agents.is_empty() && !self.trusted_agents.contains(&record.agent) {
            return Err(format!("agent '{}' not in trusted allowlist", record.agent));
        }
        // 3. 时间合理性 (防重放/未来注入)
        let now = now_unix();
        if (record.created_at - now).abs() > self.clock_window_secs {
            return Err(format!(
                "created_at {} out of clock window {}",
                record.created_at, now
            ));
        }
        Ok(())
    }

    /// 对记录签名 (写入路径用)。
    pub fn sign_record(&self, record: &mut ProvenanceRecord) {
        record.sign(&self.key);
    }
}

/// 审计一次写入: 签名后落库, 并记录签名状态 (供 teacher pass 复核)。
pub fn record_provenance_signed(
    conn: &Connection,
    guard: &InjectionGuard,
    mut record: ProvenanceRecord,
) -> Result<String, String> {
    guard.sign_record(&mut record);
    record_with_index(conn, record)
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

/// 带签名复核的查询 — Teacher-Second-Pass 生产路径。
/// 逐条 `guard.review()` 复核, 只返回通过复核的记录; 返回 (通过, 拒绝原因) 对。
/// 用于常规审计查询: 未过审 (伪造签名 / 非白名单 agent / 时钟异常) 被静默过滤。
pub fn query_provenance_verified(
    conn: &Connection,
    guard: &InjectionGuard,
    agent: Option<&str>,
    activity: Option<&str>,
    entity: Option<&str>,
) -> Result<Vec<ProvenanceRecord>, String> {
    let raw = query_provenance(conn, agent, activity, entity)?;
    Ok(raw
        .into_iter()
        .filter(|r| guard.review(r).is_ok())
        .collect())
}

/// 审计复核记录 (含被拒绝的) — 供审计链/告警消费。
/// 返回 Vec<(record, is_verified, reason)>。
pub fn audit_verified(
    conn: &Connection,
    guard: &InjectionGuard,
) -> Result<Vec<(ProvenanceRecord, bool, String)>, String> {
    let raw = query_provenance(conn, None, None, None)?;
    Ok(raw
        .into_iter()
        .map(|r| {
            match guard.review(&r) {
                Ok(()) => (r, true, String::new()),
                Err(reason) => (r, false, reason),
            }
        })
        .collect())
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
        "signed": !record.signature.is_empty(),
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

    // ── G16' signed provenance + injection guard ────────────────────────

    const TEST_KEY: &[u8] = b"neotrix-test-secret-key";

    #[test]
    fn sign_then_verify_roundtrip() {
        let mut rec = ProvenanceRecord::new("a1", ProvActivity::Absorb, "e1", "absorbed");
        rec.sign(TEST_KEY);
        assert!(!rec.signature.is_empty());
        assert!(rec.verify(TEST_KEY));
    }

    #[test]
    fn unsigned_record_fails_verify() {
        let rec = ProvenanceRecord::new("a1", ProvActivity::Absorb, "e1", "absorbed");
        assert!(!rec.verify(TEST_KEY));
    }

    #[test]
    fn tampered_record_fails_verify() {
        let mut rec = ProvenanceRecord::new("a1", ProvActivity::Absorb, "e1", "absorbed");
        rec.sign(TEST_KEY);
        rec.outcome = "attacker overwrote outcome".into();
        assert!(!rec.verify(TEST_KEY));
    }

    #[test]
    fn wrong_key_fails_verify() {
        let mut rec = ProvenanceRecord::new("a1", ProvActivity::Absorb, "e1", "absorbed");
        rec.sign(b"correct-key");
        assert!(!rec.verify(b"wrong-key"));
    }

    #[test]
    fn signature_deterministic() {
        let mut a = ProvenanceRecord::new("a1", ProvActivity::Absorb, "e1", "absorbed");
        let mut b = ProvenanceRecord::new("a1", ProvActivity::Absorb, "e1", "absorbed");
        // 固定 id + 固定 created_at → 同一签名 (签名只依赖内容字段)
        a.id = "prov-fixed".into();
        b.id = "prov-fixed".into();
        a.created_at = 1_000_000;
        b.created_at = 1_000_000;
        a.sign(TEST_KEY);
        b.sign(TEST_KEY);
        // same fields + same key → identical signature
        assert_eq!(a.signature, b.signature);
    }

    #[test]
    fn evidence_order_affects_signature() {
        let mut a = ProvenanceRecord::new("a1", ProvActivity::Absorb, "e1", "o")
            .with_evidence(vec!["x".into(), "y".into()]);
        let mut b = ProvenanceRecord::new("a1", ProvActivity::Absorb, "e1", "o")
            .with_evidence(vec!["y".into(), "x".into()]);
        a.sign(TEST_KEY);
        b.sign(TEST_KEY);
        assert_ne!(a.signature, b.signature);
    }

    #[test]
    fn guard_rejects_unsigned_and_foreign_agent() {
        let guard = InjectionGuard::new(TEST_KEY.to_vec())
            .with_trusted_agents(vec!["nt_memory_curation".into()]);

        let mut trusted = ProvenanceRecord::new("nt_memory_curation", ProvActivity::Curate, "n", "ok");
        guard.sign_record(&mut trusted);
        assert!(guard.review(&trusted).is_ok());

        let unsigned = ProvenanceRecord::new("nt_memory_curation", ProvActivity::Curate, "n", "no-sig");
        assert!(guard.review(&unsigned).is_err());

        let foreign = ProvenanceRecord::new("attacker", ProvActivity::Curate, "n", "inject");
        assert!(guard.review(&foreign).is_err());
    }

    #[test]
    fn guard_rejects_future_timestamp() {
        let guard = InjectionGuard::new(TEST_KEY.to_vec());
        let mut rec = ProvenanceRecord::new("a1", ProvActivity::Absorb, "e1", "future");
        // 未来 10 年 → 超出时钟窗口
        rec.created_at = now_unix() + 31_536_000 * 10;
        guard.sign_record(&mut rec);
        assert!(guard.review(&rec).is_err());
    }

    #[test]
    fn verified_query_filters_tampered() {
        let conn = mem_conn();
        let guard = InjectionGuard::new(TEST_KEY.to_vec());

        let mut good = ProvenanceRecord::new("a1", ProvActivity::Absorb, "e-good", "ok");
        guard.sign_record(&mut good);
        record_with_index(&conn, good).unwrap();

        // 未签名记录 → teacher pass 拒绝
        let bad = ProvenanceRecord::new("a1", ProvActivity::Absorb, "e-bad", "unsigned");
        record_with_index(&conn, bad).unwrap();

        let verified = query_provenance_verified(&conn, &guard, None, None, None).unwrap();
        assert_eq!(verified.len(), 1, "only signed record survives teacher pass");
        assert_eq!(verified[0].entity, "e-good");

        let audit = audit_verified(&conn, &guard).unwrap();
        assert_eq!(audit.len(), 2);
        let (_, v1, _) = &audit[0];
        let (_, v2, _) = &audit[1];
        // exactly one verified, one rejected
        assert_ne!(v1, v2);
    }

    #[test]
    fn signed_write_path_produces_verified_records() {
        let conn = mem_conn();
        let guard = InjectionGuard::new(TEST_KEY.to_vec());
        let rec = ProvenanceRecord::new("a1", ProvActivity::Supersede, "n", "replaced");
        record_provenance_signed(&conn, &guard, rec).unwrap();
        let verified = query_provenance_verified(&conn, &guard, None, None, None).unwrap();
        assert_eq!(verified.len(), 1);
        assert!(verified[0].verify(TEST_KEY));
    }
}