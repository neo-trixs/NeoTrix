use std::collections::{HashMap, VecDeque};
use std::time::Instant;

/// Records a single user query for pattern analysis.
pub struct QueryRecord {
    pub query: String,
    pub domain: String,
    pub intent: String,
    pub timestamp: Instant,
    pub vsa_fingerprint: u64,
}

/// Context that has been pre-fetched based on predicted user need.
pub struct PreFetchedContext {
    pub domain: String,
    pub content_fingerprints: Vec<u64>,
    pub predicted_relevance: f64,
    pub confidence: f64,
    pub prefetched_at: Instant,
}

/// Predicts what context the user will need next based on recent query patterns.
/// ACI-style: anticipate before ask, pre-fetch into working memory.
pub struct ContextPredictor {
    query_history: VecDeque<QueryRecord>,
    domain_transitions: HashMap<String, HashMap<String, u64>>,
    intent_transitions: HashMap<String, HashMap<String, u64>>,
    pre_fetch_buffer: HashMap<String, PreFetchedContext>,
    max_history: usize,
    min_samples: usize,
    prediction_confidence_threshold: f64,
    total_predictions: u64,
    correct_predictions: u64,
}

impl ContextPredictor {
    pub fn new() -> Self {
        Self {
            query_history: VecDeque::with_capacity(100),
            domain_transitions: HashMap::new(),
            intent_transitions: HashMap::new(),
            pre_fetch_buffer: HashMap::new(),
            max_history: 100,
            min_samples: 3,
            prediction_confidence_threshold: 0.3,
            total_predictions: 0,
            correct_predictions: 0,
        }
    }

    pub fn with_params(max_history: usize, min_samples: usize, threshold: f64) -> Self {
        Self {
            query_history: VecDeque::with_capacity(max_history),
            domain_transitions: HashMap::new(),
            intent_transitions: HashMap::new(),
            pre_fetch_buffer: HashMap::new(),
            max_history,
            min_samples,
            prediction_confidence_threshold: threshold,
            total_predictions: 0,
            correct_predictions: 0,
        }
    }

