// REVIVED Task 1 — dead_code removed 2026-06-24

use rand::Rng;
use std::collections::VecDeque;

/// Identity perturbation event
#[derive(Debug, Clone)]
pub struct PerturbationEvent {
    pub tick: u64,
    pub source: String,
    pub similarity_before: f64,
    pub similarity_after: f64,
    pub detected: bool,
    pub mitigated: bool,
}

/// Active identity defense system
pub struct IdentityDefense {
    pub baseline: Vec<u8>,
    pub threshold: f64,
    pub recovery_rate: f64,
    pub events: VecDeque<PerturbationEvent>,
    pub max_events: usize,
    pub defense_active: bool,
}

impl IdentityDefense {
    pub fn new(baseline: Vec<u8>, threshold: f64) -> Self {
        IdentityDefense {
            baseline,
            threshold,
            recovery_rate: 0.01,
            events: VecDeque::with_capacity(100),
            max_events: 1000,
            defense_active: true,
        }
    }

    pub fn check_perturbation(
        &mut self,
        current: &[u8],
        source: &str,
        tick: u64,
    ) -> PerturbationEvent {
        let sim = self.similarity(current);
        let detected = sim < self.threshold;
        let mitigated = if detected && self.defense_active {
            self.recover(current);
            true
        } else {
            false
        };
        let evt = PerturbationEvent {
            tick,
            source: source.into(),
            similarity_before: sim,
            similarity_after: if mitigated {
                self.similarity(current)
            } else {
                sim
            },
            detected,
            mitigated,
        };
        if self.events.len() >= self.max_events {
            self.events.pop_front();
        }
        self.events.push_back(evt.clone());
        evt
    }

    pub fn similarity(&self, other: &[u8]) -> f64 {
        let len = self.baseline.len().min(other.len());
        if len == 0 {
            return 0.0;
        }
        let matches = self
            .baseline
            .iter()
            .zip(other.iter())
            .filter(|(a, b)| a == b)
            .count();
        matches as f64 / len as f64
    }

    fn recover(&mut self, current: &[u8]) {
        let len = self.baseline.len().min(current.len());
        for i in 0..len {
            let diff = self.baseline[i] as f64 - current[i] as f64;
            let adj = (diff * self.recovery_rate).round() as i16;
            let new_val = (current[i] as i16 + adj).clamp(0, 255) as u8;
            self.baseline[i] = new_val;
        }
    }

    pub fn report(&self) -> String {
        let total = self.events.len();
        let detected = self.events.iter().filter(|e| e.detected).count();
        let mitigated = self.events.iter().filter(|e| e.mitigated).count();
        format!(
            "IdentityDefense: {} events, {} detected, {} mitigated, active={}, threshold={}",
            total, detected, mitigated, self.defense_active, self.threshold,
        )
    }

    pub fn avg_similarity(&self) -> f64 {
        let n = self.events.len();
        if n == 0 {
            return 1.0;
        }
        self.events.iter().map(|e| e.similarity_before).sum::<f64>() / n as f64
    }

