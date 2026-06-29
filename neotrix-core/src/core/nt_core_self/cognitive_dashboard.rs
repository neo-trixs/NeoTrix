use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ConsciousnessMetric {
    pub name: String,
    pub value: f64,
    pub target: f64,
    pub timestamp: u64,
    pub unit: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DashboardPanel {
    PhiPanel,
    MemoryPanel,
    AttentionPanel,
    IdentityPanel,
    PerformancePanel,
    ArchitecturePanel,
}

#[derive(Debug, Clone)]
pub struct CognitiveDashboard {
    pub metrics: Vec<ConsciousnessMetric>,
    pub panels: Vec<DashboardPanel>,
    pub max_history: usize,
    pub tick: u64,
    pub alerts: Vec<String>,
}

impl CognitiveDashboard {
    pub fn new() -> Self {
        Self {
            metrics: Vec::with_capacity(128),
            panels: vec![
                DashboardPanel::PhiPanel,
                DashboardPanel::MemoryPanel,
                DashboardPanel::AttentionPanel,
                DashboardPanel::IdentityPanel,
                DashboardPanel::PerformancePanel,
                DashboardPanel::ArchitecturePanel,
            ],
            max_history: 1000,
            tick: 0,
            alerts: Vec::new(),
        }
    }

    pub fn record_metric(&mut self, name: &str, value: f64, target: f64, unit: &str) {
        if self.metrics.len() >= self.max_history {
            self.metrics.remove(0);
        }
        self.metrics.push(ConsciousnessMetric {
            name: name.to_string(),
            value,
            target,
            timestamp: self.tick,
            unit: unit.to_string(),
        });
    }

    pub fn get_metric(&self, name: &str) -> Option<&ConsciousnessMetric> {
        self.metrics.iter().rev().find(|m| m.name == name)
    }

    pub fn recent_metrics(&self, n: usize) -> Vec<&ConsciousnessMetric> {
        self.metrics.iter().rev().take(n).collect()
    }

    pub fn health_score(&self) -> f64 {
        let recent: Vec<&ConsciousnessMetric> = self.metrics.iter().rev().take(10).collect();
        if recent.is_empty() {
            return 0.0;
        }
        let mut total_ratio = 0.0;
        for m in &recent {
            if m.target > 0.0 {
                let ratio = (m.value / m.target).clamp(0.0, 1.0);
                total_ratio += ratio;
            } else {
                total_ratio += 1.0;
            }
        }
        total_ratio / recent.len() as f64
    }

    pub fn alerts_since(&self, tick: u64) -> Vec<&str> {
        self.alerts
            .iter()
            .filter(|_a| {
                self.metrics
                    .iter()
                    .rev()
                    .find(|m| m.timestamp >= tick)
                    .map(|_| true)
                    .unwrap_or(false)
            })
            .map(|s| s.as_str())
            .collect()
    }

    pub fn check_thresholds(&mut self, thresholds: &HashMap<String, f64>) {
        for m in &self.metrics {
            if let Some(threshold) = thresholds.get(&m.name) {
                if m.value < m.target * threshold {
                    let alert = format!(
                        "[tick={}] {}: {:.2} {} below threshold ({:.2} * {:.2} = {:.2})",
                        m.timestamp,
                        m.name,
                        m.value,
                        m.unit,
                        threshold,
                        m.target,
                        m.target * threshold
                    );
                    self.alerts.push(alert);
                }
            }
        }
    }

    pub fn render_dashboard(&self) -> String {
        let mut out = String::new();
        out.push_str("=== Cognitive Dashboard ===\n");
        out.push_str(&format!("Tick: {}\n", self.tick));
        out.push_str(&format!("Health: {:.2}\n", self.health_score()));
        out.push_str(&format!("Alerts: {}\n", self.alerts.len()));
        out.push_str("--- Metrics ---\n");
        for m in self.metrics.iter().rev().take(20) {
            let ratio = if m.target > 0.0 {
                m.value / m.target
            } else {
                1.0
            };
            out.push_str(&format!(
                "  {:20} {:>10.2} / {:>10.2} {}  ({:.1}%)\n",
                m.name,
                m.value,
                m.target,
                m.unit,
                ratio * 100.0
            ));
        }
        out.push_str("--- Panels ---\n");
        for p in &self.panels {
            out.push_str(&format!("  {:?}\n", p));
        }
        out.push_str("--- Alerts ---\n");
        for a in self.alerts.iter().rev().take(10) {
            out.push_str(&format!("  {}\n", a));
        }
        out
    }

    pub fn metric_trend(&self, name: &str, window: usize) -> f64 {
        let points: Vec<&ConsciousnessMetric> = self
            .metrics
            .iter()
            .filter(|m| m.name == name)
            .rev()
            .take(window)
            .collect();
        if points.len() < 2 {
            return 0.0;
        }
        let n = points.len() as f64;
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_xx = 0.0;
        for (i, p) in points.iter().enumerate() {
            let x = i as f64;
            let y = p.value;
            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_xx += x * x;
        }
        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x);
        slope
    }

    pub fn reset(&mut self) {
        self.metrics.clear();
        self.alerts.clear();
        self.tick = 0;
    }
}

