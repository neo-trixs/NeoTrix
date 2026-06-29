use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum HeuristicCategory {
    ActionSelection,
    ProblemDecomposition,
    ReasoningStrategy,
    ErrorAvoidance,
    ResourceManagement,
    ExplorationStrategy,
}

impl HeuristicCategory {
    pub fn all() -> Vec<Self> {
        vec![
            Self::ActionSelection,
            Self::ProblemDecomposition,
            Self::ReasoningStrategy,
            Self::ErrorAvoidance,
            Self::ResourceManagement,
            Self::ExplorationStrategy,
        ]
    }

    pub fn seed(&self) -> u64 {
        match self {
            Self::ActionSelection => 0xAC_10_0000_0000_0000,
            Self::ProblemDecomposition => 0xDE_C0_0000_0000_0000,
            Self::ReasoningStrategy => 0xE4_E5_0000_0000_0000,
            Self::ErrorAvoidance => 0xE5_12_0000_0000_0000,
            Self::ResourceManagement => 0xE4_E5_0D_0000_0000,
            Self::ExplorationStrategy => 0xE5_78_0000_0000_0000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Heuristic {
    pub id: u64,
    pub pattern: Vec<u8>,
    pub context: Vec<u8>,
    pub effectiveness: f64,
    pub confidence: f64,
    pub source_count: u64,
    pub success_count: u64,
    pub category: HeuristicCategory,
    pub description: String,
    pub created_at: u64,
    pub last_activated_at: u64,
    pub activation_count: u64,
}

pub struct HeuristicFilter {
    pub min_effectiveness: f64,
    pub min_confidence: f64,
    pub categories: Option<Vec<HeuristicCategory>>,
}

impl Default for HeuristicFilter {
    fn default() -> Self {
        Self {
            min_effectiveness: 0.0,
            min_confidence: 0.0,
            categories: None,
        }
    }
}

pub struct ReflectorConfig {
    pub max_heuristics: usize,
    pub min_confidence_threshold: f64,
    pub similarity_threshold: f64,
    pub decay_factor: f64,
}

impl Default for ReflectorConfig {
    fn default() -> Self {
        Self {
            max_heuristics: 500,
            min_confidence_threshold: 0.3,
            similarity_threshold: 0.75,
            decay_factor: 0.97,
        }
    }
}

pub struct ExperienceReflector {
    heuristics: Vec<Heuristic>,
    next_id: u64,
    cycle: u64,
    pub config: ReflectorConfig,
    recent_patterns: VecDeque<u64>,
}

impl ExperienceReflector {
    pub fn new(config: ReflectorConfig) -> Self {
        Self {
            heuristics: Vec::with_capacity(config.max_heuristics),
            next_id: 1,
            cycle: 0,
            config,
            recent_patterns: VecDeque::with_capacity(32),
        }
    }

    pub fn reflect(
        &mut self,
        context_str: &str,
        action_str: &str,
        success: bool,
        outcome_quality: f64,
    ) -> Option<u64> {
        self.cycle += 1;
        let context_vec = QuantizedVSA::seeded_random(self.stable_hash(context_str), 4096);
        let action_vec = QuantizedVSA::seeded_random(self.stable_hash(action_str), 4096);
        let pattern = QuantizedVSA::bind(&context_vec, &action_vec);

        let category = if success && outcome_quality > 0.8 {
            HeuristicCategory::ActionSelection
        } else if outcome_quality < 0.3 {
            HeuristicCategory::ErrorAvoidance
        } else {
            HeuristicCategory::ReasoningStrategy
        };

        if let Some(existing) = self.find_similar(&pattern, self.config.similarity_threshold) {
            self.heuristics[existing].success_count += if success { 1 } else { 0 };
            self.heuristics[existing].source_count += 1;
            let new_rate = self.heuristics[existing].success_count as f64
                / self.heuristics[existing].source_count as f64;
            self.heuristics[existing].effectiveness =
                self.heuristics[existing].effectiveness * 0.7 + new_rate * 0.3;
            self.heuristics[existing].confidence = (self.heuristics[existing].confidence
                + 0.05 * if success { 1.0 } else { -0.05 })
            .clamp(0.0, 1.0);
            self.heuristics[existing].last_activated_at = self.cycle;
            self.heuristics[existing].activation_count += 1;
            self.recent_patterns.push_back(self.heuristics[existing].id);
            return Some(self.heuristics[existing].id);
        }

        if self.heuristics.len() >= self.config.max_heuristics {
            self.prune(self.config.min_confidence_threshold);
        }
        if self.heuristics.len() >= self.config.max_heuristics {
            return None;
        }

        let id = self.next_id;
        self.next_id += 1;
        self.heuristics.push(Heuristic {
            id,
            pattern,
            context: context_vec,
            effectiveness: if success { 0.6 } else { 0.2 },
            confidence: 0.3,
            source_count: 1,
            success_count: if success { 1 } else { 0 },
            category,
            description: format!("{} → {}", context_str, action_str),
            created_at: self.cycle,
            last_activated_at: self.cycle,
            activation_count: 1,
        });
        self.recent_patterns.push_back(id);
        Some(id)
    }

    pub fn retrieve(&self, context_vec: &[u8], top_k: usize) -> Vec<&Heuristic> {
        let mut scored: Vec<(f64, usize)> = self
            .heuristics
            .iter()
            .enumerate()
            .map(|(i, h)| {
                let sim = QuantizedVSA::similarity(context_vec, &h.context);
                let score = sim * h.effectiveness * h.confidence;
                (score, i)
            })
            .filter(|(s, _)| *s > self.config.min_confidence_threshold * 0.5)
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored
            .into_iter()
            .take(top_k)
            .map(|(_, i)| &self.heuristics[i])
            .collect()
    }

    pub fn retrieve_by_category(
        &self,
        category: &HeuristicCategory,
        top_k: usize,
    ) -> Vec<&Heuristic> {
        let mut filtered: Vec<&Heuristic> = self
            .heuristics
            .iter()
            .filter(|h| h.category == *category)
            .collect();
        filtered.sort_by(|a, b| {
            b.effectiveness
                .partial_cmp(&a.effectiveness)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        filtered.truncate(top_k);
        filtered
    }

    fn find_similar(&self, pattern: &[u8], threshold: f64) -> Option<usize> {
        self.heuristics
            .iter()
            .enumerate()
            .filter(|(_, h)| {
                let sim = QuantizedVSA::similarity(&h.pattern, pattern);
                sim >= threshold
            })
            .max_by(|a, b| {
                let sa = QuantizedVSA::similarity(&a.1.pattern, pattern);
                let sb = QuantizedVSA::similarity(&b.1.pattern, pattern);
                sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(i, _)| i)
    }

    pub fn prune(&mut self, min_confidence: f64) -> usize {
        let before = self.heuristics.len();
        self.heuristics.retain(|h| {
            let should_keep = h.confidence >= min_confidence && h.activation_count > 0;
            if !should_keep && self.cycle - h.last_activated_at > 100 {
                self.recent_patterns.retain(|id| *id != h.id);
            }
            should_keep || self.cycle - h.last_activated_at <= 100
        });
        self.heuristics.sort_by(|a, b| {
            let score_a = a.effectiveness * a.confidence * (a.activation_count as f64);
            let score_b = b.effectiveness * b.confidence * (b.activation_count as f64);
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        self.heuristics.truncate(self.config.max_heuristics);
        let removed = before.saturating_sub(self.heuristics.len());
        if removed > 0 {
            let valid: std::collections::HashSet<u64> =
                self.heuristics.iter().map(|h| h.id).collect();
            self.recent_patterns.retain(|id| valid.contains(id));
        }
        removed
    }

    pub fn decay_old(&mut self) {
        for h in &mut self.heuristics {
            if self.cycle - h.last_activated_at > 50 {
                h.confidence *= self.config.decay_factor;
                h.effectiveness *= self.config.decay_factor;
            }
        }
    }

    pub fn heuristic_count(&self) -> usize {
        self.heuristics.len()
    }

    pub fn best_heuristics(&self, top_k: usize) -> Vec<&Heuristic> {
        let mut sorted: Vec<&Heuristic> = self.heuristics.iter().collect();
        sorted.sort_by(|a, b| {
            let sa = a.effectiveness * a.confidence;
            let sb = b.effectiveness * b.confidence;
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted.truncate(top_k);
        sorted
    }

    fn stable_hash(&self, s: &str) -> u64 {
        let mut h: u64 = 0x517cc1b727220a95u64;
        for b in s.bytes() {
            h = h.wrapping_mul(0x9e3779b97f4a7c15u64);
            h ^= b as u64;
            h = h.rotate_left(13);
        }
        h
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_reflector() -> ExperienceReflector {
        ExperienceReflector::new(ReflectorConfig {
            max_heuristics: 100,
            min_confidence_threshold: 0.3,
            similarity_threshold: 0.75,
            decay_factor: 0.97,
        })
    }

    #[test]
    fn test_reflect_creates_heuristic() -> Result<(), String> {
        let mut r = make_reflector();
        let id = r.reflect("ctx_test", "action_test", true, 0.9);
        assert!(id.is_some());
        assert_eq!(r.heuristic_count(), 1);
        let id = id.ok_or_else(|| "reflect returned None after assert".to_string())?;
        assert!(id > 0);
        Ok(())
    }

    #[test]
    fn test_reflect_updates_existing_on_similar() {
        let mut r = make_reflector();
        let id1 = r.reflect("same_context", "same_action", true, 0.9).unwrap();
        let id2 = r.reflect("same_context", "same_action", true, 0.9).unwrap();
        assert_eq!(id1, id2, "similar patterns should reuse same heuristic");
        assert_eq!(r.heuristic_count(), 1);
        let h = &r.heuristics[0];
        assert_eq!(h.activation_count, 2);
        assert_eq!(h.success_count, 2);
    }

    #[test]
    fn test_reflect_updates_effectiveness() {
        let mut r = make_reflector();
        r.reflect("ctx", "act", true, 0.9);
        r.reflect("ctx", "act", false, 0.2);
        let h = &r.heuristics[0];
        assert!(h.effectiveness > 0.0);
        assert!(h.confidence < 0.5);
    }

    #[test]
    fn test_retrieve_returns_by_similarity() {
        let mut r = make_reflector();
        r.reflect("alpha_context", "alpha_action", true, 0.9);
        r.reflect("beta_context", "beta_action", false, 0.2);
        let vec = QuantizedVSA::seeded_random(r.stable_hash("alpha_context"), 4096);
        let results = r.retrieve(&vec, 5);
        assert!(!results.is_empty());
        assert_eq!(results[0].category, HeuristicCategory::ActionSelection);
    }

    #[test]
    fn test_retrieve_by_category_filters() {
        let mut r = make_reflector();
        r.reflect("good_ctx", "good_act", true, 0.95);
        r.reflect("bad_ctx", "bad_act", false, 0.2);
        let results = r.retrieve_by_category(&HeuristicCategory::ActionSelection, 10);
        assert!(results
            .iter()
            .all(|h| h.category == HeuristicCategory::ActionSelection));
    }

    #[test]
    fn test_retrieve_by_category_orders_by_effectiveness() {
        let mut r = make_reflector();
        r.reflect("c1", "a1", true, 0.95);
        r.reflect("c2", "a2", true, 0.85);
        let results = r.retrieve_by_category(&HeuristicCategory::ActionSelection, 10);
        if results.len() >= 2 {
            assert!(results[0].effectiveness >= results[1].effectiveness);
        }
    }

    #[test]
    fn test_prune_removes_low_confidence() {
        let mut r = make_reflector();
        r.reflect("ctx", "act", true, 0.9);
        assert_eq!(r.heuristic_count(), 1);
        r.heuristics[0].confidence = 0.1;
        r.heuristics[0].activation_count = 0;
        r.cycle = 200;
        let pruned = r.prune(0.5);
        assert!(pruned > 0 || r.heuristic_count() == 0);
    }

    #[test]
    fn test_decay_old_reduces_confidence() {
        let mut r = make_reflector();
        r.reflect("ctx", "act", true, 0.9);
        let conf_before = r.heuristics[0].confidence;
        let eff_before = r.heuristics[0].effectiveness;
        r.cycle = 100;
        r.decay_old();
        assert!(r.heuristics[0].confidence <= conf_before);
        assert!(r.heuristics[0].effectiveness <= eff_before);
    }

    #[test]
    fn test_best_heuristics_orders_correctly() {
        let mut r = make_reflector();
        r.reflect("high", "act", true, 0.95);
        r.reflect("low", "act", false, 0.1);
        let best = r.best_heuristics(10);
        assert_eq!(best.len(), 2);
        assert!(
            best[0].effectiveness * best[0].confidence
                >= best[1].effectiveness * best[1].confidence
        );
    }

    #[test]
    fn test_heuristic_count_zero_initial() {
        let r = make_reflector();
        assert_eq!(r.heuristic_count(), 0);
    }

    #[test]
    fn test_find_similar_returns_none_for_dissimilar() {
        let mut r = make_reflector();
        r.reflect("abc", "def", true, 0.9);
        let dissimilar = QuantizedVSA::seeded_random(99999, 4096);
        let result = r.find_similar(&dissimilar, 0.99);
        assert!(result.is_none());
    }

    #[test]
    fn test_reflect_returns_none_when_full() {
        let mut r = ExperienceReflector::new(ReflectorConfig {
            max_heuristics: 1,
            min_confidence_threshold: 0.3,
            similarity_threshold: 1.0,
            decay_factor: 0.97,
        });
        r.reflect("c1", "a1", true, 0.9);
        let id2 = r.reflect("c2", "a2", true, 0.9);
        assert!(id2.is_none());
    }
}
