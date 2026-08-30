//! Confirmation Gate — browser-act/skills Confirmation Gate 协议吸收
//! 
//! 显式用户确认门控：浏览器创建/删除/敏感操作需用户批准
//! 对标 EgressPolicy (P2) + nt_shield_approval

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

/// 确认请求类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConfirmationType {
    BrowserCreation,
    BrowserDeletion,
    Login,
    FormSubmission,
    FileUpload,
    SensitiveNavigation,
    ProfileSwitch,
}

/// 确认请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationRequest {
    pub id: String,
    pub request_type: ConfirmationType,
    pub description: String,
    pub details: HashMap<String, String>,
    pub requested_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub status: ConfirmationStatus,
}

/// 确认状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConfirmationStatus {
    Pending,
    Approved,
    Denied,
    Expired,
}

/// 确认策略 (allow/deny-wins + default_allow fallback)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationPolicy {
    pub request_type: ConfirmationType,
    pub default_action: DefaultAction,
    pub require_explicit: bool,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DefaultAction {
    Allow,
    Deny,
}

/// 确认门控引擎
#[derive(Debug)]
pub struct ConfirmationGate {
    pending_requests: Arc<RwLock<HashMap<String, ConfirmationRequest>>>,
    policies: HashMap<ConfirmationType, ConfirmationPolicy>,
    approval_callback: Option<Arc<dyn Fn(ConfirmationRequest) -> Result<bool, String> + Send + Sync>>,
}

impl ConfirmationGate {
    pub fn new() -> Self {
        let mut policies = HashMap::new();
        
        // 默认策略：敏感操作默认拒绝，需显式批准
        policies.insert(ConfirmationType::BrowserCreation, ConfirmationPolicy {
            request_type: ConfirmationType::BrowserCreation,
            default_action: DefaultAction::Deny,
            require_explicit: true,
            timeout_seconds: 60,
        });
        policies.insert(ConfirmationType::BrowserDeletion, ConfirmationPolicy {
            request_type: ConfirmationType::BrowserDeletion,
            default_action: DefaultAction::Deny,
            require_explicit: true,
            timeout_seconds: 60,
        });
        policies.insert(ConfirmationType::Login, ConfirmationPolicy {
            request_type: ConfirmationType::Login,
            default_action: DefaultAction::Deny,
            require_explicit: true,
            timeout_seconds: 120,
        });
        policies.insert(ConfirmationType::FormSubmission, ConfirmationPolicy {
            request_type: ConfirmationType::FormSubmission,
            default_action: DefaultAction::Deny,
            require_explicit: true,
            timeout_seconds: 30,
        });
        policies.insert(ConfirmationType::FileUpload, ConfirmationPolicy {
            request_type: ConfirmationType::FileUpload,
            default_action: DefaultAction::Deny,
            require_explicit: true,
            timeout_seconds: 30,
        });
        policies.insert(ConfirmationType::SensitiveNavigation, ConfirmationPolicy {
            request_type: ConfirmationType::SensitiveNavigation,
            default_action: DefaultAction::Deny,
            require_explicit: true,
            timeout_seconds: 30,
        });
        policies.insert(ConfirmationType::ProfileSwitch, ConfirmationPolicy {
            request_type: ConfirmationType::ProfileSwitch,
            default_action: DefaultAction::Allow, // 切换配置文件相对安全
            require_explicit: false,
            timeout_seconds: 10,
        });

        Self {
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
            policies,
            approval_callback: None,
        }
    }

    /// 设置批准回调 (用于集成外部确认 UI)
    pub fn set_approval_callback<F>(&mut self, callback: F)
    where
        F: Fn(ConfirmationRequest) -> Result<bool, String> + Send + Sync + 'static,
    {
        self.approval_callback = Some(Arc::new(callback));
    }

    /// 请求确认
    pub async fn request_confirmation(
        &self,
        request_type: ConfirmationType,
        description: &str,
        details: HashMap<String, String>,
    ) -> Result<bool, String> {
        let policy = self.policies.get(&request_type).ok_or("No policy for request type")?;
        
        let request = ConfirmationRequest {
            id: uuid::Uuid::new_v4().to_string(),
            request_type,
            description: description.to_string(),
            details,
            requested_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + chrono::Duration::seconds(policy.timeout_seconds as i64),
            status: ConfirmationStatus::Pending,
        };

        let request_id = request.id.clone();
        
        // 存储待处理请求
        {
            let mut pending = self.pending_requests.write().await;
            pending.insert(request_id.clone(), request.clone());
        }

        // 如果不需要显式确认且默认允许，自动通过
        if !policy.require_explicit && policy.default_action == DefaultAction::Allow {
            self.update_status(&request_id, ConfirmationStatus::Approved).await;
            return Ok(true);
        }

        // 如果有回调，使用回调
        if let Some(ref callback) = self.approval_callback {
            let result = callback(request.clone())?;
            let status = if result { ConfirmationStatus::Approved } else { ConfirmationStatus::Denied };
            self.update_status(&request_id, status).await;
            return Ok(result);
        }

        // 否则等待外部处理 (在实际集成中，这会通过事件总线通知 UI)
        // 这里简化：返回需要外部处理
        Err("Confirmation required - no callback registered".to_string())
    }

