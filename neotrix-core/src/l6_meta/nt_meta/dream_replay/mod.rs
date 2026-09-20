//! # Dream-RSI 回放引擎
//!
//! 借鉴 Dream-RSI (arXiv:2609.14858) 的核心模式：
//! - 用累积发现历史作为「梦境回放模拟器」
//! - 历史发现树 → 探索策略的回放环境
//! - Off-policy 评估: 零成本即时反馈
//!
//! 用于 NT-MIND 自进化循环的计算成本优化。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 发现树节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryNode {
    pub id: String,
    pub parent_id: Option<String>,
    pub action: String,
    pub reward: f64,
    pub metadata: HashMap<String, String>,
}

/// 发现树
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryTree {
    pub nodes: Vec<DiscoveryNode>,
    pub root_id: String,
}

impl DiscoveryTree {
    pub fn new() -> Self {
        let root = DiscoveryNode {
            id: "root".into(),
            parent_id: None,
            action: "initialize".into(),
            reward: 0.0,
            metadata: HashMap::new(),
        };
        Self {
            nodes: vec![root],
            root_id: "root".into(),
        }
    }
    
    pub fn add_node(&mut self, parent_id: &str, action: String, reward: f64) -> String {
        let id = format!("node_{}", self.nodes.len());
        let node = DiscoveryNode {
            id: id.clone(),
            parent_id: Some(parent_id.into()),
            action,
            reward,
            metadata: HashMap::new(),
        };
        self.nodes.push(node);
        id
    }
    
    /// 获取从根到指定节点的路径
    pub fn get_path(&self, node_id: &str) -> Vec<&DiscoveryNode> {
        let mut path = Vec::new();
        let mut current_id = Some(node_id);
        while let Some(id) = current_id {
            if let Some(node) = self.nodes.iter().find(|n| n.id == id) {
                path.push(node);
                current_id = node.parent_id.as_deref();
            } else {
                break;
            }
        }
        path.reverse();
        path
    }
    
    /// 计算路径累积奖励
    pub fn path_reward(&self, node_id: &str) -> f64 {
        self.get_path(node_id).iter().map(|n| n.reward).sum()
    }
}

/// 回放模拟器
pub struct DreamReplaySimulator {
    /// 历史发现树
    trees: Vec<DiscoveryTree>,
    /// 探索策略评估结果
    evaluations: Vec<PolicyEvaluation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyEvaluation {
    pub policy_name: String,
    pub avg_reward: f64,
    pub win_rate: f64,
    pub sample_count: usize,
}

impl DreamReplaySimulator {
    pub fn new() -> Self {
        Self {
            trees: Vec::new(),
            evaluations: Vec::new(),
        }
    }
    
    /// 加载历史发现树
    pub fn load_tree(&mut self, tree: DiscoveryTree) {
        self.trees.push(tree);
    }
    
    /// 在历史树上评估探索策略
    pub fn evaluate_policy(&self, policy_name: &str, strategy: &dyn Fn(&DiscoveryNode) -> bool) -> PolicyEvaluation {
        let mut total_reward = 0.0;
        let mut wins = 0;
        let mut count = 0;
        
        for tree in &self.trees {
            for node in &tree.nodes {
                if strategy(node) {
                    total_reward += node.reward;
                    if node.reward > 0.0 {
                        wins += 1;
                    }
                    count += 1;
                }
            }
        }
        
        PolicyEvaluation {
            policy_name: policy_name.into(),
            avg_reward: if count > 0 { total_reward / count as f64 } else { 0.0 },
            win_rate: if count > 0 { wins as f64 / count as f64 } else { 0.0 },
            sample_count: count,
        }
    }
    
    /// 梦境回放: 在历史树上模拟新策略的执行
    pub fn dream_replay(&self, new_strategy: &dyn Fn(&DiscoveryNode) -> bool) -> Vec<PolicyEvaluation> {
        self.trees.iter().map(|tree| {
            let mut total_reward = 0.0;
            let mut wins = 0;
            let mut count = 0;
            
            for node in &tree.nodes {
                if new_strategy(node) {
                    total_reward += node.reward;
                    if node.reward > 0.0 {
                        wins += 1;
                    }
                    count += 1;
                }
            }
            
            PolicyEvaluation {
                policy_name: "dream_replay".into(),
                avg_reward: if count > 0 { total_reward / count as f64 } else { 0.0 },
                win_rate: if count > 0 { wins as f64 / count as f64 } else { 0.0 },
                sample_count: count,
            }
        }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_discovery_tree() {
        let mut tree = DiscoveryTree::new();
        let n1 = tree.add_node("root", "explore_a".into(), 1.0);
        let _n2 = tree.add_node(&n1, "explore_b".into(), 0.5);
        assert_eq!(tree.nodes.len(), 3);
    }
    
    #[test]
    fn test_dream_replay() {
        let mut sim = DreamReplaySimulator::new();
        let mut tree = DiscoveryTree::new();
        tree.add_node("root", "action_a".into(), 1.0);
        tree.add_node("root", "action_b".into(), -0.5);
        sim.load_tree(tree);
        
        let eval = sim.evaluate_policy("greedy", &|node| node.reward > 0.0);
        assert_eq!(eval.sample_count, 1);
        assert_eq!(eval.avg_reward, 1.0);
    }
}