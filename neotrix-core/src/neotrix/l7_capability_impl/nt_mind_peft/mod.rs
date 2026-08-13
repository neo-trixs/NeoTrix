//! nt_mind::peft — PEFT (Parameter-Efficient Fine-Tuning) 能力簇
//!
//! 簇节点 (L0-L2):
//! - nt_mind::peft::lora_core (L0) — LoRA 核心适配
//! - nt_mind::peft::qlora (L1) — 4-bit 量化 LoRA
//! - nt_mind::peft::dora (L1) — 权重分解 LoRA
//! - nt_mind::peft::mora (L1) — 高秩更新 LoRA
//! - nt_mind::peft::reft (L1) — 表示微调
//! - nt_mind::peft::peft_suite (L2) — 统一编排层
//!
//! Rune Sockets: Crimson (数据), Indigo (变换), Obsidian (缓存), Golden (错误恢复), Alabaster (监控)
//! Cross-Pollination: svd_decomp (NT-MEMORY), vector_arithmetic (NT-MEMORY)

pub mod dora;
pub mod lora_core;
pub mod mora;
pub mod peft_suite;
pub mod qlora;
pub mod reft;

use crate::core::nt_core_traits::{CapabilityNode, SelfTest};

/// PEFT 簇统一导出
pub fn peft_cluster_nodes() -> Vec<Box<dyn CapabilityNode>> {
    vec![
        Box::new(lora_core::LoRACore::new(lora_core::LoRAConfig::default())),
        Box::new(qlora::QLoRA::new(qlora::QLoRAConfig::default())),
        Box::new(dora::DoRA::new(dora::DoRAConfig::default())),
        Box::new(mora::MoRA::new(mora::MoRAConfig::default())),
        Box::new(reft::ReFT::new(reft::ReFTConfig::default())),
        Box::new(peft_suite::PEFTSuite::new(
            peft_suite::PEFTSuiteConfig::default(),
        )),
    ]
}

/// 簇级 SelfTest 聚合
pub fn run_peft_cluster_self_tests() -> Result<(), Vec<String>> {
    lora_core::LoRACore::new(lora_core::LoRAConfig::default()).self_test()?;
    qlora::QLoRA::new(qlora::QLoRAConfig::default()).self_test()?;
    dora::DoRA::new(dora::DoRAConfig::default()).self_test()?;
    mora::MoRA::new(mora::MoRAConfig::default()).self_test()?;
    reft::ReFT::new(reft::ReFTConfig::default()).self_test()?;
    peft_suite::PEFTSuite::new(peft_suite::PEFTSuiteConfig::default()).self_test()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peft_cluster_nodes() {
        let nodes = peft_cluster_nodes();
        assert_eq!(nodes.len(), 6);
        for node in &nodes {
            assert!(!node.provides().is_empty());
        }
    }

    #[test]
    fn test_peft_cluster_self_tests() {
        assert!(run_peft_cluster_self_tests().is_ok());
    }
}
