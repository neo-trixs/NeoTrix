//! nt_io::web_data — 网页数据摄取 (web data acquisition) 能力簇
//!
//! 簇节点 (L0-L1):
//! - nt_io::web_data::planner (L0) — 摄取规划 (目标校验/去重/URL 卫生)
//! - nt_io::web_data::quality (L1) — 来源质量评分 (域名信誉/内容完整度)
//!
//! 设计来源 (吸收): 网页摄取工具链 (crawl4ai 77k★/Grab 等) — 摄取必须
//! 有规划 (输入卫生) 与质量门 (输出评分), 拒绝垃圾 URL 与不完整内容。

#![forbid(unsafe_code)]

pub mod planner;
pub mod quality;

use crate::core::nt_core_traits::{CapabilityNode, SelfTest};

/// 网页数据摄取簇统一导出
pub fn io_web_data_cluster_nodes() -> Vec<Box<dyn CapabilityNode>> {
    vec![
        Box::new(planner::AcquisitionPlanner::new()),
        Box::new(quality::SourceQualityGate::new()),
    ]
}

/// 簇级 SelfTest 聚合
pub fn run_io_web_data_self_tests() -> Result<(), Vec<String>> {
    planner::AcquisitionPlanner::new().self_test()?;
    quality::SourceQualityGate::new().self_test()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_nodes() {
        let nodes = io_web_data_cluster_nodes();
        assert_eq!(nodes.len(), 2);
        for node in &nodes {
            assert!(!node.provides().is_empty());
        }
    }

    #[test]
    fn test_cluster_self_tests() {
        assert!(run_io_web_data_self_tests().is_ok());
    }
}
