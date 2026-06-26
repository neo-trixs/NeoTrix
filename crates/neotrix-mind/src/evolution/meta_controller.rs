use std::collections::HashMap;
use crate::evolution::evolution_task::*;

#[derive(Debug, Clone)]
pub struct MetaTelemetry {
    pub layer_health: HashMap<TargetLayer, f64>,
    pub module_metrics: HashMap<String, f64>,
    pub overall_health: f64,
    pub cycle: u64,
    pub evolution_history: Vec<String>,
}

impl Default for MetaTelemetry {
    fn default() -> Self {
        Self {
            layer_health: HashMap::from([
                (TargetLayer::Self_, 1.0),
                (TargetLayer::Mind, 1.0),
                (TargetLayer::Core, 1.0),
                (TargetLayer::Body, 1.0),
            ]),
            module_metrics: HashMap::new(),
            overall_health: 1.0,
            cycle: 0,
            evolution_history: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModuleRegistration {
    pub name: String,
    pub layer: TargetLayer,
    pub capabilities: Vec<String>,
    pub current_health: f64,
    pub last_evolution: u64,
}

#[derive(Debug, Clone)]
pub struct MetaEvolutionController {
    scheduler: TaskScheduler,
    telemetry: MetaTelemetry,
    registered_modules: HashMap<String, ModuleRegistration>,
    evolution_count: u64,
}

impl MetaEvolutionController {
    pub fn new(max_active_tasks: usize) -> Self {
        Self {
            scheduler: TaskScheduler::new(max_active_tasks),
            telemetry: MetaTelemetry::default(),
            registered_modules: HashMap::new(),
            evolution_count: 0,
        }
    }

    pub fn register_module(&mut self, name: &str, layer: TargetLayer, capabilities: &[&str]) {
        self.registered_modules.insert(name.to_string(), ModuleRegistration {
            name: name.to_string(),
            layer,
            capabilities: capabilities.iter().map(|s| s.to_string()).collect(),
            current_health: 1.0,
            last_evolution: 0,
        });
    }

    pub fn unregister_module(&mut self, name: &str) {
        self.registered_modules.remove(name);
    }

    pub fn update_telemetry(&mut self, telem: MetaTelemetry) {
        self.telemetry = telem;
        self.telemetry.cycle += 1;
    }

    pub fn update_module_health(&mut self, name: &str, health: f64) {
        if let Some(m) = self.registered_modules.get_mut(name) {
            m.current_health = health;
        }
    }

    pub fn detect_needs_evolution(&self) -> Vec<(String, f64)> {
        let mut needs: Vec<(String, f64)> = self.registered_modules.iter()
            .filter(|(_, m)| (1.0 - m.current_health) > 0.15)
            .map(|(name, m)| (name.clone(), 1.0 - m.current_health))
            .collect();
        for (layer, health) in &self.telemetry.layer_health {
            if *health < 0.7 {
                needs.push((format!("{:?}_layer", layer), 1.0 - health));
            }
        }
        needs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        needs
    }

    pub fn auto_propose(&mut self) -> Vec<u64> {
        let needs = self.detect_needs_evolution();
        let mut submitted = Vec::new();
        for (module_name, gap) in needs.iter().take(3) {
            if let Some(module) = self.registered_modules.get(module_name) {
                let change_type = match module.layer {
                    TargetLayer::Body => ChangeType::HeuristicMutation,
                    TargetLayer::Self_ => ChangeType::ParameterTuning,
                    TargetLayer::Mind => ChangeType::StrategySwitch,
                    TargetLayer::Core => ChangeType::ArchitectureChange,
                };
                let id = self.scheduler.submit(
                    &format!("evolve_{}", module_name),
                    ModuleTarget { layer: module.layer, module_name: module_name.clone() },
                    TaskProposal {
                        change_type,
                        expected_impact: (*gap * 0.5).min(0.8),
                        estimated_risk: 0.3,
                        parameters: HashMap::new(),
                    },
                    if *gap > 0.3 { TaskPriority::High } else { TaskPriority::Medium },
                );
                submitted.push(id);
            }
        }
        submitted
    }

    pub fn tick(&mut self) -> Vec<u64> {
        self.telemetry.cycle += 1;
        self.evolution_count += 1;
        if self.evolution_count % 10 == 0 {
            return self.auto_propose();
        }
        Vec::new()
    }

    pub fn next_task(&mut self) -> Option<EvolutionTask> {
        self.scheduler.next_pending()
    }

    pub fn complete_task(&mut self, id: u64, result: TaskResult) {
        if let Some(completed) = self.scheduler.complete(id, result) {
            if let Some(module) = self.registered_modules.get_mut(&completed.target.module_name) {
                if let Some(ref r) = completed.result {
                    module.current_health = r.health_after;
                }
                module.last_evolution = completed.completed_at.unwrap_or(0);
            }
        }
    }

    pub fn scheduler_report(&self) -> String {
        self.scheduler.report()
    }

    pub fn modules_report(&self) -> String {
        let parts: Vec<String> = self.registered_modules.values().map(|m| {
            format!("{}:{:.2}", m.name, m.current_health)
        }).collect();
        format!("Modules[{}]", parts.join(","))
    }

    pub fn telemetry_report(&self) -> String {
        let layer_parts: Vec<String> = self.telemetry.layer_health.iter()
            .map(|(l, h)| format!("{:?}:{:.2}", l, h))
            .collect();
        format!("Tele[cycle={} overall={:.2} layers=[{}]]",
            self.telemetry.cycle, self.telemetry.overall_health, layer_parts.join(","))
    }

    pub fn get_module(&self, name: &str) -> Option<&ModuleRegistration> {
        self.registered_modules.get(name)
    }

    pub fn list_modules(&self) -> Vec<&ModuleRegistration> {
        self.registered_modules.values().collect()
    }

    pub fn module_count(&self) -> usize {
        self.registered_modules.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_controller() -> MetaEvolutionController {
        let mut ctrl = MetaEvolutionController::new(5);
        ctrl.register_module("tls_fingerprint", TargetLayer::Body, &["tls", "fingerprint"]);
        ctrl.register_module("network_evolution", TargetLayer::Body, &["evolution", "heuristic"]);
        ctrl.register_module("identity_core", TargetLayer::Self_, &["identity", "vsa"]);
        ctrl
    }

    #[test]
    fn test_register_and_list() {
        let ctrl = default_controller();
        assert_eq!(ctrl.module_count(), 3);
    }

    #[test]
    fn test_unregister_module() {
        let mut ctrl = default_controller();
        ctrl.unregister_module("tls_fingerprint");
        assert_eq!(ctrl.module_count(), 2);
    }

    #[test]
    fn test_update_health() {
        let mut ctrl = default_controller();
        ctrl.update_module_health("tls_fingerprint", 0.5);
        assert!((ctrl.get_module("tls_fingerprint").unwrap().current_health - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_detect_low_health() {
        let mut ctrl = default_controller();
        ctrl.update_module_health("tls_fingerprint", 0.3);
        ctrl.update_module_health("network_evolution", 0.9);
        let needs = ctrl.detect_needs_evolution();
        assert!(!needs.is_empty());
        assert_eq!(needs[0].0, "tls_fingerprint");
    }

    #[test]
    fn test_auto_propose_creates_tasks() {
        let mut ctrl = default_controller();
        ctrl.update_module_health("tls_fingerprint", 0.2);
        let ids = ctrl.auto_propose();
        assert!(!ids.is_empty());
    }

    #[test]
    fn test_tick_every_10_cycles_proposes() {
        let mut ctrl = default_controller();
        ctrl.update_module_health("tls_fingerprint", 0.2);
        let mut total = 0;
        for _ in 0..20 {
            total += ctrl.tick().len();
        }
        assert!(total >= 2);
    }

    #[test]
    fn test_complete_task_updates_health() {
        let mut ctrl = default_controller();
        ctrl.update_module_health("tls_fingerprint", 0.2);
        let ids = ctrl.auto_propose();
        assert!(!ids.is_empty());
        ctrl.complete_task(ids[0], TaskResult {
            success: true,
            metric_delta: HashMap::new(),
            health_after: 0.85,
            health_before: 0.2,
        });
        assert!((ctrl.get_module("tls_fingerprint").unwrap().current_health - 0.85).abs() < 0.01);
    }

    #[test]
    fn test_telemetry_update_increments_cycle() {
        let mut ctrl = default_controller();
        let old_cycle = ctrl.telemetry.cycle;
        ctrl.update_telemetry(MetaTelemetry::default());
        assert_eq!(ctrl.telemetry.cycle, old_cycle + 1);
    }

    #[test]
    fn test_layer_health_detection() {
        let mut ctrl = default_controller();
        let mut telem = MetaTelemetry::default();
        telem.layer_health.insert(TargetLayer::Body, 0.5);
        ctrl.update_telemetry(telem);
        let needs = ctrl.detect_needs_evolution();
        let body_layer = needs.iter().find(|(name, _)| name.contains("Body"));
        assert!(body_layer.is_some());
    }

    #[test]
    fn test_report_formatting() {
        let ctrl = default_controller();
        let sched_report = ctrl.scheduler_report();
        let mod_report = ctrl.modules_report();
        let telem_report = ctrl.telemetry_report();
        assert!(sched_report.contains("Scheduler["));
        assert!(mod_report.contains("Modules["));
        assert!(telem_report.contains("Tele["));
    }
}
