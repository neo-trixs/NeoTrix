//! NT-CAPABILITY-TREE: 立体多维度进化能力树
//!
//! 三维坐标: 领域(X) × 成熟度(Y) × 抽象层(Z)
//! 节点类型: Root Primitive(L0) / Composite Node(L1-L2) / Capability Constellation(L3-L4)
//! 演化机制: Budding / Grafting / Pruning / Cross-pollination / Maturation

#![forbid(unsafe_code)]

pub mod node;
pub mod registry;
pub mod evolution;
pub mod cli;
pub mod serialize;

pub use node::{CapabilityNode, NodeLayer, ConstellationLevel, Domain, RuneSocket, EvolutionOp, EvolutionLogEntry};
pub use registry::{CapabilityRegistry, RegistryError};
pub use evolution::{EvolutionEngine, EvolutionPlan, EvolutionAction};
pub use cli::CapabilityCli;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Commands;
    use crate::registry::RegistryExport;

    #[test]
    fn test_registry_basic() {
        let mut reg = CapabilityRegistry::new();
        let node = CapabilityNode::new_primitive(
            "nt_http::fetch_safe_http".into(),
            Domain::Memory,
            vec!["ssrf_guard".into(), "dns_pinning".into()],
        );
        reg.register(node).unwrap();
        assert!(reg.get("nt_http::fetch_safe_http").is_some());
    }

    #[test]
    fn test_evolution_graft() {
        let mut reg = CapabilityRegistry::new();
        
        // Register primitive
        let primitive = CapabilityNode::new_primitive(
            "nt_http::fetch_safe_http".into(),
            Domain::Memory,
            vec!["ssrf_guard".into()],
        );
        reg.register(primitive).unwrap();
        
        // Graft: fold scattered fetchers into the primitive
        let mut engine = EvolutionEngine::new(&mut reg);
        let plan = engine.plan_graft(
            "nt_http::fetch_safe_http".into(),
            vec![
                "web_miner.http_client".into(),
                "search.client".into(),
                "self_evolver.fetch_http".into(),
            ],
            "cycle_165_nt_mind_fetcher_integration".into(),
        );
        
        assert_eq!(plan.actions.len(), 1);
        assert!(matches!(plan.actions[0], EvolutionAction::Graft { .. }));
    }

    #[test]
    fn test_experience_targets_roundtrip() {
        // 经验驱动迭代目标: export → 序列化 → 反序列化 → import 保留 (save 不丢 targets)
        let mut reg = CapabilityRegistry::new();
        reg.experience_targets.push(serde_json::json!({
            "domain": "NT-ACT",
            "capability": "sandbox_guard",
            "action": "strengthen_or_bud",
            "rationale": "磁盘沙盒越界防护接线经验",
            "signal": 0.9,
        }));
        let export = reg.export();
        let json = serde_json::to_string(&export).unwrap();
        // 旧文件兼容: 无 experience_targets 字段也能反序列化
        let old_json = r#"{"nodes":[],"edges":[]}"#;
        let parsed_old: RegistryExport = serde_json::from_str(old_json).unwrap();
        assert!(parsed_old.experience_targets.is_empty());
        // 完整往返
        let parsed: RegistryExport = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.experience_targets.len(), 1);
        assert_eq!(
            parsed.experience_targets[0]["capability"].as_str().unwrap(),
            "sandbox_guard"
        );
    }

    #[test]
    fn test_experience_target_bud_plan() {
        // scan 消费 experience_targets → 缺失能力节点生成 Budding 计划
        let mut reg = CapabilityRegistry::new();
        reg.experience_targets.push(serde_json::json!({
            "domain": "NT-ACT",
            "capability": "sandbox_guard",
            "action": "strengthen_or_bud",
            "rationale": "磁盘沙盒越界防护",
            "signal": 0.9,
        }));
        let cli = CapabilityCli {
            command: Commands::Scan { apply: false },
            registry: std::path::PathBuf::from("/tmp/nt_unused.json"),
            cycle: "test".to_string(),
        };
        let plans = cli.experience_target_plans(&reg).unwrap();
        assert_eq!(plans.len(), 1);
        // 无现有节点 → Budding 建议
        assert!(matches!(plans[0].actions[0], EvolutionAction::Budding { .. }));
    }
}