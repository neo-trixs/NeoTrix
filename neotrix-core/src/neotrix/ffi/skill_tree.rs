// SkillTree Implementation
// POE-inspired capability progression: Small Passive / Notable Passive / Keystone
//
// UCN Phase 4.1: 数据源从内置硬编码 → capability_tree DAG。
// 优先读取 .neotrix/capability_registry.json (RegistryExport 格式, 单一事实源),
// registry 缺失/空时回退内置种子节点 (保持 iOS 契约不变)。

use uniffi;
use std::sync::{Arc, RwLock};
use crate::neotrix::ffi::types::*;
use crate::neotrix::{CapabilityRegistry, NodeLayer};

/// capability_tree 注册表路径 (与 nt_core_capability_tree CLI 默认值一致)。
const REGISTRY_PATH: &str = ".neotrix/capability_registry.json";

struct SkillTreeInner {
    nodes: Vec<SkillNode>,
    allocated: u32,
    available: u32,
}

#[derive(Clone)]
#[derive(uniffi::Object)]
pub struct SkillTreeImpl {
    inner: Arc<RwLock<SkillTreeInner>>,
}

#[uniffi::export]
impl SkillTreeImpl {
    #[uniffi::constructor]
    pub fn init() -> Result<Self, NeoTrixError> {
        Ok(Self {
            inner: Arc::new(RwLock::new(SkillTreeInner {
                nodes: load_skill_nodes(),
                allocated: 0,
                available: 10,
            })),
        })
    }

    pub fn get_state(&self) -> SkillTreeState {
        let inner = self.inner.read().expect("ffi rwlock poisoned");
        SkillTreeState {
            nodes: inner.nodes.clone(),
            allocated_points: inner.allocated,
            available_points: inner.available,
            active_constellations: compute_constellations(&inner),
        }
    }

    pub fn allocate_point(&self, node_id: &str) -> Result<SkillNode, NeoTrixError> {
        let mut inner = self.inner.write().expect("ffi rwlock poisoned");
        if inner.available == 0 {
            return Err(NeoTrixError::OperationFailed);
        }
        let idx = inner.nodes.iter().position(|n| n.id == node_id).ok_or(NeoTrixError::NotFound)?;
        let prereqs_ok = inner.nodes[idx].prerequisites.iter().all(|p| {
            inner.nodes.iter().any(|n| n.id == *p && n.unlocked)
        });
        if !prereqs_ok {
            return Err(NeoTrixError::InvalidInput);
        }
        inner.nodes[idx].unlocked = true;
        inner.nodes[idx].progress = 1.0;
        inner.allocated += 1;
        inner.available -= 1;
        Ok(inner.nodes[idx].clone())
    }

    pub fn respec(&self) -> SkillTreeState {
        let mut inner = self.inner.write().expect("ffi rwlock poisoned");
        for node in inner.nodes.iter_mut() {
            node.unlocked = false;
            node.progress = 0.0;
        }
        inner.available = inner.allocated + 10;
        inner.allocated = 0;
        SkillTreeState {
            nodes: inner.nodes.clone(),
            allocated_points: inner.allocated,
            available_points: inner.available,
            active_constellations: compute_constellations(&inner),
        }
    }

    pub fn get_node(&self, node_id: &str) -> Result<SkillNode, NeoTrixError> {
        self.inner.read().expect("ffi rwlock poisoned").nodes.iter().find(|n| n.id == node_id).cloned().ok_or(NeoTrixError::NotFound)
    }

    pub fn is_constellation_active(&self, constellation: &str) -> bool {
        let inner = self.inner.read().expect("ffi rwlock poisoned");
        let constellation_level = constellation[1..].parse::<u8>().unwrap_or(0);
        let total = inner.nodes.iter().filter(|n| n.unlocked).count() as u8;
        total >= constellation_level
    }

    pub fn get_recommendations(&self, playstyle: &str) -> Vec<String> {
        match playstyle {
            "acquisition" => vec!["NT-WORLD-1".into(), "NT-WORLD-2".into(), "NT-MEMORY-1".into()],
            "evolution" => vec!["NT-MIND-1".into(), "NT-MIND-2".into(), "NT-CORE-1".into()],
            "balanced" => vec!["NT-CORE-1".into(), "NT-MIND-1".into(), "NT-WORLD-1".into()],
            _ => Vec::new(),
        }
    }
}

