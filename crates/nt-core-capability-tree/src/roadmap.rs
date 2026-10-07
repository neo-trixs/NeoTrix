//! Architecture Evolution Roadmap — 批量注册 18 个新模块到能力树
//!
//! R-P100: 所有新模块必须注册到能力树。
//! 数据源: `roadmap_modules.json` (架构演进路线图 18 模块清单)

use crate::node::{CapabilityNode, ConstellationLevel, Domain, NodeLayer};
use crate::registry::{CapabilityTreeRegistry, RegistryError};
use serde::Deserialize;
use std::path::Path;

/// 从 JSON 加载的模块定义
#[derive(Debug, Deserialize)]
pub struct RoadmapModuleEntry {
    pub id: String,
    pub domain: String,
    #[serde(default = "default_layer")]
    pub layer: String,
    #[serde(default = "default_constellation")]
    pub constellation: String,
    #[serde(default)]
    pub provides: Vec<String>,
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default)]
    pub consciousness_layer: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub wiring_evidence: Option<String>,
}

fn default_layer() -> String {
    "L0Primitive".into()
}

fn default_constellation() -> String {
    "C0Compile".into()
}

/// Roadmap modules manifest
#[derive(Debug, Deserialize)]
pub struct RoadmapManifest {
    pub description: Option<String>,
    pub version: Option<String>,
    pub modules: Vec<RoadmapModuleEntry>,
}

/// 从 JSON 文件加载 roadmap manifest
pub fn load_manifest(path: &Path) -> Result<RoadmapManifest, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let manifest: RoadmapManifest = serde_json::from_str(&content)?;
    Ok(manifest)
}

/// 解析 domain 字符串为 Domain 枚举
fn parse_domain(s: &str) -> Option<Domain> {
    Domain::parse(&format!("NT-{}", s.to_uppercase()))
}

/// 解析 layer 字符串为 NodeLayer 枚举
fn parse_layer(s: &str) -> Option<NodeLayer> {
    match s.to_lowercase().as_str() {
        "l0" | "l0primitive" => Some(NodeLayer::L0Primitive),
        "l1" | "l1composite" => Some(NodeLayer::L1Composite),
        "l2" | "l2orchestrator" => Some(NodeLayer::L2Orchestrator),
        "l2world" => Some(NodeLayer::L2World),
        "l3" | "l3domainservice" => Some(NodeLayer::L3DomainService),
        "l3memory" => Some(NodeLayer::L3Memory),
        "l4" | "l4application" => Some(NodeLayer::L4Application),
        "l4cognition" => Some(NodeLayer::L4Cognition),
        "l5" | "l5conscious" => Some(NodeLayer::L5Conscious),
        "l6" | "l6self" => Some(NodeLayer::L6Self),
        "l7" | "l7capability" => Some(NodeLayer::L7Capability),
        "l8" | "l8autonomic" => Some(NodeLayer::L8Autonomic),
        _ => None,
    }
}

/// 解析 constellation 字符串为 ConstellationLevel 枚举
fn parse_constellation(s: &str) -> Option<ConstellationLevel> {
    match s.to_lowercase().as_str() {
        "c0" | "c0compile" => Some(ConstellationLevel::C0Compile),
        "c1" | "c1unittest" => Some(ConstellationLevel::C1UnitTest),
        "c2" | "c2integrationtest" => Some(ConstellationLevel::C2IntegrationTest),
        "c3" | "c3benchmark" => Some(ConstellationLevel::C3Benchmark),
        "c4" | "c4mainpipeline" => Some(ConstellationLevel::C4MainPipeline),
        "c5" | "c5selfhealing" => Some(ConstellationLevel::C5SelfHealing),
        "c6" | "c6evolutionloop" => Some(ConstellationLevel::C6EvolutionLoop),
        _ => None,
    }
}

/// 将 RoadmapModuleEntry 转换为 CapabilityNode
fn entry_to_node(entry: &RoadmapModuleEntry) -> Result<CapabilityNode, RegistryError> {
    let domain = parse_domain(&entry.domain).ok_or_else(|| {
        RegistryError::Validation(format!("未知域: {}", entry.domain))
    })?;

    let layer = parse_layer(&entry.layer).ok_or_else(|| {
        RegistryError::Validation(format!("未知层: {}", entry.layer))
    })?;
    let constellation = parse_constellation(&entry.constellation).ok_or_else(|| {
        RegistryError::Validation(format!("未知星座: {}", entry.constellation))
    })?;

    let mut node = CapabilityNode::new_primitive(
        entry.id.clone(),
        domain,
        entry.provides.clone(),
    );
    node.layer = layer;
    node.constellation = constellation;
    node.requires = entry.requires.clone();

    // 写入 wiring_evidence 供晋升门禁审计
    if let Some(evidence) = &entry.wiring_evidence {
        node.metadata.insert(
            "wiring_evidence".into(),
            serde_json::Value::String(evidence.clone()),
        );
    }

    // 写入意识层级标注 (元数据, 不影响树结构)
    if let Some(cl) = &entry.consciousness_layer {
        node.metadata.insert(
            "consciousness_layer".into(),
            serde_json::Value::String(cl.clone()),
        );
    }

    // 写入备注
    if let Some(note) = &entry.note {
        node.metadata.insert(
            "note".into(),
            serde_json::Value::String(note.clone()),
        );
    }

    // contract_deferred: 新注册模块尚未有完整契约
    node.metadata.insert(
        "contract_deferred".into(),
        serde_json::Value::Bool(true),
    );

    Ok(node)
}

