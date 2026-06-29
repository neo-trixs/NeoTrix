#![allow(dead_code)]
use std::collections::VecDeque;

/// Evidence level in the spatial hierarchy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EvidenceLevel {
    /// Multimodal perception: raw image → LLM description → VSA embedding.
    Level1Perception = 1,
    /// VSA spatial encoding: position/orientation/layout in VSA hypervectors.
    Level2VSASpatial = 2,
    /// Spatial graph: relationships, topology, consistency constraints.
    Level3SpatialGraph = 3,
}

impl EvidenceLevel {
    pub fn name(&self) -> &'static str {
        match self {
            EvidenceLevel::Level1Perception => "multimodal_perception",
            EvidenceLevel::Level2VSASpatial => "vsa_spatial_encoding",
            EvidenceLevel::Level3SpatialGraph => "spatial_graph",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            EvidenceLevel::Level1Perception => "ImagePipeline → LLM description of visual scene",
            EvidenceLevel::Level2VSASpatial => {
                "VSASpatialEncoder + SpatialSceneEngine VSA positions"
            }
            EvidenceLevel::Level3SpatialGraph => "SpatialGraph geographic relationships & topology",
        }
    }
}

/// A single piece of spatial evidence at a given level.
#[derive(Debug, Clone)]
pub struct SpatialEvidence {
    pub level: EvidenceLevel,
    pub label: String,
    pub description: String,
    pub confidence: f64,
    pub source: String,
}

/// Result of a spatial query spanning multiple evidence levels.
#[derive(Debug, Clone)]
pub struct SpatialQueryResult {
    pub query: String,
    pub evidence: Vec<SpatialEvidence>,
    pub aggregate_confidence: f64,
    pub coverage: u8,
    pub summary: String,
}

impl SpatialQueryResult {
    fn push(&mut self, ev: SpatialEvidence) {
        self.coverage |= 1 << (ev.level as u8 - 1);
        self.evidence.push(ev);
    }

    fn finalize(&mut self) {
        if self.evidence.is_empty() {
            self.aggregate_confidence = 0.0;
            self.summary = "no spatial evidence available".into();
            return;
        }
        let total: f64 = self
            .evidence
            .iter()
            .map(|e| e.confidence * (e.level as u8 as f64))
            .sum();
        let weights: f64 = self.evidence.iter().map(|e| e.level as u8 as f64).sum();
        self.aggregate_confidence = if weights > 0.0 { total / weights } else { 0.0 };
    }
}

/// Factory for creating level-specific evidence from closures.
pub struct SpatialEvidenceFactory;

impl SpatialEvidenceFactory {
    pub fn level1(label: &str, description: &str, confidence: f64) -> SpatialEvidence {
        SpatialEvidence {
            level: EvidenceLevel::Level1Perception,
            label: label.to_string(),
            description: description.to_string(),
            confidence,
            source: "ImagePipeline".into(),
        }
    }

    pub fn level2(label: &str, description: &str, confidence: f64) -> SpatialEvidence {
        SpatialEvidence {
            level: EvidenceLevel::Level2VSASpatial,
            label: label.to_string(),
            description: description.to_string(),
            confidence,
            source: "VSASpatialEncoder/SpatialSceneEngine".into(),
        }
    }

    pub fn level3(label: &str, description: &str, confidence: f64) -> SpatialEvidence {
        SpatialEvidence {
            level: EvidenceLevel::Level3SpatialGraph,
            label: label.to_string(),
            description: description.to_string(),
            confidence,
            source: "SpatialGraph".into(),
        }
    }
}

/// Spatial reasoning engine with 3-level evidence hierarchy.
///
/// Level 1 — Multimodal perception (via ImagePipeline)
/// Level 2 — VSA spatial encoding (via existing VSASpatialEncoder + SpatialSceneEngine)
/// Level 3 — Spatial graph / consistency (via existing SpatialGraph)
pub struct SpatialReasoner {
    max_evidence: usize,
    query_history: Vec<SpatialQueryResult>,
    /// Pending spatial queries awaiting processing.
    pending_queries: VecDeque<String>,
    /// Completed query results available for consumption.
    completed_results: VecDeque<SpatialQueryResult>,
    rate_limit: usize,
    queries_processed: u64,
}

