use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Judgment {
    LikelyWorks,
    Uncertain,
    LikelyFails,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IntuitionHeuristic {
    SimilarityToPastSuccess,
    ComplexityPenalty,
    NoveltyBonus,
    AnalogicalTransfer,
}

#[derive(Debug, Clone)]
pub struct PastExperience {
    pub id: u64,
    pub description: String,
    pub features: Vec<f64>,
    pub outcome_score: f64,
    pub timestamp: u64,
    pub domain: String,
}

#[derive(Debug, Clone)]
pub struct IntuitionSignal {
    pub confidence: f64,
    pub judgment: Judgment,
    pub supporting_evidence: Vec<String>,
    pub heuristic: IntuitionHeuristic,
    pub response_time_estimate: u64,
}

pub struct ResearchIntuition {
    experiences: Vec<PastExperience>,
    max_experiences: usize,
    similarity_threshold: f64,
    novelty_weight: f64,
    #[allow(dead_code)]
    recency_decay: f64,
    domain_boost: HashMap<String, f64>,
    next_id: u64,
}

impl ResearchIntuition {
    pub fn new() -> Self {
        Self {
            experiences: Vec::new(),
            max_experiences: 500,
            similarity_threshold: 0.6,
            novelty_weight: 0.3,
            recency_decay: 0.95,
            domain_boost: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn record_experience(
        &mut self,
        description: &str,
        features: &[f64],
        outcome: f64,
        domain: &str,
    ) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let exp = PastExperience {
            id: self.next_id,
            description: description.to_string(),
            features: features.to_vec(),
            outcome_score: outcome,
            timestamp,
            domain: domain.to_string(),
        };
        self.next_id += 1;

        let boost = self.domain_boost.entry(domain.to_string()).or_insert(0.0);
        *boost = (*boost * 0.9) + (outcome * 0.1);

        self.experiences.push(exp);

        if self.experiences.len() > self.max_experiences {
            self.experiences.remove(0);
        }
    }

    pub fn similarity(&self, a: &[f64], b: &[f64]) -> f64 {
        let n = a.len().min(b.len());
        if n == 0 {
            return 0.0;
        }
        let dot: f64 = a[..n].iter().zip(b[..n].iter()).map(|(x, y)| x * y).sum();
        let norm_a: f64 = a[..n].iter().map(|x| x * x).sum::<f64>().sqrt();
        let norm_b: f64 = b[..n].iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            (dot / (norm_a * norm_b)).clamp(0.0, 1.0)
        }
    }

    pub fn nearest_experiences(&self, features: &[f64], k: usize) -> Vec<&PastExperience> {
        let mut scored: Vec<(f64, usize)> = self
            .experiences
            .iter()
            .enumerate()
            .map(|(i, e)| (self.similarity(features, &e.features), i))
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(k);
        scored
            .into_iter()
            .map(|(_, i)| &self.experiences[i])
            .collect()
    }

    pub fn domain_expertise(&self, domain: &str) -> f64 {
        let domain_exps: Vec<&PastExperience> = self
            .experiences
            .iter()
            .filter(|e| e.domain == domain)
            .collect();
        if domain_exps.is_empty() {
            return 0.0;
        }
        let raw_avg =
            domain_exps.iter().map(|e| e.outcome_score).sum::<f64>() / domain_exps.len() as f64;
        let boost = self.domain_boost.get(domain).copied().unwrap_or(0.0);
        ((raw_avg + boost) / 2.0).clamp(0.0, 1.0)
    }

    pub fn analogical_transfer(
        &self,
        source_features: &[f64],
        source_outcome: f64,
        target_features: &[f64],
    ) -> f64 {
        let sim = self.similarity(source_features, target_features);
        sim * source_outcome * 0.5 + 0.25
    }

    pub fn heuristic_blend(&self, features: &[f64], domain: &str) -> IntuitionSignal {
        let nearest = self.nearest_experiences(features, 3);
        let mut evidence = Vec::new();
        let mut similarity_scores = Vec::new();

        for exp in &nearest {
            let sim = self.similarity(features, &exp.features);
            similarity_scores.push((sim, exp.outcome_score, &exp.description));
        }

        let sim_judgment = if similarity_scores.is_empty() {
            Judgment::Uncertain
        } else {
            let weighted_outcome: f64 = similarity_scores
                .iter()
                .map(|(sim, score, _)| sim * score)
                .sum();
            let total_sim: f64 = similarity_scores.iter().map(|(sim, _, _)| sim).sum();
            let avg = if total_sim > 0.0 {
                weighted_outcome / total_sim
            } else {
                0.5
            };
            let best_sim = similarity_scores
                .iter()
                .map(|(s, _, _)| *s)
                .fold(0.0_f64, f64::max);
            if best_sim >= self.similarity_threshold && avg > 0.6 {
                evidence.push(format!("similar past success (sim={:.3})", best_sim));
                Judgment::LikelyWorks
            } else if best_sim >= self.similarity_threshold && avg < 0.3 {
                evidence.push(format!("similar past failure (sim={:.3})", best_sim));
                Judgment::LikelyFails
            } else {
                evidence.push(format!(
                    "mixed signals (best_sim={:.3}, avg_outcome={:.3})",
                    best_sim, avg
                ));
                Judgment::Uncertain
            }
        };

        let n = features.len() as f64;
        let complexity_penalty = if n > 100.0 { (n - 100.0) / 900.0 } else { 0.0 };
        if complexity_penalty > 0.3 {
            evidence.push(format!("high complexity ({} features)", n as u64));
        }

        let domain_exp = self.domain_expertise(domain);
        evidence.push(format!("domain expertise: {:.3}", domain_exp));

        if nearest.len() < 3 {
            let novelty_gap = (3 - nearest.len()) as f64 / 3.0;
            evidence.push(format!("novelty bonus: {:.3}", novelty_gap));
        }

        let (chosen_heuristic, blended_confidence, final_judgment) = match sim_judgment {
            Judgment::LikelyWorks => {
                if self.novelty_weight > 0.2 && nearest.len() < 3 {
                    (
                        IntuitionHeuristic::NoveltyBonus,
                        0.55 + domain_exp * 0.3,
                        Judgment::LikelyWorks,
                    )
                } else {
                    (
                        IntuitionHeuristic::SimilarityToPastSuccess,
                        0.6 + domain_exp * 0.35,
                        Judgment::LikelyWorks,
                    )
                }
            }
            Judgment::LikelyFails => (
                IntuitionHeuristic::ComplexityPenalty,
                (0.7 - complexity_penalty * 0.5).max(0.1),
                Judgment::LikelyFails,
            ),
            Judgment::Uncertain => {
                if !nearest.is_empty() && similarity_scores.iter().any(|(s, _, _)| *s > 0.3) {
                    (IntuitionHeuristic::AnalogicalTransfer, 0.4, sim_judgment)
                } else {
                    (IntuitionHeuristic::NoveltyBonus, 0.3, sim_judgment)
                }
            }
        };

        let confidence = blended_confidence.clamp(0.0, 1.0);
        let rt_estimate = if let IntuitionHeuristic::AnalogicalTransfer = chosen_heuristic {
            500 + (n as u64 / 5)
        } else {
            100 + (n as u64 / 20)
        };

        IntuitionSignal {
            confidence,
            judgment: final_judgment,
            supporting_evidence: evidence,
            heuristic: chosen_heuristic,
            response_time_estimate: rt_estimate,
        }
    }

    pub fn judge(&self, _description: &str, features: &[f64], domain: &str) -> IntuitionSignal {
        self.heuristic_blend(features, domain)
    }

    pub fn confidence_calibration(&self, predicted: f64, actual: f64) -> f64 {
        1.0 - (predicted - actual).abs()
    }

    pub fn top_domains(&self, k: usize) -> Vec<(String, usize)> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for exp in &self.experiences {
            *counts.entry(exp.domain.clone()).or_insert(0) += 1;
        }
        let mut sorted: Vec<(String, usize)> = counts.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.truncate(k);
        sorted
    }

    pub fn intuition_summary(&self) -> String {
        let total = self.experiences.len();
        let avg_outcome: f64 = if total > 0 {
            self.experiences
                .iter()
                .map(|e| e.outcome_score)
                .sum::<f64>()
                / total as f64
        } else {
            0.0
        };
        let top = self.top_domains(3);
        let domains_str: Vec<String> = top
            .into_iter()
            .map(|(d, c)| format!("{}:{}", d, c))
            .collect();
        format!(
            "ResearchIntuition(experiences={}, avg_outcome={:.3}, domains=[{}])",
            total,
            avg_outcome,
            domains_str.join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_new_research_intuition() {
        let ri = ResearchIntuition::new();
        assert_eq!(ri.max_experiences, 500);
        assert_eq!(ri.similarity_threshold, 0.6);
        assert!(ri.experiences.is_empty());
    }

    #[test]
    fn test_record_experience() {
        let mut ri = ResearchIntuition::new();
        ri.record_experience("test attempt", &[0.5, 0.3, 0.8], 0.9, "testing");
        assert_eq!(ri.experiences.len(), 1);
        assert_eq!(ri.experiences[0].description, "test attempt");
        assert_eq!(ri.experiences[0].outcome_score, 0.9);
        assert_eq!(ri.experiences[0].domain, "testing");
    }

    #[test]
    fn test_similarity_identical() {
        let ri = ResearchIntuition::new();
        let v = vec![0.5, 0.3, 0.8];
        let sim = ri.similarity(&v, &v);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_similarity_orthogonal() {
        let ri = ResearchIntuition::new();
        let sim = ri.similarity(&[1.0, 0.0], &[0.0, 1.0]);
        assert!((sim - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_similarity_dimension_mismatch() {
        let ri = ResearchIntuition::new();
        let sim = ri.similarity(&[1.0, 0.0], &[1.0, 0.0, 1.0]);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_nearest_experiences_returns_closest() {
        let mut ri = ResearchIntuition::new();
        ri.record_experience("far", &[1.0, 0.0, 0.0], 0.5, "math");
        ri.record_experience("close", &[0.9, 0.1, 0.0], 0.8, "math");
        ri.record_experience("middle", &[0.5, 0.5, 0.0], 0.3, "math");
        let nearest = ri.nearest_experiences(&[1.0, 0.0, 0.0], 2);
        assert_eq!(nearest.len(), 2);
        assert_eq!(nearest[0].description, "close");
    }

    #[test]
    fn test_domain_expertise_no_experiences() {
        let ri = ResearchIntuition::new();
        assert_eq!(ri.domain_expertise("physics"), 0.0);
    }

    #[test]
    fn test_domain_expertise_with_experiences() {
        let mut ri = ResearchIntuition::new();
        ri.record_experience("e1", &[0.1], 0.8, "physics");
        ri.record_experience("e2", &[0.2], 0.6, "physics");
        ri.record_experience("e3", &[0.3], 0.4, "chemistry");
        let phys_exp = ri.domain_expertise("physics");
        assert!(phys_exp > 0.0);
        assert!(phys_exp <= 1.0);
    }

    #[test]
    fn test_judge_returns_intuition_signal() {
        let mut ri = ResearchIntuition::new();
        ri.record_experience("previous success", &[0.8, 0.7, 0.9], 0.9, "code");
        let signal = ri.judge("new idea", &[0.75, 0.72, 0.85], "code");
        assert!(signal.confidence >= 0.0);
        assert!(signal.confidence <= 1.0);
        assert!(!signal.supporting_evidence.is_empty());
    }

    #[test]
    fn test_judge_uncertain_with_no_experience() {
        let ri = ResearchIntuition::new();
        let signal = ri.judge("brand new", &[0.1, 0.2, 0.3], "unknown");
        assert_eq!(signal.judgment, Judgment::Uncertain);
    }

    #[test]
    fn test_analogical_transfer_positive() {
        let ri = ResearchIntuition::new();
        let transfer = ri.analogical_transfer(&[0.9, 0.8], 0.9, &[0.85, 0.75]);
        assert!(transfer > 0.0);
        assert!(transfer <= 1.0);
    }

    #[test]
    fn test_confidence_calibration_perfect() {
        let ri = ResearchIntuition::new();
        let cal = ri.confidence_calibration(0.8, 0.8);
        assert!((cal - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_confidence_calibration_off() {
        let ri = ResearchIntuition::new();
        let cal = ri.confidence_calibration(0.9, 0.3);
        assert!((cal - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_top_domains() {
        let mut ri = ResearchIntuition::new();
        ri.record_experience("a", &[0.1], 0.5, "math");
        ri.record_experience("b", &[0.2], 0.5, "physics");
        ri.record_experience("c", &[0.3], 0.5, "math");
        let top = ri.top_domains(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].0, "math");
        assert_eq!(top[0].1, 2);
    }

    #[test]
    fn test_intuition_summary() {
        let mut ri = ResearchIntuition::new();
        ri.record_experience("a", &[0.1], 0.8, "math");
        let summary = ri.intuition_summary();
        assert!(summary.contains("experiences=1"));
        assert!(summary.contains("avg_outcome=0.800"));
        assert!(summary.contains("math"));
    }

    #[test]
    fn test_max_experiences_trims_oldest() {
        let mut ri = ResearchIntuition::new();
        ri.max_experiences = 2;
        ri.record_experience("first", &[0.1], 0.5, "a");
        ri.record_experience("second", &[0.2], 0.5, "a");
        ri.record_experience("third", &[0.3], 0.5, "a");
        assert_eq!(ri.experiences.len(), 2);
        assert_eq!(ri.experiences[0].description, "second");
    }

    #[test]
    fn test_domain_boost_updates_on_record() {
        let mut ri = ResearchIntuition::new();
        ri.record_experience("a", &[0.1], 0.9, "vision");
        assert!(ri.domain_boost.contains_key("vision"));
        let boost = ri.domain_boost["vision"];
        assert!((boost - 0.09).abs() < 1e-6);
    }

    #[test]
    fn test_heuristic_blend_novelty_on_empty() {
        let ri = ResearchIntuition::new();
        let signal = ri.heuristic_blend(&[0.1, 0.2], "new_domain");
        assert_eq!(signal.heuristic, IntuitionHeuristic::NoveltyBonus);
    }

    #[test]
    fn test_response_time_scales_with_features() {
        let ri = ResearchIntuition::new();
        let signal_small = ri.judge("small", &[0.1], "x");
        let signal_large = ri.judge("large", &[0.1; 200], "x");
        assert!(signal_large.response_time_estimate > signal_small.response_time_estimate);
    }
}
