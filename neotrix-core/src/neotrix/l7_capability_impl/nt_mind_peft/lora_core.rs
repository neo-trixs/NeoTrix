//! nt_mind::peft::lora_core — LoRA 核心适配器
//!
//! 对应能力注册表节点: nt_mind::peft::lora_core (L0)
//! Provides: lora_adaptation
//! Requires: tensor_algebra
//! Rune Sockets: Crimson (数据摄取), Indigo (变换)
//! Constellation Path: C0→C1→C2→C3→C4→C5
//!
//! 论文参考: LoRA (2106.09685) — 低秩分解 A·B = ΔW
//! 实现参考: LLaMA-Factory (hiyouga/LLaMA-Factory), PEFT (huggingface/peft)

#![forbid(unsafe_code)]

use crate::core::nt_core_error::NeoTrixError;
use crate::core::nt_core_traits::{CapabilityNode, RuneSocket, SelfTest};
use std::collections::HashMap;

/// LoRA 配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LoRAConfig {
    /// 秩
    pub rank: usize,
    /// LoRA alpha 缩放因子
    pub alpha: f32,
    /// Dropout 概率
    pub dropout: f32,
    /// 目标模块 (如 "q_proj", "v_proj", "k_proj", "o_proj", "gate_proj", "up_proj", "down_proj")
    pub target_modules: Vec<String>,
    /// 是否使用 RSLoRA 缩放
    pub use_rslora: bool,
    /// 初始化方式
    pub init_method: LoRAInitMethod,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum LoRAInitMethod {
    Gaussian { std: f32 },
    LoRADefault,
    ZeroInit,
}

impl Default for LoRAConfig {
    fn default() -> Self {
        Self {
            rank: 8,
            alpha: 16.0,
            dropout: 0.0,
            target_modules: vec![
                "q_proj".into(),
                "v_proj".into(),
                "k_proj".into(),
                "o_proj".into(),
                "gate_proj".into(),
                "up_proj".into(),
                "down_proj".into(),
            ],
            use_rslora: false,
            init_method: LoRAInitMethod::LoRADefault,
        }
    }
}

/// LoRA 适配器权重
#[derive(Debug, Clone)]
pub struct LoRAWeights {
    /// A 矩阵: (rank, in_features)
    pub a: Vec<Vec<f32>>,
    /// B 矩阵: (out_features, rank)
    pub b: Vec<Vec<f32>>,
    /// 缩放因子 alpha / rank
    pub scaling: f32,
}

/// LoRA 核心能力实现
pub struct LoRACore {
    config: LoRAConfig,
    adapters: HashMap<String, LoRAWeights>, // module_name -> weights
    metadata: HashMap<String, serde_json::Value>,
}

impl LoRACore {
    /// 创建新的 LoRA 核心实例
    pub fn new(config: LoRAConfig) -> Self {
        Self {
            config,
            adapters: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// 为指定模块初始化 LoRA 适配器
    ///
    /// # Arguments
    /// * `module_name` - 模块名称 (如 "model.layers.0.self_attn.q_proj")
    /// * `in_features` - 输入维度
    /// * `out_features` - 输出维度
    pub fn init_adapter(
        &mut self,
        module_name: &str,
        in_features: usize,
        out_features: usize,
    ) -> Result<(), NeoTrixError> {
        let rank = self.config.rank.min(in_features).min(out_features);
        let scaling = self.config.alpha / rank as f32;

        let (a, b) = match self.config.init_method {
            LoRAInitMethod::Gaussian { std } => {
                let a = (0..rank)
                    .map(|_| {
                        (0..in_features)
                            .map(|_| rand::random::<f32>() * std)
                            .collect()
                    })
                    .collect();
                let b = vec![vec![0.0; rank]; out_features];
                (a, b)
            }
            LoRAInitMethod::LoRADefault => {
                let std = (2.0 / (in_features + rank) as f32).sqrt();
                let a = (0..rank)
                    .map(|_| {
                        (0..in_features)
                            .map(|_| rand::random::<f32>() * std)
                            .collect()
                    })
                    .collect();
                let b = vec![vec![0.0; rank]; out_features];
                (a, b)
            }
            LoRAInitMethod::ZeroInit => {
                let a = vec![vec![0.0; in_features]; rank];
                let b = vec![vec![0.0; rank]; out_features];
                (a, b)
            }
        };

        self.adapters
            .insert(module_name.to_string(), LoRAWeights { a, b, scaling });
        Ok(())
    }

    /// 前向传播：计算 LoRA 增量输出
    ///
    /// # Arguments
    /// * `module_name` - 模块名称
    /// * `input` - 输入张量 [..., in_features]
    ///
    /// # Returns
    /// 增量输出 [..., out_features]
    pub fn forward(&self, module_name: &str, input: &[f32]) -> Result<Vec<f32>, NeoTrixError> {
        let weights = self
            .adapters
            .get(module_name)
            .ok_or_else(|| NeoTrixError::NotFound(format!("Adapter not found: {}", module_name)))?;

        let in_features = weights.a[0].len();
        let out_features = weights.b.len();
        let rank = weights.a.len();

        // 校验输入维度
        if input.len() % in_features != 0 {
            return Err(NeoTrixError::InvalidInput(format!(
                "Input length {} not divisible by in_features {}",
                input.len(),
                in_features
            )));
        }

        let batch = input.len() / in_features;
        let mut output = vec![0.0; batch * out_features];

        // 计算: output = (input @ A^T) @ B^T * scaling
        // 这里简化为单批次实现，实际应用中应使用 BLAS/GEMM
        for b in 0..batch {
            let x = &input[b * in_features..(b + 1) * in_features];
            // 1. x @ A^T -> [rank]
            let mut hidden = vec![0.0; rank];
            for r in 0..rank {
                let mut sum = 0.0;
                for i in 0..in_features {
                    sum += x[i] * weights.a[r][i];
                }
                hidden[r] = sum;
            }
            // 应用 dropout (训练时)
            if self.config.dropout > 0.0 {
                // 实际应用: 随机置零
            }
            // 2. hidden @ B^T -> [out_features]
            for o in 0..out_features {
                let mut sum = 0.0;
                for r in 0..rank {
                    sum += hidden[r] * weights.b[o][r];
                }
                output[b * out_features + o] = sum * weights.scaling;
            }
        }

        Ok(output)
    }

    /// 合并 LoRA 权重到基础模型 (推理时使用，无额外开销)
    ///
    /// # Returns
    /// (delta_W) 权重增量矩阵 [out_features, in_features]
    pub fn merge_weights(&self, module_name: &str) -> Result<Vec<Vec<f32>>, NeoTrixError> {
        let weights = self
            .adapters
            .get(module_name)
            .ok_or_else(|| NeoTrixError::NotFound(format!("Adapter not found: {}", module_name)))?;

        let out_features = weights.b.len();
        let in_features = weights.a[0].len();
        let rank = weights.a.len();

        // delta_W = B @ A * scaling
        let mut delta_w = vec![vec![0.0; in_features]; out_features];
        for o in 0..out_features {
            for i in 0..in_features {
                let mut sum = 0.0;
                for r in 0..rank {
                    sum += weights.b[o][r] * weights.a[r][i];
                }
                delta_w[o][i] = sum * weights.scaling;
            }
        }

        Ok(delta_w)
    }

    /// 获取配置
    pub fn config(&self) -> &LoRAConfig {
        &self.config
    }

    /// 获取所有适配器名称
    pub fn adapter_names(&self) -> Vec<String> {
        self.adapters.keys().cloned().collect()
    }

    /// 设置元数据 (用于能力注册表记录演化日志等)
    pub fn set_metadata(&mut self, key: &str, value: serde_json::Value) {
        self.metadata.insert(key.to_string(), value);
    }

    /// 获取元数据
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }
}

/// CapabilityNode 实现
impl CapabilityNode for LoRACore {
    fn node_id(&self) -> &str {
        "nt_mind::peft::lora_core"
    }

