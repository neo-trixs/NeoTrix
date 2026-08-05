//! 序列化/反序列化与 KB 集成

use crate::node::{EvolutionLogEntry, EvolutionOp};
use crate::registry::{CapabilityRegistry, RegistryExport};
use serde::{Deserialize, Serialize};
/// KB 存储格式 (kv_store capability_tree namespace)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KBCapabilityTree {
    pub version: u32,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub registry: RegistryExport,
}

impl KBCapabilityTree {
    pub fn from_registry(registry: &CapabilityRegistry) -> Self {
        Self {
            version: 1,
            updated_at: chrono::Utc::now(),
            registry: registry.export(),
        }
    }

    pub fn to_registry(&self) -> CapabilityRegistry {
        let mut reg = CapabilityRegistry::new();
        for node in &self.registry.nodes {
            reg.register(node.clone()).unwrap_or_else(|e| {
                eprintln!("[KBCapabilityTree] Failed to register {}: {}", node.id, e);
            });
        }
        // 重建边
        for (from, to) in &self.registry.edges {
            reg.add_dependency(from, to).unwrap_or_else(|e| {
                eprintln!("[KBCapabilityTree] Failed to add edge {} -> {}: {}", from, to, e);
            });
        }
        reg
    }
}

/// 经验条目映射 (用于 experience-tree 吸收)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityEvolutionExperience {
    pub cycle: String,
    pub node_id: String,
    pub op: EvolutionOp,
    pub from_nodes: Vec<String>,
    pub to_node: Option<String>,
    pub note: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl From<EvolutionLogEntry> for CapabilityEvolutionExperience {
    fn from(entry: EvolutionLogEntry) -> Self {
        Self {
            cycle: entry.cycle,
            node_id: entry.to_node.clone().unwrap_or_default(),
            op: entry.op,
            from_nodes: entry.from_nodes,
            to_node: entry.to_node,
            note: entry.note,
            timestamp: entry.timestamp,
        }
    }
}

/// 导出为 experience-tree 兼容格式
pub fn export_experiences(registry: &CapabilityRegistry) -> Vec<CapabilityEvolutionExperience> {
    let mut exps = Vec::new();
    for node in registry.nodes.values() {
        for entry in &node.evolution_log {
            exps.push(CapabilityEvolutionExperience {
                cycle: entry.cycle.clone(),
                node_id: node.id.clone(),
                op: entry.op,
                from_nodes: entry.from_nodes.clone(),
                to_node: entry.to_node.clone(),
                note: entry.note.clone(),
                timestamp: entry.timestamp,
            });
        }
    }
    exps
}

/// 从 experience-tree 导入演化日志
pub fn import_experiences(registry: &mut CapabilityRegistry, experiences: &[CapabilityEvolutionExperience]) {
    for exp in experiences {
        if let Some(node) = registry.get_mut(&exp.node_id) {
            // 去重: 检查是否已有相同日志
            let exists = node.evolution_log.iter().any(|e| 
                e.cycle == exp.cycle && e.op == exp.op && e.note == exp.note
            );
            if !exists {
                node.evolution_log.push(EvolutionLogEntry {
                    cycle: exp.cycle.clone(),
                    op: exp.op,
                    from_nodes: exp.from_nodes.clone(),
                    to_node: exp.to_node.clone(),
                    note: exp.note.clone(),
                    timestamp: exp.timestamp,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::{CapabilityNode, Domain};
    use crate::registry::CapabilityRegistry;

    #[test]
    fn test_kb_roundtrip() {
        let mut reg = CapabilityRegistry::new();
        let node = CapabilityNode::new_primitive(
            "test::primitive".into(),
            Domain::Core,
            vec!["test".into()],
        );
        reg.register(node).unwrap();

        let kb = KBCapabilityTree::from_registry(&reg);
        let reg2 = kb.to_registry();
        
        assert!(reg2.get("test::primitive").is_some());
    }
}