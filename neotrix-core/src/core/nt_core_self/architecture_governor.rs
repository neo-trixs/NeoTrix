#![allow(dead_code)]

use std::collections::HashMap;

/// Type of architectural insight
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InsightType {
    CodeSmell,
    DependencyIssue,
    PerformanceGap,
    ArchitectureDrift,
    RefactoringOpportunity,
}

/// Per-module runtime metrics
#[derive(Debug, Clone)]
pub struct ModuleMetrics {
    pub module_name: String,
    pub invocation_count: u64,
    pub error_count: u64,
    pub avg_duration_ms: f64,
    pub last_health: f64,
}

impl ModuleMetrics {
    pub fn new(module_name: &str) -> Self {
        ModuleMetrics {
            module_name: module_name.to_string(),
            invocation_count: 0,
            error_count: 0,
            avg_duration_ms: 0.0,
            last_health: 1.0,
        }
    }
}

/// A discovered insight about architecture quality
#[derive(Debug, Clone)]
pub struct ArchitectureInsight {
    pub id: u64,
    pub description: String,
    pub insight_type: InsightType,
    pub severity: f64,
    pub module: String,
}

/// Architectural self-awareness layer: tracks module health, detects code smells,
/// analyzes dependency issues, and produces self-reports.
#[derive(Debug, Clone)]
pub struct ArchitectureSelfModel {
    pub modules: HashMap<String, ModuleMetrics>,
    pub insights: Vec<ArchitectureInsight>,
    pub next_insight_id: u64,
    pub governor: ArchitectureGovernor,
}

impl ArchitectureSelfModel {
    pub fn new() -> Self {
        ArchitectureSelfModel {
            modules: HashMap::new(),
            insights: Vec::new(),
            next_insight_id: 1,
            governor: ArchitectureGovernor::new(),
        }
    }

    pub fn track_invocation(&mut self, module: &str, duration_ms: f64, success: bool) {
        let entry = self
            .modules
            .entry(module.to_string())
            .or_insert_with(|| ModuleMetrics::new(module));
        let n = entry.invocation_count as f64;
        entry.avg_duration_ms = (entry.avg_duration_ms * n + duration_ms) / (n + 1.0);
        entry.invocation_count += 1;
        if !success {
            entry.error_count += 1;
        }
        entry.last_health = 1.0 - entry.error_count as f64 / entry.invocation_count.max(1) as f64;
    }

    pub fn detect_code_smells(&mut self) -> Vec<ArchitectureInsight> {
        let mut new_insights = Vec::new();
        for (name, metrics) in &self.modules {
            let error_rate = if metrics.invocation_count > 0 {
                metrics.error_count as f64 / metrics.invocation_count as f64
            } else {
                0.0
            };
            if error_rate > 0.2 {
                new_insights.push(ArchitectureInsight {
                    id: self.next_insight_id,
                    description: format!(
                        "Module '{}' has high error rate: {:.2}",
                        name, error_rate
                    ),
                    insight_type: InsightType::CodeSmell,
                    severity: error_rate,
                    module: name.clone(),
                });
                self.next_insight_id += 1;
            }
            if metrics.avg_duration_ms > 100.0 {
                new_insights.push(ArchitectureInsight {
                    id: self.next_insight_id,
                    description: format!(
                        "Module '{}' is slow: avg {:.1}ms",
                        name, metrics.avg_duration_ms
                    ),
                    insight_type: InsightType::PerformanceGap,
                    severity: (metrics.avg_duration_ms - 100.0) / 100.0,
                    module: name.clone(),
                });
                self.next_insight_id += 1;
            }
            if metrics.invocation_count < 10 {
                new_insights.push(ArchitectureInsight {
                    id: self.next_insight_id,
                    description: format!(
                        "Module '{}' is underutilized: {} invocations",
                        name, metrics.invocation_count
                    ),
                    insight_type: InsightType::ArchitectureDrift,
                    severity: 1.0 - metrics.invocation_count as f64 / 10.0,
                    module: name.clone(),
                });
                self.next_insight_id += 1;
            }
        }
        self.insights.extend(new_insights.clone());
        new_insights
    }

