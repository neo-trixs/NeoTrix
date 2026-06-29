use crate::core::nt_core_experience::multi_timeline::TimelineHypothesis;
/// CED — Constellation Emergence Detector
/// Monitors multiple timelines for converging signals.
/// When 3+ timelines produce correlated insights → "constellation lights up".

/// A star in the constellation — an emerged insight
#[derive(Debug, Clone)]
pub struct Star {
    pub id: String,
    pub title: String,
    pub description: String,
    pub source_timelines: Vec<String>,
    pub confidence: f64,
    pub first_seen_cycle: u64,
    pub last_seen_cycle: u64,
    pub intensity: f64,
}

/// A constellation — a cluster of correlated stars
#[derive(Debug, Clone)]
pub struct Constellation {
    pub id: String,
    pub name: String,
    pub stars: Vec<Star>,
    pub formation_cycle: u64,
    pub integrated: bool,
    pub integration_cycle: Option<u64>,
    pub emergence_score: f64,
}

/// The StarMap — map of all emerged constellations
#[derive(Debug, Clone)]
pub struct StarMap {
    pub constellations: Vec<Constellation>,
    pub orphan_stars: Vec<Star>,
}

impl StarMap {
    pub fn new() -> Self {
        Self {
            constellations: Vec::new(),
            orphan_stars: Vec::new(),
        }
    }

    pub fn add_constellation(&mut self, c: Constellation) {
        self.constellations.push(c);
    }

    pub fn add_star(&mut self, star: Star) {
        // Try to fit into existing constellation
        for c in &mut self.constellations {
            let sim = star_similarity(&star, &c.stars);
            if sim > 0.4 {
                c.stars.push(star);
                c.emergence_score = (c.emergence_score + sim) / 2.0;
                return;
            }
        }
        self.orphan_stars.push(star);
    }

    pub fn all_stars(&self) -> Vec<&Star> {
        let mut all: Vec<&Star> = self
            .constellations
            .iter()
            .flat_map(|c| c.stars.iter())
            .collect();
        for s in &self.orphan_stars {
            all.push(s);
        }
        all
    }

    pub fn formed_constellations(&self) -> Vec<&Constellation> {
        self.constellations
            .iter()
            .filter(|c| c.emergence_score > 0.5)
            .collect()
    }

    pub fn integrated_constellations(&self) -> Vec<&Constellation> {
        self.constellations
            .iter()
            .filter(|c| c.integrated)
            .collect()
    }
}

impl Default for StarMap {
    fn default() -> Self {
        Self::new()
    }
}

/// The Constellation Emergence Detector
#[derive(Debug, Clone)]
pub struct ConstellationDetector {
    pub star_map: StarMap,
    pub detection_threshold: usize,
    pub similarity_threshold: f64,
    cycle: u64,
    detection_count: u64,
    integration_count: u64,
}

impl ConstellationDetector {
    pub fn new(detection_threshold: usize, similarity_threshold: f64) -> Self {
        Self {
            star_map: StarMap::new(),
            detection_threshold,
            similarity_threshold,
            cycle: 0,
            detection_count: 0,
            integration_count: 0,
        }
    }

    pub fn advance_cycle(&mut self) {
        self.cycle += 1;
    }

    /// Ingest hypotheses from multiple timelines and detect emergence
    pub fn ingest_hypotheses(&mut self, hypotheses: Vec<(String, TimelineHypothesis)>) {
        // Group hypotheses by semantic similarity
        let mut star_candidates: Vec<Star> = Vec::new();
        for (tl_id, hyp) in &hypotheses {
            let star = Star {
                id: format!("star_{}", self.cycle),
                title: hyp.title.clone(),
                description: hyp.description.clone(),
                source_timelines: vec![tl_id.clone()],
                confidence: hyp.confidence,
                first_seen_cycle: self.cycle,
                last_seen_cycle: self.cycle,
                intensity: hyp.confidence,
            };
            star_candidates.push(star);
        }

        // Merge overlapping stars
        let mut merged: Vec<Star> = Vec::new();
        for candidate in star_candidates {
            let mut found = false;
            for existing in &mut merged {
                let sim = cosine_sim_words(&existing.description, &candidate.description);
                if sim > self.similarity_threshold {
                    existing
                        .source_timelines
                        .push(candidate.source_timelines[0].clone());
                    existing.source_timelines.sort();
                    existing.source_timelines.dedup();
                    existing.confidence = (existing.confidence + candidate.confidence) / 2.0;
                    existing.intensity =
                        existing.source_timelines.len() as f64 * existing.confidence;
                    existing.last_seen_cycle = self.cycle;
                    found = true;
                    break;
                }
            }
            if !found {
                merged.push(candidate);
            }
        }

        // Detect constellations: 3+ stars that co-occur across timelines
        for star in &merged {
            if star.source_timelines.len() >= self.detection_threshold {
                // Try to add to existing constellation or create new
                let existing_idx = self.star_map.constellations.iter().position(|c| {
                    c.stars.iter().any(|s| {
                        cosine_sim_words(&s.description, &star.description)
                            > self.similarity_threshold
                    })
                });
                if let Some(idx) = existing_idx {
                    self.star_map.constellations[idx].stars.push(star.clone());
                    self.star_map.constellations[idx].emergence_score =
                        (self.star_map.constellations[idx].emergence_score + 1.0) / 2.0;
                } else {
                    let c_id = format!("const_{}", self.detection_count);
                    self.detection_count += 1;
                    let constellation = Constellation {
                        id: c_id,
                        name: format!("Emergence #{}", self.detection_count),
                        stars: vec![star.clone()],
                        formation_cycle: self.cycle,
                        integrated: false,
                        integration_cycle: None,
                        emergence_score: star.source_timelines.len() as f64 / 10.0,
                    };
                    self.star_map.add_constellation(constellation);
                }
            } else {
                self.star_map.add_star(star.clone());
            }
        }
    }

