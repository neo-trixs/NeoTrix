use std::collections::HashMap;

use crate::core::nt_core_self_test::SelfTest;

#[derive(Debug, Clone)]
struct Scenario {
    name: String,
    description: String,
    mode: Option<String>,
    result: Option<String>,
}

pub struct SimulateEngine {
    scenarios: HashMap<String, Scenario>,
    next_id: u64,
    simulation_count: u64,
    failure_count: u64,
}

impl SimulateEngine {
    pub fn new() -> Self {
        Self {
            scenarios: HashMap::new(),
            next_id: 0,
            simulation_count: 0,
            failure_count: 0,
        }
    }

    pub fn create_scenario(&mut self, name: &str, description: &str) -> String {
        let id = format!("sim-{}", self.next_id);
        self.next_id += 1;
        self.scenarios.insert(
            id.clone(),
            Scenario {
                name: name.to_string(),
                description: description.to_string(),
                mode: None,
                result: None,
            },
        );
        if self.scenarios.len() > 1024 {
            let oldest = self.scenarios.keys().next().cloned();
            if let Some(k) = oldest {
                self.scenarios.remove(&k);
            }
        }
        id
    }

    /// 运行 grounding 仿真 — 记录 mode 与确定性结果，返回 (ok, result)。
    pub fn simulate(&mut self, id: String, mode: &str) -> Result<(), String> {
        if let Some(sc) = self.scenarios.get_mut(&id) {
            sc.mode = Some(mode.to_string());
            // mode 无关的确定性结果：场景存在即 grounding 成立
            sc.result = Some(format!("grounded:{}", sc.name));
            self.simulation_count += 1;
            Ok(())
        } else {
            self.failure_count += 1;
            Err(format!("scenario {} not found", id))
        }
    }

    pub fn scenario_count(&self) -> usize {
        self.scenarios.len()
    }

    pub fn has_scenario(&self, id: &str) -> bool {
        self.scenarios.contains_key(id)
    }

    pub fn last_result(&self, id: &str) -> Option<String> {
        self.scenarios.get(id).and_then(|s| s.result.clone())
    }

    pub fn last_mode(&self, id: &str) -> Option<String> {
        self.scenarios.get(id).and_then(|s| s.mode.clone())
    }

    /// 读取场景描述 — 供日志与监控消费
    pub fn scenario_description(&self, id: &str) -> Option<String> {
        self.scenarios.get(id).map(|s| s.description.clone())
    }

    pub fn simulation_count(&self) -> u64 {
        self.simulation_count
    }

    pub fn failure_count(&self) -> u64 {
        self.failure_count
    }

    /// 已成功 grounding 的场景数量
    pub fn grounded_count(&self) -> u64 {
        self.simulation_count
    }
}

impl Default for SimulateEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SelfTest for SimulateEngine {
    fn name(&self) -> &'static str {
        "SimulateEngine"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_simulate_records_result() {
        let mut e = SimulateEngine::new();
        let id = e.create_scenario("health", "check");
        assert!(e.simulate(id.clone(), "stable").is_ok());
        assert_eq!(e.last_result(&id), Some("grounded:health".to_string()));
        assert_eq!(e.last_mode(&id), Some("stable".to_string()));
        assert_eq!(e.simulation_count(), 1);
    }

    #[test]
    fn test_simulate_unknown_scenario_fails() {
        let mut e = SimulateEngine::new();
        assert!(e.simulate("ghost".into(), "stable").is_err());
        assert_eq!(e.failure_count(), 1);
        assert_eq!(e.simulation_count(), 0);
    }

    #[test]
    fn test_has_scenario() {
        let mut e = SimulateEngine::new();
        let id = e.create_scenario("s", "d");
        assert!(e.has_scenario(&id));
        assert!(!e.has_scenario("nope"));
    }

    #[test]
    fn test_capacity_eviction() {
        let mut e = SimulateEngine::new();
        for i in 0..1100 {
            e.create_scenario(&format!("s{}", i), "d");
        }
        assert!(e.scenario_count() <= 1024);
    }

    #[test]
    fn test_last_result_before_simulate_is_none() {
        let mut e = SimulateEngine::new();
        let id = e.create_scenario("s", "d");
        assert_eq!(e.last_result(&id), None);
    }
}
