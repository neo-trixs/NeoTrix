//! 能力安全特性
//!
//! 提供能力调用的安全控制和访问管理

use crate::core::nt_core_capability::*;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// 安全策略
#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    /// 启用认证
    pub enable_authentication: bool,
    /// 启用授权
    pub enable_authorization: bool,
    /// 启用加密
    pub enable_encryption: bool,
    /// 启用审计
    pub enable_audit: bool,
    /// 最大请求速率
    pub max_request_rate: u32,
    /// 请求窗口
    pub request_window: Duration,
    /// IP白名单
    pub ip_whitelist: Vec<String>,
    /// IP黑名单
    pub ip_blacklist: Vec<String>,
    /// 能力白名单
    pub capability_whitelist: Vec<String>,
    /// 能力黑名单
    pub capability_blacklist: Vec<String>,
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            enable_authentication: true,
            enable_authorization: true,
            enable_encryption: true,
            enable_audit: true,
            max_request_rate: 100,
            request_window: Duration::from_secs(60),
            ip_whitelist: Vec::new(),
            ip_blacklist: Vec::new(),
            capability_whitelist: Vec::new(),
            capability_blacklist: Vec::new(),
        }
    }
}

/// 访问令牌
#[derive(Debug, Clone)]
pub struct AccessToken {
    /// 令牌ID
    pub id: String,
    /// 用户ID
    pub user_id: String,
    /// 权限列表
    pub permissions: Vec<String>,
    /// 过期时间
    pub expires_at: Instant,
    /// 签名
    pub signature: String,
}

/// 审计日志
#[derive(Debug, Clone)]
pub struct AuditLog {
    /// 时间戳
    pub timestamp: Instant,
    /// 用户ID
    pub user_id: String,
    /// 能力ID
    pub capability_id: String,
    /// 操作
    pub action: String,
    /// 结果
    pub result: AuditResult,
    /// IP地址
    pub ip_address: String,
    /// 元数据
    pub metadata: std::collections::HashMap<String, String>,
}

/// 审计结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditResult {
    Success,
    Denied,
    Error,
}

/// 速率限制器
pub struct RateLimiter {
    /// 请求计数
    counts: std::collections::HashMap<String, Vec<Instant>>,
    /// 配置
    max_requests: u32,
    /// 窗口
    window: Duration,
}

impl RateLimiter {
    /// 创建新的速率限制器
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            counts: std::collections::HashMap::new(),
            max_requests,
            window,
        }
    }

    /// 检查请求是否允许
    pub fn check(&mut self, key: &str) -> bool {
        let now = Instant::now();
        let requests = self.counts.entry(key.to_string()).or_insert_with(Vec::new);

        // 清理过期请求
        requests.retain(|t| now.duration_since(*t) < self.window);

        if requests.len() >= self.max_requests as usize {
            false
        } else {
            requests.push(now);
            true
        }
    }

    /// 获取当前请求计数
    pub fn current_count(&self, key: &str) -> usize {
        self.counts.get(key).map(|r| r.len()).unwrap_or(0)
    }

    /// 重置计数
    pub fn reset(&mut self, key: &str) {
        self.counts.remove(key);
    }
}

/// 安全管理器
pub struct SecurityManager {
    /// 策略
    policy: SecurityPolicy,
    /// 令牌存储
    tokens: std::collections::HashMap<String, AccessToken>,
    /// 审计日志
    audit_logs: Vec<AuditLog>,
    /// 速率限制器
    rate_limiter: RateLimiter,
    /// 最大审计日志数
    max_audit_logs: usize,
}

impl SecurityManager {
    /// 创建新的安全管理器
    pub fn new(policy: SecurityPolicy) -> Self {
        let rate_limiter = RateLimiter::new(policy.max_request_rate, policy.request_window);

        Self {
            policy,
            tokens: std::collections::HashMap::new(),
            audit_logs: Vec::new(),
            rate_limiter,
            max_audit_logs: 10000,
        }
    }

    /// 验证令牌
    pub fn validate_token(&self, token_id: &str) -> Option<&AccessToken> {
        let token = self.tokens.get(token_id)?;
        if Instant::now() < token.expires_at {
            Some(token)
        } else {
            None
        }
    }

    /// 检查访问权限
    pub fn check_access(&self, user_id: &str, capability_id: &str) -> bool {
        // 检查能力黑名单
        if self
            .policy
            .capability_blacklist
            .contains(&capability_id.to_string())
        {
            return false;
        }

        // 检查能力白名单（如果配置了白名单）
        if !self.policy.capability_whitelist.is_empty()
            && !self
                .policy
                .capability_whitelist
                .contains(&capability_id.to_string())
        {
            return false;
        }

        // 检查用户权限
        for token in self.tokens.values() {
            if token.user_id == user_id
                && token
                    .permissions
                    .contains(&format!("capability:{}", capability_id))
            {
                return true;
            }
        }

        false
    }

