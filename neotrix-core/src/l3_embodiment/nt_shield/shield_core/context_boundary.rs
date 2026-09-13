//! 上下文权限边界 — 防止 CPE 攻击
//!
//! 验证上下文来源，阻止权限提升。
//!
//! ## Privilege Separation (Absorbed: airgorah pattern)
//!
//! airgorah achieves privilege separation via polkit agent: the GTK4 GUI runs
//! as a normal user while the Rust backend performs privileged operations
//! through polkit-authenticated D-Bus calls. NeoTrix maps this to trust-level
//! isolation:
//!
//! | airgorah Layer      | NeoTrix TrustLevel | Allowed Operations           |
//! |---------------------|--------------------|------------------------------|
//! | polkit agent (root) | `System`           | Everything (wildcard)        |
//! | GTK4 GUI (user)     | `User`             | read/write/execute/search    |
//! | D-Bus interface     | `Tool`             | read/search only             |
//! | External input      | `External`         | read only                    |
//! | Sandbox / untrusted | `Untrusted`        | nothing                      |
//!
//! Key principle: **the agent never escalates its own trust level**.
//! Privilege escalation requires an external `ContextRequest` with a higher
//! `TrustLevel` — the agent cannot forge this.

use std::collections::HashMap;

/// 上下文来源信任级别
#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]

pub enum TrustLevel {
    /// 系统内部（完全信任）
    System,
    /// 用户输入（高信任）
    User,
    /// 工具输出（中信任）
    Tool,
    /// 外部来源（低信任）
    External,
    /// 不信任（沙箱）
    Untrusted,
}

/// 上下文请求
#[derive(Clone, Debug)]

pub struct ContextRequest {
    pub source: String,
    pub trust_level: TrustLevel,
    pub content: String,
    pub requested_actions: Vec<String>,
}

/// 验证结果
#[derive(Clone, Debug)]

pub enum ValidationResult {
    /// 允许
    Allowed,
    /// 拒绝（原因）
    Denied { reason: String },
    /// 降级（降低信任级别后允许）
    Degraded {
        new_trust_level: TrustLevel,
        reason: String,
    },
}

/// 上下文权限边界
#[derive(Clone, Debug)]

pub struct ContextBoundary {
    /// 每个信任级别允许的操作
    allowed_actions: HashMap<TrustLevel, Vec<String>>,
    /// 危险模式
    dangerous_patterns: Vec<String>,
    /// 验证历史
    history: Vec<(ContextRequest, ValidationResult)>,
}


impl ContextBoundary {
    pub fn new() -> Self {
        let mut allowed_actions = HashMap::new();

        // System 级别：所有操作
        allowed_actions.insert(TrustLevel::System, vec!["*".to_string()]);

        // User 级别：大部分操作
        allowed_actions.insert(
            TrustLevel::User,
            vec![
                "read".to_string(),
                "write".to_string(),
                "execute".to_string(),
                "search".to_string(),
                "create".to_string(),
            ],
        );

        // Tool 级别：只读 + 搜索
        allowed_actions.insert(
            TrustLevel::Tool,
            vec!["read".to_string(), "search".to_string()],
        );

        // External 级别：只读
        allowed_actions.insert(TrustLevel::External, vec!["read".to_string()]);

        // Untrusted 级别：无操作
        allowed_actions.insert(TrustLevel::Untrusted, vec![]);

        Self {
            allowed_actions,
            dangerous_patterns: vec![
                "rm -rf".to_string(),
                "DROP TABLE".to_string(),
                "DELETE FROM".to_string(),
                "eval(".to_string(),
                "exec(".to_string(),
                "__import__".to_string(),
            ],
            history: Vec::new(),
        }
    }

    /// 验证上下文请求
    pub fn validate(&mut self, request: &ContextRequest) -> ValidationResult {
        // 检查危险模式
        for pattern in &self.dangerous_patterns {
            if request.content.contains(pattern.as_str()) {
                let result = ValidationResult::Denied {
                    reason: format!("包含危险模式: {}", pattern),
                };
                self.history.push((request.clone(), result.clone()));
                return result;
            }
        }

        // Privilege escalation detection (airgorah pattern)
        // Check if the request is attempting to act beyond its trust level
        if let Some(escalation) = self.detect_privilege_escalation(request) {
            let result = ValidationResult::Denied {
                reason: escalation,
            };
            self.history.push((request.clone(), result.clone()));
            return result;
        }

        // 检查操作权限
        if let Some(allowed) = self.allowed_actions.get(&request.trust_level) {
            if allowed.contains(&"*".to_string()) {
                let result = ValidationResult::Allowed;
                self.history.push((request.clone(), result.clone()));
                return result;
            }

            for action in &request.requested_actions {
                if !allowed.contains(action) {
                    let result = ValidationResult::Denied {
                        reason: format!(
                            "操作 '{}' 不被信任级别 {:?} 允许",
                            action, request.trust_level
                        ),
                    };
                    self.history.push((request.clone(), result.clone()));
                    return result;
                }
            }
        }

        let result = ValidationResult::Allowed;
        self.history.push((request.clone(), result.clone()));
        result
    }

