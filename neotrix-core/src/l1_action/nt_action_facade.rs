#![forbid(unsafe_code)]

//! L1 Action Facade — 行动层唯一对外门面
//!
//! Platform shells and upper layers only talk to this facade.
//! Internal modules (`nt_memory`, `nt_act`, `nt_io`) are not directly accessible.

use std::sync::Arc;

use crate::l1_action::nt_act::async_tool_executor::AsyncToolExecutor;
use crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase;
use crate::l1_action::traits::LlmRouter;

// ════════════════════════════════════════════════════════════════
// Public types
// ════════════════════════════════════════════════════════════════

/// A single search hit returned by the facade.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub title: String,
    pub score: f64,
    pub snippet: String,
}

/// Health snapshot for the entire L1 Action layer.
#[derive(Debug, Clone)]
pub struct LayerHealth {
    /// Aggregate health score in `[0.0, 1.0]`.
    pub score: f64,
    /// Names of sub-modules and their status.
    pub modules: Vec<String>,
}

/// Errors that can surface through the facade boundary.
#[derive(Debug)]
pub enum FacadeError {
    /// The KB search path failed.
    SearchFailed(String),
    /// The KB store path failed.
    StoreFailed(String),
    /// The facade has not been initialized with valid internals.
    NotInitialized,
}

impl std::fmt::Display for FacadeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SearchFailed(msg) => write!(f, "search failed: {msg}"),
            Self::StoreFailed(msg) => write!(f, "store failed: {msg}"),
            Self::NotInitialized => write!(f, "facade not initialized"),
        }
    }
}

impl std::error::Error for FacadeError {}

// ════════════════════════════════════════════════════════════════
// Configuration (internal — not exposed in the public API)
// ════════════════════════════════════════════════════════════════

/// Builder input for `ActionFacade`.
pub struct ActionFacadeConfig {
    pub kb: Arc<KnowledgeBase>,
    pub tool_executor: Arc<AsyncToolExecutor>,
    pub llm_router: Arc<dyn LlmRouter>,
}

// ════════════════════════════════════════════════════════════════
// ActionFacade
// ════════════════════════════════════════════════════════════════

/// The sole facade for L1 Action layer access.
///
/// Platform shells and upper layers talk **only** to this struct.
/// Internal modules (`nt_memory`, `nt_act`, `nt_io`) remain hidden.
pub struct ActionFacade {
    kb: Option<Arc<KnowledgeBase>>,
    tool_executor: Option<Arc<AsyncToolExecutor>>,
    llm_router: Option<Arc<dyn LlmRouter>>,
}

impl ActionFacade {
    /// Construct an uninitialized facade.
    ///
    /// Call `init()` with real components before using any method.
    pub fn new() -> Self {
        Self {
            kb: None,
            tool_executor: None,
            llm_router: None,
        }
    }

    /// Inject real internal components.
    pub fn init(&mut self, config: ActionFacadeConfig) {
        self.kb = Some(config.kb);
        self.tool_executor = Some(config.tool_executor);
        self.llm_router = Some(config.llm_router);
    }

    // ── Search ──────────────────────────────────────────────────

    /// Search the knowledge base.
    pub fn search(&self, query: &str) -> Result<Vec<SearchResult>, FacadeError> {
        let kb = self.kb.as_ref().ok_or(FacadeError::NotInitialized)?;

        let conn = kb
            .raw_conn()
            .map_err(|e| FacadeError::SearchFailed(format!("KB lock: {e}")))?;

        let mut results = Vec::new();

        // BM25 path (always available)
        kb.rebuild_bm25();
        if let Ok(bm25_guard) = kb.bm25.read() {
            if let Some(ref bm25) = *bm25_guard {
                for hit in bm25.search(query, 10) {
                    results.push(SearchResult {
                        title: hit.id.clone(),
                        score: hit.score,
                        snippet: String::new(),
                    });
                }
            }
        }

        // Fallback to LIKE search
        if results.is_empty() {
            let like_pattern = format!("%{query}%");
            let mut stmt = conn
                .prepare(
                    "SELECT title, COALESCE(summary, '') FROM nodes
                     WHERE title LIKE ?1 OR content LIKE ?1
                     ORDER BY updated_at DESC LIMIT 10",
                )
                .map_err(|e| FacadeError::SearchFailed(format!("stmt: {e}")))?;

            let rows = stmt
                .query_map([&like_pattern], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
                .map_err(|e| FacadeError::SearchFailed(format!("query: {e}")))?;

            for row in rows.flatten() {
                results.push(SearchResult {
                    title: row.0,
                    score: 1.0,
                    snippet: row.1,
                });
            }
        }

        Ok(results)
    }

    // ── Store ───────────────────────────────────────────────────

    /// Store a knowledge node. Returns the node ID on success.
    pub fn store(
        &self,
        title: &str,
        node_type: &str,
        summary: &str,
    ) -> Result<String, FacadeError> {
        let kb = self.kb.as_ref().ok_or(FacadeError::NotInitialized)?;

        let node_id = kb
            .insert_or_get_node(title, node_type, Some(summary), None, None)
            .map_err(|e| FacadeError::StoreFailed(format!("{e}")))?;

        Ok(node_id)
    }

    // ── Health ─────────────────────────────────────────────────

    /// Return a health snapshot for the L1 layer.
    pub fn health(&self) -> LayerHealth {
        let mut modules = Vec::new();
        let mut healthy_count = 0u32;
        let mut total = 0u32;

        // nt_memory
        total += 1;
        if self.kb.as_ref().map_or(false, |kb| kb.raw_conn().is_ok()) {
            modules.push("nt_memory:ok".into());
            healthy_count += 1;
        } else {
            modules.push("nt_memory:down".into());
        }

        // nt_act
        total += 1;
        if self.tool_executor.is_some() {
            modules.push("nt_act:ok".into());
            healthy_count += 1;
        } else {
            modules.push("nt_act:down".into());
        }

        // nt_io
        total += 1;
        if self
            .llm_router
            .as_ref()
            .map_or(false, |r| r.health_check().healthy)
        {
            modules.push("nt_io:ok".into());
            healthy_count += 1;
        } else {
            modules.push("nt_io:down".into());
        }

        let score = if total == 0 {
            0.0
        } else {
            healthy_count as f64 / total as f64
        };

        LayerHealth { score, modules }
    }
}

impl Default for ActionFacade {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ActionFacade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActionFacade")
            .field("initialized", &self.kb.is_some())
            .finish()
    }
}

