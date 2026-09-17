use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

/// 写前检查裁决 — Allow 放行 / RequiresApproval 需人工 / Reject 硬阻断 /
/// Hold (被 hold, 需先 acknowledge 才能 override)。
///
/// Hold 变体吸收 cumora §5d hold-token 机制: 当写操作被 Reject 或
/// RequiresApproval 时, 如果调用方持有对应 scope 的 hold token,
/// 可以通过 `force_override=true` 放行。Token 必须由 server 在展示
/// HELD 原因后签发, 客户端无法伪造。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WriteGuardVerdict {
    Allow,
    RequiresApproval,
    Reject(Vec<String>),
    /// 被 hold — 写操作被拦截, 但 agent 已被展示原因。
    /// 只有持有对应 scope token 的 override 才能放行。
    Hold {
        reasons: Vec<String>,
        scope: String,
    },
}

/// Hold token — 绑定到特定 scope (action + content fingerprint),
/// 由 server 在展示 HELD 原因后签发。Token:
/// - 绑定 scope: 同一 content 的同一 action 才能消费
/// - 有 TTL: 超时自动失效
/// - 一次性: 消费后即销毁
/// - turn-end 清除: 一个决策周期结束后清除所有未消费 token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoldToken {
    pub scope: String,
    pub issued_at_ms: u64,
    pub ttl_ms: u64,
}

impl HoldToken {
    pub fn is_expired(&self, now_ms: u64) -> bool {
        now_ms.saturating_sub(self.issued_at_ms) > self.ttl_ms
    }
}

/// 线程安全的 Hold Token 存储 — 每个 scope 最多一个 token。
/// TTL 默认 5 分钟 (比 cumora 的 2 分钟长, 因为 KB 写入不那么时间敏感)。
pub struct HoldTokenStore {
    tokens: Mutex<HashMap<String, HoldToken>>,
    default_ttl: Duration,
}

impl HoldTokenStore {
    pub fn new() -> Self {
        Self::with_ttl(Duration::from_secs(300))
    }

    pub fn with_ttl(ttl: Duration) -> Self {
        Self {
            tokens: Mutex::new(HashMap::new()),
            default_ttl: ttl,
        }
    }

    /// 签发一个 hold token (server 在展示 HELD 原因后调用)。
    pub fn issue(&self, scope: &str) -> HoldToken {
        let now_ms = now_millis();
        let token = HoldToken {
            scope: scope.to_string(),
            issued_at_ms: now_ms,
            ttl_ms: self.default_ttl.as_millis() as u64,
        };
        let mut tokens = self.tokens.lock().unwrap();
        tokens.insert(scope.to_string(), token.clone());
        token
    }

    /// 尝试消费一个 hold token — 原子操作:
    /// - scope 存在 + 未过期 → 删除并返回 true
    /// - scope 不存在或已过期 → 删除过期 token 并返回 false
    pub fn try_consume(&self, scope: &str) -> bool {
        let now_ms = now_millis();
        let mut tokens = self.tokens.lock().unwrap();
        if let Some(token) = tokens.get(scope) {
            if !token.is_expired(now_ms) {
                tokens.remove(scope);
                return true;
            }
            // 过期, 清理
            tokens.remove(scope);
        }
        false
    }

    /// 检查 scope 是否有有效的未消费 token (不消费)。
    pub fn has_valid(&self, scope: &str) -> bool {
        let now_ms = now_millis();
        let tokens = self.tokens.lock().unwrap();
        tokens
            .get(scope)
            .map(|t| !t.is_expired(now_ms))
            .unwrap_or(false)
    }

    /// 清除指定 scope 的 token。
    pub fn clear(&self, scope: &str) {
        let mut tokens = self.tokens.lock().unwrap();
        tokens.remove(scope);
    }

    /// 清除所有 token (turn-end 调用)。
    pub fn clear_all(&self) {
        let mut tokens = self.tokens.lock().unwrap();
        tokens.clear();
    }

    /// 清除所有过期 token (垃圾回收)。
    pub fn gc(&self) {
        let now_ms = now_millis();
        let mut tokens = self.tokens.lock().unwrap();
        tokens.retain(|_, t| !t.is_expired(now_ms));
    }

    /// 当前活跃 token 数量 (诊断用)。
    pub fn active_count(&self) -> usize {
        let now_ms = now_millis();
        let tokens = self.tokens.lock().unwrap();
        tokens.values().filter(|t| !t.is_expired(now_ms)).count()
    }
}

impl Default for HoldTokenStore {
    fn default() -> Self {
        Self::new()
    }
}

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// 全局 hold token store (singleton)。
/// 用途: WriteGuard hold 裁决 → agent acknowledge → override 门控。
pub static GLOBAL_HOLD_TOKENS: once_cell::sync::Lazy<HoldTokenStore> =
    once_cell::sync::Lazy::new(HoldTokenStore::new);

/// 写操作 scope 计算 — action + content fingerprint。
/// 同一 action + 相同 payload 产生相同 scope, 不同 payload 产生不同 scope。
pub fn compute_scope(action: &str, payload: &serde_json::Value) -> String {
    use std::hash::{Hash, Hasher};
    let payload_str = serde_json::to_string(payload).unwrap_or_default();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    payload_str.hash(&mut hasher);
    let hash = hasher.finish();
    format!("{}:{:016x}", action, hash)
}

impl WriteGuardVerdict {
    pub fn is_allowed(&self) -> bool {
        matches!(self, WriteGuardVerdict::Allow)
    }

    pub fn requires_approval(&self) -> bool {
        matches!(self, WriteGuardVerdict::RequiresApproval)
    }

