//! 智慧池
//! 
//! 存储和管理累积的智慧

use crate::core::l7_capability::types::Wisdom;
use std::collections::VecDeque;

/// 智慧池
pub struct WisdomPool {
    /// 智慧存储
    wisdoms: VecDeque<Wisdom>,
    /// 最大容量
    max_capacity: usize,
}

impl WisdomPool {
    /// 创建新的智慧池
    pub fn new() -> Self {
        Self {
            wisdoms: VecDeque::new(),
            max_capacity: 1000,
        }
    }
    
    /// 创建指定容量的智慧池
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            wisdoms: VecDeque::with_capacity(capacity),
            max_capacity: capacity,
        }
    }
    
    /// 添加智慧
    pub fn add(&mut self, wisdom: Wisdom) {
        if self.wisdoms.len() >= self.max_capacity {
            // 移除最旧的智慧
            self.wisdoms.pop_front();
        }
        self.wisdoms.push_back(wisdom);
    }
    
    /// 获取所有智慧
    pub fn get_all(&self) -> Vec<Wisdom> {
        self.wisdoms.iter().cloned().collect()
    }
    
    /// 清空智慧池
    pub fn clear(&mut self) {
        self.wisdoms.clear();
    }
    
    /// 获取智慧数量
    pub fn count(&self) -> usize {
        self.wisdoms.len()
    }
    
    /// 获取智慧池容量
    pub fn capacity(&self) -> usize {
        self.max_capacity
    }
    
    /// 检查智慧池是否已满
    pub fn is_full(&self) -> bool {
        self.wisdoms.len() >= self.max_capacity
    }
    
    /// 检查智慧池是否为空
    pub fn is_empty(&self) -> bool {
        self.wisdoms.is_empty()
    }
    
    /// 获取最新的智慧
    pub fn latest(&self) -> Option<&Wisdom> {
        self.wisdoms.back()
    }
    
    /// 获取最旧的智慧
    pub fn oldest(&self) -> Option<&Wisdom> {
        self.wisdoms.front()
    }
    
    /// 按层级过滤智慧
    pub fn filter_by_layer(&self, layer: crate::core::l7_capability::types::Layer) -> Vec<&Wisdom> {
        self.wisdoms
            .iter()
            .filter(|w| w.layer() == layer)
            .collect()
    }
    
    /// 按强度过滤智慧
    pub fn filter_by_strength(&self, min_strength: f64) -> Vec<&Wisdom> {
        self.wisdoms
            .iter()
            .filter(|w| w.strength() >= min_strength)
            .collect()
    }
    
    /// 获取智慧统计
    pub fn stats(&self) -> WisdomPoolStats {
        let mut layer_counts = std::collections::HashMap::new();
        let mut total_strength = 0.0;
        
        for wisdom in &self.wisdoms {
            let layer = wisdom.layer();
            *layer_counts.entry(layer).or_insert(0) += 1;
            total_strength += wisdom.strength();
        }
        
        WisdomPoolStats {
            total_count: self.wisdoms.len(),
            max_capacity: self.max_capacity,
            layer_distribution: layer_counts,
            average_strength: if self.wisdoms.is_empty() {
                0.0
            } else {
                total_strength / self.wisdoms.len() as f64
            },
        }
    }
}

impl Default for WisdomPool {
    fn default() -> Self {
        Self::new()
    }
}

/// 智慧池统计
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WisdomPoolStats {
    pub total_count: usize,
    pub max_capacity: usize,
    pub layer_distribution: std::collections::HashMap<crate::core::l7_capability::types::Layer, usize>,
    pub average_strength: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add_wisdom() {
        let mut pool = WisdomPool::new();
        
        let wisdom = Wisdom::Action {
            capability_id: "test".to_string(),
            insight: "Test insight".to_string(),
            strength: 0.8,
        };
        
        pool.add(wisdom);
        assert_eq!(pool.count(), 1);
    }
    
    #[test]
    fn test_pool_capacity() {
        let mut pool = WisdomPool::with_capacity(2);
        
        let wisdom1 = Wisdom::Action {
            capability_id: "test1".to_string(),
            insight: "Test insight 1".to_string(),
            strength: 0.8,
        };
        
        let wisdom2 = Wisdom::Action {
            capability_id: "test2".to_string(),
            insight: "Test insight 2".to_string(),
            strength: 0.9,
        };
        
        let wisdom3 = Wisdom::Action {
            capability_id: "test3".to_string(),
            insight: "Test insight 3".to_string(),
            strength: 0.7,
        };
        
        pool.add(wisdom1);
        pool.add(wisdom2);
        pool.add(wisdom3);
        
        assert_eq!(pool.count(), 2);
        assert!(pool.is_full());
    }
    
    #[test]
    fn test_filter_by_layer() {
        let mut pool = WisdomPool::new();
        
        let wisdom1 = Wisdom::Action {
            capability_id: "test1".to_string(),
            insight: "Test insight 1".to_string(),
            strength: 0.8,
        };
        
        let wisdom2 = Wisdom::Perception {
            capability_id: "test2".to_string(),
            pattern: "Test pattern".to_string(),
            confidence: 0.9,
        };
        
        pool.add(wisdom1);
        pool.add(wisdom2);
        
        let action_wisdoms = pool.filter_by_layer(crate::core::l7_capability::types::Layer::L1Action);
        assert_eq!(action_wisdoms.len(), 1);
        
        let perception_wisdoms = pool.filter_by_layer(crate::core::l7_capability::types::Layer::L2Perception);
        assert_eq!(perception_wisdoms.len(), 1);
    }
}