/// 从 capability_tree DAG 加载技能节点；registry 缺失/空时回退内置种子。
fn load_skill_nodes() -> Vec<SkillNode> {
    load_from_capability_tree().unwrap_or_else(build_skill_nodes)
}

/// 读取 .neotrix/capability_registry.json (RegistryExport 格式) 并映射为 SkillNode。
///
/// 映射契约 (capability_tree → skill_tree):
/// - domain: CapabilityNode.domain.as_str() 即 "NT-*" 全名, 直接透传
/// - tier: layer 映射 (L0-L1 → Small Passive, L2 → Notable Passive, L3-L4 → Keystone)
/// - prerequisites: direct_dependencies() 得出的 DAG 前置节点 id
/// - effects: 以 provides 标签生成 stat_boost 效果 (value 1.0 标记能力存在)
fn load_from_capability_tree() -> Option<Vec<SkillNode>> {
    load_from_registry_path(std::path::Path::new(REGISTRY_PATH))
}

/// 从指定路径加载 registry 并映射 (路径可注入, 便于测试)。
fn load_from_registry_path(path: &std::path::Path) -> Option<Vec<SkillNode>> {
    if !path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(path).ok()?;
    let export: nt_core_capability_tree::registry::RegistryExport =
        serde_json::from_str(&content).ok()?;
    if export.nodes.is_empty() {
        return None;
    }
    let mut registry = CapabilityRegistry::new();
    for node in export.nodes {
        registry.register(node).ok()?;
    }
    // 外部消费者容错: 端点不在注册表中的边跳过 (与 cli.rs load_registry 一致)
    for (from, to) in export.edges {
        if registry.nodes.contains_key(&from) && registry.nodes.contains_key(&to) {
            registry.add_dependency(&from, &to).ok()?;
        }
    }
    let mut nodes = Vec::new();
    for (id, node) in &registry.nodes {
        // prerequisites 语义: requires 标签 → 提供该标签的节点 id (DAG 前置)
        let prereqs: Vec<String> = node
            .requires
            .iter()
            .flat_map(|tag| registry.by_provides(tag).into_iter().map(|n| n.id.clone()))
            .collect();
        let tier = match node.layer {
            NodeLayer::L0Primitive | NodeLayer::L1Composite => "Small Passive",
            NodeLayer::L2Orchestrator => "Notable Passive",
            NodeLayer::L3DomainService | NodeLayer::L4Application => "Keystone",
        };
        let effects = node
            .provides
            .iter()
            .map(|tag| SkillEffect {
                effect_type: "stat_boost".into(),
                target: id.clone(),
                value: 1.0,
                description: tag.clone(),
            })
            .collect();
        nodes.push(SkillNode {
            id: id.clone(),
            name: id.clone(),
            description: node.provides.join(", "),
            tier: tier.into(),
            domain: node.domain.as_str().into(),
            prerequisites: prereqs,
            unlocked: false,
            progress: 0.0,
            effects,
        });
    }
    Some(nodes)
}

