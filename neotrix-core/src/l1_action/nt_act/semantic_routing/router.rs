//! PatternRouter — Routes to nearest skill based on behavior pattern

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::{BehaviorPattern, PatternCatalog, RouteResult, BehaviorPatternType, SimilarityScore, PolicyTrace, TrustBoundary, FanInQueue, PatternRequest};
use super::behavior_pattern::BehaviorPatternType as BPT;
use super::semantic_routing::{BehaviorPatternType, TrustBoundary, SimilarityScore, PolicyTrace, RouteResult, FanInQueue, PatternRequest};

/// Pattern router that matches queries to the nearest skill
pub struct PatternRouter {
    /// Registered behavior patterns
    patterns: HashMap<String, BehaviorPattern>,
    /// Pattern catalog for lookup
    catalog: PatternCatalog,
    /// Policy traces for audit
    policy_traces: Vec<PolicyTrace>,
    /// Fan-in queues for batching
    fan_in_queues: HashMap<String, FanInQueue>,
    /// Default trust boundary
    default_trust_boundary: TrustBoundary,
    /// Routing cache
    routing_cache: Arc<Mutex<HashMap<String, RouteResult>>>,
}

impl Default for PatternRouter {
    fn default() -> Self { Self::new() }
}

impl PatternRouter {
    /// Create a new PatternRouter
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
            catalog: PatternCatalog::new(),
            policy_traces: Vec::new(),
            fan_in_queues: HashMap::new(),
            default_trust_boundary: TrustBoundary::Internal,
            routing_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register a behavior pattern
    pub fn register_pattern(&mut self, pattern: BehaviorPattern) {
        let id = pattern.id.clone();
        self.catalog.register(pattern.clone());
        self.patterns.insert(id, pattern);
    }

    /// Route a query to the nearest skill based on behavior patterns
    pub fn route(&mut self, query: &str) -> Option<RouteResult> {
        let best_match = self.find_best_pattern(query)?;
        let skill_id = best_match.primary_skill()?.to_string();
        let score = best_match.compute_similarity(query);
        let confidence = score.value;

        // Create policy trace
        let trace = PolicyTrace {
            skill_id: skill_id.clone(),
            pattern_id: best_match.id.clone(),
            confidence,
            trust_boundary: best_match.trust_boundary.clone(),
            timestamp: now_timestamp(),
            path: vec![best_match.id.clone()],
        };

        self.policy_traces.push(trace.clone());

        // Check fan-in queue
        if let Some(fan_in) = best_match.fan_in_queue.as_ref() {
            self.enqueue_request(&fan_in.queue_id, query, &trace);
        }

        // Cache the result
        let cache_key = format!("{}:{}", query, skill_id);
        let result = RouteResult {
            skill_id: skill_id.clone(),
            skill_name: best_match.name.clone(),
            score,
            policy_trace: trace,
            references_loaded: Vec::new(),
            scripts_loaded: Vec::new(),
        };

        self.routing_cache.lock().ok()?.insert(cache_key, result.clone());

        Some(result)
    }

    /// Route with explicit context
    pub fn route_with_context(&mut self, query: &str, context: &HashMap<String, String>) -> Option<RouteResult> {
        let mut best_score = 0.0;
        let mut best_pattern: Option<&BehaviorPattern> = None;

        for pattern in self.patterns.values() {
            let score = pattern.compute_similarity(query);
            // Boost score if context keywords match
            let context_boost = self.context_boost(pattern, context);
            let total = score.value + context_boost;
            if total > best_score {
                best_score = total;
                best_pattern = Some(pattern);
            }
        }

        best_pattern.map(|p| {
            let score = SimilarityScore { value: best_score.min(1.0), method: "context_aware".to_string() };
            let trace = PolicyTrace {
                skill_id: p.primary_skill()?.to_string(),
                pattern_id: p.id.clone(),
                confidence: best_score,
                trust_boundary: p.trust_boundary.clone(),
                timestamp: now_timestamp(),
                path: vec![p.id.clone()],
            };
            RouteResult {
                skill_id: p.primary_skill()?.to_string(),
                skill_name: p.name.clone(),
                score,
                policy_trace: trace,
                references_loaded: Vec::new(),
                scripts_loaded: Vec::new(),
            }
        })
    }

    /// Find the best matching pattern for a query
    fn find_best_pattern(&self, query: &str) -> Option<&BehaviorPattern> {
        let mut best: Option<(&BehaviorPattern, f64)> = None;
        for pattern in self.patterns.values() {
            let score = pattern.compute_similarity(query);
            match best {
                None => { best = Some((pattern, score.value)); }
                Some((_, b_score)) if score.value > b_score => {
                    best = Some((pattern, score.value));
                }
                _ => {}
            }
        }
        best.map(|(p, _)| p)
    }

