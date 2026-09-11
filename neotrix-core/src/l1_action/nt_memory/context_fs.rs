//! Context-as-Filesystem — 基于 OpenViking 模式
//!
//! 将上下文组织为虚拟文件系统，支持分层加载和 URI 寻址。

use std::collections::HashMap;

/// 上下文节点类型

#[derive(Clone, Debug)]
pub enum ContextNodeType {
    /// 资源节点（文档、代码）
    Resource,
    /// 记忆节点（认知、经验）
    Memory,
    /// 技能节点（可调用）
    Skill,
    /// 目录节点（包含子节点）
    Directory,
}

/// 上下文节点

#[derive(Clone, Debug)]
pub struct ContextNode {
    pub uri: String, // e.g. "ctx://memory/experience/cycle_001"
    pub name: String,
    pub node_type: ContextNodeType,
    pub content: Option<String>,
    pub children: Vec<String>, // 子节点 URI
    pub metadata: HashMap<String, String>,
    pub token_count: usize,
}

/// 上下文文件系统

#[derive(Debug)]
pub struct ContextFileSystem {
    nodes: HashMap<String, ContextNode>,
    _root: String,
}


impl ContextFileSystem {
    pub fn new() -> Self {
        let mut fs = Self {
            nodes: HashMap::new(),
            _root: "ctx://".to_string(),
        };
        // 创建根目录
        fs.nodes.insert(
            "ctx://".to_string(),
            ContextNode {
                uri: "ctx://".to_string(),
                name: "/".to_string(),
                node_type: ContextNodeType::Directory,
                content: None,
                children: Vec::new(),
                metadata: HashMap::new(),
                token_count: 0,
            },
        );
        fs
    }

    /// 创建节点
    pub fn create_node(
        &mut self,
        uri: &str,
        name: &str,
        node_type: ContextNodeType,
        content: Option<String>,
    ) {
        let token_count = content.as_ref().map(|c| c.len() / 4).unwrap_or(0);
        self.nodes.insert(
            uri.to_string(),
            ContextNode {
                uri: uri.to_string(),
                name: name.to_string(),
                node_type,
                content,
                children: Vec::new(),
                metadata: HashMap::new(),
                token_count,
            },
        );
    }

    /// 列出目录内容
    pub fn list_dir(&self, path: &str) -> Vec<(&str, &ContextNodeType)> {
        self.nodes
            .get(path)
            .map(|node| {
                node.children
                    .iter()
                    .filter_map(|child_uri| {
                        self.nodes
                            .get(child_uri)
                            .map(|n| (n.name.as_str(), &n.node_type))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 读取节点内容
    pub fn read_node(&self, uri: &str) -> Option<(&str, usize)> {
        self.nodes.get(uri).map(|n| (n.uri.as_str(), n.token_count))
    }

    /// 搜索节点
    pub fn search(&self, query: &str) -> Vec<&ContextNode> {
        self.nodes
            .values()
            .filter(|n| n.name.contains(query) || n.uri.contains(query))
            .collect()
    }

    /// 获取总 token 数
    pub fn total_tokens(&self) -> usize {
        self.nodes.values().map(|n| n.token_count).sum()
    }

    /// 统计
    pub fn stats(&self) -> (usize, usize, usize) {
        let resources = self
            .nodes
            .values()
            .filter(|n| matches!(n.node_type, ContextNodeType::Resource))
            .count();
        let memories = self
            .nodes
            .values()
            .filter(|n| matches!(n.node_type, ContextNodeType::Memory))
            .count();
        let skills = self
            .nodes
            .values()
            .filter(|n| matches!(n.node_type, ContextNodeType::Skill))
            .count();
        (resources, memories, skills)
    }
}