fn build_skill_nodes() -> Vec<SkillNode> {
    let mut nodes = Vec::new();
    let defs: Vec<(&str, &str, &str, &str, Vec<&str>, &str, &str, f32)> = vec![
        // NT-CORE
        ("NT-CORE-1", "E8 Clarity", "E8 reasoning confidence +15%", "Small Passive", vec![], "NT-CORE", "stat_boost", 0.15),
        ("NT-CORE-2", "GWT Resonance", "Attention routing efficiency +20%", "Small Passive", vec!["NT-CORE-1"], "NT-CORE", "stat_boost", 0.20),
        ("NT-CORE-3", "VSA Mastery", "HyperCube dimension 1024→2048", "Notable Passive", vec!["NT-CORE-2"], "NT-CORE", "new_ability", 0.0),
        ("NT-CORE-4", "Consciousness Core", "Phi integration score +0.1", "Keystone", vec!["NT-CORE-3"], "NT-CORE", "stat_boost", 0.1),
        // NT-MIND
        ("NT-MIND-1", "Pattern Extractor", "Distillation pattern extraction +30%", "Small Passive", vec![], "NT-MIND", "efficiency", 0.30),
        ("NT-MIND-2", "Skill Crystallizer", "New skills crystallize 25% faster", "Small Passive", vec!["NT-MIND-1"], "NT-MIND", "efficiency", 0.25),
        ("NT-MIND-3", "Evolution Accelerator", "SEAL cycle velocity +40%", "Notable Passive", vec!["NT-MIND-2"], "NT-MIND", "efficiency", 0.40),
        ("NT-MIND-4", "Meta-Crystallizer", "Auto-crystallize meta patterns", "Keystone", vec!["NT-MIND-3"], "NT-MIND", "new_ability", 0.0),
        // NT-MEMORY
        ("NT-MEMORY-1", "Spatial Memory", "Unlock spatial memory store", "Small Passive", vec![], "NT-MEMORY", "new_ability", 0.0),
        ("NT-MEMORY-2", "Semantic Indexer", "Search relevance +25%", "Small Passive", vec!["NT-MEMORY-1"], "NT-MEMORY", "stat_boost", 0.25),
        ("NT-MEMORY-3", "Knowledge Weaver", "Cross-namespace edge linking", "Notable Passive", vec!["NT-MEMORY-2"], "NT-MEMORY", "new_ability", 0.0),
        ("NT-MEMORY-4", "Infinite Archive", "Unlimited KB capacity", "Keystone", vec!["NT-MEMORY-3"], "NT-MEMORY", "new_ability", 0.0),
        // NT-WORLD
        ("NT-WORLD-1", "Sensor Fusion", "Multi-sensor data fusion", "Small Passive", vec![], "NT-WORLD", "new_ability", 0.0),
        ("NT-WORLD-2", "Pattern Radar", "Discovery confidence +20%", "Small Passive", vec!["NT-WORLD-1"], "NT-WORLD", "stat_boost", 0.20),
        ("NT-WORLD-3", "World Model", "Predictive world model", "Notable Passive", vec!["NT-WORLD-2"], "NT-WORLD", "new_ability", 0.0),
        ("NT-WORLD-4", "Omniscient View", "Full omniscient perception", "Keystone", vec!["NT-WORLD-3"], "NT-WORLD", "new_ability", 0.0),
    ];

    for (id, name, desc, tier, prereqs, domain, effect_type, value) in defs {
        nodes.push(SkillNode {
            id: id.into(),
            name: name.into(),
            description: desc.into(),
            tier: tier.into(),
            domain: domain.into(),
            prerequisites: prereqs.iter().map(|s| s.to_string()).collect(),
            unlocked: false,
            progress: 0.0,
            effects: vec![SkillEffect {
                effect_type: effect_type.into(),
                target: id.into(),
                value,
                description: desc.into(),
            }],
        });
    }
    nodes
}

fn compute_constellations(inner: &SkillTreeInner) -> Vec<String> {
    let mut active = Vec::new();
    let tiers: Vec<(&str, usize)> = vec![("NT-CORE", 0), ("NT-MIND", 1), ("NT-MEMORY", 2), ("NT-WORLD", 3)];
    for (domain, _base) in tiers {
        let domain_nodes: Vec<&SkillNode> = inner.nodes.iter().filter(|n| n.domain == domain && n.unlocked).collect();
        let c = (domain_nodes.len() as u8).min(6);
        if c > 0 {
            active.push(format!("C{}", c));
        }
    }
    active
}

#[cfg(test)]
mod tests {
    use super::*;
    use nt_core_capability_tree::node::{CapabilityNode, Domain as CapabilityDomain};