// ════════════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l1_action::nt_act::async_tool_executor::AsyncToolExecutor;
    use crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase;
    use crate::l1_action::traits::{
        CapabilityCategory, CapabilityHealth, ConstellationLevel, L1Capability, LlmRequest,
        LlmRoute,
    };

    struct DummyRouter;

    impl L1Capability for DummyRouter {
        fn capability_id(&self) -> &str {
            "test.router"
        }
        fn category(&self) -> CapabilityCategory {
            CapabilityCategory::Cognition
        }
        fn constellation(&self) -> ConstellationLevel {
            ConstellationLevel::C0Compiled
        }
        fn health_check(&self) -> CapabilityHealth {
            CapabilityHealth::default()
        }
        fn description(&self) -> &str {
            "Dummy router for tests"
        }
    }

    impl LlmRouter for DummyRouter {
        fn route(
            &self,
            _request: &LlmRequest,
        ) -> Result<LlmRoute, crate::l1_action::traits::CapabilityError> {
            Ok(LlmRoute {
                provider: "test".into(),
                model: "dummy".into(),
                estimated_cost: 0.0,
            })
        }
        fn providers(&self) -> Vec<String> {
            vec!["test".into()]
        }
    }

    fn test_facade() -> ActionFacade {
        let kb = Arc::new(
            KnowledgeBase::open(Some(std::path::PathBuf::from(":memory:"))).expect("in-memory KB"),
        );
        let executor = Arc::new(AsyncToolExecutor::new());
        let router = Arc::new(DummyRouter);

        let mut facade = ActionFacade::new();
        facade.init(ActionFacadeConfig {
            kb,
            tool_executor: executor,
            llm_router: router,
        });
        facade
    }

    #[test]
    fn new_is_uninitialized() {
        let f = ActionFacade::new();
        assert!(f.kb.is_none());
        assert!(f.health().score < 0.5);
    }

    #[test]
    fn search_empty_kb_returns_empty() {
        let facade = test_facade();
        let results = facade.search("anything").expect("search should not fail");
        assert!(results.is_empty());
    }

    #[test]
    fn store_and_search() {
        let facade = test_facade();
        let id = facade
            .store("Rust Facade Pattern", "concept", "L1 action layer facade")
            .expect("store");
        assert!(!id.is_empty());

        let results = facade.search("Facade").expect("search");
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.title == "Rust Facade Pattern"));
    }

    #[test]
    fn health_all_healthy() {
        let facade = test_facade();
        let h = facade.health();
        assert!((h.score - 1.0).abs() < f64::EPSILON);
        assert_eq!(h.modules.len(), 3);
        assert!(h.modules.iter().all(|m| m.ends_with(":ok")));
    }

    #[test]
    fn uninitialized_returns_not_initialized() {
        let facade = ActionFacade::new();
        assert!(matches!(
            facade.search("q"),
            Err(FacadeError::NotInitialized)
        ));
        assert!(matches!(
            facade.store("t", "n", "s"),
            Err(FacadeError::NotInitialized)
        ));
    }

    #[test]
    fn facade_error_display() {
        let e = FacadeError::SearchFailed("boom".into());
        assert_eq!(e.to_string(), "search failed: boom");

        let e = FacadeError::StoreFailed("oops".into());
        assert_eq!(e.to_string(), "store failed: oops");

        let e = FacadeError::NotInitialized;
        assert_eq!(e.to_string(), "facade not initialized");
    }

    #[test]
    fn search_result_is_clone() {
        let sr = SearchResult {
            title: "t".into(),
            score: 0.5,
            snippet: "s".into(),
        };
        let sr2 = sr.clone();
        assert_eq!(sr.title, sr2.title);
    }
}
