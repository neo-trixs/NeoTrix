//! C5: NIST ZTA Policy Engine (SANS-IO)
//!
//! NIST SP 800-207 零信任架构策略引擎实现。
//! PE (Policy Engine) + PA (Policy Administrator) + PEP (Policy Enforcement Point)。

use std::collections::HashMap;
use std::time::Instant;

/// 主体身份
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubjectIdentity {
    /// 用户ID
    pub user_id: Option<String>,
    /// 设备ID
    pub device_id: Option<String>,
    /// 设备证书指纹
    pub device_cert_fingerprint: Option<String>,
    /// IP地址
    pub source_ip: String,
    /// 认证方式
    pub auth_method: AuthMethod,
    /// 认证时间
    pub auth_time: Option<Instant>,
}

/// 认证方式
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthMethod {
    /// 无认证
    None,
    /// 密码
    Password,
    /// 证书
    Certificate,
    /// MFA
    Mfa,
    /// 服务账户
    ServiceAccount,
}

/// 资源
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Resource {
    /// 资源类型
    pub resource_type: ResourceType,
    /// 资源路径
    pub path: String,
    /// 标签
    pub labels: HashMap<String, String>,
}

/// 资源类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceType {
    /// 隧道
    Tunnel,
    /// 服务
    Service,
    /// 数据
    Data,
}

/// 动作
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// 允许
    Allow,
    /// 拒绝
    Deny,
    /// 需要额外认证
    RequireAuth,
    /// 限制 (带宽/时间)
    RateLimit { max_bytes_per_sec: u64 },
}

/// 策略规则
#[derive(Debug, Clone)]
pub struct PolicyRule {
    /// 规则ID
    pub id: String,
    /// 优先级 (越小越高)
    pub priority: i32,
    /// 条件
    pub condition: PolicyCondition,
    /// 动作
    pub action: Action,
    /// 描述
    pub description: String,
}

/// 策略条件
#[derive(Debug, Clone)]
pub enum PolicyCondition {
    /// 所有条件满足
    And(Vec<PolicyCondition>),
    /// 任一条件满足
    Or(Vec<PolicyCondition>),
    /// 非
    Not(Box<PolicyCondition>),
    /// 主体匹配
    SubjectMatches { field: String, value: String },
    /// 资源匹配
    ResourceMatches { path_pattern: String },
    /// 时间范围
    TimeRange { start_hour: u8, end_hour: u8 },
    /// 位置
    Location { country: String },
    /// 设备健康
    DeviceHealth { min_score: u8 },
}

/// 策略决策
#[derive(Debug, Clone)]
pub struct PolicyDecision {
    /// 动作
    pub action: Action,
    /// 匹配的规则ID
    pub matched_rule: Option<String>,
    /// 决策原因
    pub reason: String,
    /// 附加属性
    pub attributes: HashMap<String, String>,
}

/// 策略引擎
pub struct PolicyEngine {
    /// 规则库
    rules: Vec<PolicyRule>,
    /// 默认动作
    default_action: Action,
    /// 审计日志
    audit_log: Vec<AuditEntry>,
}

/// 审计条目
#[derive(Debug, Clone)]
pub struct AuditEntry {
    /// 时间戳
    pub timestamp: Instant,
    /// 主体
    pub subject: SubjectIdentity,
    /// 资源
    pub resource: Resource,
    /// 动作
    pub action: Action,
    /// 匹配规则
    pub matched_rule: Option<String>,
}

impl PolicyEngine {
    /// 创建新的策略引擎
    pub fn new(default_action: Action) -> Self {
        Self {
            rules: Vec::new(),
            default_action,
            audit_log: Vec::new(),
        }
    }

    /// 添加规则
    pub fn add_rule(&mut self, rule: PolicyRule) {
        self.rules.push(rule);
        // 按优先级排序
        self.rules.sort_by(|a, b| a.priority.cmp(&b.priority));
    }

    /// 评估策略
    pub fn evaluate(
        &mut self,
        subject: &SubjectIdentity,
        resource: &Resource,
        action: &Action,
    ) -> PolicyDecision {
        // 按优先级检查规则
        for rule in &self.rules {
            if self.evaluate_condition(&rule.condition, subject, resource, action) {
                let decision = PolicyDecision {
                    action: rule.action.clone(),
                    matched_rule: Some(rule.id.clone()),
                    reason: rule.description.clone(),
                    attributes: HashMap::new(),
                };

                // 记录审计
                self.audit_log.push(AuditEntry {
                    timestamp: Instant::now(),
                    subject: subject.clone(),
                    resource: resource.clone(),
                    action: decision.action.clone(),
                    matched_rule: decision.matched_rule.clone(),
                });

                return decision;
            }
        }

        // 无匹配规则，使用默认动作
        PolicyDecision {
            action: self.default_action.clone(),
            matched_rule: None,
            reason: "No matching rule, using default".into(),
            attributes: HashMap::new(),
        }
    }

    /// 评估条件
    fn evaluate_condition(
        &self,
        condition: &PolicyCondition,
        subject: &SubjectIdentity,
        resource: &Resource,
        action: &Action,
    ) -> bool {
        match condition {
            PolicyCondition::And(conditions) => {
                conditions.iter().all(|c| self.evaluate_condition(c, subject, resource, action))
            }
            PolicyCondition::Or(conditions) => {
                conditions.iter().any(|c| self.evaluate_condition(c, subject, resource, action))
            }
            PolicyCondition::Not(inner) => {
                !self.evaluate_condition(inner, subject, resource, action)
            }
            PolicyCondition::SubjectMatches { field, value } => {
                match field.as_str() {
                    "user_id" => subject.user_id.as_deref() == Some(value.as_str()),
                    "device_id" => subject.device_id.as_deref() == Some(value.as_str()),
                    "auth_method" => format!("{:?}", subject.auth_method) == *value,
                    _ => false,
                }
            }
            PolicyCondition::ResourceMatches { path_pattern } => {
                resource.path.contains(path_pattern.as_str())
            }
            PolicyCondition::TimeRange { start_hour, end_hour } => {
                // 简化实现：总是返回true
                true
            }
            PolicyCondition::Location { country } => {
                // 简化实现：总是返回true
                true
            }
            PolicyCondition::DeviceHealth { min_score } => {
                // 简化实现：总是返回true
                true
            }
        }
    }

