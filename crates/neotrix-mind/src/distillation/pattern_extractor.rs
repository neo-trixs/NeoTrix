use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::capture::CapturedInteraction;

/// A distilled behavioral pattern extracted from model responses.
/// Model-agnostic: captures what works, not which model did it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralPattern {
    /// Unique ID
    pub id: String,
    /// Human-readable description
    pub description: String,
    /// The structural fingerprint (hash of format features)
    pub structure_hash: u64,
    /// Topic this pattern applies to
    pub topic: String,
    /// Observed outcome score (0.0–1.0)
    pub avg_outcome: f64,
    /// How many interactions contributed to this pattern
    pub observation_count: u32,
    /// Confidence based on observation count
    pub confidence: f64,
    /// Structural features
    pub has_code: bool,
    pub code_block_count_avg: f64,
    pub has_sections: bool,
    pub has_lists: bool,
    pub avg_word_count: f64,
    /// Which models exhibited this pattern
    pub observed_models: Vec<String>,
    /// Which providers
    pub observed_providers: Vec<String>,
}

/// Extracts recurring behavioral patterns from captured interactions.
/// Groups by structure_hash and topic to find what response formats
/// correlate with high outcome scores.
#[derive(Debug, Clone)]
pub struct PatternExtractor {
    min_observations: usize,
    min_confidence: f64,
    patterns_extracted: u64,
}

impl PatternExtractor {
    pub fn new() -> Self {
        Self {
            min_observations: 3,
            min_confidence: 0.5,
            patterns_extracted: 0,
        }
    }

    pub fn with_min_observations(mut self, min: usize) -> Self {
        self.min_observations = min;
        self
    }

    pub fn with_min_confidence(mut self, min: f64) -> Self {
        self.min_confidence = min;
        self
    }

    /// Extract behavioral patterns from a batch of captured interactions.
    /// Groups by (structure_hash, topic) and computes aggregate stats.
    pub fn extract(&mut self, interactions: &[CapturedInteraction]) -> Vec<BehavioralPattern> {
        let mut groups: HashMap<(u64, String), Vec<&CapturedInteraction>> = HashMap::new();

        for i in interactions {
            let key = (i.structure_hash, i.topic_hint().to_string());
            groups.entry(key).or_default().push(i);
        }

        let mut patterns = Vec::new();

        for ((hash, topic), group) in groups {
            if group.len() < self.min_observations {
                continue;
            }

            let total_outcome: f64 = group.iter().map(|i| i.outcome_score).sum();
            let avg_outcome = total_outcome / group.len() as f64;
            let confidence = PatternExtractor::compute_confidence(group.len(), avg_outcome);

            if confidence < self.min_confidence {
                continue;
            }

            let avg_word_count: f64 =
                group.iter().map(|i| i.word_count as f64).sum::<f64>() / group.len() as f64;
            let avg_code_blocks: f64 =
                group.iter().map(|i| i.code_block_count as f64).sum::<f64>() / group.len() as f64;

            let has_code = group.iter().any(|i| i.has_code);
            let has_sections = group.iter().any(|i| i.has_sections);
            let has_lists = group.iter().any(|i| i.has_lists);

            let mut observed_models: Vec<String> = group.iter().map(|i| i.model.clone()).collect();
            observed_models.sort();
            observed_models.dedup();

            let mut observed_providers: Vec<String> =
                group.iter().map(|i| i.provider.clone()).collect();
            observed_providers.sort();
            observed_providers.dedup();

            let desc = format!(
                "Pattern[{}]: {} structure across {} observations, avg_score={:.3}, models=[{}]",
                topic,
                Self::describe_structure(hash),
                group.len(),
                avg_outcome,
                observed_models.join(",")
            );

            self.patterns_extracted += 1;
            patterns.push(BehavioralPattern {
                id: format!("bp_{}_{}", self.patterns_extracted, hash),
                description: desc,
                structure_hash: hash,
                topic,
                avg_outcome,
                observation_count: group.len() as u32,
                confidence,
                has_code,
                code_block_count_avg: avg_code_blocks,
                has_sections,
                has_lists,
                avg_word_count,
                observed_models,
                observed_providers,
            });
        }

        // Sort by confidence descending
        patterns.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        patterns
    }