    /// Calculate context boost for a pattern
    fn context_boost(&self, pattern: &BehaviorPattern, context: &HashMap<String, String>) -> f64 {
        let mut boost = 0.0;
        for (key, value) in context {
            if pattern.keywords.iter().any(|kw| kw.to_lowercase() == key.to_lowercase()) {
                boost += 0.2;
            }
            if pattern.metadata.get(key).map_or(false, |v| v == value) {
                boost += 0.1;
            }
        }
        boost.min(0.5)
    }

    /// Enqueue a request in a fan-in queue
    fn enqueue_request(&mut self, queue_id: &str, query: &str, trace: &PolicyTrace) {
        let request = PatternRequest {
            query: query.to_string(),
            context: HashMap::new(),
            priority: 1,
            timestamp: now_timestamp(),
        };
        let queue = self.fan_in_queues.entry(queue_id.to_string()).or_insert_with(|| {
            FanInQueue {
                queue_id: queue_id.to_string(),
                pattern_type: "Unknown".to_string(),
                pending_requests: Vec::new(),
                max_batch_size: 10,
                timeout_ms: 5000,
            }
        });
        queue.pending_requests.push(request);
    }

    /// Get all policy traces
    pub fn policy_traces(&self) -> &[PolicyTrace] {
        &self.policy_traces
    }

    /// Get pattern by ID
    pub fn get_pattern(&self, id: &str) -> Option<&BehaviorPattern> {
        self.patterns.get(id)
    }

    /// Get number of registered patterns
    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }

    /// Add a fan-in queue
    pub fn add_fan_in_queue(&mut self, queue: FanInQueue) {
        self.fan_in_queues.insert(queue.queue_id.clone(), queue);
    }

    /// Set default trust boundary
    pub fn set_default_trust_boundary(&mut self, boundary: TrustBoundary) {
        self.default_trust_boundary = boundary;
    }
}

fn now_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use super::semantic_routing::BehaviorPatternType;

    fn create_test_router() -> PatternRouter {
        let mut router = PatternRouter::new();
        let mut pattern = BehaviorPattern::new("p1".to_string(), "Research Pattern".to_string(), BehaviorPatternType::Research);
        pattern.add_keyword("research".to_string());
        pattern.add_keyword("analyze".to_string());
        pattern.add_skill("nt-act-research-skill".to_string());
        pattern.trust_boundary = TrustBoundary::Internal;
        pattern.priority = 5;
        router.register_pattern(pattern);

        let mut pattern2 = BehaviorPattern::new("p2".to_string(), "Code Pattern".to_string(), BehaviorPatternType::CodeGeneration);
        pattern2.add_keyword("code".to_string());
        pattern2.add_keyword("generate".to_string());
        pattern2.add_skill("nt-act-code-skill".to_string());
        pattern2.trust_boundary = TrustBoundary::Public;
        pattern2.priority = 3;
        router.register_pattern(pattern2);

        router
    }

    #[test]
    fn test_route_matches_research() {
        let mut router = create_test_router();
        let result = router.route("research analysis paper");
        assert!(result.is_some());
        assert_eq!(result.as_ref().unwrap().skill_id, "nt-act-research-skill");
    }

    #[test]
    fn test_route_matches_code() {
        let mut router = create_test_router();
        let result = router.route("generate code implementation");
        assert!(result.is_some());
        assert_eq!(result.as_ref().unwrap().skill_id, "nt-act-code-skill");
    }

    #[test]
    fn test_route_no_match() {
        let mut router = PatternRouter::new();
        let result = router.route("unrelated query");
        assert!(result.is_none());
    }

    #[test]
    fn test_route_with_context() {
        let mut router = create_test_router();
        let mut context = HashMap::new();
        context.insert("domain".to_string(), "research".to_string());
        let result = router.route_with_context("analysis", &context);
        assert!(result.is_some());
    }

    #[test]
    fn test_pattern_count() {
        let router = create_test_router();
        assert_eq!(router.pattern_count(), 2);
    }

    #[test]
    fn test_get_pattern() {
        let router = create_test_router();
        assert!(router.get_pattern("p1").is_some());
        assert!(router.get_pattern("nonexistent").is_none());
    }

    #[test]
    fn test_policy_traces() {
        let mut router = create_test_router();
        router.route("research paper");
        router.route("code generation");
        assert_eq!(router.policy_traces().len(), 2);
    }
}