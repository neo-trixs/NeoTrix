//! G23 时序图审计 (opencontext 吸收) — NT-SHIELD 审计强化。
//!
//! 为安全/治理审计记录提供时序上下文: 每条审计记录带
//! `valid_from`/`valid_until` + supersession 链 (取代关系), 并用
//! HMAC-SHA256 签名保证完整性与防注入 (与 nt_memory_provenance 同族)。

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// 单条时序审计记录。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TemporalAuditRecord {
    pub id: String,
    /// 审计主体 (entity / agent)。
    pub subject: String,
    /// 审计动作 (如 "config_change" / "credential_access" / "proxy_rotation")。
    pub action: String,
    /// 动作详情 (参数摘要)。
    pub detail: String,
    /// 审计结果 (pass / fail / warn)。
    pub verdict: String,
    /// 生效开始时间戳 (unix 秒)。
    pub valid_from: i64,
    /// 失效时间戳 (None = 当前版本仍有效)。
    pub valid_until: Option<i64>,
    /// 被哪个新版本取代 (supersession 指针)。
    pub superseded_by: Option<String>,
    /// 取代了哪个旧版本。
    pub supersedes: Option<String>,
    /// HMAC-SHA256 签名 (十六进制)。
    pub signature: String,
}

impl TemporalAuditRecord {
    pub fn new(subject: impl Into<String>, action: impl Into<String>, detail: impl Into<String>, verdict: impl Into<String>) -> Self {
        Self {
            id: format!("audit-{}", uuid::Uuid::new_v4()),
            subject: subject.into(),
            action: action.into(),
            detail: detail.into(),
            verdict: verdict.into(),
            valid_from: now_unix(),
            valid_until: None,
            superseded_by: None,
            supersedes: None,
            signature: String::new(),
        }
    }

    /// 对记录内容签名 (signature 字段不参与摘要)。
    pub fn sign(&mut self, key: &[u8]) {
        self.signature = sign_audit(key, self);
    }

    /// 校验签名 (空签名 / 失配均无效)。
    pub fn verify(&self, key: &[u8]) -> bool {
        if self.signature.is_empty() {
            return false;
        }
        let expected = sign_audit(key, self);
        constant_time_eq_hex(&self.signature, &expected)
    }
}

/// 时序审计账本 — SQLite 持久化, 支持 supersession 链与时序查询。
pub struct TemporalAuditLedger {
    conn: rusqlite::Connection,
}

const CREATE_SQL: &str = "CREATE TABLE IF NOT EXISTS temporal_audit (
    id TEXT PRIMARY KEY,
    subject TEXT NOT NULL,
    action TEXT NOT NULL,
    detail TEXT NOT NULL,
    verdict TEXT NOT NULL,
    valid_from INTEGER NOT NULL,
    valid_until INTEGER,
    superseded_by TEXT,
    supersedes TEXT,
    signature TEXT NOT NULL DEFAULT ''
)";

impl TemporalAuditLedger {
    pub fn open(db_path: Option<&std::path::Path>) -> Result<Self, String> {
        let path = db_path.map(|p| p.to_path_buf()).unwrap_or_else(|| {
            std::env::temp_dir().join("neotrix-temporal-audit.db")
        });
        let conn = rusqlite::Connection::open(&path)
            .map_err(|e| format!("temporal audit open {}: {}", path.display(), e))?;
        conn.execute_batch(CREATE_SQL)
            .map_err(|e| format!("temporal audit schema: {}", e))?;
        Ok(Self { conn })
    }

    /// 追加一条审计记录 (签名后可落盘)。
    pub fn append(&self, rec: &TemporalAuditRecord) -> Result<(), String> {
        let valid_until = rec.valid_until.unwrap_or(0);
        self.conn
            .execute(
                "INSERT OR REPLACE INTO temporal_audit
                 (id, subject, action, detail, verdict, valid_from, valid_until, superseded_by, supersedes, signature)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                rusqlite::params![
                    rec.id, rec.subject, rec.action, rec.detail, rec.verdict,
                    rec.valid_from, valid_until, rec.superseded_by, rec.supersedes, rec.signature
                ],
            )
            .map_err(|e| format!("temporal audit append: {}", e))?;
        Ok(())
    }

    /// 使某记录失效并新开一条取代它的记录 (supersession 链)。
    /// 返回新记录 (未签名, 调用方需 sign 后 append)。
    pub fn supersede(&self, old_id: &str, key: &[u8], detail: impl Into<String>) -> Result<TemporalAuditRecord, String> {
        let old = self.get(old_id)?.ok_or_else(|| format!("supersede: no such record {}", old_id))?;
        // 校验旧记录签名, 防止篡改链
        if !old.verify(key) {
            return Err(format!("supersede: old record {} signature invalid", old_id));
        }
        let mut next = TemporalAuditRecord::new(old.subject, old.action, detail, old.verdict);
        next.valid_from = now_unix();
        next.supersedes = Some(old.id.clone());
        self.conn
            .execute(
                "UPDATE temporal_audit SET valid_until = ?1, superseded_by = ?2 WHERE id = ?3",
                rusqlite::params![next.valid_from, next.id, old_id],
            )
            .map_err(|e| format!("supersede close old: {}", e))?;
        Ok(next)
    }

