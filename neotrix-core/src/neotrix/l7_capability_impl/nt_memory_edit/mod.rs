//! nt_memory::edit — 知识编辑 (knowledge editing) 能力簇
//!
//! 簇节点 (L0-L1):
//! - nt_memory::edit::edit_log (L0) — 版本化知识编辑日志 (插入/更新/删除)
//! - nt_memory::edit::guardrail (L1) — 编辑完整性护栏 (证据校验 + 回滚)
//!
//! 设计来源 (吸收): arXiv 2602.06052 记忆 3D 分类法 + TencentDB-Agent-Memory
//! 4 类记忆资产 + claude-mem/semantica 版本化记忆持久化。
//! 每条编辑带 provenance (来源/理由), 可回滚, 恶意/畸形编辑被护栏拒绝。
//!
//! Rune Sockets: Crimson (数据), Obsidian (缓存), Golden (错误恢复)

#![forbid(unsafe_code)]

pub mod edit_log;
pub mod guardrail;

use crate::core::nt_core_traits::{CapabilityNode, SelfTest};

/// 知识编辑簇统一导出
pub fn memory_edit_cluster_nodes() -> Vec<Box<dyn CapabilityNode>> {
    vec![
        Box::new(edit_log::KnowledgeEditLog::new()),
        Box::new(guardrail::EditGuardrail::new(10)),
    ]
}

/// 簇级 SelfTest 聚合
pub fn run_memory_edit_self_tests() -> Result<(), Vec<String>> {
    edit_log::KnowledgeEditLog::new().self_test()?;
    guardrail::EditGuardrail::new(10).self_test()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_nodes() {
        let nodes = memory_edit_cluster_nodes();
        assert_eq!(nodes.len(), 2);
        for node in &nodes {
            assert!(!node.provides().is_empty());
            assert!(!node.node_id().is_empty());
        }
    }

    #[test]
    fn test_cluster_self_tests() {
        assert!(run_memory_edit_self_tests().is_ok());
    }
}
