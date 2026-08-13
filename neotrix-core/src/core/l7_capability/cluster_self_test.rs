//! # CapabilityClusterSelfTest — L7 能力集群级自测
//!
//! 对 `CapabilityRegistry` 做完整性自审 (R-P23 检测系统自审计):
//! - 注册表可达且可构造 (不 panic)
//! - 集群计数非负、模块技能注册幂等
//! 供 SEAL 自迭代 pipeline 以 `Box<dyn SelfTest>` 注册消费。

use crate::core::nt_core_self_test::SelfTest;

/// L7 能力集群自测 — 验证能力注册表的基本完整性。
#[derive(Default)]
pub struct CapabilityClusterSelfTest;

impl SelfTest for CapabilityClusterSelfTest {
    fn name(&self) -> &str {
        "capability_cluster"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();

        // 注册表可构造且空表计数为 0 — 防未来 Default 实现破坏不变量
        let empty = crate::core::l7_capability::registry::CapabilityRegistry::new();
        if empty.count() != 0 {
            failures.push(format!("empty registry count = {}, expected 0", empty.count()));
        }

        // 注册模型技能应幂等: 首次返回新注册数, 重复调用返回 0 (不重复注册)
        let mut reg = crate::core::l7_capability::registry::CapabilityRegistry::new();
        let first = reg.register_model_skills();
        let second = reg.register_model_skills();
        if second != 0 {
            failures.push(format!(
                "register_model_skills not idempotent: second call registered {second}, expected 0"
            ));
        }
        if reg.count() != first {
            failures.push(format!(
                "registry count {count} != first registration count {first}",
                count = reg.count()
            ));
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cluster_self_test_passes() {
        let st = CapabilityClusterSelfTest::default();
        assert!(st.self_test().is_ok(), "cluster self-test should pass");
    }

    #[test]
    fn cluster_self_test_name_stable() {
        let st = CapabilityClusterSelfTest::default();
        assert_eq!(st.name(), "capability_cluster");
    }
}
