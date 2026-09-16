#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Dynamic task routing with rule-based, load-balanced, priority-based, and fallback strategies.
pub struct TradeRouter {
    routes: Vec<RouteRule>,
    worker_loads: HashMap<String, AtomicUsize>,
    fallback_worker: String,
    stats: RouterStatsInner,
}

pub struct RouteRule {
    pub name: String,
    pub condition: RouteCondition,
    pub target_worker: String,
    pub priority: u32,
}

pub enum RouteCondition {
    TaskType(String),
    PayloadField {
        field: String,
        value: serde_json::Value,
    },
    Composite(Vec<RouteCondition>),
    Always,
}

#[derive(Default)]
struct RouterStatsInner {
    total_routed: AtomicUsize,
    by_worker: HashMap<String, AtomicUsize>,
    by_rule: HashMap<String, AtomicUsize>,
    fallback_count: AtomicUsize,
}

pub struct RouterStats {
    pub total_routed: u64,
    pub by_worker: HashMap<String, u64>,
    pub by_rule: HashMap<String, u64>,
    pub fallback_count: u64,
}

impl TradeRouter {
    pub fn new(fallback_worker: &str) -> Self {
        let mut worker_loads = HashMap::new();
        worker_loads.insert(
            fallback_worker.to_string(),
            AtomicUsize::new(0),
        );

        Self {
            routes: Vec::new(),
            worker_loads,
            fallback_worker: fallback_worker.to_string(),
            stats: RouterStatsInner::default(),
        }
    }

    pub fn add_rule(&mut self, rule: RouteRule) {
        self.worker_loads
            .entry(rule.target_worker.clone())
            .or_insert_with(|| AtomicUsize::new(0));
        self.routes.push(rule);
    }

    /// Route a task to the appropriate worker.
    /// Rules are sorted by descending priority; first match wins.
    /// If no rule matches, the fallback worker is returned.
    pub fn route(&self, task_type: &str, payload: &serde_json::Value) -> String {
        let mut candidates: Vec<&RouteRule> = self.routes.iter().collect();
        candidates.sort_by(|a, b| b.priority.cmp(&a.priority));

        for rule in &candidates {
            if rule.condition.matches(task_type, payload) {
                self.stats.total_routed.fetch_add(1, Ordering::Relaxed);
                self.stats
                    .by_rule
                    .get(&rule.name)
                    .map(|c| c.fetch_add(1, Ordering::Relaxed));
                self.stats
                    .by_worker
                    .get(&rule.target_worker)
                    .map(|c| c.fetch_add(1, Ordering::Relaxed));
                return rule.target_worker.clone();
            }
        }

        self.stats.total_routed.fetch_add(1, Ordering::Relaxed);
        self.stats
            .by_worker
            .get(&self.fallback_worker)
            .map(|c| c.fetch_add(1, Ordering::Relaxed));
        self.stats.fallback_count.fetch_add(1, Ordering::Relaxed);
        self.fallback_worker.clone()
    }

    /// Atomically adjust a worker's load counter by `delta`.
    pub fn update_load(&self, worker: &str, delta: isize) {
        if let Some(load) = self.worker_loads.get(worker) {
            if delta >= 0 {
                load.fetch_add(delta as usize, Ordering::Relaxed);
            } else {
                let abs = (-delta) as usize;
                load.fetch_max(abs, Ordering::Relaxed);
                load.fetch_sub(abs.min(load.load(Ordering::Relaxed)), Ordering::Relaxed);
            }
        }
    }

    /// Return the worker with the lowest current load.
    pub fn least_loaded_worker(&self) -> String {
        self.worker_loads
            .iter()
            .min_by_key(|(_, v)| v.load(Ordering::Relaxed))
            .map(|(k, _)| k.clone())
            .unwrap_or_else(|| self.fallback_worker.clone())
    }

