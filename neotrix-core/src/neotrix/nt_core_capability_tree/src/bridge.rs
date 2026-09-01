//! # Capability Tree ↔ Registry Bridge
//!
//! 连接演化视图 (CapabilityTree) 和运行视图 (CapabilityRegistry)。
//! 消除三重技能注册表冗余: 让两个注册表协同工作而非各自为政。
//!
//! ## 设计原则
//! - **演化视图**: 管理能力的生命周期、成熟度、依赖关系
//! - **运行视图**: 管理能力的执行、降级、资源分配
//! - **桥接**: 演化视图的节点可引用运行视图的能力，反之亦然

use crate::registry::CapabilityRegistry;
use std::collections::HashMap;

/// 从 core/l7_capability/registry 导入运行时类型
/// (避免循环依赖, 使用类型别名)
pub type RuntimeCapabilityId = String;
pub type TreeCapabilityId = String;

/// 桥接映射: 演化节点 ID → 运行时能力 ID
#[derive(Debug, Clone, Default)]
pub struct CapabilityBridge {
    /// 演化节点 → 运行时能力
    tree_to_runtime: HashMap<TreeCapabilityId, RuntimeCapabilityId>,
    /// 运行时能力 → 演化节点
    runtime_to_tree: HashMap<RuntimeCapabilityId, TreeCapabilityId>,
}

impl CapabilityBridge {
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册映射关系
    pub fn register(&mut self, tree_id: impl Into<String>, runtime_id: impl Into<String>) {
        let tree_id = tree_id.into();
        let runtime_id = runtime_id.into();
        
        // 如果树节点已有映射，先删除旧的运行时映射
        if let Some(old_runtime_id) = self.tree_to_runtime.get(&tree_id) {
            self.runtime_to_tree.remove(old_runtime_id);
        }
        
        // 如果运行时 ID 已有映射，先删除旧的树节点映射
        if let Some(old_tree_id) = self.runtime_to_tree.get(&runtime_id) {
            self.tree_to_runtime.remove(old_tree_id);
        }
        
        self.tree_to_runtime.insert(tree_id.clone(), runtime_id.clone());
        self.runtime_to_tree.insert(runtime_id, tree_id);
    }

    /// 从演化节点查找运行时能力 ID
    pub fn runtime_id_of(&self, tree_id: &str) -> Option<&str> {
        self.tree_to_runtime.get(tree_id).map(|s| s.as_str())
    }

    /// 从运行时能力查找演化节点 ID
    pub fn tree_id_of(&self, runtime_id: &str) -> Option<&str> {
        self.runtime_to_tree.get(runtime_id).map(|s| s.as_str())
    }

    /// 所有映射数量
    pub fn len(&self) -> usize {
        self.tree_to_runtime.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tree_to_runtime.is_empty()
    }
}

/// 自动发现: 扫描演化注册表的 provides tags, 匹配运行时注册表的 tags
/// 生成建议映射 (需人工确认后 apply)
pub fn discover_mappings(
    tree: &CapabilityRegistry,
    runtime_tags: &HashMap<String, Vec<String>>, // tag -> [runtime_capability_names]
) -> Vec<(String, String, String)> { // (tree_id, runtime_name, match_reason)
    let mut suggestions = Vec::new();

    for (node_id, node) in &tree.nodes {
        for provides in &node.provides {
            if let Some(runtime_names) = runtime_tags.get(provides.as_str()) {
                for name in runtime_names {
                    suggestions.push((
                        node_id.clone(),
                        name.clone(),
                        format!("provides tag '{}' matches", provides),
                    ));
                }
            }
        }
    }

    suggestions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_register_and_lookup() {
        let mut bridge = CapabilityBridge::new();
        bridge.register("nt_core_e8_reasoning", "e8_reasoning_tool");
        assert_eq!(bridge.runtime_id_of("nt_core_e8_reasoning"), Some("e8_reasoning_tool"));
        assert_eq!(bridge.tree_id_of("e8_reasoning_tool"), Some("nt_core_e8_reasoning"));
        assert_eq!(bridge.len(), 1);
    }

    #[test]
    fn test_bridge_empty() {
        let bridge = CapabilityBridge::new();
        assert!(bridge.is_empty());
        assert!(bridge.runtime_id_of("nonexistent").is_none());
    }

    #[test]
    fn test_bridge_reverse_lookup() {
        let mut bridge = CapabilityBridge::new();
        bridge.register("tree_1", "runtime_1");
        bridge.register("tree_2", "runtime_2");
        assert_eq!(bridge.tree_id_of("runtime_1"), Some("tree_1"));
        assert_eq!(bridge.tree_id_of("runtime_2"), Some("tree_2"));
        assert_eq!(bridge.tree_id_of("runtime_3"), None);
    }
}