    pub fn is_hold(&self) -> bool {
        matches!(self, WriteGuardVerdict::Hold { .. })
    }

    pub fn reasons(&self) -> Vec<String> {
        match self {
            WriteGuardVerdict::Reject(rs) => rs.clone(),
            WriteGuardVerdict::RequiresApproval => vec!["需要人工审批 (force 未置位)".into()],
            WriteGuardVerdict::Hold { reasons, .. } => reasons.clone(),
            WriteGuardVerdict::Allow => Vec::new(),
        }
    }

    /// Hold 裁决的 scope (仅 Hold 变体返回 Some)。
    pub fn hold_scope(&self) -> Option<&str> {
        match self {
            WriteGuardVerdict::Hold { scope, .. } => Some(scope),
            _ => None,
        }
    }

    /// 判断 override (force=true) 是否应被放行:
    /// - Allow → true
    /// - Reject → false (force 无效)
    /// - RequiresApproval → force 放行
    /// - Hold → 必须持有对应 scope 的 token 才放行
    pub fn should_allow_override(&self, force: bool, has_hold_token: bool) -> bool {
        match self {
            WriteGuardVerdict::Allow => true,
            WriteGuardVerdict::Reject(_) => false,
            WriteGuardVerdict::RequiresApproval => force,
            WriteGuardVerdict::Hold { .. } => force && has_hold_token,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WriteGuardStats {
    pub total: usize,
    pub allowed: usize,
    pub requires_approval: usize,
    pub rejected: usize,
    pub rejected_actions: std::collections::BTreeMap<String, usize>,
    pub anomalies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteGuardEvidence {
    pub key: String,
    pub action: String,
    pub verdict: WriteGuardVerdict,
    pub executed: bool,
    pub ts_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verdict_allow_roundtrip() {
        let v = WriteGuardVerdict::Allow;
        let json = serde_json::to_string(&v).unwrap();
        let back: WriteGuardVerdict = serde_json::from_str(&json).unwrap();
        assert_eq!(back, WriteGuardVerdict::Allow);
    }

    #[test]
    fn test_verdict_reject_roundtrip() {
        let v = WriteGuardVerdict::Reject(vec!["bad".into()]);
        let json = serde_json::to_string(&v).unwrap();
        let back: WriteGuardVerdict = serde_json::from_str(&json).unwrap();
        match back {
            WriteGuardVerdict::Reject(rs) => assert_eq!(rs, vec!["bad"]),
            _ => panic!("expected Reject"),
        }
    }

    #[test]
    fn test_verdict_requires_approval() {
        let v = WriteGuardVerdict::RequiresApproval;
        assert!(!v.is_allowed());
        assert!(v.requires_approval());
        assert_eq!(v.reasons().len(), 1);
    }

    #[test]
    fn test_verdict_hold_roundtrip() {
        let v = WriteGuardVerdict::Hold {
            reasons: vec!["被 hold".into()],
            scope: "node:create:abc123".into(),
        };
        let json = serde_json::to_string(&v).unwrap();
        let back: WriteGuardVerdict = serde_json::from_str(&json).unwrap();
        assert!(back.is_hold());
        assert_eq!(back.hold_scope(), Some("node:create:abc123"));
        assert_eq!(back.reasons(), vec!["被 hold"]);
    }

    #[test]
    fn test_should_allow_override() {
        // Allow always passes
        assert!(WriteGuardVerdict::Allow.should_allow_override(false, false));
        // Reject never passes, even with force
        assert!(!WriteGuardVerdict::Reject(vec![]).should_allow_override(true, false));
        // RequiresApproval passes with force
        assert!(WriteGuardVerdict::RequiresApproval.should_allow_override(true, false));
        assert!(!WriteGuardVerdict::RequiresApproval.should_allow_override(false, false));
        // Hold requires both force AND hold token
        let hold = WriteGuardVerdict::Hold {
            reasons: vec![],
            scope: "s".into(),
        };
        assert!(!hold.should_allow_override(false, false));
        assert!(!hold.should_allow_override(true, false)); // force but no token
        assert!(!hold.should_allow_override(false, true)); // token but no force
        assert!(hold.should_allow_override(true, true)); // both
    }

    #[test]
    fn test_hold_token_store_lifecycle() {
        let store = HoldTokenStore::new();
        let token = store.issue("node:create:abc");
        assert_eq!(token.scope, "node:create:abc");
        assert!(!token.is_expired(now_millis()));
        // 有效 token 可消费
        assert!(store.try_consume("node:create:abc"));
        // 消费后不存在
        assert!(!store.try_consume("node:create:abc"));
        // 不存在的 scope
        assert!(!store.try_consume("nonexistent"));
    }

    #[test]
    fn test_hold_token_gc() {
        let store = HoldTokenStore::with_ttl(Duration::from_millis(1));
        store.issue("short-lived");
        std::thread::sleep(Duration::from_millis(5));
        store.gc();
        assert_eq!(store.active_count(), 0);
    }

    #[test]
    fn test_hold_token_clear_all() {
        let store = HoldTokenStore::new();
        store.issue("a");
        store.issue("b");
        assert_eq!(store.active_count(), 2);
        store.clear_all();
        assert_eq!(store.active_count(), 0);
    }

    #[test]
    fn test_compute_scope_deterministic() {
        let payload = serde_json::json!({"title": "test"});
        let s1 = compute_scope("node:create", &payload);
        let s2 = compute_scope("node:create", &payload);
        assert_eq!(s1, s2);
        // 不同 payload → 不同 scope
        let s3 = compute_scope("node:create", &serde_json::json!({"title": "other"}));
        assert_ne!(s1, s3);
    }
}