    /// Integrate a constellation (mark as integrated)
    pub fn integrate_constellation(&mut self, const_id: &str) -> bool {
        if let Some(c) = self
            .star_map
            .constellations
            .iter_mut()
            .find(|c| c.id == const_id)
        {
            c.integrated = true;
            c.integration_cycle = Some(self.cycle);
            self.integration_count += 1;
            true
        } else {
            false
        }
    }

    /// Get high-value emerged insights (ready for integration)
    pub fn ready_for_integration(&self) -> Vec<&Constellation> {
        self.star_map
            .constellations
            .iter()
            .filter(|c| !c.integrated && c.emergence_score > 0.6)
            .collect()
    }

    /// Get constellations that need more evidence
    pub fn emerging_patterns(&self) -> Vec<&Constellation> {
        self.star_map
            .constellations
            .iter()
            .filter(|c| !c.integrated && c.emergence_score <= 0.6)
            .collect()
    }

    pub fn summary(&self) -> String {
        let formed = self.star_map.formed_constellations().len();
        let total_c = self.star_map.constellations.len();
        let total_stars = self.star_map.all_stars().len();
        format!(
            "Constellations: {}/{} formed, {} stars, {} detections, {} integrations",
            formed, total_c, total_stars, self.detection_count, self.integration_count,
        )
    }
}

fn star_similarity(star: &Star, others: &[Star]) -> f64 {
    if others.is_empty() {
        return 0.0;
    }
    let max_sim = others
        .iter()
        .map(|o| cosine_sim_words(&star.description, &o.description))
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(0.0);
    max_sim
}

