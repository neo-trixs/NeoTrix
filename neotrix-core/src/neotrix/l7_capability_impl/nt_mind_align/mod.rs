//! nt_mind::align — 偏好对齐 (preference alignment) 能力簇
//!
//! 簇节点 (L0-L1):
//! - nt_mind::align::reward (L0) — 偏好对奖励校准 (DPO/ORPO 风格)
//! - nt_mind::align::policy (L1) — 对齐策略评分 + 门控
//!
//! 设计来源 (吸收): 偏好对齐主线 (DPO/ORPO/SimPO/Zephyr 知识吸收)。

#![forbid(unsafe_code)]

pub mod policy;
pub mod reward;

use crate::core::nt_core_traits::{CapabilityNode, SelfTest};

/// 偏好对齐簇统一导出
pub fn mind_align_cluster_nodes() -> Vec<Box<dyn CapabilityNode>> {
    vec![
        Box::new(reward::RewardCalibrator::new()),
        Box::new(policy::AlignPolicy::new()),
    ]
}

/// 簇级 SelfTest 聚合
pub fn run_mind_align_self_tests() -> Result<(), Vec<String>> {
    reward::RewardCalibrator::new().self_test()?;
    policy::AlignPolicy::new().self_test()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_nodes() {
        let nodes = mind_align_cluster_nodes();
        assert_eq!(nodes.len(), 2);
        for node in &nodes {
            assert!(!node.provides().is_empty());
        }
    }

    #[test]
    fn test_cluster_self_tests() {
        assert!(run_mind_align_self_tests().is_ok());
    }
}
