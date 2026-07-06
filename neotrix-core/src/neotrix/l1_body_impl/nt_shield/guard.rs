//! 交互式安全守卫 — HashCortX dual-layer guard 模式
//!
//! Layer 1: 会话记忆权限 (allow-once/allow-session/deny-session)
//! Layer 2: 硬编码 denylist (不可绕过, 路径+命令阻断)
//! Layer 3: Append-only audit log (所有安全操作可追溯)
//!
//! 对标: HashCortX guard.js + denylist.rs + audit log

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

/// 守卫决策
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GuardDecision {
    Allowed,
    AllowedOnce,
    AllowedSession,
    Denied,
    DeniedSession,
    RequiresConfirmation,
}

/// 守卫请求
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardRequest {
    pub id: String,
    pub action: String,
    pub target: String,
    pub reason: String,
    pub timestamp: i64,
    pub decision: Option<GuardDecision>,
}

impl GuardRequest {
    pub fn new(action: &str, target: &str, reason: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            action: action.to_string(),
            target: target.to_string(),
            reason: reason.to_string(),
            timestamp: Utc::now().timestamp(),
            decision: None,
        }
    }
}

/// 双层安全守卫
pub struct SecurityGuard {
    /// Layer 1: 会话记忆
    session_memory: Mutex<HashMap<(String, String), GuardDecision>>,
    /// 项目根路径 (自动放行读操作)
    project_root: Mutex<Option<String>>,
    /// 待确认请求
    pending: Mutex<Vec<GuardRequest>>,
    /// Layer 2: 硬编码 denylist
    denylist: DenyList,
    /// Layer 3: audit log
    audit: AuditLog,
}

impl SecurityGuard {
    pub fn new() -> Self {
        Self {
            session_memory: Mutex::new(HashMap::new()),
            project_root: Mutex::new(None),
            pending: Mutex::new(Vec::new()),
            denylist: DenyList::new(),
            audit: AuditLog::new(),
        }
    }

    /// 设置项目根路径
    pub fn set_project_root(&self, root: &str) {
        if let Ok(mut r) = self.project_root.lock() {
            *r = Some(root.to_string());
        }
    }

    /// 检查操作是否放行
    /// Ok(true) = 放行, Ok(false) = denylist 拒绝, Err(请求) = 需交互确认
    pub fn check(&self, action: &str, target: &str) -> Result<bool, GuardRequest> {
        // Layer 2: 先查 denylist
        if self.denylist.is_blocked(action, target) {
            self.audit.append("DENY", action, target, "denylist");
            return Ok(false);
        }

        // Layer 1: 会话记忆
        let key = (action.to_string(), target.to_string());
        if let Ok(memory) = self.session_memory.lock() {
            if let Some(decision) = memory.get(&key) {
                match decision {
                    GuardDecision::Allowed
                    | GuardDecision::AllowedOnce
                    | GuardDecision::AllowedSession => return Ok(true),
                    GuardDecision::Denied | GuardDecision::DeniedSession => {
                        self.audit.append("DENY", action, target, "session_memory");
                        return Ok(false);
                    }
                    _ => {}
                }
            }
        }

        // 项目根自动放行 (read 操作)
        if action == "file_read" || action == "file_list" || action == "file_search" {
            if let Ok(root) = self.project_root.lock() {
                if let Some(root_path) = root.as_ref() {
                    if target.starts_with(root_path) {
                        self.audit.append("ALLOW", action, target, "project_root");
                        return Ok(true);
                    }
                }
            }
        }

        // 需要交互确认
        let request = GuardRequest::new(action, target, &format!("{} on {} needs approval", action, target));
        if let Ok(mut pending) = self.pending.lock() {
            pending.push(request.clone());
        }
        Err(request)
    }

    /// 解析待确认请求
    pub fn resolve(&self, id: &str, decision: GuardDecision) -> bool {
        if let Ok(mut pending) = self.pending.lock() {
            if let Some(pos) = pending.iter().position(|r| r.id == id) {
                let req = pending.remove(pos);
                let key = (req.action.clone(), req.target.clone());
                if let Ok(mut memory) = self.session_memory.lock() {
                    memory.insert(key, decision.clone());
                }
                let label = match &decision {
                    GuardDecision::Allowed | GuardDecision::AllowedOnce | GuardDecision::AllowedSession => "ALLOW",
                    _ => "DENY",
                };
                self.audit.append(label, &req.action, &req.target, "user_resolved");
                return true;
            }
        }
        false
    }

    /// 获取待确认请求列表
    pub fn pending_requests(&self) -> Vec<GuardRequest> {
        match self.pending.lock() {
            Ok(p) => p.clone(),
            Err(e) => {
                log::warn!("[guard] pending lock: {}", e);
                Vec::new()
            }
        }
    }

