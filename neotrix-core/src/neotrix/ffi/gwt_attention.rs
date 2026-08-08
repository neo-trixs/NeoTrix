// GWT Attention Router Implementation
// Global Workspace Theory — resonance-based attention routing

use uniffi;
use std::sync::{Arc, RwLock};
use crate::neotrix::ffi::types::*;
use std::collections::HashMap;

struct GWTAttentionRouterInner {
    modules: HashMap<String, Vec<String>>,
    thresholds: HashMap<String, f32>,
    workspace: WorkspaceState,
}

#[derive(Clone)]
#[derive(uniffi::Object)]
pub struct GWTAttentionRouterImpl {
    inner: Arc<RwLock<GWTAttentionRouterInner>>,
}

#[uniffi::export]
impl GWTAttentionRouterImpl {
    #[uniffi::constructor]
    pub fn init(module_names: Vec<String>) -> Result<Self, NeoTrixError> {
        let mut modules = HashMap::new();
        let mut thresholds = HashMap::new();
        for name in module_names {
            modules.insert(name.clone(), Vec::new());
            thresholds.insert(name, 0.3);
        }
        Ok(Self {
            inner: Arc::new(RwLock::new(GWTAttentionRouterInner {
                modules,
                thresholds,
                workspace: WorkspaceState {
                    active_signals: Vec::new(),
                    broadcast_history: Vec::new(),
                    resonance_map: HashMap::new(),
                },
            })),
        })
    }

    pub fn submit_signal(&self, signal: AttentionSignal) -> RoutingResponse {
        let mut inner = self.inner.write().unwrap();
        let mut resonance_scores = HashMap::new();
        let mut recipients = Vec::new();

        for (module, keywords) in &inner.modules {
            let resonance = compute_resonance(&signal.content, keywords, signal.salience);
            resonance_scores.insert(module.clone(), resonance);
            let threshold = inner.thresholds.get(module).copied().unwrap_or(0.3);
            if resonance >= threshold {
                recipients.push(module.clone());
            }
        }

        let routed = !recipients.is_empty();
        let event = BroadcastEvent {
            signal: signal.clone(),
            recipients: recipients.clone(),
            resonance: resonance_scores.values().copied().fold(0.0, f32::max),
            timestamp: now_ms(),
        };

        inner.workspace.active_signals.push(signal.clone());
        if inner.workspace.active_signals.len() > 50 {
            inner.workspace.active_signals.remove(0);
        }
        inner.workspace.broadcast_history.push(event.clone());
        if inner.workspace.broadcast_history.len() > 100 {
            inner.workspace.broadcast_history.remove(0);
        }
        inner.workspace.resonance_map = resonance_scores.clone();

        RoutingResponse {
            routed,
            resonance_scores,
            broadcast_event: event,
        }
    }

    pub fn get_workspace_state(&self) -> WorkspaceState {
        self.inner.read().unwrap().workspace.clone()
    }

    pub fn register_module(&self, name: &str, interest_keywords: Vec<String>) -> bool {
        self.inner.write().unwrap().modules.insert(name.to_string(), interest_keywords);
        self.inner.write().unwrap().thresholds.entry(name.to_string()).or_insert(0.3);
        true
    }

    pub fn unregister_module(&self, name: &str) -> bool {
        self.inner.write().unwrap().modules.remove(name).is_some()
    }

    pub fn set_salience_threshold(&self, module: &str, threshold: f32) -> bool {
        let mut inner = self.inner.write().unwrap();
        if inner.modules.contains_key(module) {
            inner.thresholds.insert(module.to_string(), threshold);
            true
        } else {
            false
        }
    }

    pub fn get_resonance(&self, module_a: &str, module_b: &str) -> f32 {
        let inner = self.inner.read().unwrap();
        let kw_a = inner.modules.get(module_a).cloned().unwrap_or_default();
        let kw_b = inner.modules.get(module_b).cloned().unwrap_or_default();
        let overlap = kw_a.iter().filter(|k| kw_b.contains(k)).count();
        if kw_a.is_empty() || kw_b.is_empty() {
            0.0
        } else {
            overlap as f32 / kw_a.len().max(kw_b.len()) as f32
        }
    }
}

fn compute_resonance(content: &str, keywords: &[String], salience: f32) -> f32 {
    if keywords.is_empty() {
        return salience * 0.5;
    }
    let lower = content.to_lowercase();
    let hits = keywords.iter().filter(|k| lower.contains(&k.to_lowercase())).count();
    let keyword_score = hits as f32 / keywords.len() as f32;
    (keyword_score * 0.7 + salience * 0.3).clamp(0.0, 1.0)
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}