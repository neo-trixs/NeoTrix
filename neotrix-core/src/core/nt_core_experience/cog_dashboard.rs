// G403: Cognitive dashboard — Understand-Anything style interactive module visualization
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModuleLayer {
    Substrate,
    Perception,
    Cognition,
    MetaCognition,
    SelfEvolution,
    MetaArchitecture,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveModule {
    pub id: String,
    pub name: String,
    pub layer: ModuleLayer,
    pub health: f64,
    pub dependencies: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSnapshot {
    pub timestamp: u64,
    pub modules: Vec<CognitiveModule>,
    pub total_health: f64,
    pub layer_health: HashMap<String, f64>,
    pub critical_modules: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CogDashboard {
    pub modules: HashMap<String, CognitiveModule>,
    pub snapshots: Vec<DashboardSnapshot>,
    pub max_snapshots: usize,
}

impl CogDashboard {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            snapshots: Vec::new(),
            max_snapshots: 50,
        }
    }

    pub fn register_module(
        &mut self,
        id: &str,
        name: &str,
        layer: ModuleLayer,
        dependencies: Vec<String>,
    ) {
        self.modules.insert(
            id.to_string(),
            CognitiveModule {
                id: id.to_string(),
                name: name.to_string(),
                layer,
                health: 1.0,
                dependencies,
                description: String::new(),
            },
        );
    }

    pub fn update_health(&mut self, module_id: &str, health: f64) {
        if let Some(module) = self.modules.get_mut(module_id) {
            module.health = health.clamp(0.0, 1.0);
        }
    }

    pub fn snapshot(&mut self) -> DashboardSnapshot {
        let mut layer_health: HashMap<String, Vec<f64>> = HashMap::new();
        let mut critical = Vec::new();
        let mut warnings = Vec::new();

        for module in self.modules.values() {
            let layer_name = format!("{:?}", module.layer);
            layer_health
                .entry(layer_name)
                .or_default()
                .push(module.health);

            if module.health < 0.3 {
                critical.push(module.id.clone());
                warnings.push(format!(
                    "CRITICAL: Module '{}' health at {:.2}",
                    module.name, module.health
                ));
            } else if module.health < 0.6 {
                warnings.push(format!(
                    "WARNING: Module '{}' health at {:.2}",
                    module.name, module.health
                ));
            }
        }

        let total_health =
            self.modules.values().map(|m| m.health).sum::<f64>() / self.modules.len().max(1) as f64;

        let avg_layer_health: HashMap<String, f64> = layer_health
            .into_iter()
            .map(|(k, v)| {
                let avg = v.iter().sum::<f64>() / v.len() as f64;
                (k, avg)
            })
            .collect();

        let snapshot = DashboardSnapshot {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            modules: self.modules.values().cloned().collect(),
            total_health,
            layer_health: avg_layer_health,
            critical_modules: critical,
            warnings,
        };

        if self.snapshots.len() >= self.max_snapshots {
            self.snapshots.remove(0);
        }
        self.snapshots.push(snapshot.clone());
        snapshot
    }

    pub fn render_text(&self) -> String {
        let mut out = String::new();
        out.push_str("╔══════════════════════════════════════════╗\n");
        out.push_str("║        NEOTRIX COGNITIVE DASHBOARD      ║\n");
        out.push_str("╚══════════════════════════════════════════╝\n\n");

        let latest = self.snapshots.last();
        if let Some(snap) = latest {
            out.push_str(&format!(
                "Total Health: {:.1}%\n",
                snap.total_health * 100.0
            ));
            out.push_str(&format!("Module Count: {}\n", snap.modules.len()));

            out.push_str("\n── Layer Health ──\n");
            let mut layers: Vec<_> = snap.layer_health.iter().collect();
            layers.sort_by_key(|(k, _)| (*k).clone());
            for (layer, health) in &layers {
                let bar = Self::health_bar(**health);
                out.push_str(&format!("  {:20} {:5.1}% {}\n", layer, **health * 100.0, bar));
            }

            if !snap.warnings.is_empty() {
                out.push_str("\n── Warnings ──\n");
                for w in &snap.warnings {
                    out.push_str(&format!("  ⚠ {}\n", w));
                }
            }
        } else {
            out.push_str("No snapshots recorded yet.\n");
        }
        out
    }

    fn health_bar(health: f64) -> String {
        let filled = (health * 10.0) as usize;
        let empty = 10 - filled;
        format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
    }
}
