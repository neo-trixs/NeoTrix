use std::collections::{HashMap, HashSet};

use crate::core::nt_core_self_test::SelfTest;

/// 策略评估结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyDecision {
    Allow,
    RequireConfirmation,
    Deny,
}

impl PolicyDecision {
    /// 严格度排序 (DBX 权限单调性, 吸收 2026-08-19): Allow=0 < RequireConfirmation=1 < Deny=2。
    /// 值越大越严格。runtime/env 覆盖层只能收紧、绝不能放宽已保存策略。
    pub fn strictness(&self) -> u8 {
        match self {
            PolicyDecision::Allow => 0,
            PolicyDecision::RequireConfirmation => 1,
            PolicyDecision::Deny => 2,
        }
    }

    /// 单调合并: 返回两者中更严格者。保证低信任源 (env/config 覆盖) 永远无法
    /// 提升权限 — 已保存策略若为 Deny 则保持 Deny。
    pub fn tightened_with(&self, other: &PolicyDecision) -> PolicyDecision {
        if other.strictness() > self.strictness() {
            other.clone()
        } else {
            self.clone()
        }
    }
}

/// 安全配置文件
#[derive(Debug, Clone)]
pub struct ActionPolicy {
    rules: HashMap<String, PolicyDecision>,
    /// 当前安全配置文件 (nt_shield / strict-nt_shield / general)
    pub profile: String,
    /// 网络请求域名白名单（自动放行的 LLM API 域名）
    _network_allowlist: HashSet<String>,
}