    /// 外部处理确认结果
    pub async fn respond_confirmation(&self, request_id: &str, approved: bool) -> Result<(), String> {
        let status = if approved { ConfirmationStatus::Approved } else { ConfirmationStatus::Denied };
        self.update_status(request_id, status).await;
        Ok(())
    }

    async fn update_status(&self, request_id: &str, status: ConfirmationStatus) {
        let mut pending = self.pending_requests.write().await;
        if let Some(req) = pending.get_mut(request_id) {
            req.status = status;
        }
    }

    /// 获取待处理请求
    pub async fn get_pending(&self) -> Vec<ConfirmationRequest> {
        let pending = self.pending_requests.read().await;
        pending.values().cloned().collect()
    }

    /// 清理过期请求
    pub async fn cleanup_expired(&self) {
        let mut pending = self.pending_requests.write().await;
        let now = chrono::Utc::now();
        pending.retain(|_, req| {
            if req.expires_at < now && req.status == ConfirmationStatus::Pending {
                req.status = ConfirmationStatus::Expired;
                false
            } else {
                true
            }
        });
    }

    /// 更新策略
    pub fn set_policy(&mut self, policy: ConfirmationPolicy) {
        self.policies.insert(policy.request_type, policy);
    }

    /// SelfTest for C1 promotion
    pub fn self_test() -> Result<(), String> {
        let gate = ConfirmationGate::new();
        
        // Test 1: Default policies exist
        assert!(gate.policies.contains_key(&ConfirmationType::BrowserCreation));
        assert!(gate.policies.contains_key(&ConfirmationType::Login));
        assert!(gate.policies.contains_key(&ConfirmationType::ProfileSwitch));
        
        // Test 2: Sensitive ops default deny
        let browser_policy = gate.policies.get(&ConfirmationType::BrowserCreation).unwrap();
        assert_eq!(browser_policy.default_action, DefaultAction::Deny);
        assert!(browser_policy.require_explicit);
        
        // Test 3: Profile switch default allow
        let profile_policy = gate.policies.get(&ConfirmationType::ProfileSwitch).unwrap();
        assert_eq!(profile_policy.default_action, DefaultAction::Allow);
        assert!(!profile_policy.require_explicit);
        
        // Test 4: Policy update
        let mut gate = ConfirmationGate::new();
        gate.set_policy(ConfirmationPolicy {
            request_type: ConfirmationType::FormSubmission,
            default_action: DefaultAction::Allow,
            require_explicit: false,
            timeout_seconds: 10,
        });
        let policy = gate.policies.get(&ConfirmationType::FormSubmission).unwrap();
        assert_eq!(policy.default_action, DefaultAction::Allow);
        assert!(!policy.require_explicit);

        Ok(())
    }
}

impl Default for ConfirmationGate {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_policies() {
        let gate = ConfirmationGate::new();
        assert!(gate.policies.contains_key(&ConfirmationType::BrowserCreation));
        assert!(gate.policies.contains_key(&ConfirmationType::Login));
    }

    #[test]
    fn test_sensitive_ops_deny_by_default() {
        let gate = ConfirmationGate::new();
        let policy = gate.policies.get(&ConfirmationType::BrowserCreation).unwrap();
        assert_eq!(policy.default_action, DefaultAction::Deny);
        assert!(policy.require_explicit);
    }

    #[test]
    fn test_profile_switch_allow_by_default() {
        let gate = ConfirmationGate::new();
        let policy = gate.policies.get(&ConfirmationType::ProfileSwitch).unwrap();
        assert_eq!(policy.default_action, DefaultAction::Allow);
        assert!(!policy.require_explicit);
    }

    #[test]
    fn test_policy_update() {
        let mut gate = ConfirmationGate::new();
        gate.set_policy(ConfirmationPolicy {
            request_type: ConfirmationType::FormSubmission,
            default_action: DefaultAction::Allow,
            require_explicit: false,
            timeout_seconds: 10,
        });
        let policy = gate.policies.get(&ConfirmationType::FormSubmission).unwrap();
        assert_eq!(policy.default_action, DefaultAction::Allow);
    }

    #[test]
    fn test_self_test_passes() {
        assert!(ConfirmationGate::self_test().is_ok());
    }
}