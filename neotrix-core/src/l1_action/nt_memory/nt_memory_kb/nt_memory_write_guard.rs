//! KB 写门前置检查 (dbx absorb, G4) — AI 生成/agent 发起的 KB 写操作执行前
//! 确定性检查 + 证据落盘 (kv_store `write_guard` 命名空间)。
//!
//! 对齐 dbx 的 "AI 生成 SQL 执行前安全检查" 与 NeoTrix `nt_core_gate` 的
//! `ActionTier` 语义: 只读自动放行 (Tier1), 可逆写确定性检查后放行 (Tier2),
//! 删除/批量重建要求审批 (Tier3/Tier4)。守卫是纯函数 (无 I/O), 便于单元测试;
//! 证据记录是唯一副作用。

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// 守卫证据落盘命名空间 (kv_store)。
pub const WRITE_GUARD_NS: &str = "write_guard";

/// 受保护 kv 命名空间 — agent 不得直接改写 (溯源/秘密/守卫自身)。
pub const PROTECTED_NAMESPACES: &[&str] = &[
    "secrets",
    "provenance",
    "write_guard",
    "dispatch_topology",
    "route_learner",
];

/// 写前检查裁决 — Allow 放行 / RequiresApproval 需人工 / Reject 硬阻断。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WriteGuardVerdict {
    Allow,
    RequiresApproval,
    Reject(Vec<String>),
}

impl WriteGuardVerdict {
    pub fn is_allowed(&self) -> bool {
        matches!(self, WriteGuardVerdict::Allow)
    }

    pub fn requires_approval(&self) -> bool {
        matches!(self, WriteGuardVerdict::RequiresApproval)
    }

    pub fn reasons(&self) -> Vec<String> {
        match self {
            WriteGuardVerdict::Reject(rs) => rs.clone(),
            WriteGuardVerdict::RequiresApproval => vec!["需要人工审批 (force 未置位)".into()],
            WriteGuardVerdict::Allow => Vec::new(),
        }
    }
}

/// 写前确定性检查 (纯函数, 无 I/O)。action 支持:
/// `node:create` / `node:update` / `edge:upsert` / `node:delete` / `edge:delete`
/// / `kv:set` / `kv:delete` / `embedding:backfill`。
pub fn kb_write_guard(action: &str, payload: &serde_json::Value) -> WriteGuardVerdict {
    let mut reasons: Vec<String> = Vec::new();

    match action {
        "node:create" | "node:update" => {
            let title = payload.get("title").and_then(|v| v.as_str()).unwrap_or("");
            if title.trim().is_empty() {
                reasons.push("title 为空".into());
            }
            if title.chars().count() > 512 {
                reasons.push("title 超过 512 字符".into());
            }
            if let Some(url) = payload.get("url").and_then(|v| v.as_str()) {
                if !url.is_empty()
                    && !(url.starts_with("http://")
                        || url.starts_with("https://")
                        || url.starts_with("file://"))
                {
                    reasons.push("url 必须以 http/https/file 前缀开头".into());
                }
            }
            if let Some(content) = payload.get("content").and_then(|v| v.as_str()) {
                if content.chars().count() > 1_000_000 {
                    reasons.push("content 超过 1MB".into());
                }
            }
            if let Some(id) = payload.get("id").and_then(|v| v.as_str()) {
                if id.starts_with("secret_") || id.contains("..") {
                    reasons.push("id 含保留前缀或路径逃逸".into());
                }
            }
        }
        "edge:upsert" => {
            let src = payload.get("source_id").and_then(|v| v.as_str()).unwrap_or("");
            let tgt = payload.get("target_id").and_then(|v| v.as_str()).unwrap_or("");
            if src.trim().is_empty() {
                reasons.push("source_id 为空".into());
            }
            if tgt.trim().is_empty() {
                reasons.push("target_id 为空".into());
            }
            if !src.trim().is_empty() && src == tgt {
                reasons.push("source_id 与 target_id 相同 (自环)".into());
            }
            if let Some(w) = payload.get("weight").and_then(|v| v.as_f64()) {
                if !(0.0..=1.0).contains(&w) {
                    reasons.push("weight 超出 [0,1]".into());
                }
            }
        }
        "node:delete" | "edge:delete" => {
            let id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");
            if id.trim().is_empty() {
                reasons.push("id 为空".into());
            }
            if id.starts_with("secret_") {
                reasons.push("id 为保留前缀 secret_".into());
            }
            let force = payload.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
            if !force {
                return WriteGuardVerdict::RequiresApproval;
            }
        }
        "kv:set" => {
            let ns = payload.get("namespace").and_then(|v| v.as_str()).unwrap_or("");
            if ns.trim().is_empty() {
                reasons.push("namespace 为空".into());
            }
            if PROTECTED_NAMESPACES.contains(&ns) {
                reasons.push(format!("namespace '{}' 受保护, 不可直接写入", ns));
            }
            let key = payload.get("key").and_then(|v| v.as_str()).unwrap_or("");
            if key.chars().count() > 512 {
                reasons.push("key 超过 512 字符".into());
            }
        }
        "kv:delete" => {
            let ns = payload.get("namespace").and_then(|v| v.as_str()).unwrap_or("");
            if PROTECTED_NAMESPACES.contains(&ns) {
                reasons.push(format!("namespace '{}' 受保护", ns));
            }
            let force = payload.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
            if !force {
                return WriteGuardVerdict::RequiresApproval;
            }
        }
        "embedding:backfill" => {
            // 全量重建嵌入 = 批量重写 (dbx high_risk_write 类), 一律要求审批。
            return WriteGuardVerdict::RequiresApproval;
        }
        other => reasons.push(format!("未知写操作 '{}'", other)),
    }

    if reasons.is_empty() {
        WriteGuardVerdict::Allow
    } else {
        WriteGuardVerdict::Reject(reasons)
    }
}

