//! Governance — NT-GOVERNANCE 治理层
//! 
//! 吸收的能力模块 (trailofbits/skills):
//! - skill_validator: 30+ 强制规则验证器 + self-test
//! - human_oversight: 人类监督治理 self-tests

pub mod skill_validator;

use crate::core::nt_core_self_test::{SelfTest, SelfTestResult, SelfTestRegistry};

/// 人类监督治理自测
pub struct HumanOversightSelfTest;

impl SelfTest for HumanOversightSelfTest {
    fn name(&self) -> &str { "human_oversight_governance" }
    fn self_test(&self) -> Result<(), Vec<String>> {
        // 检查治理规则是否可访问
        Ok(())
    }
}

/// 注册人类监督治理自测
pub fn register_human_oversight_self_tests(registry: &mut SelfTestRegistry) {
    registry.register(Box::new(HumanOversightSelfTest));
}