//! 震动层
//! 
//! 实现能量的震动转换
//! 
//! 底层算法：能量 → 频率 → 震动 → 显化

use super::frequency::Frequency;
use crate::core::l7_capability::types::Wisdom;
use serde::{Deserialize, Serialize};

/// 震动类型 - 能量的具体表现形式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Vibration {
    /// 行动震动 - 能量转化为物理动作
    Action {
        /// 震动强度
        intensity: f64,
        /// 震动方向
        direction: String,
        /// 持续时间
        duration_ms: u64,
    },
    
    /// 感知震动 - 能量转化为感知信号
    Perception {
        /// 震动频率
        frequency: f64,
        /// 震动模式
        pattern: String,
        /// 感知深度
        depth: f64,
    },
    
    /// 具身震动 - 能量在身体中的共振
    Embodiment {
        /// 共振强度
        resonance: f64,
        /// 共振区域
        region: String,
        /// 协调度
        coherence: f64,
    },
    
    /// 情感震动 - 能量转化为情感波动
    Emotion {
        /// 波动幅度
        amplitude: f64,
        /// 情感类型
        emotion_type: String,
        /// 持续时间
        duration_ms: u64,
    },
    
    /// 认知震动 - 能量转化为思维活动
    Cognition {
        /// 处理速度
        processing_speed: f64,
        /// 思考深度
        depth: f64,
        /// 创造力
        creativity: f64,
    },
    
    /// 元认知震动 - 能量转化为觉知活动
    MetaCognition {
        /// 觉知精度
        awareness: f64,
        /// 反思深度
        reflection_depth: f64,
        /// 洞察力
        insight: f64,
    },
}

impl Vibration {
    /// 从频率创建震动
    pub fn from_frequency(frequency: &Frequency, input: &str) -> Self {
        match frequency {
            Frequency::Action { intensity, stability } => {
                Vibration::Action {
                    intensity: *intensity,
                    direction: input.to_string(),
                    duration_ms: (*stability * 1000.0) as u64,
                }
            }
            Frequency::Perception { bandwidth, sensitivity } => {
                Vibration::Perception {
                    frequency: *bandwidth,
                    pattern: input.to_string(),
                    depth: *sensitivity,
                }
            }
            Frequency::Embodiment { resonance, coherence } => {
                Vibration::Embodiment {
                    resonance: *resonance,
                    region: input.to_string(),
                    coherence: *coherence,
                }
            }
            Frequency::Emotion { amplitude, emotional_intensity } => {
                Vibration::Emotion {
                    amplitude: *amplitude,
                    emotion_type: input.to_string(),
                    duration_ms: (*emotional_intensity * 1000.0) as u64,
                }
            }
            Frequency::Cognition { processing_speed, depth } => {
                Vibration::Cognition {
                    processing_speed: *processing_speed,
                    depth: *depth,
                    creativity: 0.5,
                }
            }
            Frequency::MetaCognition { awareness精度, meta_ability } => {
                Vibration::MetaCognition {
                    awareness: *awareness精度,
                    reflection_depth: *meta_ability,
                    insight: 0.5,
                }
            }
        }
    }
    
    /// 计算震动能量
    pub fn energy(&self) -> f64 {
        match self {
            Vibration::Action { intensity, .. } => *intensity,
            Vibration::Perception { frequency, .. } => *frequency,
            Vibration::Embodiment { resonance, .. } => *resonance,
            Vibration::Emotion { amplitude, .. } => *amplitude,
            Vibration::Cognition { processing_speed, .. } => *processing_speed,
            Vibration::MetaCognition { awareness, .. } => *awareness,
        }
    }
    
    /// 转化为智慧
    pub fn to_wisdom(&self, capability_id: &str) -> Wisdom {
        match self {
            Vibration::Action { intensity, direction, .. } => {
                Wisdom::Action {
                    capability_id: capability_id.to_string(),
                    insight: format!("Action vibration: {} with intensity {}", direction, intensity),
                    strength: *intensity,
                }
            }
            Vibration::Perception { frequency, pattern, .. } => {
                Wisdom::Perception {
                    capability_id: capability_id.to_string(),
                    pattern: pattern.clone(),
                    confidence: *frequency,
                }
            }
            Vibration::Embodiment { resonance, region, .. } => {
                Wisdom::Action {
                    capability_id: capability_id.to_string(),
                    insight: format!("Embodiment resonance in {}: {}", region, resonance),
                    strength: *resonance,
                }
            }
            Vibration::Emotion { amplitude, emotion_type, .. } => {
                Wisdom::Emotion {
                    capability_id: capability_id.to_string(),
                    emotion: emotion_type.clone(),
                    intensity: *amplitude,
                }
            }
            Vibration::Cognition { depth, .. } => {
                Wisdom::Cognition {
                    capability_id: capability_id.to_string(),
                    reasoning: "Cognitive vibration".to_string(),
                    depth: (*depth * 10.0) as u32,
                }
            }
            Vibration::MetaCognition { awareness, .. } => {
                Wisdom::MetaCognition {
                    capability_id: capability_id.to_string(),
                    reflection: "Meta-cognitive vibration".to_string(),
                    insight: format!("Awareness level: {}", awareness),
                }
            }
        }
    }
}

/// 震动序列 - 表示一系列震动的转换
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibrationSequence {
    /// 震动序列
    vibrations: Vec<Vibration>,
    /// 总能量
    total_energy: f64,
}

impl VibrationSequence {
    /// 创建新的震动序列
    pub fn new() -> Self {
        Self {
            vibrations: vec![],
            total_energy: 0.0,
        }
    }
    
    /// 添加震动
    pub fn add(&mut self, vibration: Vibration) {
        self.total_energy += vibration.energy();
        self.vibrations.push(vibration);
    }
    
    /// 获取总能量
    pub fn total_energy(&self) -> f64 {
        self.total_energy
    }
    
    /// 获取震动数量
    pub fn count(&self) -> usize {
        self.vibrations.len()
    }
    
    /// 转化为智慧序列
    pub fn to_wisdoms(&self, capability_id: &str) -> Vec<Wisdom> {
        self.vibrations
            .iter()
            .map(|v| v.to_wisdom(capability_id))
            .collect()
    }
}

impl Default for VibrationSequence {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vibration_from_frequency() {
        let frequency = Frequency::Action {
            intensity: 0.8,
            stability: 0.9,
        };
        
        let vibration = Vibration::from_frequency(&frequency, "test input");
        
        match vibration {
            Vibration::Action { intensity, .. } => {
                assert_eq!(intensity, 0.8);
            }
            _ => panic!("Expected Action vibration"),
        }
    }
    
    #[test]
    fn test_vibration_energy() {
        let vibration = Vibration::Cognition {
            processing_speed: 0.7,
            depth: 0.8,
            creativity: 0.6,
        };
        
        assert_eq!(vibration.energy(), 0.7);
    }
    
    #[test]
    fn test_vibration_to_wisdom() {
        let vibration = Vibration::Emotion {
            amplitude: 0.6,
            emotion_type: "joy".to_string(),
            duration_ms: 1000,
        };
        
        let wisdom = vibration.to_wisdom("test_capability");
        
        match wisdom {
            Wisdom::Emotion { emotion, intensity, .. } => {
                assert_eq!(emotion, "joy");
                assert_eq!(intensity, 0.6);
            }
            _ => panic!("Expected Emotion wisdom"),
        }
    }
}
