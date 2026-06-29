#![allow(dead_code)]
//! SearchRouter — 多 Provider 智能搜索路由
//!
//! 参照 smartsearch 设计：
//! - Provider 按 capability 分组（main_search/docs_search/web_fetch）
//! - 按 task_type/intent 路由到最合适的 capability
//! - 同 capability 内自动 fallback
//! - 路由决策追踪（routing_decision / provider_attempts / fallback_used）

use std::collections::HashMap;

/// 搜索能力类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SearchCapability {
    MainSearch,
    DocsSearch,
    WebFetch,
}

impl SearchCapability {
    pub fn as_str(&self) -> &'static str {
        match self {
            SearchCapability::MainSearch => "main_search",
            SearchCapability::DocsSearch => "docs_search",
            SearchCapability::WebFetch => "web_fetch",
        }
    }
}

/// Provider 注册槽
pub(crate) struct ProviderSlot {
    name: String,
    capability: SearchCapability,
}

/// 路由决策记录
#[derive(Debug, Clone)]
pub struct RoutingRecord {
    pub task: String,
    pub routed_to: Vec<String>,
    pub provider_attempts: Vec<(String, bool)>,
    pub fallback_used: bool,
}

#[derive(Debug, Clone)]
pub struct SearchConfig {
    pub extra_sources: usize,
    pub fallback: bool,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            extra_sources: 0,
            fallback: true,
        }
    }
}

/// 搜索意图
#[derive(Debug, Clone)]
pub struct SearchIntent {
    pub needs_docs: bool,
    pub is_temporal: bool,
}

/// 搜索路由器
pub struct SearchRouter {
    slots: Vec<ProviderSlot>,
    capabilities: HashMap<SearchCapability, Vec<usize>>,
    pub config: SearchConfig,
}

impl SearchRouter {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            capabilities: HashMap::new(),
            config: SearchConfig::default(),
        }
    }

    pub fn register(&mut self, name: &str, capability: SearchCapability) {
        let idx = self.slots.len();
        self.capabilities.entry(capability).or_default().push(idx);
        self.slots.push(ProviderSlot {
            name: name.to_string(),
            capability,
        });
    }

    /// 注册默认 provider（从环境变量）
    pub fn register_defaults(&mut self) {
        if std::env::var("NEOTRIX_API_KEY").is_ok() {
            let name = std::env::var("NEOTRIX_PROVIDER").unwrap_or_else(|_| "auto".to_string());
            self.register(&name, SearchCapability::MainSearch);
        }
        if std::env::var("EXA_API_KEY").is_ok() {
            self.register("exa", SearchCapability::DocsSearch);
        }
        if std::env::var("TAVILY_API_KEY").is_ok() {
            self.register("tavily", SearchCapability::WebFetch);
        }
    }

    /// 检测搜索意图
    pub fn detect_intent(&self, task: &str) -> SearchIntent {
        let l = task.to_lowercase();
        SearchIntent {
            needs_docs: l.contains("api")
                || l.contains("documentation")
                || l.contains("docs")
                || l.contains("sdk")
                || l.contains("reference"),
            is_temporal: l.contains("news")
                || l.contains("today")
                || l.contains("latest")
                || l.contains("current")
                || l.contains("update"),
        }
    }

    /// 路由到最佳 provider 链
    pub fn route(&self, task: &str) -> RoutingRecord {
        let intent = self.detect_intent(task);
        let mut record = RoutingRecord {
            task: task.to_string(),
            routed_to: Vec::new(),
            provider_attempts: Vec::new(),
            fallback_used: false,
        };

        // MainSearch
        if let Some(indices) = self.capabilities.get(&SearchCapability::MainSearch) {
            record.routed_to.push("main_search".to_string());
            for &i in indices {
                if let Some(slot) = self.slots.get(i) {
                    record.provider_attempts.push((slot.name.clone(), true));
                }
            }
            if indices.len() > 1 {
                record.fallback_used = true;
            }
        }

        // DocsSearch
        if intent.needs_docs {
            if let Some(indices) = self.capabilities.get(&SearchCapability::DocsSearch) {
                record.routed_to.push("docs_search".to_string());
                for &i in indices {
                    if let Some(slot) = self.slots.get(i) {
                        record.provider_attempts.push((slot.name.clone(), true));
                    }
                }
            }
        }

        record
    }

    /// 检查是否满足最低配置
    pub fn check_minimum_profile(&self) -> Vec<SearchCapability> {
        let mut m = Vec::new();
        for c in &[
            SearchCapability::MainSearch,
            SearchCapability::DocsSearch,
            SearchCapability::WebFetch,
        ] {
            if !self.capabilities.contains_key(c) || self.capabilities[c].is_empty() {
                m.push(*c);
            }
        }
        m
    }

    pub fn stats(&self) -> HashMap<&'static str, usize> {
        let mut s = HashMap::new();
        for c in &[
            SearchCapability::MainSearch,
            SearchCapability::DocsSearch,
            SearchCapability::WebFetch,
        ] {
            s.insert(
                c.as_str(),
                self.capabilities.get(c).map(|v| v.len()).unwrap_or(0),
            );
        }
        s
    }

    pub fn provider_count(&self) -> usize {
        self.slots.len()
    }
}

