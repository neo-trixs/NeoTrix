//! nt_mind::merge — 模型合并能力簇
//!
//! 簇节点 (L1-L2):
//! - nt_mind::merge::soups (L1) — Model Soups 平均
//! - nt_mind::merge::ties (L1) — TIES-Merging
//! - nt_mind::merge::zip_lora (L1) — ZipLoRA 风格+主体融合
//! - nt_mind::merge::ct_merging (L1) — CT-Merging 共识方向
//! - nt_mind::merge::mergekit_adapter (L2) — MergeKit 统一适配器
//!
//! Rune: Indigo (变换), Obsidian (缓存), Golden (错误恢复), Alabaster (监控)

pub mod ct_merging;
pub mod mergekit_adapter;
pub mod soups;
pub mod ties;
pub mod zip_lora;

use crate::core::nt_core_traits::{CapabilityNode, SelfTest};

pub fn merge_cluster_nodes() -> Vec<Box<dyn CapabilityNode>> {
    vec![
        Box::new(soups::ModelSoups::new(soups::ModelSoupsConfig::default())),
        Box::new(ties::TIESMerging::new(ties::TIESConfig::default())),
        Box::new(zip_lora::ZipLoRA::new(zip_lora::ZipLoRAConfig::default())),
        Box::new(ct_merging::CTMerging::new(
            ct_merging::CTMergingConfig::default(),
        )),
        Box::new(mergekit_adapter::MergeKitAdapter::new(
            mergekit_adapter::MergeKitConfig::default(),
        )),
    ]
}

pub fn run_merge_cluster_self_tests() -> Result<(), Vec<String>> {
    soups::ModelSoups::new(soups::ModelSoupsConfig::default()).self_test()?;
    ties::TIESMerging::new(ties::TIESConfig::default()).self_test()?;
    zip_lora::ZipLoRA::new(zip_lora::ZipLoRAConfig::default()).self_test()?;
    ct_merging::CTMerging::new(ct_merging::CTMergingConfig::default()).self_test()?;
    mergekit_adapter::MergeKitAdapter::new(mergekit_adapter::MergeKitConfig::default())
        .self_test()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_merge_cluster() {
        let nodes = merge_cluster_nodes();
        assert_eq!(nodes.len(), 5);
        assert!(run_merge_cluster_self_tests().is_ok());
    }
}
