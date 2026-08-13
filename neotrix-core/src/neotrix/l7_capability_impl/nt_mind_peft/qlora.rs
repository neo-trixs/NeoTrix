//! nt_mind::peft::qlora — QLoRA: 4-bit 量化 LoRA
//!
//! 论文: QLoRA (2305.14314) — 4-bit NF4 量化 + 双重量化 + 分页优化器
//! 对应节点: nt_mind::peft::qlora (L1)
//! Provides: quantized_lora
//! Requires: lora_core, quantization
//! Rune: Indigo, Obsidian

#![forbid(unsafe_code)]

use crate::core::nt_core_error::NeoTrixError;
use crate::core::nt_core_traits::{CapabilityNode, RuneSocket, SelfTest};
use crate::neotrix::l7_capability_impl::nt_mind_peft::lora_core::{LoRAConfig, LoRAWeights};
use std::collections::HashMap;

/// NF4 量化数据类型 (4-bit NormalFloat)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NF4 {
    // 16 个量化值 (-1.0 到 1.0 之间，针对正态分布优化)
    V0,
    V1,
    V2,
    V3,
    V4,
    V5,
    V6,
    V7,
    V8,
    V9,
    V10,
    V11,
    V12,
    V13,
    V14,
    V15,
}

impl NF4 {
    pub fn to_f32(self) -> f32 {
        // NF4 量化表 (近似值，实际应使用精确表)
        const NF4_TABLE: [f32; 16] = [
            -1.0,
            -0.6961928,
            -0.52507305,
            -0.39491748,
            -0.28444138,
            -0.18477344,
            -0.09105003,
            0.0,
            0.07958029,
            0.1609302,
            0.2461123,
            0.33791524,
            0.44070982,
            0.56261706,
            0.72295684,
            1.0,
        ];
        NF4_TABLE[self as usize]
    }

    pub fn from_f32(val: f32) -> Self {
        // 简单量化映射
        let idx = ((val + 1.0) * 7.5).clamp(0.0, 15.0) as u8;
        match idx {
            0 => NF4::V0,
            1 => NF4::V1,
            2 => NF4::V2,
            3 => NF4::V3,
            4 => NF4::V4,
            5 => NF4::V5,
            6 => NF4::V6,
            7 => NF4::V7,
            8 => NF4::V8,
            9 => NF4::V9,
            10 => NF4::V10,
            11 => NF4::V11,
            12 => NF4::V12,
            13 => NF4::V13,
            14 => NF4::V14,
            _ => NF4::V15,
        }
    }
}

/// 双重量化常数
#[derive(Debug, Clone)]
pub struct DoubleQuantConstants {
    /// 量化常数的量化值 (8-bit)
    pub quantized_constants: Vec<u8>,
    /// 反量化参数
    pub scale: f32,
    pub zero_point: i32,
}

/// QLoRA 配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QLoRAConfig {
    #[serde(flatten)]
    pub lora: LoRAConfig,
    /// 量化位宽 (4-bit NF4)
    pub quant_bits: u8,
    /// 是否使用双重量化
    pub double_quant: bool,
    /// 分页优化器页面大小
    pub page_size: usize,
    /// NF4 量化块大小
    pub block_size: usize,
}

impl Default for QLoRAConfig {
    fn default() -> Self {
        Self {
            lora: LoRAConfig::default(),
            quant_bits: 4,
            double_quant: true,
            page_size: 1024 * 1024, // 1MB
            block_size: 64,
        }
    }
}

/// QLoRA 量化权重
#[derive(Debug, Clone)]
pub struct QLoRAWeights {
    /// 量化后的基础模型权重 (NF4)
    pub quantized_base: Vec<NF4>,
    /// 反量化参数 (每个 block 一个)
    pub dequant_scales: Vec<f32>,
    /// LoRA 适配器权重 (保持 FP16/BF16)
    pub lora_weights: LoRAWeights,
    /// 双重量化常数
    pub double_quant: Option<DoubleQuantConstants>,
}

