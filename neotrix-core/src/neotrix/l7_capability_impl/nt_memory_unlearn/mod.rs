//! nt_memory::unlearn — 机器遗忘护栏 (unlearning guardrail) 能力簇
//!
//! 簇节点 (L0-L1):
//! - nt_memory::unlearn::forget (L0) — 选择性遗忘执行 (作用域校验 + 溯源)
//! - nt_memory::unlearn::audit (L1) — 遗忘审计 (范围确认 + 恢复能力)
//!
//! 设计来源 (吸收): 机器遗忘 (SoK-Unlearning / GROM) — 遗忘必须可审计、
//! 范围受限、且被遗忘条目保留恢复凭据, 禁止无边界清除。

#![forbid(unsafe_code)]

pub mod audit;
pub mod forget;

use crate::core::nt_core_traits::{CapabilityNode, SelfTest};

/// 机器遗忘簇统一导出
pub fn memory_unlearn_cluster_nodes() -> Vec<Box<dyn CapabilityNode>> {
    vec![
        Box::new(forget::ForgetEngine::new()),
        Box::new(audit::UnlearningAudit::new()),
    ]
}

/// 簇级 SelfTest 聚合
pub fn run_memory_unlearn_self_tests() -> Result<(), Vec<String>> {
    forget::ForgetEngine::new().self_test()?;
    audit::UnlearningAudit::new().self_test()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_nodes() {
        let nodes = memory_unlearn_cluster_nodes();
        assert_eq!(nodes.len(), 2);
        for node in &nodes {
            assert!(!node.provides().is_empty());
        }
    }

    #[test]
    fn test_cluster_self_tests() {
        assert!(run_memory_unlearn_self_tests().is_ok());
    }
}
