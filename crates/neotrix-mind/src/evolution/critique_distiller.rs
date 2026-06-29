use std::collections::VecDeque;
use crate::memory::hypercube::HyperCube;

const MAX_CRITIQUES: usize = 256;
const DISTILLATION_THRESHOLD: f64 = 0.6;
const GUIDELINE_TTL_CYCLES: u64 = 1000;

#[derive(Debug, Clone)]
pub struct Critique {
    pub id: u64,
    pub source: String,
    pub content_vsa: Vec<f64>,
    pub confidence: f64,
    pub frequency: u64,
    pub cycle: u64,
}

#[derive(Debug, Clone)]
pub struct Guideline {
    pub id: u64,
    pub pattern_vsa: Vec<f64>,
    pub derived_from: Vec<u64>,
    pub applicability: f64,
    pub cycle_created: u64,
    pub hit_count: u64,
}

#[derive(Debug, Clone)]
pub struct CritiqueDistiller {
    critiques: VecDeque<Critique>,
    guidelines: Vec<Guideline>,
    next_id: u64,
    current_cycle: u64,
    distillation_count: u64,
}

impl CritiqueDistiller {
    pub fn new() -> Self {
        Self {
            critiques: VecDeque::with_capacity(MAX_CRITIQUES),
            guidelines: Vec::new(),
            next_id: 1,
            current_cycle: 0,
            distillation_count: 0,
        }
    }

    pub fn ingest(&mut self, source: &str, content_vsa: Vec<f64>, confidence: f64) {
        if self.critiques.len() >= MAX_CRITIQUES {
            self.critiques.pop_front();
        }
        self.critiques.push_back(Critique {
            id: self.next_id,
            source: source.to_string(),
            content_vsa,
            confidence,
            frequency: 1,
            cycle: self.current_cycle,
        });
        self.next_id += 1;
    }

    pub fn distill(&mut self) -> Vec<u64> {
        let mut new_guidelines = Vec::new();

        let recent: Vec<&Critique> = self.critiques.iter()
            .filter(|c| c.confidence > DISTILLATION_THRESHOLD)
            .collect();

        for (i, c1) in recent.iter().enumerate() {
            if self.guidelines.iter().any(|g| {
                HyperCube::similarity(&g.pattern_vsa, &c1.content_vsa) > 0.85
            }) {
                continue;
            }

            let mut cluster: Vec<u64> = vec![c1.id];
            for c2 in recent.iter().skip(i + 1) {
                let sim = HyperCube::similarity(&c1.content_vsa, &c2.content_vsa);
                if sim > 0.7 {
                    cluster.push(c2.id);
                }
            }

            if cluster.len() >= 2 {
                let pattern = HyperCube::bundle(&c1.content_vsa, &c1.content_vsa);
                let guideline_id = self.next_id;
                self.next_id += 1;
                self.guidelines.push(Guideline {
                    id: guideline_id,
                    pattern_vsa: pattern,
                    derived_from: cluster,
                    applicability: c1.confidence,
                    cycle_created: self.current_cycle,
                    hit_count: 0,
                });
                new_guidelines.push(guideline_id);
                self.distillation_count += 1;
            }
        }

        new_guidelines
    }

    pub fn match_guideline(&mut self, query: &[f64]) -> Option<&Guideline> {
        let best = self.guidelines.iter_mut()
            .max_by(|a, b| {
                let sim_a = HyperCube::similarity(&a.pattern_vsa, query);
                let sim_b = HyperCube::similarity(&b.pattern_vsa, query);
                sim_a.partial_cmp(&sim_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .filter(|g| HyperCube::similarity(&g.pattern_vsa, query) > 0.6)?;
        best.hit_count += 1;
        Some(best)
    }

    pub fn prune_expired(&mut self) -> usize {
        let before = self.guidelines.len();
        self.guidelines.retain(|g| {
            let age = self.current_cycle - g.cycle_created;
            age < GUIDELINE_TTL_CYCLES || g.hit_count > 5
        });
        let pruned = before - self.guidelines.len();
        self.critiques.retain(|c| {
            let age = self.current_cycle - c.cycle;
            age < GUIDELINE_TTL_CYCLES
        });
        pruned
    }

    pub fn tick(&mut self) {
        self.current_cycle += 1;
        if self.current_cycle % 10 == 0 {
            self.distill();
        }
        if self.current_cycle % 50 == 0 {
            self.prune_expired();
        }
    }

    pub fn guideline_count(&self) -> usize {
        self.guidelines.len()
    }

    pub fn critique_count(&self) -> usize {
        self.critiques.len()
    }

    pub fn distillation_events(&self) -> u64 {
        self.distillation_count
    }

    pub fn summary(&self) -> String {
        format!("CritiqueDistiller[critiques={} guidelines={} distillations={} cycle={}]",
            self.critiques.len(), self.guidelines.len(), self.distillation_count, self.current_cycle)
    }
}

impl Default for CritiqueDistiller {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_distiller() {
        let cd = CritiqueDistiller::new();
        assert_eq!(cd.critique_count(), 0);
        assert_eq!(cd.guideline_count(), 0);
    }

    #[test]
    fn test_ingest_adds_critique() {
        let mut cd = CritiqueDistiller::new();
        cd.ingest("test", vec![0.8; 8], 0.9);
        assert_eq!(cd.critique_count(), 1);
    }

    #[test]
    fn test_low_confidence_no_distillation() {
        let mut cd = CritiqueDistiller::new();
        cd.ingest("a", vec![0.8; 8], 0.3);
        cd.ingest("b", vec![0.81; 8], 0.4);
        let new = cd.distill();
        assert!(new.is_empty());
    }

    #[test]
    fn test_similar_critiques_distill() {
        let mut cd = CritiqueDistiller::new();
        cd.ingest("a", vec![0.8; 8], 0.8);
        cd.ingest("b", vec![0.81; 8], 0.85);
        let new = cd.distill();
        assert!(!new.is_empty());
    }

    #[test]
    fn test_tick_triggers_distillation() {
        let mut cd = CritiqueDistiller::new();
        cd.ingest("a", vec![0.8; 8], 0.8);
        cd.ingest("b", vec![0.81; 8], 0.85);
        for _ in 0..10 {
            cd.tick();
        }
        assert!(cd.distillation_events() > 0 || cd.guideline_count() > 0);
    }

    #[test]
    fn test_prune_expired() {
        let mut cd = CritiqueDistiller::new();
        cd.ingest("a", vec![0.8; 8], 0.8);
        cd.ingest("b", vec![0.81; 8], 0.85);
        cd.distill();
        assert_eq!(cd.guideline_count(), 1);
        for _ in 0..GUIDELINE_TTL_CYCLES + 10 {
            cd.tick();
        }
        cd.prune_expired();
    }

    #[test]
    fn test_summary_format() {
        let mut cd = CritiqueDistiller::new();
        cd.ingest("x", vec![0.5; 8], 0.7);
        let s = cd.summary();
        assert!(s.starts_with("CritiqueDistiller["));
        assert!(s.contains("critiques="));
    }
}
