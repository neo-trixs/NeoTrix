//! nt_act::rcm — 根因方法 (root cause method) 能力簇
//!
//! 簇节点 (L0-L1):
//! - nt_act::rcm::chain (L0) — 因果链构建 (症状→根因回溯)
//! - nt_act::rcm::counterfactual (L1) — 反事实验证 (根因确认)
//!
//! 设计来源 (吸收): witr 身份优先解析 + core-trace 溯因推演 — 根因判定
//! 必须走"症状→候选根因→反事实验证"闭环, 拒绝只修表象。

#![forbid(unsafe_code)]

pub mod chain;
pub mod counterfactual;

use crate::core::nt_core_traits::{CapabilityNode, SelfTest};

/// 根因方法簇统一导出
pub fn act_rcm_cluster_nodes() -> Vec<Box<dyn CapabilityNode>> {
    vec![
        Box::new(chain::CausalChain::new()),
        Box::new(counterfactual::CounterfactualCheck::new()),
    ]
}

/// 簇级 SelfTest 聚合
pub fn run_act_rcm_self_tests() -> Result<(), Vec<String>> {
    chain::CausalChain::new().self_test()?;
    counterfactual::CounterfactualCheck::new().self_test()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_nodes() {
        let nodes = act_rcm_cluster_nodes();
        assert_eq!(nodes.len(), 2);
        for node in &nodes {
            assert!(!node.provides().is_empty());
        }
    }

    #[test]
    fn test_cluster_self_tests() {
        assert!(run_act_rcm_self_tests().is_ok());
    }
}
