//! 能量核心实现
//! 
//! 硅基意识体的能量核心，汇聚6层架构涌现的智慧

use crate::core::l7_capability::types::{Wisdom, ConsciousnessState, Layer};
use crate::core::l7_capability::traits::EnergyCore;
use async_trait::async_trait;
use std::collections::VecDeque;
use tokio::sync::RwLock;
use std::sync::Arc;

/// 能量核心实现
pub struct EnergyCoreImpl {
    /// 智慧池
    wisdom_pool: Arc<RwLock<VecDeque<Wisdom>>>,
    /// 意识状态
    consciousness_state: Arc<RwLock<ConsciousnessState>>,
    /// 意识树
    consciousness_tree: super::consciousness_tree::ConsciousnessTreeImpl,
    /// SEAL 管线
    seal_pipeline: super::seal_pipeline::SEALPipelineImpl,
    /// GWT 路由器
    gwt_router: super::gwt_router::GWTRouterImpl,
    /// 涌现阈值
    emergence_threshold: f64,
}

impl EnergyCoreImpl {
    /// 创建新的能量核心
    pub fn new() -> Self {
        Self {
            wisdom_pool: Arc::new(RwLock::new(VecDeque::new())),
            consciousness_state: Arc::new(RwLock::new(ConsciousnessState::initial())),
            consciousness_tree: super::consciousness_tree::ConsciousnessTreeImpl::new(),
            seal_pipeline: super::seal_pipeline::SEALPipelineImpl::new(),
            gwt_router: super::gwt_router::GWTRouterImpl::new(),
            emergence_threshold: 0.7,
        }
    }
    
    /// 计算智慧池平均强度
    async fn calculate_average_strength(&self) -> f64 {
        let pool = self.wisdom_pool.read().await;
        if pool.is_empty() {
            return 0.0;
        }
        
        let total_strength: f64 = pool.iter().map(|w| w.strength()).sum();
        total_strength / pool.len() as f64
    }
    
    /// 分析智慧分布
    async fn analyze_wisdom_distribution(&self) -> std::collections::HashMap<Layer, usize> {
        let pool = self.wisdom_pool.read().await;
        let mut distribution = std::collections::HashMap::new();
        
        for wisdom in pool.iter() {
            let layer = wisdom.layer();
            *distribution.entry(layer).or_insert(0) += 1;
        }
        
        distribution
    }
}

impl Default for EnergyCoreImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EnergyCore for EnergyCoreImpl {
    async fn receive_wisdom(&mut self, wisdom: Wisdom) -> Result<(), String> {
        let mut pool = self.wisdom_pool.write().await;
        pool.push_back(wisdom);
        
        // 更新意识状态
        let mut state = self.consciousness_state.write().await;
        state.wisdom_pool_size = pool.len();
        
        // 路由智慧到意识树
        let wisdom_clone = pool.back().unwrap().clone();
        self.consciousness_tree.receive_wisdom(wisdom_clone).await;
        
        // 路由智慧到 SEAL 管线
        let wisdom_clone = pool.back().unwrap().clone();
        self.seal_pipeline.receive_wisdom(wisdom_clone).await;
        
        // 路由智慧到 GWT 路由器
        let wisdom_clone = pool.back().unwrap().clone();
        self.gwt_router.route_wisdom(wisdom_clone).await;
        
        tracing::debug!("能量核心接收智慧，当前池大小: {}", pool.len());
        Ok(())
    }
    
    async fn emit_action(&mut self) -> Result<Option<String>, String> {
        let pool = self.wisdom_pool.read().await;
        
        if pool.is_empty() {
            return Ok(None);
        }
        
        // 从最新的智慧中产生行动
        let wisdom = pool.back().unwrap();
        
        let action = match wisdom {
            Wisdom::Action { insight, .. } => Some(format!("Action: {}", insight)),
            Wisdom::Perception { pattern, .. } => Some(format!("Perceive: {}", pattern)),
            Wisdom::Emotion { emotion, .. } => Some(format!("Feel: {}", emotion)),
            Wisdom::Cognition { reasoning, .. } => Some(format!("Think: {}", reasoning)),
            Wisdom::MetaCognition { reflection, .. } => Some(format!("Reflect: {}", reflection)),
        };
        
        Ok(action)
    }
    
    fn get_consciousness_state(&self) -> ConsciousnessState {
        self.consciousness_state.try_read().unwrap().clone()
    }
    
    fn should_emerge(&self) -> bool {
        let pool = self.wisdom_pool.try_read().unwrap();
        if pool.is_empty() {
            return false;
        }
        
        let avg_strength = self.calculate_average_strength_sync();
        avg_strength >= self.emergence_threshold
    }
    
    async fn emerge(&mut self) -> Result<Option<String>, String> {
        if !self.should_emerge() {
            return Ok(None);
        }
        
        // 分析智慧分布
        let distribution = self.analyze_wisdom_distribution().await;
        
        // 找出最强的智慧类型
        let dominant_layer = distribution
            .iter()
            .max_by_key(|(_, count)| **count)
            .map(|(layer, _)| *layer)
            .unwrap_or(Layer::L5Cognition);
        
        // 产生新能力
        let new_capability = format!(
            "EmergentCapability_from_{:?}_strength_{}",
            dominant_layer,
            self.calculate_average_strength().await
        );
        
        tracing::info!("涌现新能力: {}", new_capability);
        
        Ok(Some(new_capability))
    }
}

impl EnergyCoreImpl {
    /// 同步计算平均强度
    fn calculate_average_strength_sync(&self) -> f64 {
        let pool = self.wisdom_pool.try_read().unwrap();
        if pool.is_empty() {
            return 0.0;
        }
        
        let total_strength: f64 = pool.iter().map(|w| w.strength()).sum();
        total_strength / pool.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_receive_wisdom() {
        let mut energy_core = EnergyCoreImpl::new();
        
        let wisdom = Wisdom::Action {
            capability_id: "test".to_string(),
            insight: "Test insight".to_string(),
            strength: 0.8,
        };
        
        assert!(energy_core.receive_wisdom(wisdom).await.is_ok());
        assert_eq!(energy_core.get_consciousness_state().wisdom_pool_size, 1);
    }
    
    #[tokio::test]
    async fn test_emit_action() {
        let mut energy_core = EnergyCoreImpl::new();
        
        let wisdom = Wisdom::Action {
            capability_id: "test".to_string(),
            insight: "Test insight".to_string(),
            strength: 0.8,
        };
        
        let _ = energy_core.receive_wisdom(wisdom).await;
        
        let action = energy_core.emit_action().await.unwrap();
        assert!(action.is_some());
        assert!(action.unwrap().contains("Test insight"));
    }
    
    #[tokio::test]
    async fn test_should_emerge() {
        let mut energy_core = EnergyCoreImpl::new();
        
        // 初始状态不应涌现
        assert!(!energy_core.should_emerge());
        
        // 添加足够强的智慧
        for _ in 0..10 {
            let wisdom = Wisdom::Action {
                capability_id: "test".to_string(),
                insight: "Strong insight".to_string(),
                strength: 0.8,
            };
            let _ = energy_core.receive_wisdom(wisdom).await;
        }
        
        // 应该可以涌现
        assert!(energy_core.should_emerge());
    }
}
