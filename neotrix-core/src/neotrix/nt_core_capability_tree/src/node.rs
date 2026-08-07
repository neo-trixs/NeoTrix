//! 能力树节点定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// NT-* 领域轴 (X 轴)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Domain {
    Core,
    Mind,
    Memory,
    World,
    Act,
    Shield,
    Io,
    Meta,
    Nexus,
    Governance,
    Repair,
}

impl Domain {
    pub fn as_str(&self) -> &'static str {
        match self {
            Domain::Core => "NT-CORE",
            Domain::Mind => "NT-MIND",
            Domain::Memory => "NT-MEMORY",
            Domain::World => "NT-WORLD",
            Domain::Act => "NT-ACT",
            Domain::Shield => "NT-SHIELD",
            Domain::Io => "NT-IO",
            Domain::Meta => "NT-META",
            Domain::Nexus => "NT-NEXUS",
            Domain::Governance => "NT-GOVERNANCE",
            Domain::Repair => "NT-REPAIR",
        }
    }

    /// 从 "NT-*" 字符串解析域 (大小写不敏感); 无法识别时返回 None。
    pub fn from_str(name: &str) -> Option<Self> {
        match name.to_uppercase().as_str() {
            "NT-CORE" => Some(Domain::Core),
            "NT-MIND" => Some(Domain::Mind),
            "NT-MEMORY" => Some(Domain::Memory),
            "NT-WORLD" => Some(Domain::World),
            "NT-ACT" => Some(Domain::Act),
            "NT-SHIELD" => Some(Domain::Shield),
            "NT-IO" => Some(Domain::Io),
            "NT-META" => Some(Domain::Meta),
            "NT-NEXUS" => Some(Domain::Nexus),
            "NT-GOVERNANCE" => Some(Domain::Governance),
            "NT-REPAIR" => Some(Domain::Repair),
            _ => None,
        }
    }
}

impl std::fmt::Display for Domain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// 抽象层轴 (Z 轴)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeLayer {
    L0Primitive,
    L1Composite,
    L2Orchestrator,
    L3DomainService,
    L4Application,
}

impl NodeLayer {
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeLayer::L0Primitive => "L0",
            NodeLayer::L1Composite => "L1",
            NodeLayer::L2Orchestrator => "L2",
            NodeLayer::L3DomainService => "L3",
            NodeLayer::L4Application => "L4",
        }
    }
}

/// 星座成熟度 (Y 轴) - C0 到 C6
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConstellationLevel {
    C0Compile,
    C1UnitTest,
    C2IntegrationTest,
    C3Benchmark,
    C4MainPipeline,
    C5SelfHealing,
    C6EvolutionLoop,
}

impl ConstellationLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConstellationLevel::C0Compile => "C0",
            ConstellationLevel::C1UnitTest => "C1",
            ConstellationLevel::C2IntegrationTest => "C2",
            ConstellationLevel::C3Benchmark => "C3",
            ConstellationLevel::C4MainPipeline => "C4",
            ConstellationLevel::C5SelfHealing => "C5",
            ConstellationLevel::C6EvolutionLoop => "C6",
        }
    }

    pub fn next(&self) -> Option<Self> {
        match self {
            ConstellationLevel::C0Compile => Some(ConstellationLevel::C1UnitTest),
            ConstellationLevel::C1UnitTest => Some(ConstellationLevel::C2IntegrationTest),
            ConstellationLevel::C2IntegrationTest => Some(ConstellationLevel::C3Benchmark),
            ConstellationLevel::C3Benchmark => Some(ConstellationLevel::C4MainPipeline),
            ConstellationLevel::C4MainPipeline => Some(ConstellationLevel::C5SelfHealing),
            ConstellationLevel::C5SelfHealing => Some(ConstellationLevel::C6EvolutionLoop),
            ConstellationLevel::C6EvolutionLoop => None,
        }
    }
}

/// Rune 插槽类型 (5 槽)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuneSocket {
    Crimson,    // 数据摄取
    Indigo,     // 变换
    Obsidian,   // 缓存
    Golden,     // 错误恢复
    Alabaster,  // 监控
}

impl RuneSocket {
    pub fn as_str(&self) -> &'static str {
        match self {
            RuneSocket::Crimson => "Crimson",
            RuneSocket::Indigo => "Indigo",
            RuneSocket::Obsidian => "Obsidian",
            RuneSocket::Golden => "Golden",
            RuneSocket::Alabaster => "Alabaster",
        }
    }
}

/// 演化操作类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvolutionOp {
    Budding,           // 萌芽: 新建 Primitive
    Grafting,          // 嫁接: 折叠分散实现到 Primitive/Composite
    Pruning,           // 修剪: 标记废弃/删除无用节点
    CrossPollination,  // 异花授粉: 跨域抽象共享 Primitive
    Maturation,        // 成熟晋升: Cn -> Cn+1
    Strengthen,        // 强化: 吸收经验强化既有节点 (R-P42 吸收强化现有节点, 不新建)
}

