use std::collections::{HashMap, HashSet};

/// 策略评估结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyDecision {
    Allow,
    RequireConfirmation,
    Deny,
}

/// 安全配置文件
#[derive(Debug, Clone)]
pub struct ActionPolicy {
    rules: HashMap<String, PolicyDecision>,
    /// 当前安全配置文件 (nt_shield / strict-nt_shield / general)
    pub profile: String,
    /// 网络请求域名白名单（自动放行的 LLM API 域名）
    network_allowlist: HashSet<String>,
}

/// LLM API 提供商默认域名白名单
fn default_llm_domains() -> HashSet<String> {
    [
        "api.openai.com",
        "api.anthropic.com",
        "generativelanguage.googleapis.com",
        "api.deepseek.com",
        "api.groq.com",
        "api.together.xyz",
        "api.mistral.ai",
        "api.cohere.ai",
        "api.openrouter.ai",
        "api.fireworks.ai",
        "oapi.safety.cloud",
    ]
    .into_iter()
    .map(|s| s.to_string())
    .collect()
}

impl ActionPolicy {
    pub fn new() -> Self {
        let mut rules = HashMap::new();
        // 默认安全规则 — network_request 默认 Deny（需要白名单放行）
        rules.insert("read_file".to_string(), PolicyDecision::Allow);
        rules.insert("write_file".to_string(), PolicyDecision::RequireConfirmation);
        rules.insert("execute_command".to_string(), PolicyDecision::RequireConfirmation);
        rules.insert("network_request".to_string(), PolicyDecision::Deny);
        rules.insert("read_secrets".to_string(), PolicyDecision::Deny);
        rules.insert("compile_check".to_string(), PolicyDecision::Allow);
        rules.insert("git_push".to_string(), PolicyDecision::RequireConfirmation);
        rules.insert("git_force_push".to_string(), PolicyDecision::Deny);
        rules.insert("delete_file".to_string(), PolicyDecision::RequireConfirmation);
        rules.insert("modify_dependency".to_string(), PolicyDecision::RequireConfirmation);
        rules.insert("access_nt_world_browse_auto".to_string(), PolicyDecision::RequireConfirmation);
        rules.insert("access_tor_network".to_string(), PolicyDecision::RequireConfirmation);

        Self {
            rules,
            profile: "nt_shield".to_string(),
            network_allowlist: default_llm_domains(),
        }
    }

    /// 设置安全配置文件
    pub fn set_profile(&mut self, profile: &str) {
        self.profile = profile.to_string();
        match profile {
            "strict-nt_shield" => {
                self.rules.insert("read_file".to_string(), PolicyDecision::Allow);
                self.rules.insert("write_file".to_string(), PolicyDecision::Deny);
                self.rules.insert("execute_command".to_string(), PolicyDecision::RequireConfirmation);
                self.rules.insert("network_request".to_string(), PolicyDecision::RequireConfirmation);
                self.rules.insert("access_tor_network".to_string(), PolicyDecision::Deny);
            }
            "general" => {
                self.rules.insert("write_file".to_string(), PolicyDecision::Allow);
                self.rules.insert("execute_command".to_string(), PolicyDecision::Allow);
                self.rules.insert("network_request".to_string(), PolicyDecision::Allow);
            }
            "network-isolated" => {
                // 仅允许 LLM API 域名 + 显式白名单
                self.rules.insert("network_request".to_string(), PolicyDecision::Deny);
                self.rules.insert("write_file".to_string(), PolicyDecision::RequireConfirmation);
                self.rules.insert("execute_command".to_string(), PolicyDecision::RequireConfirmation);
            }
            _ => {}
        }
    }

    /// 动态添加自定义规则
    pub fn add_rule(&mut self, action: &str, decision: PolicyDecision) {
        self.rules.insert(action.to_string(), decision);
    }

    /// 将域名加入网络白名单
    pub fn allowlist_domain(&mut self, domain: &str) {
        self.network_allowlist.insert(domain.to_string());
    }

    /// 从网络白名单移除域名
    pub fn remove_domain(&mut self, domain: &str) {
        self.network_allowlist.remove(domain);
    }

    /// 检查域名是否在白名单中
    pub fn is_domain_allowed(&self, domain: &str) -> bool {
        self.network_allowlist.contains(domain)
            || self.network_allowlist.iter().any(|d| domain.ends_with(&format!(".{}", d)) || domain == d)
    }

    /// 获取白名单引用
    pub fn network_allowlist(&self) -> &HashSet<String> {
        &self.network_allowlist
    }

    /// 评估网络请求 — 先检查域名白名单，再查规则表
    pub fn evaluate_network(&self, domain: &str) -> PolicyDecision {
        if self.is_domain_allowed(domain) {
            return PolicyDecision::Allow;
        }
        self.decide("network_request")
    }