    pub fn dependency_analysis(
        &self,
        dependencies: &HashMap<String, Vec<String>>,
    ) -> Vec<ArchitectureInsight> {
        let mut insights = Vec::new();
        let mut visited = Vec::new();
        let mut stack: Vec<&str> = Vec::new();

        fn dfs<'a>(
            node: &'a str,
            deps: &'a HashMap<String, Vec<String>>,
            visited: &mut Vec<&'a str>,
            stack: &mut Vec<&'a str>,
            path: &mut Vec<String>,
        ) -> Option<Vec<String>> {
            if stack.contains(&node) {
                let idx = stack.iter().position(|&n| n == node).unwrap();
                let cycle = path[path.len() - (stack.len() - idx)..].to_vec();
                return Some(cycle);
            }
            if visited.contains(&node) {
                return None;
            }
            visited.push(node);
            stack.push(node);
            path.push(node.to_string());
            if let Some(neighbors) = deps.get(node) {
                for neighbor in neighbors {
                    if let Some(cycle) = dfs(neighbor, deps, visited, stack, path) {
                        return Some(cycle);
                    }
                }
            }
            stack.pop();
            path.pop();
            None
        }

        let keys: Vec<String> = dependencies.keys().cloned().collect();
        for key in &keys {
            let mut path = Vec::new();
            if let Some(cycle) = dfs(key, dependencies, &mut visited, &mut stack, &mut path) {
                insights.push(ArchitectureInsight {
                    id: 0,
                    description: format!("Circular dependency detected: {}", cycle.join(" -> ")),
                    insight_type: InsightType::DependencyIssue,
                    severity: 1.0,
                    module: cycle.join(" -> "),
                });
            }
        }
        insights
    }

    pub fn generate_report(&self) -> String {
        let mut lines = Vec::new();
        lines.push("=== ArchitectureSelfModel Report ===".to_string());
        lines.push(format!("Modules: {}", self.modules.len()));
        lines.push(format!("Insights: {}", self.insights.len()));
        lines.push(String::new());
        for (name, m) in &self.modules {
            let health = m.last_health;
            let status = if health > 0.8 {
                "healthy"
            } else if health > 0.5 {
                "degraded"
            } else {
                "critical"
            };
            lines.push(format!(
                "  {}: invocations={}, errors={}, avg={:.1}ms, health={:.2} [{}]",
                name, m.invocation_count, m.error_count, m.avg_duration_ms, health, status
            ));
        }
        if !self.insights.is_empty() {
            lines.push(String::new());
            lines.push("--- Insights ---".to_string());
            for ins in &self.insights {
                lines.push(format!(
                    "  [#{}] [{:?}] {} (sev={:.2})",
                    ins.id, ins.insight_type, ins.description, ins.severity
                ));
            }
        }
        lines.push(String::new());
        lines.push(self.governor.report());
        lines.join("\n")
    }

    pub fn module_count(&self) -> usize {
        self.modules.len()
    }

    pub fn healthy_module_count(&self) -> usize {
        self.modules
            .values()
            .filter(|m| m.last_health > 0.8)
            .count()
    }

    /// Bridge from SelfUnderstanding graph: ingest entities as tracked modules.
    /// Takes (entity_name, layer_name, description) tuples to avoid cross-module dependency.
    pub fn ingest_entities(
        &mut self,
        entities: &[(String, String, String)],
    ) -> Vec<ArchitectureInsight> {
        let mut insights = Vec::new();
        for (name, _layer, _desc) in entities {
            if !self.modules.contains_key(name) {
                self.modules.insert(
                    name.clone(),
                    ModuleMetrics {
                        module_name: name.clone(),
                        invocation_count: 0,
                        error_count: 0,
                        avg_duration_ms: 0.0,
                        last_health: 1.0,
                    },
                );
            }
        }

        // Detect layer gaps: layers with few entities are red flags
        let mut layer_counts: HashMap<&str, usize> = HashMap::new();
        for (_, layer, _) in entities {
            *layer_counts.entry(layer.as_str()).or_insert(0) += 1;
        }
        let expected = [
            "Substrate",
            "Perception",
            "Cognition",
            "MetaCognition",
            "SelfEvolution",
            "MetaArchitecture",
        ];
        for exp_layer in &expected {
            let count = layer_counts.get(exp_layer).copied().unwrap_or(0);
            if count == 0 {
                insights.push(ArchitectureInsight {
                    id: self.next_insight_id,
                    description: format!(
                        "Layer '{}' has zero entities — architecture gap",
                        exp_layer
                    ),
                    insight_type: InsightType::ArchitectureDrift,
                    severity: 1.0,
                    module: exp_layer.to_string(),
                });
                self.next_insight_id += 1;
                self.governor.record(DashboardMetric::ErrorRate, 0.9);
            } else if count < 3 {
                insights.push(ArchitectureInsight {
                    id: self.next_insight_id,
                    description: format!(
                        "Layer '{}' has only {} entities — sparse",
                        exp_layer, count
                    ),
                    insight_type: InsightType::ArchitectureDrift,
                    severity: 1.0 - (count as f64 / 5.0),
                    module: exp_layer.to_string(),
                });
                self.next_insight_id += 1;
            }
        }

        self.insights.extend(insights.clone());
        insights
    }
}