/// 演化日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionLogEntry {
    pub cycle: String,
    pub op: EvolutionOp,
    pub from_nodes: Vec<String>,      // 来源节点 (Grafting/Pruning 时)
    pub to_node: Option<String>,      // 目标节点 (Budding/Maturation 时)
    pub note: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 能力节点核心定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityNode {
    pub id: String,                           // 全局唯一 ID: "domain::module::function"
    pub domain: Domain,
    pub layer: NodeLayer,
    pub constellation: ConstellationLevel,
    pub provides: Vec<String>,                // 提供的能力标签
    pub requires: Vec<String>,                // 依赖的能力标签
    pub rune_sockets: Vec<RuneSocket>,        // 占用的 Rune 槽
    pub dependents: Vec<String>,              // 反向依赖 (谁在用我)
    pub evolution_log: Vec<EvolutionLogEntry>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub deprecated: bool,
    pub deprecated_reason: Option<String>,
}

impl CapabilityNode {
    /// 创建 L0 Primitive (Root)
    pub fn new_primitive(
        id: String,
        domain: Domain,
        provides: Vec<String>,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            id,
            domain,
            layer: NodeLayer::L0Primitive,
            constellation: ConstellationLevel::C0Compile,
            provides,
            requires: vec![],
            rune_sockets: vec![],
            dependents: vec![],
            evolution_log: vec![],
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
            deprecated: false,
            deprecated_reason: None,
        }
    }

    /// 创建 Composite Node (L1-L2)
    pub fn new_composite(
        id: String,
        domain: Domain,
        layer: NodeLayer,
        provides: Vec<String>,
        requires: Vec<String>,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            id,
            domain,
            layer,
            constellation: ConstellationLevel::C0Compile,
            provides,
            requires,
            rune_sockets: vec![],
            dependents: vec![],
            evolution_log: vec![],
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
            deprecated: false,
            deprecated_reason: None,
        }
    }

    /// 创建 Constellation (L3-L4)
    pub fn new_constellation(
        id: String,
        domain: Domain,
        layer: NodeLayer,
        provides: Vec<String>,
        requires: Vec<String>,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            id,
            domain,
            layer,
            constellation: ConstellationLevel::C0Compile,
            provides,
            requires,
            rune_sockets: vec![],
            dependents: vec![],
            evolution_log: vec![],
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
            deprecated: false,
            deprecated_reason: None,
        }
    }

    /// 记录演化操作
    pub fn record_evolution(&mut self, entry: EvolutionLogEntry) {
        self.evolution_log.push(entry);
        self.updated_at = chrono::Utc::now();
    }

    /// 添加依赖者
    pub fn add_dependent(&mut self, dependent_id: String) {
        if !self.dependents.contains(&dependent_id) {
            self.dependents.push(dependent_id);
            self.updated_at = chrono::Utc::now();
        }
    }

    /// 移除依赖者
    pub fn remove_dependent(&mut self, dependent_id: &str) {
        self.dependents.retain(|d| d != dependent_id);
        self.updated_at = chrono::Utc::now();
    }

    /// 晋升星座等级
    pub fn promote_constellation(&mut self) -> bool {
        if let Some(next) = self.constellation.next() {
            self.constellation = next;
            self.record_evolution(EvolutionLogEntry {
                cycle: "auto".into(),
                op: EvolutionOp::Maturation,
                from_nodes: vec![],
                to_node: Some(self.id.clone()),
                note: format!("Promoted to {}", next.as_str()),
                timestamp: chrono::Utc::now(),
            });
            true
        } else {
            false
        }
    }

    /// 标记废弃
    pub fn deprecate(&mut self, reason: String) {
        self.deprecated = true;
        self.deprecated_reason = Some(reason.clone());
        self.record_evolution(EvolutionLogEntry {
            cycle: "auto".into(),
            op: EvolutionOp::Pruning,
            from_nodes: vec![],
            to_node: Some(self.id.clone()),
            note: format!("Deprecated: {}", reason),
            timestamp: chrono::Utc::now(),
        });
    }

    /// 检查是否为 L0 Primitive
    pub fn is_primitive(&self) -> bool {
        self.layer == NodeLayer::L0Primitive
    }

    /// 检查是否为 Composite
    pub fn is_composite(&self) -> bool {
        matches!(self.layer, NodeLayer::L1Composite | NodeLayer::L2Orchestrator)
    }

    /// 检查是否为 Constellation
    pub fn is_constellation(&self) -> bool {
        matches!(self.layer, NodeLayer::L3DomainService | NodeLayer::L4Application)
    }
}