use std::collections::HashMap;

use crate::core::nt_core_self_test::{SelfTest, SelfTestResult};

struct Scenario {
    name: String,
    description: String,
    result: Option<String>,
}

pub struct SimulateEngine {
    scenarios: HashMap<String, Scenario>,
    next_id: u64,
}

impl SimulateEngine {
    pub fn new() -> Self {
        Self {
            scenarios: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn create_scenario(&mut self, name: &str, description: &str) -> String {
        let id = format!("sim-{}", self.next_id);
        self.next_id += 1;
        self.scenarios.insert(id.clone(), Scenario {
            name: name.to_string(),
            description: description.to_string(),
            result: None,
        });
        id
    }

    pub fn simulate(&self, id: String, _mode: &str) -> Result<(), String> {
        if self.scenarios.contains_key(&id) {
            Ok(())
        } else {
            Err(format!("scenario {} not found", id))
        }
    }

    pub fn scenario_count(&self) -> usize {
        self.scenarios.len()
    }

    pub fn has_scenario(&self, id: &str) -> bool {
        self.scenarios.contains_key(id)
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
