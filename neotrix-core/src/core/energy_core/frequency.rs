//! 频率层
//! 
//! 定义不同层级的能量频率模式
//! 
//! 底层算法：能量 → 频率 → 震动 → 显化

use serde::{Deserialize, Serialize};

/// 频率类型 - 每个层级有独特的振动模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Frequency {
    /// L1 行动层：低频、稳定、接地
    /// 如：物理世界的重力频率
    Action {
        /// 频率强度 (0.0-1.0)
        intensity: f64,
        /// 频率稳定性 (0.0-1.0)
        stability: f64,
    },
    
    /// L2 感知层：中频、流动、敏感
    /// 如：感官神经的传导频率
    Perception {
        /// 频率带宽 (0.0-1.0)
        bandwidth: f64,
        /// 灵敏度 (0.0-1.0)
        sensitivity: f64,
    },
    
    /// L3 具身层：共振频率、协调、整体
    /// 如：身体各器官的共振频率
    Embodiment {
        /// 共振强度 (0.0-1.0)
        resonance: f64,
        /// 协调度 (0.0-1.0)
        coherence: f64,
    },
    
    /// L4 情感层：波动频率、起伏、情感
    /// 如：情绪的波动频率
    Emotion {
        /// 波动幅度 (0.0-1.0)
        amplitude: f64,
        /// 情感强度 (0.0-1.0)
        emotional_intensity: f64,
    },
    
    /// L5 认知层：高频、精细、思考
    /// 如：神经元放电频率
    Cognition {
        /// 处理速度 (0.0-1.0)
        processing_speed: f64,
        /// 思考深度 (0.0-1.0)
        depth: f64,
    },
    
    /// L6 元认知层：超高频、精微、觉知
    /// 如：意识的觉知频率
    MetaCognition {
        /// 觉知精度 (0.0-1.0)
        awareness精度: f64,
        /// 元认知能力 (0.0-1.0)
        meta_ability: f64,
    },
}

impl Frequency {
    /// 计算基础频率值 (0.0-1.0)
    pub fn base_frequency(&self) -> f64 {
        match self {
            Frequency::Action { intensity, stability } => {
                // 低频：0.1-0.3
                0.2 * intensity * stability
            }
            Frequency::Perception { bandwidth, sensitivity } => {
                // 中频：0.3-0.5
                0.4 * bandwidth * sensitivity
            }
            Frequency::Embodiment { resonance, coherence } => {
                // 共振频率：0.4-0.6
                0.5 * resonance * coherence
            }
            Frequency::Emotion { amplitude, emotional_intensity } => {
                // 波动频率：0.5-0.7
                0.6 * amplitude * emotional_intensity
            }
            Frequency::Cognition { processing_speed, depth } => {
                // 高频：0.7-0.9
                0.8 * processing_speed * depth
            }
            Frequency::MetaCognition { awareness精度, meta_ability } => {
                // 超高频：0.9-1.0
                0.95 * awareness精度 * meta_ability
            }
        }
    }
    
    /// 获取频率层级
    pub fn layer(&self) -> crate::core::l7_capability::types::Layer {
        match self {
            Frequency::Action { .. } => crate::core::l7_capability::types::Layer::L1Action,
            Frequency::Perception { .. } => crate::core::l7_capability::types::Layer::L2Perception,
            Frequency::Embodiment { .. } => crate::core::l7_capability::types::Layer::L3Embodiment,
            Frequency::Emotion { .. } => crate::core::l7_capability::types::Layer::L4Emotion,
            Frequency::Cognition { .. } => crate::core::l7_capability::types::Layer::L5Cognition,
            Frequency::MetaCognition { .. } => crate::core::l7_capability::types::Layer::L6MetaCognition,
        }
    }
    
    /// 检查频率是否兼容（可以产生共振）
    pub fn is_compatible_with(&self, other: &Frequency) -> bool {
        let self_freq = self.base_frequency();
        let other_freq = other.base_frequency();
        
        // 频率差异小于0.2则兼容
        (self_freq - other_freq).abs() < 0.2
    }
    
    /// 计算两个频率的共振强度
    pub fn resonance_with(&self, other: &Frequency) -> f64 {
        let self_freq = self.base_frequency();
        let other_freq = other.base_frequency();
        
        // 频率越接近，共振越强
        let diff = (self_freq - other_freq).abs();
        (1.0 - diff).max(0.0)
    }
}

/// 频率集合 - 表示多个层级的频率组合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequencySet {
    /// 所有频率
    frequencies: Vec<Frequency>,
    /// 综合频率
    combined_frequency: f64,
}

impl FrequencySet {
    /// 创建新的频率集合
    pub fn new() -> Self {
        Self {
            frequencies: vec![],
            combined_frequency: 0.0,
        }
    }
    
    /// 添加频率
    pub fn add(&mut self, frequency: Frequency) {
        self.frequencies.push(frequency);
        self.recalculate();
    }
    
    /// 重新计算综合频率
    fn recalculate(&mut self) {
        if self.frequencies.is_empty() {
            self.combined_frequency = 0.0;
            return;
        }
        
        let total: f64 = self.frequencies.iter().map(|f| f.base_frequency()).sum();
        self.combined_frequency = total / self.frequencies.len() as f64;
    }
    
    /// 获取综合频率
    pub fn combined(&self) -> f64 {
        self.combined_frequency
    }
    
    /// 获取频率数量
    pub fn count(&self) -> usize {
        self.frequencies.len()
    }
    
    /// 检查是否包含特定层级的频率
    pub fn has_layer(&self, layer: crate::core::l7_capability::types::Layer) -> bool {
        self.frequencies.iter().any(|f| f.layer() == layer)
    }
    
    /// 获取特定层级的频率
    pub fn get_by_layer(&self, layer: crate::core::l7_capability::types::Layer) -> Option<&Frequency> {
        self.frequencies.iter().find(|f| f.layer() == layer)
    }
}

impl Default for FrequencySet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_frequency_base() {
        let action_freq = Frequency::Action {
            intensity: 0.8,
            stability: 0.9,
        };
        
        assert!((action_freq.base_frequency() - 0.144).abs() < 0.001);
    }
    
    #[test]
    fn test_frequency_compatibility() {
        let freq1 = Frequency::Action {
            intensity: 0.5,
            stability: 0.5,
        };
        
        let freq2 = Frequency::Perception {
            bandwidth: 0.5,
            sensitivity: 0.5,
        };
        
        assert!(freq1.is_compatible_with(&freq2));
    }
    
    #[test]
    fn test_resonance() {
        let freq1 = Frequency::Cognition {
            processing_speed: 0.8,
            depth: 0.8,
        };
        
        let freq2 = Frequency::MetaCognition {
            awareness精度: 0.9,
            meta_ability: 0.9,
        };
        
        let resonance = freq1.resonance_with(&freq2);
        assert!(resonance > 0.5);
    }
}
