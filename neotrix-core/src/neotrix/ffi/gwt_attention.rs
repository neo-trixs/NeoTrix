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
    /// Cost-aware thinking budget (tokens). Cheap tasks get small budgets, expensive tasks get large budgets.
    /// Modulates salience computation: higher budget → stronger salience boost for high-complexity signals.
    thinking_budget: u32,
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
                    thinking_budget: 4096,
                },
                thinking_budget: 4096,
            })),
        })
    }

    pub fn submit_signal(&self, signal: AttentionSignal) -> RoutingResponse {
        let mut inner = self.inner.write().expect("ffi rwlock poisoned");
        let budget = inner.thinking_budget;
        let mut resonance_scores = HashMap::new();
        let mut recipients = Vec::new();

        for (module, keywords) in &inner.modules {
            let resonance = compute_resonance(&signal.content, keywords, signal.salience, budget);
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
        self.inner.read().expect("ffi rwlock poisoned").workspace.clone()
    }

    pub fn register_module(&self, name: &str, interest_keywords: Vec<String>) -> bool {
        self.inner.write().expect("ffi rwlock poisoned").modules.insert(name.to_string(), interest_keywords);
        self.inner.write().expect("ffi rwlock poisoned").thresholds.entry(name.to_string()).or_insert(0.3);
        true
    }

    pub fn unregister_module(&self, name: &str) -> bool {
        self.inner.write().expect("ffi rwlock poisoned").modules.remove(name).is_some()
    }

    pub fn set_salience_threshold(&self, module: &str, threshold: f32) -> bool {
        let mut inner = self.inner.write().expect("ffi rwlock poisoned");
        if inner.modules.contains_key(module) {
            inner.thresholds.insert(module.to_string(), threshold);
            true
        } else {
            false
        }
    }

    pub fn get_resonance(&self, module_a: &str, module_b: &str) -> f32 {
        let inner = self.inner.read().expect("ffi rwlock poisoned");
        let kw_a = inner.modules.get(module_a).cloned().unwrap_or_default();
        let kw_b = inner.modules.get(module_b).cloned().unwrap_or_default();
        let overlap = kw_a.iter().filter(|k| kw_b.contains(k)).count();
        if kw_a.is_empty() || kw_b.is_empty() {
            0.0
        } else {
            overlap as f32 / kw_a.len().max(kw_b.len()) as f32
        }
    }

    /// Set the thinking budget (tokens) for cost-aware salience modulation.
    /// Cheap tasks: 1024–2048 tokens. Expensive tasks: 8192–16384 tokens.
    pub fn set_thinking_budget(&self, budget: u32) {
        let mut inner = self.inner.write().expect("ffi rwlock poisoned");
        inner.thinking_budget = budget;
        inner.workspace.thinking_budget = budget;
    }

    /// Get the current thinking budget.
    pub fn get_thinking_budget(&self) -> u32 {
        self.inner.read().expect("ffi rwlock poisoned").thinking_budget
    }
}

