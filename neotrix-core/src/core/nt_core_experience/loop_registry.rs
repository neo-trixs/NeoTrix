use std::collections::HashMap;

/// Lifecycle state of a registered loop
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum LoopLifecycle {
    /// Defined but not yet active
    Registered,
    /// Currently active and running
    Active,
    /// Paused (will not tick)
    Paused,
    /// Retired (no longer in use, kept for history)
    Retired,
    /// Deprecated (will be removed)
    Deprecated,
}

/// Standardized trigger configuration for a loop
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum LoopTrigger {
    /// Fixed interval in seconds
    Interval { interval_secs: u64 },
    /// Cron expression
    Cron { expression: String },
    /// Event-driven (reacts to specific events)
    EventDriven { event_types: Vec<String> },
    /// On-demand (manually triggered)
    OnDemand,
    /// Chained from another loop's completion
    Chained {
        parent_loop: String,
        on_outcome: String,
    },
}

/// A quantifiable objective that defines what "done" means for this loop
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LoopObjective {
    pub description: String,
    pub success_criteria: Vec<String>,
    pub max_iterations: u64,
    pub max_duration_secs: Option<u64>,
    pub stop_on_failure: bool,
}

/// Standardized loop definition (LSS-inspired)
///
/// Based on KanakMalpani's Loop Specification Standard (LSS 1.0) and
/// adapted for NeoTrix's VSA-native consciousness architecture.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LoopDefinition {
    /// Unique name for this loop
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Semantic version
    pub version: String,
    /// The loop's objective and stop conditions
    pub objective: LoopObjective,
    /// Trigger configuration
    pub trigger: LoopTrigger,
    /// Which handler to invoke
    pub handler_name: String,
    /// Required capabilities/resources
    pub required_capabilities: Vec<String>,
    /// Tags for discovery
    pub tags: Vec<String>,
    /// Lifecycle state
    pub lifecycle: LoopLifecycle,
    /// Time of last run
    pub last_run: Option<u64>,
    /// Total run count
    pub run_count: u64,
    /// Success rate
    pub success_rate: f64,
}

impl LoopDefinition {
    pub fn is_runnable(&self) -> bool {
        matches!(self.lifecycle, LoopLifecycle::Active)
    }

    pub fn record_run(&mut self, success: bool) {
        self.last_run = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );
        self.run_count += 1;
        let n = self.run_count as f64;
        self.success_rate =
            self.success_rate * ((n - 1.0) / n) + if success { 1.0 / n } else { 0.0 };
    }
}

/// The LoopRegistry implements the **standardized loop definition** primitive
/// of loop engineering.
///
/// Based on KanakMalpani's Loop Specification Standard (LSS 1.0) and
/// the loop definition patterns from cobusgreyling/loop-engineering.
///
/// Every loop in the system is registered here with a name, version,
/// objective, trigger, and lifecycle state. This allows:
/// - Discovery of available loops
/// - Standardized lifecycle management
/// - Cross-loop dependency tracking
/// - Audit trail for all loop executions
#[derive(Debug)]
pub struct LoopRegistry {
    loops: HashMap<String, LoopDefinition>,
    dependency_graph: HashMap<String, Vec<String>>,
    history: Vec<(String, u64, bool)>,
}

impl LoopRegistry {
    pub fn new() -> Self {
        Self {
            loops: HashMap::new(),
            dependency_graph: HashMap::new(),
            history: Vec::new(),
        }
    }

    pub fn register(&mut self, definition: LoopDefinition) -> Result<(), String> {
        let name = definition.name.clone();
        if self.loops.contains_key(&name) {
            return Err(format!("loop '{}' already registered", name));
        }
        self.loops.insert(name.clone(), definition);
        self.dependency_graph.entry(name).or_default();
        Ok(())
    }

    pub fn update(&mut self, definition: LoopDefinition) {
        let name = definition.name.clone();
        self.loops.insert(name, definition);
    }

