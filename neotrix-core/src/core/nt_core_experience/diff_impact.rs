// G404: Diff impact analysis — Understand-Anything inspired, predict ripple effects before self-modification
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleNode {
    pub id: String,
    pub name: String,
    pub dependencies: Vec<String>,
    pub dependents: Vec<String>,
    pub complexity: f64,
    pub health: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactPrediction {
    pub module_id: String,
    pub impact_severity: f64,
    pub affected_dependents: Vec<String>,
    pub risk_level: RiskLevel,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    None,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffImpactReport {
    pub changed_module: String,
    pub change_description: String,
    pub direct_impacts: Vec<ImpactPrediction>,
    pub transitive_impacts: Vec<ImpactPrediction>,
    pub total_affected_modules: usize,
    pub max_risk: RiskLevel,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffImpactAnalyzer {
    pub module_graph: HashMap<String, ModuleNode>,
    pub impact_history: Vec<DiffImpactReport>,
    pub max_history: usize,
}

impl DiffImpactAnalyzer {
    pub fn new() -> Self {
        Self {
            module_graph: HashMap::new(),
            impact_history: Vec::new(),
            max_history: 100,
        }
    }

    pub fn register_module(&mut self, id: &str, name: &str, dependencies: Vec<String>) {
        let node = ModuleNode {
            id: id.to_string(),
            name: name.to_string(),
            dependencies: dependencies.clone(),
            dependents: Vec::new(),
            complexity: 1.0,
            health: 1.0,
        };
        self.module_graph.insert(id.to_string(), node);
        // Register this module as dependent on its dependencies
        for dep in dependencies {
            if let Some(dep_node) = self.module_graph.get_mut(&dep) {
                if !dep_node.dependents.contains(&id.to_string()) {
                    dep_node.dependents.push(id.to_string());
                }
            }
        }
    }

    pub fn analyze_change(&mut self, module_id: &str, change_desc: &str) -> DiffImpactReport {
        let mut direct = Vec::new();
        let mut transitive = Vec::new();
        let mut visited = HashSet::new();
        let mut affected = HashSet::new();

        // BFS to find all transitive impacts
        let mut queue = vec![module_id.to_string()];
        while let Some(current) = queue.pop() {
            if !visited.insert(current.clone()) {
                continue;
            }
            if let Some(node) = self.module_graph.get(&current) {
                for dependent in &node.dependents {
                    if !visited.contains(dependent) {
                        queue.push(dependent.clone());
                        affected.insert(dependent.clone());
                    }
                }
            }
        }

        // Classify direct vs transitive
        if let Some(node) = self.module_graph.get(module_id) {
            for dep in &node.dependents {
                let severity = self.compute_severity(module_id, dep);
                let risk = self.classify_risk(severity);
                direct.push(ImpactPrediction {
                    module_id: dep.clone(),
                    impact_severity: severity,
                    affected_dependents: self.collect_dependents(dep),
                    risk_level: risk,
                    description: format!(
                        "Change to '{}' directly impacts '{}' (severity: {:.2})",
                        module_id, dep, severity
                    ),
                });
            }
        }

        for aid in &affected {
            if aid != module_id && !self.is_direct_dependency(module_id, aid) {
                let severity = self.compute_severity(module_id, aid) * 0.5;
                let risk = self.classify_risk(severity);
                transitive.push(ImpactPrediction {
                    module_id: aid.clone(),
                    impact_severity: severity,
                    affected_dependents: self.collect_dependents(aid),
                    risk_level: risk,
                    description: format!(
                        "Transitive impact on '{}' via dependency chain (severity: {:.2})",
                        aid, severity
                    ),
                });
            }
        }

        let total_affected = direct.len() + transitive.len();
        let max_risk = direct
            .iter()
            .chain(transitive.iter())
            .map(|i| i.risk_level)
            .max_by(|a, b| Self::risk_order(*a).cmp(&Self::risk_order(*b)))
            .unwrap_or(RiskLevel::None);

        let recommendation = match max_risk {
            RiskLevel::Critical => {
                "CRITICAL: Impact reaches critical modules. Consider isolation or staged rollout."
                    .into()
            }
            RiskLevel::High => {
                "HIGH: Multiple dependents may break. Test all affected modules before applying."
                    .into()
            }
            RiskLevel::Medium => {
                "MEDIUM: Moderate ripple effect. Review and test changes thoroughly.".into()
            }
            RiskLevel::Low => "LOW: Limited impact. Standard testing sufficient.".into(),
            RiskLevel::None => "NONE: No dependents detected. Safe to apply.".into(),
        };

        let report = DiffImpactReport {
            changed_module: module_id.to_string(),
            change_description: change_desc.to_string(),
            direct_impacts: direct,
            transitive_impacts: transitive,
            total_affected_modules: total_affected,
            max_risk,
            recommendation,
        };

        // Archive
        if self.impact_history.len() >= self.max_history {
            self.impact_history.remove(0);
        }
        self.impact_history.push(report.clone());

        report
    }

    fn compute_severity(&self, _from: &str, _to: &str) -> f64 {
        // Proxy: based on complexity and health of the target
        if let Some(node) = self.module_graph.get(_to) {
            let complexity_factor = (node.complexity / 10.0).min(1.0);
            let health_penalty = (1.0 - node.health) * 0.3;
            (0.3 + complexity_factor * 0.5 - health_penalty).clamp(0.0, 1.0)
        } else {
            0.3
        }
    }

    fn classify_risk(&self, severity: f64) -> RiskLevel {
        if severity >= 0.8 {
            RiskLevel::Critical
        } else if severity >= 0.6 {
            RiskLevel::High
        } else if severity >= 0.3 {
            RiskLevel::Medium
        } else if severity > 0.0 {
            RiskLevel::Low
        } else {
            RiskLevel::None
        }
    }

    fn risk_order(risk: RiskLevel) -> u8 {
        match risk {
            RiskLevel::None => 0,
            RiskLevel::Low => 1,
            RiskLevel::Medium => 2,
            RiskLevel::High => 3,
            RiskLevel::Critical => 4,
        }
    }

    fn is_direct_dependency(&self, from: &str, to: &str) -> bool {
        self.module_graph
            .get(from)
            .map_or(false, |n| n.dependents.contains(&to.to_string()))
    }

    fn collect_dependents(&self, module_id: &str) -> Vec<String> {
        self.module_graph
            .get(module_id)
            .map_or_else(Vec::new, |n| n.dependents.clone())
    }
}