/// 从 manifest 批量注册 roadmap 模块到能力树
///
/// 返回成功注册数和失败详情。
pub fn register_roadmap_modules(
    registry: &mut CapabilityTreeRegistry,
    manifest: &RoadmapManifest,
) -> (usize, Vec<(String, String)>) {
    let mut registered = 0;
    let mut errors = Vec::new();

    for entry in &manifest.modules {
        match entry_to_node(entry) {
            Ok(node) => {
                match registry.register(node) {
                    Ok(()) => registered += 1,
                    Err(e) => errors.push((entry.id.clone(), e.to_string())),
                }
            }
            Err(e) => errors.push((entry.id.clone(), e.to_string())),
        }
    }

    (registered, errors)
}

/// Result type for roadmap registration: (registered count, warnings).
type RoadmapResult = Result<(usize, Vec<(String, String)>), Box<dyn std::error::Error>>;

/// 从默认路径加载并注册 roadmap 模块
///
/// 默认路径: `crates/nt-core-capability-tree/roadmap_modules.json`
/// （2026-09-28 更正：原注释写 `neotrix-core/src/neotrix/nt_core_capability_tree/`，
///  该目录不存在；数据文件实际在本 crate 根下）
pub fn register_from_default_path(
    registry: &mut CapabilityTreeRegistry,
) -> RoadmapResult {
    // 相对于 crate root 的路径
    let manifest_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("roadmap_modules.json");
    let manifest = load_manifest(&manifest_path)?;
    let result = register_roadmap_modules(registry, &manifest);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_roadmap_manifest() {
        let manifest_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("roadmap_modules.json");
        if manifest_path.exists() {
            let manifest = load_manifest(&manifest_path).unwrap();
            assert_eq!(manifest.modules.len(), 41);
        }
    }

    #[test]
    fn test_register_roadmap_modules() {
        let manifest_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("roadmap_modules.json");
        if !manifest_path.exists() {
            return; // skip if file not present
        }
        let manifest = load_manifest(&manifest_path).unwrap();
        let mut registry = CapabilityTreeRegistry::new();
        let (registered, errors) = register_roadmap_modules(&mut registry, &manifest);
        assert_eq!(registered, 41, "all 41 modules should register: errors={:?}", errors);
        assert!(errors.is_empty(), "no errors expected: {:?}", errors);

        // 验证每个模块都可检索
        for entry in &manifest.modules {
            assert!(
                registry.get(&entry.id).is_some(),
                "module {} should be retrievable",
                entry.id
            );
        }
    }

    #[test]
    fn test_parse_domain_valid() {
        assert_eq!(parse_domain("memory"), Some(Domain::Memory));
        assert_eq!(parse_domain("core"), Some(Domain::Core));
        assert_eq!(parse_domain("shield"), Some(Domain::Shield));
    }

    #[test]
    fn test_parse_domain_invalid() {
        assert_eq!(parse_domain("nonexistent"), None);
    }

    #[test]
    fn test_entry_to_node() {
        let entry = RoadmapModuleEntry {
            id: "nt_memory::add_only_writes".into(),
            domain: "memory".into(),
            layer: "L0Primitive".into(),
            constellation: "C0Compile".into(),
            provides: vec!["memory.add_only_writes".into()],
            requires: vec!["memory.kv_store".into()],
            consciousness_layer: Some("L1Action".into()),
            note: None,
            wiring_evidence: Some("test_evidence".into()),
        };
        let node = entry_to_node(&entry).unwrap();
        assert_eq!(node.id, "nt_memory::add_only_writes");
        assert_eq!(node.domain, Domain::Memory);
        assert_eq!(node.layer, NodeLayer::L0Primitive);
        assert_eq!(node.constellation, ConstellationLevel::C0Compile);
        assert!(node.metadata.contains_key("wiring_evidence"));
        assert!(node.metadata.contains_key("consciousness_layer"));
        assert!(node.metadata.contains_key("contract_deferred"));
    }
}
