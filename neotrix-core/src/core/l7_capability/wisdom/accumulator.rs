//! 智慧累积器
//! 
//! 累积从能力执行中产生的智慧

use crate::core::l7_capability::types::Wisdom;
use tokio::sync::RwLock;
use std::sync::Arc;

/// 智慧累积器
pub struct WisdomAccumulator {
    /// 智慧池
    pool: Arc<RwLock<super::pool::WisdomPool>>,
}

impl WisdomAccumulator {
    /// 创建新的智慧累积器
    pub fn new() -> Self {
        Self {
            pool: Arc::new(RwLock::new(super::pool::WisdomPool::new())),
        }
    }
    
    /// 累积智慧
    pub async fn accumulate(&mut self, wisdom: Wisdom) -> Result<(), String> {
        let mut pool = self.pool.write().await;
        pool.add(wisdom);
        Ok(())
    }
    
    /// 获取所有智慧
    pub fn get_wisdom(&self) -> Vec<Wisdom> {
        let pool = self.pool.try_read().unwrap();
        pool.get_all()
    }
    
    /// 清空智慧池
    pub fn clear(&mut self) {
        if let Ok(mut pool) = self.pool.try_write() {
            pool.clear();
        }
    }
    
    /// 获取智慧数量
    pub fn count(&self) -> usize {
        let pool = self.pool.try_read().unwrap();
        pool.count()
    }
    
    /// 获取智慧池容量
    pub fn capacity(&self) -> usize {
        let pool = self.pool.try_read().unwrap();
        pool.capacity()
    }
}

impl Default for WisdomAccumulator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_accumulate_wisdom() {
        let mut accumulator = WisdomAccumulator::new();
        
        let wisdom = Wisdom::Action {
            capability_id: "test".to_string(),
            insight: "Test insight".to_string(),
            strength: 0.8,
        };
        
        assert!(accumulator.accumulate(wisdom).await.is_ok());
        assert_eq!(accumulator.count(), 1);
    }
    
    #[tokio::test]
    async fn test_clear_wisdom() {
        let mut accumulator = WisdomAccumulator::new();
        
        let wisdom = Wisdom::Action {
            capability_id: "test".to_string(),
            insight: "Test insight".to_string(),
            strength: 0.8,
        };
        
        let _ = accumulator.accumulate(wisdom).await;
        assert_eq!(accumulator.count(), 1);
        
        accumulator.clear();
        assert_eq!(accumulator.count(), 0);
    }
}