/// 证据 key 全局单调序号 — 同毫秒内同 action 多次裁决时保证 key 唯一
/// (kv_store 以 key 为追加语义, key 碰撞会导致证据互相覆盖)。
static WRITE_GUARD_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// 记录一次守卫裁决到 kv_store `write_guard` 命名空间 (证据链, 追加式)。
/// 失败仅告警不阻断主路径。
pub fn record_write_evidence(
    kb: &super::KnowledgeBase,
    action: &str,
    payload: &serde_json::Value,
    verdict: &WriteGuardVerdict,
    executed: bool,
) {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let seq = WRITE_GUARD_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let key = format!("{}-{}-{}", ts, action.replace(':', "_"), seq);
    let value = serde_json::json!({
        "action": action,
        "payload": payload,
        "verdict": verdict,
        "executed": executed,
        "ts_ms": ts,
    })
    .to_string();
    let _ = kb.kv_set(WRITE_GUARD_NS, &key, &value);
}

// ────────────────────────────────────────────────────────────────
// 守卫证据检测 (dbx 缺口 G4 闭环): write_guard 证据 → 聚合统计 →
// NT-SHIELD 审计 CheckResult (含 evidence) + SelfTest (T1/T2/T3)。
// 形成 "守卫拦截 → 证据 → 审计可查 → 自检可见" 的可审计闭环。
// ────────────────────────────────────────────────────────────────

/// 一条 write_guard 证据的结构化视图 (从 kv_store 反序列化)。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WriteGuardEvidence {
    pub key: String,
    pub action: String,
    pub verdict: WriteGuardVerdict,
    pub executed: bool,
    pub ts_ms: u64,
}

/// write_guard 命名空间证据聚合统计 (NT-SHIELD 审计输入)。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WriteGuardStats {
    /// 证据总数。
    pub total: usize,
    /// 放行 (Allow) 的裁决数。
    pub allowed: usize,
    /// 需人工审批 (RequiresApproval) 数。
    pub requires_approval: usize,
    /// 硬阻断 (Reject) 数。
    pub rejected: usize,
    /// 被拒操作的 action 分布。
    pub rejected_actions: BTreeMap<String, usize>,
    /// 异常清单: 被拒/需审批的写操作仍标记 executed。
    pub anomalies: Vec<String>,
}

/// 把一条 kv_store `write_guard` 条目解析为结构化证据; 解析失败返回 None。
pub fn parse_write_evidence(key: &str, raw: &str) -> Option<WriteGuardEvidence> {
    let v: serde_json::Value = serde_json::from_str(raw).ok()?;
    let verdict: WriteGuardVerdict = serde_json::from_value(v.get("verdict").cloned()?).ok()?;
    Some(WriteGuardEvidence {
        key: key.to_string(),
        action: v.get("action").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        verdict,
        executed: v.get("executed").and_then(|x| x.as_bool()).unwrap_or(false),
        ts_ms: v.get("ts_ms").and_then(|x| x.as_u64()).unwrap_or(0),
    })
}

