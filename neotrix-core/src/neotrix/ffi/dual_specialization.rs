// DualSpecialization Implementation
// Weapon Set I/II switching: acquisition (CORE+WORLD) vs evolution (CORE+MIND)

use uniffi;
use std::sync::{Arc, RwLock};
use crate::neotrix::ffi::types::*;

struct DualSpecializationInner {
    state: SpecializationState,
}

#[derive(Clone)]
#[derive(uniffi::Object)]
pub struct DualSpecializationImpl {
    inner: Arc<RwLock<DualSpecializationInner>>,
}

#[uniffi::export]
impl DualSpecializationImpl {
    #[uniffi::constructor]
    pub fn init() -> Result<Self, NeoTrixError> {
        Ok(Self {
            inner: Arc::new(RwLock::new(DualSpecializationInner {
                state: SpecializationState {
                    current_set: 1,
                    sets: vec![
                        WeaponSet {
                            set_id: 1,
                            name: "Void Reaper".into(),
                            primary_domain: "NT-CORE".into(),
                            secondary_domain: "NT-WORLD".into(),
                            active_skills: vec!["E8 Reasoning".into(), "Pattern Radar".into(), "Sensor Fusion".into()],
                            attention_mode: "acquisition".into(),
                        },
                        WeaponSet {
                            set_id: 2,
                            name: "Ascendant Forge".into(),
                            primary_domain: "NT-CORE".into(),
                            secondary_domain: "NT-MIND".into(),
                            active_skills: vec!["E8 Reasoning".into(), "Pattern Extractor".into(), "Skill Crystallizer".into()],
                            attention_mode: "evolution".into(),
                        },
                    ],
                    switch_cooldown_ms: 5_000,
                    last_switch: 0,
                },
            })),
        })
    }

    pub fn get_state(&self) -> SpecializationState {
        self.inner.read().unwrap().state.clone()
    }

    pub fn switch_set(&self, set_id: u8) -> Result<SpecializationState, NeoTrixError> {
        let mut inner = self.inner.write().unwrap();
        let now = now_ms();
        if now - inner.state.last_switch < inner.state.switch_cooldown_ms as i64 {
            return Err(NeoTrixError::OperationFailed);
        }
        if !inner.state.sets.iter().any(|s| s.set_id == set_id) {
            return Err(NeoTrixError::NotFound);
        }
        inner.state.current_set = set_id;
        inner.state.last_switch = now;
        Ok(inner.state.clone())
    }

    pub fn configure_set(&self, set_id: u8, config: WeaponSet) -> Result<WeaponSet, NeoTrixError> {
        let mut inner = self.inner.write().unwrap();
        let set = inner.state.sets.iter_mut().find(|s| s.set_id == set_id).ok_or(NeoTrixError::NotFound)?;
        *set = config;
        Ok(set.clone())
    }

    pub fn recommend_set(&self, task_type: &str) -> u8 {
        let inner = self.inner.read().unwrap();
        match task_type {
            "acquisition" | "crawling" | "research" | "exploration" => 1,
            "evolution" | "learning" | "distillation" | "absorption" => 2,
            _ => inner.state.current_set,
        }
    }
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}