    /// 获取审计日志
    pub fn audit_log(&self) -> &[AuditEntry] {
        &self.audit_log
    }

    /// 清空审计日志
    pub fn clear_audit_log(&mut self) {
        self.audit_log.clear();
    }
}

/// 策略管理员 (Policy Administrator)
pub struct PolicyAdministrator {
    /// 策略引擎引用
    engine: PolicyEngine,
    /// 会话密钥
    session_keys: HashMap<String, SessionKey>,
}

/// 会话密钥
#[derive(Debug, Clone)]
pub struct SessionKey {
    /// 密钥
    pub key: [u8; 32],
    /// 创建时间
    pub created_at: Instant,
    /// 过期时间
    pub expires_at: Instant,
}

impl PolicyAdministrator {
    /// 创建新的策略管理员
    pub fn new(engine: PolicyEngine) -> Self {
        Self {
            engine,
            session_keys: HashMap::new(),
        }
    }

    /// 创建会话
    pub fn create_session(
        &mut self,
        subject: &SubjectIdentity,
        resource: &Resource,
    ) -> Option<SessionKey> {
        let decision = self.engine.evaluate(subject, resource, &Action::Allow);

        match decision.action {
            Action::Allow => {
                let key = SessionKey {
                    key: generate_session_key(),
                    created_at: Instant::now(),
                    expires_at: Instant::now() + std::time::Duration::from_secs(3600),
                };

                let session_id = format!("{}-{}", subject.device_id.as_deref().unwrap_or("unknown"), resource.path);
                self.session_keys.insert(session_id, key.clone());

                Some(key)
            }
            _ => None,
        }
    }

    /// 验证会话
    pub fn validate_session(&self, session_id: &str) -> bool {
        if let Some(key) = self.session_keys.get(session_id) {
            Instant::now() < key.expires_at
        } else {
            false
        }
    }

    /// 获取策略引擎引用
    pub fn engine(&self) -> &PolicyEngine {
        &self.engine
    }

    /// 获取可变策略引擎引用
    pub fn engine_mut(&mut self) -> &mut PolicyEngine {
        &mut self.engine
    }
}

/// 生成会话密钥
fn generate_session_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    for byte in &mut key {
        *byte = rand::random();
    }
    key
}

/// 策略执行点 (PEP) 配置
#[derive(Debug, Clone)]
pub struct PepConfig {
    /// 允许的网络范围
    pub allowed_networks: Vec<String>,
    /// 最大并发连接
    pub max_connections: usize,
    /// 会话超时
    pub session_timeout: std::time::Duration,
    /// 是否启用审计
    pub audit_enabled: bool,
}

impl Default for PepConfig {
    fn default() -> Self {
        Self {
            allowed_networks: vec!["0.0.0.0/0".into()],
            max_connections: 1000,
            session_timeout: std::time::Duration::from_secs(3600),
            audit_enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_engine_creation() {
        let engine = PolicyEngine::new(Action::Deny);
        assert_eq!(engine.default_action, Action::Deny);
    }

    #[test]
    fn evaluate_allow_rule() {
        let mut engine = PolicyEngine::new(Action::Deny);

        engine.add_rule(PolicyRule {
            id: "allow-admin".into(),
            priority: 100,
            condition: PolicyCondition::SubjectMatches {
                field: "user_id".into(),
                value: "admin".into(),
            },
            action: Action::Allow,
            description: "Allow admin users".into(),
        });

        let subject = SubjectIdentity {
            user_id: Some("admin".into()),
            device_id: None,
            device_cert_fingerprint: None,
            source_ip: "192.168.1.1".into(),
            auth_method: AuthMethod::Password,
            auth_time: None,
        };

        let resource = Resource {
            resource_type: ResourceType::Tunnel,
            path: "/tunnel/test".into(),
            labels: HashMap::new(),
        };

        let decision = engine.evaluate(&subject, &resource, &Action::Allow);
        assert!(matches!(decision.action, Action::Allow));
        assert_eq!(decision.matched_rule, Some("allow-admin".into()));
    }

    #[test]
    fn policy_administrator_session() {
        let mut engine = PolicyEngine::new(Action::Deny);
        engine.add_rule(PolicyRule {
            id: "allow-user".into(),
            priority: 100,
            condition: PolicyCondition::SubjectMatches {
                field: "user_id".into(),
                value: "user1".into(),
            },
            action: Action::Allow,
            description: "Allow user1".into(),
        });

        let mut admin = PolicyAdministrator::new(engine);

        let subject = SubjectIdentity {
            user_id: Some("user1".into()),
            device_id: Some("device-001".into()),
            device_cert_fingerprint: None,
            source_ip: "192.168.1.1".into(),
            auth_method: AuthMethod::Password,
            auth_time: None,
        };

        let resource = Resource {
            resource_type: ResourceType::Service,
            path: "/api/data".into(),
            labels: HashMap::new(),
        };

        let session = admin.create_session(&subject, &resource);
        assert!(session.is_some());
    }
}
