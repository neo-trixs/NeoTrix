//! AbsorptionRegistry — Cyclic Absorption Ecosystem
//!
//! Replaces singleton `AbsorberState` with per-plugin registration.
//! Each plugin calls `register_absorber()` during init, gets a unique ID,
//! and can later `unregister()` for clean unloading.
//!
//! Two-phase design:
//!   Phase 1 (init): plugins call register_absorber()
//!   Phase 2 (runtime): any subsystem calls trigger_absorption()
//!
//! # Example
//! ```
//! use crate::neotrix::l8_autonomic_impl::nt_mind_absorption_registry::*;
//! let id = register_absorber("my_plugin", &["knowledge"], my_absorber);
//! assert!(id.is_some());
//! let events = trigger_absorption("my_plugin").unwrap();
//! ```

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);
const MAX_ABSORBERS: usize = 1024;
const MAX_NAME_LEN: usize = 64;

#[derive(Debug, Clone)]
pub struct AbsorberInstance {
    pub id: u64,
    pub plugin_name: String,
    pub capabilities: Vec<String>,
}

pub trait CapabilityAbsorber: Send + Sync {
    fn absorb(&self, capability: &str, context: &str) -> Vec<AbsorptionEvent>;
    fn name(&self) -> &str;
}

#[derive(Debug, Clone)]
pub struct AbsorptionEvent {
    pub target: String,
    pub source: String,
    pub confidence: f64,
    pub summary: String,
}

#[derive(Debug, Clone)]
pub enum AbsorptionError {
    NameTooLong(String),
    AlreadyRegistered(String),
    TooManyAbsorbers,
    NotFound(String),
    AbsorptionFailed(String),
}

#[derive(Debug, Clone, Default)]
pub struct AbsorptionStats {
    pub total_absorptions: u64,
    pub total_events: u64,
    pub last_error: Option<String>,
}

struct InternalState {
    by_id: HashMap<u64, AbsorberInstance>,
    by_name: HashMap<String, u64>,
    absorbers: HashMap<u64, Arc<dyn CapabilityAbsorber + Send + Sync>>,
    stats: HashMap<u64, AbsorptionStats>,
}

impl InternalState {
    fn new() -> Self {
        Self {
            by_id: HashMap::new(),
            by_name: HashMap::new(),
            absorbers: HashMap::new(),
            stats: HashMap::new(),
        }
    }
}

static REGISTRY: std::sync::LazyLock<RwLock<InternalState>> =
    std::sync::LazyLock::new(|| RwLock::new(InternalState::new()));

pub fn register_absorber(
    plugin_name: &str,
    capabilities: &[&str],
    absorber: Arc<dyn CapabilityAbsorber + Send + Sync>,
) -> Result<u64, AbsorptionError> {
    if plugin_name.len() > MAX_NAME_LEN {
        return Err(AbsorptionError::NameTooLong(plugin_name.to_string()));
    }
    let mut state = REGISTRY.write().map_err(|e| {
        AbsorptionError::AbsorptionFailed(format!("Lock poisoned: {}", e))
    })?;
    if state.by_name.contains_key(plugin_name) {
        return Err(AbsorptionError::AlreadyRegistered(plugin_name.to_string()));
    }
    if state.by_id.len() >= MAX_ABSORBERS {
        return Err(AbsorptionError::TooManyAbsorbers);
    }
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let caps: Vec<String> = capabilities.iter().map(|s| s.to_string()).collect();
    state.by_id.insert(id, AbsorberInstance {
        id,
        plugin_name: plugin_name.to_string(),
        capabilities: caps,
    });
    state.by_name.insert(plugin_name.to_string(), id);
    state.absorbers.insert(id, absorber);
    state.stats.insert(id, AbsorptionStats::default());
    Ok(id)
}

pub fn unregister(plugin_name: &str) -> Option<AbsorberInstance> {
    let mut state = REGISTRY.write().ok()?;
    let id = state.by_name.remove(plugin_name)?;
    let instance = state.by_id.remove(&id)?;
    state.absorbers.remove(&id);
    state.stats.remove(&id);
    Some(instance)
}

