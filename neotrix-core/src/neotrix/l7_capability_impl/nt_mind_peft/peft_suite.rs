//! nt_mind::peft::peft_suite — PEFT 统一编排层
//!
//! 节点: nt_mind::peft::peft_suite (L2)
//! Provides: peft_orchestration
//! Requires: lora_core, qlora, dora, mora, reft
//! Rune: All 5 (Crimson, Indigo, Obsidian, Golden, Alabaster)

#![forbid(unsafe_code)]

use crate::core::nt_core_error::NeoTrixError;
use crate::core::nt_core_traits::{CapabilityNode, RuneSocket, SelfTest};
use crate::neotrix::l7_capability_impl::nt_mind_peft::{dora, lora_core, mora, qlora, reft};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum PEFTMethod {
    LoRA,
    QLoRA,
    DoRA,
    MoRA,
    ReFT,
    Auto, // 自动选择
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PEFTSuiteConfig {
    pub default_method: PEFTMethod,
    pub method_configs: HashMap<PEFTMethod, serde_json::Value>,
    pub auto_selection_criteria: AutoSelectionCriteria,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AutoSelectionCriteria {
    pub memory_budget_gb: Option<f32>,
    pub target_rank: Option<usize>,
    pub quantization_preference: bool,
    pub representation_based: bool,
}

impl Default for PEFTSuiteConfig {
    fn default() -> Self {
        let mut configs = HashMap::new();
        configs.insert(
            PEFTMethod::LoRA,
            serde_json::to_value(lora_core::LoRAConfig::default()).unwrap(),
        );
        configs.insert(
            PEFTMethod::QLoRA,
            serde_json::to_value(qlora::QLoRAConfig::default()).unwrap(),
        );
        configs.insert(
            PEFTMethod::DoRA,
            serde_json::to_value(dora::DoRAConfig::default()).unwrap(),
        );
        configs.insert(
            PEFTMethod::MoRA,
            serde_json::to_value(mora::MoRAConfig::default()).unwrap(),
        );
        configs.insert(
            PEFTMethod::ReFT,
            serde_json::to_value(reft::ReFTConfig::default()).unwrap(),
        );

        Self {
            default_method: PEFTMethod::LoRA,
            method_configs: configs,
            auto_selection_criteria: AutoSelectionCriteria {
                memory_budget_gb: None,
                target_rank: None,
                quantization_preference: false,
                representation_based: false,
            },
        }
    }
}

/// PEFT 统一编排器
pub struct PEFTSuite {
    config: PEFTSuiteConfig,
    lora: Option<lora_core::LoRACore>,
    qlora: Option<qlora::QLoRA>,
    dora: Option<dora::DoRA>,
    mora: Option<mora::MoRA>,
    reft: Option<reft::ReFT>,
    metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl PEFTSuite {
    pub fn new(config: PEFTSuiteConfig) -> Self {
        Self {
            config,
            lora: None,
            qlora: None,
            dora: None,
            mora: None,
            reft: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// 根据配置初始化指定方法 (单激活: 初始化新方法会清空其他已激活方法)
    pub fn initialize_method(&mut self, method: PEFTMethod) -> Result<(), NeoTrixError> {
        self.lora = None;
        self.qlora = None;
        self.dora = None;
        self.mora = None;
        self.reft = None;
        match method {
            PEFTMethod::LoRA => {
                let config: lora_core::LoRAConfig = serde_json::from_value(
                    self.config
                        .method_configs
                        .get(&PEFTMethod::LoRA)
                        .unwrap()
                        .clone(),
                )
                .unwrap();
                self.lora = Some(lora_core::LoRACore::new(config));
            }
            PEFTMethod::QLoRA => {
                let config: qlora::QLoRAConfig = serde_json::from_value(
                    self.config
                        .method_configs
                        .get(&PEFTMethod::QLoRA)
                        .unwrap()
                        .clone(),
                )
                .unwrap();
                self.qlora = Some(qlora::QLoRA::new(config));
            }
            PEFTMethod::DoRA => {
                let config: dora::DoRAConfig = serde_json::from_value(
                    self.config
                        .method_configs
                        .get(&PEFTMethod::DoRA)
                        .unwrap()
                        .clone(),
                )
                .unwrap();
                self.dora = Some(dora::DoRA::new(config));
            }
            PEFTMethod::MoRA => {
                let config: mora::MoRAConfig = serde_json::from_value(
                    self.config
                        .method_configs
                        .get(&PEFTMethod::MoRA)
                        .unwrap()
                        .clone(),
                )
                .unwrap();
                self.mora = Some(mora::MoRA::new(config));
            }
            PEFTMethod::ReFT => {
                let config: reft::ReFTConfig = serde_json::from_value(
                    self.config
                        .method_configs
                        .get(&PEFTMethod::ReFT)
                        .unwrap()
                        .clone(),
                )
                .unwrap();
                self.reft = Some(reft::ReFT::new(config));
            }
            PEFTMethod::Auto => {
                // 自动选择逻辑：基于内存预算、秩要求等
                // 简化：默认选择 LoRA
                self.initialize_method(PEFTMethod::LoRA)?;
            }
        }
        Ok(())
    }

    /// 获取当前激活的方法
    pub fn active_method(&self) -> Option<PEFTMethod> {
        if self.lora.is_some() {
            Some(PEFTMethod::LoRA)
        } else if self.qlora.is_some() {
            Some(PEFTMethod::QLoRA)
        } else if self.dora.is_some() {
            Some(PEFTMethod::DoRA)
        } else if self.mora.is_some() {
            Some(PEFTMethod::MoRA)
        } else if self.reft.is_some() {
            Some(PEFTMethod::ReFT)
        } else {
            None
        }
    }

    /// 统一初始化适配器接口
    pub fn init_adapter(
        &mut self,
        module_name: &str,
        in_features: usize,
        out_features: usize,
    ) -> Result<(), NeoTrixError> {
        match self.active_method() {
            Some(PEFTMethod::LoRA) => {
                self.lora
                    .as_mut()
                    .unwrap()
                    .init_adapter(module_name, in_features, out_features)
            }
            Some(PEFTMethod::QLoRA) => self.qlora.as_mut().unwrap().init_lora_adapter(
                module_name,
                in_features,
                out_features,
            ),
            Some(PEFTMethod::DoRA) => {
                self.dora
                    .as_mut()
                    .unwrap()
                    .init_adapter(module_name, in_features, out_features)
            }
            Some(PEFTMethod::MoRA) => {
                self.mora
                    .as_mut()
                    .unwrap()
                    .init_adapter(module_name, in_features, out_features)
            }
            Some(PEFTMethod::ReFT) => {
                self.reft
                    .as_mut()
                    .unwrap()
                    .init_intervention(0, "residual", out_features)
            }
            Some(PEFTMethod::Auto) => {
                self.lora
                    .as_mut()
                    .unwrap()
                    .init_adapter(module_name, in_features, out_features)
            }
            None => Err(NeoTrixError::InvalidState(
                "No PEFT method initialized".into(),
            )),
        }
    }

    /// 统一前向接口
    pub fn forward(&self, module_name: &str, input: &[f32]) -> Result<Vec<f32>, NeoTrixError> {
        match self.active_method() {
            Some(PEFTMethod::LoRA) => self.lora.as_ref().unwrap().forward(module_name, input),
            Some(PEFTMethod::QLoRA) => self.qlora.as_ref().unwrap().forward(module_name, input),
            Some(PEFTMethod::DoRA) => Err(NeoTrixError::NotImplemented(
                "DoRA forward via suite".into(),
            )),
            Some(PEFTMethod::MoRA) => self.mora.as_ref().unwrap().forward(module_name, input),
            Some(PEFTMethod::ReFT) => Err(NeoTrixError::NotImplemented(
                "ReFT forward via suite".into(),
            )),
            Some(PEFTMethod::Auto) => self.lora.as_ref().unwrap().forward(module_name, input),
            None => Err(NeoTrixError::InvalidState(
                "No PEFT method initialized".into(),
            )),
        }
    }

    pub fn config(&self) -> &PEFTSuiteConfig {
        &self.config
    }
    pub fn set_default_method(&mut self, method: PEFTMethod) {
        self.config.default_method = method;
    }
}

impl CapabilityNode for PEFTSuite {
    fn node_id(&self) -> &str {
        "nt_mind::peft::peft_suite"
    }
    fn provides(&self) -> Vec<String> {
        vec!["peft_orchestration".into()]
    }
    fn requires(&self) -> Vec<String> {
        vec![
            "lora_core".into(),
            "qlora".into(),
            "dora".into(),
            "mora".into(),
            "reft".into(),
        ]
    }
    fn rune_sockets(&self) -> Vec<RuneSocket> {
        vec![
            RuneSocket::Crimson,
            RuneSocket::Indigo,
            RuneSocket::Obsidian,
            RuneSocket::Golden,
            RuneSocket::Alabaster,
        ]
    }
    fn constellation_level(&self) -> u8 {
        0
    }
    fn promote_constellation(&mut self) -> bool {
        false
    }
}

impl SelfTest for PEFTSuite {
    fn self_test(&self) -> Result<(), Vec<String>> {
        let inner = (|| -> Result<(), crate::core::nt_core_error::NeoTrixError> {
            let mut suite = PEFTSuite::new(PEFTSuiteConfig::default());

            // 测试各方法初始化
            for method in [
                PEFTMethod::LoRA,
                PEFTMethod::QLoRA,
                PEFTMethod::DoRA,
                PEFTMethod::MoRA,
                PEFTMethod::ReFT,
            ] {
                suite.initialize_method(method.clone())?;
                assert!(suite.active_method() == Some(method.clone()));

                // 测试适配器初始化
                suite.init_adapter("test_proj", 64, 128)?;
            }

            // 测试 Auto 选择
            let mut suite2 = PEFTSuite::new(PEFTSuiteConfig::default());
            suite2.initialize_method(PEFTMethod::Auto)?;
            assert!(suite2.active_method().is_some());

            Ok(())
        })();
        inner.map_err(|e| vec![e.to_string()])
    }
    fn name(&self) -> &str {
        "nt_mind_peft_peft_suite"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_peft_suite_self_test() {
        let suite = PEFTSuite::new(PEFTSuiteConfig::default());
        assert!(suite.self_test().is_ok());
    }
}
