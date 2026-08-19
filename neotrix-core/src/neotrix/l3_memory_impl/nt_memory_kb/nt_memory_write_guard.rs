//! KB 写门前置检查 (dbx absorb, G4) — AI 生成/agent 发起的 KB 写操作执行前
//! 确定性检查 + 证据落盘 (kv_store `write_guard` 命名空间)。
//!
//! 对齐 dbx 的 "AI 生成 SQL 执行前安全检查" 与 NeoTrix `nt_core_gate` 的
//! `ActionTier` 语义: 只读自动放行 (Tier1), 可逆写确定性检查后放行 (Tier2),
//! 删除/批量重建要求审批 (Tier3/Tier4)。守卫是纯函数 (无 I/O), 便于单元测试;
//! 证据记录是唯一副作用。

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
    let key = format!("{}-{}", ts, action.replace(':', "_"));
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
        let kb = crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase::open(Some(tmp.clone())).unwrap();
        let verdict = WriteGuardVerdict::Reject(vec!["title 为空".into()]);
        record_write_evidence(&kb, "node:create", &serde_json::json!({"title": ""}), &verdict, false);
        let entries = kb.kv_list(WRITE_GUARD_NS).unwrap();
        assert_eq!(entries.len(), 1);
        assert!(entries[0].1.contains("node:create"));
        assert!(entries[0].1.contains("Reject"));
        let _ = std::fs::remove_file(&tmp);
    }
}