pub fn trigger_absorption(
    plugin_name: &str,
    capability: &str,
    context: &str,
) -> Result<Vec<AbsorptionEvent>, AbsorptionError> {
    let state = REGISTRY.read().map_err(|e| {
        AbsorptionError::AbsorptionFailed(format!("Lock poisoned: {}", e))
    })?;
    let id = *state.by_name.get(plugin_name).ok_or_else(|| {
        AbsorptionError::NotFound(plugin_name.to_string())
    })?;
    let absorber = state.absorbers.get(&id).ok_or_else(|| {
        AbsorptionError::NotFound(plugin_name.to_string())
    })?;
    let events = absorber.absorb(capability, context);
    drop(state);

    if let Ok(mut state) = REGISTRY.write() {
        if let Some(stats) = state.stats.get_mut(&id) {
            stats.total_absorptions += 1;
            stats.total_events += events.len() as u64;
        }
    }
    Ok(events)
}

pub fn list_absorbers() -> Vec<AbsorberInstance> {
    REGISTRY.read()
        .map(|state| state.by_id.values().cloned().collect())
        .unwrap_or_default()
}

pub fn absorber_count() -> usize {
    REGISTRY.read().map(|state| state.by_id.len()).unwrap_or(0)
}

pub fn get_stats(plugin_name: &str) -> Option<AbsorptionStats> {
    let state = REGISTRY.read().ok()?;
    let id = state.by_name.get(plugin_name)?;
    state.stats.get(id).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestAbsorber;
    impl CapabilityAbsorber for TestAbsorber {
        fn absorb(&self, _capability: &str, _context: &str) -> Vec<AbsorptionEvent> {
            vec![AbsorptionEvent {
                target: "test_target".into(),
                source: "test_source".into(),
                confidence: 0.9,
                summary: "Test absorption".into(),
            }]
        }
        fn name(&self) -> &str { "test_absorber" }
    }

    #[test]
    fn test_register_and_trigger() {
        let id = register_absorber("test", &["knowledge"], Arc::new(TestAbsorber)).unwrap();
        assert!(id > 0);
        let events = trigger_absorption("test", "knowledge", "test context").unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].target, "test_target");
    }

    #[test]
    fn test_duplicate_detection() {
        let _ = register_absorber("dup_test", &["a"], Arc::new(TestAbsorber));
        let r2 = register_absorber("dup_test", &["b"], Arc::new(TestAbsorber));
        assert!(matches!(r2, Err(AbsorptionError::AlreadyRegistered(_))));
    }

    #[test]
    fn test_name_too_long() {
        let long_name = "a".repeat(65);
        let r = register_absorber(&long_name, &[], Arc::new(TestAbsorber));
        assert!(matches!(r, Err(AbsorptionError::NameTooLong(_))));
    }

    #[test]
    fn test_unregister() {
        let _ = register_absorber("unreg", &["x"], Arc::new(TestAbsorber));
        let instance = unregister("unreg");
        assert!(instance.is_some());
        assert_eq!(instance.unwrap().plugin_name, "unreg");
        let r = trigger_absorption("unreg", "x", "");
        assert!(matches!(r, Err(AbsorptionError::NotFound(_))));
    }

    #[test]
    fn test_list_absorbers() {
        let _ = register_absorber("list_a", &["a1"], Arc::new(TestAbsorber));
        let _ = register_absorber("list_b", &["b1"], Arc::new(TestAbsorber));
        let all = list_absorbers();
        assert!(all.iter().any(|a| a.plugin_name == "list_a"));
        assert!(all.iter().any(|a| a.plugin_name == "list_b"));
    }

    #[test]
    fn test_stats_tracking() {
        let _ = register_absorber("stat_test", &["s"], Arc::new(TestAbsorber));
        let _ = trigger_absorption("stat_test", "s", "ctx");
        let _ = trigger_absorption("stat_test", "s", "ctx2");
        let stats = get_stats("stat_test").unwrap();
        assert_eq!(stats.total_absorptions, 2);
        assert_eq!(stats.total_events, 2);
    }
}
