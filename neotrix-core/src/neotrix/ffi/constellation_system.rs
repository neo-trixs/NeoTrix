// ConstellationSystem Implementation
// C0-C6 module maturity ladder

use uniffi;
use std::sync::{Arc, RwLock};
use crate::neotrix::ffi::types::*;
use std::collections::HashMap;

struct ConstellationSystemInner {
    states: HashMap<String, ConstellationState>,
}

#[derive(Clone)]
#[derive(uniffi::Object)]
pub struct ConstellationSystemImpl {
    inner: Arc<RwLock<ConstellationSystemInner>>,
}

#[uniffi::export]
impl ConstellationSystemImpl {
    #[uniffi::constructor]
    pub fn init() -> Result<Self, NeoTrixError> {
        let mut states = HashMap::new();
        let modules = ["NT-CORE", "NT-MIND", "NT-MEMORY", "NT-WORLD", "NT-ACT", "NT-IO", "NT-SHIELD"];
        for (i, module) in modules.iter().enumerate() {
            let level = if i == 0 { 4 } else { 3 };
            states.insert(module.to_string(), ConstellationState {
                module: module.to_string(),
                level,
                requirements: build_requirements(level),
                progress: level as f32 / 6.0,
                unlocked_features: build_features(level),
            });
        }
        Ok(Self {
            inner: Arc::new(RwLock::new(ConstellationSystemInner { states })),
        })
    }

    pub fn get_state(&self, module: &str) -> Result<ConstellationState, NeoTrixError> {
        self.inner.read().unwrap().states.get(module).cloned().ok_or(NeoTrixError::NotFound)
    }

    pub fn get_all_states(&self) -> Vec<ConstellationState> {
        self.inner.read().unwrap().states.values().cloned().collect()
    }

    pub fn can_upgrade(&self, module: &str) -> bool {
        self.inner.read().unwrap().states.get(module).map(|s| s.level < 6).unwrap_or(false)
    }

    pub fn check_upgrade(&self, module: &str) -> Result<ConstellationState, NeoTrixError> {
        let mut inner = self.inner.write().unwrap();
        let state = inner.states.get_mut(module).ok_or(NeoTrixError::NotFound)?;
        if state.level < 6 {
            let all_satisfied = state.requirements.iter().all(|r| r.satisfied);
            if all_satisfied {
                state.level += 1;
                state.progress = state.level as f32 / 6.0;
                state.unlocked_features = build_features(state.level);
                state.requirements = build_requirements(state.level);
            }
        }
        Ok(state.clone())
    }

    pub fn get_upgrade_path(&self, module: &str) -> Result<Vec<ConstellationRequirement>, NeoTrixError> {
        self.inner.read().unwrap().states.get(module).map(|s| s.requirements.clone()).ok_or(NeoTrixError::NotFound)
    }
}

fn build_requirements(level: u8) -> Vec<ConstellationRequirement> {
    let types = ["compiles", "unit_tests", "integration_tests", "benchmarked", "pipeline_integrated", "self_healing"];
    types.iter().enumerate().map(|(i, t)| ConstellationRequirement {
        requirement_type: t.to_string(),
        description: format!("C{} requirement: {}", i, t),
        satisfied: (i as u8) < level,
        progress: if (i as u8) < level { 1.0 } else { 0.0 },
    }).collect()
}

fn build_features(level: u8) -> Vec<String> {
    let features = ["Compiles", "Unit Tests Passing", "Integration Tests Passing", "Benchmarks Met", "Pipeline Integration", "Self-Healing"];
    features.iter().enumerate().filter(|(i, _)| (*i as u8) < level).map(|(_, f)| f.to_string()).collect()
}