/// QLoRA 实现
pub struct QLoRA {
    config: QLoRAConfig,
    base_model_quantized: HashMap<String, Vec<NF4>>, // module -> quantized weights
    base_model_scales: HashMap<String, Vec<f32>>,    // module -> dequant scales
    lora_adapters: HashMap<String, LoRAWeights>,     // module -> LoRA weights
    metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl QLoRA {
    pub fn new(config: QLoRAConfig) -> Self {
        Self {
            config,
            base_model_quantized: HashMap::new(),
            base_model_scales: HashMap::new(),
            lora_adapters: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// 量化基础模型权重为 NF4
    pub fn quantize_base_model(
        &mut self,
        module_name: &str,
        weights_f32: &[f32],
    ) -> Result<(), NeoTrixError> {
        let block_size = self.config.block_size;
        let num_blocks = (weights_f32.len() + block_size - 1) / block_size;

        let mut quantized = Vec::with_capacity(weights_f32.len());
        let mut scales = Vec::with_capacity(num_blocks);

        for block_idx in 0..num_blocks {
            let start = block_idx * block_size;
            let end = (start + block_size).min(weights_f32.len());
            let block = &weights_f32[start..end];

            // 计算 block 缩放因子 (absmax 量化)
            let absmax = block.iter().map(|x| x.abs()).fold(0.0, f32::max);
            let scale = if absmax > 0.0 { 7.5 / absmax } else { 1.0 }; // NF4 量化范围 [-1, 1] -> 15 levels

            let block_quantized: Vec<NF4> = Vec::with_capacity(block.len());
            let _ = &block_quantized;
            for &w in block {
                let q = NF4::from_f32(w * scale);
                quantized.push(q);
            }
            scales.push(1.0 / scale); // 反量化 scale
        }

        self.base_model_quantized
            .insert(module_name.to_string(), quantized);
        self.base_model_scales
            .insert(module_name.to_string(), scales);
        Ok(())
    }

    /// 初始化 LoRA 适配器 (复用 lora_core 逻辑)
    pub fn init_lora_adapter(
        &mut self,
        module_name: &str,
        in_features: usize,
        out_features: usize,
    ) -> Result<(), NeoTrixError> {
        let rank = self.config.lora.rank.min(in_features).min(out_features);
        let scaling = self.config.lora.alpha / rank as f32;

        let std = (2.0 / (in_features + rank) as f32).sqrt();
        let a = (0..rank)
            .map(|_| {
                (0..in_features)
                    .map(|_| rand::random::<f32>() * std)
                    .collect()
            })
            .collect();
        let b = vec![vec![0.0; rank]; out_features];

        self.lora_adapters
            .insert(module_name.to_string(), LoRAWeights { a, b, scaling });
        Ok(())
    }

    /// 反量化基础权重 + LoRA 前向
    pub fn forward(&self, module_name: &str, input: &[f32]) -> Result<Vec<f32>, NeoTrixError> {
        // 1. 反量化基础权重
        let _quantized = self.base_model_quantized.get(module_name).ok_or_else(|| {
            NeoTrixError::NotFound(format!("Quantized base not found: {}", module_name))
        })?;
        let _scales = self
            .base_model_scales
            .get(module_name)
            .ok_or_else(|| NeoTrixError::NotFound(format!("Scales not found: {}", module_name)))?;

        let lora_w = self.lora_adapters.get(module_name).ok_or_else(|| {
            NeoTrixError::NotFound(format!("LoRA adapter not found: {}", module_name))
        })?;

        // 简化实现：实际应解包 NF4 -> f32 并矩阵乘法
        // 这里仅做维度校验和占位返回
        let in_features = lora_w.a[0].len();
        let out_features = lora_w.b.len();

        if input.len() % in_features != 0 {
            return Err(NeoTrixError::InvalidInput("Input dim mismatch".into()));
        }
        let batch = input.len() / in_features;

        // 占位：返回零向量 (实际应计算: dequant(base) @ input^T + lora_adapter @ input^T)
        Ok(vec![0.0; batch * out_features])
    }

    pub fn config(&self) -> &QLoRAConfig {
        &self.config
    }
}

impl CapabilityNode for QLoRA {
    fn node_id(&self) -> &str {
        "nt_mind::peft::qlora"
    }
    fn provides(&self) -> Vec<String> {
        vec!["quantized_lora".into()]
    }
    fn requires(&self) -> Vec<String> {
        vec!["lora_core".into(), "quantization".into()]
    }
    fn rune_sockets(&self) -> Vec<RuneSocket> {
        vec![RuneSocket::Indigo, RuneSocket::Obsidian]
    }
    fn constellation_level(&self) -> u8 {
        0
    }
    fn promote_constellation(&mut self) -> bool {
        false
    }
}

impl SelfTest for QLoRA {
    fn self_test(&self) -> Result<(), Vec<String>> {
        let inner = (|| -> Result<(), crate::core::nt_core_error::NeoTrixError> {
            // 基础结构测试
            let _qlora = QLoRA::new(QLoRAConfig::default());

            // 测试 NF4 量化/反量化
            let test_vals = [-1.0, -0.5, 0.0, 0.5, 1.0];
            for &v in &test_vals {
                let q = NF4::from_f32(v);
                let dq = q.to_f32();
                assert!((v - dq).abs() < 0.2, "NF4 量化误差过大: {} -> {}", v, dq);
            }

            // 测试配置
            let config = QLoRAConfig::default();
            assert_eq!(config.quant_bits, 4);
            assert!(config.double_quant);

            Ok(())
        })();
        inner.map_err(|e| vec![e.to_string()])
    }

    fn name(&self) -> &str {
        "nt_mind_peft_qlora"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nf4_quantization() {
        for v in [-1.0, -0.5, 0.0, 0.5, 1.0] {
            let q = NF4::from_f32(v);
            let dq = q.to_f32();
            assert!((v - dq).abs() < 0.25);
        }
    }

    #[test]
    fn test_qlora_self_test() {
        let qlora = QLoRA::new(QLoRAConfig::default());
        assert!(qlora.self_test().is_ok());
    }
}
