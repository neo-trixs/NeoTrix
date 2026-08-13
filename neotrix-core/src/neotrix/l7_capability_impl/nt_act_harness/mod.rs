//! nt_act::harness — Agent 执行 Harness 能力簇
//!
//! 簇节点 (L0-L2):
//! - nt_act::harness::harness (L0) — 单 agent turn 执行 + 输出捕获
//! - nt_act::harness::delegator (L1) — 工具委派 + 多 agent 协调
//! - nt_act::harness::orchestrator (L2) — 回合预算 + 编排控制
//!
//! 设计来源 (吸收): opencode (六层架构) + ante (单二进制) + valuecell
//! (多 agent) + cloudflare/computer (执行面)。R-P42: 强化现有节点而非
//! 平行适配器 — 本簇作为 l7 能力实现, 供 AgentLoop 与 /sandbox 复用。

#![forbid(unsafe_code)]

pub mod delegator;
pub mod harness;
pub mod orchestrator;

use crate::core::nt_core_traits::{CapabilityNode, SelfTest};

/// Agent Harness 簇统一导出
pub fn act_harness_cluster_nodes() -> Vec<Box<dyn CapabilityNode>> {
    vec![
        Box::new(harness::AgentHarness::new()),
        Box::new(delegator::ToolDelegator::new()),
        Box::new(orchestrator::TurnOrchestrator::new(10)),
    ]
}

/// 簇级 SelfTest 聚合
pub fn run_act_harness_self_tests() -> Result<(), Vec<String>> {
    harness::AgentHarness::new().self_test()?;
    delegator::ToolDelegator::new().self_test()?;
    orchestrator::TurnOrchestrator::new(10).self_test()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_nodes() {
        let nodes = act_harness_cluster_nodes();
        assert_eq!(nodes.len(), 3);
        for node in &nodes {
            assert!(!node.provides().is_empty());
        }
    }

    #[test]
    fn test_cluster_self_tests() {
        assert!(run_act_harness_self_tests().is_ok());
    }
}
