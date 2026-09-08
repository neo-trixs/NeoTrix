//! 能量场
//! 
//! 统一管理能量、频率、震动的转换
//! 
//! 底层算法：能量 → 频率 → 震动 → 显化

use super::frequency::{Frequency, FrequencySet};
use super::vibration::{Vibration, VibrationSequence};
use crate::core::l7_capability::types::{Wisdom, Layer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;

/// 能量场 - 统一管理能量转换
pub struct EnergyField {
    /// 当前能量状态
    energy_state: Arc<RwLock<EnergyState>>,
    /// 频率集合
    frequency_set: Arc<RwLock<FrequencySet>>,
    /// 震动序列
    vibration_sequence: Arc<RwLock<VibrationSequence>>,
    /// 能量转换历史
    transformation_history: Arc<RwLock<Vec<EnergyTransformation>>>,
}

/// 能量状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyState {
    /// 总能量
    pub total_energy: f64,
    /// 能量分布（按层级）
    pub energy_distribution: HashMap<Layer, f64>,
    /// 能量流动方向
    pub flow_direction: String,
    /// 能量密度
    pub density: f64,
}

/// 能量转换记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyTransformation {
    /// 转换时间戳
    pub timestamp: i64,
    /// 输入能量
    pub input_energy: f64,
    /// 输出能量
    pub output_energy: f64,
    /// 转换类型
    pub transformation_type: String,
    /// 效率
    pub efficiency: f64,
}

impl EnergyField {
    /// 创建新的能量场
    pub fn new() -> Self {
        Self {
            energy_state: Arc::new(RwLock::new(EnergyState {
                total_energy: 0.0,
                energy_distribution: HashMap::new(),
                flow_direction: "stable".to_string(),
                density: 0.0,
            })),
            frequency_set: Arc::new(RwLock::new(FrequencySet::new())),
            vibration_sequence: Arc::new(RwLock::new(VibrationSequence::new())),
            transformation_history: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// 添加能量
    pub async fn add_energy(&mut self, energy: f64, layer: Layer) {
        let mut state = self.energy_state.write().await;
        state.total_energy += energy;
        *state.energy_distribution.entry(layer).or_insert(0.0) += energy;
        state.density = state.total_energy / 100.0; // 假设最大容量为100
        
        tracing::debug!(
            "能量场添加能量: {} (层级: {:?}), 总能量: {}",
            energy,
            layer,
            state.total_energy
        );
    }
    
    /// 创建频率
    pub async fn create_frequency(&mut self, frequency: Frequency) {
        let mut freq_set = self.frequency_set.write().await;
        freq_set.add(frequency);
        
        tracing::debug!("能量场创建频率，当前频率数: {}", freq_set.count());
    }
    
    /// 产生震动
    pub async fn produce_vibration(&mut self, frequency: &Frequency, input: &str) -> Vibration {
        let vibration = Vibration::from_frequency(frequency, input);
        
        let mut seq = self.vibration_sequence.write().await;
        seq.add(vibration.clone());
        
        tracing::debug!(
            "能量场产生震动，总震动数: {}, 总能量: {}",
            seq.count(),
            seq.total_energy()
        );
        
        vibration
    }
    
    /// 能量转换：频率 → 震动 → 智慧
    pub async fn transform_energy(
        &mut self,
        input_energy: f64,
        frequency: Frequency,
        capability_id: &str,
    ) -> Wisdom {
        let _start_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        // 1. 添加能量
        self.add_energy(input_energy, frequency.layer()).await;
        
        // 2. 创建频率
        self.create_frequency(frequency.clone()).await;
        
        // 3. 产生震动
        let vibration = self.produce_vibration(&frequency, capability_id).await;
        
        // 4. 转化为智慧
        let wisdom = vibration.to_wisdom(capability_id);
        
        let end_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        // 记录转换
        let transformation = EnergyTransformation {
            timestamp: end_time,
            input_energy,
            output_energy: wisdom.strength(),
            transformation_type: format!("Frequency to Wisdom via {:?}", frequency.layer()),
            efficiency: if input_energy > 0.0 {
                wisdom.strength() / input_energy
            } else {
                0.0
            },
        };
        
        let mut history = self.transformation_history.write().await;
        history.push(transformation);
        
        tracing::debug!(
            "能量转换完成: {} → {} (效率: {})",
            input_energy,
            wisdom.strength(),
            wisdom.strength() / input_energy.max(0.001)
        );
        
        wisdom
    }
    
    /// 获取能量状态
    pub async fn get_energy_state(&self) -> EnergyState {
        self.energy_state.read().await.clone()
    }
    
    /// 获取频率集合
    pub async fn get_frequency_set(&self) -> FrequencySet {
        self.frequency_set.read().await.clone()
    }
    
    /// 获取震动序列
    pub async fn get_vibration_sequence(&self) -> VibrationSequence {
        self.vibration_sequence.read().await.clone()
    }
    
    /// 获取转换历史
    pub async fn get_transformation_history(&self, limit: usize) -> Vec<EnergyTransformation> {
        let history = self.transformation_history.read().await;
        history.iter().rev().take(limit).cloned().collect()
    }
    
    /// 计算能量场健康度
    pub async fn health(&self) -> f64 {
        let state = self.energy_state.read().await;
        
        // 健康度基于能量密度和分布均匀性
        let density_score = 1.0 - (state.density - 0.5).abs();
        
        // 分布均匀性
        let distribution_score = if state.energy_distribution.is_empty() {
            0.0
        } else {
            let total = state.total_energy;
            let variance: f64 = state.energy_distribution
                .values()
                .map(|&v| {
                    let ideal = total / state.energy_distribution.len() as f64;
                    (v - ideal).powi(2)
                })
                .sum::<f64>() / state.energy_distribution.len() as f64;
            
            1.0 - variance.sqrt()
        };
        
        (density_score + distribution_score) / 2.0
    }
}

impl Default for EnergyField {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_add_energy() {
        let mut field = EnergyField::new();
        
        field.add_energy(10.0, Layer::L1Action).await;
        
        let state = field.get_energy_state().await;
        assert_eq!(state.total_energy, 10.0);
    }
    
    #[tokio::test]
    async fn test_transform_energy() {
        let mut field = EnergyField::new();
        
        let frequency = Frequency::Cognition {
            processing_speed: 0.8,
            depth: 0.8,
        };
        
        let wisdom = field.transform_energy(0.5, frequency, "test").await;
        
        match wisdom {
            Wisdom::Cognition { .. } => assert!(true),
            _ => panic!("Expected Cognition wisdom"),
        }
        
        let state = field.get_energy_state().await;
        assert_eq!(state.total_energy, 0.5);
    }
    
    #[tokio::test]
    async fn test_health() {
        let mut field = EnergyField::new();
        
        // 添加一些能量
        field.add_energy(50.0, Layer::L1Action).await;
        field.add_energy(50.0, Layer::L5Cognition).await;
        
        let health = field.health().await;
        assert!(health > 0.5);
    }
}