/// Compute resonance with cost-aware thinking budget modulation.
/// Higher budgets amplify salience for complex signals (multi-keyword hits),
/// while low budgets suppress deep resonance to save tokens.
fn compute_resonance(content: &str, keywords: &[String], salience: f32, budget: u32) -> f32 {
    if keywords.is_empty() {
        return salience * 0.5;
    }
    let lower = content.to_lowercase();
    let hits = keywords.iter().filter(|k| lower.contains(&k.to_lowercase())).count();
    let keyword_score = hits as f32 / keywords.len() as f32;

    // Budget modulation: scale factor ∈ [0.5, 1.5] based on budget tier
    //   budget ≤ 2048  → 0.5 (cheap task: suppress deep resonance)
    //   budget ≤ 8192  → 1.0 (normal task: neutral)
    //   budget > 8192  → 1.5 (expensive task: amplify multi-hit resonance)
    let budget_factor = if budget <= 2048 {
        0.5
    } else if budget <= 8192 {
        1.0
    } else {
        1.5
    };

    (keyword_score * 0.7 * budget_factor + salience * 0.3).clamp(0.0, 1.0)
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signal(source: &str, content: &str, salience: f32) -> AttentionSignal {
        AttentionSignal {
            source_module: source.into(),
            content: content.into(),
            salience,
            timestamp: now_ms(),
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_resonance_routing_broadcasts_to_matching_modules() {
        // GWT 竞争-广播 (Goyal et al., arXiv:2103.01197): 信号按兴趣关键词
        // 共振路由到匹配模块 — 内容广播给所有共振超阈值的模块。
        let router = GWTAttentionRouterImpl::init(
            vec!["NT-WORLD".into(), "NT-MEMORY".into(), "NT-SHIELD".into()],
        ).unwrap();
        router.register_module("NT-WORLD", vec!["crawl".into(), "fetch".into(), "parse".into()]);
        router.register_module("NT-MEMORY", vec!["store".into(), "embed".into(), "retrieve".into()]);
        router.register_module("NT-SHIELD", vec!["threat".into(), "block".into(), "proxy".into()]);

        let resp = router.submit_signal(signal("NT-CORE", "crawl new pages and store embeddings", 0.8));
        assert!(resp.routed, "信号应被路由");
        let recips = &resp.broadcast_event.recipients;
        assert!(recips.contains(&"NT-WORLD".to_string()), "NT-WORLD 应接收: {recips:?}");
        assert!(recips.contains(&"NT-MEMORY".to_string()), "NT-MEMORY 应接收: {recips:?}");
        assert!(!recips.contains(&"NT-SHIELD".to_string()), "NT-SHIELD 不应接收: {recips:?}");
    }

    #[test]
    fn test_salience_threshold_gates_broadcast() {
        // 阈值门控: 低 salience 信号不触发低兴趣模块 (注意力稀缺性)
        let router = GWTAttentionRouterImpl::init(vec!["NT-WORLD".into()]).unwrap();
        router.register_module("NT-WORLD", vec!["crawl".into()]);
        // 高 salience + 关键词命中 → 路由
        let hi = router.submit_signal(signal("NT-CORE", "crawl the web", 0.9));
        assert!(hi.routed);
        // 低 salience + 无关键词 → 不路由
        let lo = router.submit_signal(signal("NT-CORE", "unrelated noise", 0.1));
        assert!(!lo.routed, "低 salience 无关键词不应路由");
    }

    #[test]
    fn test_workspace_broadcast_history_capped() {
        // 工作空间广播历史有界 (容量瓶颈 — GWT 核心属性)
        let router = GWTAttentionRouterImpl::init(vec!["NT-MEMORY".into()]).unwrap();
        router.register_module("NT-MEMORY", vec!["store".into()]);
        for i in 0..120 {
            router.submit_signal(signal("NT-CORE", &format!("store item {i}"), 0.5));
        }
        let ws = router.get_workspace_state();
        assert!(ws.broadcast_history.len() <= 100, "广播历史应封顶 100: {}", ws.broadcast_history.len());
        assert!(ws.active_signals.len() <= 50, "活跃信号应封顶 50: {}", ws.active_signals.len());
    }

    #[test]
    fn test_thinking_budget_modulates_resonance() {
        // Cost-Aware Routing (A1): budget modulates resonance strength.
        // Low budget → suppressed resonance; high budget → amplified resonance.
        let router = GWTAttentionRouterImpl::init(vec!["NT-WORLD".into()]).unwrap();
        router.register_module("NT-WORLD", vec!["crawl".into(), "fetch".into(), "parse".into()]);

        // Low budget (cheap task)
        router.set_thinking_budget(1024);
        let lo = router.submit_signal(signal("NT-CORE", "crawl and parse", 0.5));
        let lo_score = lo.resonance_scores.get("NT-WORLD").copied().unwrap_or(0.0);

        // High budget (expensive task)
        router.set_thinking_budget(16384);
        let hi = router.submit_signal(signal("NT-CORE", "crawl and parse", 0.5));
        let hi_score = hi.resonance_scores.get("NT-WORLD").copied().unwrap_or(0.0);

        assert!(hi_score > lo_score, "高预算应放大谐振: lo={lo_score}, hi={hi_score}");
        assert_eq!(router.get_thinking_budget(), 16384);
    }
}