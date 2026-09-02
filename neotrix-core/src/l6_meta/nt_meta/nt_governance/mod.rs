//! Governance — NT-GOVERNANCE 治理层
//! 
//! 吸收的能力模块 (trailofbits/skills):
//! - skill_validator: 30+ 强制规则验证器 + self-test
//! - human_oversight: 人类监督治理 self-tests

pub mod skill_validator;

// 2026-08-28: NT-GOVERNANCE 人类监督治理 affordance + 萎缩对策
pub fn register_human_oversight_self_tests(_registry: &mut crate::core::nt_core_self_test::SelfTestRegistry) {
    // TODO: 实现人类监督治理自测注册
    // 示例注册模式：
    // registry.register(Box::new(HumanOversightSelfTest));
}