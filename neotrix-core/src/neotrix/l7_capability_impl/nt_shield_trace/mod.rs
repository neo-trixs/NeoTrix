//! nt_shield::trace — 推理迹保护 (reasoning trace protection) 能力簇
//!
//! 簇节点 (L0-L1):
//! - nt_shield::trace::guard (L0) — 注入检测 + 威胁分级
//! - nt_shield::trace::anchor (L1) — 推理迹锚定 (完整性摘要)
//!
//! 设计来源 (吸收): arXiv 2608.10218 Mind Viruses — 单句防线近完全免疫;
//! 检测嵌入指令/冲突指令/越权覆盖, 锚定推理迹防篡改。
//!
//! Rune Sockets: Golden (错误恢复), Alabaster (监控)

#![forbid(unsafe_code)]

pub mod anchor;
pub mod guard;

use crate::core::nt_core_traits::{CapabilityNode, SelfTest};

/// 推理迹保护簇统一导出
pub fn shield_trace_cluster_nodes() -> Vec<Box<dyn CapabilityNode>> {
    vec![
        Box::new(guard::TraceGuard::new()),
        Box::new(anchor::TraceAnchor::new()),
    ]
}

/// 簇级 SelfTest 聚合
pub fn run_shield_trace_self_tests() -> Result<(), Vec<String>> {
    guard::TraceGuard::new().self_test()?;
    anchor::TraceAnchor::new().self_test()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_nodes() {
        let nodes = shield_trace_cluster_nodes();
        assert_eq!(nodes.len(), 2);
        for node in &nodes {
            assert!(!node.provides().is_empty());
        }
    }

    #[test]
    fn test_cluster_self_tests() {
        assert!(run_shield_trace_self_tests().is_ok());
    }
}