    pub fn get(&self, name: &str) -> Option<&LoopDefinition> {
        self.loops.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut LoopDefinition> {
        self.loops.get_mut(name)
    }

    pub fn set_lifecycle(&mut self, name: &str, lifecycle: LoopLifecycle) -> Result<(), String> {
        let def = self
            .loops
            .get_mut(name)
            .ok_or_else(|| format!("loop '{}' not found", name))?;
        def.lifecycle = lifecycle;
        Ok(())
    }

    pub fn record_run(&mut self, name: &str, success: bool) -> Result<(), String> {
        let def = self
            .loops
            .get_mut(name)
            .ok_or_else(|| format!("loop '{}' not found", name))?;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        def.record_run(success);
        self.history.push((name.to_string(), timestamp, success));
        if self.history.len() > 10000 {
            self.history.remove(0);
        }
        Ok(())
    }

    pub fn add_dependency(&mut self, loop_name: &str, depends_on: &str) -> Result<(), String> {
        if !self.loops.contains_key(loop_name) {
            return Err(format!("loop '{}' not found", loop_name));
        }
        if !self.loops.contains_key(depends_on) {
            return Err(format!("dependency '{}' not found", depends_on));
        }
        self.dependency_graph
            .entry(loop_name.to_string())
            .or_default()
            .push(depends_on.to_string());
        Ok(())
    }

    pub fn list_by_lifecycle(&self, lifecycle: LoopLifecycle) -> Vec<&LoopDefinition> {
        self.loops
            .values()
            .filter(|d| d.lifecycle == lifecycle)
            .collect()
    }

    pub fn list_active(&self) -> Vec<&LoopDefinition> {
        self.list_by_lifecycle(LoopLifecycle::Active)
    }

    pub fn list_by_tag(&self, tag: &str) -> Vec<&LoopDefinition> {
        self.loops
            .values()
            .filter(|d| d.tags.contains(&tag.to_string()))
            .collect()
    }

    pub fn remove(&mut self, name: &str) -> Option<LoopDefinition> {
        let def = self.loops.remove(name)?;
        self.dependency_graph.remove(name);
        Some(def)
    }

    pub fn count(&self) -> usize {
        self.loops.len()
    }

    pub fn all_loops(&self) -> Vec<&LoopDefinition> {
        self.loops.values().collect()
    }

    pub fn dependencies(&self, name: &str) -> Vec<&str> {
        self.dependency_graph
            .get(name)
            .map(|deps| deps.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    pub fn stats(&self) -> LoopRegistryStats {
        let total = self.loops.len();
        let active = self.list_active().len();
        let success_rate = self
            .history
            .last()
            .map(|(_, _, success)| if *success { 1.0 } else { 0.0 })
            .unwrap_or(0.0);
        LoopRegistryStats {
            total_loops: total,
            active_loops: active,
            total_runs: self.history.len(),
            overall_success_rate: success_rate,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct LoopRegistryStats {
    pub total_loops: usize,
    pub active_loops: usize,
    pub total_runs: usize,
    pub overall_success_rate: f64,
}

impl Default for LoopRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_loop(name: &str) -> LoopDefinition {
        LoopDefinition {
            name: name.to_string(),
            description: format!("{} loop", name),
            version: "0.1.0".to_string(),
            objective: LoopObjective {
                description: "test loop".to_string(),
                success_criteria: vec!["all tests pass".to_string()],
                max_iterations: 10,
                max_duration_secs: None,
                stop_on_failure: true,
            },
            trigger: LoopTrigger::Interval {
                interval_secs: 3600,
            },
            handler_name: format!("{}_handler", name),
            required_capabilities: vec![],
            tags: vec!["test".to_string()],
            lifecycle: LoopLifecycle::Active,
            last_run: None,
            run_count: 0,
            success_rate: 0.0,
        }
    }

    #[test]
    fn test_registry_new() {
        let r = LoopRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = LoopRegistry::new();
        assert!(r.register(sample_loop("morning_triage")).is_ok());
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_registry_duplicate() {
        let mut r = LoopRegistry::new();
        r.register(sample_loop("test_loop")).unwrap();
        let result = r.register(sample_loop("test_loop"));
        assert!(result.is_err());
    }

    #[test]
    fn test_registry_get() {
        let mut r = LoopRegistry::new();
        r.register(sample_loop("my_loop")).unwrap();
        let def = r.get("my_loop");
        assert!(def.is_some());
        assert_eq!(def.unwrap().version, "0.1.0");
    }

    #[test]
    fn test_registry_set_lifecycle() {
        let mut r = LoopRegistry::new();
        r.register(sample_loop("test")).unwrap();
        assert!(r.set_lifecycle("test", LoopLifecycle::Paused).is_ok());
        assert_eq!(r.get("test").unwrap().lifecycle, LoopLifecycle::Paused);
    }

    #[test]
    fn test_registry_list_active() {
        let mut r = LoopRegistry::new();
        let mut def = sample_loop("active_loop");
        def.lifecycle = LoopLifecycle::Active;
        r.register(def).unwrap();
        let mut def2 = sample_loop("paused_loop");
        def2.lifecycle = LoopLifecycle::Paused;
        r.register(def2).unwrap();
        assert_eq!(r.list_active().len(), 1);
    }

    #[test]
    fn test_registry_record_run() {
        let mut r = LoopRegistry::new();
        r.register(sample_loop("tracked")).unwrap();
        assert!(r.record_run("tracked", true).is_ok());
        let def = r.get("tracked").unwrap();
        assert_eq!(def.run_count, 1);
        assert!((def.success_rate - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_registry_remove() {
        let mut r = LoopRegistry::new();
        r.register(sample_loop("to_remove")).unwrap();
        assert!(r.remove("to_remove").is_some());
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_add_dependency() {
        let mut r = LoopRegistry::new();
        r.register(sample_loop("child")).unwrap();
        r.register(sample_loop("parent")).unwrap();
        assert!(r.add_dependency("child", "parent").is_ok());
        assert_eq!(r.dependencies("child").len(), 1);
    }

    #[test]
    fn test_registry_add_dependency_missing() {
        let mut r = LoopRegistry::new();
        r.register(sample_loop("child")).unwrap();
        let result = r.add_dependency("child", "nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_registry_list_by_tag() {
        let mut r = LoopRegistry::new();
        let mut a = sample_loop("a");
        a.tags = vec!["critical".to_string()];
        r.register(a).unwrap();
        let mut b = sample_loop("b");
        b.tags = vec!["background".to_string()];
        r.register(b).unwrap();
        assert_eq!(r.list_by_tag("critical").len(), 1);
    }

    #[test]
    fn test_loop_definition_is_runnable() {
        let mut def = sample_loop("runnable");
        assert!(def.is_runnable());
        def.lifecycle = LoopLifecycle::Paused;
        assert!(!def.is_runnable());
        def.lifecycle = LoopLifecycle::Retired;
        assert!(!def.is_runnable());
    }

    #[test]
    fn test_registry_stats() {
        let mut r = LoopRegistry::new();
        r.register(sample_loop("loop_a")).unwrap();
        r.register(sample_loop("loop_b")).unwrap();
        r.record_run("loop_a", true).unwrap();
        let stats = r.stats();
        assert_eq!(stats.total_loops, 2);
        assert_eq!(stats.active_loops, 2);
    }
}