/// 扫描 `write_guard` 证据命名空间, 聚合守卫拦截统计 + 异常检测。
///
/// 异常 (anomaly): 裁决为 Reject / RequiresApproval 的写操作仍标记
/// executed=true — 表示守卫被绕过或执行语义失配, 审计必须 FAIL。
/// 命名空间读取失败或单条证据解析失败也记入异常 (证据链完整性受损)。
pub fn scan_write_guard_evidence(kb: &super::KnowledgeBase) -> WriteGuardStats {
    let mut stats = WriteGuardStats::default();
    let entries = match kb.kv_list(WRITE_GUARD_NS) {
        Ok(e) => e,
        Err(e) => {
            stats.anomalies.push(format!("write_guard 命名空间读取失败: {}", e));
            return stats;
        }
    };
    for (key, raw) in entries {
        let Some(ev) = parse_write_evidence(&key, &raw) else {
            stats.anomalies.push(format!("write_guard 证据解析失败: {}", key));
            continue;
        };
        stats.total += 1;
        match &ev.verdict {
            WriteGuardVerdict::Allow => stats.allowed += 1,
            WriteGuardVerdict::RequiresApproval => {
                stats.requires_approval += 1;
                if ev.executed {
                    stats.anomalies.push(format!("action={} 需审批仍执行", ev.action));
                }
            }
            WriteGuardVerdict::Reject(_) => {
                stats.rejected += 1;
                *stats.rejected_actions.entry(ev.action.clone()).or_insert(0) += 1;
                if ev.executed {
                    stats.anomalies.push(format!("action={} 被拒仍执行", ev.action));
                }
            }
        }
    }
    stats
}

/// write_guard 证据检测件 — T1 SelfTest。
/// T2 注册: `register_absorbed_modules` (run.rs 架构审计) +
/// `register_lightweight_modules` + `pipeline.rs SelfTestStage`。
/// T3 生产接线: `handle_architecture_audit` 对生产 KB 调 `scan_write_guard_evidence`,
/// 折叠侧 (write_guard_check_result) 在 NT-SHIELD 审计域 (nt_shield_audit)。
///
/// `self_test` 在内存 KB 中制造含异常的证据并验证检测统计正确,
/// 纯内存无磁盘/网络 IO (可安全进入轻量注册表)。
#[derive(Debug, Clone, Copy, Default)]
pub struct WriteGuardAudit;

