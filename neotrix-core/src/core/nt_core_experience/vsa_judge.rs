use crate::core::nt_core_hcube::QuantizedVSA;

#[derive(Debug, Clone)]
pub struct ConsensusRegion {
    pub centroid: Vec<u8>,
    pub support_count: usize,
    pub coherence: f64,
    pub member_ids: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct Contradiction {
    pub region_a: Vec<u8>,
    pub region_b: Vec<u8>,
    pub divergence: f64,
    pub chain_ids: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct CoverageGap {
    pub query_region: Vec<u8>,
    pub gap_magnitude: f64,
}

#[derive(Debug, Clone)]
pub struct UniqueInsight {
    pub chain_id: usize,
    pub vector: Vec<u8>,
    pub novelty: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct BlindSpotReport {
    pub overall_uncertainty: f64,
    pub low_confidence_regions: Vec<Vec<u8>>,
    pub panel_diversity: f64,
}

#[derive(Debug, Clone)]
pub struct JudgeAnalysis {
    pub consensus: Option<ConsensusRegion>,
    pub contradictions: Vec<Contradiction>,
    pub coverage_gaps: Vec<CoverageGap>,
    pub unique_insights: Vec<UniqueInsight>,
    pub blind_spots: BlindSpotReport,
    pub overall_confidence: f64,
    pub panel_diversity: f64,
    pub n_contributors: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum DeliberationOutcome {
    Recommendation,
    Alternatives,
    Question,
    Investigate,
}

impl DeliberationOutcome {
    pub fn label(&self) -> &'static str {
        match self {
            DeliberationOutcome::Recommendation => "recommendation",
            DeliberationOutcome::Alternatives => "alternatives",
            DeliberationOutcome::Question => "question",
            DeliberationOutcome::Investigate => "investigate",
        }
    }
}

impl JudgeAnalysis {
    pub fn has_strong_consensus(&self) -> bool {
        self.consensus.as_ref().map_or(false, |c| {
            c.coherence > 0.7 && c.support_count as f64 / self.n_contributors as f64 > 0.5
        })
    }

    pub fn has_critical_contradictions(&self) -> bool {
        self.contradictions.iter().any(|c| c.divergence > 0.6)
    }

    pub fn consensus_ratio(&self) -> f64 {
        self.consensus.as_ref().map_or(0.0, |c| {
            c.support_count as f64 / self.n_contributors.max(1) as f64
        })
    }

    pub fn contradiction_intensity(&self) -> f64 {
        self.contradictions
            .iter()
            .map(|c| c.divergence)
            .sum::<f64>()
            / self.n_contributors.max(1) as f64
    }

    pub fn recommended_outcome(&self) -> DeliberationOutcome {
        if self.has_strong_consensus() && !self.has_critical_contradictions() {
            DeliberationOutcome::Recommendation
        } else if self.has_critical_contradictions() && self.contradiction_intensity() > 0.3 {
            DeliberationOutcome::Alternatives
        } else if !self.coverage_gaps.is_empty() {
            DeliberationOutcome::Question
        } else {
            DeliberationOutcome::Investigate
        }
    }
}

#[derive(Debug, Clone)]
pub struct VSAJudge {
    pub consensus_threshold: f64,
    pub contradiction_threshold: f64,
    pub novelty_threshold: f64,
    pub coverage_threshold: f64,
}

impl Default for VSAJudge {
    fn default() -> Self {
        Self {
            consensus_threshold: 0.70,
            contradiction_threshold: 0.45,
            novelty_threshold: 0.55,
            coverage_threshold: 0.35,
        }
    }
}

impl VSAJudge {
    pub fn new(
        consensus_threshold: f64,
        contradiction_threshold: f64,
        novelty_threshold: f64,
    ) -> Self {
        Self {
            consensus_threshold,
            contradiction_threshold,
            novelty_threshold,
            coverage_threshold: 0.35,
        }
    }

    pub fn analyze(&self, query: &[u8], results: &[PanelResult]) -> JudgeAnalysis {
        let n = results.len();
        if n == 0 {
            return JudgeAnalysis {
                consensus: None,
                contradictions: vec![],
                coverage_gaps: vec![],
                unique_insights: vec![],
                blind_spots: BlindSpotReport {
                    overall_uncertainty: 1.0,
                    low_confidence_regions: vec![],
                    panel_diversity: 0.0,
                },
                overall_confidence: 0.0,
                panel_diversity: 0.0,
                n_contributors: 0,
            };
        }

        let vectors: Vec<&[u8]> = results
            .iter()
            .map(|r| r.thought_vector.as_slice())
            .collect();

        let similarity_matrix = self.compute_similarity_matrix(&vectors);
        let panel_diversity = self.compute_diversity(&similarity_matrix);

        let consensus = self.detect_consensus(&vectors, &similarity_matrix, results);
        let contradictions = self.detect_contradictions(&vectors, &similarity_matrix, results);
        let unique_insights = self.detect_unique_insights(&vectors, &similarity_matrix, results);

        let coverage_gaps = self.detect_coverage_gaps(query, &vectors, results);

        let avg_confidence = results.iter().map(|r| r.confidence).sum::<f64>() / n as f64;
        let blind_spots = self.assess_blind_spots(results, &vectors, &similarity_matrix);

        let overall_confidence = consensus.as_ref().map_or(avg_confidence * 0.5, |c| {
            avg_confidence * 0.3 + c.coherence * 0.4 + c.support_count as f64 / n as f64 * 0.3
        });

        JudgeAnalysis {
            consensus,
            contradictions,
            coverage_gaps,
            unique_insights,
            blind_spots,
            overall_confidence,
            panel_diversity,
            n_contributors: n,
        }
    }

    fn compute_similarity_matrix(&self, vectors: &[&[u8]]) -> Vec<Vec<f64>> {
        let n = vectors.len();
        let mut mat = vec![vec![0.0; n]; n];
        for i in 0..n {
            for j in (i + 1)..n {
                let sim = QuantizedVSA::cosine(vectors[i], vectors[j]);
                mat[i][j] = sim;
                mat[j][i] = sim;
            }
            mat[i][i] = 1.0;
        }
        mat
    }

    fn compute_diversity(&self, sim_matrix: &[Vec<f64>]) -> f64 {
        let n = sim_matrix.len();
        if n < 2 {
            return 0.0;
        }
        let mut total = 0.0;
        let mut count = 0;
        for i in 0..n {
            for j in (i + 1)..n {
                total += 1.0 - sim_matrix[i][j];
                count += 1;
            }
        }
        total / count as f64
    }

    fn detect_consensus(
        &self,
        vectors: &[&[u8]],
        sim_matrix: &[Vec<f64>],
        _results: &[PanelResult],
    ) -> Option<ConsensusRegion> {
        let n = vectors.len();
        if n == 0 {
            return None;
        }

        let mut best_cluster: Vec<usize> = vec![];
        let mut best_coherence = 0.0;

        for i in 0..n {
            let mut cluster = vec![i];
            for j in 0..n {
                if i != j && sim_matrix[i][j] >= self.consensus_threshold {
                    cluster.push(j);
                }
            }
            if cluster.len() < 2 {
                continue;
            }
            let cluster_sims: Vec<f64> = cluster
                .iter()
                .flat_map(|&a| {
                    cluster.iter().filter_map(
                        move |&b| {
                            if a < b {
                                Some(sim_matrix[a][b])
                            } else {
                                None
                            }
                        },
                    )
                })
                .collect();
            let avg_sim = if cluster_sims.is_empty() {
                0.0
            } else {
                cluster_sims.iter().sum::<f64>() / cluster_sims.len() as f64
            };
            if cluster.len() > best_cluster.len()
                || (cluster.len() == best_cluster.len() && avg_sim > best_coherence)
            {
                best_cluster = cluster;
                best_coherence = avg_sim;
            }
        }

        if best_cluster.len() < 2 {
            return None;
        }

        let dim = vectors[0].len();
        let mut centroid = vec![0u64; dim];
        for &idx in &best_cluster {
            for (c, &v) in centroid.iter_mut().zip(vectors[idx].iter()) {
                *c += v as u64;
            }
        }
        let threshold = (best_cluster.len() as u64 + 1) / 2;
        let centroid: Vec<u8> = centroid
            .iter()
            .map(|&c| if c >= threshold { 1 } else { 0 })
            .collect();

        Some(ConsensusRegion {
            centroid,
            support_count: best_cluster.len(),
            coherence: best_coherence,
            member_ids: best_cluster,
        })
    }

    fn detect_contradictions(
        &self,
        vectors: &[&[u8]],
        sim_matrix: &[Vec<f64>],
        results: &[PanelResult],
    ) -> Vec<Contradiction> {
        let n = vectors.len();
        let mut contradictions = vec![];
        for i in 0..n {
            for j in (i + 1)..n {
                let divergence = 1.0 - sim_matrix[i][j];
                if divergence >= self.contradiction_threshold {
                    contradictions.push(Contradiction {
                        region_a: vectors[i].to_vec(),
                        region_b: vectors[j].to_vec(),
                        divergence,
                        chain_ids: vec![results[i].chain_id, results[j].chain_id],
                    });
                }
            }
        }
        contradictions.sort_by(|a, b| {
            b.divergence
                .partial_cmp(&a.divergence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        contradictions.truncate(5);
        contradictions
    }

    fn detect_unique_insights(
        &self,
        vectors: &[&[u8]],
        sim_matrix: &[Vec<f64>],
        results: &[PanelResult],
    ) -> Vec<UniqueInsight> {
        let n = vectors.len();
        let mut insights = vec![];
        for i in 0..n {
            let avg_sim_to_others: f64 = sim_matrix[i]
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(_, s)| s)
                .sum::<f64>()
                / (n - 1).max(1) as f64;
            let novelty = 1.0 - avg_sim_to_others;
            if novelty >= self.novelty_threshold && results[i].confidence > 0.3 {
                insights.push(UniqueInsight {
                    chain_id: results[i].chain_id,
                    vector: vectors[i].to_vec(),
                    novelty,
                    confidence: results[i].confidence,
                });
            }
        }
        insights.sort_by(|a, b| {
            (b.novelty * b.confidence)
                .partial_cmp(&(a.novelty * a.confidence))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        insights.truncate(3);
        insights
    }

    fn detect_coverage_gaps(
        &self,
        query: &[u8],
        vectors: &[&[u8]],
        _results: &[PanelResult],
    ) -> Vec<CoverageGap> {
        let mut gaps = vec![];
        let query_sim_to_panel: Vec<f64> = vectors
            .iter()
            .map(|v| QuantizedVSA::cosine(query, v))
            .collect();
        let max_coverage = query_sim_to_panel.iter().cloned().fold(0.0, f64::max);
        if max_coverage < self.coverage_threshold {
            gaps.push(CoverageGap {
                query_region: query.to_vec(),
                gap_magnitude: 1.0 - max_coverage,
            });
        }
        gaps
    }

    fn assess_blind_spots(
        &self,
        results: &[PanelResult],
        _vectors: &[&[u8]],
        sim_matrix: &[Vec<f64>],
    ) -> BlindSpotReport {
        let n = results.len();
        let avg_confidence = results.iter().map(|r| r.confidence).sum::<f64>() / n.max(1) as f64;
        let diversity = self.compute_diversity(sim_matrix);
        let overall_uncertainty = 1.0 - avg_confidence * (1.0 - diversity * 0.3);

        let low_conf_regions: Vec<Vec<u8>> = results
            .iter()
            .filter(|r| r.confidence < 0.3)
            .map(|r| r.thought_vector.clone())
            .collect();

        BlindSpotReport {
            overall_uncertainty,
            low_confidence_regions: low_conf_regions,
            panel_diversity: diversity,
        }
    }

    pub fn synthesis_by_consensus(
        &self,
        analysis: &JudgeAnalysis,
        results: &[PanelResult],
    ) -> Vec<u8> {
        if let Some(ref consensus) = analysis.consensus {
            let mut members: Vec<&[u8]> = consensus
                .member_ids
                .iter()
                .filter_map(|&idx| results.get(idx))
                .map(|r| r.thought_vector.as_slice())
                .collect();
            if analysis.unique_insights.len() >= 2 {
                for insight in &analysis.unique_insights[..2.min(analysis.unique_insights.len())] {
                    if let Some(r) = results.iter().find(|r| r.chain_id == insight.chain_id) {
                        members.push(r.thought_vector.as_slice());
                    }
                }
            }
            QuantizedVSA::bundle(&members)
        } else {
            let all: Vec<&[u8]> = results
                .iter()
                .map(|r| r.thought_vector.as_slice())
                .collect();
            QuantizedVSA::bundle(&all)
        }
    }
}

#[derive(Debug, Clone)]
pub struct PanelResult {
    pub chain_id: usize,
    pub thought_vector: Vec<u8>,
    pub confidence: f64,
    pub reasoning_label: String,
    pub execution_time_ns: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hcube::QuantizedVSA;

    fn make_result(chain_id: usize, seed: u64, confidence: f64) -> PanelResult {
        PanelResult {
            chain_id,
            thought_vector: QuantizedVSA::seeded_random(seed, 4096),
            confidence,
            reasoning_label: format!("chain_{}", chain_id),
            execution_time_ns: 0,
        }
    }

    #[test]
    fn test_judge_empty_panel() {
        let judge = VSAJudge::default();
        let query = QuantizedVSA::random_binary();
        let analysis = judge.analyze(&query, &[]);
        assert!(analysis.consensus.is_none());
        assert_eq!(analysis.n_contributors, 0);
        assert_eq!(analysis.overall_confidence, 0.0);
    }

    #[test]
    fn test_judge_single_result() {
        let judge = VSAJudge::default();
        let query = QuantizedVSA::random_binary();
        let results = vec![make_result(0, 42, 0.8)];
        let analysis = judge.analyze(&query, &results);
        assert!(analysis.consensus.is_none());
        assert!(analysis.contradictions.is_empty());
        assert_eq!(analysis.n_contributors, 1);
    }

    #[test]
    fn test_judge_consensus_detection() {
        let judge = VSAJudge::default();
        let query = QuantizedVSA::random_binary();
        let common = QuantizedVSA::seeded_random(100, 4096);
        let results = vec![
            PanelResult {
                thought_vector: common.clone(),
                confidence: 0.8,
                ..make_result(0, 1, 0.8)
            },
            PanelResult {
                thought_vector: common.clone(),
                confidence: 0.7,
                ..make_result(1, 2, 0.7)
            },
            PanelResult {
                thought_vector: common.clone(),
                confidence: 0.9,
                ..make_result(2, 3, 0.9)
            },
            PanelResult {
                thought_vector: QuantizedVSA::seeded_random(200, 4096),
                confidence: 0.5,
                ..make_result(3, 4, 0.5)
            },
        ];
        let analysis = judge.analyze(&query, &results);
        assert!(analysis.consensus.is_some());
        let c = analysis.consensus.unwrap();
        assert!(c.support_count >= 3);
        assert!(c.coherence > 0.9);
    }

    #[test]
    fn test_judge_contradiction_detection() {
        let judge = VSAJudge::new(0.75, 0.20, 0.55);
        let query = QuantizedVSA::random_binary();
        let v1 = QuantizedVSA::seeded_random(10, 4096);
        let v2 = QuantizedVSA::seeded_random(9999, 4096);
        let results = vec![
            PanelResult {
                thought_vector: v1,
                confidence: 0.8,
                ..make_result(0, 1, 0.8)
            },
            PanelResult {
                thought_vector: v2,
                confidence: 0.7,
                ..make_result(1, 2, 0.7)
            },
        ];
        let analysis = judge.analyze(&query, &results);
        assert!(analysis.contradictions.len() >= 1 || analysis.panel_diversity > 0.3);
    }

    #[test]
    fn test_judge_unique_insight_detection() {
        let judge = VSAJudge::new(0.75, 0.45, 0.30);
        let query = QuantizedVSA::random_binary();
        let common = QuantizedVSA::seeded_random(100, 4096);
        let unique = QuantizedVSA::seeded_random(9999, 4096);
        let results = vec![
            PanelResult {
                thought_vector: common.clone(),
                confidence: 0.8,
                ..make_result(0, 1, 0.8)
            },
            PanelResult {
                thought_vector: common.clone(),
                confidence: 0.7,
                ..make_result(1, 2, 0.7)
            },
            PanelResult {
                thought_vector: unique,
                confidence: 0.6,
                ..make_result(2, 3, 0.6)
            },
        ];
        let analysis = judge.analyze(&query, &results);
        assert!(!analysis.unique_insights.is_empty());
    }

    #[test]
    fn test_judge_coverage_gap() {
        let judge = VSAJudge {
            coverage_threshold: 0.50,
            ..Default::default()
        };
        let query = QuantizedVSA::seeded_random(42, 4096);
        let far = QuantizedVSA::seeded_random(9999, 4096);
        let results = vec![PanelResult {
            thought_vector: far,
            confidence: 0.5,
            ..make_result(0, 1, 0.5)
        }];
        let analysis = judge.analyze(&query, &results);
        assert!(
            !analysis.coverage_gaps.is_empty() || analysis.blind_spots.overall_uncertainty > 0.5
        );
    }

    #[test]
    fn test_judge_blind_spots() {
        let judge = VSAJudge::default();
        let query = QuantizedVSA::random_binary();
        let results = vec![make_result(0, 1, 0.2), make_result(1, 2, 0.3)];
        let analysis = judge.analyze(&query, &results);
        assert!(analysis.blind_spots.overall_uncertainty > 0.5);
        assert!(analysis.blind_spots.panel_diversity >= 0.0);
    }

    #[test]
    fn test_synthesis_by_consensus() {
        let judge = VSAJudge::default();
        let query = QuantizedVSA::random_binary();
        let common = QuantizedVSA::seeded_random(100, 4096);
        let results = vec![
            PanelResult {
                thought_vector: common.clone(),
                confidence: 0.8,
                ..make_result(0, 1, 0.8)
            },
            PanelResult {
                thought_vector: common.clone(),
                confidence: 0.7,
                ..make_result(1, 2, 0.7)
            },
            PanelResult {
                thought_vector: QuantizedVSA::seeded_random(200, 4096),
                confidence: 0.5,
                ..make_result(2, 3, 0.5)
            },
        ];
        let analysis = judge.analyze(&query, &results);
        let synthesis = judge.synthesis_by_consensus(&analysis, &results);
        assert_eq!(synthesis.len(), 4096);
        assert!(QuantizedVSA::similarity(&synthesis, &common) > 0.5);
    }

    #[test]
    fn test_judge_consensus_ratio() {
        let judge = VSAJudge::default();
        let query = QuantizedVSA::random_binary();
        let common = QuantizedVSA::seeded_random(100, 4096);
        let results = vec![
            PanelResult {
                thought_vector: common.clone(),
                confidence: 0.8,
                ..make_result(0, 1, 0.8)
            },
            PanelResult {
                thought_vector: common.clone(),
                confidence: 0.7,
                ..make_result(1, 2, 0.7)
            },
            PanelResult {
                thought_vector: QuantizedVSA::seeded_random(200, 4096),
                confidence: 0.5,
                ..make_result(2, 3, 0.5)
            },
            PanelResult {
                thought_vector: QuantizedVSA::seeded_random(201, 4096),
                confidence: 0.5,
                ..make_result(3, 4, 0.5)
            },
        ];
        let analysis = judge.analyze(&query, &results);
        assert!(analysis.consensus_ratio() > 0.0);
        assert!(analysis.contradiction_intensity() >= 0.0);
    }

    #[test]
    fn test_judge_has_strong_consensus() {
        let judge = VSAJudge::default();
        let query = QuantizedVSA::random_binary();
        let common = QuantizedVSA::seeded_random(100, 4096);
        let results = vec![
            PanelResult {
                thought_vector: common.clone(),
                confidence: 0.9,
                ..make_result(0, 1, 0.9)
            },
            PanelResult {
                thought_vector: common.clone(),
                confidence: 0.9,
                ..make_result(1, 2, 0.9)
            },
            PanelResult {
                thought_vector: common.clone(),
                confidence: 0.9,
                ..make_result(2, 3, 0.9)
            },
        ];
        let analysis = judge.analyze(&query, &results);
        assert!(analysis.has_strong_consensus());
        assert!(!analysis.has_critical_contradictions());
    }

    #[test]
    fn test_recommended_outcome_consensus() {
        let judge = VSAJudge::default();
        let query = QuantizedVSA::random_binary();
        let common = QuantizedVSA::seeded_random(100, 4096);
        let results = vec![
            PanelResult {
                thought_vector: common.clone(),
                confidence: 0.9,
                ..make_result(0, 1, 0.9)
            },
            PanelResult {
                thought_vector: common.clone(),
                confidence: 0.9,
                ..make_result(1, 2, 0.9)
            },
            PanelResult {
                thought_vector: common.clone(),
                confidence: 0.9,
                ..make_result(2, 3, 0.9)
            },
        ];
        let analysis = judge.analyze(&query, &results);
        assert_eq!(
            analysis.recommended_outcome(),
            DeliberationOutcome::Recommendation
        );
    }

    #[test]
    fn test_recommended_outcome_low_coverage() {
        let judge = VSAJudge {
            coverage_threshold: 0.70,
            ..Default::default()
        };
        let query = QuantizedVSA::seeded_random(42, 4096);
        let far = QuantizedVSA::seeded_random(9999, 4096);
        let results = vec![PanelResult {
            thought_vector: far,
            confidence: 0.5,
            ..make_result(0, 1, 0.5)
        }];
        let analysis = judge.analyze(&query, &results);
        let outcome = analysis.recommended_outcome();
        assert!(
            outcome == DeliberationOutcome::Question || outcome == DeliberationOutcome::Investigate
        );
    }
}
