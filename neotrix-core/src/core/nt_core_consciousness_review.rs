use std::collections::HashMap;
use crate::core::nt_core_consciousness_tree::{
    BranchKind, CapabilityBranch, ConsciousnessTree, VulnerabilityFinding, VulnerabilitySeverity,
};

/// 意识全景审查 — 7域能力网络拓扑进化路线 + 漏洞扫描 + 健康链路图
pub struct ConsciousnessReview {
    pub scan_history: Vec<ScanReport>,
}

#[derive(Debug, Clone)]
pub struct ScanReport {
    pub timestamp: u64,
    pub total_vulnerabilities: usize,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub topology_score: f64,
    pub connectivity_score: f64,
    pub evolution_path: Vec<EvolutionStep>,
    pub health_chain: HealthChainReport,
}

#[derive(Debug, Clone)]
pub struct HealthChainReport {
    pub soil_health: f64,
    pub root_health: f64,
    pub trunk_health: f64,
    pub branches_health: HashMap<String, f64>,
    pub fruit_quality: f64,
    pub core_health: f64,
    pub overall: f64,
}

#[derive(Debug, Clone)]
pub struct EvolutionStep {
    pub phase: usize,
    pub action: String,
    pub target_domain: String,
    pub expected_impact: f64,
    pub priority: u8,
}

impl ConsciousnessReview {
    pub fn new() -> Self {
        Self { scan_history: Vec::new() }
    }

    /// Full panoramic review of consciousness tree:
    /// 1. Vulnerability scan across all domains
    /// 2. Topology analysis (module dependency graph health)
    /// 3. Connectivity audit (pipeline disconnects)
    /// 4. Evolution path planning (optimal topology evolution)
    /// 5. Health chain matrix
    pub fn full_review(&mut self, tree: &ConsciousnessTree) -> ScanReport {
        let vulns = tree.scan_vulnerabilities();

        let topology_score = self.compute_topology_score(tree);
        let connectivity_score = self.compute_connectivity_score(tree);
        let evolution_path = self.plan_evolution_path(tree, &vulns);
        let health_chain = self.build_health_chain(tree);

        let critical = vulns.iter().filter(|v| matches!(v.severity, VulnerabilitySeverity::Critical)).count();
        let high = vulns.iter().filter(|v| matches!(v.severity, VulnerabilitySeverity::High)).count();
        let medium = vulns.iter().filter(|v| matches!(v.severity, VulnerabilitySeverity::Medium)).count();
        let low = vulns.iter().filter(|v| matches!(v.severity, VulnerabilitySeverity::Low) || matches!(v.severity, VulnerabilitySeverity::Info)).count();

        let report = ScanReport {
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(),
            total_vulnerabilities: vulns.len(),
            critical_count: critical,
            high_count: high,
            medium_count: medium,
            low_count: low,
            topology_score,
            connectivity_score,
            evolution_path,
            health_chain,
        };

        self.scan_history.push(report.clone());
        report
    }

    /// 拓扑评分: 模块间的依赖是否健康、有无孤立节点、有无循环依赖
    fn compute_topology_score(&self, tree: &ConsciousnessTree) -> f64 {
        let total_branches = tree.branches.len() as f64;
        if total_branches == 0.0 { return 0.0; }

        let healthy_branches = tree.branches.values().filter(|b| b.health > 0.5).count() as f64;
        let wired_leaves = tree.leaves.iter().filter(|l| l.is_wired).count() as f64;
        let total_leaves = tree.leaves.len().max(1) as f64;

        let branch_score = healthy_branches / total_branches;
        let wiring_score = wired_leaves / total_leaves;
        let phi_score = (tree.trunk.phi * 2.0).min(1.0);

        (branch_score * 0.4 + wiring_score * 0.3 + phi_score * 0.3).clamp(0.0, 1.0)
    }