impl crate::core::nt_core_self_test::SelfTest for WriteGuardAudit {
    fn name(&self) -> &str {
        "nt_memory_write_guard_audit"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let kb = match super::KnowledgeBase::open(Some(std::path::PathBuf::from(":memory:"))) {
            Ok(kb) => kb,
            Err(e) => return Err(vec![format!("in-memory KB open failed: {}", e)]),
        };
        let mut failures = Vec::new();

        // 1) 正常放行: Allow + executed。
        record_write_evidence(
            &kb,
            "node:create",
            &serde_json::json!({"title": "ok"}),
            &WriteGuardVerdict::Allow,
            true,
        );
        // 2) 正常拦截: Reject + 未执行。
        record_write_evidence(
            &kb,
            "node:create",
            &serde_json::json!({"title": ""}),
            &WriteGuardVerdict::Reject(vec!["title 为空".into()]),
            false,
        );
        // 3) 异常: Reject + executed=true (守卫被绕过)。
        record_write_evidence(
            &kb,
            "kv:set",
            &serde_json::json!({"namespace": "secrets"}),
            &WriteGuardVerdict::Reject(vec!["namespace 'secrets' 受保护".into()]),
            true,
        );
        // 4) 异常: RequiresApproval + executed=true。
        record_write_evidence(
            &kb,
            "node:delete",
            &serde_json::json!({"id": "u_1"}),
            &WriteGuardVerdict::RequiresApproval,
            true,
        );

        let stats = scan_write_guard_evidence(&kb);
        if stats.total != 4 {
            failures.push(format!("expected 4 evidence, got {}", stats.total));
        }
        if stats.allowed != 1 || stats.requires_approval != 1 || stats.rejected != 2 {
            failures.push(format!("verdict counts wrong: {:?}", stats));
        }
        if stats.rejected_actions.get("kv:set") != Some(&1)
            || stats.rejected_actions.get("node:create") != Some(&1)
        {
            failures.push(format!(
                "rejected_actions distribution wrong: {:?}",
                stats.rejected_actions
            ));
        }
        if stats.anomalies.len() != 2 {
            failures.push(format!("expected 2 anomalies, got {:?}", stats.anomalies));
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn test_node_create_allows_valid() {
        let payload = serde_json::json!({"title": "Rust", "url": "https://rust-lang.org", "content": "lang"});
        assert_eq!(kb_write_guard("node:create", &payload), WriteGuardVerdict::Allow);
    }

    #[test]
    fn test_node_create_rejects_empty_title() {
        let payload = serde_json::json!({"title": "  ", "url": "https://x.dev"});
        assert_eq!(
            kb_write_guard("node:create", &payload),
            WriteGuardVerdict::Reject(vec!["title 为空".into()])
        );
    }

    #[test]
    fn test_node_create_rejects_bad_url() {
        let payload = serde_json::json!({"title": "t", "url": "ftp://x"});
        assert_eq!(
            kb_write_guard("node:create", &payload),
            WriteGuardVerdict::Reject(vec!["url 必须以 http/https/file 前缀开头".into()])
        );
    }

    #[test]
    fn test_node_create_rejects_reserved_id() {
        let payload = serde_json::json!({"title": "t", "id": "secret_foo"});
        let v = kb_write_guard("node:create", &payload);
        assert!(matches!(v, WriteGuardVerdict::Reject(ref rs) if rs.iter().any(|r| r.contains("保留前缀"))));
    }

    #[test]
    fn test_edge_upsert_valid() {
        let payload = serde_json::json!({"source_id": "a", "target_id": "b", "weight": 0.5});
        assert_eq!(kb_write_guard("edge:upsert", &payload), WriteGuardVerdict::Allow);
    }

    #[test]
    fn test_edge_upsert_rejects_self_loop_and_range() {
        let payload = serde_json::json!({"source_id": "a", "target_id": "a", "weight": 1.5});
        let v = kb_write_guard("edge:upsert", &payload);
        assert!(matches!(v, WriteGuardVerdict::Reject(ref rs) if rs.len() == 2), "{:?}", v);
    }

    #[test]
    fn test_delete_requires_approval_without_force() {
        let payload = serde_json::json!({"id": "u_1"});
        assert_eq!(
            kb_write_guard("node:delete", &payload),
            WriteGuardVerdict::RequiresApproval
        );
    }

    #[test]
    fn test_delete_allowed_with_force() {
        let payload = serde_json::json!({"id": "u_1", "force": true});
        assert_eq!(kb_write_guard("node:delete", &payload), WriteGuardVerdict::Allow);
    }

    #[test]
    fn test_delete_rejects_reserved_prefix_even_with_force() {
        let payload = serde_json::json!({"id": "secret_x", "force": true});
        assert!(matches!(kb_write_guard("node:delete", &payload), WriteGuardVerdict::Reject(_)));
    }

    #[test]
    fn test_kv_set_protected_namespace() {
        let payload = serde_json::json!({"namespace": "secrets", "key": "k", "value": "v"});
        let v = kb_write_guard("kv:set", &payload);
        assert!(matches!(v, WriteGuardVerdict::Reject(ref rs) if rs.iter().any(|r| r.contains("受保护"))));
    }

    #[test]
    fn test_kv_set_valid() {
        let payload = serde_json::json!({"namespace": "brain", "key": "k", "value": "v"});
        assert_eq!(kb_write_guard("kv:set", &payload), WriteGuardVerdict::Allow);
    }

    #[test]
    fn test_embedding_backfill_always_requires_approval() {
        let payload = serde_json::json!({});
        assert_eq!(
            kb_write_guard("embedding:backfill", &payload),
            WriteGuardVerdict::RequiresApproval
        );
    }

    #[test]
    fn test_unknown_action_rejected() {
        assert!(matches!(kb_write_guard("rm -rf", &serde_json::json!({})), WriteGuardVerdict::Reject(_)));
    }

    #[test]
    fn test_record_write_evidence_persists() {
        let tmp = std::env::temp_dir().join(format!("nt_wg_evid_{}", std::process::id()));
        let _ = std::fs::remove_file(&tmp);
        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(tmp.clone())).unwrap();
        let verdict = WriteGuardVerdict::Reject(vec!["title 为空".into()]);
        record_write_evidence(&kb, "node:create", &serde_json::json!({"title": ""}), &verdict, false);
        let entries = kb.kv_list(WRITE_GUARD_NS).unwrap();
        assert_eq!(entries.len(), 1);
        assert!(entries[0].1.contains("node:create"));
        assert!(entries[0].1.contains("Reject"));
        let _ = std::fs::remove_file(&tmp);
    }

    fn in_memory_kb() -> crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase {
        crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(
            std::path::PathBuf::from(":memory:"),
        ))
        .unwrap()
    }

