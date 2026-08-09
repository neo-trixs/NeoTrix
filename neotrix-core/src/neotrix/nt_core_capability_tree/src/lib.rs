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

    #[test]
    fn test_constellation_promotion_metadata() {
        // P0 (novel-causal-chain 启发2): C0-C6 带晋级条件+代价+能力表现 (对标网文境界体系)
        let levels = [
            ConstellationLevel::C0Compile,
            ConstellationLevel::C1UnitTest,
            ConstellationLevel::C2IntegrationTest,
            ConstellationLevel::C3Benchmark,
            ConstellationLevel::C4MainPipeline,
            ConstellationLevel::C5SelfHealing,
            ConstellationLevel::C6EvolutionLoop,
        ];
        for lvl in levels {
            assert!(!lvl.promotion_requirement().is_empty(), "{} req", lvl.as_str());
            assert!(!lvl.promotion_cost().is_empty(), "{} cost", lvl.as_str());
            assert!(!lvl.capability_manifest().is_empty(), "{} manifest", lvl.as_str());
        }
        // 阶梯语义: 每一级有明确的晋级条件且非空
        assert_eq!(levels[0].promotion_requirement(), "cargo check 0 errors (可编译)");
        assert_eq!(levels[6].promotion_requirement(), "吸收循环闭环 (快照→蒸馏→落盘→反馈)");
        // next() 链完整性: C0→C1→...→C6
        let mut cur = Some(ConstellationLevel::C0Compile);
        let mut steps = 0;
        while let Some(c) = cur {
            cur = c.next();
            steps += 1;
        }
        assert_eq!(steps, 7, "C0-C6 共 7 级");
    }

    #[test]
    fn test_promotion_cost_monotonic() {
        // 代价随境界递增 (网文"境界越高代价越大"): C0 代价 < C6 代价
        let cost_c0 = ConstellationLevel::C0Compile.promotion_cost();
        let cost_c6 = ConstellationLevel::C6EvolutionLoop.promotion_cost();
        assert_ne!(cost_c0, cost_c6);
        // 每一级代价文本不同 (可区分性)
        let costs: std::collections::HashSet<_> = [
            ConstellationLevel::C0Compile,
            ConstellationLevel::C1UnitTest,
            ConstellationLevel::C2IntegrationTest,
            ConstellationLevel::C3Benchmark,
            ConstellationLevel::C4MainPipeline,
            ConstellationLevel::C5SelfHealing,
            ConstellationLevel::C6EvolutionLoop,
        ]
        .iter()
        .map(|l| l.promotion_cost())
        .collect();
        assert_eq!(costs.len(), 7, "7 级代价应互不相同");
    }
}