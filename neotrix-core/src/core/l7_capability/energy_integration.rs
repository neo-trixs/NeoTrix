//! 能量-频率-震动模型与能力网的集成
//! 
//! 将底层算法渗透到能力网生态中

use super::types::{Layer, CapabilityKind, Wisdom};
use super::traits::{CapabilityPlugin, WisdomBridge};
use crate::core::energy_core::{Frequency, Vibration, EnergyField};
use async_trait::async_trait;

/// 基于能量-频率-震动的能力插件
pub struct EnergyCapabilityPlugin {
    /// 插件名称
    name: String,
    /// 所属层级
    layer: Layer,
    /// 能力类型
    capability_kind: CapabilityKind,
    /// 频率配置
    frequency_config: Frequency,
    /// 能量场引用
    energy_field: EnergyField,
}

impl EnergyCapabilityPlugin {
    /// 创建新的能量能力插件
    pub fn new(
        name: &str,
        layer: Layer,
        capability_kind: CapabilityKind,
        frequency_config: Frequency,
    ) -> Self {
        Self {
            name: name.to_string(),
            layer,
            capability_kind,
            frequency_config,
            energy_field: EnergyField::new(),
        }
    }
    
    /// 获取频率配置
    pub fn frequency_config(&self) -> &Frequency {
        &self.frequency_config
    }
    
    /// 获取能量场
    pub fn energy_field(&self) -> &EnergyField {
        &self.energy_field
    }
}

#[async_trait]
impl CapabilityPlugin for EnergyCapabilityPlugin {
    fn name(&self) -> &'static str {
        // 注意：这里需要返回静态字符串
        Box::leak(self.name.clone().into_boxed_str())
    }
    
    fn version(&self) -> &'static str {
        "1.0.0"
    }
    
    fn layer(&self) -> Layer {
        self.layer
    }
    
    fn capability_kinds(&self) -> Vec<CapabilityKind> {
        vec![self.capability_kind]
    }
    
    fn tags(&self) -> Vec<&'static str> {
        vec!["energy", "frequency", "vibration"]
    }
    
    async fn execute(&self, input: &str) -> Result<String, String> {
        tracing::debug!(
            "能量能力插件执行: {}, 输入: {}",
            self.name,
            input
        );
        
        // 创建震动
        let vibration = Vibration::from_frequency(&self.frequency_config, input);
        
        // 计算震动能量
        let energy = vibration.energy();
        
        // 转化为智慧
        let wisdom = vibration.to_wisdom(&self.name);
        
        Ok(format!(
            "Energy plugin '{}' executed: energy={}, wisdom_strength={}",
            self.name,
            energy,
            wisdom.strength()
        ))
    }
    
    fn vector(&self) -> super::types::CapabilityVector {
        let base_freq = self.frequency_config.base_frequency();
        super::types::CapabilityVector {
            strength: base_freq,
            efficiency: 0.8,
            reliability: 0.9,
            adaptability: 0.7,
            creativity: 0.6,
        }
    }
    
    fn cost(&self) -> super::types::CapabilityCost {
        super::types::CapabilityCost {
            cpu_ms: 10,
            memory_bytes: 1024,
            network_bytes: 0,
            tokens: 0,
        }
    }
}

/// 基于能量-频率-震动的智慧桥接
pub struct EnergyWisdomBridge {
    /// 能量场
    energy_field: EnergyField,
}

impl EnergyWisdomBridge {
    /// 创建新的能量智慧桥接
    pub fn new() -> Self {
        Self {
            energy_field: EnergyField::new(),
        }
    }
    
    /// 获取能量场
    pub fn energy_field(&self) -> &EnergyField {
        &self.energy_field
    }
    
    /// 从频率创建智慧
    pub fn frequency_to_wisdom(
        &self,
        frequency: &Frequency,
        capability_id: &str,
        input: &str,
    ) -> Wisdom {
        let vibration = Vibration::from_frequency(frequency, input);
        vibration.to_wisdom(capability_id)
    }
}

impl Default for EnergyWisdomBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WisdomBridge for EnergyWisdomBridge {
    fn capability_to_wisdom(&self, capability_id: &str, result: &str, layer: Layer) -> Wisdom {
        // 根据层级创建相应的频率
        let frequency = match layer {
            Layer::L1Action => Frequency::Action {
                intensity: 0.8,
                stability: 0.9,
            },
            Layer::L2Perception => Frequency::Perception {
                bandwidth: 0.7,
                sensitivity: 0.8,
            },
            Layer::L3Embodiment => Frequency::Embodiment {
                resonance: 0.8,
                coherence: 0.9,
            },
            Layer::L4Emotion => Frequency::Emotion {
                amplitude: 0.7,
                emotional_intensity: 0.8,
            },
            Layer::L5Cognition => Frequency::Cognition {
                processing_speed: 0.8,
                depth: 0.9,
            },
            Layer::L6MetaCognition => Frequency::MetaCognition {
                awareness精度: 0.9,
                meta_ability: 0.9,
            },
        };
        
        // 创建震动并转化为智慧
        let vibration = Vibration::from_frequency(&frequency, result);
        vibration.to_wisdom(capability_id)
    }
    
    async fn accumulate(&mut self, wisdom: Wisdom) -> Result<(), String> {
        // 根据智慧类型添加能量
        let (energy, layer) = match &wisdom {
            Wisdom::Action { strength, .. } => (*strength, Layer::L1Action),
            Wisdom::Perception { confidence, .. } => (*confidence, Layer::L2Perception),
            Wisdom::Emotion { intensity, .. } => (*intensity, Layer::L4Emotion),
            Wisdom::Cognition { depth, .. } => (*depth as f64 / 10.0, Layer::L5Cognition),
            Wisdom::MetaCognition { .. } => (0.8, Layer::L6MetaCognition),
        };
        
        self.energy_field.add_energy(energy, layer).await;
        
        tracing::debug!(
            "能量智慧桥接累积智慧: 能量={}, 层级={:?}",
            energy,
            layer
        );
        
        Ok(())
    }
    
    fn get_wisdom(&self) -> Vec<Wisdom> {
        // 从能量场状态生成智慧
        vec![]
    }
    
    fn clear_wisdom(&mut self) {
        // 重置能量场
        self.energy_field = EnergyField::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_energy_capability_plugin() {
        let plugin = EnergyCapabilityPlugin::new(
            "test_plugin",
            Layer::L1Action,
            CapabilityKind::Download,
            Frequency::Action {
                intensity: 0.8,
                stability: 0.9,
            },
        );
        
        assert_eq!(plugin.layer(), Layer::L1Action);
        assert_eq!(plugin.name(), "test_plugin");
    }
    
    #[tokio::test]
    async fn test_energy_wisdom_bridge() {
        let mut bridge = EnergyWisdomBridge::new();
        
        let wisdom = Wisdom::Action {
            capability_id: "test".to_string(),
            insight: "Test insight".to_string(),
            strength: 0.8,
        };
        
        assert!(bridge.accumulate(wisdom).await.is_ok());
    }
}
