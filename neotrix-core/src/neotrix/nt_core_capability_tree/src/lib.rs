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
}