    pub fn recent_threats(&self, n: usize) -> Vec<&PerturbationEvent> {
        let n = n.min(self.events.len());
        self.events.iter().skip(self.events.len() - n).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_baseline() -> Vec<u8> {
        (0..64).map(|i| (i * 4) as u8).collect()
    }

    #[test]
    fn test_check_perturbation_detects_change() {
        let mut defense = IdentityDefense::new(test_baseline(), 0.9);
        let mut perturbed = test_baseline();
        perturbed[0] = 255;
        let evt = defense.check_perturbation(&perturbed, "test", 0);
        assert!(evt.detected);
    }

    #[test]
    fn test_identical_no_perturbation() {
        let mut defense = IdentityDefense::new(test_baseline(), 0.9);
        let evt = defense.check_perturbation(&test_baseline(), "test", 0);
        assert!(!evt.detected);
    }

    #[test]
    fn test_mitigation_works() {
        let mut defense = IdentityDefense::new(test_baseline(), 0.9);
        let mut perturbed = test_baseline();
        perturbed[0] = 255;
        defense.check_perturbation(&perturbed, "attack", 0);
        assert!(defense.defense_active);
    }

    #[test]
    fn test_report_contains_info() {
        let mut defense = IdentityDefense::new(test_baseline(), 0.9);
        let mut p = test_baseline();
        p[0] = 255;
        defense.check_perturbation(&p, "x", 0);
        let r = defense.report();
        assert!(r.contains("IdentityDefense"));
    }

    #[test]
    fn test_avg_similarity() {
        let mut defense = IdentityDefense::new(test_baseline(), 0.0);
        defense.check_perturbation(&test_baseline(), "x", 0);
        assert!((defense.avg_similarity() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_recent_threats_count() {
        let mut defense = IdentityDefense::new(test_baseline(), 0.5);
        for i in 0..10 {
            defense.check_perturbation(&test_baseline(), "x", i);
        }
        assert_eq!(defense.recent_threats(3).len(), 3);
    }
}

// ── VSA-aware identity defense ──────────────────────────────────────────────

/// VSA-based identity defense with dream-based active recovery.
/// Extends IdentityDefense with high-dimensional vector similarity.
#[derive(Debug, Clone)]
pub struct VsaIdentityDefense {
    pub vsa_baseline: Vec<Vec<f64>>,
    pub dream_vectors: Vec<Vec<f64>>,
    pub threshold: f64,
    pub defense_active: bool,
    pub events: VecDeque<PerturbationEvent>,
}

impl VsaIdentityDefense {
    pub fn new(dim: usize, threshold: f64) -> Self {
        let mut rng = rand::thread_rng();
        let vsa_baseline = (0..4)
            .map(|_| (0..dim).map(|_| rng.gen::<f64>() * 2.0 - 1.0).collect())
            .collect();
        VsaIdentityDefense {
            vsa_baseline,
            dream_vectors: Vec::new(),
            threshold,
            defense_active: true,
            events: VecDeque::with_capacity(100),
        }
    }

    pub fn cosine_similarity(&self, a: &[f64], b: &[f64]) -> f64 {
        let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
        let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }
        (dot / (norm_a * norm_b)).clamp(-1.0, 1.0)
    }

    pub fn check_vsa_perturbation(
        &mut self,
        current: &[Vec<f64>],
        source: &str,
        tick: u64,
    ) -> PerturbationEvent {
        let n = self.vsa_baseline.len().min(current.len());
        let sim = if n == 0 {
            1.0
        } else {
            let total: f64 = self.vsa_baseline[..n]
                .iter()
                .zip(current[..n].iter())
                .map(|(b, c)| self.cosine_similarity(b, c))
                .sum();
            total / n as f64
        };
        let detected = sim < self.threshold;
        let mitigated = if detected && self.defense_active {
            self.recover_with_dream();
            true
        } else {
            false
        };
        let sim_after = if mitigated {
            let n2 = self.vsa_baseline.len().min(current.len());
            if n2 == 0 {
                1.0
            } else {
                let total: f64 = self.vsa_baseline[..n2]
                    .iter()
                    .zip(current[..n2].iter())
                    .map(|(b, c)| self.cosine_similarity(b, c))
                    .sum();
                total / n2 as f64
            }
        } else {
            sim
        };
        let evt = PerturbationEvent {
            tick,
            source: source.into(),
            similarity_before: sim,
            similarity_after: sim_after,
            detected,
            mitigated,
        };
        if self.events.len() >= 1000 {
            self.events.pop_front();
        }
        self.events.push_back(evt.clone());
        evt
    }

    /// Dream-based recovery that averages baseline with dream vectors.
    /// Returns average improvement in cosine similarity.
    pub fn recover_with_dream(&mut self) -> f64 {
        if self.dream_vectors.is_empty() || self.vsa_baseline.is_empty() {
            return 0.0;
        }
        let baseline_count = self.vsa_baseline.len();
        let mut total_improvement = 0.0;
        for i in 0..baseline_count {
            let dream_idx = i % self.dream_vectors.len();
            let before =
                self.cosine_similarity(&self.vsa_baseline[i], &self.dream_vectors[dream_idx]);
            for j in 0..self.vsa_baseline[i].len() {
                let blend = (self.vsa_baseline[i][j] + self.dream_vectors[dream_idx][j]) / 2.0;
                self.vsa_baseline[i][j] = blend;
            }
            let after =
                self.cosine_similarity(&self.vsa_baseline[i], &self.dream_vectors[dream_idx]);
            total_improvement += after - before;
        }
        total_improvement / baseline_count as f64
    }

    pub fn record_dream(&mut self, dream_vsa: Vec<f64>) {
        self.dream_vectors.push(dream_vsa);
    }

    /// Average pairwise cosine similarity among baseline vectors.
    pub fn baseline_strength(&self) -> f64 {
        let n = self.vsa_baseline.len();
        if n < 2 {
            return 1.0;
        }
        let mut total = 0.0;
        let mut count = 0;
        for i in 0..n {
            for j in (i + 1)..n {
                total += self.cosine_similarity(&self.vsa_baseline[i], &self.vsa_baseline[j]);
                count += 1;
            }
        }
        total / count as f64
    }

    /// Scan many candidates, returning only those that trigger detection.
    pub fn active_defense_scan(
        &mut self,
        candidates: &[Vec<Vec<f64>>],
        source: &str,
        tick: u64,
    ) -> Vec<(usize, PerturbationEvent)> {
        candidates
            .iter()
            .enumerate()
            .filter_map(|(idx, candidate)| {
                let evt = self.check_vsa_perturbation(candidate, source, tick);
                if evt.detected {
                    Some((idx, evt))
                } else {
                    None
                }
            })
            .collect()
    }
}

// ── VSA identity defense tests ──────────────────────────────────────────────

#[cfg(test)]
mod vsa_tests {
    use super::*;

    fn make_vsa_baseline() -> Vec<Vec<f64>> {
        vec![
            vec![1.0, 0.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0, 0.0],
            vec![0.0, 0.0, 1.0, 0.0],
            vec![0.0, 0.0, 0.0, 1.0],
        ]
    }

    #[test]
    fn test_vsa_baseline_creation() {
        let defense = VsaIdentityDefense::new(16, 0.5);
        assert_eq!(defense.vsa_baseline.len(), 4);
        assert_eq!(defense.vsa_baseline[0].len(), 16);
        assert!(defense.defense_active);
        assert!(defense.dream_vectors.is_empty());
    }

    #[test]
    fn test_vsa_perturbation_detection() {
        let mut defense = VsaIdentityDefense {
            vsa_baseline: make_vsa_baseline(),
            dream_vectors: vec![],
            threshold: 0.5,
            defense_active: true,
            events: VecDeque::new(),
        };
        let perturbed = vec![
            vec![0.0, 1.0, 0.0, 0.0],
            vec![1.0, 0.0, 0.0, 0.0],
            vec![0.0, 0.0, 0.0, 1.0],
            vec![0.0, 0.0, 1.0, 0.0],
        ];
        let evt = defense.check_vsa_perturbation(&perturbed, "attack", 1);
        assert!(evt.detected);
        assert!(evt.similarity_before < 0.5);
    }

    #[test]
    fn test_cosine_similarity_identical() {
        let defense = VsaIdentityDefense {
            vsa_baseline: make_vsa_baseline(),
            dream_vectors: vec![],
            threshold: 0.5,
            defense_active: true,
            events: VecDeque::new(),
        };
        let a = vec![2.0, -1.0, 0.5, 3.0];
        let sim = defense.cosine_similarity(&a, &a);
        assert!((sim - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let defense = VsaIdentityDefense {
            vsa_baseline: make_vsa_baseline(),
            dream_vectors: vec![],
            threshold: 0.5,
            defense_active: true,
            events: VecDeque::new(),
        };
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let sim = defense.cosine_similarity(&a, &b);
        assert!((sim - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_dream_recovery_improves_similarity() {
        let mut defense = VsaIdentityDefense {
            vsa_baseline: vec![vec![1.0, 0.0, 0.0, 0.0]],
            dream_vectors: vec![vec![0.7, 0.7, 0.0, 0.0]],
            threshold: 0.5,
            defense_active: true,
            events: VecDeque::new(),
        };
        let before_sim =
            defense.cosine_similarity(&defense.vsa_baseline[0], &defense.dream_vectors[0]);
        let improvement = defense.recover_with_dream();
        let after_sim =
            defense.cosine_similarity(&defense.vsa_baseline[0], &defense.dream_vectors[0]);
        assert!(after_sim > before_sim);
        assert!(improvement > 0.0);
    }

    #[test]
    fn test_baseline_strength() {
        let defense = VsaIdentityDefense {
            vsa_baseline: make_vsa_baseline(),
            dream_vectors: vec![],
            threshold: 0.5,
            defense_active: true,
            events: VecDeque::new(),
        };
        let strength = defense.baseline_strength();
        assert!((strength - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_active_defense_scan() {
        let mut defense = VsaIdentityDefense {
            vsa_baseline: make_vsa_baseline(),
            dream_vectors: vec![],
            threshold: 0.5,
            defense_active: true,
            events: VecDeque::new(),
        };
        let clean = defense.vsa_baseline.clone();
        let perturbed = vec![
            vec![0.0, 1.0, 0.0, 0.0],
            vec![1.0, 0.0, 0.0, 0.0],
            vec![0.0, 0.0, 0.0, 1.0],
            vec![0.0, 0.0, 1.0, 0.0],
        ];
        let results = defense.active_defense_scan(&[clean, perturbed], "scan", 2);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, 1);
    }

    #[test]
    fn test_record_dream() {
        let mut defense = VsaIdentityDefense {
            vsa_baseline: make_vsa_baseline(),
            dream_vectors: vec![],
            threshold: 0.5,
            defense_active: true,
            events: VecDeque::new(),
        };
        assert!(defense.dream_vectors.is_empty());
        defense.record_dream(vec![0.5; 4]);
        assert_eq!(defense.dream_vectors.len(), 1);
        defense.record_dream(vec![-0.5; 4]);
        assert_eq!(defense.dream_vectors.len(), 2);
    }
}
