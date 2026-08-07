//! 能力注册表

use crate::node::{CapabilityNode, Domain, NodeLayer};
use indexmap::IndexMap;
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("节点已存在: {0}")]
    AlreadyExists(String),
    #[error("节点不存在: {0}")]
    NotFound(String),
    #[error("依赖循环: {0} -> {1}")]
    CircularDependency(String, String),
    #[error("非法层级跨度: {0} (L{1}) 依赖 {2} (L{3})")]
    InvalidLayerSpan(String, u8, String, u8),
    #[error("序列化错误: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
}

/// 注册表核心
#[derive(Debug, Clone)]
pub struct CapabilityRegistry {
    pub nodes: IndexMap<String, CapabilityNode>,  // id -> node
    domain_index: HashMap<Domain, Vec<String>>,
    layer_index: HashMap<NodeLayer, Vec<String>>,
    constellation_index: HashMap<u8, Vec<String>>,  // C0-C6 -> node ids
    provides_index: HashMap<String, Vec<String>>,   // capability tag -> node ids
    dag: petgraph::Graph<String, ()>,               // 依赖图
    node_indices: HashMap<String, petgraph::graph::NodeIndex>,
    /// 经验驱动迭代目标 (distill 蒸馏写入, scan --apply 消费)
    /// 值结构: {domain, capability, action, rationale, signal, promoted_at}
    pub experience_targets: Vec<serde_json::Value>,
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CapabilityRegistry {
    pub fn new() -> Self {
        Self {
            nodes: IndexMap::new(),
            domain_index: HashMap::new(),
            layer_index: HashMap::new(),
            constellation_index: HashMap::new(),
            provides_index: HashMap::new(),
            dag: petgraph::Graph::new(),
            node_indices: HashMap::new(),
            experience_targets: Vec::new(),
        }
    }

    /// 注册新节点
    pub fn register(&mut self, node: CapabilityNode) -> Result<(), RegistryError> {
        if self.nodes.contains_key(&node.id) {
            return Err(RegistryError::AlreadyExists(node.id));
        }

        // 验证依赖存在
        for req in &node.requires {
            if !self.provides_index.contains_key(req) {
                // 允许前向声明，但发出警告
                eprintln!("[capability_tree] WARNING: dependency '{}' not yet registered for node '{}'", req, node.id);
            }
        }

        // 添加到 DAG
        let idx = self.dag.add_node(node.id.clone());
        self.node_indices.insert(node.id.clone(), idx);

        // 建立依赖边
        for req in &node.requires {
            if let Some(&req_idx) = self.node_indices.get(req) {
                self.dag.add_edge(idx, req_idx, ());
            }
        }

        // 检查循环依赖
        if petgraph::algo::is_cyclic_directed(&self.dag) {
            // 回滚
            self.dag.remove_node(idx);
            self.node_indices.remove(&node.id);
            return Err(RegistryError::CircularDependency(node.id, "circular".into()));
        }

        // 更新索引
        let id = node.id.clone();
        let domain = node.domain;
        let layer = node.layer;
        let constellation = node.constellation as u8;
        let provides = node.provides.clone();

        self.domain_index.entry(domain).or_default().push(id.clone());
        self.layer_index.entry(layer).or_default().push(id.clone());
        self.constellation_index.entry(constellation).or_default().push(id.clone());
        for tag in provides {
            self.provides_index.entry(tag).or_default().push(id.clone());
        }

        self.nodes.insert(id, node);
        Ok(())
    }

    /// 获取节点
    pub fn get(&self, id: &str) -> Option<&CapabilityNode> {
        self.nodes.get(id)
    }

    /// 可变获取
    pub fn get_mut(&mut self, id: &str) -> Option<&mut CapabilityNode> {
        self.nodes.get_mut(id)
    }

    /// 删除节点 (仅允许无依赖者时)
    pub fn remove(&mut self, id: &str) -> Result<CapabilityNode, RegistryError> {
        let node = self.nodes.shift_remove(id)
            .ok_or(RegistryError::NotFound(id.to_string()))?;

        if !node.dependents.is_empty() {
            return Err(RegistryError::CircularDependency(
                id.to_string(),
                format!("has dependents: {:?}", node.dependents)
            ));
        }

        // 从 DAG 移除
        if let Some(&idx) = self.node_indices.get(id) {
            self.dag.remove_node(idx);
            self.node_indices.remove(id);
        }

        // 更新索引
        self.domain_index.get_mut(&node.domain).map(|v| v.retain(|x| x != id));
        self.layer_index.get_mut(&node.layer).map(|v| v.retain(|x| x != id));
        self.constellation_index.get_mut(&(node.constellation as u8)).map(|v| v.retain(|x| x != id));
        for tag in &node.provides {
            self.provides_index.get_mut(tag).map(|v| v.retain(|x| x != id));
        }

        // 从依赖者的 dependents 移除
        for req in &node.requires {
            if let Some(dep_node) = self.nodes.get_mut(req) {
                dep_node.remove_dependent(id);
            }
        }

        Ok(node)
    }

    /// 添加依赖关系
    pub fn add_dependency(&mut self, from: &str, to: &str) -> Result<(), RegistryError> {
        let from_idx = *self.node_indices.get(from).ok_or_else(|| RegistryError::NotFound(from.into()))?;
        let to_idx = *self.node_indices.get(to).ok_or_else(|| RegistryError::NotFound(to.into()))?;

        // 检查层级跨度 (L0 只能被 L1+ 依赖，不能依赖 L3+ 等)
        let from_node = &self.nodes[from];
        let to_node = &self.nodes[to];
        let from_layer = from_node.layer as u8;
        let to_layer = to_node.layer as u8;
        
        // 允许向下依赖 (高层依赖低层)，禁止向上依赖超过 1 层
        if to_layer > from_layer + 1 {
            return Err(RegistryError::InvalidLayerSpan(
                from.into(), from_layer, to.into(), to_layer
            ));
        }

        self.dag.add_edge(from_idx, to_idx, ());
        
        if petgraph::algo::is_cyclic_directed(&self.dag) {
            self.dag.remove_edge(self.dag.find_edge(from_idx, to_idx).unwrap());
            return Err(RegistryError::CircularDependency(from.into(), to.into()));
        }

        // 更新双向索引
        if let Some(node) = self.nodes.get_mut(to) {
            node.add_dependent(from.into());
        }

        Ok(())
    }

    /// 查询: 按领域
    pub fn by_domain(&self, domain: Domain) -> Vec<&CapabilityNode> {
        self.domain_index.get(&domain)
            .map(|ids| ids.iter().filter_map(|id| self.nodes.get(id)).collect())
            .unwrap_or_default()
    }

    /// 查询: 按层级
    pub fn by_layer(&self, layer: NodeLayer) -> Vec<&CapabilityNode> {
        self.layer_index.get(&layer)
            .map(|ids| ids.iter().filter_map(|id| self.nodes.get(id)).collect())
            .unwrap_or_default()
    }

    /// 查询: 按星座等级
    pub fn by_constellation(&self, level: u8) -> Vec<&CapabilityNode> {
        self.constellation_index.get(&level)
            .map(|ids| ids.iter().filter_map(|id| self.nodes.get(id)).collect())
            .unwrap_or_default()
    }

    /// 查询: 按提供能力标签
    pub fn by_provides(&self, tag: &str) -> Vec<&CapabilityNode> {
        self.provides_index.get(tag)
            .map(|ids| ids.iter().filter_map(|id| self.nodes.get(id)).collect())
            .unwrap_or_default()
    }

    /// 查询: 所有 L0 Primitive
    pub fn all_primitives(&self) -> Vec<&CapabilityNode> {
        self.by_layer(NodeLayer::L0Primitive)
    }

    /// 查询: 所有待晋升节点 (Cn 全绿 + 无阻塞)
    pub fn promotable_nodes(&self) -> Vec<&CapabilityNode> {
        self.nodes.values()
            .filter(|n| !n.deprecated && n.constellation.next().is_some())
            .collect()
    }

    /// 查询: 孤儿节点 (无 dependents 且非入口点)
    pub fn orphan_nodes(&self) -> Vec<&CapabilityNode> {
        self.nodes.values()
            .filter(|n| n.dependents.is_empty() && !n.is_constellation())
            .collect()
    }

    /// 查询: 过期节点 (C0/C1 超 3 周期无更新)
    pub fn stale_nodes(&self, cycles_threshold: u32) -> Vec<&CapabilityNode> {
        // 简化: 基于 evolution_log 的最后 cycle 判断
        self.nodes.values()
            .filter(|n| {
                n.constellation as u8 <= 1 && 
                n.evolution_log.len() < cycles_threshold as usize
            })
            .collect()
    }

    /// 获取依赖拓扑序 (用于构建顺序)
    pub fn topological_order(&self) -> Result<Vec<String>, RegistryError> {
        use petgraph::algo::toposort;
        let order = toposort(&self.dag, None)
            .map_err(|_| RegistryError::CircularDependency("graph".into(), "has cycles".into()))?;
        Ok(order.into_iter().map(|idx| self.dag[idx].clone()).collect())
    }

    /// 获取反向依赖 (谁依赖我)
    pub fn reverse_dependencies(&self, id: &str) -> Vec<String> {
        self.nodes.get(id).map(|n| n.dependents.clone()).unwrap_or_default()
    }

    /// 获取直接依赖 (我依赖谁)
    pub fn direct_dependencies(&self, id: &str) -> Vec<String> {
        self.nodes.get(id).map(|n| n.requires.clone()).unwrap_or_default()
    }

    /// 统计信息
    pub fn stats(&self) -> RegistryStats {
        let mut by_domain = HashMap::new();
        let mut by_layer = HashMap::new();
        let mut by_constellation = HashMap::new();

        for node in self.nodes.values() {
            *by_domain.entry(node.domain.as_str().to_string()).or_insert(0) += 1;
            *by_layer.entry(node.layer.as_str().to_string()).or_insert(0) += 1;
            *by_constellation.entry(node.constellation.as_str().to_string()).or_insert(0) += 1;
        }

        RegistryStats {
            total_nodes: self.nodes.len(),
            by_domain,
            by_layer,
            by_constellation,
            deprecated_count: self.nodes.values().filter(|n| n.deprecated).count(),
            primitive_count: self.all_primitives().len(),
        }
    }

    /// 检查是否存在循环依赖
    pub fn has_cycles(&self) -> bool {
        petgraph::algo::is_cyclic_directed(&self.dag)
    }

    /// 导出为可序列化结构
    pub fn export(&self) -> RegistryExport {
        let mut edges = Vec::new();
        for edge in self.dag.edge_references() {
            let a_idx = edge.source();
            let b_idx = edge.target();
            let a = &self.dag[a_idx];
            let b = &self.dag[b_idx];
            edges.push((a.clone(), b.clone()));
        }
        RegistryExport {
            nodes: self.nodes.values().cloned().collect(),
            edges,
            experience_targets: self.experience_targets.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryExport {
    pub nodes: Vec<CapabilityNode>,
    pub edges: Vec<(String, String)>,
    /// 经验驱动迭代目标 (distill 蒸馏写入, scan --apply 消费)。
    /// 默认空: 旧文件无此字段时兼容反序列化。
    #[serde(default)]
    pub experience_targets: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct RegistryStats {
    pub total_nodes: usize,
    pub by_domain: HashMap<String, usize>,
    pub by_layer: HashMap<String, usize>,
    pub by_constellation: HashMap<String, usize>,
    pub deprecated_count: usize,
    pub primitive_count: usize,
}