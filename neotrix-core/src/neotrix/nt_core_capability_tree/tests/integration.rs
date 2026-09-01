//! CapabilityBridge 集成测试

#[cfg(test)]
mod capability_bridge_integration {
    use nt_core_capability_tree::bridge::{CapabilityBridge, discover_mappings};
    use nt_core_capability_tree::registry::CapabilityRegistry;
    use nt_core_capability_tree::node::{CapabilityNode, Domain};
    use std::collections::HashMap;

    #[test]
    fn test_capability_bridge_discover_mappings() {
        // 创建演化注册表
        let mut tree = CapabilityRegistry::new();
        let node = CapabilityNode::new_primitive(
            "nt_core::e8_reasoning".into(),
            Domain::Core,
            vec!["e8_reasoning".into(), "hexagram".into()],
        );
        tree.register(node).unwrap();

        // 创建运行时标签映射
        let mut runtime_tags = HashMap::new();
        runtime_tags.insert("e8_reasoning".into(), vec!["e8_tool".into()]);
        runtime_tags.insert("hexagram".into(), vec!["hex_engine".into()]);

        // 发现映射
        let suggestions = discover_mappings(&tree, &runtime_tags);
        assert_eq!(suggestions.len(), 2);
        
        // 验证映射内容
        let tree_ids: Vec<_> = suggestions.iter().map(|(t, _, _)| t.as_str()).collect();
        assert!(tree_ids.contains(&"nt_core::e8_reasoning"));
    }

    #[test]
    fn test_capability_bridge_register_and_lookup() {
        let mut bridge = CapabilityBridge::new();
        
        // 注册映射
        bridge.register("nt_core::e8_reasoning", "e8_tool");
        bridge.register("nt_memory::kb_store", "kb_storage");
        
        // 验证双向查找
        assert_eq!(bridge.runtime_id_of("nt_core::e8_reasoning"), Some("e8_tool"));
        assert_eq!(bridge.tree_id_of("e8_tool"), Some("nt_core::e8_reasoning"));
        assert_eq!(bridge.runtime_id_of("nt_memory::kb_store"), Some("kb_storage"));
        assert_eq!(bridge.tree_id_of("kb_storage"), Some("nt_memory::kb_store"));
        
        // 验证不存在的键
        assert_eq!(bridge.runtime_id_of("nonexistent"), None);
        assert_eq!(bridge.tree_id_of("nonexistent"), None);
    }

    #[test]
    fn test_capability_bridge_len() {
        let mut bridge = CapabilityBridge::new();
        assert!(bridge.is_empty());
        
        bridge.register("tree_1", "runtime_1");
        assert_eq!(bridge.len(), 1);
        assert!(!bridge.is_empty());
        
        bridge.register("tree_2", "runtime_2");
        assert_eq!(bridge.len(), 2);
    }

    #[test]
    fn test_capability_bridge_overwrite() {
        let mut bridge = CapabilityBridge::new();
        
        // 注册映射
        bridge.register("tree_1", "runtime_1");
        assert_eq!(bridge.runtime_id_of("tree_1"), Some("runtime_1"));
        
        // 覆盖映射
        bridge.register("tree_1", "runtime_2");
        assert_eq!(bridge.runtime_id_of("tree_1"), Some("runtime_2"));
        assert_eq!(bridge.tree_id_of("runtime_2"), Some("tree_1"));
        // 旧的运行时 ID 不再映射
        assert_eq!(bridge.tree_id_of("runtime_1"), None);
    }

    #[test]
    fn test_capability_bridge_multiple_domain_mapping() {
        let mut bridge = CapabilityBridge::new();
        
        // 多个演化节点映射到不同的运行时能力
        bridge.register("nt_core::reasoning", "core_tool");
        bridge.register("nt_mind::evolution", "mind_tool");
        
        // 每个演化节点映射到自己的运行时能力
        assert_eq!(bridge.runtime_id_of("nt_core::reasoning"), Some("core_tool"));
        assert_eq!(bridge.runtime_id_of("nt_mind::evolution"), Some("mind_tool"));
        
        // 每个运行时能力映射回对应的演化节点
        assert_eq!(bridge.tree_id_of("core_tool"), Some("nt_core::reasoning"));
        assert_eq!(bridge.tree_id_of("mind_tool"), Some("nt_mind::evolution"));
    }
}