    /// 临时 registry 文件：drop 时自动删除。
    struct TempRegistry(std::path::PathBuf);
    impl TempRegistry {
        fn write(&self, export: &nt_core_capability_tree::registry::RegistryExport) {
            let content = serde_json::to_string(export).unwrap();
            if let Some(parent) = self.0.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            std::fs::write(&self.0, content).unwrap();
        }
        fn path(&self) -> &std::path::Path {
            &self.0
        }
    }
    impl Drop for TempRegistry {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    fn temp_registry(name: &str) -> TempRegistry {
        let dir = std::env::temp_dir().join(format!("nt-skill-tree-test-{}", std::process::id()));
        TempRegistry(dir.join(name))
    }

    #[test]
    fn test_load_skill_nodes_seed_fallback() {
        // 无 registry 文件 → 回退内置 16 节点种子
        let nodes = build_skill_nodes();
        assert_eq!(nodes.len(), 16, "seed has 16 nodes");
        assert_eq!(nodes[0].domain, "NT-CORE");
        // 前置依赖成链: NT-CORE-2 依赖 NT-CORE-1
        let n2 = nodes.iter().find(|n| n.id == "NT-CORE-2").unwrap();
        assert_eq!(n2.prerequisites, vec!["NT-CORE-1".to_string()]);
        // tier 语义
        assert_eq!(nodes[0].tier, "Small Passive");
        assert_eq!(nodes[2].tier, "Notable Passive");
        assert_eq!(nodes[3].tier, "Keystone");
    }

    #[test]
    fn test_load_from_registry_path_absent_returns_none() {
        // 不存在的路径 → None (触发 seed 回退)
        let missing = temp_registry("does-not-exist.json");
        assert!(load_from_registry_path(missing.path()).is_none());
    }

    #[test]
    fn test_load_from_registry_path_maps_nodes() {
        // 构造 2 节点 registry (core 域) + 依赖边 → 真实调用映射函数
        let reg_path = temp_registry("capability_tree.json");
        let mut registry = CapabilityRegistry::new();
        let p1 = CapabilityNode::new_primitive(
            "nt_test::e8_reason".into(),
            CapabilityDomain::Core,
            vec!["e8_reasoning".into()],
        );
        let mut p2 = CapabilityNode::new_primitive(
            "nt_test::gwt_route".into(),
            CapabilityDomain::Core,
            vec!["gwt_routing".into()],
        );
        p2.requires = vec!["e8_reasoning".into()];
        registry.register(p1).unwrap();
        registry.register(p2).unwrap();
        reg_path.write(&registry.export());

        let nodes = load_from_registry_path(reg_path.path()).expect("registry maps to nodes");
        assert_eq!(nodes.len(), 2);
        // gwt_route 依赖 e8_reason (requires 标签 → DAG 边)
        let gwt = nodes.iter().find(|n| n.id == "nt_test::gwt_route").unwrap();
        assert!(
            gwt.prerequisites.contains(&"nt_test::e8_reason".to_string()),
            "prereqs: {:?}",
            gwt.prerequisites
        );
        // domain 透传为 NT-* 全名; L0 → Small Passive
        assert!(nodes.iter().all(|n| n.domain == "NT-CORE"));
        assert!(nodes.iter().all(|n| n.tier == "Small Passive"));
        // provides 标签进 effects
        let e8 = nodes.iter().find(|n| n.id == "nt_test::e8_reason").unwrap();
        assert!(e8.effects.iter().any(|e| e.description == "e8_reasoning"));
    }

    #[test]
    fn test_load_from_registry_path_empty_returns_none() {
        // 空 registry (0 节点) → None → seed 回退
        let reg_path = temp_registry("empty.json");
        reg_path.write(&nt_core_capability_tree::registry::RegistryExport {
            nodes: vec![],
            edges: vec![],
            experience_targets: vec![],
        });
        assert!(load_from_registry_path(reg_path.path()).is_none());
    }

    #[test]
    fn test_skill_tree_init_contract() {
        // iOS 契约: available_points == 10, get_state 可用
        let tree = SkillTreeImpl::init().unwrap();
        let state = tree.get_state();
        assert_eq!(state.available_points, 10);
        // 双数据源任一可用: 无 registry → seed (16), 有 registry → DAG 映射 (≥1)
        assert!(!state.nodes.is_empty(), "nodes must be available from seed or DAG");
    }
}