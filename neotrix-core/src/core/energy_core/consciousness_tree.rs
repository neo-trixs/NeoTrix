//! 意识树实现
//! 
//! 聚合6层架构涌现的智慧，形成意识树

use crate::core::l7_capability::types::{Wisdom, Layer};
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;

/// 意识树节点
#[derive(Debug, Clone)]
pub struct ConsciousnessNode {
    /// 节点 ID
    pub id: String,
    /// 节点层级
    pub layer: Layer,
    /// 节点智慧
    pub wisdoms: Vec<Wisdom>,
    /// 子节点
    pub children: Vec<String>,
    /// 节点强度
    pub strength: f64,
}

/// 意识树实现
pub struct ConsciousnessTreeImpl {
    /// 节点存储
    nodes: Arc<RwLock<HashMap<String, ConsciousnessNode>>>,
    /// 根节点 ID
    root_id: String,
}

impl ConsciousnessTreeImpl {
    /// 创建新的意识树
    pub fn new() -> Self {
        let root_id = "root".to_string();
        let mut nodes = HashMap::new();
        
        // 创建根节点
        nodes.insert(
            root_id.clone(),
            ConsciousnessNode {
                id: root_id.clone(),
                layer: Layer::L5Cognition,
                wisdoms: vec![],
                children: vec![],
                strength: 0.0,
            },
        );
        
        Self {
            nodes: Arc::new(RwLock::new(nodes)),
            root_id,
        }
    }
    
    /// 接收智慧
    pub async fn receive_wisdom(&mut self, wisdom: Wisdom) {
        let layer = wisdom.layer();
        let node_id = format!("{:?}", layer);
        
        let mut nodes = self.nodes.write().await;
        
        // 获取或创建层级节点
        let node = nodes
            .entry(node_id.clone())
            .or_insert_with(|| ConsciousnessNode {
                id: node_id.clone(),
                layer,
                wisdoms: vec![],
                children: vec![],
                strength: 0.0,
            });
        
        // 添加智慧
        node.wisdoms.push(wisdom);
        
        // 更新强度
        node.strength = node.wisdoms.iter().map(|w| w.strength()).sum::<f64>() / node.wisdoms.len() as f64;
        
        // 确保根节点包含此节点
        let root = nodes.get_mut(&self.root_id).unwrap();
        if !root.children.contains(&node_id) {
            root.children.push(node_id);
        }
        
        tracing::debug!("意识树接收智慧，层级: {:?}, 当前节点数: {}", layer, nodes.len());
    }
    
    /// 获取节点
    pub async fn get_node(&self, id: &str) -> Option<ConsciousnessNode> {
        let nodes = self.nodes.read().await;
        nodes.get(id).cloned()
    }
    
    /// 获取所有节点
    pub async fn get_all_nodes(&self) -> Vec<ConsciousnessNode> {
        let nodes = self.nodes.read().await;
        nodes.values().cloned().collect()
    }
    
    /// 获取根节点
    pub async fn get_root(&self) -> Option<ConsciousnessNode> {
        self.get_node(&self.root_id).await
    }
    
    /// 获取节点数量
    pub async fn node_count(&self) -> usize {
        let nodes = self.nodes.read().await;
        nodes.len()
    }
    
    /// 获取树的总强度
    pub async fn total_strength(&self) -> f64 {
        let nodes = self.nodes.read().await;
        nodes.values().map(|n| n.strength).sum()
    }
    
    /// 获取层级分布
    pub async fn layer_distribution(&self) -> HashMap<Layer, usize> {
        let nodes = self.nodes.read().await;
        let mut distribution = HashMap::new();
        
        for node in nodes.values() {
            *distribution.entry(node.layer).or_insert(0) += 1;
        }
        
        distribution
    }
    
    /// 修剪弱节点
    pub async fn prune(&mut self, threshold: f64) -> usize {
        let mut nodes = self.nodes.write().await;
        let mut pruned = 0;
        
        // 收集要删除的节点
        let to_remove: Vec<String> = nodes
            .iter()
            .filter(|(id, node)| **id != self.root_id && node.strength < threshold)
            .map(|(id, _)| id.clone())
            .collect();
        
        // 删除节点
        for id in to_remove {
            nodes.remove(&id);
            pruned += 1;
        }
        
        // 更新根节点的子节点
        if let Some(root) = nodes.get_mut(&self.root_id) {
            root.children.retain(|id| nodes.contains_key(id));
        }
        
        pruned
    }
}

impl Default for ConsciousnessTreeImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_receive_wisdom() {
        let mut tree = ConsciousnessTreeImpl::new();
        
        let wisdom = Wisdom::Action {
            capability_id: "test".to_string(),
            insight: "Test insight".to_string(),
            strength: 0.8,
        };
        
        tree.receive_wisdom(wisdom).await;
        
        assert_eq!(tree.node_count().await, 2); // root + L1Action
        assert!(tree.get_node("L1Action").await.is_some());
    }
    
    #[tokio::test]
    async fn test_prune() {
        let mut tree = ConsciousnessTreeImpl::new();
        
        // 添加弱智慧
        let weak_wisdom = Wisdom::Action {
            capability_id: "weak".to_string(),
            insight: "Weak insight".to_string(),
            strength: 0.1,
        };
        
        // 添加强智慧
        let strong_wisdom = Wisdom::Action {
            capability_id: "strong".to_string(),
            insight: "Strong insight".to_string(),
            strength: 0.9,
        };
        
        tree.receive_wisdom(weak_wisdom).await;
        tree.receive_wisdom(strong_wisdom).await;
        
        // 修剪阈值 0.5
        let pruned = tree.prune(0.5).await;
        
        // 应该保留根节点和 L1Action 节点（因为有强智慧）
        assert_eq!(pruned, 0);
    }
}