    /// 检查速率限制
    pub fn check_rate_limit(&mut self, key: &str) -> bool {
        self.rate_limiter.check(key)
    }

    /// 记录审计日志
    pub fn audit(&mut self, log: AuditLog) {
        if self.policy.enable_audit && self.audit_logs.len() < self.max_audit_logs {
            self.audit_logs.push(log);
        }
    }

    /// 获取审计日志
    pub fn get_audit_logs(&self, count: usize) -> Vec<&AuditLog> {
        self.audit_logs.iter().rev().take(count).collect()
    }

    /// 获取用户审计日志
    pub fn get_user_audit_logs(&self, user_id: &str, count: usize) -> Vec<&AuditLog> {
        self.audit_logs
            .iter()
            .filter(|l| l.user_id == user_id)
            .rev()
            .take(count)
            .collect()
    }

    /// 清理旧审计日志
    pub fn cleanup_audit_logs(&mut self, max_age: Duration) {
        let cutoff = Instant::now() - max_age;
        self.audit_logs.retain(|l| l.timestamp > cutoff);
    }

    /// 添加令牌
    pub fn add_token(&mut self, token: AccessToken) {
        self.tokens.insert(token.id.clone(), token);
    }

    /// 移除令牌
    pub fn remove_token(&mut self, token_id: &str) {
        self.tokens.remove(token_id);
    }

    /// 检查IP是否允许
    pub fn check_ip(&self, ip: &str) -> bool {
        // 检查黑名单
        if self.policy.ip_blacklist.contains(&ip.to_string()) {
            return false;
        }

        // 检查白名单（如果配置了白名单）
        if !self.policy.ip_whitelist.is_empty()
            && !self.policy.ip_whitelist.contains(&ip.to_string())
        {
            return false;
        }

        true
    }
}

/// 安全包装器
pub struct SecureCapability {
    /// 底层能力
    capability: Arc<dyn UnifiedCapability>,
    /// 安全管理器
    security_manager: Arc<std::sync::Mutex<SecurityManager>>,
}

impl SecureCapability {
    /// 创建安全包装器
    pub fn new(
        capability: Arc<dyn UnifiedCapability>,
        security_manager: Arc<std::sync::Mutex<SecurityManager>>,
    ) -> Self {
        Self {
            capability,
            security_manager,
        }
    }

    /// 带安全的执行
    pub fn execute_secure(
        &self,
        input: CapabilityInput,
        token_id: &str,
        ip_address: &str,
    ) -> Result<CapabilityOutput, CapabilityError> {
        let mut manager = self.security_manager.lock().unwrap();

        // 验证令牌
        if manager.policy.enable_authentication {
            let token = manager
                .validate_token(token_id)
                .ok_or_else(|| CapabilityError::ExecutionFailed("无效的访问令牌".into()))?;

            // 检查访问权限
            if manager.policy.enable_authorization
                && !manager.check_access(&token.user_id, &self.capability.meta().id)
            {
                return Err(CapabilityError::ExecutionFailed("访问被拒绝".into()));
            }
        }

        // 检查IP
        if !manager.check_ip(ip_address) {
            return Err(CapabilityError::ExecutionFailed("IP地址被拒绝".into()));
        }

        // 检查速率限制
        if !manager.check_rate_limit(ip_address) {
            return Err(CapabilityError::ExecutionFailed("请求速率超限".into()));
        }

        drop(manager);

        // 执行能力
        self.capability.execute(input)
    }
}

impl UnifiedCapability for SecureCapability {
    fn meta(&self) -> CapabilityMeta {
        self.capability.meta()
    }

    fn health(&self) -> CapabilityHealth {
        self.capability.health()
    }

    fn execute(&self, input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError> {
        self.capability.execute(input)
    }

    fn supports(&self, input: &CapabilityInput) -> bool {
        self.capability.supports(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_limiter() {
        let mut limiter = RateLimiter::new(5, Duration::from_secs(60));

        for _ in 0..5 {
            assert!(limiter.check("test"));
        }

        assert!(!limiter.check("test"));
    }

    #[test]
    fn security_manager() {
        let policy = SecurityPolicy::default();
        let mut manager = SecurityManager::new(policy);

        let token = AccessToken {
            id: "token_1".into(),
            user_id: "user_1".into(),
            permissions: vec!["capability:test".into()],
            expires_at: Instant::now() + Duration::from_secs(3600),
            signature: "sig".into(),
        };

        manager.add_token(token);
        assert!(manager.validate_token("token_1").is_some());
    }

    #[test]
    fn ip_check() {
        let policy = SecurityPolicy {
            ip_whitelist: vec!["192.168.1.0/24".into()],
            ..Default::default()
        };
        let manager = SecurityManager::new(policy);

        assert!(manager.check_ip("192.168.1.1"));
        assert!(!manager.check_ip("10.0.0.1"));
    }
}
