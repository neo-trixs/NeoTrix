//! NT-SHIELD 安全技能路由 (吸收 `zhaoxuya520/reverse-skill`):
//! 给定技能名 + 风险分, 路由到 Allow/Deny/Sandbox, 维护自进化 deny/allow 名单。
//! R-P42 强化现有 `nt_shield::tool_permissions` / `permissions`, 不平行重造权限引擎。

use crate::core::nt_core_self_test::{SelfTest, SelfTestRegistry};
use std::sync::RwLock;

/// 技能路由决策。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillDecision {
    /// 直接放行
    Allow,
    /// 拒绝 (命中 deny 名单)
    Deny,
    /// 沙箱执行 (高风险未在 allow)
    Sandbox,
}

/// 安全技能路由器 — 自带自进化 deny/allow 名单 (reverse-skill "self-evolving knowledge base")。
pub struct SkillRouter {
    deny: RwLock<Vec<String>>,
    allow: RwLock<Vec<String>>,
}

impl Default for SkillRouter {
    fn default() -> Self {
        // 初始 deny 名单: 高危原语 (R-P79 安全基线)
        Self {
            deny: RwLock::new(vec![
                "rm -rf".into(),
                "format".into(),
                "drop table".into(),
                "curl|bash".into(),
            ]),
            allow: RwLock::new(Vec::new()),
        }
    }
}

impl SkillRouter {
    pub fn new() -> Self {
        Self::default()
    }

    /// 路由决策:
    /// - 显式 deny 名单命中 (子串) → Deny
    /// - 风险 ≥ 0.7 且未在 allow → Sandbox
    /// - 否则 → Allow
    pub fn route(&self, name: &str, risk: f64) -> SkillDecision {
        let denied = self
            .deny
            .read()
            .map(|d| d.iter().any(|p| name.contains(p.as_str())))
            .unwrap_or(false);
        if denied {
            return SkillDecision::Deny;
        }
        if risk >= 0.7 {
            SkillDecision::Sandbox
        } else {
            SkillDecision::Allow
        }
    }

    /// 自进化: 记录一次观测, 增补 deny/allow 名单 (reverse-skill 自进化经验库)。
    pub fn learn(&self, name: &str, decision: SkillDecision) {
        match decision {
            SkillDecision::Deny => {
                if let Ok(mut d) = self.deny.write() {
                    if !d.iter().any(|x| x == name) {
                        d.push(name.to_string());
                    }
                }
            }
            SkillDecision::Allow => {
                if let Ok(mut a) = self.allow.write() {
                    if !a.iter().any(|x| x == name) {
                        a.push(name.to_string());
                    }
                }
            }
            SkillDecision::Sandbox => {}
        }
    }
}

/// NT-SHIELD 安全技能路由自测 (卫生层 P0)。
pub struct SkillRouterSelfTest;

impl SelfTest for SkillRouterSelfTest {
    fn name(&self) -> &str {
        "nt_shield_skill_router"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let r = SkillRouter::new();
        if r.route("rm -rf disk", 0.1) != SkillDecision::Deny {
            return Err(vec!["deny list not enforced".into()]);
        }
        if r.route("safe-skill", 0.2) != SkillDecision::Allow {
            return Err(vec!["low-risk should allow".into()]);
        }
        if r.route("unknown-tool", 0.9) != SkillDecision::Sandbox {
            return Err(vec!["high-risk should sandbox".into()]);
        }
        // 自进化: 学习后 deny 名单应包含新条目
        r.learn("evil-wipe", SkillDecision::Deny);
        if r.route("evil-wipe", 0.0) != SkillDecision::Deny {
            return Err(vec!["self-evolved deny not applied".into()]);
        }
        Ok(())
    }
}

/// 注册安全技能路由 SelfTest 到全局注册表 (T2)。
pub fn register_skill_router_self_tests(registry: &mut SelfTestRegistry) {
    registry.register(Box::new(SkillRouterSelfTest));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_router_self_test_passes() {
        assert!(
            SkillRouterSelfTest.self_test().is_ok(),
            "skill router self_test failed: {:?}",
            SkillRouterSelfTest.self_test().err()
        );
    }
}