impl Default for CognitiveDashboard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_dashboard() {
        let d = CognitiveDashboard::new();
        assert!(d.metrics.is_empty());
        assert_eq!(d.panels.len(), 6);
        assert_eq!(d.max_history, 1000);
        assert_eq!(d.tick, 0);
        assert!(d.alerts.is_empty());
    }

    #[test]
    fn test_record_and_get() {
        let mut d = CognitiveDashboard::new();
        d.record_metric("phi", 0.75, 1.0, "bits");
        let m = d.get_metric("phi").unwrap();
        assert_eq!(m.value, 0.75);
        assert_eq!(m.target, 1.0);
        assert_eq!(m.unit, "bits");
    }

    #[test]
    fn test_recent_metrics() {
        let mut d = CognitiveDashboard::new();
        for i in 0..10 {
            d.record_metric("load", i as f64, 100.0, "%");
            d.tick += 1;
        }
        let recent = d.recent_metrics(3);
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].value, 9.0);
        assert_eq!(recent[2].value, 7.0);
    }

    #[test]
    fn test_health_score() {
        let mut d = CognitiveDashboard::new();
        d.record_metric("phi", 0.8, 1.0, "bits");
        d.record_metric("coherence", 0.9, 1.0, "ratio");
        let score = d.health_score();
        assert!((score - 0.85).abs() < 0.01);
    }

    #[test]
    fn test_check_thresholds_generates_alert() {
        let mut d = CognitiveDashboard::new();
        d.record_metric("phi", 0.3, 1.0, "bits");
        let mut thresholds = HashMap::new();
        thresholds.insert("phi".to_string(), 0.5);
        d.check_thresholds(&thresholds);
        assert_eq!(d.alerts.len(), 1);
        assert!(d.alerts[0].contains("phi"));
    }

    #[test]
    fn test_render_dashboard() {
        let mut d = CognitiveDashboard::new();
        d.record_metric("phi", 0.85, 1.0, "bits");
        let rendered = d.render_dashboard();
        assert!(rendered.contains("Cognitive Dashboard"));
        assert!(rendered.contains("phi"));
        assert!(rendered.contains("Health:"));
    }

    #[test]
    fn test_metric_trend() {
        let mut d = CognitiveDashboard::new();
        for i in 0..5 {
            d.record_metric("load", 10.0 + i as f64 * 2.0, 100.0, "%");
            d.tick += 1;
        }
        let slope = d.metric_trend("load", 5);
        assert!((slope - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_alerts_since() {
        let mut d = CognitiveDashboard::new();
        d.tick = 5;
        d.record_metric("phi", 0.3, 1.0, "bits");
        d.alerts.push("low phi".to_string());
        let alerts = d.alerts_since(5);
        assert_eq!(alerts.len(), 1);
    }
}