    /// 清空会话记忆
    pub fn clear_session(&self) {
        if let Ok(mut memory) = self.session_memory.lock() {
            memory.clear();
        }
    }
}

impl Default for SecurityGuard {
    fn default() -> Self {
        Self::new()
    }
}

/// 硬编码 denylist (不可绕过)
pub struct DenyList {
    blocked_path_prefixes: Vec<&'static str>,
    blocked_path_exact: Vec<&'static str>,
    blocked_commands: Vec<&'static str>,
}

impl Default for DenyList {
    fn default() -> Self {
        Self::new()
    }
}

impl DenyList {
    pub fn new() -> Self {
        Self {
            blocked_path_prefixes: vec![
                "/System",
                "/etc",
                "/usr/bin",
                "/usr/sbin",
                "/private/etc",
                "/Library/Keychains",
                "/dev/",
                "/proc/",
            ],
            blocked_path_exact: vec![
                ".ssh",
                ".aws",
                ".config/gcloud",
                ".config/opencode",
                ".gnupg",
                ".kube",
                ".docker",
            ],
            blocked_commands: vec![
                "sudo",
                "rm -rf /",
                "rm -rf /*",
                "dd if=",
                "chmod 777",
                "chown",
                "> /dev/",
                "| sh",
                "| bash",
                "bash <(",
                "sh <(",
                ":(){ :|:& };:",
                "mkfs",
                "fdisk",
                "dd",
                "reboot",
                "shutdown",
                "init",
                "halt",
                "poweroff",
            ],
        }
    }

    /// 检查操作是否被 denylist 阻断
    pub fn is_blocked(&self, action: &str, target: &str) -> bool {
        match action {
            "file_write" | "file_delete" | "file_exec" => {
                let path = Path::new(target);
                let path_str = path.to_string_lossy();
                // 先检查原始路径 (文件可能不存在)
                if self.blocked_path_exact.iter().any(|p| {
                    let pat = format!("/{}", p);
                    path_str.contains(&pat)
                }) {
                    return true;
                }
                // 再检查规范化路径
                if let Ok(canonical) = path.canonicalize() {
                    let canon = canonical.to_string_lossy();
                    if self.blocked_path_prefixes.iter().any(|p| canon.starts_with(p)) {
                        return true;
                    }
                }
                false
            }
            "command_exec" => {
                let lower = target.to_lowercase();
                self.blocked_commands
                    .iter()
                    .any(|bc| lower.contains(bc))
            }
            "network_access" => {
                target.starts_with("http://localhost:")
                    || target.starts_with("http://127.0.0.1:")
                    || target.starts_with("http://[::1]:")
            }
            _ => false,
        }
    }

    pub fn blocked_count(&self) -> usize {
        self.blocked_path_prefixes.len()
            + self.blocked_path_exact.len()
            + self.blocked_commands.len()
    }
}

/// Append-only audit log
pub struct AuditLog {
    entries: Mutex<Vec<AuditEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: String,
    pub decision: String,
    pub action: String,
    pub target: String,
    pub reason: String,
}

impl AuditLog {
    pub fn new() -> Self {
        Self {
            entries: Mutex::new(Vec::new()),
        }
    }

    /// 追加审计条目
    pub fn append(&self, decision: &str, action: &str, target: &str, reason: &str) {
        let entry = AuditEntry {
            timestamp: Utc::now().to_rfc3339(),
            decision: decision.to_string(),
            action: action.to_string(),
            target: target.to_string(),
            reason: reason.to_string(),
        };
        if let Ok(mut entries) = self.entries.lock() {
            entries.push(entry);
        }
    }

    /// 获取审计日志
    pub fn entries(&self) -> Vec<AuditEntry> {
        match self.entries.lock() {
            Ok(e) => e.clone(),
            Err(e) => {
                log::warn!("[guard] entries lock: {}", e);
                Vec::new()
            }
        }
    }

    /// 最近 N 条
    pub fn recent(&self, n: usize) -> Vec<AuditEntry> {
        match self.entries.lock() {
            Ok(e) => {
                let len = e.len();
                e.iter()
                    .skip(len.saturating_sub(n))
                    .cloned()
                    .collect()
            }
            Err(e) => {
                log::warn!("[guard] recent lock: {}", e);
                Vec::new()
            }
        }
    }

    pub fn count(&self) -> usize {
        match self.entries.lock() {
            Ok(e) => e.len(),
            Err(e) => {
                log::warn!("[guard] count lock: {}", e);
                0
            }
        }
    }
}