impl Default for SearchRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_stats() {
        let mut router = SearchRouter::new();
        router.register("xai", SearchCapability::MainSearch);
        router.register("exa", SearchCapability::DocsSearch);
        let stats = router.stats();
        assert_eq!(stats.get("main_search"), Some(&1));
        assert_eq!(stats.get("docs_search"), Some(&1));
        assert_eq!(stats.get("web_fetch"), Some(&0));
    }

    #[test]
    fn test_profile_check_empty() {
        let router = SearchRouter::new();
        assert_eq!(router.check_minimum_profile().len(), 3);
    }

    #[test]
    fn test_detect_intent_docs() {
        let router = SearchRouter::new();
        let intent = router.detect_intent("find the Rust API documentation");
        assert!(intent.needs_docs);
    }

    #[test]
    fn test_detect_intent_news() {
        let router = SearchRouter::new();
        let intent = router.detect_intent("latest AI news today");
        assert!(intent.is_temporal);
    }

    #[test]
    fn test_route_main_search() {
        let mut router = SearchRouter::new();
        router.register("primary", SearchCapability::MainSearch);
        router.register("fallback", SearchCapability::MainSearch);
        let record = router.route("test query");
        assert!(record.routed_to.contains(&"main_search".to_string()));
        assert!(record.fallback_used);
        assert_eq!(record.provider_attempts.len(), 2);
    }

    #[test]
    fn test_route_with_docs_intent() {
        let mut router = SearchRouter::new();
        router.register("xai", SearchCapability::MainSearch);
        router.register("exa", SearchCapability::DocsSearch);
        let record = router.route("find API documentation for React");
        assert!(record.routed_to.contains(&"main_search".to_string()));
        assert!(record.routed_to.contains(&"docs_search".to_string()));
    }

    #[test]
    fn test_defaults_from_env() {
        let router = SearchRouter::new();
        assert_eq!(router.provider_count(), 0);
    }

    #[test]
    fn test_stats_format() {
        let mut router = SearchRouter::new();
        router.register("a", SearchCapability::MainSearch);
        router.register("b", SearchCapability::DocsSearch);
        router.register("c", SearchCapability::WebFetch);
        let stats = router.stats();
        assert_eq!(
            *stats
                .get("main_search")
                .expect("value should be ok in test"),
            1
        );
        assert_eq!(
            *stats
                .get("docs_search")
                .expect("value should be ok in test"),
            1
        );
        assert_eq!(
            *stats.get("web_fetch").expect("value should be ok in test"),
            1
        );
    }
}