    /// 查询单个记录。
    pub fn get(&self, id: &str) -> Result<Option<TemporalAuditRecord>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, subject, action, detail, verdict, valid_from, valid_until, superseded_by, supersedes, signature FROM temporal_audit WHERE id = ?1")
            .map_err(|e| format!("get prepare: {}", e))?;
        let mut rows = stmt
            .query_map([id], |row| {
                let valid_until: i64 = row.get(6)?;
                Ok(TemporalAuditRecord {
                    id: row.get(0)?,
                    subject: row.get(1)?,
                    action: row.get(2)?,
                    detail: row.get(3)?,
                    verdict: row.get(4)?,
                    valid_from: row.get(5)?,
                    valid_until: if valid_until == 0 { None } else { Some(valid_until) },
                    superseded_by: row.get(7)?,
                    supersedes: row.get(8)?,
                    signature: row.get(9)?,
                })
            })
            .map_err(|e| format!("get query: {}", e))?;
        match rows.next() {
            Some(Ok(r)) => Ok(Some(r)),
            Some(Err(e)) => Err(format!("get row: {}", e)),
            None => Ok(None),
        }
    }

    /// 查询在某时间戳有效的记录 (valid_from <= ts < valid_until)。
    pub fn query_valid_at(&self, ts: i64) -> Result<Vec<TemporalAuditRecord>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, subject, action, detail, verdict, valid_from, valid_until, superseded_by, supersedes, signature
                 FROM temporal_audit WHERE valid_from <= ?1 AND (valid_until = 0 OR valid_until > ?1)",
            )
            .map_err(|e| format!("valid_at prepare: {}", e))?;
        let mut out = Vec::new();
        let rows = stmt
            .query_map([ts], |row| {
                let valid_until: i64 = row.get(6)?;
                Ok(TemporalAuditRecord {
                    id: row.get(0)?,
                    subject: row.get(1)?,
                    action: row.get(2)?,
                    detail: row.get(3)?,
                    verdict: row.get(4)?,
                    valid_from: row.get(5)?,
                    valid_until: if valid_until == 0 { None } else { Some(valid_until) },
                    superseded_by: row.get(7)?,
                    supersedes: row.get(8)?,
                    signature: row.get(9)?,
                })
            })
            .map_err(|e| format!("valid_at query: {}", e))?;
        for r in rows {
            out.push(r.map_err(|e| format!("valid_at row: {}", e))?);
        }
        Ok(out)
    }

    /// 按主体查询审计链 (未失效记录优先)。
    pub fn by_subject(&self, subject: &str) -> Result<Vec<TemporalAuditRecord>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, subject, action, detail, verdict, valid_from, valid_until, superseded_by, supersedes, signature
                 FROM temporal_audit WHERE subject = ?1 ORDER BY valid_from DESC",
            )
            .map_err(|e| format!("subject prepare: {}", e))?;
        let mut out = Vec::new();
        let rows = stmt
            .query_map([subject], |row| {
                let valid_until: i64 = row.get(6)?;
                Ok(TemporalAuditRecord {
                    id: row.get(0)?,
                    subject: row.get(1)?,
                    action: row.get(2)?,
                    detail: row.get(3)?,
                    verdict: row.get(4)?,
                    valid_from: row.get(5)?,
                    valid_until: if valid_until == 0 { None } else { Some(valid_until) },
                    superseded_by: row.get(7)?,
                    supersedes: row.get(8)?,
                    signature: row.get(9)?,
                })
            })
            .map_err(|e| format!("subject query: {}", e))?;
        for r in rows {
            out.push(r.map_err(|e| format!("subject row: {}", e))?);
        }
        Ok(out)
    }

    /// 校验记录是否被篡改 (签名无效 / 无签名)。
    pub fn verify_record(&self, id: &str, key: &[u8]) -> Result<bool, String> {
        let rec = self.get(id)?.ok_or_else(|| format!("verify: no such record {}", id))?;
        Ok(rec.verify(key))
    }

    /// 审计统计: (总记录, 当前有效数, 失效数)。
    pub fn stats(&self) -> Result<(usize, usize, usize), String> {
        let total: usize = self
            .conn
            .query_row("SELECT COUNT(*) FROM temporal_audit", [], |r| r.get(0))
            .map_err(|e| format!("count: {}", e))?;
        let active: usize = self
            .conn
            .query_row("SELECT COUNT(*) FROM temporal_audit WHERE valid_until = 0", [], |r| r.get(0))
            .map_err(|e| format!("count active: {}", e))?;
        Ok((total, active, total.saturating_sub(active)))
    }
}