    pub fn stats(&self) -> RouterStats {
        RouterStats {
            total_routed: self.stats.total_routed.load(Ordering::Relaxed) as u64,
            by_worker: self
                .stats
                .by_worker
                .iter()
                .map(|(k, v)| (k.clone(), v.load(Ordering::Relaxed) as u64))
                .collect(),
            by_rule: self
                .stats
                .by_rule
                .iter()
                .map(|(k, v)| (k.clone(), v.load(Ordering::Relaxed) as u64))
                .collect(),
            fallback_count: self.stats.fallback_count.load(Ordering::Relaxed) as u64,
        }
    }
}

impl RouteCondition {
    fn matches(&self, task_type: &str, payload: &serde_json::Value) -> bool {
        match self {
            RouteCondition::TaskType(t) => t == task_type,
            RouteCondition::PayloadField { field, value } => payload.get(field) == Some(value),
            RouteCondition::Composite(conditions) => {
                conditions.iter().all(|c| c.matches(task_type, payload))
            }
            RouteCondition::Always => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_router() -> TradeRouter {
        let mut router = TradeRouter::new("fallback_worker");
        router.add_rule(RouteRule {
            name: "urgent".into(),
            condition: RouteCondition::PayloadField {
                field: "priority".into(),
                value: json!("urgent"),
            },
            target_worker: "urgent_worker".into(),
            priority: 100,
        });
        router.add_rule(RouteRule {
            name: "query".into(),
            condition: RouteCondition::TaskType("query".into()),
            target_worker: "query_worker".into(),
            priority: 50,
        });
        router.add_rule(RouteRule {
            name: "always_rule".into(),
            condition: RouteCondition::Always,
            target_worker: "catch_all_worker".into(),
            priority: 10,
        });
        router
    }

    #[test]
    fn test_fallback_when_no_rules() {
        let router = TradeRouter::new("fallback");
        assert_eq!(router.route("anything", &json!({})), "fallback");
    }

    #[test]
    fn test_task_type_match() {
        let router = make_router();
        let result = router.route("query", &json!({}));
        assert_eq!(result, "query_worker");
    }

    #[test]
    fn test_payload_field_match() {
        let router = make_router();
        let result = router.route("other", &json!({"priority": "urgent"}));
        assert_eq!(result, "urgent_worker");
    }

    #[test]
    fn test_priority_wins_over_always() {
        let router = make_router();
        let result = router.route("task", &json!({"priority": "urgent"}));
        assert_eq!(result, "urgent_worker");
    }

    #[test]
    fn test_always_fallback() {
        let router = make_router();
        let result = router.route("unknown_task", &json!({"foo": "bar"}));
        assert_eq!(result, "catch_all_worker");
    }

    #[test]
    fn test_composite_condition() {
        let mut router = TradeRouter::new("fallback");
        router.add_rule(RouteRule {
            name: "combo".into(),
            condition: RouteCondition::Composite(vec![
                RouteCondition::TaskType("export".into()),
                RouteCondition::PayloadField {
                    field: "region".into(),
                    value: json!("asia"),
                },
            ]),
            target_worker: "asia_export".into(),
            priority: 80,
        });
        assert_eq!(
            router.route("export", &json!({"region": "asia"})),
            "asia_export"
        );
        // partial match should not trigger
        assert_eq!(
            router.route("export", &json!({"region": "eu"})),
            "fallback"
        );
    }

    #[test]
    fn test_least_loaded_worker() {
        let mut router = TradeRouter::new("w0");
        router.add_rule(RouteRule {
            name: "a".into(),
            condition: RouteCondition::Always,
            target_worker: "w1".into(),
            priority: 1,
        });
        router.add_rule(RouteRule {
            name: "b".into(),
            condition: RouteCondition::Always,
            target_worker: "w2".into(),
            priority: 1,
        });
        router.update_load("w1", 5);
        router.update_load("w2", 2);
        assert_eq!(router.least_loaded_worker(), "w2");
    }

    #[test]
    fn test_stats() {
        let router = make_router();
        router.route("query", &json!({}));
        router.route("query", &json!({}));
        router.route("unknown", &json!({}));

        let stats = router.stats();
        assert_eq!(stats.total_routed, 3);
        assert_eq!(stats.fallback_count, 1);
    }
}
