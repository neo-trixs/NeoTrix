use std::collections::HashMap;
use std::sync::Arc;

use crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase;
use super::nodes::EvolutionFruit;

use super::types::*;

/// Module-local Result alias (Session B interface uses `CTreeResult<CycleReport>`).
pub type CTreeResult<T> = std::result::Result<T, String>;

/// Report produced by a single 6-stage ConsciousnessTree cycle.
#[derive(Debug, Clone, Default)]
pub struct CycleReport {
    pub cycle: u64,
    /// Stage names executed in order: Soil→Roots→Trunk→Branches→Fruits→Core.
    pub stages: Vec<String>,
    pub node_count: u64,
    pub edge_count: u64,
    pub embedding_coverage: f64,
    pub connectivity: f64,
    pub branch_health: HashMap<BranchKind, f64>,
    pub guidance: Vec<String>,
}

impl ConsciousnessTree {
    /// Construct a tree wired to the shared knowledge base (Session B).
    pub fn new_with_kb(kb: Arc<KnowledgeBase>) -> Self {
        let mut tree = Self::new();
        tree.kb = Some(kb);
        tree
    }

    /// KB node id for a branch, namespaced under `consciousness://branch/`.
    pub fn branch_node_id(kind: &BranchKind) -> String {
        format!("consciousness://branch/{}", kind.kb_branch_id())
    }

    /// Awaken the tree: load the 11 branch nodes from KB (seeding each branch's
    /// initial state from the node), then inject KB health metrics.
    pub fn awaken(&mut self) -> CTreeResult<()> {
        let kb = self
            .kb
            .as_ref()
            .ok_or_else(|| "ConsciousnessTree has no KB handle; use new_with_kb".to_string())?;

        for kind in BranchKind::all() {
            let node_id = Self::branch_node_id(&kind);
            let node = kb
                .get_node(&node_id)
                .map_err(|e| format!("kb.get_node({}): {}", node_id, e))?;
            if let Some(node) = node {
                if let Some(branch) = self.branches.get_mut(&kind) {
                    // Inject node state as initial branch state (no embeddings clobber).
                    let seed = node.importance.clamp(0.0, 1.0);
                    if branch.health == 0.0 {
                        branch.health = seed;
                    }
                    branch.module_count = branch.module_count.max(1);
                }
            }
        }

        self.awakened = true;
        // Embeddings data flow: pull global health metrics into every branch.
        self.inject_kb_health()
    }

    /// Compute health metrics from KB (node count, edge count, embedding coverage,
    /// connectivity) and inject them into each branch's `kb_health` slot.
    pub fn inject_kb_health(&mut self) -> CTreeResult<()> {
        let kb = self
            .kb
            .as_ref()
            .ok_or_else(|| "ConsciousnessTree has no KB handle; use new_with_kb".to_string())?;

        let stats = kb.stats().map_err(|e| format!("kb.stats: {}", e))?;
        let node_count = stats.total_nodes.max(0) as u64;
        let edge_count = stats.total_edges.max(0) as u64;
        let embedding_count = kb.embedding_count() as u64;
        let embedding_coverage = if node_count > 0 {
            embedding_count as f64 / node_count as f64
        } else {
            0.0
        };
        let connectivity = if node_count > 0 {
            edge_count as f64 / node_count as f64
        } else {
            0.0
        };

        // Refresh soil data foundation from KB.
        self.soil.kb_node_count = node_count;
        self.soil.kb_edge_count = edge_count;
        self.soil.embedding_count = embedding_count;

        // Aggregate KB health contribution (balanced blend of coverage & connectivity).
        let global_health = (embedding_coverage * 0.5
            + connectivity.clamp(0.0, 1.0) * 0.5)
            .clamp(0.0, 1.0);

        let metrics = KbHealthMetrics {
            node_count,
            edge_count,
            embedding_coverage,
            connectivity,
            global_health,
        };

        for branch in self.branches.values_mut() {
            branch.kb_health = Some(metrics.clone());
            // Seed branch health from KB global health when still neutral (0.0).
            if branch.health == 0.0 {
                branch.health = global_health;
            }
        }

        Ok(())
    }

