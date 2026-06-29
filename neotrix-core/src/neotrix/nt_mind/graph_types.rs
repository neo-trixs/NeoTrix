use std::collections::HashMap;
use std::path::PathBuf;

/// 图节点类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NodeKind {
    File,
    Module,
    Function,
    Struct,
    Trait,
    Impl,
    Const,
}

impl NodeKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Module => "module",
            Self::Function => "function",
            Self::Struct => "struct",
            Self::Trait => "trait",
            Self::Impl => "impl",
            Self::Const => "const",
        }
    }
}

/// 边类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EdgeKind {
    Imports,
    Calls,
    Uses,
    Implements,
    Extends,
    Contains,
}

impl EdgeKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Imports => "imports",
            Self::Calls => "calls",
            Self::Uses => "uses",
            Self::Implements => "implements",
            Self::Extends => "extends",
            Self::Contains => "contains",
        }
    }
}

/// 图节点
#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: String,
    pub name: String,
    pub kind: NodeKind,
    pub file_path: Option<PathBuf>,
    pub start_line: usize,
    pub end_line: usize,
}

/// 图边
#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub kind: EdgeKind,
    pub confidence: f64,
}

/// 影响分析结果
#[derive(Debug, Clone)]
pub struct ImpactResult {
    pub target: String,
    pub upstream: Vec<ImpactHop>,
    pub downstream: Vec<ImpactHop>,
}

#[derive(Debug, Clone)]
pub struct ImpactHop {
    pub node_id: String,
    pub node_name: String,
    pub node_kind: NodeKind,
    pub depth: usize,
    pub confidence: f64,
    pub path: Vec<String>,
}

/// 函数调用信息
#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub file_path: PathBuf,
    pub start_line: usize,
    pub calls: Vec<String>,
    pub called_by: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CodeGraphStats {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub type_counts: HashMap<String, usize>,
    pub kind_counts: HashMap<String, usize>,
    pub community_count: usize,
}

#[derive(Debug, Clone)]
pub struct EnrichedSearchResult {
    pub node: GraphNode,
    pub outgoing_edges: Vec<String>,
    pub incoming_edges: Vec<String>,
    pub community_id: Option<usize>,
}