/// Metric type tracked by the governance dashboard
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DashboardMetric {
    PhiStar,
    MeanActivation,
    SyncIndex,
    Boredom,
    Curiosity,
    Competence,
    ScarCount,
    ActiveScars,
    PropCount,
    ErrorRate,
}

/// A single data point
#[derive(Debug, Clone)]
pub struct MetricPoint {
    pub tick: u64,
    pub value: f64,
}

/// Cognitive governance dashboard — real-time telemetry
#[derive(Debug, Clone)]
pub struct ArchitectureGovernor {
    pub metrics: HashMap<DashboardMetric, Vec<MetricPoint>>,
    pub max_history: usize,
    pub tick: u64,
    pub anomaly_threshold: f64,
}

impl ArchitectureGovernor {
    pub fn new() -> Self {
        ArchitectureGovernor {
            metrics: HashMap::new(),
            max_history: 1000,
            tick: 0,
            anomaly_threshold: 2.0,
        }
    }

    pub fn record(&mut self, metric: DashboardMetric, value: f64) {
        let entry = self
            .metrics
            .entry(metric)
            .or_insert_with(|| Vec::with_capacity(128));
        if entry.len() >= self.max_history {
            entry.remove(0);
        }
        entry.push(MetricPoint {
            tick: self.tick,
            value,
        });
    }

    pub fn current(&self, metric: DashboardMetric) -> Option<f64> {
        self.metrics
            .get(&metric)
            .and_then(|v| v.last().map(|p| p.value))
    }

    pub fn average(&self, metric: DashboardMetric) -> f64 {
        self.metrics.get(&metric).map_or(0.0, |v| {
            if v.is_empty() {
                0.0
            } else {
                v.iter().map(|p| p.value).sum::<f64>() / v.len() as f64
            }
        })
    }

    pub fn trend(&self, metric: DashboardMetric) -> f64 {
        let vals = match self.metrics.get(&metric) {
            Some(v) if v.len() >= 2 => v,
            _ => return 0.0,
        };
        let n = vals.len();
        let indices: Vec<f64> = (0..n).map(|i| i as f64).collect();
        let values: Vec<f64> = vals.iter().map(|p| p.value).collect();
        let mean_i = indices.iter().sum::<f64>() / n as f64;
        let mean_v = values.iter().sum::<f64>() / n as f64;
        let num: f64 = indices
            .iter()
            .zip(values.iter())
            .map(|(&i, &v)| (i - mean_i) * (v - mean_v))
            .sum();
        let den: f64 = indices.iter().map(|&i| (i - mean_i).powi(2)).sum();
        if den.abs() < 1e-10 {
            0.0
        } else {
            num / den
        }
    }