impl Default for AuditLog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_denylist_blocks_system_path() {
        let dl = DenyList::new();
        assert!(dl.is_blocked("file_write", "/etc/passwd"));
        assert!(dl.is_blocked("file_delete", "/etc/hosts"));
    }

    #[test]
    fn test_denylist_blocks_dot_ssh() {
        let dl = DenyList::new();
        assert!(dl.is_blocked("file_write", "/Users/test/.ssh/id_rsa"));
    }

    #[test]
    fn test_denylist_blocks_sudo() {
        let dl = DenyList::new();
        assert!(dl.is_blocked("command_exec", "sudo rm -rf /tmp"));
    }

    #[test]
    fn test_denylist_allows_normal_path() {
        let dl = DenyList::new();
        assert!(!dl.is_blocked("file_write", "/tmp/test.txt"));
        assert!(!dl.is_blocked("file_write", "/Users/test/Documents/code/main.rs"));
    }

    #[test]
    fn test_denylist_allows_normal_command() {
        let dl = DenyList::new();
        assert!(!dl.is_blocked("command_exec", "ls -la"));
        assert!(!dl.is_blocked("command_exec", "cargo check --lib"));
    }

    #[test]
    fn test_denylist_blocks_fork_bomb() {
        let dl = DenyList::new();
        assert!(dl.is_blocked("command_exec", ":(){ :|:& };:"));
    }

    #[test]
    fn test_guard_auto_allows_project_root_read() {
        let guard = SecurityGuard::new();
        guard.set_project_root("/Users/test/project");
        assert_eq!(
            guard.check("file_read", "/Users/test/project/src/main.rs"),
            Ok(true)
        );
    }

    #[test]
    fn test_guard_requires_confirmation_outside_project() {
        let guard = SecurityGuard::new();
        guard.set_project_root("/Users/test/project");
        let result = guard.check("file_write", "/etc/passwd");
        assert_eq!(result, Ok(false)); // denylist
    }

    #[test]
    fn test_guard_requires_confirmation_for_write() {
        let guard = SecurityGuard::new();
        guard.set_project_root("/Users/test/project");
        let result = guard.check("file_write", "/Users/test/project/Cargo.toml");
        assert!(result.is_err());
        let req = result.unwrap_err();
        assert_eq!(req.action, "file_write");
    }

    #[test]
    fn test_guard_resolve_allow_once() {
        let guard = SecurityGuard::new();
        guard.set_project_root("/p");
        let result = guard.check("file_write", "/p/file.txt");
        assert!(result.is_err());
        let req = result.unwrap_err();

        assert!(guard.resolve(&req.id, GuardDecision::AllowedOnce));
        assert_eq!(guard.check("file_write", "/p/file.txt"), Ok(true));
    }

    #[test]
    fn test_guard_resolve_deny_session() {
        let guard = SecurityGuard::new();
        guard.set_project_root("/p");
        let result = guard.check("file_write", "/p/secret.txt");
        assert!(result.is_err());
        let req = result.unwrap_err();

        assert!(guard.resolve(&req.id, GuardDecision::DeniedSession));
        assert_eq!(guard.check("file_write", "/p/secret.txt"), Ok(false));
    }

    #[test]
    fn test_audit_log_append() {
        let log = AuditLog::new();
        log.append("ALLOW", "file_read", "/tmp/test.txt", "test");
        assert_eq!(log.count(), 1);
    }

    #[test]
    fn test_audit_log_recent() {
        let log = AuditLog::new();
        log.append("ALLOW", "a", "t1", "r1");
        log.append("ALLOW", "b", "t2", "r2");
        log.append("DENY", "c", "t3", "r3");
        let recent = log.recent(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].action, "b");
        assert_eq!(recent[1].action, "c");
    }

    #[test]
    fn test_resolve_nonexistent_fails() {
        let guard = SecurityGuard::new();
        assert!(!guard.resolve("no-such-id", GuardDecision::Allowed));
    }

    #[test]
    fn test_clear_session() {
        let guard = SecurityGuard::new();
        guard.set_project_root("/p");
        let result = guard.check("file_write", "/p/f.txt");
        assert!(result.is_err());
        let req = result.unwrap_err();
        guard.resolve(&req.id, GuardDecision::AllowedSession);

        guard.clear_session();
        let result2 = guard.check("file_write", "/p/f.txt");
        assert!(result2.is_err());
    }

    #[test]
    fn test_pending_requests() {
        let guard = SecurityGuard::new();
        guard.set_project_root("/p");
        let _ = guard.check("file_write", "/p/a.txt");
        let _ = guard.check("file_write", "/p/b.txt");
        assert_eq!(guard.pending_requests().len(), 2);
    }

    #[test]
    fn test_denylist_count() {
        let dl = DenyList::new();
        assert!(dl.blocked_count() > 30);
    }
}