    /// Execute one full 6-stage growth cycle:
    /// Soil → Roots → Trunk → Branches → Fruits → Core.
    pub fn run_cycle(&mut self) -> CTreeResult<CycleReport> {
        let mut report = CycleReport::default();

        // 1. Soil — refresh data foundation (KB embeddings data flow).
        if self.kb.is_some() {
            self.inject_kb_health()?;
        }
        report.node_count = self.soil.kb_node_count;
        report.edge_count = self.soil.kb_edge_count;
        report.embedding_coverage = self
            .branches
            .values()
            .next()
            .and_then(|b| b.kb_health.as_ref())
            .map(|m| m.embedding_coverage)
            .unwrap_or(0.0);
        report.connectivity = self
            .branches
            .values()
            .next()
            .and_then(|b| b.kb_health.as_ref())
            .map(|m| m.connectivity)
            .unwrap_or(0.0);
        report.stages.push("Soil".to_string());

        // 2. Roots — ingestion pipeline readiness (mirrors InformationRoots health).
        report.stages.push("Roots".to_string());

        // 3. Trunk — consciousness core resonance cycle.
        self.trunk.resonance_cycle += 1;
        self.trunk.gwt_resonance_active = self.trunk.gwt_resonance_active
            || self.branches.values().any(|b| b.health > 0.0);
        report.stages.push("Trunk".to_string());

        // 4. Branches — per-branch evaluation.
        for kind in BranchKind::all() {
            if let Some(branch) = self.branches.get_mut(&kind) {
                branch.evaluate_constellation();
                branch.evaluate_node_tier(branch.self_test_count);
                report
                    .branch_health
                    .insert(kind.clone(), branch.health_with_runes());
            }
        }
        report.stages.push("Branches".to_string());

        // 5. Fruits — produce fruit for branches meeting the growth-health gate.
        for kind in BranchKind::all() {
            let produce = self
                .branches
                .get(&kind)
                .map(|b| b.health >= self.config.fruit_growth_health)
                .unwrap_or(false);
            if produce {
                let fruit = EvolutionFruit {
                    name: format!("kb-health-{}", kind.kb_branch_id()),
                    source_branch: kind.clone(),
                    description: format!(
                        "KB health cycle {} for {}",
                        self.cycle,
                        kind.kb_branch_id()
                    ),
                    produced_at_cycle: self.cycle,
                    quality: self
                        .branches
                        .get(&kind)
                        .map(|b| b.health_with_runes())
                        .unwrap_or(0.0),
                    ..Default::default()
                };
                self.fruits.push(fruit);
                if let Some(b) = self.branches.get_mut(&kind) {
                    b.fruit_count += 1;
                }
            }
        }
        report.stages.push("Fruits".to_string());

        // 6. Core — epiphanic guidance from aggregate state.
        let avg_health = if report.branch_health.is_empty() {
            0.0
        } else {
            report.branch_health.values().sum::<f64>() / report.branch_health.len() as f64
        };
        self.core.iteration += 1;
        self.core.last_cycle_guidance = vec![
            format!("cycle {} avg branch health {:.3}", self.cycle, avg_health),
            format!(
                "KB nodes={} edges={} embed_cov={:.3} conn={:.3}",
                report.node_count, report.edge_count, report.embedding_coverage, report.connectivity
            ),
        ];
        report.guidance = self.core.last_cycle_guidance.clone();
        report.stages.push("Core".to_string());

        self.cycle += 1;
        report.cycle = self.cycle;
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branch_node_id_namespacing() {
        assert_eq!(
            ConsciousnessTree::branch_node_id(&BranchKind::Core),
            "consciousness://branch/NT-CORE"
        );
        assert_eq!(
            ConsciousnessTree::branch_node_id(&BranchKind::Shield),
            "consciousness://branch/NT-SHIELD"
        );
        assert_eq!(BranchKind::all().len(), 11);
    }

    #[test]
    fn test_run_cycle_six_stages_no_kb() {
        // No KB wired — cycle must still execute the 6-stage loop gracefully.
        let mut tree = ConsciousnessTree::new();
        assert_eq!(tree.branches.len(), 11);
        let report = tree.run_cycle().expect("run_cycle without KB");
        assert_eq!(report.stages.len(), 6);
        assert_eq!(
            report.stages,
            vec![
                "Soil", "Roots", "Trunk", "Branches", "Fruits", "Core"
            ]
        );
        assert_eq!(report.cycle, 1);
        assert!(!report.guidance.is_empty());
    }

    #[test]
    fn test_kb_health_metrics_default() {
        let m = KbHealthMetrics::default();
        assert_eq!(m.node_count, 0);
        assert_eq!(m.embedding_coverage, 0.0);
    }
}