    /// 评估某个操作是否允许
    pub fn evaluate(&self, action: &str) -> bool {
        match self.rules.get(action) {
            Some(PolicyDecision::Allow) => true,
            Some(PolicyDecision::RequireConfirmation) => true,
            Some(PolicyDecision::Deny) => false,
            None => false,
        }
    }

    /// 获取操作决策
    pub fn decide(&self, action: &str) -> PolicyDecision {
        self.rules.get(action).cloned().unwrap_or(PolicyDecision::Deny)
    }

    pub fn action_count(&self) -> usize {
        self.rules.len()
    }
}

impl Default for ActionPolicy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_policy_new_has_default_rules() {
        let p = ActionPolicy::new();
        assert!(p.action_count() > 5);
        assert_eq!(p.profile, "nt_shield");
    }

    #[test]
    fn test_evaluate_allow_returns_true() {
        let p = ActionPolicy::new();
        assert!(p.evaluate("read_file"));
    }

    #[test]
    fn test_evaluate_deny_returns_false() {
        let p = ActionPolicy::new();
        assert!(!p.evaluate("read_secrets"));
    }

    #[test]
    fn test_decide_allow() {
        let p = ActionPolicy::new();
        assert_eq!(p.decide("read_file"), PolicyDecision::Allow);
    }

    #[test]
    fn test_decide_deny() {
        let p = ActionPolicy::new();
        assert_eq!(p.decide("git_force_push"), PolicyDecision::Deny);
    }

    #[test]
    fn test_decide_require_confirmation() {
        let p = ActionPolicy::new();
        assert_eq!(p.decide("write_file"), PolicyDecision::RequireConfirmation);
    }

    #[test]
    fn test_network_default_deny() {
        let p = ActionPolicy::new();
        assert!(!p.evaluate("network_request"), "network_request should default to Deny");
    }

    #[test]
    fn test_llm_domain_auto_allowed() {
        let p = ActionPolicy::new();
        assert_eq!(p.evaluate_network("api.openai.com"), PolicyDecision::Allow);
        assert_eq!(p.evaluate_network("api.anthropic.com"), PolicyDecision::Allow);
        assert_eq!(p.evaluate_network("generativelanguage.googleapis.com"), PolicyDecision::Allow);
    }

    #[test]
    fn test_unknown_domain_denied() {
        let p = ActionPolicy::new();
        assert_eq!(p.evaluate_network("evil.example.com"), PolicyDecision::Deny);
    }

    #[test]
    fn test_custom_domain_allowlist() {
        let mut p = ActionPolicy::new();
        p.allowlist_domain("my.internal.api.com");
        assert_eq!(p.evaluate_network("my.internal.api.com"), PolicyDecision::Allow);
    }

    #[test]
    fn test_remove_domain_from_allowlist() {
        let mut p = ActionPolicy::new();
        p.allowlist_domain("test.com");
        assert!(p.is_domain_allowed("test.com"));
        p.remove_domain("test.com");
        assert!(!p.is_domain_allowed("test.com"));
    }

    #[test]
    fn test_subdomain_matches_allowlist() {
        let p = ActionPolicy::new();
        assert!(p.is_domain_allowed("eu.api.openai.com"));
    }

    #[test]
    fn test_set_profile_strict_nt_shield() {
        let mut p = ActionPolicy::new();
        p.set_profile("strict-nt_shield");
        assert_eq!(p.profile, "strict-nt_shield");
    }

    #[test]
    fn test_strict_nt_shield_denies_write() {
        let mut p = ActionPolicy::new();
        p.set_profile("strict-nt_shield");
        assert!(!p.evaluate("write_file"));
    }

    #[test]
    fn test_general_profile_allows_write() {
        let mut p = ActionPolicy::new();
        p.set_profile("general");
        assert!(p.evaluate("write_file"));
        assert!(p.evaluate("execute_command"));
    }

    #[test]
    fn test_network_isolated_profile() {
        let mut p = ActionPolicy::new();
        p.set_profile("network-isolated");
        assert!(!p.evaluate("network_request"));
    }

    #[test]
    fn test_add_rule_custom() {
        let mut p = ActionPolicy::new();
        p.add_rule("custom_action", PolicyDecision::Allow);
        assert!(p.evaluate("custom_action"));
    }

    #[test]
    fn test_decide_unknown_action_returns_deny() {
        let p = ActionPolicy::new();
        assert_eq!(p.decide("nonexistent_action"), PolicyDecision::Deny);
    }

    #[test]
    fn test_action_count_increases_with_rules() {
        let mut p = ActionPolicy::new();
        let before = p.action_count();
        p.add_rule("extra_op", PolicyDecision::Allow);
        assert_eq!(p.action_count(), before + 1);
    }
}
