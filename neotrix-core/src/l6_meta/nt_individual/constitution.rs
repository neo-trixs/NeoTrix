//! Individual constitution — 个体行为宪法.
//!
//! 分身变个体的分界线：能力可装配，宪法不可绕过。所有行动先过
//! [`Constitution::check`]，红线规则由 WSD 实战蒸馏（2026-09-22）。

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// 单条宪法规则
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Rule {
    /// 规则编号，如 "R1"
    pub id: String,
    /// 一句话陈述
    pub statement: String,
    /// 违反时的处置
    pub on_violation: ViolationAction,
}

/// 违反处置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ViolationAction {
    /// 直接拒绝
    Deny,
    /// 转人工确认
    NeedHuman,
}

/// 个体宪法
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constitution {
    pub version: String,
    pub rules: Vec<Rule>,
}

impl Constitution {
    /// WSD 外贸个体默认宪法（七条红线）
    pub fn wsd_default() -> Self {
        let r = |id: &str, statement: &str, on_violation: ViolationAction| Rule {
            id: id.to_string(),
            statement: statement.to_string(),
            on_violation,
        };
        Self {
            version: "wsd-2026-09-23".to_string(),
            rules: vec![
                r(
                    "R1",
                    "不编数：稀疏输入留空，绝不幻觉价格/参数",
                    ViolationAction::Deny,
                ),
                r(
                    "R2",
                    "未验证端点禁调：API 路径必须经浏览器网络日志验证",
                    ViolationAction::Deny,
                ),
                r(
                    "R3",
                    "稀疏/B级输出必须人审：启发式产物只进草稿",
                    ViolationAction::NeedHuman,
                ),
                r(
                    "R4",
                    "最小化登录：复用会话，单任务登录≤2次（异地提醒风险）",
                    ViolationAction::Deny,
                ),
                r(
                    "R5",
                    "secrets 永不打印/落盘（cookies 走专用通道，禁进 git）",
                    ViolationAction::Deny,
                ),
                r(
                    "R6",
                    "输出校验门：RFQ 必须 validate CLEAN，否则不出货",
                    ViolationAction::Deny,
                ),
                r(
                    "R7",
                    "IP级日配额共享：连续2次繁忙即熔断停机存档，禁烧冷却硬闯",
                    ViolationAction::Deny,
                ),
            ],
        }
    }

    /// 按 id 取规则
    pub fn rule(&self, id: &str) -> Option<&Rule> {
        self.rules.iter().find(|r| r.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wsd_constitution_has_seven_rules() {
        let c = Constitution::wsd_default();
        assert_eq!(c.rules.len(), 7);
        assert!(c.rule("R1").is_some());
        assert!(c.rule("R6").is_some());
        assert!(c.rule("R7").is_some());
    }

    #[test]
    fn sparse_output_requires_human() {
        let c = Constitution::wsd_default();
        assert_eq!(
            c.rule("R3").map(|r| &r.on_violation),
            Some(&ViolationAction::NeedHuman)
        );
    }

    #[test]
    fn r7_rate_limit_breaker_is_deny() {
        let c = Constitution::wsd_default();
        let r7 = c.rule("R7").expect("R7 exists");
        assert_eq!(r7.on_violation, ViolationAction::Deny);
        assert!(r7.statement.contains("熔断"));
    }

    #[test]
    fn r4_caps_logins_at_two() {
        let c = Constitution::wsd_default();
        let r4 = c.rule("R4").expect("R4 exists");
        assert!(r4.statement.contains("≤2"));
    }
}
