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
                a_s.partial_cmp(&b_s)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    // ⚠️ 2026-10-05 修正：原先**没有**名字兜底 ⇒ `error_rate` 并列时
                    // 胜者完全取决于 `engines` 的**注册顺序**。`engines` 是 `Vec`
                    // (不是 `HashMap`，故遍历序本身确定)，但注册顺序由调用方决定 ——
                // 一旦调用方从某个 `HashMap` 派生出注册序列，胜者就变成哈希序 ⇒
                // 跨进程漂移 ⇒ 同一查询在两次运行里可能路由到不同引擎。
                // ⇒ 补 `capability_id` 升序兜底，让胜者只由**内容**决定，
                //   与 `selection.rs::select_best` 的 `.then(na.cmp(nb))` 同一范式。
                    .then_with(|| a.capability_id().cmp(b.capability_id()))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l1_action::traits::{
        CapabilityCategory, ConstellationLevel, Document, L1Capability, SearchResult,
    };

    /// 8 个并列引擎的名字 (字典序 ⇒ 最后一个最大)。
    const TIED_IDS: [&str; 8] = [
        "eng-00", "eng-01", "eng-02", "eng-03", "eng-04", "eng-05", "eng-06", "eng-07",
    ];

    /// 最小 mock 搜索引擎 — 只为构造**同分**并列集, 不做真实检索。
    struct StubEngine {
        id: &'static str,
        error_rate: f64,
        healthy: bool,
    }

    impl StubEngine {
        fn new(id: &'static str, error_rate: f64) -> Self {
            Self {
                id,
                error_rate,
                healthy: true,
            }
        }
        fn unhealthy(id: &'static str, error_rate: f64) -> Self {
            Self {
                id,
                error_rate,
                healthy: false,
            }
        }
    }

    impl L1Capability for StubEngine {
        fn capability_id(&self) -> &str {
            self.id
        }
        fn category(&self) -> CapabilityCategory {
            CapabilityCategory::Data
        }
        fn constellation(&self) -> ConstellationLevel {
            ConstellationLevel::C1UnitTest
        }
        fn health_check(&self) -> CapabilityHealth {
            CapabilityHealth {
                healthy: self.healthy,
                latency_ms: None,
                error_rate: self.error_rate,
                last_check: 0,
                message: None,
            }
        }
        fn description(&self) -> &str {
            "stub"
        }
    }

    impl SearchEngineTrait for StubEngine {
        fn search(
            &self,
            _query: &str,
            _limit: usize,
        ) -> Result<Vec<SearchResult>, CapabilityError> {
            Ok(Vec::new())
        }
        fn index(&self, _doc: &Document) -> Result<(), CapabilityError> {
            Ok(())
        }
    }

    /// `SearchRegistry::optimal` 同分兜底: 胜者只由 `capability_id` 决定,
    /// 与**注册顺序**无关 (D13 确定性)。
    ///
    /// 原实现无兜底 ⇒ 同 `error_rate` 时 `max_by` 取**最后一个**最大值
    /// ⇒ 胜者 = 最后注册的引擎 ⇒ 逆序注册会翻转结果。
    #[test]
    fn optimal_tie_is_name_deterministic_not_registration_order() {
        let expected = TIED_IDS[TIED_IDS.len() - 1];

        let mut asc = SearchRegistry::new();
        for id in TIED_IDS {
            asc.register(Box::new(StubEngine::new(id, 0.25)));
        }
        let mut desc = SearchRegistry::new();
        for id in TIED_IDS.iter().rev() {
            desc.register(Box::new(StubEngine::new(*id, 0.25)));
        }
        assert_eq!(
            asc.optimal().map(|e| e.capability_id()),
            Some(expected),
            "正序注册的同分胜者不是名字最大者"
        );
        assert_eq!(
            desc.optimal().map(|e| e.capability_id()),
            Some(expected),
            "同分胜者随注册顺序漂移 ⇒ 顺序泄漏"
        );
    }

    /// 非并列时主判据 (error_rate 越低越好) 不得被兜底反转。
    #[test]
    fn optimal_primary_key_still_wins_over_name() {
        let mut reg = SearchRegistry::new();
        reg.register(Box::new(StubEngine::new("eng-bad", 0.9)));
        reg.register(Box::new(StubEngine::new("eng-good", 0.01)));
        reg.register(Box::new(StubEngine::new("eng-mid", 0.4)));
        assert_eq!(
            reg.optimal().map(|e| e.capability_id()),
            Some("eng-good")
        );
    }

    /// 不健康引擎必须被滤掉 (兜底不得把 unhealthy 拉回来), 全空 ⇒ None。
    #[test]
    fn optimal_filters_unhealthy() {
        let mut reg = SearchRegistry::new();
        reg.register(Box::new(StubEngine::unhealthy("eng-zzz", 0.0)));
        assert!(reg.optimal().is_none(), "只有不健康引擎 ⇒ None");

        reg.register(Box::new(StubEngine::new("eng-aaa", 0.5)));
        assert_eq!(
            reg.optimal().map(|e| e.capability_id()),
            Some("eng-aaa"),
            "不健康的高名字引擎不得胜出"
        );

        let empty = SearchRegistry::new();
        assert!(empty.optimal().is_none(), "无引擎 ⇒ None");
    }
}
