// RuneSocketing Implementation
// 5-slot rune configuration: Crimson / Indigo / Obsidian / Golden / Alabaster

use uniffi;
use std::sync::{Arc, RwLock};
use crate::neotrix::ffi::types::*;
use std::collections::HashMap;

struct RuneSocketingInner {
    runes: Vec<Rune>,
    runewords: Vec<Runeword>,
    configs: HashMap<String, SocketConfig>,
}

#[derive(Clone)]
#[derive(uniffi::Object)]
pub struct RuneSocketingImpl {
    inner: Arc<RwLock<RuneSocketingInner>>,
}

#[uniffi::export]
impl RuneSocketingImpl {
    #[uniffi::constructor]
    pub fn init() -> Result<Self, NeoTrixError> {
        let runes = vec![
            Rune { color: "Crimson".into(), name: "Crimson Flow".into(), description: "Data ingestion throughput +20%".into(), effects: vec![RuneEffect { target: "ingestion".into(), modifier: 0.2, condition: "always".into() }] },
            Rune { color: "Indigo".into(), name: "Indigo Transform".into(), description: "Transformation efficiency +25%".into(), effects: vec![RuneEffect { target: "transform".into(), modifier: 0.25, condition: "always".into() }] },
            Rune { color: "Obsidian".into(), name: "Obsidian Cache".into(), description: "Cache hit rate +30%".into(), effects: vec![RuneEffect { target: "cache".into(), modifier: 0.3, condition: "always".into() }] },
            Rune { color: "Golden".into(), name: "Golden Recovery".into(), description: "Error recovery rate +35%".into(), effects: vec![RuneEffect { target: "recovery".into(), modifier: 0.35, condition: "error".into() }] },
            Rune { color: "Alabaster".into(), name: "Alabaster Watch".into(), description: "Monitoring granularity +40%".into(), effects: vec![RuneEffect { target: "monitor".into(), modifier: 0.4, condition: "always".into() }] },
        ];
        let runewords = vec![
            Runeword { name: "Scry".into(), runes: vec!["Crimson".into(), "Indigo".into(), "Obsidian".into()], effect: "Complete ETL pipeline: ingest → transform → cache".into(), description: "Crimson + Indigo + Obsidian".into() },
            Runeword { name: "Aegis".into(), runes: vec!["Golden".into(), "Alabaster".into()], effect: "Self-healing monitor: recover + observe".into(), description: "Golden + Alabaster".into() },
            Runeword { name: "Oracle".into(), runes: vec!["Crimson".into(), "Indigo".into(), "Golden".into(), "Alabaster".into()], effect: "Resilient intelligence: full data → transform → recover → watch".into(), description: "All but Obsidian".into() },
            Runeword { name: "Genesis".into(), runes: vec!["Crimson".into(), "Indigo".into(), "Obsidian".into(), "Golden".into(), "Alabaster".into()], effect: "Complete runeword: all five runes active".into(), description: "All five runes".into() },
        ];
        Ok(Self {
            inner: Arc::new(RwLock::new(RuneSocketingInner {
                runes,
                runewords,
                configs: HashMap::new(),
            })),
        })
    }

    pub fn get_runes(&self) -> Vec<Rune> {
        self.inner.read().expect("ffi rwlock poisoned").runes.clone()
    }

    pub fn get_runewords(&self) -> Vec<Runeword> {
        self.inner.read().expect("ffi rwlock poisoned").runewords.clone()
    }

    pub fn configure_sockets(&self, config: SocketConfig) -> bool {
        self.inner.write().expect("ffi rwlock poisoned").configs.insert(config.module.clone(), config);
        true
    }

    pub fn get_module_config(&self, module: &str) -> Result<SocketConfig, NeoTrixError> {
        self.inner.read().expect("ffi rwlock poisoned").configs.get(module).cloned().ok_or(NeoTrixError::NotFound)
    }

    pub fn compute_runewords(&self, module: &str) -> Vec<Runeword> {
        let inner = self.inner.read().expect("ffi rwlock poisoned");
        let config = match inner.configs.get(module) {
            Some(c) => c,
            None => return Vec::new(),
        };
        let socketed: Vec<String> = config.sockets.values().cloned().collect();
        inner.runewords
            .iter()
            .filter(|rw| rw.runes.iter().all(|r| socketed.contains(r)))
            .cloned()
            .collect()
    }
}