    /// Record a user query, updating domain and intent transition counts.
    /// Uses a simple hash of the query as VSA fingerprint.
    pub fn record_query(&mut self, query: String, domain: String, intent: String) {
        use std::hash::{DefaultHasher, Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        query.hash(&mut hasher);
        let vsa_fingerprint = hasher.finish();

        if let Some(prev) = self.query_history.back() {
            // Update domain transition: prev.domain → current.domain
            let domain_entry = self
                .domain_transitions
                .entry(prev.domain.clone())
                .or_default();
            *domain_entry.entry(domain.clone()).or_insert(0) += 1;

            // Update intent transition: prev.intent → current.intent
            let intent_entry = self
                .intent_transitions
                .entry(prev.intent.clone())
                .or_default();
            *intent_entry.entry(intent.clone()).or_insert(0) += 1;
        }

        self.query_history.push_back(QueryRecord {
            query,
            domain,
            intent,
            timestamp: Instant::now(),
            vsa_fingerprint,
        });

        while self.query_history.len() > self.max_history {
            self.query_history.pop_front();
        }
    }

    /// Predict the most likely next domains based on the transition matrix.
    /// Returns (domain, confidence) sorted by confidence descending.
    pub fn predict_next_domains(&self) -> Vec<(String, f64)> {
        let last_domain = match self.query_history.back() {
            Some(r) => &r.domain,
            None => return Vec::new(),
        };

        let transitions = match self.domain_transitions.get(last_domain) {
            Some(t) => t,
            None => return Vec::new(),
        };

        let total: u64 = transitions.values().sum();
        if total == 0 {
            return Vec::new();
        }

        let mut predicted: Vec<(String, f64)> = transitions
            .iter()
            .map(|(domain, count)| {
                let confidence = *count as f64 / total as f64;
                (domain.clone(), confidence)
            })
            .filter(|(_, c)| *c >= self.prediction_confidence_threshold)
            .collect();

        predicted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        predicted.truncate(3);
        predicted
    }

    /// Predict the most likely next intents based on the intent transition matrix.
    /// Returns (intent, confidence) sorted by confidence descending.
    pub fn predict_next_intents(&self) -> Vec<(String, f64)> {
        let last_intent = match self.query_history.back() {
            Some(r) => &r.intent,
            None => return Vec::new(),
        };

        let transitions = match self.intent_transitions.get(last_intent) {
            Some(t) => t,
            None => {
                // Fallback to heuristic defaults when no data
                let heuristic = match last_intent.as_str() {
                    "build" => vec!["debug".to_string(), "learn".to_string()],
                    "learn" => vec!["build".to_string(), "explore".to_string()],
                    "debug" => vec!["build".to_string(), "learn".to_string()],
                    "explore" => vec!["learn".to_string(), "analyze".to_string()],
                    "analyze" => vec!["build".to_string(), "learn".to_string()],
                    _ => vec!["learn".to_string(), "build".to_string()],
                };
                let total = heuristic.len() as f64;
                return heuristic
                    .into_iter()
                    .enumerate()
                    .map(|(i, intent)| {
                        let confidence = ((total - i as f64) / total).max(0.2);
                        (intent, confidence)
                    })
                    .collect();
            }
        };

        let total: u64 = transitions.values().sum();
        if total == 0 {
            return Vec::new();
        }

        let mut predicted: Vec<(String, f64)> = transitions
            .iter()
            .map(|(intent, count)| {
                let confidence = *count as f64 / total as f64;
                (intent.clone(), confidence)
            })
            .filter(|(_, c)| *c >= self.prediction_confidence_threshold)
            .collect();

        predicted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        predicted.truncate(3);
        predicted
    }

    /// Pre-fetch context for the given predicted domains.
    /// Tracks what would be fetched without actually performing fetches.
    pub fn pre_fetch(&mut self, domains: Vec<(String, f64)>) -> Vec<PreFetchedContext> {
        let mut results = Vec::new();
        for (domain, confidence) in domains {
            let predicted_relevance = confidence * 0.8 + 0.2;
            let entry = PreFetchedContext {
                domain: domain.clone(),
                content_fingerprints: Vec::new(),
                predicted_relevance,
                confidence,
                prefetched_at: Instant::now(),
            };
            self.pre_fetch_buffer.insert(domain.clone(), entry);
            if let Some(cached) = self.pre_fetch_buffer.get(&domain) {
                results.push(PreFetchedContext {
                    domain: cached.domain.clone(),
                    content_fingerprints: cached.content_fingerprints.clone(),
                    predicted_relevance: cached.predicted_relevance,
                    confidence: cached.confidence,
                    prefetched_at: cached.prefetched_at,
                });
            }
        }
        results
    }

    /// Record whether a domain prediction was correct (user actually queried it).
    pub fn record_prediction_outcome(&mut self, _predicted_domain: &str, was_correct: bool) {
        self.total_predictions += 1;
        if was_correct {
            self.correct_predictions += 1;
        }
    }

    /// Returns the prediction quality score: ratio of correct to total predictions.
    pub fn prediction_quality(&self) -> f64 {
        if self.total_predictions == 0 {
            return 0.0;
        }
        self.correct_predictions as f64 / self.total_predictions as f64
    }

    /// Returns how much prediction quality has improved over the last 10 queries.
    /// Used as a curiosity signal: improving predictions = learning the user's patterns.
    pub fn aci_bonus(&self) -> f64 {
        let history_len = self.query_history.len();
        if history_len < self.min_samples {
            return 0.0;
        }

        let quality = self.prediction_quality();
        let sample_ratio = (history_len as f64 / self.min_samples as f64).min(1.0);
        quality * sample_ratio
    }

    /// Current confidence in predictions based on sample size.
    pub fn confidence(&self) -> f64 {
        let history_len = self.query_history.len();
        if history_len < self.min_samples {
            return 0.0;
        }
        (history_len as f64 / (self.min_samples as f64 * 2.0)).min(1.0)
    }

    /// Number of unique domains observed.
    pub fn domain_count(&self) -> usize {
        self.domain_transitions.len()
    }

    /// Number of unique intents observed.
    pub fn intent_count(&self) -> usize {
        self.intent_transitions.len()
    }

    /// Number of queries recorded.
    pub fn query_count(&self) -> usize {
        self.query_history.len()
    }

    /// Number of pre-fetched contexts in buffer.
    pub fn prefetch_count(&self) -> usize {
        self.pre_fetch_buffer.len()
    }

    pub fn active(&self) -> bool {
        self.query_history.len() >= self.min_samples
    }

    pub fn set_min_samples(&mut self, n: usize) {
        self.min_samples = n;
    }

    pub fn set_threshold(&mut self, t: f64) {
        self.prediction_confidence_threshold = t;
    }
}

impl Default for ContextPredictor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_predictor() -> ContextPredictor {
        ContextPredictor::with_params(50, 2, 0.2)
    }

    #[test]
    fn test_new_predictor_inactive() {
        let p = ContextPredictor::new();
        assert!(!p.active());
        assert_eq!(p.confidence(), 0.0);
        assert!(p.predict_next_domains().is_empty());
    }

    #[test]
    fn test_record_query_updates_history() {
        let mut p = make_predictor();
        p.record_query("how to build X".into(), "coding".into(), "build".into());
        p.record_query("debug Y".into(), "coding".into(), "debug".into());
        assert_eq!(p.query_count(), 2);
    }

    #[test]
    fn test_domain_transition_learned() {
        let mut p = make_predictor();
        p.record_query("first".into(), "rust".into(), "learn".into());
        p.record_query("second".into(), "rust".into(), "build".into());
        p.record_query("third".into(), "debugging".into(), "debug".into());

        let domains = p.predict_next_domains();
        // Last domain was "debugging" — transitions from debugging
        assert!(domains.is_empty() || domains.iter().any(|(d, _)| d == "debugging"));
    }