impl SpatialReasoner {
    pub fn new(max_evidence: usize, rate_limit: usize) -> Self {
        Self {
            max_evidence,
            query_history: Vec::with_capacity(max_evidence.min(128)),
            pending_queries: VecDeque::new(),
            completed_results: VecDeque::new(),
            rate_limit,
            queries_processed: 0,
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(1000, 10)
    }

    pub fn queries_processed(&self) -> u64 {
        self.queries_processed
    }

    /// Enqueue a spatial query for background processing.
    pub fn enqueue_query(&mut self, query: String) {
        const MAX_PENDING: usize = 200;
        if self.pending_queries.len() >= MAX_PENDING {
            self.pending_queries.pop_front();
        }
        self.pending_queries.push_back(query);
    }

    pub fn pending_count(&self) -> usize {
        self.pending_queries.len()
    }

    /// Process one pending spatial query using provided level callbacks.
    pub fn process_pending(
        &mut self,
        l1_fn: &mut dyn FnMut(&str) -> Option<SpatialEvidence>,
        l2_fn: &mut dyn FnMut(&str) -> Option<SpatialEvidence>,
        l3_fn: &mut dyn FnMut(&str) -> Option<SpatialEvidence>,
    ) -> Option<SpatialQueryResult> {
        let query = self.pending_queries.pop_front()?;
        let result = self.query(&query, l1_fn, l2_fn, l3_fn);
        self.completed_results.push_back(result.clone());
        if self.completed_results.len() > 500 {
            self.completed_results.pop_front();
        }
        Some(result)
    }

    /// Take completed results for consumption.
    pub fn take_results(&mut self) -> Vec<SpatialQueryResult> {
        self.completed_results.drain(..).collect()
    }

    /// Answer a spatial query by building evidence from available sources.
    pub fn query(
        &mut self,
        query: &str,
        perception_fn: &mut dyn FnMut(&str) -> Option<SpatialEvidence>,
        vsa_fn: &mut dyn FnMut(&str) -> Option<SpatialEvidence>,
        spatial_fn: &mut dyn FnMut(&str) -> Option<SpatialEvidence>,
    ) -> SpatialQueryResult {
        self.queries_processed += 1;
        let mut result = SpatialQueryResult {
            query: query.to_string(),
            evidence: Vec::new(),
            aggregate_confidence: 0.0,
            coverage: 0,
            summary: String::new(),
        };
        if let Some(ev) = perception_fn(query) {
            result.push(ev);
        }
        if let Some(ev) = vsa_fn(query) {
            result.push(ev);
        }
        if let Some(ev) = spatial_fn(query) {
            result.push(ev);
        }
        result.finalize();
        if self.query_history.len() >= self.max_evidence {
            self.query_history.drain(0..self.max_evidence / 5);
        }
        self.query_history.push(result.clone());
        result
    }

    pub fn history(&self) -> &[SpatialQueryResult] {
        &self.query_history
    }

    pub fn report(&self) -> String {
        let levels = [
            EvidenceLevel::Level1Perception,
            EvidenceLevel::Level2VSASpatial,
            EvidenceLevel::Level3SpatialGraph,
        ];
        let coverage: Vec<&str> = levels
            .iter()
            .filter(|l| {
                self.query_history
                    .iter()
                    .any(|r| r.coverage & (1 << (**l as u8 - 1)) != 0)
            })
            .map(|l| l.name())
            .collect();
        format!(
            "SpatialReasoner: {} queries, {} history, {} pending, {} results, coverage: [{}]",
            self.queries_processed,
            self.query_history.len(),
            self.pending_queries.len(),
            self.completed_results.len(),
            coverage.join(", "),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- helpers ---

    fn level1(label: &str, conf: f64) -> SpatialEvidence {
        SpatialEvidenceFactory::level1(label, "perception desc", conf)
    }
    fn level2(label: &str, conf: f64) -> SpatialEvidence {
        SpatialEvidenceFactory::level2(label, "vsa desc", conf)
    }
    fn level3(label: &str, conf: f64) -> SpatialEvidence {
        SpatialEvidenceFactory::level3(label, "spatial desc", conf)
    }

    fn mock_perception(q: &str) -> Option<SpatialEvidence> {
        if q.contains("visible") {
            Some(level1(q, 0.9))
        } else {
            None
        }
    }
    fn mock_vsa(q: &str) -> Option<SpatialEvidence> {
        if q.contains("position") {
            Some(level2(q, 0.8))
        } else {
            None
        }
    }
    fn mock_spatial(q: &str) -> Option<SpatialEvidence> {
        if q.contains("region") {
            Some(level3(q, 0.7))
        } else {
            None
        }
    }

    fn always_l1(_: &str) -> Option<SpatialEvidence> {
        Some(level1("all", 0.5))
    }
    fn always_l2(_: &str) -> Option<SpatialEvidence> {
        Some(level2("all", 0.5))
    }
    fn always_l3(_: &str) -> Option<SpatialEvidence> {
        Some(level3("all", 0.5))
    }

    // --- SpatialReasoner construction ---

    #[test]
    fn test_new() {
        let mut sr = SpatialReasoner::new(500, 20);
        assert_eq!(sr.pending_count(), 0);
        assert_eq!(sr.queries_processed(), 0);
        assert!(sr.history().is_empty());
        assert!(sr.take_results().is_empty());
    }

    #[test]
    fn test_with_defaults() {
        let sr = SpatialReasoner::with_defaults();
        assert_eq!(sr.pending_count(), 0);
        assert_eq!(sr.queries_processed(), 0);
    }

    // --- enqueue / bounded capacity ---

    #[test]
    fn test_enqueue_query() {
        let mut sr = SpatialReasoner::new(100, 10);
        sr.enqueue_query("query A".into());
        sr.enqueue_query("query B".into());
        assert_eq!(sr.pending_count(), 2);
    }

    #[test]
    fn test_enqueue_bounded() {
        let mut sr = SpatialReasoner::new(100, 10);
        for i in 0..210 {
            sr.enqueue_query(format!("q{}", i));
        }
        assert_eq!(sr.pending_count(), 200);
        assert_eq!(sr.take_results().len(), 0);
    }

    // --- process_pending ---

    #[test]
    fn test_process_pending_empty() {
        let mut sr = SpatialReasoner::new(100, 10);
        let result = sr.process_pending(&mut |_| None, &mut |_| None, &mut |_| None);
        assert!(result.is_none());
        assert_eq!(sr.queries_processed(), 0);
    }

    #[test]
    fn test_process_pending_single() {
        let mut sr = SpatialReasoner::new(100, 10);
        sr.enqueue_query("visible landmark".into());

        let result = sr.process_pending(&mut mock_perception, &mut mock_vsa, &mut mock_spatial);
        assert!(result.is_some());
        let r = result.unwrap();
        assert_eq!(r.query, "visible landmark");
        assert_eq!(r.evidence.len(), 1); // only perception matches
        assert_eq!(r.evidence[0].level, EvidenceLevel::Level1Perception);
        assert_eq!(sr.queries_processed(), 1);
    }

    #[test]
    fn test_process_pending_all_levels() {
        let mut sr = SpatialReasoner::new(100, 10);
        sr.enqueue_query("visible position region".into());

        let r = sr
            .process_pending(&mut mock_perception, &mut mock_vsa, &mut mock_spatial)
            .unwrap();
        assert_eq!(r.evidence.len(), 3);
        assert_eq!(r.coverage, 0b111);
        assert!(r.aggregate_confidence > 0.0);
    }

    #[test]
    fn test_take_results() {
        let mut sr = SpatialReasoner::new(100, 10);
        sr.enqueue_query("visible".into());
        sr.process_pending(&mut mock_perception, &mut mock_vsa, &mut mock_spatial);

        let results = sr.take_results();
        assert_eq!(results.len(), 1);
        assert_eq!(sr.take_results().len(), 0); // second drain is empty
    }

    // --- completed_results bounded ---

    #[test]
    fn test_completed_results_bounded() {
        let mut sr = SpatialReasoner::new(100, 10);
        for i in 0..600 {
            sr.enqueue_query(format!("visible q{}", i));
            sr.process_pending(&mut mock_perception, &mut mock_vsa, &mut mock_spatial);
        }
        // completed_results should not exceed 500
        let rem = sr.take_results();
        assert!(rem.len() <= 500);
        assert_eq!(sr.queries_processed(), 600);
    }

    // --- query direct ---

    #[test]
    fn test_query_all_levels() {
        let mut sr = SpatialReasoner::new(100, 10);
        let r = sr.query(
            "visible position region",
            &mut mock_perception,
            &mut mock_vsa,
            &mut mock_spatial,
        );
        assert_eq!(r.evidence.len(), 3);
        assert_eq!(r.coverage, 0b111);
        assert!(r.aggregate_confidence > 0.0);
        assert_eq!(sr.queries_processed(), 1);
    }

    #[test]
    fn test_query_no_evidence() {
        let mut sr = SpatialReasoner::new(100, 10);
        let r = sr.query("unknown", &mut |_| None, &mut |_| None, &mut |_| None);
        assert!(r.evidence.is_empty());
        assert_eq!(r.coverage, 0);
        assert_eq!(r.aggregate_confidence, 0.0);
        assert_eq!(r.summary, "no spatial evidence available");
    }

    // --- confidence propagation ---

    #[test]
    fn test_confidence_weighted() {
        let mut sr = SpatialReasoner::new(100, 10);
        // l1 = level 1 @ 0.5, l2 = level 2 @ 0.8, l3 = level 3 @ 0.3
        let r = sr.query("q", &mut always_l1, &mut always_l2, &mut always_l3);
        // weighted: (0.5*1 + 0.8*2 + 0.3*3) / (1+2+3) = (0.5+1.6+0.9)/6 = 3.0/6 = 0.5
        assert!((r.aggregate_confidence - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_confidence_single_level() {
        let mut sr = SpatialReasoner::new(100, 10);
        let r = sr.query("q", &mut always_l1, &mut |_| None, &mut |_| None);
        assert!((r.aggregate_confidence - 0.5).abs() < 1e-10);
    }

    // --- history bounded ---

    #[test]
    fn test_history_bounded() {
        let mut sr = SpatialReasoner::new(10, 10);
        for i in 0..20 {
            sr.query(
                &format!("visible q{}", i),
                &mut mock_perception,
                &mut mock_vsa,
                &mut mock_spatial,
            );
        }
        assert!(sr.history().len() <= 10);
    }

    // --- SpatialEvidenceFactory ---

    #[test]
    fn test_factory_level1() {
        let ev = SpatialEvidenceFactory::level1("door", "a wooden door", 0.85);
        assert_eq!(ev.level, EvidenceLevel::Level1Perception);
        assert_eq!(ev.label, "door");
        assert_eq!(ev.confidence, 0.85);
        assert_eq!(ev.source, "ImagePipeline");
    }

    #[test]
    fn test_factory_level2() {
        let ev = SpatialEvidenceFactory::level2("pos", "x=10 y=20", 0.75);
        assert_eq!(ev.level, EvidenceLevel::Level2VSASpatial);
        assert_eq!(ev.source, "VSASpatialEncoder/SpatialSceneEngine");
    }

    #[test]
    fn test_factory_level3() {
        let ev = SpatialEvidenceFactory::level3("graph", "topology ok", 0.65);
        assert_eq!(ev.level, EvidenceLevel::Level3SpatialGraph);
        assert_eq!(ev.source, "SpatialGraph");
    }

    // --- EvidenceLevel ---

    #[test]
    fn test_evidence_level_ordering() {
        assert!(EvidenceLevel::Level1Perception < EvidenceLevel::Level2VSASpatial);
        assert!(EvidenceLevel::Level2VSASpatial < EvidenceLevel::Level3SpatialGraph);
    }

    #[test]
    fn test_evidence_level_names() {
        assert_eq!(
            EvidenceLevel::Level1Perception.name(),
            "multimodal_perception"
        );
        assert_eq!(
            EvidenceLevel::Level2VSASpatial.name(),
            "vsa_spatial_encoding"
        );
        assert_eq!(EvidenceLevel::Level3SpatialGraph.name(), "spatial_graph");
    }

    // --- report ---

    #[test]
    fn test_report_empty() {
        let sr = SpatialReasoner::new(100, 10);
        let r = sr.report();
        assert!(r.starts_with("SpatialReasoner:"));
        assert!(r.contains("0 queries"));
    }

    #[test]
    fn test_report_after_queries() {
        let mut sr = SpatialReasoner::new(100, 10);
        sr.query(
            "visible position region",
            &mut mock_perception,
            &mut mock_vsa,
            &mut mock_spatial,
        );
        let r = sr.report();
        assert!(r.contains("1 queries"));
        assert!(r.contains("1 history"));
    }

    // --- SpatialQueryResult ---

    #[test]
    fn test_query_result_push_finalize() {
        let mut result = SpatialQueryResult {
            query: "test".into(),
            evidence: vec![],
            aggregate_confidence: 0.0,
            coverage: 0,
            summary: String::new(),
        };
        result.push(level1("a", 0.8));
        result.push(level2("b", 0.9));
        assert_eq!(result.coverage, 0b011);
        result.finalize();
        assert!(result.aggregate_confidence > 0.0);
    }

    #[test]
    fn test_query_result_empty_finalize() {
        let mut result = SpatialQueryResult {
            query: "empty".into(),
            evidence: vec![],
            aggregate_confidence: 1.0,
            coverage: 0,
            summary: "something".into(),
        };
        result.finalize();
        assert_eq!(result.aggregate_confidence, 0.0);
        assert_eq!(result.summary, "no spatial evidence available");
    }
}
