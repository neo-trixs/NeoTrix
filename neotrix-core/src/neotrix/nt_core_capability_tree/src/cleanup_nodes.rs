//! 清理子系统能力节点注册
//!
//! 12 个清理能力节点 → NT-ACT / NT-WORLD / NT-SHIELD / NT-META 跨域注册

use crate::node::{CapabilityNode, Domain};
use crate::registry::{CapabilityRegistry, RegistryError};

pub fn register_cleanup_capabilities(registry: &mut CapabilityRegistry) -> Result<(), RegistryError> {
    // L1 Action (NT-ACT)
    let safe_deleter = CapabilityNode::new_primitive(
        "cleanup_safe_deleter".to_string(),
        Domain::Act,
        vec!["safe_delete".to_string(), "trash".to_string(), "archive".to_string()],
    );
    registry.register(safe_deleter)?;

    let cache_cleaner = CapabilityNode::new_primitive(
        "cleanup_cache_cleaner".to_string(),
        Domain::Act,
        vec!["cache_clean".to_string(), "npm_cache".to_string(), "pip_cache".to_string()],
    );
    registry.register(cache_cleaner)?;

    let dev_tool_cleaner = CapabilityNode::new_primitive(
        "cleanup_dev_tool_cleaner".to_string(),
        Domain::Act,
        vec!["dev_clean".to_string(), "node_modules".to_string(), "target".to_string()],
    );
    registry.register(dev_tool_cleaner)?;

    // L2 Perception (NT-WORLD)
    let system_scanner = CapabilityNode::new_primitive(
        "cleanup_system_scanner".to_string(),
        Domain::World,
        vec!["system_scan".to_string(), "large_file".to_string(), "old_log".to_string()],
    );
    registry.register(system_scanner)?;

    let cache_detector = CapabilityNode::new_primitive(
        "cleanup_cache_detector".to_string(),
        Domain::World,
        vec!["cache_detect".to_string(), "tool_cache".to_string()],
    );
    registry.register(cache_detector)?;

    let large_file_finder = CapabilityNode::new_primitive(
        "cleanup_large_file_finder".to_string(),
        Domain::World,
        vec!["large_file_find".to_string(), "size_scan".to_string()],
    );
    registry.register(large_file_finder)?;

    // L3 Embodiment (NT-SHIELD)
    let path_validator = CapabilityNode::new_primitive(
        "cleanup_path_validator".to_string(),
        Domain::Shield,
        vec!["path_validate".to_string(), "toctou_guard".to_string()],
    );
    registry.register(path_validator)?;

    let risk_assessor = CapabilityNode::new_primitive(
        "cleanup_risk_assessor".to_string(),
        Domain::Shield,
        vec!["risk_assess".to_string(), "safety_check".to_string()],
    );
    registry.register(risk_assessor)?;

    let permission_manager = CapabilityNode::new_primitive(
        "cleanup_permission_manager".to_string(),
        Domain::Shield,
        vec!["permission_check".to_string(), "sudo_guard".to_string()],
    );
    registry.register(permission_manager)?;

    // L6 Meta (NT-META)
    let coordinator = CapabilityNode::new_primitive(
        "cleanup_coordinator".to_string(),
        Domain::Meta,
        vec!["cleanup_coord".to_string(), "strategy_select".to_string()],
    );
    registry.register(coordinator)?;

    let rule_store = CapabilityNode::new_primitive(
        "cleanup_rule_store".to_string(),
        Domain::Memory,
        vec!["rule_store".to_string(), "config_persist".to_string()],
    );
    registry.register(rule_store)?;

    let history_log = CapabilityNode::new_primitive(
        "cleanup_history_log".to_string(),
        Domain::Memory,
        vec!["history_log".to_string(), "audit_trail".to_string()],
    );
    registry.register(history_log)?;

    Ok(())
}