    /// Detect privilege escalation attempts (airgorah polkit pattern).
    ///
    /// An agent running at TrustLevel::Tool cannot escalate to System by
    /// requesting System-level actions. The request must originate from a
    /// higher-trust source — the agent cannot forge its own trust level.
    fn detect_privilege_escalation(&self, request: &ContextRequest) -> Option<String> {
        // Untrusted sources can never request anything
        if request.trust_level == TrustLevel::Untrusted && !request.requested_actions.is_empty() {
            return Some(format!(
                "Privilege escalation blocked: Untrusted source '{}' cannot request any actions",
                request.source
            ));
        }

        // External sources can only read — block any write/execute/delete
        if request.trust_level == TrustLevel::External {
            let mutating = ["write", "execute", "delete", "create", "update"];
            for action in &request.requested_actions {
                if mutating.contains(&action.as_str()) {
                    return Some(format!(
                        "Privilege escalation blocked: External source '{}' cannot perform mutating action '{}'",
                        request.source, action
                    ));
                }
            }
        }

        // Tool sources can only read/search — block write/execute/delete
        if request.trust_level == TrustLevel::Tool {
            let mutating = ["write", "execute", "delete", "create", "update"];
            for action in &request.requested_actions {
                if mutating.contains(&action.as_str()) {
                    return Some(format!(
                        "Privilege escalation blocked: Tool source '{}' cannot perform mutating action '{}'",
                        request.source, action
                    ));
                }
            }
        }

        None
    }

    /// 获取验证历史
    pub fn history(&self) -> &[(ContextRequest, ValidationResult)] {
        &self.history
    }

    /// 获取统计
    pub fn stats(&self) -> (usize, usize, usize) {
        let allowed = self
            .history
            .iter()
            .filter(|(_, r)| matches!(r, ValidationResult::Allowed))
            .count();
        let denied = self
            .history
            .iter()
            .filter(|(_, r)| matches!(r, ValidationResult::Denied { .. }))
            .count();
        let degraded = self
            .history
            .iter()
            .filter(|(_, r)| matches!(r, ValidationResult::Degraded { .. }))
            .count();
        (allowed, denied, degraded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_allows_all() {
        let mut cb = ContextBoundary::new();
        let req = ContextRequest {
            source: "core".into(),
            trust_level: TrustLevel::System,
            content: "anything".into(),
            requested_actions: vec!["write".into(), "delete".into()],
        };
        assert!(matches!(cb.validate(&req), ValidationResult::Allowed));
    }

    #[test]
    fn test_dangerous_pattern_denied() {
        let mut cb = ContextBoundary::new();
        let req = ContextRequest {
            source: "external".into(),
            trust_level: TrustLevel::External,
            content: "run rm -rf /".into(),
            requested_actions: vec!["read".into()],
        };
        assert!(matches!(cb.validate(&req), ValidationResult::Denied { .. }));
    }

    #[test]
    fn test_tool_cannot_write() {
        let mut cb = ContextBoundary::new();
        let req = ContextRequest {
            source: "mcp_tool".into(),
            trust_level: TrustLevel::Tool,
            content: "safe content".into(),
            requested_actions: vec!["write".into()],
        };
        assert!(matches!(cb.validate(&req), ValidationResult::Denied { .. }));
    }

    #[test]
    fn test_stats() {
        let mut cb = ContextBoundary::new();
        let sys_req = ContextRequest {
            source: "s".into(),
            trust_level: TrustLevel::System,
            content: "ok".into(),
            requested_actions: vec![],
        };
        let ext_req = ContextRequest {
            source: "e".into(),
            trust_level: TrustLevel::External,
            content: "rm -rf".into(),
            requested_actions: vec![],
        };
        cb.validate(&sys_req);
        cb.validate(&ext_req);
        let (a, d, _) = cb.stats();
        assert_eq!(a, 1);
        assert_eq!(d, 1);
    }

    #[test]
    fn test_tool_cannot_write_via_privilege_escalation() {
        let mut cb = ContextBoundary::new();
        let req = ContextRequest {
            source: "mcp_tool".into(),
            trust_level: TrustLevel::Tool,
            content: "safe content".into(),
            requested_actions: vec!["write".into()],
        };
        let result = cb.validate(&req);
        match result {
            ValidationResult::Denied { reason } => {
                assert!(reason.contains("Privilege escalation blocked"));
            }
            _ => panic!("Tool requesting write should be denied via privilege escalation"),
        }
    }

    #[test]
    fn test_external_cannot_execute_via_privilege_escalation() {
        let mut cb = ContextBoundary::new();
        let req = ContextRequest {
            source: "external_api".into(),
            trust_level: TrustLevel::External,
            content: "safe content".into(),
            requested_actions: vec!["execute".into()],
        };
        let result = cb.validate(&req);
        assert!(matches!(result, ValidationResult::Denied { .. }));
    }

    #[test]
    fn test_untrusted_blocked_from_all_actions() {
        let mut cb = ContextBoundary::new();
        let req = ContextRequest {
            source: "sandbox".into(),
            trust_level: TrustLevel::Untrusted,
            content: "hello".into(),
            requested_actions: vec!["read".into()],
        };
        let result = cb.validate(&req);
        assert!(matches!(result, ValidationResult::Denied { .. }));
    }
}