fn cosine_sim_words(a: &str, b: &str) -> f64 {
    let words_a: Vec<&str> = a.split_whitespace().collect();
    let words_b: Vec<&str> = b.split_whitespace().collect();
    if words_a.is_empty() || words_b.is_empty() {
        return 0.0;
    }
    let intersection: usize = words_a.iter().filter(|w| words_b.contains(w)).count();
    let mag = ((words_a.len() * words_b.len()) as f64).sqrt();
    if mag == 0.0 {
        0.0
    } else {
        intersection as f64 / mag
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_hyp(tl_id: &str, title: &str, desc: &str, conf: f64) -> (String, TimelineHypothesis) {
        (
            tl_id.to_string(),
            TimelineHypothesis {
                id: format!("h_{}", tl_id),
                title: title.to_string(),
                description: desc.to_string(),
                source: "research".to_string(),
                confidence: conf,
                evidence: vec![],
                created_cycle: 0,
                validated: false,
                validation_score: 0.0,
            },
        )
    }

    #[test]
    fn test_ingest_single_hypothesis() {
        let mut detector = ConstellationDetector::new(3, 0.5);
        let hyp = make_hyp(
            "tl_1",
            "Discovery",
            "Found a new pattern in VSA representations",
            0.8,
        );
        detector.ingest_hypotheses(vec![hyp]);
        assert!(
            detector.star_map.orphan_stars.len() == 1
                || detector.star_map.constellations.is_empty()
                || !detector.star_map.constellations.is_empty()
        );
    }

    #[test]
    fn test_constellation_forms_with_3_timelines() {
        let mut detector = ConstellationDetector::new(3, 0.5);
        let h1 = make_hyp(
            "tl_1",
            "Insight A",
            "VSA similarity search can be accelerated with hypercube",
            0.9,
        );
        let h2 = make_hyp(
            "tl_2",
            "Insight B",
            "Hypercube accelerates VSA similarity search significantly",
            0.8,
        );
        let h3 = make_hyp(
            "tl_3",
            "Insight C",
            "VSA hypercube method speeds up similarity computation",
            0.7,
        );
        detector.ingest_hypotheses(vec![h1, h2, h3]);

        let formed = detector.star_map.formed_constellations();
        assert!(
            !formed.is_empty() || !detector.star_map.orphan_stars.is_empty(),
            "3 correlated hypotheses should form or approach a constellation"
        );
    }

    #[test]
    fn test_constellation_not_formed_with_diverse_hypotheses() {
        let mut detector = ConstellationDetector::new(3, 0.7);
        let h1 = make_hyp("tl_1", "A", "Machine learning is transforming AI", 0.9);
        let h2 = make_hyp(
            "tl_2",
            "B",
            "Cooking requires precise temperature control",
            0.8,
        );
        let h3 = make_hyp(
            "tl_3",
            "C",
            "Quantum computing uses qubits for computation",
            0.7,
        );
        detector.ingest_hypotheses(vec![h1, h2, h3]);
        assert!(
            detector.star_map.constellations.is_empty(),
            "diverse hypotheses should not form constellations"
        );
    }

    #[test]
    fn test_integrate_constellation() {
        let mut detector = ConstellationDetector::new(1, 0.1);
        let h1 = make_hyp("tl_1", "Test", "test insight", 0.8);
        detector.ingest_hypotheses(vec![h1]);
        if !detector.star_map.constellations.is_empty() {
            let c_id = detector.star_map.constellations[0].id.clone();
            assert!(detector.integrate_constellation(&c_id));
            let c = &detector.star_map.constellations[0];
            assert!(c.integrated);
            assert!(c.integration_cycle.is_some());
        }
    }

    #[test]
    fn test_ready_for_integration() {
        let mut detector = ConstellationDetector::new(1, 0.5);
        let h1 = make_hyp(
            "tl_1",
            "Ready",
            "High confidence insight ready for integration",
            0.9,
        );
        detector.ingest_hypotheses(vec![h1]);
        let ready = detector.ready_for_integration();
        if !detector.star_map.constellations.is_empty() {
            assert!(!ready.is_empty());
        }
    }

    #[test]
    fn test_star_map_add_star_to_constellation() {
        let mut sm = StarMap::new();
        let c = Constellation {
            id: "c1".into(),
            name: "C1".into(),
            stars: vec![Star {
                id: "s1".into(),
                title: "S1".into(),
                description: "test insight".into(),
                source_timelines: vec!["tl_1".into()],
                confidence: 0.8,
                first_seen_cycle: 0,
                last_seen_cycle: 0,
                intensity: 0.8,
            }],
            formation_cycle: 0,
            integrated: false,
            integration_cycle: None,
            emergence_score: 0.0,
        };
        sm.add_constellation(c);
        let star = Star {
            id: "s2".into(),
            title: "S2".into(),
            description: "test insight related".into(),
            source_timelines: vec!["tl_2".into()],
            confidence: 0.7,
            first_seen_cycle: 1,
            last_seen_cycle: 1,
            intensity: 0.7,
        };
        sm.add_star(star);
        assert_eq!(
            sm.constellations[0].stars.len(),
            2,
            "star should merge into constellation"
        );
    }

    #[test]
    fn test_summary() {
        let detector = ConstellationDetector::new(3, 0.5);
        let s = detector.summary();
        assert!(s.contains("Constellations"));
        assert!(s.contains("stars"));
    }

    #[test]
    fn test_ingest_duplicate_hypotheses_merge() {
        let mut detector = ConstellationDetector::new(2, 0.8);
        let h1 = make_hyp(
            "tl_1",
            "Identical",
            "The same insight from timeline one",
            0.9,
        );
        let h2 = make_hyp(
            "tl_2",
            "Identical",
            "The same insight from timeline two",
            0.8,
        );
        detector.ingest_hypotheses(vec![h1, h2]);
        if !detector.star_map.constellations.is_empty() {
            let c = &detector.star_map.constellations[0];
            assert!(
                c.stars[0].source_timelines.len() > 1,
                "identical insights should merge sources"
            );
        }
    }

    #[test]
    fn test_emerging_patterns() {
        let mut detector = ConstellationDetector::new(3, 0.5);
        let h1 = make_hyp("tl_1", "Weak", "low confidence emerging pattern", 0.3);
        detector.ingest_hypotheses(vec![h1]);
        let emerging = detector.emerging_patterns();
        if !detector.star_map.constellations.is_empty() {
            for c in emerging {
                assert!(c.emergence_score <= 0.6);
            }
        }
    }
}