    pub fn detect_anomaly(&self, metric: DashboardMetric) -> bool {
        let vals = match self.metrics.get(&metric) {
            Some(v) if v.len() >= 5 => v,
            _ => return false,
        };
        let recent: Vec<f64> = vals.iter().rev().take(3).map(|p| p.value).collect();
        let older: Vec<f64> = vals
            .iter()
            .rev()
            .skip(3)
            .take(10)
            .map(|p| p.value)
            .collect();
        if older.is_empty() {
            return false;
        }
        let avg_older = older.iter().sum::<f64>() / older.len() as f64;
        let std_older = {
            let var =
                older.iter().map(|&v| (v - avg_older).powi(2)).sum::<f64>() / older.len() as f64;
            var.sqrt().max(0.001)
        };
        recent
            .iter()
            .any(|&v| (v - avg_older).abs() > self.anomaly_threshold * std_older)
    }

    pub fn step(&mut self) {
        self.tick += 1;
    }

    pub fn report(&self) -> String {
        let mut lines = Vec::new();
        for (metric, vals) in &self.metrics {
            if let Some(last) = vals.last() {
                let avg = vals.iter().map(|p| p.value).sum::<f64>() / vals.len() as f64;
                let anom = self.detect_anomaly(*metric);
                lines.push(format!(
                    "{:?}: current={:.3}, avg={:.3}, anom={}",
                    metric, last.value, avg, anom
                ));
            }
        }
        format!(
            "ArchitectureGovernor @ tick={}:\n  {}",
            self.tick,
            lines.join("\n  ")
        )
    }

