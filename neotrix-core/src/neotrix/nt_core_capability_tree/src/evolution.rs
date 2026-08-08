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
                    };
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
                        node.promote_constellation();
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

        // 3. 发现可晋升节点
        for node in self.registry.promotable_nodes() {
            // 简化: 所有 dependents 都在生产即可晋升
            let all_deps_prod = node.dependents.iter().all(|d| {
                self.registry.get(d).map(|n| n.constellation as u8 >= 3).unwrap_or(false)
            });
            if all_deps_prod || node.dependents.is_empty() {
                plans.push(self.plan_mature(node.id.clone()));
            }
        }

        // 4. 发现分散重复能力 (同 provides 标签下有多个同层节点)
        let mut provides_map: HashMap<String, Vec<&CapabilityNode>> = HashMap::new();
        for node in self.registry.nodes.values() {
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