//! nt_mind::testtime — 测试时计算 (test-time reasoning) 能力簇
//!
//! 簇节点 (L0-L1):
//! - nt_mind::testtime::verify (L0) — 验证循环 (生成-校验-重试)
//! - nt_mind::testtime::budget (L1) — 计算预算分配 (防无限思考)
//!
//! 设计来源 (吸收): 测试时计算主线 (s1 / DeepSeek-R1 / LIMO) — 思考越多
//! 不一定越好, 需要验证循环 + 预算封顶的严谨工程化。

#![forbid(unsafe_code)]

pub mod budget;
pub mod verify;

use crate::core::nt_core_traits::{CapabilityNode, SelfTest};

/// 测试时计算簇统一导出
pub fn mind_testtime_cluster_nodes() -> Vec<Box<dyn CapabilityNode>> {
    vec![
        Box::new(verify::VerifyLoop::new()),
        Box::new(budget::ReasonBudget::new(8)),
    ]
}

/// 簇级 SelfTest 聚合
pub fn run_mind_testtime_self_tests() -> Result<(), Vec<String>> {
    verify::VerifyLoop::new().self_test()?;
    budget::ReasonBudget::new(8).self_test()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_nodes() {
        let nodes = mind_testtime_cluster_nodes();
        assert_eq!(nodes.len(), 2);
        for node in &nodes {
            assert!(!node.provides().is_empty());
        }
    }

    #[test]
    fn test_cluster_self_tests() {
        assert!(run_mind_testtime_self_tests().is_ok());
    }
}