    /// Extract which response structures work best per topic.
    /// Returns (topic, best_pattern_description, best_avg_outcome)
    pub fn best_per_topic(
        &mut self,
        interactions: &[CapturedInteraction],
    ) -> Vec<(String, String, f64)> {
        let patterns = self.extract(interactions);
        let mut best: HashMap<String, (String, f64)> = HashMap::new();
        for p in patterns {
            let entry = best.entry(p.topic.clone()).or_default();
            if p.avg_outcome > entry.1 {
                *entry = (p.description.clone(), p.avg_outcome);
            }
        }
        let mut result: Vec<(String, String, f64)> = best
            .into_iter()
            .map(|(topic, (desc, score))| (topic, desc, score))
            .collect();
        result.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        result
    }

    /// Identify which models perform best for which topics.
    /// Returns (topic, best_model, avg_outcome_for_that_model).
    pub fn best_model_per_topic(
        &self,
        interactions: &[CapturedInteraction],
    ) -> Vec<(String, String, f64)> {
        // Group by (topic, model)
        let mut groups: HashMap<(String, String), Vec<f64>> = HashMap::new();
        for i in interactions {
            let key = (i.topic_hint().to_string(), i.model.clone());
            groups.entry(key).or_default().push(i.outcome_score);
        }

        let mut per_topic: HashMap<String, (String, f64)> = HashMap::new();
        for ((topic, model), scores) in groups {
            if scores.len() < 2 {
                continue;
            }
            let avg: f64 = scores.iter().sum::<f64>() / scores.len() as f64;
            let entry = per_topic.entry(topic).or_default();
            if avg > entry.1 {
                *entry = (model, avg);
            }
        }

        let mut result: Vec<(String, String, f64)> = per_topic
            .into_iter()
            .map(|(topic, (model, score))| (topic, model, score))
            .collect();
        result.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        result
    }

    fn describe_structure(hash: u64) -> String {
        // Decode the hash's low bits as qualitative description
        let has_code = hash & 1 == 1;
        let has_headings = (hash >> 1) & 1 == 1;
        let has_lists = (hash >> 2) & 1 == 1;
        let len_bucket = (hash >> 3) & 0b1111;

        let mut parts = Vec::new();
        if has_code {
            parts.push("code");
        }
        if has_headings {
            parts.push("sectioned");
        }
        if has_lists {
            parts.push("listed");
        }
        parts.push(match len_bucket {
            0 => "very_short",
            1..=3 => "short",
            4..=8 => "medium",
            9..=12 => "long",
            _ => "very_long",
        });

        parts.join("_")
    }

    fn compute_confidence(obs_count: usize, avg_outcome: f64) -> f64 {
        let count_factor = (obs_count as f64 / 10.0).min(1.0);
        let outcome_factor = avg_outcome;
        count_factor * 0.4 + outcome_factor * 0.6
    }

    pub fn patterns_extracted(&self) -> u64 {
        self.patterns_extracted
    }
}