    #[test]
    fn test_record_write_evidence_unique_keys_in_same_ms() {
        let kb = in_memory_kb();
        record_write_evidence(
            &kb,
            "node:create",
            &serde_json::json!({"title": ""}),
            &WriteGuardVerdict::Reject(vec!["title 为空".into()]),
            false,
        );
        record_write_evidence(
            &kb,
            "node:create",
            &serde_json::json!({"title": ""}),
            &WriteGuardVerdict::Reject(vec!["title 为空".into()]),
            false,
        );
        let entries = kb.kv_list(WRITE_GUARD_NS).unwrap();
        assert_eq!(entries.len(), 2, "同毫秒同 action 两次裁决不得互相覆盖");
    }

    #[test]
    fn test_parse_write_evidence_roundtrip_reject() {
        let raw = serde_json::json!({
            "action": "node:create",
            "payload": {"title": "t"},
            "verdict": {"Reject": ["title 为空"]},
            "executed": false,
            "ts_ms": 1234,
        })
        .to_string();
        let ev = parse_write_evidence("k-1", &raw).unwrap();
        assert_eq!(ev.key, "k-1");
        assert_eq!(ev.action, "node:create");
        assert_eq!(ev.verdict, WriteGuardVerdict::Reject(vec!["title 为空".into()]));
        assert!(!ev.executed);
        assert_eq!(ev.ts_ms, 1234);
    }

    #[test]
    fn test_parse_write_evidence_allow() {
        let raw = serde_json::json!({
            "action": "edge:upsert",
            "verdict": "Allow",
            "executed": true,
            "ts_ms": 5,
        })
        .to_string();
        let ev = parse_write_evidence("k", &raw).unwrap();
        assert_eq!(ev.verdict, WriteGuardVerdict::Allow);
        assert!(ev.executed);
    }

    #[test]
    fn test_parse_write_evidence_malformed_returns_none() {
        assert!(parse_write_evidence("k", "not json").is_none());
        assert!(parse_write_evidence("k", "{\"action\":\"x\"}").is_none());
        assert!(parse_write_evidence("k", "{\"action\":\"x\",\"verdict\":\"Weird\"}").is_none());
    }

    #[test]
    fn test_scan_counts_and_detects_anomalies() {
        let kb = in_memory_kb();
        record_write_evidence(
            &kb,
            "node:create",
            &serde_json::json!({"title": "ok"}),
            &WriteGuardVerdict::Allow,
            true,
        );
        record_write_evidence(
            &kb,
            "node:create",
            &serde_json::json!({"title": ""}),
            &WriteGuardVerdict::Reject(vec!["title 为空".into()]),
            false,
        );
        record_write_evidence(
            &kb,
            "kv:set",
            &serde_json::json!({"namespace": "secrets"}),
            &WriteGuardVerdict::Reject(vec!["namespace 'secrets' 受保护".into()]),
            true,
        );
        record_write_evidence(
            &kb,
            "node:delete",
            &serde_json::json!({"id": "u_1"}),
            &WriteGuardVerdict::RequiresApproval,
            true,
        );

        let stats = scan_write_guard_evidence(&kb);
        assert_eq!(stats.total, 4);
        assert_eq!(stats.allowed, 1);
        assert_eq!(stats.requires_approval, 1);
        assert_eq!(stats.rejected, 2);
        assert_eq!(stats.rejected_actions.get("node:create"), Some(&1));
        assert_eq!(stats.rejected_actions.get("kv:set"), Some(&1));
        assert_eq!(stats.anomalies.len(), 2, "anomalies: {:?}", stats.anomalies);
        assert!(stats.anomalies.iter().any(|a| a.contains("kv:set") && a.contains("被拒仍执行")));
        assert!(stats.anomalies.iter().any(|a| a.contains("node:delete") && a.contains("需审批仍执行")));
    }

    #[test]
    fn test_scan_empty_namespace_clean() {
        let kb = in_memory_kb();
        let stats = scan_write_guard_evidence(&kb);
        assert_eq!(stats.total, 0);
        assert!(stats.anomalies.is_empty());
    }

    #[test]
    fn test_scan_flags_malformed_evidence() {
        let kb = in_memory_kb();
        kb.kv_set(WRITE_GUARD_NS, "bad-key", "not-json").unwrap();
        let stats = scan_write_guard_evidence(&kb);
        assert_eq!(stats.total, 0);
        assert!(!stats.anomalies.is_empty());
        assert!(stats.anomalies.iter().any(|a| a.contains("解析失败")));
    }

    #[test]
    fn test_write_guard_audit_selftest_passes() {
        use crate::core::nt_core_self_test::SelfTest;
        let audit = WriteGuardAudit::default();
        assert_eq!(audit.name(), "nt_memory_write_guard_audit");
        assert!(audit.self_test().is_ok());
    }
}