/// 计算审计记录 HMAC (字段有序拼接)。
fn sign_audit(key: &[u8], rec: &TemporalAuditRecord) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts any key");
    mac.update(rec.id.as_bytes());
    mac.update([0u8; 1].as_slice());
    mac.update(rec.subject.as_bytes());
    mac.update([0u8; 1].as_slice());
    mac.update(rec.action.as_bytes());
    mac.update([0u8; 1].as_slice());
    mac.update(rec.detail.as_bytes());
    mac.update([0u8; 1].as_slice());
    mac.update(rec.verdict.as_bytes());
    mac.update([0u8; 1].as_slice());
    mac.update(rec.valid_from.to_le_bytes().as_slice());
    mac.update([0u8; 1].as_slice());
    if let Some(until) = rec.valid_until {
        mac.update(until.to_le_bytes().as_slice());
    }
    mac.update([0u8; 1].as_slice());
    if let Some(sb) = &rec.superseded_by {
        mac.update(sb.as_bytes());
    }
    mac.update([0u8; 1].as_slice());
    if let Some(sp) = &rec.supersedes {
        mac.update(sp.as_bytes());
    }
    mac.finalize().into_bytes().iter().map(|b| format!("{:02x}", b)).collect()
}

/// 常量时间十六进制比较。
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

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_sign_and_verify() {
        let key = b"test-key";
        let mut rec = TemporalAuditRecord::new("nt_shield", "credential_access", "read secret", "pass");
        assert!(!rec.verify(key), "未签名不通过");
        rec.sign(key);
        assert!(rec.verify(key), "签名后通过");
        // 篡改 detail → 校验失败
        let mut forged = rec.clone();
        forged.detail = "tampered".into();
        assert!(!forged.verify(key), "篡改应检测");
    }

    #[test]
    fn test_ledger_append_and_query_valid_at() {
        let ledger = TemporalAuditLedger::open(Some(&test_db_path())).unwrap();
        let key = b"k";
        let mut rec = TemporalAuditRecord::new("nt_shield", "proxy_rotation", "rotate to pool-2", "pass");
        rec.valid_from = 1000;
        rec.sign(key);
        ledger.append(&rec).unwrap();
        let ts = 1500;
        let active = ledger.query_valid_at(ts).unwrap();
        assert!(active.iter().any(|r| r.id == rec.id), "ts 在窗口内应有效");
        let earlier = ledger.query_valid_at(999).unwrap();
        assert!(!earlier.iter().any(|r| r.id == rec.id), "ts 在窗口外应无效");
    }

    #[test]
    fn test_ledger_supersede_chain() {
        let ledger = TemporalAuditLedger::open(Some(&test_db_path())).unwrap();
        let key = b"k";
        let mut rec = TemporalAuditRecord::new("nt_shield", "config_change", "v1", "pass");
        rec.valid_from = 100;
        rec.sign(key);
        ledger.append(&rec).unwrap();
        // supersede 前旧记录签名校验
        assert!(ledger.verify_record(&rec.id, key).unwrap());
        let mut next = ledger.supersede(&rec.id, key, "v2").unwrap();
        assert_eq!(next.supersedes.as_deref(), Some(rec.id.as_str()));
        next.sign(key);
        ledger.append(&next).unwrap();
        let old = ledger.get(&rec.id).unwrap().unwrap();
        assert!(old.valid_until.is_some(), "旧记录应失效");
        assert_eq!(old.superseded_by.as_deref(), Some(next.id.as_str()));
        let chain = ledger.by_subject("nt_shield").unwrap();
        assert!(chain.len() >= 2, "审计链保留两版本");
        let (total, active, closed) = ledger.stats().unwrap();
        assert_eq!(total, 2);
        assert_eq!(active, 1, "仅最新版本有效");
        assert_eq!(closed, 1);
    }

    #[test]
    fn test_supersede_rejects_tampered_old() {
        let ledger = TemporalAuditLedger::open(Some(&test_db_path())).unwrap();
        let key = b"k";
        let mut rec = TemporalAuditRecord::new("nt_shield", "config_change", "v1", "pass");
        rec.sign(key);
        // 落盘后篡改 detail (绕过 append 直接改库不现实, 这里模拟: 用错误密钥签名的旧记录)
        let mut forged = rec.clone();
        forged.sign(b"wrong-key");
        ledger.append(&forged).unwrap();
        assert!(
            ledger.supersede(&forged.id, key, "v2").is_err(),
            "旧记录签名无效 → supersede 拒绝"
        );
    }

    fn test_db_path() -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "neotrix-temporal-audit-test-{}.db",
            uuid::Uuid::new_v4()
        ))
    }
}