/// LLM API 提供商默认域名白名单
fn default_llm_domains() -> HashSet<String> {
    [
        "api.siliconflow.cn",
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
        // 主流国产/海外 LLM API（2026-08 供应商扩展）
        "api.x.ai",
        "api.moonshot.cn",
        "dashscope.aliyuncs.com",
        "ark.cn-beijing.volces.com",
        "api.minimax.chat",
        "api.perplexity.ai",
        "api.modelscope.cn",
        "open.bigmodel.cn",
        "integrate.api.nvidia.com",
        "models.inference.ai.azure.com",
        "api-inference.huggingface.co",
        "ai-endpoints.ovh.net",        // 已注册 provider 池实际端点（config.toml active provider 等）
        "aihub.humorously.cn",
        "opencode.ai",
        // 2026-08: llm7 是 RouterConfig 默认 tier 端点 (keyless 实测 codestral-latest
        // 匿名可用)，零配置可用性优先 — 从 default-deny 移入白名单。
        "api.llm7.io",
        // free.empero.org — keyless 免费端点, 收编进 failover mesh (R-P42)。
        "free.empero.org",
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
        rules.insert("access_stealth_browser_auto".to_string(), PolicyDecision::RequireConfirmation);
        rules.insert("access_tor_network".to_string(), PolicyDecision::RequireConfirmation);

        Self {
            rules,
            profile: "nt_shield".to_string(),
            _network_allowlist: default_llm_domains(),
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

    /// 单调收紧规则 (DBX 权限单调性不变量, 吸收 2026-08-19):
    /// 低信任源 (env/config 覆盖层) 只能把规则收紧到更严格 (Allow→Ask/Deny, Ask→Deny),
    /// 永远无法放宽已保存策略 (Deny 不可被覆盖为 Ask/Allow)。返回生效后的决策。
    /// 消费者: `ShieldEnforcer::set_rule_monotonic` → `/perm set-rule` (perm_cmds.rs)。
    pub fn set_rule_monotonic(&mut self, action: &str, decision: PolicyDecision) -> PolicyDecision {
        let effective = self
            .rules
            .get(action)
            .cloned()
            .unwrap_or(PolicyDecision::Deny)
            .tightened_with(&decision);
        if self.rules.insert(action.to_string(), effective.clone()).is_none() {
            // 新 action 首次引入 — 单调语义下以更严格者为准已满足
        }
        effective
    }

    /// 将域名加入网络白名单
    pub fn _allowlist_domain(&mut self, domain: &str) {
        self._network_allowlist.insert(domain.to_string());
    }

    /// 从网络白名单移除域名
    pub fn _remove_domain(&mut self, domain: &str) {
        self._network_allowlist.remove(domain);
    }

    /// 检查域名是否在白名单中
    pub fn _is_domain_allowed(&self, domain: &str) -> bool {
        self._network_allowlist.contains(domain)
            || self._network_allowlist.iter().any(|d| domain.ends_with(&format!(".{}", d)) || domain == d)
    }

    /// 获取白名单引用
    pub fn _network_allowlist(&self) -> &HashSet<String> {
        &self._network_allowlist
    }

    /// 评估网络请求 — 先检查域名白名单，再查规则表
    pub fn evaluate_network(&self, domain: &str) -> PolicyDecision {
        if self._is_domain_allowed(domain) {
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

impl crate::core::nt_core_traits::NetworkPolicy for ActionPolicy {
    fn check_network_access(&self, domain: &str) -> crate::core::nt_core_traits::NetworkPolicyResult {
        match self.evaluate_network(domain) {
            PolicyDecision::Allow => crate::core::nt_core_traits::NetworkPolicyResult::Allow,
            PolicyDecision::RequireConfirmation => crate::core::nt_core_traits::NetworkPolicyResult::RequireConfirmation,
            PolicyDecision::Deny => crate::core::nt_core_traits::NetworkPolicyResult::Deny,
        }
    }
}

impl SelfTest for ActionPolicy {
    fn name(&self) -> &str { "action_policy" }
    fn self_test(&self) -> Result<(), Vec<String>> {
        if self.action_count() < 5 {
            return Err(vec![format!("expected >=5 rules, got {}", self.action_count())]);
        }
        if self.profile != "nt_shield" {
            return Err(vec![format!("expected profile nt_shield, got {}", self.profile)]);
        }
        if self.decide("network_request") != PolicyDecision::Deny {
            return Err(vec!["network_request should be Deny by default".into()]);
        }
        Ok(())
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
        p._allowlist_domain("my.internal.api.com");
        assert_eq!(p.evaluate_network("my.internal.api.com"), PolicyDecision::Allow);
    }

    #[test]
    fn test_remove_domain_from_allowlist() {
        let mut p = ActionPolicy::new();
        p._allowlist_domain("test.com");
        assert!(p._is_domain_allowed("test.com"));
        p._remove_domain("test.com");
        assert!(!p._is_domain_allowed("test.com"));
    }

    #[test]
    fn test_subdomain_matches_allowlist() {
        let p = ActionPolicy::new();
        assert!(p._is_domain_allowed("eu.api.openai.com"));
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

    #[test]
    fn test_policy_decision_strictness_ordering() {
        assert!(PolicyDecision::Allow.strictness() < PolicyDecision::RequireConfirmation.strictness());
        assert!(PolicyDecision::RequireConfirmation.strictness() < PolicyDecision::Deny.strictness());
    }

    #[test]
    fn test_tightened_with_keeps_stricter() {
        assert_eq!(
            PolicyDecision::Allow.tightened_with(&PolicyDecision::Deny),
            PolicyDecision::Deny
        );
        assert_eq!(
            PolicyDecision::Deny.tightened_with(&PolicyDecision::Allow),
            PolicyDecision::Deny
        );
        assert_eq!(
            PolicyDecision::RequireConfirmation.tightened_with(&PolicyDecision::Deny),
            PolicyDecision::Deny
        );
        assert_eq!(
            PolicyDecision::Allow.tightened_with(&PolicyDecision::RequireConfirmation),
            PolicyDecision::RequireConfirmation
        );
    }

    #[test]
    fn test_set_rule_monotonic_cannot_elevate_saved_policy() {
        let mut p = ActionPolicy::new();
        // 已保存策略: write_file = RequireConfirmation
        assert_eq!(p.decide("write_file"), PolicyDecision::RequireConfirmation);
        // 低信任源尝试放宽 → 被拒绝, 保留 RequireConfirmation
        let effective = p.set_rule_monotonic("write_file", PolicyDecision::Allow);
        assert_eq!(effective, PolicyDecision::RequireConfirmation);
        assert_eq!(p.decide("write_file"), PolicyDecision::RequireConfirmation);
        // 低信任源尝试收紧 → 生效
        let effective = p.set_rule_monotonic("write_file", PolicyDecision::Deny);
        assert_eq!(effective, PolicyDecision::Deny);
        assert_eq!(p.decide("write_file"), PolicyDecision::Deny);
    }

    #[test]
    fn test_set_rule_monotonic_deny_never_unlocked() {
        let mut p = ActionPolicy::new();
        // 已保存策略: read_secrets = Deny
        assert_eq!(p.decide("read_secrets"), PolicyDecision::Deny);
        // env/覆盖层无论如何都不能解锁 Deny
        let effective = p.set_rule_monotonic("read_secrets", PolicyDecision::Allow);
        assert_eq!(effective, PolicyDecision::Deny);
        let effective = p.set_rule_monotonic("read_secrets", PolicyDecision::RequireConfirmation);
        assert_eq!(effective, PolicyDecision::Deny);
        assert_eq!(p.decide("read_secrets"), PolicyDecision::Deny);
    }

    #[test]
    fn test_set_rule_monotonic_unknown_action_defaults_deny() {
        let mut p = ActionPolicy::new();
        // 未注册 action 的决策基线是 Deny (decide 默认), 任何放宽提案都被卡在 Deny
        let effective = p.set_rule_monotonic("brand_new_action", PolicyDecision::Allow);
        assert_eq!(effective, PolicyDecision::Deny);
        assert_eq!(p.decide("brand_new_action"), PolicyDecision::Deny);
    }
}
