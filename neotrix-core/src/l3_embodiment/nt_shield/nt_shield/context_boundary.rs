//! 上下文权限边界 — 防止 CPE 攻击
//!
//! 验证上下文来源，阻止权限提升。

use std::collections::HashMap;

/// 上下文来源信任级别
#[allow(dead_code)]
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
#[allow(dead_code)]
pub struct ContextRequest {
    pub source: String,
    pub trust_level: TrustLevel,
    pub content: String,
    pub requested_actions: Vec<String>,
}

/// 验证结果
#[allow(dead_code)]
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
#[allow(dead_code)]
pub struct ContextBoundary {
    /// 每个信任级别允许的操作
    allowed_actions: HashMap<TrustLevel, Vec<String>>,
    /// 危险模式
    dangerous_patterns: Vec<String>,
    /// 验证历史
    history: Vec<(ContextRequest, ValidationResult)>,
}

#[allow(dead_code)]
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

impl Clone for ContextRequest {
    fn clone(&self) -> Self {
        Self {
            source: self.source.clone(),
            trust_level: match self.trust_level {
                TrustLevel::System => TrustLevel::System,
                TrustLevel::User => TrustLevel::User,
                TrustLevel::Tool => TrustLevel::Tool,
                TrustLevel::External => TrustLevel::External,
                TrustLevel::Untrusted => TrustLevel::Untrusted,
            },
            content: self.content.clone(),
            requested_actions: self.requested_actions.clone(),
        }
    }
}

impl Clone for ValidationResult {
    fn clone(&self) -> Self {
        match self {
            ValidationResult::Allowed => ValidationResult::Allowed,
            ValidationResult::Denied { reason } => ValidationResult::Denied {
                reason: reason.clone(),
            },
            ValidationResult::Degraded {
                new_trust_level,
                reason,
            } => ValidationResult::Degraded {
                new_trust_level: match new_trust_level {
                    TrustLevel::System => TrustLevel::System,
                    TrustLevel::User => TrustLevel::User,
                    TrustLevel::Tool => TrustLevel::Tool,
                    TrustLevel::External => TrustLevel::External,
                    TrustLevel::Untrusted => TrustLevel::Untrusted,
                },
                reason: reason.clone(),
            },
        }
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
        assert!(matches!(
            cb.validate(&req),
            ValidationResult::Denied { .. }
        ));
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
        assert!(matches!(
            cb.validate(&req),
            ValidationResult::Denied { .. }
        ));
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
}