    pub fn suggest_refactoring(&self) -> Vec<String> {
        let mut suggestions = Vec::new();
        if let Some(phi) = self.current(DashboardMetric::PhiStar) {
            if phi < 0.1 {
                suggestions
                    .push("LOW PHI: consider increasing connectivity between subsystems".into());
            }
        }
        if let Some(boredom) = self.current(DashboardMetric::Boredom) {
            if boredom > 0.7 {
                suggestions
                    .push("HIGH BOREDOM: introduce novel stimuli or exploration targets".into());
            }
        }
        if let Some(err) = self.current(DashboardMetric::ErrorRate) {
            if err > 0.3 {
                suggestions.push("HIGH ERROR RATE: roll back recent modifications".into());
            }
        }
        suggestions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_current() {
        let mut g = ArchitectureGovernor::new();
        g.record(DashboardMetric::PhiStar, 0.5);
        assert!((g.current(DashboardMetric::PhiStar).unwrap() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_average() {
        let mut g = ArchitectureGovernor::new();
        g.record(DashboardMetric::PhiStar, 0.3);
        g.record(DashboardMetric::PhiStar, 0.5);
        assert!((g.average(DashboardMetric::PhiStar) - 0.4).abs() < 0.01);
    }

    #[test]
    fn test_trend_increasing() {
        let mut g = ArchitectureGovernor::new();
        for i in 0..10 {
            g.record(DashboardMetric::PhiStar, i as f64 * 0.1);
            g.step();
        }
        assert!(g.trend(DashboardMetric::PhiStar) > 0.0);
    }

    #[test]
    fn test_anomaly_detection() {
        let mut g = ArchitectureGovernor::new();
        g.anomaly_threshold = 1.0;
        for _ in 0..10 {
            g.record(DashboardMetric::Boredom, 0.5);
            g.step();
        }
        assert!(!g.detect_anomaly(DashboardMetric::Boredom));
        g.record(DashboardMetric::Boredom, 10.0);
        g.step();
        assert!(g.detect_anomaly(DashboardMetric::Boredom));
    }

    #[test]
    fn test_report() {
        let mut g = ArchitectureGovernor::new();
        g.record(DashboardMetric::PhiStar, 0.5);
        let r = g.report();
        assert!(r.contains("PhiStar"));
    }

    #[test]
    fn test_suggest_refactoring() {
        let mut g = ArchitectureGovernor::new();
        g.record(DashboardMetric::PhiStar, 0.05);
        g.record(DashboardMetric::Boredom, 0.8);
        g.record(DashboardMetric::ErrorRate, 0.4);
        let s = g.suggest_refactoring();
        assert!(s.len() >= 2);
    }

    // --- ArchitectureSelfModel tests ---

    #[test]
    fn test_module_metrics_new() {
        let m = ModuleMetrics::new("test_module");
        assert_eq!(m.module_name, "test_module");
        assert_eq!(m.invocation_count, 0);
        assert_eq!(m.error_count, 0);
        assert!((m.avg_duration_ms - 0.0).abs() < 0.01);
        assert!((m.last_health - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_track_invocation() {
        let mut model = ArchitectureSelfModel::new();
        model.track_invocation("module_a", 50.0, true);
        model.track_invocation("module_a", 30.0, false);
        let m = model.modules.get("module_a").unwrap();
        assert_eq!(m.invocation_count, 2);
        assert_eq!(m.error_count, 1);
        assert!((m.avg_duration_ms - 40.0).abs() < 0.01);
        assert!((m.last_health - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_detect_high_error_rate() {
        let mut model = ArchitectureSelfModel::new();
        model.track_invocation("buggy_mod", 10.0, true);
        model.track_invocation("buggy_mod", 10.0, false);
        model.track_invocation("buggy_mod", 10.0, false);
        model.track_invocation("buggy_mod", 10.0, false);
        let smells = model.detect_code_smells();
        let has_high_error = smells
            .iter()
            .any(|i| matches!(i.insight_type, InsightType::CodeSmell) && i.module == "buggy_mod");
        assert!(has_high_error);
    }

    #[test]
    fn test_detect_slow_module() {
        let mut model = ArchitectureSelfModel::new();
        model.track_invocation("slow_mod", 200.0, true);
        let smells = model.detect_code_smells();
        let has_slow = smells.iter().any(|i| {
            matches!(i.insight_type, InsightType::PerformanceGap) && i.module == "slow_mod"
        });
        assert!(has_slow);
    }

    #[test]
    fn test_detect_underutilized() {
        let mut model = ArchitectureSelfModel::new();
        model.track_invocation("rare_mod", 10.0, true);
        model.track_invocation("freq_mod", 10.0, true);
        for _ in 0..12 {
            model.track_invocation("freq_mod", 10.0, true);
        }
        let smells = model.detect_code_smells();
        let has_underutilized = smells.iter().any(|i| {
            matches!(i.insight_type, InsightType::ArchitectureDrift) && i.module == "rare_mod"
        });
        let freq_underutilized = smells.iter().any(|i| {
            matches!(i.insight_type, InsightType::ArchitectureDrift) && i.module == "freq_mod"
        });
        assert!(has_underutilized);
        assert!(!freq_underutilized);
    }

    #[test]
    fn test_dependency_circular() {
        let mut deps: HashMap<String, Vec<String>> = HashMap::new();
        deps.insert("A".into(), vec!["B".into()]);
        deps.insert("B".into(), vec!["C".into()]);
        deps.insert("C".into(), vec!["A".into()]);
        let model = ArchitectureSelfModel::new();
        let insights = model.dependency_analysis(&deps);
        assert!(insights
            .iter()
            .any(|i| matches!(i.insight_type, InsightType::DependencyIssue)));
    }

    #[test]
    fn test_generate_report() {
        let mut model = ArchitectureSelfModel::new();
        model.track_invocation("alpha", 25.0, true);
        model.track_invocation("beta", 150.0, false);
        model.track_invocation("beta", 200.0, false);
        model.detect_code_smells();
        let report = model.generate_report();
        assert!(report.contains("alpha"));
        assert!(report.contains("beta"));
        assert!(report.contains("ArchitectureSelfModel Report"));
    }

    #[test]
    fn test_healthy_module_count() {
        let mut model = ArchitectureSelfModel::new();
        model.track_invocation("healthy_a", 10.0, true);
        model.track_invocation("healthy_b", 10.0, true);
        model.track_invocation("sick_c", 10.0, false);
        model.track_invocation("sick_c", 10.0, false);
        model.track_invocation("sick_c", 10.0, false);
        assert_eq!(model.module_count(), 3);
        assert_eq!(model.healthy_module_count(), 2);
    }
}