impl Default for PatternExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_interactions(
        n: usize,
        model: &str,
        topic_words: &[&str],
    ) -> Vec<CapturedInteraction> {
        (0..n)
            .map(|i| {
                let msg = if i % 2 == 0 {
                    format!("write {} code", topic_words[i % topic_words.len()])
                } else {
                    format!("debug {} issue", topic_words[i % topic_words.len()])
                };
                let response = if i % 3 == 0 {
                    format!("## Solution\n\n```rust\nfn solve() {{}}\n```\n\n- step 1\n- step 2")
                } else {
                    "Short answer.".to_string()
                };
                CapturedInteraction::new(
                    "test", model, "", &msg, &response, 50, 100, 100, true, "stop",
                )
            })
            .collect()
    }

    #[test]
    fn test_extract_empty() {
        let mut extractor = PatternExtractor::new();
        let patterns = extractor.extract(&[]);
        assert!(patterns.is_empty());
    }

    #[test]
    fn test_extract_below_min_observations() {
        let mut extractor = PatternExtractor::new().with_min_observations(5);
        let interactions = sample_interactions(3, "m1", &["rust"]);
        let patterns = extractor.extract(&interactions);
        assert!(patterns.is_empty());
    }

    #[test]
    fn test_extract_finds_patterns() {
        let mut extractor = PatternExtractor::new().with_min_observations(2);
        let interactions = sample_interactions(6, "m1", &["rust"]);
        let patterns = extractor.extract(&interactions);
        assert!(!patterns.is_empty());
        for p in &patterns {
            assert!(p.observation_count >= 2);
            assert!(p.confidence > 0.0);
        }
    }

    #[test]
    fn test_best_per_topic() {
        let mut extractor = PatternExtractor::new().with_min_observations(2);
        let interactions = sample_interactions(6, "m1", &["rust"]);
        let best = extractor.best_per_topic(&interactions);
        // Should find at least one topic
        assert!(!best.is_empty() || interactions.len() < 3);
    }

    #[test]
    fn test_best_model_per_topic() {
        let extractor = PatternExtractor::new();
        let mut interactions = Vec::new();
        interactions.extend(sample_interactions(4, "model-a", &["rust"]));
        interactions.extend(sample_interactions(4, "model-b", &["rust"]));
        let best = extractor.best_model_per_topic(&interactions);
        // Should identify at least one best model per topic
        assert!(!best.is_empty() || interactions.len() < 8);
    }

    #[test]
    fn test_patterns_sorted_by_confidence() {
        let mut extractor = PatternExtractor::new().with_min_observations(1);
        let mut interactions = Vec::new();
        interactions.extend(sample_interactions(5, "ma", &["rust"]));
        interactions.extend(sample_interactions(5, "mb", &["python"]));
        let patterns = extractor.extract(&interactions);
        if patterns.len() >= 2 {
            for i in 1..patterns.len() {
                assert!(
                    patterns[i - 1].confidence >= patterns[i].confidence,
                    "patterns should be sorted by descending confidence"
                );
            }
        }
    }

    #[test]
    fn test_describe_structure() {
        let desc = PatternExtractor::describe_structure(0b101);
        assert!(!desc.is_empty());
        assert!(desc.contains("code") || desc.contains("sectioned") || desc.contains("listed"));
    }

    #[test]
    fn test_estimate_confidence() {
        let c = PatternExtractor::compute_confidence(10, 0.8);
        assert!(c > 0.0 && c <= 1.0);
        let c_low = PatternExtractor::compute_confidence(1, 0.1);
        let c_high = PatternExtractor::compute_confidence(100, 1.0);
        assert!(c_low < c_high);
    }

    #[test]
    fn test_extract_tracks_models() {
        let mut extractor = PatternExtractor::new().with_min_observations(2);
        let mut interactions = Vec::new();
        interactions.extend(sample_interactions(3, "m1", &["rust"]));
        interactions.extend(sample_interactions(3, "m2", &["rust"]));
        let patterns = extractor.extract(&interactions);
        for p in &patterns {
            if p.observation_count >= 2 {
                assert!(!p.observed_models.is_empty());
            }
        }
    }

    #[test]
    fn test_extract_tracks_providers() {
        let mut extractor = PatternExtractor::new().with_min_observations(2);
        let mut interactions = Vec::new();
        interactions.extend(sample_interactions(3, "m1", &["rust"]));
        let patterns = extractor.extract(&interactions);
        for p in &patterns {
            if p.observation_count >= 2 {
                assert!(!p.observed_providers.is_empty());
            }
        }
    }
}
