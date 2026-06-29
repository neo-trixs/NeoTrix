use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ExperienceRecord {
    pub id: u64,
    pub context: String,
    pub action: String,
    pub reward: f64,
    pub success: bool,
    pub timestamp: u64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct Heuristic {
    pub pattern: String,
    pub principle: String,
    pub confidence: f64,
    pub source_count: usize,
    pub is_positive: bool,
}

impl Heuristic {
    pub fn apply(&self, context: &str) -> Option<String> {
        let context_lower = context.to_lowercase();
        let pattern_lower = self.pattern.to_lowercase();
        let keywords: Vec<&str> = pattern_lower.split_whitespace().collect();
        let match_count = keywords
            .iter()
            .filter(|&&kw| context_lower.contains(kw))
            .count();
        if match_count > 0 {
            let ratio = match_count as f64 / keywords.len() as f64;
            if ratio >= 0.3 {
                return Some(self.principle.clone());
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct TrajectoryHeuristicExtractor {
    pub heuristics: Vec<Heuristic>,
    max_heuristics: usize,
    pub experience_buffer: Vec<ExperienceRecord>,
    next_id: u64,
}

impl TrajectoryHeuristicExtractor {
    pub fn new(max_heuristics: usize) -> Self {
        Self {
            heuristics: Vec::with_capacity(max_heuristics),
            max_heuristics,
            experience_buffer: Vec::new(),
            next_id: 0,
        }
    }

    pub fn record(&mut self, context: String, action: String, success: bool, reward: f64) {
        let record = ExperienceRecord {
            id: self.next_id,
            context,
            action,
            reward,
            success,
            timestamp: 0,
            metadata: std::collections::HashMap::new(),
        };
        self.next_id += 1;
        self.experience_buffer.push(record);
    }

    pub fn drain_buffer(&mut self) -> Vec<ExperienceRecord> {
        std::mem::take(&mut self.experience_buffer)
    }

    pub fn extract_heuristics(&self, trajectory: &[ExperienceRecord]) -> Vec<Heuristic> {
        if trajectory.is_empty() {
            return Vec::new();
        }

        let successes: Vec<&ExperienceRecord> = trajectory.iter().filter(|e| e.success).collect();
        let failures: Vec<&ExperienceRecord> = trajectory.iter().filter(|e| !e.success).collect();

        let mut heuristics = Vec::new();

        let success_heuristics = Self::extract_pattern_heuristics(&successes, true);
        heuristics.extend(success_heuristics);

        let failure_heuristics = Self::extract_pattern_heuristics(&failures, false);
        heuristics.extend(failure_heuristics);

        heuristics
    }

    fn extract_pattern_heuristics(
        records: &[&ExperienceRecord],
        is_positive: bool,
    ) -> Vec<Heuristic> {
        if records.is_empty() {
            return Vec::new();
        }

        let mut context_groups: HashMap<String, Vec<&ExperienceRecord>> = HashMap::new();
        for r in records {
            let key = Self::extract_domain_key(&r.context);
            context_groups.entry(key).or_default().push(r);
        }

        let mut heuristics = Vec::new();
        for (_domain, group) in &context_groups {
            let count = group.len();
            if count < 2 {
                continue;
            }

            let avg_reward: f64 = group.iter().map(|r| r.reward).sum::<f64>() / count as f64;
            let confidence = (avg_reward * 0.7 + (count as f64 / 20.0).min(0.3)).clamp(0.0, 1.0);

            let pattern = Self::extract_common_pattern(group);
            let principle = if is_positive {
                format!(
                    "When context matches '{}', apply similar action pattern",
                    pattern
                )
            } else {
                format!(
                    "When context matches '{}', avoid this action pattern",
                    pattern
                )
            };

            heuristics.push(Heuristic {
                pattern,
                principle,
                confidence,
                source_count: count,
                is_positive,
            });
        }

        heuristics.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        heuristics
    }

    fn extract_domain_key(context: &str) -> String {
        let words: Vec<&str> = context.split_whitespace().collect();
        let key_count = 3.min(words.len());
        words
            .iter()
            .take(key_count)
            .map(|w| w.to_lowercase())
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn extract_common_pattern(records: &[&ExperienceRecord]) -> String {
        let mut word_freq: HashMap<String, usize> = HashMap::new();
        for r in records {
            for word in r.context.split_whitespace() {
                let w = word
                    .to_lowercase()
                    .chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect::<String>();
                if w.len() >= 3 {
                    *word_freq.entry(w).or_insert(0) += 1;
                }
            }
        }

        let mut freq: Vec<(String, usize)> = word_freq.into_iter().collect();
        freq.sort_by(|a, b| b.1.cmp(&a.1));
        freq.iter()
            .take(5)
            .map(|(w, _)| w.clone())
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn distill_heuristics(&mut self, trajectory: &[ExperienceRecord]) -> Vec<Heuristic> {
        let new_heuristics = self.extract_heuristics(trajectory);
        for h in new_heuristics {
            if self.heuristics.len() >= self.max_heuristics {
                self.prune_lowest_confidence();
            }
            if self.heuristics.len() < self.max_heuristics {
                if !self
                    .heuristics
                    .iter()
                    .any(|existing| existing.pattern == h.pattern)
                {
                    self.heuristics.push(h);
                }
            }
        }
        self.heuristics.clone()
    }

    fn prune_lowest_confidence(&mut self) {
        if let Some(idx) = self
            .heuristics
            .iter()
            .enumerate()
            .min_by(|a, b| {
                a.1.confidence
                    .partial_cmp(&b.1.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(i, _)| i)
        {
            self.heuristics.swap_remove(idx);
        }
    }

    pub fn best_heuristics(&self, top_k: usize) -> Vec<&Heuristic> {
        let mut sorted: Vec<&Heuristic> = self.heuristics.iter().collect();
        sorted.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted.into_iter().take(top_k).collect()
    }

    pub fn matching_heuristics(&self, context: &str) -> Vec<&Heuristic> {
        self.heuristics
            .iter()
            .filter(|h| h.apply(context).is_some())
            .collect()
    }

    pub fn heuristic_count(&self) -> usize {
        self.heuristics.len()
    }
}

impl Default for TrajectoryHeuristicExtractor {
    fn default() -> Self {
        Self::new(200)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_record(
        id: u64,
        context: &str,
        action: &str,
        reward: f64,
        success: bool,
    ) -> ExperienceRecord {
        ExperienceRecord {
            id,
            context: context.to_string(),
            action: action.to_string(),
            reward,
            success,
            timestamp: 0,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_extract_empty_trajectory() {
        let extractor = TrajectoryHeuristicExtractor::new(100);
        let heuristics = extractor.extract_heuristics(&[]);
        assert!(heuristics.is_empty());
    }

    #[test]
    fn test_extract_success_heuristics() {
        let trajectory = vec![
            make_record(
                1,
                "user asks about weather in Tokyo",
                "fetch_weather_api",
                0.9,
                true,
            ),
            make_record(
                2,
                "user asks about weather in London",
                "fetch_weather_api",
                0.85,
                true,
            ),
            make_record(
                3,
                "user asks about weather in Paris",
                "fetch_weather_api",
                0.95,
                true,
            ),
        ];
        let extractor = TrajectoryHeuristicExtractor::new(100);
        let heuristics = extractor.extract_heuristics(&trajectory);
        assert!(!heuristics.is_empty());
        assert!(heuristics.iter().all(|h| h.is_positive));
        assert!(heuristics[0].confidence > 0.0);
    }

    #[test]
    fn test_extract_failure_heuristics() {
        let trajectory = vec![
            make_record(1, "complex math problem", "direct_answer", 0.1, false),
            make_record(2, "complex physics problem", "direct_answer", 0.2, false),
        ];
        let extractor = TrajectoryHeuristicExtractor::new(100);
        let heuristics = extractor.extract_heuristics(&trajectory);
        assert!(!heuristics.is_empty());
        assert!(heuristics.iter().all(|h| !h.is_positive));
    }

    #[test]
    fn test_heuristic_apply_match() {
        let h = Heuristic {
            pattern: "weather forecast".to_string(),
            principle: "Use weather API for climate queries".to_string(),
            confidence: 0.8,
            source_count: 5,
            is_positive: true,
        };
        let suggestion = h.apply("what is the weather forecast for today");
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("weather API"));
    }

    #[test]
    fn test_heuristic_apply_no_match() {
        let h = Heuristic {
            pattern: "weather forecast".to_string(),
            principle: "Use weather API".to_string(),
            confidence: 0.8,
            source_count: 5,
            is_positive: true,
        };
        let suggestion = h.apply("calculate 2 + 2");
        assert!(suggestion.is_none());
    }

    #[test]
    fn test_distill_heuristics_accumulates() {
        let mut extractor = TrajectoryHeuristicExtractor::new(100);
        let t1 = vec![make_record(1, "weather Tokyo", "fetch", 0.9, true)];
        let r1 = extractor.distill_heuristics(&t1);
        assert!(r1.is_empty());
        let t2 = vec![
            make_record(2, "weather London", "fetch", 0.8, true),
            make_record(3, "weather Paris", "fetch", 0.9, true),
        ];
        let r2 = extractor.distill_heuristics(&t2);
        assert!(
            !r2.is_empty(),
            "should produce heuristic after 3 records with 2+ in same domain"
        );
        assert_eq!(extractor.heuristic_count(), 1);
    }

    #[test]
    fn test_best_heuristics_orders_by_confidence() {
        let mut extractor = TrajectoryHeuristicExtractor::new(100);
        extractor.heuristics.push(Heuristic {
            pattern: "low".to_string(),
            principle: "low confidence".to_string(),
            confidence: 0.3,
            source_count: 2,
            is_positive: true,
        });
        extractor.heuristics.push(Heuristic {
            pattern: "high".to_string(),
            principle: "high confidence".to_string(),
            confidence: 0.9,
            source_count: 10,
            is_positive: true,
        });
        let best = extractor.best_heuristics(1);
        assert_eq!(best.len(), 1);
        assert!((best[0].confidence - 0.9).abs() < 1e-9);
    }

    #[test]
    fn test_matching_heuristics_filters_by_context() {
        let mut extractor = TrajectoryHeuristicExtractor::new(100);
        extractor.heuristics.push(Heuristic {
            pattern: "code review".to_string(),
            principle: "run linter first".to_string(),
            confidence: 0.8,
            source_count: 5,
            is_positive: true,
        });
        let matches = extractor.matching_heuristics("please review this code");
        assert_eq!(matches.len(), 1);
        let no_matches = extractor.matching_heuristics("what is the weather");
        assert!(no_matches.is_empty());
    }

    #[test]
    fn test_prune_removes_lowest_confidence() {
        let mut extractor = TrajectoryHeuristicExtractor::new(2);
        extractor.heuristics.push(Heuristic {
            pattern: "a".to_string(),
            principle: "a".to_string(),
            confidence: 0.9,
            source_count: 10,
            is_positive: true,
        });
        extractor.heuristics.push(Heuristic {
            pattern: "b".to_string(),
            principle: "b".to_string(),
            confidence: 0.3,
            source_count: 1,
            is_positive: true,
        });
        extractor.prune_lowest_confidence();
        assert_eq!(extractor.heuristics.len(), 1);
        assert!((extractor.heuristics[0].confidence - 0.9).abs() < 1e-9);
    }

    #[test]
    fn test_max_heuristics_enforced() {
        let mut extractor = TrajectoryHeuristicExtractor::new(3);
        extractor.heuristics.push(Heuristic {
            pattern: "x".to_string(),
            principle: "x".to_string(),
            confidence: 0.5,
            source_count: 2,
            is_positive: true,
        });
        extractor.heuristics.push(Heuristic {
            pattern: "y".to_string(),
            principle: "y".to_string(),
            confidence: 0.6,
            source_count: 3,
            is_positive: true,
        });
        extractor.heuristics.push(Heuristic {
            pattern: "z".to_string(),
            principle: "z".to_string(),
            confidence: 0.7,
            source_count: 4,
            is_positive: true,
        });
        let extra = Heuristic {
            pattern: "new".to_string(),
            principle: "new".to_string(),
            confidence: 0.8,
            source_count: 5,
            is_positive: true,
        };
        let t = vec![
            make_record(1, "new domain alpha", "action_a", 0.9, true),
            make_record(2, "new domain beta", "action_a", 0.8, true),
        ];
        extractor.distill_heuristics(&t);
        assert_eq!(extractor.heuristic_count(), 3);
    }

    #[test]
    fn test_heuristic_confidence_from_reward_and_count() {
        let trajectory = vec![
            make_record(1, "data analysis python", "run_pandas", 0.95, true),
            make_record(2, "data analysis rust", "run_pandas", 0.90, true),
        ];
        let extractor = TrajectoryHeuristicExtractor::new(100);
        let heuristics = extractor.extract_heuristics(&trajectory);
        assert!(!heuristics.is_empty());
        let h = &heuristics[0];
        assert!(h.confidence > 0.5);
        assert!(!h.pattern.is_empty());
        assert!(!h.principle.is_empty());
    }
}