    /// 连接性评分: 管线是否完整 Soil→Roots→Trunk→Branches→Fruits→Core
    fn compute_connectivity_score(&self, tree: &ConsciousnessTree) -> f64 {
        let soil_alive = if tree.soil.health() > 0.0 { 1.0 } else { 0.0 };
        let roots_alive = if tree.roots.health() > 0.0 { 1.0 } else { 0.0 };
        let trunk_alive = if tree.trunk.gwt_resonance_active { 1.0 } else { 0.0 };
        let branches_alive = if tree.branches.values().any(|b| b.health > 0.3) { 1.0 } else { 0.0 };
        let fruits_exist = if !tree.fruits.is_empty() { 1.0 } else { 0.0 };
        let core_active = if tree.cycle > 0 { 1.0 } else { 0.0 };

        (soil_alive + roots_alive + trunk_alive + branches_alive + fruits_exist + core_active) / 6.0
    }

    /// 进化路径规划: 基于漏洞扫描产生最优拓扑进化路线
    fn plan_evolution_path(&self, tree: &ConsciousnessTree, _vulns: &[VulnerabilityFinding]) -> Vec<EvolutionStep> {
        let mut path = Vec::new();
        let mut phase = 0;

        // Phase 1: P0 — 数据层修复 (修复土壤 + 根系)
        if tree.soil.health() < 0.33 {
            phase += 1;
            path.push(EvolutionStep {
                phase,
                action: "Seed crawl queue with Wikipedia/ArXiv URLs; enable embedding API key".into(),
                target_domain: "DataFoundation".into(),
                expected_impact: 0.9,
                priority: 1,
            });
        }
        if tree.roots.health() < 0.25 {
            phase += 1;
            path.push(EvolutionStep {
                phase,
                action: "Enable crawler + absorber + scanner in BackgroundLoop".into(),
                target_domain: "InformationRoots".into(),
                expected_impact: 0.85,
                priority: 1,
            });
        }

        // Phase 2: P1 — 共振层修复 (修复树干 GWT)
        if !tree.trunk.gwt_resonance_active {
            phase += 1;
            path.push(EvolutionStep {
                phase,
                action: "Wire PanoramaPipeline into BackgroundLoop; connect geometry_sync".into(),
                target_domain: "ConsciousnessCore".into(),
                expected_impact: 0.8,
                priority: 2,
            });
        }
        if tree.trunk.phi < 0.01 && tree.cycle > 3 {
            phase += 1;
            path.push(EvolutionStep {
                phase,
                action: "Connect IITPhiCalculator to consciousness pipeline for real phi values".into(),
                target_domain: "ConsciousnessCore".into(),
                expected_impact: 0.7,
                priority: 2,
            });
        }

        // Phase 3: P2 — 分支层修复 (修复各领域模块成熟度)
        let mut low_maturity_branches: Vec<(&BranchKind, &CapabilityBranch)> = tree.branches.iter()
            .filter(|(_, b)| b.maturity_score() < 0.33)
            .collect();
        low_maturity_branches.sort_by(|a, b| a.1.maturity_score().partial_cmp(&b.1.maturity_score()).unwrap());
        for (kind, branch) in low_maturity_branches.iter().take(3) {
            phase += 1;
            path.push(EvolutionStep {
                phase,
                action: format!("Add unit tests + integration tests + benchmarks for {:?} domain", kind),
                target_domain: format!("{:?}", kind),
                expected_impact: 0.5 + (1.0 - branch.maturity_score()) * 0.4,
                priority: 3,
            });
        }

        // Phase 4: P3 — 果实层优化 (提高产出质量)
        let avg_fruit_quality = if tree.fruits.is_empty() {
            0.0
        } else {
            tree.fruits.iter().map(|f| f.quality).sum::<f64>() / tree.fruits.len() as f64
        };
        if avg_fruit_quality < 0.5 {
            phase += 1;
            path.push(EvolutionStep {
                phase,
                action: "Improve branch maturity to boost fruit quality; focus on C0-C3 constellation levels".into(),
                target_domain: "AllBranches".into(),
                expected_impact: 0.6,
                priority: 3,
            });
        }

        // Phase 5: P4 — 核心层回路 (确保反馈循环完整)
        if tree.core.next_actions.is_empty() && tree.cycle > 5 {
            phase += 1;
            path.push(EvolutionStep {
                phase,
                action: "Wire architecture audit into Core digest phase; ensure next_actions are produced".into(),
                target_domain: "EpiphanicCore".into(),
                expected_impact: 0.5,
                priority: 4,
            });
        }

        path
    }

