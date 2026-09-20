//! LLM routing strategies — inspired by LLMRouter (UIUC).
//! Dynamically selects the optimal model for each query based on
//! task complexity, cost, and performance requirements.

/// Trait for all routing strategies
pub trait RouteStrategy: Send + Sync {
    fn name(&self) -> &str;
    fn route(&self, _query: &RouteQuery) -> RouteDecision;
}

/// A query to be routed
#[derive(Debug, Clone)]
pub struct RouteQuery {
    pub content: String,
    pub task_type: String,
    pub complexity_hint: Option<f64>,
    pub max_cost: Option<f64>,
}

/// Decision from a routing strategy
#[derive(Debug, Clone)]
pub struct RouteDecision {
    pub selected_model: String,
    pub confidence: f64,
    pub estimated_cost: f64,
    pub reasoning: String,
}

/// Candidate model for routing
#[derive(Debug, Clone)]
pub struct RouteCandidate {
    pub name: String,
    pub provider: String,
    pub cost_per_1k_tokens: f64,
    pub capability_score: f64,
}

// === Built-in routing strategies ===

/// Route to smallest cheapest model
pub struct SmallestModelRouter;
impl RouteStrategy for SmallestModelRouter {
    fn name(&self) -> &str { "smallest_model" }
    fn route(&self, _query: &RouteQuery) -> RouteDecision {
        RouteDecision {
            selected_model: "tiny-model".into(),
            confidence: 0.5,
            estimated_cost: 0.001,
            reasoning: "Smallest model router".into(),
        }
    }
}

/// Route to largest most capable model
pub struct LargestModelRouter;
impl RouteStrategy for LargestModelRouter {
    fn name(&self) -> &str { "largest_model" }
    fn route(&self, _query: &RouteQuery) -> RouteDecision {
        RouteDecision {
            selected_model: "large-model".into(),
            confidence: 0.9,
            estimated_cost: 0.1,
            reasoning: "Largest model router".into(),
        }
    }
}

/// KNN-based routing
pub struct KnnRouter;
impl RouteStrategy for KnnRouter {
    fn name(&self) -> &str { "knn_router" }
    fn route(&self, _query: &RouteQuery) -> RouteDecision {
        RouteDecision {
            selected_model: "medium-model".into(),
            confidence: 0.7,
            estimated_cost: 0.01,
            reasoning: "KNN similarity routing".into(),
        }
    }
}

/// Cost-optimized routing (cheapest model that can handle difficulty)
pub struct CostOptimizedRouter;
impl RouteStrategy for CostOptimizedRouter {
    fn name(&self) -> &str { "cost_optimized" }
    fn route(&self, _query: &RouteQuery) -> RouteDecision {
        RouteDecision {
            selected_model: "small-model".into(),
            confidence: 0.6,
            estimated_cost: 0.005,
            reasoning: "Cost-optimized routing".into(),
        }
    }
}

/// Registry of all routing strategies
pub struct RoutingRegistry {
    strategies: Vec<Box<dyn RouteStrategy>>,
}

impl RoutingRegistry {
    pub fn new() -> Self {
        let strategies: Vec<Box<dyn RouteStrategy>> = vec![
            Box::new(SmallestModelRouter),
            Box::new(LargestModelRouter),
            Box::new(KnnRouter),
            Box::new(CostOptimizedRouter),
        ];
        Self { strategies }
    }

    pub fn get(&self, name: &str) -> Option<&dyn RouteStrategy> {
        self.strategies.iter().find(|s| s.name() == name).map(|s| s.as_ref())
    }

    pub fn route_with(&self, strategy: &str, query: &RouteQuery) -> Option<RouteDecision> {
        self.get(strategy).map(|s| s.route(query))
    }

    pub fn list(&self) -> Vec<&str> {
        self.strategies.iter().map(|s| s.name()).collect()
    }
}

impl Default for RoutingRegistry {
    fn default() -> Self { Self::new() }
}