    fn provides(&self) -> Vec<String> {
        vec!["lora_adaptation".to_string()]
    }

    fn requires(&self) -> Vec<String> {
        vec!["tensor_algebra".to_string()]
    }

    fn rune_sockets(&self) -> Vec<RuneSocket> {
        vec![RuneSocket::Crimson, RuneSocket::Indigo]
    }

    fn constellation_level(&self) -> u8 {
        0 // C0: 编译通过
    }

    fn promote_constellation(&mut self) -> bool {
        // C0 -> C1: 单元测试通过
        // C1 -> C2: 集成测试通过
        // 实际实现中应检查测试结果
        false
    }
}

/// SelfTest 实现 (T1 Existence + T2 Registration)
impl SelfTest for LoRACore {
    fn self_test(&self) -> Result<(), Vec<String>> {
        let inner = (|| -> Result<(), crate::core::nt_core_error::NeoTrixError> {
            // T1: 结构存在性
            // T2: 注册表注册 (外部完成)
            // 这里做基础功能冒烟测试

            let mut lora = LoRACore::new(LoRAConfig {
                rank: 4,
                alpha: 8.0,
                target_modules: vec!["test_proj".into()],
                ..Default::default()
            });

            // 测试初始化
            lora.init_adapter("test_proj", 32, 64)?;

            // 测试前向
            let input = vec![1.0; 32];
            let output = lora.forward("test_proj", &input)?;
            assert_eq!(output.len(), 64);

            // 测试合并权重
            let delta = lora.merge_weights("test_proj")?;
            assert_eq!(delta.len(), 64);
            assert_eq!(delta[0].len(), 32);

            Ok(())
        })();
        inner.map_err(|e| vec![e.to_string()])
    }

    fn name(&self) -> &str {
        "nt_mind_peft_lora_core"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lora_core_basic() {
        let mut lora = LoRACore::new(LoRAConfig {
            rank: 8,
            alpha: 16.0,
            target_modules: vec!["q_proj".into(), "v_proj".into()],
            ..Default::default()
        });

        // 初始化两个适配器
        lora.init_adapter("q_proj", 512, 512).unwrap();
        lora.init_adapter("v_proj", 512, 512).unwrap();

        // 前向测试
        let input = vec![0.5; 512];
        let out_q = lora.forward("q_proj", &input).unwrap();
        let out_v = lora.forward("v_proj", &input).unwrap();
        assert_eq!(out_q.len(), 512);
        assert_eq!(out_v.len(), 512);

        // 合并权重测试
        let delta_q = lora.merge_weights("q_proj").unwrap();
        assert_eq!(delta_q.len(), 512);
        assert_eq!(delta_q[0].len(), 512);
    }

    #[test]
    fn test_lora_config_defaults() {
        let config = LoRAConfig::default();
        assert_eq!(config.rank, 8);
        assert_eq!(config.alpha, 16.0);
        assert!(config.target_modules.contains(&"q_proj".to_string()));
    }

    #[test]
    fn test_lora_self_test() {
        let lora = LoRACore::new(LoRAConfig::default());
        assert!(lora.self_test().is_ok());
    }
}
