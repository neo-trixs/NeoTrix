//! 演化引擎: 规划与执行 Budding/Grafting/Pruning/CrossPollination/Maturation

use crate::node::{CapabilityNode, Domain, EvolutionLogEntry, EvolutionOp, NodeLayer};
use crate::registry::{CapabilityRegistry, RegistryError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvolutionAction {
    Budding {
        new_node_id: String,
        domain: Domain,
        provides: Vec<String>,
        layer: NodeLayer,
        note: String,
    },
    Graft {
        target_node_id: String,
        folded_nodes: Vec<String>,
        note: String,
    },
    Prune {
        node_id: String,
        reason: String,
    },
    CrossPollinate {
        shared_node_id: String,
        domain_a: Domain,
        domain_b: Domain,
        note: String,
    },
    Mature {
        node_id: String,
    },
    Strengthen {
        node_id: String,
        note: String,
    },
    /// JIT harness 合成: 为特定任务签名即时合成 agent harness (吸收 arXiv:2608.25593 JIT-Agent)。
    /// 强化既有进化闭环 — 将"任务自适应 harness"作为可复利归档的能力信号。
    HarnessSynthesize {
        node_id: String,
        task_signature: String,
        note: String,
    },
    /// 修复稳执行: 对不稳定/失败的 harness 执行修复, 提升可靠执行率 (JIT-Agent repair-for-stable)。
    RepairStable {
        node_id: String,
        note: String,
    },
    /// 性能档案复利: 将本次 harness 性能信号归档到节点 metadata, 复利驱动自进化。
    CompoundArchive {
        node_id: String,
        perf: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionPlan {
    pub cycle: String,
    pub actions: Vec<EvolutionAction>,
    pub rationale: String,
}

pub struct EvolutionEngine<'a> {
    registry: &'a mut CapabilityRegistry,
}

impl<'a> EvolutionEngine<'a> {
    pub fn new(registry: &'a mut CapabilityRegistry) -> Self {
        Self { registry }
    }

    /// 规划萌芽: 创建新 Primitive
    pub fn plan_bud(
        &self,
        new_node_id: String,
        domain: Domain,
        provides: Vec<String>,
        layer: NodeLayer,
        note: String,
    ) -> EvolutionPlan {
        let rationale = format!("Bud new capability: {}", note);
        EvolutionPlan {
            cycle: "pending".into(),
            actions: vec![EvolutionAction::Budding {
                new_node_id,
                domain,
                provides,
                layer,
                note,
            }],
            rationale,
        }
    }

    /// 规划嫁接: 将分散实现折叠到目标节点
    pub fn plan_graft(
        &self,
        target_node_id: String,
        folded_nodes: Vec<String>,
        note: String,
    ) -> EvolutionPlan {
        let rationale = format!("Graft scattered implementations into {}", target_node_id);
        EvolutionPlan {
            cycle: "pending".into(),
            actions: vec![EvolutionAction::Graft {
                target_node_id,
                folded_nodes,
                note,
            }],
            rationale,
        }
    }

    /// 规划修剪: 标记废弃/删除
    pub fn plan_prune(&self, node_id: String, reason: String) -> EvolutionPlan {
        EvolutionPlan {
            cycle: "pending".into(),
            actions: vec![EvolutionAction::Prune { node_id: node_id.clone(), reason: reason.clone() }],
            rationale: format!("Prune: {}", reason),
        }
    }

    /// 规划异花授粉: 跨域抽象共享 Primitive
    pub fn plan_cross_pollinate(
        &self,
        shared_node_id: String,
        domain_a: Domain,
        domain_b: Domain,
        note: String,
    ) -> EvolutionPlan {
        let rationale = format!("Cross-pollinate {} between {} and {}", shared_node_id, domain_a, domain_b);
        EvolutionPlan {
            cycle: "pending".into(),
            actions: vec![EvolutionAction::CrossPollinate {
                shared_node_id,
                domain_a,
                domain_b,
                note,
            }],
            rationale,
        }
    }

    /// 规划成熟晋升
    pub fn plan_mature(&self, node_id: String) -> EvolutionPlan {
        let rationale = format!("Promote {} to next constellation level", node_id);
        EvolutionPlan {
            cycle: "pending".into(),
            actions: vec![EvolutionAction::Mature { node_id: node_id.clone() }],
            rationale,
        }
    }

    /// 规划强化: 吸收经验强化既有节点 (R-P42 吸收强化现有节点, 不新建)
    pub fn plan_strengthen(&self, node_id: String, note: String) -> EvolutionPlan {
        let rationale = format!("Strengthen {} with absorbed experience: {}", node_id, note);
        EvolutionPlan {
            cycle: "pending".into(),
            actions: vec![EvolutionAction::Strengthen { node_id: node_id.clone(), note }],
            rationale,
        }
    }

    /// JIT harness 合成规划 (吸收 arXiv:2608.25593 JIT-Agent)。
    /// 为给定节点规划一次"任务自适应 harness 即时合成", 强化既有进化闭环而非新建模块。
    pub fn plan_harness_synthesize(&self, node_id: String, task_signature: String, note: String) -> EvolutionPlan {
        let rationale = format!("JIT harness synthesis for {}: {}", node_id, note);
        EvolutionPlan {
            cycle: "pending".into(),
            actions: vec![EvolutionAction::HarnessSynthesize { node_id, task_signature, note }],
            rationale,
        }
    }

    /// JIT harness 修复稳执行规划 (JIT-Agent repair-for-stable-execution)。
    pub fn plan_harness_repair(&self, node_id: String, note: String) -> EvolutionPlan {
        let rationale = format!("JIT harness repair-stable for {}: {}", node_id, note);
        EvolutionPlan {
            cycle: "pending".into(),
            actions: vec![EvolutionAction::RepairStable { node_id, note }],
            rationale,
        }
    }

    /// 性能档案复利归档规划 — 将 harness 性能信号写入节点 metadata 复利驱动自进化。
    pub fn plan_compound_archive(&self, node_id: String, perf: serde_json::Value) -> EvolutionPlan {
        let rationale = format!("Compound performance archive for {}", node_id);
        EvolutionPlan {
            cycle: "pending".into(),
            actions: vec![EvolutionAction::CompoundArchive { node_id, perf }],
            rationale,
        }
    }

    /// 执行计划
    pub fn execute(&mut self, mut plan: EvolutionPlan) -> Result<(), RegistryError> {
        for action in plan.actions.drain(..) {
            match action {
                EvolutionAction::Budding { new_node_id, domain, provides, layer, note } => {
                    let mut node = match layer {
                        NodeLayer::L0Primitive => CapabilityNode::new_primitive(new_node_id.clone(), domain, provides),
                        NodeLayer::L1Composite | NodeLayer::L2Orchestrator => {
                            CapabilityNode::new_composite(new_node_id.clone(), domain, layer, provides, vec![])
                        }
                        NodeLayer::L3DomainService | NodeLayer::L4Application => {
                            CapabilityNode::new_constellation(new_node_id.clone(), domain, layer, provides, vec![])
                        }
                        _ => {
                            // L5-L8 (transcendent/autonomic) 仍作为 composite 注册, 行为同 L3/L4 域服务
                            CapabilityNode::new_constellation(new_node_id.clone(), domain, layer, provides, vec![])
                        }
                    };
                    // P1 契约写门: bud 未声明契约 (input_schema/output_schema/fallback_chain)
                    // 时标记 contract_deferred — 允许注册 (R-P42 不强制改写既有路径),
                    // 但契约合规率如实区分 deferred, 供 consciousness_status/审查暴露缺口。
                    if !node
                        .metadata
                        .iter()
                        .any(|(k, _)| k == "input_schema" || k == "output_schema" || k == "fallback_chain")
                    {
                        node.metadata.insert(
                            "contract_deferred".into(),
                            serde_json::Value::Bool(true),
                        );
                    }
                    node.record_evolution(EvolutionLogEntry {
                        cycle: plan.cycle.clone(),
                        op: EvolutionOp::Budding,
                        from_nodes: vec![],
                        to_node: Some(new_node_id.clone()),
                        note,
                        timestamp: chrono::Utc::now(),
                    });
                    self.registry.register(node)?;
                }
                EvolutionAction::Graft { target_node_id, folded_nodes, note } => {
                    // 记录到目标节点
                    if let Some(target) = self.registry.get_mut(&target_node_id) {
                        target.record_evolution(EvolutionLogEntry {
                            cycle: plan.cycle.clone(),
                            op: EvolutionOp::Grafting,
                            from_nodes: folded_nodes.clone(),
                            to_node: Some(target_node_id.clone()),
                            note,
                            timestamp: chrono::Utc::now(),
                        });
                        // 标记 folded 为废弃
                        for folded in &folded_nodes {
                            if let Some(n) = self.registry.get_mut(folded) {
                                n.deprecate(format!("Folded into {}", target_node_id));
                            }
                        }
                    }
                }
                EvolutionAction::Prune { node_id, reason } => {
                    if let Some(node) = self.registry.get_mut(&node_id) {
                        node.deprecate(reason.clone());
                    }
                    // 如果无 dependents，可直接删除
                    if let Some(node) = self.registry.get(&node_id) {
                        if node.dependents.is_empty() {
                            self.registry.remove(&node_id)?;
                        }
                    }
                }
                EvolutionAction::CrossPollinate { shared_node_id, domain_a, domain_b, note } => {
                    // 将共享节点注册为两个域的依赖
                    if let Some(node) = self.registry.get_mut(&shared_node_id) {
                        node.record_evolution(EvolutionLogEntry {
                            cycle: plan.cycle.clone(),
                            op: EvolutionOp::CrossPollination,
                            from_nodes: vec![],
                            to_node: Some(shared_node_id.clone()),
                            note: format!("Cross-pollinated between {} and {}: {}", domain_a, domain_b, note),
                            timestamp: chrono::Utc::now(),
                        });
                    }
                }
                EvolutionAction::Mature { node_id } => {
                    if let Some(node) = self.registry.get_mut(&node_id) {
                        node.promote_constellation().map_err(|reason| {
                            RegistryError::Validation(format!("mature {} rejected by evidence gate: {}", node_id, reason))
                        })?;
                    }
                }
                EvolutionAction::Strengthen { node_id, note } => {
                    if let Some(node) = self.registry.get_mut(&node_id) {
                        node.record_evolution(EvolutionLogEntry {
                            cycle: plan.cycle.clone(),
                            op: EvolutionOp::Strengthen,
                            from_nodes: vec![],
                            to_node: Some(node_id.clone()),
                            note,
                            timestamp: chrono::Utc::now(),
                        });
                    }
                }
                EvolutionAction::HarnessSynthesize { node_id, task_signature, note } => {
                    if let Some(node) = self.registry.get_mut(&node_id) {
                        node.record_evolution(EvolutionLogEntry {
                            cycle: plan.cycle.clone(),
                            op: EvolutionOp::Strengthen,
                            from_nodes: vec![],
                            to_node: Some(node_id.clone()),
                            note: format!("jit_harness_synthesize({}): {}", task_signature, note),
                            timestamp: chrono::Utc::now(),
                        });
                        let entry = serde_json::json!({ "task_signature": task_signature, "note": note });
                        let mut sig = serde_json::json!([entry]);
                        if let Some(existing) = node.metadata.get("harness_syntheses") {
                            if let Some(arr) = existing.as_array() {
                                let mut v = arr.clone();
                                v.push(entry);
                                sig = serde_json::Value::Array(v);
                            }
                        }
                        node.metadata.insert("harness_syntheses".into(), sig);
                    }
                }
                EvolutionAction::RepairStable { node_id, note } => {
                    if let Some(node) = self.registry.get_mut(&node_id) {
                        node.record_evolution(EvolutionLogEntry {
                            cycle: plan.cycle.clone(),
                            op: EvolutionOp::Strengthen,
                            from_nodes: vec![],
                            to_node: Some(node_id.clone()),
                            note: format!("jit_harness_repair: {}", note),
                            timestamp: chrono::Utc::now(),
                        });
                        node.metadata.insert("harness_repaired".into(), serde_json::Value::Bool(true));
                    }
                }
                EvolutionAction::CompoundArchive { node_id, perf } => {
                    if let Some(node) = self.registry.get_mut(&node_id) {
                        node.record_evolution(EvolutionLogEntry {
                            cycle: plan.cycle.clone(),
                            op: EvolutionOp::Strengthen,
                            from_nodes: vec![],
                            to_node: Some(node_id.clone()),
                            note: "compounding_archive: record harness performance signal".into(),
                            timestamp: chrono::Utc::now(),
                        });
                        let mut archive = serde_json::json!([&perf]);
                        if let Some(existing) = node.metadata.get("compounding_archive") {
                            if let Some(arr) = existing.as_array() {
                                let mut v = arr.clone();
                                v.push(perf);
                                archive = serde_json::Value::Array(v);
                            }
                        }
                        node.metadata.insert("compounding_archive".into(), archive);
                    }
                }
            }
        }
        Ok(())
    }

    /// 自动扫描并建议演化计划
    pub fn auto_scan(&self, _current_cycle: &str) -> Vec<EvolutionPlan> {
        let mut plans = vec![];

        // 1. 发现孤儿 Primitive (无 dependents 且非入口)
        for node in self.registry.orphan_nodes() {
            if node.is_primitive() && node.constellation as u8 >= 2 {
                plans.push(self.plan_prune(
                    node.id.clone(),
                    format!("Orphan primitive at C{}, no dependents", node.constellation.as_str()),
                ));
            }
        }

        // 2. 发现过期节点 (C0/C1 超阈值) — 但跳过经验驱动新节点 (exp:: 前缀):
        //    刚被经验提升创建的 C0 节点, log 少会被误判 stale, 避免"建了又删"循环
        for node in self.registry.stale_nodes(3) {
            if node.id.starts_with("exp::") {
                continue;
            }
            plans.push(self.plan_prune(
                node.id.clone(),
                format!("Stale at {} for 3+ cycles", node.constellation.as_str()),
            ));
        }

        // 2.5 老化 exp:: 虚拟节点回收 (断链 #3: exp:: 节点只增不灭 → 90 天无演化活动即回收)
        // 经验虚拟节点是经验蒸馏的临时载体; 90 天无新演化活动 (Strengthen/Budding) 说明
        // 该经验已不再被强化, 标记 deprecated 进入回收候选 (真实模块节点不受影响)。
        for node_id in self.registry.aged_exp_nodes(90) {
            plans.push(self.plan_prune(
                node_id.clone(),
                "Aged exp:: virtual node, no evolution activity for 90+ days".to_string(),
            ));
        }

        // 3. 发现可晋升节点 — 仅自动晋升 C0→C1 (编译→单元测试, 拓扑/测试证据足够)。
        //    C1→C2 (生产接线) 及以上必须人工 cmd_mature 提供 wiring_evidence / evidence_gated
        //    (D16 自欺防线: dependents 是设计依赖非运行接线, 自动晋升会虚标 C2)。
        for node in self.registry.promotable_nodes() {
            if node.constellation as u8 >= 1 {
                continue;
            }
            plans.push(self.plan_mature(node.id.clone()));
        }

        // 4. 发现分散重复能力 (同 provides 标签下有多个同层节点)
        let mut provides_map: HashMap<String, Vec<&CapabilityNode>> = HashMap::new();
        for node in self.registry.nodes.values() {
            if node.deprecated {
                continue;
            }
            for tag in &node.provides {
                provides_map.entry(tag.clone()).or_default().push(node);
            }
        }
        for (tag, nodes) in provides_map {
            if nodes.len() > 1 {
                // 同层同域多个提供相同能力 -> 建议嫁接
                let by_domain_layer: HashMap<(Domain, NodeLayer), Vec<_>> = nodes.into_iter()
                    .fold(HashMap::new(), |mut acc, n| {
                        acc.entry((n.domain, n.layer)).or_default().push(n);
                        acc
                    });
                for ((domain, layer), group) in by_domain_layer {
                    if group.len() > 1 && layer <= NodeLayer::L2Orchestrator {
                        // 选择 constellation 最高的作为目标（group.len()>1 保证非空）。
                        let target = group.iter()
                            .max_by_key(|n| n.constellation as u8)
                            .expect("group.len() > 1 guarded above");
                        let folded: Vec<String> = group.iter()
                            .filter(|n| n.id != target.id)
                            .map(|n| n.id.clone())
                            .collect();
                        if !folded.is_empty() {
                            plans.push(self.plan_graft(
                                target.id.clone(),
                                folded,
                                format!("Consolidate {} providers in {}/{}", tag, domain, layer.as_str()),
                            ));
                        }
                    }
                }
            }
        }

        plans
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::CapabilityRegistry;

    fn seeded_registry() -> (CapabilityRegistry, String) {
        let mut reg = CapabilityRegistry::new();
        let id = "mind::harness::synthesizer".to_string();
        let node = CapabilityNode::new_primitive(id.clone(), Domain::Mind, vec!["agent_harness".into()]);
        reg.register(node).unwrap();
        (reg, id)
    }

    #[test]
    fn plan_harness_synthesize_carries_task_signature() {
        let mut reg = CapabilityRegistry::new();
        let engine = EvolutionEngine::new(&mut reg);
        let plan = engine.plan_harness_synthesize(
            "mind::harness::s".into(),
            "web_audit".into(),
            "task-adaptive harness".into(),
        );
        assert!(matches!(
            plan.actions[0],
            EvolutionAction::HarnessSynthesize { .. }
        ));
        if let EvolutionAction::HarnessSynthesize { task_signature, .. } = &plan.actions[0] {
            assert_eq!(task_signature, "web_audit");
        }
    }

    #[test]
    fn execute_synthesize_appends_to_harness_syntheses() {
        let (mut reg, id) = seeded_registry();
        {
            let mut engine = EvolutionEngine::new(&mut reg);
            let plan = engine.plan_harness_synthesize(
                id.clone(),
                "dark_web_osint".into(),
                "modular search/scrape/llm".into(),
            );
            engine.execute(plan).unwrap();
        }
        let node = reg.get(&id).unwrap();
        let arr = node.metadata.get("harness_syntheses").unwrap().as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["task_signature"].as_str().unwrap(), "dark_web_osint");
    }

    #[test]
    fn execute_compound_archive_aggregates_perf_signals() {
        let (mut reg, id) = seeded_registry();
        {
            let mut engine = EvolutionEngine::new(&mut reg);
            let p1 = serde_json::json!({ "reliability": 0.82 });
            let p2 = serde_json::json!({ "reliability": 0.91 });
            let plan1 = engine.plan_compound_archive(id.clone(), p1);
            engine.execute(plan1).unwrap();
            let plan2 = engine.plan_compound_archive(id.clone(), p2);
            engine.execute(plan2).unwrap();
        }
        let node = reg.get(&id).unwrap();
        let arr = node.metadata.get("compounding_archive").unwrap().as_array().unwrap();
        assert_eq!(arr.len(), 2, "compounding archive should aggregate perf signals");
    }

    #[test]
    fn execute_repair_marks_node_repaired() {
        let (mut reg, id) = seeded_registry();
        {
            let mut engine = EvolutionEngine::new(&mut reg);
            let plan = engine.plan_harness_repair(id.clone(), "stabilize deauth loop".into());
            engine.execute(plan).unwrap();
        }
        let node = reg.get(&id).unwrap();
        assert_eq!(node.metadata.get("harness_repaired"), Some(&serde_json::Value::Bool(true)));
    }
}