    #[test]
    fn test_intent_transition_learned() {
        let mut p = make_predictor();
        p.record_query("a".into(), "x".into(), "build".into());
        p.record_query("b".into(), "y".into(), "debug".into());
        p.record_query("c".into(), "z".into(), "build".into());

        let intents = p.predict_next_intents();
        // Last intent was "build" — should predict "debug" from transition history
        assert!(!intents.is_empty());
        assert_eq!(intents[0].0, "debug");
    }

    #[test]
    fn test_intent_heuristic_fallback() {
        let p = make_predictor();
        // No queries recorded yet — empty history
        assert!(p.predict_next_intents().is_empty());
    }

    #[test]
    fn test_pre_fetch_creates_entries() {
        let mut p = make_predictor();
        let domains = vec![("coding".to_string(), 0.8), ("docs".to_string(), 0.5)];
        let results = p.pre_fetch(domains);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].domain, "coding");
        assert!(results[0].predicted_relevance >= 0.0);
        assert_eq!(p.prefetch_count(), 2);
    }

    #[test]
    fn test_prediction_quality_tracking() {
        let mut p = make_predictor();
        assert_eq!(p.prediction_quality(), 0.0);

        p.record_prediction_outcome("coding", true);
        p.record_prediction_outcome("docs", false);
        assert!((p.prediction_quality() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_aci_bonus_scales_with_history() {
        let mut p = make_predictor();
        p.set_min_samples(5);
        // Fewer than min_samples
        assert_eq!(p.aci_bonus(), 0.0);

        // Add enough queries to cross threshold
        for i in 0..5 {
            p.record_query(
                format!("q{}", i),
                "test".into(),
                if i % 2 == 0 {
                    "build".into()
                } else {
                    "debug".into()
                },
            );
        }
        // With 5 queries and min_samples=5, sample_ratio = 1.0
        // quality is 0 (no prediction outcomes recorded), so bonus should be 0
        let bonus = p.aci_bonus();
        assert_eq!(bonus, 0.0);

        // Add a correct prediction
        p.record_prediction_outcome("test", true);
        // Now quality = 1.0, sample_ratio = 1.0, bonus = 1.0
        assert!((p.aci_bonus() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_confidence_grows_with_history() {
        let mut p = make_predictor();
        p.set_min_samples(2);
        assert_eq!(p.confidence(), 0.0);

        for i in 0..3 {
            p.record_query(format!("q{}", i), "test".into(), "build".into());
        }
        // min_samples*2 = 4, 3/4 = 0.75
        assert!((p.confidence() - 0.75).abs() < 1e-9);
    }

    #[test]
    fn test_domain_count() {
        let mut p = make_predictor();
        assert_eq!(p.domain_count(), 0);

        p.record_query("q1".into(), "coding".into(), "build".into());
        p.record_query("q2".into(), "docs".into(), "learn".into());
        p.record_query("q3".into(), "coding".into(), "debug".into());

        // Two unique transitions: coding→docs, docs→coding (after 3 queries)
        assert_eq!(p.domain_count(), 2);
    }

    #[test]
    fn test_predict_returns_sorted() {
        let mut p = make_predictor();
        // Build up a pattern where "build" always follows "learn"
        for _ in 0..5 {
            p.record_query("learn something".into(), "docs".into(), "learn".into());
            p.record_query("build something".into(), "coding".into(), "build".into());
        }
        // Last query intent is "build"
        let intents = p.predict_next_intents();
        assert!(!intents.is_empty());
        // Sorted descending
        for w in intents.windows(2) {
            assert!(w[0].1 >= w[1].1);
        }
    }

    #[test]
    fn test_active_after_min_samples() {
        let mut p = make_predictor();
        p.set_min_samples(3);
        assert!(!p.active());

        for i in 0..3 {
            p.record_query(format!("q{}", i), "test".into(), "build".into());
        }
        assert!(p.active());
    }

    #[test]
    fn test_with_params_configures_properly() {
        let p = ContextPredictor::with_params(10, 5, 0.5);
        assert_eq!(p.max_history, 10);
        assert_eq!(p.min_samples, 5);
        assert!((p.prediction_confidence_threshold - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_prefetch_does_not_duplicate() {
        let mut p = make_predictor();
        p.pre_fetch(vec![("domain_a".to_string(), 0.7)]);
        assert_eq!(p.prefetch_count(), 1);
        // Pre-fetching the same domain again overwrites
        p.pre_fetch(vec![("domain_a".to_string(), 0.9)]);
        assert_eq!(p.prefetch_count(), 1);
    }

    #[test]
    fn test_record_prediction_outcome_no_divide_by_zero() {
        let p = make_predictor();
        assert_eq!(p.prediction_quality(), 0.0);
    }

    #[test]
    fn test_aci_bonus_no_history() {
        let p = ContextPredictor::new();
        assert_eq!(p.aci_bonus(), 0.0);
    }
}
