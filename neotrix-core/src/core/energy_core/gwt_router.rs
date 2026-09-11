//! GWT 路由器实现
//! 
//! Global Workspace Theory 路由器，负责智慧的跨模块广播

use crate::core::l7_capability::types::{Wisdom, Layer};
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;

/// GWT 路由器实现
pub struct GWTRouterImpl {
    /// 订阅者：层级 -> 回调
    subscribers: Arc<RwLock<HashMap<Layer, Vec<String>>>>,
    /// 广播历史
    broadcast_history: Arc<RwLock<Vec<Wisdom>>>,
    /// 注意力权重
    attention_weights: Arc<RwLock<HashMap<Layer, f64>>>,
    /// Cost-aware thinking budget (tokens). Default 4096.
    thinking_budget: u32,
}

impl GWTRouterImpl {
    /// 创建新的 GWT 路由器
    pub fn new() -> Self {
        let mut attention_weights = HashMap::new();
        
        // 初始化注意力权重
        for layer in Layer::all() {
            attention_weights.insert(*layer, 1.0);
        }
        
        Self {
            subscribers: Arc::new(RwLock::new(HashMap::new())),
            broadcast_history: Arc::new(RwLock::new(Vec::new())),
            attention_weights: Arc::new(RwLock::new(attention_weights)),
            thinking_budget: 4096,
        }
    }
    
    /// 路由智慧
    pub async fn route_wisdom(&mut self, wisdom: Wisdom) {
        let layer = wisdom.layer();
        
        // 记录广播历史
        {
            let mut history = self.broadcast_history.write().await;
            history.push(wisdom.clone());
            
            // 限制历史大小
            if history.len() > 1000 {
                history.remove(0);
            }
        }
        
        // 获取订阅者
        let subscribers = self.subscribers.read().await;
        if let Some(subscriber_list) = subscribers.get(&layer) {
            tracing::debug!(
                "GWT 路由智慧到 {:?} 层，{} 个订阅者",
                layer,
                subscriber_list.len()
            );
        }
        
        // 获取注意力权重
        let weights = self.attention_weights.read().await;
        if let Some(weight) = weights.get(&layer) {
            tracing::debug!("GWT 注意力权重 {:?}: {}", layer, weight);
        }
    }
    
    /// 订阅层级
    pub async fn subscribe(&mut self, layer: Layer, subscriber_id: &str) {
        let mut subscribers = self.subscribers.write().await;
        subscribers
            .entry(layer)
            .or_insert_with(Vec::new)
            .push(subscriber_id.to_string());
        
        tracing::debug!("GWT 新订阅者 {} 订阅 {:?}", subscriber_id, layer);
    }
    
    /// 取消订阅
    pub async fn unsubscribe(&mut self, layer: Layer, subscriber_id: &str) {
        let mut subscribers = self.subscribers.write().await;
        if let Some(subscriber_list) = subscribers.get_mut(&layer) {
            subscriber_list.retain(|id| id != subscriber_id);
        }
        
        tracing::debug!("GWT 订阅者 {} 取消订阅 {:?}", subscriber_id, layer);
    }
    
    /// 获取订阅者数量
    pub async fn subscriber_count(&self, layer: Layer) -> usize {
        let subscribers = self.subscribers.read().await;
        subscribers.get(&layer).map(|l| l.len()).unwrap_or(0)
    }
    
    /// 获取总订阅者数量
    pub async fn total_subscriber_count(&self) -> usize {
        let subscribers = self.subscribers.read().await;
        subscribers.values().map(|l| l.len()).sum()
    }
    
    /// 设置注意力权重
    pub async fn set_attention_weight(&mut self, layer: Layer, weight: f64) {
        let mut weights = self.attention_weights.write().await;
        weights.insert(layer, weight);
        
        tracing::debug!("GWT 设置注意力权重 {:?}: {}", layer, weight);
    }
    
    /// 获取注意力权重
    pub async fn get_attention_weight(&self, layer: Layer) -> f64 {
        let weights = self.attention_weights.read().await;
        weights.get(&layer).copied().unwrap_or(1.0)
    }
    
    /// 获取广播历史
    pub async fn get_broadcast_history(&self, limit: usize) -> Vec<Wisdom> {
        let history = self.broadcast_history.read().await;
        history.iter().rev().take(limit).cloned().collect()
    }
    
    /// 清除广播历史
    pub async fn clear_broadcast_history(&mut self) {
        let mut history = self.broadcast_history.write().await;
        history.clear();
    }

    /// Check if `tokens_used` is within the thinking budget.
    pub fn within_budget(&self, tokens_used: u32) -> bool {
        tokens_used < self.thinking_budget
    }

    /// Set the thinking budget (tokens).
    pub fn set_thinking_budget(&mut self, budget: u32) {
        self.thinking_budget = budget;
    }

    /// Get the current thinking budget.
    pub fn thinking_budget(&self) -> u32 {
        self.thinking_budget
    }
    
    /// 获取共振强度（多个层级的注意力权重乘积）
    pub async fn resonance_strength(&self, layers: &[Layer]) -> f64 {
        let weights = self.attention_weights.read().await;
        layers
            .iter()
            .filter_map(|layer| weights.get(layer))
            .product()
    }
}

impl Default for GWTRouterImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_subscribe() {
        let mut router = GWTRouterImpl::new();
        
        router.subscribe(Layer::L1Action, "subscriber1").await;
        
        assert_eq!(router.subscriber_count(Layer::L1Action).await, 1);
    }
    
    #[tokio::test]
    async fn test_route_wisdom() {
        let mut router = GWTRouterImpl::new();
        
        let wisdom = Wisdom::Action {
            capability_id: "test".to_string(),
            insight: "Test insight".to_string(),
            strength: 0.8,
        };
        
        router.route_wisdom(wisdom).await;
        
        let history = router.get_broadcast_history(10).await;
        assert_eq!(history.len(), 1);
    }
    
    #[tokio::test]
    async fn test_attention_weight() {
        let mut router = GWTRouterImpl::new();
        
        router.set_attention_weight(Layer::L1Action, 2.0).await;
        
        let weight = router.get_attention_weight(Layer::L1Action).await;
        assert_eq!(weight, 2.0);
    }
    
    #[tokio::test]
    async fn test_resonance_strength() {
        let mut router = GWTRouterImpl::new();
        
        router.set_attention_weight(Layer::L1Action, 2.0).await;
        router.set_attention_weight(Layer::L2Perception, 3.0).await;
        
        let strength = router.resonance_strength(&[Layer::L1Action, Layer::L2Perception]).await;
        assert_eq!(strength, 6.0);
    }
}