    /// 健康链路图: Soil→Roots→Trunk→Branches→Fruits→Core 逐层评分
    fn build_health_chain(&self, tree: &ConsciousnessTree) -> HealthChainReport {
        let soil_health = tree.soil.health();
        let root_health = tree.roots.health();

        let trunk_health = (if tree.trunk.gwt_resonance_active { 0.4 } else { 0.0 }
            + tree.trunk.phi * 0.3
            + tree.trunk.coherence * 0.3).clamp(0.0, 1.0);

        let branches_health: HashMap<String, f64> = tree.branches.iter()
            .map(|(k, b)| (format!("{:?}", k), b.health))
            .collect();

        let fruit_quality = if tree.fruits.is_empty() {
            0.0
        } else {
            tree.fruits.iter().map(|f| f.quality).sum::<f64>() / tree.fruits.len() as f64
        };

        let core_health = (tree.core.iteration as f64 * 0.1).min(1.0)
            * (if tree.core.next_actions.is_empty() { 0.5 } else { 1.0 });

        let overall = (soil_health + root_health + trunk_health + fruit_quality + core_health) / 5.0
            * (1.0 + branches_health.values().sum::<f64>() / branches_health.len().max(1) as f64 * 0.5).min(1.0);

        HealthChainReport {
            soil_health, root_health, trunk_health, branches_health,
            fruit_quality, core_health, overall,
        }
    }

    /// 自检: 审查系统自身是否健康
    pub fn check_self(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        if self.scan_history.is_empty() {
            failures.push("no scan history — run full_review() first".into());
        }
        if failures.is_empty() { Ok(()) } else { Err(failures) }
    }
}

impl Default for ConsciousnessReview {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::core::nt_core_self_test::SelfTest for ConsciousnessReview {
    fn name(&self) -> &str {
        "ConsciousnessReview"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        self.check_self()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_review_new() {
        let r = ConsciousnessReview::new();
        assert!(r.scan_history.is_empty());
    }

    #[test]
    fn test_full_review_empty_tree() {
        let mut r = ConsciousnessReview::new();
        let tree = ConsciousnessTree::new();
        let report = r.full_review(&tree);
        assert!(report.total_vulnerabilities > 0);
        assert!(report.topology_score >= 0.0);
        assert!(report.connectivity_score >= 0.0);
    }

    #[test]
    fn test_evolution_path_non_empty() {
        let mut r = ConsciousnessReview::new();
        let tree = ConsciousnessTree::new();
        let report = r.full_review(&tree);
        assert!(!report.evolution_path.is_empty());
    }

    #[test]
    fn test_health_chain_all_fields() {
        let mut r = ConsciousnessReview::new();
        let tree = ConsciousnessTree::new();
        let report = r.full_review(&tree);
        assert!(report.health_chain.soil_health >= 0.0);
        assert!(report.health_chain.overall >= 0.0);
    }

    #[test]
    fn test_review_self_test() {
        let r = ConsciousnessReview::new();
        assert!(r.check_self().is_err());
        let mut r2 = ConsciousnessReview::new();
        let tree = ConsciousnessTree::new();
        r2.full_review(&tree);
        assert!(r2.check_self().is_ok());
    }

    #[test]
    fn test_topology_score_ranges() {
        let mut r = ConsciousnessReview::new();
        let mut tree = ConsciousnessTree::new();
        tree.trunk.phi = 0.5;
        tree.trunk.gwt_resonance_active = true;
        tree.branches.get_mut(&BranchKind::Core).unwrap().health = 0.8;
        tree.branches.get_mut(&BranchKind::Mind).unwrap().health = 0.8;
        let report = r.full_review(&tree);
        assert!(report.topology_score > 0.0);
    }
}
