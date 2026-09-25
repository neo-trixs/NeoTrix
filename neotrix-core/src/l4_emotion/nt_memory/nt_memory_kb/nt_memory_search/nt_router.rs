use crate::l1_action::traits::{
    CapabilityError, CapabilityHealth, SearchEngine as SearchEngineTrait,
    SearchResult as UnifiedSearchResult,
};

// ── Registry + Router + Bridge (moved from nt_memory_search.rs, pure move) ──

// ════════════════════════════════════════════════════════════════
// Registry + Router + Bridge
// ════════════════════════════════════════════════════════════════

/// 搜索能力注册中心
pub struct SearchRegistry {
    engines: Vec<Box<dyn SearchEngineTrait>>,
}

impl Default for SearchRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchRegistry {
    pub fn new() -> Self {
        Self {
            engines: Vec::new(),
        }
    }
    pub fn register(&mut self, engine: Box<dyn SearchEngineTrait>) {
        self.engines.push(engine);
    }
    pub fn get(&self, id: &str) -> Option<&dyn SearchEngineTrait> {
        self.engines
            .iter()
            .find(|e| e.capability_id() == id)
            .map(|e| e.as_ref())
    }
    pub fn health_check_all(&self) -> Vec<(String, CapabilityHealth)> {
        self.engines
            .iter()
            .map(|e| (e.capability_id().to_string(), e.health_check()))
            .collect()
    }
    pub fn optimal(&self) -> Option<&dyn SearchEngineTrait> {
        self.engines
            .iter()
            .filter(|e| e.health_check().healthy)
            .max_by(|a, b| {
                let a_s = 1.0 - a.health_check().error_rate;
                let b_s = 1.0 - b.health_check().error_rate;
                a_s.partial_cmp(&b_s).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|e| e.as_ref())
    }
}

/// 搜索路由器 — 按查询类型选择最佳引擎
pub struct SearchRouter {
    registry: SearchRegistry,
}

impl SearchRouter {
    pub fn new(registry: SearchRegistry) -> Self {
        Self { registry }
    }
    pub fn route(&self, _query: &str) -> Option<&dyn SearchEngineTrait> {
        self.registry.optimal()
    }
    pub fn search(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<UnifiedSearchResult>, CapabilityError> {
        self.registry
            .optimal()
            .ok_or_else(|| CapabilityError::NotAvailable("No search engine".into()))?
            .search(query, limit)
    }
}

/// 搜索桥接 — L5 领域技能 → SearchRouter
pub struct SearchBridge {
    router: SearchRouter,
}

impl SearchBridge {
    pub fn new(router: SearchRouter) -> Self {
        Self { router }
    }
    pub fn search(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<UnifiedSearchResult>, CapabilityError> {
        self.router.search(query, limit)
    }
}
